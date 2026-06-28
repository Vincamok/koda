use anyhow::Context;
use bollard::{
    container::{
        Config, CreateContainerOptions, LogsOptions, RemoveContainerOptions,
        StartContainerOptions, WaitContainerOptions,
    },
    models::{HostConfig, Resources},
    Docker,
};
use futures::StreamExt;
use redis::{aio::MultiplexedConnection, AsyncCommands};
use serde::Deserialize;
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

const STREAM: &str = "koda:jobs:pipeline";
const DEAD_LETTER: &str = "koda:jobs:pipeline:dead";
const MAX_RETRIES: u8 = 3;
const SAST_MAX_FILE_BYTES: u64 = 64 * 1024; // 64 KB per file
const SAST_MAX_FILES: usize = 20;

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum PipelineJob {
    RunPipeline {
        pipeline_id: Uuid,
        workspace_id: Uuid,
        org_id: Uuid,
        trigger: String,
    },
}

pub struct PipelineRunner {
    pub pool: PgPool,
    pub redis: MultiplexedConnection,
    pub group: String,
    pub consumer: String,
    pub http: reqwest::Client,
    pub docker_host: String,
    pub anthropic_api_key: Option<String>,
}

impl PipelineRunner {
    pub async fn init(&mut self) -> anyhow::Result<()> {
        let _: redis::RedisResult<()> = self
            .redis
            .xgroup_create_mkstream(STREAM, &self.group, "$")
            .await;
        Ok(())
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        self.init().await?;
        tracing::info!(group = %self.group, "pipeline runner started");

        let mut failure_counts: HashMap<String, u8> = HashMap::new();

        loop {
            let entries: redis::streams::StreamReadReply = self
                .redis
                .xread_options(
                    &[STREAM],
                    &[">"],
                    &redis::streams::StreamReadOptions::default()
                        .group(&self.group, &self.consumer)
                        .count(5)
                        .block(5000),
                )
                .await
                .unwrap_or_else(|_| redis::streams::StreamReadReply { keys: vec![] });

            for stream_key in entries.keys {
                for message in stream_key.ids {
                    let id = message.id.clone();
                    match self.process(&message).await {
                        Ok(_) => {
                            let _: Result<(), _> = self.redis.xack(STREAM, &self.group, &[&id]).await;
                            failure_counts.remove(&id);
                        }
                        Err(e) => {
                            let attempts = {
                                let c = failure_counts.entry(id.clone()).or_insert(0);
                                *c += 1;
                                *c
                            };
                            if attempts >= MAX_RETRIES {
                                tracing::error!(msg_id = %id, attempts, error = %e, "pipeline dead-lettered");
                                let _: Result<(), _> = self.redis.xack(STREAM, &self.group, &[&id]).await;
                                let _: Result<String, _> = self
                                    .redis
                                    .xadd(DEAD_LETTER, "*", &[
                                        ("original_id", id.as_str()),
                                        ("error", e.to_string().as_str()),
                                        ("attempts", attempts.to_string().as_str()),
                                    ])
                                    .await;
                                failure_counts.remove(&id);
                            } else {
                                tracing::warn!(msg_id = %id, attempts, max = MAX_RETRIES, error = %e, "pipeline job failed — will retry");
                            }
                        }
                    }
                }
            }
        }
    }

    async fn process(&mut self, msg: &redis::streams::StreamId) -> anyhow::Result<()> {
        let payload: String = msg
            .map
            .get("payload")
            .and_then(|v| match v {
                redis::Value::Data(b) => Some(String::from_utf8_lossy(b).into_owned()),
                _ => None,
            })
            .context("missing payload")?;

        let job: PipelineJob = serde_json::from_str(&payload).context("deserialize pipeline job")?;

        match job {
            PipelineJob::RunPipeline { pipeline_id, workspace_id, org_id, trigger } => {
                self.run_pipeline(pipeline_id, workspace_id, org_id, trigger).await?;
            }
        }

        Ok(())
    }

    async fn run_pipeline(
        &self,
        pipeline_id: Uuid,
        workspace_id: Uuid,
        org_id: Uuid,
        trigger: String,
    ) -> anyhow::Result<()> {
        let pipeline = sqlx::query!(
            "SELECT id, pipeline_type, config FROM cicd_pipelines WHERE id = $1",
            pipeline_id,
        )
        .fetch_one(&self.pool)
        .await?;

        let job = sqlx::query!(
            r#"INSERT INTO jobs (job_type, payload, status, attempts)
               VALUES ('pipeline', $1, 'running', 1)
               RETURNING id"#,
            serde_json::json!({
                "pipeline_id": pipeline_id,
                "workspace_id": workspace_id,
                "org_id": org_id,
                "trigger": trigger,
            }),
        )
        .fetch_one(&self.pool)
        .await?;

        // Create ephemeral pipeline branch if workspace repo is available
        let branch_name = format!(
            "pipeline/{}/{}",
            &workspace_id.to_string()[..8],
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0)
        );
        if let Err(e) = self.create_ephemeral_branch(workspace_id, &branch_name).await {
            tracing::debug!(error = %e, "ephemeral branch skipped (no repo yet)");
        }

        tracing::info!(
            job_id = %job.id,
            pipeline_id = %pipeline_id,
            pipeline_type = %pipeline.pipeline_type,
            "executing pipeline"
        );

        let result = self.execute_pipeline_type(
            &pipeline.pipeline_type,
            pipeline_id,
            workspace_id,
            org_id,
            &pipeline.config,
        ).await;

        match result {
            Ok(report_id) => {
                sqlx::query!(
                    "UPDATE jobs SET status = 'success', result = $1, updated_at = NOW() WHERE id = $2",
                    serde_json::json!({"report_id": report_id}),
                    job.id,
                )
                .execute(&self.pool)
                .await?;

                sqlx::query!(
                    "UPDATE cicd_pipelines SET status = 'success', updated_at = NOW() WHERE id = $1",
                    pipeline_id,
                )
                .execute(&self.pool)
                .await?;

                // Enforce SecurityPolicy — block workspace if critical findings exist
                if let Some(rid) = report_id {
                    if let Err(e) = self.enforce_security_policy(workspace_id, org_id, rid).await {
                        tracing::warn!(error = %e, "security policy enforcement error");
                    }
                }
            }
            Err(e) => {
                sqlx::query!(
                    "UPDATE jobs SET status = 'failed', error = $1, updated_at = NOW() WHERE id = $2",
                    e.to_string(),
                    job.id,
                )
                .execute(&self.pool)
                .await?;

                sqlx::query!(
                    "UPDATE cicd_pipelines SET status = 'failed', updated_at = NOW() WHERE id = $1",
                    pipeline_id,
                )
                .execute(&self.pool)
                .await?;

                return Err(e);
            }
        }

        Ok(())
    }

    async fn execute_pipeline_type(
        &self,
        pipeline_type: &str,
        pipeline_id: Uuid,
        workspace_id: Uuid,
        org_id: Uuid,
        config: &serde_json::Value,
    ) -> anyhow::Result<Option<Uuid>> {
        match pipeline_type {
            "secret_scan" => {
                let report_id = self.run_secret_scan(pipeline_id, workspace_id, org_id, config).await?;
                Ok(Some(report_id))
            }
            "dependency_scan" => {
                let report_id = self.run_dependency_scan(pipeline_id, workspace_id, org_id, config).await?;
                Ok(Some(report_id))
            }
            "sast" => {
                let report_id = self.run_sast(pipeline_id, workspace_id, org_id, config).await?;
                Ok(Some(report_id))
            }
            "build" | "lint" | "image_scan" => {
                self.run_container_pipeline(pipeline_type, pipeline_id, workspace_id, config).await?;
                Ok(None)
            }
            _ => Err(anyhow::anyhow!("unknown pipeline type: {}", pipeline_type)),
        }
    }

    // ── Ephemeral pipeline branch ──────────────────────────────────────────────

    async fn create_ephemeral_branch(
        &self,
        workspace_id: Uuid,
        branch_name: &str,
    ) -> anyhow::Result<()> {
        let volume_name: Option<String> = sqlx::query_scalar::<_, String>(
            "SELECT volume_name FROM workspace_volumes WHERE workspace_id = $1 LIMIT 1",
        )
        .bind(workspace_id)
        .fetch_optional(&self.pool)
        .await?;

        let Some(volume_name) = volume_name else {
            anyhow::bail!("no volume for workspace {workspace_id}");
        };

        let repo_path = format!("/var/lib/docker/volumes/{}", volume_name);
        let repo = git2::Repository::open(&repo_path)
            .with_context(|| format!("open git repo at {repo_path}"))?;

        let head = repo.head().context("get HEAD")?;
        let commit = head.peel_to_commit().context("peel HEAD to commit")?;
        repo.branch(branch_name, &commit, false)
            .with_context(|| format!("create branch {branch_name}"))?;

        tracing::info!(branch = %branch_name, workspace_id = %workspace_id, "ephemeral pipeline branch created");
        Ok(())
    }

    // ── Security Policy Enforcement ────────────────────────────────────────────

    async fn enforce_security_policy(
        &self,
        workspace_id: Uuid,
        org_id: Uuid,
        report_id: Uuid,
    ) -> anyhow::Result<()> {
        let policy = sqlx::query!(
            "SELECT min_severity_to_block FROM security_policies WHERE organization_id = $1",
            org_id,
        )
        .fetch_optional(&self.pool)
        .await?;

        let min_severity = policy
            .map(|p| p.min_severity_to_block)
            .unwrap_or_else(|| "critical".to_string());

        if min_severity == "none" {
            return Ok(());
        }

        let severity_rank = |s: &str| match s {
            "critical" => 4,
            "high" => 3,
            "medium" => 2,
            "low" => 1,
            _ => 0,
        };
        let threshold = severity_rank(&min_severity);

        let findings = sqlx::query!(
            "SELECT severity FROM vulnerability_findings WHERE security_report_id = $1",
            report_id,
        )
        .fetch_all(&self.pool)
        .await?;

        let blocking = findings
            .iter()
            .any(|f| severity_rank(&f.severity) >= threshold);

        if blocking {
            sqlx::query!(
                "UPDATE workspaces SET status = 'reviewing', updated_at = NOW() WHERE id = $1 AND status NOT IN ('closed', 'stopped', 'stopping')",
                workspace_id,
            )
            .execute(&self.pool)
            .await?;

            tracing::warn!(
                workspace_id = %workspace_id,
                report_id = %report_id,
                min_severity = %min_severity,
                "workspace blocked by security policy"
            );
        }

        Ok(())
    }

    // ── Secret Scan ────────────────────────────────────────────────────────────

    async fn run_secret_scan(
        &self,
        pipeline_id: Uuid,
        workspace_id: Uuid,
        org_id: Uuid,
        _config: &serde_json::Value,
    ) -> anyhow::Result<Uuid> {
        let report = sqlx::query!(
            r#"INSERT INTO security_reports (workspace_id, organization_id, pipeline_id, scan_type, status)
               VALUES ($1, $2, $3, 'secret_scan', 'running')
               RETURNING id"#,
            workspace_id,
            org_id,
            pipeline_id,
        )
        .fetch_one(&self.pool)
        .await?;

        let rules = sqlx::query!(
            r#"SELECT id, name, rule_type, pattern, entropy_threshold, severity
               FROM scan_rules
               WHERE is_active = true AND (is_builtin = true OR organization_id = $1)"#,
            org_id,
        )
        .fetch_all(&self.pool)
        .await?;

        let volume_name: Option<String> = sqlx::query_scalar::<_, String>(
            "SELECT volume_name FROM workspace_volumes WHERE workspace_id = $1 LIMIT 1",
        )
        .bind(workspace_id)
        .fetch_optional(&self.pool)
        .await?;

        let scan_path = volume_name
            .map(|n| format!("/var/lib/docker/volumes/{}", n))
            .unwrap_or_else(|| "/tmp/workspace_scan".to_string());

        let mut findings: Vec<(String, String, String, Option<String>, Option<String>, Option<i32>, Option<String>)> = Vec::new();

        for rule in &rules {
            if rule.rule_type == "regex" {
                if let Some(pattern) = &rule.pattern {
                    if let Ok(re) = regex::Regex::new(pattern) {
                        if let Ok(entries) = scan_files_in_path(&scan_path, &re) {
                            for (file, line, evidence) in entries {
                                findings.push((
                                    rule.name.clone(),
                                    format!("Secret pattern '{}' found", rule.name),
                                    rule.severity.clone(),
                                    Some(rule.id.to_string()),
                                    Some(file),
                                    Some(line as i32),
                                    Some(evidence),
                                ));
                            }
                        }
                    }
                }
            } else if rule.rule_type == "entropy" {
                let threshold = rule.entropy_threshold.unwrap_or(4.5);
                if let Ok(entries) = scan_high_entropy_strings(&scan_path, threshold) {
                    for (file, line, evidence) in entries {
                        findings.push((
                            rule.name.clone(),
                            "High-entropy string (possible secret) found".to_string(),
                            rule.severity.clone(),
                            Some(rule.id.to_string()),
                            Some(file),
                            Some(line as i32),
                            Some(evidence),
                        ));
                    }
                }
            }
        }

        for (title, description, severity, rule_id, file_path, line_number, evidence) in &findings {
            sqlx::query!(
                r#"INSERT INTO vulnerability_findings
                       (security_report_id, title, description, severity, rule_id, file_path, line_number, evidence)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
                report.id,
                title,
                description,
                severity,
                rule_id.as_deref(),
                file_path.as_deref(),
                *line_number,
                evidence.as_deref(),
            )
            .execute(&self.pool)
            .await?;
        }

        let summary = format!("Secret scan completed. {} finding(s) found.", findings.len());
        sqlx::query!(
            "UPDATE security_reports SET status = 'completed', summary = $1, updated_at = NOW() WHERE id = $2",
            summary,
            report.id,
        )
        .execute(&self.pool)
        .await?;

        tracing::info!(report_id = %report.id, findings = findings.len(), "secret scan completed");
        Ok(report.id)
    }

    // ── Dependency Scan ────────────────────────────────────────────────────────

    async fn run_dependency_scan(
        &self,
        pipeline_id: Uuid,
        workspace_id: Uuid,
        org_id: Uuid,
        _config: &serde_json::Value,
    ) -> anyhow::Result<Uuid> {
        let report = sqlx::query!(
            r#"INSERT INTO security_reports (workspace_id, organization_id, pipeline_id, scan_type, status)
               VALUES ($1, $2, $3, 'dependency_scan', 'running')
               RETURNING id"#,
            workspace_id,
            org_id,
            pipeline_id,
        )
        .fetch_one(&self.pool)
        .await?;

        let volume_name: Option<String> = sqlx::query_scalar::<_, String>(
            "SELECT volume_name FROM workspace_volumes WHERE workspace_id = $1 LIMIT 1",
        )
        .bind(workspace_id)
        .fetch_optional(&self.pool)
        .await?;

        let scan_path = volume_name
            .map(|n| format!("/var/lib/docker/volumes/{}", n))
            .unwrap_or_else(|| "/tmp/workspace_scan".to_string());

        let mut findings = Vec::new();

        let cargo_lock = std::path::Path::new(&scan_path).join("Cargo.lock");
        if cargo_lock.exists() {
            match run_cargo_audit(&scan_path) {
                Ok(vulns) => findings.extend(vulns),
                Err(e) => tracing::warn!(error = %e, "cargo audit failed"),
            }
        }

        let package_lock = std::path::Path::new(&scan_path).join("package-lock.json");
        if package_lock.exists() {
            match run_npm_audit(&scan_path) {
                Ok(vulns) => findings.extend(vulns),
                Err(e) => tracing::warn!(error = %e, "npm audit failed"),
            }
        }

        for (title, description, severity, file_path) in &findings {
            sqlx::query!(
                r#"INSERT INTO vulnerability_findings
                       (security_report_id, title, description, severity, file_path)
                   VALUES ($1, $2, $3, $4, $5)"#,
                report.id,
                title,
                description,
                severity,
                file_path.as_deref(),
            )
            .execute(&self.pool)
            .await?;
        }

        let summary = format!("Dependency scan completed. {} vulnerability finding(s).", findings.len());
        sqlx::query!(
            "UPDATE security_reports SET status = 'completed', summary = $1, updated_at = NOW() WHERE id = $2",
            summary,
            report.id,
        )
        .execute(&self.pool)
        .await?;

        Ok(report.id)
    }

    // ── SAST — LLM-based OWASP Top 10 analysis ────────────────────────────────

    async fn run_sast(
        &self,
        pipeline_id: Uuid,
        workspace_id: Uuid,
        org_id: Uuid,
        _config: &serde_json::Value,
    ) -> anyhow::Result<Uuid> {
        let report = sqlx::query!(
            r#"INSERT INTO security_reports (workspace_id, organization_id, pipeline_id, scan_type, status)
               VALUES ($1, $2, $3, 'sast', 'running')
               RETURNING id"#,
            workspace_id,
            org_id,
            pipeline_id,
        )
        .fetch_one(&self.pool)
        .await?;

        let api_key = match &self.anthropic_api_key {
            Some(k) if !k.is_empty() => k.clone(),
            _ => {
                let summary = "SAST skipped — ANTHROPIC_API_KEY not configured.".to_string();
                sqlx::query!(
                    "UPDATE security_reports SET status = 'completed', summary = $1, updated_at = NOW() WHERE id = $2",
                    summary, report.id,
                )
                .execute(&self.pool)
                .await?;
                return Ok(report.id);
            }
        };

        let volume_name: Option<String> = sqlx::query_scalar::<_, String>(
            "SELECT volume_name FROM workspace_volumes WHERE workspace_id = $1 LIMIT 1",
        )
        .bind(workspace_id)
        .fetch_optional(&self.pool)
        .await?;

        let scan_path = volume_name
            .map(|n| format!("/var/lib/docker/volumes/{}", n))
            .unwrap_or_else(|| "/tmp/workspace_scan".to_string());

        // Collect source files for analysis (skip binary, skip huge files)
        let code_snippets = collect_source_files_for_sast(&scan_path);

        if code_snippets.is_empty() {
            let summary = "SAST completed — no source files found to analyze.".to_string();
            sqlx::query!(
                "UPDATE security_reports SET status = 'completed', summary = $1, updated_at = NOW() WHERE id = $2",
                summary, report.id,
            )
            .execute(&self.pool)
            .await?;
            return Ok(report.id);
        }

        let code_context = code_snippets
            .iter()
            .map(|(path, content)| format!("=== {} ===\n{}", path, content))
            .collect::<Vec<_>>()
            .join("\n\n");

        let prompt = format!(
            r#"You are a security expert performing a static analysis security test (SAST) focused on OWASP Top 10 vulnerabilities.

Analyze the following source code and identify security vulnerabilities. For each finding, provide:
- title: short name of the vulnerability
- severity: one of "critical", "high", "medium", "low", "info"
- description: detailed explanation of the issue
- file: the file path where the issue was found
- line: line number (approximate is fine, use 0 if unknown)
- remediation: how to fix the issue

Focus on:
- A01: Broken Access Control
- A02: Cryptographic Failures
- A03: Injection (SQL, XSS, command injection)
- A04: Insecure Design
- A05: Security Misconfiguration
- A06: Vulnerable and Outdated Components
- A07: Identification and Authentication Failures
- A08: Software and Data Integrity Failures
- A09: Security Logging and Monitoring Failures
- A10: Server-Side Request Forgery (SSRF)

Respond ONLY with valid JSON in this exact format:
{{"findings": [{{"title": "...", "severity": "...", "description": "...", "file": "...", "line": 0, "remediation": "..."}}]}}

Source code to analyze:
{}
"#,
            code_context
        );

        let response = self
            .http
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&serde_json::json!({
                "model": "claude-haiku-4-5-20251001",
                "max_tokens": 4096,
                "messages": [{"role": "user", "content": prompt}]
            }))
            .send()
            .await
            .context("anthropic API request")?;

        let status = response.status();
        let body: serde_json::Value = response.json().await.context("parse anthropic response")?;

        if !status.is_success() {
            anyhow::bail!("anthropic API error {}: {:?}", status, body);
        }

        let llm_text = body
            .pointer("/content/0/text")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let findings_json: serde_json::Value = serde_json::from_str(llm_text)
            .unwrap_or_else(|_| {
                // Try to extract JSON from the text if it contains extra content
                extract_json_from_text(llm_text)
                    .unwrap_or(serde_json::json!({"findings": []}))
            });

        let findings = findings_json
            .get("findings")
            .and_then(|f| f.as_array())
            .cloned()
            .unwrap_or_default();

        let mut saved = 0usize;
        for f in &findings {
            let title = f.get("title").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string();
            let severity = f.get("severity").and_then(|v| v.as_str()).unwrap_or("info").to_string();
            let description = f.get("description").and_then(|v| v.as_str()).map(str::to_string);
            let file_path = f.get("file").and_then(|v| v.as_str()).map(str::to_string);
            let line_number = f.get("line").and_then(|v| v.as_i64()).map(|n| n as i32);
            let remediation = f.get("remediation").and_then(|v| v.as_str()).map(str::to_string);

            let valid_severities = ["critical", "high", "medium", "low", "info"];
            let severity = if valid_severities.contains(&severity.as_str()) { severity } else { "info".to_string() };

            sqlx::query!(
                r#"INSERT INTO vulnerability_findings
                       (security_report_id, title, description, severity, file_path, line_number, remediation)
                   VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
                report.id,
                title,
                description,
                severity,
                file_path.as_deref(),
                line_number,
                remediation.as_deref(),
            )
            .execute(&self.pool)
            .await?;
            saved += 1;
        }

        let summary = format!(
            "SAST completed. {} finding(s) identified via LLM-based OWASP Top 10 analysis.",
            saved
        );
        sqlx::query!(
            "UPDATE security_reports SET status = 'completed', summary = $1, updated_at = NOW() WHERE id = $2",
            summary, report.id,
        )
        .execute(&self.pool)
        .await?;

        tracing::info!(report_id = %report.id, findings = saved, "SAST completed");
        Ok(report.id)
    }

    // ── Container-based pipelines (build, lint, image_scan) ───────────────────

    async fn run_container_pipeline(
        &self,
        pipeline_type: &str,
        pipeline_id: Uuid,
        workspace_id: Uuid,
        config: &serde_json::Value,
    ) -> anyhow::Result<()> {
        let image = config
            .get("image")
            .and_then(|v| v.as_str())
            .unwrap_or(default_image_for(pipeline_type))
            .to_string();

        let command = config
            .get("command")
            .and_then(|v| v.as_str())
            .unwrap_or(default_command_for(pipeline_type))
            .to_string();

        let container_name = format!(
            "koda-pipeline-{}-{}",
            pipeline_type,
            &pipeline_id.to_string()[..8]
        );

        let docker = Docker::connect_with_http(
            &self.docker_host,
            120,
            bollard::API_DEFAULT_VERSION,
        )
        .context("connect to docker-socket-proxy")?;

        let mut labels = HashMap::new();
        labels.insert("koda.managed", "true");
        labels.insert("koda.type", "pipeline");
        let workspace_id_str = workspace_id.to_string();
        let pipeline_id_str = pipeline_id.to_string();
        labels.insert("koda.workspace_id", &workspace_id_str);
        labels.insert("koda.pipeline_id", &pipeline_id_str);

        // Get workspace volume for mounting source code
        let volume_name: Option<String> = sqlx::query_scalar::<_, String>(
            "SELECT volume_name FROM workspace_volumes WHERE workspace_id = $1 LIMIT 1",
        )
        .bind(workspace_id)
        .fetch_optional(&self.pool)
        .await?;

        let binds = volume_name.map(|vn| vec![format!("{}:/workspace:ro", vn)]);

        docker
            .create_container(
                Some(CreateContainerOptions {
                    name: &container_name,
                    platform: None,
                }),
                Config {
                    image: Some(image.as_str()),
                    cmd: Some(vec!["/bin/sh", "-c", &command]),
                    working_dir: Some("/workspace"),
                    labels: Some(labels),
                    host_config: Some(HostConfig {
                        binds,
                        resources: Some(Resources {
                            nano_cpus: Some(1_000_000_000),
                            memory: Some(512 * 1024 * 1024),
                            pids_limit: Some(256),
                            cpu_period: Some(100_000),
                            cpu_quota: Some(100_000),
                            ..Default::default()
                        }),
                        auto_remove: Some(false),
                        network_mode: Some("none".to_string()),
                        cap_drop: Some(vec!["ALL".to_string()]),
                        security_opt: Some(vec!["no-new-privileges:true".to_string()]),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            )
            .await
            .context("create pipeline container")?;

        docker
            .start_container(&container_name, None::<StartContainerOptions<String>>)
            .await
            .context("start pipeline container")?;

        // Wait for container to finish (timeout 10 min)
        let mut wait_stream = docker.wait_container(
            &container_name,
            None::<WaitContainerOptions<String>>,
        );

        let exit_status = tokio::time::timeout(
            std::time::Duration::from_secs(600),
            wait_stream.next(),
        )
        .await
        .context("pipeline container timed out after 600s")?
        .context("wait stream ended unexpectedly")?
        .context("container wait error")?;

        // Collect logs
        let log_opts = LogsOptions::<String> {
            stdout: true,
            stderr: true,
            tail: "100".to_string(),
            ..Default::default()
        };
        let mut logs_stream = docker.logs(&container_name, Some(log_opts));
        let mut log_lines = Vec::new();
        while let Some(Ok(msg)) = logs_stream.next().await {
            log_lines.push(msg.to_string());
            if log_lines.len() >= 100 {
                break;
            }
        }

        // Remove container
        docker
            .remove_container(
                &container_name,
                Some(RemoveContainerOptions { force: true, ..Default::default() }),
            )
            .await
            .ok();

        if exit_status.status_code != 0 {
            let logs = log_lines.join("");
            anyhow::bail!(
                "pipeline container '{}' exited with code {}. Logs:\n{}",
                container_name,
                exit_status.status_code,
                &logs[..logs.len().min(2000)]
            );
        }

        tracing::info!(
            container = %container_name,
            pipeline_type = %pipeline_type,
            exit_code = exit_status.status_code,
            "container pipeline completed"
        );

        Ok(())
    }
}

// ── SAST helpers ──────────────────────────────────────────────────────────────

fn collect_source_files_for_sast(path: &str) -> Vec<(String, String)> {
    let source_exts = [
        "rs", "py", "ts", "tsx", "js", "jsx", "go", "java", "cs", "php",
        "rb", "kt", "swift", "cpp", "c", "h", "sql",
    ];

    let mut files = Vec::new();
    let walker = walkdir::WalkDir::new(path)
        .follow_links(false)
        .max_depth(10);

    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        if files.len() >= SAST_MAX_FILES {
            break;
        }
        if !entry.file_type().is_file() {
            continue;
        }
        let path_str = entry.path().to_string_lossy().to_string();
        if should_skip_path(&path_str) {
            continue;
        }
        let ext = entry.path().extension().and_then(|e| e.to_str()).unwrap_or("");
        if !source_exts.contains(&ext) {
            continue;
        }
        let metadata = entry.metadata().ok();
        if metadata.map(|m| m.len()).unwrap_or(0) > SAST_MAX_FILE_BYTES {
            continue;
        }
        if let Ok(content) = std::fs::read_to_string(entry.path()) {
            files.push((path_str, content));
        }
    }

    files
}

fn extract_json_from_text(text: &str) -> Option<serde_json::Value> {
    let start = text.find('{')?;
    let end = text.rfind('}')?;
    if end >= start {
        serde_json::from_str(&text[start..=end]).ok()
    } else {
        None
    }
}

// ── Scan helpers ───────────────────────────────────────────────────────────────

fn scan_files_in_path(
    path: &str,
    re: &regex::Regex,
) -> anyhow::Result<Vec<(String, usize, String)>> {
    let mut results = Vec::new();
    let walker = walkdir::WalkDir::new(path)
        .follow_links(false)
        .max_depth(20);

    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path_str = entry.path().to_string_lossy().to_string();
        if should_skip_path(&path_str) {
            continue;
        }
        if let Ok(content) = std::fs::read_to_string(entry.path()) {
            for (lineno, line) in content.lines().enumerate() {
                if re.is_match(line) {
                    results.push((path_str.clone(), lineno + 1, line.to_string()));
                }
            }
        }
    }

    Ok(results)
}

fn scan_high_entropy_strings(
    path: &str,
    threshold: f64,
) -> anyhow::Result<Vec<(String, usize, String)>> {
    let re = regex::Regex::new(r#"(?i)(password|secret|token|key|api_key|apikey)\s*[=:]\s*['"]?([A-Za-z0-9+/=_\-\.]{16,})['"]?"#)?;
    let mut results = Vec::new();
    let walker = walkdir::WalkDir::new(path)
        .follow_links(false)
        .max_depth(20);

    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path_str = entry.path().to_string_lossy().to_string();
        if should_skip_path(&path_str) {
            continue;
        }
        if let Ok(content) = std::fs::read_to_string(entry.path()) {
            for (lineno, line) in content.lines().enumerate() {
                if let Some(cap) = re.captures(line) {
                    if let Some(m) = cap.get(2) {
                        let entropy = shannon_entropy(m.as_str());
                        if entropy >= threshold {
                            results.push((path_str.clone(), lineno + 1, line.to_string()));
                        }
                    }
                }
            }
        }
    }

    Ok(results)
}

fn shannon_entropy(s: &str) -> f64 {
    if s.is_empty() {
        return 0.0;
    }
    let mut freq = [0u64; 256];
    for b in s.bytes() {
        freq[b as usize] += 1;
    }
    let len = s.len() as f64;
    freq.iter()
        .filter(|&&c| c > 0)
        .map(|&c| {
            let p = c as f64 / len;
            -p * p.log2()
        })
        .sum()
}

fn should_skip_path(path: &str) -> bool {
    let skip_dirs = [
        "/.git/", "/node_modules/", "/target/", "/.cargo/",
        "/dist/", "/build/", "/.next/",
    ];
    skip_dirs.iter().any(|d| path.contains(d))
        || path.ends_with(".lock")
        || path.ends_with(".png")
        || path.ends_with(".jpg")
        || path.ends_with(".wasm")
}

fn run_cargo_audit(path: &str) -> anyhow::Result<Vec<(String, String, String, Option<String>)>> {
    let output = std::process::Command::new("cargo")
        .args(["audit", "--json"])
        .current_dir(path)
        .output();

    let Ok(output) = output else {
        return Ok(vec![]);
    };

    if output.stdout.is_empty() {
        return Ok(vec![]);
    }

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap_or_default();
    let mut findings = Vec::new();

    if let Some(vulns) = json.get("vulnerabilities").and_then(|v| v.get("list")).and_then(|v| v.as_array()) {
        for v in vulns {
            let title = v
                .pointer("/advisory/title")
                .and_then(|s| s.as_str())
                .unwrap_or("Unknown vulnerability")
                .to_string();
            let description = v
                .pointer("/advisory/description")
                .and_then(|s| s.as_str())
                .unwrap_or("")
                .to_string();
            let severity = map_cvss_to_severity(
                v.pointer("/advisory/cvss").and_then(|s| s.as_str()).unwrap_or(""),
            );
            findings.push((title, description, severity, Some("Cargo.lock".to_string())));
        }
    }

    Ok(findings)
}

fn run_npm_audit(path: &str) -> anyhow::Result<Vec<(String, String, String, Option<String>)>> {
    let output = std::process::Command::new("npm")
        .args(["audit", "--json"])
        .current_dir(path)
        .output();

    let Ok(output) = output else {
        return Ok(vec![]);
    };

    if output.stdout.is_empty() {
        return Ok(vec![]);
    }

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap_or_default();
    let mut findings = Vec::new();

    if let Some(vulns) = json.get("vulnerabilities").and_then(|v| v.as_object()) {
        for (pkg_name, vuln) in vulns {
            let severity = vuln
                .get("severity")
                .and_then(|s| s.as_str())
                .unwrap_or("low")
                .to_string();
            let title = format!("Vulnerable npm package: {pkg_name}");
            let description = vuln
                .get("via")
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.first())
                .and_then(|v| v.get("title"))
                .and_then(|s| s.as_str())
                .unwrap_or("")
                .to_string();
            findings.push((title, description, severity, Some("package-lock.json".to_string())));
        }
    }

    Ok(findings)
}

fn map_cvss_to_severity(cvss: &str) -> String {
    if cvss.is_empty() {
        return "medium".to_string();
    }
    if let Ok(score) = cvss.parse::<f64>() {
        return if score >= 9.0 { "critical" }
            else if score >= 7.0 { "high" }
            else if score >= 4.0 { "medium" }
            else { "low" }
            .to_string();
    }
    "medium".to_string()
}

fn default_image_for(pipeline_type: &str) -> &'static str {
    match pipeline_type {
        "build" => "rust:1.79-slim",
        "lint" => "rust:1.79-slim",
        "image_scan" => "aquasec/trivy:latest",
        _ => "debian:bookworm-slim",
    }
}

fn default_command_for(pipeline_type: &str) -> &'static str {
    match pipeline_type {
        "build" => "cargo build --release",
        "lint" => "cargo clippy -- -D warnings",
        "image_scan" => "trivy image --exit-code 0 --severity HIGH,CRITICAL .",
        _ => "echo 'pipeline complete'",
    }
}

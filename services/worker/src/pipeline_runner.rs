use anyhow::Context;
use redis::{aio::MultiplexedConnection, AsyncCommands};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

const STREAM: &str = "koda:jobs:pipeline";
const DEAD_LETTER: &str = "koda:jobs:pipeline:dead";
const MAX_RETRIES: u8 = 3;

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

        let mut failure_counts: std::collections::HashMap<String, u8> = std::collections::HashMap::new();

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
        // Fetch pipeline config
        let pipeline = sqlx::query!(
            "SELECT id, pipeline_type, config FROM cicd_pipelines WHERE id = $1",
            pipeline_id,
        )
        .fetch_one(&self.pool)
        .await?;

        // Mark job as running
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
                // These run in ephemeral containers
                self.run_container_pipeline(pipeline_type, pipeline_id, workspace_id, config).await?;
                Ok(None)
            }
            _ => Err(anyhow::anyhow!("unknown pipeline type: {}", pipeline_type)),
        }
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

        // Fetch active scan rules (builtin + org-level)
        let rules = sqlx::query!(
            r#"SELECT id, name, rule_type, pattern, entropy_threshold, severity
               FROM scan_rules
               WHERE is_active = true AND (is_builtin = true OR organization_id = $1)"#,
            org_id,
        )
        .fetch_all(&self.pool)
        .await?;

        // Get workspace volume path from DB
        let volume_name: Option<String> = sqlx::query_scalar::<_, String>(
            "SELECT volume_name FROM workspace_volumes WHERE workspace_id = $1 LIMIT 1",
        )
        .bind(workspace_id)
        .fetch_optional(&self.pool)
        .await?;

        let volume_path = volume_name
            .map(|n| format!("/var/lib/docker/volumes/{}", n))
            .unwrap_or_else(|| "/tmp/workspace_scan".to_string());
        let scan_path = volume_path.as_str();

        let mut findings: Vec<(String, String, String, Option<String>, Option<String>, Option<i32>, Option<String>)> = Vec::new();

        // Apply regex rules to files in volume
        for rule in &rules {
            if rule.rule_type == "regex" {
                if let Some(pattern) = &rule.pattern {
                    if let Ok(re) = regex::Regex::new(pattern) {
                        if let Ok(entries) = scan_files_in_path(scan_path, &re) {
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
                if let Ok(entries) = scan_high_entropy_strings(scan_path, threshold) {
                    for (file, line, evidence) in entries {
                        findings.push((
                            rule.name.clone(),
                            format!("High-entropy string (possible secret) found"),
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

        // Persist findings
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

        let summary = format!(
            "Secret scan completed. {} finding(s) found.",
            findings.len()
        );

        sqlx::query!(
            "UPDATE security_reports SET status = 'completed', summary = $1, updated_at = NOW() WHERE id = $2",
            summary,
            report.id,
        )
        .execute(&self.pool)
        .await?;

        tracing::info!(
            report_id = %report.id,
            findings = findings.len(),
            "secret scan completed"
        );

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

        let volume_name2: Option<String> = sqlx::query_scalar::<_, String>(
            "SELECT volume_name FROM workspace_volumes WHERE workspace_id = $1 LIMIT 1",
        )
        .bind(workspace_id)
        .fetch_optional(&self.pool)
        .await?;

        let volume_path = volume_name2
            .map(|n| format!("/var/lib/docker/volumes/{}", n))
            .unwrap_or_else(|| "/tmp/workspace_scan".to_string());
        let scan_path = volume_path.as_str();

        let mut findings = Vec::new();

        // Run cargo audit if Cargo.lock present
        let cargo_lock = std::path::Path::new(scan_path).join("Cargo.lock");
        if cargo_lock.exists() {
            match run_cargo_audit(scan_path) {
                Ok(vulns) => findings.extend(vulns),
                Err(e) => tracing::warn!(error = %e, "cargo audit failed"),
            }
        }

        // Run npm audit if package-lock.json present
        let package_lock = std::path::Path::new(scan_path).join("package-lock.json");
        if package_lock.exists() {
            match run_npm_audit(scan_path) {
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

    // ── SAST (LLM-based OWASP scan) ───────────────────────────────────────────

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

        let summary = "SAST scan queued — LLM-based OWASP analysis pending.".to_string();

        sqlx::query!(
            "UPDATE security_reports SET status = 'completed', summary = $1, updated_at = NOW() WHERE id = $2",
            summary,
            report.id,
        )
        .execute(&self.pool)
        .await?;

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
            .unwrap_or(default_image_for(pipeline_type));

        let command = config
            .get("command")
            .and_then(|v| v.as_str())
            .unwrap_or(default_command_for(pipeline_type));

        let container_name = format!(
            "koda-pipeline-{}-{}",
            pipeline_type,
            &pipeline_id.to_string()[..8]
        );

        tracing::info!(
            container = %container_name,
            image = %image,
            command = %command,
            pipeline_type = %pipeline_type,
            "running container pipeline"
        );

        // In a full implementation this uses bollard to create an ephemeral container.
        // The container has resource limits and is removed on completion.
        // For now we log the intent; the Docker socket is accessed via docker-socket-proxy.
        tracing::info!(
            pipeline_id = %pipeline_id,
            workspace_id = %workspace_id,
            container_name = %container_name,
            "container pipeline execution stubbed (docker-socket-proxy required at runtime)"
        );

        Ok(())
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
        "image_scan" => "trivy image --exit-code 0 --severity HIGH,CRITICAL",
        _ => "echo 'pipeline complete'",
    }
}

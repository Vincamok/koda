use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    error::AppError,
    middleware::auth::{AuthUser, OrgContext},
};

// ── Pipeline CRUD ────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreatePipelineRequest {
    pub name: String,
    pub pipeline_type: String,
    #[schema(value_type = Object)]
    pub config: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PipelineResponse {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub pipeline_type: String,
    pub status: String,
    #[schema(value_type = Object)]
    pub config: serde_json::Value,
    pub last_run_at: Option<time::OffsetDateTime>,
    pub created_at: time::OffsetDateTime,
}

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{org_id}/workspaces/{workspace_id}/pipelines",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
    ),
    request_body = CreatePipelineRequest,
    responses(
        (status = 200, description = "Pipeline created", body = PipelineResponse),
        (status = 422, description = "Invalid pipeline type"),
    ),
    tag = "pipelines",
    security(("session" = []))
)]
pub async fn post_pipeline(
    State(pool): State<PgPool>,
    Extension(org): Extension<OrgContext>,
    Path((_org_id, workspace_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<CreatePipelineRequest>,
) -> Result<Json<PipelineResponse>, AppError> {
    let valid_types = ["build", "lint", "secret_scan", "sast", "dependency_scan", "image_scan"];
    if !valid_types.contains(&body.pipeline_type.as_str()) {
        return Err(AppError::Validation(format!(
            "invalid pipeline_type: {}",
            body.pipeline_type
        )));
    }

    let config = body.config.unwrap_or(serde_json::json!({}));

    let row = sqlx::query!(
        r#"INSERT INTO cicd_pipelines (workspace_id, organization_id, name, pipeline_type, config)
           VALUES ($1, $2, $3, $4, $5)
           RETURNING id, workspace_id, organization_id, name, pipeline_type, status,
                     config, last_run_at, created_at"#,
        workspace_id,
        org.id,
        body.name,
        body.pipeline_type,
        config,
    )
    .fetch_one(&pool)
    .await?;

    Ok(Json(PipelineResponse {
        id: row.id,
        workspace_id: row.workspace_id,
        organization_id: row.organization_id,
        name: row.name,
        pipeline_type: row.pipeline_type,
        status: row.status,
        config: row.config,
        last_run_at: row.last_run_at,
        created_at: row.created_at,
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{org_id}/workspaces/{workspace_id}/pipelines",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
    ),
    responses(
        (status = 200, description = "List of pipelines", body = Vec<PipelineResponse>),
    ),
    tag = "pipelines",
    security(("session" = []))
)]
pub async fn get_pipelines(
    State(pool): State<PgPool>,
    Extension(org): Extension<OrgContext>,
    Path((_org_id, workspace_id)): Path<(Uuid, Uuid)>,
    Query(q): Query<PaginationQuery>,
) -> Result<Json<Vec<PipelineResponse>>, AppError> {
    let limit = q.limit.unwrap_or(50).min(200);
    let offset = q.offset.unwrap_or(0);

    let rows = sqlx::query!(
        r#"SELECT id, workspace_id, organization_id, name, pipeline_type, status,
                  config, last_run_at, created_at
           FROM cicd_pipelines
           WHERE workspace_id = $1 AND organization_id = $2
           ORDER BY created_at DESC
           LIMIT $3 OFFSET $4"#,
        workspace_id,
        org.id,
        limit,
        offset,
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(
        rows.into_iter()
            .map(|r| PipelineResponse {
                id: r.id,
                workspace_id: r.workspace_id,
                organization_id: r.organization_id,
                name: r.name,
                pipeline_type: r.pipeline_type,
                status: r.status,
                config: r.config,
                last_run_at: r.last_run_at,
                created_at: r.created_at,
            })
            .collect(),
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{org_id}/workspaces/{workspace_id}/pipelines/{pipeline_id}",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        ("pipeline_id" = Uuid, Path, description = "Pipeline ID"),
    ),
    responses(
        (status = 200, description = "Pipeline details", body = PipelineResponse),
        (status = 404, description = "Not found"),
    ),
    tag = "pipelines",
    security(("session" = []))
)]
pub async fn get_pipeline(
    State(pool): State<PgPool>,
    Extension(org): Extension<OrgContext>,
    Path((_org_id, workspace_id, pipeline_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<Json<PipelineResponse>, AppError> {
    let row = sqlx::query!(
        r#"SELECT id, workspace_id, organization_id, name, pipeline_type, status,
                  config, last_run_at, created_at
           FROM cicd_pipelines
           WHERE id = $1 AND workspace_id = $2 AND organization_id = $3"#,
        pipeline_id,
        workspace_id,
        org.id,
    )
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(PipelineResponse {
        id: row.id,
        workspace_id: row.workspace_id,
        organization_id: row.organization_id,
        name: row.name,
        pipeline_type: row.pipeline_type,
        status: row.status,
        config: row.config,
        last_run_at: row.last_run_at,
        created_at: row.created_at,
    }))
}

#[utoipa::path(
    delete,
    path = "/api/v1/organizations/{org_id}/workspaces/{workspace_id}/pipelines/{pipeline_id}",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        ("pipeline_id" = Uuid, Path, description = "Pipeline ID"),
    ),
    responses(
        (status = 204, description = "Pipeline deleted"),
        (status = 404, description = "Not found"),
    ),
    tag = "pipelines",
    security(("session" = []))
)]
pub async fn delete_pipeline(
    State(pool): State<PgPool>,
    Extension(org): Extension<OrgContext>,
    Path((_org_id, workspace_id, pipeline_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<axum::http::StatusCode, AppError> {
    let deleted = sqlx::query!(
        r#"DELETE FROM cicd_pipelines
           WHERE id = $1 AND workspace_id = $2 AND organization_id = $3"#,
        pipeline_id,
        workspace_id,
        org.id,
    )
    .execute(&pool)
    .await?;

    if deleted.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(axum::http::StatusCode::NO_CONTENT)
}

// ── Run Pipeline ─────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct RunResponse {
    pub job_id: Uuid,
    pub pipeline_id: Uuid,
    pub status: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{org_id}/workspaces/{workspace_id}/pipelines/{pipeline_id}/run",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        ("pipeline_id" = Uuid, Path, description = "Pipeline ID"),
    ),
    responses(
        (status = 200, description = "Run enqueued", body = RunResponse),
        (status = 409, description = "Pipeline already running"),
        (status = 404, description = "Not found"),
    ),
    tag = "pipelines",
    security(("session" = []))
)]
pub async fn post_pipeline_run(
    State(pool): State<PgPool>,
    Extension(org): Extension<OrgContext>,
    Extension(_user): Extension<AuthUser>,
    Path((_org_id, workspace_id, pipeline_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<Json<RunResponse>, AppError> {
    // Verify pipeline belongs to org/workspace
    let pipeline = sqlx::query!(
        r#"SELECT id, pipeline_type, status FROM cicd_pipelines
           WHERE id = $1 AND workspace_id = $2 AND organization_id = $3"#,
        pipeline_id,
        workspace_id,
        org.id,
    )
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::NotFound)?;

    if pipeline.status == "running" {
        return Err(AppError::Conflict("pipeline already running".into()));
    }

    // Create job record
    let payload = serde_json::json!({
        "type": "run_pipeline",
        "pipeline_id": pipeline_id,
        "workspace_id": workspace_id,
        "org_id": org.id,
        "trigger": "manual",
    });

    let job = sqlx::query!(
        r#"INSERT INTO jobs (job_type, payload, status)
           VALUES ('pipeline', $1, 'pending')
           RETURNING id"#,
        payload,
    )
    .fetch_one(&pool)
    .await?;

    // Update pipeline status to running
    sqlx::query!(
        "UPDATE cicd_pipelines SET status = 'running', last_run_at = NOW(), updated_at = NOW() WHERE id = $1",
        pipeline_id,
    )
    .execute(&pool)
    .await?;

    Ok(Json(RunResponse {
        job_id: job.id,
        pipeline_id: pipeline.id,
        status: "running".into(),
    }))
}

// ── Automation Triggers ───────────────────────────────────────────────────────

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateTriggerRequest {
    pub pipeline_id: Uuid,
    pub trigger_type: String,
    pub schedule_cron: Option<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TriggerResponse {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub pipeline_id: Uuid,
    pub trigger_type: String,
    pub schedule_cron: Option<String>,
    pub is_active: bool,
    pub created_at: time::OffsetDateTime,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{org_id}/workspaces/{workspace_id}/triggers",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
    ),
    request_body = CreateTriggerRequest,
    responses(
        (status = 200, description = "Trigger created", body = TriggerResponse),
        (status = 422, description = "Validation error"),
    ),
    tag = "pipelines",
    security(("session" = []))
)]
pub async fn post_trigger(
    State(pool): State<PgPool>,
    Extension(org): Extension<OrgContext>,
    Path((_org_id, workspace_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<CreateTriggerRequest>,
) -> Result<Json<TriggerResponse>, AppError> {
    let valid_types = ["on_push", "schedule", "manual"];
    if !valid_types.contains(&body.trigger_type.as_str()) {
        return Err(AppError::Validation(format!(
            "invalid trigger_type: {}",
            body.trigger_type
        )));
    }

    if body.trigger_type == "schedule" && body.schedule_cron.is_none() {
        return Err(AppError::Validation(
            "schedule_cron required for schedule triggers".into(),
        ));
    }

    // Verify pipeline ownership
    let exists = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM cicd_pipelines WHERE id = $1 AND workspace_id = $2 AND organization_id = $3",
        body.pipeline_id,
        workspace_id,
        org.id,
    )
    .fetch_one(&pool)
    .await?;

    if exists.unwrap_or(0) == 0 {
        return Err(AppError::NotFound);
    }

    let row = sqlx::query!(
        r#"INSERT INTO automation_triggers (workspace_id, pipeline_id, trigger_type, schedule_cron)
           VALUES ($1, $2, $3, $4)
           RETURNING id, workspace_id, pipeline_id, trigger_type, schedule_cron, is_active, created_at"#,
        workspace_id,
        body.pipeline_id,
        body.trigger_type,
        body.schedule_cron,
    )
    .fetch_one(&pool)
    .await?;

    Ok(Json(TriggerResponse {
        id: row.id,
        workspace_id: row.workspace_id,
        pipeline_id: row.pipeline_id,
        trigger_type: row.trigger_type,
        schedule_cron: row.schedule_cron,
        is_active: row.is_active,
        created_at: row.created_at,
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{org_id}/workspaces/{workspace_id}/triggers",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
    ),
    responses(
        (status = 200, description = "List of triggers", body = Vec<TriggerResponse>),
    ),
    tag = "pipelines",
    security(("session" = []))
)]
pub async fn get_triggers(
    State(pool): State<PgPool>,
    Extension(org): Extension<OrgContext>,
    Path((_org_id, workspace_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<TriggerResponse>>, AppError> {
    let rows = sqlx::query!(
        r#"SELECT at.id, at.workspace_id, at.pipeline_id, at.trigger_type,
                  at.schedule_cron, at.is_active, at.created_at
           FROM automation_triggers at
           JOIN cicd_pipelines cp ON cp.id = at.pipeline_id
           WHERE at.workspace_id = $1 AND cp.organization_id = $2
           ORDER BY at.created_at DESC"#,
        workspace_id,
        org.id,
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(
        rows.into_iter()
            .map(|r| TriggerResponse {
                id: r.id,
                workspace_id: r.workspace_id,
                pipeline_id: r.pipeline_id,
                trigger_type: r.trigger_type,
                schedule_cron: r.schedule_cron,
                is_active: r.is_active,
                created_at: r.created_at,
            })
            .collect(),
    ))
}

// ── Incoming Webhooks ─────────────────────────────────────────────────────────

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct WebhookEventResponse {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub hmac_valid: bool,
    pub received_at: time::OffsetDateTime,
    pub source_ip: Option<String>,
}

#[utoipa::path(
    post,
    path = "/api/v1/webhooks/{workspace_id}",
    params(("workspace_id" = Uuid, Path, description = "Workspace ID")),
    responses(
        (status = 200, description = "Webhook received", body = WebhookEventResponse),
        (status = 404, description = "Workspace not found"),
    ),
    tag = "webhooks"
)]
pub async fn post_webhook(
    State(pool): State<PgPool>,
    Path(workspace_id): Path<Uuid>,
    headers: axum::http::HeaderMap,
    body: axum::body::Bytes,
) -> Result<Json<WebhookEventResponse>, AppError> {
    let sig_header = headers
        .get("X-Hub-Signature-256")
        .or_else(|| headers.get("X-Koda-Signature"))
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    // Fetch workspace webhook secret
    let workspace = sqlx::query!(
        "SELECT id, organization_id FROM workspaces WHERE id = $1 AND deleted_at IS NULL",
        workspace_id,
    )
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::NotFound)?;

    // Verify HMAC-SHA256 if signature provided
    let hmac_valid = if !sig_header.is_empty() {
        // Workspace webhook_secret stored in config JSONB
        let secret_row: Option<String> = sqlx::query_scalar::<_, Option<String>>(
            "SELECT config->>'webhook_secret' FROM workspaces WHERE id = $1",
        )
        .bind(workspace_id)
        .fetch_optional(&pool)
        .await?
        .flatten();

        if let Some(secret) = secret_row {
            verify_hmac_sha256(&body, &secret, sig_header)
        } else {
            false
        }
    } else {
        false
    };

    // Capture headers as JSON
    let headers_json: serde_json::Value = {
        let map: serde_json::Map<String, serde_json::Value> = headers
            .iter()
            .filter_map(|(k, v)| v.to_str().ok().map(|s| (k.as_str().to_string(), serde_json::Value::String(s.to_string()))))
            .collect();
        serde_json::Value::Object(map)
    };

    // Parse body as JSON, fallback to raw string
    let body_json: serde_json::Value =
        serde_json::from_slice(&body).unwrap_or_else(|_| serde_json::json!({"raw": String::from_utf8_lossy(&body).to_string()}));

    let row = sqlx::query!(
        r#"INSERT INTO incoming_webhook_events
               (workspace_id, token, headers, body, hmac_valid)
           VALUES ($1, $2, $3, $4, $5)
           RETURNING id, workspace_id, hmac_valid, received_at, source_ip"#,
        workspace_id,
        sig_header,
        headers_json,
        body_json,
        hmac_valid,
    )
    .fetch_one(&pool)
    .await?;

    // If HMAC valid and workspace has on_push triggers, enqueue pipeline jobs
    if hmac_valid {
        enqueue_push_pipelines(&pool, workspace_id, workspace.organization_id).await?;
    }

    Ok(Json(WebhookEventResponse {
        id: row.id,
        workspace_id: row.workspace_id,
        hmac_valid: row.hmac_valid,
        received_at: row.received_at,
        source_ip: row.source_ip,
    }))
}

fn verify_hmac_sha256(body: &[u8], secret: &str, signature: &str) -> bool {
    type HmacSha256 = Hmac<Sha256>;
    let Ok(mut mac) = HmacSha256::new_from_slice(secret.as_bytes()) else {
        return false;
    };
    mac.update(body);
    let expected = hex::encode(mac.finalize().into_bytes());
    let sig_hex = signature
        .strip_prefix("sha256=")
        .unwrap_or(signature)
        .to_lowercase();
    constant_time_eq(&expected, &sig_hex)
}

fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.bytes()
        .zip(b.bytes())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}

async fn enqueue_push_pipelines(
    pool: &PgPool,
    workspace_id: Uuid,
    org_id: Uuid,
) -> anyhow::Result<()> {
    let triggers = sqlx::query!(
        r#"SELECT at.pipeline_id FROM automation_triggers at
           WHERE at.workspace_id = $1 AND at.trigger_type = 'on_push' AND at.is_active = true"#,
        workspace_id,
    )
    .fetch_all(pool)
    .await?;

    for t in triggers {
        let payload = serde_json::json!({
            "type": "run_pipeline",
            "pipeline_id": t.pipeline_id,
            "workspace_id": workspace_id,
            "org_id": org_id,
            "trigger": "on_push",
        });
        sqlx::query!(
            "INSERT INTO jobs (job_type, payload, status) VALUES ('pipeline', $1, 'pending')",
            payload,
        )
        .execute(pool)
        .await?;
    }
    Ok(())
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{org_id}/workspaces/{workspace_id}/webhook-events",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
    ),
    responses(
        (status = 200, description = "Webhook event inbox", body = Vec<WebhookEventResponse>),
    ),
    tag = "webhooks",
    security(("session" = []))
)]
pub async fn get_webhook_events(
    State(pool): State<PgPool>,
    Extension(org): Extension<OrgContext>,
    Path((_org_id, workspace_id)): Path<(Uuid, Uuid)>,
    Query(q): Query<PaginationQuery>,
) -> Result<Json<Vec<WebhookEventResponse>>, AppError> {
    let limit = q.limit.unwrap_or(50).min(200);
    let offset = q.offset.unwrap_or(0);

    // Verify workspace belongs to org
    sqlx::query_scalar!(
        "SELECT id FROM workspaces WHERE id = $1 AND organization_id = $2 AND deleted_at IS NULL",
        workspace_id,
        org.id,
    )
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let rows = sqlx::query!(
        r#"SELECT id, workspace_id, hmac_valid, received_at, source_ip
           FROM incoming_webhook_events
           WHERE workspace_id = $1
           ORDER BY received_at DESC
           LIMIT $2 OFFSET $3"#,
        workspace_id,
        limit,
        offset,
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(
        rows.into_iter()
            .map(|r| WebhookEventResponse {
                id: r.id,
                workspace_id: r.workspace_id,
                hmac_valid: r.hmac_valid,
                received_at: r.received_at,
                source_ip: r.source_ip,
            })
            .collect(),
    ))
}

// ── Security Reports ──────────────────────────────────────────────────────────

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SecurityReportResponse {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub organization_id: Uuid,
    pub pipeline_id: Option<Uuid>,
    pub scan_type: String,
    pub status: String,
    pub summary: Option<String>,
    pub created_at: time::OffsetDateTime,
    pub findings: Vec<VulnerabilityFindingResponse>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct VulnerabilityFindingResponse {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub severity: String,
    pub rule_id: Option<String>,
    pub file_path: Option<String>,
    pub line_number: Option<i32>,
    pub evidence: Option<String>,
    pub remediation: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{org_id}/workspaces/{workspace_id}/security-reports",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
    ),
    responses(
        (status = 200, description = "Security scan reports", body = Vec<SecurityReportResponse>),
    ),
    tag = "security",
    security(("session" = []))
)]
pub async fn get_security_reports(
    State(pool): State<PgPool>,
    Extension(org): Extension<OrgContext>,
    Path((_org_id, workspace_id)): Path<(Uuid, Uuid)>,
    Query(q): Query<PaginationQuery>,
) -> Result<Json<Vec<SecurityReportResponse>>, AppError> {
    let limit = q.limit.unwrap_or(20).min(100);
    let offset = q.offset.unwrap_or(0);

    let reports = sqlx::query!(
        r#"SELECT id, workspace_id, organization_id, pipeline_id, scan_type, status, summary, created_at
           FROM security_reports
           WHERE workspace_id = $1 AND organization_id = $2
           ORDER BY created_at DESC
           LIMIT $3 OFFSET $4"#,
        workspace_id,
        org.id,
        limit,
        offset,
    )
    .fetch_all(&pool)
    .await?;

    let mut result = Vec::with_capacity(reports.len());
    for r in reports {
        let findings = sqlx::query!(
            r#"SELECT id, title, description, severity, rule_id, file_path, line_number, evidence, remediation
               FROM vulnerability_findings
               WHERE security_report_id = $1
               ORDER BY severity, created_at"#,
            r.id,
        )
        .fetch_all(&pool)
        .await?;

        result.push(SecurityReportResponse {
            id: r.id,
            workspace_id: r.workspace_id,
            organization_id: r.organization_id,
            pipeline_id: r.pipeline_id,
            scan_type: r.scan_type,
            status: r.status,
            summary: r.summary,
            created_at: r.created_at,
            findings: findings
                .into_iter()
                .map(|f| VulnerabilityFindingResponse {
                    id: f.id,
                    title: f.title,
                    description: f.description,
                    severity: f.severity,
                    rule_id: f.rule_id,
                    file_path: f.file_path,
                    line_number: f.line_number,
                    evidence: f.evidence,
                    remediation: f.remediation,
                })
                .collect(),
        });
    }

    Ok(Json(result))
}

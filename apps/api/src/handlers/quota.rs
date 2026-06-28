use axum::{
    extract::{Extension, Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    error::AppError,
    middleware::auth::OrgContext,
    AppState,
};

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct QuotaResponse {
    pub organization_id: Uuid,
    pub max_workspaces: i32,
    pub max_cpu_cores: i32,
    pub max_ram_gb: i32,
    pub max_storage_gb: i32,
    pub max_members: i32,
    pub used_workspaces: i64,
    pub used_members: i64,
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{org_id}/quota",
    params(("org_id" = Uuid, Path, description = "Organization ID")),
    responses(
        (status = 200, description = "Quota usage", body = QuotaResponse),
    ),
    tag = "organizations",
    security(("session" = []))
)]
pub async fn get_org_quota(
    State(state): State<AppState>,
    Extension(org): Extension<OrgContext>,
    Path(_org_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let quota = sqlx::query!(
        "SELECT max_workspaces, max_cpu_cores, max_ram_gb, max_storage_gb, max_members
         FROM organization_quotas WHERE organization_id = $1",
        org.id,
    )
    .fetch_optional(&state.pool)
    .await?;

    let (max_ws, max_cpu, max_ram, max_storage, max_members) = quota
        .map(|q| (q.max_workspaces, q.max_cpu_cores, q.max_ram_gb, q.max_storage_gb, q.max_members))
        .unwrap_or((10, 20, 40, 200, 50));

    let used_ws = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM workspaces WHERE organization_id = $1 AND status != 'closed'",
        org.id,
    )
    .fetch_one(&state.pool)
    .await?
    .unwrap_or(0);

    let used_members = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM memberships WHERE organization_id = $1",
        org.id,
    )
    .fetch_one(&state.pool)
    .await?
    .unwrap_or(0);

    Ok(Json(serde_json::json!({ "data": QuotaResponse {
        organization_id: org.id,
        max_workspaces: max_ws,
        max_cpu_cores: max_cpu,
        max_ram_gb: max_ram,
        max_storage_gb: max_storage,
        max_members,
        used_workspaces: used_ws,
        used_members,
    }})))
}

// ── AI Provider Config ────────────────────────────────────────────────────────

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct AiConfigResponse {
    pub provider: String,
    pub model_nano: String,
    pub model_quick: String,
    pub model_standard: String,
    pub model_deep: String,
    pub model_agent: String,
    pub system_prompt: Option<String>,
    pub max_tokens: i32,
    pub temperature: f64,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateAiConfigRequest {
    pub provider: Option<String>,
    pub model_nano: Option<String>,
    pub model_quick: Option<String>,
    pub model_standard: Option<String>,
    pub model_deep: Option<String>,
    pub model_agent: Option<String>,
    pub system_prompt: Option<String>,
    pub max_tokens: Option<i32>,
    pub temperature: Option<f64>,
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{org_id}/ai-config",
    params(("org_id" = Uuid, Path, description = "Organization ID")),
    responses(
        (status = 200, description = "AI provider configuration", body = AiConfigResponse),
    ),
    tag = "organizations",
    security(("session" = []))
)]
pub async fn get_org_ai_config(
    State(state): State<AppState>,
    Extension(org): Extension<OrgContext>,
    Path(_org_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let config = sqlx::query!(
        r#"SELECT provider, model_nano, model_quick, model_standard, model_deep, model_agent,
                  system_prompt, max_tokens, temperature
           FROM ai_provider_configs WHERE organization_id = $1"#,
        org.id,
    )
    .fetch_optional(&state.pool)
    .await?;

    // Fall back to global default
    let config = if let Some(c) = config {
        c
    } else {
        sqlx::query!(
            r#"SELECT provider, model_nano, model_quick, model_standard, model_deep, model_agent,
                      system_prompt, max_tokens, temperature
               FROM ai_provider_configs WHERE is_global_default = TRUE LIMIT 1"#,
        )
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("no global AI config found")))?
    };

    Ok(Json(serde_json::json!({ "data": AiConfigResponse {
        provider: config.provider,
        model_nano: config.model_nano,
        model_quick: config.model_quick,
        model_standard: config.model_standard,
        model_deep: config.model_deep,
        model_agent: config.model_agent,
        system_prompt: config.system_prompt,
        max_tokens: config.max_tokens,
        temperature: config.temperature.unwrap_or(0.7),
    }})))
}

#[utoipa::path(
    patch,
    path = "/api/v1/organizations/{org_id}/ai-config",
    params(("org_id" = Uuid, Path, description = "Organization ID")),
    request_body = UpdateAiConfigRequest,
    responses(
        (status = 200, description = "Config updated"),
        (status = 403, description = "Only owner/admin can update"),
    ),
    tag = "organizations",
    security(("session" = []))
)]
pub async fn patch_org_ai_config(
    State(state): State<AppState>,
    Extension(auth): Extension<crate::middleware::auth::AuthUser>,
    Extension(org): Extension<OrgContext>,
    Path(_org_id): Path<Uuid>,
    Json(body): Json<UpdateAiConfigRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    if !matches!(org.role.as_str(), "owner" | "admin") {
        return Err(AppError::Forbidden("owner or admin required".into()));
    }

    let valid_providers = ["anthropic", "openai", "mistral", "local"];
    if let Some(ref p) = body.provider {
        if !valid_providers.contains(&p.as_str()) {
            return Err(AppError::Validation(format!("invalid provider: {p}")));
        }
    }

    // Upsert
    sqlx::query!(
        r#"INSERT INTO ai_provider_configs
               (organization_id, provider, model_nano, model_quick, model_standard, model_deep,
                model_agent, system_prompt, max_tokens, temperature)
           VALUES ($1,
               COALESCE($2, 'anthropic'),
               COALESCE($3, 'claude-haiku-4-5-20251001'),
               COALESCE($4, 'claude-haiku-4-5-20251001'),
               COALESCE($5, 'claude-sonnet-4-6'),
               COALESCE($6, 'claude-sonnet-4-6'),
               COALESCE($7, 'claude-opus-4-8'),
               $8, COALESCE($9, 4096), COALESCE($10, 0.7))
           ON CONFLICT (organization_id) DO UPDATE SET
               provider         = COALESCE($2, ai_provider_configs.provider),
               model_nano       = COALESCE($3, ai_provider_configs.model_nano),
               model_quick      = COALESCE($4, ai_provider_configs.model_quick),
               model_standard   = COALESCE($5, ai_provider_configs.model_standard),
               model_deep       = COALESCE($6, ai_provider_configs.model_deep),
               model_agent      = COALESCE($7, ai_provider_configs.model_agent),
               system_prompt    = COALESCE($8, ai_provider_configs.system_prompt),
               max_tokens       = COALESCE($9, ai_provider_configs.max_tokens),
               temperature      = COALESCE($10, ai_provider_configs.temperature),
               updated_at       = NOW()"#,
        org.id,
        body.provider,
        body.model_nano,
        body.model_quick,
        body.model_standard,
        body.model_deep,
        body.model_agent,
        body.system_prompt,
        body.max_tokens,
        body.temperature,
    )
    .execute(&state.pool)
    .await?;

    crate::audit::record_audit_event(
        &state.pool,
        Some(auth.id),
        Some(org.id),
        "ai_config.update",
        Some("ai_config"),
        Some(&org.id.to_string()),
        serde_json::json!({"provider": body.provider}),
        None,
        None,
    )
    .await
    .ok();

    Ok(Json(serde_json::json!({ "data": null })))
}

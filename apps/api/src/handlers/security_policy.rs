use axum::{
    extract::{Extension, Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    audit::record_audit_event,
    error::AppError,
    middleware::auth::{AuthUser, OrgContext},
    AppState,
};

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SecurityPolicyResponse {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub min_severity_to_block: String,
    pub image_scan_trigger: String,
    pub required_scans: Vec<String>,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateSecurityPolicyRequest {
    pub min_severity_to_block: Option<String>,
    pub image_scan_trigger: Option<String>,
    pub required_scans: Option<Vec<String>>,
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{org_id}/security-policy",
    params(("org_id" = Uuid, Path, description = "Organization ID")),
    responses(
        (status = 200, description = "Security policy", body = SecurityPolicyResponse),
    ),
    tag = "security",
    security(("session" = []))
)]
pub async fn get_security_policy(
    State(state): State<AppState>,
    Extension(_auth): Extension<AuthUser>,
    Extension(org): Extension<OrgContext>,
    Path(_org_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let row = sqlx::query!(
        r#"SELECT id, organization_id, min_severity_to_block, image_scan_trigger,
                  security_ai_config, created_at, updated_at
           FROM security_policies WHERE organization_id = $1"#,
        org.id,
    )
    .fetch_optional(&state.pool)
    .await?;

    if let Some(r) = row {
        let required_scans: Vec<String> = r
            .security_ai_config
            .get("required_scans")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();

        Ok(Json(serde_json::json!({ "data": {
            "id": r.id,
            "organization_id": r.organization_id,
            "min_severity_to_block": r.min_severity_to_block,
            "image_scan_trigger": r.image_scan_trigger,
            "required_scans": required_scans,
            "created_at": r.created_at,
            "updated_at": r.updated_at,
        }})))
    } else {
        // Auto-create default
        let created = sqlx::query!(
            r#"INSERT INTO security_policies (organization_id) VALUES ($1)
               RETURNING id, organization_id, min_severity_to_block, image_scan_trigger,
                         security_ai_config, created_at, updated_at"#,
            org.id,
        )
        .fetch_one(&state.pool)
        .await?;

        Ok(Json(serde_json::json!({ "data": {
            "id": created.id,
            "organization_id": created.organization_id,
            "min_severity_to_block": created.min_severity_to_block,
            "image_scan_trigger": created.image_scan_trigger,
            "required_scans": Vec::<String>::new(),
            "created_at": created.created_at,
            "updated_at": created.updated_at,
        }})))
    }
}

#[utoipa::path(
    patch,
    path = "/api/v1/organizations/{org_id}/security-policy",
    params(("org_id" = Uuid, Path, description = "Organization ID")),
    request_body = UpdateSecurityPolicyRequest,
    responses(
        (status = 200, description = "Policy updated"),
        (status = 403, description = "Only owner/admin can update"),
        (status = 422, description = "Invalid severity or trigger"),
    ),
    tag = "security",
    security(("session" = []))
)]
pub async fn patch_security_policy(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Extension(org): Extension<OrgContext>,
    Path(_org_id): Path<Uuid>,
    Json(body): Json<UpdateSecurityPolicyRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    if !matches!(org.role.as_str(), "owner" | "admin") {
        return Err(AppError::Forbidden("owner or admin required".into()));
    }

    let valid_severities = ["critical", "high", "medium", "low", "none"];
    if let Some(ref sev) = body.min_severity_to_block {
        if !valid_severities.contains(&sev.as_str()) {
            return Err(AppError::Validation(format!("invalid severity: {sev}")));
        }
    }

    let valid_triggers = ["OnBuild", "OnLaunch", "Both", "Disabled"];
    if let Some(ref trig) = body.image_scan_trigger {
        if !valid_triggers.contains(&trig.as_str()) {
            return Err(AppError::Validation(format!("invalid trigger: {trig}")));
        }
    }

    // Upsert policy
    let existing = sqlx::query_scalar!(
        "SELECT id FROM security_policies WHERE organization_id = $1",
        org.id,
    )
    .fetch_optional(&state.pool)
    .await?;

    if existing.is_none() {
        sqlx::query!(
            "INSERT INTO security_policies (organization_id) VALUES ($1)",
            org.id,
        )
        .execute(&state.pool)
        .await?;
    }

    if let Some(ref sev) = body.min_severity_to_block {
        sqlx::query!(
            "UPDATE security_policies SET min_severity_to_block = $1, updated_at = NOW() WHERE organization_id = $2",
            sev,
            org.id,
        )
        .execute(&state.pool)
        .await?;
    }

    if let Some(ref trig) = body.image_scan_trigger {
        sqlx::query!(
            "UPDATE security_policies SET image_scan_trigger = $1, updated_at = NOW() WHERE organization_id = $2",
            trig,
            org.id,
        )
        .execute(&state.pool)
        .await?;
    }

    if let Some(ref scans) = body.required_scans {
        let scans_json = serde_json::json!({ "required_scans": scans });
        sqlx::query!(
            "UPDATE security_policies SET security_ai_config = $1, updated_at = NOW() WHERE organization_id = $2",
            scans_json,
            org.id,
        )
        .execute(&state.pool)
        .await?;
    }

    record_audit_event(
        &state.pool,
        Some(auth.id),
        Some(org.id),
        "security_policy.update",
        Some("security_policy"),
        Some(&org.id.to_string()),
        serde_json::json!({
            "min_severity_to_block": body.min_severity_to_block,
            "image_scan_trigger": body.image_scan_trigger,
        }),
        None,
        None,
    )
    .await
    .ok();

    Ok(Json(serde_json::json!({ "data": null })))
}

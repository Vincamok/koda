use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::{
    error::AppError,
    middleware::auth::{AuthUser, OrgContext},
    AppState,
};

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct McpConnectorResponse {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    pub category: String,
    pub capabilities: serde_json::Value,
    pub config_fields: serde_json::Value,
    pub tools: serde_json::Value,
    pub is_builtin: bool,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct McpBindingResponse {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub connector_definition_id: Uuid,
    pub connector_slug: String,
    pub connector_name: String,
    pub config: serde_json::Value,
    pub enabled: bool,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateMcpBindingRequest {
    pub connector_definition_id: Uuid,
    pub config: Option<serde_json::Value>,
    pub enabled: Option<bool>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{org_id}/workspaces/{workspace_id}/mcp/connectors",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
    ),
    responses(
        (status = 200, description = "List of available MCP connectors"),
    ),
    tag = "mcp",
    security(("session" = []))
)]
pub async fn get_mcp_connectors(
    State(state): State<AppState>,
    Extension(_auth): Extension<AuthUser>,
    Extension(org): Extension<OrgContext>,
    Path((_org_id, workspace_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    // Verify workspace belongs to org
    sqlx::query!(
        "SELECT id FROM workspaces WHERE id = $1 AND organization_id = $2",
        workspace_id,
        org.id,
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let rows = sqlx::query!(
        r#"SELECT id, slug, name, description, version, category,
                  capabilities, config_fields, tools, is_builtin
           FROM mcp_connector_definitions
           ORDER BY category, name"#
    )
    .fetch_all(&state.pool)
    .await?;

    let data: Vec<McpConnectorResponse> = rows
        .into_iter()
        .map(|r| McpConnectorResponse {
            id: r.id,
            slug: r.slug,
            name: r.name,
            description: r.description,
            version: r.version,
            category: r.category,
            capabilities: r.capabilities,
            config_fields: r.config_fields,
            tools: r.tools,
            is_builtin: r.is_builtin,
        })
        .collect();

    Ok(Json(json!({ "data": data })))
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{org_id}/workspaces/{workspace_id}/mcp/bindings",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
    ),
    responses(
        (status = 200, description = "Active MCP bindings for the workspace"),
        (status = 404, description = "Workspace not found"),
    ),
    tag = "mcp",
    security(("session" = []))
)]
pub async fn get_workspace_mcp_bindings(
    State(state): State<AppState>,
    Extension(_auth): Extension<AuthUser>,
    Extension(org): Extension<OrgContext>,
    Path((_org_id, workspace_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    sqlx::query!(
        "SELECT id FROM workspaces WHERE id = $1 AND organization_id = $2",
        workspace_id,
        org.id,
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let rows = sqlx::query!(
        r#"SELECT wmb.id, wmb.workspace_id, wmb.connector_definition_id,
                  wmb.config, wmb.enabled,
                  mcd.slug AS connector_slug, mcd.name AS connector_name
           FROM workspace_mcp_bindings wmb
           JOIN mcp_connector_definitions mcd ON mcd.id = wmb.connector_definition_id
           WHERE wmb.workspace_id = $1
           ORDER BY mcd.name"#,
        workspace_id,
    )
    .fetch_all(&state.pool)
    .await?;

    let data: Vec<McpBindingResponse> = rows
        .into_iter()
        .map(|r| McpBindingResponse {
            id: r.id,
            workspace_id: r.workspace_id,
            connector_definition_id: r.connector_definition_id,
            connector_slug: r.connector_slug,
            connector_name: r.connector_name,
            config: r.config,
            enabled: r.enabled,
        })
        .collect();

    Ok(Json(json!({ "data": data })))
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{org_id}/workspaces/{workspace_id}/mcp/bindings",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
    ),
    request_body = CreateMcpBindingRequest,
    responses(
        (status = 200, description = "MCP binding created or updated"),
        (status = 404, description = "Workspace or connector not found"),
    ),
    tag = "mcp",
    security(("session" = []))
)]
pub async fn post_workspace_mcp_binding(
    State(state): State<AppState>,
    Extension(_auth): Extension<AuthUser>,
    Extension(org): Extension<OrgContext>,
    Path((_org_id, workspace_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<CreateMcpBindingRequest>,
) -> Result<impl IntoResponse, AppError> {
    sqlx::query!(
        "SELECT id FROM workspaces WHERE id = $1 AND organization_id = $2",
        workspace_id,
        org.id,
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let config = body.config.unwrap_or(json!({}));
    let enabled = body.enabled.unwrap_or(true);

    let row = sqlx::query!(
        r#"INSERT INTO workspace_mcp_bindings
               (workspace_id, connector_definition_id, config, enabled)
           VALUES ($1, $2, $3, $4)
           ON CONFLICT (workspace_id, connector_definition_id)
           DO UPDATE SET config = EXCLUDED.config,
                         enabled = EXCLUDED.enabled,
                         updated_at = NOW()
           RETURNING id, workspace_id, connector_definition_id, config, enabled"#,
        workspace_id,
        body.connector_definition_id,
        config,
        enabled,
    )
    .fetch_one(&state.pool)
    .await?;

    let mcd = sqlx::query!(
        "SELECT slug, name FROM mcp_connector_definitions WHERE id = $1",
        row.connector_definition_id,
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(json!({
        "data": McpBindingResponse {
            id: row.id,
            workspace_id: row.workspace_id,
            connector_definition_id: row.connector_definition_id,
            connector_slug: mcd.slug,
            connector_name: mcd.name,
            config: row.config,
            enabled: row.enabled,
        }
    })))
}

#[utoipa::path(
    delete,
    path = "/api/v1/organizations/{org_id}/workspaces/{workspace_id}/mcp/bindings/{binding_id}",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        ("binding_id" = Uuid, Path, description = "Binding ID"),
    ),
    responses(
        (status = 200, description = "MCP binding deleted"),
        (status = 404, description = "Binding not found"),
    ),
    tag = "mcp",
    security(("session" = []))
)]
pub async fn delete_workspace_mcp_binding(
    State(state): State<AppState>,
    Extension(_auth): Extension<AuthUser>,
    Extension(org): Extension<OrgContext>,
    Path((_org_id, workspace_id, binding_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    sqlx::query!(
        "SELECT id FROM workspaces WHERE id = $1 AND organization_id = $2",
        workspace_id,
        org.id,
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let result = sqlx::query!(
        "DELETE FROM workspace_mcp_bindings WHERE id = $1 AND workspace_id = $2",
        binding_id,
        workspace_id,
    )
    .execute(&state.pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(Json(json!({ "data": null })))
}

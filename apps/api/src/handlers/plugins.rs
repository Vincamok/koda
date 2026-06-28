use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{error::AppError, middleware::auth::AuthUser, AppState};

#[derive(Serialize)]
pub struct PluginListItem {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub version: String,
    pub category: Option<String>,
    pub icon_url: Option<String>,
    pub is_builtin: bool,
    pub approved: bool,
    pub install_count: i64,
}

#[derive(Serialize)]
pub struct PluginDetail {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub version: String,
    pub category: Option<String>,
    pub icon_url: Option<String>,
    pub repo_url: Option<String>,
    pub config_schema: serde_json::Value,
    pub is_builtin: bool,
    pub approved: bool,
    pub install_count: i64,
}

#[derive(Deserialize)]
pub struct SubmitPluginRequest {
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub version: Option<String>,
    pub category: Option<String>,
    pub icon_url: Option<String>,
    pub repo_url: Option<String>,
    pub config_schema: Option<serde_json::Value>,
}

/// GET /api/v1/organizations/:org_id/plugins
/// Lists all approved plugins in the marketplace (built-in + community).
pub async fn list_plugins(
    State(state): State<AppState>,
    Extension(_auth): Extension<AuthUser>,
    Path(_org_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let rows = sqlx::query!(
        r#"SELECT id, slug, name, description, author, version, category, icon_url, is_builtin, approved,
                  (SELECT COUNT(*) FROM workspace_plugin_bindings wpb WHERE wpb.plugin_definition_id = pd.id) AS install_count
           FROM plugin_definitions pd
           WHERE approved = true OR is_builtin = true
           ORDER BY is_builtin DESC, install_count DESC, name ASC"#,
    )
    .fetch_all(&state.pool)
    .await?;

    let plugins: Vec<PluginListItem> = rows
        .into_iter()
        .map(|r| PluginListItem {
            id: r.id,
            slug: r.slug,
            name: r.name,
            description: r.description,
            author: r.author,
            version: r.version,
            category: r.category,
            icon_url: r.icon_url,
            is_builtin: r.is_builtin,
            approved: r.approved,
            install_count: r.install_count.unwrap_or(0),
        })
        .collect();

    Ok(Json(plugins))
}

/// GET /api/v1/organizations/:org_id/plugins/:plugin_id
pub async fn get_plugin(
    State(state): State<AppState>,
    Extension(_auth): Extension<AuthUser>,
    Path((_org_id, plugin_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let row = sqlx::query!(
        r#"SELECT id, slug, name, description, author, version, category, icon_url, repo_url,
                  config_schema, is_builtin, approved,
                  (SELECT COUNT(*) FROM workspace_plugin_bindings wpb WHERE wpb.plugin_definition_id = pd.id) AS install_count
           FROM plugin_definitions pd
           WHERE id = $1 AND (approved = true OR is_builtin = true)"#,
        plugin_id,
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(PluginDetail {
        id: row.id,
        slug: row.slug,
        name: row.name,
        description: row.description,
        author: row.author,
        version: row.version,
        category: row.category,
        icon_url: row.icon_url,
        repo_url: row.repo_url,
        config_schema: row.config_schema,
        is_builtin: row.is_builtin,
        approved: row.approved,
        install_count: row.install_count.unwrap_or(0),
    }))
}

/// POST /api/v1/organizations/:org_id/plugins
/// Submit a new community plugin (requires approval before it appears in the marketplace).
pub async fn submit_plugin(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(org_id): Path<Uuid>,
    Json(body): Json<SubmitPluginRequest>,
) -> Result<impl IntoResponse, AppError> {
    let slug = body.slug.trim().to_lowercase();
    if slug.is_empty() || slug.contains(' ') {
        return Err(AppError::BadRequest("slug must be non-empty and contain no spaces".into()));
    }

    let existing = sqlx::query_scalar!(
        "SELECT id FROM plugin_definitions WHERE slug = $1",
        slug,
    )
    .fetch_optional(&state.pool)
    .await?;

    if existing.is_some() {
        return Err(AppError::Conflict("a plugin with this slug already exists".into()));
    }

    let plugin_id: Uuid = sqlx::query_scalar!(
        r#"INSERT INTO plugin_definitions
               (slug, name, description, author, version, category, icon_url, repo_url, config_schema,
                is_builtin, approved, submitted_by_org, submitted_by_user)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, false, false, $10, $11)
           RETURNING id"#,
        slug,
        body.name,
        body.description,
        None::<String>,
        body.version,
        body.category,
        body.icon_url,
        body.repo_url,
        body.config_schema.unwrap_or(serde_json::json!({})),
        org_id,
        auth.id,
    )
    .fetch_one(&state.pool)
    .await?;

    Ok((
        axum::http::StatusCode::CREATED,
        Json(serde_json::json!({
            "id": plugin_id,
            "status": "pending_review",
            "message": "Plugin submitted for review. It will appear in the marketplace once approved."
        })),
    ))
}

/// GET /api/v1/organizations/:org_id/workspaces/:workspace_id/plugins
/// Lists plugins installed in this workspace.
pub async fn list_workspace_plugins(
    State(state): State<AppState>,
    Extension(_auth): Extension<AuthUser>,
    Path((_org_id, workspace_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let rows = sqlx::query!(
        r#"SELECT pd.id, pd.slug, pd.name, pd.description, pd.icon_url, pd.category,
                  wpb.id AS binding_id, wpb.status, wpb.config, wpb.container_id
           FROM workspace_plugin_bindings wpb
           JOIN plugin_definitions pd ON pd.id = wpb.plugin_definition_id
           WHERE wpb.workspace_id = $1
           ORDER BY pd.name"#,
        workspace_id,
    )
    .fetch_all(&state.pool)
    .await?;

    let plugins: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|r| serde_json::json!({
            "plugin_id": r.id,
            "binding_id": r.binding_id,
            "slug": r.slug,
            "name": r.name,
            "description": r.description,
            "icon_url": r.icon_url,
            "category": r.category,
            "status": r.status,
            "config": r.config,
        }))
        .collect();

    Ok(Json(plugins))
}

/// POST /api/v1/organizations/:org_id/workspaces/:workspace_id/plugins/:plugin_id/install
pub async fn install_plugin(
    State(state): State<AppState>,
    Extension(_auth): Extension<AuthUser>,
    Path((_org_id, workspace_id, plugin_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    // Verify plugin exists and is approved/builtin
    let plugin = sqlx::query!(
        "SELECT id, slug FROM plugin_definitions WHERE id = $1 AND (approved = true OR is_builtin = true)",
        plugin_id,
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    // Check not already installed
    let existing = sqlx::query_scalar!(
        "SELECT id FROM workspace_plugin_bindings WHERE workspace_id = $1 AND plugin_definition_id = $2",
        workspace_id,
        plugin_id,
    )
    .fetch_optional(&state.pool)
    .await?;

    if existing.is_some() {
        return Err(AppError::Conflict("plugin already installed in this workspace".into()));
    }

    let binding_id: Uuid = sqlx::query_scalar!(
        r#"INSERT INTO workspace_plugin_bindings (workspace_id, plugin_definition_id, status, config)
           VALUES ($1, $2, 'pending', '{}')
           RETURNING id"#,
        workspace_id,
        plugin_id,
    )
    .fetch_one(&state.pool)
    .await?;

    tracing::info!(
        workspace_id = %workspace_id,
        plugin_slug = %plugin.slug,
        binding_id = %binding_id,
        "plugin installed"
    );

    Ok((
        axum::http::StatusCode::CREATED,
        Json(serde_json::json!({ "binding_id": binding_id, "status": "pending" })),
    ))
}

/// POST /api/v1/organizations/:org_id/workspaces/:workspace_id/plugins/:plugin_id/uninstall
pub async fn uninstall_plugin(
    State(state): State<AppState>,
    Extension(_auth): Extension<AuthUser>,
    Path((_org_id, workspace_id, plugin_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let deleted = sqlx::query!(
        "DELETE FROM workspace_plugin_bindings WHERE workspace_id = $1 AND plugin_definition_id = $2",
        workspace_id,
        plugin_id,
    )
    .execute(&state.pool)
    .await?;

    if deleted.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(Json(serde_json::json!({ "status": "uninstalled" })))
}

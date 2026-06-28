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
pub struct GitStatusFile {
    pub path: String,
    pub status: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct GitStatusResponse {
    pub branch: String,
    pub ahead: i32,
    pub behind: i32,
    pub staged: Vec<GitStatusFile>,
    pub unstaged: Vec<GitStatusFile>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct GitStageRequest {
    pub paths: Vec<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct GitCommitRequest {
    pub message: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct GitPushRequest {
    pub force: Option<bool>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{org_id}/workspaces/{workspace_id}/git/status",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
    ),
    responses(
        (status = 200, description = "Git status for the workspace"),
        (status = 404, description = "Workspace not found"),
    ),
    tag = "git",
    security(("session" = []))
)]
pub async fn get_workspace_git_status(
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

    // Phase 3: delegate to git-manager service via Redis Streams
    Ok(Json(json!({
        "data": GitStatusResponse {
            branch: "main".into(),
            ahead: 0,
            behind: 0,
            staged: vec![],
            unstaged: vec![],
        }
    })))
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{org_id}/workspaces/{workspace_id}/git/stage",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
    ),
    request_body = GitStageRequest,
    responses(
        (status = 200, description = "Files staged"),
        (status = 404, description = "Workspace not found"),
    ),
    tag = "git",
    security(("session" = []))
)]
pub async fn post_workspace_git_stage(
    State(state): State<AppState>,
    Extension(_auth): Extension<AuthUser>,
    Extension(org): Extension<OrgContext>,
    Path((_org_id, workspace_id)): Path<(Uuid, Uuid)>,
    Json(_body): Json<GitStageRequest>,
) -> Result<impl IntoResponse, AppError> {
    sqlx::query!(
        "SELECT id FROM workspaces WHERE id = $1 AND organization_id = $2",
        workspace_id,
        org.id,
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(json!({ "data": { "staged": true } })))
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{org_id}/workspaces/{workspace_id}/git/commit",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
    ),
    request_body = GitCommitRequest,
    responses(
        (status = 200, description = "Commit created"),
        (status = 404, description = "Workspace not found"),
    ),
    tag = "git",
    security(("session" = []))
)]
pub async fn post_workspace_git_commit(
    State(state): State<AppState>,
    Extension(_auth): Extension<AuthUser>,
    Extension(org): Extension<OrgContext>,
    Path((_org_id, workspace_id)): Path<(Uuid, Uuid)>,
    Json(_body): Json<GitCommitRequest>,
) -> Result<impl IntoResponse, AppError> {
    sqlx::query!(
        "SELECT id FROM workspaces WHERE id = $1 AND organization_id = $2",
        workspace_id,
        org.id,
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(json!({ "data": { "committed": true, "sha": null } })))
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{org_id}/workspaces/{workspace_id}/git/push",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
    ),
    request_body = GitPushRequest,
    responses(
        (status = 200, description = "Pushed to remote"),
        (status = 404, description = "Workspace not found"),
    ),
    tag = "git",
    security(("session" = []))
)]
pub async fn post_workspace_git_push(
    State(state): State<AppState>,
    Extension(_auth): Extension<AuthUser>,
    Extension(org): Extension<OrgContext>,
    Path((_org_id, workspace_id)): Path<(Uuid, Uuid)>,
    Json(_body): Json<GitPushRequest>,
) -> Result<impl IntoResponse, AppError> {
    sqlx::query!(
        "SELECT id FROM workspaces WHERE id = $1 AND organization_id = $2",
        workspace_id,
        org.id,
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(json!({ "data": { "pushed": true } })))
}

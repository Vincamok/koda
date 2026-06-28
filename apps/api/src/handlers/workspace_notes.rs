use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    error::AppError,
    middleware::auth::{AuthUser, OrgContext},
    AppState,
};

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct WorkspaceNoteResponse {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub content: String,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpsertWorkspaceNoteRequest {
    pub content: String,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{org_id}/workspaces/{workspace_id}/notes",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
    ),
    responses(
        (status = 200, description = "Workspace note for the current user (null if none)"),
        (status = 404, description = "Workspace not found"),
    ),
    tag = "ide",
    security(("session" = []))
)]
pub async fn get_workspace_note(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
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

    let note = sqlx::query!(
        r#"SELECT id, workspace_id, content, updated_at
           FROM workspace_notes
           WHERE workspace_id = $1 AND user_id = $2"#,
        workspace_id,
        auth.id,
    )
    .fetch_optional(&state.pool)
    .await?;

    match note {
        Some(n) => Ok(Json(json!({
            "data": WorkspaceNoteResponse {
                id: n.id,
                workspace_id: n.workspace_id,
                content: n.content,
                updated_at: n.updated_at,
            }
        }))),
        None => Ok(Json(json!({ "data": null }))),
    }
}

#[utoipa::path(
    put,
    path = "/api/v1/organizations/{org_id}/workspaces/{workspace_id}/notes",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
    ),
    request_body = UpsertWorkspaceNoteRequest,
    responses(
        (status = 200, description = "Note saved"),
        (status = 404, description = "Workspace not found"),
    ),
    tag = "ide",
    security(("session" = []))
)]
pub async fn put_workspace_note(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Extension(org): Extension<OrgContext>,
    Path((_org_id, workspace_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpsertWorkspaceNoteRequest>,
) -> Result<impl IntoResponse, AppError> {
    sqlx::query!(
        "SELECT id FROM workspaces WHERE id = $1 AND organization_id = $2",
        workspace_id,
        org.id,
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let row = sqlx::query!(
        r#"INSERT INTO workspace_notes (workspace_id, organization_id, user_id, content)
           VALUES ($1, $2, $3, $4)
           ON CONFLICT (workspace_id, user_id)
           DO UPDATE SET content = EXCLUDED.content, updated_at = NOW()
           RETURNING id, workspace_id, content, updated_at"#,
        workspace_id,
        org.id,
        auth.id,
        body.content,
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(json!({
        "data": WorkspaceNoteResponse {
            id: row.id,
            workspace_id: row.workspace_id,
            content: row.content,
            updated_at: row.updated_at,
        }
    })))
}

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::AppError,
    middleware::auth::{AuthUser, OrgContext},
    AppState,
};

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct TicketResponse {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub organization_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub external_url: Option<String>,
    pub external_system: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateTicketRequest {
    #[validate(length(min = 1, max = 500))]
    pub title: String,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub external_url: Option<String>,
    pub external_system: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTicketRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub external_url: Option<String>,
}

// ── Validation helpers ────────────────────────────────────────────────────────

fn validate_status(s: &str) -> Result<(), AppError> {
    if !["open", "in_progress", "closed"].contains(&s) {
        return Err(AppError::BadRequest(format!("invalid status '{s}'")));
    }
    Ok(())
}

fn validate_priority(p: &str) -> Result<(), AppError> {
    if !["critical", "high", "medium", "low"].contains(&p) {
        return Err(AppError::BadRequest(format!("invalid priority '{p}'")));
    }
    Ok(())
}

fn validate_external_system(s: &str) -> Result<(), AppError> {
    if !["jira", "linear", "github", "gitlab", "notion"].contains(&s) {
        return Err(AppError::BadRequest(format!("invalid external_system '{s}'")));
    }
    Ok(())
}

async fn verify_workspace_access(
    state: &AppState,
    workspace_id: Uuid,
    org_id: Uuid,
) -> Result<(), AppError> {
    let exists = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM workspaces WHERE id = $1 AND organization_id = $2 AND status != 'closed')",
        workspace_id,
        org_id,
    )
    .fetch_one(&state.pool)
    .await?
    .unwrap_or(false);

    if !exists {
        return Err(AppError::NotFound);
    }
    Ok(())
}

// ── GET /organizations/:org_id/workspaces/:workspace_id/tickets ───────────────

pub async fn get_workspace_tickets(
    State(state): State<AppState>,
    Extension(_auth): Extension<AuthUser>,
    Extension(org): Extension<OrgContext>,
    Path((_org_id, workspace_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    verify_workspace_access(&state, workspace_id, org.id).await?;

    let rows = sqlx::query!(
        r#"SELECT id, workspace_id, organization_id, title, description, status, priority,
                  external_url, external_system, created_by, created_at, updated_at
           FROM ticket_records
           WHERE workspace_id = $1 AND organization_id = $2
           ORDER BY created_at DESC"#,
        workspace_id,
        org.id,
    )
    .fetch_all(&state.pool)
    .await?;

    let data: Vec<TicketResponse> = rows
        .into_iter()
        .map(|r| TicketResponse {
            id: r.id,
            workspace_id: r.workspace_id,
            organization_id: r.organization_id,
            title: r.title,
            description: r.description,
            status: r.status,
            priority: r.priority,
            external_url: r.external_url,
            external_system: r.external_system,
            created_by: r.created_by,
            created_at: r.created_at,
            updated_at: r.updated_at,
        })
        .collect();

    Ok(Json(serde_json::json!({ "data": data })))
}

// ── POST /organizations/:org_id/workspaces/:workspace_id/tickets ──────────────

pub async fn post_workspace_ticket(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Extension(org): Extension<OrgContext>,
    Path((_org_id, workspace_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<CreateTicketRequest>,
) -> Result<impl IntoResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    verify_workspace_access(&state, workspace_id, org.id).await?;

    let status = body.status.as_deref().unwrap_or("open");
    let priority = body.priority.as_deref().unwrap_or("medium");
    validate_status(status)?;
    validate_priority(priority)?;
    if let Some(sys) = body.external_system.as_deref() {
        validate_external_system(sys)?;
    }

    let row = sqlx::query!(
        r#"INSERT INTO ticket_records
               (workspace_id, organization_id, title, description, status, priority,
                external_url, external_system, created_by)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
           RETURNING id, workspace_id, organization_id, title, description, status, priority,
                     external_url, external_system, created_by, created_at, updated_at"#,
        workspace_id,
        org.id,
        body.title,
        body.description,
        status,
        priority,
        body.external_url,
        body.external_system,
        auth.id,
    )
    .fetch_one(&state.pool)
    .await?;

    let resp = TicketResponse {
        id: row.id,
        workspace_id: row.workspace_id,
        organization_id: row.organization_id,
        title: row.title,
        description: row.description,
        status: row.status,
        priority: row.priority,
        external_url: row.external_url,
        external_system: row.external_system,
        created_by: row.created_by,
        created_at: row.created_at,
        updated_at: row.updated_at,
    };

    Ok((
        axum::http::StatusCode::CREATED,
        Json(serde_json::json!({ "data": resp })),
    ))
}

// ── PATCH /organizations/:org_id/workspaces/:workspace_id/tickets/:ticket_id ──

pub async fn patch_workspace_ticket(
    State(state): State<AppState>,
    Extension(_auth): Extension<AuthUser>,
    Extension(org): Extension<OrgContext>,
    Path((_org_id, workspace_id, ticket_id)): Path<(Uuid, Uuid, Uuid)>,
    Json(body): Json<UpdateTicketRequest>,
) -> Result<impl IntoResponse, AppError> {
    verify_workspace_access(&state, workspace_id, org.id).await?;

    // Validate optional fields before touching the DB
    if let Some(s) = body.status.as_deref() {
        validate_status(s)?;
    }
    if let Some(p) = body.priority.as_deref() {
        validate_priority(p)?;
    }

    // Fetch existing ticket
    let existing = sqlx::query!(
        r#"SELECT id, workspace_id, organization_id, title, description, status, priority,
                  external_url, external_system, created_by, created_at, updated_at
           FROM ticket_records
           WHERE id = $1 AND workspace_id = $2 AND organization_id = $3"#,
        ticket_id,
        workspace_id,
        org.id,
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let new_title = body.title.as_deref().unwrap_or(&existing.title);
    let new_description = body.description.as_deref().or(existing.description.as_deref());
    let new_status = body.status.as_deref().unwrap_or(&existing.status);
    let new_priority = body.priority.as_deref().unwrap_or(&existing.priority);
    // external_url: if provided in body (even as None via explicit null), use it; otherwise keep existing
    let new_external_url = if body.external_url.is_some() {
        body.external_url.as_deref()
    } else {
        existing.external_url.as_deref()
    };

    let row = sqlx::query!(
        r#"UPDATE ticket_records
           SET title = $1, description = $2, status = $3, priority = $4,
               external_url = $5, updated_at = NOW()
           WHERE id = $6 AND workspace_id = $7 AND organization_id = $8
           RETURNING id, workspace_id, organization_id, title, description, status, priority,
                     external_url, external_system, created_by, created_at, updated_at"#,
        new_title,
        new_description,
        new_status,
        new_priority,
        new_external_url,
        ticket_id,
        workspace_id,
        org.id,
    )
    .fetch_one(&state.pool)
    .await?;

    let resp = TicketResponse {
        id: row.id,
        workspace_id: row.workspace_id,
        organization_id: row.organization_id,
        title: row.title,
        description: row.description,
        status: row.status,
        priority: row.priority,
        external_url: row.external_url,
        external_system: row.external_system,
        created_by: row.created_by,
        created_at: row.created_at,
        updated_at: row.updated_at,
    };

    Ok(Json(serde_json::json!({ "data": resp })))
}

// ── DELETE /organizations/:org_id/workspaces/:workspace_id/tickets/:ticket_id ─

pub async fn delete_workspace_ticket(
    State(state): State<AppState>,
    Extension(_auth): Extension<AuthUser>,
    Extension(org): Extension<OrgContext>,
    Path((_org_id, workspace_id, ticket_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    verify_workspace_access(&state, workspace_id, org.id).await?;

    let result = sqlx::query!(
        "DELETE FROM ticket_records WHERE id = $1 AND workspace_id = $2 AND organization_id = $3",
        ticket_id,
        workspace_id,
        org.id,
    )
    .execute(&state.pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(Json(serde_json::json!({ "data": null })))
}

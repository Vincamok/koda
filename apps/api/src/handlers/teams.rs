use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    audit::record_audit_event,
    error::AppError,
    middleware::auth::{AuthUser, OrgContext},
};

// ── Teams ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateTeamRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TeamResponse {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: time::OffsetDateTime,
    pub member_count: i64,
}

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn post_team(
    State(pool): State<PgPool>,
    Extension(org): Extension<OrgContext>,
    Extension(user): Extension<AuthUser>,
    Path(_org_id): Path<Uuid>,
    Json(body): Json<CreateTeamRequest>,
) -> Result<Json<TeamResponse>, AppError> {
    require_admin_or_owner(&org)?;

    if body.name.is_empty() || body.name.len() > 100 {
        return Err(AppError::Validation("team name must be 1-100 chars".into()));
    }

    let row = sqlx::query!(
        r#"INSERT INTO teams (organization_id, name, description, created_by)
           VALUES ($1, $2, $3, $4)
           RETURNING id, organization_id, name, description, created_at"#,
        org.id,
        body.name,
        body.description,
        user.id,
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        if e.to_string().contains("unique") {
            AppError::Conflict(format!("team '{}' already exists", body.name))
        } else {
            e.into()
        }
    })?;

    record_audit_event(
        &pool,
        Some(user.id),
        Some(org.id),
        "team.create",
        Some("team"),
        Some(&row.id.to_string()),
        serde_json::json!({"name": row.name}),
        None,
        None,
    )
    .await
    .ok();

    Ok(Json(TeamResponse {
        id: row.id,
        organization_id: row.organization_id,
        name: row.name,
        description: row.description,
        created_at: row.created_at,
        member_count: 0,
    }))
}

pub async fn get_teams(
    State(pool): State<PgPool>,
    Extension(org): Extension<OrgContext>,
    Path(_org_id): Path<Uuid>,
    Query(q): Query<PaginationQuery>,
) -> Result<Json<Vec<TeamResponse>>, AppError> {
    let limit = q.limit.unwrap_or(50).min(200);
    let offset = q.offset.unwrap_or(0);

    let rows = sqlx::query!(
        r#"SELECT t.id, t.organization_id, t.name, t.description, t.created_at,
                  COUNT(tm.id) AS member_count
           FROM teams t
           LEFT JOIN team_memberships tm ON tm.team_id = t.id
           WHERE t.organization_id = $1
           GROUP BY t.id
           ORDER BY t.name
           LIMIT $2 OFFSET $3"#,
        org.id,
        limit,
        offset,
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(
        rows.into_iter()
            .map(|r| TeamResponse {
                id: r.id,
                organization_id: r.organization_id,
                name: r.name,
                description: r.description,
                created_at: r.created_at,
                member_count: r.member_count.unwrap_or(0),
            })
            .collect(),
    ))
}

pub async fn delete_team(
    State(pool): State<PgPool>,
    Extension(org): Extension<OrgContext>,
    Extension(user): Extension<AuthUser>,
    Path((_org_id, team_id)): Path<(Uuid, Uuid)>,
) -> Result<axum::http::StatusCode, AppError> {
    require_admin_or_owner(&org)?;

    let deleted = sqlx::query!(
        "DELETE FROM teams WHERE id = $1 AND organization_id = $2",
        team_id,
        org.id,
    )
    .execute(&pool)
    .await?;

    if deleted.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    record_audit_event(
        &pool,
        Some(user.id),
        Some(org.id),
        "team.delete",
        Some("team"),
        Some(&team_id.to_string()),
        serde_json::json!({}),
        None,
        None,
    )
    .await
    .ok();

    Ok(axum::http::StatusCode::NO_CONTENT)
}

// ── Team Memberships ──────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct AddMemberRequest {
    pub user_id: Uuid,
    pub role: String,
}

#[derive(Debug, Serialize)]
pub struct TeamMemberResponse {
    pub id: Uuid,
    pub team_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub email: String,
    pub display_name: String,
    pub created_at: time::OffsetDateTime,
}

pub async fn post_team_member(
    State(pool): State<PgPool>,
    Extension(org): Extension<OrgContext>,
    Extension(user): Extension<AuthUser>,
    Path((_org_id, team_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<AddMemberRequest>,
) -> Result<Json<TeamMemberResponse>, AppError> {
    require_admin_or_owner(&org)?;

    let valid_roles = ["lead", "developer", "reviewer", "viewer"];
    if !valid_roles.contains(&body.role.as_str()) {
        return Err(AppError::Validation(format!("invalid role: {}", body.role)));
    }

    // Verify team belongs to org
    sqlx::query_scalar!(
        "SELECT id FROM teams WHERE id = $1 AND organization_id = $2",
        team_id,
        org.id,
    )
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::NotFound)?;

    // Verify user is org member
    let member_user = sqlx::query!(
        r#"SELECT u.id, u.email, u.display_name FROM users u
           JOIN memberships m ON m.user_id = u.id
           WHERE u.id = $1 AND m.organization_id = $2"#,
        body.user_id,
        org.id,
    )
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::Validation("user is not an org member".into()))?;

    let row = sqlx::query!(
        r#"INSERT INTO team_memberships (team_id, user_id, role, granted_by)
           VALUES ($1, $2, $3, $4)
           ON CONFLICT (team_id, user_id) DO UPDATE SET role = $3, updated_at = NOW()
           RETURNING id, team_id, user_id, role, created_at"#,
        team_id,
        body.user_id,
        body.role,
        user.id,
    )
    .fetch_one(&pool)
    .await?;

    record_audit_event(
        &pool,
        Some(user.id),
        Some(org.id),
        "team_membership.upsert",
        Some("team_membership"),
        Some(&row.id.to_string()),
        serde_json::json!({"team_id": team_id, "target_user_id": body.user_id, "role": body.role}),
        None,
        None,
    )
    .await
    .ok();

    Ok(Json(TeamMemberResponse {
        id: row.id,
        team_id: row.team_id,
        user_id: row.user_id,
        role: row.role,
        email: member_user.email,
        display_name: member_user.display_name,
        created_at: row.created_at,
    }))
}

pub async fn get_team_members(
    State(pool): State<PgPool>,
    Extension(org): Extension<OrgContext>,
    Path((_org_id, team_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<TeamMemberResponse>>, AppError> {
    // Verify team belongs to org
    sqlx::query_scalar!(
        "SELECT id FROM teams WHERE id = $1 AND organization_id = $2",
        team_id,
        org.id,
    )
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let rows = sqlx::query!(
        r#"SELECT tm.id, tm.team_id, tm.user_id, tm.role, tm.created_at,
                  u.email, u.display_name
           FROM team_memberships tm
           JOIN users u ON u.id = tm.user_id
           WHERE tm.team_id = $1
           ORDER BY tm.role, u.email"#,
        team_id,
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(
        rows.into_iter()
            .map(|r| TeamMemberResponse {
                id: r.id,
                team_id: r.team_id,
                user_id: r.user_id,
                role: r.role,
                email: r.email,
                display_name: r.display_name,
                created_at: r.created_at,
            })
            .collect(),
    ))
}

pub async fn delete_team_member(
    State(pool): State<PgPool>,
    Extension(org): Extension<OrgContext>,
    Extension(user): Extension<AuthUser>,
    Path((_org_id, team_id, target_user_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<axum::http::StatusCode, AppError> {
    require_admin_or_owner(&org)?;

    let deleted = sqlx::query!(
        r#"DELETE FROM team_memberships
           WHERE team_id = $1 AND user_id = $2"#,
        team_id,
        target_user_id,
    )
    .execute(&pool)
    .await?;

    if deleted.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    record_audit_event(
        &pool,
        Some(user.id),
        Some(org.id),
        "team_membership.delete",
        Some("team_membership"),
        Some(&target_user_id.to_string()),
        serde_json::json!({"team_id": team_id}),
        None,
        None,
    )
    .await
    .ok();

    Ok(axum::http::StatusCode::NO_CONTENT)
}

// ── Workspace Shares ──────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ShareWorkspaceRequest {
    pub user_id: Uuid,
    pub role: String,
}

#[derive(Debug, Serialize)]
pub struct WorkspaceShareResponse {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub email: String,
    pub created_at: time::OffsetDateTime,
}

pub async fn post_workspace_share(
    State(pool): State<PgPool>,
    Extension(org): Extension<OrgContext>,
    Extension(user): Extension<AuthUser>,
    Path((_org_id, workspace_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<ShareWorkspaceRequest>,
) -> Result<Json<WorkspaceShareResponse>, AppError> {
    let valid_roles = ["editor", "reviewer", "viewer"];
    if !valid_roles.contains(&body.role.as_str()) {
        return Err(AppError::Validation(format!(
            "invalid role: {} — must be editor, reviewer, or viewer",
            body.role
        )));
    }

    let target_user = sqlx::query!(
        "SELECT id, email FROM users WHERE id = $1",
        body.user_id,
    )
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let row = sqlx::query!(
        r#"INSERT INTO workspace_shares (workspace_id, shared_with_user_id, role, created_by)
           VALUES ($1, $2, $3, $4)
           RETURNING id, workspace_id, shared_with_user_id, role, created_at"#,
        workspace_id,
        body.user_id,
        body.role,
        user.id,
    )
    .fetch_one(&pool)
    .await?;

    record_audit_event(
        &pool,
        Some(user.id),
        Some(org.id),
        "workspace_share.create",
        Some("workspace_share"),
        Some(&row.id.to_string()),
        serde_json::json!({"workspace_id": workspace_id, "target_user_id": body.user_id, "role": body.role}),
        None,
        None,
    )
    .await
    .ok();

    Ok(Json(WorkspaceShareResponse {
        id: row.id,
        workspace_id: row.workspace_id,
        user_id: row.shared_with_user_id.unwrap_or(body.user_id),
        role: row.role,
        email: target_user.email,
        created_at: row.created_at,
    }))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn require_admin_or_owner(org: &OrgContext) -> Result<(), AppError> {
    if !matches!(org.role.as_str(), "owner" | "admin") {
        return Err(AppError::Forbidden("admin or owner role required".into()));
    }
    Ok(())
}

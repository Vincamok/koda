use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;
use validator::Validate;

use crate::{error::AppError, middleware::auth::AuthUser, AppState};

// ── Create organization ───────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate)]
pub struct CreateOrgRequest {
    #[validate(length(min = 1, max = 120))]
    pub name: String,
    #[validate(length(min = 1, max = 60), regex(path = *SLUG_REGEX))]
    pub slug: String,
}

lazy_static::lazy_static! {
    static ref SLUG_REGEX: regex::Regex = regex::Regex::new(r"^[a-z0-9-]+$").unwrap();
}

#[derive(Debug, Serialize)]
pub struct OrgResponse {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub status: String,
    pub created_at: OffsetDateTime,
}

pub async fn post_organization(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(body): Json<CreateOrgRequest>,
) -> Result<impl IntoResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let slug_taken = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM organizations WHERE slug = $1)",
        body.slug
    )
    .fetch_one(&state.pool)
    .await?
    .unwrap_or(false);

    if slug_taken {
        return Err(AppError::Conflict("slug already taken".into()));
    }

    let org = sqlx::query_as!(
        crate::models::user::Organization,
        r#"INSERT INTO organizations (name, slug) VALUES ($1, $2) RETURNING *"#,
        body.name,
        body.slug,
    )
    .fetch_one(&state.pool)
    .await?;

    // Creator becomes owner
    sqlx::query!(
        "INSERT INTO memberships (organization_id, user_id, role) VALUES ($1, $2, 'owner')",
        org.id,
        auth.id,
    )
    .execute(&state.pool)
    .await?;

    // Initialize default quota
    sqlx::query!(
        "INSERT INTO organization_quotas (organization_id) VALUES ($1)",
        org.id
    )
    .execute(&state.pool)
    .await?;

    Ok((
        axum::http::StatusCode::CREATED,
        Json(serde_json::json!({ "data": OrgResponse {
            id: org.id,
            name: org.name,
            slug: org.slug,
            status: org.status,
            created_at: org.created_at,
        }})),
    ))
}

// ── Get organization ──────────────────────────────────────────────────────────

pub async fn get_organization(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(org_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    // Membership verified by with_org_context middleware upstream
    let org = sqlx::query_as!(
        crate::models::user::Organization,
        "SELECT * FROM organizations WHERE id = $1",
        org_id
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(serde_json::json!({ "data": OrgResponse {
        id: org.id,
        name: org.name,
        slug: org.slug,
        status: org.status,
        created_at: org.created_at,
    }})))
}

// ── List members ──────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct MemberResponse {
    pub user_id: Uuid,
    pub email: String,
    pub display_name: String,
    pub role: String,
    pub joined_at: OffsetDateTime,
}

pub async fn get_org_members(
    State(state): State<AppState>,
    Extension(_auth): Extension<AuthUser>,
    Path(org_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let members = sqlx::query_as!(
        MemberResponse,
        r#"SELECT u.id as user_id, u.email, u.display_name, m.role, m.created_at as joined_at
           FROM memberships m
           JOIN users u ON u.id = m.user_id
           WHERE m.organization_id = $1
           ORDER BY m.created_at"#,
        org_id
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(serde_json::json!({ "data": members })))
}

// ── Invite member ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate)]
pub struct InviteMemberRequest {
    #[validate(email)]
    pub email: String,
    pub role: String,
}

pub async fn post_org_member(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(org_id): Path<Uuid>,
    Json(body): Json<InviteMemberRequest>,
) -> Result<impl IntoResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    if !["owner", "admin", "member"].contains(&body.role.as_str()) {
        return Err(AppError::BadRequest("invalid role".into()));
    }

    // Only owner/admin can invite
    let caller_role = auth.org_role.as_deref().unwrap_or("");
    if !["owner", "admin", "super_admin"].contains(&caller_role) {
        return Err(AppError::Forbidden("insufficient role".into()));
    }

    let target_user = sqlx::query_as!(
        crate::models::user::User,
        "SELECT * FROM users WHERE email = $1",
        body.email
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    sqlx::query!(
        r#"INSERT INTO memberships (organization_id, user_id, role, invited_by)
           VALUES ($1, $2, $3, $4)
           ON CONFLICT (organization_id, user_id) DO NOTHING"#,
        org_id,
        target_user.id,
        body.role,
        auth.id,
    )
    .execute(&state.pool)
    .await?;

    Ok(Json(serde_json::json!({ "data": null })))
}

// ── Change member role ────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ChangeRoleRequest {
    pub role: String,
}

pub async fn patch_org_member(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path((org_id, user_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<ChangeRoleRequest>,
) -> Result<impl IntoResponse, AppError> {
    if !["owner", "admin", "member"].contains(&body.role.as_str()) {
        return Err(AppError::BadRequest("invalid role".into()));
    }

    let caller_role = auth.org_role.as_deref().unwrap_or("");
    if !["owner", "super_admin"].contains(&caller_role) {
        return Err(AppError::Forbidden("insufficient role".into()));
    }

    sqlx::query!(
        "UPDATE memberships SET role = $1, updated_at = NOW() WHERE organization_id = $2 AND user_id = $3",
        body.role,
        org_id,
        user_id,
    )
    .execute(&state.pool)
    .await?;

    Ok(Json(serde_json::json!({ "data": null })))
}

// ── Remove member ─────────────────────────────────────────────────────────────

pub async fn delete_org_member(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path((org_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let caller_role = auth.org_role.as_deref().unwrap_or("");
    if !["owner", "admin", "super_admin"].contains(&caller_role) {
        return Err(AppError::Forbidden("insufficient role".into()));
    }

    sqlx::query!(
        "DELETE FROM memberships WHERE organization_id = $1 AND user_id = $2",
        org_id,
        user_id
    )
    .execute(&state.pool)
    .await?;

    Ok(Json(serde_json::json!({ "data": null })))
}

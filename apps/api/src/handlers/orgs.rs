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

#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct CreateOrgRequest {
    #[validate(length(min = 1, max = 120))]
    pub name: String,
    #[validate(length(min = 1, max = 60), regex(path = *SLUG_REGEX))]
    pub slug: String,
}

lazy_static::lazy_static! {
    static ref SLUG_REGEX: regex::Regex = regex::Regex::new(r"^[a-z0-9-]+$").unwrap();
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct OrgResponse {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub status: String,
    pub created_at: OffsetDateTime,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations",
    request_body = CreateOrgRequest,
    responses(
        (status = 201, description = "Organization created", body = OrgResponse),
        (status = 409, description = "Slug already taken"),
    ),
    tag = "organizations",
    security(("session" = []))
)]
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

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{org_id}",
    params(("org_id" = Uuid, Path, description = "Organization ID")),
    responses(
        (status = 200, description = "Organization details", body = OrgResponse),
        (status = 404, description = "Not found"),
    ),
    tag = "organizations",
    security(("session" = []))
)]
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

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct MemberResponse {
    pub user_id: Uuid,
    pub email: String,
    pub display_name: String,
    pub role: String,
    pub joined_at: OffsetDateTime,
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{org_id}/members",
    params(("org_id" = Uuid, Path, description = "Organization ID")),
    responses(
        (status = 200, description = "List of members", body = Vec<MemberResponse>),
    ),
    tag = "organizations",
    security(("session" = []))
)]
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

#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct InviteMemberRequest {
    #[validate(email)]
    pub email: String,
    pub role: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{org_id}/members",
    params(("org_id" = Uuid, Path, description = "Organization ID")),
    request_body = InviteMemberRequest,
    responses(
        (status = 200, description = "Member invited"),
        (status = 403, description = "Insufficient role"),
        (status = 404, description = "User not found"),
    ),
    tag = "organizations",
    security(("session" = []))
)]
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

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ChangeRoleRequest {
    pub role: String,
}

#[utoipa::path(
    patch,
    path = "/api/v1/organizations/{org_id}/members/{user_id}",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("user_id" = Uuid, Path, description = "User ID"),
    ),
    request_body = ChangeRoleRequest,
    responses(
        (status = 200, description = "Role updated"),
        (status = 403, description = "Insufficient role"),
    ),
    tag = "organizations",
    security(("session" = []))
)]
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

#[utoipa::path(
    delete,
    path = "/api/v1/organizations/{org_id}/members/{user_id}",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("user_id" = Uuid, Path, description = "User ID"),
    ),
    responses(
        (status = 200, description = "Member removed"),
        (status = 403, description = "Insufficient role"),
    ),
    tag = "organizations",
    security(("session" = []))
)]
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

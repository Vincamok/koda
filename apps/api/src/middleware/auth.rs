use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{error::AppError, models::user::AuthUser};

/// Carries resolved org context injected by with_org_context middleware.
#[derive(Clone)]
pub struct OrgContext {
    pub id: Uuid,
    pub role: String,
}

const SESSION_USER_KEY: &str = "user_id";

/// Extracts the authenticated user from the session.
/// Returns 401 if no valid session is present.
pub async fn require_auth(
    State(pool): State<PgPool>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let session = request
        .extensions()
        .get::<tower_sessions::Session>()
        .cloned()
        .ok_or(AppError::Unauthorized)?;

    let user_id: Option<String> = session
        .get(SESSION_USER_KEY)
        .await
        .map_err(|_| AppError::Unauthorized)?;

    let user_id: Uuid = user_id
        .and_then(|s| Uuid::parse_str(&s).ok())
        .ok_or(AppError::Unauthorized)?;

    let user = sqlx::query_as!(
        crate::models::user::User,
        "SELECT * FROM users WHERE id = $1",
        user_id
    )
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::Unauthorized)?;

    let auth_user = AuthUser {
        id: user.id,
        email: user.email,
        display_name: user.display_name,
        is_super_admin: user.is_super_admin,
        org_role: None,
        org_id: None,
    };

    request.extensions_mut().insert(auth_user);
    Ok(next.run(request).await)
}

/// Resolves org role for a given org context.
/// Call after require_auth — enriches the AuthUser with org_role.
pub async fn with_org_context(
    State(pool): State<PgPool>,
    axum::extract::Path(org_id): axum::extract::Path<Uuid>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let auth_user = request
        .extensions()
        .get::<AuthUser>()
        .cloned()
        .ok_or(AppError::Unauthorized)?;

    // super_admin bypasses org membership check
    let role = if auth_user.is_super_admin {
        "super_admin".to_string()
    } else {
        let membership = sqlx::query!(
            "SELECT role FROM memberships WHERE organization_id = $1 AND user_id = $2",
            org_id,
            auth_user.id
        )
        .fetch_optional(&pool)
        .await?
        .ok_or(AppError::Forbidden)?;
        membership.role
    };

    let mut enriched = auth_user;
    enriched.org_id = Some(org_id);
    enriched.org_role = Some(role.clone());
    request.extensions_mut().insert(enriched);
    request.extensions_mut().insert(OrgContext { id: org_id, role });

    Ok(next.run(request).await)
}

/// Requires super_admin role for admin panel routes.
pub async fn require_super_admin(
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let auth_user = request
        .extensions()
        .get::<AuthUser>()
        .cloned()
        .ok_or(AppError::Unauthorized)?;

    if !auth_user.is_super_admin {
        return Err(AppError::Forbidden);
    }

    Ok(next.run(request).await)
}

pub async fn set_session_user(session: &tower_sessions::Session, user_id: Uuid) -> anyhow::Result<()> {
    session.insert(SESSION_USER_KEY, user_id.to_string()).await?;
    Ok(())
}

pub async fn clear_session(session: &tower_sessions::Session) -> anyhow::Result<()> {
    session.flush().await?;
    Ok(())
}

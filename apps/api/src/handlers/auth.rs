use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Redirect},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use time::OffsetDateTime;
use tower_sessions::Session;
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::AppError,
    middleware::auth::{clear_session, set_session_user, AuthUser},
    AppState,
};

// ── Register ──────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
    #[validate(length(min = 1, max = 120))]
    pub display_name: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub email_verified: bool,
    pub created_at: OffsetDateTime,
}

pub async fn post_register(
    State(state): State<AppState>,
    session: Session,
    Json(body): Json<RegisterRequest>,
) -> Result<impl IntoResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let exists = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)",
        body.email
    )
    .fetch_one(&state.pool)
    .await?
    .unwrap_or(false);

    if exists {
        return Err(AppError::Conflict("email already registered".into()));
    }

    let hash = hash_password(&body.password)?;
    let user = sqlx::query_as!(
        crate::models::user::User,
        r#"INSERT INTO users (email, password_hash, display_name)
           VALUES ($1, $2, $3)
           RETURNING *"#,
        body.email,
        hash,
        body.display_name,
    )
    .fetch_one(&state.pool)
    .await?;

    // Bootstrap super_admin on first registration if email matches
    if let Some(ref admin_email) = state.config.bootstrap_super_admin_email {
        if !admin_email.is_empty() && user.email == *admin_email {
            sqlx::query!(
                "UPDATE users SET is_super_admin = true WHERE id = $1",
                user.id
            )
            .execute(&state.pool)
            .await?;
            tracing::info!(user_id = %user.id, "bootstrapped super_admin");
        }
    }

    set_session_user(&session, user.id).await?;

    Ok(Json(json_data(UserResponse {
        id: user.id,
        email: user.email,
        display_name: user.display_name,
        email_verified: user.email_verified,
        created_at: user.created_at,
    })))
}

// ── Login ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    pub password: String,
}

pub async fn post_login(
    State(state): State<AppState>,
    session: Session,
    Json(body): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let user = sqlx::query_as!(
        crate::models::user::User,
        "SELECT * FROM users WHERE email = $1",
        body.email
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::Unauthorized)?;

    let hash = user.password_hash.as_deref().ok_or(AppError::Unauthorized)?;
    verify_password(&body.password, hash)?;

    set_session_user(&session, user.id).await?;

    Ok(Json(json_data(UserResponse {
        id: user.id,
        email: user.email,
        display_name: user.display_name,
        email_verified: user.email_verified,
        created_at: user.created_at,
    })))
}

// ── Logout ────────────────────────────────────────────────────────────────────

pub async fn post_logout(session: Session) -> Result<impl IntoResponse, AppError> {
    clear_session(&session).await?;
    Ok(Json(serde_json::json!({ "data": null })))
}

// ── Get me ────────────────────────────────────────────────────────────────────

pub async fn get_me(
    Extension(auth): Extension<AuthUser>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let user = sqlx::query_as!(
        crate::models::user::User,
        "SELECT * FROM users WHERE id = $1",
        auth.id
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(json_data(UserResponse {
        id: user.id,
        email: user.email,
        display_name: user.display_name,
        email_verified: user.email_verified,
        created_at: user.created_at,
    })))
}

// ── OAuth — Google / GitHub / Authentik ───────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct OAuthCallbackQuery {
    pub code: String,
    pub state: Option<String>,
}

pub async fn get_oauth_authorize(
    Path(provider): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let auth_url = build_oauth_url(&provider, &state)?;
    Ok(Redirect::to(&auth_url))
}

pub async fn get_oauth_callback(
    Path(provider): Path<String>,
    Query(params): Query<OAuthCallbackQuery>,
    State(state): State<AppState>,
    session: Session,
) -> Result<impl IntoResponse, AppError> {
    let (email, display_name, avatar_url) =
        exchange_oauth_code(&provider, &params.code, &state).await?;

    let user = sqlx::query_as!(
        crate::models::user::User,
        r#"INSERT INTO users (email, display_name, avatar_url, email_verified)
           VALUES ($1, $2, $3, true)
           ON CONFLICT (email) DO UPDATE
             SET display_name = COALESCE(EXCLUDED.display_name, users.display_name),
                 avatar_url   = COALESCE(EXCLUDED.avatar_url, users.avatar_url),
                 email_verified = true,
                 updated_at   = NOW()
           RETURNING *"#,
        email,
        display_name,
        avatar_url,
    )
    .fetch_one(&state.pool)
    .await?;

    set_session_user(&session, user.id).await?;

    Ok(Redirect::to(&format!("{}/dashboard", state.config.app_base_url)))
}

// ── Internal helpers ──────────────────────────────────────────────────────────

fn hash_password(password: &str) -> anyhow::Result<String> {
    use argon2::{
        password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
        Argon2,
    };
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("hash error: {e}"))?
        .to_string();
    Ok(hash)
}

fn verify_password(password: &str, hash: &str) -> Result<(), AppError> {
    use argon2::{
        password_hash::{PasswordHash, PasswordVerifier},
        Argon2,
    };
    let parsed = PasswordHash::new(hash).map_err(|_| AppError::Unauthorized)?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .map_err(|_| AppError::Unauthorized)
}

fn build_oauth_url(provider: &str, state: &AppState) -> Result<String, AppError> {
    match provider {
        "google" => {
            let cfg = state
                .config
                .oauth
                .google
                .as_ref()
                .ok_or_else(|| AppError::BadRequest("Google OAuth not configured".into()))?;
            Ok(format!(
                "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}/api/v1/auth/oauth/google/callback&response_type=code&scope=openid%20email%20profile",
                cfg.client_id,
                state.config.app_base_url
            ))
        }
        "github" => {
            let cfg = state
                .config
                .oauth
                .github
                .as_ref()
                .ok_or_else(|| AppError::BadRequest("GitHub OAuth not configured".into()))?;
            Ok(format!(
                "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}/api/v1/auth/oauth/github/callback&scope=user:email",
                cfg.client_id,
                state.config.app_base_url
            ))
        }
        "authentik" => {
            let cfg = state
                .config
                .oauth
                .authentik
                .as_ref()
                .ok_or_else(|| AppError::BadRequest("Authentik OAuth not configured".into()))?;
            let issuer = cfg.issuer_url.as_deref().unwrap_or("");
            Ok(format!(
                "{}/authorize/?client_id={}&redirect_uri={}/api/v1/auth/oauth/authentik/callback&response_type=code&scope=openid%20email%20profile",
                issuer,
                cfg.client_id,
                state.config.app_base_url
            ))
        }
        _ => Err(AppError::BadRequest(format!("unknown provider: {provider}"))),
    }
}

async fn exchange_oauth_code(
    provider: &str,
    code: &str,
    state: &AppState,
) -> Result<(String, String, Option<String>), AppError> {
    match provider {
        "google" => exchange_google(code, state).await,
        "github" => exchange_github(code, state).await,
        "authentik" => exchange_authentik(code, state).await,
        _ => Err(AppError::BadRequest(format!("unknown provider: {provider}"))),
    }
}

async fn exchange_google(
    code: &str,
    state: &AppState,
) -> Result<(String, String, Option<String>), AppError> {
    let cfg = state
        .config
        .oauth
        .google
        .as_ref()
        .ok_or_else(|| AppError::BadRequest("Google OAuth not configured".into()))?;

    let token_resp = state
        .http
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("code", code),
            ("client_id", &cfg.client_id),
            ("client_secret", &cfg.client_secret),
            ("redirect_uri", &format!("{}/api/v1/auth/oauth/google/callback", state.config.app_base_url)),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await
        .map_err(|e| AppError::Internal(e.into()))?
        .json::<serde_json::Value>()
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    let access_token = token_resp["access_token"]
        .as_str()
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("no access_token from Google")))?;

    let userinfo = state
        .http
        .get("https://www.googleapis.com/oauth2/v3/userinfo")
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| AppError::Internal(e.into()))?
        .json::<serde_json::Value>()
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    let email = userinfo["email"]
        .as_str()
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("no email from Google")))?
        .to_string();
    let name = userinfo["name"].as_str().unwrap_or(&email).to_string();
    let picture = userinfo["picture"].as_str().map(str::to_string);

    Ok((email, name, picture))
}

async fn exchange_github(
    code: &str,
    state: &AppState,
) -> Result<(String, String, Option<String>), AppError> {
    let cfg = state
        .config
        .oauth
        .github
        .as_ref()
        .ok_or_else(|| AppError::BadRequest("GitHub OAuth not configured".into()))?;

    let token_resp = state
        .http
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .form(&[
            ("code", code),
            ("client_id", &cfg.client_id),
            ("client_secret", &cfg.client_secret),
        ])
        .send()
        .await
        .map_err(|e| AppError::Internal(e.into()))?
        .json::<serde_json::Value>()
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    let access_token = token_resp["access_token"]
        .as_str()
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("no access_token from GitHub")))?;

    let user_data = state
        .http
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("User-Agent", "koda/1.0")
        .send()
        .await
        .map_err(|e| AppError::Internal(e.into()))?
        .json::<serde_json::Value>()
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    // GitHub may not expose email directly — fetch from /user/emails
    let email = if let Some(e) = user_data["email"].as_str() {
        e.to_string()
    } else {
        let emails = state
            .http
            .get("https://api.github.com/user/emails")
            .header("Authorization", format!("Bearer {}", access_token))
            .header("User-Agent", "koda/1.0")
            .send()
            .await
            .map_err(|e| AppError::Internal(e.into()))?
            .json::<Vec<serde_json::Value>>()
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        emails
            .iter()
            .find(|e| e["primary"].as_bool().unwrap_or(false))
            .and_then(|e| e["email"].as_str())
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("no primary email from GitHub")))?
            .to_string()
    };

    let name = user_data["name"]
        .as_str()
        .or_else(|| user_data["login"].as_str())
        .unwrap_or(&email)
        .to_string();
    let avatar = user_data["avatar_url"].as_str().map(str::to_string);

    Ok((email, name, avatar))
}

async fn exchange_authentik(
    code: &str,
    state: &AppState,
) -> Result<(String, String, Option<String>), AppError> {
    let cfg = state
        .config
        .oauth
        .authentik
        .as_ref()
        .ok_or_else(|| AppError::BadRequest("Authentik OAuth not configured".into()))?;

    let issuer = cfg.issuer_url.as_deref().unwrap_or("");
    let token_url = format!("{}/token/", issuer);

    let token_resp = state
        .http
        .post(&token_url)
        .form(&[
            ("code", code),
            ("client_id", &cfg.client_id),
            ("client_secret", &cfg.client_secret),
            ("redirect_uri", &format!("{}/api/v1/auth/oauth/authentik/callback", state.config.app_base_url)),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await
        .map_err(|e| AppError::Internal(e.into()))?
        .json::<serde_json::Value>()
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    let access_token = token_resp["access_token"]
        .as_str()
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("no access_token from Authentik")))?;

    let userinfo_url = format!("{}/userinfo/", issuer);
    let userinfo = state
        .http
        .get(&userinfo_url)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| AppError::Internal(e.into()))?
        .json::<serde_json::Value>()
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    let email = userinfo["email"]
        .as_str()
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("no email from Authentik")))?
        .to_string();
    let name = userinfo["name"]
        .as_str()
        .or_else(|| userinfo["preferred_username"].as_str())
        .unwrap_or(&email)
        .to_string();

    Ok((email, name, None))
}

// ── Response helpers ──────────────────────────────────────────────────────────

fn json_data<T: Serialize>(data: T) -> serde_json::Value {
    serde_json::json!({ "data": data })
}

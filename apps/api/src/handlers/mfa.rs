use axum::{extract::State, Extension, Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use totp_rs::{Algorithm, Secret, TOTP};
use uuid::Uuid;

use crate::{
    audit::record_audit_event,
    error::AppError,
    middleware::auth::AuthUser,
};

const TOTP_ISSUER: &str = "Koda";
const TOTP_DIGITS: usize = 6;
const TOTP_STEP: u64 = 30;

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TotpSetupResponse {
    pub provisioning_uri: String,
    pub secret_base32: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/user/mfa/setup",
    responses(
        (status = 200, description = "TOTP setup initiated", body = TotpSetupResponse),
    ),
    tag = "mfa",
    security(("session" = []))
)]
pub async fn post_mfa_setup(
    State(pool): State<PgPool>,
    Extension(user): Extension<AuthUser>,
) -> Result<Json<TotpSetupResponse>, AppError> {
    // Generate a new TOTP secret
    let secret_bytes: Vec<u8> = (0..20).map(|_| rand::random::<u8>()).collect();
    let secret_b32 = base32::encode(base32::Alphabet::RFC4648 { padding: false }, &secret_bytes);

    let totp = TOTP::new(
        Algorithm::SHA1,
        TOTP_DIGITS,
        1,
        TOTP_STEP,
        secret_bytes.clone(),
        Some(TOTP_ISSUER.to_string()),
        user.email.clone(),
    )
    .map_err(|e| AppError::Internal(anyhow::anyhow!("{}", e)))?;

    let uri = totp.get_url();

    // Store unverified secret (replace if exists)
    sqlx::query!(
        r#"INSERT INTO totp_credentials (user_id, secret, verified)
           VALUES ($1, $2, false)
           ON CONFLICT (user_id) DO UPDATE SET secret = $2, verified = false, updated_at = NOW()"#,
        user.id,
        secret_b32,
    )
    .execute(&pool)
    .await?;

    Ok(Json(TotpSetupResponse {
        provisioning_uri: uri,
        secret_base32: secret_b32,
    }))
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct VerifyTotpRequest {
    pub code: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TotpStatusResponse {
    pub enabled: bool,
    pub verified: bool,
}

#[utoipa::path(
    post,
    path = "/api/v1/user/mfa/verify",
    request_body = VerifyTotpRequest,
    responses(
        (status = 200, description = "TOTP verified and enabled", body = TotpStatusResponse),
        (status = 422, description = "Invalid or expired code"),
    ),
    tag = "mfa",
    security(("session" = []))
)]
pub async fn post_mfa_verify(
    State(pool): State<PgPool>,
    Extension(user): Extension<AuthUser>,
    Json(body): Json<VerifyTotpRequest>,
) -> Result<Json<TotpStatusResponse>, AppError> {
    let cred = sqlx::query!(
        "SELECT secret, verified FROM totp_credentials WHERE user_id = $1",
        user.id,
    )
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::Validation("TOTP not set up — call POST /mfa/setup first".into()))?;

    if cred.verified {
        return Err(AppError::Conflict("TOTP already verified".into()));
    }

    let secret_bytes = base32::decode(
        base32::Alphabet::RFC4648 { padding: false },
        &cred.secret,
    )
    .ok_or_else(|| AppError::Internal(anyhow::anyhow!("invalid stored TOTP secret")))?;

    let totp = TOTP::new(
        Algorithm::SHA1,
        TOTP_DIGITS,
        1,
        TOTP_STEP,
        secret_bytes,
        Some(TOTP_ISSUER.to_string()),
        user.email.clone(),
    )
    .map_err(|e| AppError::Internal(anyhow::anyhow!("{}", e)))?;

    let valid = totp
        .check_current(&body.code)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("{}", e)))?;

    if !valid {
        return Err(AppError::Validation("invalid TOTP code".into()));
    }

    sqlx::query!(
        "UPDATE totp_credentials SET verified = true, updated_at = NOW() WHERE user_id = $1",
        user.id,
    )
    .execute(&pool)
    .await?;

    record_audit_event(
        &pool,
        Some(user.id),
        None,
        "mfa.enabled",
        Some("user"),
        Some(&user.id.to_string()),
        serde_json::json!({}),
        None,
        None,
    )
    .await
    .ok();

    Ok(Json(TotpStatusResponse {
        enabled: true,
        verified: true,
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/user/mfa/status",
    responses(
        (status = 200, description = "MFA status", body = TotpStatusResponse),
    ),
    tag = "mfa",
    security(("session" = []))
)]
pub async fn get_mfa_status(
    State(pool): State<PgPool>,
    Extension(user): Extension<AuthUser>,
) -> Result<Json<TotpStatusResponse>, AppError> {
    let cred = sqlx::query!(
        "SELECT verified FROM totp_credentials WHERE user_id = $1",
        user.id,
    )
    .fetch_optional(&pool)
    .await?;

    Ok(Json(TotpStatusResponse {
        enabled: cred.is_some(),
        verified: cred.map(|c| c.verified).unwrap_or(false),
    }))
}

#[utoipa::path(
    delete,
    path = "/api/v1/user/mfa",
    request_body = VerifyTotpRequest,
    responses(
        (status = 204, description = "MFA disabled"),
        (status = 422, description = "Invalid code"),
    ),
    tag = "mfa",
    security(("session" = []))
)]
pub async fn delete_mfa(
    State(pool): State<PgPool>,
    Extension(user): Extension<AuthUser>,
    Json(body): Json<VerifyTotpRequest>,
) -> Result<axum::http::StatusCode, AppError> {
    let cred = sqlx::query!(
        "SELECT secret, verified FROM totp_credentials WHERE user_id = $1",
        user.id,
    )
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::NotFound)?;

    if !cred.verified {
        return Err(AppError::Validation("TOTP not verified".into()));
    }

    // Require valid code to disable
    let secret_bytes = base32::decode(
        base32::Alphabet::RFC4648 { padding: false },
        &cred.secret,
    )
    .ok_or_else(|| AppError::Internal(anyhow::anyhow!("invalid stored TOTP secret")))?;

    let totp = TOTP::new(
        Algorithm::SHA1,
        TOTP_DIGITS,
        1,
        TOTP_STEP,
        secret_bytes,
        Some(TOTP_ISSUER.to_string()),
        user.email.clone(),
    )
    .map_err(|e| AppError::Internal(anyhow::anyhow!("{}", e)))?;

    let valid = totp
        .check_current(&body.code)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("{}", e)))?;

    if !valid {
        return Err(AppError::Validation("invalid TOTP code".into()));
    }

    sqlx::query!(
        "DELETE FROM totp_credentials WHERE user_id = $1",
        user.id,
    )
    .execute(&pool)
    .await?;

    record_audit_event(
        &pool,
        Some(user.id),
        None,
        "mfa.disabled",
        Some("user"),
        Some(&user.id.to_string()),
        serde_json::json!({}),
        None,
        None,
    )
    .await
    .ok();

    Ok(axum::http::StatusCode::NO_CONTENT)
}

// ── TOTP authentication check ─────────────────────────────────────────────────

pub async fn verify_totp_code(
    pool: &PgPool,
    user_id: Uuid,
    user_email: &str,
    code: &str,
) -> anyhow::Result<bool> {
    let Some(cred) = sqlx::query!(
        "SELECT secret, verified FROM totp_credentials WHERE user_id = $1 AND verified = true",
        user_id,
    )
    .fetch_optional(pool)
    .await?
    else {
        return Ok(true); // MFA not enabled — allow
    };

    let secret_bytes = base32::decode(
        base32::Alphabet::RFC4648 { padding: false },
        &cred.secret,
    )
    .ok_or_else(|| anyhow::anyhow!("invalid stored TOTP secret"))?;

    let totp = TOTP::new(
        Algorithm::SHA1,
        TOTP_DIGITS,
        1,
        TOTP_STEP,
        secret_bytes,
        Some(TOTP_ISSUER.to_string()),
        user_email.to_string(),
    )?;

    Ok(totp.check_current(code)?)
}

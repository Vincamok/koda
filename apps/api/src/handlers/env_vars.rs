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
    models::secret::SecretCrypto,
    AppState,
};

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct EnvVarResponse {
    pub id: Uuid,
    pub key: String,
    pub value: String,
    pub is_secret: bool,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateEnvVarRequest {
    #[validate(length(min = 1, max = 256))]
    pub key: String,
    pub value: String,
    pub is_secret: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEnvVarRequest {
    pub value: String,
}

// ── Helper ────────────────────────────────────────────────────────────────────

fn get_crypto() -> Result<SecretCrypto, AppError> {
    let key_hex = std::env::var("SECRET_ENCRYPTION_KEY")
        .map_err(|_| AppError::Internal(anyhow::anyhow!("SECRET_ENCRYPTION_KEY not set")))?;
    SecretCrypto::from_hex_key(&key_hex)
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))
}

fn encrypt_value(crypto: &SecretCrypto, plaintext: &str) -> Result<(String, String), AppError> {
    let (ciphertext, nonce) = crypto
        .encrypt(plaintext)
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    Ok((
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &ciphertext),
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &nonce),
    ))
}

fn decrypt_value(crypto: &SecretCrypto, value_enc: &str, nonce_b64: &str) -> Result<String, AppError> {
    let ciphertext = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        value_enc,
    )
    .map_err(|e| AppError::Internal(anyhow::anyhow!("base64 decode ciphertext: {e}")))?;
    let nonce = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        nonce_b64,
    )
    .map_err(|e| AppError::Internal(anyhow::anyhow!("base64 decode nonce: {e}")))?;
    crypto
        .decrypt(&ciphertext, &nonce)
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))
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

// ── GET /organizations/:org_id/workspaces/:workspace_id/env ──────────────────

pub async fn get_env_vars(
    State(state): State<AppState>,
    Extension(_auth): Extension<AuthUser>,
    Extension(org): Extension<OrgContext>,
    Path((_org_id, workspace_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    verify_workspace_access(&state, workspace_id, org.id).await?;

    let rows = sqlx::query!(
        r#"SELECT id, key, value_enc, nonce, is_secret, created_at, updated_at
           FROM workspace_env_vars
           WHERE workspace_id = $1 AND org_id = $2
           ORDER BY key ASC"#,
        workspace_id,
        org.id,
    )
    .fetch_all(&state.pool)
    .await?;

    let crypto = get_crypto()?;

    let mut data: Vec<EnvVarResponse> = Vec::with_capacity(rows.len());
    for row in rows {
        let value = if row.is_secret {
            "***".to_string()
        } else {
            decrypt_value(&crypto, &row.value_enc, &row.nonce)?
        };
        data.push(EnvVarResponse {
            id: row.id,
            key: row.key,
            value,
            is_secret: row.is_secret,
            created_at: row.created_at,
            updated_at: row.updated_at,
        });
    }

    Ok(Json(serde_json::json!({ "data": data })))
}

// ── POST /organizations/:org_id/workspaces/:workspace_id/env ─────────────────

pub async fn post_env_var(
    State(state): State<AppState>,
    Extension(_auth): Extension<AuthUser>,
    Extension(org): Extension<OrgContext>,
    Path((_org_id, workspace_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<CreateEnvVarRequest>,
) -> Result<impl IntoResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    verify_workspace_access(&state, workspace_id, org.id).await?;

    let crypto = get_crypto()?;
    let (value_enc, nonce) = encrypt_value(&crypto, &body.value)?;

    let row = sqlx::query!(
        r#"INSERT INTO workspace_env_vars (workspace_id, org_id, key, value_enc, nonce, is_secret)
           VALUES ($1, $2, $3, $4, $5, $6)
           RETURNING id, key, value_enc, nonce, is_secret, created_at, updated_at"#,
        workspace_id,
        org.id,
        body.key,
        value_enc,
        nonce,
        body.is_secret,
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(ref db_err) = e {
            if db_err.constraint().map_or(false, |c| c.contains("workspace_env_vars_workspace_id_key_key")) {
                return AppError::Conflict(format!("env var '{}' already exists", body.key));
            }
        }
        AppError::Database(e)
    })?;

    let value = if row.is_secret {
        "***".to_string()
    } else {
        body.value.clone()
    };

    let resp = EnvVarResponse {
        id: row.id,
        key: row.key,
        value,
        is_secret: row.is_secret,
        created_at: row.created_at,
        updated_at: row.updated_at,
    };

    Ok((
        axum::http::StatusCode::CREATED,
        Json(serde_json::json!({ "data": resp })),
    ))
}

// ── PUT /organizations/:org_id/workspaces/:workspace_id/env/:key ──────────────

pub async fn put_env_var(
    State(state): State<AppState>,
    Extension(_auth): Extension<AuthUser>,
    Extension(org): Extension<OrgContext>,
    Path((_org_id, workspace_id, key)): Path<(Uuid, Uuid, String)>,
    Json(body): Json<UpdateEnvVarRequest>,
) -> Result<impl IntoResponse, AppError> {
    verify_workspace_access(&state, workspace_id, org.id).await?;

    let crypto = get_crypto()?;
    let (value_enc, nonce) = encrypt_value(&crypto, &body.value)?;

    let row = sqlx::query!(
        r#"UPDATE workspace_env_vars
           SET value_enc = $1, nonce = $2, updated_at = NOW()
           WHERE workspace_id = $3 AND org_id = $4 AND key = $5
           RETURNING id, key, value_enc, nonce, is_secret, created_at, updated_at"#,
        value_enc,
        nonce,
        workspace_id,
        org.id,
        key,
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let value = if row.is_secret {
        "***".to_string()
    } else {
        body.value.clone()
    };

    let resp = EnvVarResponse {
        id: row.id,
        key: row.key,
        value,
        is_secret: row.is_secret,
        created_at: row.created_at,
        updated_at: row.updated_at,
    };

    Ok(Json(serde_json::json!({ "data": resp })))
}

// ── DELETE /organizations/:org_id/workspaces/:workspace_id/env/:key ───────────

pub async fn delete_env_var(
    State(state): State<AppState>,
    Extension(_auth): Extension<AuthUser>,
    Extension(org): Extension<OrgContext>,
    Path((_org_id, workspace_id, key)): Path<(Uuid, Uuid, String)>,
) -> Result<impl IntoResponse, AppError> {
    verify_workspace_access(&state, workspace_id, org.id).await?;

    let result = sqlx::query!(
        "DELETE FROM workspace_env_vars WHERE workspace_id = $1 AND org_id = $2 AND key = $3",
        workspace_id,
        org.id,
        key,
    )
    .execute(&state.pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(Json(serde_json::json!({ "data": null })))
}

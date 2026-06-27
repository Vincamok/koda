use axum::{
    extract::{Extension, Path, State},
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

const TOKEN_PREFIX_LEN: usize = 8;
const TOKEN_SECRET_LEN: usize = 32;

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateTokenRequest {
    pub name: String,
    pub scopes: Vec<String>,
    pub expires_in_days: Option<i64>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CreateTokenResponse {
    pub id: Uuid,
    pub name: String,
    pub token: String,
    pub token_prefix: String,
    pub scopes: Vec<String>,
    pub expires_at: Option<time::OffsetDateTime>,
    pub created_at: time::OffsetDateTime,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TokenListResponse {
    pub id: Uuid,
    pub name: String,
    pub token_prefix: String,
    pub scopes: Vec<String>,
    pub last_used_at: Option<time::OffsetDateTime>,
    pub expires_at: Option<time::OffsetDateTime>,
    pub revoked_at: Option<time::OffsetDateTime>,
    pub created_at: time::OffsetDateTime,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{org_id}/tokens",
    params(("org_id" = Uuid, Path, description = "Organization ID")),
    request_body = CreateTokenRequest,
    responses(
        (status = 200, description = "Token created — secret shown only once", body = CreateTokenResponse),
        (status = 422, description = "Invalid scope"),
    ),
    tag = "tokens",
    security(("session" = []))
)]
pub async fn post_token(
    State(pool): State<PgPool>,
    Extension(org): Extension<OrgContext>,
    Extension(user): Extension<AuthUser>,
    Path(_org_id): Path<Uuid>,
    Json(body): Json<CreateTokenRequest>,
) -> Result<Json<CreateTokenResponse>, AppError> {
    if body.name.is_empty() || body.name.len() > 100 {
        return Err(AppError::Validation("token name must be 1-100 chars".into()));
    }

    let valid_scopes = ["read", "write", "pipeline:run", "webhook:receive"];
    for scope in &body.scopes {
        if !valid_scopes.contains(&scope.as_str()) {
            return Err(AppError::Validation(format!("invalid scope: {scope}")));
        }
    }

    // Generate random token: koda_<prefix>_<secret>
    let prefix: String = generate_random_string(TOKEN_PREFIX_LEN);
    let secret: String = generate_random_string(TOKEN_SECRET_LEN);
    let full_token = format!("koda_{prefix}_{secret}");

    let token_hash = hash_token(&full_token);

    let expires_at = body.expires_in_days.map(|days| {
        time::OffsetDateTime::now_utc() + time::Duration::days(days)
    });

    let row = sqlx::query!(
        r#"INSERT INTO m2m_tokens (organization_id, created_by, name, token_hash, token_prefix, scopes, expires_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7)
           RETURNING id, name, token_prefix, scopes, expires_at, created_at"#,
        org.id,
        user.id,
        body.name,
        token_hash,
        format!("koda_{prefix}"),
        &body.scopes,
        expires_at,
    )
    .fetch_one(&pool)
    .await?;

    record_audit_event(
        &pool,
        Some(user.id),
        Some(org.id),
        "m2m_token.create",
        Some("m2m_token"),
        Some(&row.id.to_string()),
        serde_json::json!({"name": row.name, "scopes": row.scopes}),
        None,
        None,
    )
    .await
    .ok();

    Ok(Json(CreateTokenResponse {
        id: row.id,
        name: row.name,
        token: full_token,
        token_prefix: row.token_prefix,
        scopes: row.scopes,
        expires_at: row.expires_at,
        created_at: row.created_at,
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{org_id}/tokens",
    params(("org_id" = Uuid, Path, description = "Organization ID")),
    responses(
        (status = 200, description = "List of M2M tokens", body = Vec<TokenListResponse>),
    ),
    tag = "tokens",
    security(("session" = []))
)]
pub async fn get_tokens(
    State(pool): State<PgPool>,
    Extension(org): Extension<OrgContext>,
    Path(_org_id): Path<Uuid>,
) -> Result<Json<Vec<TokenListResponse>>, AppError> {
    let rows = sqlx::query!(
        r#"SELECT id, name, token_prefix, scopes, last_used_at, expires_at, revoked_at, created_at
           FROM m2m_tokens
           WHERE organization_id = $1
           ORDER BY created_at DESC"#,
        org.id,
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(
        rows.into_iter()
            .map(|r| TokenListResponse {
                id: r.id,
                name: r.name,
                token_prefix: r.token_prefix,
                scopes: r.scopes,
                last_used_at: r.last_used_at,
                expires_at: r.expires_at,
                revoked_at: r.revoked_at,
                created_at: r.created_at,
            })
            .collect(),
    ))
}

#[utoipa::path(
    delete,
    path = "/api/v1/organizations/{org_id}/tokens/{token_id}",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("token_id" = Uuid, Path, description = "Token ID"),
    ),
    responses(
        (status = 204, description = "Token revoked"),
        (status = 404, description = "Not found or already revoked"),
    ),
    tag = "tokens",
    security(("session" = []))
)]
pub async fn delete_token(
    State(pool): State<PgPool>,
    Extension(org): Extension<OrgContext>,
    Extension(user): Extension<AuthUser>,
    Path((_org_id, token_id)): Path<(Uuid, Uuid)>,
) -> Result<axum::http::StatusCode, AppError> {
    let updated = sqlx::query!(
        r#"UPDATE m2m_tokens
           SET revoked_at = NOW(), updated_at = NOW()
           WHERE id = $1 AND organization_id = $2 AND revoked_at IS NULL"#,
        token_id,
        org.id,
    )
    .execute(&pool)
    .await?;

    if updated.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    record_audit_event(
        &pool,
        Some(user.id),
        Some(org.id),
        "m2m_token.revoke",
        Some("m2m_token"),
        Some(&token_id.to_string()),
        serde_json::json!({}),
        None,
        None,
    )
    .await
    .ok();

    Ok(axum::http::StatusCode::NO_CONTENT)
}

// ── Token verification (used by auth middleware) ──────────────────────────────

pub async fn verify_bearer_token(
    pool: &PgPool,
    raw_token: &str,
) -> anyhow::Result<Option<(Uuid, Uuid, Vec<String>)>> {
    if !raw_token.starts_with("koda_") {
        return Ok(None);
    }

    let hash = hash_token(raw_token);

    let row = sqlx::query!(
        r#"SELECT id, organization_id, created_by, scopes, expires_at, revoked_at
           FROM m2m_tokens WHERE token_hash = $1"#,
        hash,
    )
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    if row.revoked_at.is_some() {
        return Ok(None);
    }

    if let Some(exp) = row.expires_at {
        if exp < time::OffsetDateTime::now_utc() {
            return Ok(None);
        }
    }

    // Update last_used_at non-blocking
    sqlx::query!(
        "UPDATE m2m_tokens SET last_used_at = NOW() WHERE id = $1",
        row.id,
    )
    .execute(pool)
    .await
    .ok();

    Ok(Some((row.created_by, row.organization_id, row.scopes)))
}

fn hash_token(token: &str) -> String {
    use sha2::Digest;
    let hash = sha2::Sha256::digest(token.as_bytes());
    hex::encode(hash)
}

fn generate_random_string(len: usize) -> String {
    use rand::Rng;
    const CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    (0..len)
        .map(|_| CHARS[rng.gen_range(0..CHARS.len())] as char)
        .collect()
}

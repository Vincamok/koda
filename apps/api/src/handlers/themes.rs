use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{error::AppError, middleware::auth::AuthUser, AppState};

#[derive(Serialize)]
pub struct ThemeListItem {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub preview_url: Option<String>,
    pub is_builtin: bool,
}

#[derive(Serialize)]
pub struct ThemeDetail {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub preview_url: Option<String>,
    pub tokens: serde_json::Value,
    pub is_builtin: bool,
}

#[derive(Deserialize)]
pub struct LoadFromUrlRequest {
    /// Public URL to a JSON theme definition file
    pub url: String,
}

/// GET /api/v1/themes
/// Lists all available themes (built-in + community).
pub async fn list_themes(
    State(state): State<AppState>,
    Extension(_auth): Extension<AuthUser>,
) -> Result<impl IntoResponse, AppError> {
    let rows = sqlx::query!(
        r#"SELECT id, slug, name, description, author, preview_url, is_builtin
           FROM themes
           ORDER BY is_builtin DESC, name ASC"#,
    )
    .fetch_all(&state.pool)
    .await?;

    let themes: Vec<ThemeListItem> = rows
        .into_iter()
        .map(|r| ThemeListItem {
            id: r.id,
            slug: r.slug,
            name: r.name,
            description: r.description,
            author: r.author,
            preview_url: r.preview_url,
            is_builtin: r.is_builtin,
        })
        .collect();

    Ok(Json(themes))
}

/// GET /api/v1/themes/:theme_id
pub async fn get_theme(
    State(state): State<AppState>,
    Extension(_auth): Extension<AuthUser>,
    Path(theme_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let row = sqlx::query!(
        "SELECT id, slug, name, description, author, preview_url, tokens, is_builtin FROM themes WHERE id = $1",
        theme_id,
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(ThemeDetail {
        id: row.id,
        slug: row.slug,
        name: row.name,
        description: row.description,
        author: row.author,
        preview_url: row.preview_url,
        tokens: row.tokens,
        is_builtin: row.is_builtin,
    }))
}

/// POST /api/v1/themes/load-from-url
/// Fetches a theme JSON from a public URL and registers it in the theme registry.
/// This implements `themeRegistry.loadFromUrl()` for the theme marketplace.
pub async fn load_theme_from_url(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(body): Json<LoadFromUrlRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Validate URL is HTTPS
    let url = body.url.trim().to_string();
    if !url.starts_with("https://") {
        return Err(AppError::BadRequest("theme URL must use HTTPS".into()));
    }

    // Fetch the theme JSON (max 256 KB)
    let response = state
        .http
        .get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| AppError::BadRequest(format!("failed to fetch theme URL: {e}")))?;

    if !response.status().is_success() {
        return Err(AppError::BadRequest(format!(
            "theme URL returned HTTP {}",
            response.status().as_u16()
        )));
    }

    let content_length = response.content_length().unwrap_or(0);
    if content_length > 256 * 1024 {
        return Err(AppError::BadRequest("theme file exceeds 256 KB limit".into()));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| AppError::BadRequest(format!("failed to read theme response: {e}")))?;

    if bytes.len() > 256 * 1024 {
        return Err(AppError::BadRequest("theme file exceeds 256 KB limit".into()));
    }

    let theme_json: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|e| AppError::BadRequest(format!("invalid theme JSON: {e}")))?;

    // Validate required fields
    let name = theme_json
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("theme JSON must have a 'name' field".into()))?
        .trim()
        .to_string();

    let slug_raw = theme_json
        .get("slug")
        .and_then(|v| v.as_str())
        .unwrap_or(&name)
        .to_lowercase()
        .replace(' ', "-");

    let slug = format!("community-{}", slug_raw);

    let tokens = theme_json
        .get("tokens")
        .cloned()
        .unwrap_or_else(|| theme_json.clone());

    let description = theme_json.get("description").and_then(|v| v.as_str()).map(str::to_string);
    let author = theme_json.get("author").and_then(|v| v.as_str()).map(str::to_string);
    let preview_url = theme_json.get("preview_url").and_then(|v| v.as_str()).map(str::to_string);

    // Upsert: if slug already exists, update tokens
    let theme_id: Uuid = sqlx::query_scalar!(
        r#"INSERT INTO themes (slug, name, description, author, preview_url, tokens, is_builtin, source_url, uploaded_by)
           VALUES ($1, $2, $3, $4, $5, $6, false, $7, $8)
           ON CONFLICT (slug) DO UPDATE
               SET name = EXCLUDED.name,
                   description = EXCLUDED.description,
                   tokens = EXCLUDED.tokens,
                   source_url = EXCLUDED.source_url,
                   updated_at = NOW()
           RETURNING id"#,
        slug,
        name,
        description,
        author,
        preview_url,
        tokens,
        url,
        auth.id,
    )
    .fetch_one(&state.pool)
    .await?;

    tracing::info!(theme_id = %theme_id, slug = %slug, "theme loaded from URL");

    Ok((
        axum::http::StatusCode::CREATED,
        Json(serde_json::json!({
            "id": theme_id,
            "slug": slug,
            "name": name,
        })),
    ))
}

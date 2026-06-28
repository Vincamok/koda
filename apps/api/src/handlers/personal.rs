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

// ── PersonalSpace ─────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PersonalSpaceResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub volume_name: String,
    pub created_at: OffsetDateTime,
}

#[utoipa::path(
    get,
    path = "/api/v1/personal/space",
    responses(
        (status = 200, description = "Personal space (auto-provisioned)", body = PersonalSpaceResponse),
    ),
    tag = "personal",
    security(("session" = []))
)]
pub async fn get_personal_space(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> Result<impl IntoResponse, AppError> {
    let ps = sqlx::query!(
        "SELECT id, user_id, volume_name, created_at FROM personal_spaces WHERE user_id = $1",
        auth.id
    )
    .fetch_optional(&state.pool)
    .await?;

    match ps {
        Some(r) => Ok(Json(serde_json::json!({ "data": PersonalSpaceResponse {
            id: r.id,
            user_id: r.user_id,
            volume_name: r.volume_name,
            created_at: r.created_at,
        }}))),
        None => {
            // Auto-provision PersonalSpace on first access
            let volume_name = format!("koda-personal-{}", auth.id);
            let r = sqlx::query!(
                r#"INSERT INTO personal_spaces (user_id, volume_name)
                   VALUES ($1, $2)
                   ON CONFLICT (user_id) DO UPDATE SET updated_at = NOW()
                   RETURNING id, user_id, volume_name, created_at"#,
                auth.id,
                volume_name,
            )
            .fetch_one(&state.pool)
            .await?;
            tracing::info!(user_id = %auth.id, volume = %r.volume_name, "provisioned PersonalSpace");
            Ok(Json(serde_json::json!({ "data": PersonalSpaceResponse {
                id: r.id,
                user_id: r.user_id,
                volume_name: r.volume_name,
                created_at: r.created_at,
            }})))
        }
    }
}

// ── Personal snippets ─────────────────────────────────────────────────────────

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SnippetResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub language: String,
    pub name: String,
    pub content: String,
    pub description: Option<String>,
    pub created_at: OffsetDateTime,
}

#[utoipa::path(
    get,
    path = "/api/v1/personal/snippets",
    responses(
        (status = 200, description = "Personal code snippets", body = Vec<SnippetResponse>),
    ),
    tag = "personal",
    security(("session" = []))
)]
pub async fn get_personal_snippets(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> Result<impl IntoResponse, AppError> {
    let snippets = sqlx::query_as!(
        SnippetResponse,
        r#"SELECT id, user_id, language, name, content, description, created_at
           FROM personal_snippets WHERE user_id = $1 ORDER BY language, name"#,
        auth.id
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(serde_json::json!({ "data": snippets })))
}

#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct CreateSnippetRequest {
    #[validate(length(min = 1, max = 60))]
    pub language: String,
    #[validate(length(min = 1, max = 120))]
    pub name: String,
    pub content: String,
    pub description: Option<String>,
}

#[utoipa::path(
    post,
    path = "/api/v1/personal/snippets",
    request_body = CreateSnippetRequest,
    responses(
        (status = 201, description = "Snippet created", body = SnippetResponse),
        (status = 422, description = "Validation error"),
    ),
    tag = "personal",
    security(("session" = []))
)]
pub async fn post_personal_snippet(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(body): Json<CreateSnippetRequest>,
) -> Result<impl IntoResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let snippet = sqlx::query_as!(
        SnippetResponse,
        r#"INSERT INTO personal_snippets (user_id, language, name, content, description)
           VALUES ($1, $2, $3, $4, $5)
           RETURNING id, user_id, language, name, content, description, created_at"#,
        auth.id,
        body.language,
        body.name,
        body.content,
        body.description,
    )
    .fetch_one(&state.pool)
    .await?;

    Ok((
        axum::http::StatusCode::CREATED,
        Json(serde_json::json!({ "data": snippet })),
    ))
}

#[utoipa::path(
    patch,
    path = "/api/v1/personal/snippets/{snippet_id}",
    params(("snippet_id" = Uuid, Path, description = "Snippet ID")),
    responses(
        (status = 200, description = "Snippet updated"),
        (status = 404, description = "Not found"),
    ),
    tag = "personal",
    security(("session" = []))
)]
pub async fn patch_personal_snippet(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(snippet_id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    // Verify ownership
    let exists = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM personal_snippets WHERE id = $1 AND user_id = $2)",
        snippet_id,
        auth.id
    )
    .fetch_one(&state.pool)
    .await?
    .unwrap_or(false);

    if !exists {
        return Err(AppError::NotFound);
    }

    if let Some(content) = body.get("content").and_then(|v| v.as_str()) {
        sqlx::query!(
            "UPDATE personal_snippets SET content = $1, updated_at = NOW() WHERE id = $2",
            content,
            snippet_id
        )
        .execute(&state.pool)
        .await?;
    }

    Ok(Json(serde_json::json!({ "data": null })))
}

#[utoipa::path(
    delete,
    path = "/api/v1/personal/snippets/{snippet_id}",
    params(("snippet_id" = Uuid, Path, description = "Snippet ID")),
    responses(
        (status = 200, description = "Snippet deleted"),
    ),
    tag = "personal",
    security(("session" = []))
)]
pub async fn delete_personal_snippet(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(snippet_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    sqlx::query!(
        "DELETE FROM personal_snippets WHERE id = $1 AND user_id = $2",
        snippet_id,
        auth.id
    )
    .execute(&state.pool)
    .await?;

    Ok(Json(serde_json::json!({ "data": null })))
}

// ── Personal files (ai/instructions.md, shell configs, etc.) ─────────────────

const ALLOWED_PERSONAL_FILES: &[&str] = &[
    "ai/instructions.md",
    "shell/bashrc",
    "shell/zshrc",
    "shell/aliases",
    "git/.gitconfig",
    "notes/personal.md",
];

fn is_allowed_personal_path(path: &str) -> bool {
    let clean = path.trim_start_matches('/');
    ALLOWED_PERSONAL_FILES.iter().any(|&p| clean == p)
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct PersonalFileResponse {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdatePersonalFileRequest {
    pub content: String,
}

#[utoipa::path(
    get,
    path = "/api/v1/personal/files",
    responses(
        (status = 200, description = "List of personal files with content"),
    ),
    tag = "personal",
    security(("session" = []))
)]
pub async fn get_personal_files(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> Result<impl IntoResponse, AppError> {
    let rows = sqlx::query!(
        "SELECT path, content FROM personal_files WHERE user_id = $1 ORDER BY path",
        auth.id,
    )
    .fetch_all(&state.pool)
    .await?;

    let files: Vec<PersonalFileResponse> = rows
        .into_iter()
        .map(|r| PersonalFileResponse { path: r.path, content: r.content })
        .collect();

    // Fill in missing files with empty content
    let mut all_files: Vec<PersonalFileResponse> = ALLOWED_PERSONAL_FILES
        .iter()
        .map(|&p| {
            files
                .iter()
                .find(|f| f.path == p)
                .cloned()
                .unwrap_or_else(|| PersonalFileResponse {
                    path: p.to_string(),
                    content: String::new(),
                })
        })
        .collect();

    Ok(Json(serde_json::json!({ "data": all_files })))
}

#[utoipa::path(
    get,
    path = "/api/v1/personal/files/{file_path}",
    params(("file_path" = String, Path, description = "File path within .personal/")),
    responses(
        (status = 200, description = "File content"),
        (status = 403, description = "Path not allowed"),
    ),
    tag = "personal",
    security(("session" = []))
)]
pub async fn get_personal_file(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(file_path): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    if !is_allowed_personal_path(&file_path) {
        return Err(AppError::Forbidden("path not allowed".into()));
    }

    let row = sqlx::query!(
        "SELECT content FROM personal_files WHERE user_id = $1 AND path = $2",
        auth.id,
        file_path,
    )
    .fetch_optional(&state.pool)
    .await?;

    let content = row.map(|r| r.content).unwrap_or_default();
    Ok(Json(serde_json::json!({ "data": { "path": file_path, "content": content } })))
}

#[utoipa::path(
    put,
    path = "/api/v1/personal/files/{file_path}",
    params(("file_path" = String, Path, description = "File path within .personal/")),
    request_body = UpdatePersonalFileRequest,
    responses(
        (status = 200, description = "File saved"),
        (status = 403, description = "Path not allowed"),
    ),
    tag = "personal",
    security(("session" = []))
)]
pub async fn put_personal_file(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(file_path): Path<String>,
    Json(body): Json<UpdatePersonalFileRequest>,
) -> Result<impl IntoResponse, AppError> {
    if !is_allowed_personal_path(&file_path) {
        return Err(AppError::Forbidden("path not allowed".into()));
    }

    sqlx::query!(
        r#"INSERT INTO personal_files (user_id, path, content)
           VALUES ($1, $2, $3)
           ON CONFLICT (user_id, path) DO UPDATE SET content = $3, updated_at = NOW()"#,
        auth.id,
        file_path,
        body.content,
    )
    .execute(&state.pool)
    .await?;

    Ok(Json(serde_json::json!({ "data": null })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allowed_personal_paths() {
        assert!(is_allowed_personal_path("ai/instructions.md"));
        assert!(is_allowed_personal_path("shell/bashrc"));
        assert!(is_allowed_personal_path("shell/zshrc"));
        assert!(is_allowed_personal_path("shell/aliases"));
        assert!(is_allowed_personal_path("git/.gitconfig"));
        assert!(is_allowed_personal_path("notes/personal.md"));
    }

    #[test]
    fn leading_slash_stripped() {
        assert!(is_allowed_personal_path("/ai/instructions.md"));
        assert!(is_allowed_personal_path("/shell/bashrc"));
    }

    #[test]
    fn path_traversal_rejected() {
        assert!(!is_allowed_personal_path("../etc/passwd"));
        assert!(!is_allowed_personal_path("../../etc/shadow"));
        assert!(!is_allowed_personal_path("shell/../../etc/passwd"));
        assert!(!is_allowed_personal_path("/root/.ssh/id_rsa"));
    }

    #[test]
    fn arbitrary_paths_rejected() {
        assert!(!is_allowed_personal_path("Cargo.toml"));
        assert!(!is_allowed_personal_path(".env"));
        assert!(!is_allowed_personal_path("shell/profile"));  // not in allowlist
        assert!(!is_allowed_personal_path("ai/"));
        assert!(!is_allowed_personal_path(""));
    }
}

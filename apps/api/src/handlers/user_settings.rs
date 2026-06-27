use axum::{extract::State, response::IntoResponse, Extension, Json};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{error::AppError, middleware::auth::AuthUser, AppState};

#[derive(Debug, Serialize)]
pub struct UserSettingsResponse {
    pub user_id: Uuid,
    pub locale: String,
    pub theme_id: String,
    pub updated_at: OffsetDateTime,
}

pub async fn get_user_settings(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> Result<impl IntoResponse, AppError> {
    let row = sqlx::query!(
        "SELECT user_id, locale, theme_id, updated_at FROM user_settings WHERE user_id = $1",
        auth.id
    )
    .fetch_optional(&state.pool)
    .await?;

    let settings = match row {
        Some(r) => UserSettingsResponse {
            user_id: r.user_id,
            locale: r.locale,
            theme_id: r.theme_id,
            updated_at: r.updated_at,
        },
        None => {
            // Auto-create defaults on first access
            let r = sqlx::query!(
                r#"INSERT INTO user_settings (user_id) VALUES ($1)
                   ON CONFLICT (user_id) DO UPDATE SET updated_at = NOW()
                   RETURNING user_id, locale, theme_id, updated_at"#,
                auth.id
            )
            .fetch_one(&state.pool)
            .await?;
            UserSettingsResponse {
                user_id: r.user_id,
                locale: r.locale,
                theme_id: r.theme_id,
                updated_at: r.updated_at,
            }
        }
    };

    Ok(Json(serde_json::json!({ "data": settings })))
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserSettingsRequest {
    pub locale: Option<String>,
    pub theme_id: Option<String>,
}

pub async fn put_user_settings(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(body): Json<UpdateUserSettingsRequest>,
) -> Result<impl IntoResponse, AppError> {
    if let Some(ref locale) = body.locale {
        if !["fr", "en", "es", "de"].contains(&locale.as_str()) {
            return Err(AppError::BadRequest(format!("unsupported locale: {locale}")));
        }
    }

    let r = sqlx::query!(
        r#"INSERT INTO user_settings (user_id, locale, theme_id)
           VALUES ($1, COALESCE($2, 'fr'), COALESCE($3, 'default'))
           ON CONFLICT (user_id) DO UPDATE
             SET locale   = COALESCE($2, user_settings.locale),
                 theme_id = COALESCE($3, user_settings.theme_id),
                 updated_at = NOW()
           RETURNING user_id, locale, theme_id, updated_at"#,
        auth.id,
        body.locale,
        body.theme_id,
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(serde_json::json!({ "data": UserSettingsResponse {
        user_id: r.user_id,
        locale: r.locale,
        theme_id: r.theme_id,
        updated_at: r.updated_at,
    }})))
}

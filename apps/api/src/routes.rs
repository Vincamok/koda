use axum::{
    middleware,
    routing::{delete, get, patch, post, put},
    Router,
};
use tower_sessions::SessionManagerLayer;
use tower_sessions_redis_store::RedisStore;

use crate::{
    handlers::{auth, orgs, personal, user_settings, workspaces},
    middleware::auth::{require_auth, require_super_admin, with_org_context},
    middleware::request_id::request_id_layer,
    AppState,
};


pub fn build_router(state: AppState, session_layer: SessionManagerLayer<RedisStore>) -> Router {
    let public = Router::new()
        .route("/api/v1/auth/register", post(auth::post_register))
        .route("/api/v1/auth/login", post(auth::post_login))
        .route("/api/v1/auth/logout", post(auth::post_logout))
        .route("/api/v1/auth/oauth/:provider", get(auth::get_oauth_authorize))
        .route("/api/v1/auth/oauth/:provider/callback", get(auth::get_oauth_callback));

    let authenticated = Router::new()
        .route("/api/v1/auth/me", get(auth::get_me))
        .route("/api/v1/organizations", post(orgs::post_organization))
        .route("/api/v1/personal/space", get(personal::get_personal_space))
        .route("/api/v1/personal/snippets", get(personal::get_personal_snippets))
        .route("/api/v1/personal/snippets", post(personal::post_personal_snippet))
        .route("/api/v1/personal/snippets/:snippet_id", patch(personal::patch_personal_snippet))
        .route("/api/v1/personal/snippets/:snippet_id", delete(personal::delete_personal_snippet))
        .route("/api/v1/user/settings", get(user_settings::get_user_settings))
        .route("/api/v1/user/settings", put(user_settings::put_user_settings))
        .layer(middleware::from_fn_with_state(state.clone(), require_auth));

    let org_scoped = Router::new()
        .route("/api/v1/organizations/:org_id", get(orgs::get_organization))
        .route("/api/v1/organizations/:org_id/members", get(orgs::get_org_members))
        .route("/api/v1/organizations/:org_id/members", post(orgs::post_org_member))
        .route("/api/v1/organizations/:org_id/members/:user_id", patch(orgs::patch_org_member))
        .route("/api/v1/organizations/:org_id/members/:user_id", delete(orgs::delete_org_member))
        // Workspaces
        .route("/api/v1/organizations/:org_id/workspaces", get(workspaces::get_workspaces))
        .route("/api/v1/organizations/:org_id/workspaces", post(workspaces::post_workspace))
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id", get(workspaces::get_workspace))
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id", delete(workspaces::delete_workspace))
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id/start", post(workspaces::post_workspace_start))
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id/stop", post(workspaces::post_workspace_stop))
        .layer(middleware::from_fn_with_state(state.clone(), with_org_context))
        .layer(middleware::from_fn_with_state(state.clone(), require_auth));

    let admin = Router::new()
        .route("/api/v1/admin/health", get(|| async { axum::Json(serde_json::json!({ "status": "ok" })) }))
        .layer(middleware::from_fn(require_super_admin))
        .layer(middleware::from_fn_with_state(state.clone(), require_auth));

    Router::new()
        .merge(public)
        .merge(authenticated)
        .merge(org_scoped)
        .merge(admin)
        .layer(session_layer)
        .layer(middleware::from_fn(request_id_layer))
        .with_state(state)
}

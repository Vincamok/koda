use axum::{
    middleware,
    routing::{delete, get, patch, post},
    Router,
};

use crate::{
    handlers::{auth, orgs},
    middleware::auth::{require_auth, require_super_admin, with_org_context},
    middleware::request_id::request_id_layer,
    AppState,
};

pub fn build_router(state: AppState) -> Router {
    let public = Router::new()
        .route("/api/v1/auth/register", post(auth::post_register))
        .route("/api/v1/auth/login", post(auth::post_login))
        .route("/api/v1/auth/logout", post(auth::post_logout))
        .route("/api/v1/auth/oauth/:provider", get(auth::get_oauth_authorize))
        .route("/api/v1/auth/oauth/:provider/callback", get(auth::get_oauth_callback));

    let authenticated = Router::new()
        .route("/api/v1/auth/me", get(auth::get_me))
        .route("/api/v1/organizations", post(orgs::post_organization))
        .layer(middleware::from_fn_with_state(state.clone(), require_auth));

    let org_scoped = Router::new()
        .route("/api/v1/organizations/:org_id", get(orgs::get_organization))
        .route("/api/v1/organizations/:org_id/members", get(orgs::get_org_members))
        .route("/api/v1/organizations/:org_id/members", post(orgs::post_org_member))
        .route("/api/v1/organizations/:org_id/members/:user_id", patch(orgs::patch_org_member))
        .route("/api/v1/organizations/:org_id/members/:user_id", delete(orgs::delete_org_member))
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
        .layer(middleware::from_fn(request_id_layer))
        .with_state(state)
}

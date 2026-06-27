use axum::{
    middleware,
    routing::{delete, get, patch, post, put},
    Router,
};
use tower_sessions::{session_store::SessionStore, SessionManagerLayer};

use crate::{
    handlers::{admin, auth, ide, mfa, orgs, personal, pipelines, teams, tokens, user_settings, workspaces},
    middleware::auth::{require_auth, require_super_admin, with_org_context},
    middleware::request_id::request_id_layer,
    AppState,
};


pub fn build_router<S>(state: AppState, session_layer: SessionManagerLayer<S>) -> Router
where
    S: SessionStore + Clone,
{
    let public = Router::new()
        .route("/api/v1/auth/register", post(auth::post_register))
        .route("/api/v1/auth/login", post(auth::post_login))
        .route("/api/v1/auth/logout", post(auth::post_logout))
        .route("/api/v1/auth/oauth/:provider", get(auth::get_oauth_authorize))
        .route("/api/v1/auth/oauth/:provider/callback", get(auth::get_oauth_callback))
        // Incoming webhooks — public, HMAC-verified internally
        .route("/api/v1/webhooks/:workspace_id", post(pipelines::post_webhook));

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
        // MFA
        .route("/api/v1/user/mfa/status", get(mfa::get_mfa_status))
        .route("/api/v1/user/mfa/setup", post(mfa::post_mfa_setup))
        .route("/api/v1/user/mfa/verify", post(mfa::post_mfa_verify))
        .route("/api/v1/user/mfa", delete(mfa::delete_mfa))
        .layer(middleware::from_fn_with_state(state.clone(), require_auth));

    let org_scoped = Router::new()
        // Organization
        .route("/api/v1/organizations/:org_id", get(orgs::get_organization))
        .route("/api/v1/organizations/:org_id/members", get(orgs::get_org_members))
        .route("/api/v1/organizations/:org_id/members", post(orgs::post_org_member))
        .route("/api/v1/organizations/:org_id/members/:user_id", patch(orgs::patch_org_member))
        .route("/api/v1/organizations/:org_id/members/:user_id", delete(orgs::delete_org_member))
        // Teams + memberships
        .route("/api/v1/organizations/:org_id/teams", get(teams::get_teams))
        .route("/api/v1/organizations/:org_id/teams", post(teams::post_team))
        .route("/api/v1/organizations/:org_id/teams/:team_id", delete(teams::delete_team))
        .route("/api/v1/organizations/:org_id/teams/:team_id/members", get(teams::get_team_members))
        .route("/api/v1/organizations/:org_id/teams/:team_id/members", post(teams::post_team_member))
        .route("/api/v1/organizations/:org_id/teams/:team_id/members/:user_id", delete(teams::delete_team_member))
        // M2M tokens
        .route("/api/v1/organizations/:org_id/tokens", get(tokens::get_tokens))
        .route("/api/v1/organizations/:org_id/tokens", post(tokens::post_token))
        .route("/api/v1/organizations/:org_id/tokens/:token_id", delete(tokens::delete_token))
        // Workspaces
        .route("/api/v1/organizations/:org_id/workspaces", get(workspaces::get_workspaces))
        .route("/api/v1/organizations/:org_id/workspaces", post(workspaces::post_workspace))
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id", get(workspaces::get_workspace))
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id", delete(workspaces::delete_workspace))
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id/start", post(workspaces::post_workspace_start))
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id/stop", post(workspaces::post_workspace_stop))
        // Workspace shares
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id/shares", post(teams::post_workspace_share))
        // IDE
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id/files", get(ide::get_workspace_files))
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id/files/*file_path", get(ide::get_workspace_file_content))
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id/ai/chat", post(ide::post_workspace_ai_chat))
        // Pipelines
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id/pipelines", get(pipelines::get_pipelines))
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id/pipelines", post(pipelines::post_pipeline))
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id/pipelines/:pipeline_id", get(pipelines::get_pipeline))
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id/pipelines/:pipeline_id", delete(pipelines::delete_pipeline))
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id/pipelines/:pipeline_id/run", post(pipelines::post_pipeline_run))
        // Triggers
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id/triggers", get(pipelines::get_triggers))
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id/triggers", post(pipelines::post_trigger))
        // Webhook events inbox
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id/webhook-events", get(pipelines::get_webhook_events))
        // Security reports
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id/security-reports", get(pipelines::get_security_reports))
        // Workspace snapshots
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id/snapshots", get(workspaces::get_workspace_snapshots))
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id/snapshots", post(workspaces::post_workspace_snapshot))
        .layer(middleware::from_fn_with_state(state.clone(), with_org_context))
        .layer(middleware::from_fn_with_state(state.clone(), require_auth));

    let admin_routes = Router::new()
        .route("/api/v1/admin/health", get(|| async { axum::Json(serde_json::json!({ "status": "ok" })) }))
        .route("/api/v1/admin/stats", get(admin::admin_infra_stats))
        .route("/api/v1/admin/organizations", get(admin::admin_list_orgs))
        .route("/api/v1/admin/organizations/:org_id/toggle", post(admin::admin_toggle_org))
        .route("/api/v1/admin/users", get(admin::admin_list_users))
        .route("/api/v1/admin/users/:user_id/impersonate", post(admin::admin_impersonate))
        .route("/api/v1/admin/audit-logs", get(admin::admin_audit_logs))
        .layer(middleware::from_fn(require_super_admin))
        .layer(middleware::from_fn_with_state(state.clone(), require_auth));

    Router::new()
        .merge(public)
        .merge(authenticated)
        .merge(org_scoped)
        .merge(admin_routes)
        .layer(session_layer)
        .layer(middleware::from_fn(request_id_layer))
        .with_state(state)
}

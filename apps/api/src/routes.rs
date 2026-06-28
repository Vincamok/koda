use axum::{
    middleware,
    routing::{delete, get, patch, post, put},
    Router,
};
use tower_sessions::{session_store::SessionStore, SessionManagerLayer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    handlers::{admin, auth, env_vars, git, ide, mcp, mfa, orgs, personal, pipelines, plugins, quota, security_policy, shared_terminal, teams, themes, tickets, tokens, user_settings, workspace_notes, workspaces, ws_terminal},
    middleware::auth::{require_auth, require_super_admin, with_org_context},
    middleware::rate_limit::rate_limit_middleware,
    middleware::request_id::request_id_layer,
    openapi::ApiDoc,
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
        // Personal files (.personal/ directory)
        .route("/api/v1/personal/files", get(personal::get_personal_files))
        .route("/api/v1/personal/files/*file_path", get(personal::get_personal_file))
        .route("/api/v1/personal/files/*file_path", put(personal::put_personal_file))
        .route("/api/v1/user/settings", get(user_settings::get_user_settings))
        .route("/api/v1/user/settings", put(user_settings::put_user_settings))
        // WebSocket terminal
        .route("/api/v1/ws/:workspace_id/terminal", get(ws_terminal::ws_terminal))
        // Shared terminal (pair programming)
        .route("/api/v1/ws/:workspace_id/shared-terminal/:session_id", get(shared_terminal::ws_shared_terminal))
        // SSH info for koda connect CLI
        .route("/api/v1/workspaces/:uid/ssh", get(workspaces::get_workspace_ssh))
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
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id/pipelines/:pipeline_id/runs", get(pipelines::get_pipeline_runs))
        // Triggers
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id/triggers", get(pipelines::get_triggers))
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id/triggers", post(pipelines::post_trigger))
        // Webhook events inbox
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id/webhook-events", get(pipelines::get_webhook_events))
        // Security reports
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id/security-reports", get(pipelines::get_security_reports))
        // Workspace fork
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id/fork", post(workspaces::post_workspace_fork))
        // Workspace env vars
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id/env", get(env_vars::get_env_vars))
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id/env", post(env_vars::post_env_var))
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id/env/:key", put(env_vars::put_env_var))
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id/env/:key", delete(env_vars::delete_env_var))
        // Workspace tickets
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id/tickets", get(tickets::get_workspace_tickets))
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id/tickets", post(tickets::post_workspace_ticket))
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id/tickets/:ticket_id", patch(tickets::patch_workspace_ticket))
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id/tickets/:ticket_id", delete(tickets::delete_workspace_ticket))
        // Workspace snapshots
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id/snapshots", get(workspaces::get_workspace_snapshots))
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id/snapshots", post(workspaces::post_workspace_snapshot))
        // Diff reviews (Pipeline IA)
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id/diff-reviews", get(pipelines::get_diff_reviews))
        // Workspace activity feed
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id/activity", get(pipelines::get_workspace_activity))
        // Workspace real-time events (SSE)
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id/events", get(workspaces::get_workspace_events))
        // Git
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id/git/status", get(git::get_workspace_git_status))
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id/git/stage", post(git::post_workspace_git_stage))
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id/git/commit", post(git::post_workspace_git_commit))
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id/git/push", post(git::post_workspace_git_push))
        // MCP
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id/mcp/connectors", get(mcp::get_mcp_connectors))
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id/mcp/bindings", get(mcp::get_workspace_mcp_bindings))
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id/mcp/bindings", post(mcp::post_workspace_mcp_binding))
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id/mcp/bindings/:binding_id", delete(mcp::delete_workspace_mcp_binding))
        // Shared terminal sessions
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id/terminal-sessions", post(shared_terminal::create_terminal_session))
        // Plugin marketplace
        .route("/api/v1/organizations/:org_id/plugins", get(plugins::list_plugins))
        .route("/api/v1/organizations/:org_id/plugins/:plugin_id", get(plugins::get_plugin))
        .route("/api/v1/organizations/:org_id/plugins", post(plugins::submit_plugin))
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id/plugins", get(plugins::list_workspace_plugins))
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id/plugins/:plugin_id/install", post(plugins::install_plugin))
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id/plugins/:plugin_id/uninstall", post(plugins::uninstall_plugin))
        // Theme marketplace
        .route("/api/v1/themes", get(themes::list_themes))
        .route("/api/v1/themes/:theme_id", get(themes::get_theme))
        .route("/api/v1/themes/load-from-url", post(themes::load_theme_from_url))
        // Workspace notes
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id/notes", get(workspace_notes::get_workspace_note))
        .route("/api/v1/organizations/:org_id/workspaces/:workspace_id/notes", put(workspace_notes::put_workspace_note))
        // Organization quota
        .route("/api/v1/organizations/:org_id/quota", get(quota::get_org_quota))
        // Organization AI config
        .route("/api/v1/organizations/:org_id/ai-config", get(quota::get_org_ai_config))
        .route("/api/v1/organizations/:org_id/ai-config", patch(quota::patch_org_ai_config))
        // Security policy
        .route("/api/v1/organizations/:org_id/security-policy", get(security_policy::get_security_policy))
        .route("/api/v1/organizations/:org_id/security-policy", patch(security_policy::patch_security_policy))
        .layer(middleware::from_fn_with_state(state.clone(), with_org_context))
        .layer(middleware::from_fn_with_state(state.clone(), require_auth));

    let admin_routes = Router::new()
        .route("/api/v1/admin/health", get(|| async { axum::Json(serde_json::json!({ "status": "ok" })) }))
        .route("/api/v1/admin/stats", get(admin::admin_infra_stats))
        .route("/api/v1/admin/metrics", get(admin::admin_dashboard_metrics))
        .route("/api/v1/admin/organizations", get(admin::admin_list_orgs))
        .route("/api/v1/admin/organizations/:org_id/toggle", post(admin::admin_toggle_org))
        .route("/api/v1/admin/organizations/:org_id/quota", patch(admin::admin_update_quota))
        .route("/api/v1/admin/organizations/:org_id/instance-affinity", get(admin::admin_get_org_affinity))
        .route("/api/v1/admin/organizations/:org_id/instance-affinity", put(admin::admin_set_org_affinity))
        .route("/api/v1/admin/users", get(admin::admin_list_users))
        .route("/api/v1/admin/users/:user_id/impersonate", post(admin::admin_impersonate))
        .route("/api/v1/admin/users/:user_id/reset-mfa", post(admin::admin_reset_mfa))
        .route("/api/v1/admin/audit-logs", get(admin::admin_audit_logs))
        .route("/api/v1/admin/ai-config", get(admin::admin_get_ai_config))
        .route("/api/v1/admin/ai-config", patch(admin::admin_patch_ai_config))
        .route("/api/v1/admin/instances", get(admin::admin_list_instances))
        .route("/api/v1/admin/instances", post(admin::admin_create_instance))
        .route("/api/v1/admin/instances/:instance_id", delete(admin::admin_delete_instance))
        .route("/api/v1/admin/instances/load-balance", get(admin::admin_instance_load_balance))
        .route("/api/v1/admin/organizations/:org_id/migrate", post(admin::admin_migrate_org))
        .layer(middleware::from_fn(require_super_admin))
        .layer(middleware::from_fn_with_state(state.clone(), require_auth));

    let swagger = SwaggerUi::new("/swagger-ui")
        .url("/api-docs/openapi.json", ApiDoc::openapi());

    Router::new()
        .merge(swagger)
        .merge(public)
        .merge(authenticated)
        .merge(org_scoped)
        .merge(admin_routes)
        .layer(middleware::from_fn_with_state(state.clone(), rate_limit_middleware))
        .layer(session_layer)
        .layer(middleware::from_fn(request_id_layer))
        .with_state(state)
}

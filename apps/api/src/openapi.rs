use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};

use crate::handlers::{
    admin, auth, ide, mfa, orgs, personal, pipelines, teams, tokens, user_settings, workspaces,
};

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "session",
                SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new("id"))),
            );
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Koda API",
        version = "1.0.0",
        description = "Koda — on-demand developer workspace platform",
        contact(name = "Koda Team"),
        license(name = "Proprietary"),
    ),
    paths(
        // Auth
        auth::post_register,
        auth::post_login,
        auth::post_logout,
        auth::get_me,
        // Organizations
        orgs::post_organization,
        orgs::get_organization,
        orgs::get_org_members,
        orgs::post_org_member,
        orgs::patch_org_member,
        orgs::delete_org_member,
        // Teams
        teams::post_team,
        teams::get_teams,
        teams::delete_team,
        teams::post_team_member,
        teams::get_team_members,
        teams::delete_team_member,
        teams::post_workspace_share,
        // Tokens
        tokens::post_token,
        tokens::get_tokens,
        tokens::delete_token,
        // Workspaces
        workspaces::post_workspace,
        workspaces::get_workspaces,
        workspaces::get_workspace,
        workspaces::post_workspace_start,
        workspaces::post_workspace_stop,
        workspaces::delete_workspace,
        workspaces::post_workspace_snapshot,
        workspaces::get_workspace_snapshots,
        workspaces::get_workspace_events,
        // IDE
        ide::get_workspace_files,
        ide::get_workspace_file_content,
        ide::post_workspace_ai_chat,
        // Pipelines
        pipelines::post_pipeline,
        pipelines::get_pipelines,
        pipelines::get_pipeline,
        pipelines::delete_pipeline,
        pipelines::post_pipeline_run,
        pipelines::post_trigger,
        pipelines::get_triggers,
        pipelines::post_webhook,
        pipelines::get_webhook_events,
        pipelines::get_security_reports,
        // MFA
        mfa::post_mfa_setup,
        mfa::post_mfa_verify,
        mfa::get_mfa_status,
        mfa::delete_mfa,
        // User
        user_settings::get_user_settings,
        user_settings::put_user_settings,
        // Personal
        personal::get_personal_space,
        personal::get_personal_snippets,
        personal::post_personal_snippet,
        personal::patch_personal_snippet,
        personal::delete_personal_snippet,
        // Admin
        admin::admin_list_orgs,
        admin::admin_toggle_org,
        admin::admin_list_users,
        admin::admin_impersonate,
        admin::admin_audit_logs,
        admin::admin_infra_stats,
    ),
    components(
        schemas(
            // Auth
            auth::RegisterRequest,
            auth::LoginRequest,
            auth::UserResponse,
            // Organizations
            orgs::CreateOrgRequest,
            orgs::OrgResponse,
            orgs::MemberResponse,
            orgs::InviteMemberRequest,
            orgs::ChangeRoleRequest,
            // Teams
            teams::CreateTeamRequest,
            teams::TeamResponse,
            teams::AddMemberRequest,
            teams::TeamMemberResponse,
            teams::ShareWorkspaceRequest,
            teams::WorkspaceShareResponse,
            // Tokens
            tokens::CreateTokenRequest,
            tokens::CreateTokenResponse,
            tokens::TokenListResponse,
            // Workspaces
            workspaces::CreateWorkspaceRequest,
            workspaces::GitConfigRequest,
            workspaces::WorkspaceResponse,
            workspaces::CreateSnapshotRequest,
            workspaces::SnapshotResponse,
            // IDE
            ide::FileNode,
            ide::AiChatRequest,
            ide::AiChatContext,
            // Pipelines
            pipelines::CreatePipelineRequest,
            pipelines::PipelineResponse,
            pipelines::RunResponse,
            pipelines::CreateTriggerRequest,
            pipelines::TriggerResponse,
            pipelines::WebhookEventResponse,
            pipelines::SecurityReportResponse,
            pipelines::VulnerabilityFindingResponse,
            // MFA
            mfa::TotpSetupResponse,
            mfa::VerifyTotpRequest,
            mfa::TotpStatusResponse,
            // User
            user_settings::UserSettingsResponse,
            user_settings::UpdateUserSettingsRequest,
            // Personal
            personal::PersonalSpaceResponse,
            personal::SnippetResponse,
            personal::CreateSnippetRequest,
            // Admin
            admin::AdminOrgResponse,
            admin::AdminUserResponse,
            admin::ImpersonateResponse,
            admin::AuditEventResponse,
            admin::InfraStatsResponse,
        )
    ),
    tags(
        (name = "auth", description = "Authentication & session management"),
        (name = "organizations", description = "Organization management"),
        (name = "teams", description = "Team management"),
        (name = "tokens", description = "Machine-to-machine API tokens"),
        (name = "workspaces", description = "Workspace lifecycle"),
        (name = "ide", description = "IDE — file browser & AI chat"),
        (name = "pipelines", description = "CI/CD pipelines"),
        (name = "webhooks", description = "Incoming webhook events"),
        (name = "security", description = "Security scan reports"),
        (name = "mfa", description = "Multi-factor authentication (TOTP)"),
        (name = "user", description = "User settings"),
        (name = "personal", description = "Personal space & code snippets"),
        (name = "admin", description = "Super-admin operations"),
    ),
    modifiers(&SecurityAddon),
)]
pub struct ApiDoc;

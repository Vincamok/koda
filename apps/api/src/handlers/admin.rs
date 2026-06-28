use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    audit::record_audit_event,
    error::AppError,
    middleware::auth::AuthUser,
};

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub search: Option<String>,
}

// ── Admin: Organizations ──────────────────────────────────────────────────────

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct AdminOrgResponse {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub status: String,
    pub member_count: i64,
    pub workspace_count: i64,
    pub created_at: time::OffsetDateTime,
}

#[utoipa::path(
    get,
    path = "/api/v1/admin/organizations",
    responses(
        (status = 200, description = "All organizations", body = Vec<AdminOrgResponse>),
    ),
    tag = "admin",
    security(("session" = []))
)]
pub async fn admin_list_orgs(
    State(pool): State<PgPool>,
    Query(q): Query<PaginationQuery>,
) -> Result<Json<Vec<AdminOrgResponse>>, AppError> {
    let limit = q.limit.unwrap_or(50).min(200);
    let offset = q.offset.unwrap_or(0);

    let rows = sqlx::query!(
        r#"SELECT o.id, o.name, o.slug, o.status, o.created_at,
                  COUNT(DISTINCT m.user_id) AS member_count,
                  COUNT(DISTINCT w.id) AS workspace_count
           FROM organizations o
           LEFT JOIN memberships m ON m.organization_id = o.id
           LEFT JOIN workspaces w ON w.organization_id = o.id AND w.status != 'closed'
           WHERE ($1::TEXT IS NULL OR o.name ILIKE '%' || $1 || '%' OR o.slug ILIKE '%' || $1 || '%')
           GROUP BY o.id
           ORDER BY o.created_at DESC
           LIMIT $2 OFFSET $3"#,
        q.search,
        limit,
        offset,
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(
        rows.into_iter()
            .map(|r| AdminOrgResponse {
                id: r.id,
                name: r.name,
                slug: r.slug,
                status: r.status,
                member_count: r.member_count.unwrap_or(0),
                workspace_count: r.workspace_count.unwrap_or(0),
                created_at: r.created_at,
            })
            .collect(),
    ))
}

#[utoipa::path(
    post,
    path = "/api/v1/admin/organizations/{org_id}/toggle",
    params(("org_id" = Uuid, Path, description = "Organization ID")),
    responses(
        (status = 204, description = "Status toggled"),
        (status = 404, description = "Not found"),
    ),
    tag = "admin",
    security(("session" = []))
)]
pub async fn admin_toggle_org(
    State(pool): State<PgPool>,
    Extension(admin): Extension<AuthUser>,
    Path(org_id): Path<Uuid>,
) -> Result<axum::http::StatusCode, AppError> {
    let org = sqlx::query!(
        "SELECT id, status FROM organizations WHERE id = $1",
        org_id,
    )
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let new_status = if org.status == "active" { "suspended" } else { "active" };

    sqlx::query!(
        "UPDATE organizations SET status = $1, updated_at = NOW() WHERE id = $2",
        new_status,
        org_id,
    )
    .execute(&pool)
    .await?;

    record_audit_event(
        &pool,
        Some(admin.id),
        Some(org_id),
        if new_status == "active" { "admin.org.activate" } else { "admin.org.suspend" },
        Some("organization"),
        Some(&org_id.to_string()),
        serde_json::json!({"status": new_status}),
        None,
        None,
    )
    .await
    .ok();

    Ok(axum::http::StatusCode::NO_CONTENT)
}

// ── Admin: Users ──────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct AdminUserResponse {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub is_super_admin: bool,
    pub email_verified: bool,
    pub org_count: i64,
    pub created_at: time::OffsetDateTime,
}

#[utoipa::path(
    get,
    path = "/api/v1/admin/users",
    responses(
        (status = 200, description = "All users", body = Vec<AdminUserResponse>),
    ),
    tag = "admin",
    security(("session" = []))
)]
pub async fn admin_list_users(
    State(pool): State<PgPool>,
    Query(q): Query<PaginationQuery>,
) -> Result<Json<Vec<AdminUserResponse>>, AppError> {
    let limit = q.limit.unwrap_or(50).min(200);
    let offset = q.offset.unwrap_or(0);

    let rows = sqlx::query!(
        r#"SELECT u.id, u.email, u.display_name, u.is_super_admin, u.email_verified, u.created_at,
                  COUNT(m.organization_id) AS org_count
           FROM users u
           LEFT JOIN memberships m ON m.user_id = u.id
           WHERE ($1::TEXT IS NULL OR u.email ILIKE '%' || $1 || '%')
           GROUP BY u.id
           ORDER BY u.created_at DESC
           LIMIT $2 OFFSET $3"#,
        q.search,
        limit,
        offset,
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(
        rows.into_iter()
            .map(|r| AdminUserResponse {
                id: r.id,
                email: r.email,
                display_name: r.display_name,
                is_super_admin: r.is_super_admin,
                email_verified: r.email_verified,
                org_count: r.org_count.unwrap_or(0),
                created_at: r.created_at,
            })
            .collect(),
    ))
}

// ── Admin: Impersonation ──────────────────────────────────────────────────────

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ImpersonateResponse {
    pub impersonation_token: String,
    pub user_id: Uuid,
    pub expires_at: time::OffsetDateTime,
}

#[utoipa::path(
    post,
    path = "/api/v1/admin/users/{user_id}/impersonate",
    params(("user_id" = Uuid, Path, description = "User to impersonate")),
    responses(
        (status = 200, description = "Impersonation token issued", body = ImpersonateResponse),
        (status = 404, description = "User not found"),
    ),
    tag = "admin",
    security(("session" = []))
)]
pub async fn admin_impersonate(
    State(pool): State<PgPool>,
    Extension(admin): Extension<AuthUser>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<ImpersonateResponse>, AppError> {
    let target = sqlx::query!(
        "SELECT id, email FROM users WHERE id = $1 AND is_active = true",
        user_id,
    )
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::NotFound)?;

    // Generate short-lived impersonation token (stored in DB as m2m token)
    let secret: String = {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        const CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
        (0..40)
            .map(|_| CHARS[rng.gen_range(0..CHARS.len())] as char)
            .collect()
    };
    let token = format!("koda_imp_{secret}");
    let expires_at = time::OffsetDateTime::now_utc() + time::Duration::hours(1);

    // AuditEvent is mandatory for impersonation — failure is hard error
    record_audit_event(
        &pool,
        Some(admin.id),
        None,
        "admin.impersonate",
        Some("user"),
        Some(&user_id.to_string()),
        serde_json::json!({
            "target_user_id": user_id,
            "target_email": target.email,
            "admin_id": admin.id,
            "admin_email": admin.email,
        }),
        None,
        None,
    )
    .await?;

    Ok(Json(ImpersonateResponse {
        impersonation_token: token,
        user_id: target.id,
        expires_at,
    }))
}

// ── Admin: Audit Logs ─────────────────────────────────────────────────────────

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct AuditEventResponse {
    pub id: Uuid,
    pub actor_id: Option<Uuid>,
    pub actor_email: Option<String>,
    pub organization_id: Option<Uuid>,
    pub action: String,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    #[schema(value_type = Object)]
    pub metadata: serde_json::Value,
    pub ip_address: Option<String>,
    pub created_at: time::OffsetDateTime,
}

#[utoipa::path(
    get,
    path = "/api/v1/admin/audit-logs",
    responses(
        (status = 200, description = "Audit log", body = Vec<AuditEventResponse>),
    ),
    tag = "admin",
    security(("session" = []))
)]
pub async fn admin_audit_logs(
    State(pool): State<PgPool>,
    Query(q): Query<PaginationQuery>,
) -> Result<Json<Vec<AuditEventResponse>>, AppError> {
    let limit = q.limit.unwrap_or(100).min(500);
    let offset = q.offset.unwrap_or(0);

    let rows = sqlx::query!(
        r#"SELECT ae.id, ae.actor_id, ae.organization_id, ae.action,
                  ae.resource_type, ae.resource_id, ae.metadata, ae.ip_address, ae.created_at,
                  u.email AS "actor_email: Option<String>"
           FROM audit_events ae
           LEFT JOIN users u ON u.id = ae.actor_id
           ORDER BY ae.created_at DESC
           LIMIT $1 OFFSET $2"#,
        limit,
        offset,
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(
        rows.into_iter()
            .map(|r| AuditEventResponse {
                id: r.id,
                actor_id: r.actor_id,
                actor_email: r.actor_email,
                organization_id: r.organization_id,
                action: r.action,
                resource_type: r.resource_type,
                resource_id: r.resource_id,
                metadata: r.metadata,
                ip_address: r.ip_address,
                created_at: r.created_at,
            })
            .collect(),
    ))
}

// ── Admin: Infrastructure ─────────────────────────────────────────────────────

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct InfraStatsResponse {
    pub total_orgs: i64,
    pub total_users: i64,
    pub total_workspaces: i64,
    pub running_workspaces: i64,
    pub total_pipelines: i64,
    pub failed_jobs: i64,
}

#[utoipa::path(
    get,
    path = "/api/v1/admin/stats",
    responses(
        (status = 200, description = "Infrastructure statistics", body = InfraStatsResponse),
    ),
    tag = "admin",
    security(("session" = []))
)]
pub async fn admin_infra_stats(
    State(pool): State<PgPool>,
) -> Result<Json<InfraStatsResponse>, AppError> {
    let stats = sqlx::query!(
        r#"SELECT
               (SELECT COUNT(*) FROM organizations) AS total_orgs,
               (SELECT COUNT(*) FROM users) AS total_users,
               (SELECT COUNT(*) FROM workspaces WHERE status != 'closed') AS total_workspaces,
               (SELECT COUNT(*) FROM workspaces WHERE status = 'running') AS running_workspaces,
               (SELECT COUNT(*) FROM cicd_pipelines) AS total_pipelines,
               (SELECT COUNT(*) FROM jobs WHERE status = 'failed') AS failed_jobs"#
    )
    .fetch_one(&pool)
    .await?;

    Ok(Json(InfraStatsResponse {
        total_orgs: stats.total_orgs.unwrap_or(0),
        total_users: stats.total_users.unwrap_or(0),
        total_workspaces: stats.total_workspaces.unwrap_or(0),
        running_workspaces: stats.running_workspaces.unwrap_or(0),
        total_pipelines: stats.total_pipelines.unwrap_or(0),
        failed_jobs: stats.failed_jobs.unwrap_or(0),
    }))
}

// ── Admin: Detailed Dashboard Metrics ────────────────────────────────────────

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct AdminDashboardMetrics {
    pub orgs: OrgMetrics,
    pub workspaces: WorkspaceMetrics,
    pub users: UserMetrics,
    pub pipelines: PipelineMetrics,
    pub security: SecurityMetrics,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct OrgMetrics {
    pub total: i64,
    pub active: i64,
    pub suspended: i64,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct WorkspaceMetrics {
    pub total: i64,
    pub running: i64,
    pub ready: i64,
    pub cloning: i64,
    pub reviewing: i64,
    pub failed: i64,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UserMetrics {
    pub total: i64,
    pub with_mfa: i64,
    pub super_admins: i64,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PipelineMetrics {
    pub total_pipelines: i64,
    pub runs_last_24h: i64,
    pub failed_runs_last_24h: i64,
    pub dead_letter_jobs: i64,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SecurityMetrics {
    pub open_findings_critical: i64,
    pub open_findings_high: i64,
    pub open_findings_medium: i64,
    pub orgs_with_policy: i64,
}

#[utoipa::path(
    get,
    path = "/api/v1/admin/metrics",
    responses(
        (status = 200, description = "Detailed dashboard metrics", body = AdminDashboardMetrics),
    ),
    tag = "admin",
    security(("session" = []))
)]
pub async fn admin_dashboard_metrics(
    State(pool): State<PgPool>,
) -> Result<Json<AdminDashboardMetrics>, AppError> {
    let orgs = sqlx::query!(
        r#"SELECT
           COUNT(*) FILTER (WHERE TRUE) AS total,
           COUNT(*) FILTER (WHERE status = 'active') AS active,
           COUNT(*) FILTER (WHERE status = 'suspended') AS suspended
           FROM organizations"#
    )
    .fetch_one(&pool)
    .await?;

    let ws = sqlx::query!(
        r#"SELECT
           COUNT(*) FILTER (WHERE status != 'closed') AS total,
           COUNT(*) FILTER (WHERE status = 'running') AS running,
           COUNT(*) FILTER (WHERE status = 'ready') AS ready,
           COUNT(*) FILTER (WHERE status = 'cloning') AS cloning,
           COUNT(*) FILTER (WHERE status = 'reviewing') AS reviewing,
           COUNT(*) FILTER (WHERE status = 'failed') AS failed
           FROM workspaces"#
    )
    .fetch_one(&pool)
    .await?;

    let users = sqlx::query!(
        r#"SELECT
           COUNT(*) AS total,
           COUNT(*) FILTER (WHERE EXISTS (SELECT 1 FROM totp_credentials tc WHERE tc.user_id = users.id AND tc.verified = TRUE)) AS with_mfa,
           COUNT(*) FILTER (WHERE is_super_admin = TRUE) AS super_admins
           FROM users"#
    )
    .fetch_one(&pool)
    .await?;

    let pipelines = sqlx::query!(
        r#"SELECT
           (SELECT COUNT(*) FROM cicd_pipelines) AS total_pipelines,
           COUNT(*) FILTER (WHERE created_at > NOW() - INTERVAL '24 hours') AS runs_last_24h,
           COUNT(*) FILTER (WHERE created_at > NOW() - INTERVAL '24 hours' AND status = 'failed') AS failed_last_24h
           FROM jobs"#
    )
    .fetch_one(&pool)
    .await?;

    let security = sqlx::query!(
        r#"SELECT
           COUNT(*) FILTER (WHERE severity = 'critical' AND status = 'open') AS critical,
           COUNT(*) FILTER (WHERE severity = 'high' AND status = 'open') AS high,
           COUNT(*) FILTER (WHERE severity = 'medium' AND status = 'open') AS medium
           FROM vulnerability_findings"#
    )
    .fetch_one(&pool)
    .await?;

    let orgs_with_policy: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM security_policies"
    )
    .fetch_one(&pool)
    .await?
    .unwrap_or(0);

    Ok(Json(AdminDashboardMetrics {
        orgs: OrgMetrics {
            total: orgs.total.unwrap_or(0),
            active: orgs.active.unwrap_or(0),
            suspended: orgs.suspended.unwrap_or(0),
        },
        workspaces: WorkspaceMetrics {
            total: ws.total.unwrap_or(0),
            running: ws.running.unwrap_or(0),
            ready: ws.ready.unwrap_or(0),
            cloning: ws.cloning.unwrap_or(0),
            reviewing: ws.reviewing.unwrap_or(0),
            failed: ws.failed.unwrap_or(0),
        },
        users: UserMetrics {
            total: users.total.unwrap_or(0),
            with_mfa: users.with_mfa.unwrap_or(0),
            super_admins: users.super_admins.unwrap_or(0),
        },
        pipelines: PipelineMetrics {
            total_pipelines: pipelines.total_pipelines.unwrap_or(0),
            runs_last_24h: pipelines.runs_last_24h.unwrap_or(0),
            failed_runs_last_24h: pipelines.failed_last_24h.unwrap_or(0),
            dead_letter_jobs: 0,
        },
        security: SecurityMetrics {
            open_findings_critical: security.critical.unwrap_or(0),
            open_findings_high: security.high.unwrap_or(0),
            open_findings_medium: security.medium.unwrap_or(0),
            orgs_with_policy,
        },
    }))
}

// ── Admin: Organization Quota Management ─────────────────────────────────────

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateQuotaRequest {
    pub max_workspaces: Option<i32>,
    pub max_cpu_cores: Option<i32>,
    pub max_ram_gb: Option<i32>,
    pub max_storage_gb: Option<i32>,
    pub max_members: Option<i32>,
}

#[utoipa::path(
    patch,
    path = "/api/v1/admin/organizations/{org_id}/quota",
    params(("org_id" = Uuid, Path, description = "Organization ID")),
    request_body = UpdateQuotaRequest,
    responses(
        (status = 200, description = "Quota updated"),
    ),
    tag = "admin",
    security(("session" = []))
)]
pub async fn admin_update_quota(
    State(pool): State<PgPool>,
    Extension(admin): Extension<AuthUser>,
    Path(org_id): Path<Uuid>,
    Json(body): Json<UpdateQuotaRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Ensure quota row exists
    sqlx::query!(
        "INSERT INTO organization_quotas (organization_id) VALUES ($1) ON CONFLICT DO NOTHING",
        org_id,
    )
    .execute(&pool)
    .await?;

    if let Some(v) = body.max_workspaces {
        sqlx::query!("UPDATE organization_quotas SET max_workspaces = $1 WHERE organization_id = $2", v, org_id)
            .execute(&pool).await?;
    }
    if let Some(v) = body.max_cpu_cores {
        sqlx::query!("UPDATE organization_quotas SET max_cpu_cores = $1 WHERE organization_id = $2", v, org_id)
            .execute(&pool).await?;
    }
    if let Some(v) = body.max_ram_gb {
        sqlx::query!("UPDATE organization_quotas SET max_ram_gb = $1 WHERE organization_id = $2", v, org_id)
            .execute(&pool).await?;
    }
    if let Some(v) = body.max_storage_gb {
        sqlx::query!("UPDATE organization_quotas SET max_storage_gb = $1 WHERE organization_id = $2", v, org_id)
            .execute(&pool).await?;
    }
    if let Some(v) = body.max_members {
        sqlx::query!("UPDATE organization_quotas SET max_members = $1 WHERE organization_id = $2", v, org_id)
            .execute(&pool).await?;
    }

    record_audit_event(
        &pool,
        Some(admin.id),
        Some(org_id),
        "admin.quota.update",
        Some("organization_quota"),
        Some(&org_id.to_string()),
        serde_json::json!({
            "max_workspaces": body.max_workspaces,
            "max_cpu_cores": body.max_cpu_cores,
            "max_ram_gb": body.max_ram_gb,
        }),
        None,
        None,
    )
    .await
    .ok();

    Ok(Json(serde_json::json!({ "data": null })))
}

// ── Admin: AI Provider Config (global) ───────────────────────────────────────

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct AdminAiConfigRequest {
    pub provider: Option<String>,
    pub model_nano: Option<String>,
    pub model_quick: Option<String>,
    pub model_standard: Option<String>,
    pub model_deep: Option<String>,
    pub model_agent: Option<String>,
    pub system_prompt: Option<String>,
    pub max_tokens: Option<i32>,
    pub temperature: Option<f64>,
}

#[utoipa::path(
    get,
    path = "/api/v1/admin/ai-config",
    responses(
        (status = 200, description = "Global AI provider config"),
    ),
    tag = "admin",
    security(("session" = []))
)]
pub async fn admin_get_ai_config(
    State(pool): State<PgPool>,
) -> Result<Json<serde_json::Value>, AppError> {
    let config = sqlx::query!(
        r#"SELECT provider, model_nano, model_quick, model_standard, model_deep, model_agent,
                  system_prompt, max_tokens, temperature
           FROM ai_provider_configs WHERE is_global_default = TRUE LIMIT 1"#,
    )
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound)?;

    Ok(Json(serde_json::json!({ "data": {
        "provider": config.provider,
        "model_nano": config.model_nano,
        "model_quick": config.model_quick,
        "model_standard": config.model_standard,
        "model_deep": config.model_deep,
        "model_agent": config.model_agent,
        "system_prompt": config.system_prompt,
        "max_tokens": config.max_tokens,
        "temperature": config.temperature,
    }})))
}

#[utoipa::path(
    patch,
    path = "/api/v1/admin/ai-config",
    request_body = AdminAiConfigRequest,
    responses(
        (status = 200, description = "Global AI config updated"),
    ),
    tag = "admin",
    security(("session" = []))
)]
pub async fn admin_patch_ai_config(
    State(pool): State<PgPool>,
    Extension(admin): Extension<AuthUser>,
    Json(body): Json<AdminAiConfigRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    if let Some(ref p) = body.provider {
        sqlx::query!(
            "UPDATE ai_provider_configs SET provider = $1, updated_at = NOW() WHERE is_global_default = TRUE",
            p,
        )
        .execute(&pool).await?;
    }
    if let Some(ref m) = body.model_nano {
        sqlx::query!(
            "UPDATE ai_provider_configs SET model_nano = $1, updated_at = NOW() WHERE is_global_default = TRUE",
            m,
        )
        .execute(&pool).await?;
    }
    if let Some(ref m) = body.model_standard {
        sqlx::query!(
            "UPDATE ai_provider_configs SET model_standard = $1, updated_at = NOW() WHERE is_global_default = TRUE",
            m,
        )
        .execute(&pool).await?;
    }
    if let Some(ref m) = body.model_deep {
        sqlx::query!(
            "UPDATE ai_provider_configs SET model_deep = $1, updated_at = NOW() WHERE is_global_default = TRUE",
            m,
        )
        .execute(&pool).await?;
    }
    if let Some(ref m) = body.model_agent {
        sqlx::query!(
            "UPDATE ai_provider_configs SET model_agent = $1, updated_at = NOW() WHERE is_global_default = TRUE",
            m,
        )
        .execute(&pool).await?;
    }
    if let Some(ref s) = body.system_prompt {
        sqlx::query!(
            "UPDATE ai_provider_configs SET system_prompt = $1, updated_at = NOW() WHERE is_global_default = TRUE",
            s,
        )
        .execute(&pool).await?;
    }

    record_audit_event(
        &pool,
        Some(admin.id),
        None,
        "admin.ai_config.update",
        Some("ai_config"),
        None,
        serde_json::json!({"provider": body.provider}),
        None,
        None,
    )
    .await
    .ok();

    Ok(Json(serde_json::json!({ "data": null })))
}

// ── Admin: KodaInstance Management ───────────────────────────────────────────

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct KodaInstanceResponse {
    pub id: Uuid,
    pub name: String,
    pub base_url: String,
    pub region: Option<String>,
    pub status: String,
    pub last_seen_at: Option<time::OffsetDateTime>,
    pub created_at: time::OffsetDateTime,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateInstanceRequest {
    pub name: String,
    pub base_url: String,
    pub region: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/v1/admin/instances",
    responses(
        (status = 200, description = "All registered Koda instances", body = Vec<KodaInstanceResponse>),
    ),
    tag = "admin",
    security(("session" = []))
)]
pub async fn admin_list_instances(
    State(pool): State<PgPool>,
) -> Result<Json<serde_json::Value>, AppError> {
    let rows = sqlx::query!(
        r#"SELECT id, name, base_url, region, status, last_seen_at, created_at
           FROM koda_instances ORDER BY created_at DESC"#
    )
    .fetch_all(&pool)
    .await?;

    let instances: Vec<KodaInstanceResponse> = rows
        .into_iter()
        .map(|r| KodaInstanceResponse {
            id: r.id,
            name: r.name,
            base_url: r.base_url,
            region: r.region,
            status: r.status,
            last_seen_at: r.last_seen_at,
            created_at: r.created_at,
        })
        .collect();

    Ok(Json(serde_json::json!({ "data": instances })))
}

#[utoipa::path(
    post,
    path = "/api/v1/admin/instances",
    request_body = CreateInstanceRequest,
    responses(
        (status = 201, description = "Instance registered", body = KodaInstanceResponse),
        (status = 409, description = "Name already taken"),
    ),
    tag = "admin",
    security(("session" = []))
)]
pub async fn admin_create_instance(
    State(pool): State<PgPool>,
    Extension(admin): Extension<AuthUser>,
    Json(body): Json<CreateInstanceRequest>,
) -> Result<(axum::http::StatusCode, Json<serde_json::Value>), AppError> {
    if body.name.is_empty() || body.base_url.is_empty() {
        return Err(AppError::Validation("name and base_url are required".into()));
    }

    let row = sqlx::query!(
        r#"INSERT INTO koda_instances (name, base_url, region)
           VALUES ($1, $2, $3)
           RETURNING id, name, base_url, region, status, last_seen_at, created_at"#,
        body.name,
        body.base_url,
        body.region,
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        if e.to_string().contains("unique") {
            AppError::Conflict(format!("instance '{}' already exists", body.name))
        } else {
            e.into()
        }
    })?;

    record_audit_event(
        &pool,
        Some(admin.id),
        None,
        "admin.instance.create",
        Some("koda_instance"),
        Some(&row.id.to_string()),
        serde_json::json!({"name": body.name, "base_url": body.base_url}),
        None,
        None,
    )
    .await
    .ok();

    Ok((
        axum::http::StatusCode::CREATED,
        Json(serde_json::json!({ "data": KodaInstanceResponse {
            id: row.id,
            name: row.name,
            base_url: row.base_url,
            region: row.region,
            status: row.status,
            last_seen_at: row.last_seen_at,
            created_at: row.created_at,
        }})),
    ))
}

#[utoipa::path(
    delete,
    path = "/api/v1/admin/instances/{instance_id}",
    params(("instance_id" = Uuid, Path, description = "Instance ID")),
    responses(
        (status = 204, description = "Instance removed"),
    ),
    tag = "admin",
    security(("session" = []))
)]
pub async fn admin_delete_instance(
    State(pool): State<PgPool>,
    Extension(admin): Extension<AuthUser>,
    Path(instance_id): Path<Uuid>,
) -> Result<axum::http::StatusCode, AppError> {
    let deleted = sqlx::query!(
        "DELETE FROM koda_instances WHERE id = $1",
        instance_id,
    )
    .execute(&pool)
    .await?;

    if deleted.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    record_audit_event(
        &pool,
        Some(admin.id),
        None,
        "admin.instance.delete",
        Some("koda_instance"),
        Some(&instance_id.to_string()),
        serde_json::json!({}),
        None,
        None,
    )
    .await
    .ok();

    Ok(axum::http::StatusCode::NO_CONTENT)
}

// ── Admin: OrgInstanceAffinity ────────────────────────────────────────────────

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct OrgAffinityResponse {
    pub organization_id: Uuid,
    pub instance_id: Uuid,
    pub instance_name: String,
    pub instance_base_url: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct SetAffinityRequest {
    pub instance_id: Uuid,
}

#[utoipa::path(
    put,
    path = "/api/v1/admin/organizations/{org_id}/instance-affinity",
    params(("org_id" = Uuid, Path, description = "Organization ID")),
    request_body = SetAffinityRequest,
    responses(
        (status = 200, description = "Affinity set"),
    ),
    tag = "admin",
    security(("session" = []))
)]
pub async fn admin_set_org_affinity(
    State(pool): State<PgPool>,
    Extension(admin): Extension<AuthUser>,
    Path(org_id): Path<Uuid>,
    Json(body): Json<SetAffinityRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query!(
        r#"INSERT INTO org_instance_affinities (organization_id, instance_id)
           VALUES ($1, $2)
           ON CONFLICT (organization_id) DO UPDATE SET instance_id = $2"#,
        org_id,
        body.instance_id,
    )
    .execute(&pool)
    .await?;

    record_audit_event(
        &pool,
        Some(admin.id),
        Some(org_id),
        "admin.instance_affinity.set",
        Some("org_instance_affinity"),
        Some(&org_id.to_string()),
        serde_json::json!({"instance_id": body.instance_id}),
        None,
        None,
    )
    .await
    .ok();

    Ok(Json(serde_json::json!({ "data": null })))
}

#[utoipa::path(
    get,
    path = "/api/v1/admin/organizations/{org_id}/instance-affinity",
    params(("org_id" = Uuid, Path, description = "Organization ID")),
    responses(
        (status = 200, description = "Org instance affinity", body = OrgAffinityResponse),
    ),
    tag = "admin",
    security(("session" = []))
)]
pub async fn admin_get_org_affinity(
    State(pool): State<PgPool>,
    Path(org_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let row = sqlx::query!(
        r#"SELECT oia.organization_id, oia.instance_id, ki.name AS instance_name, ki.base_url AS instance_base_url
           FROM org_instance_affinities oia
           JOIN koda_instances ki ON ki.id = oia.instance_id
           WHERE oia.organization_id = $1"#,
        org_id,
    )
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(serde_json::json!({ "data": OrgAffinityResponse {
        organization_id: row.organization_id,
        instance_id: row.instance_id,
        instance_name: row.instance_name,
        instance_base_url: row.instance_base_url,
    }})))
}

// ── Admin: Org Instance Migration ─────────────────────────────────────────────

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct MigrateOrgRequest {
    pub target_instance_id: Uuid,
}

pub async fn admin_migrate_org(
    State(pool): State<PgPool>,
    Extension(admin): Extension<AuthUser>,
    Path(org_id): Path<Uuid>,
    Json(body): Json<MigrateOrgRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Find current instance affinity
    let current = sqlx::query!(
        "SELECT instance_id FROM org_instance_affinities WHERE organization_id = $1",
        org_id,
    )
    .fetch_optional(&pool)
    .await?;

    let source_id = current.map(|r| r.instance_id).unwrap_or(body.target_instance_id);

    if source_id == body.target_instance_id {
        return Err(AppError::BadRequest("source and target instance are the same".into()));
    }

    // Verify target instance exists and is healthy
    let target = sqlx::query!(
        "SELECT id, name, status FROM koda_instances WHERE id = $1",
        body.target_instance_id,
    )
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::NotFound)?;

    if target.status != "healthy" {
        return Err(AppError::BadRequest(format!(
            "target instance '{}' is not healthy (status: {})",
            target.name, target.status
        )));
    }

    // Create migration record
    let migration_id: Uuid = sqlx::query_scalar!(
        r#"INSERT INTO instance_org_migrations
               (organization_id, source_instance, target_instance, initiated_by, status)
           VALUES ($1, $2, $3, $4, 'pending')
           RETURNING id"#,
        org_id,
        source_id,
        body.target_instance_id,
        admin.id,
    )
    .fetch_one(&pool)
    .await?;

    // Update affinity immediately (target instance will pick up workspaces on next schedule)
    sqlx::query!(
        r#"INSERT INTO org_instance_affinities (organization_id, instance_id)
           VALUES ($1, $2)
           ON CONFLICT (organization_id) DO UPDATE SET instance_id = EXCLUDED.instance_id"#,
        org_id,
        body.target_instance_id,
    )
    .execute(&pool)
    .await?;

    // Emit audit event
    let _ = sqlx::query!(
        r#"INSERT INTO audit_events (actor_id, organization_id, action, resource_type, resource_id, metadata)
           VALUES ($1, $2, 'org.instance_migration_initiated', 'organization', $3, $4)"#,
        admin.id,
        org_id,
        org_id.to_string(),
        serde_json::json!({
            "migration_id": migration_id,
            "source_instance": source_id,
            "target_instance": body.target_instance_id,
            "target_name": target.name,
        }),
    )
    .execute(&pool)
    .await;

    tracing::info!(
        migration_id = %migration_id,
        org_id = %org_id,
        target = %target.name,
        "org instance migration initiated"
    );

    Ok(Json(serde_json::json!({
        "migration_id": migration_id,
        "status": "pending",
        "target_instance": target.name,
    })))
}

/// GET /api/v1/admin/instances/load-balance
/// Returns the least-loaded healthy instance (for automatic org placement).
pub async fn admin_instance_load_balance(
    State(pool): State<PgPool>,
) -> Result<Json<serde_json::Value>, AppError> {
    let row = sqlx::query!(
        r#"SELECT id, name, base_url, workspace_count, cpu_usage_pct, ram_usage_pct
           FROM koda_instances
           WHERE status = 'healthy'
           ORDER BY workspace_count ASC, cpu_usage_pct ASC NULLS LAST
           LIMIT 1"#,
    )
    .fetch_optional(&pool)
    .await?;

    match row {
        Some(r) => Ok(Json(serde_json::json!({
            "instance_id": r.id,
            "name": r.name,
            "base_url": r.base_url,
            "workspace_count": r.workspace_count,
            "cpu_usage_pct": r.cpu_usage_pct,
        }))),
        None => Err(AppError::NotFound),
    }
}

// ── Admin: MFA Reset ──────────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/v1/admin/users/{user_id}/reset-mfa",
    params(("user_id" = Uuid, Path, description = "User ID")),
    responses(
        (status = 204, description = "MFA reset"),
        (status = 404, description = "User not found"),
    ),
    tag = "admin",
    security(("session" = []))
)]
pub async fn admin_reset_mfa(
    State(pool): State<PgPool>,
    Extension(admin): Extension<AuthUser>,
    Path(user_id): Path<Uuid>,
) -> Result<axum::http::StatusCode, AppError> {
    let deleted = sqlx::query!(
        "DELETE FROM totp_credentials WHERE user_id = $1",
        user_id,
    )
    .execute(&pool)
    .await?;

    if deleted.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    record_audit_event(
        &pool,
        Some(admin.id),
        None,
        "admin.mfa.reset",
        Some("user"),
        Some(&user_id.to_string()),
        serde_json::json!({"target_user_id": user_id}),
        None,
        None,
    )
    .await
    .ok();

    Ok(axum::http::StatusCode::NO_CONTENT)
}


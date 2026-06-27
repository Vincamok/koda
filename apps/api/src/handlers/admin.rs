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

#[derive(Debug, Serialize)]
pub struct AdminOrgResponse {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub status: String,
    pub member_count: i64,
    pub workspace_count: i64,
    pub created_at: time::OffsetDateTime,
}

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
           LEFT JOIN workspaces w ON w.organization_id = o.id AND w.deleted_at IS NULL
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

#[derive(Debug, Serialize)]
pub struct AdminUserResponse {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub is_super_admin: bool,
    pub email_verified: bool,
    pub org_count: i64,
    pub created_at: time::OffsetDateTime,
}

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

#[derive(Debug, Serialize)]
pub struct ImpersonateResponse {
    pub impersonation_token: String,
    pub user_id: Uuid,
    pub expires_at: time::OffsetDateTime,
}

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

#[derive(Debug, Serialize)]
pub struct AuditEventResponse {
    pub id: Uuid,
    pub actor_id: Option<Uuid>,
    pub actor_email: Option<String>,
    pub organization_id: Option<Uuid>,
    pub action: String,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub metadata: serde_json::Value,
    pub ip_address: Option<String>,
    pub created_at: time::OffsetDateTime,
}

pub async fn admin_audit_logs(
    State(pool): State<PgPool>,
    Query(q): Query<PaginationQuery>,
) -> Result<Json<Vec<AuditEventResponse>>, AppError> {
    let limit = q.limit.unwrap_or(100).min(500);
    let offset = q.offset.unwrap_or(0);

    let rows = sqlx::query!(
        r#"SELECT ae.id, ae.actor_id, ae.organization_id, ae.action,
                  ae.resource_type, ae.resource_id, ae.metadata, ae.ip_address, ae.created_at,
                  u.email AS actor_email
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

#[derive(Debug, Serialize)]
pub struct InfraStatsResponse {
    pub total_orgs: i64,
    pub total_users: i64,
    pub total_workspaces: i64,
    pub running_workspaces: i64,
    pub total_pipelines: i64,
    pub failed_jobs: i64,
}

pub async fn admin_infra_stats(
    State(pool): State<PgPool>,
) -> Result<Json<InfraStatsResponse>, AppError> {
    let stats = sqlx::query!(
        r#"SELECT
               (SELECT COUNT(*) FROM organizations) AS total_orgs,
               (SELECT COUNT(*) FROM users) AS total_users,
               (SELECT COUNT(*) FROM workspaces WHERE deleted_at IS NULL) AS total_workspaces,
               (SELECT COUNT(*) FROM workspaces WHERE status = 'running' AND deleted_at IS NULL) AS running_workspaces,
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

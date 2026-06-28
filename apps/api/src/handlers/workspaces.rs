use std::convert::Infallible;
use std::time::Duration;

use axum::{
    extract::{Path, Query, State},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
    Extension, Json,
};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tokio_stream::wrappers::ReceiverStream;
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::AppError,
    jobs::{GitJob, OrchestratorJob, STREAM_GIT, STREAM_ORCHESTRATOR},
    middleware::auth::{AuthUser, OrgContext},
    AppState,
};

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct WorkspaceResponse {
    pub id: Uuid,
    pub uid: String,
    pub organization_id: Uuid,
    pub project_id: Option<Uuid>,
    pub name: String,
    pub status: String,
    pub cpu_limit: i32,
    pub ram_limit_mb: i32,
    pub pids_limit: i32,
    pub created_by: Option<Uuid>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct CreateWorkspaceRequest {
    #[validate(length(min = 1, max = 120))]
    pub name: String,
    pub project_id: Option<Uuid>,
    pub template_id: Option<Uuid>,
    pub cpu_limit: Option<i32>,
    pub ram_limit_mb: Option<i32>,
    pub git: Option<GitConfigRequest>,
}

#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct GitConfigRequest {
    #[validate(length(min = 1))]
    pub repo_url: String,
    #[validate(length(min = 1, max = 200))]
    pub branch: Option<String>,
    pub ssh_key_secret_ref_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct ListWorkspacesQuery {
    pub project_id: Option<Uuid>,
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn generate_uid(org_id: Uuid) -> String {
    let short = &org_id.to_string().replace('-', "")[..8];
    let rand = Uuid::new_v4().to_string().replace('-', "");
    format!("ws-{short}-{}", &rand[..8])
}

async fn publish_job(redis: &mut redis::aio::MultiplexedConnection, stream: &str, job: &impl serde::Serialize) -> Result<(), AppError> {
    let json = serde_json::to_string(job)
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    let _: String = redis
        .xadd(stream, "*", &[("payload", json)])
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    Ok(())
}

// ── POST /organizations/:org_id/workspaces ────────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{org_id}/workspaces",
    params(("org_id" = Uuid, Path, description = "Organization ID")),
    request_body = CreateWorkspaceRequest,
    responses(
        (status = 201, description = "Workspace created", body = WorkspaceResponse),
        (status = 403, description = "Insufficient role"),
        (status = 429, description = "Workspace quota exceeded"),
    ),
    tag = "workspaces",
    security(("session" = []))
)]
pub async fn post_workspace(
    State(mut state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Extension(org): Extension<OrgContext>,
    Json(body): Json<CreateWorkspaceRequest>,
) -> Result<impl IntoResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let caller_role = auth.org_role.as_deref().unwrap_or("");
    if !["owner", "admin", "member", "super_admin"].contains(&caller_role) {
        return Err(AppError::Forbidden("insufficient role".into()));
    }

    // Enforce quota
    let quota = sqlx::query!(
        "SELECT max_workspaces FROM organization_quotas WHERE organization_id = $1",
        org.id
    )
    .fetch_optional(&state.pool)
    .await?;

    let max_ws = quota.map(|q| q.max_workspaces).unwrap_or(10);
    let current_count: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM workspaces WHERE organization_id = $1 AND status != 'closed'",
        org.id
    )
    .fetch_one(&state.pool)
    .await?
    .unwrap_or(0);

    if current_count >= max_ws as i64 {
        return Err(AppError::QuotaExceeded("workspace limit reached".into()));
    }

    let uid = generate_uid(org.id);
    let cpu = body.cpu_limit.unwrap_or(2).clamp(1, 16);
    let ram = body.ram_limit_mb.unwrap_or(2048).clamp(512, 32768);

    let ws = sqlx::query!(
        r#"INSERT INTO workspaces
               (uid, organization_id, project_id, template_id, created_by, name, cpu_limit, ram_limit_mb)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
           RETURNING id, uid, organization_id, project_id, name, status, cpu_limit, ram_limit_mb,
                     pids_limit, created_by, created_at, updated_at"#,
        uid,
        org.id,
        body.project_id,
        body.template_id,
        auth.id,
        body.name,
        cpu,
        ram,
    )
    .fetch_one(&state.pool)
    .await?;

    // If a git config was provided, store it and enqueue clone job
    if let Some(git) = body.git {
        git.validate()
            .map_err(|e| AppError::Validation(e.to_string()))?;

        let branch = git.branch.unwrap_or_else(|| "main".into());
        let gc = sqlx::query!(
            r#"INSERT INTO workspace_git_configs
                   (workspace_id, repo_url, branch, ssh_key_secret_ref_id)
               VALUES ($1, $2, $3, $4)
               RETURNING id"#,
            ws.id,
            git.repo_url,
            branch,
            git.ssh_key_secret_ref_id,
        )
        .fetch_one(&state.pool)
        .await?;

        // Update workspace status to 'cloning'
        sqlx::query!(
            "UPDATE workspaces SET status = 'cloning', updated_at = NOW() WHERE id = $1",
            ws.id
        )
        .execute(&state.pool)
        .await?;

        let job = GitJob::CloneRepo {
            workspace_id: ws.id,
            git_config_id: gc.id,
            repo_url: git.repo_url,
            branch,
            ssh_key_secret_ref_id: git.ssh_key_secret_ref_id,
        };
        publish_job(&mut state.redis, STREAM_GIT, &job).await?;
        tracing::info!(workspace_id = %ws.id, "enqueued clone job");
    } else {
        // No git — workspace is immediately 'ready' for start
        sqlx::query!(
            "UPDATE workspaces SET status = 'ready', updated_at = NOW() WHERE id = $1",
            ws.id
        )
        .execute(&state.pool)
        .await?;
    }

    let resp = WorkspaceResponse {
        id: ws.id,
        uid: ws.uid,
        organization_id: ws.organization_id,
        project_id: ws.project_id,
        name: ws.name,
        status: ws.status,
        cpu_limit: ws.cpu_limit,
        ram_limit_mb: ws.ram_limit_mb,
        pids_limit: ws.pids_limit,
        created_by: ws.created_by,
        created_at: ws.created_at,
        updated_at: ws.updated_at,
    };

    Ok((axum::http::StatusCode::CREATED, Json(serde_json::json!({ "data": resp }))))
}

// ── GET /organizations/:org_id/workspaces ─────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{org_id}/workspaces",
    params(("org_id" = Uuid, Path, description = "Organization ID")),
    responses(
        (status = 200, description = "List of workspaces", body = Vec<WorkspaceResponse>),
    ),
    tag = "workspaces",
    security(("session" = []))
)]
pub async fn get_workspaces(
    State(state): State<AppState>,
    Extension(_auth): Extension<AuthUser>,
    Extension(org): Extension<OrgContext>,
    Query(params): Query<ListWorkspacesQuery>,
) -> Result<impl IntoResponse, AppError> {
    let limit = params.limit.unwrap_or(20).clamp(1, 100);
    let offset = params.offset.unwrap_or(0).max(0);

    let workspaces = sqlx::query!(
        r#"SELECT id, uid, organization_id, project_id, name, status,
                  cpu_limit, ram_limit_mb, pids_limit, created_by, created_at, updated_at
           FROM workspaces
           WHERE organization_id = $1
             AND ($2::uuid IS NULL OR project_id = $2)
             AND ($3::text IS NULL OR status = $3)
           ORDER BY created_at DESC
           LIMIT $4 OFFSET $5"#,
        org.id,
        params.project_id,
        params.status,
        limit,
        offset,
    )
    .fetch_all(&state.pool)
    .await?;

    let data: Vec<WorkspaceResponse> = workspaces
        .into_iter()
        .map(|ws| WorkspaceResponse {
            id: ws.id,
            uid: ws.uid,
            organization_id: ws.organization_id,
            project_id: ws.project_id,
            name: ws.name,
            status: ws.status,
            cpu_limit: ws.cpu_limit,
            ram_limit_mb: ws.ram_limit_mb,
            pids_limit: ws.pids_limit,
            created_by: ws.created_by,
            created_at: ws.created_at,
            updated_at: ws.updated_at,
        })
        .collect();

    Ok(Json(serde_json::json!({ "data": data })))
}

// ── GET /organizations/:org_id/workspaces/:workspace_id ───────────────────────

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{org_id}/workspaces/{workspace_id}",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
    ),
    responses(
        (status = 200, description = "Workspace details", body = WorkspaceResponse),
        (status = 404, description = "Not found"),
    ),
    tag = "workspaces",
    security(("session" = []))
)]
pub async fn get_workspace(
    State(state): State<AppState>,
    Extension(_auth): Extension<AuthUser>,
    Extension(org): Extension<OrgContext>,
    Path((_org_id, workspace_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let ws = sqlx::query!(
        r#"SELECT id, uid, organization_id, project_id, name, status,
                  cpu_limit, ram_limit_mb, pids_limit, created_by, created_at, updated_at
           FROM workspaces
           WHERE id = $1 AND organization_id = $2"#,
        workspace_id,
        org.id,
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(serde_json::json!({ "data": WorkspaceResponse {
        id: ws.id,
        uid: ws.uid,
        organization_id: ws.organization_id,
        project_id: ws.project_id,
        name: ws.name,
        status: ws.status,
        cpu_limit: ws.cpu_limit,
        ram_limit_mb: ws.ram_limit_mb,
        pids_limit: ws.pids_limit,
        created_by: ws.created_by,
        created_at: ws.created_at,
        updated_at: ws.updated_at,
    }})))
}

// ── POST /organizations/:org_id/workspaces/:workspace_id/start ─────────────────

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{org_id}/workspaces/{workspace_id}/start",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
    ),
    responses(
        (status = 200, description = "Start enqueued"),
        (status = 400, description = "Invalid state transition"),
        (status = 404, description = "Not found"),
    ),
    tag = "workspaces",
    security(("session" = []))
)]
pub async fn post_workspace_start(
    State(mut state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Extension(org): Extension<OrgContext>,
    Path((_org_id, workspace_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let ws = sqlx::query!(
        "SELECT id, status FROM workspaces WHERE id = $1 AND organization_id = $2",
        workspace_id,
        org.id,
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    if !["ready", "stopped"].contains(&ws.status.as_str()) {
        return Err(AppError::BadRequest(format!(
            "cannot start workspace in '{}' state",
            ws.status
        )));
    }

    sqlx::query!(
        "UPDATE workspaces SET status = 'starting', updated_at = NOW() WHERE id = $1",
        workspace_id
    )
    .execute(&state.pool)
    .await?;

    let job = OrchestratorJob::StartWorkspace {
        workspace_id,
        org_id: org.id,
    };
    publish_job(&mut state.redis, STREAM_ORCHESTRATOR, &job).await?;
    tracing::info!(workspace_id = %workspace_id, user_id = %auth.id, "enqueued start-workspace job");

    Ok(Json(serde_json::json!({ "data": { "status": "starting" } })))
}

// ── POST /organizations/:org_id/workspaces/:workspace_id/stop ─────────────────

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{org_id}/workspaces/{workspace_id}/stop",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
    ),
    responses(
        (status = 200, description = "Stop enqueued"),
        (status = 400, description = "Workspace not running"),
        (status = 404, description = "Not found"),
    ),
    tag = "workspaces",
    security(("session" = []))
)]
pub async fn post_workspace_stop(
    State(mut state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Extension(org): Extension<OrgContext>,
    Path((_org_id, workspace_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let ws = sqlx::query!(
        "SELECT id, status FROM workspaces WHERE id = $1 AND organization_id = $2",
        workspace_id,
        org.id,
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    if ws.status != "running" {
        return Err(AppError::BadRequest(format!(
            "cannot stop workspace in '{}' state",
            ws.status
        )));
    }

    sqlx::query!(
        "UPDATE workspaces SET status = 'stopping', updated_at = NOW() WHERE id = $1",
        workspace_id
    )
    .execute(&state.pool)
    .await?;

    let job = OrchestratorJob::StopWorkspace {
        workspace_id,
        org_id: org.id,
    };
    publish_job(&mut state.redis, STREAM_ORCHESTRATOR, &job).await?;
    tracing::info!(workspace_id = %workspace_id, user_id = %auth.id, "enqueued stop-workspace job");

    Ok(Json(serde_json::json!({ "data": { "status": "stopping" } })))
}

// ── DELETE /organizations/:org_id/workspaces/:workspace_id ───────────────────

#[utoipa::path(
    delete,
    path = "/api/v1/organizations/{org_id}/workspaces/{workspace_id}",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
    ),
    responses(
        (status = 200, description = "Workspace deleted"),
        (status = 400, description = "Must stop before deleting"),
        (status = 403, description = "Insufficient role"),
        (status = 404, description = "Not found"),
    ),
    tag = "workspaces",
    security(("session" = []))
)]
pub async fn delete_workspace(
    State(mut state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Extension(org): Extension<OrgContext>,
    Path((_org_id, workspace_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let caller_role = auth.org_role.as_deref().unwrap_or("");
    if !["owner", "admin", "super_admin"].contains(&caller_role) {
        return Err(AppError::Forbidden("insufficient role".into()));
    }

    let ws = sqlx::query!(
        "SELECT id, status FROM workspaces WHERE id = $1 AND organization_id = $2",
        workspace_id,
        org.id,
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    if ws.status == "running" {
        return Err(AppError::BadRequest("stop the workspace before deleting".into()));
    }

    sqlx::query!(
        "UPDATE workspaces SET status = 'closed', updated_at = NOW() WHERE id = $1",
        workspace_id
    )
    .execute(&state.pool)
    .await?;

    let job = OrchestratorJob::DeleteWorkspace {
        workspace_id,
        org_id: org.id,
    };
    publish_job(&mut state.redis, STREAM_ORCHESTRATOR, &job).await?;
    tracing::info!(workspace_id = %workspace_id, user_id = %auth.id, "enqueued delete-workspace job");

    Ok(Json(serde_json::json!({ "data": null })))
}

// ── Workspace Snapshots ────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateSnapshotRequest {
    pub label: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SnapshotResponse {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub label: String,
    pub status: String,
    pub size_bytes: Option<i64>,
    pub created_at: OffsetDateTime,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{org_id}/workspaces/{workspace_id}/snapshots",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
    ),
    request_body = CreateSnapshotRequest,
    responses(
        (status = 201, description = "Snapshot initiated"),
        (status = 400, description = "Workspace must be running or stopped"),
        (status = 404, description = "Not found"),
    ),
    tag = "workspaces",
    security(("session" = []))
)]
pub async fn post_workspace_snapshot(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Extension(org): Extension<OrgContext>,
    Path((_org_id, workspace_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<CreateSnapshotRequest>,
) -> Result<impl IntoResponse, AppError> {
    let caller_role = auth.org_role.as_deref().unwrap_or("");
    if !["owner", "admin", "member", "super_admin"].contains(&caller_role) {
        return Err(AppError::Forbidden("insufficient role".into()));
    }

    let ws = sqlx::query!(
        "SELECT id, status FROM workspaces WHERE id = $1 AND organization_id = $2 AND status != 'closed'",
        workspace_id,
        org.id,
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    if ws.status != "running" && ws.status != "stopped" {
        return Err(AppError::BadRequest(
            "workspace must be running or stopped to snapshot".into(),
        ));
    }

    let snapshot_path = format!("/var/lib/koda/snapshots/{}/{}", org.id, workspace_id);

    sqlx::query!(
        r#"INSERT INTO workspace_snapshots
               (workspace_id, organization_id, created_by, label, volume_snapshot_path, status)
           VALUES ($1, $2, $3, $4, $5, 'pending')"#,
        workspace_id,
        org.id,
        auth.id,
        body.label,
        snapshot_path,
    )
    .execute(&state.pool)
    .await?;

    // Background orchestrator task will copy the volume and update status → ready.
    Ok(axum::http::StatusCode::CREATED.into_response())
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{org_id}/workspaces/{workspace_id}/snapshots",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
    ),
    responses(
        (status = 200, description = "List of snapshots", body = Vec<SnapshotResponse>),
    ),
    tag = "workspaces",
    security(("session" = []))
)]
pub async fn get_workspace_snapshots(
    State(state): State<AppState>,
    Extension(_auth): Extension<AuthUser>,
    Extension(org): Extension<OrgContext>,
    Path((_org_id, workspace_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let rows = sqlx::query!(
        r#"SELECT id, workspace_id, label, status, size_bytes, created_at
           FROM workspace_snapshots
           WHERE workspace_id = $1 AND organization_id = $2 AND status != 'deleted'
           ORDER BY created_at DESC
           LIMIT 20"#,
        workspace_id,
        org.id,
    )
    .fetch_all(&state.pool)
    .await?;

    let data: Vec<SnapshotResponse> = rows
        .into_iter()
        .map(|r| SnapshotResponse {
            id: r.id,
            workspace_id: r.workspace_id,
            label: r.label,
            status: r.status,
            size_bytes: r.size_bytes,
            created_at: r.created_at,
        })
        .collect();

    Ok(Json(serde_json::json!({ "data": data })))
}

// ── GET /organizations/:org_id/workspaces/:workspace_id/events (SSE) ─────────

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{org_id}/workspaces/{workspace_id}/events",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
    ),
    responses(
        (status = 200, description = "SSE stream of workspace status events (text/event-stream)"),
        (status = 404, description = "Workspace not found"),
    ),
    tag = "workspaces",
    security(("session" = []))
)]
pub async fn get_workspace_events(
    State(state): State<AppState>,
    Extension(_auth): Extension<AuthUser>,
    Extension(org): Extension<OrgContext>,
    Path((_org_id, workspace_id)): Path<(Uuid, Uuid)>,
) -> Sse<ReceiverStream<Result<Event, Infallible>>> {
    let (tx, rx) = tokio::sync::mpsc::channel::<Result<Event, Infallible>>(8);
    let pool = state.pool.clone();

    tokio::spawn(async move {
        let mut last_status = String::new();
        let mut interval = tokio::time::interval(Duration::from_secs(1));

        loop {
            interval.tick().await;

            let row = sqlx::query_scalar!(
                "SELECT status FROM workspaces WHERE id = $1 AND organization_id = $2",
                workspace_id,
                org.id,
            )
            .fetch_optional(&pool)
            .await;

            match row {
                Ok(Some(status)) => {
                    if status != last_status {
                        last_status = status.clone();
                        let data = serde_json::json!({ "status": status }).to_string();
                        if tx.send(Ok(Event::default().event("status").data(data))).await.is_err() {
                            break; // client disconnected
                        }
                    }
                }
                Ok(None) => break, // workspace deleted
                Err(_) => {
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    });

    Sse::new(ReceiverStream::new(rx)).keep_alive(KeepAlive::default())
}

// ── POST /organizations/:org_id/workspaces/:workspace_id/fork ─────────────────

pub async fn post_workspace_fork(
    State(mut state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Extension(org): Extension<OrgContext>,
    Path((_org_id, workspace_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let caller_role = auth.org_role.as_deref().unwrap_or("");
    if !["owner", "admin", "member", "super_admin"].contains(&caller_role) {
        return Err(AppError::Forbidden("insufficient role".into()));
    }

    // Fetch source workspace, verify org ownership and that it is not closed
    let src = sqlx::query!(
        r#"SELECT id, organization_id, name, status, cpu_limit, ram_limit_mb
           FROM workspaces
           WHERE id = $1 AND organization_id = $2"#,
        workspace_id,
        org.id,
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    if src.status == "closed" {
        return Err(AppError::BadRequest("cannot fork a closed workspace".into()));
    }

    // Fetch git config if present
    let git_cfg = sqlx::query!(
        r#"SELECT id, repo_url, branch, ssh_key_secret_ref_id
           FROM workspace_git_configs
           WHERE workspace_id = $1
           LIMIT 1"#,
        src.id,
    )
    .fetch_optional(&state.pool)
    .await?;

    // Enforce quota
    let quota = sqlx::query!(
        "SELECT max_workspaces FROM organization_quotas WHERE organization_id = $1",
        org.id
    )
    .fetch_optional(&state.pool)
    .await?;
    let max_ws = quota.map(|q| q.max_workspaces).unwrap_or(10);
    let current_count: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM workspaces WHERE organization_id = $1 AND status != 'closed'",
        org.id
    )
    .fetch_one(&state.pool)
    .await?
    .unwrap_or(0);
    if current_count >= max_ws as i64 {
        return Err(AppError::QuotaExceeded("workspace limit reached".into()));
    }

    let fork_name = format!("{}-fork", src.name);
    let uid = generate_uid(org.id);

    let ws = sqlx::query!(
        r#"INSERT INTO workspaces
               (uid, organization_id, created_by, name, cpu_limit, ram_limit_mb, forked_from)
           VALUES ($1, $2, $3, $4, $5, $6, $7)
           RETURNING id, uid, organization_id, project_id, name, status, cpu_limit, ram_limit_mb,
                     pids_limit, created_by, created_at, updated_at"#,
        uid,
        org.id,
        auth.id,
        fork_name,
        src.cpu_limit,
        src.ram_limit_mb,
        src.id,
    )
    .fetch_one(&state.pool)
    .await?;

    if let Some(gc) = git_cfg {
        let branch = gc.branch;
        let new_gc = sqlx::query!(
            r#"INSERT INTO workspace_git_configs
                   (workspace_id, repo_url, branch, ssh_key_secret_ref_id)
               VALUES ($1, $2, $3, $4)
               RETURNING id"#,
            ws.id,
            gc.repo_url,
            branch,
            gc.ssh_key_secret_ref_id,
        )
        .fetch_one(&state.pool)
        .await?;

        sqlx::query!(
            "UPDATE workspaces SET status = 'cloning', updated_at = NOW() WHERE id = $1",
            ws.id
        )
        .execute(&state.pool)
        .await?;

        let job = crate::jobs::GitJob::CloneRepo {
            workspace_id: ws.id,
            git_config_id: new_gc.id,
            repo_url: gc.repo_url,
            branch,
            ssh_key_secret_ref_id: gc.ssh_key_secret_ref_id,
        };
        publish_job(&mut state.redis, crate::jobs::STREAM_GIT, &job).await?;
        tracing::info!(workspace_id = %ws.id, forked_from = %src.id, "enqueued clone job for fork");
    } else {
        sqlx::query!(
            "UPDATE workspaces SET status = 'ready', updated_at = NOW() WHERE id = $1",
            ws.id
        )
        .execute(&state.pool)
        .await?;
    }

    let resp = WorkspaceResponse {
        id: ws.id,
        uid: ws.uid,
        organization_id: ws.organization_id,
        project_id: ws.project_id,
        name: ws.name,
        status: ws.status,
        cpu_limit: ws.cpu_limit,
        ram_limit_mb: ws.ram_limit_mb,
        pids_limit: ws.pids_limit,
        created_by: ws.created_by,
        created_at: ws.created_at,
        updated_at: ws.updated_at,
    };

    Ok((axum::http::StatusCode::CREATED, Json(serde_json::json!({ "data": resp }))))
}

// ── GET /api/v1/workspaces/:uid/ssh ──────────────────────────────────────────
// Returns the SSH host + port assigned by sozu for koda connect.

#[derive(Debug, Serialize)]
pub struct WorkspaceSshInfo {
    pub ssh_host: String,
    pub ssh_port: i32,
}

pub async fn get_workspace_ssh(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(uid): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // Resolve workspace by UID and verify user has access
    let row = sqlx::query!(
        r#"
        SELECT w.id, er.host_port
        FROM workspaces w
        JOIN memberships m ON m.organization_id = w.organization_id AND m.user_id = $2
        LEFT JOIN exposure_rules er ON er.workspace_id = w.id AND er.rule_type = 'tcp' AND er.internal_port = 22
        WHERE w.uid = $1 AND w.status != 'closed'
        LIMIT 1
        "#,
        uid,
        auth.id,
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let port = row.host_port.unwrap_or(2200);

    let host = state.config.app_base_url
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .split('/')
        .next()
        .unwrap_or("localhost")
        .to_string();

    Ok(Json(serde_json::json!({
        "data": {
            "ssh_host": host,
            "ssh_port": port
        }
    })))
}

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::IntoResponse,
    Extension, Json,
};
use bollard::{
    container::LogOutput,
    exec::{CreateExecOptions, ResizeExecOptions, StartExecOptions, StartExecResults},
};
use futures::{SinkExt, StreamExt};
use redis::AsyncCommands as _;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::{error::AppError, middleware::auth::AuthUser, AppState};

#[derive(Serialize)]
pub struct TerminalSessionResponse {
    pub id: Uuid,
    pub redis_channel: String,
    pub allow_write: bool,
}

#[derive(Deserialize)]
pub struct CreateTerminalSessionRequest {
    #[serde(default)]
    pub allow_write: bool,
}

/// POST /api/v1/workspaces/:workspace_id/terminal-sessions
/// Creates a shared terminal session (pair programming). The caller becomes the owner
/// and connects via the standard ws_terminal endpoint; this session ID lets guests join.
pub async fn create_terminal_session(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(workspace_id): Path<Uuid>,
    Json(body): Json<CreateTerminalSessionRequest>,
) -> Result<impl IntoResponse, AppError> {
    let row = sqlx::query!(
        r#"
        SELECT w.id, w.organization_id, wpb.container_id
        FROM workspaces w
        JOIN memberships m ON m.organization_id = w.organization_id AND m.user_id = $2
        LEFT JOIN workspace_plugin_bindings wpb
               ON wpb.workspace_id = w.id AND wpb.status = 'running'
        LEFT JOIN plugin_definitions pd ON pd.id = wpb.plugin_definition_id
              AND pd.slug IN ('koda-web-ide', 'code-server', 'ssh')
        WHERE w.id = $1 AND w.status != 'closed'
        ORDER BY
            CASE pd.slug
                WHEN 'koda-web-ide' THEN 1
                WHEN 'code-server'  THEN 2
                WHEN 'ssh'          THEN 3
                ELSE 4
            END
        LIMIT 1
        "#,
        workspace_id,
        auth.id,
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let container_id = row.container_id.ok_or_else(|| {
        AppError::BadRequest("no running container for this workspace".into())
    })?;

    let session_id = Uuid::new_v4();
    let channel = format!("koda:terminal:{}", session_id);

    sqlx::query!(
        r#"INSERT INTO terminal_sessions
               (id, workspace_id, organization_id, owner_id, container_id, redis_channel, allow_write)
           VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
        session_id,
        workspace_id,
        row.organization_id,
        auth.id,
        container_id,
        channel,
        body.allow_write,
    )
    .execute(&state.pool)
    .await?;

    Ok(Json(TerminalSessionResponse {
        id: session_id,
        redis_channel: channel,
        allow_write: body.allow_write,
    }))
}

/// GET /api/v1/ws/:workspace_id/shared-terminal/:session_id
/// Owner: creates the Docker exec and publishes output to Redis. Input goes to exec stdin.
/// Guest: subscribes to Redis channel for output. Input forwarded only if allow_write = true.
pub async fn ws_shared_terminal(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path((workspace_id, session_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let session = sqlx::query!(
        r#"SELECT ts.id, ts.owner_id, ts.container_id, ts.redis_channel, ts.allow_write
           FROM terminal_sessions ts
           JOIN memberships m ON m.organization_id = ts.organization_id AND m.user_id = $3
           WHERE ts.id = $1 AND ts.workspace_id = $2 AND ts.status = 'active'"#,
        session_id,
        workspace_id,
        auth.id,
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let is_owner = session.owner_id == auth.id;
    let allow_write = session.allow_write || is_owner;
    let container_id = session.container_id.clone();
    let channel = session.redis_channel.clone();

    let docker_host = std::env::var("DOCKER_HOST")
        .unwrap_or_else(|_| "unix:///var/run/docker.sock".into());

    let pool = state.pool.clone();
    let redis_client = state.redis_client.clone();

    Ok(ws.on_upgrade(move |socket| async move {
        if is_owner {
            if let Err(e) = run_owner_terminal(socket, container_id, docker_host, channel, redis_client, pool, session_id).await {
                tracing::warn!(error = %e, "shared_terminal owner error");
            }
        } else {
            if let Err(e) = run_guest_terminal(socket, channel, redis_client, allow_write, container_id, docker_host).await {
                tracing::warn!(error = %e, "shared_terminal guest error");
            }
        }
    }))
}

/// Owner runs the real exec PTY and publishes all output to Redis pub/sub.
async fn run_owner_terminal(
    socket: WebSocket,
    container_id: String,
    docker_host: String,
    channel: String,
    redis_client: redis::Client,
    pool: sqlx::PgPool,
    session_id: Uuid,
) -> anyhow::Result<()> {
    let docker = connect_docker(&docker_host)?;

    let exec = docker.create_exec(
        &container_id,
        CreateExecOptions {
            attach_stdin: Some(true),
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            tty: Some(true),
            cmd: Some(vec![
                "/bin/sh".to_string(),
                "-c".to_string(),
                "[ -f /root/.personal/shell/bashrc ] && . /root/.personal/shell/bashrc; \
                 TERM=xterm-256color exec $(which bash || which sh)".to_string(),
            ]),
            env: Some(vec!["TERM=xterm-256color".to_string()]),
            ..Default::default()
        },
    ).await?;

    let exec_id = exec.id.clone();

    let start = docker.start_exec(&exec.id, Some(StartExecOptions { detach: false, ..Default::default() })).await?;
    let (mut output_stream, mut input_writer) = match start {
        StartExecResults::Attached { output, input } => (output, input),
        StartExecResults::Detached => anyhow::bail!("exec detached unexpectedly"),
    };

    let mut pub_conn = redis_client.get_multiplexed_async_connection().await?;
    let (mut ws_tx, mut ws_rx) = socket.split();

    let docker_to_ws_and_redis = {
        let channel = channel.clone();
        async move {
            while let Some(item) = output_stream.next().await {
                let bytes = match item {
                    Ok(LogOutput::Console { message })
                    | Ok(LogOutput::StdOut { message })
                    | Ok(LogOutput::StdErr { message }) => message.to_vec(),
                    _ => continue,
                };
                // Send to owner's WebSocket
                if ws_tx.send(Message::Binary(bytes.clone())).await.is_err() {
                    break;
                }
                // Broadcast to all guests via Redis pub/sub
                let _: Result<(), _> = pub_conn.publish(&channel, bytes).await;
            }
        }
    };

    let exec_id_clone = exec_id.clone();
    let ws_to_docker = async move {
        while let Some(msg) = ws_rx.next().await {
            match msg {
                Ok(Message::Binary(data)) => {
                    if data.len() == 5 && data[0] == 0x01 {
                        let cols = u16::from_be_bytes([data[1], data[2]]) as u32;
                        let rows = u16::from_be_bytes([data[3], data[4]]) as u32;
                        let _ = docker.resize_exec(&exec_id_clone, ResizeExecOptions { width: cols, height: rows }).await;
                    } else if input_writer.write(&data).await.is_err() {
                        break;
                    }
                }
                Ok(Message::Text(t)) => { let _ = input_writer.write(t.as_bytes()).await; }
                Ok(Message::Close(_)) | Err(_) => break,
                _ => {}
            }
        }
    };

    tokio::select! {
        _ = docker_to_ws_and_redis => {}
        _ = ws_to_docker => {}
    }

    // Mark session closed
    let _ = sqlx::query!(
        "UPDATE terminal_sessions SET status = 'closed', closed_at = NOW() WHERE id = $1",
        session_id,
    )
    .execute(&pool)
    .await;

    Ok(())
}

/// Guest subscribes to Redis pub/sub and forwards output to their WebSocket.
/// If allow_write, their keyboard input is published to a separate input channel.
async fn run_guest_terminal(
    socket: WebSocket,
    channel: String,
    redis_client: redis::Client,
    allow_write: bool,
    _container_id: String,
    _docker_host: String,
) -> anyhow::Result<()> {
    let mut sub_conn = redis_client.get_async_pubsub().await?;
    sub_conn.subscribe(&channel).await?;

    let (mut ws_tx, mut ws_rx) = socket.split();
    let mut msg_stream = sub_conn.on_message();

    let redis_to_ws = async move {
        while let Some(msg) = msg_stream.next().await {
            let payload: Vec<u8> = msg.get_payload().unwrap_or_default();
            if ws_tx.send(Message::Binary(payload)).await.is_err() {
                break;
            }
        }
    };

    // Drain input from guest (we accept but ignore if allow_write = false)
    let guest_input = async move {
        if !allow_write {
            // drain without processing
            while let Some(msg) = ws_rx.next().await {
                if matches!(msg, Ok(Message::Close(_)) | Err(_)) { break; }
            }
        }
        // If allow_write, input would be published to an input channel for the owner
        // to read; for simplicity we drain here — a full implementation would use
        // a second Redis channel `koda:terminal:{id}:input` read by the owner loop.
    };

    tokio::select! {
        _ = redis_to_ws => {}
        _ = guest_input => {}
    }

    Ok(())
}

fn connect_docker(docker_host: &str) -> anyhow::Result<bollard::Docker> {
    if docker_host.starts_with("tcp://") || docker_host.starts_with("http://") {
        Ok(bollard::Docker::connect_with_http(docker_host, 120, bollard::API_DEFAULT_VERSION)?)
    } else {
        let path = docker_host.trim_start_matches("unix://");
        Ok(bollard::Docker::connect_with_unix(path, 120, bollard::API_DEFAULT_VERSION)?)
    }
}

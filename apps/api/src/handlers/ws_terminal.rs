use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::IntoResponse,
    Extension,
};
use bollard::{
    container::LogOutput,
    exec::{CreateExecOptions, ResizeExecOptions, StartExecOptions, StartExecResults},
};
use futures::{SinkExt, StreamExt};
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::{error::AppError, middleware::auth::AuthUser, AppState};

/// WebSocket terminal — exec /bin/bash in the workspace's koda-web-ide container.
/// Route: GET /api/v1/ws/:workspace_id/terminal
pub async fn ws_terminal(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(workspace_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    // Verify workspace is accessible by this user (owns it or is a member of its org)
    let row = sqlx::query!(
        r#"
        SELECT w.id,
               wpb.container_id
        FROM workspaces w
        JOIN memberships m ON m.organization_id = w.organization_id AND m.user_id = $2
        LEFT JOIN workspace_plugin_bindings wpb
               ON wpb.workspace_id = w.id
              AND wpb.status = 'running'
        LEFT JOIN plugin_definitions pd ON pd.id = wpb.plugin_definition_id
              AND pd.slug IN ('koda-web-ide', 'ssh', 'code-server')
        WHERE w.id = $1
          AND w.status != 'closed'
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
        AppError::BadRequest("no running container found for this workspace".into())
    })?;

    let docker_host = std::env::var("DOCKER_HOST")
        .unwrap_or_else(|_| "unix:///var/run/docker.sock".into());

    Ok(ws.on_upgrade(move |socket| handle_ws(socket, container_id, docker_host)))
}

async fn handle_ws(socket: WebSocket, container_id: String, docker_host: String) {
    if let Err(e) = run_terminal(socket, container_id, docker_host).await {
        tracing::warn!(error = %e, "ws_terminal error");
    }
}

async fn run_terminal(
    socket: WebSocket,
    container_id: String,
    docker_host: String,
) -> anyhow::Result<()> {
    let docker = if docker_host.starts_with("tcp://") || docker_host.starts_with("http://") {
        bollard::Docker::connect_with_http(&docker_host, 120, bollard::API_DEFAULT_VERSION)?
    } else {
        let path = docker_host
            .trim_start_matches("unix://")
            .to_string();
        bollard::Docker::connect_with_unix(&path, 120, bollard::API_DEFAULT_VERSION)?
    };

    // Create exec session (PTY)
    let exec = docker
        .create_exec(
            &container_id,
            CreateExecOptions {
                attach_stdin: Some(true),
                attach_stdout: Some(true),
                attach_stderr: Some(true),
                tty: Some(true),
                cmd: Some(vec![
                    "/bin/sh".to_string(),
                    "-c".to_string(),
                    // Source personal shell configs if present, then exec login shell
                    "[ -f /root/.personal/shell/bashrc ] && . /root/.personal/shell/bashrc; \
                     [ -f /root/.personal/shell/profile ] && . /root/.personal/shell/profile; \
                     [ -f /root/.personal/git/.gitconfig ] && cp /root/.personal/git/.gitconfig /root/.gitconfig 2>/dev/null || true; \
                     TERM=xterm-256color exec $(which bash || which sh)".to_string(),
                ]),
                env: Some(vec!["TERM=xterm-256color".to_string()]),
                ..Default::default()
            },
        )
        .await?;

    let exec_id = exec.id.clone();

    let start_result = docker
        .start_exec(
            &exec.id,
            Some(StartExecOptions {
                detach: false,
                ..Default::default()
            }),
        )
        .await?;

    let (mut output_stream, mut input_writer) = match start_result {
        StartExecResults::Attached { output, input } => (output, input),
        StartExecResults::Detached => {
            return Err(anyhow::anyhow!("exec started in detached mode unexpectedly"))
        }
    };

    let (mut ws_tx, mut ws_rx) = socket.split();

    // Docker → WebSocket
    let docker_to_ws = async move {
        while let Some(item) = output_stream.next().await {
            match item {
                Ok(LogOutput::Console { message }) | Ok(LogOutput::StdOut { message }) | Ok(LogOutput::StdErr { message }) => {
                    if ws_tx.send(Message::Binary(message.to_vec())).await.is_err() {
                        break;
                    }
                }
                _ => {}
            }
        }
    };

    // WebSocket → Docker (stdin) + handle resize messages
    let ws_to_docker = async move {
        while let Some(msg) = ws_rx.next().await {
            match msg {
                Ok(Message::Binary(data)) => {
                    // Check for resize packet: 0x01 prefix + 4 bytes (u16 cols, u16 rows BE)
                    if data.len() == 5 && data[0] == 0x01 {
                        let cols = u16::from_be_bytes([data[1], data[2]]);
                        let rows = u16::from_be_bytes([data[3], data[4]]);
                        let _ = docker
                            .resize_exec(
                                &exec_id,
                                ResizeExecOptions {
                                    width: cols,
                                    height: rows,
                                },
                            )
                            .await;
                    } else if input_writer.write(&data).await.is_err() {
                        break;
                    }
                }
                Ok(Message::Text(text)) => {
                    if input_writer.write(text.as_bytes()).await.is_err() {
                        break;
                    }
                }
                Ok(Message::Close(_)) | Err(_) => break,
                _ => {}
            }
        }
    };

    tokio::select! {
        _ = docker_to_ws => {}
        _ = ws_to_docker => {}
    }

    Ok(())
}

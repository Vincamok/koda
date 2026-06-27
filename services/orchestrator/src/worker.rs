use anyhow::Context;
use redis::{aio::MultiplexedConnection, AsyncCommands};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::docker::DockerManager;

const STREAM: &str = "koda:jobs:orchestrator";
const DEAD_LETTER: &str = "koda:jobs:orchestrator:dead";
const MAX_RETRIES: u64 = 3;

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum OrchestratorJob {
    StartWorkspace { workspace_id: Uuid, org_id: Uuid },
    StopWorkspace { workspace_id: Uuid, org_id: Uuid },
    DeleteWorkspace { workspace_id: Uuid, org_id: Uuid },
}

pub struct OrchestratorWorker {
    pub pool: PgPool,
    pub redis: MultiplexedConnection,
    pub docker: DockerManager,
    pub group: String,
    pub consumer: String,
}

impl OrchestratorWorker {
    pub async fn init_consumer_group(&mut self) -> anyhow::Result<()> {
        let result: redis::RedisResult<()> = self
            .redis
            .xgroup_create_mkstream(STREAM, &self.group, "$")
            .await;

        match result {
            Ok(_) | Err(_) => {} // group already exists is fine
        }
        self.docker.ensure_network().await?;
        Ok(())
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        self.init_consumer_group().await?;
        tracing::info!(group = %self.group, consumer = %self.consumer, "orchestrator worker started");

        loop {
            let entries: redis::streams::StreamReadReply = self
                .redis
                .xread_options(
                    &[STREAM],
                    &[">"],
                    &redis::streams::StreamReadOptions::default()
                        .group(&self.group, &self.consumer)
                        .count(10)
                        .block(5000),
                )
                .await
                .unwrap_or_else(|_| redis::streams::StreamReadReply { keys: vec![] });

            for stream_key in entries.keys {
                for message in stream_key.ids {
                    let id = message.id.clone();
                    if let Err(e) = self.process_message(&message).await {
                        tracing::error!(msg_id = %id, error = %e, "failed to process message");
                        self.handle_failure(&id, &message).await;
                    } else {
                        let _: () = self.redis.xack(STREAM, &self.group, &[&id]).await?;
                    }
                }
            }

            // Also process pending (unacked) messages from crashed consumers
            self.process_pending().await?;
        }
    }

    async fn process_message(&mut self, msg: &redis::streams::StreamId) -> anyhow::Result<()> {
        let payload: String = msg
            .map
            .get("payload")
            .and_then(|v| match v {
                redis::Value::Data(b) => Some(String::from_utf8_lossy(b).into_owned()),
                redis::Value::BulkString(b) => Some(String::from_utf8_lossy(b).into_owned()),
                _ => None,
            })
            .context("missing payload field")?;

        let job: OrchestratorJob = serde_json::from_str(&payload)
            .context("deserialize orchestrator job")?;

        match job {
            OrchestratorJob::StartWorkspace { workspace_id, org_id } => {
                self.start_workspace(workspace_id, org_id).await?;
            }
            OrchestratorJob::StopWorkspace { workspace_id, org_id } => {
                self.stop_workspace(workspace_id, org_id).await?;
            }
            OrchestratorJob::DeleteWorkspace { workspace_id, org_id } => {
                self.delete_workspace(workspace_id, org_id).await?;
            }
        }

        Ok(())
    }

    async fn start_workspace(&mut self, workspace_id: Uuid, org_id: Uuid) -> anyhow::Result<()> {
        let ws = sqlx::query!(
            "SELECT id, cpu_limit, ram_limit_mb, pids_limit, created_by FROM workspaces WHERE id = $1 AND organization_id = $2",
            workspace_id,
            org_id
        )
        .fetch_optional(&self.pool)
        .await?
        .context("workspace not found")?;

        let user_id = ws.created_by.context("workspace has no owner")?;
        let env_vars = vec![
            format!("KODA_WORKSPACE_ID={}", workspace_id),
            format!("KODA_ORG_ID={}", org_id),
        ];

        match self
            .docker
            .start_workspace(
                workspace_id,
                org_id,
                user_id,
                ws.cpu_limit,
                ws.ram_limit_mb,
                ws.pids_limit,
                env_vars,
            )
            .await
        {
            Ok(container_id) => {
                sqlx::query!(
                    "UPDATE workspaces SET status = 'running', updated_at = NOW() WHERE id = $1",
                    workspace_id
                )
                .execute(&self.pool)
                .await?;
                tracing::info!(workspace_id = %workspace_id, container_id = %container_id, "workspace running");
            }
            Err(e) => {
                sqlx::query!(
                    "UPDATE workspaces SET status = 'failed', updated_at = NOW() WHERE id = $1",
                    workspace_id
                )
                .execute(&self.pool)
                .await?;
                return Err(e.context("start workspace container"));
            }
        }

        Ok(())
    }

    async fn stop_workspace(&mut self, workspace_id: Uuid, _org_id: Uuid) -> anyhow::Result<()> {
        match self.docker.stop_workspace(workspace_id).await {
            Ok(_) => {
                sqlx::query!(
                    "UPDATE workspaces SET status = 'stopped', updated_at = NOW() WHERE id = $1",
                    workspace_id
                )
                .execute(&self.pool)
                .await?;
            }
            Err(e) => {
                tracing::warn!(workspace_id = %workspace_id, error = %e, "stop failed, marking failed");
                sqlx::query!(
                    "UPDATE workspaces SET status = 'failed', updated_at = NOW() WHERE id = $1",
                    workspace_id
                )
                .execute(&self.pool)
                .await?;
                return Err(e);
            }
        }
        Ok(())
    }

    async fn delete_workspace(&mut self, workspace_id: Uuid, _org_id: Uuid) -> anyhow::Result<()> {
        self.docker.delete_workspace(workspace_id).await?;
        sqlx::query!(
            "DELETE FROM workspaces WHERE id = $1 AND status = 'closed'",
            workspace_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn handle_failure(&mut self, msg_id: &str, msg: &redis::streams::StreamId) {
        // Acknowledge to remove from PEL, then push to dead-letter stream
        let _: Result<(), _> = self.redis.xack(STREAM, &self.group, &[msg_id]).await;

        let payload = msg
            .map
            .get("payload")
            .and_then(|v| match v {
                redis::Value::Data(b) => Some(String::from_utf8_lossy(b).into_owned()),
                redis::Value::BulkString(b) => Some(String::from_utf8_lossy(b).into_owned()),
                _ => None,
            })
            .unwrap_or_default();

        let _: Result<String, _> = self
            .redis
            .xadd(DEAD_LETTER, "*", &[("original_id", msg_id), ("payload", &payload)])
            .await;
    }

    async fn process_pending(&mut self) -> anyhow::Result<()> {
        let pending: redis::streams::StreamPendingReply = self
            .redis
            .xpending(STREAM, &self.group, "-", "+", 10i64, None::<&str>)
            .await
            .unwrap_or(redis::streams::StreamPendingReply::Empty);

        if let redis::streams::StreamPendingReply::Data(data) = pending {
            for entry in data.ids {
                if entry.times_delivered >= MAX_RETRIES {
                    tracing::warn!(msg_id = %entry.id, "max retries reached, sending to dead-letter");
                    let _: Result<(), _> = self
                        .redis
                        .xack(STREAM, &self.group, &[&entry.id])
                        .await;
                    let _: Result<String, _> = self
                        .redis
                        .xadd(DEAD_LETTER, "*", &[("original_id", &entry.id)])
                        .await;
                }
            }
        }

        Ok(())
    }
}

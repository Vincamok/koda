use redis::{aio::MultiplexedConnection, AsyncCommands};
use serde::Serialize;
use uuid::Uuid;

pub const STREAM_ORCHESTRATOR: &str = "koda:jobs:orchestrator";
pub const STREAM_GIT: &str = "koda:jobs:git";

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OrchestratorJob {
    StartWorkspace { workspace_id: Uuid, org_id: Uuid },
    StopWorkspace { workspace_id: Uuid, org_id: Uuid },
    DeleteWorkspace { workspace_id: Uuid, org_id: Uuid },
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GitJob {
    CloneRepo {
        workspace_id: Uuid,
        git_config_id: Uuid,
        repo_url: String,
        branch: String,
        ssh_key_secret_ref_id: Option<Uuid>,
    },
}

pub struct JobPublisher {
    conn: MultiplexedConnection,
}

impl JobPublisher {
    pub async fn new(redis_url: &str) -> anyhow::Result<Self> {
        let client = redis::Client::open(redis_url)?;
        let conn = client.get_multiplexed_async_connection().await?;
        Ok(Self { conn })
    }

    async fn publish(&mut self, stream: &str, payload: &impl Serialize) -> anyhow::Result<String> {
        let json = serde_json::to_string(payload)?;
        let id: String = self
            .conn
            .xadd(stream, "*", &[("payload", json)])
            .await?;
        Ok(id)
    }

    pub async fn start_workspace(&mut self, workspace_id: Uuid, org_id: Uuid) -> anyhow::Result<String> {
        self.publish(STREAM_ORCHESTRATOR, &OrchestratorJob::StartWorkspace { workspace_id, org_id }).await
    }

    pub async fn stop_workspace(&mut self, workspace_id: Uuid, org_id: Uuid) -> anyhow::Result<String> {
        self.publish(STREAM_ORCHESTRATOR, &OrchestratorJob::StopWorkspace { workspace_id, org_id }).await
    }

    pub async fn delete_workspace(&mut self, workspace_id: Uuid, org_id: Uuid) -> anyhow::Result<String> {
        self.publish(STREAM_ORCHESTRATOR, &OrchestratorJob::DeleteWorkspace { workspace_id, org_id }).await
    }

    pub async fn clone_repo(
        &mut self,
        workspace_id: Uuid,
        git_config_id: Uuid,
        repo_url: String,
        branch: String,
        ssh_key_secret_ref_id: Option<Uuid>,
    ) -> anyhow::Result<String> {
        self.publish(
            STREAM_GIT,
            &GitJob::CloneRepo { workspace_id, git_config_id, repo_url, branch, ssh_key_secret_ref_id },
        )
        .await
    }
}

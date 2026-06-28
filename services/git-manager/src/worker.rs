use std::{path::PathBuf, time::Duration};

use anyhow::Context;
use redis::{aio::MultiplexedConnection, AsyncCommands};
use serde::Deserialize;
use sqlx::PgPool;
use tokio::time::timeout;
use uuid::Uuid;

use crate::cloner;

const STREAM: &str = "koda:jobs:git";
const DEAD_LETTER: &str = "koda:jobs:git:dead";
const MAX_RETRIES: u64 = 3;

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum GitJob {
    CloneRepo {
        workspace_id: Uuid,
        git_config_id: Uuid,
        repo_url: String,
        branch: String,
        ssh_key_secret_ref_id: Option<Uuid>,
    },
}

pub struct GitWorker {
    pub pool: PgPool,
    pub redis: MultiplexedConnection,
    pub group: String,
    pub consumer: String,
    pub volumes_base: String,
    pub clone_timeout: Duration,
}

impl GitWorker {
    pub async fn init_consumer_group(&mut self) -> anyhow::Result<()> {
        let _: redis::RedisResult<()> = self
            .redis
            .xgroup_create_mkstream(STREAM, &self.group, "$")
            .await;
        Ok(())
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        self.init_consumer_group().await?;
        tracing::info!(group = %self.group, consumer = %self.consumer, "git-manager worker started");

        loop {
            let entries: redis::streams::StreamReadReply = self
                .redis
                .xread_options(
                    &[STREAM],
                    &[">"],
                    &redis::streams::StreamReadOptions::default()
                        .group(&self.group, &self.consumer)
                        .count(5)
                        .block(5000),
                )
                .await
                .unwrap_or_else(|_| redis::streams::StreamReadReply { keys: vec![] });

            for stream_key in entries.keys {
                for message in stream_key.ids {
                    let id = message.id.clone();
                    if let Err(e) = self.process_message(&message).await {
                        tracing::error!(msg_id = %id, error = %e, "git job failed");
                        self.handle_failure(&id, &message).await;
                    } else {
                        let _: () = self.redis.xack(STREAM, &self.group, &[&id]).await?;
                    }
                }
            }
        }
    }

    async fn process_message(&mut self, msg: &redis::streams::StreamId) -> anyhow::Result<()> {
        let payload: String = msg
            .map
            .get("payload")
            .and_then(|v| match v {
                redis::Value::Data(b) => Some(String::from_utf8_lossy(b).into_owned()),
                _ => None,
            })
            .context("missing payload")?;

        let job: GitJob = serde_json::from_str(&payload).context("deserialize git job")?;

        match job {
            GitJob::CloneRepo {
                workspace_id,
                git_config_id,
                repo_url,
                branch,
                ssh_key_secret_ref_id,
            } => {
                self.clone_repo(workspace_id, git_config_id, repo_url, branch, ssh_key_secret_ref_id)
                    .await?
            }
        }

        Ok(())
    }

    async fn clone_repo(
        &mut self,
        workspace_id: Uuid,
        git_config_id: Uuid,
        repo_url: String,
        branch: String,
        ssh_key_secret_ref_id: Option<Uuid>,
    ) -> anyhow::Result<()> {
        // Mark clone as in-progress
        sqlx::query!(
            "UPDATE workspace_git_configs SET clone_status = 'cloning', updated_at = NOW() WHERE id = $1",
            git_config_id
        )
        .execute(&self.pool)
        .await?;

        // Decrypt SSH key if present
        let ssh_key_pem: Option<String> = if let Some(secret_id) = ssh_key_secret_ref_id {
            let row = sqlx::query!(
                "SELECT encrypted_value, nonce FROM secret_refs WHERE id = $1",
                secret_id
            )
            .fetch_optional(&self.pool)
            .await?;

            if let Some(r) = row {
                // Note: decryption happens here; key from KODA_SECRET_ENCRYPTION_KEY env
                let key_hex = std::env::var("SECRET_ENCRYPTION_KEY")
                    .context("SECRET_ENCRYPTION_KEY not set")?;
                use aes_gcm::{Aes256Gcm, Key, Nonce};
                use aes_gcm::aead::{Aead, KeyInit};
                let key_bytes = hex::decode(&key_hex).context("decode encryption key")?;
                let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
                let cipher = Aes256Gcm::new(key);
                let nonce = Nonce::from_slice(&r.nonce);
                let plaintext = cipher
                    .decrypt(nonce, r.encrypted_value.as_ref())
                    .map_err(|e| anyhow::anyhow!("decrypt ssh key: {e}"))?;
                Some(String::from_utf8(plaintext).context("ssh key utf8")?)
            } else {
                None
            }
        } else {
            None
        };

        let target_dir = PathBuf::from(&self.volumes_base).join(workspace_id.to_string());
        std::fs::create_dir_all(&target_dir).context("create workspace volume dir")?;

        let clone_result = {
            let url = repo_url.clone();
            let b = branch.clone();
            let dir = target_dir.clone();
            let pem = ssh_key_pem.clone();
            timeout(self.clone_timeout, tokio::task::spawn_blocking(move || {
                cloner::clone_repo(&url, &b, &dir, pem.as_deref())
            }))
            .await
            .context("clone timeout")?
            .context("join error")?
        };

        match clone_result {
            Ok(result) => {
                sqlx::query!(
                    r#"UPDATE workspace_git_configs
                       SET clone_status = 'ready', clone_error = NULL,
                           last_cloned_at = NOW(), updated_at = NOW()
                       WHERE id = $1"#,
                    git_config_id
                )
                .execute(&self.pool)
                .await?;

                // Workspace is now ready
                sqlx::query!(
                    "UPDATE workspaces SET status = 'ready', updated_at = NOW() WHERE id = $1",
                    workspace_id
                )
                .execute(&self.pool)
                .await?;

                tracing::info!(
                    workspace_id = %workspace_id,
                    sha = %result.commit_sha,
                    "cloned successfully"
                );
            }
            Err(e) => {
                let err_msg = e.to_string();
                sqlx::query!(
                    r#"UPDATE workspace_git_configs
                       SET clone_status = 'failed', clone_error = $1, updated_at = NOW()
                       WHERE id = $2"#,
                    err_msg,
                    git_config_id
                )
                .execute(&self.pool)
                .await?;

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

    async fn handle_failure(&mut self, msg_id: &str, msg: &redis::streams::StreamId) {
        let _: Result<(), _> = self.redis.xack(STREAM, &self.group, &[msg_id]).await;
        let payload = msg
            .map
            .get("payload")
            .and_then(|v| match v {
                redis::Value::Data(b) => Some(String::from_utf8_lossy(b).into_owned()),
                _ => None,
            })
            .unwrap_or_default();
        let _: Result<String, _> = self
            .redis
            .xadd(DEAD_LETTER, "*", &[("original_id", msg_id), ("payload", &payload)])
            .await;
    }
}

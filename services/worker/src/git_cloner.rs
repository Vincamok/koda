use redis::{aio::MultiplexedConnection, AsyncCommands};
use serde::Deserialize;
use sqlx::PgPool;
use std::collections::HashMap;
use std::path::Path;
use uuid::Uuid;

const STREAM: &str = "koda:jobs:git";
const DEAD_LETTER: &str = "koda:jobs:git:dead";
const MAX_RETRIES: u8 = 3;

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

pub struct GitCloner {
    pub pool: PgPool,
    pub redis: MultiplexedConnection,
    pub group: String,
    pub consumer: String,
    pub workspace_root: String,
}

impl GitCloner {
    pub async fn init(&mut self) -> anyhow::Result<()> {
        let _: redis::RedisResult<()> = self
            .redis
            .xgroup_create_mkstream(STREAM, &self.group, "$")
            .await;
        Ok(())
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        self.init().await?;
        tracing::info!(group = %self.group, "git cloner started");

        let mut failure_counts: HashMap<String, u8> = HashMap::new();

        loop {
            let entries: redis::streams::StreamReadReply = self
                .redis
                .xread_options(
                    &[STREAM],
                    &[">"],
                    &redis::streams::StreamReadOptions::default()
                        .group(&self.group, &self.consumer)
                        .count(3)
                        .block(5000),
                )
                .await
                .unwrap_or_else(|_| redis::streams::StreamReadReply { keys: vec![] });

            for stream_key in entries.keys {
                for message in stream_key.ids {
                    let id = message.id.clone();
                    match self.process(&message).await {
                        Ok(_) => {
                            let _: Result<(), _> = self.redis.xack(STREAM, &self.group, &[&id]).await;
                            failure_counts.remove(&id);
                        }
                        Err(e) => {
                            let count = failure_counts.entry(id.clone()).or_insert(0);
                            *count += 1;
                            tracing::warn!(id = %id, attempt = *count, error = %e, "git clone failed");
                            if *count >= MAX_RETRIES {
                                tracing::error!(id = %id, "moving git job to dead letter");
                                if let Some(payload) = message.map.get("payload") {
                                    let _: Result<(), _> = self.redis
                                        .xadd(DEAD_LETTER, "*", &[("payload", payload.as_ref())])
                                        .await;
                                }
                                let _: Result<(), _> = self.redis.xack(STREAM, &self.group, &[&id]).await;
                                failure_counts.remove(&id);
                            }
                        }
                    }
                }
            }
        }
    }

    async fn process(&mut self, message: &redis::streams::StreamId) -> anyhow::Result<()> {
        let payload = message
            .map
            .get("payload")
            .and_then(|v| v.as_ref().to_str().ok())
            .ok_or_else(|| anyhow::anyhow!("missing payload"))?;

        let job: GitJob = serde_json::from_str(payload)?;

        match job {
            GitJob::CloneRepo {
                workspace_id,
                git_config_id,
                repo_url,
                branch,
                ssh_key_secret_ref_id,
            } => {
                self.clone_repo(workspace_id, git_config_id, repo_url, branch, ssh_key_secret_ref_id)
                    .await?;
            }
        }

        Ok(())
    }

    async fn clone_repo(
        &self,
        workspace_id: Uuid,
        _git_config_id: Uuid,
        repo_url: String,
        branch: String,
        ssh_key_secret_ref_id: Option<Uuid>,
    ) -> anyhow::Result<()> {
        tracing::info!(workspace_id = %workspace_id, repo_url = %repo_url, "starting git clone");

        // Mark as cloning
        sqlx::query!(
            "UPDATE workspaces SET status = 'cloning', updated_at = NOW() WHERE id = $1",
            workspace_id,
        )
        .execute(&self.pool)
        .await?;

        // Resolve SSH key if provided
        let ssh_key_pem: Option<String> = if let Some(ref_id) = ssh_key_secret_ref_id {
            self.resolve_secret(ref_id).await.ok()
        } else {
            None
        };

        let clone_path = format!("{}/{}", self.workspace_root, workspace_id);
        let path = Path::new(&clone_path);

        // Remove target dir if exists
        if path.exists() {
            std::fs::remove_dir_all(path).ok();
        }

        let result = self.do_clone(&repo_url, &branch, &clone_path, ssh_key_pem.as_deref()).await;

        match result {
            Ok(()) => {
                tracing::info!(workspace_id = %workspace_id, "git clone successful");
                // Read devcontainer.json and auto-bind plugins
                self.apply_devcontainer(&clone_path, workspace_id).await.ok();
                sqlx::query!(
                    "UPDATE workspaces SET status = 'ready', updated_at = NOW() WHERE id = $1",
                    workspace_id,
                )
                .execute(&self.pool)
                .await?;
            }
            Err(e) => {
                tracing::error!(workspace_id = %workspace_id, error = %e, "git clone failed");
                sqlx::query!(
                    "UPDATE workspaces SET status = 'failed', updated_at = NOW() WHERE id = $1",
                    workspace_id,
                )
                .execute(&self.pool)
                .await?;
                return Err(e);
            }
        }

        Ok(())
    }

    async fn do_clone(
        &self,
        repo_url: &str,
        branch: &str,
        target: &str,
        ssh_key_pem: Option<&str>,
    ) -> anyhow::Result<()> {
        let mut callbacks = git2::RemoteCallbacks::new();

        if let Some(key_pem) = ssh_key_pem {
            // Write key to temp file
            let key_path = format!("/tmp/koda_clone_key_{}", uuid::Uuid::new_v4());
            std::fs::write(&key_path, key_pem)?;
            // Set file permissions 0600
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&key_path, std::fs::Permissions::from_mode(0o600))?;
            }

            let key_path_clone = key_path.clone();
            callbacks.credentials(move |_url, username_from_url, _allowed| {
                git2::Cred::ssh_key(
                    username_from_url.unwrap_or("git"),
                    None,
                    std::path::Path::new(&key_path_clone),
                    None,
                )
            });
        } else {
            // HTTPS or public repo — try SSH agent then default
            callbacks.credentials(|_url, username_from_url, allowed| {
                if allowed.contains(git2::CredentialType::SSH_KEY) {
                    git2::Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
                } else {
                    git2::Cred::default()
                }
            });
        }

        let mut fetch_opts = git2::FetchOptions::new();
        fetch_opts.remote_callbacks(callbacks);
        fetch_opts.depth(1); // shallow clone

        let mut builder = git2::build::RepoBuilder::new();
        builder.fetch_options(fetch_opts);
        builder.branch(branch);

        builder
            .clone(repo_url, std::path::Path::new(target))
            .map(|_| ())
            .map_err(|e| anyhow::anyhow!("git clone failed: {e}"))
    }

    async fn apply_devcontainer(&self, clone_path: &str, workspace_id: Uuid) -> anyhow::Result<()> {
        // Try .devcontainer/devcontainer.json then .devcontainer.json
        let paths = [
            format!("{clone_path}/.devcontainer/devcontainer.json"),
            format!("{clone_path}/.devcontainer.json"),
        ];

        let content = paths.iter()
            .find_map(|p| std::fs::read_to_string(p).ok());

        let content = match content {
            Some(c) => c,
            None => return Ok(()),
        };

        let json: serde_json::Value = serde_json::from_str(&content)
            .unwrap_or(serde_json::Value::Null);

        if json.is_null() {
            return Ok(());
        }

        // Extract features → map to plugin slugs
        let features = json.get("features")
            .and_then(|f| f.as_object())
            .map(|o| o.keys().cloned().collect::<Vec<_>>())
            .unwrap_or_default();

        // Map common devcontainer feature patterns to Koda plugin slugs
        let plugin_slugs: Vec<&str> = features.iter()
            .flat_map(|f| {
                let lower = f.to_lowercase();
                if lower.contains("jupyter") { vec!["jupyter"] }
                else if lower.contains("ssh") { vec!["ssh"] }
                else if lower.contains("code-server") { vec!["code-server"] }
                else { vec![] }
            })
            .collect();

        for slug in plugin_slugs {
            let def = sqlx::query!(
                "SELECT id FROM plugin_definitions WHERE slug = $1",
                slug,
            )
            .fetch_optional(&self.pool)
            .await?;

            if let Some(def) = def {
                sqlx::query!(
                    r#"INSERT INTO workspace_plugin_bindings (workspace_id, plugin_definition_id)
                       VALUES ($1, $2) ON CONFLICT DO NOTHING"#,
                    workspace_id,
                    def.id,
                )
                .execute(&self.pool)
                .await?;
                tracing::info!(workspace_id = %workspace_id, slug = slug, "auto-bound plugin from devcontainer.json");
            }
        }

        // Store parsed devcontainer config in workspace metadata
        sqlx::query!(
            r#"UPDATE workspaces SET metadata = jsonb_set(
                COALESCE(metadata, '{}'),
                '{devcontainer}',
                $1::jsonb
               ), updated_at = NOW()
               WHERE id = $2"#,
            json,
            workspace_id,
        )
        .execute(&self.pool)
        .await
        .ok(); // non-critical, ignore if metadata column doesn't exist

        Ok(())
    }

    async fn resolve_secret(&self, secret_ref_id: Uuid) -> anyhow::Result<String> {
        let row = sqlx::query!(
            "SELECT encrypted_value, encryption_key_id FROM secret_refs WHERE id = $1",
            secret_ref_id,
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("secret ref not found"))?;

        // AES-256-GCM decryption — key from env
        let key_hex = std::env::var("SECRET_ENCRYPTION_KEY")
            .unwrap_or_default();
        if key_hex.is_empty() {
            return Err(anyhow::anyhow!("SECRET_ENCRYPTION_KEY not set"));
        }

        let key_bytes = hex::decode(&key_hex)?;
        decrypt_aes_gcm(&key_bytes, &row.encrypted_value)
    }
}

fn decrypt_aes_gcm(key: &[u8], ciphertext_b64: &str) -> anyhow::Result<String> {
    use aes_gcm::{aead::Aead, Aes256Gcm, KeyInit, Nonce};

    let ct = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        ciphertext_b64,
    )?;
    if ct.len() < 12 {
        return Err(anyhow::anyhow!("ciphertext too short"));
    }

    let (nonce_bytes, ciphertext) = ct.split_at(12);
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| anyhow::anyhow!("invalid key: {e}"))?;
    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| anyhow::anyhow!("decryption failed: {e}"))?;

    Ok(String::from_utf8(plaintext)?)
}

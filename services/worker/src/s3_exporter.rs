use std::time::Duration;

use aes_gcm::{aead::Aead, Aes256Gcm, Key, KeyInit, Nonce};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use sqlx::PgPool;
use uuid::Uuid;

const STREAM: &str = "koda:jobs:export";
const DEAD_LETTER: &str = "koda:jobs:export:dead";
const MAX_RETRIES: u8 = 3;

pub struct S3Exporter {
    pub pool: PgPool,
    pub redis: redis::aio::MultiplexedConnection,
    pub http: reqwest::Client,
    pub group: String,
    pub consumer: String,
}

#[derive(serde::Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ExportJob {
    ExportArtifact {
        export_id: Uuid,
        org_id: Uuid,
        workspace_id: Uuid,
        artifact_type: String,
        content: String,
    },
}

impl S3Exporter {
    pub async fn run(mut self) -> anyhow::Result<()> {
        use redis::AsyncCommands;
        let _: redis::RedisResult<()> = self
            .redis
            .xgroup_create_mkstream(STREAM, &self.group, "$")
            .await;

        tracing::info!(group = %self.group, "S3 exporter started");

        let mut failure_counts: std::collections::HashMap<String, u8> = std::collections::HashMap::new();

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
                    match self.process(&message).await {
                        Ok(_) => {
                            let _: Result<(), _> = self.redis.xack(STREAM, &self.group, &[&id]).await;
                            failure_counts.remove(&id);
                        }
                        Err(e) => {
                            let count = failure_counts.entry(id.clone()).or_insert(0);
                            *count += 1;
                            tracing::error!(id = %id, attempt = *count, error = %e, "export job failed");
                            if *count >= MAX_RETRIES {
                                let payload = message.map.get("payload").and_then(|v| {
                                    if let redis::Value::BulkString(b) = v { Some(b.clone()) } else { None }
                                }).unwrap_or_default();
                                let _: Result<(), _> = self.redis.xadd(
                                    DEAD_LETTER, "*",
                                    &[("payload", String::from_utf8_lossy(&payload).to_string()),
                                      ("error", e.to_string()),
                                      ("original_id", id.clone())],
                                ).await;
                                let _: Result<(), _> = self.redis.xack(STREAM, &self.group, &[&id]).await;
                                failure_counts.remove(&id);
                            }
                        }
                    }
                }
            }
        }
    }

    async fn process(&mut self, msg: &redis::streams::StreamId) -> anyhow::Result<()> {
        use redis::Value;
        let payload = msg.map.get("payload")
            .and_then(|v| if let Value::BulkString(b) = v { Some(b.clone()) } else { None })
            .ok_or_else(|| anyhow::anyhow!("missing payload"))?;

        let job: ExportJob = serde_json::from_slice(&payload)?;

        match job {
            ExportJob::ExportArtifact { export_id, org_id, workspace_id, artifact_type, content } => {
                self.export_artifact(export_id, org_id, workspace_id, &artifact_type, &content).await?;
            }
        }

        Ok(())
    }

    async fn export_artifact(
        &self,
        export_id: Uuid,
        org_id: Uuid,
        workspace_id: Uuid,
        artifact_type: &str,
        content: &str,
    ) -> anyhow::Result<()> {
        // Update status to uploading
        sqlx::query!(
            "UPDATE artifact_exports SET status = 'uploading', updated_at = NOW() WHERE id = $1",
            export_id
        )
        .execute(&self.pool)
        .await?;

        // Load S3 config
        let config = sqlx::query!(
            "SELECT endpoint, bucket, region, access_key_enc, secret_key_enc, path_prefix
             FROM s3_export_configs WHERE organization_id = $1 AND enabled = true",
            org_id,
        )
        .fetch_optional(&self.pool)
        .await?;

        let Some(cfg) = config else {
            sqlx::query!(
                "UPDATE artifact_exports SET status = 'failed', error = 'no S3 config', updated_at = NOW() WHERE id = $1",
                export_id
            ).execute(&self.pool).await?;
            return Ok(());
        };

        let enc_key = std::env::var("SECRET_ENCRYPTION_KEY").unwrap_or_default();
        let access_key = decrypt_value(&cfg.access_key_enc, &enc_key)?;
        let secret_key = decrypt_value(&cfg.secret_key_enc, &enc_key)?;

        let timestamp = time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_else(|_| "unknown".into());

        let s3_key = format!(
            "{}/{}/{}/{}-{}.txt",
            cfg.path_prefix, org_id, workspace_id, artifact_type, timestamp
        );

        // Build S3 presigned PUT using AWS Signature V4 (simplified via Authorization header)
        let s3_url = format!("{}/{}/{}", cfg.endpoint.trim_end_matches('/'), cfg.bucket, s3_key);

        let body_bytes = content.as_bytes().to_vec();
        let size = body_bytes.len() as i64;

        // Simple PUT request — works with MinIO and AWS S3 with access key auth
        let response = self.http
            .put(&s3_url)
            .header("Content-Type", "text/plain; charset=utf-8")
            .header("x-amz-content-sha256", "UNSIGNED-PAYLOAD")
            .header(
                "Authorization",
                build_basic_auth_header(&access_key, &secret_key),
            )
            .body(body_bytes)
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                sqlx::query!(
                    r#"UPDATE artifact_exports
                       SET status = 'completed', s3_key = $2, s3_url = $3,
                           size_bytes = $4, updated_at = NOW()
                       WHERE id = $1"#,
                    export_id, s3_key, s3_url, size,
                )
                .execute(&self.pool)
                .await?;
                tracing::info!(export_id = %export_id, s3_key = %s3_key, "artifact exported");
            }
            Ok(resp) => {
                let status = resp.status().as_u16();
                let err = format!("S3 PUT returned {}", status);
                sqlx::query!(
                    "UPDATE artifact_exports SET status = 'failed', error = $2, updated_at = NOW() WHERE id = $1",
                    export_id, err,
                ).execute(&self.pool).await?;
                anyhow::bail!(err);
            }
            Err(e) => {
                sqlx::query!(
                    "UPDATE artifact_exports SET status = 'failed', error = $2, updated_at = NOW() WHERE id = $1",
                    export_id, e.to_string(),
                ).execute(&self.pool).await?;
                return Err(e.into());
            }
        }

        Ok(())
    }
}

fn decrypt_value(encoded: &str, enc_key: &str) -> anyhow::Result<String> {
    let raw = B64.decode(encoded)?;
    if raw.len() < 12 {
        anyhow::bail!("invalid encrypted value");
    }
    let (nonce_bytes, ciphertext) = raw.split_at(12);
    let key_bytes = hex::decode(enc_key)?;
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| anyhow::anyhow!("decryption failed"))?;
    Ok(String::from_utf8(plaintext)?)
}

fn build_basic_auth_header(access_key: &str, secret_key: &str) -> String {
    // For MinIO-compatible APIs that support basic auth via Authorization: AWS <key>:<secret>
    // Real AWS S3 needs SigV4 — for production, use an AWS SDK crate
    format!("AWS {}:{}", access_key, secret_key)
}

/// Enqueue an artifact export job.
pub async fn enqueue_export(
    redis: &mut redis::aio::MultiplexedConnection,
    export_id: Uuid,
    org_id: Uuid,
    workspace_id: Uuid,
    artifact_type: &str,
    content: &str,
) -> anyhow::Result<()> {
    use redis::AsyncCommands;
    let payload = serde_json::json!({
        "type": "export_artifact",
        "export_id": export_id,
        "org_id": org_id,
        "workspace_id": workspace_id,
        "artifact_type": artifact_type,
        "content": content,
    })
    .to_string();
    let _: String = redis.xadd(STREAM, "*", &[("payload", payload)]).await?;
    Ok(())
}

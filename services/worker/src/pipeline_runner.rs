use anyhow::Context;
use redis::{aio::MultiplexedConnection, AsyncCommands};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

const STREAM: &str = "koda:jobs:pipeline";
const DEAD_LETTER: &str = "koda:jobs:pipeline:dead";

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum PipelineJob {
    RunPipeline {
        pipeline_id: Uuid,
        workspace_id: Uuid,
        org_id: Uuid,
        trigger: String,
    },
}

pub struct PipelineRunner {
    pub pool: PgPool,
    pub redis: MultiplexedConnection,
    pub group: String,
    pub consumer: String,
}

impl PipelineRunner {
    pub async fn init(&mut self) -> anyhow::Result<()> {
        let _: redis::RedisResult<()> = self
            .redis
            .xgroup_create_mkstream(STREAM, &self.group, "$")
            .await;
        Ok(())
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        self.init().await?;
        tracing::info!(group = %self.group, "pipeline runner started");

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
                    if let Err(e) = self.process(&message).await {
                        tracing::error!(msg_id = %id, error = %e, "pipeline job failed");
                        let _: Result<(), _> = self.redis.xack(STREAM, &self.group, &[&id]).await;
                        let _: Result<String, _> = self
                            .redis
                            .xadd(DEAD_LETTER, "*", &[("original_id", &id)])
                            .await;
                    } else {
                        let _: () = self.redis.xack(STREAM, &self.group, &[&id]).await?;
                    }
                }
            }
        }
    }

    async fn process(&mut self, msg: &redis::streams::StreamId) -> anyhow::Result<()> {
        let payload: String = msg
            .map
            .get("payload")
            .and_then(|v| match v {
                redis::Value::Data(b) => Some(String::from_utf8_lossy(b).into_owned()),
                redis::Value::BulkString(b) => Some(String::from_utf8_lossy(b).into_owned()),
                _ => None,
            })
            .context("missing payload")?;

        let job: PipelineJob = serde_json::from_str(&payload).context("deserialize pipeline job")?;

        match job {
            PipelineJob::RunPipeline {
                pipeline_id,
                workspace_id,
                org_id,
                trigger,
            } => self.run_pipeline(pipeline_id, workspace_id, org_id, trigger).await?,
        }

        Ok(())
    }

    async fn run_pipeline(
        &self,
        pipeline_id: Uuid,
        workspace_id: Uuid,
        _org_id: Uuid,
        trigger: String,
    ) -> anyhow::Result<()> {
        // Create a job record
        let job = sqlx::query!(
            r#"INSERT INTO jobs (pipeline_id, workspace_id, trigger_ref, status)
               VALUES ($1, $2, $3, 'running')
               RETURNING id"#,
            pipeline_id,
            workspace_id,
            trigger,
        )
        .fetch_one(&self.pool)
        .await?;

        tracing::info!(
            job_id = %job.id,
            pipeline_id = %pipeline_id,
            workspace_id = %workspace_id,
            "pipeline job started — execution TBD in Phase 2"
        );

        // Phase 1: mark as succeeded immediately (actual container exec in Phase 2)
        sqlx::query!(
            "UPDATE jobs SET status = 'succeeded', finished_at = NOW() WHERE id = $1",
            job.id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

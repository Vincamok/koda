use sqlx::PgPool;
use std::time::Duration;
use uuid::Uuid;

const GC_INTERVAL_SECS: u64 = 3600; // every hour
const PREWARM_INTERVAL_SECS: u64 = 86400; // every 24h
const ORPHAN_VOLUME_AGE_HOURS: i64 = 24;

#[derive(sqlx::FromRow)]
struct OrphanRow {
    id: Uuid,
    volume_name: String,
}

pub struct GarbageCollector {
    pub pool: PgPool,
    pub docker_host: String,
}

impl GarbageCollector {
    pub async fn run(self) -> anyhow::Result<()> {
        tracing::info!("garbage collector started");
        let gc = std::sync::Arc::new(self);

        let gc_clone = gc.clone();
        let gc_task = tokio::spawn(async move {
            loop {
                if let Err(e) = gc_clone.collect_orphaned_volumes().await {
                    tracing::error!(error = %e, "volume GC error");
                }
                tokio::time::sleep(Duration::from_secs(GC_INTERVAL_SECS)).await;
            }
        });

        let prewarm_task = tokio::spawn(async move {
            loop {
                if let Err(e) = gc.prewarm_images().await {
                    tracing::error!(error = %e, "image pre-warm error");
                }
                tokio::time::sleep(Duration::from_secs(PREWARM_INTERVAL_SECS)).await;
            }
        });

        tokio::try_join!(gc_task, prewarm_task)?;
        Ok(())
    }

    async fn collect_orphaned_volumes(&self) -> anyhow::Result<()> {
        // Find workspace volumes that belong to deleted workspaces or are detached > threshold
        let orphans = sqlx::query_as::<_, OrphanRow>(
            r#"SELECT wv.id, wv.volume_name
               FROM workspace_volumes wv
               LEFT JOIN workspaces w ON w.id = wv.workspace_id
               WHERE (w.id IS NULL OR w.status = 'closed')
                 AND wv.status != 'deleted'
                 AND wv.updated_at < NOW() - ($1 || ' hours')::INTERVAL"#,
        )
        .bind(ORPHAN_VOLUME_AGE_HOURS.to_string())
        .fetch_all(&self.pool)
        .await?;

        if orphans.is_empty() {
            return Ok(());
        }

        tracing::info!(count = orphans.len(), "found orphaned volumes");

        for orphan in &orphans {
            tracing::info!(volume = %orphan.volume_name, "marking orphaned volume as deleted");
            sqlx::query!(
                "UPDATE workspace_volumes SET status = 'deleted', updated_at = NOW() WHERE id = $1",
                orphan.id,
            )
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    async fn prewarm_images(&self) -> anyhow::Result<()> {
        // Pre-pull commonly used pipeline images so pipelines start fast
        let images = [
            "rust:1.79-slim",
            "node:20-slim",
            "debian:bookworm-slim",
        ];

        for image in &images {
            tracing::debug!(image = %image, "pre-warming docker image");
            let result = std::process::Command::new("docker")
                .args(["pull", image])
                .env("DOCKER_HOST", &self.docker_host)
                .output();

            match result {
                Ok(out) if out.status.success() => {
                    tracing::info!(image = %image, "image pre-warmed");
                }
                Ok(out) => {
                    tracing::warn!(
                        image = %image,
                        stderr = %String::from_utf8_lossy(&out.stderr),
                        "image pre-warm failed"
                    );
                }
                Err(e) => {
                    tracing::warn!(image = %image, error = %e, "docker not available for pre-warm");
                }
            }
        }

        Ok(())
    }
}

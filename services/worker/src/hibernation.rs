use std::time::Duration;

use bollard::{container::StopContainerOptions, Docker};
use sqlx::PgPool;

const CHECK_INTERVAL_SECS: u64 = 120;
const DEFAULT_IDLE_MINUTES: i32 = 30;

pub struct HibernationWatcher {
    pub pool: PgPool,
    pub docker_host: String,
}

impl HibernationWatcher {
    pub async fn run(self) -> anyhow::Result<()> {
        tracing::info!("hibernation watcher started — checking every {}s", CHECK_INTERVAL_SECS);
        loop {
            if let Err(e) = self.tick().await {
                tracing::error!(error = %e, "hibernation tick error");
            }
            tokio::time::sleep(Duration::from_secs(CHECK_INTERVAL_SECS)).await;
        }
    }

    async fn tick(&self) -> anyhow::Result<()> {
        // Find running workspaces whose last_activity_at exceeds the org threshold
        let rows = sqlx::query!(
            r#"
            SELECT w.id,
                   w.uid,
                   w.organization_id,
                   COALESCE(hc.idle_threshold_minutes, $1) AS idle_threshold_minutes,
                   COALESCE(hc.enabled, true) AS enabled
            FROM workspaces w
            LEFT JOIN workspace_hibernation_configs hc
                   ON hc.organization_id = w.organization_id
            WHERE w.status = 'running'
              AND COALESCE(hc.enabled, true) = true
              AND w.last_activity_at IS NOT NULL
              AND w.last_activity_at < NOW() - (COALESCE(hc.idle_threshold_minutes, $1) * INTERVAL '1 minute')
            "#,
            DEFAULT_IDLE_MINUTES,
        )
        .fetch_all(&self.pool)
        .await?;

        if rows.is_empty() {
            return Ok(());
        }

        let docker = connect_docker(&self.docker_host)?;

        for row in rows {
            let container_name = format!("ws-{}", row.id);
            tracing::info!(
                workspace_id = %row.id,
                uid = %row.uid,
                idle_minutes = row.idle_threshold_minutes,
                "hibernating idle workspace"
            );

            // Stop the container gracefully
            let stop_result = docker
                .stop_container(&container_name, Some(StopContainerOptions { t: 30 }))
                .await;

            if let Err(e) = stop_result {
                tracing::warn!(workspace_id = %row.id, error = %e, "failed to stop container for hibernation");
                continue;
            }

            // Update workspace status to 'stopped'
            if let Err(e) = sqlx::query!(
                "UPDATE workspaces SET status = 'stopped', updated_at = NOW() WHERE id = $1",
                row.id,
            )
            .execute(&self.pool)
            .await
            {
                tracing::error!(workspace_id = %row.id, error = %e, "failed to update workspace status after hibernation");
            }

            // Emit audit event
            let _ = sqlx::query!(
                r#"INSERT INTO audit_events (actor_id, organization_id, action, resource_type, resource_id, metadata)
                   VALUES (NULL, $1, 'workspace.hibernated', 'workspace', $2, $3)"#,
                row.organization_id,
                row.id.to_string(),
                serde_json::json!({ "reason": "idle_timeout", "idle_threshold_minutes": row.idle_threshold_minutes }),
            )
            .execute(&self.pool)
            .await;
        }

        Ok(())
    }
}

fn connect_docker(docker_host: &str) -> anyhow::Result<Docker> {
    if docker_host.starts_with("tcp://") || docker_host.starts_with("http://") {
        Ok(Docker::connect_with_http(docker_host, 30, bollard::API_DEFAULT_VERSION)?)
    } else {
        let path = docker_host.trim_start_matches("unix://");
        Ok(Docker::connect_with_unix(path, 30, bollard::API_DEFAULT_VERSION)?)
    }
}

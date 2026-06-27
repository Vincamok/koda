use std::time::Duration;

use reqwest::Client;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(sqlx::FromRow)]
struct BindingRow {
    id: Uuid,
    workspace_id: Uuid,
    health_check_url: Option<String>,
    plugin_name: String,
}

/// Periodically probe all active plugin bindings and update their health status.
pub struct PluginProber {
    pub pool: PgPool,
    pub http: Client,
    pub interval: Duration,
}

impl PluginProber {
    pub async fn run(&self) -> anyhow::Result<()> {
        loop {
            if let Err(e) = self.probe_all().await {
                tracing::error!(error = %e, "plugin probe cycle failed");
            }
            tokio::time::sleep(self.interval).await;
        }
    }

    async fn probe_all(&self) -> anyhow::Result<()> {
        let bindings = sqlx::query_as::<_, BindingRow>(
            r#"SELECT wpb.id, wpb.workspace_id, pd.health_check_url, pd.name as plugin_name
               FROM workspace_plugin_bindings wpb
               JOIN plugin_definitions pd ON pd.id = wpb.plugin_definition_id
               WHERE wpb.status = 'active'
                 AND pd.health_check_url IS NOT NULL"#,
        )
        .fetch_all(&self.pool)
        .await?;

        for binding in bindings {
            if let Some(url) = binding.health_check_url {
                let healthy = self.probe_url(&url).await;
                let status = if healthy { "healthy" } else { "unhealthy" };
                sqlx::query!(
                    "UPDATE workspace_plugin_bindings SET health_status = $1, last_probed_at = NOW() WHERE id = $2",
                    status,
                    binding.id,
                )
                .execute(&self.pool)
                .await?;

                if !healthy {
                    tracing::warn!(
                        binding_id = %binding.id,
                        plugin = %binding.plugin_name,
                        workspace_id = %binding.workspace_id,
                        "plugin health check failed"
                    );
                }
            }
        }

        Ok(())
    }

    async fn probe_url(&self, url: &str) -> bool {
        match self.http.get(url).send().await {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }
}

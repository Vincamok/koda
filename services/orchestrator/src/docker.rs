use std::collections::HashMap;

use anyhow::{bail, Context};
use bollard::{
    container::{Config, CreateContainerOptions, InspectContainerOptions, StartContainerOptions, StopContainerOptions, RemoveContainerOptions},
    models::{HostConfig, Resources},
    network::{ConnectNetworkOptions, CreateNetworkOptions},
    volume::CreateVolumeOptions,
    Docker,
};
use uuid::Uuid;

pub struct DockerManager {
    docker: Docker,
    pub network: String,
    pub workspace_image: String,
    pub personal_volume_prefix: String,
}

impl DockerManager {
    pub fn new(
        socket_path: &str,
        network: impl Into<String>,
        workspace_image: impl Into<String>,
        personal_volume_prefix: impl Into<String>,
    ) -> anyhow::Result<Self> {
        let docker = if socket_path.starts_with("unix://") {
            Docker::connect_with_unix(
                socket_path.trim_start_matches("unix://"),
                120,
                bollard::API_DEFAULT_VERSION,
            )?
        } else {
            Docker::connect_with_local_defaults()?
        };
        Ok(Self {
            docker,
            network: network.into(),
            workspace_image: workspace_image.into(),
            personal_volume_prefix: personal_volume_prefix.into(),
        })
    }

    /// Ensure the workspace overlay network exists.
    pub async fn ensure_network(&self) -> anyhow::Result<()> {
        match self.docker.inspect_network::<&str>(&self.network, None).await {
            Ok(_) => Ok(()),
            Err(_) => {
                self.docker
                    .create_network(CreateNetworkOptions {
                        name: self.network.clone(),
                        driver: "bridge".to_string(),
                        ..Default::default()
                    })
                    .await
                    .context("create workspace network")?;
                tracing::info!(network = %self.network, "created workspace network");
                Ok(())
            }
        }
    }

    /// Get or create the personal-space volume for a user.
    pub async fn ensure_personal_volume(&self, user_id: Uuid) -> anyhow::Result<String> {
        let volume_name = format!("{}-{}", self.personal_volume_prefix, user_id);
        match self.docker.inspect_volume(&volume_name).await {
            Ok(_) => {}
            Err(_) => {
                self.docker
                    .create_volume(CreateVolumeOptions {
                        name: volume_name.clone(),
                        labels: HashMap::from([
                            ("koda.managed".to_string(), "true".to_string()),
                            ("koda.type".to_string(), "personal".to_string()),
                            ("koda.user_id".to_string(), user_id.to_string()),
                        ]),
                        ..Default::default()
                    })
                    .await
                    .context("create personal volume")?;
                tracing::info!(volume = %volume_name, "created personal volume");
            }
        }
        Ok(volume_name)
    }

    /// Start a workspace container.
    pub async fn start_workspace(
        &self,
        workspace_id: Uuid,
        org_id: Uuid,
        user_id: Uuid,
        cpu_limit: i32,
        ram_limit_mb: i32,
        pids_limit: i32,
        env_vars: Vec<String>,
    ) -> anyhow::Result<String> {
        let container_name = format!("ws-{}", workspace_id);
        let personal_volume = self.ensure_personal_volume(user_id).await?;
        let workspace_volume = format!("koda-ws-{}", workspace_id);

        // Required security labels
        let labels = HashMap::from([
            ("koda.managed".to_string(), "true".to_string()),
            ("koda.type".to_string(), "workspace".to_string()),
            ("koda.workspace_id".to_string(), workspace_id.to_string()),
            ("koda.org_id".to_string(), org_id.to_string()),
        ]);

        // Resource limits — always enforced
        let cpu_period: i64 = 100_000;
        let cpu_quota: i64 = (cpu_limit as i64) * cpu_period;
        let memory: i64 = (ram_limit_mb as i64) * 1024 * 1024;

        let host_config = HostConfig {
            cpu_period: Some(cpu_period),
            cpu_quota: Some(cpu_quota),
            memory: Some(memory),
            pids_limit: Some(pids_limit as i64),
            network_mode: Some(self.network.clone()),
            binds: Some(vec![
                format!("{workspace_volume}:/workspace"),
                format!("{personal_volume}:/personal:ro"),
            ]),
            // No --privileged, no extra capabilities
            cap_drop: Some(vec!["ALL".to_string()]),
            security_opt: Some(vec!["no-new-privileges:true".to_string()]),
            ..Default::default()
        };

        let config = Config {
            image: Some(self.workspace_image.clone()),
            labels: Some(labels),
            env: Some(env_vars),
            host_config: Some(host_config),
            ..Default::default()
        };

        // Remove stale container if exists
        let _ = self
            .docker
            .remove_container(
                &container_name,
                Some(RemoveContainerOptions { force: true, ..Default::default() }),
            )
            .await;

        let create_resp = self
            .docker
            .create_container(
                Some(CreateContainerOptions {
                    name: container_name.clone(),
                    platform: None,
                }),
                config,
            )
            .await
            .context("create workspace container")?;

        self.docker
            .start_container(&container_name, None::<StartContainerOptions<String>>)
            .await
            .context("start workspace container")?;

        tracing::info!(
            workspace_id = %workspace_id,
            container_id = %create_resp.id,
            "workspace container started"
        );

        Ok(create_resp.id)
    }

    /// Stop a workspace container gracefully (10s timeout then kill).
    pub async fn stop_workspace(&self, workspace_id: Uuid) -> anyhow::Result<()> {
        let container_name = format!("ws-{}", workspace_id);
        self.docker
            .stop_container(
                &container_name,
                Some(StopContainerOptions { t: 10 }),
            )
            .await
            .context("stop workspace container")?;
        tracing::info!(workspace_id = %workspace_id, "workspace container stopped");
        Ok(())
    }

    /// Remove a workspace container and its volume.
    pub async fn delete_workspace(&self, workspace_id: Uuid) -> anyhow::Result<()> {
        let container_name = format!("ws-{}", workspace_id);
        let _ = self
            .docker
            .remove_container(
                &container_name,
                Some(RemoveContainerOptions { force: true, v: true, ..Default::default() }),
            )
            .await;

        // Remove workspace-specific volume
        let volume_name = format!("koda-ws-{}", workspace_id);
        let _ = self.docker.remove_volume(&volume_name, None).await;

        tracing::info!(workspace_id = %workspace_id, "workspace container + volume removed");
        Ok(())
    }

    /// Check if a workspace container is healthy/running.
    pub async fn is_running(&self, workspace_id: Uuid) -> anyhow::Result<bool> {
        let container_name = format!("ws-{}", workspace_id);
        match self
            .docker
            .inspect_container(&container_name, None::<InspectContainerOptions>)
            .await
        {
            Ok(info) => {
                let running = info
                    .state
                    .and_then(|s| s.running)
                    .unwrap_or(false);
                Ok(running)
            }
            Err(_) => Ok(false),
        }
    }
}

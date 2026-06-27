use figment::{providers::{Env, Format, Yaml}, Figment};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct OrchestratorConfig {
    pub database_url: String,
    pub redis_url: String,
    pub consumer_group: String,
    pub consumer_name: String,
    pub docker_socket: String,
    pub workspace_network: String,
    pub workspace_image: String,
    pub personal_volume_prefix: String,
    pub sozu_socket: Option<String>,
    pub base_domain: String,
    pub workspace_port: u16,
}

impl OrchestratorConfig {
    pub fn load() -> anyhow::Result<Self> {
        let _ = dotenvy::dotenv();
        Ok(Figment::new()
            .merge(Yaml::file("config/default.yaml"))
            .merge(Env::raw())
            .extract()?)
    }
}

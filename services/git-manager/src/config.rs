use figment::{providers::{Env, Format, Yaml}, Figment};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct GitManagerConfig {
    pub database_url: String,
    pub redis_url: String,
    pub consumer_group: String,
    pub consumer_name: String,
    pub workspace_volumes_base: String,
    pub clone_timeout_seconds: u64,
}

impl GitManagerConfig {
    pub fn load() -> anyhow::Result<Self> {
        let _ = dotenvy::dotenv();
        Ok(Figment::new()
            .merge(Yaml::file("config/default.yaml"))
            .merge(Env::raw())
            .extract()?)
    }
}

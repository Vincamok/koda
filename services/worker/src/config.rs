use figment::{providers::{Env, Format, Yaml}, Figment};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct WorkerConfig {
    pub database_url: String,
    pub redis_url: String,
    pub consumer_group: String,
    pub consumer_name: String,
    pub plugin_probe_interval_seconds: u64,
    pub http_timeout_seconds: u64,
    pub docker_host: String,
}

impl WorkerConfig {
    pub fn load() -> anyhow::Result<Self> {
        let _ = dotenvy::dotenv();
        Ok(Figment::new()
            .merge(Yaml::file("config/default.yaml"))
            .merge(Env::raw())
            .extract()?)
    }
}

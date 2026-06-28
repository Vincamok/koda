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
    pub anthropic_api_key: Option<String>,
    pub workspace_root: Option<String>,
    /// AES-256-GCM key (hex-encoded 32 bytes) for decrypting S3 credentials
    pub secret_encryption_key: Option<String>,
    /// Loki push API base URL (e.g. http://loki:3100). Empty = disabled.
    pub loki_url: Option<String>,
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

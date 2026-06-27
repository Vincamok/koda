use figment::{providers::{Env, Format, Yaml}, Figment};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct GatewayConfig {
    pub database_url: String,
    pub redis_url: String,
    pub consumer_group: String,
    pub consumer_name: String,
    pub sozu_socket: String,
    pub base_domain: String,
}

impl GatewayConfig {
    pub fn load() -> anyhow::Result<Self> {
        let _ = dotenvy::dotenv();
        Ok(Figment::new()
            .merge(Yaml::file("config/default.yaml"))
            .merge(Env::raw())
            .extract()?)
    }
}

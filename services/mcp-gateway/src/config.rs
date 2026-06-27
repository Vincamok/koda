use figment::{Figment, providers::{Format, Yaml, Env}};
use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct Config {
    pub redis_url:                String,
    pub database_url:             String,
    pub worker_id:                String,
    pub dead_letter_max_attempts: u8,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        let cfg: Self = Figment::new()
            .merge(Yaml::file("config/default.yaml"))
            .merge(Env::raw())
            .extract()?;

        Ok(cfg)
    }
}

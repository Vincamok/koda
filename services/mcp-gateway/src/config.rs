#[derive(Clone)]
pub struct Config {
    pub redis_url:    String,
    pub database_url: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            redis_url:    std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".into()),
            database_url: std::env::var("DATABASE_URL")?,
        })
    }
}

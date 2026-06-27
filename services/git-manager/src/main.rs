mod cloner;
mod config;
mod worker;

use std::time::Duration;

use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{config::GitManagerConfig, worker::GitWorker};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "koda_git_manager=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    let config = GitManagerConfig::load()?;

    let pool = PgPoolOptions::new()
        .max_connections(3)
        .connect(&config.database_url)
        .await?;

    let redis_client = redis::Client::open(config.redis_url.as_str())?;
    let redis = redis_client.get_multiplexed_async_connection().await?;

    let mut worker = GitWorker {
        pool,
        redis,
        group: config.consumer_group,
        consumer: config.consumer_name,
        volumes_base: config.workspace_volumes_base,
        clone_timeout: Duration::from_secs(config.clone_timeout_seconds),
    };

    worker.run().await?;

    Ok(())
}

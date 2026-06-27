mod config;
mod pipeline_runner;
mod plugin_prober;

use std::time::Duration;

use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    config::WorkerConfig,
    pipeline_runner::PipelineRunner,
    plugin_prober::PluginProber,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "koda_worker=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    let config = WorkerConfig::load()?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;

    let redis_client = redis::Client::open(config.redis_url.as_str())?;
    let redis = redis_client.get_multiplexed_async_connection().await?;

    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(config.http_timeout_seconds))
        .user_agent("koda-worker/0.1")
        .build()?;

    let prober = PluginProber {
        pool: pool.clone(),
        http,
        interval: Duration::from_secs(config.plugin_probe_interval_seconds),
    };

    let mut runner = PipelineRunner {
        pool: pool.clone(),
        redis,
        group: config.consumer_group.clone(),
        consumer: config.consumer_name.clone(),
    };

    // Run prober and pipeline runner concurrently
    tokio::try_join!(
        prober.run(),
        runner.run(),
    )?;

    Ok(())
}

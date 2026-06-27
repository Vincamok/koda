mod config;
mod docker;
mod worker;

use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    config::OrchestratorConfig,
    docker::DockerManager,
    worker::OrchestratorWorker,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "koda_orchestrator=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    let config = OrchestratorConfig::load()?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;

    let redis_client = redis::Client::open(config.redis_url.as_str())?;
    let redis = redis_client.get_multiplexed_async_connection().await?;

    let docker = DockerManager::new(
        &config.docker_socket,
        &config.workspace_network,
        &config.workspace_image,
        &config.personal_volume_prefix,
    )?;

    let mut worker = OrchestratorWorker {
        pool,
        redis,
        docker,
        group: config.consumer_group,
        consumer: config.consumer_name,
    };

    worker.run().await?;

    Ok(())
}

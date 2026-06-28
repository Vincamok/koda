mod alerting;
mod config;
mod cron_scheduler;
mod garbage_collector;
mod git_cloner;
mod hibernation;
mod loki_shipper;
mod pipeline_runner;
mod plugin_prober;
mod s3_exporter;

use std::time::Duration;

use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    alerting::AlertWatcher,
    config::WorkerConfig,
    cron_scheduler::CronScheduler,
    garbage_collector::GarbageCollector,
    git_cloner::GitCloner,
    hibernation::HibernationWatcher,
    loki_shipper::LokiShipper,
    pipeline_runner::PipelineRunner,
    plugin_prober::PluginProber,
    s3_exporter::S3Exporter,
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

    let runner_http = reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .user_agent("koda-worker/0.1")
        .build()?;

    let mut runner = PipelineRunner {
        pool: pool.clone(),
        redis,
        group: config.consumer_group.clone(),
        consumer: config.consumer_name.clone(),
        http: runner_http,
        docker_host: config.docker_host.clone(),
        anthropic_api_key: config.anthropic_api_key.clone(),
    };

    let redis2 = redis_client.get_multiplexed_async_connection().await?;

    let mut cloner = GitCloner {
        pool: pool.clone(),
        redis: redis2,
        group: config.consumer_group.clone(),
        consumer: format!("{}-git", config.consumer_name),
        workspace_root: config.workspace_root.clone().unwrap_or_else(|| "/data/workspaces".into()),
    };

    let cron = CronScheduler {
        pool: pool.clone(),
    };

    let gc = GarbageCollector {
        pool: pool.clone(),
        docker_host: config.docker_host.clone(),
    };

    let hibernation = HibernationWatcher {
        pool: pool.clone(),
        docker_host: config.docker_host.clone(),
    };

    let alert_http = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("koda-worker/0.1")
        .build()?;

    let alerter = AlertWatcher {
        pool: pool.clone(),
        http: alert_http,
    };

    let redis3 = redis_client.get_multiplexed_async_connection().await?;
    let s3_http = reqwest::Client::builder()
        .timeout(Duration::from_secs(300))
        .user_agent("koda-worker/0.1")
        .build()?;

    let exporter = S3Exporter {
        pool: pool.clone(),
        redis: redis3,
        http: s3_http,
        group: config.consumer_group.clone(),
        consumer: format!("{}-s3", config.consumer_name),
    };

    let loki_http = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .user_agent("koda-worker/0.1")
        .build()?;

    let loki = LokiShipper {
        pool: pool.clone(),
        http: loki_http,
        loki_url: config.loki_url.clone().unwrap_or_default(),
    };

    tokio::try_join!(
        prober.run(),
        runner.run(),
        cloner.run(),
        cron.run(),
        gc.run(),
        hibernation.run(),
        alerter.run(),
        exporter.run(),
        loki.run(),
    )?;

    Ok(())
}

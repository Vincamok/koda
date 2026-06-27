mod config;
mod sozu_client;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::GatewayConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "koda_gateway=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    let config = GatewayConfig::load()?;

    tracing::info!(
        sozu_socket = %config.sozu_socket,
        base_domain = %config.base_domain,
        "gateway service starting"
    );

    // Gateway is invoked by orchestrator via sozu_client directly.
    // This binary runs as a sidecar and provides a health endpoint.
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        tracing::debug!("gateway heartbeat");
    }
}

use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use koda_api::{config::AppConfig, db, routes::build_router, AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "koda_api=debug,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    let config = AppConfig::load()?;

    let pool = db::create_pool(&config.database_url).await?;
    db::run_migrations(&pool).await?;
    tracing::info!("migrations applied");

    let http = reqwest::Client::builder()
        .user_agent("koda-api/0.1")
        .build()?;

    let state = AppState { pool, config: config.clone(), http };

    let router = build_router(state);

    let addr: SocketAddr = config.listen_addr.parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(%addr, "listening");

    axum::serve(
        listener,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}

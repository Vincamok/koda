use std::net::SocketAddr;

use opentelemetry::global;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{propagation::TraceContextPropagator, runtime};
use tower_sessions::{Expiry, SessionManagerLayer};
use tower_sessions_redis_store::RedisStore;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

use koda_api::{config::AppConfig, db, routes::build_router, AppState};

fn init_telemetry(config: &AppConfig) -> Option<opentelemetry_sdk::trace::Tracer> {
    let endpoint = config.otel_endpoint.as_deref().unwrap_or("");
    if endpoint.is_empty() {
        return None;
    }

    global::set_text_map_propagator(TraceContextPropagator::new());

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .http()
                .with_endpoint(endpoint),
        )
        .with_trace_config(
            opentelemetry_sdk::trace::config()
                .with_resource(opentelemetry_sdk::Resource::new(vec![
                    opentelemetry::KeyValue::new("service.name", "koda-api"),
                    opentelemetry::KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
                ])),
        )
        .install_batch(runtime::Tokio)
        .ok()?;

    Some(tracer)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::load()?;

    // ── Sentry (optional) ────────────────────────────────────────────────────
    let _sentry_guard = config.sentry_dsn.as_deref().filter(|d| !d.is_empty()).map(|dsn| {
        sentry::init((
            dsn,
            sentry::ClientOptions {
                release: sentry::release_name!(),
                traces_sample_rate: 0.2,
                ..Default::default()
            },
        ))
    });

    // ── OpenTelemetry OTLP tracer ────────────────────────────────────────────
    let otel_tracer = init_telemetry(&config);

    // ── Tracing subscriber ───────────────────────────────────────────────────
    let fmt_layer = tracing_subscriber::fmt::layer()
        .json()
        .boxed();

    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "koda_api=debug,tower_http=info".into());

    let registry = tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer);

    if let Some(tracer) = otel_tracer {
        registry
            .with(OpenTelemetryLayer::new(tracer))
            .init();
    } else {
        registry.init();
    }

    // ── Database ─────────────────────────────────────────────────────────────
    let pool = db::create_pool(&config.database_url).await?;
    db::run_migrations(&pool).await?;
    tracing::info!("migrations applied");

    // ── HTTP client ───────────────────────────────────────────────────────────
    let http = reqwest::Client::builder()
        .user_agent("koda-api/0.1")
        .build()?;

    // ── Redis ─────────────────────────────────────────────────────────────────
    let redis_client_jobs = redis::Client::open(config.redis_url.as_str())?;
    let redis_conn = redis_client_jobs.get_multiplexed_async_connection().await?;

    let state = AppState { pool, config: config.clone(), http, redis: redis_conn, redis_client: redis_client_jobs.clone() };

    // ── Session store ─────────────────────────────────────────────────────────
    let redis_client = redis::Client::open(config.redis_url.as_str())?;
    let session_store = RedisStore::new(redis_client);
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnSessionEnd);

    // ── Router ────────────────────────────────────────────────────────────────
    let router = build_router(state, session_layer);

    let addr: SocketAddr = config.listen_addr.parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(%addr, "listening");

    axum::serve(
        listener,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    global::shutdown_tracer_provider();
    Ok(())
}

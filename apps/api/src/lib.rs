pub mod ai;
pub mod config;
pub mod db;
pub mod error;
pub mod handlers;
pub mod jobs;
pub mod middleware;
pub mod models;
pub mod routes;

use axum::extract::FromRef;
use config::AppConfig;
use redis::aio::MultiplexedConnection;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: AppConfig,
    pub http: reqwest::Client,
    pub redis: MultiplexedConnection,
}

impl FromRef<AppState> for PgPool {
    fn from_ref(state: &AppState) -> Self {
        state.pool.clone()
    }
}

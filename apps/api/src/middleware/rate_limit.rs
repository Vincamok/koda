use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{Request, Response, StatusCode},
    middleware::Next,
};
use redis::{aio::MultiplexedConnection, AsyncCommands};
use std::net::SocketAddr;

use crate::{error::ErrorBody, AppState};

const WINDOW_SECONDS: i64 = 60;
const MAX_PER_IP: i64 = 300;
const MAX_PER_USER: i64 = 600;

pub async fn rate_limit_middleware(
    axum::extract::State(state): axum::extract::State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request<Body>,
    next: Next,
) -> Result<Response<Body>, StatusCode> {
    let ip = addr.ip().to_string();
    let user_id = request
        .extensions()
        .get::<crate::middleware::auth::AuthUser>()
        .map(|u| u.id.to_string());

    let mut redis = state.redis.clone();

    // IP-based limiting
    if !check_limit(&mut redis, &format!("rl:ip:{ip}"), MAX_PER_IP, WINDOW_SECONDS).await {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    // User-based limiting (if authenticated)
    if let Some(uid) = user_id {
        if !check_limit(&mut redis, &format!("rl:user:{uid}"), MAX_PER_USER, WINDOW_SECONDS).await {
            return Err(StatusCode::TOO_MANY_REQUESTS);
        }
    }

    Ok(next.run(request).await)
}

async fn check_limit(redis: &mut MultiplexedConnection, key: &str, limit: i64, window: i64) -> bool {
    let count: i64 = redis.incr(key, 1).await.unwrap_or(0);
    if count == 1 {
        let _: () = redis.expire(key, window).await.unwrap_or(());
    }
    count <= limit
}

// Used by error.rs for the Too Many Requests response
#[derive(serde::Serialize)]
pub struct RateLimitError {
    pub error: ErrorBody,
}

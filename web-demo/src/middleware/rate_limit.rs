use axum::{
    extract::{ConnectInfo, Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use redis::AsyncCommands;
use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::instrument;

use crate::state::AppState;

#[instrument(skip(state, addr, request, next))]
pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if std::env::var("DISABLE_RATE_LIMIT").is_ok() {
        return Ok(next.run(request).await);
    }
    let ip = addr.ip().to_string();
    let session_id = request
        .extensions()
        .get::<String>()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)
        .inspect_err(|e| {
            tracing::error!("Session ID not found: {}", e);
        })?
        .clone();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Check rate limits
    check_ip_limit(&state.redis, &ip, now)
        .await
        .inspect_err(|e| {
            tracing::error!("IP rate limit error: {}", e);
        })?;
    check_session_limit(&state.redis, &session_id, now)
        .await
        .inspect_err(|e| {
            tracing::error!("Session rate limit error: {}", e);
        })?;
    check_global_limit(&state.redis, now)
        .await
        .inspect_err(|e| {
            tracing::error!("Global rate limit error: {}", e);
        })?;

    Ok(next.run(request).await)
}

#[instrument(skip(redis_client))]
async fn check_ip_limit(
    redis_client: &redis::Client,
    ip: &str,
    now: u64,
) -> Result<(), StatusCode> {
    let mut conn = redis_client
        .get_multiplexed_async_connection()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let key = format!("rate_limit:ip:{}", ip);
    let window_start = now - 60; // 1 minute window

    // Remove old entries
    let _: () = conn
        .zrembyscore(&key, 0, window_start as isize)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Count current requests
    let count: isize = conn
        .zcard(&key)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if count >= 10 {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    // Add current request
    let _: () = conn
        .zadd(&key, now as isize, now)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Set expiry
    let _: () = conn
        .expire(&key, 60)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(())
}

#[instrument(skip(redis_client))]
async fn check_session_limit(
    redis_client: &redis::Client,
    session_id: &str,
    now: u64,
) -> Result<(), StatusCode> {
    let mut conn = redis_client
        .get_multiplexed_async_connection()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let key = format!("rate_limit:session:{}", session_id);
    let window_start = now - 3600; // 1 hour window

    // Remove old entries
    let _: () = conn
        .zrembyscore(&key, 0, window_start as isize)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Count current requests
    let count: isize = conn
        .zcard(&key)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if count >= 20 {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    // Add current request
    let _: () = conn
        .zadd(&key, now as isize, now)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Set expiry
    let _: () = conn
        .expire(&key, 3600)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(())
}

#[instrument(skip(redis_client))]
async fn check_global_limit(redis_client: &redis::Client, now: u64) -> Result<(), StatusCode> {
    let mut conn = redis_client
        .get_multiplexed_async_connection()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let key = "rate_limit:global";
    let window_start = now - 3600; // 1 hour window

    // Remove old entries
    let _: () = conn
        .zrembyscore(key, 0, window_start as isize)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Count current requests
    let count: isize = conn
        .zcard(key)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if count >= 1000 {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    // Add current request
    let _: () = conn
        .zadd(key, now as isize, now)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Set expiry
    let _: () = conn
        .expire(key, 3600)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(())
}

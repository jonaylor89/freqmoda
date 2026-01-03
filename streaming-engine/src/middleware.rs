use crate::state::AppStateDyn;
use crate::streamingpath::hasher::{suffix_result_storage_hasher, verify_hash};
use crate::streamingpath::params::Params;
use axum::http::{HeaderValue, Response, StatusCode, header};
use axum::{
    body::{Body, to_bytes},
    extract::{Request, State},
    middleware::Next,
    response::IntoResponse,
};
use bytes::Bytes;
use std::time::Duration;
use tracing::debug;

const CACHE_KEY_PREFIX: &str = "req_cache:";
const META_CACHE_KEY_PREFIX: &str = "meta_cache:";
const CACHE_TTL: Duration = Duration::from_secs(3600); // 1 hour

#[tracing::instrument(skip(state, req, next))]
pub async fn cache_middleware(
    State(state): State<AppStateDyn>,
    params: Params,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let params_hash = suffix_result_storage_hasher(&params);

    let path = params.to_string();
    let cache_key_prefix = if path.starts_with("/meta") {
        META_CACHE_KEY_PREFIX
    } else {
        CACHE_KEY_PREFIX
    };

    let cache_key = format!("{}:{}:{}", cache_key_prefix, req.method(), params_hash);

    debug!("Cache key: {}", cache_key);
    let cache_response = state.cache.get(&cache_key).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to get cache: {}", e),
        )
    })?;
    if let Some(buf) = cache_response {
        // Return cached response if available
        let content_type = infer::get(&buf)
            .map(|mime| mime.to_string())
            .unwrap_or("audio/mpeg".to_string());
        let total_size = buf.len();

        debug!("Cache hit key={}", cache_key);
        let headers = req.headers();

        // Handle range request
        if let Some(range) = headers.get(header::RANGE)
            && let Ok(range_str) = range.to_str()
                && let Some(range_val) = range_str.strip_prefix("bytes=") {
                    let (start, end) = parse_range(range_val, total_size);
                    let length = end - start + 1;

                    let content = Bytes::copy_from_slice(&buf[start..=end]);

                    let res = Response::builder()
                        .status(StatusCode::PARTIAL_CONTENT)
                        .header(header::CONTENT_TYPE, content_type)
                        .header(header::ACCEPT_RANGES, "bytes")
                        .header(header::CONTENT_LENGTH, length.to_string())
                        .header(
                            header::CONTENT_RANGE,
                            format!("bytes {}-{}/{}", start, end, total_size),
                        )
                        .header(header::CACHE_CONTROL, "no-cache")
                        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                        .header(
                            header::CONTENT_DISPOSITION,
                            HeaderValue::from_static("inline"),
                        )
                        .body(Body::from(content))
                        .map_err(|e| {
                            (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                format!("Failed to build response: {}", e),
                            )
                        })?;

                    return Ok(res);
                }

        // Return full content if no range request
        let res = Response::builder()
            .header(header::CONTENT_TYPE, content_type)
            .header(header::ACCEPT_RANGES, "bytes")
            .header(header::CONTENT_LENGTH, total_size.to_string())
            .header(header::CACHE_CONTROL, "no-cache")
            .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
            .header(
                header::CONTENT_DISPOSITION,
                HeaderValue::from_static("inline"),
            )
            .body(Body::from(buf))
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to build response: {}", e),
                )
            })?;

        return Ok(res);
    }

    // If not cached, proceed with the request
    let response = next.run(req).await;
    if response.status() != StatusCode::OK {
        return Ok(response);
    }

    // Cache the response
    let (parts, body) = response.into_parts();
    let bytes = to_bytes(body, usize::MAX).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to read response body: {}", e),
        )
    })?;

    let _ = state
        .cache
        .set(&cache_key, bytes.as_ref(), Some(CACHE_TTL))
        .await;

    Ok(Response::from_parts(parts, Body::from(bytes)))
}

pub async fn auth_middleware(
    State(_): State<AppStateDyn>,
    params: Params,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let path = params.to_string();

    let hash = req
        .uri()
        .path()
        .trim_start_matches("/meta")
        .strip_prefix("/")
        .and_then(|s| s.split("/").next())
        .ok_or((
            StatusCode::BAD_REQUEST,
            "Failed to parse URI hash".to_string(),
        ))?;

    if hash != "unsafe" {
        verify_hash(hash.to_owned().into(), path.to_owned().into()).map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                format!("Failed to verify hash: {}", e),
            )
        })?;
    }

    Ok(next.run(req).await)
}

fn parse_range(range: &str, total_size: usize) -> (usize, usize) {
    let mut parts = range.split('-');
    let start = parts
        .next()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0);
    let end = parts
        .next()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(total_size - 1);
    (start, end.min(total_size - 1))
}

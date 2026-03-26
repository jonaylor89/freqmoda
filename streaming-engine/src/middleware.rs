use crate::state::AppStateDyn;
use crate::streamingpath::hasher::{suffix_result_storage_hasher, verify_hash};
use crate::streamingpath::params::Params;
use axum::http::{HeaderMap, HeaderValue, Response, StatusCode, header};
use axum::{
    body::{Body, Bytes, to_bytes},
    extract::{Request, State},
    middleware::Next,
    response::IntoResponse,
};
use std::time::Duration;
use tracing::debug;

const CACHE_KEY_PREFIX: &str = "req_cache:";
const META_CACHE_KEY_PREFIX: &str = "meta_cache:";
const CACHE_TTL: Duration = Duration::from_secs(3600); // 1 hour

#[tracing::instrument(skip(state, req, next))]
pub async fn cache_middleware(
    State(state): State<AppStateDyn>,
    params: Params,
    mut req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let params_hash = suffix_result_storage_hasher(&params);
    let request_headers = req.headers().clone();

    let uri_path = req.uri().path();
    let cache_key_prefix = if uri_path.starts_with("/meta") {
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
        let content_type = infer::get(&buf)
            .map(|mime| mime.to_string())
            .unwrap_or("audio/mpeg".to_string());
        debug!("Cache hit key={}", cache_key);
        return build_audio_response(&request_headers, &content_type, Bytes::from(buf));
    }

    // Cache the full response body, then apply range handling locally so first
    // requests and cache hits behave the same way.
    req.headers_mut().remove(header::RANGE);
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

    let content_type = parts
        .headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned)
        .or_else(|| infer::get(bytes.as_ref()).map(|mime| mime.to_string()))
        .unwrap_or_else(|| "application/octet-stream".to_string());

    build_audio_response(&request_headers, &content_type, bytes)
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

fn build_audio_response(
    request_headers: &HeaderMap,
    content_type: &str,
    body: Bytes,
) -> Result<Response<Body>, (StatusCode, String)> {
    match parse_range(request_headers, body.len())? {
        Some((start, end)) => {
            let content = body.slice(start..=end);
            Response::builder()
                .status(StatusCode::PARTIAL_CONTENT)
                .header(header::CONTENT_TYPE, content_type)
                .header(header::ACCEPT_RANGES, "bytes")
                .header(header::CONTENT_LENGTH, content.len().to_string())
                .header(
                    header::CONTENT_RANGE,
                    format!("bytes {}-{}/{}", start, end, body.len()),
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
                })
        }
        None => Response::builder()
            .header(header::CONTENT_TYPE, content_type)
            .header(header::ACCEPT_RANGES, "bytes")
            .header(header::CONTENT_LENGTH, body.len().to_string())
            .header(header::CACHE_CONTROL, "no-cache")
            .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
            .header(
                header::CONTENT_DISPOSITION,
                HeaderValue::from_static("inline"),
            )
            .body(Body::from(body))
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to build response: {}", e),
                )
            }),
    }
}

fn parse_range(
    request_headers: &HeaderMap,
    total_size: usize,
) -> Result<Option<(usize, usize)>, (StatusCode, String)> {
    let Some(range) = request_headers.get(header::RANGE) else {
        return Ok(None);
    };

    let range = range.to_str().map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            "Range header must be valid ASCII".to_string(),
        )
    })?;

    let range = range.strip_prefix("bytes=").ok_or_else(|| {
        (
            StatusCode::RANGE_NOT_SATISFIABLE,
            "Only byte ranges are supported".to_string(),
        )
    })?;

    if total_size == 0 || range.contains(',') {
        return Err((
            StatusCode::RANGE_NOT_SATISFIABLE,
            "Requested range cannot be satisfied".to_string(),
        ));
    }

    let Some((start, end)) = range.split_once('-') else {
        return Err((
            StatusCode::RANGE_NOT_SATISFIABLE,
            "Invalid Range header".to_string(),
        ));
    };

    if start.is_empty() {
        let suffix_len = end.parse::<usize>().map_err(|_| {
            (
                StatusCode::RANGE_NOT_SATISFIABLE,
                "Invalid suffix byte range".to_string(),
            )
        })?;

        if suffix_len == 0 {
            return Err((
                StatusCode::RANGE_NOT_SATISFIABLE,
                "Requested range cannot be satisfied".to_string(),
            ));
        }

        let start = total_size.saturating_sub(suffix_len);
        return Ok(Some((start, total_size - 1)));
    }

    let start = start.parse::<usize>().map_err(|_| {
        (
            StatusCode::RANGE_NOT_SATISFIABLE,
            "Invalid byte range start".to_string(),
        )
    })?;

    if start >= total_size {
        return Err((
            StatusCode::RANGE_NOT_SATISFIABLE,
            "Requested range cannot be satisfied".to_string(),
        ));
    }

    let end = if end.is_empty() {
        total_size - 1
    } else {
        end.parse::<usize>().map_err(|_| {
            (
                StatusCode::RANGE_NOT_SATISFIABLE,
                "Invalid byte range end".to_string(),
            )
        })?
    };

    if end < start {
        return Err((
            StatusCode::RANGE_NOT_SATISFIABLE,
            "Requested range cannot be satisfied".to_string(),
        ));
    }

    Ok(Some((start, end.min(total_size - 1))))
}

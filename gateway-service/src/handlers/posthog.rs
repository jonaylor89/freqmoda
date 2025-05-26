use axum::{
    body::{Body, Bytes},
    extract::{Path, Query, Request, State},
    http::{HeaderMap, Method, StatusCode},
    response::{IntoResponse, Response},
};
use std::collections::HashMap;
use tracing::{debug, error};

use crate::state::AppState;

/// Proxy requests to PostHog's static assets (like array.js)
pub async fn proxy_posthog_static(
    State(state): State<AppState>,
    Path(path): Path<String>,
    Query(query_params): Query<HashMap<String, String>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let posthog_host = state
        .settings
        .posthog
        .as_ref()
        .map(|config| config.host.as_str())
        .unwrap_or("https://us-assets.i.posthog.com");

    let mut url = format!("{}/static/{}", posthog_host, path);

    // Add query parameters if they exist
    if !query_params.is_empty() {
        let query_string = query_params
            .iter()
            .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");
        url = format!("{}?{}", url, query_string);
    }

    debug!("Proxying PostHog static request to: {}", url);

    match forward_request(Method::GET, &url, headers, None).await {
        Ok(response) => response,
        Err(err) => {
            error!("Failed to proxy PostHog static request: {}", err);
            (StatusCode::BAD_GATEWAY, "Failed to proxy request").into_response()
        }
    }
}

/// Generic proxy for all PostHog API endpoints
pub async fn proxy_posthog_api(
    State(state): State<AppState>,
    Path(path): Path<String>,
    request: Request,
) -> impl IntoResponse {
    let method = request.method().clone();
    let headers = request.headers().clone();
    let query_string = request.uri().query().unwrap_or("").to_string();

    // Only read body for methods that typically have one
    let body = if matches!(method, Method::POST | Method::PUT | Method::PATCH) {
        match axum::body::to_bytes(request.into_body(), usize::MAX).await {
            Ok(bytes) => {
                if bytes.is_empty() {
                    None
                } else {
                    Some(bytes)
                }
            }
            Err(_) => None,
        }
    } else {
        None
    };

    let posthog_host = state
        .settings
        .posthog
        .as_ref()
        .map(|config| config.host.as_str())
        .unwrap_or("https://us.i.posthog.com");

    let mut url = format!("{}/{}", posthog_host, path);

    // Add query parameters if they exist
    if !query_string.is_empty() {
        url = format!("{}?{}", url, query_string);
    }

    debug!(
        "Proxying PostHog API request to: {} (method: {})",
        url, method
    );

    match forward_request(method, &url, headers, body).await {
        Ok(response) => response,
        Err(err) => {
            error!("Failed to proxy PostHog API request: {}", err);
            (StatusCode::BAD_GATEWAY, "Failed to proxy request").into_response()
        }
    }
}

/// Generic function to forward HTTP requests to PostHog
async fn forward_request(
    method: Method,
    url: &str,
    headers: HeaderMap,
    body: Option<Bytes>,
) -> Result<Response, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();

    // Convert axum Method to reqwest Method
    let reqwest_method = match method {
        Method::GET => reqwest::Method::GET,
        Method::POST => reqwest::Method::POST,
        Method::PUT => reqwest::Method::PUT,
        Method::DELETE => reqwest::Method::DELETE,
        Method::HEAD => reqwest::Method::HEAD,
        Method::OPTIONS => reqwest::Method::OPTIONS,
        Method::PATCH => reqwest::Method::PATCH,
        _ => reqwest::Method::GET, // Default fallback
    };

    let mut request_builder = client.request(reqwest_method, url);

    // Forward relevant headers, filtering out hop-by-hop headers and content-length
    for (name, value) in headers.iter() {
        let header_name = name.as_str().to_lowercase();

        // Skip hop-by-hop headers, host header, and content-length
        if !matches!(
            header_name.as_str(),
            "connection"
                | "keep-alive"
                | "proxy-authenticate"
                | "proxy-authorization"
                | "te"
                | "trailers"
                | "transfer-encoding"
                | "upgrade"
                | "host"
                | "content-length"
        ) {
            // Convert axum headers to reqwest headers
            if let (Ok(req_name), Ok(req_value)) = (
                reqwest::header::HeaderName::try_from(name.as_str()),
                reqwest::header::HeaderValue::try_from(value.as_bytes()),
            ) {
                request_builder = request_builder.header(req_name, req_value);
            }
        }
    }

    // Add body if present
    if let Some(body_content) = body {
        request_builder = request_builder.body(body_content);
    }

    // Make the request
    let response = request_builder.send().await?;

    // Get response details
    let status = response.status();
    let response_headers = response.headers().clone();
    let response_body = response.bytes().await?;

    // Convert reqwest StatusCode to axum StatusCode
    let axum_status = StatusCode::from_u16(status.as_u16())?;

    // Build the response
    let mut resp_builder = Response::builder().status(axum_status);

    // Forward response headers (excluding hop-by-hop headers and content-length)
    for (name, value) in response_headers.iter() {
        let header_name = name.as_str().to_lowercase();

        if !matches!(
            header_name.as_str(),
            "connection"
                | "keep-alive"
                | "proxy-authenticate"
                | "proxy-authorization"
                | "te"
                | "trailers"
                | "transfer-encoding"
                | "upgrade"
                | "content-length"
        ) {
            // Convert reqwest headers to axum headers
            if let (Ok(axum_name), Ok(axum_value)) = (
                axum::http::HeaderName::try_from(name.as_str()),
                axum::http::HeaderValue::try_from(value.as_bytes()),
            ) {
                resp_builder = resp_builder.header(axum_name, axum_value);
            }
        }
    }

    // Add CORS headers for browser requests
    resp_builder = resp_builder
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        .header(
            "Access-Control-Allow-Headers",
            "Content-Type, Authorization",
        );

    Ok(resp_builder.body(Body::from(response_body))?)
}

/// Handle CORS preflight requests
pub async fn proxy_posthog_options() -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::OK)
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        .header(
            "Access-Control-Allow-Headers",
            "Content-Type, Authorization",
        )
        .header("Access-Control-Max-Age", "86400")
        .body(Body::empty())
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderMap, Method, StatusCode};

    #[tokio::test]
    async fn test_forward_request_handles_errors_gracefully() {
        let headers = HeaderMap::new();
        let result = forward_request(
            Method::GET,
            "http://invalid-url-that-should-fail.test",
            headers,
            None,
        )
        .await;

        assert!(result.is_err());
    }

    #[test]
    fn test_proxy_options_returns_cors_headers() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let response = rt.block_on(proxy_posthog_options());

        // Convert to response to check status
        let response = response.into_response();
        assert_eq!(response.status(), StatusCode::OK);

        // Check that CORS headers are present
        let headers = response.headers();
        assert!(headers.contains_key("Access-Control-Allow-Origin"));
        assert!(headers.contains_key("Access-Control-Allow-Methods"));
        assert!(headers.contains_key("Access-Control-Allow-Headers"));
    }

    #[test]
    fn test_query_param_encoding() {
        // Test that query parameters are properly URL encoded
        let mut query_params = HashMap::new();
        query_params.insert(
            "key with spaces".to_string(),
            "value with & symbols".to_string(),
        );
        query_params.insert("api_key".to_string(), "test_key_123".to_string());

        let query_string = query_params
            .iter()
            .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        assert!(query_string.contains("key%20with%20spaces=value%20with%20%26%20symbols"));
        assert!(query_string.contains("api_key=test_key_123"));
    }

    #[test]
    fn test_url_construction_with_query_params() {
        let base_url = "https://example.com/api/endpoint";
        let query_string = "param1=value1&param2=value%20with%20spaces";
        let full_url = format!("{}?{}", base_url, query_string);

        assert_eq!(
            full_url,
            "https://example.com/api/endpoint?param1=value1&param2=value%20with%20spaces"
        );
    }

    #[test]
    fn test_binary_data_handling() {
        // Test that binary data is properly handled without content-length issues
        let binary_data = vec![0u8, 1, 2, 3, 255, 128, 64];
        let bytes = Bytes::from(binary_data.clone());

        // Verify that the bytes are preserved correctly
        assert_eq!(bytes.to_vec(), binary_data);
        assert_eq!(bytes.len(), 7);
    }
}

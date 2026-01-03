use axum::{
    extract::Request,
    http::{HeaderMap, HeaderValue},
    middleware::Next,
    response::Response,
};
use tracing::instrument;
use uuid::Uuid;

#[instrument(skip(request, next))]
pub async fn session_middleware(mut request: Request, next: Next) -> Response {
    let headers = request.headers().clone();
    let session_id = extract_or_create_session(&headers);
    let is_new_session = !has_session_cookie(&headers);

    request.extensions_mut().insert(session_id.clone());
    let mut response = next.run(request).await;

    // Set session cookie if it's a new session
    if is_new_session {
        let cookie = format!(
            "session_id={}; HttpOnly; SameSite=Lax; Path=/; Max-Age=86400",
            session_id
        );
        if let Ok(cookie_value) = HeaderValue::from_str(&cookie) {
            response.headers_mut().insert("set-cookie", cookie_value);
        }
    }

    response
}

fn extract_or_create_session(headers: &HeaderMap) -> String {
    // Try to get session from cookie
    if let Some(cookie_header) = headers.get("cookie")
        && let Ok(cookie_str) = cookie_header.to_str() {
            for cookie in cookie_str.split(';') {
                let cookie = cookie.trim();
                if let Some(session_value) = cookie.strip_prefix("session_id=") {
                    return session_value.to_string();
                }
            }
        }

    // Generate new session ID
    Uuid::new_v4().to_string()
}

fn has_session_cookie(headers: &HeaderMap) -> bool {
    if let Some(cookie_header) = headers.get("cookie")
        && let Ok(cookie_str) = cookie_header.to_str() {
            return cookie_str.contains("session_id=");
        }
    false
}

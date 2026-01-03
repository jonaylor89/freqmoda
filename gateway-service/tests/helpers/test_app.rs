use crate::helpers::{
    config::{TestConfigBuilder, test_settings_with_mock_urls},
    database::TestDatabase,
    mocks::MockEnvironment,
    redis::TestRedis,
};
use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{Request, StatusCode},
};
use gateway_service::{routes::create_router, state::AppState};
use serde_json::Value;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower::ServiceExt;
use uuid::Uuid;

#[derive(Clone)]
pub struct TestApp {
    pub address: SocketAddr,
    pub state: Arc<AppState>,
    pub db: Arc<TestDatabase>,
    pub mocks: Arc<MockEnvironment>,
    pub http_client: reqwest::Client,
}

impl TestApp {
    pub async fn new() -> Self {
        crate::helpers::ensure_tracing();
        let db = TestDatabase::new().await;
        let redis = TestRedis::new().await;
        let mocks = MockEnvironment::new().await;

        // Setup default mocks
        mocks.setup_default_mocks().await;

        let settings = test_settings_with_mock_urls(
            &mocks.get_claude_base_url(),
            &mocks.get_streaming_base_url(),
        );

        let state = AppState::new(settings, db.connection_pool.clone(), redis.get_client())
            .expect("Failed to create AppState");

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind to random port");
        let address = listener.local_addr().expect("Failed to get local address");

        let http_client = reqwest::Client::new();

        Self {
            address,
            state: Arc::new(state),
            db: Arc::new(db),
            mocks: Arc::new(mocks),
            http_client,
        }
    }

    pub async fn new_with_config(config_builder: TestConfigBuilder) -> Self {
        crate::helpers::ensure_tracing();
        let db = TestDatabase::new().await;
        let redis = TestRedis::new().await;
        let mocks = MockEnvironment::new().await;

        mocks.setup_default_mocks().await;

        let mut settings = config_builder.build();
        settings.claude.base_url = mocks.get_claude_base_url();
        settings.streaming_engine.base_url = mocks.get_streaming_base_url();

        let state = AppState::new(settings, db.connection_pool.clone(), redis.get_client())
            .expect("Failed to create AppState");

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind to random port");
        let address = listener.local_addr().expect("Failed to get local address");

        let http_client = reqwest::Client::new();

        Self {
            address,
            state: Arc::new(state),
            db: Arc::new(db),
            mocks: Arc::new(mocks),
            http_client,
        }
    }

    pub async fn seed_test_data(&self) -> crate::helpers::database::TestData {
        self.db.seed_test_data().await
    }

    pub async fn cleanup(&self) {
        self.db.cleanup().await;
    }

    // HTTP request helpers
    fn attach_connect_info(&self, mut request: Request<Body>) -> Request<Body> {
        request.extensions_mut().insert(ConnectInfo(self.address));
        request
    }

    pub async fn get(&self, path: &str) -> TestResponse {
        let request = Request::builder()
            .method("GET")
            .uri(path)
            .body(Body::empty())
            .expect("Failed to build request");

        let router = create_router(self.state.as_ref().clone());
        let response = router
            .oneshot(self.attach_connect_info(request))
            .await
            .expect("Failed to send request");
        TestResponse::new(response).await
    }

    pub async fn post<T: serde::Serialize>(&self, path: &str, body: &T) -> TestResponse {
        let json_body = serde_json::to_string(body).expect("Failed to serialize body");

        let request = Request::builder()
            .method("POST")
            .uri(path)
            .header("content-type", "application/json")
            .body(Body::from(json_body))
            .expect("Failed to build request");

        let router = create_router(self.state.as_ref().clone());
        let response = router
            .oneshot(self.attach_connect_info(request))
            .await
            .expect("Failed to send request");
        TestResponse::new(response).await
    }

    pub async fn post_json(&self, path: &str, body: Value) -> TestResponse {
        let json_body = body.to_string();

        let request = Request::builder()
            .method("POST")
            .uri(path)
            .header("content-type", "application/json")
            .body(Body::from(json_body))
            .expect("Failed to build request");

        let router = create_router(self.state.as_ref().clone());
        let response = router
            .oneshot(self.attach_connect_info(request))
            .await
            .expect("Failed to send request");
        TestResponse::new(response).await
    }

    pub async fn put<T: serde::Serialize>(&self, path: &str, body: &T) -> TestResponse {
        let json_body = serde_json::to_string(body).expect("Failed to serialize body");

        let request = Request::builder()
            .method("PUT")
            .uri(path)
            .header("content-type", "application/json")
            .body(Body::from(json_body))
            .expect("Failed to build request");

        let router = create_router(self.state.as_ref().clone());
        let response = router
            .oneshot(self.attach_connect_info(request))
            .await
            .expect("Failed to send request");
        TestResponse::new(response).await
    }

    pub async fn delete(&self, path: &str) -> TestResponse {
        let request = Request::builder()
            .method("DELETE")
            .uri(path)
            .body(Body::empty())
            .expect("Failed to build request");

        let router = create_router(self.state.as_ref().clone());
        let response = router
            .oneshot(self.attach_connect_info(request))
            .await
            .expect("Failed to send request");
        TestResponse::new(response).await
    }

    // API endpoint helpers
    pub async fn health_check(&self) -> TestResponse {
        self.get("/health").await
    }

    pub async fn chat(&self, message: &str, conversation_id: Option<&str>) -> TestResponse {
        let mut body = serde_json::json!({
            "message": message
        });

        if let Some(conv_id) = conversation_id {
            body["conversation_id"] = serde_json::json!(conv_id);
        }

        self.post_json("/api/chat", body).await
    }

    pub async fn process_audio(&self, audio_name: &str, parameters: Value) -> TestResponse {
        let body = serde_json::json!({
            "audio_name": audio_name,
            "parameters": parameters
        });

        self.post_json("/api/audio/process", body).await
    }

    pub async fn list_audio_samples(&self) -> TestResponse {
        self.get("/api/audio/samples").await
    }

    pub async fn get_audio_metadata(&self, audio_name: &str) -> TestResponse {
        let body = serde_json::json!({
            "audio_name": audio_name
        });

        self.post_json("/api/audio/metadata", body).await
    }

    // Test data helpers
    pub async fn create_conversation(&self, user_id: Uuid) -> Uuid {
        let conversation_id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO conversations (id, user_id, created_at, updated_at)
            VALUES ($1, $2, NOW(), NOW())
            "#,
        )
        .bind(conversation_id)
        .bind(user_id)
        .execute(&self.db.connection_pool)
        .await
        .expect("Failed to create test conversation");

        conversation_id
    }

    pub async fn create_message(&self, conversation_id: Uuid, role: &str, content: &str) -> Uuid {
        self.db
            .create_test_message(conversation_id, role, content)
            .await
    }

    pub async fn get_message_count(&self, conversation_id: Uuid) -> i64 {
        self.db
            .get_conversation_message_count(conversation_id)
            .await
    }

    // Mock configuration helpers
    pub async fn setup_claude_success(&self) {
        self.mocks.claude_mock.mock_chat_completion_success().await;
    }

    pub async fn setup_claude_error(&self, status_code: u16, error_message: &str) {
        self.mocks
            .claude_mock
            .mock_chat_completion_error(status_code, error_message)
            .await;
    }

    pub async fn setup_claude_text_response(&self, response_text: &str) {
        self.mocks
            .claude_mock
            .mock_chat_completion_with_text_response(response_text)
            .await;
    }

    pub async fn setup_streaming_engine_success(&self, audio_filename: &str) {
        self.mocks
            .streaming_mock
            .mock_audio_processing_success(audio_filename)
            .await;
    }

    pub async fn setup_streaming_engine_error(&self) {
        self.mocks.streaming_mock.mock_processing_error().await;
    }

    pub async fn verify_claude_request_count(&self, expected_count: u64) -> bool {
        self.mocks
            .claude_mock
            .verify_request_count(expected_count)
            .await
    }

    pub async fn get_last_claude_request(&self) -> Option<wiremock::Request> {
        self.mocks.claude_mock.get_last_request().await
    }
}

pub struct TestResponse {
    pub status: StatusCode,
    pub headers: axum::http::HeaderMap,
    pub body: String,
}

impl TestResponse {
    async fn new(response: axum::response::Response) -> Self {
        let status = response.status();
        let headers = response.headers().clone();
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("Failed to read response body");
        let body = String::from_utf8(body_bytes.to_vec())
            .expect("Failed to convert response body to string");

        Self {
            status,
            headers,
            body,
        }
    }

    pub fn json<T>(&self) -> T
    where
        T: serde::de::DeserializeOwned,
    {
        serde_json::from_str(&self.body).expect("Failed to deserialize response body as JSON")
    }

    pub fn json_value(&self) -> Value {
        serde_json::from_str(&self.body).expect("Failed to deserialize response body as JSON Value")
    }

    pub fn assert_status(&self, expected_status: StatusCode) {
        assert_eq!(
            self.status, expected_status,
            "Expected status {}, got {}. Response body: {}",
            expected_status, self.status, self.body
        );
    }

    pub fn assert_json_contains(&self, expected: &Value) {
        let actual = self.json_value();

        fn contains_value(haystack: &Value, needle: &Value) -> bool {
            match (haystack, needle) {
                (Value::Object(h), Value::Object(n)) => n.iter().all(|(key, value)| {
                    h.get(key).is_some_and(|h_val| contains_value(h_val, value))
                }),
                (h, n) => h == n,
            }
        }

        assert!(
            contains_value(&actual, expected),
            "Expected JSON to contain {:?}, but got {:?}",
            expected,
            actual
        );
    }

    pub fn assert_header(&self, header_name: &str, expected_value: &str) {
        let actual_value = self
            .headers
            .get(header_name)
            .and_then(|h| h.to_str().ok())
            .unwrap_or("");

        assert_eq!(
            actual_value, expected_value,
            "Expected header '{}' to be '{}', got '{}'",
            header_name, expected_value, actual_value
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_app_setup() {
        let app = TestApp::new().await;

        // Test that the app is properly configured
        assert!(!app.address.to_string().is_empty());
        assert!(!app.mocks.get_claude_base_url().is_empty());
        assert!(!app.mocks.get_streaming_base_url().is_empty());

        app.cleanup().await;
    }

    #[tokio::test]
    async fn test_health_check() {
        let app = TestApp::new().await;

        let response = app.health_check().await;
        response.assert_status(StatusCode::OK);

        app.cleanup().await;
    }

    #[tokio::test]
    async fn test_response_json_parsing() {
        let app = TestApp::new().await;

        let response = app.health_check().await;
        let json = response.json_value();

        assert!(json.is_object());

        app.cleanup().await;
    }

    #[tokio::test]
    async fn test_seed_data() {
        let app = TestApp::new().await;

        let test_data = app.seed_test_data().await;

        assert!(!test_data.user_id.is_nil());
        assert!(!test_data.conversation_id.is_nil());
        assert_eq!(test_data.sample_ids.len(), 5);

        app.cleanup().await;
    }

    #[tokio::test]
    async fn test_mock_setup() {
        let app = TestApp::new().await;

        app.setup_claude_text_response("Test response").await;
        app.setup_streaming_engine_success("test.mp3").await;

        // The mocks should be ready for use
        assert!(!app.mocks.get_claude_base_url().is_empty());
        assert!(!app.mocks.get_streaming_base_url().is_empty());

        app.cleanup().await;
    }
}

use base64::{Engine as _, engine::general_purpose};
use serde_json::{Value, json};
use std::collections::HashMap;
use uuid::Uuid;
use wiremock::{
    Mock, MockServer, Request, ResponseTemplate,
    matchers::{header, method, path, path_regex},
};

pub struct ClaudeMockServer {
    pub server: MockServer,
    pub base_url: String,
}

impl ClaudeMockServer {
    pub async fn new() -> Self {
        let server = MockServer::start().await;
        let base_url = server.uri();

        Self { server, base_url }
    }

    pub async fn mock_chat_completion_success(&self) -> &Self {
        self.server.reset().await;
        Mock::given(method("POST"))
            .and(path("/v1/messages"))
            .and(header("content-type", "application/json"))
            .and(header("x-api-key", "test-api-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "msg_test123",
                "type": "message",
                "role": "assistant",
                "content": [
                    {
                        "type": "text",
                        "text": "I'll process that audio for you."
                    },
                    {
                        "type": "tool_use",
                        "id": "toolu_test123",
                        "name": "process_audio",
                        "input": {
                            "audio_name": "Sample 1",
                            "effects": {
                                "reverse": true,
                                "echo": "medium"
                            }
                        }
                    }
                ],
                "model": "claude-3-5-sonnet-20241022",
                "stop_reason": "tool_use",
                "stop_sequence": null,
                "usage": {
                    "input_tokens": 100,
                    "output_tokens": 50
                }
            })))
            .mount(&self.server)
            .await;

        self
    }

    pub async fn mock_chat_completion_with_text_response(&self, response_text: &str) -> &Self {
        self.server.reset().await;
        Mock::given(method("POST"))
            .and(path("/v1/messages"))
            .and(header("content-type", "application/json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "msg_text123",
                "type": "message",
                "role": "assistant",
                "content": [
                    {
                        "type": "text",
                        "text": response_text
                    }
                ],
                "model": "claude-3-5-sonnet-20241022",
                "stop_reason": "end_turn",
                "stop_sequence": null,
                "usage": {
                    "input_tokens": 50,
                    "output_tokens": 25
                }
            })))
            .mount(&self.server)
            .await;

        self
    }

    pub async fn mock_chat_completion_error(&self, status_code: u16, error_message: &str) -> &Self {
        self.server.reset().await;
        Mock::given(method("POST"))
            .and(path("/v1/messages"))
            .respond_with(ResponseTemplate::new(status_code).set_body_json(json!({
                "type": "error",
                "error": {
                    "type": "invalid_request_error",
                    "message": error_message
                }
            })))
            .mount(&self.server)
            .await;

        self
    }

    pub async fn mock_rate_limit_error(&self) -> &Self {
        self.server.reset().await;
        Mock::given(method("POST"))
            .and(path("/v1/messages"))
            .respond_with(ResponseTemplate::new(429).set_body_json(json!({
                "type": "error",
                "error": {
                    "type": "rate_limit_error",
                    "message": "Rate limit exceeded"
                }
            })))
            .mount(&self.server)
            .await;

        self
    }

    pub async fn mock_auth_error(&self) -> &Self {
        self.server.reset().await;
        Mock::given(method("POST"))
            .and(path("/v1/messages"))
            .respond_with(ResponseTemplate::new(401).set_body_json(json!({
                "type": "error",
                "error": {
                    "type": "authentication_error",
                    "message": "Invalid API key"
                }
            })))
            .mount(&self.server)
            .await;

        self
    }

    pub async fn verify_request_count(&self, expected_count: u64) -> bool {
        let received_requests = self.server.received_requests().await.unwrap();
        received_requests.len() as u64 == expected_count
    }

    pub async fn get_last_request(&self) -> Option<Request> {
        let received_requests = self.server.received_requests().await.unwrap();
        received_requests.last().cloned()
    }
}

pub struct StreamingEngineMockServer {
    pub server: MockServer,
    pub base_url: String,
}

impl StreamingEngineMockServer {
    pub async fn new() -> Self {
        let server = MockServer::start().await;
        let base_url = server.uri();

        Self { server, base_url }
    }

    pub async fn mock_audio_processing_success(&self, audio_filename: &str) -> &Self {
        // Clear any previous error handlers so success responses take priority
        self.server.reset().await;
        let processed_url = format!("{}/unsafe/{}", self.base_url, audio_filename);

        for verb in ["GET", "HEAD"] {
            Mock::given(method(verb))
                .and(path_regex(r"/unsafe/.*"))
                .respond_with(
                    ResponseTemplate::new(200)
                        .insert_header("content-type", "audio/mpeg")
                        .insert_header("x-processed-url", processed_url.as_str())
                        .set_body_raw("mock-audio-data", "audio/mpeg"),
                )
                .mount(&self.server)
                .await;
        }

        self
    }

    pub async fn mock_metadata_endpoint(&self, audio_filename: &str, duration: f64) -> &Self {
        for verb in ["GET", "HEAD"] {
            Mock::given(method(verb))
                .and(path_regex(r"/meta/unsafe/.*"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "filename": audio_filename,
                    "duration": duration,
                    "format": "mp3",
                    "channels": 2,
                    "sample_rate": 44100,
                    "bitrate": 320000
                })))
                .mount(&self.server)
                .await;
        }

        self
    }

    pub async fn mock_audio_not_found(&self) -> &Self {
        Mock::given(method("GET"))
            .and(path_regex(r"/unsafe/nonexistent.*"))
            .respond_with(ResponseTemplate::new(404).set_body_json(json!({
                "error": "Audio file not found"
            })))
            .mount(&self.server)
            .await;

        self
    }

    pub async fn mock_processing_error(&self) -> &Self {
        self.server.reset().await;
        for verb in ["GET", "HEAD"] {
            Mock::given(method(verb))
                .and(path_regex(r"/unsafe/.*"))
                .respond_with(ResponseTemplate::new(500).set_body_json(json!({
                    "error": "Audio processing failed"
                })))
                .mount(&self.server)
                .await;
        }

        self
    }

    pub async fn mock_effects_processing(&self, _effects: HashMap<&str, &str>) -> &Self {
        for verb in ["GET", "HEAD"] {
            Mock::given(method(verb))
                .and(path_regex(r"/unsafe/.*"))
                .respond_with(
                    ResponseTemplate::new(200)
                        .insert_header("content-type", "audio/mpeg")
                        .set_body_raw("mock-processed-audio-with-effects", "audio/mpeg"),
                )
                .mount(&self.server)
                .await;
        }

        self
    }

    pub async fn verify_effect_parameters(&self, expected_effects: &HashMap<&str, &str>) -> bool {
        let received_requests = self.server.received_requests().await.unwrap();

        if let Some(last_request) = received_requests.last() {
            let url = last_request.url.clone();
            let query_pairs: Vec<(String, String)> = url
                .query_pairs()
                .map(|(k, v)| (k.into_owned(), v.into_owned()))
                .collect();

            // Prefer checking the encoded parameter payload if present
            if let Some(encoded_value) = url
                .query_pairs()
                .find(|(key, _)| key == "encoded")
                .map(|(_, value)| value.to_string())
            {
                if let Ok(decoded) = general_purpose::URL_SAFE_NO_PAD.decode(&encoded_value) {
                    if let Ok(json_value) = serde_json::from_slice::<Value>(&decoded) {
                        for (effect, expected_value) in expected_effects {
                            // First look in encoded payload
                            if let Some(match_val) = json_value.get(*effect) {
                                match match_val {
                                    Value::Bool(val) => {
                                        if (*val && *expected_value != "true")
                                            || (!*val && *expected_value != "false")
                                        {
                                            return false;
                                        }
                                    }
                                    Value::Number(num) => {
                                        if num.to_string() != *expected_value {
                                            return false;
                                        }
                                    }
                                    Value::String(val) => {
                                        if val != expected_value {
                                            return false;
                                        }
                                    }
                                    _ => return false,
                                }
                                continue;
                            }

                            // Then look at raw query params for explicit overrides (e.g. speed/volume)
                            if let Some(actual_value) = query_pairs
                                .iter()
                                .find(|(k, _)| k == effect)
                                .map(|(_, v)| v.clone())
                            {
                                if &actual_value != expected_value {
                                    return false;
                                }
                                continue;
                            }

                            // If not found anywhere, fail
                            return false;
                        }
                        return true;
                    }
                }
            }

            // Fallback to checking raw query params if no encoded payload was found
            for (effect, expected_value) in expected_effects {
                if let Some(actual_value) = query_pairs
                    .iter()
                    .find(|(k, _)| k == effect)
                    .map(|(_, v)| v.clone())
                {
                    if &actual_value != expected_value {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }
}

pub struct MockEnvironment {
    pub claude_mock: ClaudeMockServer,
    pub streaming_mock: StreamingEngineMockServer,
}

impl MockEnvironment {
    pub async fn new() -> Self {
        let claude_mock = ClaudeMockServer::new().await;
        let streaming_mock = StreamingEngineMockServer::new().await;

        Self {
            claude_mock,
            streaming_mock,
        }
    }

    pub async fn setup_default_mocks(&self) {
        // Setup default successful responses
        self.claude_mock.mock_chat_completion_success().await;
        self.streaming_mock
            .mock_audio_processing_success("sample1.mp3")
            .await;
        self.streaming_mock
            .mock_metadata_endpoint("sample1.mp3", 45.5)
            .await;
    }

    pub async fn setup_error_scenario(&self) {
        self.claude_mock
            .mock_chat_completion_error(500, "Internal server error")
            .await;
        self.streaming_mock.mock_processing_error().await;
    }

    pub fn get_claude_base_url(&self) -> String {
        self.claude_mock.base_url.clone()
    }

    pub fn get_streaming_base_url(&self) -> String {
        self.streaming_mock.base_url.clone()
    }
}

// Test data generators
pub fn create_mock_chat_request(message: &str, conversation_id: Option<&str>) -> Value {
    let mut request = json!({
        "message": message
    });

    if let Some(conv_id) = conversation_id {
        request["conversation_id"] = json!(conv_id);
    }

    request
}

pub fn create_mock_audio_process_request(audio_name: &str, parameters: Value) -> Value {
    json!({
        "audio_name": audio_name,
        "parameters": parameters
    })
}

pub fn create_mock_claude_tool_response(tool_name: &str, tool_input: Value) -> Value {
    json!({
        "id": format!("msg_{}", Uuid::new_v4()),
        "type": "message",
        "role": "assistant",
        "content": [
            {
                "type": "tool_use",
                "id": format!("toolu_{}", Uuid::new_v4()),
                "name": tool_name,
                "input": tool_input
            }
        ],
        "model": "claude-3-5-sonnet-20241022",
        "stop_reason": "tool_use",
        "usage": {
            "input_tokens": 100,
            "output_tokens": 50
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_claude_mock_server_setup() {
        let claude_mock = ClaudeMockServer::new().await;
        assert!(!claude_mock.base_url.is_empty());
        assert!(claude_mock.base_url.starts_with("http://"));
    }

    #[tokio::test]
    async fn test_streaming_mock_server_setup() {
        let streaming_mock = StreamingEngineMockServer::new().await;
        assert!(!streaming_mock.base_url.is_empty());
        assert!(streaming_mock.base_url.starts_with("http://"));
    }

    #[tokio::test]
    async fn test_mock_environment_setup() {
        let mock_env = MockEnvironment::new().await;
        mock_env.setup_default_mocks().await;

        assert!(!mock_env.get_claude_base_url().is_empty());
        assert!(!mock_env.get_streaming_base_url().is_empty());
    }

    #[tokio::test]
    async fn test_mock_chat_request_creation() {
        let request = create_mock_chat_request("Test message", Some("test-conv-id"));

        assert_eq!(request["message"], "Test message");
        assert_eq!(request["conversation_id"], "test-conv-id");
    }

    #[tokio::test]
    async fn test_mock_audio_request_creation() {
        let parameters = json!({
            "reverse": true,
            "echo": "medium"
        });

        let request = create_mock_audio_process_request("sample1.mp3", parameters);

        assert_eq!(request["audio_name"], "sample1.mp3");
        assert_eq!(request["parameters"]["reverse"], true);
        assert_eq!(request["parameters"]["echo"], "medium");
    }
}

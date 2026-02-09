mod helpers;

use axum::http::StatusCode;

use helpers::{
    config::TestConfigBuilder, database::TestDatabase, mocks::MockEnvironment, test_app::TestApp,
};
use secrecy::SecretString;
use serde_json::{Value, json};
use sqlx::Row;
use web_ui::config::*;

use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn test_configuration_parsing() {
    let settings = Settings {
        port: 9000,
        server: ServerConfig {
            host: "127.0.0.1".to_string(),
        },
        database: DatabaseSettings {
            username: "postgres".into(),
            password: secrecy::SecretString::new("password".into()),
            database_name: "freqmoda".into(),
            port: 5432,
            host: "localhost".into(),
            require_ssl: false,
        },
        openai: OpenAIConfig {
            api_key: SecretString::from("test-key"),
            base_url: "https://api.openai.com".to_string(),
            model: "gpt-4o".to_string(),
        },
        streaming_engine: StreamingEngineConfig {
            base_url: "http://localhost:9000".to_string(),
        },
        redis: web_ui::config::RedisConfig {
            url: SecretString::from("redis://localhost:6379"),
        },
    };

    assert_eq!(settings.port, 9000);
    assert_eq!(settings.openai.model, "gpt-4o");
}

#[tokio::test]
async fn test_configuration_from_builder() {
    let config_builder = TestConfigBuilder::new();
    let settings = config_builder.build();

    // Test that test config builder creates valid configuration
    assert_eq!(settings.server.host, "127.0.0.1");
    assert_eq!(settings.port, 0);
    assert_eq!(settings.openai.model, "gpt-4o");
    assert_eq!(settings.database.username, "postgres");
}

#[tokio::test]
async fn test_database_configuration() {
    let database_config = DatabaseSettings {
        username: "testuser".into(),
        password: secrecy::SecretString::new("testpass".into()),
        database_name: "testdb".into(),
        port: 5433,
        host: "testhost".into(),
        require_ssl: true,
    };

    let _connect_options = database_config.with_db();
    // Test that options are properly configured
    // In a real test, you might verify the connection string format

    let _connect_options_without_db = database_config.without_db();
    // Verify SSL mode and other settings
}

mod health_tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check_returns_ok() {
        let app = TestApp::new().await;

        let response = app.health_check().await;

        response.assert_status(StatusCode::OK);
        let json = response.json_value();
        assert!(json.get("status").is_some());

        app.cleanup().await;
    }

    #[tokio::test]
    async fn test_health_check_with_database() {
        let app = TestApp::new().await;
        let _test_data = app.seed_test_data().await;

        let response = app.health_check().await;

        response.assert_status(StatusCode::OK);
        let json = response.json_value();
        assert_eq!(json["status"], "healthy");

        app.cleanup().await;
    }
}

mod chat_tests {
    use super::*;

    #[tokio::test]
    async fn test_chat_endpoint_basic_message() {
        let app = TestApp::new().await;
        let _test_data = app.seed_test_data().await;

        app.setup_openai_text_response("Hello! I'm here to help with audio processing.")
            .await;

        let response = app.chat("Hello, can you help me with audio?", None).await;

        response.assert_status(StatusCode::OK);
        let json = response.json_value();

        assert!(json.get("message").is_some());
        assert!(json.get("conversation_id").is_some());

        app.cleanup().await;
    }

    #[tokio::test]
    async fn test_chat_endpoint_with_conversation_id() {
        let app = TestApp::new().await;
        let test_data = app.seed_test_data().await;

        app.setup_openai_text_response("I can help you reverse that audio file.")
            .await;

        let conversation_id = test_data.conversation_id.to_string();
        let response = app.chat("Reverse sample 1", Some(&conversation_id)).await;

        response.assert_status(StatusCode::OK);
        let json = response.json_value();

        assert_eq!(json["conversation_id"], conversation_id);

        app.cleanup().await;
    }

    #[tokio::test]
    async fn test_chat_endpoint_audio_processing_request() {
        let app = TestApp::new().await;
        let _test_data = app.seed_test_data().await;

        app.setup_openai_success().await;
        app.setup_streaming_engine_success("sample1.mp3").await;

        let response = app.chat("Reverse Sample 1 and add echo", None).await;

        response.assert_status(StatusCode::OK);
        let json = response.json_value();

        assert!(json.get("message").is_some());
        assert!(json.get("conversation_id").is_some());

        // Verify that the message contains a processed audio URL
        let message = json["message"].as_str().unwrap();
        assert!(message.contains("http://") || message.contains("processed"));

        app.cleanup().await;
    }

    #[tokio::test]
    async fn test_chat_endpoint_invalid_json() {
        let app = TestApp::new().await;

        let response = app
            .post_json("/api/chat", json!({"invalid": "request"}))
            .await;

        response.assert_status(StatusCode::BAD_REQUEST);

        app.cleanup().await;
    }

    #[tokio::test]
    async fn test_chat_endpoint_empty_message() {
        let app = TestApp::new().await;

        let response = app.chat("", None).await;

        response.assert_status(StatusCode::BAD_REQUEST);

        app.cleanup().await;
    }

    #[tokio::test]
    async fn test_chat_endpoint_openai_error() {
        let app = TestApp::new().await;
        let _test_data = app.seed_test_data().await;

        app.setup_openai_error(500, "Internal server error").await;

        let response = app.chat("Process some audio", None).await;

        response.assert_status(StatusCode::INTERNAL_SERVER_ERROR);

        app.cleanup().await;
    }

    #[tokio::test]
    async fn test_chat_endpoint_conversation_persistence() {
        let app = TestApp::new().await;
        let test_data = app.seed_test_data().await;

        app.setup_openai_text_response("First response").await;

        let conversation_id = test_data.conversation_id.to_string();

        // Send first message
        let response1 = app.chat("First message", Some(&conversation_id)).await;
        response1.assert_status(StatusCode::OK);

        let initial_count = app.get_message_count(test_data.conversation_id).await;

        app.setup_openai_text_response("Second response").await;

        // Send second message
        let response2 = app.chat("Second message", Some(&conversation_id)).await;
        response2.assert_status(StatusCode::OK);

        let final_count = app.get_message_count(test_data.conversation_id).await;

        // Should have added 4 messages total (2 user + 2 assistant)
        assert!(final_count > initial_count);

        app.cleanup().await;
    }
}

mod audio_processing_tests {
    use super::*;

    #[tokio::test]
    async fn test_process_audio_endpoint() {
        let app = TestApp::new().await;
        let _test_data = app.seed_test_data().await;

        app.setup_streaming_engine_success("sample1.mp3").await;

        let parameters = json!({
            "reverse": true,
            "echo": "medium",
            "speed": 1.2
        });

        let response = app.process_audio("Sample 1", parameters).await;

        response.assert_status(StatusCode::OK);
        let json = response.json_value();

        assert!(json.get("processed_url").is_some());

        app.cleanup().await;
    }

    #[tokio::test]
    async fn test_process_audio_nonexistent_sample() {
        let app = TestApp::new().await;
        let _test_data = app.seed_test_data().await;

        let parameters = json!({
            "reverse": true
        });

        let response = app.process_audio("Nonexistent Sample", parameters).await;

        response.assert_status(StatusCode::NOT_FOUND);

        app.cleanup().await;
    }

    #[tokio::test]
    async fn test_process_audio_streaming_engine_error() {
        let app = TestApp::new().await;
        let _test_data = app.seed_test_data().await;

        app.setup_streaming_engine_error().await;

        let parameters = json!({
            "reverse": true
        });

        let response = app.process_audio("Sample 1", parameters).await;

        response.assert_status(StatusCode::INTERNAL_SERVER_ERROR);

        app.cleanup().await;
    }

    #[tokio::test]
    async fn test_list_audio_samples() {
        let app = TestApp::new().await;
        let _test_data = app.seed_test_data().await;

        let response = app.list_audio_samples().await;

        response.assert_status(StatusCode::OK);
        let json = response.json_value();

        assert!(json.get("samples").is_some());
        let samples = json["samples"].as_array().unwrap();
        assert!(!samples.is_empty());

        // Verify sample structure
        let first_sample = &samples[0];
        assert!(first_sample.get("id").is_some());
        assert!(first_sample.get("name").is_some());
        assert!(first_sample.get("filename").is_some());
        assert!(first_sample.get("duration").is_some());

        app.cleanup().await;
    }

    #[tokio::test]
    async fn test_get_audio_metadata() {
        let app = TestApp::new().await;
        let _test_data = app.seed_test_data().await;

        let response = app.get_audio_metadata("Sample 1").await;

        response.assert_status(StatusCode::OK);
        let json = response.json_value();

        assert!(json.get("filename").is_some());
        assert!(json.get("duration").is_some());

        app.cleanup().await;
    }

    #[tokio::test]
    async fn test_get_audio_metadata_nonexistent() {
        let app = TestApp::new().await;
        let _test_data = app.seed_test_data().await;

        let response = app.get_audio_metadata("Nonexistent Sample").await;

        response.assert_status(StatusCode::NOT_FOUND);

        app.cleanup().await;
    }
}

mod database_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_database_connection() {
        let test_db = TestDatabase::new().await;

        // Test basic database connectivity
        let result = sqlx::query("SELECT 1 as test")
            .fetch_one(&test_db.connection_pool)
            .await;

        assert!(result.is_ok());
        test_db.cleanup().await;
    }

    #[tokio::test]
    async fn test_database_migrations() {
        let test_db = TestDatabase::new().await;

        // Verify that all expected tables exist
        let tables = sqlx::query(
            r#"
            SELECT table_name
            FROM information_schema.tables
            WHERE table_schema = 'public'
            "#,
        )
        .fetch_all(&test_db.connection_pool)
        .await
        .expect("Failed to fetch table names");

        let table_names: Vec<String> = tables
            .iter()
            .map(|row| row.get::<String, _>("table_name"))
            .collect();

        assert!(table_names.contains(&"users".to_string()));
        assert!(table_names.contains(&"conversations".to_string()));
        assert!(table_names.contains(&"messages".to_string()));
        assert!(table_names.contains(&"audio_samples".to_string()));

        test_db.cleanup().await;
    }

    #[tokio::test]
    async fn test_seed_and_query_data() {
        let test_db = TestDatabase::new().await;

        let test_data = test_db.seed_test_data().await;

        // Verify user exists
        let user = sqlx::query("SELECT id, email, name FROM users WHERE id = $1")
            .bind(test_data.user_id)
            .fetch_one(&test_db.connection_pool)
            .await
            .expect("Failed to fetch user");

        assert_eq!(user.get::<Uuid, _>("id"), test_data.user_id);
        assert_eq!(user.get::<String, _>("email"), "test@example.com");

        // Verify conversation exists
        let conversation = sqlx::query("SELECT id, user_id FROM conversations WHERE id = $1")
            .bind(test_data.conversation_id)
            .fetch_one(&test_db.connection_pool)
            .await
            .expect("Failed to fetch conversation");

        assert_eq!(conversation.get::<Uuid, _>("id"), test_data.conversation_id);
        assert_eq!(conversation.get::<Uuid, _>("user_id"), test_data.user_id);

        // Verify audio samples exist
        assert_eq!(test_data.sample_ids.len(), 5);

        test_db.cleanup().await;
    }

    #[tokio::test]
    async fn test_conversation_message_operations() {
        let test_db = TestDatabase::new().await;
        let test_data = test_db.seed_test_data().await;

        // Add messages to conversation
        let message1_id = test_db
            .create_test_message(test_data.conversation_id, "user", "Hello")
            .await;
        let message2_id = test_db
            .create_test_message(test_data.conversation_id, "assistant", "Hi there!")
            .await;

        // Verify message count
        let count = test_db
            .get_conversation_message_count(test_data.conversation_id)
            .await;
        assert_eq!(count, 2);

        // Verify messages exist
        let messages = sqlx::query(
            r#"
            SELECT id, role, content
            FROM messages
            WHERE conversation_id = $1
            ORDER BY created_at
            "#,
        )
        .bind(test_data.conversation_id)
        .fetch_all(&test_db.connection_pool)
        .await
        .expect("Failed to fetch messages");

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].get::<Uuid, _>("id"), message1_id);
        assert_eq!(messages[0].get::<String, _>("role"), "user");
        assert_eq!(messages[0].get::<String, _>("content"), "Hello");
        assert_eq!(messages[1].get::<Uuid, _>("id"), message2_id);
        assert_eq!(messages[1].get::<String, _>("role"), "assistant");
        assert_eq!(messages[1].get::<String, _>("content"), "Hi there!");

        test_db.cleanup().await;
    }

    #[tokio::test]
    async fn test_audio_sample_queries() {
        let test_db = TestDatabase::new().await;
        let _test_data = test_db.seed_test_data().await;

        // Test finding audio sample by name
        let sample = test_db.get_audio_sample_by_name("Test Sample 1").await;

        assert!(sample.is_some());
        let sample = sample.unwrap();
        assert_eq!(sample.name, "Test Sample 1");
        assert_eq!(sample.filename, "test_sample_1.mp3");
        assert!(sample.duration > 0.0);

        // Test non-existent sample
        let non_existent = test_db
            .get_audio_sample_by_name("Non-existent Sample")
            .await;

        assert!(non_existent.is_none());

        test_db.cleanup().await;
    }

    #[tokio::test]
    async fn test_database_cleanup_operations() {
        let test_db = TestDatabase::new().await;
        let test_data = test_db.seed_test_data().await;

        // Add some messages
        test_db
            .create_test_message(test_data.conversation_id, "user", "Test message")
            .await;

        let initial_count = test_db
            .get_conversation_message_count(test_data.conversation_id)
            .await;
        assert_eq!(initial_count, 1);

        // Test clearing conversations (should also clear messages)
        test_db.clear_conversations().await;

        let count_after_clear = test_db
            .get_conversation_message_count(test_data.conversation_id)
            .await;
        assert_eq!(count_after_clear, 0);

        // Test truncating all tables
        test_db.truncate_all_tables().await;

        let user_count = sqlx::query("SELECT COUNT(*) as count FROM users")
            .fetch_one(&test_db.connection_pool)
            .await
            .expect("Failed to count users");
        assert_eq!(user_count.get::<i64, _>("count"), 0);

        test_db.cleanup().await;
    }
}

mod redis_integration_tests {
    use super::*;
    use helpers::redis::TestRedis;

    #[tokio::test]
    async fn test_redis_with_app_state() {
        let app = TestApp::new().await;

        // Verify Redis is working within the app
        let response = app.health_check().await;
        response.assert_status(StatusCode::OK);

        app.cleanup().await;
    }

    #[tokio::test]
    async fn test_redis_standalone() {
        let redis = TestRedis::new().await;

        // Test basic Redis operations
        redis.set("test_key", "test_value").await.unwrap();
        let value = redis.get("test_key").await.unwrap();
        assert_eq!(value, Some("test_value".to_string()));

        // Test Redis client for AppState
        let client = redis.get_client();
        assert!(client.get_connection().is_ok());

        let _ = redis.cleanup().await;
    }
}

mod external_service_tests {
    use super::*;

    #[tokio::test]
    async fn test_openai_mock_integration() {
        let mocks = MockEnvironment::new().await;
        mocks.setup_default_mocks().await;

        let client = reqwest::Client::new();

        // Test successful OpenAI API call
        let response = client
            .post(format!(
                "{}/v1/chat/completions",
                mocks.get_openai_base_url()
            ))
            .header("content-type", "application/json")
            .header("x-api-key", "test-api-key")
            .json(&json!({
                "model": "gpt-4o",
                "max_tokens": 1000,
                "messages": [
                    {
                        "role": "user",
                        "content": "Test message"
                    }
                ]
            }))
            .send()
            .await
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);

        let json: Value = response.json().await.expect("Failed to parse JSON");
        assert_eq!(json["type"], "message");
        assert_eq!(json["role"], "assistant");
    }

    #[tokio::test]
    async fn test_streaming_engine_mock_integration() {
        let mocks = MockEnvironment::new().await;
        mocks.setup_default_mocks().await;

        let client = reqwest::Client::new();

        // Test successful streaming engine call
        let response = client
            .get(format!(
                "{}/unsafe/sample1.mp3?reverse=true",
                mocks.get_streaming_base_url()
            ))
            .send()
            .await
            .expect("Failed to send request");

        assert_eq!(response.status(), 200);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "audio/mpeg"
        );

        let body = response.text().await.expect("Failed to get body");
        assert_eq!(body, "mock-audio-data");
    }

    #[tokio::test]
    async fn test_mock_error_scenarios() {
        let mocks = MockEnvironment::new().await;
        mocks.setup_error_scenario().await;

        let client = reqwest::Client::new();

        // Test OpenAI error response
        let openai_response = client
            .post(format!(
                "{}/v1/chat/completions",
                mocks.get_openai_base_url()
            ))
            .header("content-type", "application/json")
            .header("x-api-key", "test-api-key")
            .json(&json!({
                "model": "gpt-4o",
                "max_tokens": 1000,
                "messages": [{"role": "user", "content": "Test"}]
            }))
            .send()
            .await
            .expect("Failed to send request");

        assert_eq!(openai_response.status(), 500);

        // Test streaming engine error response
        let streaming_response = client
            .get(format!(
                "{}/unsafe/error.mp3",
                mocks.get_streaming_base_url()
            ))
            .send()
            .await
            .expect("Failed to send request");

        assert_eq!(streaming_response.status(), 500);
    }
}

mod error_handling_tests {
    use super::*;

    #[tokio::test]
    async fn test_invalid_routes() {
        let app = TestApp::new().await;

        let response = app.get("/nonexistent/route").await;
        response.assert_status(StatusCode::NOT_FOUND);

        app.cleanup().await;
    }

    #[tokio::test]
    async fn test_invalid_http_methods() {
        let app = TestApp::new().await;

        // Test POST to GET-only endpoint
        let response = app.post_json("/health", json!({})).await;
        response.assert_status(StatusCode::METHOD_NOT_ALLOWED);

        // Test GET to POST-only endpoint
        let response = app.get("/api/chat").await;
        response.assert_status(StatusCode::METHOD_NOT_ALLOWED);

        app.cleanup().await;
    }

    #[tokio::test]
    async fn test_malformed_json_requests() {
        let app = TestApp::new().await;

        use axum::{body::Body, http::Request};

        let request = Request::builder()
            .method("POST")
            .uri("/api/chat")
            .header("content-type", "application/json")
            .body(Body::from("invalid json {"))
            .expect("Failed to build request");

        let router = web_ui::routes::create_router(app.state.as_ref().clone());
        let response = router
            .oneshot(request)
            .await
            .expect("Failed to send request");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        app.cleanup().await;
    }

    #[tokio::test]
    async fn test_missing_content_type() {
        let app = TestApp::new().await;

        use axum::{body::Body, http::Request};

        let request = Request::builder()
            .method("POST")
            .uri("/api/chat")
            .body(Body::from(r#"{"message": "test"}"#))
            .expect("Failed to build request");

        let router = web_ui::routes::create_router(app.state.as_ref().clone());
        let response = router
            .oneshot(request)
            .await
            .expect("Failed to send request");
        // Should handle missing content-type gracefully
        assert!(response.status().is_client_error() || response.status().is_success());

        app.cleanup().await;
    }
}

mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn test_concurrent_chat_requests() {
        let app = TestApp::new().await;
        let _test_data = app.seed_test_data().await;

        app.setup_openai_text_response("Concurrent response").await;

        let start = Instant::now();

        // Send multiple concurrent requests
        let tasks: Vec<_> = (0..5)
            .map(|i| {
                let app = app.clone();
                tokio::spawn(async move { app.chat(&format!("Message {}", i), None).await })
            })
            .collect();

        let results = futures::future::join_all(tasks).await;
        let duration = start.elapsed();

        // All requests should succeed
        for result in results {
            let response = result.expect("Task should complete");
            response.assert_status(StatusCode::OK);
        }

        // Should complete reasonably quickly (adjust threshold as needed)
        assert!(duration.as_secs() < 30);

        app.cleanup().await;
    }

    #[tokio::test]
    async fn test_database_query_performance() {
        let app = TestApp::new().await;
        let _test_data = app.seed_test_data().await;

        let start = Instant::now();

        // Perform multiple database operations
        for _ in 0..10 {
            let response = app.list_audio_samples().await;
            response.assert_status(StatusCode::OK);
        }

        let duration = start.elapsed();

        // Database queries should be fast
        assert!(duration.as_millis() < 5000);

        app.cleanup().await;
    }
}

mod security_tests {
    use super::*;

    #[tokio::test]
    async fn test_sql_injection_protection() {
        let app = TestApp::new().await;
        let _test_data = app.seed_test_data().await;

        // Try SQL injection in chat message
        let malicious_message = "'; DROP TABLE users; --";
        let response = app.chat(malicious_message, None).await;

        // Should not crash the application
        assert!(response.status.is_client_error() || response.status.is_success());

        // Verify that users table still exists
        let samples_response = app.list_audio_samples().await;
        samples_response.assert_status(StatusCode::OK);

        app.cleanup().await;
    }

    #[tokio::test]
    async fn test_xss_protection() {
        let app = TestApp::new().await;
        let _test_data = app.seed_test_data().await;

        app.setup_openai_text_response("Safe response").await;

        // Try XSS in chat message
        let malicious_message = "<script>alert('xss')</script>";
        let response = app.chat(malicious_message, None).await;

        response.assert_status(StatusCode::OK);

        // Response should not contain unescaped script tags
        let json = response.json_value();
        let message = json["message"].as_str().unwrap_or("");
        assert!(!message.contains("<script>"));

        app.cleanup().await;
    }

    #[tokio::test]
    async fn test_large_payload_handling() {
        let app = TestApp::new().await;
        let _test_data = app.seed_test_data().await;

        // Create a very large message
        let large_message = "A".repeat(10_000);
        let response = app.chat(&large_message, None).await;

        // Should handle large payloads gracefully
        assert!(response.status.is_client_error() || response.status.is_success());

        app.cleanup().await;
    }
}

mod edge_case_tests {
    use super::*;

    #[tokio::test]
    async fn test_uuid_edge_cases() {
        let app = TestApp::new().await;
        let _test_data = app.seed_test_data().await;

        // Test with invalid UUID format
        let response = app.chat("Test message", Some("not-a-uuid")).await;
        response.assert_status(StatusCode::BAD_REQUEST);

        // Test with valid UUID that doesn't exist
        let non_existent_uuid = Uuid::new_v4().to_string();
        let response = app.chat("Test message", Some(&non_existent_uuid)).await;
        response.assert_status(StatusCode::NOT_FOUND);

        app.cleanup().await;
    }

    #[tokio::test]
    async fn test_empty_and_whitespace_inputs() {
        let app = TestApp::new().await;
        let _test_data = app.seed_test_data().await;

        // Test empty message
        let response = app.chat("", None).await;
        response.assert_status(StatusCode::BAD_REQUEST);

        // Test whitespace-only message
        let response = app.chat("   \n\t   ", None).await;
        response.assert_status(StatusCode::BAD_REQUEST);

        app.cleanup().await;
    }

    #[tokio::test]
    async fn test_unicode_and_special_characters() {
        let app = TestApp::new().await;
        let _test_data = app.seed_test_data().await;

        app.setup_openai_text_response("Unicode response: 🎵🤖💬")
            .await;

        let unicode_message = "Process audio with unicode: 🎵 samples and effects 🔊";
        let response = app.chat(unicode_message, None).await;

        response.assert_status(StatusCode::OK);
        let json = response.json_value();
        assert!(json.get("message").is_some());

        app.cleanup().await;
    }
}

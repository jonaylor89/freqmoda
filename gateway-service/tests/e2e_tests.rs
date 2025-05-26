mod helpers;

use axum::http::StatusCode;
use helpers::test_app::TestApp;
use serde_json::json;
use std::collections::HashMap;
use tokio::time::{Duration, sleep};
use uuid::Uuid;

#[tokio::test]
async fn e2e_complete_audio_processing_workflow() {
    let app = TestApp::new().await;
    let _test_data = app.seed_test_data().await;

    // Setup mock responses for complete workflow
    app.mocks.claude_mock.mock_chat_completion_success().await;
    app.mocks
        .streaming_mock
        .mock_audio_processing_success("sample1.mp3")
        .await;
    app.mocks
        .streaming_mock
        .mock_metadata_endpoint("sample1.mp3", 45.5)
        .await;

    // Step 1: User starts a conversation
    let response = app
        .chat("Hello, I'd like to process some audio", None)
        .await;
    response.assert_status(StatusCode::OK);

    let chat_response = response.json_value();
    let conversation_id = chat_response["conversation_id"].as_str().unwrap();

    // Step 2: User requests audio processing
    app.mocks
        .claude_mock
        .mock_chat_completion_with_text_response(
            "I'll help you process that audio. Let me reverse Sample 1 and add echo effects.",
        )
        .await;

    let response = app
        .chat(
            "Can you reverse Sample 1 and add a medium echo effect?",
            Some(conversation_id),
        )
        .await;
    response.assert_status(StatusCode::OK);

    // Step 3: Verify conversation history
    let conv_uuid = Uuid::parse_str(conversation_id).unwrap();
    let message_count = app.get_message_count(conv_uuid).await;
    assert!(message_count >= 4); // At least 2 user + 2 assistant messages

    // Step 4: Check that audio samples are accessible
    let samples_response = app.list_audio_samples().await;
    samples_response.assert_status(StatusCode::OK);

    let samples = samples_response.json_value();
    assert!(!samples["samples"].as_array().unwrap().is_empty());

    // Step 5: Get metadata for processed audio
    let metadata_response = app.get_audio_metadata("Sample 1").await;
    metadata_response.assert_status(StatusCode::OK);

    app.cleanup().await;
}

#[tokio::test]
async fn e2e_multi_user_concurrent_conversations() {
    let app = TestApp::new().await;
    let _test_data = app.seed_test_data().await;

    app.mocks
        .claude_mock
        .mock_chat_completion_with_text_response("Response from Claude")
        .await;

    // Simulate multiple users having concurrent conversations
    let user_tasks: Vec<_> = (1..=5)
        .map(|user_id| {
            let app = app.clone();
            tokio::spawn(async move {
                let mut conversation_id = None;
                let mut results = Vec::new();

                // Each user has a conversation with multiple exchanges
                for message_num in 1..=3 {
                    let message = format!("User {} message {}", user_id, message_num);
                    let response = app.chat(&message, conversation_id.as_deref()).await;

                    if response.status == StatusCode::OK {
                        let json = response.json_value();
                        if conversation_id.is_none() {
                            conversation_id =
                                Some(json["conversation_id"].as_str().unwrap().to_string());
                        }
                        results.push((user_id, message_num, true));
                    } else {
                        results.push((user_id, message_num, false));
                    }

                    // Small delay to simulate human typing
                    sleep(Duration::from_millis(100)).await;
                }

                (user_id, conversation_id, results)
            })
        })
        .collect();

    let results = futures::future::join_all(user_tasks).await;

    // Verify all users completed their conversations successfully
    for result in results {
        let (user_id, conversation_id, message_results) = result.unwrap();
        assert!(
            conversation_id.is_some(),
            "User {} should have a conversation ID",
            user_id
        );

        for (_, _, success) in message_results {
            assert!(success, "All messages should succeed for user {}", user_id);
        }
    }

    app.cleanup().await;
}

#[tokio::test]
async fn e2e_error_recovery_workflow() {
    let app = TestApp::new().await;
    let _test_data = app.seed_test_data().await;

    // Step 1: Start with a successful interaction
    app.mocks
        .claude_mock
        .mock_chat_completion_with_text_response("Hello! How can I help?")
        .await;

    let response = app.chat("Hello", None).await;
    response.assert_status(StatusCode::OK);

    let conversation_id = response.json_value()["conversation_id"]
        .as_str()
        .unwrap()
        .to_string();

    // Step 2: Simulate Claude API failure
    app.mocks
        .claude_mock
        .mock_chat_completion_error(503, "Service temporarily unavailable")
        .await;

    let response = app.chat("Process some audio", Some(&conversation_id)).await;
    response.assert_status(StatusCode::INTERNAL_SERVER_ERROR);

    // Step 3: Simulate recovery - Claude API comes back online
    app.mocks
        .claude_mock
        .mock_chat_completion_with_text_response("I'm back online! How can I help?")
        .await;

    let response = app
        .chat("Are you working now?", Some(&conversation_id))
        .await;
    response.assert_status(StatusCode::OK);

    // Step 4: Verify conversation integrity after error
    let conv_uuid = Uuid::parse_str(&conversation_id).unwrap();
    let message_count = app.get_message_count(conv_uuid).await;
    assert!(message_count >= 4); // Should have recorded the failed attempt too

    // Step 5: Test streaming engine failure and recovery
    app.mocks.streaming_mock.mock_processing_error().await;

    let parameters = json!({"reverse": true});
    let response = app.process_audio("Sample 1", parameters.clone()).await;
    response.assert_status(StatusCode::INTERNAL_SERVER_ERROR);

    // Recovery
    app.mocks
        .streaming_mock
        .mock_audio_processing_success("sample1.mp3")
        .await;

    let response = app.process_audio("Sample 1", parameters).await;
    response.assert_status(StatusCode::OK);

    app.cleanup().await;
}

#[tokio::test]
async fn e2e_complex_audio_processing_pipeline() {
    let app = TestApp::new().await;
    let _test_data = app.seed_test_data().await;

    // Setup complex audio processing scenario
    let effects = HashMap::from([
        ("reverse", "true"),
        ("speed", "1.2"),
        ("echo", "0.8:0.88:60:0.4"),
        ("volume", "0.9"),
        ("fade_in", "1.0"),
        ("fade_out", "2.0"),
    ]);

    app.mocks
        .streaming_mock
        .mock_effects_processing(effects.clone())
        .await;
    app.mocks
        .streaming_mock
        .mock_metadata_endpoint("sample1.mp3", 38.2)
        .await;

    // Step 1: Get initial sample information
    let samples_response = app.list_audio_samples().await;
    samples_response.assert_status(StatusCode::OK);

    let samples = samples_response.json_value();
    let sample_count = samples["samples"].as_array().unwrap().len();
    assert!(sample_count > 0);

    // Step 2: Process audio with multiple effects
    let complex_parameters = json!({
        "reverse": true,
        "speed": 1.2,
        "echo": "medium",
        "volume": 0.9,
        "fade_in": 1.0,
        "fade_out": 2.0,
        "normalize": true
    });

    let response = app.process_audio("Sample 1", complex_parameters).await;
    response.assert_status(StatusCode::OK);

    let processed_response = response.json_value();
    assert!(processed_response.get("processed_url").is_some());

    // Step 3: Verify effect parameters were passed correctly
    let verification_passed = app
        .mocks
        .streaming_mock
        .verify_effect_parameters(&effects)
        .await;
    assert!(
        verification_passed,
        "Effect parameters should be passed correctly"
    );

    // Step 4: Get metadata for processed audio
    let metadata_response = app.get_audio_metadata("Sample 1").await;
    metadata_response.assert_status(StatusCode::OK);

    let metadata = metadata_response.json_value();
    assert!(metadata["duration"].as_f64().unwrap() > 0.0);

    // Step 5: Test processing multiple samples in sequence
    for i in 2..=3 {
        let sample_name = format!("Sample {}", i);
        app.mocks
            .streaming_mock
            .mock_audio_processing_success(&format!("sample{}.mp3", i))
            .await;

        let simple_params = json!({"volume": 0.8});
        let response = app.process_audio(&sample_name, simple_params).await;
        response.assert_status(StatusCode::OK);
    }

    app.cleanup().await;
}

#[tokio::test]
async fn e2e_conversation_with_audio_processing_integration() {
    let app = TestApp::new().await;
    let _test_data = app.seed_test_data().await;

    // Setup mocks for conversation that leads to audio processing
    app.mocks.claude_mock.mock_chat_completion_success().await;
    app.mocks
        .streaming_mock
        .mock_audio_processing_success("sample1.mp3")
        .await;

    // Step 1: User asks about available samples
    let response = app
        .chat("What audio samples do you have available?", None)
        .await;
    response.assert_status(StatusCode::OK);

    let conversation_id = response.json_value()["conversation_id"]
        .as_str()
        .unwrap()
        .to_string();

    // Verify samples endpoint works
    let samples_response = app.list_audio_samples().await;
    samples_response.assert_status(StatusCode::OK);

    // Step 2: User requests specific audio processing through chat
    app.mocks
        .claude_mock
        .mock_chat_completion_with_text_response(
            "I'll process Sample 1 with reverb and speed adjustments for you.",
        )
        .await;

    let response = app
        .chat(
            "Please apply reverb to Sample 1 and make it play 1.5x faster",
            Some(&conversation_id),
        )
        .await;
    response.assert_status(StatusCode::OK);

    // Step 3: Simulate direct audio processing call (as if triggered by Claude)
    let audio_params = json!({
        "reverb": "hall",
        "speed": 1.5
    });

    let direct_processing = app.process_audio("Sample 1", audio_params).await;
    direct_processing.assert_status(StatusCode::OK);

    // Step 4: User asks for the result in conversation
    app.mocks
        .claude_mock
        .mock_chat_completion_with_text_response(
            "Your audio has been processed! The processed file is ready.",
        )
        .await;

    let response = app.chat("Is my audio ready?", Some(&conversation_id)).await;
    response.assert_status(StatusCode::OK);

    // Step 5: Verify conversation history includes all interactions
    let conv_uuid = Uuid::parse_str(&conversation_id).unwrap();
    let message_count = app.get_message_count(conv_uuid).await;
    assert!(message_count >= 6); // 3 user + 3 assistant messages

    app.cleanup().await;
}

#[tokio::test]
async fn e2e_data_consistency_across_failures() {
    let app = TestApp::new().await;
    let _test_data = app.seed_test_data().await;

    // Start a conversation
    app.mocks
        .claude_mock
        .mock_chat_completion_with_text_response("Hello!")
        .await;

    let response = app.chat("Start conversation", None).await;
    response.assert_status(StatusCode::OK);

    let conversation_id = response.json_value()["conversation_id"]
        .as_str()
        .unwrap()
        .to_string();
    let conv_uuid = Uuid::parse_str(&conversation_id).unwrap();

    let initial_count = app.get_message_count(conv_uuid).await;

    // Simulate partial failure - message sent but Claude fails
    app.mocks
        .claude_mock
        .mock_chat_completion_error(500, "Internal error")
        .await;

    let response = app.chat("This should fail", Some(&conversation_id)).await;
    response.assert_status(StatusCode::INTERNAL_SERVER_ERROR);

    // Check if user message was still recorded despite Claude failure
    let count_after_failure = app.get_message_count(conv_uuid).await;
    assert!(
        count_after_failure > initial_count,
        "User message should be recorded even if Claude fails"
    );

    // Recovery - continue conversation
    app.mocks
        .claude_mock
        .mock_chat_completion_with_text_response("I'm back!")
        .await;

    let response = app
        .chat("Are you working now?", Some(&conversation_id))
        .await;
    response.assert_status(StatusCode::OK);

    let final_count = app.get_message_count(conv_uuid).await;
    assert!(
        final_count >= count_after_failure + 2,
        "Should have user message + assistant response"
    );

    // Test audio processing data consistency
    let samples_before = app.list_audio_samples().await;
    samples_before.assert_status(StatusCode::OK);

    let sample_count_before = samples_before.json_value()["samples"]
        .as_array()
        .unwrap()
        .len();

    // Processing should not affect sample library
    app.mocks
        .streaming_mock
        .mock_audio_processing_success("sample1.mp3")
        .await;

    let params = json!({"volume": 0.8});
    let processing_response = app.process_audio("Sample 1", params).await;
    processing_response.assert_status(StatusCode::OK);

    let samples_after = app.list_audio_samples().await;
    samples_after.assert_status(StatusCode::OK);

    let sample_count_after = samples_after.json_value()["samples"]
        .as_array()
        .unwrap()
        .len();
    assert_eq!(
        sample_count_before, sample_count_after,
        "Sample library should remain consistent"
    );

    app.cleanup().await;
}

#[tokio::test]
async fn e2e_performance_under_realistic_load() {
    let app = TestApp::new().await;
    let _test_data = app.seed_test_data().await;

    app.mocks
        .claude_mock
        .mock_chat_completion_with_text_response("Quick response")
        .await;
    app.mocks
        .streaming_mock
        .mock_audio_processing_success("sample1.mp3")
        .await;

    let start_time = tokio::time::Instant::now();

    // Simulate realistic user behavior patterns
    let user_scenarios = vec![
        // Heavy chat users
        ("heavy_chat", 10, "chat"),
        // Audio processing users
        ("audio_user", 5, "audio"),
        // Mixed usage
        ("mixed_user", 8, "mixed"),
    ];

    let scenario_tasks: Vec<_> = user_scenarios
        .into_iter()
        .map(|(user_type, iterations, pattern)| {
            let app = app.clone();
            tokio::spawn(async move {
                let mut conversation_id = None;
                let mut success_count = 0;

                for i in 0..iterations {
                    match pattern {
                        "chat" => {
                            let message = format!("{} message {}", user_type, i);
                            let response = app.chat(&message, conversation_id.as_deref()).await;

                            if response.status == StatusCode::OK {
                                if conversation_id.is_none() {
                                    conversation_id = Some(
                                        response.json_value()["conversation_id"]
                                            .as_str()
                                            .unwrap()
                                            .to_string(),
                                    );
                                }
                                success_count += 1;
                            }
                        }
                        "audio" => {
                            let params = json!({"volume": 0.5 + (i as f64) * 0.1});
                            let response = app.process_audio("Sample 1", params).await;
                            if response.status == StatusCode::OK {
                                success_count += 1;
                            }
                        }
                        "mixed" => {
                            if i % 2 == 0 {
                                let message = format!("{} message {}", user_type, i);
                                let response = app.chat(&message, conversation_id.as_deref()).await;
                                if response.status == StatusCode::OK {
                                    if conversation_id.is_none() {
                                        conversation_id = Some(
                                            response.json_value()["conversation_id"]
                                                .as_str()
                                                .unwrap()
                                                .to_string(),
                                        );
                                    }
                                    success_count += 1;
                                }
                            } else {
                                let params = json!({"speed": 1.0 + (i as f64) * 0.1});
                                let response = app.process_audio("Sample 1", params).await;
                                if response.status == StatusCode::OK {
                                    success_count += 1;
                                }
                            }
                        }
                        _ => {}
                    }

                    // Realistic delay between requests
                    sleep(Duration::from_millis(50)).await;
                }

                (user_type, iterations, success_count)
            })
        })
        .collect();

    let results = futures::future::join_all(scenario_tasks).await;
    let total_duration = start_time.elapsed();

    // Verify performance and success rates
    let mut total_requests = 0;
    let mut total_successes = 0;

    for result in results {
        let (user_type, iterations, successes) = result.unwrap();
        total_requests += iterations;
        total_successes += successes;

        let success_rate = (successes as f64) / (iterations as f64);
        assert!(
            success_rate >= 0.95,
            "Success rate for {} should be >= 95%, got {}",
            user_type,
            success_rate
        );
    }

    // Performance assertions
    assert!(
        total_duration.as_secs() < 30,
        "Should complete within 30 seconds"
    );

    let overall_success_rate = (total_successes as f64) / (total_requests as f64);
    assert!(
        overall_success_rate >= 0.95,
        "Overall success rate should be >= 95%, got {}",
        overall_success_rate
    );

    app.cleanup().await;
}

#[tokio::test]
async fn e2e_full_system_integration_with_edge_cases() {
    let app = TestApp::new().await;
    let _test_data = app.seed_test_data().await;

    // Test edge cases in realistic scenarios

    // Edge case 1: Very long conversation
    app.mocks
        .claude_mock
        .mock_chat_completion_with_text_response("Response")
        .await;

    let response = app.chat("Start long conversation", None).await;
    let conversation_id = response.json_value()["conversation_id"]
        .as_str()
        .unwrap()
        .to_string();

    // Add many messages to test conversation limits
    for i in 1..=20 {
        let message = format!("Message number {} in this very long conversation", i);
        let response = app.chat(&message, Some(&conversation_id)).await;
        response.assert_status(StatusCode::OK);

        if i % 5 == 0 {
            sleep(Duration::from_millis(10)).await; // Brief pause
        }
    }

    // Edge case 2: Processing multiple audio files rapidly
    for i in 1..=5 {
        let sample_name = format!("Sample {}", i);
        app.mocks
            .streaming_mock
            .mock_audio_processing_success(&format!("sample{}.mp3", i))
            .await;

        let params = json!({
            "speed": 1.0 + (i as f64) * 0.1,
            "volume": 1.0 - (i as f64) * 0.1
        });

        let response = app.process_audio(&sample_name, params).await;
        response.assert_status(StatusCode::OK);
    }

    // Edge case 3: Malformed but not invalid requests
    let edge_case_params = json!({
        "speed": 1.0,
        "volume": 1.0,
        "unknown_param": "should_be_ignored",
        "empty_string": "",
        "null_value": null
    });

    app.mocks
        .streaming_mock
        .mock_audio_processing_success("sample1.mp3")
        .await;
    let response = app.process_audio("Sample 1", edge_case_params).await;
    // Should succeed or fail gracefully, not crash
    assert!(response.status.is_success() || response.status.is_client_error());

    // Edge case 4: Unicode in messages
    app.mocks
        .claude_mock
        .mock_chat_completion_with_text_response("Unicode response: 🎵🤖")
        .await;

    let unicode_message = "Process audio with effects: 🎵 reverb, echo 🔊, and speed ⚡";
    let response = app.chat(unicode_message, Some(&conversation_id)).await;
    response.assert_status(StatusCode::OK);

    // Edge case 5: Rapid sequential requests (stress test)
    let rapid_tasks: Vec<_> = (0..10)
        .map(|i| {
            let app = app.clone();
            tokio::spawn(async move {
                let response = app.health_check().await;
                response.status == StatusCode::OK
            })
        })
        .collect();

    let rapid_results = futures::future::join_all(rapid_tasks).await;
    let all_succeeded = rapid_results.into_iter().all(|r| r.unwrap());
    assert!(all_succeeded, "All rapid health checks should succeed");

    // Final verification: system should still be responsive
    let final_response = app.chat("Final test message", Some(&conversation_id)).await;
    final_response.assert_status(StatusCode::OK);

    let samples_response = app.list_audio_samples().await;
    samples_response.assert_status(StatusCode::OK);

    app.cleanup().await;
}

#[tokio::test]
async fn e2e_graceful_degradation_scenarios() {
    let app = TestApp::new().await;
    let _test_data = app.seed_test_data().await;

    // Scenario 1: Claude API degraded but not completely down
    app.mocks.claude_mock.mock_rate_limit_error().await;

    let response = app.chat("Test message during rate limiting", None).await;
    assert_eq!(response.status, StatusCode::TOO_MANY_REQUESTS);

    // Scenario 2: Streaming engine slow but responsive
    app.mocks
        .streaming_mock
        .mock_audio_processing_success("sample1.mp3")
        .await;

    let params = json!({"volume": 0.8});
    let response = app.process_audio("Sample 1", params).await;
    response.assert_status(StatusCode::OK);

    // Scenario 3: Partial system recovery
    app.mocks
        .claude_mock
        .mock_chat_completion_with_text_response("Service recovered")
        .await;

    let response = app.chat("Are you back online?", None).await;
    response.assert_status(StatusCode::OK);

    // Scenario 4: Database under stress but functional
    let stress_tasks: Vec<_> = (0..20)
        .map(|i| {
            let app = app.clone();
            tokio::spawn(async move {
                let response = app.list_audio_samples().await;
                response.status == StatusCode::OK
            })
        })
        .collect();

    let stress_results = futures::future::join_all(stress_tasks).await;
    let success_rate = stress_results
        .iter()
        .filter(|r| *r.as_ref().unwrap())
        .count() as f64
        / stress_results.len() as f64;

    // Should maintain good performance even under stress
    assert!(
        success_rate >= 0.90,
        "Database should maintain 90%+ success rate under stress"
    );

    app.cleanup().await;
}

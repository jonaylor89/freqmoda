use axum::{extract::State, response::Json};
use serde_json::Value;
use tracing::instrument;

use crate::{
    database::{
        create_conversation, get_all_audio_samples, get_audio_sample_by_key,
        get_audio_sample_by_title, get_conversation, get_conversation_messages, store_message,
        update_conversation_timestamp,
    },
    error::{AppError, Result},
    models::{ChatRequest, ChatResponse, ClaudeContent, ClaudeMessage},
    services::{claude::ClaudeService, streaming_engine::StreamingEngineService},
    state::AppState,
};

#[instrument(skip(state, request))]
pub async fn chat(
    State(state): State<AppState>,
    Json(request): Json<ChatRequest>,
) -> Result<Json<ChatResponse>> {
    tracing::info!("Starting chat request processing");

    // Get or create conversation
    let conversation = if let Some(conversation_id) = request.conversation_id {
        tracing::debug!("Looking up existing conversation: {}", conversation_id);
        match get_conversation(&state.db, &conversation_id).await {
            Ok(Some(conv)) => {
                tracing::debug!("Found existing conversation: {}", conversation_id);
                conv
            }
            Ok(None) => {
                tracing::warn!(
                    "Conversation not found: {}, creating new one",
                    conversation_id
                );
                match create_conversation(&state.db, None, None).await {
                    Ok(conv) => {
                        tracing::debug!(
                            "Created new conversation: {} (replacing missing {})",
                            conv.id,
                            conversation_id
                        );
                        conv
                    }
                    Err(e) => {
                        tracing::error!("Failed to create replacement conversation: {:?}", e);
                        return Err(e);
                    }
                }
            }
            Err(e) => {
                tracing::error!(
                    "Database error getting conversation {}: {:?}",
                    conversation_id,
                    e
                );
                return Err(e);
            }
        }
    } else {
        tracing::debug!("Creating new conversation");
        match create_conversation(&state.db, None, None).await {
            Ok(conv) => {
                tracing::debug!("Created new conversation: {}", conv.id);
                conv
            }
            Err(e) => {
                tracing::error!("Failed to create conversation: {:?}", e);
                return Err(e);
            }
        }
    };

    // Store user message
    tracing::debug!("Storing user message for conversation: {}", conversation.id);
    if let Err(e) = store_message(&state.db, &conversation.id, "user", &request.message).await {
        tracing::error!("Failed to store user message: {:?}", e);
        return Err(e);
    }

    // Get conversation history
    tracing::debug!("Retrieving conversation history for: {}", conversation.id);
    let messages =
        match get_conversation_messages(&state.db, &conversation.id, Some(20), None).await {
            Ok(msgs) => {
                tracing::debug!(
                    "Retrieved {} messages from conversation history",
                    msgs.len()
                );
                msgs
            }
            Err(e) => {
                tracing::error!("Failed to get conversation messages: {:?}", e);
                return Err(e);
            }
        };

    // Convert to Claude format
    let claude_messages: Vec<ClaudeMessage> = messages
        .into_iter()
        .map(|msg| ClaudeMessage {
            role: msg.role,
            content: msg.content,
        })
        .collect();

    tracing::debug!(
        "Converted {} messages to Claude format",
        claude_messages.len()
    );

    // Debug: Log the actual conversation history being sent to Claude
    for (i, msg) in claude_messages.iter().enumerate() {
        tracing::debug!(
            "Message {}: role='{}', content='{}'",
            i,
            msg.role,
            msg.content
        );
    }

    // Create Claude service
    let claude = ClaudeService::new(
        state.http_client.clone(),
        state.settings.claude.api_key.clone(),
        state.settings.claude.base_url.clone(),
        state.settings.claude.model.clone(),
    );

    // Send to Claude
    tracing::debug!("Sending message to Claude API");
    let claude_response = match claude.send_message(claude_messages, None).await {
        Ok(response) => {
            tracing::debug!(
                "Received response from Claude with {} content items",
                response.content.len()
            );
            response
        }
        Err(e) => {
            tracing::error!("Claude API request failed: {:?}", e);
            return Err(e);
        }
    };

    // Process Claude's response
    let mut response_text = String::new();
    let mut processed_urls = Vec::new();

    for (i, content) in claude_response.content.iter().enumerate() {
        tracing::debug!("Processing Claude response content item {}", i);
        match content {
            ClaudeContent::Text { text } => {
                tracing::debug!("Processing text content ({} chars)", text.len());
                response_text.push_str(text);
            }
            ClaudeContent::ToolUse { name, input, .. } => {
                tracing::debug!("Processing tool use: {}", name);
                match name.as_str() {
                    "process_audio" => {
                        tracing::debug!("Handling process_audio tool call with input: {:?}", input);
                        match handle_process_audio_tool(&state, input).await {
                            Ok(processed_url) => {
                                tracing::info!("Successfully processed audio: {}", processed_url);
                                processed_urls.push(processed_url.clone());
                                response_text
                                    .push_str(&format!("\n\nProcessed audio: {}", processed_url));
                            }
                            Err(e) => {
                                tracing::error!("Failed to process audio tool: {:?}", e);
                                return Err(e);
                            }
                        }
                    }
                    "list_audio_samples" => {
                        tracing::debug!("Handling list_audio_samples tool call");
                        match handle_list_audio_samples_tool(&state).await {
                            Ok(samples_info) => {
                                tracing::debug!("Successfully retrieved audio samples list");
                                response_text
                                    .push_str(&format!("\n\nAvailable samples:\n{}", samples_info));
                            }
                            Err(e) => {
                                tracing::error!("Failed to list audio samples: {:?}", e);
                                return Err(e);
                            }
                        }
                    }
                    "format_response" => {
                        tracing::debug!(
                            "Handling format_response tool call with input: {:?}",
                            input
                        );
                        match handle_format_response_tool(input).await {
                            Ok(formatted_response) => {
                                tracing::debug!("Successfully formatted response");
                                response_text.clear(); // Clear any previous text
                                response_text.push_str(&formatted_response);
                            }
                            Err(e) => {
                                tracing::error!("Failed to format response: {:?}", e);
                                return Err(e);
                            }
                        }
                    }
                    _ => {
                        tracing::warn!("Unknown tool requested by Claude: {}", name);
                    }
                }
            }
        }
    }

    // Store assistant's response
    tracing::debug!("Storing assistant response ({} chars)", response_text.len());
    if let Err(e) = store_message(&state.db, &conversation.id, "assistant", &response_text).await {
        tracing::error!("Failed to store assistant message: {:?}", e);
        return Err(e);
    }

    // Update conversation timestamp
    tracing::debug!("Updating conversation timestamp");
    if let Err(e) = update_conversation_timestamp(&state.db, &conversation.id).await {
        tracing::error!("Failed to update conversation timestamp: {:?}", e);
        return Err(e);
    }

    let response = ChatResponse {
        message: response_text,
        conversation_id: conversation.id,
    };

    tracing::info!(
        "Chat request completed successfully for conversation: {}",
        conversation.id
    );
    Ok(Json(response))
}

#[instrument(skip(state))]
async fn handle_process_audio_tool(state: &AppState, input: &Value) -> Result<String> {
    let audio_name = input
        .get("audio_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Validation("Missing audio_name in tool call".to_string()))?;

    // Resolve audio name
    let resolved_audio_name = resolve_audio_name(state, audio_name).await?;

    let streaming_engine = StreamingEngineService::new(
        state.http_client.clone(),
        state.settings.streaming_engine.base_url.clone(),
    );

    let processed_url = streaming_engine
        .process_audio(&resolved_audio_name, input)
        .await?;

    Ok(processed_url)
}

#[instrument(skip(state))]
async fn handle_list_audio_samples_tool(state: &AppState) -> Result<String> {
    let samples = get_all_audio_samples(&state.db).await?;

    let mut samples_text = String::new();
    for sample in samples {
        samples_text.push_str(&format!(
            "- {} ({}): {:.1}s\n",
            sample.title,
            sample.streaming_key,
            sample.duration.unwrap_or(0.0)
        ));
    }

    Ok(samples_text)
}

#[instrument]
async fn handle_format_response_tool(input: &Value) -> Result<String> {
    let description = input
        .get("description")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            AppError::Validation("Missing description in format_response tool call".to_string())
        })?;

    let sample_url = input
        .get("sample_url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            AppError::Validation("Missing sample_url in format_response tool call".to_string())
        })?;

    let formatted_response = format!(
        "<text>\n{}\n</text>\n\n<sample_url>\n{}\n</sample_url>",
        description, sample_url
    );

    Ok(formatted_response)
}

#[instrument(skip(state))]
async fn resolve_audio_name(state: &AppState, audio_name: &str) -> Result<String> {
    // First, try to find by exact streaming key
    if let Some(sample) = get_audio_sample_by_key(&state.db, audio_name).await? {
        return Ok(sample.streaming_key);
    }

    // Then try to find by title (case insensitive)
    if let Some(sample) = get_audio_sample_by_title(&state.db, audio_name).await? {
        return Ok(sample.streaming_key);
    }

    // Check if it looks like a sample reference (e.g., "Sample 1", "sample1", etc.)
    let normalized = audio_name.to_lowercase();
    if normalized.starts_with("sample") {
        // Extract number and try to find corresponding sample
        let number_part = normalized
            .chars()
            .filter(|c| c.is_ascii_digit())
            .collect::<String>();

        if !number_part.is_empty() {
            let sample_key = format!("sample{}.mp3", number_part);
            if let Some(sample) = get_audio_sample_by_key(&state.db, &sample_key).await? {
                return Ok(sample.streaming_key);
            }
        }
    }

    // If no sample found, assume it's a direct file reference
    Ok(audio_name.to_string())
}

use askama::Template;
use axum::{
    Json,
    body::Body,
    extract::{Extension, Form, Path, State},
    http::{StatusCode, header},
    response::{Html, Redirect, Response},
};
use chrono::Utc;
use color_eyre::Result;
use redis::AsyncCommands;
use serde::Deserialize;
use tracing::{debug, error, info, instrument};
use uuid::Uuid;

use crate::{
    database::{get_all_audio_samples, get_audio_sample_by_key},
    error::AppError,
    handlers::chat::chat,
    models::{AudioSample, ChatRequest},
    state::AppState,
};

// Maximum message length constants
const MAX_CHAT_MESSAGE_LENGTH: usize = 2000;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    samples: Vec<AudioSample>,
}

#[derive(Template)]
#[template(path = "sample_chat.html")]
struct SampleChatTemplate {
    sample: AudioSample,
    messages: Vec<DisplayMessage>,
    session_requests_remaining: i32,
    redis_available: bool,
    streaming_engine_base_url: String,
}

#[derive(Template)]
#[template(path = "chat.html")]
struct ChatTemplate {
    messages: Vec<DisplayMessage>,
    session_id: String,
    session_requests_remaining: i32,
    redis_available: bool,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct DisplayMessage {
    role: String,
    content: String,
    audio_url: String,
}

#[derive(Deserialize)]
pub struct ChatForm {
    message: String,
}

#[instrument(skip(state))]
pub async fn index_page(State(state): State<AppState>) -> Result<Html<String>, StatusCode> {
    let samples = get_all_audio_samples(&state.db).await.map_err(|e| {
        error!("Failed to get audio samples: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let template = IndexTemplate {
        samples,
    };

    let html = template.render().map_err(|e| {
        error!("Failed to render index template: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Html(html))
}

#[instrument(skip(state))]
pub async fn sample_chat_page(
    State(state): State<AppState>,
    Extension(session_id): Extension<String>,
    Path(sample_id): Path<String>,
) -> Result<Html<String>, StatusCode> {
    // Get the sample
    let sample = crate::database::get_audio_sample_by_key(&state.db, &sample_id)
        .await
        .map_err(|e| {
            error!("Failed to get sample: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            error!("Sample not found: {}", sample_id);
            StatusCode::NOT_FOUND
        })?;

    // Check Redis availability
    let redis_available = check_redis_availability(&state).await;

    let (remaining, messages) = if redis_available {
        let remaining = get_session_requests_remaining(&state, &session_id)
            .await
            .unwrap_or_else(|e| {
                error!("Failed to get session requests remaining: {:?}", e);
                0
            });

        let messages = get_recent_sample_messages(&state, &session_id, &sample_id)
            .await
            .unwrap_or_else(|e| {
                error!("Failed to get recent messages: {:?}", e);
                Vec::new()
            });

        (remaining, messages)
    } else {
        error!("Redis is unavailable, disabling chat functionality");
        (0, Vec::new())
    };

    let template = SampleChatTemplate {
        sample,
        messages,
        session_requests_remaining: remaining,
        redis_available,
        streaming_engine_base_url: state.settings.streaming_engine.base_url.clone(),
    };

    let html = template.render().map_err(|e| {
        error!("Failed to render sample chat template: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Html(html))
}

#[instrument(skip(state))]
pub async fn chat_page(
    State(state): State<AppState>,
    Extension(session_id): Extension<String>,
) -> Result<Html<String>, StatusCode> {
    // Check Redis availability first
    let redis_available = check_redis_availability(&state).await;

    let (remaining, messages) = if redis_available {
        // Redis is available, get real data
        let remaining = get_session_requests_remaining(&state, &session_id)
            .await
            .unwrap_or_else(|e| {
                error!("Failed to get session requests remaining: {:?}", e);
                0 // Conservative fallback
            });

        let messages = get_recent_messages(&state, &session_id)
            .await
            .unwrap_or_else(|e| {
                error!("Failed to get recent messages: {:?}", e);
                Vec::new()
            });

        (remaining, messages)
    } else {
        // Redis is down, use safe defaults
        error!("Redis is unavailable, disabling chat functionality");
        (0, Vec::new())
    };

    info!("Rendering chat page with {} messages", messages.len());

    let template = ChatTemplate {
        messages,
        session_id,
        session_requests_remaining: remaining,
        redis_available,
    };

    let html = template.render().map_err(|e| {
        error!("Failed to render chat template: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Html(html))
}

#[instrument(skip(state, form))]
pub async fn sample_chat_form_handler(
    State(state): State<AppState>,
    Extension(session_id): Extension<String>,
    Path(sample_id): Path<String>,
    Form(form): Form<ChatForm>,
) -> Result<Redirect, StatusCode> {
    // Validate message length
    if form.message.len() > MAX_CHAT_MESSAGE_LENGTH {
        return Ok(Redirect::to(&format!(
            "/sample/{}?error=message_too_long",
            sample_id
        )));
    }

    // Check Redis availability first
    if !check_redis_availability(&state).await {
        error!("Redis unavailable, rejecting chat request");
        return Ok(Redirect::to(&format!(
            "/sample/{}?error=redis_unavailable",
            sample_id
        )));
    }

    // Check if user has requests remaining
    let remaining = get_session_requests_remaining(&state, &session_id)
        .await
        .map_err(|e| {
            error!("Failed to get session requests remaining: {:?}", e);
            StatusCode::SERVICE_UNAVAILABLE
        })?;
    if remaining <= 0 {
        return Ok(Redirect::to(&format!(
            "/sample/{}?error=quota_exceeded",
            sample_id
        )));
    }

    // Get or create session-based conversation ID
    let conversation_id = get_session_conversation_id(&state, &session_id)
        .await
        .map_err(|e| {
            error!("Failed to get session conversation ID: {:?}", e);
            StatusCode::SERVICE_UNAVAILABLE
        })?;

    // Get the sample to inject context
    let sample = get_audio_sample_by_key(&state.db, &sample_id)
        .await
        .map_err(|e| {
            error!("Failed to get sample for context: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            error!("Sample not found for context: {}", sample_id);
            StatusCode::NOT_FOUND
        })?;

    // Create enhanced message with sample context
    let enhanced_message = format!(
        "I'm working with the audio sample '{}' (file: {}). {}",
        sample.title, sample.streaming_key, form.message,
    );

    // Create chat request
    let chat_request = ChatRequest {
        message: enhanced_message,
        conversation_id: Some(conversation_id),
    };

    // Use the real chat API logic
    match chat(State(state.clone()), Json(chat_request)).await {
        Ok(response) => {
            let chat_response = response.0;

            // Update conversation ID if changed
            if chat_response.conversation_id != conversation_id {
                debug!(
                    "Conversation ID changed from {} to {}, updating session",
                    conversation_id, chat_response.conversation_id
                );
                if let Err(e) = update_session_conversation_id(
                    &state,
                    &session_id,
                    chat_response.conversation_id,
                )
                .await
                {
                    error!(
                        "Failed to update session conversation ID (continuing): {:?}",
                        e
                    );
                }
            }

            // Store messages for this specific sample (store original message, not enhanced)
            store_sample_message(&state, &session_id, &sample_id, "user", &form.message)
                .await
                .map_err(|e| {
                    error!("Failed to store user message: {:?}", e);
                    StatusCode::SERVICE_UNAVAILABLE
                })?;
            store_sample_message(
                &state,
                &session_id,
                &sample_id,
                "assistant",
                &chat_response.message,
            )
            .await
            .map_err(|e| {
                error!("Failed to store assistant message: {:?}", e);
                StatusCode::SERVICE_UNAVAILABLE
            })?;

        }
        Err(e) => {
            error!(
                "Chat API call failed for sample '{}' with message '{}': {:?}",
                sample_id, form.message, e
            );

            // Log additional context about the error
            match &e {
                AppError::StreamingEngine(msg) => {
                    error!("Streaming engine error details: {}", msg);
                }
                AppError::HttpClient(http_err) => {
                    error!("HTTP client error details: {}", http_err);
                }
                AppError::OpenAI(openai_err) => {
                    error!("OpenAI API error details: {}", openai_err);
                }
                AppError::Database(db_err) => {
                    error!("Database error details: {}", db_err);
                }
                AppError::Validation(val_err) => {
                    error!("Validation error details: {}", val_err);
                }
                _ => {
                    error!("Other error type: {:?}", e);
                }
            }

            let _ =
                store_sample_message(&state, &session_id, &sample_id, "user", &form.message).await;
            let error_message =
                "Sorry, I encountered an error processing your request. Please try again.";
            let _ =
                store_sample_message(&state, &session_id, &sample_id, "assistant", error_message)
                    .await;
            return Ok(Redirect::to(&format!(
                "/sample/{}?error=chat_failed",
                sample_id
            )));
        }
    }

    // Decrement session request count
    decrement_session_requests(&state, &session_id)
        .await
        .map_err(|e| {
            error!("Failed to decrement session requests: {:?}", e);
            StatusCode::SERVICE_UNAVAILABLE
        })?;

    // Redirect back to sample chat page
    Ok(Redirect::to(&format!("/sample/{}", sample_id)))
}

#[instrument(skip(state, form))]
pub async fn chat_form_handler(
    State(state): State<AppState>,
    Extension(session_id): Extension<String>,
    Form(form): Form<ChatForm>,
) -> Result<Redirect, StatusCode> {
    // Validate message length
    if form.message.len() > MAX_CHAT_MESSAGE_LENGTH {
        return Ok(Redirect::to("/?error=message_too_long"));
    }

    // Check Redis availability first
    if !check_redis_availability(&state).await {
        error!("Redis unavailable, rejecting chat request");
        return Ok(Redirect::to("/?error=redis_unavailable"));
    }

    // Check if user has requests remaining
    let remaining = get_session_requests_remaining(&state, &session_id)
        .await
        .map_err(|e| {
            error!("Failed to get session requests remaining: {:?}", e);
            StatusCode::SERVICE_UNAVAILABLE
        })?;
    if remaining <= 0 {
        return Ok(Redirect::to("/?error=quota_exceeded"));
    }

    // Get or create session-based conversation ID
    let conversation_id = get_session_conversation_id(&state, &session_id)
        .await
        .map_err(|e| {
            error!("Failed to get session conversation ID: {:?}", e);
            StatusCode::SERVICE_UNAVAILABLE
        })?;

    // Create chat request
    let chat_request = ChatRequest {
        message: form.message.clone(),
        conversation_id: Some(conversation_id),
    };

    // Use the real chat API logic
    match chat(State(state.clone()), Json(chat_request)).await {
        Ok(response) => {
            let chat_response = response.0; // Extract from Json wrapper

            // If the returned conversation ID is different from what we sent,
            // update the session to use the new conversation ID
            if chat_response.conversation_id != conversation_id {
                debug!(
                    "Conversation ID changed from {} to {}, updating session",
                    conversation_id, chat_response.conversation_id
                );
                if let Err(e) = update_session_conversation_id(
                    &state,
                    &session_id,
                    chat_response.conversation_id,
                )
                .await
                {
                    error!(
                        "Failed to update session conversation ID (continuing): {:?}",
                        e
                    );
                }
            }

            // Store both user and assistant messages in Redis for the HTML interface
            store_message(&state, &session_id, "user", &form.message)
                .await
                .map_err(|e| {
                    error!("Failed to store user message: {:?}", e);
                    StatusCode::SERVICE_UNAVAILABLE
                })?;
            store_message(&state, &session_id, "assistant", &chat_response.message)
                .await
                .map_err(|e| {
                    error!("Failed to store assistant message: {:?}", e);
                    StatusCode::SERVICE_UNAVAILABLE
                })?;
        }
        Err(e) => {
            error!("Chat API call failed: {:?}", e);
            // If chat fails, store an error message
            let _ = store_message(&state, &session_id, "user", &form.message).await;
            let error_message =
                "Sorry, I encountered an error processing your request. Please try again.";
            let _ = store_message(&state, &session_id, "assistant", error_message).await;
            return Ok(Redirect::to("/?error=chat_failed"));
        }
    }

    // Decrement session request count
    decrement_session_requests(&state, &session_id)
        .await
        .map_err(|e| {
            error!("Failed to decrement session requests: {:?}", e);
            StatusCode::SERVICE_UNAVAILABLE
        })?;

    // Redirect back to chat page to show new messages
    Ok(Redirect::to("/"))
}

#[instrument(skip(state))]
pub async fn download_audio(
    State(state): State<AppState>,
    Path(filename): Path<String>,
) -> Result<Response<Body>, StatusCode> {
    // Construct the streaming engine URL
    let audio_url = format!("{}/{}", state.settings.streaming_engine.base_url, filename);

    // Fetch the audio file
    let response = state
        .http_client
        .get(&audio_url)
        .send()
        .await
        .map_err(|e| {
            error!("Failed to fetch audio file: {:?}", e);
            StatusCode::NOT_FOUND
        })?;

    if !response.status().is_success() {
        error!("Audio file not found: {}", audio_url);
        return Err(StatusCode::NOT_FOUND);
    }

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("audio/mpeg")
        .to_string();

    let bytes = response.bytes().await.map_err(|e| {
        error!("Failed to read audio file bytes: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Create download response with proper headers
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(
            header::CONTENT_DISPOSITION,
            format!(
                "attachment; filename=\"audio-demo-processed-{}.mp3\"",
                Utc::now().timestamp()
            ),
        )
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(bytes))
        .map_err(|e| {
            error!("Failed to build download response: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(response)
}

#[instrument(skip(state))]
async fn check_redis_availability(state: &AppState) -> bool {
    match state.redis.get_multiplexed_async_connection().await {
        Ok(mut conn) => {
            // Try a simple ping command using AsyncCommands
            match conn.ping().await {
                Ok(()) => {
                    debug!("Redis is available");
                    true
                }
                Err(e) => {
                    error!("Redis ping failed: {:?}", e);
                    false
                }
            }
        }
        Err(e) => {
            error!("Failed to connect to Redis: {:?}", e);
            false
        }
    }
}

#[instrument(skip(state))]
async fn get_session_conversation_id(state: &AppState, session_id: &str) -> Result<Uuid> {
    let mut conn = state.redis.get_multiplexed_async_connection().await?;

    let key = format!("session_conversation:{}", session_id);
    let conversation_id: Option<String> = conn.get(&key).await.unwrap_or(None);

    if let Some(id_str) = conversation_id {
        if let Ok(id) = Uuid::parse_str(&id_str) {
            Ok(id)
        } else {
            // Invalid UUID stored, create a new one
            let new_id = Uuid::new_v4();
            let _: () = conn.set(&key, new_id.to_string()).await?;

            // Set expiry to 24 hours
            let _: () = conn.expire(&key, 86400).await?;

            Ok(new_id)
        }
    } else {
        // Create new conversation ID for this session
        let new_id = Uuid::new_v4();
        let _: () = conn.set(&key, new_id.to_string()).await?;

        // Set expiry to 24 hours
        let _: () = conn.expire(&key, 86400).await?;

        Ok(new_id)
    }
}

#[instrument(skip(state))]
async fn update_session_conversation_id(
    state: &AppState,
    session_id: &str,
    conversation_id: Uuid,
) -> Result<()> {
    let mut conn = state.redis.get_multiplexed_async_connection().await?;

    let key = format!("session_conversation:{}", session_id);
    let _: () = conn.set(&key, conversation_id.to_string()).await?;

    // Set expiry to 24 hours
    let _: () = conn.expire(&key, 86400).await?;

    Ok(())
}

#[instrument(skip(state))]
async fn get_session_requests_remaining(state: &AppState, session_id: &str) -> Result<i32> {
    let mut conn = state.redis.get_multiplexed_async_connection().await?;

    let key = format!("session_requests:{}", session_id);
    let used: i32 = conn.get(&key).await.unwrap_or(0);

    debug!("Used requests: {}", used);

    Ok(20 - used) // 20 requests per session
}

#[instrument(skip(state))]
async fn decrement_session_requests(state: &AppState, session_id: &str) -> Result<()> {
    let mut conn = state.redis.get_multiplexed_async_connection().await?;

    let key = format!("session_requests:{}", session_id);
    let _: () = conn.incr(&key, 1).await?;

    // Set expiry to 24 hours
    let _: () = conn.expire(&key, 86400).await?;

    Ok(())
}

#[instrument(skip(state))]
async fn get_recent_sample_messages(
    state: &AppState,
    session_id: &str,
    sample_id: &str,
) -> Result<Vec<DisplayMessage>> {
    let mut conn = state.redis.get_multiplexed_async_connection().await?;

    let key = format!("session_sample_messages:{}:{}", session_id, sample_id);
    let messages_json: Vec<String> = conn.lrange(&key, 0, -1).await.unwrap_or_default();

    let mut messages = Vec::new();
    for msg_json in messages_json {
        if let Ok(msg) = serde_json::from_str::<DisplayMessage>(&msg_json) {
            messages.push(msg);
        }
    }

    Ok(messages)
}

#[instrument(skip(state))]
async fn get_recent_messages(state: &AppState, session_id: &str) -> Result<Vec<DisplayMessage>> {
    // For MVP, just return empty messages for now
    // We'll store conversation in Redis later
    let mut conn = state.redis.get_multiplexed_async_connection().await?;

    let key = format!("session_messages:{}", session_id);
    let messages_json: Vec<String> = conn.lrange(&key, 0, -1).await.unwrap_or_default();

    let mut messages = Vec::new();
    for msg_json in messages_json {
        if let Ok(msg) = serde_json::from_str::<DisplayMessage>(&msg_json) {
            messages.push(msg);
        }
    }

    Ok(messages)
}

#[instrument(skip(state))]
async fn store_sample_message(
    state: &AppState,
    session_id: &str,
    sample_id: &str,
    role: &str,
    content: &str,
) -> Result<()> {
    let mut conn = state.redis.get_multiplexed_async_connection().await?;

    let (cleaned_content, audio_url) = if role == "assistant" {
        // First try the new structured format
        let (text_content, structured_url) =
            parse_structured_message(content, &state.settings.streaming_engine.base_url);
        if let Some(url) = structured_url {
            (text_content, Some(url))
        } else {
            // Fallback to original URL extraction
            extract_and_remove_audio_url(content, &state.settings.streaming_engine.base_url)
        }
    } else {
        (content.to_string(), None)
    };

    let message = DisplayMessage {
        role: role.to_string(),
        content: cleaned_content,
        audio_url: audio_url.unwrap_or_default(),
    };

    let key = format!("session_sample_messages:{}:{}", session_id, sample_id);
    let msg_json = serde_json::to_string(&message).unwrap_or_default();

    let _: () = conn.rpush(&key, msg_json).await?;

    // Keep only last 20 messages
    let _: () = conn.ltrim(&key, -20, -1).await?;

    // Set expiry to 24 hours
    let _: () = conn.expire(&key, 86400).await?;

    Ok(())
}

#[instrument(skip(state))]
async fn store_message(
    state: &AppState,
    session_id: &str,
    role: &str,
    content: &str,
) -> Result<()> {
    let mut conn = state.redis.get_multiplexed_async_connection().await?;

    let (cleaned_content, audio_url) = if role == "assistant" {
        // First try the new structured format
        let (text_content, structured_url) =
            parse_structured_message(content, &state.settings.streaming_engine.base_url);
        if let Some(url) = structured_url {
            (text_content, Some(url))
        } else {
            // Fallback to original URL extraction
            extract_and_remove_audio_url(content, &state.settings.streaming_engine.base_url)
        }
    } else {
        (content.to_string(), None)
    };

    let message = DisplayMessage {
        role: role.to_string(),
        content: cleaned_content,
        audio_url: audio_url.unwrap_or_default(),
    };

    let key = format!("session_messages:{}", session_id);
    let msg_json = serde_json::to_string(&message).unwrap_or_default();

    let _: () = conn.rpush(&key, msg_json).await?;

    // Keep only last 20 messages
    let _: () = conn.ltrim(&key, -20, -1).await?;

    // Set expiry to 24 hours
    let _: () = conn.expire(&key, 86400).await?;

    Ok(())
}

#[instrument]
fn extract_and_remove_audio_url(
    content: &str,
    streaming_engine_base_url: &str,
) -> (String, Option<String>) {
    // Look for URLs in the content that point to the streaming engine
    if let Some(start) = content.find(streaming_engine_base_url) {
        let url_end = if let Some(end) = content[start..].find(char::is_whitespace) {
            start + end
        } else {
            // URL goes to end of string
            content.len()
        };

        let url = content[start..url_end].to_string();

        // Remove the URL from content, handling potential whitespace
        let mut cleaned_content = String::new();
        cleaned_content.push_str(&content[..start]);

        // Skip any trailing whitespace after the URL
        let remaining_start =
            if url_end < content.len() && content.chars().nth(url_end).unwrap().is_whitespace() {
                url_end + 1
            } else {
                url_end
            };

        if remaining_start < content.len() {
            cleaned_content.push_str(&content[remaining_start..]);
        }

        // Clean up any double spaces or trailing whitespace
        let cleaned_content = cleaned_content.trim().to_string();

        (cleaned_content, Some(url))
    } else {
        (content.to_string(), None)
    }
}

#[instrument]
fn extract_audio_url(content: &str, streaming_engine_base_url: &str) -> Option<String> {
    let (_, url) = extract_and_remove_audio_url(content, streaming_engine_base_url);
    url
}

fn parse_structured_message(content: &str, base_url: &str) -> (String, Option<String>) {
    // Debug logging to see what Claude is actually returning
    tracing::debug!("Claude response content: {}", content);

    // Check if the message has the new structured format
    if let (Some(text_start), Some(text_end)) = (content.find("<text>"), content.find("</text>"))
        && let (Some(url_start), Some(url_end)) =
            (content.find("<sample_url>"), content.find("</sample_url>"))
        {
            // Extract text content
            let text_content = content[text_start + 6..text_end].trim().to_string();

            // Extract sample URL
            let sample_url = content[url_start + 12..url_end].trim().to_string();

            tracing::debug!(
                "Parsed structured message - text: '{}', url: '{}'",
                text_content,
                sample_url
            );
            return (text_content, Some(sample_url));
        }

    // Claude is being stubborn - try to parse the old format and convert it to structured format
    if let Some(url) = extract_audio_url(content, base_url) {
        tracing::debug!("Found audio URL in old format, converting to structured format");

        // Extract the descriptive text (everything before "Processed audio:")
        let description = if let Some(pos) = content.find("Processed audio:") {
            content[..pos].trim().to_string()
        } else {
            // If no "Processed audio:" text, try to find text before the URL
            let url_pos = content.find(&url).unwrap_or(content.len());
            content[..url_pos].trim().to_string()
        };

        // Clean up the description - remove empty lines and common prefixes
        let clean_description = description
            .lines()
            .filter(|line| !line.trim().is_empty())
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string();

        let final_description = if clean_description.is_empty() {
            "Audio processed successfully".to_string()
        } else {
            clean_description
        };

        tracing::debug!(
            "Converted old format - description: '{}', url: '{}'",
            final_description,
            url
        );
        return (final_description, Some(url));
    }

    // Fallback to original behavior if not structured
    tracing::debug!("No structured format found, using fallback");
    (content.to_string(), None)
}

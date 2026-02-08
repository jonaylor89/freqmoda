use axum::{extract::State, response::Json};
use serde_json::Value;

use crate::{
    database::{get_all_audio_samples, get_audio_sample_by_key, get_audio_sample_by_title},
    error::Result,
    models::{AudioProcessRequest, AudioProcessResponse},
    services::streaming_engine::StreamingEngineService,
    state::AppState,
};

pub async fn process_audio(
    State(state): State<AppState>,
    Json(request): Json<AudioProcessRequest>,
) -> Result<Json<AudioProcessResponse>> {
    let streaming_engine = StreamingEngineService::new(
        state.http_client.clone(),
        state.settings.streaming_engine.base_url.clone(),
    );

    // Resolve audio name - check if it's a sample name or direct file
    let audio_name = resolve_audio_name(&state, &request.audio_name).await?;

    let processed_url = streaming_engine
        .process_audio(&audio_name, &request.parameters)
        .await?;

    let response = AudioProcessResponse {
        processed_url,
        parameters: request.parameters,
    };

    Ok(Json(response))
}

pub async fn list_audio_samples(State(state): State<AppState>) -> Result<Json<Value>> {
    let samples = get_all_audio_samples(&state.db).await?;

    let response = serde_json::json!({
        "samples": samples
    });

    Ok(Json(response))
}

pub async fn get_audio_metadata(
    State(state): State<AppState>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<Value>> {
    let audio_name = request
        .get("audio_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| crate::error::AppError::Validation("Missing audio_name".to_string()))?;

    // Resolve audio name
    let resolved_audio_name = resolve_audio_name(&state, audio_name).await?;

    let streaming_engine = StreamingEngineService::new(
        state.http_client.clone(),
        state.settings.streaming_engine.base_url.clone(),
    );

    let metadata = streaming_engine
        .get_audio_metadata(&resolved_audio_name)
        .await?;

    Ok(Json(metadata))
}

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

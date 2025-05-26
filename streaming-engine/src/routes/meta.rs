use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, instrument};
use utoipa::ToSchema;

use crate::{blob::AudioBuffer, state::AppStateDyn, streamingpath::params::Params};

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct AudioMetadata {
    pub format: String,
    pub duration: Option<f64>,
    pub bit_rate: Option<i64>,
    pub sample_rate: Option<i64>,
    pub channels: Option<i64>,
    pub codec: Option<String>,
    pub size: Option<i64>,
    pub tags: HashMap<String, String>,
}

#[instrument(skip(state))]
pub async fn meta_handler(
    State(state): State<AppStateDyn>,
    params: Params,
) -> Result<Json<AudioMetadata>, (StatusCode, String)> {
    info!("meta: {:?}", params);

    let blob = if params.key.starts_with("https://") || params.key.starts_with("http://") {
        let raw_bytes = reqwest::get(&params.key)
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch audio from URL {}: {}", params.key, e);
                (
                    StatusCode::NOT_FOUND,
                    format!("Failed to fetch audio: {}", e),
                )
            })?
            .bytes()
            .await
            .map_err(|e| {
                tracing::error!("Failed to read bytes from URL {}: {}", params.key, e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to fetch audio: {}", e),
                )
            })?
            .to_vec();

        AudioBuffer::from_bytes(raw_bytes)
    } else {
        state.storage.get(&params.key).await.map_err(|e| {
            tracing::error!("Failed to fetch audio from storage {}: {}", params.key, e);
            (
                StatusCode::NOT_FOUND,
                format!("Failed to fetch audio: {}", e),
            )
        })?
    };

    let processed_blob = state.processor.process(&blob, &params).await.map_err(|e| {
        tracing::error!("Failed to process audio with params {:?}: {}", params, e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to process audio: {}", e),
        )
    })?;

    let metadata = extract_metadata(&processed_blob).await.map_err(|e| {
        tracing::error!("Failed to extract metadata: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to extract metadata: {}", e),
        )
    })?;

    Ok(Json(metadata))
}

#[instrument(skip(audio))]
async fn extract_metadata(audio: &AudioBuffer) -> Result<AudioMetadata, color_eyre::eyre::Error> {
    use tempfile::TempDir;
    use tokio::process::Command;

    let temp_dir = TempDir::new()?;
    let input_path = temp_dir
        .path()
        .join(format!("input.{}", audio.format().extension()));

    // Write audio to temporary file
    tokio::fs::write(&input_path, audio.as_ref()).await?;

    // Run ffprobe to extract metadata
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "quiet",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
            input_path.to_str().unwrap(),
        ])
        .output()
        .await?;

    if !output.status.success() {
        return Err(color_eyre::eyre::eyre!(
            "ffprobe failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let probe_data: serde_json::Value = serde_json::from_slice(&output.stdout)?;

    let format_info = probe_data
        .get("format")
        .ok_or_else(|| color_eyre::eyre::eyre!("No format information found"))?;

    let empty_streams = vec![];
    let streams = probe_data
        .get("streams")
        .and_then(|s| s.as_array())
        .unwrap_or(&empty_streams);

    // Find the first audio stream
    let audio_stream = streams
        .iter()
        .find(|stream| stream.get("codec_type").and_then(|ct| ct.as_str()) == Some("audio"));

    let mut metadata = AudioMetadata {
        format: format_info
            .get("format_name")
            .and_then(|f| f.as_str())
            .unwrap_or("unknown")
            .to_string(),
        duration: format_info
            .get("duration")
            .and_then(|d| d.as_str())
            .and_then(|d| d.parse::<f64>().ok()),
        bit_rate: format_info
            .get("bit_rate")
            .and_then(|br| br.as_str())
            .and_then(|br| br.parse::<i64>().ok()),
        size: format_info
            .get("size")
            .and_then(|s| s.as_str())
            .and_then(|s| s.parse::<i64>().ok()),
        sample_rate: None,
        channels: None,
        codec: None,
        tags: HashMap::new(),
    };

    // Extract audio stream specific information
    if let Some(stream) = audio_stream {
        metadata.sample_rate = stream
            .get("sample_rate")
            .and_then(|sr| sr.as_str())
            .and_then(|sr| sr.parse::<i64>().ok());

        metadata.channels = stream.get("channels").and_then(|c| c.as_i64());

        metadata.codec = stream
            .get("codec_name")
            .and_then(|cn| cn.as_str())
            .map(|cn| cn.to_string());
    }

    // Extract tags from format
    if let Some(tags) = format_info.get("tags").and_then(|t| t.as_object()) {
        for (key, value) in tags {
            if let Some(value_str) = value.as_str() {
                metadata.tags.insert(key.clone(), value_str.to_string());
            }
        }
    }

    // Extract tags from audio stream
    if let Some(stream) = audio_stream {
        if let Some(tags) = stream.get("tags").and_then(|t| t.as_object()) {
            for (key, value) in tags {
                if let Some(value_str) = value.as_str() {
                    metadata.tags.insert(key.clone(), value_str.to_string());
                }
            }
        }
    }

    Ok(metadata)
}

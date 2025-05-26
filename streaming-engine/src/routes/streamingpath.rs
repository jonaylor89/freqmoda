use axum::{
    body::Body,
    extract::State,
    http::{Response, StatusCode, header},
    response::IntoResponse,
};
use tracing::{info, instrument, warn};

use crate::{
    blob::AudioBuffer,
    state::AppStateDyn,
    streamingpath::{hasher::suffix_result_storage_hasher, params::Params},
};

#[instrument(skip(state))]
pub async fn streamingpath_handler(
    State(state): State<AppStateDyn>,
    params: Params,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let params_hash = suffix_result_storage_hasher(&params);
    let result = state.storage.get(&params_hash).await.inspect_err(|_| {
        info!("no audio in results storage: {}", &params);
    });
    if let Ok(blob) = result {
        return Response::builder()
            .header(header::CONTENT_TYPE, blob.mime_type())
            .body(Body::from(blob.into_bytes()))
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to build response: {}", e),
                )
            });
    }

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

    state
        .storage
        .put(&params_hash, &processed_blob)
        .await
        .map_err(|e| {
            warn!("Failed to save result audio [{}]: {}", &params_hash, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to save result audio: {}", e),
            )
        })?;

    Response::builder()
        .header(header::CONTENT_TYPE, processed_blob.mime_type())
        .body(Body::from(processed_blob.into_bytes()))
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to build response: {}", e),
            )
        })
}

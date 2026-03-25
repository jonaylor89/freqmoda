use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;
use tracing::instrument;

use crate::state::AppStateDyn;

#[derive(Serialize)]
pub struct ListResponse {
    pub keys: Vec<String>,
}

#[instrument(skip(state))]
pub async fn list_handler(
    State(state): State<AppStateDyn>,
) -> Result<Json<ListResponse>, (StatusCode, String)> {
    let keys = state.storage.list().await.map_err(|e| {
        tracing::error!("Failed to list audio files: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to list audio files: {}", e),
        )
    })?;

    Ok(Json(ListResponse { keys }))
}

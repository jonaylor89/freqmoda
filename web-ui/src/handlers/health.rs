use axum::{extract::State, response::Json};
use serde_json::{Value, json};

use crate::{error::Result, services::streaming_engine::StreamingEngineService, state::AppState};

pub async fn health_check(State(state): State<AppState>) -> Result<Json<Value>> {
    let streaming_engine = StreamingEngineService::new(
        state.http_client.clone(),
        state.settings.streaming_engine.base_url.clone(),
    );

    let streaming_engine_healthy = streaming_engine.health_check().await.unwrap_or(false);

    let response = json!({
        "status": "healthy",
        "services": {
            "database": "healthy", // TODO: Add actual DB health check
            "streaming_engine": if streaming_engine_healthy { "healthy" } else { "unhealthy" }
        }
    });

    Ok(Json(response))
}

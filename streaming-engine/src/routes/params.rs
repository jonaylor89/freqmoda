use axum::{Json, http::StatusCode};
use tracing::info;

use crate::streamingpath::params::Params;

#[tracing::instrument]
pub async fn params(params: Params) -> Result<Json<Params>, (StatusCode, String)> {
    info!("params: {:?}", params);

    Ok(Json(params))
}

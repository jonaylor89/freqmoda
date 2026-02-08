use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("HTTP client error: {0}")]
    HttpClient(#[from] reqwest::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("OpenAI API error: {0}")]
    OpenAI(String),

    #[error("Streaming engine error: {0}")]
    StreamingEngine(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal server error: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Database(ref e) => {
                tracing::error!("Database error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error")
            }
            AppError::Migration(ref e) => {
                tracing::error!("Migration error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Migration error")
            }
            AppError::HttpClient(ref e) => {
                tracing::error!("HTTP client error: {}", e);
                (StatusCode::BAD_GATEWAY, "External service error")
            }
            AppError::Json(_) => (StatusCode::BAD_REQUEST, "Invalid JSON"),
            AppError::Validation(ref msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
            AppError::OpenAI(ref msg) => {
                tracing::error!("OpenAI API error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "AI service error")
            }
            AppError::StreamingEngine(ref msg) => {
                tracing::error!("Streaming engine error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Audio processing service error",
                )
            }
            AppError::NotFound(ref msg) => (StatusCode::NOT_FOUND, msg.as_str()),
            AppError::Internal(ref msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

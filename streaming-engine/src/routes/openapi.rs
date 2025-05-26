use axum::{Json, response::IntoResponse};
use serde_json::json;
use utoipa::OpenApi;

use crate::streamingpath::params::Params;

#[derive(OpenApi)]
#[openapi(
    paths(
        process_audio,
        preview_params,
        get_health
    ),
    components(schemas(Params)),
    tags(
        (name = "audio", description = "Audio processing endpoints")
    ),
    info(
        title = "Streaming Engine API",
        description = "Advanced Audio Streaming Server",
        version = "1.0.0",
        contact(
            name = "Johannes Naylor",
            email = "jonaylor89@gmail.com"
        )
    )
)]
pub struct ApiDoc;

/// Process audio with various effects and transformations
#[utoipa::path(
    get,
    path = "/unsafe/{audio_name}",
    params(
        ("audio_name" = String, Path, description = "URL/URI/filename or path to the audio file"),
        ("format" = Option<String>, Query, description = "Output format (mp3, wav, etc.)"),
        ("start_time" = Option<f64>, Query, description = "Start time in seconds"),
        ("duration" = Option<f64>, Query, description = "Duration in seconds"),
        ("speed" = Option<f64>, Query, description = "Playback speed multiplier"),
        ("reverse" = Option<bool>, Query, description = "Reverse the audio"),
        ("volume" = Option<f64>, Query, description = "Volume adjustment multiplier"),
        ("normalize" = Option<bool>, Query, description = "Normalize audio levels"),
        ("lowpass" = Option<f64>, Query, description = "Lowpass filter cutoff frequency"),
        ("highpass" = Option<f64>, Query, description = "Highpass filter cutoff frequency"),
        ("bass" = Option<f64>, Query, description = "Bass boost/cut level in dB"),
        ("treble" = Option<f64>, Query, description = "Treble boost/cut level in dB"),
        ("fade_in" = Option<f64>, Query, description = "Fade in duration in seconds"),
        ("fade_out" = Option<f64>, Query, description = "Fade out duration in seconds"),
        ("echo" = Option<String>, Query, description = "Echo effect parameters"),
        ("chorus" = Option<String>, Query, description = "Chorus effect parameters"),
        ("flanger" = Option<String>, Query, description = "Flanger effect parameters"),
    ),
    responses(
        (status = 200, description = "Processed audio file", content_type = "audio/*"),
        (status = 400, description = "Invalid parameters"),
        (status = 404, description = "Audio file not found"),
        (status = 500, description = "Processing error")
    ),
    tag = "audio"
)]
pub async fn process_audio() {
    // This is just for documentation - actual implementation is in streamingpath_handler
}

/// Analyze audio with various effects and transformations
#[utoipa::path(
    get,
    path = "/meta/unsafe/{audio_name}",
    params(
        ("audio_name" = String, Path, description = "URL/URI/filename or path to the audio file"),
        ("format" = Option<String>, Query, description = "Output format (mp3, wav, etc.)"),
        ("start_time" = Option<f64>, Query, description = "Start time in seconds"),
        ("duration" = Option<f64>, Query, description = "Duration in seconds"),
        ("speed" = Option<f64>, Query, description = "Playback speed multiplier"),
        ("reverse" = Option<bool>, Query, description = "Reverse the audio"),
        ("volume" = Option<f64>, Query, description = "Volume adjustment multiplier"),
        ("normalize" = Option<bool>, Query, description = "Normalize audio levels"),
        ("lowpass" = Option<f64>, Query, description = "Lowpass filter cutoff frequency"),
        ("highpass" = Option<f64>, Query, description = "Highpass filter cutoff frequency"),
        ("bass" = Option<f64>, Query, description = "Bass boost/cut level in dB"),
        ("treble" = Option<f64>, Query, description = "Treble boost/cut level in dB"),
        ("fade_in" = Option<f64>, Query, description = "Fade in duration in seconds"),
        ("fade_out" = Option<f64>, Query, description = "Fade out duration in seconds"),
        ("echo" = Option<String>, Query, description = "Echo effect parameters"),
        ("chorus" = Option<String>, Query, description = "Chorus effect parameters"),
        ("flanger" = Option<String>, Query, description = "Flanger effect parameters"),
    ),
    responses(
        (status = 200, description = "Processed audio file", content_type = "audio/*"),
        (status = 400, description = "Invalid parameters"),
        (status = 404, description = "Audio file not found"),
        (status = 500, description = "Processing error")
    ),
    tag = "audio"
)]
pub async fn analyze_audio() {
    // This is just for documentation - actual implementation is in streamingpath_handler
}

/// Preview audio processing parameters
#[utoipa::path(
    get,
    path = "/params/unsafe/{audio_name}",
    params(
        ("audio_name" = String, Path, description = "URL/URI/filename or path to the audio file"),
        ("format" = Option<String>, Query, description = "Output format (mp3, wav, etc.)"),
        ("start_time" = Option<f64>, Query, description = "Start time in seconds"),
        ("duration" = Option<f64>, Query, description = "Duration in seconds"),
        ("speed" = Option<f64>, Query, description = "Playback speed multiplier"),
        ("reverse" = Option<bool>, Query, description = "Reverse the audio"),
        ("volume" = Option<f64>, Query, description = "Volume adjustment multiplier"),
        ("normalize" = Option<bool>, Query, description = "Normalize audio levels"),
        ("lowpass" = Option<f64>, Query, description = "Lowpass filter cutoff frequency"),
        ("highpass" = Option<f64>, Query, description = "Highpass filter cutoff frequency"),
        ("bass" = Option<f64>, Query, description = "Bass boost/cut level in dB"),
        ("treble" = Option<f64>, Query, description = "Treble boost/cut level in dB"),
        ("fade_in" = Option<f64>, Query, description = "Fade in duration in seconds"),
        ("fade_out" = Option<f64>, Query, description = "Fade out duration in seconds"),
    ),
    responses(
        (status = 200, description = "Parameter preview", body = Params),
        (status = 400, description = "Invalid parameters")
    ),
    tag = "audio"
)]
pub async fn preview_params() {
    // This is just for documentation - actual implementation is in params handler
}

/// Check server health
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Server is healthy"),
    ),
    tag = "audio"
)]
pub async fn get_health() {
    // This is just for documentation - actual implementation is in health handler
}

pub async fn openapi_json() -> impl IntoResponse {
    Json(ApiDoc::openapi())
}

pub async fn get_openapi_schema() -> impl IntoResponse {
    let schema = json!({
        "openapi": "3.0.0",
        "info": {
            "title": "Streaming Engine API",
            "description": "Advanced Audio Streaming Server. Process audio files with various effects and transformations.",
            "version": "1.0.0",
            "contact": {
                "name": "Johannes Naylor",
                "email": "jonaylor89@gmail.com"
            }
        },
        "servers": [
            {
                "url": "http://localhost:8080",
                "description": "Local development server"
            }
        ],
        "paths": {
            "/unsafe/{audio_name}": {
                "get": {
                    "summary": "Process audio with effects",
                    "description": "Process an audio file with various effects and transformations",
                    "operationId": "processAudio",
                    "parameters": [
                        {
                            "name": "audio_name",
                            "in": "path",
                            "required": true,
                            "schema": { "type": "string" },
                            "description": "URL/URI/filename of the audio to process"
                        },
                        {
                            "name": "format",
                            "in": "query",
                            "schema": {
                                "type": "string",
                                "enum": ["mp3", "wav", "flac", "ogg", "m4a"]
                            },
                            "description": "Output audio format"
                        },
                        {
                            "name": "start_time",
                            "in": "query",
                            "schema": { "type": "number" },
                            "description": "Start time in seconds"
                        },
                        {
                            "name": "duration",
                            "in": "query",
                            "schema": { "type": "number" },
                            "description": "Duration in seconds"
                        },
                        {
                            "name": "speed",
                            "in": "query",
                            "schema": { "type": "number", "minimum": 0.1, "maximum": 10.0 },
                            "description": "Playback speed multiplier (0.5 = half speed, 2.0 = double speed)"
                        },
                        {
                            "name": "reverse",
                            "in": "query",
                            "schema": { "type": "boolean" },
                            "description": "Reverse the audio"
                        },
                        {
                            "name": "volume",
                            "in": "query",
                            "schema": { "type": "number", "minimum": 0.0, "maximum": 10.0 },
                            "description": "Volume adjustment multiplier"
                        },
                        {
                            "name": "normalize",
                            "in": "query",
                            "schema": { "type": "boolean" },
                            "description": "Normalize audio levels"
                        },
                        {
                            "name": "lowpass",
                            "in": "query",
                            "schema": { "type": "number" },
                            "description": "Lowpass filter cutoff frequency in Hz"
                        },
                        {
                            "name": "highpass",
                            "in": "query",
                            "schema": { "type": "number" },
                            "description": "Highpass filter cutoff frequency in Hz"
                        },
                        {
                            "name": "bass",
                            "in": "query",
                            "schema": { "type": "number" },
                            "description": "Bass boost/cut level in dB"
                        },
                        {
                            "name": "treble",
                            "in": "query",
                            "schema": { "type": "number" },
                            "description": "Treble boost/cut level in dB"
                        },
                        {
                            "name": "fade_in",
                            "in": "query",
                            "schema": { "type": "number" },
                            "description": "Fade in duration in seconds"
                        },
                        {
                            "name": "fade_out",
                            "in": "query",
                            "schema": { "type": "number" },
                            "description": "Fade out duration in seconds"
                        },
                        {
                            "name": "echo",
                            "in": "query",
                            "schema": {
                                "type": "string",
                                "enum": ["light", "medium", "heavy"]
                            },
                            "description": "Echo effect level - use 'light', 'medium', or 'heavy'"
                        },
                        {
                            "name": "chorus",
                            "in": "query",
                            "schema": {
                                "type": "string",
                                "enum": ["light", "medium", "heavy"]
                            },
                            "description": "Chorus effect level - use 'light', 'medium', or 'heavy'"
                        },
                        {
                            "name": "flanger",
                            "in": "query",
                            "schema": {
                                "type": "string",
                                "enum": ["light", "medium", "heavy"]
                            },
                            "description": "Flanger effect level - use 'light', 'medium', or 'heavy'"
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "Processed audio file",
                            "content": {
                                "audio/*": {
                                    "schema": {
                                        "type": "string",
                                        "format": "binary"
                                    }
                                }
                            }
                        },
                        "400": {
                            "description": "Invalid parameters"
                        },
                        "404": {
                            "description": "Audio file not found"
                        },
                        "500": {
                            "description": "Processing error"
                        }
                    }
                }
            },
            "/params/unsafe/{audio_name}": {
                "get": {
                    "summary": "Preview processing parameters",
                    "description": "Preview the parameters that would be used for processing without actually processing the audio",
                    "operationId": "previewParams",
                    "parameters": [
                        {
                            "name": "audio_name",
                            "in": "path",
                            "required": true,
                            "schema": { "type": "string" },
                            "description": "URL/URI/filename of the audio to process"
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "Parameter preview",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "description": "Audio processing parameters"
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/health": {
                "get": {
                    "summary": "Health check",
                    "description": "Check if the server is running and healthy",
                    "operationId": "healthCheck",
                    "responses": {
                        "200": {
                            "description": "Server is healthy"
                        }
                    }
                }
            }
        }
    });

    Json(schema)
}

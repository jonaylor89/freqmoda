use std::net::SocketAddr;

use axum::{
    Router,
    extract::ConnectInfo,
    middleware,
    routing::{get, post},
};
use tower_http::services::ServeDir;

use crate::{
    handlers::{
        audio::{get_audio_metadata, list_audio_samples, process_audio},
        chat::chat,
        health::health_check,
        html::{
            chat_form_handler, chat_page, download_audio, index_page, sample_chat_form_handler,
            sample_chat_page,
        },
    },
    middleware::{rate_limit::rate_limit_middleware, session::session_middleware},
    state::AppState,
};

pub fn create_router(state: AppState) -> Router {
    // Routes that need rate limiting (chat and API endpoints)
    let rate_limited_routes = Router::new()
        .route("/sample/{sample_id}/chat", post(sample_chat_form_handler))
        .route("/chat", post(chat_form_handler))
        .route("/api/chat", post(chat))
        .route("/api/audio/process", post(process_audio))
        .route("/api/audio/metadata", post(get_audio_metadata))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            rate_limit_middleware,
        ));

    // Routes that don't need rate limiting (pages, static content)
    let unrestricted_routes = Router::new()
        .route("/health", get(health_check))
        .route("/ping", get(pong))
        // HTML interface
        .route("/", get(index_page))
        .route("/sample/{sample_id}", get(sample_chat_page))
        .route("/chat", get(chat_page))
        .route("/download/{filename}", get(download_audio))
        .route("/api/audio/samples", get(list_audio_samples))
        // Static file serving for SEO assets, favicons, etc.
        .nest_service("/static", ServeDir::new("static"))
        // Serve favicon.ico at root level
        .route(
            "/favicon.ico",
            get(|| async { axum::response::Redirect::permanent("/static/favicon.ico") }),
        )
        .route(
            "/favicon-16x16.png",
            get(|| async { axum::response::Redirect::permanent("/static/favicon-16x16.png") }),
        )
        .route(
            "/favicon-32x32.png",
            get(|| async { axum::response::Redirect::permanent("/static/favicon-32x32.png") }),
        )
        .route(
            "/apple-touch-icon.png",
            get(|| async { axum::response::Redirect::permanent("/static/apple-touch-icon.png") }),
        )
        .route(
            "/site.webmanifest",
            get(|| async { axum::response::Redirect::permanent("/static/site.webmanifest") }),
        )
        .route(
            "/og-image.png",
            get(|| async { axum::response::Redirect::permanent("/static/og-image.png") }),
        )
        .route(
            "/twitter-card.png",
            get(|| async { axum::response::Redirect::permanent("/static/twitter-card.png") }),
        );

    Router::new()
        .merge(rate_limited_routes)
        .merge(unrestricted_routes)
        .layer(middleware::from_fn(session_middleware))
        .with_state(state)
}

async fn pong(ConnectInfo(addr): ConnectInfo<SocketAddr>) -> String {
    format!("pong {addr}")
}

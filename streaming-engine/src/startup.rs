use crate::cache::{AudioCache, Cache};
use crate::config::{Settings, StorageClient};
use crate::metrics::{setup_metrics_recorder, track_metrics};
use crate::middleware::auth_middleware;
use crate::middleware::cache_middleware;
use crate::processor::{AudioProcessor, Processor};
use crate::routes::health::health_check;
use crate::routes::meta::meta_handler;
use crate::routes::params::params;
use crate::routes::root::root_handler;
use crate::routes::streamingpath::streamingpath_handler;
use crate::state::AppStateDyn;
#[cfg(feature = "filesystem")]
use crate::storage::file::FileStorage;
#[cfg(feature = "gcs")]
use crate::storage::gcs::GCloudStorage;
#[cfg(feature = "s3")]
use crate::storage::s3::S3Storage;
use crate::storage::AudioStorage;
use crate::tags::create_tags;
use axum::extract::{MatchedPath, Request};
use axum::middleware;
use axum::routing::get;
use axum::{Router, serve::Serve};
use color_eyre::Result;
use color_eyre::eyre::WrapErr;
#[cfg(feature = "s3")]
use secrecy::ExposeSecret;
use std::future::ready;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{debug, info, info_span};

pub struct Application {
    pub port: u16,
    server: Serve<TcpListener, Router, Router>,
}

impl Application {
    pub async fn build(config: Settings) -> Result<Self> {
        let address = format!("{}:{}", config.application.host, config.port);
        println!("Server started at {}\n", &address);
        let listener = TcpListener::bind(address).await.wrap_err(
            "Failed to bind to the port. Make sure you have the correct permissions to bind to the port",
        )?;
        let port = listener.local_addr()?.port();

        let additional_tags = create_tags(config.custom_tags)?;

        let processor = Processor::new(config.processor, additional_tags);
        let cache = Cache::new(config.cache)?;

        let server = match config.storage.client {
            #[cfg(feature = "s3")]
            Some(StorageClient::S3(s3_settings)) => {
                info!("Using S3 storage");
                let storage = S3Storage::new(
                    config.storage.base_dir,
                    config.storage.path_prefix,
                    config.storage.safe_chars,
                    s3_settings.endpoint,
                    s3_settings.region,
                    s3_settings.bucket,
                    s3_settings.access_key.expose_secret(),
                    s3_settings.secret_key.expose_secret(),
                )
                .await?;

                // Ensure bucket exists
                storage.ensure_bucket_exists().await?;

                run(listener, storage, processor, cache).await?
            }
            #[cfg(feature = "gcs")]
            Some(StorageClient::GCS(gcs_settings)) => {
                info!("using GCS storage");
                let storage = GCloudStorage::new(
                    config.storage.base_dir,
                    config.storage.path_prefix,
                    config.storage.safe_chars,
                    gcs_settings.bucket,
                )
                .await;

                run(listener, storage, processor, cache).await?
            }
            #[cfg(feature = "filesystem")]
            None => {
                info!("using filesystem storage");
                let storage = FileStorage::new(
                    PathBuf::from(config.storage.base_dir),
                    config.storage.path_prefix,
                    config.storage.safe_chars,
                );

                run(listener, storage, processor, cache).await?
            }
            #[cfg(not(any(feature = "s3", feature = "gcs", feature = "filesystem")))]
            _ => {
                return Err(color_eyre::eyre::eyre!(
                    "No storage backend feature enabled. Enable one of: filesystem, gcs, s3"
                ));
            }
            #[cfg(not(feature = "s3"))]
            Some(StorageClient::S3(_)) => {
                return Err(color_eyre::eyre::eyre!(
                    "S3 storage requested but s3 feature not enabled"
                ));
            }
            #[cfg(not(feature = "gcs"))]
            Some(StorageClient::GCS(_)) => {
                return Err(color_eyre::eyre::eyre!(
                    "GCS storage requested but gcs feature not enabled"
                ));
            }
            #[cfg(not(feature = "filesystem"))]
            None => {
                return Err(color_eyre::eyre::eyre!(
                    "Filesystem storage requested but filesystem feature not enabled"
                ));
            }
        };

        Ok(Self { port, server })
    }
    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        println!(
            r#"\n
  ____  _                             _             _____             _
 / ___|| |_ _ __ ___  __ _ _ __ ___ (_)_ __   __ _| ____|_ __   __ _(_)_ __   ___
 \___ \| __| '__/ _ \/ _` | '_ ` _ \| | '_ \ / _` |  _| | '_ \ / _` | | '_ \ / _ \
  ___) | |_| | |  __/ (_| | | | | | | | | | | (_| | |___| | | | (_| | | | | |  __/
 |____/ \__|_|  \___|\__,_|_| |_| |_|_|_| |_|\__, |_____|_| |_|\__, |_|_| |_|\___|
                                             |___/              |___/
        "#
        );
        self.server.await
    }
}

async fn run<S, P, C>(
    listener: TcpListener,
    storage: S,
    processor: P,
    cache: C,
) -> Result<Serve<TcpListener, Router, Router>>
where
    S: AudioStorage + Clone + Send + Sync + 'static,
    P: AudioProcessor + Send + Sync + 'static,
    C: AudioCache + Clone + Send + Sync + 'static,
{
    let recorder_handle = setup_metrics_recorder();

    let state = AppStateDyn {
        storage: Arc::new(storage.clone()),
        processor: Arc::new(processor),
        cache: Arc::new(cache.clone()),
    };

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/metrics", get(move || ready(recorder_handle.render())))
        .route("/openapi.json", get(crate::routes::openapi::openapi_json))
        .route(
            "/api-schema",
            get(crate::routes::openapi::get_openapi_schema),
        )
        .route("/", get(root_handler))
        .route("/params/{*streamingpath}", get(params))
        .route_layer(middleware::from_fn(track_metrics))
        .merge(
            Router::new()
                .route("/meta/{*streamingpath}", get(meta_handler))
                .route("/{*streamingpath}", get(streamingpath_handler))
                .route_layer(middleware::from_fn_with_state(
                    state.clone(),
                    auth_middleware,
                ))
                .route_layer(middleware::from_fn_with_state(
                    state.clone(),
                    cache_middleware,
                )),
        )
        // Allow all origins for CORS - this is an open streaming server with custom auth/rate limiting
        .layer(CorsLayer::permissive())
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                // Log the matched route's path (with placeholders not filled in).
                // Use request.uri() or OriginalUri if you want the real path.
                let matched_path = request
                    .extensions()
                    .get::<MatchedPath>()
                    .map(MatchedPath::as_str);

                info_span!(
                    "http_request",
                    method = ?request.method(),
                    matched_path,
                    some_other_field = tracing::field::Empty,
                )
            }),
        )
        .with_state(state);

    debug!("listening on {}", listener.local_addr().unwrap());
    let server = axum::serve(listener, app);

    Ok(server)
}

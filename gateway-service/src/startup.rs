use std::net::SocketAddr;

use crate::routes::create_router;
use crate::state::AppState;
use crate::{config::Settings, database::get_connection_pool};
use axum::extract::ConnectInfo;
use axum::extract::connect_info::IntoMakeServiceWithConnectInfo;
use axum::middleware::AddExtension;
use axum::{Router, serve::Serve};
use color_eyre::Result;
use color_eyre::eyre::WrapErr;
use secrecy::ExposeSecret;
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{debug, info};

pub struct Application {
    pub port: u16,
    server: Serve<
        IntoMakeServiceWithConnectInfo<Router, SocketAddr>,
        AddExtension<Router, ConnectInfo<SocketAddr>>,
    >,
}

impl Application {
    pub async fn build(config: Settings) -> Result<Self> {
        let address = format!("{}:{}", config.server.host, config.port);
        println!("Gateway service starting at {}\n", &address);

        let listener = TcpListener::bind(&address).await.wrap_err(
            "Failed to bind to the port. Make sure you have the correct permissions to bind to the port",
        )?;
        let port = listener.local_addr()?.port();

        let connection_pool = get_connection_pool(&config.database);
        let redis_client = redis::Client::open(config.redis.url.expose_secret())?;

        // Create application state
        let state = AppState::new(config, connection_pool, redis_client)?;

        // Create router with middleware
        let app = create_router(state)
            .layer(CorsLayer::permissive())
            .layer(TraceLayer::new_for_http());

        debug!("listening on {}", listener.local_addr().unwrap());
        let server = axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        );

        Ok(Self { port, server })
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        println!(
            r#"

  ▄████  ▄▄▄     ▄▄▄█████▓▓█████  █     █░ ▄▄▄     ▓██   ██▓
 ██▒ ▀█▒▒████▄   ▓  ██▒ ▓▒▓█   ▀ ▓█░ █ ░█░▒████▄    ▒██  ██▒
▒██░▄▄▄░▒██  ▀█▄ ▒ ▓██░ ▒░▒███   ▒█░ █ ░█ ▒██  ▀█▄   ▒██ ██░
░▓█  ██▓░██▄▄▄▄██░ ▓██▓ ░ ▒▓█  ▄ ░█░ █ ░█ ░██▄▄▄▄██  ░ ▐██▓░
░▒▓███▀▒ ▓█   ▓██▒ ▒██▒ ░ ░▒████▒░░██▒██▓  ▓█   ▓██▒ ░ ██▒▓░
 ░▒   ▒  ▒▒   ▓▒█░ ▒ ░░   ░░ ▒░ ░░ ▓░▒ ▒   ▒▒   ▓▒█░  ██▒▒▒
  ░   ░   ▒   ▒▒ ░   ░     ░ ░  ░  ▒ ░ ░    ▒   ▒▒ ░▓██ ░▒░
░ ░   ░   ░   ▒    ░         ░     ░   ░    ░   ▒   ▒ ▒ ░░
      ░       ░  ░           ░  ░    ░          ░  ░░ ░
                                                    ░ ░
        "#
        );
        info!("Gateway service is running");
        self.server.await
    }
}

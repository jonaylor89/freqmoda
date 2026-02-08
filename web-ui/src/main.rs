use color_eyre::Result;
use web_ui::config::get_configuration;
use web_ui::startup::Application;
use web_ui::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let parse_dotenv = dotenvy::dotenv();
    if let Err(e) = parse_dotenv {
        tracing::warn!("failed to parse .env file: {}", e);
    }

    let configuration = get_configuration()
        .inspect_err(|e| tracing::error!("Failed to load configuration: {}", e))
        .expect("Failed to read configuration");

    let subscriber = get_subscriber("web-ui".into(), "debug".into(), std::io::stdout);
    init_subscriber(subscriber);

    let app = Application::build(configuration).await?;
    let outcome = app.run_until_stopped().await;

    match outcome {
        Ok(()) => {
            tracing::info!("server has exited")
        }
        Err(e) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "server failed",
            )
        }
    }

    Ok(())
}

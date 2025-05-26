#[tracing::instrument]
pub async fn health_check() -> &'static str {
    tracing::info!("Health check called");
    "OK"
}

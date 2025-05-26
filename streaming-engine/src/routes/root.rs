#[tracing::instrument]
pub async fn root_handler() -> &'static str {
    "Hello, World"
}

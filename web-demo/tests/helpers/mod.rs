pub mod config;
pub mod database;
pub mod mocks;
pub mod redis;
pub mod test_app;

use once_cell::sync::Lazy;

static TRACING: Lazy<()> = Lazy::new(|| {
    // Simple tracing setup for tests
    if std::env::var("TEST_LOG").is_ok() {
        tracing_subscriber::fmt()
            .with_env_filter("debug")
            .try_init()
            .ok();
    }
});

pub fn ensure_tracing() {
    Lazy::force(&TRACING);
}

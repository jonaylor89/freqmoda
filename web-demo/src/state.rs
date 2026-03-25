use crate::config::Settings;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub settings: Settings,
    pub db: PgPool,
    pub http_client: reqwest::Client,
    pub redis: redis::Client,
}

impl AppState {
    pub fn new(
        settings: Settings,
        db: PgPool,
        redis: redis::Client,
    ) -> Result<Self, redis::RedisError> {
        let http_client = reqwest::Client::new();

        Ok(Self {
            settings,
            db,
            http_client,
            redis,
        })
    }
}

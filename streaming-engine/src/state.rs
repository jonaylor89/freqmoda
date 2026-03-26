use crate::{cache::AudioCache, processor::AudioProcessor, storage::AudioStorage};
use std::sync::Arc;

#[derive(Clone)]
pub struct WebConfig {
    pub port: u16,
    pub host: String,
    pub storage_backend: String,
    pub storage_base_dir: String,
    pub storage_path_prefix: String,
    pub cache_backend: String,
    pub max_filter_ops: usize,
    pub concurrency: Option<usize>,
    pub environment: String,
}

#[derive(Clone)]
pub struct AppStateDyn {
    pub storage: Arc<dyn AudioStorage>,
    pub processor: Arc<dyn AudioProcessor>,
    pub cache: Arc<dyn AudioCache>,
    pub web_config: Option<WebConfig>,
}

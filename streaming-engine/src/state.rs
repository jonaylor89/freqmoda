use crate::{
    cache::cache::AudioCache, processor::processor::AudioProcessor, storage::storage::AudioStorage,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppStateDyn {
    pub storage: Arc<dyn AudioStorage>,
    pub processor: Arc<dyn AudioProcessor>,
    pub cache: Arc<dyn AudioCache>,
}

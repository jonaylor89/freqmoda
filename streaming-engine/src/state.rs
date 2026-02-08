use crate::{
    cache::AudioCache, processor::AudioProcessor, storage::AudioStorage,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppStateDyn {
    pub storage: Arc<dyn AudioStorage>,
    pub processor: Arc<dyn AudioProcessor>,
    pub cache: Arc<dyn AudioCache>,
}

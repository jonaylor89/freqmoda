use async_trait::async_trait;
use color_eyre::Result;

use crate::blob::AudioBuffer;

#[async_trait]
pub trait AudioStorage: Send + Sync {
    async fn get(&self, key: &str) -> Result<AudioBuffer>;
    async fn put(&self, key: &str, blob: &AudioBuffer) -> Result<()>;
    async fn delete(&self, key: &str) -> Result<()>;
}

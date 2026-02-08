use async_trait::async_trait;
use color_eyre::Result;
use std::time::Duration;

use crate::config::{CacheSettings, FilesystemCacheSettings};

use super::{fs::FileSystemCache, redis::RedisCache};

#[derive(Debug, Clone)]
pub enum Cache {
    Redis(RedisCache),
    Filesystem(FileSystemCache),
}

impl Cache {
    pub fn new(config: CacheSettings) -> Result<Self> {
        match config {
            CacheSettings::Redis { uri } => Ok(Cache::Redis(RedisCache::new(&uri)?)),
            CacheSettings::Filesystem(FilesystemCacheSettings { base_dir }) => {
                Ok(Cache::Filesystem(FileSystemCache::new(base_dir)?))
            }
        }
    }
}

#[async_trait]
pub trait AudioCache: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    async fn set(&self, key: &str, value: &[u8], ttl: Option<Duration>) -> Result<()>;
    async fn delete(&self, key: &str) -> Result<()>;
}

#[async_trait]
impl AudioCache for Cache {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        match self {
            Cache::Redis(cache) => cache.get(key).await,
            Cache::Filesystem(cache) => cache.get(key).await,
        }
    }

    async fn set(&self, key: &str, value: &[u8], ttl: Option<Duration>) -> Result<()> {
        match self {
            Cache::Redis(cache) => cache.set(key, value, ttl).await,
            Cache::Filesystem(cache) => cache.set(key, value, ttl).await,
        }
    }

    async fn delete(&self, key: &str) -> Result<()> {
        match self {
            Cache::Redis(cache) => cache.delete(key).await,
            Cache::Filesystem(cache) => cache.delete(key).await,
        }
    }
}

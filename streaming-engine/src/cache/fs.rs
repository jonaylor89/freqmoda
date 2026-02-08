use super::AudioCache;
use async_trait::async_trait;
use color_eyre::Result;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use tokio::fs as tokio_fs;

#[derive(Debug, Clone)]
pub struct FileSystemCache {
    base_path: PathBuf,
}

impl FileSystemCache {
    pub fn new<P: AsRef<Path>>(base_path: P) -> Result<Self> {
        let base_path = base_path.as_ref().to_path_buf();
        // Create directory if it doesn't exist
        fs::create_dir_all(&base_path)?;
        Ok(FileSystemCache { base_path })
    }

    fn get_file_path(&self, key: &str) -> PathBuf {
        self.base_path.join(key)
    }

    fn get_meta_path(&self, key: &str) -> PathBuf {
        self.base_path.join(format!("{}.meta", key))
    }

    async fn is_expired(&self, key: &str) -> Result<bool> {
        let meta_path = self.get_meta_path(key);

        if !meta_path.exists() {
            return Ok(false);
        }

        let content = tokio_fs::read_to_string(&meta_path).await?;
        if let Ok(expiry) = content.parse::<u64>() {
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_secs();
            Ok(now > expiry)
        } else {
            Ok(false)
        }
    }
}

#[async_trait]
impl AudioCache for FileSystemCache {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let file_path = self.get_file_path(key);

        // Check if file exists and isn't expired
        if !file_path.exists() || self.is_expired(key).await? {
            return Ok(None);
        }

        // Read file contents
        let contents = tokio_fs::read(&file_path).await?;
        Ok(Some(contents))
    }

    async fn set(&self, key: &str, value: &[u8], ttl: Option<Duration>) -> Result<()> {
        let file_path = self.get_file_path(key);

        // Write the actual data
        tokio_fs::write(&file_path, value).await?;

        // If TTL is specified, write the expiration time
        if let Some(duration) = ttl {
            let meta_path = self.get_meta_path(key);
            let expiry = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_secs()
                + duration.as_secs();

            tokio_fs::write(&meta_path, expiry.to_string()).await?;
        }

        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let file_path = self.get_file_path(key);
        let meta_path = self.get_meta_path(key);

        // Delete both the data file and meta file if they exist
        if file_path.exists() {
            tokio_fs::remove_file(&file_path).await?;
        }
        if meta_path.exists() {
            tokio_fs::remove_file(&meta_path).await?;
        }

        Ok(())
    }
}

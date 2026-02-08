use crate::blob::AudioBuffer;
use crate::storage::AudioStorage;
use crate::streamingpath::normalize::{SafeCharsType, normalize};
use async_trait::async_trait;
use color_eyre::Result;
use std::fs;
use std::path::{Path, PathBuf};
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::debug;

#[derive(Debug, Clone)]
pub struct FileStorage {
    pub base_dir: PathBuf,
    pub path_prefix: String,
    pub safe_chars: SafeCharsType,
}

#[async_trait]
impl AudioStorage for FileStorage {
    #[tracing::instrument(skip(self))]
    async fn get(&self, key: &str) -> Result<AudioBuffer> {
        let full_path = self.get_full_path(key);
        if let Some(p) = full_path.to_str() {
            debug!("full path {}", p);
        }

        let mut file = File::open(full_path).await?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;
        Ok(AudioBuffer::from_bytes(buffer))
    }

    #[tracing::instrument(skip(self, blob))]
    async fn put(&self, key: &str, blob: &AudioBuffer) -> Result<()> {
        let full_path = self.get_full_path(key);
        if let Some(p) = full_path.to_str() {
            debug!("full path {}", p);
        }

        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(full_path)
            .await?;
        file.write_all(blob.as_ref()).await?;
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn delete(&self, key: &str) -> Result<()> {
        let full_path = self.get_full_path(key);
        if let Some(p) = full_path.to_str() {
            debug!("full path {}", p);
        }

        tokio::fs::remove_file(full_path).await?;
        Ok(())
    }
}

impl FileStorage {
    pub fn new(base_dir: PathBuf, path_prefix: String, safe_chars: SafeCharsType) -> Self {
        FileStorage {
            base_dir,
            path_prefix,
            safe_chars,
        }
    }

    pub fn get_full_path(&self, key: &str) -> PathBuf {
        let safe_key = normalize(key, &self.safe_chars);
        self.base_dir
            .join(Path::new(&self.path_prefix))
            .join(safe_key)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::blob::AudioFormat;
    use tempfile::tempdir;
    use tokio::fs as tokio_fs;

    // Helper function to create a test audio buffer
    fn create_test_audio_buffer(data: &[u8]) -> AudioBuffer {
        // Use the public constructor method instead of directly accessing private fields
        // This avoids the "field is private" errors
        AudioBuffer::from_bytes_with_format(data.to_vec(), AudioFormat::Mp3)
    }

    #[tokio::test]
    async fn test_new_storage() {
        let base_dir = PathBuf::from("/tmp");
        let path_prefix = "audio_files".to_string();
        let safe_chars = SafeCharsType::Default;

        let storage = FileStorage::new(base_dir.clone(), path_prefix.clone(), safe_chars);

        assert_eq!(storage.base_dir, base_dir);
        assert_eq!(storage.path_prefix, path_prefix);
        assert_eq!(storage.safe_chars, SafeCharsType::Default);
    }

    #[tokio::test]
    async fn test_get_full_path() {
        let base_dir = PathBuf::from("/tmp");
        let path_prefix = "audio_files".to_string();
        let storage = FileStorage::new(
            base_dir.clone(),
            path_prefix.clone(),
            SafeCharsType::Default,
        );

        let key = "test/my-audio.mp3";
        let full_path = storage.get_full_path(key);

        assert_eq!(
            full_path,
            PathBuf::from("/tmp/audio_files/test/my-audio.mp3")
        );
    }

    #[tokio::test]
    async fn test_get_full_path_with_special_chars() {
        let base_dir = PathBuf::from("/tmp");
        let path_prefix = "audio_files".to_string();
        let storage = FileStorage::new(
            base_dir.clone(),
            path_prefix.clone(),
            SafeCharsType::Default,
        );

        let key = "test/my audio!@#.mp3";
        let full_path = storage.get_full_path(key);

        // Special characters should be normalized
        assert_ne!(
            full_path,
            PathBuf::from("/tmp/audio_files/test/my audio!@#.mp3")
        );
    }

    #[tokio::test]
    async fn test_put_and_get() -> Result<()> {
        let temp_dir = tempdir()?;
        let base_dir = temp_dir.path().to_path_buf();
        let path_prefix = "audio_files".to_string();
        let storage = FileStorage::new(base_dir, path_prefix, SafeCharsType::Default);

        // Create test data
        let test_data = b"This is test audio data";
        let audio_buffer = create_test_audio_buffer(test_data);
        let key = "test/my-audio.mp3";

        // Put the audio
        storage.put(key, &audio_buffer).await?;

        // Get the audio
        let retrieved_buffer = storage.get(key).await?;

        // Verify the content
        assert_eq!(retrieved_buffer.as_ref(), test_data);

        Ok(())
    }

    #[tokio::test]
    async fn test_put_and_delete() -> Result<()> {
        let temp_dir = tempdir()?;
        let base_dir = temp_dir.path().to_path_buf();
        let path_prefix = "audio_files".to_string();
        let storage = FileStorage::new(base_dir, path_prefix, SafeCharsType::Default);

        // Create test data
        let test_data = b"This is test audio data";
        let audio_buffer = create_test_audio_buffer(test_data);
        let key = "test/my-audio.mp3";

        // Put the audio
        storage.put(key, &audio_buffer).await?;

        // Verify file exists
        let full_path = storage.get_full_path(key);
        assert!(tokio_fs::try_exists(&full_path).await?);

        // Delete the file
        storage.delete(key).await?;

        // Verify file doesn't exist anymore
        assert!(!tokio_fs::try_exists(&full_path).await?);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_nonexistent_file() {
        let temp_dir = tempdir().unwrap();
        let base_dir = temp_dir.path().to_path_buf();
        let path_prefix = "audio_files".to_string();
        let storage = FileStorage::new(base_dir, path_prefix, SafeCharsType::Default);

        let key = "nonexistent/audio.mp3";
        let result = storage.get(key).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_nonexistent_file() {
        let temp_dir = tempdir().unwrap();
        let base_dir = temp_dir.path().to_path_buf();
        let path_prefix = "audio_files".to_string();
        let storage = FileStorage::new(base_dir, path_prefix, SafeCharsType::Default);

        let key = "nonexistent/audio.mp3";
        let result = storage.delete(key).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_put_with_nested_directories() -> Result<()> {
        let temp_dir = tempdir()?;
        let base_dir = temp_dir.path().to_path_buf();
        let path_prefix = "audio_files".to_string();
        let storage = FileStorage::new(base_dir.clone(), path_prefix, SafeCharsType::Default);

        // Create test data with deeply nested path
        let test_data = b"This is test audio data";
        let audio_buffer = create_test_audio_buffer(test_data);
        let key = "deep/nested/path/structure/audio.mp3";

        // Put the audio
        storage.put(key, &audio_buffer).await?;

        // Verify full directory structure was created
        let expected_dir = base_dir.join("audio_files/deep/nested/path/structure");
        assert!(tokio_fs::try_exists(&expected_dir).await?);

        // Verify file exists
        let full_path = storage.get_full_path(key);
        assert!(tokio_fs::try_exists(&full_path).await?);

        Ok(())
    }

    #[tokio::test]
    async fn test_different_safe_chars_types() -> Result<()> {
        let temp_dir = tempdir()?;
        let base_dir = temp_dir.path().to_path_buf();
        let path_prefix = "audio_files".to_string();

        // Create storage with different SafeCharsType
        let strict_storage = FileStorage::new(
            base_dir.clone(),
            path_prefix.clone(),
            SafeCharsType::Default,
        );

        let relaxed_storage =
            FileStorage::new(base_dir.clone(), path_prefix.clone(), SafeCharsType::Noop);

        let key_with_spaces = "test/my audio file.mp3";

        // Get paths with different normalization
        let strict_path = strict_storage.get_full_path(key_with_spaces);
        let relaxed_path = relaxed_storage.get_full_path(key_with_spaces);

        // Paths should be different due to different normalization
        assert_ne!(strict_path, relaxed_path);

        Ok(())
    }
}

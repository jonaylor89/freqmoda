use crate::blob::AudioBuffer;
use crate::storage::storage::AudioStorage;
use crate::streamingpath::normalize::{SafeCharsType, normalize};
use async_trait::async_trait;
use color_eyre::Result;
use google_cloud_storage::client::{Client, ClientConfig};
use google_cloud_storage::http::objects::delete::DeleteObjectRequest;
use google_cloud_storage::http::objects::download::Range;
use google_cloud_storage::http::objects::get::GetObjectRequest;
use google_cloud_storage::http::objects::upload::{Media, UploadObjectRequest, UploadType};
use tracing::debug;

#[derive(Clone)]
pub struct GCloudStorage {
    pub base_dir: String,
    pub path_prefix: String,
    pub safe_chars: SafeCharsType,
    pub client: Client,
    pub bucket: String,
    // pub expiration: time::Duration,
    // pub acl: String,
}

#[async_trait]
impl AudioStorage for GCloudStorage {
    #[tracing::instrument(skip(self))]
    async fn get(&self, key: &str) -> Result<AudioBuffer> {
        let full_path = self.get_full_path(key);
        debug!("GCS get: {}", full_path);
        let buffer = self
            .client
            .download_object(
                &GetObjectRequest {
                    bucket: self.bucket.clone(),
                    object: full_path,
                    ..Default::default()
                },
                &Range::default(),
            )
            .await?;

        Ok(AudioBuffer::from_bytes(buffer))
    }

    #[tracing::instrument(skip(self, blob))]
    async fn put(&self, key: &str, blob: &AudioBuffer) -> Result<()> {
        let full_path = self.get_full_path(key);
        debug!("GCS set: {}", full_path);
        let upload_type = UploadType::Simple(Media::new(full_path));
        let blob_data = blob.as_ref().to_vec();
        self.client
            .upload_object(
                &UploadObjectRequest {
                    bucket: self.bucket.clone(),
                    ..Default::default()
                },
                blob_data,
                &upload_type,
            )
            .await?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn delete(&self, key: &str) -> Result<()> {
        let full_path = self.get_full_path(key);
        debug!("GCS delete: {}", full_path);
        self.client
            .delete_object(&DeleteObjectRequest {
                bucket: self.bucket.clone(),
                object: full_path,
                ..Default::default()
            })
            .await?;
        Ok(())
    }
}

impl GCloudStorage {
    #[tracing::instrument]
    pub async fn new(
        base_dir: String,
        path_prefix: String,
        safe_chars: SafeCharsType,
        bucket: String,
        // expiration: time::Duration,
        // acl: String,
    ) -> Self {
        let config = ClientConfig::default().with_auth().await.unwrap();
        let client = Client::new(config);
        GCloudStorage {
            base_dir,
            path_prefix,
            safe_chars,
            client,
            bucket,
            // acl,
            // expiration,
        }
    }

    pub fn get_full_path(&self, key: &str) -> String {
        let safe_key = normalize(key, &self.safe_chars);
        if self.path_prefix.ends_with('/') {
            format!("{}{}", self.path_prefix, safe_key)
        } else {
            format!("{}/{}", self.path_prefix, safe_key)
        }
    }
}

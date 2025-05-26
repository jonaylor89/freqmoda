use std::time::Duration;

use crate::blob::AudioBuffer;
use crate::storage::storage::AudioStorage;
use crate::streamingpath::normalize::{SafeCharsType, normalize};
use aws_sdk_s3::Client;
use aws_sdk_s3::config::{Credentials, Region};
use aws_sdk_s3::primitives::ByteStream;
use async_trait::async_trait;
use color_eyre::Result;
use tracing::{debug, info, warn};

#[derive(Clone)]
pub struct S3Storage {
    pub base_dir: String,
    pub path_prefix: String,
    pub safe_chars: SafeCharsType,
    pub client: Client,
    pub bucket: String,
    // pub expiration: time::Duration,
    // pub acl: String,
}

#[async_trait]
impl AudioStorage for S3Storage {
    #[tracing::instrument(skip(self))]
    async fn get(&self, key: &str) -> Result<AudioBuffer> {
        let full_path = self.get_full_path(key);

        let output = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(full_path)
            .send()
            .await?;

        let data = output.body.collect().await?.into_bytes();
        Ok(AudioBuffer::from_bytes(data.to_vec()))
    }

    #[tracing::instrument(skip(self, blob))]
    async fn put(&self, key: &str, blob: &AudioBuffer) -> Result<()> {
        let full_path = self.get_full_path(key);

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(full_path)
            .body(ByteStream::from(blob.as_ref().to_vec()))
            .send()
            .await?;

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn delete(&self, key: &str) -> Result<()> {
        let full_path = self.get_full_path(key);

        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(full_path)
            .send()
            .await?;

        Ok(())
    }
}

impl S3Storage {
    #[tracing::instrument]
    pub async fn new(
        base_dir: String,
        path_prefix: String,
        safe_chars: SafeCharsType,
        endpoint_url: String,
        bucket: String,
        region: String,
        access_key: &str,
        secret_key: &str,
    ) -> Result<Self> {
        // Create custom credentials
        let credentials = Credentials::new(
            access_key, secret_key, None, // session token
            None, // expiry
            "minio",
        );

        // Create the config
        let config = aws_sdk_s3::Config::builder()
            .behavior_version_latest()
            .region(Region::new(region))
            .endpoint_url(&endpoint_url)
            .credentials_provider(credentials)
            .force_path_style(true) // This is important for MinIO
            .build();

        let client = Client::from_conf(config);

        // Wait for MinIO to be ready
        debug!(
            "Waiting for MinIO to be ready... {} - {}",
            endpoint_url, bucket
        );
        wait_for_minio(&client, 5, Duration::from_secs(2)).await?;

        Ok(S3Storage {
            base_dir,
            path_prefix,
            safe_chars,
            client,
            bucket,
        })
    }

    #[tracing::instrument(skip(self))]
    pub async fn ensure_bucket_exists(&self) -> Result<()> {
        let exists = self
            .client
            .head_bucket()
            .bucket(&self.bucket)
            .send()
            .await
            .is_ok();

        if !exists {
            warn!("Bucket does not exist, creating it...");
            self.client
                .create_bucket()
                .bucket(&self.bucket)
                .send()
                .await?;
        }

        Ok(())
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

async fn wait_for_minio(client: &Client, max_retries: u32, delay: Duration) -> Result<()> {
    for i in 0..max_retries {
        match client.list_buckets().send().await {
            Ok(_) => {
                info!("Successfully connected to MinIO");
                return Ok(());
            }
            Err(e) => {
                if i == max_retries - 1 {
                    return Err(e.into());
                }
                info!(
                    "Waiting for Storage to be ready... (attempt {}/{})",
                    i + 1,
                    max_retries
                );
                tokio::time::sleep(delay).await;
            }
        }
    }
    Err(color_eyre::eyre::eyre!(
        "Failed to connect to MinIO after {} retries",
        max_retries
    ))
}

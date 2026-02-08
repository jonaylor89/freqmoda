use std::{collections::HashMap, num::NonZeroUsize};

use async_trait::async_trait;
use color_eyre::Result;
use tokio::sync::Semaphore;
use tracing::{info, instrument};

use crate::{
    blob::AudioBuffer, config::ProcessorSettings, processor::ffmpeg::process_audio,
    streamingpath::params::Params,
};

#[async_trait]
pub trait AudioProcessor: Send + Sync {
    async fn process(&self, blob: &AudioBuffer, params: &Params) -> Result<AudioBuffer>;
}

#[derive(Debug)]
pub struct Processor {
    semaphore: Semaphore,
    tags: HashMap<String, String>,
}

#[async_trait]
impl AudioProcessor for Processor {
    #[tracing::instrument(skip(self, blob, params))]
    async fn process(&self, blob: &AudioBuffer, params: &Params) -> Result<AudioBuffer> {
        let _permit = self.semaphore.acquire().await?;
        info!(params = ?params, "Processing with FFmpeg native bindings");

        let processed_audio = process_audio(blob, params, &self.tags).await?;
        info!("Audio processing completed successfully");

        Ok(processed_audio)
    }
}

impl Processor {
    #[instrument(skip(config, tags))]
    pub fn new(config: ProcessorSettings, tags: HashMap<String, String>) -> Self {
        let max_concurrent = config
            .concurrency
            .map(|concurrency| {
                NonZeroUsize::new(concurrency).expect("Concurrency should be non-zero")
            })
            .unwrap_or_else(|| {
                NonZeroUsize::new(num_cpus::get())
                    .expect("Number of CPUs should always be non-zero")
            });

        info!(
            max_concurrent = max_concurrent.get(),
            "Initializing processor"
        );

        Self {
            semaphore: Semaphore::new(max_concurrent.get()),
            tags,
        }
    }
}

//! Safe Rust wrapper for FFmpeg audio processing.
//!
//! This crate provides a high-level API for audio transcoding and filtering
//! using FFmpeg's libav* libraries with in-memory buffers.

mod error;
mod handle;
mod io;
mod pipeline;

pub use error::FfmpegError;
pub use pipeline::{AudioProcessor, OutputFormat, ProcessOptions};

use std::sync::Once;

static INIT: Once = Once::new();

/// Initialize FFmpeg. Called automatically when creating an AudioProcessor.
pub fn init() {
    INIT.call_once(|| {
        // In modern FFmpeg (4.0+), av_register_all() is deprecated and no-op.
        // Network initialization is still needed for some protocols.
        #[cfg(feature = "network")]
        unsafe {
            ffmpeg_sys::avformat_network_init();
        }
    });
}

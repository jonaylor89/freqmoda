//! Error types for FFmpeg operations.

use std::ffi::CStr;
use std::fmt;

/// Error type for FFmpeg operations.
#[derive(Debug)]
pub enum FfmpegError {
    /// FFmpeg returned an error code
    Ffmpeg {
        code: i32,
        operation: &'static str,
        detail: String,
    },
    /// Memory allocation failed
    Allocation(&'static str),
    /// No audio stream found in input
    NoAudioStream,
    /// Unsupported codec or format
    Unsupported(String),
    /// Invalid parameter
    InvalidParameter(String),
    /// I/O error
    Io(std::io::Error),
    /// Filter graph configuration error
    FilterConfig(String),
    /// Encoder/decoder not found
    CodecNotFound(String),
}

impl std::error::Error for FfmpegError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            FfmpegError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl fmt::Display for FfmpegError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FfmpegError::Ffmpeg {
                code,
                operation,
                detail,
            } => {
                write!(f, "FFmpeg error {} in {}: {}", code, operation, detail)
            }
            FfmpegError::Allocation(what) => write!(f, "Failed to allocate {}", what),
            FfmpegError::NoAudioStream => write!(f, "No audio stream found in input"),
            FfmpegError::Unsupported(msg) => write!(f, "Unsupported: {}", msg),
            FfmpegError::InvalidParameter(msg) => write!(f, "Invalid parameter: {}", msg),
            FfmpegError::Io(e) => write!(f, "I/O error: {}", e),
            FfmpegError::FilterConfig(msg) => write!(f, "Filter configuration error: {}", msg),
            FfmpegError::CodecNotFound(name) => write!(f, "Codec not found: {}", name),
        }
    }
}

impl From<std::io::Error> for FfmpegError {
    fn from(e: std::io::Error) -> Self {
        FfmpegError::Io(e)
    }
}

/// Convert FFmpeg error code to human-readable string.
pub fn error_string(code: i32) -> String {
    let mut buf = [0i8; 256];
    unsafe {
        ffmpeg_sys::av_strerror(code, buf.as_mut_ptr(), buf.len());
        CStr::from_ptr(buf.as_ptr()).to_string_lossy().into_owned()
    }
}

/// Check FFmpeg return value and convert to Result.
#[inline]
pub fn check(code: i32, operation: &'static str) -> Result<i32, FfmpegError> {
    if code < 0 {
        Err(FfmpegError::Ffmpeg {
            code,
            operation,
            detail: error_string(code),
        })
    } else {
        Ok(code)
    }
}

/// Check FFmpeg return value, treating EAGAIN/EOF as Ok(false).
#[inline]
pub fn check_again(code: i32, operation: &'static str) -> Result<bool, FfmpegError> {
    if code >= 0 {
        Ok(true)
    } else if ffmpeg_sys::is_eagain(code) || ffmpeg_sys::is_eof(code) {
        Ok(false)
    } else {
        Err(FfmpegError::Ffmpeg {
            code,
            operation,
            detail: error_string(code),
        })
    }
}

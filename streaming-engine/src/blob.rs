use bytes::Bytes;
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::{fmt, path::PathBuf, str::FromStr};
use tokio::fs;
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
pub enum AudioFormat {
    Mp3,
    Wav,
    Flac,
    Ogg,
    M4a,
    Opus,
    Unknown,
}

impl AudioFormat {
    fn from_header(data: &[u8]) -> Self {
        match data {
            [0xFF, 0xFB, ..] => Self::Mp3,
            d if d.starts_with(b"RIFF") => Self::Wav,
            d if d.starts_with(b"fLaC") => Self::Flac,
            d if d.starts_with(b"OggS") => Self::Ogg,
            d if d.starts_with(b"ftypM4A ") => Self::M4a,
            d if d.starts_with(b"OpusHead") => Self::Opus,
            _ => Self::Unknown,
        }
    }

    fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "mp3" => Self::Mp3,
            "wav" => Self::Wav,
            "flac" => Self::Flac,
            "ogg" => Self::Ogg,
            "m4a" => Self::M4a,
            "opus" => Self::Opus,
            _ => Self::Unknown,
        }
    }

    fn mime_type(&self) -> &'static str {
        match self {
            Self::Mp3 => "audio/mpeg",
            Self::Wav => "audio/wav",
            Self::Flac => "audio/flac",
            Self::Ogg => "audio/ogg",
            Self::M4a => "audio/mp4",
            Self::Opus => "audio/opus",
            Self::Unknown => "application/octet-stream",
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            Self::Mp3 => "mp3",
            Self::Wav => "wav",
            Self::Flac => "flac",
            Self::Ogg => "ogg",
            Self::M4a => "m4a",
            Self::Opus => "opus",
            Self::Unknown => "",
        }
    }
}

impl FromStr for AudioFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mp3" => Ok(Self::Mp3),
            "wav" => Ok(Self::Wav),
            "flac" => Ok(Self::Flac),
            "ogg" => Ok(Self::Ogg),
            "m4a" => Ok(Self::M4a),
            "opus" => Ok(Self::Opus),
            "" => Ok(Self::Unknown),
            _ => Err(format!("Unknown audio format: {}", s)),
        }
    }
}

impl fmt::Display for AudioFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.extension())
    }
}

impl Serialize for AudioFormat {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.extension())
    }
}

impl<'de> Deserialize<'de> for AudioFormat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

#[derive(Debug)]
pub struct AudioBuffer {
    data: Bytes,
    format: AudioFormat,
}

impl AsRef<[u8]> for AudioBuffer {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

impl From<AudioBuffer> for Bytes {
    fn from(val: AudioBuffer) -> Self {
        val.data
    }
}

impl AudioBuffer {
    pub async fn from_file(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        let data = fs::read(&path).await?;
        let format = match AudioFormat::from_header(&data) {
            AudioFormat::Unknown => {
                let extension = path.to_str().unwrap_or_default().split(".").last();
                if let Some(ext) = extension {
                    AudioFormat::from_extension(ext)
                } else {
                    AudioFormat::Unknown
                }
            }
            rest => rest,
        };

        Ok(Self {
            data: data.into(),
            format,
        })
    }

    pub fn from_bytes(data: impl Into<Bytes>) -> Self {
        let data = data.into();
        let format = AudioFormat::from_header(&data);

        Self { data, format }
    }

    pub fn from_bytes_with_format(data: impl Into<Bytes>, format: AudioFormat) -> Self {
        let data = data.into();

        Self { data, format }
    }

    pub fn format(&self) -> AudioFormat {
        self.format
    }

    pub fn extension(&self) -> &'static str {
        self.format.extension()
    }

    pub fn mime_type(&self) -> &'static str {
        self.format.mime_type()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn into_bytes(self) -> Bytes {
        self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_audio_buffer() {
        // MP3 test data
        let mp3_data = {
            let mut data = vec![0xFF, 0xFB];
            data.extend_from_slice(&[0; 1024]);
            data
        };

        let buffer = AudioBuffer::from_bytes(mp3_data);
        assert_eq!(buffer.format(), AudioFormat::Mp3);
        assert_eq!(buffer.mime_type(), "audio/mpeg");
        assert_eq!(buffer.len(), 1026);
        assert!(!buffer.is_empty());
    }
}

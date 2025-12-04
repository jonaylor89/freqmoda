use color_eyre::Result;
use std::collections::HashMap;
use tracing::instrument;

use crate::{
    blob::{AudioBuffer, AudioFormat},
    streamingpath::params::Params,
};

use ffmpeg::{AudioProcessor, OutputFormat, ProcessOptions};

/// Convert AudioFormat to output format specification.
fn audio_format_to_output(format: AudioFormat, params: &Params) -> OutputFormat {
    let mut output = OutputFormat::from_extension(format.extension());

    // Override codec if specified
    if let Some(ref codec) = params.codec {
        output.codec = Some(codec.clone());
    }

    // Apply other parameters
    output.sample_rate = params.sample_rate;
    output.channels = params.channels;
    output.bit_rate = params.bit_rate.map(|r| r as i64 * 1000); // Convert kbps to bps
    output.quality = params.quality.map(|q| q as f32);
    output.compression_level = params.compression_level;

    output
}

/// Convert Params filters to FFmpeg filter string.
fn collect_filters(params: &Params) -> Option<String> {
    let mut filters = Vec::new();

    if let Some(speed) = params.speed {
        if speed != 1.0 {
            filters.push(format!("atempo={:.3}", speed));
        }
    }
    if let Some(true) = params.reverse {
        filters.push("areverse".to_string());
    }
    if let Some(volume) = params.volume {
        if volume != 1.0 {
            filters.push(format!("volume={:.2}", volume));
        }
    }
    if let Some(true) = params.normalize {
        let level = params.normalize_level.unwrap_or(-16.0);
        filters.push(format!("loudnorm=I={:.1}", level));
    }
    if let Some(freq) = params.lowpass {
        filters.push(format!("lowpass=f={:.1}", freq));
    }
    if let Some(freq) = params.highpass {
        filters.push(format!("highpass=f={:.1}", freq));
    }
    if let Some(band) = &params.bandpass {
        filters.push(format!("bandpass={}", band));
    }
    if let Some(bass) = params.bass {
        filters.push(format!("bass=g={:.1}", bass));
    }
    if let Some(treble) = params.treble {
        filters.push(format!("treble=g={:.1}", treble));
    }
    if let Some(echo) = &params.echo {
        filters.push(format!("aecho={}", echo));
    }
    if let Some(chorus) = &params.chorus {
        filters.push(format!("chorus={}", chorus));
    }
    if let Some(flanger) = &params.flanger {
        filters.push(format!("flanger={}", flanger));
    }
    if let Some(phaser) = &params.phaser {
        filters.push(format!("aphaser={}", phaser));
    }
    if let Some(tremolo) = &params.tremolo {
        filters.push(format!("tremolo={}", tremolo));
    }
    if let Some(compressor) = &params.compressor {
        filters.push(format!("acompressor={}", compressor));
    }
    if let Some(nr) = &params.noise_reduction {
        filters.push(format!("anlmdn={}", nr));
    }
    if let Some(fade) = params.fade_in {
        filters.push(format!("afade=t=in:d={:.3}", fade));
    }
    if let Some(fade) = params.fade_out {
        filters.push(format!("afade=t=out:d={:.3}", fade));
    }
    if let Some(fade) = params.cross_fade {
        filters.push(format!("acrossfade=d={:.3}", fade));
    }

    if let Some(custom_filters) = &params.custom_filters {
        filters.extend(custom_filters.clone());
    }

    if filters.is_empty() {
        None
    } else {
        Some(filters.join(","))
    }
}

#[instrument(skip(input, params, additional_tags))]
pub async fn process_audio(
    input: &AudioBuffer,
    params: &Params,
    additional_tags: &HashMap<String, String>,
) -> Result<AudioBuffer> {
    let output_format = params.format.unwrap_or(AudioFormat::Mp3);

    // Combine tags
    let mut metadata = additional_tags.clone();
    if let Some(tags) = &params.tags {
        for (k, v) in tags {
            metadata.insert(k.clone(), v.clone());
        }
    }

    // Collect filters
    let filters = collect_filters(params);

    // Build output format
    let output_spec = audio_format_to_output(output_format, params);

    // Get input data
    let input_data = input.as_ref().to_vec();
    let start_time = params.start_time;
    let duration = params.duration;

    // Process in blocking task since FFmpeg is CPU-bound
    let processed = tokio::task::spawn_blocking(move || -> Result<Vec<u8>, ffmpeg::FfmpegError> {
        let processor = AudioProcessor::new()?;
        processor.process(ProcessOptions {
            input: &input_data,
            output_format: output_spec,
            filters,
            metadata: &metadata,
            start_time,
            duration,
        })
    })
    .await??;

    Ok(AudioBuffer::from_bytes_with_format(
        processed,
        output_format,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_filters_empty() {
        let params = Params::default();
        assert!(collect_filters(&params).is_none());
    }

    #[test]
    fn test_collect_filters_volume() {
        let params = Params {
            volume: Some(0.5),
            ..Default::default()
        };
        assert_eq!(collect_filters(&params), Some("volume=0.50".to_string()));
    }

    #[test]
    fn test_collect_filters_multiple() {
        let params = Params {
            volume: Some(0.8),
            speed: Some(1.5),
            lowpass: Some(8000.0),
            ..Default::default()
        };
        let filters = collect_filters(&params).unwrap();
        assert!(filters.contains("atempo=1.500"));
        assert!(filters.contains("volume=0.80"));
        assert!(filters.contains("lowpass=f=8000.0"));
    }

    #[test]
    fn test_audio_format_to_output() {
        let params = Params {
            bit_rate: Some(320),
            sample_rate: Some(48000),
            ..Default::default()
        };
        let output = audio_format_to_output(AudioFormat::Mp3, &params);
        assert_eq!(output.format, "mp3");
        assert_eq!(output.codec, Some("libmp3lame".to_string()));
        assert_eq!(output.bit_rate, Some(320_000));
        assert_eq!(output.sample_rate, Some(48000));
    }
}

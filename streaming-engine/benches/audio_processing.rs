fn main() {
    divan::main();
}

use divan::{Bencher, black_box};
use std::collections::HashMap;
use streaming_engine::{
    blob::{AudioBuffer, AudioFormat},
    config::ProcessorSettings,
    processor::processor::{AudioProcessor, Processor},
    streamingpath::params::Params,
};

// Mock audio data generators
fn generate_audio_data(size_kb: usize, format: AudioFormat) -> Vec<u8> {
    let size_bytes = size_kb * 1024;
    let mut data = Vec::with_capacity(size_bytes);

    match format {
        AudioFormat::Mp3 => {
            // MP3 header
            data.extend_from_slice(&[0xFF, 0xFB, 0x90, 0x00]);
            // Fill with pseudo-random audio data
            for i in 0..size_bytes - 4 {
                data.push(((i * 31) % 256) as u8);
            }
        }
        AudioFormat::Wav => {
            // WAV header (simplified)
            data.extend_from_slice(b"RIFF");
            data.extend_from_slice(&(size_bytes as u32 - 8).to_le_bytes());
            data.extend_from_slice(b"WAVE");
            data.extend_from_slice(b"fmt ");
            data.extend_from_slice(&16u32.to_le_bytes()); // fmt chunk size
            data.extend_from_slice(&1u16.to_le_bytes()); // audio format (PCM)
            data.extend_from_slice(&2u16.to_le_bytes()); // channels
            data.extend_from_slice(&44100u32.to_le_bytes()); // sample rate
            data.extend_from_slice(&176400u32.to_le_bytes()); // byte rate
            data.extend_from_slice(&4u16.to_le_bytes()); // block align
            data.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
            data.extend_from_slice(b"data");
            data.extend_from_slice(&(size_bytes as u32 - 44).to_le_bytes());

            // Fill remaining with audio data
            for i in 0..size_bytes - 44 {
                data.push(((i * 17) % 256) as u8);
            }
        }
        AudioFormat::Flac => {
            data.extend_from_slice(b"fLaC");
            for i in 0..size_bytes - 4 {
                data.push(((i * 23) % 256) as u8);
            }
        }
        _ => {
            // Generic data for other formats
            for i in 0..size_bytes {
                data.push(((i * 13) % 256) as u8);
            }
        }
    }

    data
}

fn create_test_params(complexity: &str) -> Params {
    match complexity {
        "simple" => Params {
            key: "test.mp3".to_string(),
            format: Some(AudioFormat::Mp3),
            ..Default::default()
        },
        "medium" => Params {
            key: "test.mp3".to_string(),
            format: Some(AudioFormat::Wav),
            sample_rate: Some(48000),
            channels: Some(2),
            volume: Some(1.2),
            lowpass: Some(8000.0),
            ..Default::default()
        },
        "complex" => Params {
            key: "test.mp3".to_string(),
            format: Some(AudioFormat::Flac),
            sample_rate: Some(96000),
            channels: Some(2),
            bit_depth: Some(24),
            volume: Some(0.8),
            normalize: Some(true),
            lowpass: Some(20000.0),
            highpass: Some(20.0),
            echo: Some("0.8:0.88:60:0.4".to_string()),
            compressor: Some("6:1:1:-3:0.1:0.1".to_string()),
            fade_in: Some(2.0),
            fade_out: Some(3.0),
            ..Default::default()
        },
        _ => Default::default(),
    }
}

#[divan::bench_group]
mod audio_buffer_creation {
    use super::*;
    #[divan::bench(args = [1, 10, 100, 1000])]
    fn mp3_from_bytes(size_kb: usize) -> AudioBuffer {
        let data = generate_audio_data(size_kb, AudioFormat::Mp3);
        black_box(AudioBuffer::from_bytes(black_box(data)))
    }

    #[divan::bench(args = [1, 10, 100, 1000])]
    fn wav_from_bytes(size_kb: usize) -> AudioBuffer {
        let data = generate_audio_data(size_kb, AudioFormat::Wav);
        black_box(AudioBuffer::from_bytes(black_box(data)))
    }

    #[divan::bench(args = [1, 10, 100, 1000])]
    fn flac_from_bytes(size_kb: usize) -> AudioBuffer {
        let data = generate_audio_data(size_kb, AudioFormat::Flac);
        black_box(AudioBuffer::from_bytes(black_box(data)))
    }

    #[divan::bench(args = [1, 10, 100, 1000])]
    fn mp3_from_bytes_with_format(size_kb: usize) -> AudioBuffer {
        let data = generate_audio_data(size_kb, AudioFormat::Mp3);
        black_box(AudioBuffer::from_bytes_with_format(
            black_box(data),
            black_box(AudioFormat::Mp3),
        ))
    }

    #[divan::bench(args = [1, 10, 100, 1000])]
    fn wav_from_bytes_with_format(size_kb: usize) -> AudioBuffer {
        let data = generate_audio_data(size_kb, AudioFormat::Wav);
        black_box(AudioBuffer::from_bytes_with_format(
            black_box(data),
            black_box(AudioFormat::Wav),
        ))
    }
}

#[divan::bench_group]
mod audio_format_detection {
    use super::*;

    #[divan::bench]
    fn detect_mp3() -> AudioFormat {
        let mut data = vec![0xFF, 0xFB];
        data.extend(vec![0u8; 1024]);
        let buffer = AudioBuffer::from_bytes(black_box(data));
        black_box(buffer.format())
    }

    #[divan::bench]
    fn detect_wav() -> AudioFormat {
        let mut data = b"RIFF".to_vec();
        data.extend(vec![0u8; 1024]);
        let buffer = AudioBuffer::from_bytes(black_box(data));
        black_box(buffer.format())
    }

    #[divan::bench]
    fn detect_flac() -> AudioFormat {
        let mut data = b"fLaC".to_vec();
        data.extend(vec![0u8; 1024]);
        let buffer = AudioBuffer::from_bytes(black_box(data));
        black_box(buffer.format())
    }

    #[divan::bench]
    fn detect_ogg() -> AudioFormat {
        let mut data = b"OggS".to_vec();
        data.extend(vec![0u8; 1024]);
        let buffer = AudioBuffer::from_bytes(black_box(data));
        black_box(buffer.format())
    }
}

#[divan::bench_group(sample_count = 10)] // Fewer samples due to expensive operations
mod audio_processing {
    use super::*;

    #[divan::bench(args = [("simple", 10), ("medium", 10), ("complex", 10), ("simple", 100), ("medium", 100)])]
    fn process_audio(bencher: Bencher<'_, '_>, (complexity, size_kb): (&str, usize)) {
        let processor = Processor::new(
            ProcessorSettings {
                disabled_filters: Vec::new(),
                max_filter_ops: 100,
                concurrency: Some(1),
                max_cache_files: 1000,
                max_cache_mem: 100 * 1024 * 1024,   // 100MB
                max_cache_size: 1024 * 1024 * 1024, // 1GB
            },
            HashMap::new(),
        );

        let audio_data = generate_audio_data(size_kb, AudioFormat::Mp3);
        let audio_buffer = AudioBuffer::from_bytes(audio_data);
        let params = create_test_params(complexity);

        bencher.bench(|| async {
            let result = processor
                .process(black_box(&audio_buffer), black_box(&params))
                .await;
            black_box(result)
        })
    }
}

#[divan::bench_group(sample_count = 5)] // Even fewer samples for concurrent tests
mod concurrent_processing {
    use super::*;

    #[divan::bench(args = [1, 2, 4, 8])]
    fn concurrent_processing(bencher: Bencher<'_, '_>, concurrency: usize) {
        let processor = Processor::new(
            ProcessorSettings {
                disabled_filters: Vec::new(),
                max_filter_ops: 100,
                concurrency: Some(concurrency),
                max_cache_files: 1000,
                max_cache_mem: 100 * 1024 * 1024,   // 100MB
                max_cache_size: 1024 * 1024 * 1024, // 1GB
            },
            HashMap::new(),
        );

        let audio_data = generate_audio_data(50, AudioFormat::Mp3);
        let audio_buffer = AudioBuffer::from_bytes(audio_data);
        let params = create_test_params("medium");

        bencher.bench(|| async {
            let tasks: Vec<_> = (0..concurrency)
                .map(|_| {
                    let processor = &processor;
                    let audio_buffer = &audio_buffer;
                    let params = &params;

                    async move {
                        processor
                            .process(black_box(audio_buffer), black_box(params))
                            .await
                    }
                })
                .collect();

            let results = futures::future::join_all(tasks).await;
            black_box(results)
        })
    }
}

#[divan::bench_group]
mod memory_operations {
    use super::*;

    #[divan::bench(args = [1, 10, 100, 1000])]
    fn clone_buffer(size_kb: usize) -> Vec<u8> {
        let data = generate_audio_data(size_kb, AudioFormat::Mp3);
        let buffer = AudioBuffer::from_bytes(data);
        // AudioBuffer doesn't have Clone, so clone the underlying data
        let cloned_data: &[u8] = buffer.as_ref();
        black_box(cloned_data.to_vec())
    }

    #[divan::bench(args = [1, 10, 100, 1000])]
    fn into_bytes(bencher: Bencher<'_, '_>, size_kb: usize) {
        bencher
            .with_inputs(|| {
                let data = generate_audio_data(size_kb, AudioFormat::Mp3);
                AudioBuffer::from_bytes(data)
            })
            .bench_values(|buf| {
                let bytes = buf.into_bytes();
                black_box(bytes)
            })
    }

    #[divan::bench(args = [1, 10, 100, 1000])]
    fn as_ref_len(size_kb: usize) -> usize {
        let data = generate_audio_data(size_kb, AudioFormat::Mp3);
        let buffer = AudioBuffer::from_bytes(data);
        let slice: &[u8] = black_box(&buffer).as_ref();
        black_box(slice.len())
    }

    #[divan::bench(args = [1, 10, 100, 1000])]
    fn buffer_len(size_kb: usize) -> usize {
        let data = generate_audio_data(size_kb, AudioFormat::Mp3);
        let buffer = AudioBuffer::from_bytes(data);
        black_box(buffer.len())
    }

    #[divan::bench(args = [1, 10, 100, 1000])]
    fn buffer_is_empty(size_kb: usize) -> bool {
        let data = generate_audio_data(size_kb, AudioFormat::Mp3);
        let buffer = AudioBuffer::from_bytes(data);
        black_box(buffer.is_empty())
    }
}

#[divan::bench_group]
mod format_operations {
    use super::*;

    #[divan::bench]
    fn format_extension() -> &'static str {
        let buffer = AudioBuffer::from_bytes(generate_audio_data(10, AudioFormat::Mp3));
        black_box(buffer.extension())
    }

    #[divan::bench]
    fn format_mime_type() -> &'static str {
        let buffer = AudioBuffer::from_bytes(generate_audio_data(10, AudioFormat::Mp3));
        black_box(buffer.mime_type())
    }

    #[divan::bench]
    fn format_to_string() -> String {
        let format = AudioFormat::Mp3;
        black_box(format.to_string())
    }

    #[divan::bench]
    fn format_from_str() -> Result<AudioFormat, String> {
        black_box("mp3".parse::<AudioFormat>())
    }
}

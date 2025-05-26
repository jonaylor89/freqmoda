fn main() {
    divan::main();
}

use divan::{Bencher, black_box};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use streaming_engine::{
    blob::{AudioBuffer, AudioFormat},
    cache::{cache::AudioCache, fs::FileSystemCache},
    config::ProcessorSettings,
    processor::processor::{AudioProcessor, Processor},
    storage::{file::FileStorage, storage::AudioStorage},
    streamingpath::{
        hasher::{digest_result_storage_hasher, digest_storage_hasher},
        normalize::SafeCharsType,
        params::Params,
    },
};
use tempfile::tempdir;

// Mock audio data generators matching real-world scenarios
fn generate_realistic_audio_data(size_kb: usize, format: AudioFormat) -> Vec<u8> {
    let size_bytes = size_kb * 1024;
    let mut data = Vec::with_capacity(size_bytes);

    match format {
        AudioFormat::Mp3 => {
            // Valid MP3 header with MPEG-1 Layer 3
            data.extend_from_slice(&[0xFF, 0xFB, 0x90, 0x00]);
            // Add some realistic MP3 frame data
            for i in 0..size_bytes - 4 {
                data.push(((i * 31 + 17) % 256) as u8);
            }
        }
        AudioFormat::Wav => {
            // Complete WAV header
            data.extend_from_slice(b"RIFF");
            data.extend_from_slice(&(size_bytes as u32 - 8).to_le_bytes());
            data.extend_from_slice(b"WAVE");
            data.extend_from_slice(b"fmt ");
            data.extend_from_slice(&16u32.to_le_bytes());
            data.extend_from_slice(&1u16.to_le_bytes()); // PCM format
            data.extend_from_slice(&2u16.to_le_bytes()); // stereo
            data.extend_from_slice(&44100u32.to_le_bytes()); // 44.1kHz
            data.extend_from_slice(&176400u32.to_le_bytes()); // byte rate
            data.extend_from_slice(&4u16.to_le_bytes()); // block align
            data.extend_from_slice(&16u16.to_le_bytes()); // 16-bit
            data.extend_from_slice(b"data");
            data.extend_from_slice(&(size_bytes as u32 - 44).to_le_bytes());

            // Generate stereo audio samples
            for i in 0..(size_bytes - 44) / 2 {
                let sample = (((i as f64 * 440.0 * 2.0 * std::f64::consts::PI) / 44100.0).sin()
                    * 16384.0) as i16;
                data.extend_from_slice(&sample.to_le_bytes());
            }
        }
        AudioFormat::Flac => {
            data.extend_from_slice(b"fLaC");
            // Add FLAC metadata block
            data.extend_from_slice(&[0x00, 0x00, 0x00, 0x22]); // STREAMINFO block
            // Fill rest with compressed audio data
            for i in 0..size_bytes - 8 {
                data.push(((i * 23 + 41) % 256) as u8);
            }
        }
        AudioFormat::Ogg => {
            data.extend_from_slice(b"OggS");
            data.extend_from_slice(&[0x00, 0x02]); // version, header type
            for i in 0..size_bytes - 6 {
                data.push(((i * 19 + 37) % 256) as u8);
            }
        }
        _ => {
            // Generic binary data
            for i in 0..size_bytes {
                data.push(((i * 13 + 7) % 256) as u8);
            }
        }
    }

    data
}

fn create_realistic_params(scenario: &str) -> Params {
    match scenario {
        "podcast" => Params {
            key: "podcast_episode_001.mp3".to_string(),
            format: Some(AudioFormat::Mp3),
            sample_rate: Some(44100),
            channels: Some(1), // mono for podcasts
            volume: Some(1.0),
            normalize: Some(true),
            lowpass: Some(8000.0), // speech optimization
            compressor: Some("3:1:1:-12:0.1:0.1".to_string()),
            ..Default::default()
        },
        "music_streaming" => Params {
            key: "song_high_quality.flac".to_string(),
            format: Some(AudioFormat::Mp3),
            sample_rate: Some(44100),
            channels: Some(2),
            volume: Some(1.0),
            bit_depth: Some(16),
            ..Default::default()
        },
        "radio_broadcast" => Params {
            key: "live_broadcast.wav".to_string(),
            format: Some(AudioFormat::Mp3),
            sample_rate: Some(22050),
            channels: Some(1),
            volume: Some(0.9),
            normalize: Some(true),
            lowpass: Some(11000.0),
            highpass: Some(80.0),
            compressor: Some("4:1:1:-18:0.05:0.1".to_string()),
            ..Default::default()
        },
        "audiobook" => Params {
            key: "chapter_01.mp3".to_string(),
            format: Some(AudioFormat::Mp3),
            sample_rate: Some(22050),
            channels: Some(1),
            volume: Some(1.1),
            normalize: Some(true),
            lowpass: Some(8000.0),
            compressor: Some("6:1:1:-15:0.1:0.05".to_string()),
            fade_in: Some(0.5),
            fade_out: Some(1.0),
            ..Default::default()
        },
        "gaming_audio" => Params {
            key: "game_soundtrack.ogg".to_string(),
            format: Some(AudioFormat::Mp3),
            sample_rate: Some(48000),
            channels: Some(2),
            volume: Some(0.8),
            bit_depth: Some(16),
            ..Default::default()
        },
        "mobile_optimized" => Params {
            key: "mobile_track.mp3".to_string(),
            format: Some(AudioFormat::Mp3),
            sample_rate: Some(32000),
            channels: Some(2),
            volume: Some(1.0),
            normalize: Some(true),
            lowpass: Some(15000.0),
            ..Default::default()
        },
        _ => Default::default(),
    }
}

#[divan::bench_group]
mod end_to_end_pipeline {
    use super::*;

    #[divan::bench(args = [
        ("podcast", AudioFormat::Mp3, 50),
        ("music_streaming", AudioFormat::Flac, 200),
        ("radio_broadcast", AudioFormat::Wav, 100),
        ("audiobook", AudioFormat::Mp3, 30),
        ("gaming_audio", AudioFormat::Ogg, 150),
        ("mobile_optimized", AudioFormat::Mp3, 80)
    ])]
    fn full_streaming_pipeline(
        bencher: Bencher<'_, '_>,
        (scenario, input_format, size_kb): (&str, AudioFormat, usize),
    ) {
        let temp_dir = tempdir().unwrap();
        let cache_dir = temp_dir.path().join("cache");
        let storage_dir = temp_dir.path().join("storage");

        std::fs::create_dir_all(&cache_dir).unwrap();
        std::fs::create_dir_all(&storage_dir).unwrap();

        let processor = Processor::new(
            ProcessorSettings {
                disabled_filters: Vec::new(),
                max_filter_ops: 100,
                concurrency: Some(2),
                max_cache_files: 1000,
                max_cache_mem: 100 * 1024 * 1024,
                max_cache_size: 1024 * 1024 * 1024,
            },
            HashMap::new(),
        );

        let storage = FileStorage::new(storage_dir, "audio".to_string(), SafeCharsType::Default);

        let cache = FileSystemCache::new(&cache_dir).unwrap();

        let audio_data = generate_realistic_audio_data(size_kb, input_format);
        let input_buffer = AudioBuffer::from_bytes_with_format(audio_data, input_format);
        let params = create_realistic_params(scenario);

        bencher.bench(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                // 1. Hash computation for cache key
                let cache_key = digest_storage_hasher(&params.key);

                // 2. Check cache first
                if let Ok(Some(_cached)) = cache.get(&cache_key).await {
                    return Ok::<_, color_eyre::Report>(());
                }

                // 3. Process audio
                let processed = processor
                    .process(black_box(&input_buffer), black_box(&params))
                    .await?;

                // 4. Store result
                let storage_key = format!("processed_{}", params.key);
                storage.put(&storage_key, &processed).await?;

                // 5. Cache the result
                let processed_bytes = processed.into_bytes();
                cache
                    .set(
                        &cache_key,
                        &processed_bytes,
                        Some(Duration::from_secs(3600)),
                    )
                    .await?;

                Ok::<_, color_eyre::Report>(())
            })
        })
    }
}

#[divan::bench_group]
mod concurrent_streaming {
    use super::*;
    use futures::future::join_all;

    #[divan::bench(args = [1, 2, 4, 8, 16])]
    fn concurrent_requests(bencher: Bencher<'_, '_>, concurrency: usize) {
        let temp_dir = tempdir().unwrap();
        let cache_dir = temp_dir.path().join("cache");
        let storage_dir = temp_dir.path().join("storage");

        std::fs::create_dir_all(&cache_dir).unwrap();
        std::fs::create_dir_all(&storage_dir).unwrap();

        let processor = Arc::new(Processor::new(
            ProcessorSettings {
                disabled_filters: Vec::new(),
                max_filter_ops: 100,
                concurrency: Some(concurrency),
                max_cache_files: 1000,
                max_cache_mem: 100 * 1024 * 1024,
                max_cache_size: 1024 * 1024 * 1024,
            },
            HashMap::new(),
        ));

        let storage = Arc::new(FileStorage::new(
            storage_dir,
            "audio".to_string(),
            SafeCharsType::Default,
        ));

        let cache = Arc::new(FileSystemCache::new(&cache_dir).unwrap());

        // Prepare test data for each concurrent request
        let test_scenarios = [
            ("podcast", AudioFormat::Mp3, 25),
            ("music_streaming", AudioFormat::Flac, 50),
            ("radio_broadcast", AudioFormat::Wav, 30),
            ("audiobook", AudioFormat::Mp3, 20),
            ("gaming_audio", AudioFormat::Ogg, 40),
            ("mobile_optimized", AudioFormat::Mp3, 35),
        ];

        bencher.bench(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let tasks = (0..concurrency).map(|i| {
                    let processor = Arc::clone(&processor);
                    let storage = Arc::clone(&storage);
                    let cache = Arc::clone(&cache);
                    let (scenario, format, size_kb) = test_scenarios[i % test_scenarios.len()];

                    async move {
                        let audio_data = generate_realistic_audio_data(size_kb, format);
                        let input_buffer = AudioBuffer::from_bytes_with_format(audio_data, format);
                        let base_params = create_realistic_params(scenario);
                        let params = Params {
                            key: format!("{}_{}", base_params.key, i), // Make keys unique
                            format: base_params.format,
                            sample_rate: base_params.sample_rate,
                            channels: base_params.channels,
                            volume: base_params.volume,
                            normalize: base_params.normalize,
                            lowpass: base_params.lowpass,
                            highpass: base_params.highpass,
                            compressor: base_params.compressor,
                            fade_in: base_params.fade_in,
                            fade_out: base_params.fade_out,
                            bit_depth: base_params.bit_depth,
                            ..Default::default()
                        };

                        let cache_key = digest_storage_hasher(&params.key);

                        if let Ok(Some(_)) = cache.get(&cache_key).await {
                            return Ok::<_, color_eyre::Report>(());
                        }

                        let processed = processor
                            .process(black_box(&input_buffer), black_box(&params))
                            .await?;

                        let storage_key = format!("processed_{}", params.key);
                        storage.put(&storage_key, &processed).await?;

                        let processed_bytes = processed.into_bytes();
                        cache
                            .set(
                                &cache_key,
                                &processed_bytes,
                                Some(Duration::from_secs(1800)),
                            )
                            .await?;

                        Ok::<_, color_eyre::Report>(())
                    }
                });

                let results = join_all(tasks).await;
                black_box(results)
            })
        })
    }
}

#[divan::bench_group]
mod cache_performance_patterns {
    use super::*;

    #[divan::bench(args = [10, 25, 50, 100])]
    fn cache_hit_ratio_simulation(bencher: Bencher<'_, '_>, num_unique_files: usize) {
        let temp_dir = tempdir().unwrap();
        let cache = FileSystemCache::new(temp_dir.path()).unwrap();

        // Pre-populate cache with some files (simulate 50% cache hit ratio)
        let rt = tokio::runtime::Runtime::new().unwrap();
        for i in 0..num_unique_files / 2 {
            let data = generate_realistic_audio_data(50, AudioFormat::Mp3);
            let key = format!("cached_file_{}", i);
            rt.block_on(async {
                cache
                    .set(&key, &data, Some(Duration::from_secs(3600)))
                    .await
                    .unwrap();
            });
        }

        bencher.bench(|| {
            // Simulate realistic access pattern (80% requests to popular files)
            let file_id = if rand::random::<f32>() < 0.8 {
                rand::random::<usize>() % (num_unique_files / 2) // Popular files (cached)
            } else {
                (num_unique_files / 2) + (rand::random::<usize>() % (num_unique_files / 2)) // Less popular files
            };

            let key = format!("cached_file_{}", file_id);
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let result = cache.get(&key).await;

                if result.is_ok() && result.as_ref().unwrap().is_some() {
                    // Cache hit - just return
                    black_box(result)
                } else {
                    // Cache miss - simulate processing and caching
                    let data = generate_realistic_audio_data(50, AudioFormat::Mp3);
                    cache
                        .set(&key, &data, Some(Duration::from_secs(3600)))
                        .await?;
                    Ok(Some(data))
                }
            })
        })
    }
}

#[divan::bench_group]
mod memory_pressure_scenarios {
    use super::*;

    #[divan::bench(args = [1, 5, 10, 20])]
    fn high_memory_usage_processing(bencher: Bencher<'_, '_>, large_files_mb: usize) {
        let processor = Processor::new(
            ProcessorSettings {
                disabled_filters: Vec::new(),
                max_filter_ops: 50,
                concurrency: Some(1), // Single thread to test memory pressure
                max_cache_files: 100,
                max_cache_mem: 50 * 1024 * 1024, // Reduced cache memory
                max_cache_size: 200 * 1024 * 1024,
            },
            HashMap::new(),
        );

        bencher.bench(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                // Process multiple large files sequentially to test memory management
                for i in 0..3 {
                    let audio_data =
                        generate_realistic_audio_data(large_files_mb * 1024, AudioFormat::Wav);
                    let input_buffer =
                        AudioBuffer::from_bytes_with_format(audio_data, AudioFormat::Wav);

                    let params = Params {
                        key: format!("large_file_{}_{}.wav", large_files_mb, i),
                        format: Some(AudioFormat::Mp3),
                        sample_rate: Some(44100),
                        channels: Some(2),
                        volume: Some(1.0),
                        bit_depth: Some(16),
                        ..Default::default()
                    };

                    let result = processor
                        .process(black_box(&input_buffer), black_box(&params))
                        .await;

                    black_box(result)?;
                }

                Ok::<_, color_eyre::Report>(())
            })
        })
    }
}

#[divan::bench_group]
mod hash_performance_real_world {
    use super::*;

    #[divan::bench(args = [
        "short_track.mp3",
        "artist_name_song_title_featuring_other_artist_remix_version.mp3",
        "podcast/season_01/episode_123_very_long_descriptive_title_with_guest_names.mp3",
        "audiobooks/fantasy/series_name/book_01_chapter_001_the_beginning_of_adventure.mp3",
        "music/rock/band_name/album_name_deluxe_edition/01_track_name_featuring_guest_vocalist.flac"
    ])]
    fn hash_realistic_filenames(filename: &str) -> String {
        let params = Params {
            key: filename.to_string(),
            format: Some(AudioFormat::Mp3),
            sample_rate: Some(44100),
            channels: Some(2),
            volume: Some(1.0),
            normalize: Some(true),
            lowpass: Some(15000.0),
            compressor: Some("3:1:1:-12:0.1:0.1".to_string()),
            ..Default::default()
        };

        black_box(digest_result_storage_hasher(&params))
    }

    #[divan::bench(args = [10, 50, 100, 500, 1000])]
    fn hash_batch_operations(batch_size: usize) -> Vec<String> {
        let base_params = Params {
            format: Some(AudioFormat::Mp3),
            sample_rate: Some(44100),
            channels: Some(2),
            volume: Some(1.0),
            ..Default::default()
        };

        let mut results = Vec::with_capacity(batch_size);

        for i in 0..batch_size {
            let params = Params {
                key: format!("batch_file_{:04}.mp3", i),
                format: base_params.format,
                sample_rate: base_params.sample_rate,
                channels: base_params.channels,
                volume: base_params.volume,
                ..Default::default()
            };

            let hash = digest_result_storage_hasher(&params);
            results.push(hash);
        }

        black_box(results)
    }
}

#[divan::bench_group]
mod error_recovery_patterns {
    use super::*;

    #[divan::bench]
    fn storage_failure_recovery() -> color_eyre::Result<()> {
        let temp_dir = tempdir().unwrap();
        let storage = FileStorage::new(
            temp_dir.path().to_path_buf(),
            "audio".to_string(),
            SafeCharsType::Default,
        );

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            // Try to access non-existent files (common error scenario)
            for i in 0..10 {
                let key = format!("nonexistent_file_{}.mp3", i);
                let result = storage.get(&key).await;
                black_box(result.is_err()); // Should always be an error
            }

            Ok(())
        })
    }

    #[divan::bench]
    fn cache_failure_recovery() -> color_eyre::Result<()> {
        let temp_dir = tempdir().unwrap();
        let cache = FileSystemCache::new(temp_dir.path()).unwrap();

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            // Simulate cache misses and expired entries
            for i in 0..20 {
                let key = format!("cache_test_{}", i);

                // Try to get non-existent key
                let result = cache.get(&key).await;
                black_box(result.is_ok() && result.unwrap().is_none());

                // Set with very short TTL and immediate access
                let data = generate_realistic_audio_data(1, AudioFormat::Mp3);
                cache
                    .set(&key, &data, Some(Duration::from_nanos(1)))
                    .await?;

                // Should be expired by now
                let expired_result = cache.get(&key).await;
                black_box(expired_result.is_ok());
            }

            Ok(())
        })
    }
}

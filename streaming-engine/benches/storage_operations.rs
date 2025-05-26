fn main() {
    divan::main();
}

use divan::black_box;
use std::time::Duration;
use streaming_engine::{
    blob::{AudioBuffer, AudioFormat},
    cache::{cache::AudioCache, fs::FileSystemCache},
    storage::{file::FileStorage, storage::AudioStorage},
    streamingpath::normalize::SafeCharsType,
};
use tempfile::tempdir;

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
            data.extend_from_slice(&16u32.to_le_bytes());
            data.extend_from_slice(&1u16.to_le_bytes());
            data.extend_from_slice(&2u16.to_le_bytes());
            data.extend_from_slice(&44100u32.to_le_bytes());
            data.extend_from_slice(&176400u32.to_le_bytes());
            data.extend_from_slice(&4u16.to_le_bytes());
            data.extend_from_slice(&16u16.to_le_bytes());
            data.extend_from_slice(b"data");
            data.extend_from_slice(&(size_bytes as u32 - 44).to_le_bytes());

            // Fill remaining with audio data
            for i in 0..size_bytes - 44 {
                data.push(((i * 17) % 256) as u8);
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

#[divan::bench_group]
mod file_storage_operations {
    use super::*;

    #[divan::bench(args = [1, 10, 100, 1000])]
    async fn put_operation(size_kb: usize) -> color_eyre::Result<()> {
        let temp_dir = tempdir().unwrap();
        let storage = FileStorage::new(
            temp_dir.path().to_path_buf(),
            "audio".to_string(),
            SafeCharsType::Default,
        );
        let data = generate_audio_data(size_kb, AudioFormat::Mp3);
        let buffer = AudioBuffer::from_bytes(data);

        black_box(storage.put("test.mp3", &buffer).await)
    }

    #[divan::bench(args = [1, 10, 100, 1000])]
    async fn get_operation(size_kb: usize) -> color_eyre::Result<AudioBuffer> {
        let temp_dir = tempdir().unwrap();
        let storage = FileStorage::new(
            temp_dir.path().to_path_buf(),
            "audio".to_string(),
            SafeCharsType::Default,
        );
        let data = generate_audio_data(size_kb, AudioFormat::Mp3);
        let buffer = AudioBuffer::from_bytes(data);
        storage.put("test.mp3", &buffer).await.unwrap();

        black_box(storage.get("test.mp3").await)
    }

    #[divan::bench]
    async fn delete_operation() -> color_eyre::Result<()> {
        let temp_dir = tempdir().unwrap();
        let storage = FileStorage::new(
            temp_dir.path().to_path_buf(),
            "audio".to_string(),
            SafeCharsType::Default,
        );
        let data = generate_audio_data(10, AudioFormat::Mp3);
        let buffer = AudioBuffer::from_bytes(data);
        storage.put("test.mp3", &buffer).await.unwrap();

        black_box(storage.delete("test.mp3").await)
    }

    #[divan::bench(args = ["short", "medium", "long", "with_special_chars", "deeply_nested"])]
    fn get_full_path_operation(key_type: &str) {
        let temp_dir = tempdir().unwrap();
        let storage = FileStorage::new(
            temp_dir.path().to_path_buf(),
            "audio".to_string(),
            SafeCharsType::Default,
        );

        let key = match key_type {
            "short" => "test.mp3",
            "medium" => "audio/music/song.wav",
            "long" => "very/long/path/to/audio/file/with/many/segments/song.flac",
            "with_special_chars" => "file with spaces & symbols.mp3",
            "deeply_nested" => "level1/level2/level3/level4/level5/audio.ogg",
            _ => "default.mp3",
        };

        let _path = storage.get_full_path(black_box(key));
    }
}

#[divan::bench_group]
mod filesystem_cache_operations {
    use super::*;

    #[divan::bench(args = [1, 10, 100, 1000])]
    async fn cache_set_operation(size_kb: usize) -> color_eyre::Result<()> {
        let temp_dir = tempdir().unwrap();
        let cache = FileSystemCache::new(temp_dir.path()).unwrap();
        let data = generate_audio_data(size_kb, AudioFormat::Mp3);

        black_box(cache.set("test_key", &data, None).await)
    }

    #[divan::bench(args = [1, 10, 100, 1000])]
    async fn cache_get_operation(size_kb: usize) -> color_eyre::Result<Option<Vec<u8>>> {
        let temp_dir = tempdir().unwrap();
        let cache = FileSystemCache::new(temp_dir.path()).unwrap();
        let data = generate_audio_data(size_kb, AudioFormat::Mp3);
        cache.set("test_key", &data, None).await.unwrap();

        black_box(cache.get("test_key").await)
    }

    #[divan::bench]
    async fn cache_delete_operation() -> color_eyre::Result<()> {
        let temp_dir = tempdir().unwrap();
        let cache = FileSystemCache::new(temp_dir.path()).unwrap();
        let data = generate_audio_data(10, AudioFormat::Mp3);
        cache.set("test_key", &data, None).await.unwrap();

        black_box(cache.delete("test_key").await)
    }

    #[divan::bench]
    async fn cache_set_with_ttl() -> color_eyre::Result<()> {
        let temp_dir = tempdir().unwrap();
        let cache = FileSystemCache::new(temp_dir.path()).unwrap();
        let data = generate_audio_data(10, AudioFormat::Mp3);
        let ttl = Some(Duration::from_secs(3600)); // 1 hour

        black_box(cache.set("test_key_ttl", &data, ttl).await)
    }

    #[divan::bench]
    async fn cache_get_expired() -> color_eyre::Result<Option<Vec<u8>>> {
        let temp_dir = tempdir().unwrap();
        let cache = FileSystemCache::new(temp_dir.path()).unwrap();
        let data = generate_audio_data(10, AudioFormat::Mp3);
        // Set with very short TTL (1 nanosecond)
        let ttl = Some(Duration::from_nanos(1));
        cache.set("expired_key", &data, ttl).await.unwrap();
        // Wait a bit to ensure expiry
        tokio::time::sleep(Duration::from_millis(1)).await;

        black_box(cache.get("expired_key").await)
    }
}

#[divan::bench_group]
mod storage_vs_cache_comparison {
    use super::*;

    #[divan::bench(args = [10, 100, 1000])]
    async fn storage_write_performance(size_kb: usize) -> color_eyre::Result<()> {
        let temp_dir = tempdir().unwrap();
        let storage = FileStorage::new(
            temp_dir.path().to_path_buf(),
            "audio".to_string(),
            SafeCharsType::Default,
        );
        let data = generate_audio_data(size_kb, AudioFormat::Mp3);
        let buffer = AudioBuffer::from_bytes(data);

        black_box(storage.put("perf_test.mp3", &buffer).await)
    }

    #[divan::bench(args = [10, 100, 1000])]
    async fn cache_write_performance(size_kb: usize) -> color_eyre::Result<()> {
        let temp_dir = tempdir().unwrap();
        let cache = FileSystemCache::new(temp_dir.path()).unwrap();
        let data = generate_audio_data(size_kb, AudioFormat::Mp3);

        black_box(cache.set("perf_test", &data, None).await)
    }

    #[divan::bench(args = [10, 100, 1000])]
    async fn storage_read_performance(size_kb: usize) -> color_eyre::Result<AudioBuffer> {
        let temp_dir = tempdir().unwrap();
        let storage = FileStorage::new(
            temp_dir.path().to_path_buf(),
            "audio".to_string(),
            SafeCharsType::Default,
        );
        let data = generate_audio_data(size_kb, AudioFormat::Mp3);
        let buffer = AudioBuffer::from_bytes(data);
        storage.put("perf_test.mp3", &buffer).await.unwrap();

        black_box(storage.get("perf_test.mp3").await)
    }

    #[divan::bench(args = [10, 100, 1000])]
    async fn cache_read_performance(size_kb: usize) -> color_eyre::Result<Option<Vec<u8>>> {
        let temp_dir = tempdir().unwrap();
        let cache = FileSystemCache::new(temp_dir.path()).unwrap();
        let data = generate_audio_data(size_kb, AudioFormat::Mp3);
        cache.set("perf_test", &data, None).await.unwrap();

        black_box(cache.get("perf_test").await)
    }
}

#[divan::bench_group]
mod path_normalization {
    use super::*;
    use streaming_engine::streamingpath::normalize::{SafeCharsType, normalize};

    #[divan::bench(args = ["simple", "with_spaces", "with_special_chars", "unicode", "mixed_complex"])]
    fn normalize_default(path_type: &str) -> String {
        let path = match path_type {
            "simple" => "test.mp3",
            "with_spaces" => "file with spaces.mp3",
            "with_special_chars" => "file&with=special?chars.mp3",
            "unicode" => "файл_с_unicode_символами.mp3",
            "mixed_complex" => "Artist Name - Song Title (Remix) [2024] & More.mp3",
            _ => "default.mp3",
        };

        black_box(normalize(path, &SafeCharsType::Default))
    }

    #[divan::bench(args = ["simple", "with_spaces", "with_special_chars", "unicode", "mixed_complex"])]
    fn normalize_noop(path_type: &str) -> String {
        let path = match path_type {
            "simple" => "test.mp3",
            "with_spaces" => "file with spaces.mp3",
            "with_special_chars" => "file&with=special?chars.mp3",
            "unicode" => "файл_с_unicode_символами.mp3",
            "mixed_complex" => "Artist Name - Song Title (Remix) [2024] & More.mp3",
            _ => "default.mp3",
        };

        black_box(normalize(path, &SafeCharsType::Noop))
    }

    #[divan::bench(args = [10, 50, 100, 500])]
    fn normalize_long_paths(path_segments: usize) -> String {
        let segment = "very_long_segment_name_with_lots_of_characters";
        let long_path = (0..path_segments)
            .map(|i| format!("{}_{}", segment, i))
            .collect::<Vec<_>>()
            .join("/");
        let final_path = format!("{}/audio_file.mp3", long_path);

        black_box(normalize(&final_path, &SafeCharsType::Default))
    }
}

#[divan::bench_group]
mod error_handling {
    use super::*;

    #[divan::bench]
    async fn storage_get_nonexistent() -> color_eyre::Result<AudioBuffer> {
        let temp_dir = tempdir().unwrap();
        let storage = FileStorage::new(
            temp_dir.path().to_path_buf(),
            "audio".to_string(),
            SafeCharsType::Default,
        );

        black_box(storage.get("nonexistent.mp3").await)
    }

    #[divan::bench]
    async fn storage_delete_nonexistent() -> color_eyre::Result<()> {
        let temp_dir = tempdir().unwrap();
        let storage = FileStorage::new(
            temp_dir.path().to_path_buf(),
            "audio".to_string(),
            SafeCharsType::Default,
        );

        black_box(storage.delete("nonexistent.mp3").await)
    }

    #[divan::bench]
    async fn cache_get_nonexistent() -> color_eyre::Result<Option<Vec<u8>>> {
        let temp_dir = tempdir().unwrap();
        let cache = FileSystemCache::new(temp_dir.path()).unwrap();

        black_box(cache.get("nonexistent_key").await)
    }

    #[divan::bench]
    async fn cache_delete_nonexistent() -> color_eyre::Result<()> {
        let temp_dir = tempdir().unwrap();
        let cache = FileSystemCache::new(temp_dir.path()).unwrap();

        black_box(cache.delete("nonexistent_key").await)
    }
}

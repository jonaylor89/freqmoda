fn main() {
    divan::main();
}

use divan::{Bencher, black_box};
use secrecy::SecretString;
use std::collections::HashMap;
use streaming_engine::{
    blob::AudioFormat,
    streamingpath::{
        hasher::{
            compute_hash, digest_result_storage_hasher, digest_storage_hasher,
            suffix_result_storage_hasher, verify_hash,
        },
        params::Params,
    },
};

// Generate test data for hashing operations
fn generate_test_paths() -> Vec<(&'static str, String)> {
    vec![
        ("short", "test.mp3".to_string()),
        ("medium", "audio/music/song.wav".to_string()),
        ("long", "very/long/path/to/audio/file/with/many/segments/song.flac".to_string()),
        ("with_params", "test.mp3?format=wav&sample_rate=48000&channels=2".to_string()),
        ("complex_params", "audio.flac?format=mp3&sample_rate=44100&channels=2&bit_rate=320&volume=0.8&normalize=true&lowpass=8000&highpass=80&echo=0.6:0.9:55:0.25&fade_in=2.0&fade_out=3.0".to_string()),
        ("url_encoded", "file%20with%20spaces.mp3?artist=Test%20Artist&title=Song%20Title".to_string()),
        ("unicode", "файл_с_unicode.mp3?жанр=электронная".to_string()),
        ("special_chars", "file&with=special?chars.mp3?param1=value&param2=another".to_string()),
    ]
}

fn create_test_params() -> Vec<(&'static str, Params)> {
    vec![
        (
            "simple",
            Params {
                key: "test.mp3".to_string(),
                format: Some(AudioFormat::Mp3),
                ..Default::default()
            },
        ),
        (
            "medium",
            Params {
                key: "audio/music.wav".to_string(),
                format: Some(AudioFormat::Flac),
                sample_rate: Some(48000),
                channels: Some(2),
                volume: Some(1.2),
                lowpass: Some(8000.0),
                ..Default::default()
            },
        ),
        (
            "complex",
            Params {
                key: "path/to/audio.mp3".to_string(),
                format: Some(AudioFormat::Wav),
                sample_rate: Some(96000),
                channels: Some(2),
                bit_depth: Some(24),
                bit_rate: Some(320),
                volume: Some(0.8),
                normalize: Some(true),
                normalize_level: Some(-14.0),
                lowpass: Some(20000.0),
                highpass: Some(20.0),
                bass: Some(1.5),
                treble: Some(0.9),
                echo: Some("0.8:0.88:60:0.4".to_string()),
                chorus: Some("0.5:0.9:50:0.25:0.5:2.0".to_string()),
                compressor: Some("6:1:1:-3:0.1:0.1".to_string()),
                fade_in: Some(2.0),
                fade_out: Some(3.0),
                start_time: Some(30.0),
                duration: Some(120.0),
                speed: Some(1.1),
                tags: Some({
                    let mut tags = HashMap::new();
                    tags.insert("artist".to_string(), "Test Artist".to_string());
                    tags.insert("album".to_string(), "Test Album".to_string());
                    tags.insert("title".to_string(), "Test Song".to_string());
                    tags.insert("year".to_string(), "2024".to_string());
                    tags.insert("genre".to_string(), "Electronic".to_string());
                    tags
                }),
                custom_filters: Some(vec![
                    "volume=0.5".to_string(),
                    "highpass=f=200".to_string(),
                    "lowpass=f=8000".to_string(),
                ]),
                ..Default::default()
            },
        ),
        (
            "with_urls",
            Params {
                key: "https://example.com/audio/file.mp3".to_string(),
                format: Some(AudioFormat::Ogg),
                sample_rate: Some(44100),
                volume: Some(0.7),
                ..Default::default()
            },
        ),
        (
            "large_tags",
            Params {
                key: "audio.flac".to_string(),
                format: Some(AudioFormat::Mp3),
                tags: Some({
                    let mut tags = HashMap::new();
                    for i in 0..50 {
                        tags.insert(
                            format!("tag_key_{}", i),
                            format!("tag_value_with_longer_content_{}", i),
                        );
                    }
                    tags
                }),
                ..Default::default()
            },
        ),
    ]
}

#[divan::bench_group]
mod sha1_hashing {
    use super::*;
    use divan::counter::BytesCount;

    #[divan::bench(counter = BytesCount::of_str("test.mp3"))]
    fn digest_storage_hasher_short() -> String {
        black_box(digest_storage_hasher("test.mp3"))
    }

    #[divan::bench(counter = BytesCount::of_str("audio/music/song.wav"))]
    fn digest_storage_hasher_medium() -> String {
        black_box(digest_storage_hasher("audio/music/song.wav"))
    }

    #[divan::bench(counter = BytesCount::of_str("very/long/path/to/audio/file/with/many/segments/song.flac"))]
    fn digest_storage_hasher_long() -> String {
        black_box(digest_storage_hasher(
            "very/long/path/to/audio/file/with/many/segments/song.flac",
        ))
    }

    #[divan::bench(counter = BytesCount::of_str("test.mp3?format=wav&sample_rate=48000&channels=2"))]
    fn digest_storage_hasher_with_params() -> String {
        black_box(digest_storage_hasher(
            "test.mp3?format=wav&sample_rate=48000&channels=2",
        ))
    }

    #[divan::bench(counter = BytesCount::of_str("audio.flac?format=mp3&sample_rate=44100&channels=2&bit_rate=320&volume=0.8&normalize=true&lowpass=8000&highpass=80&echo=0.6:0.9:55:0.25&fade_in=2.0&fade_out=3.0"))]
    fn digest_storage_hasher_complex_params() -> String {
        black_box(digest_storage_hasher(
            "audio.flac?format=mp3&sample_rate=44100&channels=2&bit_rate=320&volume=0.8&normalize=true&lowpass=8000&highpass=80&echo=0.6:0.9:55:0.25&fade_in=2.0&fade_out=3.0",
        ))
    }

    #[divan::bench(counter = BytesCount::of_str("file%20with%20spaces.mp3?artist=Test%20Artist&title=Song%20Title"))]
    fn digest_storage_hasher_url_encoded() -> String {
        black_box(digest_storage_hasher(
            "file%20with%20spaces.mp3?artist=Test%20Artist&title=Song%20Title",
        ))
    }

    #[divan::bench(counter = BytesCount::of_str("файл_с_unicode.mp3?жанр=электронная"))]
    fn digest_storage_hasher_unicode() -> String {
        black_box(digest_storage_hasher("файл_с_unicode.mp3?жанр=электронная"))
    }

    #[divan::bench(counter = BytesCount::of_str("file&with=special?chars.mp3?param1=value&param2=another"))]
    fn digest_storage_hasher_special_chars() -> String {
        black_box(digest_storage_hasher(
            "file&with=special?chars.mp3?param1=value&param2=another",
        ))
    }
}

#[divan::bench_group]
mod params_hashing {
    use super::*;

    fn create_simple_params() -> Params {
        Params {
            key: "test.mp3".to_string(),
            format: Some(AudioFormat::Mp3),
            ..Default::default()
        }
    }

    fn create_medium_params() -> Params {
        Params {
            key: "audio/music.wav".to_string(),
            format: Some(AudioFormat::Flac),
            sample_rate: Some(48000),
            channels: Some(2),
            volume: Some(1.2),
            lowpass: Some(8000.0),
            ..Default::default()
        }
    }

    fn create_complex_params() -> Params {
        Params {
            key: "path/to/audio.mp3".to_string(),
            format: Some(AudioFormat::Wav),
            sample_rate: Some(96000),
            channels: Some(2),
            bit_depth: Some(24),
            bit_rate: Some(320),
            volume: Some(0.8),
            normalize: Some(true),
            normalize_level: Some(-14.0),
            lowpass: Some(20000.0),
            highpass: Some(20.0),
            bass: Some(1.5),
            treble: Some(0.9),
            echo: Some("0.8:0.88:60:0.4".to_string()),
            chorus: Some("0.5:0.9:50:0.25:0.5:2.0".to_string()),
            compressor: Some("6:1:1:-3:0.1:0.1".to_string()),
            fade_in: Some(2.0),
            fade_out: Some(3.0),
            start_time: Some(30.0),
            duration: Some(120.0),
            speed: Some(1.1),
            tags: Some({
                let mut tags = HashMap::new();
                tags.insert("artist".to_string(), "Test Artist".to_string());
                tags.insert("album".to_string(), "Test Album".to_string());
                tags.insert("title".to_string(), "Test Song".to_string());
                tags.insert("year".to_string(), "2024".to_string());
                tags.insert("genre".to_string(), "Electronic".to_string());
                tags
            }),
            custom_filters: Some(vec![
                "volume=0.5".to_string(),
                "highpass=f=200".to_string(),
                "lowpass=f=8000".to_string(),
            ]),
            ..Default::default()
        }
    }

    #[divan::bench]
    fn digest_result_storage_hasher_simple() -> String {
        let params = create_simple_params();
        black_box(digest_result_storage_hasher(&params))
    }

    #[divan::bench]
    fn digest_result_storage_hasher_medium() -> String {
        let params = create_medium_params();
        black_box(digest_result_storage_hasher(&params))
    }

    #[divan::bench]
    fn digest_result_storage_hasher_complex() -> String {
        let params = create_complex_params();
        black_box(digest_result_storage_hasher(&params))
    }

    #[divan::bench]
    fn suffix_result_storage_hasher_simple() -> String {
        let params = create_simple_params();
        black_box(suffix_result_storage_hasher(&params))
    }

    #[divan::bench]
    fn suffix_result_storage_hasher_medium() -> String {
        let params = create_medium_params();
        black_box(suffix_result_storage_hasher(&params))
    }

    #[divan::bench]
    fn suffix_result_storage_hasher_complex() -> String {
        let params = create_complex_params();
        black_box(suffix_result_storage_hasher(&params))
    }

    #[divan::bench]
    fn suffix_result_storage_hasher_with_urls() -> String {
        let params = Params {
            key: "https://example.com/audio/file.mp3".to_string(),
            format: Some(AudioFormat::Ogg),
            sample_rate: Some(44100),
            volume: Some(0.7),
            ..Default::default()
        };
        black_box(suffix_result_storage_hasher(&params))
    }

    #[divan::bench]
    fn suffix_result_storage_hasher_large_tags() -> String {
        let params = Params {
            key: "audio.flac".to_string(),
            format: Some(AudioFormat::Mp3),
            tags: Some({
                let mut tags = HashMap::new();
                for i in 0..50 {
                    tags.insert(
                        format!("tag_key_{}", i),
                        format!("tag_value_with_longer_content_{}", i),
                    );
                }
                tags
            }),
            ..Default::default()
        };
        black_box(suffix_result_storage_hasher(&params))
    }
}

#[divan::bench_group(sample_count = 10)] // Fewer samples due to expensive operations
mod argon2_operations {
    use super::*;
    use divan::counter::BytesCount;

    #[divan::bench(counter = BytesCount::of_str("test.mp3"))]
    fn compute_hash_short() -> Result<SecretString, color_eyre::eyre::Error> {
        black_box(compute_hash("test.mp3".to_string()))
    }

    #[divan::bench(counter = BytesCount::of_str("path/to/audio/file.wav"))]
    fn compute_hash_medium() -> Result<SecretString, color_eyre::eyre::Error> {
        black_box(compute_hash("path/to/audio/file.wav".to_string()))
    }

    #[divan::bench(counter = BytesCount::of_str("very/long/path/to/audio/file/with/many/segments/and/a/very/long/filename.flac"))]
    fn compute_hash_long() -> Result<SecretString, color_eyre::eyre::Error> {
        black_box(compute_hash(
            "very/long/path/to/audio/file/with/many/segments/and/a/very/long/filename.flac"
                .to_string(),
        ))
    }

    #[divan::bench(counter = BytesCount::of_str("test.mp3"))]
    fn verify_hash_short(bencher: Bencher<'_, '_>) {
        let path = "test.mp3";
        let hash = compute_hash(path.to_string()).unwrap();
        let path_secret = SecretString::from(path.to_string());

        bencher.bench(|| {
            let result = verify_hash(black_box(hash.clone()), black_box(path_secret.clone()));
            black_box(result)
        })
    }

    #[divan::bench(counter = BytesCount::of_str("path/to/audio/file.wav"))]
    fn verify_hash_medium(bencher: Bencher<'_, '_>) {
        let path = "path/to/audio/file.wav";
        let hash = compute_hash(path.to_string()).unwrap();
        let path_secret = SecretString::from(path.to_string());

        bencher.bench(|| {
            let result = verify_hash(black_box(hash.clone()), black_box(path_secret.clone()));
            black_box(result)
        })
    }

    #[divan::bench(counter = BytesCount::of_str("very/long/path/to/audio/file/with/many/segments/and/a/very/long/filename.flac"))]
    fn verify_hash_long(bencher: Bencher<'_, '_>) {
        let path = "very/long/path/to/audio/file/with/many/segments/and/a/very/long/filename.flac";
        let hash = compute_hash(path.to_string()).unwrap();
        let path_secret = SecretString::from(path.to_string());

        bencher.bench(|| {
            let result = verify_hash(black_box(hash.clone()), black_box(path_secret.clone()));
            black_box(result)
        })
    }
}

#[divan::bench_group]
mod hash_consistency {
    use super::*;

    #[divan::bench]
    fn identical_params() -> (String, String, bool) {
        let params1 = Params {
            key: "test.mp3".to_string(),
            format: Some(AudioFormat::Wav),
            sample_rate: Some(48000),
            channels: Some(2),
            ..Default::default()
        };
        let params2 = Params {
            key: params1.key.clone(),
            format: params1.format,
            sample_rate: params1.sample_rate,
            channels: params1.channels,
            ..Default::default()
        };

        let hash1 = suffix_result_storage_hasher(black_box(&params1));
        let hash2 = suffix_result_storage_hasher(black_box(&params2));
        let equal = hash1 == hash2;
        black_box((hash1, hash2, equal))
    }

    #[divan::bench]
    fn different_order_same_content() -> (String, String, bool) {
        let mut params1 = Params {
            key: "test.mp3".to_string(),
            format: Some(AudioFormat::Wav),
            sample_rate: Some(48000),
            channels: Some(2),
            ..Default::default()
        };
        params1.tags = Some({
            let mut tags = HashMap::new();
            tags.insert("a".to_string(), "1".to_string());
            tags.insert("b".to_string(), "2".to_string());
            tags
        });

        let mut params2 = Params {
            key: "test.mp3".to_string(),
            format: Some(AudioFormat::Wav),
            sample_rate: Some(48000),
            channels: Some(2),
            ..Default::default()
        };
        params2.tags = Some({
            let mut tags = HashMap::new();
            tags.insert("b".to_string(), "2".to_string());
            tags.insert("a".to_string(), "1".to_string());
            tags
        });

        let hash1 = suffix_result_storage_hasher(black_box(&params1));
        let hash2 = suffix_result_storage_hasher(black_box(&params2));
        let equal = hash1 == hash2;
        black_box((hash1, hash2, equal))
    }
}

#[divan::bench_group]
mod hash_collision_resistance {
    use super::*;

    #[divan::bench]
    fn minor_difference() -> (String, String, bool) {
        let params1 = Params {
            key: "test.mp3".to_string(),
            volume: Some(1.0),
            ..Default::default()
        };
        let params2 = Params {
            key: "test.mp3".to_string(),
            volume: Some(1.1),
            ..Default::default()
        };

        let hash1 = suffix_result_storage_hasher(black_box(&params1));
        let hash2 = suffix_result_storage_hasher(black_box(&params2));
        let different = hash1 != hash2;
        black_box((hash1, hash2, different))
    }

    #[divan::bench]
    fn key_difference() -> (String, String, bool) {
        let params1 = Params {
            key: "test1.mp3".to_string(),
            format: Some(AudioFormat::Mp3),
            ..Default::default()
        };
        let params2 = Params {
            key: "test2.mp3".to_string(),
            format: Some(AudioFormat::Mp3),
            ..Default::default()
        };

        let hash1 = suffix_result_storage_hasher(black_box(&params1));
        let hash2 = suffix_result_storage_hasher(black_box(&params2));
        let different = hash1 != hash2;
        black_box((hash1, hash2, different))
    }

    #[divan::bench]
    fn format_difference() -> (String, String, bool) {
        let params1 = Params {
            key: "test.mp3".to_string(),
            format: Some(AudioFormat::Mp3),
            ..Default::default()
        };
        let params2 = Params {
            key: "test.mp3".to_string(),
            format: Some(AudioFormat::Wav),
            ..Default::default()
        };

        let hash1 = suffix_result_storage_hasher(black_box(&params1));
        let hash2 = suffix_result_storage_hasher(black_box(&params2));
        let different = hash1 != hash2;
        black_box((hash1, hash2, different))
    }
}

#[divan::bench_group]
mod large_params_hashing {
    use super::*;

    #[divan::bench(args = [10, 100, 1000])]
    fn large_params_hashing(size: usize) -> String {
        let params = create_large_params(size);
        black_box(suffix_result_storage_hasher(&params))
    }

    fn create_large_params(size: usize) -> Params {
        Params {
            key: format!("test_file_{}.mp3", size),
            format: Some(AudioFormat::Flac),
            tags: Some({
                let mut tags = HashMap::new();
                for i in 0..size {
                    tags.insert(
                        format!("tag_key_{:04}", i),
                        format!("tag_value_with_content_number_{:04}", i),
                    );
                }
                tags
            }),
            custom_filters: Some({
                let mut filters = Vec::new();
                for i in 0..size / 10 {
                    filters.push(format!("filter_{}=value_{}", i, i));
                }
                filters
            }),
            ..Default::default()
        }
    }
}

#[divan::bench_group]
mod hash_format_validation {
    use super::*;

    fn create_test_params() -> Params {
        Params {
            key: "test.mp3".to_string(),
            format: Some(AudioFormat::Wav),
            sample_rate: Some(48000),
            ..Default::default()
        }
    }

    #[divan::bench]
    fn validate_digest_format() -> (String, bool) {
        let params = create_test_params();
        let hash = digest_result_storage_hasher(black_box(&params));
        // Validate format: xx/yy/zzz...
        let valid = hash.len() >= 36
            && hash.chars().nth(2) == Some('/')
            && hash.chars().nth(5) == Some('/');
        black_box((hash, valid))
    }

    #[divan::bench]
    fn validate_suffix_format() -> (String, bool) {
        let params = create_test_params();
        let hash = suffix_result_storage_hasher(black_box(&params));
        // Validate format: filename.hash.ext
        let parts: Vec<&str> = hash.split('.').collect();
        let valid = parts.len() >= 2 && parts[parts.len() - 2].len() == 20;
        black_box((hash, valid))
    }
}

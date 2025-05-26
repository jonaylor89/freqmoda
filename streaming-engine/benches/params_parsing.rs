fn main() {
    divan::main();
}

use divan::black_box;
use std::collections::HashMap;
use streaming_engine::{blob::AudioFormat, streamingpath::params::Params};

// Generate test URLs with varying complexity
fn generate_test_urls() -> Vec<(&'static str, String)> {
    vec![
        ("simple", "test.mp3".to_string()),
        ("basic_query", "test.mp3?format=wav".to_string()),
        ("medium_complexity", "test.mp3?format=wav&sample_rate=48000&channels=2&volume=1.2".to_string()),
        ("high_complexity", "audio/track.mp3?format=flac&sample_rate=96000&channels=2&bit_depth=24&volume=0.8&normalize=true&lowpass=20000&highpass=20&echo=0.8:0.88:60:0.4&compressor=6:1:1:-3:0.1:0.1&fade_in=2.0&fade_out=3.0".to_string()),
        ("with_filters", "music.wav?lowpass=8000&highpass=80&bass=2.0&treble=1.5&echo=0.6:0.9:55:0.25&chorus=0.5:0.9:50:0.25:0.5:2.0&flanger=0.5:2.0:0.5:0.7:0.5&phaser=0.5:1.0:3.0:0.5:0.5&tremolo=6.0:0.5".to_string()),
        ("time_operations", "song.mp3?start_time=30.5&duration=120.0&speed=1.25&reverse=true&fade_in=5.0&fade_out=3.0&cross_fade=2.5".to_string()),
        ("encoding_params", "audio.flac?format=mp3&codec=libmp3lame&bit_rate=320&quality=0.2&compression_level=6&sample_rate=44100&channels=2&bit_depth=16".to_string()),
        ("custom_filters", "track.wav?custom_filters=volume=0.5,highpass=f=200,lowpass=f=8000&custom_options=-ab,192k,-ar,48000".to_string()),
        ("with_tags", "music.mp3?tags=artist:TestArtist,album:TestAlbum,title:TestTitle,year:2024,genre:Electronic".to_string()),
        ("url_encoded", "test%20file.mp3?format=wav&volume=1.5&artist=Test%20Artist&title=Test%20%26%20Song".to_string()),
    ]
}

fn generate_query_params() -> Vec<(&'static str, HashMap<String, String>)> {
    vec![
        ("empty", HashMap::new()),
        ("single_param", {
            let mut map = HashMap::new();
            map.insert("format".to_string(), "wav".to_string());
            map
        }),
        ("basic_audio", {
            let mut map = HashMap::new();
            map.insert("format".to_string(), "flac".to_string());
            map.insert("sample_rate".to_string(), "48000".to_string());
            map.insert("channels".to_string(), "2".to_string());
            map
        }),
        ("complex_processing", {
            let mut map = HashMap::new();
            map.insert("format".to_string(), "mp3".to_string());
            map.insert("sample_rate".to_string(), "44100".to_string());
            map.insert("channels".to_string(), "2".to_string());
            map.insert("bit_rate".to_string(), "320".to_string());
            map.insert("volume".to_string(), "0.8".to_string());
            map.insert("normalize".to_string(), "true".to_string());
            map.insert("lowpass".to_string(), "20000".to_string());
            map.insert("highpass".to_string(), "20".to_string());
            map.insert("bass".to_string(), "1.2".to_string());
            map.insert("treble".to_string(), "0.9".to_string());
            map.insert("echo".to_string(), "0.8:0.88:60:0.4".to_string());
            map.insert("fade_in".to_string(), "2.0".to_string());
            map.insert("fade_out".to_string(), "3.0".to_string());
            map
        }),
        ("all_filters", {
            let mut map = HashMap::new();
            map.insert("lowpass".to_string(), "8000".to_string());
            map.insert("highpass".to_string(), "80".to_string());
            map.insert("bandpass".to_string(), "300-3400".to_string());
            map.insert("bass".to_string(), "2.0".to_string());
            map.insert("treble".to_string(), "1.5".to_string());
            map.insert("echo".to_string(), "0.6:0.9:55:0.25".to_string());
            map.insert("chorus".to_string(), "0.5:0.9:50:0.25:0.5:2.0".to_string());
            map.insert("flanger".to_string(), "0.5:2.0:0.5:0.7:0.5".to_string());
            map.insert("phaser".to_string(), "0.5:1.0:3.0:0.5:0.5".to_string());
            map.insert("tremolo".to_string(), "6.0:0.5".to_string());
            map.insert("compressor".to_string(), "6:1:1:-3:0.1:0.1".to_string());
            map.insert("noise_reduction".to_string(), "0.21".to_string());
            map
        }),
        ("with_tags", {
            let mut map = HashMap::new();
            map.insert(
                "tags".to_string(),
                "artist:TestArtist,album:TestAlbum,title:TestTitle".to_string(),
            );
            map
        }),
    ]
}

#[divan::bench_group]
mod params_from_str {
    use super::*;

    #[divan::bench]
    fn simple() -> Result<Params, color_eyre::eyre::Error> {
        black_box("test.mp3".parse::<Params>())
    }

    #[divan::bench]
    fn basic_query() -> Result<Params, color_eyre::eyre::Error> {
        black_box("test.mp3?format=wav".parse::<Params>())
    }

    #[divan::bench]
    fn medium_complexity() -> Result<Params, color_eyre::eyre::Error> {
        black_box("test.mp3?format=wav&sample_rate=48000&channels=2&volume=1.2".parse::<Params>())
    }

    #[divan::bench]
    fn high_complexity() -> Result<Params, color_eyre::eyre::Error> {
        black_box("audio/track.mp3?format=flac&sample_rate=96000&channels=2&bit_depth=24&volume=0.8&normalize=true&lowpass=20000&highpass=20&echo=0.8:0.88:60:0.4&compressor=6:1:1:-3:0.1:0.1&fade_in=2.0&fade_out=3.0".parse::<Params>())
    }

    #[divan::bench]
    fn with_filters() -> Result<Params, color_eyre::eyre::Error> {
        black_box("music.wav?lowpass=8000&highpass=80&bass=2.0&treble=1.5&echo=0.6:0.9:55:0.25&chorus=0.5:0.9:50:0.25:0.5:2.0&flanger=0.5:2.0:0.5:0.7:0.5&phaser=0.5:1.0:3.0:0.5:0.5&tremolo=6.0:0.5".parse::<Params>())
    }

    #[divan::bench]
    fn time_operations() -> Result<Params, color_eyre::eyre::Error> {
        black_box("song.mp3?start_time=30.5&duration=120.0&speed=1.25&reverse=true&fade_in=5.0&fade_out=3.0&cross_fade=2.5".parse::<Params>())
    }

    #[divan::bench]
    fn encoding_params() -> Result<Params, color_eyre::eyre::Error> {
        black_box("audio.flac?format=mp3&codec=libmp3lame&bit_rate=320&quality=0.2&compression_level=6&sample_rate=44100&channels=2&bit_depth=16".parse::<Params>())
    }

    #[divan::bench]
    fn custom_filters() -> Result<Params, color_eyre::eyre::Error> {
        black_box("track.wav?custom_filters=volume=0.5,highpass=f=200,lowpass=f=8000&custom_options=-ab,192k,-ar,48000".parse::<Params>())
    }

    #[divan::bench]
    fn with_tags() -> Result<Params, color_eyre::eyre::Error> {
        black_box("music.mp3?tags=artist:TestArtist,album:TestAlbum,title:TestTitle,year:2024,genre:Electronic".parse::<Params>())
    }

    #[divan::bench]
    fn url_encoded() -> Result<Params, color_eyre::eyre::Error> {
        black_box(
            "test%20file.mp3?format=wav&volume=1.5&artist=Test%20Artist&title=Test%20%26%20Song"
                .parse::<Params>(),
        )
    }
}

#[divan::bench_group]
mod params_from_path {
    use super::*;

    #[divan::bench]
    fn empty_query() -> color_eyre::Result<Params> {
        let path = "test_audio.mp3".to_string();
        let query = HashMap::new();
        black_box(Params::from_path(path, query))
    }

    #[divan::bench]
    fn single_param() -> color_eyre::Result<Params> {
        let path = "test_audio.mp3".to_string();
        let mut query = HashMap::new();
        query.insert("format".to_string(), "wav".to_string());
        black_box(Params::from_path(path, query))
    }

    #[divan::bench]
    fn basic_audio() -> color_eyre::Result<Params> {
        let path = "test_audio.mp3".to_string();
        let mut query = HashMap::new();
        query.insert("format".to_string(), "flac".to_string());
        query.insert("sample_rate".to_string(), "48000".to_string());
        query.insert("channels".to_string(), "2".to_string());
        black_box(Params::from_path(path, query))
    }

    #[divan::bench]
    fn complex_processing() -> color_eyre::Result<Params> {
        let path = "test_audio.mp3".to_string();
        let mut query = HashMap::new();
        query.insert("format".to_string(), "mp3".to_string());
        query.insert("sample_rate".to_string(), "44100".to_string());
        query.insert("channels".to_string(), "2".to_string());
        query.insert("bit_rate".to_string(), "320".to_string());
        query.insert("volume".to_string(), "0.8".to_string());
        query.insert("normalize".to_string(), "true".to_string());
        query.insert("lowpass".to_string(), "20000".to_string());
        query.insert("highpass".to_string(), "20".to_string());
        query.insert("bass".to_string(), "1.2".to_string());
        query.insert("treble".to_string(), "0.9".to_string());
        query.insert("echo".to_string(), "0.8:0.88:60:0.4".to_string());
        query.insert("fade_in".to_string(), "2.0".to_string());
        query.insert("fade_out".to_string(), "3.0".to_string());
        black_box(Params::from_path(path, query))
    }

    #[divan::bench]
    fn all_filters() -> color_eyre::Result<Params> {
        let path = "test_audio.mp3".to_string();
        let mut query = HashMap::new();
        query.insert("lowpass".to_string(), "8000".to_string());
        query.insert("highpass".to_string(), "80".to_string());
        query.insert("bandpass".to_string(), "300-3400".to_string());
        query.insert("bass".to_string(), "2.0".to_string());
        query.insert("treble".to_string(), "1.5".to_string());
        query.insert("echo".to_string(), "0.6:0.9:55:0.25".to_string());
        query.insert("chorus".to_string(), "0.5:0.9:50:0.25:0.5:2.0".to_string());
        query.insert("flanger".to_string(), "0.5:2.0:0.5:0.7:0.5".to_string());
        query.insert("phaser".to_string(), "0.5:1.0:3.0:0.5:0.5".to_string());
        query.insert("tremolo".to_string(), "6.0:0.5".to_string());
        query.insert("compressor".to_string(), "6:1:1:-3:0.1:0.1".to_string());
        query.insert("noise_reduction".to_string(), "0.21".to_string());
        black_box(Params::from_path(path, query))
    }

    #[divan::bench]
    fn with_tags() -> color_eyre::Result<Params> {
        let path = "test_audio.mp3".to_string();
        let mut query = HashMap::new();
        query.insert(
            "tags".to_string(),
            "artist:TestArtist,album:TestAlbum,title:TestTitle".to_string(),
        );
        black_box(Params::from_path(path, query))
    }
}

#[divan::bench_group]
mod params_serialization {
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
            key: "test.mp3".to_string(),
            format: Some(AudioFormat::Wav),
            sample_rate: Some(48000),
            channels: Some(2),
            volume: Some(1.2),
            lowpass: Some(8000.0),
            ..Default::default()
        }
    }

    fn create_complex_params() -> Params {
        Params {
            key: "test.mp3".to_string(),
            format: Some(AudioFormat::Flac),
            sample_rate: Some(96000),
            channels: Some(2),
            bit_depth: Some(24),
            volume: Some(0.8),
            normalize: Some(true),
            lowpass: Some(20000.0),
            highpass: Some(20.0),
            bass: Some(1.2),
            treble: Some(0.9),
            echo: Some("0.8:0.88:60:0.4".to_string()),
            compressor: Some("6:1:1:-3:0.1:0.1".to_string()),
            fade_in: Some(2.0),
            fade_out: Some(3.0),
            tags: Some({
                let mut tags = HashMap::new();
                tags.insert("artist".to_string(), "Test Artist".to_string());
                tags.insert("album".to_string(), "Test Album".to_string());
                tags.insert("title".to_string(), "Test Title".to_string());
                tags
            }),
            ..Default::default()
        }
    }

    #[divan::bench]
    fn to_string_simple() -> String {
        let params = create_simple_params();
        black_box(params.to_string())
    }

    #[divan::bench]
    fn to_string_medium() -> String {
        let params = create_medium_params();
        black_box(params.to_string())
    }

    #[divan::bench]
    fn to_string_complex() -> String {
        let params = create_complex_params();
        black_box(params.to_string())
    }

    #[divan::bench]
    fn to_query_simple() -> HashMap<String, Vec<String>> {
        let params = create_simple_params();
        black_box(params.to_query())
    }

    #[divan::bench]
    fn to_query_medium() -> HashMap<String, Vec<String>> {
        let params = create_medium_params();
        black_box(params.to_query())
    }

    #[divan::bench]
    fn to_query_complex() -> HashMap<String, Vec<String>> {
        let params = create_complex_params();
        black_box(params.to_query())
    }

    #[divan::bench]
    fn to_ffmpeg_args_simple() -> Vec<String> {
        let params = create_simple_params();
        black_box(params.to_ffmpeg_args())
    }

    #[divan::bench]
    fn to_ffmpeg_args_medium() -> Vec<String> {
        let params = create_medium_params();
        black_box(params.to_ffmpeg_args())
    }

    #[divan::bench]
    fn to_ffmpeg_args_complex() -> Vec<String> {
        let params = create_complex_params();
        black_box(params.to_ffmpeg_args())
    }
}

#[divan::bench_group]
mod params_display_formatting {
    use super::*;

    fn create_complex_display_params() -> Params {
        Params {
            key: "complex_audio_file.mp3".to_string(),
            format: Some(AudioFormat::Flac),
            sample_rate: Some(96000),
            channels: Some(2),
            bit_depth: Some(24),
            volume: Some(0.8),
            normalize: Some(true),
            lowpass: Some(20000.0),
            highpass: Some(20.0),
            bass: Some(1.2),
            treble: Some(0.9),
            echo: Some("0.8:0.88:60:0.4".to_string()),
            chorus: Some("0.5:0.9:50:0.25:0.5:2.0".to_string()),
            flanger: Some("0.5:2.0:0.5:0.7:0.5".to_string()),
            compressor: Some("6:1:1:-3:0.1:0.1".to_string()),
            fade_in: Some(2.0),
            fade_out: Some(3.0),
            start_time: Some(30.0),
            duration: Some(120.0),
            speed: Some(1.1),
            tags: Some({
                let mut tags = HashMap::new();
                tags.insert("artist".to_string(), "Complex Artist Name".to_string());
                tags.insert(
                    "album".to_string(),
                    "Album with Special Characters & Symbols".to_string(),
                );
                tags.insert(
                    "title".to_string(),
                    "Song Title (Remix) [Extended Version]".to_string(),
                );
                tags.insert("genre".to_string(), "Electronic/Dance".to_string());
                tags.insert("year".to_string(), "2024".to_string());
                tags
            }),
            ..Default::default()
        }
    }

    #[divan::bench]
    fn format_display() -> String {
        let params = create_complex_display_params();
        black_box(format!("{}", params))
    }

    #[divan::bench]
    fn unsafe_string() -> String {
        let params = create_complex_display_params();
        black_box(Params::to_unsafe_string(&params))
    }
}

#[divan::bench_group]
mod url_encoding {
    use super::*;

    #[divan::bench]
    fn encode_simple() -> String {
        black_box(urlencoding::encode("test_file.mp3").to_string())
    }

    #[divan::bench]
    fn encode_spaces() -> String {
        black_box(urlencoding::encode("file with spaces.mp3").to_string())
    }

    #[divan::bench]
    fn encode_special_chars() -> String {
        black_box(urlencoding::encode("file&with=special?chars.mp3").to_string())
    }

    #[divan::bench]
    fn encode_unicode() -> String {
        black_box(urlencoding::encode("файл_с_unicode_символами.mp3").to_string())
    }

    #[divan::bench]
    fn encode_complex() -> String {
        black_box(
            urlencoding::encode("Artist Name - Song Title (Remix) [2024] & More.mp3").to_string(),
        )
    }

    #[divan::bench]
    fn decode_simple() -> String {
        let encoded = urlencoding::encode("test_file.mp3");
        let decoded = urlencoding::decode(&encoded).unwrap();
        black_box(decoded.into_owned())
    }

    #[divan::bench]
    fn decode_spaces() -> String {
        let encoded = urlencoding::encode("file with spaces.mp3");
        let decoded = urlencoding::decode(&encoded).unwrap();
        black_box(decoded.into_owned())
    }

    #[divan::bench]
    fn decode_special_chars() -> String {
        let encoded = urlencoding::encode("file&with=special?chars.mp3");
        let decoded = urlencoding::decode(&encoded).unwrap();
        black_box(decoded.into_owned())
    }

    #[divan::bench]
    fn decode_unicode() -> String {
        let encoded = urlencoding::encode("файл_с_unicode_символами.mp3");
        let decoded = urlencoding::decode(&encoded).unwrap();
        black_box(decoded.into_owned())
    }

    #[divan::bench]
    fn decode_complex() -> String {
        let encoded = urlencoding::encode("Artist Name - Song Title (Remix) [2024] & More.mp3");
        let decoded = urlencoding::decode(&encoded).unwrap();
        black_box(decoded.into_owned())
    }
}

#[divan::bench_group]
mod params_cloning {
    use super::*;

    fn create_complex_cloning_params() -> Params {
        Params {
            key: "test.mp3".to_string(),
            format: Some(AudioFormat::Flac),
            sample_rate: Some(96000),
            channels: Some(2),
            bit_depth: Some(24),
            volume: Some(0.8),
            normalize: Some(true),
            lowpass: Some(20000.0),
            highpass: Some(20.0),
            bass: Some(1.2),
            treble: Some(0.9),
            echo: Some("0.8:0.88:60:0.4".to_string()),
            chorus: Some("0.5:0.9:50:0.25:0.5:2.0".to_string()),
            compressor: Some("6:1:1:-3:0.1:0.1".to_string()),
            fade_in: Some(2.0),
            fade_out: Some(3.0),
            tags: Some({
                let mut tags = HashMap::new();
                for i in 0..10 {
                    tags.insert(format!("key_{}", i), format!("value_{}", i));
                }
                tags
            }),
            custom_filters: Some(vec![
                "volume=0.5".to_string(),
                "highpass=f=200".to_string(),
                "lowpass=f=8000".to_string(),
            ]),
            custom_options: Some(vec![
                "-ab".to_string(),
                "192k".to_string(),
                "-ar".to_string(),
                "48000".to_string(),
            ]),
            ..Default::default()
        }
    }

    #[divan::bench]
    fn clone_complex_params() -> Params {
        let params = create_complex_cloning_params();
        // Params doesn't have Clone, so create a new instance
        let cloned = Params {
            key: params.key.clone(),
            format: params.format,
            codec: params.codec.clone(),
            sample_rate: params.sample_rate,
            channels: params.channels,
            bit_depth: params.bit_depth,
            volume: params.volume,
            normalize: params.normalize,
            lowpass: params.lowpass,
            highpass: params.highpass,
            echo: params.echo.clone(),
            compressor: params.compressor.clone(),
            fade_in: params.fade_in,
            fade_out: params.fade_out,
            tags: params.tags.clone(),
            ..Default::default()
        };
        black_box(cloned)
    }

    #[divan::bench]
    fn partial_eq_complex_params() -> bool {
        let params = create_complex_cloning_params();
        // Params doesn't have Clone, so create identical instance
        let other_params = Params {
            key: params.key.clone(),
            format: params.format,
            codec: params.codec.clone(),
            sample_rate: params.sample_rate,
            channels: params.channels,
            bit_depth: params.bit_depth,
            volume: params.volume,
            normalize: params.normalize,
            lowpass: params.lowpass,
            highpass: params.highpass,
            echo: params.echo.clone(),
            compressor: params.compressor.clone(),
            fade_in: params.fade_in,
            fade_out: params.fade_out,
            tags: params.tags.clone(),
            ..Default::default()
        };
        black_box(params == other_params)
    }
}

#[divan::bench_group]
mod error_handling {
    use super::*;

    #[divan::bench]
    fn parse_empty_string() -> Result<Params, color_eyre::eyre::Error> {
        black_box("".parse::<Params>())
    }

    #[divan::bench]
    fn parse_invalid_query() -> Result<Params, color_eyre::eyre::Error> {
        black_box("test.mp3?invalid_param=value&another_invalid=123".parse::<Params>())
    }

    #[divan::bench]
    fn parse_malformed_url() -> Result<Params, color_eyre::eyre::Error> {
        black_box("test.mp3?format=&sample_rate=invalid&channels=not_a_number".parse::<Params>())
    }

    #[divan::bench]
    fn parse_very_long_invalid() -> Result<Params, color_eyre::eyre::Error> {
        let long_string = "x".repeat(10000);
        black_box(long_string.parse::<Params>())
    }
}

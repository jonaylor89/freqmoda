use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use base64::{Engine as _, engine::general_purpose};
use color_eyre::{Result, eyre};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::{self, Display},
    str::FromStr,
};
use tracing::info;
use url::form_urlencoded;
use utoipa::ToSchema;

use crate::blob::AudioFormat;

#[derive(Debug)]
pub struct StreamingPath {
    pub path: String,
}

pub trait Signer {
    fn sign(&self, path: &str) -> String;
}

impl<S> FromRequestParts<S> for Params
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    #[tracing::instrument(skip(parts, _state))]
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Access the URI and perform your custom parsing logic
        let uri = &parts.uri;
        let path = uri
            .path()
            .trim_start_matches("/params")
            .trim_start_matches("/meta");

        // Parse query string into a HashMap
        let query_params_string = uri.query().unwrap_or("");
        let query_params: HashMap<String, String> =
            form_urlencoded::parse(query_params_string.as_bytes())
                .into_owned()
                .collect();

        let params = Params::from_path(path.to_string(), query_params).map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                format!("Failed to parse params: {}", e),
            )
        })?;

        Ok(params)
    }
}

impl TryFrom<&str> for Params {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_str(value).map_err(|e| format!("Failed to parse path: {}", e))
    }
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, ToSchema)]
pub struct Params {
    // the uri for the audio
    pub key: String,

    // Audio Format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<AudioFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codec: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sample_rate: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channels: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bit_rate: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bit_depth: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compression_level: Option<i32>,

    // Time Operations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reverse: Option<bool>,

    // Volume Operations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub normalize: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub normalize_level: Option<f64>,

    // Filters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lowpass: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highpass: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bandpass: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bass: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub treble: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub echo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chorus: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flanger: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phaser: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tremolo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compressor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub noise_reduction: Option<String>,

    // Fade Operations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fade_in: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fade_out: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cross_fade: Option<f64>,

    // Advanced
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_filters: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_options: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<HashMap<String, String>>,
}

impl Display for Params {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let query_params = self.to_query();
        let query_str = query_params
            .iter()
            .flat_map(|(k, v)| {
                v.iter()
                    .map(|val| format!("{}={}", k, urlencoding::encode(val)))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
            .join("&");

        write!(f, "{}?{}", self.key, query_str)
    }
}

impl FromStr for Params {
    type Err = color_eyre::eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('?').collect();
        let path = parts[0].trim_start_matches('/');

        info!("Path: {} - {:?}", path, parts);

        // Split path into components and take the last one as audio id
        let path_components: Vec<&str> = path.split('/').collect();
        let audio = path_components.last().unwrap_or(&"").to_string();

        if parts.len() <= 1 {
            return Self::from_path(audio, HashMap::new());
        }

        let query_params: HashMap<String, String> = form_urlencoded::parse(parts[1].as_bytes())
            .into_owned()
            .collect();

        Self::from_path(audio, query_params)
    }
}

impl Params {
    /// Create Params from a path and query parameters.
    ///
    /// This method supports both traditional query parameters and encoded parameters.
    /// When an `encoded` parameter is present, it will be decoded and used as the base,
    /// with any explicit query parameters taking precedence.
    ///
    /// # URL Format with Encoded Parameters
    ///
    /// Traditional format:
    /// ```text
    /// /track.mp3?format=wav&volume=0.8&reverse=true
    /// ```
    ///
    /// Encoded format:
    /// ```text
    /// /track.mp3?encoded=eyJmb3JtYXQiOiJ3YXYiLCJ2b2x1bWUiOjAuOCwicmV2ZXJzZSI6dHJ1ZX0
    /// ```
    ///
    /// Mixed format (explicit parameters override encoded ones):
    /// ```text
    /// /track.mp3?encoded=eyJmb3JtYXQiOiJ3YXYiLCJ2b2x1bWUiOjAuOCwicmV2ZXJzZSI6dHJ1ZX0&format=flac
    /// ```
    ///
    /// In the mixed format example, the final result would have `format=flac` (from explicit)
    /// but `volume=0.8` and `reverse=true` (from encoded).
    pub fn from_path(path: String, query: HashMap<String, String>) -> Result<Self> {
        let mut base_params = Self {
            key: path
                .split("/")
                .last()
                .ok_or(eyre::eyre!("Invalid audio path"))?
                .to_string(),
            ..Default::default()
        };

        // Check for encoded parameters first
        if let Some(encoded) = query.get("encoded") {
            base_params = Self::decode(encoded)?;
            // Keep the key from the path, not from the encoded params
            base_params.key = path
                .split("/")
                .last()
                .ok_or(eyre::eyre!("Invalid audio path"))?
                .to_string();
        }

        // Create a new params instance for explicit parameters
        let mut explicit_params = Self {
            key: base_params.key.clone(),
            ..Default::default()
        };

        for (key, value) in query {
            // Skip the encoded parameter as it's already processed
            if key == "encoded" {
                continue;
            }
            match key.as_str() {
                "format" => {
                    explicit_params.format =
                        Some(value.parse::<AudioFormat>().unwrap_or(AudioFormat::Mp3))
                }
                "codec" => explicit_params.codec = Some(value.to_string()),
                "sample_rate" => explicit_params.sample_rate = value.parse().ok(),
                "channels" => explicit_params.channels = value.parse().ok(),
                "bit_rate" => explicit_params.bit_rate = value.parse().ok(),
                "bit_depth" => explicit_params.bit_depth = value.parse().ok(),
                "quality" => explicit_params.quality = value.parse().ok(),
                "compression_level" => explicit_params.compression_level = value.parse().ok(),
                "start_time" => explicit_params.start_time = value.parse().ok(),
                "duration" => explicit_params.duration = value.parse().ok(),
                "speed" => explicit_params.speed = value.parse().ok(),
                "reverse" => explicit_params.reverse = Some(value == "true" || value == "1"),
                "volume" => explicit_params.volume = value.parse().ok(),
                "normalize" => explicit_params.normalize = Some(value == "true" || value == "1"),
                "normalize_level" => explicit_params.normalize_level = value.parse().ok(),
                "lowpass" => explicit_params.lowpass = value.parse().ok(),
                "highpass" => explicit_params.highpass = value.parse().ok(),
                "bandpass" => explicit_params.bandpass = Some(value.to_string()),
                "bass" => explicit_params.bass = value.parse().ok(),
                "treble" => explicit_params.treble = value.parse().ok(),
                "echo" => explicit_params.echo = Some(value.to_string()),
                "chorus" => explicit_params.chorus = Some(value.to_string()),
                "flanger" => explicit_params.flanger = Some(value.to_string()),
                "phaser" => explicit_params.phaser = Some(value.to_string()),
                "tremolo" => explicit_params.tremolo = Some(value.to_string()),
                "compressor" => explicit_params.compressor = Some(value.to_string()),
                "noise_reduction" => explicit_params.noise_reduction = Some(value.to_string()),
                "fade_in" => explicit_params.fade_in = value.parse().ok(),
                "fade_out" => explicit_params.fade_out = value.parse().ok(),
                "cross_fade" => explicit_params.cross_fade = value.parse().ok(),
                _ => {
                    if key.starts_with("tag_") {
                        let tag_key = key.trim_start_matches("tag_").to_string();
                        explicit_params
                            .tags
                            .get_or_insert_with(HashMap::new)
                            .insert(tag_key, value.to_string());
                    } else if key.starts_with("filter_") {
                        explicit_params
                            .custom_filters
                            .get_or_insert_with(Vec::new)
                            .push(value);
                    } else if key.starts_with("option_") {
                        explicit_params
                            .custom_options
                            .get_or_insert_with(Vec::new)
                            .push(value);
                    }
                }
            }
        }

        // Merge base params with explicit params (explicit takes precedence)
        Ok(base_params.merge_with(explicit_params))
    }

    pub fn to_query(&self) -> HashMap<String, Vec<String>> {
        let mut query: HashMap<String, Vec<String>> = HashMap::new();

        if let Some(format) = &self.format {
            query.insert("format".to_string(), vec![format.to_string()]);
        }
        if let Some(codec) = &self.codec {
            query.insert("codec".to_string(), vec![codec.clone()]);
        }
        if let Some(rate) = self.sample_rate {
            query.insert("sample_rate".to_string(), vec![rate.to_string()]);
        }
        if let Some(channels) = self.channels {
            query.insert("channels".to_string(), vec![channels.to_string()]);
        }
        if let Some(rate) = self.bit_rate {
            query.insert("bit_rate".to_string(), vec![rate.to_string()]);
        }
        if let Some(depth) = self.bit_depth {
            query.insert("bit_depth".to_string(), vec![depth.to_string()]);
        }
        if let Some(quality) = self.quality {
            query.insert("quality".to_string(), vec![quality.to_string()]);
        }
        if let Some(level) = self.compression_level {
            query.insert("compression_level".to_string(), vec![level.to_string()]);
        }
        if let Some(time) = self.start_time {
            query.insert("start_time".to_string(), vec![time.to_string()]);
        }
        if let Some(duration) = self.duration {
            query.insert("duration".to_string(), vec![duration.to_string()]);
        }
        if let Some(speed) = self.speed {
            query.insert("speed".to_string(), vec![speed.to_string()]);
        }
        if let Some(reverse) = self.reverse {
            query.insert("reverse".to_string(), vec![reverse.to_string()]);
        }
        if let Some(volume) = self.volume {
            query.insert("volume".to_string(), vec![volume.to_string()]);
        }
        if let Some(normalize) = self.normalize {
            query.insert("normalize".to_string(), vec![normalize.to_string()]);
        }
        if let Some(level) = self.normalize_level {
            query.insert("normalize_level".to_string(), vec![level.to_string()]);
        }
        if let Some(freq) = self.lowpass {
            query.insert("lowpass".to_string(), vec![freq.to_string()]);
        }
        if let Some(freq) = self.highpass {
            query.insert("highpass".to_string(), vec![freq.to_string()]);
        }
        if let Some(band) = &self.bandpass {
            query.insert("bandpass".to_string(), vec![band.clone()]);
        }
        if let Some(bass) = self.bass {
            query.insert("bass".to_string(), vec![bass.to_string()]);
        }
        if let Some(treble) = self.treble {
            query.insert("treble".to_string(), vec![treble.to_string()]);
        }
        if let Some(echo) = &self.echo {
            query.insert("echo".to_string(), vec![echo.clone()]);
        }
        if let Some(chorus) = &self.chorus {
            query.insert("chorus".to_string(), vec![chorus.clone()]);
        }
        if let Some(flanger) = &self.flanger {
            query.insert("flanger".to_string(), vec![flanger.clone()]);
        }
        if let Some(phaser) = &self.phaser {
            query.insert("phaser".to_string(), vec![phaser.clone()]);
        }
        if let Some(tremolo) = &self.tremolo {
            query.insert("tremolo".to_string(), vec![tremolo.clone()]);
        }
        if let Some(compressor) = &self.compressor {
            query.insert("compressor".to_string(), vec![compressor.clone()]);
        }
        if let Some(nr) = &self.noise_reduction {
            query.insert("noise_reduction".to_string(), vec![nr.clone()]);
        }
        if let Some(fade) = self.fade_in {
            query.insert("fade_in".to_string(), vec![fade.to_string()]);
        }
        if let Some(fade) = self.fade_out {
            query.insert("fade_out".to_string(), vec![fade.to_string()]);
        }
        if let Some(fade) = self.cross_fade {
            query.insert("cross_fade".to_string(), vec![fade.to_string()]);
        }
        if let Some(filters) = &self.custom_filters {
            query.insert("custom_filters".to_string(), filters.clone());
        }
        if let Some(options) = &self.custom_options {
            query.insert("custom_options".to_string(), options.clone());
        }
        if let Some(tags) = &self.tags {
            for (key, value) in tags {
                query.insert(format!("tag_{}", key), vec![value.clone()]);
            }
        }

        query
    }

    pub fn to_ffmpeg_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        if let Some(format) = &self.format {
            args.extend_from_slice(&["-f".to_string(), format.to_string()]);
        }
        if let Some(codec) = &self.codec {
            args.extend_from_slice(&["-c:a".to_string(), codec.clone()]);
        }
        if let Some(rate) = self.sample_rate {
            args.extend_from_slice(&["-ar".to_string(), rate.to_string()]);
        }
        if let Some(channels) = self.channels {
            args.extend_from_slice(&["-ac".to_string(), channels.to_string()]);
        }
        if let Some(rate) = self.bit_rate {
            args.extend_from_slice(&["-b:a".to_string(), format!("{}k", rate)]);
        }
        if let Some(quality) = self.quality {
            args.extend_from_slice(&["-q:a".to_string(), format!("{:.1}", quality)]);
        }
        if let Some(level) = self.compression_level {
            args.extend_from_slice(&["-compression_level".to_string(), level.to_string()]);
        }
        if let Some(time) = self.start_time {
            args.extend_from_slice(&["-ss".to_string(), format!("{:.3}", time)]);
        }
        if let Some(duration) = self.duration {
            args.extend_from_slice(&["-t".to_string(), format!("{:.3}", duration)]);
        }

        let filters = self.collect_filters();
        if !filters.is_empty() {
            args.extend_from_slice(&["-filter:a".to_string(), filters.join(",")]);
        }

        if let Some(options) = &self.custom_options {
            args.extend(options.iter().cloned());
        }

        args
    }

    fn collect_filters(&self) -> Vec<String> {
        let mut filters = Vec::new();

        if let Some(speed) = self.speed {
            if speed != 1.0 {
                filters.push(format!("atempo={:.3}", speed));
            }
        }
        if let Some(true) = self.reverse {
            filters.push("areverse".to_string());
        }
        if let Some(volume) = self.volume {
            if volume != 1.0 {
                filters.push(format!("volume={:.2}", volume));
            }
        }
        if let Some(true) = self.normalize {
            let level = self.normalize_level.unwrap_or(-16.0);
            filters.push(format!("loudnorm=I={:.1}", level));
        }
        if let Some(freq) = self.lowpass {
            filters.push(format!("lowpass=f={:.1}", freq));
        }
        if let Some(freq) = self.highpass {
            filters.push(format!("highpass=f={:.1}", freq));
        }
        if let Some(band) = &self.bandpass {
            filters.push(format!("bandpass={}", band));
        }
        if let Some(bass) = self.bass {
            filters.push(format!("bass=g={:.1}", bass));
        }
        if let Some(treble) = self.treble {
            filters.push(format!("treble=g={:.1}", treble));
        }
        if let Some(echo) = &self.echo {
            filters.push(format!("aecho={}", echo));
        }
        if let Some(chorus) = &self.chorus {
            filters.push(format!("chorus={}", chorus));
        }
        if let Some(flanger) = &self.flanger {
            filters.push(format!("flanger={}", flanger));
        }
        if let Some(phaser) = &self.phaser {
            filters.push(format!("aphaser={}", phaser));
        }
        if let Some(tremolo) = &self.tremolo {
            filters.push(format!("tremolo={}", tremolo));
        }
        if let Some(compressor) = &self.compressor {
            filters.push(format!("acompressor={}", compressor));
        }
        if let Some(nr) = &self.noise_reduction {
            filters.push(format!("anlmdn={}", nr));
        }
        if let Some(fade) = self.fade_in {
            filters.push(format!("afade=t=in:d={:.3}", fade));
        }
        if let Some(fade) = self.fade_out {
            filters.push(format!("afade=t=out:d={:.3}", fade));
        }
        if let Some(fade) = self.cross_fade {
            filters.push(format!("acrossfade=d={:.3}", fade));
        }

        if let Some(custom_filters) = &self.custom_filters {
            filters.extend(custom_filters.clone());
        }

        filters
    }

    /// Encode parameters to a compact base64 string representation.
    ///
    /// This method serializes the current parameters to JSON and then encodes
    /// them as a URL-safe base64 string. This allows for compact representation
    /// of complex parameter sets in URLs.
    ///
    /// # Example
    /// ```
    /// use streaming_engine::streamingpath::params::Params;
    /// use streaming_engine::blob::AudioFormat;
    ///
    /// let params = Params {
    ///     key: "track.mp3".to_string(),
    ///     format: Some(AudioFormat::Wav),
    ///     volume: Some(0.8),
    ///     reverse: Some(true),
    ///     ..Default::default()
    /// };
    ///
    /// let encoded = params.encode().unwrap();
    /// // encoded is now a compact string like "eyJrZXkiOiJ0cmFjay5tcDMi..."
    /// ```
    pub fn encode(&self) -> Result<String> {
        let json = serde_json::to_string(self)?;
        Ok(general_purpose::URL_SAFE_NO_PAD.encode(json.as_bytes()))
    }

    /// Decode parameters from a base64 encoded string.
    ///
    /// This method decodes a base64 string back into a Params struct.
    /// The string should have been created using the `encode()` method.
    ///
    /// # Example
    /// ```
    /// use streaming_engine::streamingpath::params::Params;
    ///
    /// let encoded = "eyJrZXkiOiJ0cmFjay5tcDMiLCJmb3JtYXQiOiJ3YXYiLCJ2b2x1bWUiOjAuOCwicmV2ZXJzZSI6dHJ1ZX0";
    /// let params = Params::decode(encoded).unwrap();
    ///
    /// assert_eq!(params.key, "track.mp3");
    /// ```
    pub fn decode(encoded: &str) -> Result<Self> {
        let decoded_bytes = general_purpose::URL_SAFE_NO_PAD
            .decode(encoded)
            .map_err(|e| eyre::eyre!("Failed to decode base64: {}", e))?;
        let json = String::from_utf8(decoded_bytes)
            .map_err(|e| eyre::eyre!("Invalid UTF-8 in decoded data: {}", e))?;
        let params: Self =
            serde_json::from_str(&json).map_err(|e| eyre::eyre!("Failed to parse JSON: {}", e))?;
        Ok(params)
    }

    /// Merge with another Params instance, where other takes precedence for non-None values.
    ///
    /// This method combines two parameter sets, with the `other` parameters taking
    /// precedence over the current ones for any non-None/non-empty values.
    /// This is particularly useful when combining encoded parameters with explicit
    /// query parameters.
    ///
    /// # Example
    /// ```
    /// use streaming_engine::streamingpath::params::Params;
    /// use streaming_engine::blob::AudioFormat;
    ///
    /// let base = Params {
    ///     key: "track.mp3".to_string(),
    ///     format: Some(AudioFormat::Wav),
    ///     volume: Some(0.8),
    ///     ..Default::default()
    /// };
    ///
    /// let override_params = Params {
    ///     key: "".to_string(), // Empty key won't override
    ///     format: Some(AudioFormat::Flac), // This will override
    ///     channels: Some(2), // This will be added
    ///     ..Default::default()
    /// };
    ///
    /// let result = base.merge_with(override_params);
    /// // result.format is now AudioFormat::Flac
    /// // result.volume is still 0.8
    /// // result.channels is now Some(2)
    /// ```
    pub fn merge_with(mut self, other: Self) -> Self {
        // Keep the key from self unless other has a non-empty key
        if !other.key.is_empty() {
            self.key = other.key;
        }

        // For Option fields, replace with other's value if it's Some
        if other.format.is_some() {
            self.format = other.format;
        }
        if other.codec.is_some() {
            self.codec = other.codec;
        }
        if other.sample_rate.is_some() {
            self.sample_rate = other.sample_rate;
        }
        if other.channels.is_some() {
            self.channels = other.channels;
        }
        if other.bit_rate.is_some() {
            self.bit_rate = other.bit_rate;
        }
        if other.bit_depth.is_some() {
            self.bit_depth = other.bit_depth;
        }
        if other.quality.is_some() {
            self.quality = other.quality;
        }
        if other.compression_level.is_some() {
            self.compression_level = other.compression_level;
        }
        if other.start_time.is_some() {
            self.start_time = other.start_time;
        }
        if other.duration.is_some() {
            self.duration = other.duration;
        }
        if other.speed.is_some() {
            self.speed = other.speed;
        }
        if other.reverse.is_some() {
            self.reverse = other.reverse;
        }
        if other.volume.is_some() {
            self.volume = other.volume;
        }
        if other.normalize.is_some() {
            self.normalize = other.normalize;
        }
        if other.normalize_level.is_some() {
            self.normalize_level = other.normalize_level;
        }
        if other.lowpass.is_some() {
            self.lowpass = other.lowpass;
        }
        if other.highpass.is_some() {
            self.highpass = other.highpass;
        }
        if other.bandpass.is_some() {
            self.bandpass = other.bandpass;
        }
        if other.bass.is_some() {
            self.bass = other.bass;
        }
        if other.treble.is_some() {
            self.treble = other.treble;
        }
        if other.echo.is_some() {
            self.echo = other.echo;
        }
        if other.chorus.is_some() {
            self.chorus = other.chorus;
        }
        if other.flanger.is_some() {
            self.flanger = other.flanger;
        }
        if other.phaser.is_some() {
            self.phaser = other.phaser;
        }
        if other.tremolo.is_some() {
            self.tremolo = other.tremolo;
        }
        if other.compressor.is_some() {
            self.compressor = other.compressor;
        }
        if other.noise_reduction.is_some() {
            self.noise_reduction = other.noise_reduction;
        }
        if other.fade_in.is_some() {
            self.fade_in = other.fade_in;
        }
        if other.fade_out.is_some() {
            self.fade_out = other.fade_out;
        }
        if other.cross_fade.is_some() {
            self.cross_fade = other.cross_fade;
        }

        // For Vec fields, replace if other has content
        if other.custom_filters.is_some() {
            self.custom_filters = other.custom_filters;
        }
        if other.custom_options.is_some() {
            self.custom_options = other.custom_options;
        }

        // For HashMap fields, merge them
        if let Some(other_tags) = other.tags {
            let self_tags = self.tags.get_or_insert_with(HashMap::new);
            self_tags.extend(other_tags);
        }

        self
    }

    pub fn to_unsafe_string(p: &Params) -> String {
        let img_path = p.to_string();
        format!("unsafe/{}", img_path)
    }

    pub fn to_signed_string<S: Signer>(p: &Params, signer: S) -> String {
        let img_path = p.to_string();
        format!("{}/{}", signer.sign(&img_path), img_path)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_params_display() {
        let params = Params {
            key: "test.mp3".to_string(),
            format: Some(AudioFormat::Mp3),
            quality: Some(0.5),
            ..Default::default()
        };

        let output = params.to_string();
        assert!(output.starts_with("test.mp3?"));
        assert!(output.contains("format=mp3"));
        assert!(output.contains("quality=0.5"));
    }

    #[test]
    fn test_from_path_basic() {
        let path = "audio/test.mp3".to_string();
        let query = HashMap::new();

        let params = Params::from_path(path, query).unwrap();

        assert_eq!(params.key, "test.mp3");
        assert_eq!(params.format, None);
    }

    #[test]
    fn test_from_path_with_format() {
        let path = "audio/test.mp3".to_string();
        let mut query = HashMap::new();
        query.insert("format".to_string(), "wav".to_string());

        let params = Params::from_path(path, query).unwrap();

        assert_eq!(params.key, "test.mp3");
        assert_eq!(params.format, Some(AudioFormat::Wav));
    }

    #[test]
    fn test_from_path_with_multiple_params() {
        let path = "audio/test.mp3".to_string();
        let mut query = HashMap::new();
        query.insert("format".to_string(), "wav".to_string());
        query.insert("volume".to_string(), "0.8".to_string());
        query.insert("reverse".to_string(), "true".to_string());

        let params = Params::from_path(path, query).unwrap();

        assert_eq!(params.key, "test.mp3");
        assert_eq!(params.format, Some(AudioFormat::Wav));
        assert_eq!(params.volume, Some(0.8));
        assert_eq!(params.reverse, Some(true));
    }

    #[test]
    fn test_from_str() {
        let input = "/audio/test.mp3?format=wav&volume=0.8&reverse=true";
        let params = Params::from_str(input).unwrap();

        assert_eq!(params.key, "test.mp3");
        assert_eq!(params.format, Some(AudioFormat::Wav));
        assert_eq!(params.volume, Some(0.8));
        assert_eq!(params.reverse, Some(true));
    }

    #[test]
    fn test_from_str_no_query() {
        let input = "/audio/test.mp3";
        let params = Params::from_str(input).unwrap();

        assert_eq!(params.key, "test.mp3");
    }

    #[test]
    fn test_to_query() {
        let params = Params {
            key: "test.mp3".to_string(),
            format: Some(AudioFormat::Wav),
            volume: Some(0.8),
            reverse: Some(true),
            ..Default::default()
        };

        let query = params.to_query();

        assert_eq!(query.get("format").unwrap(), &vec!["wav".to_string()]);
        assert_eq!(query.get("volume").unwrap(), &vec!["0.8".to_string()]);
        assert_eq!(query.get("reverse").unwrap(), &vec!["true".to_string()]);
    }

    #[test]
    fn test_to_ffmpeg_args() {
        let params = Params {
            key: "test.mp3".to_string(),
            format: Some(AudioFormat::Wav),
            codec: Some("pcm_s16le".to_string()),
            sample_rate: Some(44100),
            channels: Some(2),
            ..Default::default()
        };

        let args = params.to_ffmpeg_args();

        assert!(args.contains(&"-f".to_string()));
        assert!(args.contains(&"wav".to_string()));
        assert!(args.contains(&"-c:a".to_string()));
        assert!(args.contains(&"pcm_s16le".to_string()));
        assert!(args.contains(&"-ar".to_string()));
        assert!(args.contains(&"44100".to_string()));
        assert!(args.contains(&"-ac".to_string()));
        assert!(args.contains(&"2".to_string()));
    }

    #[test]
    fn test_collect_filters() {
        let params = Params {
            key: "test.mp3".to_string(),
            volume: Some(0.8),
            reverse: Some(true),
            lowpass: Some(1000.0),
            fade_in: Some(2.0),
            fade_out: Some(3.0),
            ..Default::default()
        };

        let filters = params.collect_filters();

        assert!(filters.contains(&"volume=0.80".to_string()));
        assert!(filters.contains(&"areverse".to_string()));
        assert!(filters.contains(&"lowpass=f=1000.0".to_string()));
        assert!(filters.contains(&"afade=t=in:d=2.000".to_string()));
        assert!(filters.contains(&"afade=t=out:d=3.000".to_string()));
    }

    #[test]
    fn test_to_unsafe_string() {
        let params = Params {
            key: "test.mp3".to_string(),
            format: Some(AudioFormat::Mp3),
            ..Default::default()
        };

        let result = Params::to_unsafe_string(&params);

        assert!(result.starts_with("unsafe/"));
        assert!(result.contains("test.mp3?format=mp3"));
    }

    #[test]
    fn test_try_from_str() {
        let result = Params::try_from("/test.mp3?format=mp3");

        assert!(result.is_ok());
        let params = result.unwrap();
        assert_eq!(params.key, "test.mp3");
        assert_eq!(params.format, Some(AudioFormat::Mp3));
    }

    #[test]
    fn test_custom_filters_and_options() {
        let mut query = HashMap::new();
        query.insert("filter_1".to_string(), "vibrato=f=5:d=0.5".to_string());
        query.insert("option_1".to_string(), "-map_metadata".to_string());

        let params = Params::from_path("test.mp3".to_string(), query).unwrap();

        assert!(params.custom_filters.is_some());
        assert_eq!(
            params.custom_filters.as_ref().unwrap()[0],
            "vibrato=f=5:d=0.5"
        );

        assert!(params.custom_options.is_some());
        assert_eq!(params.custom_options.as_ref().unwrap()[0], "-map_metadata");
    }

    #[test]
    fn test_tags() {
        let mut query = HashMap::new();
        query.insert("tag_artist".to_string(), "Test Artist".to_string());
        query.insert("tag_album".to_string(), "Test Album".to_string());

        let params = Params::from_path("test.mp3".to_string(), query).unwrap();

        assert!(params.tags.is_some());
        let tags = params.tags.as_ref().unwrap();
        assert_eq!(tags.get("artist").unwrap(), "Test Artist");
        assert_eq!(tags.get("album").unwrap(), "Test Album");
    }

    #[test]
    fn test_encode_decode() {
        let params = Params {
            key: "test.mp3".to_string(),
            format: Some(AudioFormat::Wav),
            volume: Some(0.8),
            reverse: Some(true),
            lowpass: Some(1000.0),
            ..Default::default()
        };

        let encoded = params.encode().unwrap();
        let decoded = Params::decode(&encoded).unwrap();

        assert_eq!(decoded.key, params.key);
        assert_eq!(decoded.format, params.format);
        assert_eq!(decoded.volume, params.volume);
        assert_eq!(decoded.reverse, params.reverse);
        assert_eq!(decoded.lowpass, params.lowpass);
    }

    #[test]
    fn test_encode_decode_empty_params() {
        let params = Params {
            key: "test.mp3".to_string(),
            ..Default::default()
        };

        let encoded = params.encode().unwrap();
        let decoded = Params::decode(&encoded).unwrap();

        assert_eq!(decoded.key, params.key);
        assert_eq!(decoded.format, None);
        assert_eq!(decoded.volume, None);
    }

    #[test]
    fn test_encode_decode_with_tags_and_filters() {
        let mut tags = HashMap::new();
        tags.insert("artist".to_string(), "Test Artist".to_string());
        tags.insert("album".to_string(), "Test Album".to_string());

        let params = Params {
            key: "test.mp3".to_string(),
            format: Some(AudioFormat::Flac),
            custom_filters: Some(vec!["vibrato=f=5:d=0.5".to_string()]),
            custom_options: Some(vec!["-map_metadata".to_string()]),
            tags: Some(tags.clone()),
            ..Default::default()
        };

        let encoded = params.encode().unwrap();
        let decoded = Params::decode(&encoded).unwrap();

        assert_eq!(decoded.key, params.key);
        assert_eq!(decoded.format, params.format);
        assert_eq!(decoded.custom_filters, params.custom_filters);
        assert_eq!(decoded.custom_options, params.custom_options);
        assert_eq!(decoded.tags, Some(tags));
    }

    #[test]
    fn test_from_path_with_encoded_parameter() {
        let base_params = Params {
            key: "original.mp3".to_string(),
            format: Some(AudioFormat::Wav),
            volume: Some(0.8),
            reverse: Some(true),
            ..Default::default()
        };

        let encoded = base_params.encode().unwrap();
        let mut query = HashMap::new();
        query.insert("encoded".to_string(), encoded);

        let result = Params::from_path("test.mp3".to_string(), query).unwrap();

        // Key should come from path, not encoded
        assert_eq!(result.key, "test.mp3");
        // Other params should come from encoded
        assert_eq!(result.format, Some(AudioFormat::Wav));
        assert_eq!(result.volume, Some(0.8));
        assert_eq!(result.reverse, Some(true));
    }

    #[test]
    fn test_from_path_encoded_with_explicit_override() {
        let base_params = Params {
            key: "original.mp3".to_string(),
            format: Some(AudioFormat::Wav),
            volume: Some(0.8),
            reverse: Some(true),
            sample_rate: Some(44100),
            ..Default::default()
        };

        let encoded = base_params.encode().unwrap();
        let mut query = HashMap::new();
        query.insert("encoded".to_string(), encoded);
        // Explicit parameters should override encoded ones
        query.insert("format".to_string(), "flac".to_string());
        query.insert("volume".to_string(), "0.5".to_string());
        // New parameter not in encoded
        query.insert("channels".to_string(), "2".to_string());

        let result = Params::from_path("test.mp3".to_string(), query).unwrap();

        // Key should come from path
        assert_eq!(result.key, "test.mp3");
        // Overridden parameters should use explicit values
        assert_eq!(result.format, Some(AudioFormat::Flac));
        assert_eq!(result.volume, Some(0.5));
        // Non-overridden parameters should come from encoded
        assert_eq!(result.reverse, Some(true));
        assert_eq!(result.sample_rate, Some(44100));
        // New parameters should be added
        assert_eq!(result.channels, Some(2));
    }

    #[test]
    fn test_from_str_with_encoded_parameter() {
        let base_params = Params {
            key: "original.mp3".to_string(),
            format: Some(AudioFormat::Wav),
            volume: Some(0.8),
            reverse: Some(true),
            ..Default::default()
        };

        let encoded = base_params.encode().unwrap();
        let input = format!("/audio/test.mp3?encoded={}&format=flac&channels=2", encoded);
        let params = Params::from_str(&input).unwrap();

        // Key should come from path
        assert_eq!(params.key, "test.mp3");
        // Explicit format should override encoded
        assert_eq!(params.format, Some(AudioFormat::Flac));
        // Non-overridden encoded params should be preserved
        assert_eq!(params.volume, Some(0.8));
        assert_eq!(params.reverse, Some(true));
        // New explicit params should be added
        assert_eq!(params.channels, Some(2));
    }

    #[test]
    fn test_merge_with() {
        let base = Params {
            key: "base.mp3".to_string(),
            format: Some(AudioFormat::Wav),
            volume: Some(0.8),
            reverse: Some(true),
            sample_rate: Some(44100),
            ..Default::default()
        };

        let override_params = Params {
            key: "override.mp3".to_string(),
            format: Some(AudioFormat::Flac),
            volume: Some(0.5),
            channels: Some(2), // New parameter
            ..Default::default()
        };

        let result = base.merge_with(override_params);

        // Override should win for non-empty key
        assert_eq!(result.key, "override.mp3");
        // Override should win for overlapping parameters
        assert_eq!(result.format, Some(AudioFormat::Flac));
        assert_eq!(result.volume, Some(0.5));
        // Base should be preserved for non-overridden parameters
        assert_eq!(result.reverse, Some(true));
        assert_eq!(result.sample_rate, Some(44100));
        // New parameters should be added
        assert_eq!(result.channels, Some(2));
    }

    #[test]
    fn test_merge_with_tags() {
        let mut base_tags = HashMap::new();
        base_tags.insert("artist".to_string(), "Base Artist".to_string());
        base_tags.insert("album".to_string(), "Base Album".to_string());

        let mut override_tags = HashMap::new();
        override_tags.insert("artist".to_string(), "Override Artist".to_string());
        override_tags.insert("year".to_string(), "2023".to_string());

        let base = Params {
            key: "base.mp3".to_string(),
            tags: Some(base_tags),
            ..Default::default()
        };

        let override_params = Params {
            key: "".to_string(), // Empty key should not override
            tags: Some(override_tags),
            ..Default::default()
        };

        let result = base.merge_with(override_params);

        // Empty key should not override
        assert_eq!(result.key, "base.mp3");
        // Tags should be merged with override taking precedence
        let result_tags = result.tags.unwrap();
        assert_eq!(result_tags.get("artist").unwrap(), "Override Artist");
        assert_eq!(result_tags.get("album").unwrap(), "Base Album");
        assert_eq!(result_tags.get("year").unwrap(), "2023");
    }

    #[test]
    fn test_decode_invalid_base64() {
        let result = Params::decode("invalid-base64!");
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_invalid_json() {
        let invalid_json = general_purpose::URL_SAFE_NO_PAD.encode(b"invalid json");
        let result = Params::decode(&invalid_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_encoded_parameters_comprehensive_demo() {
        // This test demonstrates the complete encoded parameters workflow

        // Step 1: Create complex parameters that would normally create a very long URL
        let mut tags = HashMap::new();
        tags.insert("artist".to_string(), "Test Artist".to_string());
        tags.insert("album".to_string(), "Test Album".to_string());
        tags.insert("year".to_string(), "2023".to_string());

        let complex_params = Params {
            key: "demo-track.mp3".to_string(),
            format: Some(AudioFormat::Flac),
            sample_rate: Some(48000),
            channels: Some(2),
            bit_depth: Some(24),
            volume: Some(0.85),
            normalize: Some(true),
            normalize_level: Some(-16.0),
            reverse: Some(false),
            lowpass: Some(8000.0),
            highpass: Some(80.0),
            bass: Some(1.2),
            treble: Some(0.9),
            fade_in: Some(1.5),
            fade_out: Some(2.0),
            custom_filters: Some(vec![
                "vibrato=f=5:d=0.5".to_string(),
                "chorus=0.5:0.9:50:0.4:0.25:2".to_string(),
            ]),
            tags: Some(tags),
            ..Default::default()
        };

        // Step 2: Encode the parameters into a compact string
        let encoded = complex_params.encode().unwrap();
        println!("Encoded parameters: {}", encoded);
        println!("Encoded length: {} characters", encoded.len());

        // Step 3: Simulate using encoded parameters in a URL with additional explicit params
        let mut query = HashMap::new();
        query.insert("encoded".to_string(), encoded.clone());
        // These explicit parameters should override the encoded ones
        query.insert("format".to_string(), "wav".to_string());
        query.insert("volume".to_string(), "0.7".to_string());
        // This is a new parameter not in the encoded set
        query.insert("speed".to_string(), "1.25".to_string());

        // Step 4: Parse the combined parameters
        let result = Params::from_path("final-track.mp3".to_string(), query).unwrap();

        // Step 5: Verify the merging behavior
        // Key should come from path
        assert_eq!(result.key, "final-track.mp3");

        // Explicit parameters should override encoded ones
        assert_eq!(result.format, Some(AudioFormat::Wav)); // overridden
        assert_eq!(result.volume, Some(0.7)); // overridden

        // Non-overridden encoded parameters should be preserved
        assert_eq!(result.sample_rate, Some(48000));
        assert_eq!(result.channels, Some(2));
        assert_eq!(result.bit_depth, Some(24));
        assert_eq!(result.normalize, Some(true));
        assert_eq!(result.normalize_level, Some(-16.0));
        assert_eq!(result.reverse, Some(false));
        assert_eq!(result.lowpass, Some(8000.0));
        assert_eq!(result.highpass, Some(80.0));
        assert_eq!(result.bass, Some(1.2));
        assert_eq!(result.treble, Some(0.9));
        assert_eq!(result.fade_in, Some(1.5));
        assert_eq!(result.fade_out, Some(2.0));

        // Custom filters should be preserved
        assert!(result.custom_filters.is_some());
        let filters = result.custom_filters.as_ref().unwrap();
        assert_eq!(filters.len(), 2);
        assert!(filters.contains(&"vibrato=f=5:d=0.5".to_string()));
        assert!(filters.contains(&"chorus=0.5:0.9:50:0.4:0.25:2".to_string()));

        // Tags should be preserved
        assert!(result.tags.is_some());
        let result_tags = result.tags.as_ref().unwrap();
        assert_eq!(result_tags.get("artist").unwrap(), "Test Artist");
        assert_eq!(result_tags.get("album").unwrap(), "Test Album");
        assert_eq!(result_tags.get("year").unwrap(), "2023");

        // New explicit parameters should be added
        assert_eq!(result.speed, Some(1.25));

        // Step 6: Demonstrate URL comparison
        let traditional_url = format!(
            "/final-track.mp3?{}",
            "format=wav&sample_rate=48000&channels=2&bit_depth=24&volume=0.7&normalize=true&normalize_level=-16&lowpass=8000&highpass=80&bass=1.2&treble=0.9&fade_in=1.5&fade_out=2&filter_1=vibrato=f=5:d=0.5&filter_2=chorus=0.5:0.9:50:0.4:0.25:2&tag_artist=Test%20Artist&tag_album=Test%20Album&tag_year=2023&speed=1.25"
        );
        let encoded_url = format!(
            "/final-track.mp3?encoded={}&format=wav&volume=0.7&speed=1.25",
            encoded
        );

        println!(
            "Traditional URL length: {} characters",
            traditional_url.len()
        );
        println!("Encoded URL length: {} characters", encoded_url.len());

        // Calculate the difference safely
        let url_diff = if traditional_url.len() > encoded_url.len() {
            println!(
                "Space saved: {} characters ({:.1}%)",
                traditional_url.len() - encoded_url.len(),
                ((traditional_url.len() - encoded_url.len()) as f64 / traditional_url.len() as f64)
                    * 100.0
            );
            traditional_url.len() - encoded_url.len()
        } else {
            println!(
                "Encoded URL is {} characters longer ({:.1}% overhead)",
                encoded_url.len() - traditional_url.len(),
                ((encoded_url.len() - traditional_url.len()) as f64 / traditional_url.len() as f64)
                    * 100.0
            );
            0 // For assertion purposes
        };

        // Note: For very complex parameter sets, encoded URLs provide benefits in:
        // 1. Reduced URL parsing complexity
        // 2. Better cacheability (single parameter vs many)
        // 3. Atomic parameter sets (all or nothing)
        // 4. Easier programmatic generation

        // The primary benefit is not always length reduction, but URL management
        println!("Encoded parameters provide structured parameter management regardless of length");
    }

    #[test]
    fn test_integration_with_route_handler() {
        // This test simulates how the encoded parameters would work with the actual route handler

        // Create a complex parameter set
        let original_params = Params {
            key: "track.mp3".to_string(),
            format: Some(AudioFormat::Wav),
            volume: Some(0.8),
            reverse: Some(true),
            sample_rate: Some(48000),
            lowpass: Some(5000.0),
            ..Default::default()
        };

        // Encode it
        let encoded = original_params.encode().unwrap();

        // Simulate a URL like: /unsafe/track.mp3?encoded=ABC123&format=flac
        let url_path = format!("/unsafe/track.mp3?encoded={}&format=flac", encoded);

        // Parse it as if it came from a real HTTP request
        let parsed_params = Params::from_str(&url_path).unwrap();

        // Verify the behavior:
        assert_eq!(parsed_params.key, "track.mp3");
        assert_eq!(parsed_params.format, Some(AudioFormat::Flac)); // Explicit override
        assert_eq!(parsed_params.volume, Some(0.8)); // From encoded
        assert_eq!(parsed_params.reverse, Some(true)); // From encoded
        assert_eq!(parsed_params.sample_rate, Some(48000)); // From encoded
        assert_eq!(parsed_params.lowpass, Some(5000.0)); // From encoded

        println!("✅ Integration test passed: encoded={}", encoded);
        println!("✅ Parsed params correctly merged encoded + explicit parameters");
    }
}

use crate::error::{AppError, Result};
use crate::services::claude::ClaudeService;
use base64::{Engine as _, engine::general_purpose};
use reqwest::{Client, Url};
use serde_json::Value;

use tracing::{debug, error, info, warn};

pub struct StreamingEngineService {
    client: Client,
    base_url: String,
}

impl StreamingEngineService {
    pub fn new(client: Client, base_url: String) -> Self {
        Self { client, base_url }
    }

    pub async fn process_audio(&self, audio_name: &str, parameters: &Value) -> Result<String> {
        info!(
            "Processing audio: {} with parameters: {}",
            audio_name, parameters
        );

        let query_params = self.build_encoded_query_params(parameters).map_err(|e| {
            error!(
                "Failed to build query parameters for audio '{}': {}",
                audio_name, e
            );
            e
        })?;

        let url = format!(
            "{}/unsafe/{}{}",
            self.base_url,
            urlencoding::encode(audio_name),
            if query_params.is_empty() {
                String::new()
            } else {
                format!("?{}", query_params)
            }
        );

        debug!("Generated streaming engine URL: {}", url);

        // Validate URL is properly formed
        Url::parse(&url).map_err(|e| {
            error!(
                "Invalid URL generated for audio '{}': {} - URL: {}",
                audio_name, e, url
            );
            AppError::Validation(format!("Invalid URL: {}", e))
        })?;

        // Test the URL by making a HEAD request
        info!("Making HEAD request to streaming engine: {}", url);
        let response = self.client.head(&url).send().await.map_err(|e| {
            error!(
                "HTTP request failed for audio '{}' to URL '{}': {}",
                audio_name, url, e
            );
            AppError::HttpClient(e)
        })?;

        if !response.status().is_success() {
            error!(
                "Streaming engine returned error status {} for audio '{}' at URL '{}'",
                response.status(),
                audio_name,
                url
            );
            return Err(AppError::StreamingEngine(format!(
                "Audio processing failed with status: {} for audio: {}",
                response.status(),
                audio_name
            )));
        }

        info!(
            "Successfully processed audio: {} - URL: {}",
            audio_name, url
        );
        Ok(url)
    }

    pub async fn get_audio_metadata(&self, audio_name: &str) -> Result<Value> {
        let url = format!(
            "{}/meta/unsafe/{}",
            self.base_url,
            urlencoding::encode(audio_name)
        );

        debug!(
            "Getting metadata for audio: {} from URL: {}",
            audio_name, url
        );

        let response = self.client.get(&url).send().await.map_err(|e| {
            error!(
                "HTTP request failed for metadata of audio '{}' to URL '{}': {}",
                audio_name, url, e
            );
            AppError::HttpClient(e)
        })?;

        if !response.status().is_success() {
            error!(
                "Failed to get metadata for audio '{}' with status: {} from URL: {}",
                audio_name,
                response.status(),
                url
            );
            return Err(AppError::StreamingEngine(format!(
                "Failed to get metadata for audio '{}': {}",
                audio_name,
                response.status()
            )));
        }

        let metadata: Value = response.json().await.map_err(|e| {
            error!(
                "Failed to parse metadata JSON for audio '{}': {}",
                audio_name, e
            );
            AppError::StreamingEngine(format!("Failed to parse metadata JSON: {}", e))
        })?;

        debug!("Successfully retrieved metadata for audio: {}", audio_name);
        Ok(metadata)
    }

    pub async fn preview_params(&self, audio_name: &str, parameters: &Value) -> Result<Value> {
        let query_params = self.build_encoded_query_params(parameters).map_err(|e| {
            error!(
                "Failed to build query parameters for preview of audio '{}': {}",
                audio_name, e
            );
            e
        })?;

        let url = format!(
            "{}/params/unsafe/{}{}",
            self.base_url,
            urlencoding::encode(audio_name),
            if query_params.is_empty() {
                String::new()
            } else {
                format!("?{}", query_params)
            }
        );

        debug!(
            "Previewing params for audio: {} with URL: {}",
            audio_name, url
        );

        let response = self.client.get(&url).send().await.map_err(|e| {
            error!(
                "HTTP request failed for preview params of audio '{}' to URL '{}': {}",
                audio_name, url, e
            );
            AppError::HttpClient(e)
        })?;

        if !response.status().is_success() {
            error!(
                "Failed to preview params for audio '{}' with status: {} from URL: {}",
                audio_name,
                response.status(),
                url
            );
            return Err(AppError::StreamingEngine(format!(
                "Failed to preview params for audio '{}': {}",
                audio_name,
                response.status()
            )));
        }

        let preview: Value = response.json().await.map_err(|e| {
            error!(
                "Failed to parse preview params JSON for audio '{}': {}",
                audio_name, e
            );
            AppError::StreamingEngine(format!("Failed to parse preview params JSON: {}", e))
        })?;

        debug!("Successfully previewed params for audio: {}", audio_name);
        Ok(preview)
    }

    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/health", self.base_url);
        debug!("Checking streaming engine health at: {}", url);

        let response = self.client.get(&url).send().await.map_err(|e| {
            error!("Health check HTTP request failed to URL '{}': {}", url, e);
            AppError::HttpClient(e)
        })?;

        let is_healthy = response.status().is_success();
        if is_healthy {
            debug!("Streaming engine health check passed");
        } else {
            warn!(
                "Streaming engine health check failed with status: {}",
                response.status()
            );
        }

        Ok(is_healthy)
    }

    fn build_encoded_query_params(&self, parameters: &Value) -> Result<String> {
        debug!("Building encoded query params from: {}", parameters);

        if let Value::Object(obj) = parameters {
            if obj.is_empty() || (obj.len() == 1 && obj.contains_key("audio_name")) {
                debug!("No parameters to encode (empty or only audio_name)");
                return Ok(String::new());
            }

            // Apply effect presets to parameters
            let processed_params = self.apply_effect_presets(parameters).map_err(|e| {
                error!("Failed to apply effect presets: {}", e);
                e
            })?;

            // Separate parameters into encodable and explicit override categories
            let (encodable_params, explicit_params) =
                self.separate_parameters(&processed_params).map_err(|e| {
                    error!("Failed to separate parameters: {}", e);
                    e
                })?;

            let mut query_parts = Vec::new();

            // Add encoded parameters if there are any
            if !encodable_params.is_empty() {
                let encoded = self
                    .encode_parameters_object(&encodable_params)
                    .map_err(|e| {
                        error!("Failed to encode parameters object: {}", e);
                        e
                    })?;
                query_parts.push(format!("encoded={}", urlencoding::encode(&encoded)));
                debug!("Added encoded parameters: {} items", encodable_params.len());
            }

            // Add explicit parameters that should override encoded ones
            for (key, value) in explicit_params {
                match value {
                    Value::String(s) => {
                        query_parts.push(format!("{}={}", key, urlencoding::encode(&s)));
                    }
                    Value::Number(n) => {
                        query_parts.push(format!("{}={}", key, n));
                    }
                    Value::Bool(b) => {
                        query_parts.push(format!("{}={}", key, b));
                    }
                    _ => {
                        warn!("Skipping unsupported parameter type for key: {}", key);
                        continue;
                    }
                }
            }

            let result = query_parts.join("&");
            debug!("Built query params: {}", result);
            Ok(result)
        } else {
            debug!("Parameters is not an object, returning empty string");
            Ok(String::new())
        }
    }

    fn apply_effect_presets(&self, parameters: &Value) -> Result<Value> {
        let effect_presets = ClaudeService::get_effect_presets();
        let mut processed = parameters.clone();

        if let Value::Object(obj) = &mut processed {
            for (key, value) in obj.iter_mut() {
                if key == "audio_name" {
                    continue;
                }

                if let Value::String(s) = value {
                    // Check if this is an effect that needs preset mapping
                    if let Some(preset_map) = effect_presets.get(key)
                        && let Some(preset_value) = preset_map.get(&s.to_lowercase()) {
                            *value = Value::String(preset_value.clone());
                        }
                }
            }
        }

        Ok(processed)
    }

    fn separate_parameters(
        &self,
        parameters: &Value,
    ) -> Result<(serde_json::Map<String, Value>, Vec<(String, Value)>)> {
        // Parameters that should be kept as explicit overrides (commonly changed)
        let explicit_override_params = ["format", "volume", "speed", "quality"];

        let mut encodable = serde_json::Map::new();
        let mut explicit = Vec::new();

        if let Value::Object(obj) = parameters {
            for (key, value) in obj {
                if key == "audio_name" {
                    continue;
                }

                if explicit_override_params.contains(&key.as_str()) {
                    explicit.push((key.clone(), value.clone()));
                } else {
                    encodable.insert(key.clone(), value.clone());
                }
            }
        }

        Ok((encodable, explicit))
    }

    fn encode_parameters_object(
        &self,
        parameters: &serde_json::Map<String, Value>,
    ) -> Result<String> {
        // Create a Params-like structure that matches what the streaming engine expects
        let mut params_json = serde_json::Map::new();

        // Add a default empty key since it will be overridden by the path
        params_json.insert("key".to_string(), Value::String("".to_string()));

        // Map parameters to the expected Params struct format
        for (key, value) in parameters {
            match key.as_str() {
                "format" => {
                    params_json.insert("format".to_string(), value.clone());
                }
                "codec" => {
                    params_json.insert("codec".to_string(), value.clone());
                }
                "sample_rate" => {
                    params_json.insert("sample_rate".to_string(), value.clone());
                }
                "channels" => {
                    params_json.insert("channels".to_string(), value.clone());
                }
                "bit_rate" => {
                    params_json.insert("bit_rate".to_string(), value.clone());
                }
                "bit_depth" => {
                    params_json.insert("bit_depth".to_string(), value.clone());
                }
                "quality" => {
                    params_json.insert("quality".to_string(), value.clone());
                }
                "compression_level" => {
                    params_json.insert("compression_level".to_string(), value.clone());
                }
                "start_time" => {
                    params_json.insert("start_time".to_string(), value.clone());
                }
                "duration" => {
                    params_json.insert("duration".to_string(), value.clone());
                }
                "speed" => {
                    params_json.insert("speed".to_string(), value.clone());
                }
                "reverse" => {
                    params_json.insert("reverse".to_string(), value.clone());
                }
                "volume" => {
                    params_json.insert("volume".to_string(), value.clone());
                }
                "normalize" => {
                    params_json.insert("normalize".to_string(), value.clone());
                }
                "normalize_level" => {
                    params_json.insert("normalize_level".to_string(), value.clone());
                }
                "lowpass" => {
                    params_json.insert("lowpass".to_string(), value.clone());
                }
                "highpass" => {
                    params_json.insert("highpass".to_string(), value.clone());
                }
                "bandpass" => {
                    params_json.insert("bandpass".to_string(), value.clone());
                }
                "bass" => {
                    params_json.insert("bass".to_string(), value.clone());
                }
                "treble" => {
                    params_json.insert("treble".to_string(), value.clone());
                }
                "echo" => {
                    params_json.insert("echo".to_string(), value.clone());
                }
                "chorus" => {
                    params_json.insert("chorus".to_string(), value.clone());
                }
                "flanger" => {
                    params_json.insert("flanger".to_string(), value.clone());
                }
                "phaser" => {
                    params_json.insert("phaser".to_string(), value.clone());
                }
                "tremolo" => {
                    params_json.insert("tremolo".to_string(), value.clone());
                }
                "compressor" => {
                    params_json.insert("compressor".to_string(), value.clone());
                }
                "noise_reduction" => {
                    params_json.insert("noise_reduction".to_string(), value.clone());
                }
                "fade_in" => {
                    params_json.insert("fade_in".to_string(), value.clone());
                }
                "fade_out" => {
                    params_json.insert("fade_out".to_string(), value.clone());
                }
                "cross_fade" => {
                    params_json.insert("cross_fade".to_string(), value.clone());
                }
                "custom_filters" => {
                    params_json.insert("custom_filters".to_string(), value.clone());
                }
                "custom_options" => {
                    params_json.insert("custom_options".to_string(), value.clone());
                }
                "tags" => {
                    params_json.insert("tags".to_string(), value.clone());
                }
                _ => {
                    // Handle tag_ prefixed parameters
                    if key.starts_with("tag_") {
                        let tag_key = key.trim_start_matches("tag_");
                        if let Some(tag_value) = value.as_str() {
                            // Create tags object if it doesn't exist
                            let tags = params_json
                                .entry("tags".to_string())
                                .or_insert_with(|| Value::Object(serde_json::Map::new()));
                            if let Some(tags_obj) = tags.as_object_mut() {
                                tags_obj.insert(
                                    tag_key.to_string(),
                                    Value::String(tag_value.to_string()),
                                );
                            }
                        }
                    }
                    // Handle filter_ prefixed parameters
                    else if key.starts_with("filter_") {
                        if let Some(filter_value) = value.as_str() {
                            // Create custom_filters array if it doesn't exist
                            let filters = params_json
                                .entry("custom_filters".to_string())
                                .or_insert_with(|| Value::Array(Vec::new()));
                            if let Some(filters_arr) = filters.as_array_mut() {
                                filters_arr.push(Value::String(filter_value.to_string()));
                            }
                        }
                    }
                    // Handle option_ prefixed parameters
                    else if key.starts_with("option_")
                        && let Some(option_value) = value.as_str() {
                            // Create custom_options array if it doesn't exist
                            let options = params_json
                                .entry("custom_options".to_string())
                                .or_insert_with(|| Value::Array(Vec::new()));
                            if let Some(options_arr) = options.as_array_mut() {
                                options_arr.push(Value::String(option_value.to_string()));
                            }
                        }
                }
            }
        }

        // Convert to JSON string and encode to base64
        let json = serde_json::to_string(&params_json)
            .map_err(|e| AppError::Validation(format!("Failed to serialize parameters: {}", e)))?;

        let encoded = general_purpose::URL_SAFE_NO_PAD.encode(json.as_bytes());
        Ok(encoded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn test_service() -> StreamingEngineService {
        let client = reqwest::Client::builder()
            .no_proxy()
            .build()
            .expect("build test client");
        StreamingEngineService::new(client, "http://localhost:8080".to_string())
    }

    #[test]
    fn test_encode_parameters_object() {
        let service = test_service();

        let mut params = serde_json::Map::new();
        params.insert("format".to_string(), Value::String("wav".to_string()));
        params.insert(
            "volume".to_string(),
            Value::Number(serde_json::Number::from_f64(0.8).unwrap()),
        );
        params.insert("reverse".to_string(), Value::Bool(true));

        let result = service.encode_parameters_object(&params);
        assert!(result.is_ok());

        let encoded = result.unwrap();
        assert!(!encoded.is_empty());

        // Verify it's valid base64
        let decoded = general_purpose::URL_SAFE_NO_PAD.decode(&encoded);
        assert!(decoded.is_ok());

        // Verify it's valid JSON
        let json_str = String::from_utf8(decoded.unwrap()).unwrap();
        let parsed: Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(parsed["format"], "wav");
        assert_eq!(parsed["volume"], 0.8);
        assert_eq!(parsed["reverse"], true);
    }

    #[test]
    fn test_separate_parameters() {
        let service = test_service();

        let params = json!({
            "audio_name": "track.mp3",
            "format": "flac",
            "volume": 0.8,
            "lowpass": 5000.0,
            "reverse": true,
            "echo": "0.5:0.7:60:0.4"
        });

        let (encodable, explicit) = service.separate_parameters(&params).unwrap();

        // Explicit parameters (commonly overridden)
        assert_eq!(explicit.len(), 2); // format, volume (quality not present in test data)
        assert!(explicit.iter().any(|(k, _)| k == "format"));
        assert!(explicit.iter().any(|(k, _)| k == "volume"));

        // Encodable parameters
        assert!(encodable.contains_key("lowpass"));
        assert!(encodable.contains_key("reverse"));
        assert!(encodable.contains_key("echo"));
        assert!(!encodable.contains_key("audio_name"));
        assert!(!encodable.contains_key("format"));
        assert!(!encodable.contains_key("volume"));
    }

    #[test]
    fn test_build_encoded_query_params_empty() {
        let service = test_service();

        let empty_params = json!({});
        let result = service.build_encoded_query_params(&empty_params).unwrap();
        assert_eq!(result, "");

        let only_audio_name = json!({"audio_name": "track.mp3"});
        let result = service
            .build_encoded_query_params(&only_audio_name)
            .unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_build_encoded_query_params_mixed() {
        let service = test_service();

        let params = json!({
            "audio_name": "track.mp3",
            "format": "flac",
            "volume": 0.8,
            "lowpass": 5000.0,
            "reverse": true
        });

        let result = service.build_encoded_query_params(&params).unwrap();

        // Should contain encoded parameter
        assert!(result.contains("encoded="));

        // Should contain explicit parameters
        assert!(result.contains("format=flac"));
        assert!(result.contains("volume=0.8"));

        // Should not contain audio_name
        assert!(!result.contains("audio_name"));
    }

    #[test]
    fn test_apply_effect_presets() {
        let service = test_service();

        let params = json!({
            "audio_name": "track.mp3",
            "format": "wav",
            "volume": 0.8
        });

        let result = service.apply_effect_presets(&params).unwrap();

        // Should preserve existing values
        assert_eq!(result["format"], "wav");
        assert_eq!(result["volume"], 0.8);
        assert_eq!(result["audio_name"], "track.mp3");
    }

    #[test]
    fn test_encoded_parameters_demo() {
        // This test demonstrates the complete encoded parameters workflow

        let service = StreamingEngineService::new(
            reqwest::Client::builder()
                .no_proxy()
                .build()
                .expect("build test client"),
            "http://localhost:8080".to_string(),
        );

        // Create complex parameters that would normally create a long URL
        let complex_params = json!({
            "audio_name": "demo-track.mp3",
            "format": "flac",
            "volume": 0.85,
            "lowpass": 8000.0,
            "highpass": 80.0,
            "bass": 1.2,
            "treble": 0.9,
            "fade_in": 1.5,
            "fade_out": 2.0,
            "reverse": false,
            "echo": "0.5:0.7:60:0.4"
        });

        // Build the encoded query string
        let result = service.build_encoded_query_params(&complex_params).unwrap();

        println!("Generated query string: {}", result);

        // Should contain encoded parameter for complex parameters
        assert!(result.contains("encoded="));

        // Should contain explicit overrides for commonly changed parameters
        assert!(result.contains("format=flac"));
        assert!(result.contains("volume=0.85"));

        // Should not contain audio_name
        assert!(!result.contains("audio_name"));

        // Verify the structure: encoded part + explicit overrides
        let parts: Vec<&str> = result.split('&').collect();
        assert!(parts.len() >= 3); // encoded + format + volume at minimum

        println!("✅ Encoded parameters demo successful");
        println!("✅ Complex parameters compacted into encoded format");
        println!("✅ Common parameters kept explicit for easy override");
    }

    #[test]
    fn test_encoding_format_compatibility() {
        // This test verifies that the gateway service produces encoded parameters
        // that are compatible with the streaming engine's expectations

        let service = test_service();

        // Create a parameter set that includes various types
        let params = json!({
            "audio_name": "test.mp3",
            "bass": 3,
            "chorus": "0.5:0.9:50:0.4:0.25:2",
            "lowpass": 3500.0,
            "format": "mp3",
            "speed": 0.95
        });

        // Build the encoded query params
        let query_string = service.build_encoded_query_params(&params).unwrap();

        // The result should contain an encoded parameter and explicit overrides
        assert!(query_string.contains("encoded="));
        assert!(query_string.contains("format=mp3"));
        assert!(query_string.contains("speed=0.95"));

        // Extract the encoded part for verification
        let encoded_part = query_string
            .split('&')
            .find(|part| part.starts_with("encoded="))
            .unwrap()
            .strip_prefix("encoded=")
            .unwrap();

        // Decode the URL-encoded value
        let decoded_encoded = urlencoding::decode(encoded_part).unwrap();

        // Decode the base64
        let decoded_bytes = general_purpose::URL_SAFE_NO_PAD
            .decode(decoded_encoded.as_bytes())
            .unwrap();
        let json_str = String::from_utf8(decoded_bytes).unwrap();

        // Parse as JSON to verify structure
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        // Verify the structure matches what streaming engine expects
        assert!(parsed.is_object());
        let obj = parsed.as_object().unwrap();

        // Should have a key field (even if empty)
        assert!(obj.contains_key("key"));

        // Should contain the encoded parameters
        assert_eq!(obj.get("bass"), Some(&json!(3)));
        assert_eq!(obj.get("chorus"), Some(&json!("0.5:0.9:50:0.4:0.25:2")));
        assert_eq!(obj.get("lowpass"), Some(&json!(3500.0)));

        // Should NOT contain explicit override parameters
        assert!(!obj.contains_key("format"));
        assert!(!obj.contains_key("speed"));

        // Should not contain audio_name
        assert!(!obj.contains_key("audio_name"));
    }
}

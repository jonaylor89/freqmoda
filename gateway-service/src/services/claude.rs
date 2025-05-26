use crate::error::{AppError, Result};
use crate::models::{ClaudeMessage, ClaudeRequest, ClaudeResponse, ClaudeTool};
use reqwest::Client;
use secrecy::{ExposeSecret, SecretString};
use serde_json::json;
use std::collections::HashMap;

pub struct ClaudeService {
    client: Client,
    api_key: SecretString,
    base_url: String,
    model: String,
}

const SYSTEM_PROMPT: &str = "You are audio processing assistant. This program is an innovative AI-powered audio editing platform that allows users to manipulate audio files through natural language conversations. Your role is to help users process, edit, and transform audio samples using various effects, filters, and transformations. You can work with the audio samples in the library or with audio files users specify. Always be helpful, creative, and focus on delivering high-quality audio processing results. When users ask about audio editing, processing, or effects, use your available tools to fulfill their requests.

Available samples:
- Sample 1 (sample1.mp3): 32.9s
- Sample 2 (sample2.mp3): 32.9s
- Sample 3 (sample3.mp3): 32.9s
- Sample 4 (sample4.mp3): 32.9s
- Sample 5 (sample5.mp3): 130.6s
- Sample 6 (sample6.mp3): 32.9s
- Sample 7 (sample7.mp3): 32.9s
- Sample 8 (sample8.mp3): 32.9s

ABSOLUTELY CRITICAL RULES - NO EXCEPTIONS:
1. When processing audio, you MUST use the process_audio tool first
2. After process_audio returns a URL, you MUST immediately call format_response tool
3. You are FORBIDDEN from responding with any text containing URLs outside of the format_response tool
4. You are FORBIDDEN from using phrases like 'Processed audio:', 'Here's the result:', etc.
5. If you process audio, your response must ONLY come from the format_response tool
6. DO NOT provide any conversational text after calling format_response - that tool provides the complete response
7. Any response containing a processed audio URL that doesn't use format_response tool is an ERROR";

impl ClaudeService {
    pub fn new(client: Client, api_key: SecretString, base_url: String, model: String) -> Self {
        Self {
            client,
            api_key,
            base_url,
            model,
        }
    }

    pub async fn send_message(
        &self,
        messages: Vec<ClaudeMessage>,
        tools: Option<Vec<ClaudeTool>>,
    ) -> Result<ClaudeResponse> {
        let tools = tools.unwrap_or_else(|| self.get_default_tools());

        let request = ClaudeRequest {
            model: self.model.clone(),
            max_tokens: 1000,
            messages,
            system: Some(SYSTEM_PROMPT.to_string()),
            tools,
        };

        let response = self
            .client
            .post(format!("{}/v1/messages", self.base_url))
            .header("x-api-key", self.api_key.expose_secret())
            .header("Content-Type", "application/json")
            .header("anthropic-version", "2023-06-01")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::Claude(format!(
                "Claude API error: {} - {}",
                status, error_text
            )));
        }

        let claude_response: ClaudeResponse = response.json().await?;
        Ok(claude_response)
    }

    fn get_default_tools(&self) -> Vec<ClaudeTool> {
        vec![
            ClaudeTool {
                name: "process_audio".to_string(),
                description: "Process audio with various effects and transformations".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "audio_name": {
                            "type": "string",
                            "description": "URL/URI/filename to audio file or sample name like 'Sample 1'"
                        },
                        "format": {
                            "type": "string",
                            "description": "Output format (mp3, wav, etc.)",
                            "enum": ["mp3", "wav", "flac", "ogg", "m4a"]
                        },
                        "start_time": {
                            "type": "number",
                            "description": "Start time in seconds"
                        },
                        "duration": {
                            "type": "number",
                            "description": "Duration in seconds"
                        },
                        "speed": {
                            "type": "number",
                            "description": "Playback speed multiplier (e.g., 0.5 = half speed, 2.0 = double speed)"
                        },
                        "reverse": {
                            "type": "boolean",
                            "description": "Reverse the audio"
                        },
                        "volume": {
                            "type": "number",
                            "description": "Volume adjustment multiplier (1.0 = original, 0.5 = half volume, 2.0 = double volume)"
                        },
                        "normalize": {
                            "type": "boolean",
                            "description": "Normalize audio levels"
                        },
                        "normalize_level": {
                            "type": "number",
                            "description": "Target normalization level in dB"
                        },
                        "lowpass": {
                            "type": "number",
                            "description": "Lowpass filter cutoff frequency in Hz"
                        },
                        "highpass": {
                            "type": "number",
                            "description": "Highpass filter cutoff frequency in Hz"
                        },
                        "bass": {
                            "type": "number",
                            "description": "Bass boost/cut level in dB"
                        },
                        "treble": {
                            "type": "number",
                            "description": "Treble boost/cut level in dB"
                        },
                        "fade_in": {
                            "type": "number",
                            "description": "Fade in duration in seconds"
                        },
                        "fade_out": {
                            "type": "number",
                            "description": "Fade out duration in seconds (important: the number of seconds is relative to the BEGINNING of the track so a fade_out: 2 means the track will fade the song out over the FIRST 2 seconds of the track. this is counter intuitive)"
                        },
                        "echo": {
                            "type": "string",
                            "description": "Echo effect - use simple values like 'light', 'medium', or 'heavy'"
                        },
                        "chorus": {
                            "type": "string",
                            "description": "Chorus effect - use simple values like 'light', 'medium', or 'heavy'"
                        },
                        "flanger": {
                            "type": "string",
                            "description": "Flanger effect - use simple values like 'light', 'medium', or 'heavy'"
                        }
                    },
                    "required": ["audio_name"]
                }),
            },
            ClaudeTool {
                name: "list_audio_samples".to_string(),
                description: "List all available audio samples in the library".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            ClaudeTool {
                name: "format_response".to_string(),
                description: "Format the final response with processed audio URL. You MUST call this tool to provide your final response when you have processed audio.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "description": {
                            "type": "string",
                            "description": "Your descriptive text about what you did to the audio"
                        },
                        "sample_url": {
                            "type": "string",
                            "description": "The processed audio sample URL"
                        }
                    },
                    "required": ["description", "sample_url"]
                }),
            },
        ]
    }

    pub fn get_effect_presets() -> HashMap<String, HashMap<String, String>> {
        let mut presets = HashMap::new();

        let mut echo_presets = HashMap::new();
        echo_presets.insert("light".to_string(), "0.6:0.3:1000:0.3".to_string());
        echo_presets.insert("medium".to_string(), "0.8:0.88:60:0.4".to_string());
        echo_presets.insert("heavy".to_string(), "0.8:0.9:1000:0.5".to_string());
        presets.insert("echo".to_string(), echo_presets);

        let mut chorus_presets = HashMap::new();
        chorus_presets.insert("light".to_string(), "0.5:0.9:50:0.4:0.25:2".to_string());
        chorus_presets.insert("medium".to_string(), "0.7:0.9:50:0.4:0.25:2".to_string());
        chorus_presets.insert("heavy".to_string(), "0.9:0.9:50:0.4:0.25:2".to_string());
        presets.insert("chorus".to_string(), chorus_presets);

        let mut flanger_presets = HashMap::new();
        flanger_presets.insert("light".to_string(), "0.5:0.75:2:0.25:2".to_string());
        flanger_presets.insert("medium".to_string(), "0.7:0.75:3:0.25:2".to_string());
        flanger_presets.insert("heavy".to_string(), "0.9:0.75:4:0.25:2".to_string());
        presets.insert("flanger".to_string(), flanger_presets);

        presets
    }
}

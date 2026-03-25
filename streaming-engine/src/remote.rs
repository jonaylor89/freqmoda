use axum::http::StatusCode;
use reqwest::Client;
use serde::Deserialize;
use tracing::warn;

use crate::blob::AudioBuffer;

const AUDIOUS_API_BASE_URL: &str = "https://api.audius.co/v1";

pub async fn fetch_audio_buffer(source: &str) -> Result<AudioBuffer, (StatusCode, String)> {
    let client = Client::new();
    let resolved = resolve_remote_source(&client, source).await?;

    let raw_bytes = client
        .get(&resolved)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch audio from URL {}: {}", resolved, e);
            (
                StatusCode::NOT_FOUND,
                format!("Failed to fetch audio: {}", e),
            )
        })?
        .bytes()
        .await
        .map_err(|e| {
            tracing::error!("Failed to read bytes from URL {}: {}", resolved, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to fetch audio: {}", e),
            )
        })?
        .to_vec();

    Ok(AudioBuffer::from_bytes(raw_bytes))
}

async fn resolve_remote_source(
    client: &Client,
    source: &str,
) -> Result<String, (StatusCode, String)> {
    if is_audius_url(source) {
        resolve_audius_stream_url(client, source).await
    } else {
        Ok(source.to_string())
    }
}

fn is_audius_url(source: &str) -> bool {
    let Ok(url) = reqwest::Url::parse(source) else {
        return false;
    };

    matches!(url.scheme(), "http" | "https")
        && url
            .host_str()
            .is_some_and(|host| host == "audius.co" || host.ends_with(".audius.co"))
}

async fn resolve_audius_stream_url(
    client: &Client,
    source: &str,
) -> Result<String, (StatusCode, String)> {
    let resolver = AudiusResolver::new(client.clone(), AUDIOUS_API_BASE_URL.to_string());
    resolver.resolve_stream_url(source).await.map_err(|e| {
        tracing::error!("Failed to resolve Audius URL {}: {}", source, e);
        (StatusCode::BAD_GATEWAY, e)
    })
}

#[derive(Clone)]
struct AudiusResolver {
    client: Client,
    base_url: String,
}

#[derive(Debug, Deserialize)]
struct AudiusDataResponse<T> {
    data: T,
}

#[derive(Debug, Deserialize)]
struct AudiusTrack {
    id: String,
}

impl AudiusResolver {
    fn new(client: Client, base_url: String) -> Self {
        Self { client, base_url }
    }

    async fn resolve_stream_url(&self, source: &str) -> Result<String, String> {
        let track_id = self.resolve_track_id(source).await?;
        self.fetch_track_stream_url(&track_id).await
    }

    async fn resolve_track_id(&self, source: &str) -> Result<String, String> {
        let resolve_url = format!("{}/resolve", self.base_url.trim_end_matches('/'));
        let response = self
            .client
            .get(&resolve_url)
            .query(&[("url", source)])
            .send()
            .await
            .map_err(|e| format!("Audius resolve request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!(
                "Audius resolve failed with status {}",
                response.status()
            ));
        }

        let final_url = response.url().clone();
        let body = response
            .text()
            .await
            .map_err(|e| format!("Audius resolve response read failed: {}", e))
            .and_then(|body| {
                serde_json::from_str::<AudiusDataResponse<AudiusTrack>>(&body)
                    .map_err(|e| format!("Audius resolve response decode failed: {}", e))
            });

        match body {
            Ok(track) => Ok(track.data.id),
            Err(err) => {
                if let Some(track_id) = track_id_from_resolved_url(&final_url) {
                    warn!(
                        "Fell back to resolved Audius URL parsing after JSON decode failed: {}",
                        err
                    );
                    Ok(track_id)
                } else {
                    Err(format!("Audius resolve response was not a track: {}", err))
                }
            }
        }
    }

    async fn fetch_track_stream_url(&self, track_id: &str) -> Result<String, String> {
        let stream_url = format!(
            "{}/tracks/{}/stream",
            self.base_url.trim_end_matches('/'),
            track_id
        );

        let response = self
            .client
            .get(&stream_url)
            .query(&[("no_redirect", "true")])
            .send()
            .await
            .map_err(|e| format!("Audius stream lookup failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!(
                "Audius stream lookup failed with status {}",
                response.status()
            ));
        }

        let body = response
            .text()
            .await
            .map_err(|e| format!("Audius stream response read failed: {}", e))?;
        let stream = serde_json::from_str::<AudiusDataResponse<String>>(&body)
            .map_err(|e| format!("Audius stream response decode failed: {}", e))?;

        Ok(stream.data)
    }
}

fn track_id_from_resolved_url(url: &reqwest::Url) -> Option<String> {
    let mut segments = url.path_segments()?;
    while let Some(segment) = segments.next() {
        if segment == "tracks" {
            return segments.next().map(|id| id.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path, query_param},
    };

    #[tokio::test]
    async fn resolves_regular_audius_url_to_stream_url() {
        let server = MockServer::start().await;
        let resolver = AudiusResolver::new(Client::new(), format!("{}/v1", server.uri()));

        Mock::given(method("GET"))
            .and(path("/v1/resolve"))
            .and(query_param(
                "url",
                "https://audius.co/Ookay/ookay-veronica-bravo-dontwakemeup-ft-veronica-bravo",
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": { "id": "track-123" }
            })))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/v1/tracks/track-123/stream"))
            .and(query_param("no_redirect", "true"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": "https://content.audius.co/audio/track-123.mp3"
            })))
            .mount(&server)
            .await;

        let resolved = resolver
            .resolve_stream_url(
                "https://audius.co/Ookay/ookay-veronica-bravo-dontwakemeup-ft-veronica-bravo",
            )
            .await
            .unwrap();

        assert_eq!(resolved, "https://content.audius.co/audio/track-123.mp3");
    }
}

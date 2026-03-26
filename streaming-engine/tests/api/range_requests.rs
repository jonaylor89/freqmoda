use std::time::{SystemTime, UNIX_EPOCH};

use crate::helpers::spawn_app;

const TEST_FILE_PREFIX: &str = "range-request-";

fn minimal_wav_file() -> Vec<u8> {
    let mut wav = Vec::new();
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&38u32.to_le_bytes());
    wav.extend_from_slice(b"WAVE");
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&8000u32.to_le_bytes());
    wav.extend_from_slice(&16000u32.to_le_bytes());
    wav.extend_from_slice(&2u16.to_le_bytes());
    wav.extend_from_slice(&16u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&2u32.to_le_bytes());
    wav.extend_from_slice(&0i16.to_le_bytes());
    wav
}

async fn cleanup_test_artifacts() {
    let mut entries = match tokio::fs::read_dir("uploads").await {
        Ok(entries) => entries,
        Err(_) => return,
    };

    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };

        if name.starts_with(TEST_FILE_PREFIX) {
            let _ = tokio::fs::remove_file(path).await;
        }
    }
}

#[tokio::test]
async fn first_audio_request_honors_byte_ranges() {
    cleanup_test_artifacts().await;

    let app = spawn_app().await;
    let unique_id = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before unix epoch")
        .as_nanos();
    let file_name = format!("{TEST_FILE_PREFIX}{unique_id}.wav");
    let audio_bytes = minimal_wav_file();

    tokio::fs::create_dir_all("uploads")
        .await
        .expect("failed to create uploads dir");
    tokio::fs::write(
        std::path::Path::new("uploads").join(&file_name),
        &audio_bytes,
    )
    .await
    .expect("failed to write test audio");

    let range_response = app
        .api_client
        .get(format!("{}/unsafe/{}", app.address, file_name))
        .header(reqwest::header::RANGE, "bytes=2-7")
        .send()
        .await
        .expect("Failed to execute range request");

    assert_eq!(
        range_response.status(),
        reqwest::StatusCode::PARTIAL_CONTENT
    );
    assert_eq!(
        range_response
            .headers()
            .get(reqwest::header::ACCEPT_RANGES)
            .and_then(|value| value.to_str().ok()),
        Some("bytes")
    );
    assert_eq!(
        range_response
            .headers()
            .get(reqwest::header::CONTENT_LENGTH)
            .and_then(|value| value.to_str().ok()),
        Some("6")
    );
    let content_range = range_response
        .headers()
        .get(reqwest::header::CONTENT_RANGE)
        .and_then(|value| value.to_str().ok())
        .expect("missing content-range")
        .to_string();
    let partial_bytes = range_response
        .bytes()
        .await
        .expect("failed to read partial response body");

    let full_response = app
        .api_client
        .get(format!("{}/unsafe/{}", app.address, file_name))
        .send()
        .await
        .expect("Failed to execute full request");

    assert_eq!(full_response.status(), reqwest::StatusCode::OK);
    let full_bytes = full_response
        .bytes()
        .await
        .expect("failed to read full response body");

    assert_eq!(content_range, format!("bytes 2-7/{}", full_bytes.len()));
    assert_eq!(partial_bytes.len(), 6);
    assert_eq!(partial_bytes, full_bytes.slice(2..8));

    cleanup_test_artifacts().await;
}

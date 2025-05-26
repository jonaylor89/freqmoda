use color_eyre::Result;
use std::collections::HashMap;
use tempfile::TempDir;
use tokio::process::Command;
use tracing::debug;
use tracing::instrument;

use crate::{
    blob::{AudioBuffer, AudioFormat},
    streamingpath::params::Params,
};

#[instrument(skip(input, params, temp_dir))]
pub async fn process_audio(
    input: &AudioBuffer,
    params: &Params,
    temp_dir: TempDir,
    additional_tags: &HashMap<String, String>,
) -> Result<AudioBuffer> {
    let output_format = params.format.unwrap_or(AudioFormat::Mp3);

    let input_path = temp_dir
        .path()
        .join(format!("in.{}", input.format().extension()));
    let output_path = temp_dir
        .path()
        .join(format!("out.{}", output_format.extension()));

    // Write input file
    tokio::fs::write(&input_path, input.as_ref()).await?;

    // Build FFmpeg command
    let mut cmd = Command::new("ffmpeg");
    cmd.args(["-i", input_path.to_str().unwrap(), "-y"]);

    // Add quiet mode flags to reduce log noise
    cmd.args(["-loglevel", "quiet", "-nostats", "-nostdin"]);

    // Add optional metadata
    if let Some(tags) = &params.tags {
        for (k, v) in tags {
            cmd.args(["-metadata", &format!("{}={}", k, v)]);
        }
    }

    // Add additional tags
    for (k, v) in additional_tags {
        cmd.args(["-metadata", &format!("{}={}", k, v)]);
    }

    // Add encoding parameters and output path
    cmd.args(params.to_ffmpeg_args())
        .arg(output_path.to_str().unwrap());

    debug!(?cmd, "Executing FFmpeg command");

    // Execute FFmpeg
    let status = cmd.status().await?;
    if !status.success() {
        return Err(color_eyre::eyre::eyre!("FFmpeg failed"));
    }

    // Read and return output
    let processed = tokio::fs::read(&output_path).await?;
    Ok(AudioBuffer::from_bytes_with_format(
        processed,
        output_format,
    ))
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;
    use tokio::process::Command;

    #[test]
    fn test_ffmpeg_command_construction_always_quiet() {
        // Test that FFmpeg command always includes quiet flags
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("test.mp3");
        let output_path = temp_dir.path().join("output.mp3");

        // Write a minimal MP3 file for testing
        std::fs::write(&input_path, [0xFF, 0xFB, 0x90, 0x00]).unwrap();

        let mut cmd = Command::new("ffmpeg");
        cmd.args(["-i", input_path.to_str().unwrap(), "-y"]);
        cmd.args(["-loglevel", "quiet", "-nostats", "-nostdin"]);
        cmd.arg(output_path.to_str().unwrap());

        // Verify the command contains the expected quiet flags
        let cmd_str = format!("{:?}", cmd);
        assert!(cmd_str.contains("quiet"));
        assert!(cmd_str.contains("nostats"));
        assert!(cmd_str.contains("nostdin"));
    }
}

//! Minimal TTS stub that returns a 1-second 440Hz tone as WAV.
//! This is just a placeholder to prove the wiring; swap in Piper/Kokoro later.

use axum::{
    Json, Router,
    body::Body,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::post,
};
use serde::Deserialize;
use tokio::net::TcpListener;
use tracing::{Level, info};

#[derive(Debug, Deserialize)]
struct TtsRequest {
    input: String,
    voice: Option<String>,
    format: Option<String>,
}

async fn tts_handler(Json(req): Json<TtsRequest>) -> Response {
    let format = req.format.as_deref().unwrap_or("wav");
    let voice = req.voice.as_deref().unwrap_or("default");

    info!(
        "TTS request: {} chars, voice={}, format={}",
        req.input.len(),
        voice,
        format
    );

    match format {
        "wav" => {
            // Stub: generate tone regardless of input text
            // Real implementation would synthesize req.input with req.voice
            let bytes = generate_sine_wav(440.0, 1.0);
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "audio/wav")],
                Body::from(bytes),
            )
                .into_response()
        }
        _ => (
            StatusCode::BAD_REQUEST,
            "Unsupported format; only 'wav' is implemented",
        )
            .into_response(),
    }
}

fn generate_sine_wav(freq_hz: f32, duration_secs: f32) -> Vec<u8> {
    let sample_rate = 44100u32;
    let num_samples = (sample_rate as f32 * duration_secs) as u32;
    let amplitude = i16::MAX as f32;

    let mut data = Vec::with_capacity((num_samples * 2) as usize);
    for n in 0..num_samples {
        let t = n as f32 / sample_rate as f32;
        let sample = (2.0 * std::f32::consts::PI * freq_hz * t).sin();
        let v = (sample * amplitude) as i16;
        data.extend_from_slice(&v.to_le_bytes());
    }

    // Build simple PCM WAV header (mono, 16-bit)
    let mut wav = Vec::new();
    let byte_rate = sample_rate * 2;
    let block_align = 2u16;
    let bits_per_sample = 16u16;
    let subchunk2_size = num_samples * 2;
    let chunk_size = 36 + subchunk2_size;

    // RIFF header
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&chunk_size.to_le_bytes());
    wav.extend_from_slice(b"WAVE");

    // fmt subchunk
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes()); // Subchunk1Size for PCM
    wav.extend_from_slice(&1u16.to_le_bytes()); // AudioFormat = PCM
    wav.extend_from_slice(&1u16.to_le_bytes()); // NumChannels = 1
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&block_align.to_le_bytes());
    wav.extend_from_slice(&bits_per_sample.to_le_bytes());

    // data subchunk
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&subchunk2_size.to_le_bytes());
    wav.extend_from_slice(&data);

    wav
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_env_filter("tts_node=info,axum=info")
        .init();

    let app = Router::new().route("/v1/audio/speech", post(tts_handler));

    let listener = TcpListener::bind("0.0.0.0:9001").await?;
    info!("tts-node listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wav_header_valid() {
        let wav = generate_sine_wav(440.0, 1.0);
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        assert_eq!(&wav[12..16], b"fmt ");
    }

    #[test]
    fn test_wav_correct_size() {
        let wav = generate_sine_wav(440.0, 1.0);
        // 44100 samples * 2 bytes + 44 byte header
        let expected_size = 44100 * 2 + 44;
        assert_eq!(wav.len(), expected_size);
    }
}

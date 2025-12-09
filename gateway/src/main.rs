//! API Gateway that routes requests to backend LLM and TTS services.
//! Exposes OpenAI-compatible endpoints and handles CORS for browser access.

use std::convert::Infallible;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::OnceCell;
use tracing::{Level, info};
use warp::Filter;

static HTTP_CLIENT: OnceCell<Client> = OnceCell::const_new();

#[derive(Debug, Deserialize, Serialize, Clone)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct TtsRequest {
    input: String,
    voice: Option<String>,
    format: Option<String>,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

/// Determine which LLM backend to route to based on model name.
///
/// Currently all models route to the single llm-node instance.
/// Future enhancement: route different model prefixes to different backends:
/// - qwen3-* -> llm-node-gpu0
/// - llama-3-* -> llm-node-gpu1
/// - etc.
fn get_llm_target(_model: &str) -> &'static str {
    "http://localhost:9000/v1/chat/completions"
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_env_filter("gateway=info,warp=info")
        .init();

    HTTP_CLIENT
        .set(Client::builder().build()?)
        .expect("client already set");

    let chat = warp::path!("v1" / "chat" / "completions")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(handle_chat);

    let tts = warp::path!("v1" / "audio" / "speech")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(handle_tts);

    let routes = chat.or(tts).with(warp::cors().allow_any_origin());

    let addr = ([0, 0, 0, 0], 8080);
    info!(
        "gateway listening on http://{}.{}.{}.{}:{}",
        addr.0[0], addr.0[1], addr.0[2], addr.0[3], addr.1
    );
    warp::serve(routes).run(addr).await;

    Ok(())
}

async fn handle_chat(body: ChatCompletionRequest) -> Result<impl warp::Reply, Infallible> {
    let target = get_llm_target(&body.model);

    info!(
        "Chat request: model={}, messages={}, target={}",
        body.model,
        body.messages.len(),
        target
    );

    let client = HTTP_CLIENT.get().expect("client not initialized");
    let resp = client.post(target).json(&body).send().await;

    match resp {
        Ok(r) => {
            let status_code = r.status().as_u16();
            let bytes = r.bytes().await.unwrap_or_default();
            let warp_status =
                warp::http::StatusCode::from_u16(status_code).unwrap_or(warp::http::StatusCode::OK);
            Ok(warp::reply::with_status(
                warp::reply::with_header(bytes.to_vec(), "Content-Type", "application/json"),
                warp_status,
            ))
        }
        Err(e) => {
            let error = ErrorResponse {
                error: format!("llm-node unreachable: {e}"),
            };
            let json_body = serde_json::to_vec(&error).unwrap_or_default();
            Ok(warp::reply::with_status(
                warp::reply::with_header(json_body, "Content-Type", "application/json"),
                warp::http::StatusCode::BAD_GATEWAY,
            ))
        }
    }
}

async fn handle_tts(body: TtsRequest) -> Result<impl warp::Reply, Infallible> {
    let client = HTTP_CLIENT.get().expect("client not initialized");
    let target = "http://localhost:9001/v1/audio/speech";

    info!(
        "TTS request: {} chars, voice={:?}, format={:?}",
        body.input.len(),
        body.voice,
        body.format
    );

    let resp = client.post(target).json(&body).send().await;
    match resp {
        Ok(r) => {
            let status_code = r.status().as_u16();
            let content_type = r
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("application/octet-stream")
                .to_string();
            let bytes = r.bytes().await.unwrap_or_default();
            let warp_status =
                warp::http::StatusCode::from_u16(status_code).unwrap_or(warp::http::StatusCode::OK);
            Ok(warp::reply::with_status(
                warp::reply::with_header(bytes.to_vec(), "Content-Type", content_type),
                warp_status,
            ))
        }
        Err(e) => {
            let error = ErrorResponse {
                error: format!("TTS node unreachable: {e}"),
            };
            let json_body = serde_json::to_vec(&error).unwrap_or_default();
            Ok(warp::reply::with_status(
                warp::reply::with_header(json_body, "Content-Type", "application/json"),
                warp::http::StatusCode::BAD_GATEWAY,
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_llm_target_returns_default() {
        // Currently all models route to the same endpoint
        assert_eq!(
            get_llm_target("qwen3-8b-instruct"),
            "http://localhost:9000/v1/chat/completions"
        );
        assert_eq!(
            get_llm_target("llama-3.1-8b"),
            "http://localhost:9000/v1/chat/completions"
        );
        assert_eq!(
            get_llm_target("unknown-model"),
            "http://localhost:9000/v1/chat/completions"
        );
    }

    #[test]
    fn test_error_response_serialization() {
        let error = ErrorResponse {
            error: "test error".into(),
        };
        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("test error"));
    }

    #[test]
    fn test_chat_request_serialization() {
        let req = ChatCompletionRequest {
            model: "test".into(),
            messages: vec![ChatMessage {
                role: "user".into(),
                content: "hello".into(),
            }],
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("test"));
        assert!(json.contains("user"));
        assert!(json.contains("hello"));
    }

    #[test]
    fn test_tts_request_serialization() {
        let req = TtsRequest {
            input: "Hello world".into(),
            voice: Some("en_US".into()),
            format: Some("wav".into()),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("Hello world"));
        assert!(json.contains("en_US"));
        assert!(json.contains("wav"));
    }
}

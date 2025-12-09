//! LLM inference service stub exposing OpenAI-compatible chat completions API.
//! This is a placeholder that echoes input; swap in mistral.rs or llama.cpp later.

use axum::{Json, Router, routing::post};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tracing::{Level, info};

#[derive(Debug, Deserialize, Serialize, Clone)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize, Clone)]
struct ChatCompletionResponse {
    id: String,
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Serialize, Clone)]
struct ChatChoice {
    index: usize,
    message: ChatMessage,
}

fn find_last_user_message(messages: &[ChatMessage]) -> ChatMessage {
    messages
        .iter()
        .rev()
        .find(|m| m.role == "user")
        .cloned()
        .unwrap_or(ChatMessage {
            role: "user".into(),
            content: "(no user message found)".into(),
        })
}

fn create_echo_response(model: &str, user_message: &ChatMessage) -> ChatCompletionResponse {
    let reply_text = format!(
        "Echo from llm-node (model={model}): {}",
        user_message.content
    );

    ChatCompletionResponse {
        id: uuid::Uuid::new_v4().to_string(),
        choices: vec![ChatChoice {
            index: 0,
            message: ChatMessage {
                role: "assistant".into(),
                content: reply_text,
            },
        }],
    }
}

async fn chat_handler(Json(req): Json<ChatCompletionRequest>) -> Json<ChatCompletionResponse> {
    info!(
        "Chat request: model={}, messages={}",
        req.model,
        req.messages.len()
    );

    let last_user = find_last_user_message(&req.messages);
    let response = create_echo_response(&req.model, &last_user);

    Json(response)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_env_filter("llm_node=info,axum=info")
        .init();

    let app = Router::new().route("/v1/chat/completions", post(chat_handler));

    let listener = TcpListener::bind("0.0.0.0:9000").await?;
    info!("llm-node listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_last_user_message_found() {
        let messages = vec![
            ChatMessage {
                role: "system".into(),
                content: "You are helpful".into(),
            },
            ChatMessage {
                role: "user".into(),
                content: "Hello".into(),
            },
            ChatMessage {
                role: "assistant".into(),
                content: "Hi there".into(),
            },
            ChatMessage {
                role: "user".into(),
                content: "How are you?".into(),
            },
        ];

        let result = find_last_user_message(&messages);
        assert_eq!(result.role, "user");
        assert_eq!(result.content, "How are you?");
    }

    #[test]
    fn test_find_last_user_message_not_found() {
        let messages = vec![ChatMessage {
            role: "system".into(),
            content: "You are helpful".into(),
        }];

        let result = find_last_user_message(&messages);
        assert_eq!(result.role, "user");
        assert_eq!(result.content, "(no user message found)");
    }

    #[test]
    fn test_create_echo_response() {
        let user_msg = ChatMessage {
            role: "user".into(),
            content: "Test message".into(),
        };

        let response = create_echo_response("test-model", &user_msg);

        assert!(!response.id.is_empty());
        assert_eq!(response.choices.len(), 1);
        assert_eq!(response.choices[0].index, 0);
        assert_eq!(response.choices[0].message.role, "assistant");
        assert!(response.choices[0].message.content.contains("test-model"));
        assert!(response.choices[0].message.content.contains("Test message"));
    }
}

# Design Document

## Overview

This document describes the technical design decisions and implementation patterns for ai-stack.

## Design Principles

1. **Rust-first**: All business logic in Rust; no Python at runtime
2. **Simplicity**: Minimal abstractions; direct implementations
3. **Composability**: Independent services with clear interfaces
4. **Compatibility**: OpenAI-style APIs for ecosystem integration

## API Design

### Chat Completions

**Endpoint**: `POST /v1/chat/completions`

**Request Schema**:
```rust
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    // Future: temperature, top_p, max_tokens, stream
}

struct ChatMessage {
    role: String,      // "system", "user", "assistant"
    content: String,
}
```

**Response Schema**:
```rust
struct ChatCompletionResponse {
    id: String,        // "chatcmpl-{uuid}"
    choices: Vec<ChatChoice>,
}

struct ChatChoice {
    index: usize,
    message: ChatMessage,
}
```

**Design Rationale**:
- Follows OpenAI API structure for compatibility
- Minimal fields for MVP; expand as needed
- UUID-based IDs for uniqueness without coordination

### Text-to-Speech

**Endpoint**: `POST /v1/audio/speech`

**Request Schema**:
```rust
struct TtsRequest {
    input: String,
    voice: Option<String>,   // Default: first available
    format: Option<String>,  // Default: "wav"
}
```

**Response**: Binary audio data with `Content-Type: audio/wav`

**Design Rationale**:
- Simple text-in, audio-out interface
- Optional fields with sensible defaults
- WAV format for simplicity; no transcoding needed

## Service Design

### Gateway

**Framework**: warp

**Why warp?**
- Composable filter-based routing
- Good async support
- Lightweight compared to axum for simple proxying

**Routing Logic**:
```rust
// Route by endpoint, not by model (yet)
/v1/chat/completions -> http://localhost:9000/v1/chat/completions
/v1/audio/speech     -> http://localhost:9001/v1/audio/speech
```

**Future Enhancement**: Model-based routing
```rust
// Route by model prefix to different backends
model.starts_with("qwen3")  -> llm-node-gpu0
model.starts_with("llama")  -> llm-node-gpu1
```

### LLM Node

**Framework**: axum

**Why axum?**
- Tower-based middleware ecosystem
- Type-safe extractors
- Good for request/response handlers

**Current Implementation**: Echo stub
- Returns formatted echo of user input
- Validates request structure
- Generates proper response format

**Target Implementation**:
```rust
// Load model at startup
let model = GgufModelBuilder::new()
    .model_id("qwen3-8b-instruct-q4_k_m.gguf")
    .build()
    .await?;

// Handle request
async fn chat_handler(req: ChatCompletionRequest) -> ChatCompletionResponse {
    let output = model.generate(&req.messages).await?;
    // Format response
}
```

### TTS Node

**Framework**: axum

**Current Implementation**: Sine wave generator
- Generates 440Hz tone as WAV
- Proves HTTP + audio response pipeline

**Target Implementation**:
```rust
// Load Piper model
let synthesizer = Synthesizer::from_files(
    "models/en_US-libritts-high.onnx",
    "models/en_US-libritts-high.json",
)?;

// Handle request
async fn tts_handler(req: TtsRequest) -> Response {
    let audio = synthesizer.synthesize(&req.input)?;
    Response::builder()
        .header("Content-Type", "audio/wav")
        .body(audio)
}
```

### UI

**Framework**: Yew (CSR mode)

**Why Yew?**
- Rust-based WASM framework
- No JavaScript/TypeScript needed
- Component model similar to React

**State Management**:
```rust
let input = use_state(|| String::new());
let output = use_state(|| String::new());

// Update on user input
let on_input_change = Callback::from(|e| input.set(e.target_value()));

// Send request on button click
let on_send = Callback::from(|_| {
    spawn_local(async {
        let response = fetch_chat_completion(input).await;
        output.set(response);
    });
});
```

## Error Handling

### Gateway Errors

| Scenario | HTTP Status | Response |
|----------|-------------|----------|
| Backend unreachable | 502 | `{"error": "llm-node unreachable: ..."}` |
| Invalid request | 400 | `{"error": "..."}` |
| Backend error | Forward | Pass through backend response |

### Service Errors

Use `anyhow` for internal errors, convert to JSON at boundaries:
```rust
async fn handler() -> Result<Json<Response>, StatusCode> {
    let result = do_thing().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(result))
}
```

## Configuration

### Current: Hardcoded

```rust
// llm-node
let listener = TcpListener::bind("0.0.0.0:9000").await?;

// tts-node
let listener = TcpListener::bind("0.0.0.0:9001").await?;

// gateway
let addr = ([0, 0, 0, 0], 8080);
let llm_target = "http://localhost:9000/v1/chat/completions";
let tts_target = "http://localhost:9001/v1/audio/speech";
```

### Future: Environment/Config File

```toml
# config.toml
[gateway]
port = 8080

[llm]
port = 9000
model_path = "models/qwen3-8b-instruct-q4_k_m.gguf"

[tts]
port = 9001
model_path = "models/en_US-libritts-high.onnx"
```

## Testing Strategy

### Unit Tests
- Request/response serialization
- WAV generation logic
- Routing decisions

### Integration Tests
- Full request flow through gateway
- Service health checks
- Error propagation

### Test Patterns

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wav_header_valid() {
        let wav = generate_sine_wav(440.0, 1.0);
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
    }

    #[tokio::test]
    async fn test_chat_handler_echo() {
        let req = ChatCompletionRequest {
            model: "test".into(),
            messages: vec![ChatMessage {
                role: "user".into(),
                content: "Hello".into(),
            }],
        };
        let resp = chat_handler(Json(req)).await;
        assert!(resp.0.choices[0].message.content.contains("Hello"));
    }
}
```

## Security Considerations

### Current Scope (Local Development)
- Services bind to 0.0.0.0 (all interfaces)
- No authentication
- CORS allows any origin

### Future Considerations
- Bind to localhost only by default
- API key authentication
- CORS allowlist
- Input sanitization (prompt injection awareness)
- Rate limiting

## Performance Considerations

### Async Runtime
- Use Tokio multi-threaded runtime
- Avoid blocking in async handlers
- Use spawn_blocking for CPU-intensive model inference

### Memory Management
- Models loaded once at startup
- Reuse HTTP client (OnceCell in gateway)
- Avoid unnecessary clones in hot paths

### GPU Utilization
- One model per GPU for VRAM efficiency
- Batch requests when possible (future)
- Use quantized models (Q4/Q5) for 12GB VRAM

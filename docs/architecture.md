# Architecture

This document describes the high-level architecture of the ai-stack project.

## Overview

ai-stack is a Rust 2024 workspace implementing a Python-free AI inference stack for local LLM and TTS services. The design prioritizes Rust-only runtime with optional C/C++ FFI for performance-critical backends.

## System Architecture

```
+-------------------------------------------+
|               UI (Yew/WASM)               |
|           Browser @ localhost             |
+---------------------+---------------------+
                      | HTTP/JSON
+---------------------v---------------------+
|              gateway (warp)               |
|              Port 8080                    |
|  /v1/chat/completions -> llm-node        |
|  /v1/audio/speech     -> tts-node        |
+----------+--------------------+-----------+
           |                    |
           | HTTP               | HTTP
           |                    |
+----------v----------+  +------v-----------+
|  llm-node (axum)    |  |  tts-node (axum) |
|    Port 9000        |  |   Port 9001      |
|  OpenAI-style API   |  |   WAV audio      |
+---------------------+  +------------------+
```

## Crate Responsibilities

### gateway (warp)

**Role**: Front-door HTTP proxy and API facade

**Responsibilities**:
- Single entry point for all external requests
- Route `/v1/chat/completions` to llm-node
- Route `/v1/audio/speech` to tts-node
- CORS handling for browser-based UI
- Model-based routing (future: route different models to different nodes)

**Port**: 8080

**Dependencies**: warp, reqwest, tokio, serde

### llm-node (axum)

**Role**: LLM inference service

**Responsibilities**:
- Expose OpenAI-compatible `/v1/chat/completions` endpoint
- Load and serve LLM models
- Handle chat completion requests

**Current State**: Echo implementation (returns input message)

**Target State**:
- mistral.rs integration for Qwen3/LLaMA/Mistral models
- llama.cpp bindings for GGUF models
- Support for 12GB VRAM GPUs with Q4/Q5 quantization

**Port**: 9000

**Dependencies**: axum, tokio, serde, uuid

### tts-node (axum)

**Role**: Text-to-Speech service

**Responsibilities**:
- Expose `/v1/audio/speech` endpoint
- Convert text to audio (WAV format)
- Support multiple voices (future)

**Current State**: Generates 440Hz sine wave WAV

**Target State**:
- piper-rs for Piper TTS models
- Kokoro TTS via sherpa-rs
- Candle TTS for MetaVoice-1B/Parler-TTS

**Port**: 9001

**Dependencies**: axum, tokio, serde

### ui (Yew/WASM)

**Role**: Browser-based frontend

**Responsibilities**:
- Provide chat interface for LLM interaction
- Display responses from gateway
- Future: audio playback for TTS responses

**Current State**: Minimal chat UI posting to gateway

**Technology**: Yew 0.21 (CSR mode), gloo-net for HTTP

**Build**: wasm-pack or Trunk

## Communication Patterns

### Request Flow

1. User enters prompt in UI
2. UI sends POST to `gateway:8080/v1/chat/completions`
3. Gateway forwards to `llm-node:9000/v1/chat/completions`
4. llm-node processes and returns response
5. Gateway returns response to UI
6. UI displays response

### API Contracts

**Chat Completions (OpenAI-compatible)**:
```json
// Request
{
  "model": "qwen3-8b-instruct",
  "messages": [
    {"role": "user", "content": "Hello"}
  ]
}

// Response
{
  "id": "chatcmpl-...",
  "choices": [
    {
      "index": 0,
      "message": {"role": "assistant", "content": "..."}
    }
  ]
}
```

**TTS (OpenAI-style)**:
```json
// Request
{
  "input": "Text to speak",
  "voice": "en_US-libritts-high",
  "format": "wav"
}

// Response: audio/wav binary
```

## Design Decisions

### Why Rust-Only Runtime?

1. **GPU compatibility**: Direct CUDA toolkit builds avoid Python wheel lag for new GPUs (e.g., Blackwell)
2. **Performance**: No Python GIL, native parallelism via Tokio
3. **Reliability**: Type safety, no runtime type errors
4. **Deployment**: Single binary per service, no virtualenv management

### Why Separate Services?

1. **Resource isolation**: Each service manages its own GPU/memory
2. **Scaling flexibility**: Run multiple llm-nodes on different GPUs
3. **Independent deployment**: Update TTS without touching LLM
4. **Failure isolation**: TTS crash doesn't affect chat

### Why OpenAI-Compatible API?

1. **Ecosystem compatibility**: Works with existing tools expecting OpenAI API
2. **Easy migration**: Swap local inference for cloud API trivially
3. **Well-documented**: No need to invent new API contracts

## Future Architecture

### Multi-Node Scaling

```
+-------------------+
|     gateway       |
+--------+----------+
         |
    +----+----+
    |         |
+---v---+ +---v---+
|llm-0  | |llm-1  | ...
|(GPU 0)| |(GPU 1)|
+-------+ +-------+
```

### Model Routing

Gateway will route based on:
- Model name prefix (qwen3 -> node-0, llama -> node-1)
- GPU VRAM availability
- Current load/queue depth

### Inference Backends

Planned backend support:
- mistral.rs (Candle-based, primary)
- llama.cpp (GGUF models, secondary)
- ONNX Runtime (TTS models)

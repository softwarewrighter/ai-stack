# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

```bash
# Build all crates
cargo build

# Build release
cargo build --release

# Run individual services (each in separate terminal)
cargo run -p llm-node    # LLM service on port 9000
cargo run -p tts-node    # TTS service on port 9001
cargo run -p gateway     # API gateway on port 8080

# Lint and format
cargo clippy
cargo fmt
```

## UI (Yew/WASM)

The `ui` crate is a Yew/WASM frontend. Build with Trunk or wasm-pack:
```bash
# With Trunk
trunk serve ui/static/index.html

# Or with wasm-pack
wasm-pack build ui --target web
```

## Architecture

This is a Rust 2024 workspace (Rust 1.85.0+) implementing a Python-free AI inference stack for local LLM and TTS. The design prioritizes Rust-only runtime with optional C/C++ FFI for performance-critical backends.

```
┌─────────────────────────────────────────┐
│               UI (Yew/WASM)              │
│           http://localhost:8080          │
└─────────────────┬───────────────────────┘
                  │ HTTP
┌─────────────────▼───────────────────────┐
│              gateway (warp)              │
│              Port 8080                   │
│  /v1/chat/completions → llm-node        │
│  /v1/audio/speech     → tts-node        │
└────────┬────────────────────┬───────────┘
         │                    │
┌────────▼─────────┐  ┌──────▼──────────┐
│  llm-node (axum) │  │ tts-node (axum) │
│    Port 9000     │  │   Port 9001     │
│  OpenAI-style    │  │   WAV audio     │
│  chat API        │  │   generation    │
└──────────────────┘  └─────────────────┘
```

**gateway**: Warp-based HTTP proxy routing to backend services. CORS enabled. Routes based on model prefix (qwen3, llama-3, etc. all currently route to llm-node:9000).

**llm-node**: Axum server exposing `/v1/chat/completions` (OpenAI-compatible). Currently echoes input; designed to be replaced with:
- mistral.rs (Candle-based, supports Qwen3, LLaMA, Mistral, Gemma, DeepSeek)
- llama.cpp bindings (llama_cpp, llama-cpp-2 crates) for GGUF models

**tts-node**: Axum server exposing `/v1/audio/speech`. Currently generates a 440Hz sine wave WAV; designed to be replaced with:
- piper-rs (pure Rust Piper TTS)
- Kokoro TTS via sherpa-rs
- Candle TTS (MetaVoice-1B, Parler-TTS)

**ui**: Yew client-side rendered WASM app. Talks to gateway at localhost:8080.

## Target Models (12GB VRAM)

LLM targets (Q4/Q5 quantization):
- Qwen3-8B-Instruct (primary, Apache-2.0)
- Llama-3.1-8B-Instruct (alternate)
- Mistral-7B-Instruct (fast/fallback)

TTS targets:
- Piper voices via piper-rs (baseline, CPU-friendly)
- Kokoro (~82M params, higher quality)
- MetaVoice-1B/Parler-TTS via Candle (premium, needs more VRAM)

## Design Philosophy

- Pure Rust runtime, no Python in production
- Python acceptable only for offline model conversion (HF → GGUF/ONNX)
- Prefer Rust/Yew/WASM over TypeScript/JavaScript for UIs
- One UI per use-case rather than monolithic multi-mode apps
- Build against system CUDA toolkit directly, not Python wheels

# Rust 2024 AI Stack Skeleton

This workspace is a minimal, **Rust-only** skeleton for:

- `llm-node`: placeholder LLM service (HTTP, OpenAI-style chat completions)
- `tts-node`: placeholder TTS service (returns a 1s 440Hz WAV tone)
- `gateway`: front-door proxy exposing `/v1/chat/completions` and `/v1/audio/speech`
- `ui`: Yew/WASM front-end talking to the gateway

## Building

You need a recent Rust toolchain (see `rust-toolchain.toml`).

```bash
cd ai-stack

# Build all crates
cargo build

# Run services (in separate terminals)
cargo run -p llm-node
cargo run -p tts-node
cargo run -p gateway
```

The UI is a Yew/WASM crate; you can integrate it with your preferred bundler
(Trunk, wasm-pack, etc.). The HTML stub is under `ui/static/index.html`.

## Next steps

- Replace the echo implementation in `llm-node` with `mistral.rs` or llama.cpp bindings.
- Replace the tone generator in `tts-node` with Piper / Kokoro / Candle TTS.
- Point Yew UI to your deployed gateway and expand it into separate per-use-case UIs.

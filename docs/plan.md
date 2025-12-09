# Implementation Plan

## Phase 1: Skeleton Validation (Current)

**Goal**: Ensure the initial skeleton builds, passes linting, and has basic tests.

### Tasks

- [ ] Validate cargo build succeeds
- [ ] Fix all clippy warnings
- [ ] Add unit tests for each crate
- [ ] Verify services start and respond
- [ ] Update CLAUDE.md with learnings

### Acceptance Criteria

1. `cargo build` succeeds with no errors
2. `cargo clippy --all-targets --all-features -- -D warnings` passes
3. `cargo test` runs with at least one test per crate
4. Each service starts and logs listening address

## Phase 2: Real LLM Integration

**Goal**: Replace echo stub with actual LLM inference.

### Tasks

- [ ] Add mistral.rs dependency to llm-node
- [ ] Download Qwen3-8B-Instruct GGUF model
- [ ] Implement model loading at startup
- [ ] Replace echo handler with inference handler
- [ ] Add basic prompt template handling
- [ ] Test with curl and UI

### Acceptance Criteria

1. llm-node loads model on startup
2. Chat requests return model-generated responses
3. Throughput >= 10 tok/s on 12GB GPU
4. Model fits in 12GB VRAM with Q4 quantization

## Phase 3: Real TTS Integration

**Goal**: Replace sine wave with Piper TTS.

### Tasks

- [ ] Add piper-rs dependency to tts-node
- [ ] Download Piper voice model (en_US-libritts-high)
- [ ] Implement synthesizer loading at startup
- [ ] Replace tone handler with TTS handler
- [ ] Test audio output quality

### Acceptance Criteria

1. tts-node loads voice model on startup
2. TTS requests return intelligible speech
3. Generation faster than real-time
4. Audio plays correctly in browser

## Phase 4: UI Enhancement

**Goal**: Make UI usable for actual conversations.

### Tasks

- [ ] Add conversation history display
- [ ] Implement message styling (user vs assistant)
- [ ] Add TTS playback button per response
- [ ] Add model selection dropdown
- [ ] Handle streaming responses (SSE)

### Acceptance Criteria

1. UI shows full conversation history
2. Clear visual distinction between user/assistant
3. Audio playback works for TTS responses
4. Model can be changed without page reload

## Phase 5: Multi-Model Support

**Goal**: Support multiple LLM models simultaneously.

### Tasks

- [ ] Add configuration file support
- [ ] Implement model registry in llm-node
- [ ] Add model-based routing in gateway
- [ ] Support loading multiple models
- [ ] Add model listing endpoint

### Acceptance Criteria

1. Gateway routes to correct model backend
2. Multiple models can be loaded concurrently
3. Model selection works in UI
4. Configuration via TOML file

## Phase 6: Production Hardening

**Goal**: Make the stack production-ready.

### Tasks

- [ ] Add health check endpoints
- [ ] Implement graceful shutdown
- [ ] Add request logging/tracing
- [ ] Add metrics endpoints
- [ ] Create Docker deployment configs
- [ ] Add API documentation

### Acceptance Criteria

1. Health checks report service status
2. Services shut down cleanly on SIGTERM
3. Structured logs with request IDs
4. Prometheus-compatible metrics
5. Docker Compose for local deployment

## Deferred Items

These items are explicitly out of scope for the current plan:

- Training/fine-tuning
- Multi-tenant authentication
- Embedding API
- Vision models
- Whisper transcription
- Kubernetes deployment
- Load balancing across multiple nodes

## Risk Mitigation

### Risk: mistral.rs API Changes
**Mitigation**: Pin to specific git commit; add version to plan.md

### Risk: Model Size Exceeds VRAM
**Mitigation**: Test with Q4_K_M quantization first; have Q3 fallback

### Risk: TTS Quality Insufficient
**Mitigation**: Kokoro as P1 alternative; document quality levels

### Risk: WASM Build Complexity
**Mitigation**: Document Trunk/wasm-pack setup; create build script

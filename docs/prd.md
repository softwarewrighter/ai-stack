# Product Requirements Document (PRD)

## Product Overview

**Product Name**: ai-stack

**Purpose**: A Rust-only local AI inference stack providing LLM chat completions and text-to-speech services without Python dependencies at runtime.

**Target Users**: Rust developers who want to run local LLM and TTS inference without Python ecosystem complexity, particularly on newer NVIDIA GPUs where Python wheels lag behind CUDA versions.

## Problem Statement

### Current Pain Points

1. **Python dependency hell**: Docker images with Python + CUDA frequently break on newer GPUs (Blackwell, etc.)
2. **Performance**: Python GIL limits parallelism; frameworks are slow compared to native code
3. **Complexity**: Managing virtualenvs, pip dependencies, and Python version conflicts
4. **Developer experience**: Rust developers unfamiliar with Python idioms forced to debug Python issues

### Solution

A pure Rust inference stack that:
- Builds directly against system CUDA toolkit
- Uses native parallelism via Tokio
- Produces single-binary services
- Exposes OpenAI-compatible APIs for easy integration

## Requirements

### Functional Requirements

#### FR-1: Chat Completions API
- **FR-1.1**: Expose `/v1/chat/completions` endpoint (OpenAI-compatible)
- **FR-1.2**: Accept model name, messages array (role/content)
- **FR-1.3**: Return assistant response with id and choices
- **FR-1.4**: Support streaming responses (future)

#### FR-2: Text-to-Speech API
- **FR-2.1**: Expose `/v1/audio/speech` endpoint
- **FR-2.2**: Accept input text, voice selection, format
- **FR-2.3**: Return audio data (WAV format initially)
- **FR-2.4**: Support multiple voices (future)

#### FR-3: Gateway Routing
- **FR-3.1**: Single entry point for all API requests
- **FR-3.2**: Route chat requests to llm-node
- **FR-3.3**: Route TTS requests to tts-node
- **FR-3.4**: Enable CORS for browser access

#### FR-4: Web UI
- **FR-4.1**: Provide browser-based chat interface
- **FR-4.2**: Display LLM responses
- **FR-4.3**: Support TTS playback (future)

### Non-Functional Requirements

#### NFR-1: Performance
- **NFR-1.1**: LLM inference at 10+ tokens/second on 12GB VRAM GPU
- **NFR-1.2**: TTS generation under 2x real-time
- **NFR-1.3**: Gateway adds less than 10ms latency

#### NFR-2: Resource Constraints
- **NFR-2.1**: LLM models must fit in 12GB VRAM (Q4/Q5 quantization)
- **NFR-2.2**: Services should start in under 30 seconds
- **NFR-2.3**: Memory usage under 2GB per service (excluding model)

#### NFR-3: Compatibility
- **NFR-3.1**: Rust 2024 edition (toolchain 1.85.0+)
- **NFR-3.2**: Build on Linux and macOS
- **NFR-3.3**: GPU support via CUDA (NVIDIA) or Metal (Apple)

#### NFR-4: Maintainability
- **NFR-4.1**: Zero clippy warnings
- **NFR-4.2**: All public APIs documented
- **NFR-4.3**: Test coverage for core functionality

## Target Models

### LLM Models (12GB VRAM)

| Model | Parameters | Use Case | Priority |
|-------|------------|----------|----------|
| Qwen3-8B-Instruct | 8B | General assistant, tool-calling | P0 |
| Llama-3.1-8B-Instruct | 8B | General assistant | P1 |
| Mistral-7B-Instruct | 7B | Fast/fallback | P2 |

### TTS Models

| Model | Parameters | Use Case | Priority |
|-------|------------|----------|----------|
| Piper voices | 5-32M | Baseline TTS, multi-language | P0 |
| Kokoro | 82M | Higher quality voices | P1 |
| MetaVoice-1B | 1B | Premium neural TTS | P2 |

## User Stories

### US-1: Basic Chat
**As a** developer
**I want to** send a prompt to a local LLM
**So that** I can get AI assistance without cloud API costs

**Acceptance Criteria**:
- POST to /v1/chat/completions returns valid response
- Response includes assistant message content
- Works with curl and browser UI

### US-2: TTS Generation
**As a** developer
**I want to** convert text to speech
**So that** I can create audio content locally

**Acceptance Criteria**:
- POST to /v1/audio/speech returns audio data
- Audio is playable WAV format
- Response time under 5 seconds for short text

### US-3: Multi-Model Support
**As a** developer
**I want to** choose between different LLM models
**So that** I can balance quality vs speed

**Acceptance Criteria**:
- Gateway routes to correct model based on request
- Multiple models can be loaded simultaneously
- Model switching doesn't require restart

## Out of Scope (v0.1)

- Training/fine-tuning
- Multi-tenant authentication
- Rate limiting
- Persistent conversation history
- Image/vision models
- Embedding API
- Audio transcription (Whisper)

## Success Metrics

### MVP Success Criteria
1. Build succeeds on Rust 1.85.0+
2. All services start and respond to health checks
3. Chat completions return valid responses
4. TTS returns playable audio
5. UI can send prompts and display responses

### Future Metrics
- Tokens/second throughput
- Time to first token
- Memory efficiency
- GPU utilization

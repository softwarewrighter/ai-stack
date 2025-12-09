# Learnings

This document captures issues encountered and their solutions for future reference.

## Rust 2024 / Dependency Issues

### HTTP Crate Version Conflicts

**Issue**: When using warp 0.3 with reqwest, the http crate versions conflict (0.2 vs 1.x), causing type mismatches like `expected warp::http::StatusCode, found reqwest::StatusCode`.

**Root Cause**: warp 0.3 depends on hyper 0.14 which uses http 0.2, while modern reqwest uses http 1.x.

**Solution**: Use warp 0.4 which is compatible with http 1.x:
```toml
warp = { version = "0.4", features = ["server"] }
```

**Prevention**: When combining HTTP frameworks, check their http crate versions in `cargo tree`.

### Axum 0.8 Body API Changes

**Issue**: `axum::body::Full` no longer exists in axum 0.8.

**Root Cause**: axum 0.8 unified body types around `axum::body::Body`.

**Solution**: Use `Body::from()` with tuple response pattern:
```rust
(
    StatusCode::OK,
    [(header::CONTENT_TYPE, "audio/wav")],
    Body::from(bytes),
).into_response()
```

**Prevention**: Check axum migration guides when updating major versions.

## Clippy Best Practices

### Never Use #[allow(dead_code)]

**Issue**: Temptation to silence dead_code warnings with `#[allow(dead_code)]`.

**Why It's Wrong**: Dead code warnings indicate real issues:
- Fields not being used that should be
- Functions that need tests
- Unused imports after refactoring

**Solution**: Fix the underlying issue:
- Use the fields (e.g., log them in stubs)
- Add tests that exercise the code
- Remove truly unused code

### Redundant Closures

**Issue**: `use_state(|| String::new())` triggers clippy warning.

**Solution**: Use function reference directly:
```rust
use_state(String::new)
```

### Identical If Blocks

**Issue**: Placeholder routing with identical branches triggers clippy.

**Solution**: If all branches are identical, simplify to single return with documentation:
```rust
/// Future: route different model prefixes to different backends
fn get_llm_target(_model: &str) -> &'static str {
    "http://localhost:9000/v1/chat/completions"
}
```

## WASM / Yew Patterns

### WASM Test Configuration

**Issue**: UI crate has no tests because WASM testing requires special setup.

**Solution**: For WASM crates, use `wasm-bindgen-test`:
```toml
[dev-dependencies]
wasm-bindgen-test = "0.3"
```

```rust
#[cfg(test)]
mod tests {
    use wasm_bindgen_test::*;
    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_something() {
        // ...
    }
}
```

**Note**: Run with `wasm-pack test --headless --firefox`

### Yew State Initialization

**Pattern**: Use function references for simple state initialization:
```rust
let state = use_state(String::new);
let state = use_state(Vec::new);
let state = use_state(|| SomeStruct::default());  // Only when needed
```

## API Design Patterns

### Stub Implementations

**Pattern**: When creating stubs, still use all input fields to avoid dead_code warnings:
```rust
async fn tts_handler(Json(req): Json<TtsRequest>) -> Response {
    info!(
        "TTS request: {} chars, voice={:?}",
        req.input.len(),  // Use input
        req.voice         // Use voice
    );
    // Generate stub response
}
```

### Test Structure

**Pattern**: Extract testable logic into pure functions:
```rust
// Testable pure function
fn find_last_user_message(messages: &[ChatMessage]) -> ChatMessage {
    // ...
}

// Handler uses pure functions
async fn chat_handler(Json(req): Json<ChatCompletionRequest>) -> Json<...> {
    let last_user = find_last_user_message(&req.messages);
    // ...
}

// Test the pure function
#[test]
fn test_find_last_user_message() {
    // Easy to test without async/HTTP
}
```

## Documentation Patterns

### Module Documentation

**Pattern**: Use `//!` for module-level docs at the top of the file:
```rust
//! LLM inference service stub exposing OpenAI-compatible chat completions API.
//! This is a placeholder that echoes input; swap in mistral.rs later.

use ...
```

### Future Enhancement Comments

**Pattern**: Document future plans in doc comments, not inline comments:
```rust
/// Determine which LLM backend to route to based on model name.
///
/// Currently all models route to the single llm-node instance.
/// Future enhancement: route different model prefixes to different backends:
/// - qwen3-* -> llm-node-gpu0
/// - llama-3-* -> llm-node-gpu1
fn get_llm_target(_model: &str) -> &'static str {
```

## Build and CI

### Strict Clippy

**Command**: Always use strict clippy in CI:
```bash
cargo clippy --all-targets --all-features -- -D warnings
```

### Pre-commit Checklist

1. `cargo build` - no errors
2. `cargo clippy --all-targets --all-features -- -D warnings` - no warnings
3. `cargo test` - all tests pass
4. `cargo fmt --all` - code formatted

//! `uh-llm` — LLM provider trait + implementations.
//!
//! v0.0.3 has one implementation: `OpenAiCompat`. It targets any
//! OpenAI-compatible `/v1/chat/completions` endpoint (OpenAI, DeepSeek,
//! local vLLM / Ollama, Azure OpenAI, etc.).
//!
//! The trait is intentionally minimal: one `chat()` method, sync.
//! Streaming arrives in v0.0.5.

pub mod openai_compat;
pub mod r#trait;

pub use openai_compat::OpenAiCompat;
pub use r#trait::*;

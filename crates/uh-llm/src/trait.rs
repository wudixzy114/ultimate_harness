//! LLM provider trait.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uh_core::transport::{Message, ToolSpec};

/// A chat request assembled by the harness.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<Message>,
    pub tools: Vec<ToolSpec>,
    pub model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
}

/// The model's response to one chat request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    /// role=assistant message (may include tool_calls)
    pub message: Message,
    /// "stop" | "tool_calls" | "length" | "error"
    pub finish_reason: String,
    pub usage: Usage,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LlmProvider {
    /// Any OpenAI-compatible `/v1/chat/completions` endpoint
    OpenAiCompat,
}

impl std::fmt::Display for LlmProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OpenAiCompat => write!(f, "openai-compat"),
        }
    }
}

/// Configuration for the LLM provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: LlmProvider,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub max_tokens: Option<u32>,
}

fn default_timeout() -> u64 {
    120
}
fn default_max_retries() -> u32 {
    3
}

/// LLM error.
#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("transport: {0}")]
    Transport(String),
    #[error("api error ({status}): {message}")]
    Api { status: u16, message: String },
    #[error("rate limit")]
    RateLimit,
    #[error("auth: invalid api key")]
    Auth,
    #[error("parse: {0}")]
    Parse(String),
    #[error("upstream: {0}")]
    Upstream(String),
    #[error("cancelled")]
    Cancelled,
}

/// LLM provider trait. One method, sync.
#[async_trait]
pub trait Llm: Send + Sync {
    fn name(&self) -> &'static str;
    fn model(&self) -> &str;
    async fn chat(&self, req: ChatRequest) -> Result<ChatResponse, LlmError>;
    /// Cancel any in-flight request from this provider. v0.0.3 only
    /// supports a single-flight; v0.0.5 may add a token.
    async fn cancel(&self);
    /// Build a default `ChatRequest` for the given system prompt and
    /// user message, with the supplied tools.
    fn build_request(
        &self,
        system: &str,
        history: &[Message],
        user_msg: &str,
        tools: Vec<ToolSpec>,
    ) -> ChatRequest {
        let mut messages = Vec::with_capacity(history.len() + 2);
        if !system.is_empty() {
            messages.push(Message {
                role: uh_core::transport::Role::System,
                content: system.to_string(),
                tool_call_id: None,
                tool_calls: None,
            });
        }
        messages.extend_from_slice(history);
        messages.push(Message {
            role: uh_core::transport::Role::User,
            content: user_msg.to_string(),
            tool_call_id: None,
            tool_calls: None,
        });
        ChatRequest {
            messages,
            tools,
            model: self.model().to_string(),
            temperature: None,
            max_tokens: None,
        }
    }
}

// re-export Value to keep imports tidy in call sites
#[allow(unused_imports)]
use serde_json::Value as _Value;

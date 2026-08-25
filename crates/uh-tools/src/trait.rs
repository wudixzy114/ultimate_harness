//! Tool trait + types.
//!
//! Borrowed from grok-build's `xai_tool_runtime`: a tool declares a
//! static spec (name / description / parameters JSON Schema) and
//! implements an async `call`.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uh_core::transport::{ToolResult, ToolSpec};

/// Default cap on tool output, sent to the model. 40 KB ≈ 10 000 tokens.
/// Matches grok-build's `DEFAULT_TOOL_OUTPUT_BYTES`.
pub const DEFAULT_TOOL_OUTPUT_BYTES: usize = 40_000;

/// Default cap on bash output. 20 KB chars ≈ 5 000 tokens.
pub const DEFAULT_BASH_OUTPUT_CHARS: usize = 20_000;

/// Tool error, recoverable (returned to the LLM as `is_error: true`).
#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("permission denied: {0}")]
    Permission(String),
    #[error("io: {0}")]
    Io(String),
    #[error("parse: {0}")]
    Parse(String),
    #[error("invalid args: {0}")]
    InvalidArgs(String),
    #[error("execution: {0}")]
    Execution(String),
}

/// A tool that the agent can call.
#[async_trait]
pub trait Tool: Send + Sync {
    /// Static spec sent to the LLM. Should not depend on runtime state.
    fn spec(&self) -> &ToolSpec;

    /// Execute the tool with parsed arguments.
    async fn call(&self, args: Value) -> Result<ToolResult, ToolError>;
}

// Re-export so call sites only need one `use uh_tools::...`.
pub use uh_core::transport::{ToolResult as TransportToolResult, ToolSpec as TransportToolSpec};

// Re-export for downstream use
pub type Spec = ToolSpec;
pub type Output = ToolResult;

/// Truncate a string to `max_bytes`, appending a marker if cut.
pub fn truncate(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        return s.to_string();
    }
    // Find a char boundary at or before max_bytes.
    let mut cut = max_bytes;
    while cut > 0 && !s.is_char_boundary(cut) {
        cut -= 1;
    }
    let marker = format!(
        "\n\n[…truncated, {} more bytes omitted…]",
        s.len().saturating_sub(cut)
    );
    let mut out = String::with_capacity(cut + marker.len());
    out.push_str(&s[..cut]);
    out.push_str(&marker);
    out
}

// Allow external code to construct a ToolResult easily.
pub fn ok_result(tool_call_id: &str, content: impl Into<String>) -> ToolResult {
    ToolResult {
        tool_call_id: tool_call_id.to_string(),
        content: content.into(),
        is_error: false,
        display: None,
    }
}

pub fn ok_result_with_display(
    tool_call_id: &str,
    content: impl Into<String>,
    display: Value,
) -> ToolResult {
    ToolResult {
        tool_call_id: tool_call_id.to_string(),
        content: content.into(),
        is_error: false,
        display: Some(display),
    }
}

pub fn err_result(tool_call_id: &str, message: impl Into<String>) -> ToolResult {
    ToolResult {
        tool_call_id: tool_call_id.to_string(),
        content: message.into(),
        is_error: true,
        display: None,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRequest {
    pub id: String,
    pub name: String,
    pub arguments: Value,
}

// Pre-call / post-call hooks are intentionally not exposed in v0.0.3.
// v0.0.4+ plan-first skill will introduce them; the shape is reserved.
// Keeping the trait shape small: tool call returns Result; nothing
// injects in between.

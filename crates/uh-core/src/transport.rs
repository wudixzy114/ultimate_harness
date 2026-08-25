//! Transport types — the wire format between the daemon and the web UI.
//!
//! Borrowed shape from dsh (Model + Llm types) and grok-build (the
//! `xai_tool_runtime` types). Kept minimal: v0.0.3 is text-only, no
//! multi-modal content blocks.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// WebSocket envelope. One direction, both request/response and events.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WsMessage {
    /// Client → Server
    Request {
        id: Uuid,
        method: String,
        params: Value,
    },
    /// Server → Client (response to a Request)
    Response {
        id: Uuid,
        result: Value,
        error: Option<WsError>,
    },
    /// Server → Client (push, no request id)
    Event {
        event: String,
        data: Value,
    },
    /// Client → Server (cancel a pending request)
    Cancel { id: Uuid },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsError {
    pub code: i32,
    pub message: String,
    #[serde(default)]
    pub data: Option<Value>,
}

// Standard error codes.
pub const ERR_INTERNAL: i32 = 1000;
pub const ERR_PROTOCOL: i32 = 1001;
pub const ERR_NO_METHOD: i32 = 1002;
pub const ERR_BAD_ARGS: i32 = 1003;
pub const ERR_LLM: i32 = 1004;
pub const ERR_TOOL: i32 = 1005;
pub const ERR_STATE: i32 = 1006;

// ── LLM-level types (shared between daemon and Llm impls) ──────────

/// A role in a conversation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

/// One message in a conversation.
///
/// For `role=assistant`, `content` is the model's text and `tool_calls`
/// lists the model's tool invocations (if any).
/// For `role=tool`, `content` is the tool output and `tool_call_id`
/// references the originating call.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message {
    pub role: Role,
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

/// A tool call issued by the model.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    /// Arguments as JSON object (already parsed).
    pub arguments: Value,
}

/// The result of a tool execution.
///
/// `content` is the model-facing text (e.g. file contents, error message).
/// `display` is optional UI-only metadata (e.g. file path, diff).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolResult {
    pub tool_call_id: String,
    pub content: String,
    #[serde(default)]
    pub is_error: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display: Option<Value>,
}

/// Tool specification sent to the LLM.
/// `parameters` is a JSON Schema object.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolSpec {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

// ── Web-facing event payloads ────────────────────────────────────

/// Event: a user message arrived.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMessageEvent {
    pub message: Message,
}

/// Event: assistant message produced.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantMessageEvent {
    pub message: Message,
}

/// Event: a tool call was made.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallEvent {
    pub tool_call: ToolCall,
}

/// Event: a tool result is back.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResultEvent {
    pub tool_call_id: String,
    pub result: ToolResult,
}

/// Event: turn boundaries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnStartEvent {
    pub turn_id: Uuid,
    pub user_message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnEndEvent {
    pub turn_id: Uuid,
    pub reason: String,
    pub finished_at: DateTime<Utc>,
}

/// A snapshot of the session state (for `get_session`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSnapshot {
    pub session_id: Uuid,
    pub messages: Vec<Message>,
    pub created_at: DateTime<Utc>,
}

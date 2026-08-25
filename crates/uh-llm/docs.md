# `uh-llm` — LLM provider trait + OpenAI compat implementation

> **设计先行**。本文件先于 `src/*.rs` 存在。
> 详细架构见 `crates/uh-daemon/docs.md`，本文件只覆盖 LLM 这一层。

## 缘由

v0.0.3 要让用户**配置 Base URL + API Key 就能跑真模型**。Mock LLM 没意义——
它只能证明流程跑通，但用户真正想测的是真模型响应。

OpenAI 的 `/v1/chat/completions` 是事实标准：OpenAI、DeepSeek、本地 vLLM、
Ollama、Azure OpenAI 全都兼容。v0.0.3 只实现这一种，v0.0.5+ 加 Anthropic。

## 设计哲学

1. **Trait 极简** —— 一个 `chat` 方法，传入 messages + tools，返回 response。
2. **不发明消息格式** —— 复用 OpenAI 的 message / content / tool_call 形状。
3. **不做流式** —— v0.0.3 同步 `chat()` 等到底。v0.0.5+ 加 `chat_stream()`。
4. **不做 cache 控制** —— 那是 v0.0.5+ 的事（Anthropic 优先）。
5. **不做工具调用循环** —— 那是 harness 的责任（`uh-daemon::run_turn`）。
   LLM 只负责"看到 messages + tools，返回下一条 response（含 tool_calls 或
   final text）"。
6. **不做 retry** —— v0.0.3 透传错误，v0.0.4+ 在 trait 上加 retry policy。

## Trait（`src/traits.rs`）

```rust
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uh_core::ids::SessionId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,            // "system" | "user" | "assistant" | "tool"
    pub content: String,       // v0.0.3 纯文本；v0.0.5+ 支持多 content block
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,  // role=tool 时
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Role { System, User, Assistant, Tool }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,  // JSON Schema
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<Message>,
    pub tools: Vec<ToolSpec>,
    pub model: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub message: Message,       // role=assistant
    pub finish_reason: String,  // "stop" | "tool_calls" | "length" | "error"
    pub usage: Usage,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("transport: {0}")]
    Transport(String),
    #[error("api: {0}")]
    Api(String),
    #[error("rate-limit")]
    RateLimit,
    #[error("auth: invalid api key")]
    Auth,
    #[error("parse: {0}")]
    Parse(String),
    #[error("upstream: {0}")]
    Upstream(String),
}

#[async_trait]
pub trait Llm: Send + Sync {
    fn name(&self) -> &str;     // "openai-compat" / "anthropic"
    fn model(&self) -> &str;
    async fn chat(&self, req: ChatRequest) -> Result<ChatResponse, LlmError>;
}
```

## OpenAI Compat 实现（v0.0.3）

`src/openai_compat.rs` —— 调 `POST {base_url}/v1/chat/completions`。

实现要点：
- `reqwest` POST，header `Authorization: Bearer {api_key}`
- body 字段对齐 OpenAI 规范（`messages` / `tools` / `temperature` / `max_tokens`）
- response parse 成 `ChatResponse`
- 错误处理：401 → `LlmError::Auth`；429 → `LlmError::RateLimit`
- 同步等待；不实现流式

覆盖的 provider：
- OpenAI（`https://api.openai.com`）
- DeepSeek（`https://api.deepseek.com`）
- 任何 OpenAI 兼容端点（vLLM / Ollama / LM Studio / Azure OpenAI mode）

## Anthropic 实现（v0.0.5+，不在 v0.0.3 范围）

`src/anthropic.rs` —— 调 `POST {base_url}/v1/messages`。
因为 Anthropic API 跟 OpenAI **不兼容**（不同的 request/response shape），
需要单独实现。但**用同一个 `Llm` trait**，所以 harness core 不需要改。

Anthropic 优势：prompt cache（cache_control 标记），这是我们 v0.0.5 设计
cache-aware context 的基础。

## 文件清单

### 新建
- `crates/uh-llm/Cargo.toml`
- `crates/uh-llm/src/lib.rs`
- `crates/uh-llm/src/traits.rs`         — Llm / Message / ToolSpec / ToolCall / ChatRequest / ChatResponse
- `crates/uh-llm/src/openai_compat.rs`  — v0.0.3 唯一实现
- **`crates/uh-llm/docs.md`**           — 本文件

### 推迟（v0.0.5+）
- `crates/uh-llm/src/anthropic.rs`
- `crates/uh-llm/src/streaming.rs`
- `crates/uh-llm/src/retry.rs`
- `crates/uh-llm/src/cache.rs`          — prompt cache 工具

## 验证清单（v0.0.3）

- [ ] `cargo test -p uh-llm` —— Llm trait 测试 + OpenAI compat 解析测试
- [ ] 配置 DeepSeek API key 能正常聊天
- [ ] 配置 OpenAI API key 能正常聊天
- [ ] 配置 vLLM（本地 11434）能正常聊天
- [ ] 错误码正确映射（401 / 429 / 5xx）
- [ ] `uh-devtool check-design crates/uh-llm` exit 0

## 范围外（明确不做）

- 流式响应（v0.0.5+）
- Anthropic provider（v0.0.5+）
- Prompt cache（v0.0.5+）
- 工具调用循环（harness 责任）
- Retry policy（v0.0.4+）
- 多模型路由 / fallback（v0.4+）

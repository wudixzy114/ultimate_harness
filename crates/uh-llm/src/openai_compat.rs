//! OpenAI-compatible provider.
//!
//! Targets any endpoint that speaks OpenAI's `/v1/chat/completions` API.
//! This covers: OpenAI, DeepSeek, vLLM, Ollama (with compat), LM Studio,
//! Azure OpenAI, etc.
//!
//! Borrowed shape from grok-build's `async-openai = "0.33"` usage, but
//! we go with `reqwest` directly to keep the request surface explicit
//! and the dependency surface small.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::sync::Mutex;

use uh_core::transport::{Message, Role, ToolCall};

use crate::r#trait::{ChatRequest, ChatResponse, Llm, LlmConfig, LlmError, Usage};

pub struct OpenAiCompat {
    config: LlmConfig,
    client: reqwest::Client,
    cancel_flag: Arc<AtomicBool>,
    // We hold an in-flight request handle so `cancel()` can interrupt it.
    inflight: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl OpenAiCompat {
    pub fn new(config: LlmConfig) -> Result<Self, LlmError> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(|e| LlmError::Transport(e.to_string()))?;
        Ok(Self {
            config,
            client,
            cancel_flag: Arc::new(AtomicBool::new(false)),
            inflight: Arc::new(Mutex::new(None)),
        })
    }
}

#[async_trait]
impl Llm for OpenAiCompat {
    fn name(&self) -> &'static str {
        "openai-compat"
    }

    fn model(&self) -> &str {
        &self.config.model
    }

    async fn chat(&self, req: ChatRequest) -> Result<ChatResponse, LlmError> {
        // Build the OpenAI request body.
        let body = build_request_body(&req);

        let url = format!(
            "{}/v1/chat/completions",
            self.config.base_url.trim_end_matches('/')
        );

        let mut last_err: Option<LlmError> = None;
        for attempt in 0..=self.config.max_retries {
            if self.cancel_flag.load(Ordering::Relaxed) {
                return Err(LlmError::Cancelled);
            }

            let mut req_builder = self
                .client
                .post(&url)
                .header("Authorization", format!("Bearer {}", self.config.api_key))
                .header("Content-Type", "application/json")
                .json(&body);

            if self.cancel_flag.load(Ordering::Relaxed) {
                return Err(LlmError::Cancelled);
            }

            let response = match req_builder.send().await {
                Ok(r) => r,
                Err(e) if e.is_timeout() => {
                    last_err = Some(LlmError::Transport(format!("timeout: {e}")));
                    continue;
                }
                Err(e) => return Err(LlmError::Transport(e.to_string())),
            };

            let status = response.status();
            if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                last_err = Some(LlmError::RateLimit);
                // simple backoff
                tokio::time::sleep(std::time::Duration::from_millis(500 * (1 << attempt))).await;
                continue;
            }
            if status == reqwest::StatusCode::UNAUTHORIZED
                || status == reqwest::StatusCode::FORBIDDEN
            {
                return Err(LlmError::Auth);
            }
            if !status.is_success() {
                let body = response.text().await.unwrap_or_default();
                return Err(LlmError::Api {
                    status: status.as_u16(),
                    message: body,
                });
            }

            let text = response
                .text()
                .await
                .map_err(|e| LlmError::Transport(e.to_string()))?;
            return parse_response(&text, &self.config.model);
        }
        Err(last_err.unwrap_or(LlmError::Upstream("max retries exhausted".into())))
    }

    async fn cancel(&self) {
        self.cancel_flag.store(true, Ordering::Relaxed);
    }
}

// ── Request body shaping (OpenAI format) ──────────────────────────

#[derive(Serialize)]
struct OpenAiRequest<'a> {
    model: &'a str,
    messages: Vec<OpenAiMessage>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<OpenAiTool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}

#[derive(Serialize)]
struct OpenAiMessage {
    role: &'static str,
    #[serde(skip_serializing_if = "String::is_empty")]
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<OpenAiToolCall>>,
}

#[derive(Serialize)]
struct OpenAiToolCall {
    id: String,
    #[serde(rename = "type")]
    kind: &'static str,
    function: OpenAiFunction,
}

#[derive(Serialize)]
struct OpenAiFunction {
    name: String,
    arguments: String, // JSON-encoded string
}

#[derive(Serialize)]
struct OpenAiTool {
    #[serde(rename = "type")]
    kind: &'static str,
    function: OpenAiToolFunction,
}

#[derive(Serialize)]
struct OpenAiToolFunction {
    name: String,
    description: String,
    parameters: Value,
}

fn build_request_body(req: &ChatRequest) -> Value {
    let mut messages: Vec<OpenAiMessage> = Vec::with_capacity(req.messages.len());
    for m in &req.messages {
        let role = match m.role {
            Role::System => "system",
            Role::User => "user",
            Role::Assistant => "assistant",
            Role::Tool => "tool",
        };
        let tool_calls = m.tool_calls.as_ref().map(|tcs| {
            tcs.iter()
                .map(|c| OpenAiToolCall {
                    id: c.id.clone(),
                    kind: "function",
                    function: OpenAiFunction {
                        name: c.name.clone(),
                        arguments: c.arguments.to_string(),
                    },
                })
                .collect()
        });
        messages.push(OpenAiMessage {
            role,
            content: m.content.clone(),
            tool_call_id: m.tool_call_id.clone(),
            tool_calls,
        });
    }
    let tools: Vec<OpenAiTool> = req
        .tools
        .iter()
        .map(|t| OpenAiTool {
            kind: "function",
            function: OpenAiToolFunction {
                name: t.name.clone(),
                description: t.description.clone(),
                parameters: t.parameters.clone(),
            },
        })
        .collect();
    let body = OpenAiRequest {
        model: &req.model,
        messages,
        tools,
        temperature: req.temperature,
        max_tokens: req.max_tokens,
    };
    serde_json::to_value(&body).unwrap_or_else(|_| json!({}))
}

// ── Response parsing ─────────────────────────────────────────────

#[derive(Deserialize)]
struct OpenAiResponse {
    choices: Vec<OpenAiChoice>,
    #[serde(default)]
    usage: Option<OpenAiUsage>,
}

#[derive(Deserialize)]
struct OpenAiChoice {
    message: OpenAiResponseMessage,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct OpenAiResponseMessage {
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    tool_calls: Option<Vec<OpenAiResponseToolCall>>,
}

#[derive(Deserialize)]
struct OpenAiResponseToolCall {
    id: String,
    function: OpenAiResponseFunction,
}

#[derive(Deserialize)]
struct OpenAiResponseFunction {
    name: String,
    /// OpenAI sends arguments as a JSON-encoded string. We deserialize
    /// it eagerly so the LLM consumer always sees a JSON object.
    arguments: String,
}

#[derive(Deserialize)]
struct OpenAiUsage {
    #[serde(default)]
    prompt_tokens: u32,
    #[serde(default)]
    completion_tokens: u32,
    #[serde(default)]
    total_tokens: u32,
}

fn parse_response(text: &str, model: &str) -> Result<ChatResponse, LlmError> {
    let parsed: OpenAiResponse = serde_json::from_str(text)
        .map_err(|e| LlmError::Parse(format!("response json: {e}")))?;

    let choice = parsed
        .choices
        .into_iter()
        .next()
        .ok_or_else(|| LlmError::Parse("no choices in response".into()))?;

    let content = choice.message.content.unwrap_or_default();
    let tool_calls = choice.message.tool_calls.map(|tcs| {
        tcs.into_iter()
            .map(|tc| {
                let args = match serde_json::from_str::<Value>(&tc.function.arguments) {
                    Ok(v) => v,
                    Err(_) => Value::Object(serde_json::Map::new()),
                };
                ToolCall {
                    id: tc.id,
                    name: tc.function.name,
                    arguments: args,
                }
            })
            .collect::<Vec<_>>()
    });

    let message = Message {
        role: Role::Assistant,
        content,
        tool_call_id: None,
        tool_calls,
    };

    let usage = parsed
        .usage
        .map(|u| Usage {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
        })
        .unwrap_or_default();

    Ok(ChatResponse {
        message,
        finish_reason: choice.finish_reason.unwrap_or_else(|| "stop".into()),
        usage,
    })
}

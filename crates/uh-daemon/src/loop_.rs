//! The agent loop — drives one turn from user message to assistant
//! final response.
//!
//! Borrowed shape from dsh's `ReactLoopAgent` (turn / step / phase state),
//! simplified to v0.0.3 essentials:
//! - one LLM call per step
//! - tool calls execute sequentially (parallel is v0.0.4)
//! - error from a tool is sent back to the LLM as `is_error: true`
//! - the loop ends when the LLM returns no `tool_calls` (or hits `length`)

use std::sync::Arc;

use serde_json::{Value, json};
use tracing::{debug, info, warn};
use uh_core::transport::{
    AssistantMessageEvent, Message, Role, ToolCallEvent, ToolResultEvent, TurnEndEvent,
    TurnStartEvent,
};
use uh_llm::ChatRequest;
use uuid::Uuid;

use crate::peers;
use crate::state::AppState;

/// The system prompt the LLM gets. Kept short and tool-agnostic — the
/// LLM is told it has tools and asked to use them.
const SYSTEM_PROMPT: &str = "You are a coding assistant with shell, file, and editing tools. \
Be concise. When asked to modify a file, read it first, then make targeted edits. \
After each action, briefly state what you did.";

pub async fn run_turn(state: Arc<AppState>, user_message: String) -> Result<Uuid, String> {
    let turn_id = Uuid::new_v4();

    // 1. Append user message
    {
        let mut s = state.session.write().await;
        s.messages.push(Message {
            role: Role::User,
            content: user_message.clone(),
            tool_call_id: None,
            tool_calls: None,
        });
    }

    // 2. Emit turn_start
    peers::broadcast(
        &state.peers,
        "turn_start",
        serde_json::to_value(TurnStartEvent {
            turn_id,
            user_message: user_message.clone(),
        })
        .unwrap_or(Value::Null),
    )
    .await;

    let mut reason = "stop".to_string();
    let mut safety_iterations = 0usize;
    const MAX_STEPS: usize = 50;

    // 3. Step loop
    loop {
        safety_iterations += 1;
        if safety_iterations > MAX_STEPS {
            reason = "max-steps".into();
            warn!(turn_id = %turn_id, "agent loop hit MAX_STEPS");
            break;
        }

        // Build request from current session
        let (messages, tools) = {
            let s = state.session.read().await;
            let tool_specs = if state.config.tools.enable.is_empty() {
                state.tools.specs()
            } else {
                state.tools.specs_filtered(&state.config.tools.enable)
            };
            (s.messages.clone(), tool_specs)
        };

        let req = ChatRequest {
            messages,
            tools,
            model: state.llm.model().to_string(),
            temperature: state.config.llm.temperature,
            max_tokens: state.config.llm.max_tokens,
        };

        // Call LLM
        debug!(turn_id = %turn_id, "calling LLM");
        let resp = match state.llm.chat(req).await {
            Ok(r) => r,
            Err(e) => {
                reason = format!("llm_error: {e}");
                warn!(turn_id = %turn_id, error = %e, "LLM error");
                break;
            }
        };

        // Emit assistant message
        let msg = resp.message.clone();
        peers::broadcast(
            &state.peers,
            "assistant_message",
            serde_json::to_value(AssistantMessageEvent { message: msg.clone() }).unwrap_or(Value::Null),
        )
        .await;

        // Append assistant message to session
        {
            let mut s = state.session.write().await;
            s.messages.push(msg.clone());
        }

        // 4. If no tool calls, we're done
        let tool_calls = msg.tool_calls.clone().unwrap_or_default();
        if tool_calls.is_empty() {
            reason = resp.finish_reason.clone();
            break;
        }

        // 5. Execute each tool call sequentially
        for call in &tool_calls {
            peers::broadcast(
                &state.peers,
                "tool_call",
                serde_json::to_value(ToolCallEvent { tool_call: call.clone() }).unwrap_or(Value::Null),
            )
            .await;

            let result = state.tools.execute(&call.id, &call.name, call.arguments.clone()).await;

            peers::broadcast(
                &state.peers,
                "tool_result",
                serde_json::to_value(ToolResultEvent {
                    tool_call_id: result.tool_call_id.clone(),
                    result: result.clone(),
                })
                .unwrap_or(Value::Null),
            )
            .await;

            // Append tool message to session
            let tool_message = Message {
                role: Role::Tool,
                content: result.content.clone(),
                tool_call_id: Some(result.tool_call_id.clone()),
                tool_calls: None,
            };
            {
                let mut s = state.session.write().await;
                s.messages.push(tool_message);
            }
        }

        // 6. Continue loop
    }

    // 7. Emit turn_end
    peers::broadcast(
        &state.peers,
        "turn_end",
        serde_json::to_value(TurnEndEvent {
            turn_id,
            reason: reason.clone(),
            finished_at: chrono::Utc::now(),
        })
        .unwrap_or(Value::Null),
    )
    .await;

    info!(turn_id = %turn_id, reason = %reason, "turn finished");
    Ok(turn_id)
}

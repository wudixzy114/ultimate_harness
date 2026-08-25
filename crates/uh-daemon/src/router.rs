//! Method dispatcher — given a `method` name and params, run the
//! right handler and return a JSON result.
//!
//! v0.0.3 methods:
//!   - `send_message`  { content }   → spawns a turn, returns turn_id
//!   - `get_session`   {}            → session snapshot
//!   - `list_tools`    {}            → tool specs
//!   - `ping`          {}            → pong
//!
//! `send_message` is the only one that triggers the agent loop; the
//! others are read-only.

use std::sync::Arc;

use serde_json::{Value, json};
use tracing::info;
use uh_core::transport::SessionSnapshot;
use uuid::Uuid;

use crate::loop_::run_turn;
use crate::state::AppState;

pub async fn dispatch(
    method: &str,
    params: Value,
    state: Arc<AppState>,
) -> Result<Value, (i32, String)> {
    match method {
        "ping" => Ok(json!({ "pong": true, "ts": chrono::Utc::now().timestamp_millis() })),

        "list_tools" => {
            let specs = if state.config.tools.enable.is_empty() {
                state.tools.specs()
            } else {
                state.tools.specs_filtered(&state.config.tools.enable)
            };
            Ok(json!({ "tools": specs }))
        }

        "get_session" => {
            let s = state.session.read().await;
            Ok(json!({
                "session": SessionSnapshot {
                    session_id: s.id,
                    messages: s.messages.clone(),
                    created_at: s.created_at,
                }
            }))
        }

        "send_message" => {
            let content = params
                .get("content")
                .and_then(|v| v.as_str())
                .ok_or((1003, "missing 'content' (string)".to_string()))?;
            if content.trim().is_empty() {
                return Err((1003, "content is empty".into()));
            }
            let content = content.to_string();
            let state2 = Arc::clone(&state);
            // Spawn the turn in the background so the request returns
            // immediately. The client follows the conversation via
            // `turn_start` / `assistant_message` / `tool_call` / `tool_result` / `turn_end` events.
            tokio::spawn(async move {
                if let Err(e) = run_turn(state2, content).await {
                    tracing::error!(error = %e, "turn failed");
                }
            });
            let turn_id = Uuid::new_v4();
            info!(turn_id = %turn_id, "queued turn");
            Ok(json!({ "turn_id": turn_id }))
        }

        _ => Err((1002, format!("unknown method: {method}"))),
    }
}

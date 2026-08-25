//! ToolRegistry — a static map of name → Tool.

use std::collections::HashMap;
use std::sync::Arc;

use serde_json::Value;
use tracing::warn;
use uh_core::transport::{ToolResult, ToolSpec};

use crate::r#trait::Tool;

pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: Box<dyn Tool>) {
        let name = tool.spec().name.clone();
        if self.tools.insert(name.clone(), tool).is_some() {
            warn!(name = %name, "tool with duplicate name registered; overriding");
        }
    }

    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.get(name).map(|t| t.as_ref())
    }

    /// Specs for all registered tools, for sending to the LLM.
    pub fn specs(&self) -> Vec<ToolSpec> {
        self.tools.values().map(|t| t.spec().clone()).collect()
    }

    /// Specs filtered by name list (for selective enabling).
    pub fn specs_filtered(&self, names: &[String]) -> Vec<ToolSpec> {
        self.tools
            .values()
            .filter(|t| names.contains(&t.spec().name))
            .map(|t| t.spec().clone())
            .collect()
    }

    /// Execute a tool call. Returns the model-facing result.
    /// Truncates the result content to `DEFAULT_TOOL_OUTPUT_BYTES`.
    pub async fn execute(
        &self,
        tool_call_id: &str,
        name: &str,
        args: Value,
    ) -> ToolResult {
        let Some(tool) = self.get(name) else {
            return ToolResult {
                tool_call_id: tool_call_id.to_string(),
                content: format!("Error: unknown tool '{name}'"),
                is_error: true,
                display: None,
            };
        };
        match tool.call(args).await {
            Ok(mut r) => {
                if r.content.len() > crate::r#trait::DEFAULT_TOOL_OUTPUT_BYTES {
                    r.content = crate::r#trait::truncate(
                        &r.content,
                        crate::r#trait::DEFAULT_TOOL_OUTPUT_BYTES,
                    );
                }
                r
            }
            Err(e) => ToolResult {
                tool_call_id: tool_call_id.to_string(),
                content: format!("Error: {e}"),
                is_error: true,
                display: None,
            },
        }
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Shared, read-only handle that callers can pass around.
pub type SharedRegistry = Arc<ToolRegistry>;

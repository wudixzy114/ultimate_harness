//! `uh-tools` — Tool trait, registry, and 5 basic tools.
//!
//! Borrowed shape from grok-build's `xai_tool_runtime`:
//! - `async fn call(args) -> Result<Content, ToolError>`
//! - static `spec()` at registration
//! - default output cap of 40KB
//!
//! Borrowed pipeline idea from dsh's 7-stage tool execution: we have a
//! `pre_call` hook point (waterfall) and a `post_call` hook point. v0.0.3
//! uses them only for output truncation. v0.0.4+ plan-first skill hooks
//! in here.

pub mod bash;
pub mod edit_file;
pub mod list_dir;
pub mod read_file;
pub mod write_file;

mod r#trait;
mod registry;
mod schema;

pub use r#trait::*;
pub use registry::*;
pub use schema::*;

/// Standard tools, registered in the order the model sees them.
pub fn default_registry() -> ToolRegistry {
    let mut reg = ToolRegistry::new();
    reg.register(Box::new(read_file::ReadFileTool));
    reg.register(Box::new(write_file::WriteFileTool));
    reg.register(Box::new(edit_file::EditFileTool));
    reg.register(Box::new(list_dir::ListDirTool));
    reg.register(Box::new(bash::BashTool));
    reg
}

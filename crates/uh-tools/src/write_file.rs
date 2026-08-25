//! `write_file` — write content to a file (overwrites existing).

use std::path::PathBuf;

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;
use uh_core::transport::ToolSpec;

use crate::r#trait::{Tool, ToolError, ok_result_with_display};
use crate::schema;

pub struct WriteFileTool;

#[derive(Deserialize)]
struct Args {
    path: String,
    content: String,
}

#[async_trait]
impl Tool for WriteFileTool {
    fn spec(&self) -> &ToolSpec {
        use std::sync::OnceLock;
        static SPEC: OnceLock<ToolSpec> = OnceLock::new();
        SPEC.get_or_init(|| ToolSpec {
            name: "write_file".into(),
            description: "Write content to a file, overwriting any existing content. Creates parent directories if needed.".into(),
            parameters: schema::object_schema(
                serde_json::json!({
                    "path": schema::string_arg("File path (absolute or relative to project root)"),
                    "content": schema::string_arg("Full file content to write"),
                }),
                &["path", "content"],
            ),
        })
    }

    async fn call(&self, args: Value) -> Result<uh_core::transport::ToolResult, ToolError> {
        let args: Args = serde_json::from_value(args)
            .map_err(|e| ToolError::InvalidArgs(e.to_string()))?;
        let path = PathBuf::from(&args.path);
        let display_path = path.display().to_string();
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| ToolError::Io(format!("mkdir {}: {e}", parent.display())))?;
        }
        let bytes = args.content.len();
        tokio::fs::write(&path, args.content.as_bytes())
            .await
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::PermissionDenied => {
                    ToolError::Permission(display_path.clone())
                }
                _ => ToolError::Io(format!("{display_path}: {e}")),
            })?;
        Ok(ok_result_with_display(
            "write_file",
            format!("wrote {bytes} bytes to {display_path}"),
            serde_json::json!({ "path": display_path, "bytes": bytes }),
        ))
    }
}

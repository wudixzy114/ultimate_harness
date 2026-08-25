//! `read_file` — read a file's content.

use std::path::PathBuf;

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;

use uh_core::transport::ToolSpec;

use crate::r#trait::{Tool, ToolError, err_result, ok_result_with_display};
use crate::schema;

pub struct ReadFileTool;

#[derive(Deserialize)]
struct Args {
    path: String,
    #[serde(default)]
    offset: Option<u64>,
    #[serde(default)]
    limit: Option<u64>,
}

#[async_trait]
impl Tool for ReadFileTool {
    fn spec(&self) -> &ToolSpec {
        use std::sync::OnceLock;
        static SPEC: OnceLock<ToolSpec> = OnceLock::new();
        SPEC.get_or_init(|| ToolSpec {
            name: "read_file".into(),
            description: "Read a file's content. For large files, use `offset` and `limit` to read a slice. Paths are relative to the project root unless absolute.".into(),
            parameters: schema::object_schema(
                serde_json::json!({
                    "path": schema::string_arg("File path (absolute or relative to project root)"),
                    "offset": schema::int_arg("Line offset to start reading from (0-indexed, optional)"),
                    "limit": schema::int_arg("Maximum number of lines to read (optional)"),
                }),
                &["path"],
            ),
        })
    }

    async fn call(&self, args: Value) -> Result<uh_core::transport::ToolResult, ToolError> {
        let args: Args = serde_json::from_value(args)
            .map_err(|e| ToolError::InvalidArgs(e.to_string()))?;
        let path = PathBuf::from(&args.path);
        let display_path = path.display().to_string();
        let content = tokio::fs::read_to_string(&path)
            .await
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::NotFound => ToolError::NotFound(display_path.clone()),
                std::io::ErrorKind::PermissionDenied => {
                    ToolError::Permission(display_path.clone())
                }
                _ => ToolError::Io(format!("{display_path}: {e}")),
            })?;
        let body = if let (Some(off), Some(lim)) = (args.offset, args.limit) {
            apply_offset_limit(&content, off, lim)
        } else {
            content
        };
        Ok(ok_result_with_display(
            "read_file",
            body,
            serde_json::json!({ "path": display_path }),
        ))
    }
}

fn apply_offset_limit(content: &str, offset: u64, limit: u64) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let start = (offset as usize).min(lines.len());
    let end = (start + limit as usize).min(lines.len());
    let mut out = String::new();
    for (i, line) in lines[start..end].iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        out.push_str(&format!("{:>6}  {}", start + i + 1, line));
    }
    out
}

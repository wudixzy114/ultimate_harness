//! `edit_file` — targeted find-and-replace edit.

use std::path::PathBuf;

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;
use uh_core::transport::ToolSpec;

use crate::r#trait::{Tool, ToolError, ok_result_with_display};
use crate::schema;

pub struct EditFileTool;

#[derive(Deserialize)]
struct Args {
    path: String,
    old: String,
    new: String,
    #[serde(default)]
    replace_all: Option<bool>,
}

#[async_trait]
impl Tool for EditFileTool {
    fn spec(&self) -> &ToolSpec {
        use std::sync::OnceLock;
        static SPEC: OnceLock<ToolSpec> = OnceLock::new();
        SPEC.get_or_init(|| ToolSpec {
            name: "edit_file".into(),
            description: "Edit a file by replacing one occurrence of `old` with `new`. By default, fails if `old` doesn't appear or appears more than once; set `replace_all: true` to allow multiple replacements.".into(),
            parameters: schema::object_schema(
                serde_json::json!({
                    "path": schema::string_arg("File path"),
                    "old": schema::string_arg("Exact text to find (must match the file content)"),
                    "new": schema::string_arg("Replacement text"),
                    "replace_all": schema::bool_arg("If true, replace every occurrence; default false (require exactly one)"),
                }),
                &["path", "old", "new"],
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
                _ => ToolError::Io(format!("{display_path}: {e}")),
            })?;
        let count = content.matches(&args.old).count();
        let replace_all = args.replace_all.unwrap_or(false);
        if count == 0 {
            return Err(ToolError::Parse(format!(
                "'old' not found in {display_path}"
            )));
        }
        if count > 1 && !replace_all {
            return Err(ToolError::Parse(format!(
                "'old' appears {count} times in {display_path}; pass replace_all=true to replace all"
            )));
        }
        let new_content = if replace_all {
            content.replace(&args.old, &args.new)
        } else {
            content.replacen(&args.old, &args.new, 1)
        };
        tokio::fs::write(&path, new_content.as_bytes())
            .await
            .map_err(|e| ToolError::Io(format!("{display_path}: {e}")))?;
        Ok(ok_result_with_display(
            "edit_file",
            format!(
                "edited {display_path} ({} replacement{})",
                count,
                if count == 1 { "" } else { "s" }
            ),
            serde_json::json!({ "path": display_path, "replacements": count }),
        ))
    }
}

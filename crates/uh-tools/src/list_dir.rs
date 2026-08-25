//! `list_dir` — list directory entries.

use std::path::PathBuf;

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;
    use tokio::fs;
use uh_core::transport::ToolSpec;

use crate::r#trait::{Tool, ToolError, ok_result_with_display};
use crate::schema;

pub struct ListDirTool;

#[derive(Deserialize)]
struct Args {
    path: String,
    #[serde(default)]
    recursive: Option<bool>,
}

#[async_trait]
impl Tool for ListDirTool {
    fn spec(&self) -> &ToolSpec {
        use std::sync::OnceLock;
        static SPEC: OnceLock<ToolSpec> = OnceLock::new();
        SPEC.get_or_init(|| ToolSpec {
            name: "list_dir".into(),
            description: "List files and directories. With `recursive: true`, walks subdirectories. Skips hidden files and common build artifacts (target, node_modules, .git).".into(),
            parameters: schema::object_schema(
                serde_json::json!({
                    "path": schema::string_arg("Directory path (absolute or relative to project root)"),
                    "recursive": schema::bool_arg("If true, recurse into subdirectories (default false)"),
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
        if !path.is_dir() {
            return Err(ToolError::NotFound(format!("not a directory: {display_path}")));
        }
        let recursive = args.recursive.unwrap_or(false);
        let entries = if recursive {
            list_recursive(&path).await?
        } else {
            list_flat(&path).await?
        };
        let count = entries.lines().count();
        Ok(ok_result_with_display(
            "list_dir",
            entries,
            serde_json::json!({ "path": display_path, "count": count }),
        ))
    }
}

async fn list_flat(path: &PathBuf) -> Result<String, ToolError> {
    let mut read = fs::read_dir(path)
        .await
        .map_err(|e| ToolError::Io(format!("{}: {e}", path.display())))?;
    let mut names: Vec<String> = Vec::new();
    while let Some(entry) = read
        .next_entry()
        .await
        .map_err(|e| ToolError::Io(format!("{}: {e}", path.display())))?
    {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        if is_excluded_dir(&name) {
            continue;
        }
        let kind = match entry.file_type().await {
            Ok(t) if t.is_dir() => "dir",
            Ok(t) if t.is_file() => "file",
            _ => "other",
        };
        names.push(format!("{name} ({kind})"));
    }
    names.sort();
    Ok(names.join("\n"))
}

fn list_recursive<'a>(path: &'a PathBuf) -> futures::future::BoxFuture<'a, Result<String, ToolError>> {
    Box::pin(async move {
        let mut out = String::new();
        walk(path, 0, &mut out).await?;
        Ok(out)
    })
}

fn walk<'a>(
    path: &'a PathBuf,
    depth: usize,
    out: &'a mut String,
) -> futures::future::BoxFuture<'a, Result<(), ToolError>> {
    Box::pin(async move {
        let mut read = fs::read_dir(path)
            .await
            .map_err(|e| ToolError::Io(format!("{}: {e}", path.display())))?;
        let mut entries: Vec<_> = Vec::new();
        while let Some(e) = read
            .next_entry()
            .await
            .map_err(|err| ToolError::Io(format!("{}: {err}", path.display())))?
        {
            entries.push(e);
        }
        entries.sort_by_key(|e| e.file_name());
        for entry in &entries {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') {
                continue;
            }
            let ft = match entry.file_type().await {
                Ok(t) => t,
                Err(_) => continue,
            };
            let indent = "  ".repeat(depth);
            if ft.is_dir() {
                if is_excluded_dir(&name) {
                    continue;
                }
                if !out.is_empty() {
                    out.push('\n');
                }
                out.push_str(&format!("{indent}{name}/\n"));
                walk(&entry.path(), depth + 1, out).await?;
            } else if ft.is_file() {
                out.push_str(&format!("{indent}{name}\n"));
            }
        }
        Ok(())
    })
}

fn is_excluded_dir(name: &str) -> bool {
    matches!(
        name,
        "target" | "node_modules" | ".git" | "dist" | "build" | ".next" | ".turbo"
    )
}

//! `bash` — execute a shell command and capture stdout + stderr + exit code.
//!
//! v0.0.3: no sandbox. v0.0.1+ adds OS-native sandbox (Landlock/Seatbelt/Job Objects).

use std::process::Stdio;
use std::time::Duration;

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use uh_core::transport::ToolSpec;

use crate::r#trait::{Tool, ToolError, ok_result_with_display, truncate};
use crate::schema;
use crate::DEFAULT_BASH_OUTPUT_CHARS;

pub struct BashTool;

#[derive(Deserialize)]
struct Args {
    command: String,
    #[serde(default)]
    timeout_ms: Option<u64>,
}

#[async_trait]
impl Tool for BashTool {
    fn spec(&self) -> &ToolSpec {
        use std::sync::OnceLock;
        static SPEC: OnceLock<ToolSpec> = OnceLock::new();
        SPEC.get_or_init(|| ToolSpec {
            name: "bash".into(),
            description: "Execute a shell command and return stdout + stderr + exit code. v0.0.3 has no sandbox — do not use on untrusted input. Default timeout 30s; max 10 minutes.".into(),
            parameters: schema::object_schema(
                serde_json::json!({
                    "command": schema::string_arg("Shell command to execute (passed to `sh -c`)"),
                    "timeout_ms": schema::int_arg("Timeout in milliseconds (default 30000, max 600000)"),
                }),
                &["command"],
            ),
        })
    }

    async fn call(&self, args: Value) -> Result<uh_core::transport::ToolResult, ToolError> {
        let args: Args = serde_json::from_value(args)
            .map_err(|e| ToolError::InvalidArgs(e.to_string()))?;
        let timeout = Duration::from_millis(args.timeout_ms.unwrap_or(30_000).min(600_000));

        let mut child = Command::new("sh")
            .arg("-c")
            .arg(&args.command)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| ToolError::Execution(format!("spawn sh: {e}")))?;

        let mut stdout = child.stdout.take().expect("piped");
        let mut stderr = child.stderr.take().expect("piped");

        let stdout_task = tokio::spawn(async move {
            let mut buf = Vec::new();
            let _ = stdout.read_to_end(&mut buf).await;
            buf
        });
        let stderr_task = tokio::spawn(async move {
            let mut buf = Vec::new();
            let _ = stderr.read_to_end(&mut buf).await;
            buf
        });

        let timed_out;
        let status = match tokio::time::timeout(timeout, child.wait()).await {
            Ok(Ok(s)) => {
                timed_out = false;
                s
            }
            Ok(Err(e)) => return Err(ToolError::Execution(format!("wait: {e}"))),
            Err(_) => {
                timed_out = true;
                let _ = child.kill().await;
                let _ = child.wait().await;
                std::process::ExitStatus::default() // unreachable
            }
        };

        let stdout_bytes = stdout_task.await.unwrap_or_default();
        let stderr_bytes = stderr_task.await.unwrap_or_default();

        let stdout = String::from_utf8_lossy(&stdout_bytes);
        let stderr = String::from_utf8_lossy(&stderr_bytes);
        let mut body = String::new();
        if !stdout.is_empty() {
            body.push_str(&stdout);
        }
        if !stderr.is_empty() {
            if !body.is_empty() {
                body.push('\n');
            }
            body.push_str("[stderr]\n");
            body.push_str(&stderr);
        }
        let code = status.code();
        body.push_str(&format!("\n[exit code: {}]", code.map_or("signal".into(), |c| c.to_string())));
        if timed_out {
            body.push_str(" (timed out)");
        }
        let body = truncate(&body, DEFAULT_BASH_OUTPUT_CHARS);
        Ok(ok_result_with_display(
            "bash",
            body,
            serde_json::json!({ "command": args.command, "exit_code": code, "timed_out": timed_out }),
        ))
    }
}

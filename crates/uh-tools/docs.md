# `uh-tools` — Tool registry + 5 basic tools

> **设计先行**。本文件先于 `src/*.rs` 存在。详细架构见 `crates/uh-daemon/docs.md`。

## 缘由

Harness 的另一半基本功能是**让 AI 能动东西**。v0.0.3 的最小工具集是 5 个：

| 工具 | 作用 |
|------|------|
| `read_file` | 读文件内容 |
| `write_file` | 写文件（覆盖） |
| `edit_file` | 精准编辑（find/replace） |
| `list_dir` | 列目录 |
| `bash` | 执行 shell 命令（v0.0.3 同步、无沙箱） |

让 AI 能**读一个文件、改一个文件、保存**。这是用户原话"至少让他能基本改一个文件"的目标。

## 设计哲学

1. **Tool 是 `async fn call(args) -> Result<Content, ToolError>`** —— 一个
   形状，5 个实现。
2. **注册表是静态的** —— v0.0.3 启动时把所有工具注册到 `ToolRegistry`，
   后面有 preset 时再考虑动态加载。
3. **Schema 是 JSON Schema** —— 直接给 LLM 用，不发明格式。
4. **错误透传** —— 工具执行失败时把错误消息返回给 LLM，**不阻塞 loop**。
5. **沙箱推迟** —— v0.0.3 的 `bash` 是无沙箱的。v0.0.1+ 沙箱（Landlock /
   Seatbelt / Win32 Job Objects）单独做。

## Tool Trait（`src/traits.rs`）

```rust
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    pub name: String,
    pub description: String,
    pub parameters: Value,  // JSON Schema
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_call_id: String,
    pub content: String,         // 给 LLM 看的内容
    pub is_error: bool,          // LLM 知道这是错误，可以重试
    #[serde(default)]
    pub display: Option<String>, // 给 UI 看的（可省略）
}

#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("permission denied: {0}")]
    Permission(String),
    #[error("io: {0}")]
    Io(String),
    #[error("parse: {0}")]
    Parse(String),
    #[error("invalid args: {0}")]
    InvalidArgs(String),
    #[error("execution: {0}")]
    Execution(String),
}

#[async_trait]
pub trait Tool: Send + Sync {
    fn spec(&self) -> &ToolSpec;
    async fn call(&self, args: Value) -> Result<ToolResult, ToolError>;
}
```

## ToolRegistry（`src/registry.rs`）

```rust
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self { Self { tools: HashMap::new() } }
    pub fn register(&mut self, tool: Arc<dyn Tool>) { self.tools.insert(tool.spec().name.clone(), tool); }
    pub fn specs(&self) -> Vec<ToolSpec> { self.tools.values().map(|t| t.spec().clone()).collect() }
    pub async fn execute(&self, name: &str, args: Value) -> ToolResult { /* ... */ }
}
```

## 5 个工具

### read_file
- **args**: `{ path: string }`
- **returns**: 文件内容字符串
- **errors**: NotFound / Permission / Io

### write_file
- **args**: `{ path: string, content: string }`
- **returns**: "wrote N bytes to {path}"
- **errors**: Permission / Io

### edit_file
- **args**: `{ path: string, old: string, new: string }`（**v0.0.3 一次一处**；v0.0.4+ 支持多处）
- **returns**: "edited {path}"
- **errors**: NotFound / Parse（old 不存在）/ InvalidArgs（old 在文件里出现 0 或 >1 次）

### list_dir
- **args**: `{ path: string }`
- **returns**: 目录条目列表（每行一个，`name (type)`）

### bash
- **args**: `{ command: string }`
- **returns**: stdout + stderr + exit code
- **errors**: Execution（non-zero exit）
- **安全**: v0.0.3 无沙箱，v0.0.1+ 加

## 文件清单

### 新建
- `crates/uh-tools/Cargo.toml`
- `crates/uh-tools/src/lib.rs`
- `crates/uh-tools/src/traits.rs`     — Tool / ToolSpec / ToolResult / ToolError
- `crates/uh-tools/src/registry.rs`   — ToolRegistry
- `crates/uh-tools/src/read_file.rs`  — read_file
- `crates/uh-tools/src/write_file.rs` — write_file
- `crates/uh-tools/src/edit_file.rs`  — edit_file
- `crates/uh-tools/src/list_dir.rs`   — list_dir
- `crates/uh-tools/src/bash.rs`       — bash
- **`crates/uh-tools/docs.md`**       — 本文件

### 推迟（v0.0.1+）
- `crates/uh-tools/src/grep.rs`       — 内容搜索
- `crates/uh-tools/src/web.rs`        — web fetch / search
- `crates/uh-tools/src/git.rs`        — git 命令封装

## 验证清单

- [ ] `cargo test -p uh-tools` —— 5 个工具的 happy path + 错误 path
- [ ] 集成测试：daemon 起 → AI 改一个文件 → 文件确实改了
- [ ] `uh-devtool check-design crates/uh-tools` exit 0

## 范围外（明确不做）

- 沙箱执行（v0.0.1+）
- 并发工具调用（v0.0.3 串行）
- 工具权限 / approval prompt（v0.4+）
- 工具调用历史回放（v0.4+）

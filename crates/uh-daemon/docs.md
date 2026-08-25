# `uh-daemon` — Agent loop + HTTP / WebSocket transport

> **设计先行**。本文件先于 `src/*.rs` 存在。`uh-devtool check-design` 要求
> 每个 `.rs` 文件的祖先链上有一个 `docs.md`；本文件就是 `uh-daemon/` 下面
> 所有 `.rs` 的设计契约。

## 缘由

Harness 最基本的功能是**一个循环 + 工具调用**。v0.0.1 我们只做了 schema 和
UI 演示，没有真正的 agent loop。v0.0.3 要把这个循环跑通：

```
while not done:
    response = llm(messages, tools)
    if response has tool_calls:
        for each call: execute tool, append result
    else:
        done
```

让用户能**配置 API Key + 启动 daemon + 浏览器告诉 AI 改一个文件**。

## 解决的问题

1. **agent loop 跑通** —— 这是 harness 的存在意义。
2. **接入真 LLM** —— OpenAI 兼容（DeepSeek / OpenAI / vLLM / Ollama）。
3. **5 个基础工具** —— read_file / write_file / edit_file / list_dir / bash。
4. **UI 真正响应** —— 浏览器发消息，daemon 跑 loop，推回结果。

## 不解决的问题（明确推迟）

| 推迟到 | 内容 |
|--------|------|
| v0.0.4+ | plan-first skill（**v0.0.3 默认不启用**） |
| v0.0.5+ | Anthropic provider、prompt cache、流式响应 |
| v0.1+ | 工具沙箱执行、工具结果 schema 校验 |
| v0.2+ | 持久化、多 session 路由 |
| v0.4+ | 多 tab 同步、preset 系统完整化 |

## 设计哲学

1. **核心极简** —— daemon 只做：收消息 → 跑 loop → 推结果。**不发明额外概念**。
2. **plan-first 是 skill，不是同级** —— 它是装在 loop 上面的一层 wrapper。
   daemon core 不知道 plan-first 存在。
3. **JSON-RPC 风格 envelope** —— 不发明协议，复用生态。
4. **WebSocket only** —— 已确认。
5. **LLM trait 提前定** —— OpenAI compat 是 v0.0.3 唯一实现，v0.0.5+ 加
   Anthropic / DeepSeek 专用实现。**前端零改动**。
6. **工具也是 trait** —— tool 是 `async fn call(args) -> Result`，
   注册到 `ToolRegistry` 里，LLM 通过 schema 知道。
7. **AppState 是单一真相源** —— daemon 内存里维护 `AppState { session, llm, tools, peers }`，
   所有 mutation 走 AppState 的方法。
8. **session 单实例** —— v0.0.3 只有一个 session；多 session 是 v0.2+。

## 整体布局

```
┌─────────────────────────────────────┐
│  Browser (apps/web)                  │
│   - Chat 消息列表                    │
│   - Tool call 卡片                   │
│   - 输入框                           │
└──────────────┬──────────────────────┘
               │ WebSocket + JSON envelope
┌──────────────┴──────────────────────┐
│  uh-daemon (Rust)                    │
│  ┌────────────────────────────────┐  │
│  │  axum::Router                 │  │
│  │   ├── GET  /                  │  │← serve apps/web/dist
│  │   ├── GET  /ws  (WS upgrade)  │  │
│  │   └── GET  /api/health        │  │
│  └────────┬─────────────────────┘  │
│           │                         │
│  ┌────────▼─────────────────────┐  │
│  │  Router (method dispatch)     │  │
│  │   send_message / ping         │  │
│  │   list_tools / get_session    │  │
│  └────────┬─────────────────────┘  │
│           │                         │
│  ┌────────▼─────────────────────┐  │
│  │  AgentLoop                    │  │
│  │   run_turn() → llm → tools    │  │
│  │   → next turn until done      │  │
│  └────────┬─────────────────────┘  │
│           │                         │
│  ┌────────▼─────────────────────┐  │
│  │  AppState (Arc<RwLock>)       │  │
│  │   session: Session            │  │
│  │   llm: Arc<dyn Llm>           │  │
│  │   tools: ToolRegistry         │  │
│  │   peers: HashMap<PeerId, ...> │  │
│  └────────┬─────────────────────┘  │
│           │                         │
│  ┌────────▼─────────────────────┐  │
│  │  uh-llm::Llm trait            │  │
│  │   OpenAiCompatLlm (v0.0.3)    │  │
│  │   AnthropicLlm (v0.0.5+)      │  │
│  └──────────────────────────────┘  │
│           │                         │
│  ┌────────▼─────────────────────┐  │
│  │  uh-tools::ToolRegistry       │  │
│  │   read_file / write_file      │  │
│  │   edit_file / list_dir / bash │  │
│  └──────────────────────────────┘  │
└─────────────────────────────────────┘
```

## Envelope Schema（`uh-core/src/schema/transport.rs`）

```rust
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WsMessage {
    Request  { id: Uuid, method: String, params: Value },
    Response { id: Uuid, result: Value, error: Option<WsError> },
    Event    { event: String, data: Value },
    Cancel   { id: Uuid },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsError {
    pub code: i32,
    pub message: String,
    #[serde(default)]
    pub data: Option<Value>,
}
```

## Methods（v0.0.3 极简）

| method        | params                | result                  | 何时调用 |
|---------------|-----------------------|-------------------------|----------|
| `send_message`| `{ content: String }` | `{ message_id }`        | 用户在 UI 输入 |
| `get_session` | `{}`                  | `{ session: Session }`  | 首次连接 / 重连 |
| `list_tools`  | `{}`                  | `{ tools: [ToolSpec] }` | UI 显示可用工具 |
| `ping`        | `{}`                  | `{ pong: true, ts: i64 }`| 30s 心跳 |

错误码：
- `1000` 内部错误 / `1001` 协议错 / `1002` 方法不存在
- `1003` 参数错 / `1004` LLM 错 / `1005` 工具错

## Events（v0.0.3 极简）

| event          | data                                              | 何时触发 |
|----------------|---------------------------------------------------|----------|
| `user_message` | `{ message: Message }`                           | send_message 后 |
| `assistant_chunk` | `{ text: String, message_id: Uuid }`           | LLM 流式文本（v0.0.3 同步：只有一条） |
| `assistant_message` | `{ message: Message }`                       | LLM 完成一段响应 |
| `tool_call`    | `{ tool_call: ToolCall }`                         | LLM 决定调工具 |
| `tool_result`  | `{ tool_call_id, result: ToolResult }`            | 工具执行完成 |
| `turn_start`   | `{ turn_id: Uuid }`                               | 一次 turn 开始 |
| `turn_end`     | `{ turn_id, reason: String }`                     | 一次 turn 结束 |
| `error`        | `{ code, message, recoverable: bool }`            | 任何错误 |

v0.0.3 同步 LLM，所以 `assistant_chunk` 实际上不会触发，先放 schema 占位，
v0.0.5+ 切流式时自然用上。

## Agent Loop

```rust
pub async fn run_turn(state: &AppState, peer: PeerId) -> Result<TurnResult> {
    loop {
        // 1. 构造 messages（从 session 读）
        let messages = state.session.read().await.messages.clone();

        // 2. 构造 tools schema（从 ToolRegistry 读）
        let tools = state.tools.specs();

        // 3. 调 LLM
        let response = state.llm.chat(messages, tools).await?;

        // 4. 推 assistant_message
        state.peers.broadcast(&peer, "assistant_message", &response.message).await;

        // 5. 处理 tool_calls
        if response.tool_calls.is_empty() {
            return Ok(TurnResult::Done);
        }
        for call in response.tool_calls {
            state.peers.broadcast(&peer, "tool_call", &call).await;
            let result = state.tools.execute(&call).await;
            state.peers.broadcast(&peer, "tool_result", &result).await;
            state.session.write().await.append_tool_result(call, result);
        }
        // 6. 继续下一轮
    }
}
```

## AppState

```rust
pub struct AppState {
    pub session: RwLock<Session>,
    pub llm: Arc<dyn Llm>,
    pub tools: ToolRegistry,
    pub peers: RwLock<HashMap<PeerId, WsSender>>,
}

pub struct Session {
    pub id: SessionId,
    pub messages: Vec<Message>,  // 整个对话历史
    pub created_at: DateTime<Utc>,
}
```

## 文件清单

### 新建
- `crates/uh-daemon/Cargo.toml`
- `crates/uh-daemon/src/lib.rs`
- `crates/uh-daemon/src/server.rs`  — axum Router + WS upgrade
- `crates/uh-daemon/src/router.rs`  — method dispatch
- `crates/uh-daemon/src/state.rs`   — `AppState`
- `crates/uh-daemon/src/peers.rs`   — WS peer registry
- `crates/uh-daemon/src/loop.rs`    — `run_turn` 主体
- `crates/uh-daemon/src/config.rs`  — 读 `[llm]` 配置
- `crates/uh-daemon/src/session.rs` — `Session` 类型
- **`crates/uh-daemon/docs.md`**    — 本文件
- `crates/uh-llm/Cargo.toml`
- `crates/uh-llm/src/lib.rs`
- `crates/uh-llm/src/traits.rs`     — Llm trait + LlmError
- `crates/uh-llm/src/openai_compat.rs` — OpenAI 兼容实现（v0.0.3）
- `crates/uh-llm/src/message.rs`    — Message / ContentBlock 类型
- **`crates/uh-llm/docs.md`**       — LLM 设计契约
- `crates/uh-tools/Cargo.toml`
- `crates/uh-tools/src/lib.rs`
- `crates/uh-tools/src/traits.rs`   — Tool trait + ToolSpec
- `crates/uh-tools/src/registry.rs` — ToolRegistry
- `crates/uh-tools/src/read_file.rs` / `write_file.rs` / `edit_file.rs` / `list_dir.rs` / `bash.rs`
- **`crates/uh-tools/docs.md`**     — 工具设计契约
- `crates/uh-core/src/schema/transport.rs` — WsMessage / WsError / Message / ToolCall

### 修改
- `Cargo.toml`（root）  —— 加 `uh-daemon` / `uh-llm` / `uh-tools` members
- `crates/uh/Cargo.toml` —— 加依赖
- `crates/uh/src/main.rs` —— 启动 daemon 替代 v0.0.1 的 print
- `apps/web/src/lib/ws.ts` —— typed WS client
- `apps/web/src/App.tsx` —— chat 界面（替代 PlanPanel / ContextPanel / DeltaLog）
- `apps/web/vite.config.ts` —— 加 `/ws` proxy
- `uh.toml`（新建）—— 配置文件（[llm] section）

### 暂时不动
- `crates/uh-skill-plan-first/` —— 保留，但不进 v0.0.3 依赖
- `apps/web/src/review/` —— code viewer 独立工具，**v0.0.3 继续保留**
- `apps/web/src/components/{PlanPanel,ContextPanel,DeltaLog}.tsx` —— 保留源码
  但不挂在 `/` 路由，**留给 v0.0.4 plan-first 启用时复用**

### 删除
- `apps/web/src/data/samplePlan.ts` —— sample data 退役

## 配置（`uh.toml`）

```toml
[server]
host = "127.0.0.1"
port = 3080

[llm]
provider = "openai-compat"   # 唯一 v0.0.3 实现
base_url = "https://api.deepseek.com"
api_key  = "sk-xxx"          # 必填
model    = "deepseek-chat"
# 可选
timeout_secs = 120
max_retries  = 3
```

## 关键设计决策（vs 备选）

| 决策 | 选了 | 备选 | 原因 |
|------|------|------|------|
| 协议 | JSON-RPC 风格 envelope | gRPC / 自定义 binary | TS 生态成熟，调试方便 |
| 状态 | in-memory | 数据库 | YAGNI；v0.0.3 不持久化 |
| LLM 实现 | 直接 OpenAI compat | 先 Mock 再替换 | 用户的核心诉求是真模型能跑 |
| Tool 实现 | 同步、in-process | 沙箱隔离 | v0.0.3 不做沙箱（v0.1+） |
| UI | 简单 chat | PlanPanel 等 | core 跑通再说 |
| 端口 | 3080 | 5173 | 跟 Vite dev 区分 |

## 验证清单

- [ ] `pnpm exec tsc -b --noEmit` ✓
- [ ] `pnpm exec vite build` ✓
- [ ] `cargo test --workspace` ✓
- [ ] `cargo build -p uh-daemon -p uh-llm -p uh-tools` ✓
- [ ] `uh.exe` 启动 → `http://127.0.0.1:3080` 看到 chat 界面
- [ ] `http://127.0.0.1:3080/#/review` 还能用 code viewer
- [ ] 配置 DeepSeek API key 后，能让 AI 改一个文件
- [ ] `uh-devtool check-design crates/uh-daemon crates/uh-llm crates/uh-tools` exit 0
- [ ] 跨平台：Windows / macOS / Linux 都能跑

## 范围外（明确不做）

- 真实 Anthropic 接入（v0.0.5+）
- 流式响应（v0.0.5+）
- 沙箱 / 工具隔离（v0.1+）
- plan-first skill 接入（v0.0.4+）
- 多 session / 多 plan 路由（v0.2+）
- 持久化（v0.2+）
- 认证 / 多人协作（v0.4+）
- 历史回放（v0.4+）

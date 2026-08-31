# Ultimate Harness

> **An agent harness where structured iteration is a first-class concern.**
> LLM 负责生成内容，结构负责承担状态。迭代发生在结构（UI）里，不在 LLM 上下文里。

## 一句话

一个面向"AI 编码长期任务"的单 agent 运行时 + Web 前端。结构化 plan / step / delta / working memory 都用显式 schema 承载，LLM 只在 skill 触发时被调用，前端把 deltas 直接回填到结构上。

## 设计哲学

1. **沉淀稳定的结构，化繁为简** — Plan / Step / Delta / WorkingMemory 全部显式 schema，不用自然语言串状态
2. **零成本抽象原则** — 不写 TUI / CLI / subagent；YAGNI，需要时再加
3. **Cache-aware context** — Working Memory 永远放在 cacheable prefix 之外，不破坏 prompt cache
4. **plan-first 内建 skill** — 第一步也是最核心的 skill：先写简要方案，让人在 AI 真正动手前介入
5. **唯一前端，做到最好** — pnpm + Vite + React + TypeScript，不分散精力

## 仓库结构

```
ultimate_harness/
├─ Cargo.toml                          # workspace root (Rust 2021)
├─ Cargo.lock
├─ package.json                        # 前端 pnpm 工作区入口
├─ pnpm-lock.yaml
├─ uh.toml.example                     # 运行时配置样例
├─ apps/
│  └─ web/                             # 唯一前端 (Vite + React + TS)
├─ crates/
│  ├─ uh-core/                         # 核心 schema + 运行期接口
│  ├─ uh-skill-plan-first/             # 唯一 v0.1 内建 skill
│  └─ uh/                              # binary 入口
├─ docs/                               # 设计文档
├─ scripts/                            # 本地脚本
├─ tools/                              # 工具脚本
└─ check.txt / check-result.txt        # CI/本地检查
```

## 技术栈

- **运行时**：Rust（async/await + tokio + axum，TLS 用 rustls + aws-lc-rs）
- **前端**：pnpm + Vite + React 18 + TypeScript，WebSocket 实时通信（不用 SSE）
- **协议**：WebSocket 双向流，JSON 消息；事件用 append-only event log
- **存储**：SQLite (本地) + 文件系统
- **LLM 适配**：OpenAI 兼容 + DeepSeek，统一通过 `LLM adapter` 抽象

## 核心 schema

| 类型 | 作用 |
|---|---|
| `Plan` | 用户目标和子步骤的显式列表，每步有 `id / title / status` |
| `Step` | 单步执行单元，包含 `intent / plan_id / status` |
| `Delta` | 步骤执行后产生对结构/UI 的结构化修改 |
| `WorkingMemory` | 跨步骤的当前状态，只挂在 cacheable prefix 之外 |
| `EventLog` | append-only 事件流，支持回放与审计 |

## 当前进度

- [x] **v0.0.1** — Workspace 框架 + 核心 schema
- [ ] **v0.0.2** — uh binary + 静态 web serve
- [ ] **v0.0.3** — WebSocket 网关 + PlanPanel 渲染 + Delta 回填
- [ ] **v0.0.4** — skill-plan-first trait + mock LLM
- [ ] **v0.0.5** — 标准 LLM adapter（OpenAI 兼容 + DeepSeek）
- [ ] **v0.1.0** — 工具注册表 + 7 段串讲
- [ ] **v0.2.0** — Cache-aware context manager
- [ ] **v0.3.0** — OS-native 沙箱
- [ ] **v0.4.0** — preset 系统（启动时挂载 skill）

## 借鉴 / 不借鉴

| 来源 | 借鉴 | 不借鉴 |
|---|---|---|
| `grok-build` | OS-native 沙箱、rustls+aws-lc-rs、tokio+axum | 长篇料理、TUI、4 crate 过度拆分 |
| `deepseek-harness` | `Model-Visible ≠ Logged`、段间工具串讲思想、Capability Seam 思想、append-only event log | TypeScript 主导、Cordis 依赖、不接 PR |

## 本地开发

```bash
# 后端
cd crates/uh
cargo run

# 前端
cd apps/web
pnpm install
pnpm dev
```

## 设计文档

详见 `docs/` 目录。当前正在写 v0.0.2（uh binary）的设计说明。

## License

MIT

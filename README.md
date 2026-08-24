# Ultimate Harness

> **An agent harness where structured iteration is a first-class concern.**

## 一句话

> LLM 负责生成内容，结构负责承载状态。迭代发生在结构化 UI 里，不在 LLM 上下文里。

## 设计哲学

1. **沉淀稳定的结构，化繁为简** — Plan / Step / Delta / WorkingMemory 都有显式 schema，不靠自然语言传状态
2. **零成本抽象** — 不需要 TUI/CLI/subagent，就不实现。需要时再加
3. **Cache-aware context** — Working Memory 永远在 cacheable prefix 之外，不破坏 prompt cache
4. **plan-first** — 第一个也是最核心的 skill：每一步先写简要方案，让人知道 AI 要干嘛

## 仓库结构

```
ultimate-harness/
├── Cargo.toml                       # workspace root
├── crates/
│   ├── uh-core/                     # 核心 schema + 运行时接口
│   ├── uh-skill-plan-first/         # 唯一 v0.1 内建 skill
│   └── uh/                          # binary 入口
├── apps/
│   └── web/                         # 唯一前端（Vite + React + TS）
└── docs/                            # 设计文档
```

## 借鉴 / 不借鉴

| 来源 | 借鉴 | 不借鉴 |
|------|------|--------|
| grok-build | OS-native 沙箱、rustls+aws-lc-rs、tokio+axum | 镜像治理、TUI、84 crate 过度拆分 |
| deepseek-harness | `Model-Visible ⟺ Logged`、7 段工具管道思想、Capability Seam 思想、append-only event log | TypeScript、Cordis、不接 PR |

## 当前进度

- [x] **v0.0.1** — Workspace 骨架 + 核心 schema
- [ ] **v0.0.2** — uh binary + 静态 web serve
- [ ] **v0.0.3** — WebSocket 网关 + PlanPanel 渲染 + Delta 回传
- [ ] **v0.0.4** — skill-plan-first trait + mock LLM
- [ ] **v0.0.5** — 真 LLM adapter（OpenAI 兼容 + DeepSeek）
- [ ] **v0.1.0** — 工具注册表 + 7 段管道
- [ ] **v0.2.0** — Cache-aware context manager（5 层 memory）
- [ ] **v0.3.0** — OS-native 沙箱
- [ ] **v0.4.0** — preset 系统（启动时勾选 skill）

## 开发

```sh
cargo build
cargo test
cd apps/web && pnpm install && pnpm dev
```

## 调研资料

- [`grok-build/`](./grok-build/) — Grok Build 调研报告（xAI 2026-07 开源）
- [`deepseek-harness/`](./deepseek-harness/) — DeepSeek Harness 调研报告（2026-08 开源）
- [`docs/architecture.md`](./docs/architecture.md) — 本项目的设计文档

## License

MIT

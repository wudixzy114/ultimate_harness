# `uh` — Entry Point

## 缘由

`uh` 是 Ultimate Harness 的**二进制入口**。它做三件事：

1. 解析 CLI 参数（`uh run` / `uh serve` / `uh repl`）。
2. 加载 Skill 集合（动态注册或编译时静态链接）。
3. 启动 agent loop：接 LLM 调 Skill，emit Deltas / ContextSnapshot / 持久化状态。

为什么独立 crate 而不是 main.rs 在 `uh-core`：`uh-core` 是纯 schema/库，不应该知道 CLI 或 daemon 的存在。`uh` 是"组装者"。

## 解决的问题

1. **产品入口**——所有用户面对的命令行表面。
2. **Skill 组合**——把多个 Skill crate 装进同一个进程。
3. **配置装载**——env vars / config.toml / secrets。

## 设计哲学

1. **薄壳**——`uh` 本身不写业务逻辑，所有能力来自 `uh-core` + `uh-skill-*`。
2. **Skill 注册显式**——不在 `uh` 内部 `impl Skill for ...`，而是 `uh-skill-plan-first::register()` 这种显式注册函数。
3. **配置先于代码**——agent loop 行为由 config 驱动，不靠改 `uh` 源码。

## 具体细节设计

### CLI（占位）

```text
uh run      # 启动 agent loop，连接 LLM
uh serve    # 启动 web + websocket 桥
uh repl     # 交互式 shell（v0.3）
```

### Agent Loop（v0.2 占位骨架）

```
load skills → assemble context → call LLM → parse Plan/Delta →
apply Delta → execute Step → write Retrospective → emit WebSocket event
```

每一步都 emit `tracing::info!` event（结构化 JSON），code viewer v1.0 的 Trace tab 会读这些 event 画动态数据流。

## 演进

v0.1：CLI 骨架。
v0.2：loop tick 完整闭环。
v0.3：repl。
v1.0：多 session 并发。

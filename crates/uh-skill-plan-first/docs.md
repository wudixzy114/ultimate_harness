# `uh-skill-plan-first` — 增强版 skill（v0.0.4+ 才启用）

> **重要**：本 crate **不在 v0.0.3 依赖图里**。它是 harness core 之上的
> **可选 skill / preset**，不是 harness 的同级能力。
>
> v0.0.3 先把核心循环跑通（loop + LLM + 5 个工具），v0.0.4 再接这个 skill。

## 它是什么 / 不是什么

### 是
- **一个 skill** —— 可以通过 preset 在启动时启用
- **增强版** —— 在 harness core 之上加一层结构化迭代
- **可选** —— 不启用就跟没装一样，harness 照常工作

### 不是
- ❌ Harness 的核心能力（harness 没它也能跑）
- ❌ Harness 的同级模块（它依赖 daemon，不被 daemon 依赖）
- ❌ v0.0.3 必需的部分

## 设计哲学（参考，不在 v0.0.3 范围）

1. **plan-first 是 skill 的强形式** —— 强制每个 task 都有 Plan、每个 step
   都有 Retrospective
2. **Plan + Delta 是 structured iteration 的核心** —— 用户通过 Delta 控制
   Plan 演化，而不是在 LLM 上下文里反复读 Plan 全文
3. **Token 经济性** —— 同等信息的传递，结构化比散文省 ~7x token

## v0.0.4 计划（要等 core 跑通）

- `Skill` trait 重新设计成 `async fn on_event(event) -> Option<Action>`
- plan-first skill 监听 task_submitted / step_starting / step_finishing 事件
- 注入 plan 必填、step retrospective 必填的强制检查
- UI 上挂回 PlanPanel / ContextPanel / DeltaLog
- 加 preset 配置（`uh.toml` 里 `[presets]` section）

## v0.0.3 状态

- 现有代码：v0.0.1 写的 skeleton（无强制逻辑）
- 现状：能编译、跑、装入 daemon，但**没有 wire-up**
- v0.0.3 不会调用它
- v0.0.4 wire-up 时再改

## 与 harness core 的关系

```
v0.0.3 架构:
  uh-daemon (核心)
    └─ uh-llm (LLM trait + OpenAI compat)
    └─ uh-tools (5 个基础工具)
    [uh-skill-plan-first 装在但不被调用]

v0.0.4 架构:
  uh-daemon (核心)
    └─ uh-llm
    └─ uh-tools
    └─ SkillRegistry (新增)
         └─ uh-skill-plan-first (可选，按 preset 加载)
```

## 范围外

- v0.0.3 不实现任何强制逻辑
- v0.0.4 之前的 wire-up
- UI 集成（PlanPanel 暂时挂在产品页但不被使用）
- preset 系统

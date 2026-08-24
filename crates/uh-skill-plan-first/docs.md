# `uh-skill-plan-first` — Plan-First Iteration Skill

## 缘由

`uh-skill-plan-first` 是 Ultimate Harness 的**首个具体 Skill**——一个强制"先写 Plan、再执行"的迭代模式。它的存在是为了把"先想清楚再动手"这个工程原则变成**系统行为**，而不是文档建议。

为什么不只是 `uh` 主 crate 的一个函数：Skill 是 `uh-core::Skill` trait 的实现，按约定必须独立 crate——这样 Skill 之间的依赖是显式的 `Cargo.toml` 依赖，不是隐式的 import。

## 解决的问题

1. **AI 直觉执行的脆弱性**——LLM 倾向于"看到 task → 直接给答案"，跳过了"目标 / 成功标准 / 风险 / 回退"的明确化。
2. **人类信任成本**——用户不敢让 AI 直接改大文件，因为不知道它会做什么。
3. **Review 表面**——没有 Plan，回顾就是 "what happened"；有 Plan，回顾是 "what happened vs what we said"。

## 设计哲学

1. **Plan 是契约**——Plan 不是建议，是结构化的承诺：`goal` / `success_criteria` / `assumptions` / `unknowns` / `steps`。
2. **每步都有 Retrospective**——`matches_plan` 字段强制 LLM 自我审计。
3. **Unknown 是 first-class**——`blocking: true` 的 unknown 必须先解决，plan 才能从 Draft → Running。
4. **Delta 是唯一的修改入口**——plan body 一旦进入 Running 不可改，所有调整走 Delta。

## 具体细节设计

### Skill 实现

实现 `uh_core::Skill` trait，监听 `SkillEvent` 中的关键事件：

- `TaskSubmitted` → 引导 LLM 输出 Plan schema。
- `PlanReady` → 强制把 Plan 渲染到 web，等用户批准。
- `DeltaSubmitted` → 应用 Delta 到 Plan。
- `StepStarting` / `StepFinishing` → 强制 LLM 写 Retrospective。

### 与 web 的关系

`PlanPanel` / `DeltaLog` 组件直接消费 `uh-core` 的 Plan/Delta TS 镜像——`uh-skill-plan-first` 不引入新的 schema，只定义行为。

## 演进

v0.1：基础 plan-first 流程。
v0.2：支持"嵌套 plan"（step 内部再触发 sub-plan）。
v1.0：plan 模板 + 跨 session 复用。

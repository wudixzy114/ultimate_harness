# `uh-core` — The Stable Structure

## 缘由

`uh-core` 是 Ultimate Harness 的**所有跨层流动对象的唯一来源**。LLM 发 Plan、用户改 Delta、daemon 装配 Context、web 渲染——这些对象的形状都在 `uh-core` 定义。

为什么不分散在多个 crate：因为"形状"是系统稳定性的核心，分散 = 漂移。改一个字段要 N 处同步，必然漏。

为什么不放在 `uh` 主 crate：因为主 crate 要能引用所有 schema 而不引入反向依赖；`uh-core` 是叶子。

## 解决的问题

1. **跨层契约**——LLM 的 JSON、daemon 的 Rust、web 的 TS 都从同一份 schema 镜像。
2. **Cache 边界**——`ContextSnapshot` 是 cache-aware context 的结构化载体，让 web 能可视化 LLM "看到了什么"。
3. **Audit 起点**——`Plan` / `Step` / `Retrospective` / `Delta` 让每一步都可追溯。

## 设计哲学

1. **结构优于自然语言**——任何"状态"必须是 schema type。散文只能用于"内容"（plan 的 task 描述、retrospective 的 deviation）。
2. **显式 invariant**——关键 schema 的 `/// # @invariant` 段是契约声明，code viewer 直接抓取并展示。
3. **Cache 边界刚性**——`WorkingMemory` 和 `StepMemory` 必须在 cacheable prefix 之外，违反即 panic。
4. **所有变化是 Delta**——用户与系统交互不引入隐式状态变更。
5. **Id 强类型**——`PlanId` / `StepId` / `SessionId` 等都是新类型，避免 id 混用。

## 具体细节设计

### Schema 列表

| 类型 | 文件 | 角色 |
|---|---|---|
| `Plan` / `Step` / `PlanStatus` | `schema/plan.rs` | 主迭代对象 |
| `Delta` / `DeltaAction` | `schema/delta.rs` | 用户意图 |
| `WorkingMemory` / `StepMemory` | `schema/memory.rs` | 可变草稿 |
| `ContextSnapshot` / `ContextLayer` | `schema/context.rs` | LLM 视角的上下文 |
| `Skill` / `SkillEvent` / `SkillOutput` | `skill.rs` | 插件接口 |
| `PlanId` / `StepId` / `SkillId` / ... | `ids.rs` | 强类型 id |

### 不变式

- **每个 `Step` 完成后必须带 `Retrospective`**——保证"实际发生 vs 计划"的可见性。
- **WorkingMemory 永远不在 cacheable prefix**——这是 prompt cache 正确性的根本保证。
- **Delta 提交后不可修改**——所有变更用新 Delta 表达。

### 与 code viewer 的关系

`uh-core` 内的 schema 携带的 `/// # @invariant` / `# @data-flow` / `# @tag` 段是 code viewer 的元数据来源。这些注释就是合法 rustdoc——`cargo doc` 照常渲染，viewer 解析它们生成 reviewer 视图。

# Design-First Enforcement

> Cross-crate practice. The previous per-crate `docs.md` files (in
> each `crates/*/docs.md`, `tools/*/docs.md`, `apps/web/docs/`) are
> the **artifacts** this practice enforces. This document is the
> **policy** and the **evolution roadmap**.

## 缘由

软件系统最贵的失败模式不是"实现错了"，是"实现得对但解决的不是真问题"。这类失败发生在动手之前没有把问题想清楚的瞬间。

传统缓解手段：
- 文档先行 → 文档会漂移，**无人读**
- Code review → 太晚，问题已经在代码里
- 测试驱动 → 验证了"做对"，没验证"做对的事"

**Design-First Enforcement** 的核心思路：把"设计文件"提升为**系统可校验的资源**，让"先有设计再写代码"成为工具链强制而不是个人纪律。

跟 plan-first 是一脉相承的：
- plan-first 管住**一次任务的步骤**（`Plan` / `Step` / `Retrospective`）
- design-first 管住**代码的修改权**（改 .rs 之前必须有 docs.md）

## 解决的问题

| 痛点 | 解决 |
|---|---|
| 改了 .rs 但没想清楚 | 缺 docs.md → 工具链拒绝（软/硬） |
| 文档和代码漂移 | docs.md 是改 .rs 的**前置条件** |
| 新成员接手陌生目录 | 任何目录的 docs.md 是入口 |
| LLM 看到 task 直接改文件 | daemon 拦截，emit `Delta { type: "missing_design" }` 让 LLM 先写 docs.md |

## 设计哲学

1. **设计文件是 .rs 的 schema**——和 `uh-core` 里的 struct 一样，是稳定结构而不是散文。
2. **祖先回溯**——找最近的 `docs.md` 即可，不要求每个子目录都重复一份。`src/plan.rs` 找 `crates/uh-core/docs.md` 是合法的。
3. **软硬两档**——`Warn` / `Enforce` 由 `DesignPolicy` 决定；v0.1 CLI 走 `Enforce`（exit 1），便于 CI 接入。
4. **失败可见**——LLM 和人看到的反馈格式相同：`{ file, required, nearest_ancestor_tried }`。
5. **第一次创建豁免**——v0.2 才考虑（让 LLM 自举 docs.md）。
6. **不重复 /// 注释**——docs.md 写"为什么 + 怎么用 + 不变式"，/// 写"做了什么 + 字段意义"。两者正交。

## 具体细节设计

### 设计文件规范

- **命名**：`docs.md`（小写，统一一个文件）
- **位置**：每个 crate / 工具 / 前端单位的根目录
- **内容**：缘由 + 解决的问题 + 设计哲学 + 具体细节设计
- **不写**：代码复述、字段列表（这是 /// 注释的事）

### 检查规则（v0.1）

```
对每个 .rs 文件：
  找最近的祖先 docs.md（自身目录 → 父目录 → ... → 根）
  找到 → 通过
  没找到 → missing
```

### 当前项目的覆盖

| 目录 | docs.md | 覆盖的 .rs |
|---|---|---|
| `crates/uh-core/docs.md` | ✅ | `uh-core/src/**/*.rs` |
| `crates/uh-skill-plan-first/docs.md` | ✅ | `uh-skill-plan-first/src/lib.rs` |
| `crates/uh/docs.md` | ✅ | `uh/src/main.rs` |
| `tools/uh-devtool-analyze/docs.md` | ✅ | `uh-devtool-analyze/src/**/*.rs` |
| `tools/uh-devtool/docs.md` | ✅ | `uh-devtool/src/**/*.rs` |
| `apps/web/docs/code-viewer-design.md` | ✅ | `apps/web/src/**/*.ts(x)` |

注：apps/web 下前端代码的检查由后续前端 toolchain 处理（v0.3+），不在 v0.1 范围内。

## 实施层次

### v0.1 — CLI check（立即可做）

```bash
uh-devtool check-design <PATH>
```

- 扫描 PATH 下所有 `.rs` 文件
- 找最近祖先 `docs.md`
- 缺 → 打印缺失列表 + `exit 1`
- 全过 → `exit 0`

可以接：
- `git pre-commit`：本地拦截
- CI：在 PR 跑一次
- `uh` daemon：step 执行前调用

### v0.2 — git hook 模板

提供 `uh-devtool init-hooks`，在 `.git/hooks/pre-commit` 写入：
```bash
#!/bin/sh
exec cargo run --quiet -p uh-devtool -- check-design crates tools
```

### v0.3 — `uh-core` 内置 DesignPolicy

```rust
// crates/uh-core/src/design.rs
pub enum DesignPolicy {
    Off,
    Warn,
    Enforce,
}

pub enum DesignStatus {
    Ok(PathBuf),            // 找到的 docs.md 路径
    Missing { tried: Vec<PathBuf> },
}

pub fn check_design(file: &Path) -> DesignStatus { ... }
```

daemon 在 step 执行前调 `check_design`：
- `Off` → 跳过
- `Warn` + missing → emit `Delta { type: "design_warning", file, required: "docs.md" }`，step 继续
- `Enforce` + missing → step 转 `Blocked(reason: "missing_design")`，LLM 必须先生成 docs.md

### v1.0 — Plan ↔ docs 联动

Plan 的 step 引用文件时，editor 自动验证 step 涉及的每个文件都满足 `check_design`。
- step 涉及文件 X，X 缺 docs → step 进 `Running` 之前必须先 fix
- 这是 design-first + plan-first 的闭环

## 与现有 docs 体系的关系

刚才写的 6 个 docs.md（每个 crate 一个）就是 design-first 的**第一批实践对象**。它们遵循以下原则：
- 不重复 /// 注释
- 写"为什么"和"哲学"，不写"做什么"
- 包含"具体细节设计"小节
- 每个 crate 一个，不下沉到子目录

## 演进守则

- **v0.1 不动 uh-core**——纯 CLI，对生产代码零侵入
- **v0.2 不动 daemon**——只加 git hook
- **v0.3 才在 uh-core 加 type**——`DesignPolicy` 是 schema 的一部分
- **v1.0 才和 plan editor 联动**——改动面最大，留到最后

每个版本都可以独立 ship，每个版本都不破坏现有。

## 架构归属（重要）

**check-design 的本质是 policy enforcement，是 harness 的核心能力，不是 dev tool。**

当前 v0.1 把它放在 `tools/uh-devtool/src/commands/check_design.rs` 是**权宜之计**：

- 优点：CLI 立即可用、不动 uh-core、不破坏"tools/ 不依赖 crates/"的硬约束
- 缺点：v0.3 时 `DesignPolicy` 必须进 `uh-core`（daemon 需要），那时如果 policy 逻辑还在 `tools/`，就会出现**反向依赖**——`uh-core → tools/`，违反我们写进根 `Cargo.toml` 的边界规则

### 迁移路径

| 阶段 | 位置 | 角色 |
|---|---|---|
| v0.1（当前） | `tools/uh-devtool/src/commands/check_design.rs` | CLI 工具，**纯实现，无 product 依赖** |
| v0.2 | 同 v0.1 | 不变；只加 git hook 模板 |
| v0.3 | **新建 `crates/uh-design/` crate** | 抽出 `check_design(file: &Path) -> DesignStatus` 函数 + `DesignPolicy` enum |
| v0.3 后 | `tools/uh-devtool` 改为薄 wrapper | CLI 调用 `uh_design::check_design` |
| v0.3 后 | `uh-core` 通过 `uh-design` 调用 | daemon step 执行前 gate |

**关键约束**：
- `crates/uh-design/` 是叶子 crate，不依赖任何 `crates/uh-core` / `tools/*`
- `uh-core` 依赖 `uh-design`（policy 是 schema 的一部分）
- `tools/uh-devtool` 也依赖 `uh-design`（CLI wrapper）
- 依赖方向永远是 `crates/uh-core` ← `crates/uh-design` ← `tools/uh-devtool`（**不会反向**）

### 为什么不让 `uh-core` 直接做这事

- `uh-core` 是 schema 容器，不该有文件系统遍历逻辑
- `uh-design` 是纯逻辑（路径 + 策略），无 IO 副作用 → 容易单测
- 分离后：`uh-core` 只调 `uh_design::check_design(path)`，不关心实现细节

### 触发搬迁的信号

- daemon 开始实施 step 执行前 gate（v0.3 第一项）→ 必须搬
- 出现第二个 caller（比如 Plan editor、repl）→ 必须搬
- 之前：CLI 单点使用，保持在 `tools/`，**接受这个不优雅的位置**

**记录时间**：v0.1 ship 时
**记录位置**：`docs/design-first.md`（本文）


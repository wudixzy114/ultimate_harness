# Code Viewer — Display Design

> This document is the source of truth for the `/review` route. It is
> intentionally **not** duplicated into per-file `///` comments — the
> file-level design intent lives here, and is the only place reviewers
> need to read before evaluating the visual choices.

## 缘由

人工审核 Rust 代码的痛点：

1. 看不到"主干"——loop、skill 加载、context 装配散在多个文件，靠脑内拼图。
2. 看不到"为什么"——`pub struct` 只告诉你形状，不告诉你 invariant。
3. 看不到"边界"——一个 struct 哪些字段是 cache 关键、哪些是 user-facing，没有视觉信号。

IDE 工具（rust-analyzer、rustdoc、cargo doc）各自只解决其中一层：
- rustdoc 是 API 文档导向，不展示"实现意图"。
- rust-analyzer 是 IDE 语义层，不展示"设计意图"。
- IDE 的"file outline"是树，但不分 kind 也不展开 data-flow。

**code viewer 是第三层**——只在 Rust 源码 + `///` 注释之上做结构化提取，把"作者写出来的设计意图"以表格和卡片的形式展示出来，让 reviewer 一眼能看到 invariant、data-flow、未注释字段。

## 解决的问题

| 痛点 | 解决方式 |
|---|---|
| 文件作用不清 | 文件头部 `purpose` 段（来自 `//!`） |
| 测试在哪 | `tests` 列表（带行号） |
| 类型结构不清 | 树 + 字段/变体/方法表格 |
| 不知道哪些字段需要 doc | 表格行高亮 + 顶部 badge |
| 不知道数据怎么流 | `data-flow` 卡片（input/output/side-effect/depends-on） |
| 不知道哪些是核心不变式 | `invariants` 列表 |
| 改一个 /// 注释的反馈延迟 | 2s 轮询，无刷新更新 |

## 设计哲学

1. **结构优于自然语言**——状态、分类、列都用结构承载，不靠 reviewer 读散文。
2. **左树只承载结构**——名字 + kind + flags，不展开内容。
3. **右 panel 承载内容**——按 kind 分发到不同的 panel（FieldTable / VariantTable / MethodList / SignaturePanel / DocPanel / ModPanel）。
4. **不嵌套**——树深度 ≤ 2（mod 嵌套 1 层，其它 item 不再展开）。
5. **静态层 vs 动态层分离**——本期只做静态；运行时 trace 留到 v2（参考 "tools/uh-devtool/docs.md" 的 v2 路线）。
6. **零侵入**——`///` 注释就是合法 rustdoc，cargo doc 照常渲染；本 viewer 不要求任何额外源文件改动。
7. **独立模块**——`apps/web/src/review/` 不 import 任何 `src/types/*` 产品镜像；review 是 dev tool，路径、类型、样式全部隔离。

## 整体布局

```
┌──────────────────────────────────────────────────────────────────┐
│  code viewer /review                  10 files · [back to app]  │  ← header
├──────────┬───────────────────────────┬───────────────────────────┤
│          │  crates/uh-core/src/...   │  struct Plan              │
│ FileTree │  ┌─────────────────────┐  │  ┌──────────────────────┐  │
│          │  │ struct  Plan  :43   │  │  │ 12 fields · 2 undoc  │  │
│ uh-core  │  │ enum    StepIntent │  │  │──────────────────────│  │
│  schema/ │  │ struct  Criterion  │  │  │ name│type │doc│line  │  │
│   plan…  │  │ struct  Assumption │  │  │ id  │Plan…│⚠  │ 44  │  │
│  schema/ │  │ ...                 │  │  │ ...                   │  │
│   context│  │ mod    schema       │  │  └──────────────────────┘  │
│  skill…  │  │  enum  PlanStatus  │  │                            │
│  …       │  │  …                 │  │  invariants                │
│          │  └─────────────────────┘  │  • Every step…             │
│ uh       │                           │                            │
│  main    │                           │  data-flow                 │
│          │                           │  (none)                    │
│ uh-      │                           │                            │
│  skill-… │                           │                            │
├──────────┴───────────────────────────┴───────────────────────────┤
```

**三栏**：
- **左（280px）**：FileTree，按 crate 分组的文件列表。
- **中（自适应）**：ExposedTree，选中文件的 pub item 列表（mod 嵌套 1 层）。
- **右（自适应）**：DetailPanel，按选中 item 的 kind 分发。

## 每种 PubKind 的展示

| Kind | 树节点 | 右侧 Panel | 用途 |
|---|---|---|---|
| `struct` | 单行 | `FieldTable` | 看字段、对照未注释 |
| `enum` | 单行 | `VariantTable` | 看变体、对照未注释 |
| `trait` | 单行 | `MethodList` | 看方法签名 |
| `fn` | 单行 | `SignaturePanel` | 看签名 + data-flow + invariants |
| `const` | 单行 | `DocPanel` | 看值语义 + 文档 |
| `type` | 单行 | `DocPanel` | 看别名定义 + 文档 |
| `impl` | 单行 | `MethodList` | 看实现的方法 |
| `mod` | 嵌套 1 层 | `ModPanel` | 看 mod 内的 pub item 列表 |

**不展开的项**：
- enum 的变体不再嵌入树中显示。
- trait / impl 的方法不再嵌入树中显示。
- struct 的字段不再嵌入树中显示（只在 FieldTable 里）。
- mod 内的子 item 在树中单行展示（嵌套 1 层），不进一步展开。

## 表格列设计（"表单设计"——每一列的展示目的）

### FieldTable（struct）

| 列 | 宽度 | 内容 | 设计目的 |
|---|---|---|---|
| `name` | 自适应 | `<vis> <name> [⚠]` | 看权限 + 标识；⚠ 表示未注释 |
| `type` | 自适应 | `<code>` | 看类型（泛型已 normalize：`Vec<Criterion>`） |
| `doc` | 最大 | 注释文本或 `—` | 看是否有注释；未注释用 `—` 灰色 |
| `line` | 固定 | `:N` | 跳到源码（v2 用） |

**未注释行的视觉**：
- 背景 `#2a1a1a`（暗红）
- 名字列尾 ⚠ 标记（`#f48771`）
- doc 列显示 `—` 灰色

### VariantTable（enum）

| 列 | 宽度 | 内容 | 设计目的 |
|---|---|---|---|
| `name` | 自适应 | variant 名 | 一眼看到全部变体 |
| `doc` | 最大 | 注释文本或 `—` | 看哪些变体有注释（v0.1 review 重点） |
| `line` | 固定 | `:N` | 跳到源码（v2 用） |

### MethodList（trait / impl）

| 列 | 宽度 | 内容 | 设计目的 |
|---|---|---|---|
| `signature` | 自适应 | `fn name(...) -> ret` | 一眼对比方法签名 |
| `doc` | 最大 | 注释文本或 `—` | 看方法是否有契约说明 |
| `line` | 固定 | `:N` | 跳到源码（v2 用） |

### SignaturePanel（fn）

| 区块 | 内容 | 设计目的 |
|---|---|---|
| Header | 签名 + line | 看清整个函数形式 |
| Tags | `planner` / `executor` 等 | 角色定位 |
| Doc | 自由文本 | 契约说明 |
| Data-flow | input / output / side-effect / depends-on | 数据流 |
| Invariants | 列表 | 关键不变式 |

### DocPanel（const / type / mod）

| 区块 | 内容 | 设计目的 |
|---|---|---|
| Header | name + line | 标识 |
| Tags | tag 列表 | 角色 |
| Doc | 自由文本 | 用途说明 |
| Invariants | 列表（仅 mod） | mod 级契约 |

### ModPanel（mod）

| 区块 | 内容 | 设计目的 |
|---|---|---|
| Header | mod 名 + line | 标识 |
| Sub-items | 子 pub item 列表（按 kind 分组） | 看 mod 暴露什么 |

## 树的 flag 设计

树节点右侧的 flag 标记，让 reviewer 一眼看到"这个 item 有没有 metadata"：

| Flag | 颜色 | 含义 |
|---|---|---|
| `undoc` | 红（`#f48771` / bg `#5a1d1d`） | struct 有未注释字段 |
| `df` | 青（`#4ec9b0` / bg `#1d3a5a`） | 有 data-flow |
| `inv` | 黄（`#dcdcaa` / bg `#3a3a1d`） | 有 invariants |
| `tag:planner` 等 | 灰 | 来自 /// @tag |

文件级也有全局 flag：文件头有"undoc fields present" badge（如果任何 struct 有未注释字段）。

## 视觉信号汇总

| 信号 | 颜色 | 触发 |
|---|---|---|
| 未注释字段 | bg `#2a1a1a` + ⚠ | `field.has_doc == false` |
| 未注释变体 | bg `#2a1a1a` + ⚠ | `variant.has_doc == false`（v0.2） |
| kind badge | struct=青 / enum=黄 / trait=紫 / fn=蓝 / const=蓝 / type=绿 / impl=紫 / mod=灰 | 来自 `item.kind` |
| 选中行 | bg `#094771` | 当前选中 |
| hover | bg `#2a2d2e` | 鼠标悬停 |

## 交互

- **点击树节点** → 右侧 panel 切换为该 item 的详情。
- **点击文件** → 树的中间栏切换为该文件的 exposed 列表，右侧显示文件级 detail（purpose / file_tags / file_invariants / tests / fns with data-flow）。
- **点击 mod** → 树中间栏显示 mod 嵌套结构（一层），右侧显示 ModPanel。
- **不需要"展开/折叠"按钮**——mod 用嵌套表达层级，item 永不展开。
- **不需要"详情页"**——一切都在右侧 panel。
- **2s 轮询**——改 `///` 注释，2s 内更新，无须刷新。

## 数据 schema（来自 `tools/uh-devtool-analyze`）

```ts
interface FileAnalysis {
  path: string;
  purpose: string;             // 来自 //!
  file_tags?: string[];
  file_invariants?: string[];
  tests: TestInfo[];
  exposed: PubItem[];
  fns: FnAnalysis[];
}

interface PubItem {
  kind: "struct" | "enum" | "trait" | "fn" | "const" | "type" | "impl" | "mod";
  name: string;
  line: number;
  tags?: string[];
  invariants?: string[];
  data_flow?: DataFlow;
  children?: PubItem[];        // enum variants / trait methods / mod contents
  fields?: FieldInfo[];        // struct only
  has_undocumented_fields?: boolean;
}

interface FieldInfo {
  name: string;
  ty: string;
  line: number;
  has_doc: boolean;
  doc?: string;
  vis: string;                 // "pub" | "pub(crate)" | ""
}
```

完整 schema 见 `tools/uh-devtool-analyze/src/model.rs`。

## 实时性

- `uh-devtool serve` 提供 `/api/index` 和 `/api/file?path=...` 两个 endpoint。
- `uh-devtool watch` 在文件变更时增量 re-analyze（debounce 200ms）。
- 前端每 2s 轮询 `/api/index`（仅文件列表）和当前选中文件（详情）。
- 改 `///` 注释后 2s 内更新（无刷新）。

## 范围

### v0.1（本版）

- ✅ 静态层分析 + 注释驱动 metadata
- ✅ 三栏布局 + 8 种 kind 的 panel 分发
- ✅ 字段/变体/方法表格
- ✅ 未注释标记
- ✅ 2s 轮询

### v0.2

- CodeView（源文件 raw 渲染 + syntax highlighting）
- 外部 AI 元数据合并（`.uh/annotations/<file>.json`）
- 未注释变体同样高亮（需要 lib 提取 variant docs）
- Caller/Callee 图

### v1.0

- 运行时 trace tab（uh daemon 的 `tracing::info!` JSON 输出）
- 跨文件跳转（点击 plan.rs 的 `Plan` 跳到 schema/plan.rs）
- 文件变更 SSE 推送（替代 2s 轮询）

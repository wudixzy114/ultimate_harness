# `uh-devtool-analyze` — Rust Source Analyzer

## 缘由

code viewer 的核心是一个"读 Rust 源码、按约定提取元数据"的分析器。它**只读源码不读 AST API**——不依赖任何产品 crate，通过文件系统读 `.rs`，这样可以零侵入地分析任何 Rust 项目。

为什么不直接用 `rustc --emit=metadata` 或 rustdoc JSON：太重，且耦合到 rustc 内部 API。

为什么不直接用 rustdoc：rustdoc 是文档生成器，输出的是 HTML/Markdown 散文，不适合结构化消费。

## 解决的问题

1. **结构化代码意图**——从 `///` 注释和源码 AST 提取 `purpose / tags / data-flow / invariants / fields / variants / methods / tests / has-doc` 等结构。
2. **跨文件一致性**——同一个 struct 可以在多个文件被 `use`，但元数据只在定义点提取，避免重复。
3. **Code viewer 的数据源**——本 crate 是 code viewer 的唯一上游；前端不直接读源码。

## 设计哲学

1. **零产品依赖**——`Cargo.toml` 不出现 `uh-core`。这是硬约束，写进根 `Cargo.toml` 的注释。
2. **注释 schema 复用 rustdoc**——`/// # @tag` / `# @data-flow` / `# @invariant` 就是合法 rustdoc 段，cargo doc 照常渲染。
3. **解析失败是局部失败**——一个文件注释写错不应阻塞其他文件；`analyze_dir` 收集所有 `(path, Result)`，调用者决定怎么处理。
4. **schema 公开**——`FileAnalysis` / `PubItem` / `FieldInfo` 的字段名是公开契约，web 端 TS 直接镜像。
5. **JSON 稳定**——`#[serde(skip_serializing_if = ...)]` + `default` 让缺字段不破坏前端。

## 具体细节设计

### 注释 schema

```text
//! File purpose (first paragraph only — blank line ends it)
pub struct Foo {
    /// # @tag schema executor
    /// # @data-flow
    /// input: ...
    /// output: ...
    /// side-effect: ...
    /// depends-on: ...
    /// # @invariant
    /// ...
    pub x: T,
}
```

段头格式：`# @<name>`。`# @tag` 可多值（空格分隔）；`# @data-flow` 是 key:value 段；`# @invariant` 每行一条。

### 输出 schema

```rust
FileAnalysis { path, purpose, file_tags, file_invariants, tests, exposed, fns }
PubItem { kind, name, line, tags, invariants, data_flow, children, fields, has_undocumented_fields }
FieldInfo { name, ty, line, has_doc, doc, vis }
```

完整定义见 `src/model.rs`。

### 关键实现细节

- `extract_fields` 处理 `Fields::Named` / `Unnamed` / `Unit`。
- `normalize_type` 把 `quote!` 渲染的 `Vec < Criterion >` 压成 `Vec<Criterion>`。
- `vis_to_string` 把 `Visibility::Restricted` 拍平成 `pub(crate)` / `pub(super)`。
- `tests_collected_from_cfg_test_mod` 识别 `mod tests` 和 `#[cfg(test)] mod`。

## 公开 API

```rust
pub fn analyze_file(path: &Path) -> Result<FileAnalysis, AnalyzeError>
pub fn analyze_source(path: &Path, src: &str) -> Result<FileAnalysis, AnalyzeError>
pub fn analyze_dir(root: &Path) -> Vec<(String, Result<FileAnalysis, AnalyzeError>)>
```

`analyze_dir` 返回 `Vec` 而不是 `Result`，允许调用者收集所有结果而不是被一个错误中断。

//! Output schema for the analyzer.
//!
//! Everything the code viewer consumes comes from these types. Keep them
//! JSON-stable: the web frontend depends on the field names.

use serde::Serialize;

/// Data-flow segment parsed from `/// # @data-flow` comments.
#[derive(Debug, Clone, Serialize, Default)]
pub struct DataFlow {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub side_effects: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub depends_on: Vec<String>,
}

/// Per-item metadata (tags / data-flow / invariants).
#[derive(Debug, Clone, Serialize, Default)]
pub struct ItemAttrs {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_flow: Option<DataFlow>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub invariants: Vec<String>,
}

/// File-level metadata (module purpose, file-level tags/invariants).
#[derive(Debug, Clone, Serialize, Default)]
pub struct FileAttrs {
    pub purpose: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub invariants: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PubKind {
    #[default]
    Struct,
    Enum,
    Trait,
    Fn,
    Const,
    Type,
    Impl,
    Mod,
}

/// A publicly-exposed item, with optional nested children (e.g. enum
/// variants, trait methods, mod contents).
#[derive(Debug, Clone, Serialize, Default)]
pub struct PubItem {
    pub kind: PubKind,
    pub name: String,
    pub line: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub invariants: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_flow: Option<DataFlow>,
    /// Enum variants, trait methods, mod contents, etc.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<PubItem>,
    /// Struct fields only. Empty for other kinds. The `has_doc` flag
    /// on each field is what the viewer highlights as "missing
    /// annotation" — a real review target.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<FieldInfo>,
    /// `true` if the struct has any field without a `///` doc comment.
    /// Pre-computed so the UI can show a single badge without scanning.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub has_undocumented_fields: bool,
}

/// One field of a struct. The `has_doc` flag is intentionally redundant
/// with `doc.is_some()` so the UI can render a "missing annotation"
/// column without parsing optional fields.
#[derive(Debug, Clone, Serialize)]
pub struct FieldInfo {
    pub name: String,
    /// Rendered type as it appears in source (e.g. `Vec<ContextLayer>`).
    pub ty: String,
    pub line: usize,
    pub has_doc: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doc: Option<String>,
    /// `""` for private, `"pub"`, `"pub(crate)"`, `"pub(super)"`, etc.
    pub vis: String,
}

/// A `#[test]` function (file-scope, not inside an explicit test mod).
#[derive(Debug, Clone, Serialize)]
pub struct TestInfo {
    pub name: String,
    pub line: usize,
    #[serde(default)]
    pub attrs: ItemAttrs,
}

/// A function (free or method) that has an explicit `/// # @data-flow`
/// annotation. Only these are listed — keeps the index focused on what
/// the user explicitly marked.
#[derive(Debug, Clone, Serialize)]
pub struct FnAnalysis {
    pub name: String,
    pub line: usize,
    #[serde(default)]
    pub attrs: ItemAttrs,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileAnalysis {
    pub path: String,
    pub purpose: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub file_tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub file_invariants: Vec<String>,
    #[serde(default)]
    pub tests: Vec<TestInfo>,
    pub exposed: Vec<PubItem>,
    #[serde(default)]
    pub fns: Vec<FnAnalysis>,
}

#[derive(Debug, thiserror::Error)]
pub enum AnalyzeError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("parse: {0}")]
    Parse(String),
}

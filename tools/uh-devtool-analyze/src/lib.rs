//! Rust source analyzer for the Ultimate Harness code viewer.
//!
//! **Boundary rule**: this crate reads `.rs` source via the filesystem
//! and `syn`. It MUST NOT depend on `uh-core` or any other crate under
//! `crates/`. See `Cargo.toml` at the workspace root for the policy.
//!
//! ## Doc-comment conventions
//!
//! See [`attrs`] for the supported `///` segments. They are plain
//! rustdoc comments — `cargo doc` renders them unchanged.

pub mod attrs;
pub mod model;
mod visit;

pub use model::{AnalyzeError, DataFlow, FileAnalysis, FileAttrs, FnAnalysis, ItemAttrs, PubItem, PubKind, TestInfo};
pub use visit::analyze_syntax;

use std::path::Path;

/// Read a `.rs` file from disk and analyze it. Returns a partial analysis
/// on parse errors via the inner `Result` — never panics on a single
/// broken file (future: fall back to a stub analysis).
pub fn analyze_file(path: &Path) -> Result<FileAnalysis, AnalyzeError> {
    let source = std::fs::read_to_string(path)?;
    let syntax = syn::parse_file(&source).map_err(|e| AnalyzeError::Parse(e.to_string()))?;
    Ok(analyze_syntax(path, &syntax))
}

/// Analyze in-memory source. Used by tests and by the `watch` command.
pub fn analyze_source(path: &Path, source: &str) -> Result<FileAnalysis, AnalyzeError> {
    let syntax = syn::parse_file(source).map_err(|e| AnalyzeError::Parse(e.to_string()))?;
    Ok(analyze_syntax(path, &syntax))
}

/// Walk a directory recursively and analyze every `*.rs` file.
/// Skips files that fail to parse; never errors out.
pub fn analyze_dir(root: &Path) -> Vec<(String, Result<FileAnalysis, AnalyzeError>)> {
    let mut out = Vec::new();
    for entry in walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("rs"))
    {
        let path = entry.path().to_path_buf();
        let result = analyze_file(&path);
        out.push((path.to_string_lossy().to_string(), result));
    }
    out
}

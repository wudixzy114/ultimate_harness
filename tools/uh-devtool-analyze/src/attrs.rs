//! Parser for `///` doc-comment conventions:
//!
//! ```text
//! /// # @tag planner executor
//! /// # @data-flow
//! /// input: UserMessage
//! /// output: Plan | Continue
//! /// side-effect: append L4.WorkingMemory (NOT cacheable)
//! /// depends-on: skill.resolve_intent
//! /// # @invariant
//! /// WorkingMemory never enters cacheable prefix
//! ```
//!
//! These are plain rustdoc comments — `cargo doc` renders them
//! unchanged. The analyzer only extracts structural segments.

use crate::model::{DataFlow, FileAttrs, ItemAttrs};
use syn::{Attribute, Lit, Meta};

/// Pull all `#[doc = "..."]` lines out of an attribute list.
pub fn collect_docs(attrs: &[Attribute]) -> Vec<String> {
    let mut out = Vec::new();
    for a in attrs {
        if !a.path().is_ident("doc") {
            continue;
        }
        if let Meta::NameValue(nv) = &a.meta {
            if let syn::Expr::Lit(syn::ExprLit {
                lit: Lit::Str(s), ..
            }) = &nv.value
            {
                out.push(s.value());
            }
        }
    }
    out
}

fn looks_like_test_cfg(a: &Attribute) -> bool {
    if !a.path().is_ident("cfg") {
        return false;
    }
    let s = quote::ToTokens::to_token_stream(a).to_string();
    s.contains("test")
}

pub(crate) fn is_test_mod(m: &syn::ItemMod) -> bool {
    if m.ident == "test" || m.ident == "tests" {
        return true;
    }
    m.attrs.iter().any(looks_like_test_cfg)
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum Section {
    None,
    DataFlow,
    Invariant,
}

/// Parse per-item metadata (struct / enum / trait / fn / const / type / impl / mod).
pub fn parse_item_attrs(docs: &[String]) -> ItemAttrs {
    let mut tags = Vec::new();
    let mut data_flow_lines: Vec<String> = Vec::new();
    let mut invariants: Vec<String> = Vec::new();
    let mut section = Section::None;

    for raw in docs {
        let t = raw.trim();
        if t.is_empty() {
            section = Section::None;
            continue;
        }
        if let Some(rest) = t.strip_prefix("# @tag") {
            section = Section::None;
            tags.extend(rest.split_whitespace().map(|s| s.to_string()));
            continue;
        }
        if t == "# @data-flow" {
            section = Section::DataFlow;
            continue;
        }
        if t.starts_with("# @invariant") {
            section = Section::Invariant;
            // Optional inline body, e.g. `# @invariant x`.
            if let Some(rest) = t.strip_prefix("# @invariant") {
                let body = rest.trim();
                if !body.is_empty() {
                    invariants.push(body.to_string());
                }
            }
            continue;
        }
        if t.starts_with("# @") {
            section = Section::None;
            continue;
        }
        match section {
            Section::DataFlow => data_flow_lines.push(t.to_string()),
            Section::Invariant => invariants.push(t.to_string()),
            Section::None => {}
        }
    }

    let data_flow = if data_flow_lines.is_empty() {
        None
    } else {
        Some(parse_data_flow(&data_flow_lines))
    };

    ItemAttrs {
        tags,
        data_flow,
        invariants,
    }
}

/// Parse file-level metadata. The "purpose" is the first contiguous
/// block of freeform text before any `# @...` section header or a
/// blank line.
pub fn parse_file_attrs(docs: &[String]) -> FileAttrs {
    let mut purpose_lines: Vec<String> = Vec::new();
    let mut tags: Vec<String> = Vec::new();
    let mut invariants: Vec<String> = Vec::new();
    let mut purpose_done = false;
    let mut section = Section::None;

    for raw in docs {
        let t = raw.trim();
        if t.is_empty() {
            purpose_done = true;
            section = Section::None;
            continue;
        }
        if let Some(rest) = t.strip_prefix("# @tag") {
            purpose_done = true;
            section = Section::None;
            tags.extend(rest.split_whitespace().map(|s| s.to_string()));
            continue;
        }
        if t == "# @data-flow" {
            purpose_done = true;
            section = Section::DataFlow;
            continue;
        }
        if t.starts_with("# @invariant") {
            purpose_done = true;
            section = Section::Invariant;
            if let Some(rest) = t.strip_prefix("# @invariant") {
                let body = rest.trim();
                if !body.is_empty() {
                    invariants.push(body.to_string());
                }
            }
            continue;
        }
        if t.starts_with("# @") {
            purpose_done = true;
            section = Section::None;
            continue;
        }
        if !purpose_done {
            purpose_lines.push(t.to_string());
        } else if section == Section::Invariant {
            invariants.push(t.to_string());
        }
    }

    FileAttrs {
        purpose: purpose_lines.join(" "),
        tags,
        invariants,
    }
}

fn parse_data_flow(lines: &[String]) -> DataFlow {
    let mut df = DataFlow::default();
    for line in lines {
        if let Some(rest) = strip_key(line, "input:") {
            df.input = Some(rest.to_string());
        } else if let Some(rest) = strip_key(line, "output:") {
            df.output = Some(rest.to_string());
        } else if let Some(rest) = strip_key(line, "side-effect:") {
            let body = rest.trim();
            if !body.is_empty() {
                df.side_effects.push(body.to_string());
            }
        } else if let Some(rest) = strip_key(line, "depends-on:") {
            let body = rest.trim();
            if !body.is_empty() {
                df.depends_on.push(body.to_string());
            }
        }
    }
    df
}

fn strip_key<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let t = line.trim();
    let rest = t.strip_prefix(key)?;
    if rest.is_empty() || rest.starts_with(char::is_whitespace) {
        Some(rest.trim_start())
    } else {
        None
    }
}

/// Re-export for use in visit.rs.
#[allow(unused_imports)]
pub(crate) use is_test_mod as item_mod_is_test;

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(s: &[&str]) -> Vec<String> {
        s.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn parse_item_data_flow() {
        let docs = lines(&[
            " Handle a user task.",
            " # @tag planner executor",
            " # @data-flow",
            " input: UserMessage",
            " output: Plan | Continue",
            " side-effect: append L4 (NOT cacheable)",
            " depends-on: skill.resolve_intent",
            " # @invariant",
            " WorkingMemory never enters cacheable prefix",
        ]);
        let attrs = parse_item_attrs(&docs);
        assert_eq!(attrs.tags, vec!["planner", "executor"]);
        let df = attrs.data_flow.expect("data-flow present");
        assert_eq!(df.input.as_deref(), Some("UserMessage"));
        assert_eq!(df.output.as_deref(), Some("Plan | Continue"));
        assert_eq!(df.side_effects, vec!["append L4 (NOT cacheable)"]);
        assert_eq!(df.depends_on, vec!["skill.resolve_intent"]);
        assert_eq!(attrs.invariants, vec!["WorkingMemory never enters cacheable prefix"]);
    }

    #[test]
    fn parse_file_purpose() {
        let docs = lines(&[
            "Plan and Step types — the heart of structured iteration.",
            "",
            "Long description that should NOT be part of purpose.",
            "# @tag schema",
        ]);
        let fa = parse_file_attrs(&docs);
        assert_eq!(fa.purpose, "Plan and Step types — the heart of structured iteration.");
        assert_eq!(fa.tags, vec!["schema"]);
    }

    #[test]
    fn strip_key_respects_word_boundary() {
        assert_eq!(strip_key("input: foo", "input:"), Some("foo"));
        assert_eq!(strip_key("inputs: foo", "input:"), None);
    }
}

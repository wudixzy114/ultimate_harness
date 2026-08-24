//! Walk a `syn::File` and emit [`FileAnalysis`].
//!
//! Only public items are surfaced in `exposed` — private helpers are
//! kept out of the review surface unless they have an explicit
//! `/// # @data-flow` block, in which case they appear in `fns`.

use crate::attrs::{collect_docs, is_test_mod, parse_file_attrs, parse_item_attrs};
use crate::model::{FileAnalysis, FnAnalysis, PubItem, PubKind, TestInfo};
use std::path::Path;
use syn::{Attribute, ImplItem, Item, ItemMod, TraitItem, Type, Visibility};

pub fn analyze_syntax(path: &Path, syntax: &syn::File) -> FileAnalysis {
    let file_docs = collect_docs(&syntax.attrs);
    let file_attrs = parse_file_attrs(&file_docs);

    let mut exposed: Vec<PubItem> = Vec::new();
    let mut tests: Vec<TestInfo> = Vec::new();
    let mut fns: Vec<FnAnalysis> = Vec::new();

    for item in &syntax.items {
        walk_item(item, None, &mut exposed, &mut tests, &mut fns);
    }

    FileAnalysis {
        path: path.to_string_lossy().to_string(),
        purpose: file_attrs.purpose,
        file_tags: file_attrs.tags,
        file_invariants: file_attrs.invariants,
        tests,
        exposed,
        fns,
    }
}

/// Borrow the `attrs` slice of any `Item` variant without cloning.
fn item_attrs(item: &Item) -> &[Attribute] {
    match item {
        Item::Const(c) => &c.attrs,
        Item::Enum(e) => &e.attrs,
        Item::ExternCrate(c) => &c.attrs,
        Item::Fn(f) => &f.attrs,
        Item::ForeignMod(m) => &m.attrs,
        Item::Impl(i) => &i.attrs,
        Item::Macro(m) => &m.attrs,
        Item::Mod(m) => &m.attrs,
        Item::Static(s) => &s.attrs,
        Item::Struct(s) => &s.attrs,
        Item::Trait(t) => &t.attrs,
        Item::TraitAlias(t) => &t.attrs,
        Item::Type(t) => &t.attrs,
        Item::Union(u) => &u.attrs,
        Item::Use(u) => &u.attrs,
        _ => &[],
    }
}

fn walk_item(
    item: &Item,
    parent: Option<&str>,
    exposed: &mut Vec<PubItem>,
    tests: &mut Vec<TestInfo>,
    fns: &mut Vec<FnAnalysis>,
) {
    let attrs = parse_item_attrs(&collect_docs(item_attrs(item)));

    match item {
        Item::Struct(s) if is_pub(&s.vis) => {
            exposed.push(PubItem {
                kind: PubKind::Struct,
                name: with_parent(parent, &s.ident.to_string()),
                line: s.ident.span().start().line,
                tags: attrs.tags,
                invariants: attrs.invariants,
                data_flow: attrs.data_flow,
                children: Vec::new(),
            });
        }
        Item::Enum(e) if is_pub(&e.vis) => {
            let children = e
                .variants
                .iter()
                .map(|v| PubItem {
                    kind: PubKind::Const,
                    name: v.ident.to_string(),
                    line: v.ident.span().start().line,
                    tags: Vec::new(),
                    invariants: Vec::new(),
                    data_flow: None,
                    children: Vec::new(),
                })
                .collect();
            exposed.push(PubItem {
                kind: PubKind::Enum,
                name: with_parent(parent, &e.ident.to_string()),
                line: e.ident.span().start().line,
                tags: attrs.tags,
                invariants: attrs.invariants,
                data_flow: attrs.data_flow,
                children,
            });
        }
        Item::Trait(t) if is_pub(&t.vis) => {
            let mut children = Vec::new();
            for ti in &t.items {
                if let TraitItem::Fn(f) = ti {
                    let child_docs = collect_docs(&f.attrs);
                    let child_attrs = parse_item_attrs(&child_docs);
                    let name = format!("{}::{}", t.ident, f.sig.ident);
                    children.push(PubItem {
                        kind: PubKind::Fn,
                        name: name.clone(),
                        line: f.sig.ident.span().start().line,
                        tags: child_attrs.tags.clone(),
                        invariants: child_attrs.invariants.clone(),
                        data_flow: child_attrs.data_flow.clone(),
                        children: Vec::new(),
                    });
                    if child_attrs.data_flow.is_some() {
                        fns.push(FnAnalysis {
                            name,
                            line: f.sig.ident.span().start().line,
                            attrs: child_attrs,
                        });
                    }
                }
            }
            exposed.push(PubItem {
                kind: PubKind::Trait,
                name: with_parent(parent, &t.ident.to_string()),
                line: t.ident.span().start().line,
                tags: attrs.tags,
                invariants: attrs.invariants,
                data_flow: attrs.data_flow,
                children,
            });
        }
        Item::Fn(f) if is_pub(&f.vis) => {
            let name = with_parent(parent, &f.sig.ident.to_string());
            exposed.push(PubItem {
                kind: PubKind::Fn,
                name: name.clone(),
                line: f.sig.ident.span().start().line,
                tags: attrs.tags.clone(),
                invariants: attrs.invariants.clone(),
                data_flow: attrs.data_flow.clone(),
                children: Vec::new(),
            });
            if attrs.data_flow.is_some() {
                fns.push(FnAnalysis {
                    name,
                    line: f.sig.ident.span().start().line,
                    attrs,
                });
            }
        }
        Item::Const(c) if is_pub(&c.vis) => {
            exposed.push(PubItem {
                kind: PubKind::Const,
                name: with_parent(parent, &c.ident.to_string()),
                line: c.ident.span().start().line,
                tags: attrs.tags,
                invariants: attrs.invariants,
                data_flow: attrs.data_flow,
                children: Vec::new(),
            });
        }
        Item::Type(t) if is_pub(&t.vis) => {
            exposed.push(PubItem {
                kind: PubKind::Type,
                name: with_parent(parent, &t.ident.to_string()),
                line: t.ident.span().start().line,
                tags: attrs.tags,
                invariants: attrs.invariants,
                data_flow: attrs.data_flow,
                children: Vec::new(),
            });
        }
        Item::Impl(i) => {
            let type_name = type_short_name(&i.self_ty);
            for ii in &i.items {
                if let ImplItem::Fn(f) = ii {
                    let child_docs = collect_docs(&f.attrs);
                    let child_attrs = parse_item_attrs(&child_docs);
                    let name = format!("{}::{}", type_name, f.sig.ident);
                    let visible = matches!(f.vis, Visibility::Public(_)) || i.trait_.is_some();
                    if visible {
                        exposed.push(PubItem {
                            kind: PubKind::Impl,
                            name: name.clone(),
                            line: f.sig.ident.span().start().line,
                            tags: child_attrs.tags.clone(),
                            invariants: child_attrs.invariants.clone(),
                            data_flow: child_attrs.data_flow.clone(),
                            children: Vec::new(),
                        });
                    }
                    if child_attrs.data_flow.is_some() {
                        fns.push(FnAnalysis {
                            name,
                            line: f.sig.ident.span().start().line,
                            attrs: child_attrs,
                        });
                    }
                }
            }
        }
        Item::Mod(m) if is_pub(&m.vis) => {
            let mut children = Vec::new();
            let mut sub_fns = Vec::new();
            if let Some((_, items)) = &m.content {
                for sub in items {
                    walk_item(sub, Some(&m.ident.to_string()), &mut children, tests, &mut sub_fns);
                }
            }
            exposed.push(PubItem {
                kind: PubKind::Mod,
                name: with_parent(parent, &m.ident.to_string()),
                line: m.ident.span().start().line,
                tags: attrs.tags,
                invariants: attrs.invariants,
                data_flow: attrs.data_flow,
                children,
            });
            fns.extend(sub_fns);
        }
        _ => {}
    }

    if let Item::Mod(m) = item {
        if is_test_mod(m) {
            collect_tests_in_mod(m, tests);
        }
    }
}

fn collect_tests_in_mod(m: &ItemMod, tests: &mut Vec<TestInfo>) {
    let Some((_, items)) = &m.content else { return };
    for sub in items {
        if let Item::Fn(f) = sub {
            let is_test = f.attrs.iter().any(|a| a.path().is_ident("test"));
            if is_test {
                let docs = collect_docs(&f.attrs);
                tests.push(TestInfo {
                    name: f.sig.ident.to_string(),
                    line: f.sig.ident.span().start().line,
                    attrs: parse_item_attrs(&docs),
                });
            }
        }
    }
}

fn is_pub(v: &Visibility) -> bool {
    matches!(v, Visibility::Public(_))
}

fn with_parent(parent: Option<&str>, name: &str) -> String {
    match parent {
        Some(p) => format!("{p}::{name}"),
        None => name.to_string(),
    }
}

fn type_short_name(t: &Type) -> String {
    match t {
        Type::Path(p) => p
            .path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default(),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pub_struct_is_surfaced() {
        let src = r#"
            /// A point.
            pub struct Point { pub x: i32, pub y: i32 }
        "#;
        let f = syn::parse_file(src).unwrap();
        let a = analyze_syntax(Path::new("t.rs"), &f);
        assert_eq!(a.exposed.len(), 1);
        assert!(matches!(a.exposed[0].kind, PubKind::Struct));
        assert_eq!(a.exposed[0].name, "Point");
    }

    #[test]
    fn priv_struct_is_hidden() {
        let src = "struct Hidden { x: i32 }";
        let f = syn::parse_file(src).unwrap();
        let a = analyze_syntax(Path::new("t.rs"), &f);
        assert!(a.exposed.is_empty());
    }

    #[test]
    fn file_purpose_captured() {
        let src = "//! Plan types — the heart of iteration.\npub struct Plan;\n";
        let f = syn::parse_file(src).unwrap();
        let a = analyze_syntax(Path::new("t.rs"), &f);
        assert_eq!(a.purpose, "Plan types — the heart of iteration.");
    }

    #[test]
    fn tests_collected_from_cfg_test_mod() {
        let src = r#"
            pub fn add(a: i32, b: i32) -> i32 { a + b }

            #[cfg(test)]
            mod tests {
                #[test]
                fn add_basic() { assert_eq!(super::add(1, 2), 3); }
            }
        "#;
        let f = syn::parse_file(src).unwrap();
        let a = analyze_syntax(Path::new("t.rs"), &f);
        assert_eq!(a.exposed.len(), 1);
        assert_eq!(a.tests.len(), 1);
        assert_eq!(a.tests[0].name, "add_basic");
    }
}

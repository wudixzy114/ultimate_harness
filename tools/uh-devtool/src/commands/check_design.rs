//! `uh-devtool check-design <PATH>` — verify that every `.rs` file has a
//! nearby `docs.md` ancestor. Exits 1 if any file lacks one.
//!
//! Walk strategy: depth-first, every `.rs` is checked independently
//! against its nearest `docs.md` ancestor. The root passed by the user
//! is included; a file at the root itself is checked against the root
//! directly (and then its parent — so a single `docs.md` at the repo
//! root covers everything if needed).

use std::path::{Path, PathBuf};

pub fn run(root: &Path) -> anyhow::Result<()> {
    let mut missing: Vec<(PathBuf, Vec<PathBuf>)> = Vec::new();
    let mut checked: usize = 0;

    for entry in walkdir::WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| !is_excluded(e.path()))
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        checked += 1;
        let rs_path = entry.path().to_path_buf();
        match find_nearest_docs(&rs_path) {
            Some(_) => {}
            None => {
                let tried = ancestor_dirs(&rs_path);
                missing.push((rs_path, tried));
            }
        }
    }

    eprintln!("check-design: {checked} .rs file(s) checked under {}", root.display());
    if missing.is_empty() {
        eprintln!("all files have a nearby docs.md ancestor (Warn-or-better policy)");
        Ok(())
    } else {
        eprintln!("{} file(s) missing nearby docs.md:", missing.len());
        for (file, tried) in &missing {
            eprintln!("  {}", file.display());
            eprintln!("    tried: {}", format_tried(tried));
        }
        std::process::exit(1);
    }
}

fn find_nearest_docs(rs_file: &Path) -> Option<PathBuf> {
    let mut dir = rs_file.parent()?;
    loop {
        let candidate = dir.join("docs.md");
        if candidate.is_file() {
            return Some(candidate);
        }
        match dir.parent() {
            Some(p) => dir = p,
            None => return None,
        }
    }
}

fn ancestor_dirs(rs_file: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut dir = match rs_file.parent() {
        Some(d) => d,
        None => return out,
    };
    loop {
        out.push(dir.join("docs.md"));
        match dir.parent() {
            Some(p) => dir = p,
            None => return out,
        }
    }
}

fn format_tried(tried: &[PathBuf]) -> String {
    tried
        .iter()
        .map(|p| p.display().to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

/// Default-excluded directory names. These are build artifacts, VCS
/// internals, or vendored reference code — not part of the project's
/// design surface.
const EXCLUDED_DIRS: &[&str] = &["target", ".git", "node_modules"];

fn is_excluded(path: &Path) -> bool {
    path.components().any(|c| {
        let s = c.as_os_str().to_string_lossy();
        EXCLUDED_DIRS.iter().any(|ex| s == *ex)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn finds_docs_in_same_dir() {
        let tmp = tempdir();
        fs::write(tmp.join("docs.md"), "# design").unwrap();
        fs::write(tmp.join("foo.rs"), "pub fn x() {}").unwrap();
        assert!(find_nearest_docs(&tmp.join("foo.rs")).is_some());
    }

    #[test]
    fn walks_up_to_ancestor() {
        let tmp = tempdir();
        fs::write(tmp.join("docs.md"), "# design").unwrap();
        fs::create_dir(tmp.join("src")).unwrap();
        fs::write(tmp.join("src").join("foo.rs"), "pub fn x() {}").unwrap();
        let found = find_nearest_docs(&tmp.join("src").join("foo.rs")).unwrap();
        assert!(found.ends_with("docs.md"));
    }

    #[test]
    fn returns_none_when_no_ancestor_has_docs() {
        let tmp = tempdir();
        fs::create_dir_all(tmp.join("a").join("b")).unwrap();
        fs::write(tmp.join("a").join("b").join("foo.rs"), "pub fn x() {}").unwrap();
        assert!(find_nearest_docs(&tmp.join("a").join("b").join("foo.rs")).is_none());
    }

    fn tempdir() -> PathBuf {
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let base = std::env::temp_dir().join(format!(
            "uh-devtool-check-design-{}-{n}",
            std::process::id(),
        ));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        base
    }
}

//! `uh-devtool serve` — minimal HTTP server exposing the analyzer's JSON.
//!
//! Endpoints:
//!   GET /                       Built-in placeholder HTML (real SPA lives in apps/web).
//!   GET /api/index              List of `{path, purpose, file_tags}` for every file.
//!   GET /api/file?path=<rel>    Full analysis of a single file.
//!   GET /api/health             `{"ok": true}`
//!
//! Shared with `watch` (which reuses `analyze_into` and `handle` to keep
//! the HTTP surface identical).

use std::collections::BTreeMap;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

pub(crate) type State = RwLock<BTreeMap<String, uh_devtool_analyze::FileAnalysis>>;
pub(crate) type SharedState = Arc<State>;

pub fn run(root: &Path, port: u16) -> anyhow::Result<()> {
    let addr = format!("127.0.0.1:{port}");
    let server = tiny_http::Server::http(&addr)
        .map_err(|e| anyhow::anyhow!("http bind {addr}: {e}"))?;
    eprintln!("uh-devtool serving {root:?} on http://{addr}");
    eprintln!("  GET /api/index");
    eprintln!("  GET /api/file?path=<rel-path>");

    let state = Arc::new(RwLock::new(analyze_into(root)));

    for req in server.incoming_requests() {
        let url = req.url().to_string();
        let (path, query) = match url.split_once('?') {
            Some((p, q)) => (p, q),
            None => (url.as_str(), ""),
        };
        let resp = handle(path, query, &state);
        if let Err(e) = req.respond(resp) {
            eprintln!("respond error: {e}");
        }
    }
    Ok(())
}

pub(crate) fn analyze_into(root: &Path) -> BTreeMap<String, uh_devtool_analyze::FileAnalysis> {
    let mut out = BTreeMap::new();
    let root = root.to_path_buf();
    for (p, r) in uh_devtool_analyze::analyze_dir(&root) {
        if let Ok(a) = r {
            let rel = relative(&root, &p);
            out.insert(rel, a);
        }
    }
    out
}

fn relative(root: &Path, p: &str) -> String {
    let pb = PathBuf::from(p);
    pb.strip_prefix(root)
        .map(|x| x.to_string_lossy().to_string())
        .unwrap_or_else(|_| p.to_string())
}

pub(crate) fn handle(path: &str, query: &str, state: &SharedState) -> tiny_http::Response<Cursor<Vec<u8>>> {
    match (path, query) {
        ("/api/health", _) => json(200, r#"{"ok":true}"#),
        ("/api/index", _) => {
            let s = state.read().unwrap();
            let summary: Vec<_> = s
                .iter()
                .map(|(rel, a)| {
                    serde_json::json!({
                        "path": rel,
                        "purpose": a.purpose,
                        "file_tags": a.file_tags,
                        "exposed_count": a.exposed.len(),
                        "test_count": a.tests.len(),
                    })
                })
                .collect();
            match serde_json::to_string_pretty(&summary) {
                Ok(body) => json(200, &body),
                Err(e) => json(500, &format!("{{\"error\":\"{e}\"}}")),
            }
        }
        ("/api/file", q) if q.starts_with("path=") => {
            let rel = percent_decode(&q[5..]);
            let s = state.read().unwrap();
            match s.get(&rel) {
                Some(a) => match serde_json::to_string_pretty(a) {
                    Ok(body) => json(200, &body),
                    Err(e) => json(500, &format!("{{\"error\":\"{e}\"}}")),
                },
                None => json(404, &format!("{{\"error\":\"not found: {rel}\"}}")),
            }
        }
        ("/", _) => text(200, INDEX_HTML, "text/html; charset=utf-8"),
        _ => text(404, "not found", "text/plain; charset=utf-8"),
    }
}

fn json(code: u16, body: &str) -> tiny_http::Response<Cursor<Vec<u8>>> {
    tiny_http::Response::from_string(body.to_string())
        .with_status_code(tiny_http::StatusCode(code))
        .with_header(
            "Content-Type: application/json; charset=utf-8"
                .parse::<tiny_http::Header>()
                .unwrap(),
        )
}

fn text(code: u16, body: &str, ct: &str) -> tiny_http::Response<Cursor<Vec<u8>>> {
    let body = body.to_string();
    let h = format!("Content-Type: {ct}").parse::<tiny_http::Header>().unwrap();
    tiny_http::Response::from_string(body)
        .with_status_code(tiny_http::StatusCode(code))
        .with_header(h)
}

fn percent_decode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(b) = u8::from_str_radix(
                std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or("00"),
                16,
            ) {
                out.push(b as char);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

const INDEX_HTML: &str = r#"<!doctype html>
<html><head><meta charset="utf-8"><title>uh-devtool</title></head>
<body>
<h1>uh-devtool</h1>
<p>Code viewer API is running. See <code>apps/web/src/routes/review/</code> for the SPA.</p>
<ul>
  <li><a href="/api/health">/api/health</a></li>
  <li><a href="/api/index">/api/index</a></li>
</ul>
</body></html>
"#;

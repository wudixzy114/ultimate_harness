//! `uh-devtool watch` — `serve` + a notify watcher that re-analyzes the
//! workspace on change. Push notifications are intentionally absent in
//! this MVP: the SPA polls `/api/index` and re-renders on diff.

use crate::commands::serve::{analyze_into, handle, SharedState};
use notify::{event::EventKind, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

pub fn run(root: &Path, port: u16) -> anyhow::Result<()> {
    let root = root.to_path_buf();
    let addr = format!("127.0.0.1:{port}");
    let server = tiny_http::Server::http(&addr)
        .map_err(|e| anyhow::anyhow!("http bind {addr}: {e}"))?;
    eprintln!("uh-devtool watching {root:?} on http://{addr}");

    let state: SharedState = Arc::new(std::sync::RwLock::new(analyze_into(&root)));

    // Background watcher: debounce rapid bursts, then re-analyze whole
    // tree (single crate = < 50ms; cheap enough that incremental is
    // premature optimization).
    let (tx, rx) = mpsc::channel::<notify::Result<notify::Event>>();
    let mut watcher = notify::recommended_watcher(move |res| {
        let _ = tx.send(res);
    })?;
    watcher.watch(&root, RecursiveMode::Recursive)?;

    let state_for_watcher = Arc::clone(&state);
    let root_for_watcher = root.clone();
    thread::spawn(move || {
        // Debounce: collapse events within 200ms windows.
        let debounce = Duration::from_millis(200);
        let mut last = Instant::now() - debounce;
        let mut pending = false;
        for res in rx {
            match res {
                Ok(ev) => {
                    if matches!(
                        ev.kind,
                        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
                    ) {
                        pending = true;
                        last = Instant::now();
                    }
                }
                Err(e) => eprintln!("watch error: {e}"),
            }
            if pending && last.elapsed() >= debounce {
                pending = false;
                let t0 = Instant::now();
                let new_state = analyze_into(&root_for_watcher);
                let mut s = state_for_watcher.write().unwrap();
                *s = new_state;
                eprintln!("re-analyzed in {:?}", t0.elapsed());
            }
        }
    });

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

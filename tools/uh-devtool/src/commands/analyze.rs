//! `uh-devtool analyze <PATH>` — emit JSON for a file or a directory.

use std::path::Path;

pub fn run(path: &Path, pretty: bool) -> anyhow::Result<()> {
    if path.is_dir() {
        let results = uh_devtool_analyze::analyze_dir(path);
        let (oks, errs): (Vec<_>, Vec<_>) = results.into_iter().partition(|(_, r)| r.is_ok());
        if !errs.is_empty() {
            for (p, r) in &errs {
                if let Err(e) = r {
                    eprintln!("warn: {p}: {e}");
                }
            }
        }
        let payload: Vec<_> = oks.into_iter().filter_map(|(p, r)| r.ok().map(|a| (p, a))).collect();
        if pretty {
            println!("{}", serde_json::to_string_pretty(&payload)?);
        } else {
            println!("{}", serde_json::to_string(&payload)?);
        }
    } else {
        let analysis = uh_devtool_analyze::analyze_file(path)?;
        if pretty {
            println!("{}", serde_json::to_string_pretty(&analysis)?);
        } else {
            println!("{}", serde_json::to_string(&analysis)?);
        }
    }
    Ok(())
}

//! Ultimate Harness — entry point.
//!
//! v0.0.3: starts the daemon on `uh.toml` config.

use std::sync::Arc;

use anyhow::{Context, Result};
use std::net::SocketAddr;
use std::path::Path;
use tracing_subscriber::EnvFilter;
use uh_daemon::{Config, serve};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,uh=debug")))
        .init();

    // Parse --config / positional arg, default to "uh.toml".
    let args: Vec<String> = std::env::args().skip(1).collect();
    let config_path = parse_config_path(&args).unwrap_or_else(|| "uh.toml".into());
    let config = Config::load_from(Path::new(&config_path))
        .with_context(|| format!("loading {config_path}"))?;
    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port)
        .parse()
        .context("invalid host:port in config")?;
    let state = Arc::new(config.into_state().context("building AppState")?);

    tracing::info!(
        "Ultimate Harness v{} starting on http://{}",
        env!("CARGO_PKG_VERSION"),
        addr,
    );

    serve(state, addr).await
}

fn parse_config_path(args: &[String]) -> Option<String> {
    // Accept: --config PATH, -c PATH, or just PATH
    if let Some(i) = args.iter().position(|a| a == "--config" || a == "-c") {
        return args.get(i + 1).cloned();
    }
    if let Some(first) = args.first() {
        if !first.starts_with('-') {
            return Some(first.clone());
        }
    }
    None
}

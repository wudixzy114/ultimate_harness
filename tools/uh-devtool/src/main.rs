//! `uh-devtool` — CLI for the Ultimate Harness code viewer.
//!
//! Subcommands:
//!   analyze <PATH>     Print JSON analysis of a file or directory.
//!   serve [--port N]   Run an HTTP server serving the analyzer's JSON
//!                      and a built-in SPA placeholder.
//!   watch [--port N]   Like `serve` but also watches the workspace and
//!                      pushes incremental updates over SSE.

mod commands;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "uh-devtool", version, about = "Code viewer for Ultimate Harness", long_about = None)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Analyze a single file or recursively a directory and print JSON.
    Analyze {
        /// File or directory to analyze.
        path: PathBuf,
        /// Pretty-print JSON.
        #[arg(long, default_value_t = true)]
        pretty: bool,
    },
    /// Run an HTTP server exposing the analyzer's JSON.
    Serve {
        /// Root directory to analyze.
        #[arg(long, default_value = "crates")]
        root: PathBuf,
        /// Port to bind.
        #[arg(long, default_value_t = 7700)]
        port: u16,
    },
    /// Watch the workspace for changes and push updates via SSE.
    Watch {
        /// Root directory to analyze and watch.
        #[arg(long, default_value = "crates")]
        root: PathBuf,
        /// Port to bind.
        #[arg(long, default_value_t = 7700)]
        port: u16,
    },
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Analyze { path, pretty } => commands::analyze::run(&path, pretty),
        Cmd::Serve { root, port } => commands::serve::run(&root, port),
        Cmd::Watch { root, port } => commands::watch::run(&root, port),
    }
}

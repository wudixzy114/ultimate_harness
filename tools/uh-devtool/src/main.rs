//! `uh-devtool` — CLI for the Ultimate Harness code viewer / design-policy
//! enforcer.
//!
//! Subcommands:
//!   analyze <PATH>           Print JSON analysis of a file or directory.
//!   serve [--port N]         Run an HTTP server serving the analyzer's JSON.
//!   watch [--port N]         Like `serve` plus file-watch incremental updates.
//!   check-design <PATH>      Verify every .rs has a nearby docs.md ancestor.
//!                            Exits 1 on missing docs (CI-friendly).

mod commands;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "uh-devtool", version, about = "Code viewer / design-policy enforcer for Ultimate Harness", long_about = None)]
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
    /// Verify every .rs file under <PATH> has a nearby docs.md ancestor.
    /// Exits 1 if any file lacks one. See `docs/design-first.md`.
    CheckDesign {
        /// Root directory to scan.
        path: PathBuf,
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
        Cmd::CheckDesign { path } => commands::check_design::run(&path),
    }
}

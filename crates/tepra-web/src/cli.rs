//! CLI definition for the tepra binary.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// tepra: TEPRA Creator `WebAPI` facade server.
#[derive(Debug, Parser)]
#[command(name = "tepra", version, about)]
pub struct Cli {
    /// Subcommand to run.
    #[command(subcommand)]
    pub command: Commands,
}

/// Top-level subcommands.
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Start the HTTP server.
    Serve(ServeArgs),
    /// Print the binary version and exit.
    Version,
}

/// Arguments for the `serve` subcommand.
#[derive(Debug, clap::Args)]
pub struct ServeArgs {
    /// Directory containing label template files.
    #[arg(long, value_name = "PATH")]
    pub template_dir: Option<PathBuf>,

    /// Address to bind the HTTP server to.
    #[arg(long, value_name = "ADDR")]
    pub bind: Option<String>,

    /// Base URL of the TEPRA Creator `WebAPI`.
    #[arg(long, value_name = "URL")]
    pub creator_base: Option<String>,

    /// Path to the config file (TOML). If omitted, `./tepra.toml` is probed silently.
    #[arg(long, value_name = "PATH")]
    pub config: Option<PathBuf>,
}

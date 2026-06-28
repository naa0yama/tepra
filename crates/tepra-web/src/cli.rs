//! CLI definition for the tepra-api binary.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// tepra-api: TEPRA Creator `WebAPI` facade server.
#[derive(Debug, Parser)]
#[command(name = "tepra-api", version, about)]
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
    pub template_dir: PathBuf,

    /// Address to bind the HTTP server to.
    #[arg(long, default_value = "0.0.0.0:3000", value_name = "ADDR")]
    pub bind: String,

    /// Base URL of the TEPRA Creator `WebAPI`.
    #[arg(long, default_value = "http://localhost:29108", value_name = "URL")]
    pub creator_base: String,
}

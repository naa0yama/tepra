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
    /// Manage the config file.
    Config(ConfigArgs),
    /// Print the binary version and exit.
    Version,
}

/// Arguments for the `config` subcommand.
#[derive(Debug, clap::Args)]
pub struct ConfigArgs {
    /// Action to perform.
    #[command(subcommand)]
    pub action: ConfigAction,
}

/// Actions under the `config` subcommand.
#[derive(Debug, Subcommand)]
pub enum ConfigAction {
    /// Write a default `tepra.toml` with schema comments.
    Init(ConfigInitArgs),
}

/// Arguments for `config init`.
#[derive(Debug, clap::Args)]
pub struct ConfigInitArgs {
    /// Path to write the config file to.
    #[arg(long, value_name = "PATH", default_value = "tepra.toml")]
    pub path: PathBuf,
    /// Overwrite the file if it already exists.
    #[arg(long)]
    pub force: bool,
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

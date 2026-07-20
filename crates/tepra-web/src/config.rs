//! Config resolution for `tepra serve`.

use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result};
use figment::{
    Figment,
    providers::{Env, Format as _, Serialized, Toml},
};
use serde::{Deserialize, Serialize};

use crate::cli::ServeArgs;

/// Resolved configuration for the `serve` subcommand.
// WHY-NOT: rename to `Config` — public API in spec uses `ServeConfig`; renaming
// conflicts with plan and breaks consumer call sites.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServeConfig {
    /// Directory containing label template files.
    pub template_dir: PathBuf,
    /// Address to bind the HTTP server to.
    pub bind: String,
    /// Base URL of the TEPRA Creator `WebAPI`.
    pub creator_base: String,
}

impl Default for ServeConfig {
    fn default() -> Self {
        Self {
            template_dir: PathBuf::from("templates"),
            bind: "0.0.0.0:3000".to_owned(),
            creator_base: "http://localhost:29108".to_owned(),
        }
    }
}

enum ConfigPathResolution {
    Explicit(PathBuf),
    AutoFound(PathBuf),
    None,
}

fn resolve_config_path(explicit: Option<&Path>) -> Result<ConfigPathResolution> {
    if let Some(p) = explicit {
        if p.exists() {
            return Ok(ConfigPathResolution::Explicit(p.to_owned()));
        }
        return Err(anyhow::anyhow!("config file not found: {}", p.display()))
            .with_context(|| format!("--config {}", p.display()));
    }
    let auto = PathBuf::from("tepra.toml");
    if auto.exists() {
        return Ok(ConfigPathResolution::AutoFound(auto));
    }
    Ok(ConfigPathResolution::None)
}

/// CLI-supplied overrides; `None` fields are excluded from figment merge.
#[derive(Serialize)]
struct CliOverrides {
    #[serde(skip_serializing_if = "Option::is_none")]
    template_dir: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    creator_base: Option<String>,
}

fn build_figment(path: Option<&Path>, overrides: &CliOverrides) -> Figment {
    let mut fig = Figment::from(Serialized::defaults(ServeConfig::default()));
    if let Some(p) = path {
        fig = fig.merge(Toml::file(p));
    }
    fig.merge(Env::prefixed("TEPRA_"))
        .merge(Serialized::defaults(overrides))
}

/// Merge CLI args, env vars, config file, and built-in defaults into a
/// [`ServeConfig`].
///
/// # Errors
///
/// Returns an error if `--config <PATH>` was provided but the file is not found,
/// or if the config file cannot be parsed, or if figment extraction fails.
// WHY-NOT: rename to `load` — `load` alone is ambiguous at call sites; spec uses
// `load_config` as the public entry point name.
#[allow(clippy::module_name_repetitions)]
pub fn load_config(args: &ServeArgs) -> Result<ServeConfig> {
    let resolution =
        resolve_config_path(args.config.as_deref()).context("resolving config path")?;
    let path = match &resolution {
        ConfigPathResolution::Explicit(p) | ConfigPathResolution::AutoFound(p) => Some(p.as_path()),
        ConfigPathResolution::None => None,
    };
    let overrides = CliOverrides {
        template_dir: args.template_dir.clone(),
        bind: args.bind.clone(),
        creator_base: args.creator_base.clone(),
    };
    build_figment(path, &overrides)
        .extract::<ServeConfig>()
        .context("extracting ServeConfig from figment")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_expected_values() {
        let cfg = ServeConfig::default();
        assert_eq!(cfg.template_dir, PathBuf::from("templates"));
        assert_eq!(cfg.bind, "0.0.0.0:3000");
        assert_eq!(cfg.creator_base, "http://localhost:29108");
    }
}

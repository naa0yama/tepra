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
/// [`ServeConfig`], and return the resolved config file path (if any).
///
/// The second element of the tuple is `Some(path)` when a config file was
/// used (either `--config`-explicit or auto-discovered), `None` otherwise.
///
/// # Errors
///
/// Returns an error if `--config <PATH>` was provided but the file is not found,
/// or if the config file cannot be parsed, or if figment extraction fails.
// WHY-NOT: rename to `load` — `load` alone is ambiguous at call sites; spec uses
// `load_config` as the public entry point name.
#[allow(clippy::module_name_repetitions)]
pub fn load_config(args: &ServeArgs) -> Result<(ServeConfig, Option<PathBuf>)> {
    let resolution =
        resolve_config_path(args.config.as_deref()).context("resolving config path")?;
    let config_file_path = match &resolution {
        ConfigPathResolution::Explicit(p) | ConfigPathResolution::AutoFound(p) => Some(p.clone()),
        ConfigPathResolution::None => None,
    };
    let path = config_file_path.as_deref();
    let overrides = CliOverrides {
        template_dir: args.template_dir.clone(),
        bind: args.bind.clone(),
        creator_base: args.creator_base.clone(),
    };
    let cfg = build_figment(path, &overrides)
        .extract::<ServeConfig>()
        .context("extracting ServeConfig from figment")?;
    Ok((cfg, config_file_path))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use figment::Jail;

    use super::*;

    #[test]
    fn default_config_has_expected_values() {
        let cfg = ServeConfig::default();
        assert_eq!(cfg.template_dir, PathBuf::from("templates"));
        assert_eq!(cfg.bind, "0.0.0.0:3000");
        assert_eq!(cfg.creator_base, "http://localhost:29108");
    }

    #[test]
    // WHY-NOT: box figment::Error — Jail::expect_with signature fixes the closure's
    // return type to figment::error::Result<()>; boxing the error would require a
    // wrapper type that doesn't implement figment's error trait.
    #[allow(clippy::result_large_err)]
    fn precedence_cli_over_env_over_file_over_default_when_all_provided() {
        Jail::expect_with(|jail| {
            jail.create_file(
                "tepra.toml",
                r#"
bind = "198.51.100.1:2222"
template_dir = "file-templates"
creator_base = "http://198.51.100.1:9000"
"#,
            )
            .expect("tepra.toml 作成失敗");
            jail.set_env("TEPRA_BIND", "198.51.100.2:3333");
            jail.set_env("TEPRA_TEMPLATE_DIR", "env-templates");
            jail.set_env("TEPRA_CREATOR_BASE", "http://198.51.100.2:9000");

            let cli = CliOverrides {
                bind: Some("198.51.100.3:4444".to_owned()),
                template_dir: Some(PathBuf::from("cli-templates")),
                creator_base: Some("http://198.51.100.3:9000".to_owned()),
            };

            let cfg: ServeConfig = build_figment(Some(Path::new("tepra.toml")), &cli)
                .extract()
                .unwrap();

            assert_eq!(cfg.bind, "198.51.100.3:4444");
            assert_eq!(cfg.template_dir, PathBuf::from("cli-templates"));
            assert_eq!(cfg.creator_base, "http://198.51.100.3:9000");

            Ok(())
        });
    }

    #[test]
    #[allow(clippy::result_large_err)]
    fn env_overrides_file_when_no_cli() {
        Jail::expect_with(|jail| {
            jail.create_file(
                "tepra.toml",
                r#"
bind = "198.51.100.1:2222"
template_dir = "file-templates"
creator_base = "http://198.51.100.1:9000"
"#,
            )
            .expect("tepra.toml 作成失敗");
            jail.set_env("TEPRA_BIND", "198.51.100.2:3333");
            jail.set_env("TEPRA_TEMPLATE_DIR", "env-templates");
            jail.set_env("TEPRA_CREATOR_BASE", "http://198.51.100.2:9000");

            let cli = CliOverrides {
                bind: None,
                template_dir: None,
                creator_base: None,
            };

            let cfg: ServeConfig = build_figment(Some(Path::new("tepra.toml")), &cli)
                .extract()
                .unwrap();

            assert_eq!(cfg.bind, "198.51.100.2:3333");
            assert_eq!(cfg.template_dir, PathBuf::from("env-templates"));
            assert_eq!(cfg.creator_base, "http://198.51.100.2:9000");

            Ok(())
        });
    }

    #[test]
    #[allow(clippy::result_large_err)]
    fn file_overrides_default_when_no_env_no_cli() {
        Jail::expect_with(|jail| {
            jail.create_file(
                "tepra.toml",
                r#"
bind = "198.51.100.1:2222"
template_dir = "file-templates"
creator_base = "http://198.51.100.1:9000"
"#,
            )
            .expect("tepra.toml 作成失敗");

            let cli = CliOverrides {
                bind: None,
                template_dir: None,
                creator_base: None,
            };

            let cfg: ServeConfig = build_figment(Some(Path::new("tepra.toml")), &cli)
                .extract()
                .unwrap();

            assert_eq!(cfg.bind, "198.51.100.1:2222");
            assert_eq!(cfg.template_dir, PathBuf::from("file-templates"));
            assert_eq!(cfg.creator_base, "http://198.51.100.1:9000");

            Ok(())
        });
    }

    #[test]
    #[allow(clippy::result_large_err)]
    fn resolve_config_path_returns_none_when_no_file_and_no_explicit() {
        Jail::expect_with(|jail| {
            // empty jail directory — no tepra.toml present
            let _ = jail;
            let result = resolve_config_path(None).unwrap();
            assert!(matches!(result, ConfigPathResolution::None));
            Ok(())
        });
    }

    #[test]
    fn resolve_config_path_returns_error_when_explicit_missing() {
        let result = resolve_config_path(Some(Path::new("/nonexistent/tepra.toml")));
        assert!(result.is_err());
    }

    #[test]
    #[allow(clippy::result_large_err)]
    fn resolve_config_path_returns_auto_found_when_cwd_file_exists() {
        Jail::expect_with(|jail| {
            jail.create_file("tepra.toml", "")
                .expect("tepra.toml 作成失敗");
            let result = resolve_config_path(None).unwrap();
            assert!(matches!(result, ConfigPathResolution::AutoFound(_)));
            Ok(())
        });
    }
}

#![allow(missing_docs)]

use assert_cmd::Command;
use clap::Parser as _;
use tepra_web::cli::{Cli, Commands};

// ---------------------------------------------------------------------------
// version subcommand
// ---------------------------------------------------------------------------

#[test]
#[cfg_attr(miri, ignore)]
fn version_subcommand_exits_success() {
    Command::cargo_bin("tepra")
        .unwrap()
        .arg("version")
        .assert()
        .success();
}

#[test]
#[cfg_attr(miri, ignore)]
fn version_subcommand_prints_version() {
    Command::cargo_bin("tepra")
        .unwrap()
        .arg("version")
        .assert()
        .success()
        .stdout(predicates::str::contains(env!("CARGO_PKG_VERSION")));
}

// ---------------------------------------------------------------------------
// serve subcommand — defaults
// ---------------------------------------------------------------------------

#[test]
fn serve_without_template_dir_parses_ok() {
    let result = Cli::try_parse_from(["tepra", "serve"]);
    let cli = result.expect("parse must succeed without --template-dir");
    // WHY-NOT: unwrap — Commands has multiple variants; panic gives a clearer message in tests.
    #[allow(clippy::panic)]
    let Commands::Serve(args) = cli.command else {
        panic!("expected Serve subcommand");
    };
    assert!(
        args.template_dir.is_none(),
        "template_dir must default to None"
    );
}

#[test]
fn serve_accepts_config_option() {
    let result = Cli::try_parse_from(["tepra", "serve", "--config", "./tepra.toml"]);
    let cli = result.expect("parse must accept --config");
    // WHY-NOT: unwrap — Commands has multiple variants; panic gives a clearer message in tests.
    #[allow(clippy::panic)]
    let Commands::Serve(args) = cli.command else {
        panic!("expected Serve subcommand");
    };
    assert!(
        args.config.is_some(),
        "--config must be captured in args.config"
    );
}

#[test]
#[cfg_attr(miri, ignore)]
fn serve_with_template_dir_exits_nonzero_without_server_infra() {
    // Providing --template-dir is accepted by clap; the binary will fail later
    // because there is no server infra yet. We only check that it does NOT
    // fail with a clap parse error (exit code 2 is clap; anything else is OK).
    let output = Command::cargo_bin("tepra")
        .unwrap()
        .args(["serve", "--template-dir", "/tmp"])
        .timeout(std::time::Duration::from_secs(2))
        .output()
        .unwrap();
    // clap parse error exits with code 2 — must NOT happen.
    assert_ne!(output.status.code(), Some(2), "clap parse error");
}

// ---------------------------------------------------------------------------
// serve subcommand — bind option
// ---------------------------------------------------------------------------

#[test]
#[cfg_attr(miri, ignore)]
fn serve_accepts_bind_option() {
    let output = Command::cargo_bin("tepra")
        .unwrap()
        .args([
            "serve",
            "--template-dir",
            "/tmp",
            "--bind",
            "127.0.0.1:9999",
        ])
        .timeout(std::time::Duration::from_secs(2))
        .output()
        .unwrap();
    assert_ne!(output.status.code(), Some(2), "clap parse error on --bind");
}

// ---------------------------------------------------------------------------
// serve subcommand — creator-base option
// ---------------------------------------------------------------------------

#[test]
#[cfg_attr(miri, ignore)]
fn serve_accepts_creator_base_option() {
    let output = Command::cargo_bin("tepra")
        .unwrap()
        .args([
            "serve",
            "--template-dir",
            "/tmp",
            "--creator-base",
            "http://localhost:29108",
        ])
        .timeout(std::time::Duration::from_secs(2))
        .output()
        .unwrap();
    assert_ne!(
        output.status.code(),
        Some(2),
        "clap parse error on --creator-base"
    );
}

// ---------------------------------------------------------------------------
// config init subcommand
// ---------------------------------------------------------------------------

#[test]
fn config_init_parses_with_no_args() {
    let result = Cli::try_parse_from(["tepra", "config", "init"]);
    let cli = result.expect("parse must accept `config init`");
    #[allow(clippy::panic)]
    let Commands::Config(args) = cli.command else {
        panic!("expected Config subcommand");
    };
    let tepra_web::cli::ConfigAction::Init(init) = args.action;
    assert_eq!(init.path, std::path::PathBuf::from("tepra.toml"));
    assert!(!init.force);
}

#[test]
fn config_init_accepts_path_and_force() {
    let result = Cli::try_parse_from([
        "tepra",
        "config",
        "init",
        "--path",
        "/tmp/x.toml",
        "--force",
    ]);
    let cli = result.expect("parse must accept --path and --force");
    #[allow(clippy::panic)]
    let Commands::Config(args) = cli.command else {
        panic!("expected Config subcommand");
    };
    let tepra_web::cli::ConfigAction::Init(init) = args.action;
    assert_eq!(init.path, std::path::PathBuf::from("/tmp/x.toml"));
    assert!(init.force);
}

#[test]
#[cfg_attr(miri, ignore)]
fn config_init_writes_file_and_exits_success() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("tepra.toml");
    Command::cargo_bin("tepra")
        .unwrap()
        .args(["config", "init", "--path"])
        .arg(&path)
        .assert()
        .success();
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("template_dir"));
    assert!(content.contains("bind"));
    assert!(content.contains("creator_base"));
}

// ---------------------------------------------------------------------------
// top-level --help
// ---------------------------------------------------------------------------

#[test]
#[cfg_attr(miri, ignore)]
fn help_flag_exits_success() {
    Command::cargo_bin("tepra")
        .unwrap()
        .arg("--help")
        .assert()
        .success();
}

#[test]
#[cfg_attr(miri, ignore)]
fn help_output_contains_serve() {
    Command::cargo_bin("tepra")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("serve"));
}

#[test]
#[cfg_attr(miri, ignore)]
fn help_output_contains_version() {
    Command::cargo_bin("tepra")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("version"));
}

//! RED: clap CLI parse tests for tepra-api binary.
//!
//! These tests verify the clap derive CLI structure compiles and parses
//! subcommands / options correctly. They will fail until T18b implements the
//! actual CLI types in `tepra_web::cli`.

#![allow(missing_docs)]

use assert_cmd::Command;

// ---------------------------------------------------------------------------
// version subcommand
// ---------------------------------------------------------------------------

#[test]
#[cfg_attr(miri, ignore)]
fn version_subcommand_exits_success() {
    Command::cargo_bin("tepra-api")
        .unwrap()
        .arg("version")
        .assert()
        .success();
}

#[test]
#[cfg_attr(miri, ignore)]
fn version_subcommand_prints_version() {
    Command::cargo_bin("tepra-api")
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
#[cfg_attr(miri, ignore)]
fn serve_requires_template_dir() {
    // --template-dir is required; omitting it must fail.
    Command::cargo_bin("tepra-api")
        .unwrap()
        .args(["serve"])
        .assert()
        .failure();
}

#[test]
#[cfg_attr(miri, ignore)]
fn serve_with_template_dir_exits_nonzero_without_server_infra() {
    // Providing --template-dir is accepted by clap; the binary will fail later
    // because there is no server infra yet. We only check that it does NOT
    // fail with a clap parse error (exit code 2 is clap; anything else is OK).
    let output = Command::cargo_bin("tepra-api")
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
    let output = Command::cargo_bin("tepra-api")
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
    let output = Command::cargo_bin("tepra-api")
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
// top-level --help
// ---------------------------------------------------------------------------

#[test]
#[cfg_attr(miri, ignore)]
fn help_flag_exits_success() {
    Command::cargo_bin("tepra-api")
        .unwrap()
        .arg("--help")
        .assert()
        .success();
}

#[test]
#[cfg_attr(miri, ignore)]
fn help_output_contains_serve() {
    Command::cargo_bin("tepra-api")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("serve"));
}

#[test]
#[cfg_attr(miri, ignore)]
fn help_output_contains_version() {
    Command::cargo_bin("tepra-api")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("version"));
}

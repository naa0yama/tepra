#![allow(missing_docs)]

use assert_cmd::Command;

#[test]
#[cfg_attr(miri, ignore)]
fn bin_runs_with_help_flag() {
    Command::cargo_bin("tepra-api")
        .unwrap()
        .arg("--help")
        .assert()
        .success();
}

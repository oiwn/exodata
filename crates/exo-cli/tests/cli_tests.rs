use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn test_binary_runs_with_help() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("exodata"));

    cmd.arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Usage"));
}

#[test]
fn test_binary_runs_without_args() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("exodata"));

    // Should fail with error message when no command provided
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Usage"));
}

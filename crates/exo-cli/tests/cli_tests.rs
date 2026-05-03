use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::fs;
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

#[test]
fn test_help_shows_public_dev_group() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("exodata"));

    cmd.arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("dev"))
        .stdout(predicate::str::contains("query"))
        .stdout(predicate::str::contains("rows"))
        .stdout(predicate::str::contains("schema"));
}

#[test]
fn test_dev_help_shows_data_commands() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("exodata"));

    cmd.args(["dev", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("view-fields"))
        .stdout(predicate::str::contains("view-metadata"))
        .stdout(predicate::str::contains("convert-raw-files"))
        .stdout(predicate::str::contains("sql"));
}

#[test]
fn test_old_top_level_dev_command_fails() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("exodata"));

    cmd.arg("view-metadata");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized subcommand"));
}

#[test]
fn test_skill_install_help_shows_scopes() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("exodata"));

    cmd.args(["skill", "install", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("local"))
        .stdout(predicate::str::contains("global"));
}

#[test]
fn test_skill_install_local_writes_agents_skill() {
    let dir = std::env::temp_dir().join(format!(
        "exodata-cli-skill-install-test-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("exodata"));
    cmd.current_dir(&dir).args(["skill", "install", "local"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Installed skill:"));

    let skill_path = dir.join(".agents/skills/exodata/SKILL.md");
    let content = fs::read_to_string(&skill_path).unwrap();
    assert!(content.contains("name: exodata"));
    assert!(content.contains("installed-by: exodata"));

    fs::remove_dir_all(&dir).unwrap();
}

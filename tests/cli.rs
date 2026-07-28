//! Integration: run the built `serval` binary and assert non-launch commands.

use std::process::Command;

fn serval() -> Command {
    Command::new(env!("CARGO_BIN_EXE_serval"))
}

#[test]
fn status_runs_without_token() {
    let out = serval().arg("status").output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(s.contains("ServalAI"));
    assert!(s.contains("token"));
}

#[test]
fn code_without_token_errors_cleanly() {
    let out = serval().arg("code").output().unwrap();
    assert!(!out.status.success());
    let err = String::from_utf8(out.stderr).unwrap();
    assert!(
        err.contains("haven't authenticated")
            || err.contains("failed to exec")
            || err.contains("No such file")
    );
}

#[test]
fn version_subcommand_shows_version() {
    let out = serval().arg("version").output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(s.contains("serval"));
}

#[test]
fn help_subcommand_shows_help() {
    let out = serval().arg("help").output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(s.contains("Usage"));
}

#[test]
fn unknown_command_prints_error() {
    let out = serval().arg("foobar").output().unwrap();
    assert!(!out.status.success());
    let err = String::from_utf8(out.stderr).unwrap();
    assert!(err.contains("not a serval command"));
}

#[test]
fn logout_succeeds_without_config() {
    let out = serval().arg("logout").output().unwrap();
    assert!(out.status.success());
}

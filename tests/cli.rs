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
    assert!(s.contains("serval"));
    assert!(s.contains("token"));
}

#[test]
fn code_without_token_errors_cleanly() {
    let out = serval().arg("code").output().unwrap();
    assert!(!out.status.success());
    let err = String::from_utf8(out.stderr).unwrap();
    assert!(err.contains("no token"));
}

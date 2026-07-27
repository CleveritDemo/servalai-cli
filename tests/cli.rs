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
    // The binary runs against the real config file, so we can't know if a token
    // is present. Instead, assert that `code` produces non-zero exit code and a
    // meaningful message (auth or exec failure).
    let out = serval().arg("code").output().unwrap();
    assert!(!out.status.success());
    let err = String::from_utf8(out.stderr).unwrap();
    assert!(
        err.contains("haven't authenticated")
            || err.contains("failed to exec")
            || err.contains("No such file")
    );
}

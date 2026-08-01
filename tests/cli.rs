//! Integration: run the built `serval` binary and assert non-launch commands.

use std::process::Command;

fn serval() -> Command {
    Command::new(env!("CARGO_BIN_EXE_serval"))
}

/// Runs `serval` against an isolated, empty XDG config dir with the OS
/// keychain explicitly disabled. Without this, a test that expects "no
/// token configured" behavior would instead read whatever's in the
/// developer's real `~/.config/serval/config.toml`, and — if that file
/// doesn't already have a token cached with `use_keychain = false` — would
/// hit the real OS keychain, which on macOS prompts for permission on every
/// freshly rebuilt (unsigned) test binary. Returns the `TempDir` alongside
/// the `Command` so it isn't dropped (and cleaned up) before the test runs.
fn serval_isolated() -> (Command, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let serval_cfg_dir = dir.path().join("cfg").join("serval");
    std::fs::create_dir_all(&serval_cfg_dir).unwrap();
    std::fs::write(serval_cfg_dir.join("config.toml"), "use_keychain = false\n").unwrap();
    let mut cmd = serval();
    cmd.env("XDG_CONFIG_HOME", dir.path().join("cfg"));
    (cmd, dir)
}

#[test]
fn status_runs_without_token() {
    let (mut cmd, _dir) = serval_isolated();
    let out = cmd.arg("status").output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    assert!(s.contains("ServalAI"));
    assert!(s.contains("token"));
}

#[test]
fn code_without_token_errors_cleanly() {
    let (mut cmd, _dir) = serval_isolated();
    let out = cmd.arg("code").output().unwrap();
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
    let (mut cmd, _dir) = serval_isolated();
    let out = cmd.arg("logout").output().unwrap();
    assert!(out.status.success());
}

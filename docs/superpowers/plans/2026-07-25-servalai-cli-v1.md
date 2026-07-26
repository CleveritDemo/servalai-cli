# ServalAI CLI (`serval`) v1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A single-binary Rust CLI, `serval`, that bundles a pinned opencode binary plus a curated ServalAI config/agent/skill loadout, and launches opencode fully configured after one token paste — no config editing, no env vars.

**Architecture:** `serval` stores a token, resolves the ServalAI provider config (from the Worker, falling back to an embedded default), injects it into opencode via `OPENCODE_CONFIG_DIR` + `OPENCODE_CONFIG_CONTENT` environment variables, and `exec`s the bundled opencode binary sitting beside it in the install dir. Self-update swaps a versioned install dir and repoints a `current` symlink (atomic). The developer's own opencode config is never touched.

**Tech Stack:** Rust (edition 2021), `clap` v4 (derive), `ureq` v2 (rustls, no C deps), `serde`/`serde_json`, `toml`, `dirs` v6, `tar` v0.4, `flate2` v1, `cargo-zigbuild` for cross-compile.

## Global Constraints

- Rust stable **≥ 1.85**, edition **2021**.
- HTTP client is **`ureq` v2 with `default-features=false, features=["tls","json"]`** (rustls, pure-Rust TLS — **no C dependencies**, so `cargo-zigbuild` cross-compiles cleanly). Do not add `reqwest`/`openssl`.
- Release targets (exactly four): `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`, `x86_64-apple-darwin`, `aarch64-apple-darwin`. No native Windows target (WSL2 uses the Linux x64 build).
- Binary/command name is **`serval`**; crate/package name **`servalai-cli`**.
- **Never print or log the token.** Any user-facing display masks it to the last 4 chars.
- Progress/diagnostics → **stderr**; final status line → **stdout**.
- Injection is **non-invasive**: never write into `~/.config/opencode/`. Config reaches opencode only through `OPENCODE_CONFIG_DIR` and `OPENCODE_CONFIG_CONTENT`.
- Constants (define once in `src/constants.rs`, reused everywhere):
  - `DEFAULT_WORKER_URL = "https://ai-cf-gateway-controller.groowcity-wiki.workers.dev"`
  - `PROVIDER_KEY = "cf-gateway-clever"`
  - `TOKEN_ENV = "CF_CLEVER_DEV_TOKEN"`
  - `RELEASES_LATEST_API = "https://api.github.com/repos/CleveritDemo/servalai-cli/releases/latest"`

---

## File structure

```
servalai-cli/
├── Cargo.toml
├── .gitignore
├── install.sh                       # curl | sh installer
├── .github/workflows/ci.yml         # fmt + clippy + test on every push
├── .github/workflows/release.yml    # tag-triggered cross-build + GitHub Release
├── assets/default-bundle/           # embedded fallback config (shipped in every bundle)
│   ├── opencode.jsonc               # ServalAI provider template (no token)
│   └── AGENTS.md
└── src/
    ├── main.rs                      # clap entrypoint, dispatch, exit-code handling
    ├── constants.rs                 # the Global Constants values
    ├── config.rs                    # Config: load/store config.toml (0600), token mask
    ├── paths.rs                     # install-dir + data-dir + config-dir resolution
    ├── client.rs                    # Http trait + GET /cli/config, with default fallback
    ├── launch.rs                    # build env, locate opencode, exec (Launcher trait)
    ├── update.rs                    # target detection, release lookup, download, atomic swap
    └── commands.rs                  # one fn per subcommand, wiring the modules
```

Each `src/*.rs` module has one responsibility and is unit-tested with `#[cfg(test)]` blocks in the same file (idiomatic Rust). Integration tests for the command surface live in `tests/`.

---

## Task 1: Repo scaffold, clap skeleton, CI

**Files:**
- Create: `Cargo.toml`, `.gitignore`, `src/main.rs`, `src/constants.rs`, `.github/workflows/ci.yml`

**Interfaces:**
- Produces: `constants` module with the four `pub const` strings; a `clap`-parsed `Cli` with subcommands `Auth`, `Sync`, `Status`, `Logout`, `Update`, `Code` and a default (no subcommand) path.

- [ ] **Step 1: Create `Cargo.toml`**

```toml
[package]
name = "servalai-cli"
version = "0.1.0"
edition = "2021"
rust-version = "1.85"
description = "ServalAI CLI — one-install, token-only access to company models via opencode"
license = "MIT"

[[bin]]
name = "serval"
path = "src/main.rs"

[dependencies]
clap = { version = "4", features = ["derive"] }
dirs = "6"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "1"
flate2 = "1"
tar = "0.4"
# ureq v2 with rustls (pure-Rust TLS, no C deps) so cargo-zigbuild cross-compiles cleanly.
ureq = { version = "2", default-features = false, features = ["tls", "json"] }

[dev-dependencies]
tempfile = "3"

[profile.release]
strip = true
lto = true
```

- [ ] **Step 2: Create `.gitignore`**

```
/target
*.tmp
```

- [ ] **Step 3: Create `src/constants.rs`**

```rust
//! Project-wide constants. Defined once, reused everywhere.

pub const DEFAULT_WORKER_URL: &str =
    "https://ai-cf-gateway-controller.groowcity-wiki.workers.dev";
pub const PROVIDER_KEY: &str = "cf-gateway-clever";
pub const TOKEN_ENV: &str = "CF_CLEVER_DEV_TOKEN";
pub const RELEASES_LATEST_API: &str =
    "https://api.github.com/repos/CleveritDemo/servalai-cli/releases/latest";
```

- [ ] **Step 4: Write the failing test — CLI parses subcommands**

Add to `src/main.rs`:

```rust
mod constants;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "serval", version, about = "ServalAI CLI")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Store your ServalAI token.
    Auth {
        #[arg(long)]
        token: Option<String>,
    },
    /// Refresh provider config from the Worker.
    Sync,
    /// Show version, pinned opencode, and resolved identity.
    Status,
    /// Clear the stored token.
    Logout,
    /// Self-update to the latest release.
    Update,
    /// Launch opencode preconfigured (default action).
    Code,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        _ => {} // wired in later tasks
    }
    let _ = cli; // silence unused until dispatch lands
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn parses_auth_with_token_flag() {
        let cli = Cli::try_parse_from(["serval", "auth", "--token", "aig_x"]).unwrap();
        assert!(matches!(cli.command, Some(Command::Auth { token: Some(_) })));
    }

    #[test]
    fn no_subcommand_is_none() {
        let cli = Cli::try_parse_from(["serval"]).unwrap();
        assert!(cli.command.is_none());
    }
}
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test`
Expected: 2 tests pass; `cargo build` succeeds; `cargo run -- --version` prints `serval 0.1.0`.

- [ ] **Step 6: Create `.github/workflows/ci.yml`**

```yaml
name: CI
on:
  push:
    branches: ["**"]
  pull_request:
env:
  CARGO_TERM_COLOR: always
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: cargo-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: cargo-
      - run: cargo fmt --check
      - run: cargo clippy -- -D warnings
      - run: cargo test
```

- [ ] **Step 7: Verify fmt + clippy clean, then commit**

Run: `cargo fmt && cargo clippy -- -D warnings && cargo test`
Expected: all green.

```bash
git add Cargo.toml Cargo.lock .gitignore src/main.rs src/constants.rs .github/workflows/ci.yml
git commit -m "feat: scaffold servalai-cli (clap skeleton, constants, CI)"
```

---

## Task 2: `config` module — token storage

**Files:**
- Create: `src/config.rs`, and add `mod config;` to `src/main.rs`
- Test: inline `#[cfg(test)]` in `src/config.rs`

**Interfaces:**
- Produces:
  - `pub struct Config { pub token: Option<String>, pub worker_url: String, pub cached_email: Option<String>, pub cached_provider: Option<serde_json::Value> }`
  - `impl Config`: `pub fn load(path: &Path) -> Config` (missing file → defaults, `worker_url = DEFAULT_WORKER_URL`), `pub fn save(&self, path: &Path) -> Result<(), String>` (writes TOML, chmod `0600`), `pub fn masked_token(&self) -> String` (`"—"` if none, else `"••••" + last4`).

- [ ] **Step 1: Write the failing tests**

Create `src/config.rs`:

```rust
//! Persistent CLI config (`~/.config/serval/config.toml`). Holds the token
//! (0600) and the last provider config fetched from the Worker.

use crate::constants::DEFAULT_WORKER_URL;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub token: Option<String>,
    #[serde(default = "default_worker_url")]
    pub worker_url: String,
    #[serde(default)]
    pub cached_email: Option<String>,
    #[serde(default)]
    pub cached_provider: Option<serde_json::Value>,
}

fn default_worker_url() -> String {
    DEFAULT_WORKER_URL.to_string()
}

impl Default for Config {
    fn default() -> Self {
        Config {
            token: None,
            worker_url: default_worker_url(),
            cached_email: None,
            cached_provider: None,
        }
    }
}

impl Config {
    pub fn load(path: &Path) -> Config {
        match std::fs::read_to_string(path) {
            Ok(s) => toml::from_str(&s).unwrap_or_default(),
            Err(_) => Config::default(),
        }
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("mkdir {parent:?}: {e}"))?;
        }
        let body = toml::to_string_pretty(self).map_err(|e| format!("serialize config: {e}"))?;
        std::fs::write(path, body).map_err(|e| format!("write {path:?}: {e}"))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
                .map_err(|e| format!("chmod 0600 {path:?}: {e}"))?;
        }
        Ok(())
    }

    pub fn masked_token(&self) -> String {
        match &self.token {
            None => "—".to_string(),
            Some(t) if t.len() <= 4 => "••••".to_string(),
            Some(t) => format!("••••{}", &t[t.len() - 4..]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn load_missing_returns_defaults() {
        let dir = tempdir().unwrap();
        let cfg = Config::load(&dir.path().join("nope.toml"));
        assert_eq!(cfg.worker_url, DEFAULT_WORKER_URL);
        assert!(cfg.token.is_none());
    }

    #[test]
    fn save_then_load_roundtrips_token() {
        let dir = tempdir().unwrap();
        let p = dir.path().join("config.toml");
        let mut cfg = Config::default();
        cfg.token = Some("aig_secret1234".to_string());
        cfg.save(&p).unwrap();
        let back = Config::load(&p);
        assert_eq!(back.token.as_deref(), Some("aig_secret1234"));
    }

    #[cfg(unix)]
    #[test]
    fn save_sets_0600() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempdir().unwrap();
        let p = dir.path().join("config.toml");
        Config::default().save(&p).unwrap();
        let mode = std::fs::metadata(&p).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    #[test]
    fn masked_token_shows_last4_only() {
        let mut cfg = Config::default();
        cfg.token = Some("aig_abcd1234".to_string());
        assert_eq!(cfg.masked_token(), "••••1234");
        assert_eq!(Config::default().masked_token(), "—");
    }
}
```

Add `mod config;` under `mod constants;` in `src/main.rs`.

- [ ] **Step 2: Run tests to verify they fail then pass**

Run: `cargo test config::`
Expected: compiles and all four `config` tests pass.

- [ ] **Step 3: Commit**

```bash
git add src/config.rs src/main.rs
git commit -m "feat(config): token/config storage with 0600 perms and masking"
```

---

## Task 3: `paths` module — install/data/config locations

**Files:**
- Create: `src/paths.rs`, add `mod paths;` to `src/main.rs`
- Test: inline `#[cfg(test)]`

**Interfaces:**
- Consumes: nothing.
- Produces (all `pub fn`):
  - `install_root() -> PathBuf` — dir holding `serval` + `opencode` + `bundle/`. Resolution order: `$SERVAL_INSTALL_ROOT` if set, else `current_exe()` canonicalized, its parent.
  - `opencode_bin() -> PathBuf` = `install_root().join("opencode")`
  - `bundle_dir() -> PathBuf` = `install_root().join("bundle")`
  - `config_file() -> PathBuf` = `dirs::config_dir()/serval/config.toml`
  - `data_dir() -> PathBuf` = `dirs::data_dir()/serval`
  - `versions_dir() -> PathBuf` = `data_dir()/versions`
  - `current_link() -> PathBuf` = `data_dir()/current`

- [ ] **Step 1: Write the module + failing tests**

Create `src/paths.rs`:

```rust
//! Filesystem locations. Runtime asset paths (opencode, bundle) resolve relative
//! to the running binary so a symlinked `serval` still finds its siblings.

use std::path::PathBuf;

pub fn install_root() -> PathBuf {
    if let Ok(root) = std::env::var("SERVAL_INSTALL_ROOT") {
        return PathBuf::from(root);
    }
    let exe = std::env::current_exe().expect("cannot locate current exe");
    let exe = std::fs::canonicalize(&exe).unwrap_or(exe);
    exe.parent().expect("exe has no parent").to_path_buf()
}

pub fn opencode_bin() -> PathBuf {
    install_root().join("opencode")
}

pub fn bundle_dir() -> PathBuf {
    install_root().join("bundle")
}

pub fn config_file() -> PathBuf {
    dirs::config_dir()
        .expect("no config dir")
        .join("serval")
        .join("config.toml")
}

pub fn data_dir() -> PathBuf {
    dirs::data_dir().expect("no data dir").join("serval")
}

pub fn versions_dir() -> PathBuf {
    data_dir().join("versions")
}

pub fn current_link() -> PathBuf {
    data_dir().join("current")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_root_honors_env_override() {
        std::env::set_var("SERVAL_INSTALL_ROOT", "/tmp/serval-test-root");
        assert_eq!(install_root(), PathBuf::from("/tmp/serval-test-root"));
        assert_eq!(opencode_bin(), PathBuf::from("/tmp/serval-test-root/opencode"));
        assert_eq!(bundle_dir(), PathBuf::from("/tmp/serval-test-root/bundle"));
        std::env::remove_var("SERVAL_INSTALL_ROOT");
    }

    #[test]
    fn config_and_data_paths_end_in_serval() {
        assert!(config_file().ends_with("serval/config.toml"));
        assert!(data_dir().ends_with("serval"));
        assert!(current_link().ends_with("serval/current"));
    }
}
```

Add `mod paths;` to `src/main.rs`.

- [ ] **Step 2: Run tests**

Run: `cargo test paths::`
Expected: both tests pass.

- [ ] **Step 3: Commit**

```bash
git add src/paths.rs src/main.rs
git commit -m "feat(paths): install/data/config path resolution with env override"
```

---

## Task 4: `client` module — fetch provider config from the Worker

**Files:**
- Create: `src/client.rs`, add `mod client;` to `src/main.rs`
- Test: inline `#[cfg(test)]`

**Interfaces:**
- Consumes: `constants::PROVIDER_KEY`.
- Produces:
  - `pub struct FetchedConfig { pub email: String, pub models: Vec<String>, pub provider: serde_json::Value }` — `provider` is the opencode provider object for `PROVIDER_KEY` (npm/name/models, **no** apiKey/baseURL).
  - `pub trait Http { fn get_json(&self, url: &str, bearer: &str) -> Result<serde_json::Value, String>; }`
  - `pub struct UreqHttp;` implementing `Http` via `ureq`.
  - `pub fn fetch_config(http: &dyn Http, worker_url: &str, token: &str) -> Result<FetchedConfig, String>` — GETs `{worker_url}/cli/config`, parses `{ email, models, provider }`.

- [ ] **Step 1: Write the module + failing tests**

Create `src/client.rs`:

```rust
//! Talks to the Worker's read-only `/cli/config` route to get the user's
//! ServalAI provider config. Transport is behind the `Http` trait so tests
//! inject a fake and no network is needed.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct FetchedConfig {
    pub email: String,
    #[serde(default)]
    pub models: Vec<String>,
    pub provider: serde_json::Value,
}

pub trait Http {
    fn get_json(&self, url: &str, bearer: &str) -> Result<serde_json::Value, String>;
}

pub struct UreqHttp;

impl Http for UreqHttp {
    fn get_json(&self, url: &str, bearer: &str) -> Result<serde_json::Value, String> {
        let resp = ureq::get(url)
            .set("Authorization", &format!("Bearer {bearer}"))
            .set("User-Agent", &format!("serval/{}", env!("CARGO_PKG_VERSION")))
            .call()
            .map_err(|e| format!("request to {url} failed: {e}"))?;
        resp.into_json::<serde_json::Value>()
            .map_err(|e| format!("invalid JSON from {url}: {e}"))
    }
}

pub fn fetch_config(
    http: &dyn Http,
    worker_url: &str,
    token: &str,
) -> Result<FetchedConfig, String> {
    let url = format!("{}/cli/config", worker_url.trim_end_matches('/'));
    let value = http.get_json(&url, token)?;
    serde_json::from_value::<FetchedConfig>(value)
        .map_err(|e| format!("unexpected /cli/config shape: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeHttp {
        body: serde_json::Value,
    }
    impl Http for FakeHttp {
        fn get_json(&self, url: &str, bearer: &str) -> Result<serde_json::Value, String> {
            assert!(url.ends_with("/cli/config"));
            assert_eq!(bearer, "aig_token");
            Ok(self.body.clone())
        }
    }

    #[test]
    fn parses_well_formed_config() {
        let http = FakeHttp {
            body: serde_json::json!({
                "email": "dev@cleveritgroup.com",
                "models": ["dynamic/balanced", "dynamic/light"],
                "provider": { "npm": "@ai-sdk/openai-compatible", "name": "ServalAI" }
            }),
        };
        let cfg = fetch_config(&http, "https://w.example.dev/", "aig_token").unwrap();
        assert_eq!(cfg.email, "dev@cleveritgroup.com");
        assert_eq!(cfg.models.len(), 2);
        assert_eq!(cfg.provider["name"], "ServalAI");
    }

    #[test]
    fn errors_on_bad_shape() {
        let http = FakeHttp {
            body: serde_json::json!({ "nope": true }),
        };
        let err = fetch_config(&http, "https://w.example.dev", "aig_token").unwrap_err();
        assert!(err.contains("unexpected /cli/config shape"));
    }
}
```

Add `mod client;` to `src/main.rs`.

- [ ] **Step 2: Run tests**

Run: `cargo test client::`
Expected: both tests pass.

- [ ] **Step 3: Commit**

```bash
git add src/client.rs src/main.rs
git commit -m "feat(client): fetch provider config from Worker /cli/config (injectable Http)"
```

---

## Task 5: `launch` module — build env and exec opencode

**Files:**
- Create: `src/launch.rs`, add `mod launch;` to `src/main.rs`
- Test: inline `#[cfg(test)]`

**Interfaces:**
- Consumes: `constants::{PROVIDER_KEY, TOKEN_ENV}`, `client::FetchedConfig`.
- Produces:
  - `pub fn build_env(provider: &serde_json::Value, worker_url: &str, token: &str, bundle_dir: &Path) -> Vec<(String, String)>` — returns the env pairs to inject: `OPENCODE_CONFIG_DIR`, `OPENCODE_CONFIG_CONTENT` (JSON string wrapping the provider with `options.baseURL`+`options.apiKey`), and `TOKEN_ENV`.
  - `pub trait Launcher { fn exec(&self, program: &Path, args: &[String], env: &[(String, String)]) -> Result<(), String>; }`
  - `pub struct ExecLauncher;` — real `exec` (Unix, replaces process).

- [ ] **Step 1: Write the module + failing tests**

Create `src/launch.rs`:

```rust
//! Builds the environment that injects ServalAI into opencode, then execs the
//! bundled opencode binary. The env-building logic is pure and unit-tested; the
//! exec itself is behind a trait so tests never spawn a real process.

use crate::constants::{PROVIDER_KEY, TOKEN_ENV};
use std::path::Path;

pub fn build_env(
    provider: &serde_json::Value,
    worker_url: &str,
    token: &str,
    bundle_dir: &Path,
) -> Vec<(String, String)> {
    // Clone the provider block and stamp in the live options (baseURL + apiKey).
    let mut provider = provider.clone();
    provider["options"] = serde_json::json!({
        "baseURL": worker_url.trim_end_matches('/'),
        "apiKey": token,
    });
    let content = serde_json::json!({
        "provider": { PROVIDER_KEY: provider }
    });
    vec![
        (
            "OPENCODE_CONFIG_DIR".to_string(),
            bundle_dir.to_string_lossy().to_string(),
        ),
        (
            "OPENCODE_CONFIG_CONTENT".to_string(),
            content.to_string(),
        ),
        (TOKEN_ENV.to_string(), token.to_string()),
    ]
}

pub trait Launcher {
    fn exec(&self, program: &Path, args: &[String], env: &[(String, String)])
        -> Result<(), String>;
}

pub struct ExecLauncher;

impl Launcher for ExecLauncher {
    #[cfg(unix)]
    fn exec(
        &self,
        program: &Path,
        args: &[String],
        env: &[(String, String)],
    ) -> Result<(), String> {
        use std::os::unix::process::CommandExt;
        let mut cmd = std::process::Command::new(program);
        cmd.args(args);
        for (k, v) in env {
            cmd.env(k, v);
        }
        // exec() only returns if it FAILED.
        Err(format!("failed to exec {program:?}: {}", cmd.exec()))
    }

    #[cfg(not(unix))]
    fn exec(
        &self,
        _program: &Path,
        _args: &[String],
        _env: &[(String, String)],
    ) -> Result<(), String> {
        Err("serval only supports Unix (Linux/macOS/WSL2)".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn build_env_injects_provider_dir_content_and_token() {
        let provider = serde_json::json!({ "npm": "@ai-sdk/openai-compatible", "name": "ServalAI" });
        let env = build_env(&provider, "https://w.example.dev/", "aig_tok", &PathBuf::from("/b"));

        let map: std::collections::HashMap<_, _> = env.into_iter().collect();
        assert_eq!(map["OPENCODE_CONFIG_DIR"], "/b");
        assert_eq!(map[TOKEN_ENV], "aig_tok");

        let content: serde_json::Value =
            serde_json::from_str(&map["OPENCODE_CONFIG_CONTENT"]).unwrap();
        let p = &content["provider"][PROVIDER_KEY];
        assert_eq!(p["name"], "ServalAI");
        assert_eq!(p["options"]["baseURL"], "https://w.example.dev");
        assert_eq!(p["options"]["apiKey"], "aig_tok");
    }
}
```

Add `mod launch;` to `src/main.rs`.

- [ ] **Step 2: Run tests**

Run: `cargo test launch::`
Expected: the `build_env` test passes.

- [ ] **Step 3: Commit**

```bash
git add src/launch.rs src/main.rs
git commit -m "feat(launch): build opencode injection env; exec via Launcher trait"
```

---

## Task 6: `update` module — target detection, download, atomic swap

**Files:**
- Create: `src/update.rs`, add `mod update;` to `src/main.rs`
- Test: inline `#[cfg(test)]`

**Interfaces:**
- Consumes: `constants::RELEASES_LATEST_API`, `client::Http`, `paths::{versions_dir, current_link}`.
- Produces:
  - `pub const CURRENT_TARGET: &str` (per-platform via `cfg`).
  - `pub fn asset_name(target: &str) -> String` = `format!("serval-{target}.tar.gz")`.
  - `pub fn needs_update(current_tag: &str, latest_tag: &str) -> bool` (string inequality).
  - `pub fn extract_tar_gz(bytes: &[u8], dest: &Path) -> Result<(), String>`.
  - `pub fn repoint_current(versions: &Path, current: &Path, tag: &str) -> Result<(), String>` — atomically point `current` symlink at `versions/<tag>`.

- [ ] **Step 1: Write the module + failing tests**

Create `src/update.rs`:

```rust
//! Self-update: locate the latest GitHub release, download this platform's bundle,
//! extract it into a versioned dir, and atomically repoint the `current` symlink.

use crate::client::Http;
use crate::constants::RELEASES_LATEST_API;
use flate2::read::GzDecoder;
use std::path::Path;

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub const CURRENT_TARGET: &str = "x86_64-unknown-linux-musl";
#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
pub const CURRENT_TARGET: &str = "aarch64-unknown-linux-musl";
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
pub const CURRENT_TARGET: &str = "x86_64-apple-darwin";
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub const CURRENT_TARGET: &str = "aarch64-apple-darwin";
#[cfg(not(any(
    all(target_os = "linux", target_arch = "x86_64"),
    all(target_os = "linux", target_arch = "aarch64"),
    all(target_os = "macos", target_arch = "x86_64"),
    all(target_os = "macos", target_arch = "aarch64"),
)))]
compile_error!("serval: unsupported target — add a CURRENT_TARGET for this platform");

pub fn asset_name(target: &str) -> String {
    format!("serval-{target}.tar.gz")
}

pub fn needs_update(current_tag: &str, latest_tag: &str) -> bool {
    current_tag != latest_tag
}

pub fn extract_tar_gz(bytes: &[u8], dest: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dest).map_err(|e| format!("mkdir {dest:?}: {e}"))?;
    let gz = GzDecoder::new(bytes);
    let mut ar = tar::Archive::new(gz);
    ar.unpack(dest).map_err(|e| format!("extract to {dest:?}: {e}"))
}

#[cfg(unix)]
pub fn repoint_current(versions: &Path, current: &Path, tag: &str) -> Result<(), String> {
    use std::os::unix::fs::symlink;
    let target = versions.join(tag);
    if !target.exists() {
        return Err(format!("version dir missing: {target:?}"));
    }
    // Write a temp symlink then rename over `current` — rename is atomic.
    let tmp = current.with_extension("tmp");
    let _ = std::fs::remove_file(&tmp);
    symlink(&target, &tmp).map_err(|e| format!("symlink {tmp:?}: {e}"))?;
    std::fs::rename(&tmp, current).map_err(|e| format!("swap {current:?}: {e}"))?;
    Ok(())
}

/// Fetch the latest release tag + this platform's asset download URL.
pub fn latest_release(http: &dyn Http) -> Result<(String, String), String> {
    let release = http.get_json(RELEASES_LATEST_API, "")?;
    let tag = release["tag_name"]
        .as_str()
        .ok_or("release JSON missing tag_name")?
        .to_string();
    let want = asset_name(CURRENT_TARGET);
    let assets = release["assets"].as_array().ok_or("release JSON missing assets")?;
    for a in assets {
        if a["name"].as_str() == Some(want.as_str()) {
            let url = a["browser_download_url"]
                .as_str()
                .ok_or("asset missing browser_download_url")?
                .to_string();
            return Ok((tag, url));
        }
    }
    Err(format!("no asset named {want} in latest release"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn asset_name_matches_release_convention() {
        assert_eq!(asset_name("x86_64-apple-darwin"), "serval-x86_64-apple-darwin.tar.gz");
    }

    #[test]
    fn needs_update_compares_tags() {
        assert!(needs_update("v0.1.0", "v0.2.0"));
        assert!(!needs_update("v0.2.0", "v0.2.0"));
    }

    #[test]
    fn latest_release_finds_matching_asset() {
        struct FakeHttp;
        impl Http for FakeHttp {
            fn get_json(&self, _u: &str, _b: &str) -> Result<serde_json::Value, String> {
                Ok(serde_json::json!({
                    "tag_name": "v9.9.9",
                    "assets": [
                        { "name": asset_name(CURRENT_TARGET),
                          "browser_download_url": "https://example.com/x.tar.gz" }
                    ]
                }))
            }
        }
        let (tag, url) = latest_release(&FakeHttp).unwrap();
        assert_eq!(tag, "v9.9.9");
        assert_eq!(url, "https://example.com/x.tar.gz");
    }

    #[cfg(unix)]
    #[test]
    fn repoint_current_points_symlink_at_version() {
        let dir = tempdir().unwrap();
        let versions = dir.path().join("versions");
        std::fs::create_dir_all(versions.join("v1.0.0")).unwrap();
        let current = dir.path().join("current");
        repoint_current(&versions, &current, "v1.0.0").unwrap();
        assert_eq!(std::fs::read_link(&current).unwrap(), versions.join("v1.0.0"));
        // Repoint again — must overwrite atomically.
        std::fs::create_dir_all(versions.join("v1.1.0")).unwrap();
        repoint_current(&versions, &current, "v1.1.0").unwrap();
        assert_eq!(std::fs::read_link(&current).unwrap(), versions.join("v1.1.0"));
    }

    #[test]
    fn extract_tar_gz_roundtrip() {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::io::Write;
        // Build a tiny .tar.gz containing one file "hello".
        let mut tar_buf = Vec::new();
        {
            let mut b = tar::Builder::new(&mut tar_buf);
            let data = b"hi";
            let mut h = tar::Header::new_gnu();
            h.set_size(data.len() as u64);
            h.set_mode(0o644);
            h.set_cksum();
            b.append_data(&mut h, "hello", &data[..]).unwrap();
            b.finish().unwrap();
        }
        let mut gz = GzEncoder::new(Vec::new(), Compression::default());
        gz.write_all(&tar_buf).unwrap();
        let bytes = gz.finish().unwrap();

        let dir = tempdir().unwrap();
        extract_tar_gz(&bytes, dir.path()).unwrap();
        assert_eq!(std::fs::read_to_string(dir.path().join("hello")).unwrap(), "hi");
    }
}
```

Add `mod update;` to `src/main.rs`.

- [ ] **Step 2: Run tests**

Run: `cargo test update::`
Expected: all five `update` tests pass on Linux/macOS.

- [ ] **Step 3: Commit**

```bash
git add src/update.rs src/main.rs
git commit -m "feat(update): target detection, release lookup, extract, atomic symlink swap"
```

---

## Task 7: embedded default config + client fallback

**Files:**
- Create: `assets/default-bundle/opencode.jsonc`, `assets/default-bundle/AGENTS.md`
- Modify: `src/client.rs` (add `default_provider()` + `resolve_config`)
- Test: inline `#[cfg(test)]` in `src/client.rs`

**Interfaces:**
- Produces:
  - `pub fn default_provider() -> serde_json::Value` — the embedded ServalAI provider block (three tiers with sane default context windows), used when the Worker is unreachable and no cache exists.
  - `pub fn resolve_config(http: &dyn Http, worker_url: &str, token: &str, cached: Option<&serde_json::Value>) -> (serde_json::Value, Option<String>)` — returns `(provider, email)`; tries the Worker, else cache, else `default_provider()`. Never errors (degrade-don't-block).

- [ ] **Step 1: Create the embedded default provider file**

Create `assets/default-bundle/opencode.jsonc` (this is the fallback + what ships in the bundle dir):

```jsonc
{
  "$schema": "https://opencode.ai/config.json",
  "provider": {
    "cf-gateway-clever": {
      "npm": "@ai-sdk/openai-compatible",
      "name": "ServalAI",
      "models": {
        "dynamic/power":    { "name": "ServalAI power",    "tool_call": true, "limit": { "context": 262144, "output": 16384 } },
        "dynamic/balanced": { "name": "ServalAI balanced", "tool_call": true, "limit": { "context": 200000, "output": 16384 } },
        "dynamic/light":    { "name": "ServalAI light",    "tool_call": true, "limit": { "context": 131072, "output": 8192 } }
      }
    }
  }
}
```

Create `assets/default-bundle/AGENTS.md`:

```markdown
# ServalAI

You are running through ServalAI, Cleverit's company-funded model gateway.
Prefer the `dynamic/balanced` tier for everyday work; escalate to `dynamic/power`
for hard tasks and drop to `dynamic/light` for quick/mechanical ones.
```

- [ ] **Step 2: Write the failing tests** (append to `src/client.rs` `#[cfg(test)]`)

Add these functions to `src/client.rs` (above the tests module):

```rust
/// The provider block embedded at compile time — used as a last-resort fallback.
pub fn default_provider() -> serde_json::Value {
    // The embedded file is a superset ({ provider: { cf-gateway-clever: {...} } });
    // extract just the provider object for PROVIDER_KEY.
    const RAW: &str = include_str!("../assets/default-bundle/opencode.jsonc");
    let full: serde_json::Value =
        serde_json::from_str(RAW).expect("embedded default opencode.jsonc must be valid JSON");
    full["provider"][crate::constants::PROVIDER_KEY].clone()
}

/// Resolve the provider config without ever failing: Worker → cache → embedded default.
pub fn resolve_config(
    http: &dyn Http,
    worker_url: &str,
    token: &str,
    cached: Option<&serde_json::Value>,
) -> (serde_json::Value, Option<String>) {
    match fetch_config(http, worker_url, token) {
        Ok(fc) => (fc.provider, Some(fc.email)),
        Err(e) => {
            eprintln!("serval: using {} config ({e})", if cached.is_some() { "cached" } else { "default" });
            match cached {
                Some(v) => (v.clone(), None),
                None => (default_provider(), None),
            }
        }
    }
}
```

Add tests inside the existing `#[cfg(test)] mod tests`:

```rust
    struct FailingHttp;
    impl Http for FailingHttp {
        fn get_json(&self, _u: &str, _b: &str) -> Result<serde_json::Value, String> {
            Err("network down".to_string())
        }
    }

    #[test]
    fn default_provider_has_three_tiers() {
        let p = default_provider();
        assert_eq!(p["name"], "ServalAI");
        assert!(p["models"]["dynamic/balanced"]["limit"]["context"].is_number());
    }

    #[test]
    fn resolve_falls_back_to_cache_then_default() {
        // Worker fails, cache present → use cache.
        let cache = serde_json::json!({ "name": "cached" });
        let (p, email) = resolve_config(&FailingHttp, "https://w.dev", "t", Some(&cache));
        assert_eq!(p["name"], "cached");
        assert!(email.is_none());
        // Worker fails, no cache → embedded default.
        let (p2, _) = resolve_config(&FailingHttp, "https://w.dev", "t", None);
        assert_eq!(p2["name"], "ServalAI");
    }
```

- [ ] **Step 3: Run tests**

Run: `cargo test client::`
Expected: the new tests + the Task 4 tests all pass.

- [ ] **Step 4: Commit**

```bash
git add assets/default-bundle/opencode.jsonc assets/default-bundle/AGENTS.md src/client.rs
git commit -m "feat(client): embedded default provider + degrade-don't-block resolve_config"
```

---

## Task 8: `commands` module — wire the subcommands

**Files:**
- Create: `src/commands.rs`, add `mod commands;` to `src/main.rs` and dispatch in `main()`
- Test: `tests/cli.rs` (integration, using the built binary via env override)

**Interfaces:**
- Consumes: `config::Config`, `paths::*`, `client::{UreqHttp, resolve_config}`, `launch::{build_env, ExecLauncher, Launcher}`, `update::*`.
- Produces (all return `Result<(), String>`): `auth(token: Option<String>)`, `sync()`, `status()`, `logout()`, `update_cmd()`, `code(passthrough: Vec<String>)`.

- [ ] **Step 1: Write `src/commands.rs`**

```rust
//! One function per subcommand. Each loads Config from `paths::config_file()`,
//! does its work, and returns Result<(), String> (main maps Err → exit 1).

use crate::client::{resolve_config, UreqHttp};
use crate::config::Config;
use crate::launch::{build_env, ExecLauncher, Launcher};
use crate::{paths, update};
use std::io::Write;

fn load() -> Config {
    Config::load(&paths::config_file())
}

fn require_token(cfg: &Config) -> Result<String, String> {
    cfg.token
        .clone()
        .ok_or_else(|| "no token — run `serval auth` (get yours from Mi Portal)".to_string())
}

pub fn auth(token: Option<String>) -> Result<(), String> {
    let mut cfg = load();
    let token = match token {
        Some(t) => t,
        None => {
            eprint!("Paste your ServalAI token: ");
            std::io::stderr().flush().ok();
            let mut s = String::new();
            std::io::stdin().read_line(&mut s).map_err(|e| format!("read token: {e}"))?;
            s.trim().to_string()
        }
    };
    if token.is_empty() {
        return Err("empty token".to_string());
    }
    cfg.token = Some(token.clone());
    // Validate + warm the cache against the Worker.
    let (provider, email) = resolve_config(&UreqHttp, &cfg.worker_url, &token, cfg.cached_provider.as_ref());
    cfg.cached_provider = Some(provider);
    cfg.cached_email = email.clone();
    cfg.save(&paths::config_file())?;
    match email {
        Some(e) => println!("Authenticated as {e}."),
        None => println!("Token saved (could not reach the Worker to confirm identity)."),
    }
    Ok(())
}

pub fn sync() -> Result<(), String> {
    let mut cfg = load();
    let token = require_token(&cfg)?;
    let (provider, email) = resolve_config(&UreqHttp, &cfg.worker_url, &token, cfg.cached_provider.as_ref());
    cfg.cached_provider = Some(provider);
    if email.is_some() {
        cfg.cached_email = email;
    }
    cfg.save(&paths::config_file())?;
    println!("Config synced.");
    Ok(())
}

pub fn status() -> Result<(), String> {
    let cfg = load();
    println!("serval        {}", env!("CARGO_PKG_VERSION"));
    println!("opencode      {}", opencode_version());
    println!("worker        {}", cfg.worker_url);
    println!("token         {}", cfg.masked_token());
    println!("identity      {}", cfg.cached_email.as_deref().unwrap_or("—"));
    Ok(())
}

fn opencode_version() -> String {
    std::process::Command::new(paths::opencode_bin())
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "not bundled".to_string())
}

pub fn logout() -> Result<(), String> {
    let mut cfg = load();
    cfg.token = None;
    cfg.save(&paths::config_file())?;
    println!("Logged out.");
    Ok(())
}

pub fn update_cmd() -> Result<(), String> {
    use crate::client::Http;
    eprintln!("Checking for updates...");
    let (tag, url) = update::latest_release(&UreqHttp)?;
    let current = format!("v{}", env!("CARGO_PKG_VERSION"));
    if !update::needs_update(&current, &tag) {
        println!("Already up to date ({current}).");
        return Ok(());
    }
    println!("Update available: {current} -> {tag}");
    let bytes = UreqHttp.get_bytes(&url)?;
    let dest = paths::versions_dir().join(&tag);
    update::extract_tar_gz(&bytes, &dest)?;
    update::repoint_current(&paths::versions_dir(), &paths::current_link(), &tag)?;
    println!("Updated to {tag}. Run `serval` to use it.");
    Ok(())
}

pub fn code(passthrough: Vec<String>) -> Result<(), String> {
    let cfg = load();
    let token = require_token(&cfg)?;
    let (provider, _) = resolve_config(&UreqHttp, &cfg.worker_url, &token, cfg.cached_provider.as_ref());
    let env = build_env(&provider, &cfg.worker_url, &token, &paths::bundle_dir());
    ExecLauncher.exec(&paths::opencode_bin(), &passthrough, &env)
}
```

- [ ] **Step 2: Add a `get_bytes` method to `Http` for downloads**

In `src/client.rs`, extend the `Http` trait and `UreqHttp`:

```rust
pub trait Http {
    fn get_json(&self, url: &str, bearer: &str) -> Result<serde_json::Value, String>;
    fn get_bytes(&self, url: &str) -> Result<Vec<u8>, String>;
}
```

And in `impl Http for UreqHttp`, add:

```rust
    fn get_bytes(&self, url: &str) -> Result<Vec<u8>, String> {
        let resp = ureq::get(url)
            .set("User-Agent", &format!("serval/{}", env!("CARGO_PKG_VERSION")))
            .call()
            .map_err(|e| format!("download {url} failed: {e}"))?;
        let mut buf = Vec::new();
        std::io::Read::read_to_end(&mut resp.into_reader(), &mut buf)
            .map_err(|e| format!("read body from {url}: {e}"))?;
        Ok(buf)
    }
```

Update the test fakes (`FakeHttp`, `FailingHttp` in `client.rs`, and `FakeHttp` in `update.rs`) to add a `get_bytes` impl returning `Ok(vec![])` / `Err(...)` respectively so they still satisfy the trait.

- [ ] **Step 3: Wire dispatch in `src/main.rs`**

Replace the `match cli.command { _ => {} }` block with:

```rust
    let result = match cli.command {
        Some(Command::Auth { token }) => commands::auth(token),
        Some(Command::Sync) => commands::sync(),
        Some(Command::Status) => commands::status(),
        Some(Command::Logout) => commands::logout(),
        Some(Command::Update) => commands::update_cmd(),
        Some(Command::Code) | None => commands::code(vec![]),
    };
    if let Err(e) = result {
        eprintln!("serval: {e}");
        std::process::exit(1);
    }
```

Add `mod commands;` with the other `mod` lines.

- [ ] **Step 4: Write an integration test** — `tests/cli.rs`

```rust
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
```

- [ ] **Step 5: Run everything**

Run: `cargo test && cargo clippy -- -D warnings && cargo fmt --check`
Expected: all unit + integration tests pass; clippy clean.

- [ ] **Step 6: Commit**

```bash
git add src/commands.rs src/client.rs src/main.rs tests/cli.rs
git commit -m "feat(commands): wire auth/sync/status/logout/update/code subcommands"
```

---

## Task 9: `install.sh` + release workflow

**Files:**
- Create: `install.sh`, `.github/workflows/release.yml`

**Interfaces:**
- Consumes: the release assets `serval-<target>.tar.gz` produced by `release.yml`.
- Produces: an installed layout `~/.local/share/serval/versions/<tag>/{serval,opencode,bundle/}` + `~/.local/share/serval/current` symlink + `~/.local/bin/serval` symlink.

- [ ] **Step 1: Create `install.sh`**

```bash
#!/bin/sh
# ServalAI CLI installer. Usage: curl -fsSL <url>/install.sh | sh
set -eu

REPO="CleveritDemo/servalai-cli"
DATA="${XDG_DATA_HOME:-$HOME/.local/share}/serval"
BIN="$HOME/.local/bin"

os="$(uname -s)"; arch="$(uname -m)"
case "$os-$arch" in
  Linux-x86_64)  target="x86_64-unknown-linux-musl" ;;
  Linux-aarch64) target="aarch64-unknown-linux-musl" ;;
  Darwin-x86_64) target="x86_64-apple-darwin" ;;
  Darwin-arm64)  target="aarch64-apple-darwin" ;;
  *) echo "unsupported platform: $os-$arch" >&2; exit 1 ;;
esac

tag="$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | grep -m1 '"tag_name"' | cut -d'"' -f4)"
asset="serval-$target.tar.gz"
url="https://github.com/$REPO/releases/download/$tag/$asset"

echo "Installing serval $tag for $target..." >&2
dest="$DATA/versions/$tag"
mkdir -p "$dest" "$BIN"
curl -fsSL "$url" | tar -xz -C "$dest"

# macOS: clear quarantine so the unsigned binaries run.
if [ "$os" = "Darwin" ]; then
  xattr -dr com.apple.quarantine "$dest" 2>/dev/null || true
fi

ln -sfn "$dest" "$DATA/current"
ln -sfn "$DATA/current/serval" "$BIN/serval"

echo "Installed. Ensure $BIN is on your PATH, then run: serval auth" >&2
```

- [ ] **Step 2: Verify the installer parses** (shellcheck if available)

Run: `sh -n install.sh` (syntax check) and, if installed, `shellcheck install.sh`
Expected: no syntax errors.

- [ ] **Step 3: Create `.github/workflows/release.yml`**

```yaml
name: Release
on:
  push:
    tags: ["v[0-9]+.[0-9]+.[0-9]+"]
permissions:
  contents: write
env:
  CARGO_TERM_COLOR: always
  OPENCODE_VERSION: "1.17.18"   # pinned opencode bundled per release
jobs:
  build:
    name: ${{ matrix.target }}
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          - { target: x86_64-unknown-linux-musl,  os: ubuntu-latest,  ocode: linux-x64 }
          - { target: aarch64-unknown-linux-musl, os: ubuntu-latest,  ocode: linux-arm64 }
          - { target: x86_64-apple-darwin,        os: macos-latest,   ocode: darwin-x64 }
          - { target: aarch64-apple-darwin,       os: macos-latest,   ocode: darwin-arm64 }
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - name: Install cargo-zigbuild (Linux musl)
        if: runner.os == 'Linux'
        run: |
          pip install ziglang
          cargo install cargo-zigbuild
      - name: Build (Linux)
        if: runner.os == 'Linux'
        run: cargo zigbuild --release --target ${{ matrix.target }}
      - name: Build (macOS)
        if: runner.os == 'macOS'
        run: cargo build --release --target ${{ matrix.target }}
      - name: Assemble bundle
        run: |
          stage="stage/serval-${{ matrix.target }}"
          mkdir -p "$stage/bundle"
          cp "target/${{ matrix.target }}/release/serval" "$stage/serval"
          # Fetch the pinned opencode binary for this platform.
          curl -fsSL "https://github.com/anomalyco/opencode/releases/download/v${OPENCODE_VERSION}/opencode-${{ matrix.ocode }}.tar.gz" \
            | tar -xz -C "$stage"   # yields $stage/opencode
          # Seed the bundle config dir from the repo's default-bundle assets.
          cp -R assets/default-bundle/. "$stage/bundle/"
          tar -C stage -czf "serval-${{ matrix.target }}.tar.gz" "serval-${{ matrix.target }}"
      - uses: softprops/action-gh-release@v2
        with:
          files: serval-${{ matrix.target }}.tar.gz
```

> Note: verify the exact opencode release asset naming for `OPENCODE_VERSION` at
> implementation time (`opencode-<ocode>.tar.gz` vs `.zip`); adjust the extract
> line to match. This is the one external contract to confirm before first release.

- [ ] **Step 4: Commit**

```bash
git add install.sh .github/workflows/release.yml
git commit -m "feat: install.sh + tag-triggered cross-build release workflow"
```

---

## Self-review

**Spec coverage** (spec §→task):
- §3 architecture (bundle + env injection): Tasks 3, 5, 9 ✅
- §4 config delivery (Worker `/cli/config`, degrade-to-cache/default): Tasks 4, 7, 8 ✅ (the Worker route itself is the separate server-side plan, per scope note)
- §5 command surface: Task 8 ✅ (all six commands)
- §6 self-update (whole-bundle, atomic): Task 6 + `update_cmd` in Task 8 ✅
- §7 distribution (4 targets, zigbuild, install.sh): Task 9 ✅
- §8 components (config/client/launch/update/bundle/cli): Tasks 2–8 ✅ (`bundle`→`paths`)
- §9 error handling (no token, unreachable, update failure, never-log-token): Tasks 2, 7, 8 ✅
- §11 macOS Gatekeeper: install.sh `xattr` step (Task 9) ✅
- Non-goals (Pi, browser login, telemetry, native Windows, keychain): honored — none implemented ✅

**Placeholder scan:** the only forward-looking note is the opencode release-asset naming confirmation in Task 9 Step 3, which is an explicit external-contract check with a concrete fallback, not a code placeholder. No TODO/TBD in code steps.

**Type consistency:** `Http` trait gains `get_bytes` in Task 8 Step 2, and all three test fakes are updated in the same step — no stale impls. `resolve_config`/`fetch_config`/`FetchedConfig` names match across Tasks 4, 7, 8. `build_env`, `Launcher::exec`, `repoint_current`, `latest_release`, `extract_tar_gz`, `needs_update`, `asset_name` are referenced in Task 8 exactly as defined in Tasks 5–6.

**Deferred to the server-side plan (separate):** Worker `GET /cli/config` route + platform `buildRealKvValue` KV `config` field. The CLI works standalone before these land (Task 7 embedded default), so this plan is independently shippable.

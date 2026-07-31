# Branded Progress Feedback Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give every gateway-bound `serval` command an animated "still working" spinner (fixing the silent-hang UX bug), replace the misleading error-styled fallback message with a visually distinct note, and show a ServalAI/Cleverit/Raven text wordmark once on plain `serval` launch.

**Architecture:** A new dependency-free `src/progress.rs` module owns all terminal presentation (pure text-formatting functions + a background-thread `Spinner`). `src/client.rs::resolve_config` stops printing directly and instead returns a `fallback_note: Option<String>` so its caller decides how to present it. `src/commands.rs` wires `Spinner`/`banner` around the existing network calls in `code`, `auth`, `sync`, `ping`, `models`, `usage`, `doctor`.

**Tech Stack:** Rust (std only — `std::thread`, `std::sync::atomic`, `std::io::IsTerminal`). No new Cargo dependencies.

**Design reference:** `docs/superpowers/specs/2026-07-31-branded-progress-feedback-design.md`

## Global Constraints

- **No new Cargo dependencies.** Do not touch `Cargo.toml` / `Cargo.lock`. (Spec §1 success criteria; matches the repo's existing minimal-dependency stance — see the `ureq` feature-gating comment in `Cargo.toml`.)
- `rust-version = "1.85"` in `Cargo.toml` — `std::io::IsTerminal` is stable since 1.70, safe to use directly.
- `cargo fmt --check` and `cargo clippy -- -D warnings` must both pass (enforced in `.github/workflows/ci.yml:24-25`) — run them before every commit in this plan.
- The spinner renders on **stderr only**. Stdout is reserved for actual command output (`models`, `usage`, `status`, etc.) — never mix progress chrome into stdout.
- Animation and color must be **fully disabled** when stdout or stderr isn't a TTY, or when the `NO_COLOR` env var is set to a non-empty value (critical for CI logs / piped output). This decision logic must be a pure, unit-tested function — no hidden IO in the tested path.
- Banner wordmark text is exactly:
  ```
  ServalAI
  a Cleverit company · powered by Raven
  ```
  Plain text only — no ASCII art (per explicit user decision during brainstorming).
- The fallback note is never prefixed with `"serval:"` and never uses `✗` — it must be visually distinct from the fatal-error format in `main.rs::run()` (`eprintln!("serval: {e}{hint}")`), since a fallback is a **successful degrade**, not a failure.
- Follow the existing repo convention (see `src/client.rs:5`, per `.superpowers/sdd/progress.md`) of adding `#![allow(dead_code)]` to a new module while it's being built across multiple tasks, and removing it once every item is consumed by a caller.
- **Every task must leave the crate compiling and the full test suite passing.** `resolve_config`'s signature change and updating its call sites cannot be split across separate commits (Rust's whole-crate compilation means a signature change and its callers are one atomic unit) — see Task 3, which is deliberately larger for this reason.

---

## File Structure

- **Create `src/progress.rs`** — all terminal presentation logic for this feature:
  - Pure functions: `should_animate`, `banner`, `note_line` (Task 1)
  - `is_interactive()` — thin real-IO wrapper around `should_animate` (Task 1)
  - `Spinner` struct — background-thread animation, `start`/`finish_silent`/`finish_note`, `Drop` cleanup (Task 2)
- **Modify `src/main.rs:1-7`** — register `mod progress;` (Task 1)
- **Modify `src/client.rs`** — `resolve_config` (lines 96-118) stops calling `eprintln!` directly and returns a third tuple element, `fallback_note: Option<String>`; existing tests updated for the new 3-tuple shape (Task 3)
- **Modify `src/commands.rs`** — wire `Spinner`/`banner` into `auth` (22-59), `sync` (61-77), `code` (348-359) in Task 3; `ping` (139-162), `models` (164-187), `usage` (189-209), `doctor` (211-267) in Task 4

**Note on architecture vs. the design doc:** the design doc's §7 phrasing ("this call site is replaced with a call into `Spinner::finish_note`") is implemented here with `client.rs` staying decoupled from `progress.rs` — `resolve_config` returns data, `commands.rs` (the caller) owns presentation. This keeps the module boundary clean (`client.rs` has no reason to know about terminal rendering) without changing the observable behavior the design specifies.

---

### Task 1: `progress.rs` — pure formatting logic + module registration

**Files:**
- Create: `src/progress.rs`
- Modify: `src/main.rs:1-7`
- Test: inline `#[cfg(test)] mod tests` in `src/progress.rs`

**Interfaces:**
- Produces: `pub fn should_animate(stdout_is_tty: bool, stderr_is_tty: bool, no_color_env: bool) -> bool`, `pub fn is_interactive() -> bool`, `pub fn banner(colored: bool) -> String`, `pub fn note_line(message: &str, colored: bool) -> String`

- [ ] **Step 1: Register the module**

Modify `src/main.rs`, lines 1-7, from:
```rust
mod client;
mod commands;
mod config;
mod constants;
mod launch;
mod paths;
mod update;
```
to:
```rust
mod client;
mod commands;
mod config;
mod constants;
mod launch;
mod paths;
mod progress;
mod update;
```

- [ ] **Step 2: Write the failing tests**

Create `src/progress.rs` with this content (the module doesn't compile yet — `should_animate`/`banner`/`note_line` aren't defined — that's the expected failure):

```rust
//! Terminal progress feedback: an animated spinner for network waits and the
//! ServalAI/Cleverit/Raven text wordmark shown on `serval` launch.
//!
//! Fully inert (no ANSI, no background thread) when stdout/stderr aren't a
//! TTY or `NO_COLOR` is set — critical for CI logs and piped output.
#![allow(dead_code)]

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn animates_when_both_streams_are_ttys_and_color_allowed() {
        assert!(should_animate(true, true, false));
    }

    #[test]
    fn does_not_animate_when_stdout_is_not_a_tty() {
        assert!(!should_animate(false, true, false));
    }

    #[test]
    fn does_not_animate_when_stderr_is_not_a_tty() {
        assert!(!should_animate(true, false, false));
    }

    #[test]
    fn does_not_animate_when_no_color_env_is_set() {
        assert!(!should_animate(true, true, true));
    }

    #[test]
    fn banner_colored_wraps_lines_in_ansi_codes() {
        let b = banner(true);
        assert!(b.contains("ServalAI"));
        assert!(b.contains("a Cleverit company"));
        assert!(b.contains("powered by Raven"));
        assert!(b.contains("\x1b["));
    }

    #[test]
    fn banner_plain_has_no_ansi_codes() {
        let b = banner(false);
        assert_eq!(b, "ServalAI\na Cleverit company · powered by Raven");
        assert!(!b.contains("\x1b["));
    }

    #[test]
    fn note_line_plain_has_neutral_marker_and_no_ansi() {
        let n = note_line("using cached config", false);
        assert_eq!(n, "○ using cached config");
        assert!(!n.contains("\x1b["));
    }

    #[test]
    fn note_line_colored_still_contains_message_and_ansi() {
        let n = note_line("using cached config", true);
        assert!(n.contains("using cached config"));
        assert!(n.contains("\x1b["));
    }
}
```

- [ ] **Step 3: Run tests to verify they fail**

Run: `cargo test --lib progress:: -- --nocapture`
Expected: compile error — `cannot find function 'should_animate' in this scope` (and similarly for `banner`, `note_line`).

- [ ] **Step 4: Write the implementation**

Insert this above the `#[cfg(test)]` block in `src/progress.rs` (keep the doc comment and `#![allow(dead_code)]` already at the top of the file):

```rust
const ACCENT: &str = "\x1b[36m";
const DIM: &str = "\x1b[2m";
const RESET: &str = "\x1b[0m";

/// Pure decision: should we render an animated, colored spinner? False when
/// either stream isn't a TTY, or `NO_COLOR` is set to a non-empty value.
pub fn should_animate(stdout_is_tty: bool, stderr_is_tty: bool, no_color_env: bool) -> bool {
    stdout_is_tty && stderr_is_tty && !no_color_env
}

/// Reads real terminal/env state. Thin IO wrapper — `should_animate` carries
/// the tested decision logic; this just supplies live inputs to it.
pub fn is_interactive() -> bool {
    use std::io::IsTerminal;
    let no_color = std::env::var("NO_COLOR")
        .map(|v| !v.is_empty())
        .unwrap_or(false);
    should_animate(
        std::io::stdout().is_terminal(),
        std::io::stderr().is_terminal(),
        no_color,
    )
}

/// The ServalAI/Cleverit/Raven wordmark, shown once on plain `serval` launch.
pub fn banner(colored: bool) -> String {
    if colored {
        format!("{ACCENT}ServalAI{RESET}\n{DIM}a Cleverit company · powered by Raven{RESET}")
    } else {
        "ServalAI\na Cleverit company · powered by Raven".to_string()
    }
}

/// Formats the muted fallback note — visually distinct from the "serval:
/// <fatal error>" format used by `main.rs::run()` on real failures.
pub fn note_line(message: &str, colored: bool) -> String {
    if colored {
        format!("{DIM}○ {message}{RESET}")
    } else {
        format!("○ {message}")
    }
}
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test --lib progress::`
Expected: 8 tests pass (`animates_when_both_streams_are_ttys_and_color_allowed`, `does_not_animate_when_stdout_is_not_a_tty`, `does_not_animate_when_stderr_is_not_a_tty`, `does_not_animate_when_no_color_env_is_set`, `banner_colored_wraps_lines_in_ansi_codes`, `banner_plain_has_no_ansi_codes`, `note_line_plain_has_neutral_marker_and_no_ansi`, `note_line_colored_still_contains_message_and_ansi`).

- [ ] **Step 6: Lint and format**

Run: `cargo fmt` then `cargo clippy -- -D warnings`
Expected: no warnings. (`#![allow(dead_code)]` at the top of the file suppresses the expected "unused" warnings for `is_interactive`, which isn't called by anything yet.)

- [ ] **Step 7: Commit**

```bash
git add src/main.rs src/progress.rs
git commit -m "feat(progress): add pure banner/note formatting and TTY detection"
```

---

### Task 2: `Spinner` — background-thread animation

**Files:**
- Modify: `src/progress.rs` (add above the `mod tests` block; add new tests inside the existing `mod tests` block)

**Interfaces:**
- Consumes: `is_interactive() -> bool`, `note_line(message: &str, colored: bool) -> String` (Task 1)
- Produces: `pub struct Spinner`, `pub fn Spinner::start(message: &str) -> Spinner`, `pub fn Spinner::finish_silent(self)`, `pub fn Spinner::finish_note(self, message: &str)`

- [ ] **Step 1: Write the failing tests**

Add these test functions inside the existing `#[cfg(test)] mod tests { use super::*; ... }` block in `src/progress.rs` (alongside the ones from Task 1):

```rust
    #[test]
    fn non_animated_start_and_finish_silent_does_not_panic() {
        let spinner = Spinner::start_with("test message", false);
        spinner.finish_silent();
    }

    #[test]
    fn non_animated_start_and_finish_note_does_not_panic() {
        let spinner = Spinner::start_with("test message", false);
        spinner.finish_note("fallback happened");
    }

    #[test]
    fn animated_start_and_finish_silent_stops_the_thread_cleanly() {
        let spinner = Spinner::start_with("test message", true);
        std::thread::sleep(std::time::Duration::from_millis(50));
        spinner.finish_silent();
    }

    #[test]
    fn animated_start_and_finish_note_stops_the_thread_cleanly() {
        let spinner = Spinner::start_with("test message", true);
        std::thread::sleep(std::time::Duration::from_millis(50));
        spinner.finish_note("using cached config");
    }

    #[test]
    fn dropping_an_unfinished_spinner_stops_the_thread() {
        let spinner = Spinner::start_with("test message", true);
        std::thread::sleep(std::time::Duration::from_millis(50));
        drop(spinner);
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --lib progress::`
Expected: compile error — `no function or associated item named 'start_with' found for struct 'Spinner'` (and `Spinner` itself not found).

- [ ] **Step 3: Write the implementation**

Insert this above the `#[cfg(test)]` block in `src/progress.rs` (after the Task 1 functions):

```rust
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

const FRAMES: [&str; 8] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧"];

/// An animated "still working" indicator for network waits. Renders on
/// stderr only — stdout stays reserved for actual command output.
///
/// Fully inert when the terminal isn't interactive (see `is_interactive`):
/// `start` prints the message once as a static line instead of animating,
/// and `finish_note` still prints its note — same information, no ANSI.
pub struct Spinner {
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
    colored: bool,
}

impl Spinner {
    /// Starts the spinner using live terminal/env detection.
    pub fn start(message: &str) -> Spinner {
        Spinner::start_with(message, is_interactive())
    }

    fn start_with(message: &str, animated: bool) -> Spinner {
        if !animated {
            eprintln!("{message}");
            return Spinner {
                stop: Arc::new(AtomicBool::new(false)),
                handle: None,
                colored: false,
            };
        }
        let stop = Arc::new(AtomicBool::new(false));
        let stop_clone = Arc::clone(&stop);
        let msg = message.to_string();
        let handle = std::thread::spawn(move || {
            let mut i = 0usize;
            while !stop_clone.load(Ordering::Relaxed) {
                eprint!("\r{ACCENT}{}{RESET} {msg}", FRAMES[i % FRAMES.len()]);
                let _ = std::io::stderr().flush();
                i += 1;
                std::thread::sleep(Duration::from_millis(80));
            }
        });
        Spinner {
            stop,
            handle: Some(handle),
            colored: true,
        }
    }

    fn stop_thread(&mut self) {
        if let Some(handle) = self.handle.take() {
            self.stop.store(true, Ordering::Relaxed);
            let _ = handle.join();
            eprint!("\x1b[2K\r");
            let _ = std::io::stderr().flush();
        }
    }

    /// Stops the spinner and clears its line without printing anything
    /// further. Used on success, right before opencode's own output (or
    /// `exec`) takes over the terminal.
    pub fn finish_silent(mut self) {
        self.stop_thread();
    }

    /// Stops the spinner and prints a muted fallback note in its place.
    pub fn finish_note(mut self, message: &str) {
        self.stop_thread();
        eprintln!("{}", note_line(message, self.colored));
    }
}

impl Drop for Spinner {
    /// Safety net for early returns via `?` between `start` and a `finish_*`
    /// call: still stops the thread and clears the line so a fatal error
    /// printed right after doesn't land on top of a stuck spinner frame.
    fn drop(&mut self) {
        self.stop_thread();
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --lib progress::`
Expected: 13 tests pass (8 from Task 1 + 5 new).

- [ ] **Step 5: Lint and format**

Run: `cargo fmt` then `cargo clippy -- -D warnings`
Expected: no warnings (`#![allow(dead_code)]` still suppresses "unused" for `Spinner`/`start`, since nothing outside `progress.rs` calls it yet).

- [ ] **Step 6: Commit**

```bash
git add src/progress.rs
git commit -m "feat(progress): add background-thread Spinner with Drop cleanup"
```

---

### Task 3: `resolve_config` returns a fallback note, wired end-to-end (auth, sync, code)

This task is intentionally one atomic unit: changing `resolve_config`'s return
type and updating its three call sites cannot be split across commits without
leaving the crate in a non-compiling state in between (see Global Constraints).

**Files:**
- Modify: `src/client.rs:96-118` (`resolve_config`) and its `mod tests` block
- Modify: `src/commands.rs:22-59` (`auth`)
- Modify: `src/commands.rs:61-77` (`sync`)
- Modify: `src/commands.rs:348-359` (`code`)

**Interfaces:**
- Consumes: `crate::progress::Spinner::start`, `.finish_silent()`, `.finish_note()`, `crate::progress::banner`, `crate::progress::is_interactive` (Tasks 1-2)
- Produces: `pub fn resolve_config(...) -> (serde_json::Value, Option<String>, Option<String>)` — the tuple is now `(provider, email, fallback_note)`.

- [ ] **Step 1: Update the existing `client.rs` tests first (this will fail to compile)**

In `src/client.rs`, replace the `resolve_falls_back_to_cache_then_default` test with:

```rust
    #[test]
    fn resolve_falls_back_to_cache_then_default() {
        // Worker fails, cache present → use cache.
        let cache = serde_json::json!({ "name": "cached" });
        let (p, email, note) = resolve_config(&FailingHttp, "https://w.dev", "t", Some(&cache));
        assert_eq!(p["name"], "cached");
        assert!(email.is_none());
        assert!(note.unwrap().contains("using cached config"));
        // Worker fails, no cache → embedded default.
        let (p2, _, note2) = resolve_config(&FailingHttp, "https://w.dev", "t", None);
        assert_eq!(p2["name"], "ServalAI");
        assert!(note2.unwrap().contains("using default config"));
    }
```

Replace the `resolve_falls_back_when_worker_provider_not_object` test with:

```rust
    #[test]
    fn resolve_falls_back_when_worker_provider_not_object() {
        // Worker "succeeds" but returns provider as a non-object → must not be used;
        // with no cache we get the embedded default (an object), never a panic downstream.
        struct StringProviderHttp;
        impl Http for StringProviderHttp {
            fn get_json(&self, _u: &str, _b: &str) -> Result<serde_json::Value, String> {
                Ok(
                    serde_json::json!({ "email": "x@y.com", "models": [], "provider": "oops-not-an-object" }),
                )
            }
            fn get_bytes(&self, _url: &str) -> Result<Vec<u8>, String> {
                Ok(vec![])
            }
        }
        let (p, email, note) = resolve_config(&StringProviderHttp, "https://w.dev", "t", None);
        assert!(p.is_object());
        assert_eq!(p["name"], "ServalAI");
        assert!(email.is_none());
        assert!(note.unwrap().contains("unexpected config shape"));
    }
```

Add a new test right after it, for the success path:

```rust
    #[test]
    fn resolve_returns_no_fallback_note_on_success() {
        let http = FakeHttp {
            body: serde_json::json!({
                "email": "dev@cleveritgroup.com",
                "models": ["dynamic/balanced"],
                "provider": { "npm": "@ai-sdk/openai-compatible", "name": "ServalAI" }
            }),
        };
        let (_, email, note) = resolve_config(&http, "https://w.example.dev/", "aig_token", None);
        assert_eq!(email.as_deref(), Some("dev@cleveritgroup.com"));
        assert!(note.is_none());
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --lib client::`
Expected: compile error — `resolve_config` returns a 2-tuple, but the tests destructure a 3-tuple (`mismatched types` / `expected a tuple with 3 elements, found one with 2 elements`).

- [ ] **Step 3: Update `resolve_config`**

Replace `src/client.rs:96-118` (the whole `resolve_config` function) with:

```rust
/// Resolve the provider config without ever failing, and always return a JSON object:
/// Worker (if it returns an object) → cache (if an object) → embedded default.
///
/// Returns `(provider, email, fallback_note)`. `fallback_note` is `Some(..)`
/// exactly when we degraded to cached/default config — the caller decides how
/// to surface it (typically via `progress::Spinner::finish_note`), never as a
/// "serval: <fatal>"-styled error, since the command still succeeds.
pub fn resolve_config(
    http: &dyn Http,
    worker_url: &str,
    token: &str,
    cached: Option<&serde_json::Value>,
) -> (serde_json::Value, Option<String>, Option<String>) {
    match fetch_config(http, worker_url, token) {
        Ok(fc) if fc.provider.is_object() => (fc.provider, Some(fc.email), None),
        Ok(_) => {
            let note = fallback_note(
                fallback_source(cached),
                "gateway returned an unexpected config shape",
            );
            (fallback_provider(cached), None, Some(note))
        }
        Err(e) => {
            let note = fallback_note(fallback_source(cached), &format!("gateway unreachable: {e}"));
            (fallback_provider(cached), None, Some(note))
        }
    }
}

fn fallback_note(source: &'static str, detail: &str) -> String {
    format!("using {source} config — {detail}")
}
```

- [ ] **Step 4: Run the `client.rs` tests to verify they pass**

Run: `cargo test --lib client::`
Expected: this alone will still fail to *compile the crate* (`commands.rs` still uses the old 2-tuple in `auth`/`sync`/`code`) — that's expected. Read the compiler output and confirm the only errors are in `commands.rs`, not in `client.rs`. Proceed to Step 5 before attempting to run tests again.

- [ ] **Step 5: Update `auth`**

Replace `src/commands.rs:22-59` with:

```rust
pub fn auth(token: Option<String>) -> Result<(), String> {
    let mut cfg = load();
    let token = match token {
        Some(t) => t,
        None => {
            eprint!("Paste your ServalAI token (from {TOKEN_URL}): ");
            std::io::stderr().flush().ok();
            let mut s = String::new();
            std::io::stdin()
                .read_line(&mut s)
                .map_err(|e| format!("could not read token: {e}"))?;
            s.trim().to_string()
        }
    };
    if token.is_empty() {
        return Err(format!(
            "no token provided.\nGet yours at {TOKEN_URL} and run `serval auth` again."
        ));
    }
    cfg.token = Some(token.clone());
    let spinner = crate::progress::Spinner::start("Contacting gateway…");
    let (provider, email, fallback_note) = resolve_config(
        &UreqHttp,
        &cfg.worker_url,
        &token,
        cfg.cached_provider.as_ref(),
    );
    match fallback_note {
        Some(note) => spinner.finish_note(&note),
        None => spinner.finish_silent(),
    }
    cfg.cached_provider = Some(provider);
    cfg.cached_email = email.clone();
    cfg.save(&paths::config_file())?;
    match email {
        Some(e) => println!("Authenticated as {e}. Ready — run `serval` to start coding."),
        None => println!(
            "Token saved, but the gateway could not be reached to confirm your identity.\n\
             Your provider config uses the built-in defaults. Run `serval sync` later to refresh."
        ),
    }
    Ok(())
}
```

- [ ] **Step 6: Update `sync`**

Replace `src/commands.rs:61-77` with:

```rust
pub fn sync() -> Result<(), String> {
    let mut cfg = load();
    let token = require_token(&cfg)?;
    let spinner = crate::progress::Spinner::start("Contacting gateway…");
    let (provider, email, fallback_note) = resolve_config(
        &UreqHttp,
        &cfg.worker_url,
        &token,
        cfg.cached_provider.as_ref(),
    );
    match fallback_note {
        Some(note) => spinner.finish_note(&note),
        None => spinner.finish_silent(),
    }
    cfg.cached_provider = Some(provider);
    if email.is_some() {
        cfg.cached_email = email;
    }
    cfg.save(&paths::config_file())?;
    println!("Provider config synced from the gateway.");
    Ok(())
}
```

- [ ] **Step 7: Update `code`**

Replace `src/commands.rs:348-359` with:

```rust
pub fn code(passthrough: Vec<String>) -> Result<(), String> {
    let cfg = load();
    let token = require_token(&cfg)?;
    println!("{}", crate::progress::banner(crate::progress::is_interactive()));
    let spinner = crate::progress::Spinner::start("Connecting to gateway…");
    let (provider, _, fallback_note) = resolve_config(
        &UreqHttp,
        &cfg.worker_url,
        &token,
        cfg.cached_provider.as_ref(),
    );
    match fallback_note {
        Some(note) => spinner.finish_note(&note),
        None => spinner.finish_silent(),
    }
    let env = build_env(&provider, &cfg.worker_url, &token, &paths::bundle_dir());
    ExecLauncher.exec(&paths::opencode_bin(), &passthrough, &env)
}
```

Note: `require_token(&cfg)?` still runs *before* the banner, so `serval code` with no token errors immediately with no banner/spinner — unchanged from today's behavior (see the existing `code_without_token_errors_cleanly` integration test).

- [ ] **Step 8: Build and run the full test suite**

Run: `cargo build && cargo test`
Expected: builds clean; the entire test suite passes, including the 3 touched/added `client::` tests from Step 1, and `tests/cli.rs::code_without_token_errors_cleanly` (unaffected — see the note above) and every other existing test.

There is no new automated test for `auth`/`sync` themselves: they were never unit-testable in isolation before this change either (they call the real `UreqHttp` directly, no injected `Http`/`Launcher` trait at this call site). Verification for those two functions is the full-suite pass plus the manual smoke test in Task 5.

- [ ] **Step 9: Lint and format**

Run: `cargo fmt` then `cargo clippy -- -D warnings`
Expected: no warnings.

- [ ] **Step 10: Commit**

```bash
git add src/client.rs src/commands.rs
git commit -m "feat: replace error-styled gateway fallback with a spinner + muted note

resolve_config() no longer eprintln!s a serval:-prefixed fallback message
on the happy path (a successful degrade was indistinguishable from a
fatal error). auth/sync/code now show an animated spinner while waiting
and a distinct note only when they actually fall back to cached/default
config. code() also prints the ServalAI/Cleverit/Raven banner."
```

---

### Task 4: Wire `ping`, `models`, `usage`, `doctor` — and drop the temporary `allow(dead_code)`

**Files:**
- Modify: `src/commands.rs:139-162` (`ping`)
- Modify: `src/commands.rs:164-187` (`models`)
- Modify: `src/commands.rs:189-209` (`usage`)
- Modify: `src/commands.rs:211-267` (`doctor`)
- Modify: `src/progress.rs` (remove `#![allow(dead_code)]`)

**Interfaces:**
- Consumes: `crate::progress::Spinner` (Task 2)

- [ ] **Step 1: Update `ping`**

Replace `src/commands.rs:139-162` with:

```rust
pub fn ping() -> Result<(), String> {
    let cfg = load();
    let spinner = crate::progress::Spinner::start("Contacting gateway…");
    let status = health_check(&UreqHttp, &cfg.worker_url)
        .map_err(|e| format!("gateway unreachable: {e}"))?;
    spinner.finish_silent();
    println!("Gateway: {}", cfg.worker_url);
    println!("Status:  {status}");

    // Try to fetch provider config to show available models
    if let Ok(token) = require_token(&cfg) {
        match fetch_config(&UreqHttp, &cfg.worker_url, &token) {
            Ok(fc) if !fc.models.is_empty() => {
                println!("\nAvailable models:");
                for m in &fc.models {
                    println!("  {m}");
                }
                println!("\nIdentified as: {}", fc.email);
            }
            _ => {}
        }
    } else {
        println!("\nRun `serval auth` to authenticate and see your models.");
    }
    Ok(())
}
```

Note: if `health_check` returns `Err`, the `?` returns immediately — `spinner` (a local variable) is dropped at that point, and its `Drop` impl (Task 2) stops the thread and clears the line before `main.rs::run()` prints the fatal error. No explicit cleanup needed on the error path. The second `fetch_config` call (for the model list) intentionally has no separate spinner — it's a fast follow-on to a connection that's already warm, and the spec's per-command table only calls for one spinner per command.

- [ ] **Step 2: Update `models`**

Replace `src/commands.rs:164-187` with:

```rust
pub fn models() -> Result<(), String> {
    let cfg = load();
    let token = require_token(&cfg)?;
    let spinner = crate::progress::Spinner::start("Contacting gateway…");
    let fc = fetch_config(&UreqHttp, &cfg.worker_url, &token)
        .map_err(|e| format!("could not fetch model list: {e}"))?;
    spinner.finish_silent();

    println!("ServalAI Models");
    println!("━━━━━━━━━━━━━━━");
    println!("Account: {}", fc.email);
    println!();
    for m in &fc.models {
        let description = match m.as_str() {
            s if s.contains("dynamic/power") => "Hard tasks, architecture, large refactors",
            s if s.contains("dynamic/balanced") => "Everyday work (default)",
            s if s.contains("dynamic/light") => "Quick, mechanical tasks",
            _ => "",
        };
        println!("  {m}");
        if !description.is_empty() {
            println!("    {description}");
        }
    }
    Ok(())
}
```

- [ ] **Step 3: Update `usage`**

Replace `src/commands.rs:189-209` with:

```rust
pub fn usage() -> Result<(), String> {
    let cfg = load();
    let token = require_token(&cfg)?;
    let spinner = crate::progress::Spinner::start("Contacting gateway…");
    let data = fetch_usage(&UreqHttp, &cfg.worker_url, &token)?;
    spinner.finish_silent();

    println!("ServalAI Usage");
    println!("━━━━━━━━━━━━━━");
    if let Some(tokens) = data.get("total_tokens").and_then(|v| v.as_u64()) {
        println!("  Total tokens:   {tokens}");
    }
    if let Some(sessions) = data.get("session_count").and_then(|v| v.as_u64()) {
        println!("  Sessions:       {sessions}");
    }
    if let Some(model) = data.get("last_model").and_then(|v| v.as_str()) {
        println!("  Last model:     {model}");
    }
    if let Some(at) = data.get("last_used_at").and_then(|v| v.as_str()) {
        println!("  Last used:      {at}");
    }
    Ok(())
}
```

- [ ] **Step 4: Update `doctor`**

In `src/commands.rs`, inside `doctor` (lines 211-267), replace just the "3. Gateway" block:

```rust
    // 3. Gateway
    println!("  Gateway:      {}", cfg.worker_url);
    match health_check(&UreqHttp, &cfg.worker_url) {
        Ok(s) => println!("    ✓ reachable (status: {s})"),
        Err(e) => println!("    ✗ unreachable: {e}"),
    }
```

with:

```rust
    // 3. Gateway
    println!("  Gateway:      {}", cfg.worker_url);
    let spinner = crate::progress::Spinner::start("Checking gateway…");
    let health = health_check(&UreqHttp, &cfg.worker_url);
    spinner.finish_silent();
    match health {
        Ok(s) => println!("    ✓ reachable (status: {s})"),
        Err(e) => println!("    ✗ unreachable: {e}"),
    }
```

`doctor`'s own ✓/✗ report format already communicates the outcome clearly, so this always calls `finish_silent()` regardless of the result — the spinner only covers the wait, never duplicates the report line.

- [ ] **Step 5: Remove the temporary `#![allow(dead_code)]`**

In `src/progress.rs`, delete the `#![allow(dead_code)]` line from the top of the file (everything in the module is now called from `commands.rs`).

- [ ] **Step 6: Build, test, lint**

Run: `cargo build && cargo test && cargo fmt --check && cargo clippy -- -D warnings`
Expected: all pass, zero warnings. If `clippy` flags anything as unused, it means a Step above was missed — re-check that every `progress::` item has a real caller.

- [ ] **Step 7: Commit**

```bash
git add src/commands.rs src/progress.rs
git commit -m "feat(commands): show a spinner around ping/models/usage/doctor gateway calls"
```

---

### Task 5: Full validation and manual smoke test

**Files:** none (verification only)

- [ ] **Step 1: Full automated suite**

Run, in order:
```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo build --release
```
Expected: all four succeed with no errors or warnings. This mirrors `.github/workflows/ci.yml` exactly.

- [ ] **Step 2: Manual smoke test — banner, spinner, and exec ordering**

The animation itself can't be asserted by an automated test (it's terminal rendering), so verify it by eye:

```bash
cargo build
mkdir -p /tmp/serval-smoke/bundle
cp target/debug/serval /tmp/serval-smoke/serval
# No real `opencode` binary is placed here on purpose — this proves the
# banner/spinner render BEFORE the exec attempt, then exec fails cleanly.
SERVAL_INSTALL_ROOT=/tmp/serval-smoke XDG_CONFIG_HOME=/tmp/serval-smoke/cfg \
  /tmp/serval-smoke/serval auth --token smoke-test-token

SERVAL_INSTALL_ROOT=/tmp/serval-smoke XDG_CONFIG_HOME=/tmp/serval-smoke/cfg \
  /tmp/serval-smoke/serval
```
Expected: the `ServalAI` / `a Cleverit company · powered by Raven` banner prints in cyan/dim, a braille spinner animates next to "Connecting to gateway…" for a moment (real network call to the default gateway, or a fast fallback if unreachable — either way you should see either nothing further on success or a muted `○ using ... config — ...` note on fallback, never a `serval:`-prefixed line), and finally the command fails with `serval: failed to exec ... No such file or directory` (from the missing `opencode` binary) — confirming the banner/spinner ran to completion *before* the exec attempt, not after.

- [ ] **Step 3: Manual smoke test — non-interactive and NO_COLOR fallback**

```bash
SERVAL_INSTALL_ROOT=/tmp/serval-smoke XDG_CONFIG_HOME=/tmp/serval-smoke/cfg \
  /tmp/serval-smoke/serval | cat

NO_COLOR=1 SERVAL_INSTALL_ROOT=/tmp/serval-smoke XDG_CONFIG_HOME=/tmp/serval-smoke/cfg \
  /tmp/serval-smoke/serval
```
Expected: in both cases, plain text only — no `\x1b[` escape sequences, no animation, just the banner and a single static "Connecting to gateway…" line followed by the outcome.

- [ ] **Step 4: Clean up the smoke-test scratch directory**

```bash
rm -rf /tmp/serval-smoke
```

- [ ] **Step 5: Final review commit (if `cargo fmt` made any changes in Step 1)**

```bash
git status --short
# only run this if cargo fmt --check in Step 1 actually modified files:
git add -u
git commit -m "style: cargo fmt"
```

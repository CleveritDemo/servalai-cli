# Branded progress feedback (`serval` loading indicator) — Design

**Date:** 2026-07-31
**Status:** Approved design — ready for implementation plan
**Repo (target):** `CleveritDemo/servalai-cli`
**Related:** `docs/superpowers/specs/2026-07-25-servalai-cli-design.md` (base CLI design)

---

## 1. Problem & goal

Two user-reported symptoms trace back to the same root cause:

1. **"It gives errors on steps, even though it works right after."** `resolve_config()`
   (`src/client.rs`) is called by `auth`, `sync`, and — critically — by plain `serval`
   (`commands::code()`), the most-used path. On *any* gateway hiccup (slow response,
   cold start, DNS blip, timeout) it prints
   `serval: using {cached|default} config (<error>)` to stderr and then **recovers
   and continues successfully**. That message uses the exact same `"serval: {msg}"`
   prefix as the fatal-error path in `main.rs::run()`
   (`eprintln!("serval: {e}{hint}")`), so a purely informational recovery message
   is visually indistinguishable from a command-killing error.

2. **"`serval` takes a while and feels broken/stuck."** `code()`'s path is:
   load config → `resolve_config()` (HTTP GET, up to 5s connect + 20s read
   timeout) → build env → `exec` opencode. Nothing is printed between typing
   `serval` and either opencode appearing or the fallback message firing. A slow
   gateway means several seconds (up to ~25s worst case) of a blank prompt that
   looks hung. (`update_cmd()` already prints `"Checking for updates…"` before
   its own network call — that courtesy is missing from `code()`.)

**Goal:** give every network-bound command visible, animated feedback while it
waits, and make the plain `serval` launch — the command people run dozens of
times a day — carry ServalAI's brand identity (`ServalAI < Cleverit < Raven`)
as a text wordmark. Fix the error/warning visual confusion as part of the same
change, since both are caused by the same silent-fallback code path.

**Success criteria:**
- No network-bound command produces terminal output that looks identical to a
  fatal error unless the command actually failed.
- No network-bound command is silent for more than ~100ms while waiting on I/O.
- Plain `serval` shows the ServalAI/Cleverit/Raven wordmark once per launch.
- Zero new crate dependencies; binary stays static/small (musl cross-compile
  friendly, matching the existing `ureq` minimal-features precedent).
- Fully inert (no ANSI, no animation) when stdout/stderr aren't a TTY, or
  `NO_COLOR` is set — critical for CI logs and piped output.

---

## 2. Non-goals (v1 — YAGNI)

- No `--quiet` / config flag to suppress the banner. Can follow up later if it
  turns out to be annoying in daily use.
- No real logo/ASCII art. Text wordmark only, per explicit decision.
- No configurable color palette. One fixed accent color for now.
- No changes to `pi()` / `aider()` — neither does a gateway round-trip before
  `exec`, so neither is silently slow today.
- No retry/backoff logic changes to the HTTP layer itself — this is purely a
  presentation-layer fix around the existing `resolve_config` behavior.

---

## 3. Architecture overview

New module `src/progress.rs`, following the repo's existing pattern (see
`launch.rs`, `client.rs`) of keeping pure/testable logic separate from the thin
IO edge.

```rust
pub struct Spinner { /* stop flag (Arc<AtomicBool>), thread JoinHandle */ }

impl Spinner {
    /// Starts animating `message` on stderr if TTY + animation is enabled
    /// (see should_animate below). Otherwise prints one static line and
    /// returns a no-op handle.
    pub fn start(message: &str) -> Spinner;

    /// Stops the thread, clears the line, prints nothing further. Used on
    /// success, immediately before opencode's own output/exec takes over.
    pub fn finish_silent(self);

    /// Stops the thread, clears the line, prints a muted "○ <message>" note —
    /// visually distinct from the "serval: <fatal>" error format. Used on
    /// graceful fallback (cached/default config).
    pub fn finish_note(self, message: &str);
}

/// Pure decision function — no IO — so it's unit-testable without a real
/// terminal or env vars mutated in-process.
pub fn should_animate(stdout_is_tty: bool, stderr_is_tty: bool, no_color_env: bool) -> bool;

/// The ServalAI/Cleverit/Raven wordmark. Pure string, no IO.
pub fn banner() -> String;
```

- **Rendering:** background `std::thread`, repaints `\r<braille-frame> <message>`
  every ~80ms, clears the line (`\x1b[2K\r`) on `finish_*`. Runs on **stderr**
  (stdout is reserved for actual command output like `models`/`usage` listings —
  never mixed with progress chrome).
- **TTY / NO_COLOR detection:** `std::io::IsTerminal` (stable in std since Rust
  1.70; `rust-version = "1.85"` already covers it — no new dependency) plus the
  `NO_COLOR` env var. When `should_animate()` is false (piped, CI, redirected,
  `NO_COLOR` set), `start()` prints one static plain-text line instead of
  animating, and `finish_*` still prints its message, just without ANSI escapes
  — same information reaches the user/log either way.
- **Color:** one fixed accent (cyan, `\x1b[36m`) on the wordmark and spinner
  frames only. The `○` fallback note is uncolored/dim on purpose — it must read
  as "neutral information," not as a brand moment and not as an error.

### Braille spinner frames
`⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏` — the de facto standard used by npm/cargo/gh-style CLIs.

---

## 4. Banner content

Shown once, only at the top of plain `serval` (`commands::code()`), before the
spinner starts:

```
ServalAI
a Cleverit company · powered by Raven
```

Plain text, cyan accent on `ServalAI`, dim on the subtitle line. No ASCII art.

---

## 5. Integration per command

| Command | Banner? | Spinner message | On fallback |
|---|---|---|---|
| `code()` (plain `serval`) | Yes | `"Connecting to gateway…"` | `finish_note("using cached/default config — gateway unreachable")`; still launches opencode |
| `auth`, `sync` | No | `"Contacting gateway…"` | same note style; command still completes |
| `ping`, `models`, `usage` | No | `"Contacting gateway…"` | on a genuine failure these still return `Err` and go through `main.rs::run()`'s existing fatal path unchanged — the spinner only wraps the *wait*, not the error handling |
| `doctor` | No | `"Checking gateway…"` (wraps only the one `health_check` call) | unaffected — doctor's ✓/✗ report format already communicates pass/fail correctly; this only adds motion during the wait |

`pi()` and `aider()` are unaffected — neither hits the network before `exec`.

---

## 6. Sequencing around `exec()`

`code()`, `pi()`, `aider()` end by replacing the process image via `exec()`
(see `launch.rs::ExecLauncher`). All spinner usage is strictly sequential:
spawn thread → blocking HTTP call happens on the *main* thread inside
`resolve_config()` → `Spinner` is stopped and joined before `resolve_config()`
returns control to `code()`. There is no window where the spinner thread could
still be repainting when `exec()` replaces the process — `code()` never starts
the next spinner (there isn't one) until the previous one has fully joined.

---

## 7. Fixing the error/warning visual confusion

`resolve_config()` in `client.rs` currently does:
```rust
eprintln!("serval: using {} config ({e})", fallback_source(cached));
```
This call site is replaced with a call into `Spinner::finish_note(...)`
(muted, no `"serval:"` prefix, no `✗`). The fatal-error path in
`main.rs::run()` (`eprintln!("serval: {e}{hint}")`) is untouched — real errors
(bad token, `aider` not on PATH, malformed config, etc.) still surface exactly
as they do today. The only change is that a **successful degrade** no longer
looks like a **failure**.

---

## 8. Testing

Following the repo's established pattern (pure logic tested, IO isolated —
see `client.rs`'s `resolve_config` tests, `launch.rs`'s `Launcher` trait):

- `banner()` — pure string, exact-match tested.
- `should_animate()` — pure function of three booleans, table-tested for all
  8 combinations.
- Note/message formatting for each outcome (success / cached fallback /
  default fallback) — pure functions, unit-tested like the existing
  `resolve_falls_back_to_cache_then_default` test.
- The thread/rendering loop itself is intentionally thin and not unit-tested,
  matching how `ExecLauncher`'s real `exec()` is untested today — the `Spinner`
  struct's public API is small enough that the untestable surface stays small.

---

## 9. Open questions / follow-ups (not blocking v1)

- Whether a `--quiet` flag or `SERVAL_NO_BANNER` env var is worth adding once
  real usage feedback comes in.
- Whether `doctor`'s per-check spinner is worth the added code given doctor is
  already a synchronous, quick report — flagged as optional in this design;
  implementer may drop it if it doesn't feel warranted during implementation.

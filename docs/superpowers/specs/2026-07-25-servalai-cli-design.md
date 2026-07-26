# ServalAI CLI (`serval`) — Design

**Date:** 2026-07-25
**Status:** Approved design — ready for implementation plan
**Repo (target):** `CleveritDemo/servalai-cli`
**Related:** worker `ai-wrangler-cf-gateway-controller` (data plane), platform
`soporte-ti-knowledgebase-wikijs` module `ai_gateway` (control plane),
ADR-0008 (advertise model limits — this CLI closes its client-config gap).

---

## 1. Problem & goal

ServalAI gives Cleverit developers company-funded model access through coding
agents (opencode). Adoption is stalling: developers won't install opencode **and**
edit `~/.config/opencode/opencode.json` **and** export an env var when Copilot is
one click. Friction is the adoption killer.

**Goal:** collapse onboarding to two steps — one install, one token paste — after
which a fully-configured coding agent (ServalAI provider + a curated agent/skill
loadout) is ready to use. Everything else is handled for the developer.

**Success criteria:**
- A new dev goes from nothing to a working ServalAI-backed coding session in
  under 2 minutes, touching no config file and setting no env var.
- Provider config (tier context windows, allowed models) stays current
  **server-side** — the dev never edits a file to pick up a change.
- The dev's own opencode setup (global + project `.opencode`) is preserved, not
  clobbered.

---

## 2. Non-goals (v1 — YAGNI)

- **Pi harness.** opencode only in v1; Pi is the less-integrated harness. Design
  does not preclude adding it later.
- **Browser/device-code login.** v1 is paste-token. The CLI is structured so a
  `serval login` browser flow can slot in later without breaking `serval auth`.
- **Telemetry / observability hooks** (that is ADR-0040 territory, separate).
- **Native Windows build.** WSL2 (Linux userland) is covered by the Linux x64
  build; no native Windows target in v1.
- **OS keychain** for the token. v1 uses a `0600` file; keychain is a later option.
- **Forking opencode.** We wrap and bundle the upstream binary; we never maintain
  a source fork.

---

## 3. Architecture overview

`serval` is a thin native launcher (Rust) that bundles a pinned opencode binary
and a read-only ServalAI config directory, and injects that config into opencode
**via environment variables** at launch — so it never writes into the developer's
own opencode config.

```
serval-<target>.tar.gz               (one release archive per platform)
├── serval                           native launcher binary (Rust)
├── opencode                         pinned upstream opencode binary (per-platform)
└── bundle/                          read-only ServalAI config dir
    ├── opencode.jsonc               ServalAI provider template (no token in it)
    ├── AGENTS.md                    curated global agent instructions
    ├── agents/*.md                  curated agents (architect, developer, …)
    └── skills/…                     curated skills
```

Installed layout (per user):

```
~/.local/share/serval/current/       the extracted archive above (swapped on update)
~/.config/serval/config.toml         { token (0600), worker_url, cached_config, pinned_versions }
```

### Injection mechanism (verified against opencode source)

opencode resolves config directories in this order and merges them, and exposes
override hooks we use:

- `OPENCODE_CONFIG_DIR=<bundle>` — adds our bundle dir to opencode's config search
  path. opencode loads `opencode.jsonc`, `AGENTS.md`, `agents/**/*.md`, and
  `skills/…` from it. This is how the curated loadout arrives ready-to-use.
- `OPENCODE_CONFIG_CONTENT=<inline JSON>` — merged at "local" scope; used to inject
  the **provider block with the resolved token + baseURL** dynamically, so the
  token is never written into a file inside the bundle.
- The dev's global `~/.config/opencode/` and project `.opencode/` **still load and
  merge** — ServalAI + curated agents are layered on top, nothing is clobbered.

Token also exported as `CF_CLEVER_DEV_TOKEN` for any path that reads it directly.

`serval` then `exec`s the bundled `opencode` (replacing its own process), so opencode
owns the TTY exactly as if launched directly.

---

## 4. Config delivery — keeping config server-owned

The developer must never edit a config file, and the provider config (tier context
windows, the user's allowed models) must be able to change server-side. This closes
the gap ADR-0008 documented (the `limit.context` value that *had* to live in client
config).

**Boundary constraint (must hold):** the platform touches the data plane **only by
writing to KV**; the Worker **only reads KV**. No new platform HTTP surface.

### Mechanism

1. **Platform (control plane):** `buildRealKvValue` already writes
   `{ email, limits }` to the token's KV entry on assignment/resync. **Extend it to
   also write a rendered `config`** — the opencode provider block plus the user's
   allowed models with their context/output windows (reuse `buildOpencodeConfig` /
   `resolveEffectiveLimits`). Refreshed automatically on every catalog/limit change
   (the existing auto-resync already fires there).

2. **Worker (data plane):** add one small read-only route
   `GET /cli/config` → read the bearer token from KV → return the stored `config`
   JSON (plus `email`, allowed models). No catalog logic in the Worker; it just
   serves what the platform wrote.

3. **CLI:** on launch and on `serval sync`, `GET <worker_url>/cli/config` with the
   bearer token; cache the result in `config.toml`. On launch, build
   `OPENCODE_CONFIG_CONTENT` from the cached config + token and inject.

**Degrade, don't block:** if `/cli/config` is unreachable, `serval` uses the last
cached config and launches anyway (same spirit as the Worker's streaming-transparent,
fail-open posture). A stale-but-working session beats a hard failure.

---

## 5. Command surface (v1)

| Command | Behavior |
|---|---|
| `serval auth` | Prompt for a token (or `--token`), store to `~/.config/serval/config.toml` (`0600`), validate by calling `GET /cli/config`. Prints the resolved email + allowed models on success. |
| `serval` / `serval code` | Refresh config (best-effort) → build env → `exec` bundled opencode. Extra args pass through to opencode. |
| `serval sync` | Re-fetch provider config + token scope from the Worker; update cache. No binary change. |
| `serval update` | Self-update: fetch latest GitHub release, download the platform `serval-<target>.tar.gz`, extract, **atomic-swap the whole install dir** (bumps pinned opencode too). Idempotent — "already up to date" when current. |
| `serval status` | Show serval version, pinned opencode version, worker URL, masked token, resolved email + allowed models. |
| `serval logout` | Clear the stored token from `config.toml`. |

Global: `--version`, `--help`. Progress/diagnostics to **stderr**, final status to
**stdout** (mirrors `karluiz-tool-cli` convention).

---

## 6. Self-update (mirrors `karluiz-tool-cli`, one refinement)

Reuse the proven `ktool` pattern:
- Compile-time `CURRENT_TARGET` per platform triple; `compile_error!` on unsupported.
- `ureq` (rustls, no C deps) → `GET .../repos/CleveritDemo/servalai-cli/releases/latest`.
- Compare `tag_name` vs `v{CARGO_PKG_VERSION}`; exit 0 "already up to date" if equal.
- Select asset by `serval-<CURRENT_TARGET>.tar.gz` suffix; download; extract.

**Refinement:** `serval` ships a *bundle* (serval + opencode + config), not a single
binary, so `update` swaps the **whole `current/` install dir atomically** (extract to
a temp dir alongside, then rename-swap), instead of self-replacing one file. This
gives central control over the pinned opencode version.

`serval sync` is the lightweight cousin: config only, no download of binaries.

---

## 7. Distribution

- **Targets (4):**
  `x86_64-unknown-linux-musl` (Ubuntu x64 **+ WSL2**),
  `aarch64-unknown-linux-musl` (Ubuntu ARM),
  `x86_64-apple-darwin` (macOS Intel),
  `aarch64-apple-darwin` (macOS Apple Silicon).
  opencode publishes prebuilt binaries for all four; the release job pairs each
  serval build with the matching opencode binary.
- **Cross-compile:** `cargo-zigbuild` (same as `karluiz-tool-cli`); musl for static
  Linux binaries.
- **Release CI:** tag `vX.Y.Z` → GitHub Actions matrix → per target: build serval,
  fetch the pinned opencode binary, assemble `bundle/`, tar → upload
  `serval-<target>.tar.gz` to GitHub Releases.
- **Install:** `install.sh` (curl | sh) detects OS/arch, downloads the right archive,
  extracts to `~/.local/share/serval/current/`, symlinks `serval` onto `PATH`
  (`~/.local/bin` or `/usr/local/bin`). `serval update` uses the same asset layout.
- **CI (non-release):** `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test`
  on every push (mirrors `ktool`).

---

## 8. Components (for isolation / testability)

| Unit | Responsibility | Depends on |
|---|---|---|
| `config` | Load/store `config.toml`, `0600` perms, mask token | `dirs`, `toml`, `serde` |
| `client` | `GET /cli/config` against the Worker, cache result | `ureq`, `serde_json` |
| `launch` | Build env (`OPENCODE_CONFIG_DIR`, `OPENCODE_CONFIG_CONTENT`, `CF_CLEVER_DEV_TOKEN`), locate bundled opencode, `exec` | `config`, `client` |
| `update` | Target detection, release lookup, download, extract, atomic dir-swap | `ureq`, `tar`, `flate2` |
| `bundle` | Resolve install-dir paths (`current/`, `opencode`, `bundle/`) | `dirs` |
| `cli` | `clap` command parsing, wire the above, stderr/stdout discipline | `clap` |

Pure logic (target detection, version compare, asset selection, config merge,
env construction, token masking) is unit-tested with `cargo test`. `exec` path is
covered by a smoke test asserting the correct env + argv are assembled (dependency-
injected process spawner so no real opencode needed in unit tests).

---

## 9. Error handling

- **No/invalid token:** friendly message pointing to Mi Portal to copy the token;
  non-zero exit; never echo the token.
- **`/cli/config` unreachable at launch:** warn to stderr, fall back to cached
  config, launch anyway. Hard-fail only if there is no token at all.
- **`update` download/extract failure:** leave the existing `current/` untouched
  (swap only after a successful extract + integrity check); non-zero exit.
- **Unsupported platform:** `compile_error!` at build time; runtime guidance if a
  release asset is missing.
- **Token never logged** anywhere (matches the Worker invariant).

---

## 10. Server-side changes required (small, tracked separately)

These live in the other two repos and are prerequisites for `serval sync`:

1. **Platform** (`soporte-ti-knowledgebase-wikijs`): extend `buildRealKvValue` to
   include the rendered `config` in the KV value. No new HTTP endpoint. Each is
   an ADR-worthy change in that repo.
2. **Worker** (`ai-wrangler-cf-gateway-controller`): add `GET /cli/config`
   (read token from KV → return stored `config`). Streaming-transparent posture
   unaffected; this is a plain JSON read route.

Until these ship, the CLI can operate with a **bundled default config** (static
provider + tiers) so `serval` is usable before the server-side `sync` path lands;
`serval sync` becomes live once the route exists.

---

## 11. Open questions / risks

- **opencode env-var stability.** `OPENCODE_CONFIG_DIR` / `OPENCODE_CONFIG_CONTENT`
  are the injection contract. If upstream renames them, the launcher breaks. Mitigate:
  pin opencode per release (we control the bump) and smoke-test injection in release CI.
- **macOS Gatekeeper.** Unsigned binaries trigger a quarantine prompt. v1: document
  the `xattr -d com.apple.quarantine` step in `install.sh`; consider notarization later.
- **Bundle size.** ~50–100 MB per archive (opencode binary dominates). Acceptable for
  a one-time install; `serval update` re-downloads the whole bundle on opencode bumps.
- **Token in `config.toml`.** `0600` is pragmatic (opencode's own `auth.json` is a
  plain file); keychain is a later hardening option.
- **Curated agent/skill provenance.** The bundled `agents/` + `skills/` are seeded
  from the maintainer's `~/.config/opencode`; decide what is company-curated vs
  personal before first release.

---

## 12. Rollout sequence (informs the implementation plan)

1. Scaffold `CleveritDemo/servalai-cli` (Rust, clap, CI) with a **bundled default
   config** so `serval auth` + `serval` + `serval update` work end-to-end without
   any server change.
2. Add the Worker `GET /cli/config` route + platform KV `config` field; wire
   `serval sync` to it (degrade-to-cache).
3. Curate the shipped agent/skill loadout; finalize `install.sh` + release CI.
4. Pilot with a small group, then fold the install line into the ServalAI onboarding
   docs (replacing the manual opencode config steps).

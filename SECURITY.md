# Security

## Token storage

ServalAI tokens are stored at `~/.config/serval/config.toml` with `0600` permissions (owner read/write only). Permissions are tightened before any secret content is written — even if the file was previously created with looser permissions, it is `chmod 600`'d atomically.

- Tokens are **never** written into the bundled opencode config directory.
- Tokens are **never** logged or exposed in error messages (only the last 4 characters are shown in `serval status`).
- Tokens are **never** committed to this repository.

## Network communication

| Endpoint | Purpose | Auth |
|---|---|---|
| Worker `/cli/config` | Fetch provider config | Bearer token |
| GitHub Releases API | Self-update | None |
| GitHub Releases CDN | Download update archive | None |

All HTTP requests use 5-second connect and 20-second read timeouts to prevent hanging on degraded networks.

The `Authorization: Bearer` header is **only** sent to the ServalAI Worker — never to GitHub or any other endpoint. Requests to GitHub's API omit the header entirely (an empty bearer triggers a 401 on GitHub's CDN).

## Supply chain

- `serval` is built with `cargo-zigbuild` for Linux musl targets (no glibc dependency, works on Alpine and any musl-based distro).
- The bundled opencode binary is pinned to a specific upstream release and fetched at build time from `anomalyco/opencode`.
- macOS binaries are cleared of quarantine attributes by the installer (`xattr -dr com.apple.quarantine`).
- No checksums or signatures are verified on the opencode download during the release build — this is a known limitation tracked for a future release.

## Environment injection

At launch, `serval` sets three environment variables for the opencode process:

- `OPENCODE_CONFIG_DIR` — points to the read-only bundle directory
- `OPENCODE_CONFIG_CONTENT` — inlined JSON with the provider block (including the user's token as `apiKey`)
- `CF_CLEVER_DEV_TOKEN` — the raw token for any direct-access paths

These are process-local environment variables — they are not exported to the shell, not written to any profile file, and not visible to other processes. `serval` uses `exec()` (replaces itself) rather than spawning a child process, so the vars exist only for the opencode process.

## Reporting a vulnerability

If you discover a security issue, please report it privately to the Cleverit security team. Do not open a public issue.
# ServalAI CLI (`serval`)

**One command to install. One token to start. AI coding backed by Cleverit's company-funded models.**

`serval` bundles a pinned [opencode](https://github.com/anomalyco/opencode) binary, a curated set of agents and skills, and your organization's AI gateway config — all injected at launch without touching your personal opencode setup. Your existing opencode configuration, agents, and skills are preserved and merged alongside.

---

## Quick start

```sh
curl -fsSL https://raw.githubusercontent.com/CleveritDemo/servalai-cli/main/install.sh | sh
```

Then:

```sh
serval auth          # paste your token from Mi Portal
serval               # start coding
```

That's it. No config files, no environment variables.

---

## Install

### One-liner (recommended)

```sh
curl -fsSL https://raw.githubusercontent.com/CleveritDemo/servalai-cli/main/install.sh | sh
```

The installer:
- Detects your OS and architecture automatically
- Downloads the latest release from GitHub
- Installs to `~/.local/share/serval/` and links `serval` into `~/.local/bin/`
- Clears macOS quarantine attributes so the binary runs without a Gatekeeper prompt

After install, make sure `~/.local/bin` is on your `PATH`. Add this to your shell config if needed:

```sh
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc   # or ~/.zshrc
```

### Manual download

Pick your platform from the [releases page](https://github.com/CleveritDemo/servalai-cli/releases), extract, and put `serval` and `opencode` somewhere on your `PATH`.

---

## Usage

| Command | What it does |
|---|---|
| `serval auth` | Store your ServalAI token. You'll be prompted to paste it. |
| `serval auth --token <t>` | Store a token without interactive prompting. |
| `serval` | Launch opencode preconfigured with ServalAI. |
| `serval code` | Same as above. |
| `serval code -- --print-logs` | Launch opencode with extra flags. |
| `serval pi` | Launch [oh-my-pi](https://github.com/anomalyco/oh-my-pi) preconfigured with ServalAI (bundled). |
| `serval aider` | Launch [aider](https://aider.chat) preconfigured with ServalAI (requires aider on your `PATH`). |
| `serval status` | Show version, bundled opencode, gateway URL, and identity. |
| `serval sync` | Refresh your provider config from the gateway. |
| `serval ping` | Check gateway connectivity and list your available models. |
| `serval models` | List available models for your account, with descriptions. |
| `serval usage` | Show token usage and session statistics. |
| `serval doctor` | Run diagnostics: config permissions, token, gateway reachability, bundled binaries. |
| `serval init` | Create a `.serval.jsonc` in the current directory to pin a model tier for that project. |
| `serval report` | Summarize recent activity: identity, working directory, opencode session count, available tools. |
| `serval update` | Self-update to the latest release. |
| `serval logout` | Clear your token from this machine. |

Run `serval --help` any time for the full, always-current list.

### Getting your token

Visit [Mi Portal](https://cleverit-support.cleveritgroup.com) to generate your ServalAI token.

### Model tiers

ServalAI exposes three model tiers. opencode auto-selects based on task complexity, and you can switch manually:

| Tier | Use for |
|---|---|
| `dynamic/power` | Hard design/architecture tasks, large refactors |
| `dynamic/balanced` | Everyday work (default) |
| `dynamic/light` | Quick, mechanical tasks |

---

## What's in the box

`serval` ships a read-only bundle that opencode loads via environment variables at launch:

```
~/.local/share/serval/current/
├── serval              native launcher binary (Rust)
├── opencode             pinned upstream opencode binary
└── bundle/
    ├── opencode.jsonc   ServalAI provider config (token injected at runtime)
    ├── AGENTS.md        curated agent roster + usage guide
    ├── agents/*.md      8 curated subagents
    ├── skills/…         29 curated skill guides
    └── mcp/…            4 pre-configured MCP servers
```

### Bundled agents

| Agent | Role |
|---|---|
| `architect` | System design, ADRs, C4 diagrams |
| `fullstack-lt` | Lead technologist: plans tasks, gates quality |
| `developer` | Test-first polyglot implementer |
| `code-review` | SOLID, test coverage, edge cases |
| `sec-ops-expert` | OWASP, secrets, RBAC, supply-chain |
| `ai-engineer` | Model selection, RAG, evals, agents |
| `senior-data-engineer` | Pipelines, lakehouses, data contracts |
| `senior-designer` | UX/product design, accessibility |

### Bundled MCP servers

| Server | Description | Needs config? |
|---|---|---|
| `context7` | Library docs and code examples | No |
| `github` | PRs, issues, repos, code search | `GITHUB_TOKEN` env var |
| `kubernetes` | Cluster access via `kubectl proxy` | `kubectl proxy` running |
| `sequential-thinking` | Structured problem reasoning | No |

---

## How it works

```
serval ──sets env vars──▶ opencode (bundled, pinned version)
            │
            ├── OPENCODE_CONFIG_DIR       → bundle/ (curated agents, skills, MCP)
            ├── OPENCODE_CONFIG_CONTENT   → provider config with token inlined
            └── CF_CLEVER_DEV_TOKEN       → token for direct access
```

- **Never touches your files.** opencode merges ServalAI's config with your existing `~/.config/opencode/` and project `.opencode/`. Your personal setup is preserved.
- **Degrades gracefully.** If the gateway is unreachable, `serval` falls back to cached config, then to an embedded default — you can still work. This shows as a muted `○ using ... config — ...` note, never as an error: the command still succeeded.
- **Self-updating.** `serval update` fetches the latest release and atomically swaps the install directory. Agents, skills, and MCP config stay current fleet-wide.

### What you'll see on launch

Plain `serval` prints its wordmark and an animated spinner while it contacts the gateway, so you always know it's working, not stuck:

```
ServalAI
a Cleverit company · powered by Raven

⠙ Connecting to gateway…
```

Every other gateway-touching command (`auth`, `sync`, `ping`, `models`, `usage`, `doctor`) shows the same spinner without the banner. All of this is fully inert — no animation, no color, no extra output — when `serval`'s output isn't a real terminal (piped, scripted, CI) or when `NO_COLOR` is set.

---

## Platform support

| Platform | Status |
|---|---|
| macOS (Apple Silicon) | ✅ |
| macOS (Intel) | ✅ |
| Linux (x86_64) | ✅ |
| Linux (aarch64) | ✅ |
| Windows (WSL2) | ✅ (via Linux build) |
| Windows (native) | Not yet |

---

## Troubleshooting

**`serval: command not found`**
Add `~/.local/bin` to your `PATH` and restart your terminal.

**`serval` seems to pause after the banner**
That's the spinner working, not a hang — `serval` is contacting the gateway (5-second connect / 20-second read timeout). If the gateway can't be reached at all, it falls back to cached or default config within a few seconds and still launches; run `serval doctor` to check gateway reachability directly.

**"you haven't authenticated yet"**
Run `serval auth` and paste your token from Mi Portal.

**macOS: "cannot be opened because the developer cannot be verified"**
The installer clears quarantine automatically. If you extracted manually, run:
```sh
xattr -dr com.apple.quarantine ~/.local/share/serval/current/
```

---

## Updating

```sh
serval update
```

This downloads the latest release and atomically swaps the install directory. Your token and cached config are preserved (they live in `~/.config/serval/`, not in the install directory).

---

## Uninstall

```sh
rm -rf ~/.local/share/serval ~/.config/serval ~/.local/bin/serval
```

---

## Security

See [SECURITY.md](SECURITY.md) for details on token storage, network communication, and reporting vulnerabilities.

---

## License

MIT — see [Cargo.toml](Cargo.toml).
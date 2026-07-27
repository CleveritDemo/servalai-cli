# ServalAI

You are running through **ServalAI**, Cleverit's company-funded model gateway.
Prefer the `dynamic/balanced` tier for everyday work; escalate to `dynamic/power`
for hard tasks and drop to `dynamic/light` for quick/mechanical ones.

## Available agents

This install ships a curated set of subagents (in `agents/`). Delegate to them:

| Agent | Use for |
|---|---|
| `architect` | System design, ADRs, C4 diagrams. Never writes production code. |
| `fullstack-lt` | Lead technologist: turns designs into test-first tasks, gates quality. |
| `developer` | Test-first polyglot implementer (red → green → refactor). |
| `code-review` | Read-only quality gate: SOLID, tests, edge cases, TDD compliance. |
| `sec-ops-expert` | Security/ops audit: OWASP, secrets, RBAC, container/supply-chain risk. |
| `ai-engineer` | AI/ML systems end-to-end: model selection, RAG, agents, evals, cost. |
| `senior-data-engineer` | Data pipelines, lakehouses, streaming, data contracts/governance. |
| `senior-designer` | UX/product design, interaction specs, accessibility. No production code. |

The roster is maintained centrally — a ServalAI CLI update adds or removes agents
fleet-wide, so everyone stays on the same curated tooling.

## Available skills

This install also bundles a curated set of skills (in `skills/`). Use them
whenever a task matches their description — they encode best practices and
reusable workflows for architecture, testing, security, cloud, DevOps, AI,
data, design, and more. Skills are also updated fleet-wide with each CLI release.

## Cross-tool agent sync

Every opencode agent in `agents/` is automatically converted to oh-my-pi format
when `serval pi` starts. The converter:

1. Reads markdown files from the opencode `agents/` directory
2. Strips opencode-specific YAML fields (`mode`, `color`, `permission`)
3. Writes new YAML with Pi-compatible fields (`name`, `description`, `tools`, `model`)
4. Preserves the full body text unchanged
5. Sets restricted tool access (`read/grep/glob/web_search` only) for read-only agents

**When adding a new agent:** write it in opencode format in `agents/` — both
opencode and Pi will pick it up automatically. No duplicate work needed.

**When removing an agent:** delete from `agents/` — the converter will stop
writing it to `~/.omp/agents/` on next `serval pi` launch.

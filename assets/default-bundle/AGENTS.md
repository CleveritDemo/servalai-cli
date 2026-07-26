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

---
name: test-master
description: TDD practitioner skill. Red/green/refactor discipline, test pyramid boundaries, fixture patterns, assertion quality, anti-flake patterns. Load when writing tests or reviewing TDD compliance.
metadata:
  audience: developer, code-review, fullstack-lt
---

# Test Master

## The TDD Cycle (non-negotiable)

```
🔴 RED        write the failing test that expresses the requirement
🟢 GREEN      minimum code to make it pass
♻️  REFACTOR   improve structure with tests staying green
```

Repeat per behavior. Never skip red.

## Anatomy of a Good Test

```
ARRANGE     set up state and dependencies (minimal)
ACT         the single behavior under test
ASSERT      one logical assertion (may be multiple statements)
```

Name pattern: `should_<behavior>_when_<condition>` or `<unit>_<scenario>_<expected>`.

## Test Pyramid (your default budget)

| Layer | What | When | Speed |
|---|---|---|---|
| **Unit** | Pure logic, single function/class | Most code | ms |
| **Integration** | Component + real dependency (DB, queue) | Service boundaries | 100ms-1s |
| **Contract** | API/event schema between services | Public contracts | ms |
| **E2E** | Whole user flow | Critical paths only | seconds |

Rough ratio: 70% unit / 20% integration / 10% e2e+contract. Adjust to risk.

## Anti-Flake Patterns

- **Time** → inject a clock; never `Date.now()` directly.
- **Random** → seed RNG; or use deterministic stubs.
- **Network** → mock at boundary; use VCR/wiremock/msw for integration.
- **Concurrency** → use synchronization primitives, not `sleep()`.
- **Filesystem** → tmp dirs scoped per test, cleaned in teardown.
- **Order dependence** → randomize test order in CI; fail if results change.

## Coverage Is a Symptom, Not a Goal

- Aim to test **behaviors**, not lines.
- High coverage with weak assertions is worse than lower coverage with strong assertions.
- Mutation testing > line coverage (where the toolchain supports it).

## Assertion Quality Heuristics

| Bad | Good |
|---|---|
| `expect(result).toBeTruthy()` | `expect(result).toEqual({ id: 'x', status: 'ok' })` |
| `expect(fn).not.toThrow()` | `expect(fn()).toEqual(expected)` |
| `expect(arr.length).toBe(3)` | `expect(arr).toEqual([a, b, c])` |
| `assert err == nil` | `assert err == nil; assert result.X == expectedX` |

Assert the **shape** and **content**, not just existence.

## Fixtures > Setup Soup

- Factory functions with sensible defaults > giant `beforeEach`.
- Object mothers for domain objects.
- Builder pattern for complex aggregates.
- Avoid shared mutable state across tests.

## TDD Compliance Check (for code-review)

When reviewing a diff, verify:

1. **Test file exists** for the new logic.
2. **Test fails without the impl** — reason about test specificity.
3. **Commit history** (if available): test commit before impl commit, or same commit.
4. **Edge cases covered**: null, empty, max, min, boundary, error path.
5. **Negative tests** present: what *shouldn't* happen.

If TDD is not evident, the verdict is BLOCKED with a finding.

## Reference: Language Conventions

- **Python**: pytest, fixtures, `monkeypatch`, `pytest-mock`, hypothesis for properties.
- **TypeScript/Node**: vitest or jest, `vi.mock`/`jest.mock`, msw for HTTP, fast-check for properties.
- **Go**: stdlib `testing`, table-driven tests, `t.Run` subtests, `httptest` for handlers.
- **Rust**: `#[cfg(test)] mod tests`, `assert_eq!`, `proptest`/`quickcheck` for properties.
- **Bash**: bats or shunit2. Mock with `function` overrides scoped per test.

## Hard Rules

- No test added "later". The test is part of the change or there is no change.
- No commented-out tests. Delete or fix.
- No `expect(true).toBe(true)`. No tautological tests.
- No tests calling network/disk/clock without isolation.
- No `setTimeout`/`sleep` to "fix" race conditions.

## Output (when invoked by developer)

After writing tests + impl:

```
## Tests Added
- <file>:<lines> — <what behavior>

## TDD Evidence
- 🔴 Red: <command run, expected failure reason>
- 🟢 Green: <command run, N/N passing>

## Coverage Reasoning
<what behaviors are covered; what is intentionally out of scope>
```

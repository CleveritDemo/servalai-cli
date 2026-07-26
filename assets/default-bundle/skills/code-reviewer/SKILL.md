---
name: code-reviewer
description: Code review heuristics. SOLID, naming, complexity, error handling, performance hot paths, test quality. Load when reviewing diffs or auditing code health.
metadata:
  audience: code-review, fullstack-lt, developer
---

# Code Reviewer

## When to Use

- Reviewing a diff / PR / branch
- Auditing a module before refactor
- Pair-reviewing the developer's self-output

## Triggers

code review, pr review, refactor, solid, naming, complexity, technical debt, readability, maintainability

## The Review Lens (in order)

1. **Correctness** — does it do the thing? Edge cases? Off-by-one?
2. **Tests** — meaningful, isolated, failing-first evidence?
3. **Design** — SOLID, layering, coupling, cohesion?
4. **Naming** — intention-revealing, consistent?
5. **Error handling** — specific, contextual, no swallowing?
6. **Performance** — obvious hot paths, N+1, allocation in loops?
7. **Security** — surface-level scan; flag for sec-ops-expert if deep concern?
8. **Maintainability** — could a stranger fix a bug here in 6 months?

Each level only matters if the previous passes.

## SOLID Cheatsheet

| Principle | Smell |
|---|---|
| **S**ingle Responsibility | "And" in the class/function description |
| **O**pen-Closed | Modifying existing code for new variant; should extend |
| **L**iskov Substitution | Subclass throws or changes contract |
| **I**nterface Segregation | Class implements methods it doesn't need |
| **D**ependency Inversion | High-level depends on low-level concrete; should depend on abstraction |

SOLID is a heuristic, not a religion. Sometimes ignoring it is the right call — but say so explicitly.

## Naming

- **Functions are verbs**: `calculate_tax`, `is_valid`, `find_user_by_email`.
- **Variables are nouns**: `user_count`, not `count_users`.
- **Booleans are predicates**: `is_active`, `has_permission`, `can_delete`.
- **Constants are SCREAMING_SNAKE_CASE** in most langs.
- **No abbreviations** except domain-standard ones (`url`, `id`, `db`).
- **Plural collection, singular item**: `users`, `user`.
- **Consistent**: don't mix `fetch` / `get` / `retrieve` for the same operation.

## Function Smells

- More than 3-4 parameters (consider an object).
- Boolean flag parameter (split into two functions).
- Output parameter mutated by ref (return instead).
- Returns different types based on input (split).
- > 50 lines (probably doing too much).
- Cyclomatic complexity > 10 (extract helpers, use early returns).

## Class / Module Smells

- "And", "Or", "Manager", "Helper", "Util" in the name.
- > 7 public methods (probably two classes hiding).
- Most methods don't use most fields (low cohesion).
- Anemic — only getters/setters, no behavior.
- God class touching everything.

## Error Handling

- **Catch specific exceptions**, not `Exception`/`Throwable`.
- **Add context** when rewrapping (`raise X from e`).
- **Don't swallow** unless explicitly intentional and documented.
- **Map at the boundary** — internal errors → user-facing codes at controllers.
- **Don't use exceptions for flow control**.
- **Return errors** in Go/Rust idiomatically — don't `panic`/`unwrap` in libraries.

## Performance Hot Paths

Things to flag in a review (not optimize blindly):

- **N+1 queries** in loops.
- **Sorts/searches inside loops** of independent data.
- **Allocations inside tight loops** in perf-sensitive code.
- **String concatenation in loops** (use builders).
- **Synchronous I/O on event loops** in async runtimes.
- **Unbounded data structures** that grow indefinitely.
- **Repeated regex compilation** (compile once).

When in doubt: profile first, optimize second. But flag the suspicions.

## Test Quality

(See `test-master` skill for full TDD coverage.)

Quick review questions:

- Does the test fail without the impl?
- Is one behavior tested per test?
- Are assertions specific (shape + value, not just truthy)?
- Are fixtures isolated?
- No `sleep()` for timing.
- Negative cases covered.

## Comments

- **Why, not what.** The code shows what. Comments explain why.
- **TODO** without owner + tracking issue = MINOR finding.
- **Outdated comments** that contradict code = MAJOR.
- **Commented-out code** = delete it; git remembers.

## Diff Hygiene

- One logical change per commit.
- Reformatting separated from logic changes.
- Tests committed with (or before) impl.
- Commit message explains the why.

## Output Format

```
## Verdict: PASS | BLOCKED

## Summary
<2-3 sentences>

## BLOCKERS
- [file:line] <issue> — <why blocks> — <suggested fix>

## MAJOR
- [file:line] <issue> — <suggested fix>

## MINOR
- [file:line] <issue>

## NITS
- [file:line] <issue>

## TDD Compliance
- Tests-first evidence: <yes/no/uncertain>
- Coverage: <adequate / gap at X>
- Test quality: <notes>

## Strengths
<what was done well — keep morale honest>
```

## Tone

- Critique the code, not the person.
- Suggest, don't dictate, when style is involved.
- Be specific — vague feedback is unfixable.
- Praise what's good. Reviews aren't only for finding fault.

## References

- *Code Complete* — McConnell
- *Refactoring* — Fowler
- *Clean Code* — Martin (with critical eye)
- *A Philosophy of Software Design* — Ousterhout

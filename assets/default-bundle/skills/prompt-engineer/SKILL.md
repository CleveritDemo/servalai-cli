---
name: prompt-engineer
description: Prompt design heuristics for LLM agents. Structure, role, constraints, output format, tool use, eval. Load when authoring or reviewing prompts for AI agents (e.g. llmapps-main agents).
metadata:
  audience: fullstack-lt, developer, code-review
---

# Prompt Engineer

## When to Use

- Writing or reviewing a system prompt for an LLM agent
- Designing tool-use / function-calling schemas
- Reducing hallucination or brittleness
- Auditing prompt quality and safety
- Working on `llmapps-main` AI agent prompts

## Triggers

prompt, llm, agent, system prompt, instruction, few-shot, chain of thought, tool use, function calling, structured output, json mode, eval, hallucination, jailbreak

## Anatomy of a Strong Prompt

```
1. ROLE / IDENTITY     — who/what the model is
2. TASK                 — the specific job, unambiguous
3. CONTEXT              — facts, retrieved data, conversation history
4. CONSTRAINTS          — what must / must not be done
5. OUTPUT FORMAT        — schema, examples, length
6. TOOLS (if any)       — when to use them, when not to
7. EXAMPLES (few-shot)  — input/output pairs (optional, expensive)
```

Order matters. Models attend more to the start and end (lost-in-the-middle).

## Heuristics That Work

- **Be specific**. "Summarize in 3 bullets, each ≤ 15 words" >> "summarize briefly".
- **Show, don't tell**. Few-shot examples often beat long instructions.
- **Negative space matters**. Tell it what *not* to do, especially for safety.
- **Force structure**. JSON output with explicit schema beats prose for downstream parsing.
- **Use delimiters**. Triple backticks, XML tags (`<context>...</context>`), or `###` to separate sections.
- **Repeat the critical constraint** at start and end of long prompts.
- **Provide an escape hatch**. "If you don't have enough information, say: I don't know."

## Anti-Patterns

- "You are a helpful assistant." — meaningless, wastes tokens.
- Stacking 20 unrelated rules — model picks favorites.
- Vague output requirements ("be detailed") — produces noise.
- Mixing instructions and content without delimiters — model confuses them.
- No example of bad output — model can't recognize what to avoid.
- Forgetting the "I don't know" path — hallucinations fill the gap.
- "Important!" / "MUST!" / ALL CAPS spam — diminishing returns; pick one signal.

## Tool / Function Calling

- Define tool schemas with **precise types** and **clear descriptions**.
- Each tool description should answer: *what does it do, when to use it, what does it return?*
- **Discriminate similar tools** — if two could apply, the description must clarify.
- Tell the model: *"Use tool X for Y; never call X for Z."*
- For chained tool calls, give an example of the sequence.

```
Use search_docs when the user asks for product information.
Do not use search_docs for general knowledge questions.
After search_docs, always cite the source in your response.
```

## Structured Output

Prefer JSON mode / function calls over free-text parsing.

```json
{
  "decision": "approve" | "reject" | "needs_info",
  "reasons": ["..."],
  "confidence": 0.0-1.0
}
```

Validate with a schema (Pydantic / Zod / JSON Schema). Reject + retry on schema failure.

## Reducing Hallucination

1. **Ground in retrieval** (RAG). Force citations.
2. **Force abstention**: "If unsure, say 'I don't know'."
3. **Lower temperature** for factual tasks (0.0-0.3).
4. **Decompose** — break complex tasks into steps, each verifiable.
5. **Self-critique pass** — second model call to check the first.
6. **External validation** — code runs, schemas validate, retrieved facts match.

## Chain of Thought / Reasoning

- For complex tasks, ask the model to think step-by-step before answering.
- For modern reasoning models (o1, claude-thinking, etc.), let the model handle reasoning natively; over-prompting reasoning is wasteful.
- For non-reasoning models on complex tasks: `Let's work through this step by step.`
- Hide reasoning from end users unless they want it; surface only the conclusion.

## Safety / Robustness

- **Jailbreak resistance**: separate system prompt from user content with delimiters; never let user text reset the role.
- **PII**: instruct not to echo PII; sanitize input before/after.
- **Tool authorization**: don't trust the model to enforce permissions — enforce at the tool layer.
- **Output filtering**: scan responses for prompt-injection echo, secrets, PII.

## Versioning & Eval

- Treat prompts like code. **Version them**. Diff them.
- Build a **golden set** of inputs with expected behavior.
- On every prompt change, run the eval. No regression → ship. Regression → fix.
- Track metrics: success rate, refusal rate, hallucination rate, latency, cost.
- **Champion/challenger**: A/B new prompts against current in production.

## Prompt Compression / Efficiency

- Audit each line: does it change behavior? If not, cut it.
- Use system prompt for stable instructions, user prompt for variable parts.
- Cache prefixes when supported (Anthropic prompt caching, OpenAI prefix caching).
- Smaller prompt = lower latency, lower cost, often higher quality.

## Output Template (for prompt review)

```
## Prompt Reviewed
<path or excerpt>

## Findings
### Structure
- Role: <present/missing/weak>
- Task: <clear/ambiguous>
- Output format: <specified/implicit>
- Constraints: <list>
- Escape hatch: <yes/no>

### Issues
- BLOCKER: <e.g. no abstention path, leaks PII>
- MAJOR: <e.g. vague output spec>
- MINOR: <e.g. wordy>

### Suggested Rewrite
<diff or full new prompt>

### Eval Plan
- Cases to verify: <list>
```

## References

- Anthropic prompt engineering guide: https://docs.anthropic.com/claude/docs/prompt-engineering
- OpenAI prompt engineering guide
- *Prompt Engineering Guide* by DAIR.AI

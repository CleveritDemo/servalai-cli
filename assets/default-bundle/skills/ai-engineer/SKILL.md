---
name: ai-engineer
description: AI/ML engineering heuristics. Model selection, fine-tuning, inference, RAG, agents, evaluation, AI safety, and observability. Load when designing or implementing AI-powered features, evaluating model trade-offs, or auditing AI system quality.
metadata:
  audience: ai-engineer, architect, fullstack-lt, developer
---

# AI Engineer

## When to Use

- Selecting a model or embedding for a specific use case
- Designing a RAG, agent, or LLM-powered feature
- Defining an evaluation harness for AI components
- Reviewing AI code for correctness, cost efficiency, or safety
- Implementing prompt engineering with production rigour
- Fine-tuning or adapting a foundation model
- Designing AI observability (token cost, latency, drift)
- Auditing AI systems for prompt injection or PII leakage

## Triggers

llm, ai, ml, model, embedding, rag, agent, fine-tuning, inference, evaluation, prompt, openai, anthropic, claude, gpt, gemini, llama, mistral, vector, semantic search, hallucination, context window, token, guardrail, langchain, langgraph, llamaindex, huggingface, transformers, bert, eval, benchmark, mlops

## Core Workflow

1. **Define measurable success** — before any AI work: what is the metric, what is the threshold, what is the budget (latency, cost)?
2. **Build vs. buy vs. fine-tune decision** — use the framework below.
3. **Eval harness first** — write the evaluation tests before the implementation.
4. **Implement with observability** — every AI call is traced.
5. **Benchmark and gate** — run eval; ensure result meets threshold before declaring done.
6. **Security review** — prompt injection surface, PII exposure, output validation.

## Build vs Buy vs Fine-tune Framework

| Question | → |
|---|---|
| Does a foundation model solve this out of the box? | Buy (API) |
| Does the task need proprietary data not in training? | RAG first |
| Does RAG not achieve target quality? | Fine-tune |
| Is latency or cost the constraint (not quality)? | Smaller model / quantisation |
| Is the task truly standard (classification, NER)? | Traditional ML first |

**Rule:** Reach for the simplest solution first. RAG before fine-tuning. Fine-tuning before training from scratch. Traditional ML before LLMs for structured tasks.

## Model Selection Heuristics

| Requirement | Model Class |
|---|---|
| Complex reasoning, long context | Claude Sonnet/Opus, GPT-4o, Gemini 1.5 Pro |
| Speed + quality balanced | Claude Haiku, GPT-4o-mini, Gemini Flash |
| On-premise / private data | Llama 3.x, Mistral, Qwen2.5 (via Ollama/vLLM) |
| Code generation | Claude Sonnet, GPT-4o, Qwen2.5-Coder, DeepSeek-Coder |
| Embeddings (high quality) | text-embedding-3-large, mxbai-embed-large, BGE-M3 |
| Embeddings (speed/cost) | text-embedding-3-small, nomic-embed |
| Reranking | cross-encoder/ms-marco-MiniLM, Cohere Rerank |
| Image understanding | GPT-4o-vision, Claude 3.5, Gemini 1.5 Pro |

## RAG Architecture Checklist

```
[ ] Chunking strategy defined (size, overlap, method: fixed/semantic/sentence)
[ ] Embedding model selected and pinned to version
[ ] Vector store selected (scale, latency, filtering needs)
[ ] Sparse retrieval considered (BM25 hybrid for recall)
[ ] Reranker in pipeline for precision
[ ] Metadata filters designed (tenant, date, document type)
[ ] Context window budget allocated (system + retrieved + query + response)
[ ] Faithfulness metric defined (grounding check in eval)
[ ] Retrieval eval metrics: MRR@K, NDCG@K, Recall@K
[ ] End-to-end eval: faithfulness, answer relevance, context precision
```

## Evaluation Patterns

### LLM-as-Judge (use with care)
```python
# Only for tasks where human eval is too expensive
# Always: use a different model as judge than as generator
# Always: use a rubric, not "is this good?"
# Always: validate judge calibration with a human-labelled gold set
eval_prompt = """
Rate the following response on faithfulness to the context (1-5):
Context: {context}
Response: {response}
Score (1=hallucinated, 5=fully grounded):
"""
```

### Property-Based AI Tests
```python
# AI outputs are probabilistic — test properties, not exact strings
def test_summary_shorter_than_source():
    result = summarise(long_doc)
    assert len(result) < len(long_doc) * 0.5

def test_sentiment_consistency():
    for _ in range(10):  # run multiple times
        result = classify_sentiment(clearly_positive_text)
        assert result in ["positive", "very positive"]
        # never: assert result == "positive"
```

### Regression Eval
```
- Maintain a labelled eval set of 50–200 representative examples
- Run eval on every model upgrade or prompt change
- Gate merges on eval score ≥ baseline - tolerance
- Track eval score over time; alert on degradation
```

## Prompt Engineering Standards

### Structure Template
```
[SYSTEM: Role + constraints + output format]

[CONTEXT: Relevant background — grounding]

[TASK: Single, unambiguous instruction]

[EXAMPLES: 2-3 few-shot examples if complex]

[INPUT: {user_input}]

[OUTPUT FORMAT: JSON schema / markdown structure]
```

### Production Prompt Rules
- **Pin the model version** — never use `gpt-4` when `gpt-4o-2024-08-06` is available.
- **Version your prompts** — treat prompt strings as code; track changes, test regressions.
- **Separate concerns** — system prompt describes role + constraints; user prompt contains task + input.
- **Escape injection** — never interpolate raw user input into system prompt.
- **Define output format** — structured output (JSON mode / function calling) wherever possible.
- **Set temperature deliberately** — 0.0 for deterministic tasks; 0.2–0.4 for creative tasks; never default.

## AI Security Checklist

```
[ ] User input never directly interpolated into system prompt
[ ] Prompt injection mitigated (input sanitisation, instruction hierarchy)
[ ] PII not logged in traces (token logs, LangSmith, etc.)
[ ] Output validated before passing to downstream systems (command execution, DB writes)
[ ] Model outputs never directly rendered as HTML (XSS via AI)
[ ] Jailbreak/abuse patterns tested in red-team eval
[ ] Rate limiting on AI endpoints (cost protection)
[ ] Model endpoints authenticated (no public unauthenticated AI proxy)
[ ] Secrets never in prompts or context (API keys, passwords)
[ ] System prompt confidentiality: never reveal on request
```

## AI Observability Stack

| Signal | What to capture | Tool |
|---|---|---|
| **Traces** | Prompt, response, model, latency, token count, cost | LangSmith, Weave, OTEL |
| **Metrics** | Requests/min, p50/p95 latency, token/req, cost/req, error rate | Prometheus + Grafana |
| **Eval scores** | Faithfulness, relevance, latency regression | Scheduled eval pipeline |
| **Drift** | Output distribution shift, quality degradation over time | Custom eval + alerting |

## Inference Optimisation

| Technique | When | Trade-off |
|---|---|---|
| **Quantisation (AWQ/GGUF)** | On-premise, latency/cost bound | Small quality loss, 2–4× speedup |
| **Batching** | Throughput > latency | Higher latency per request |
| **KV cache** | Repeated system prompts | Memory cost |
| **Speculative decoding** | High-quality, speed needed | Complexity |
| **Smaller model + routing** | Mixed task complexity | Routing logic overhead |
| **Streaming** | UX: perceived responsiveness | Client-side complexity |

## Output Template

```
## AI Requirement
- Task: <what the AI must do>
- Success metric: <precision/recall/latency/cost threshold>
- Error tolerance: <acceptable failure rate>

## Decision
- Approach: <API / RAG / fine-tuning / traditional ML>
- Model: <pinned version>
- Rationale: <tied to success metric>

## Eval Harness
- Dataset: <source, size>
- Metrics: <list with thresholds>
- Pass criteria: <score ≥ X>

## Architecture / Prompt Design
<component diagram or prompt structure>

## Cost Estimate
- Tokens per request: ~<N> input / <N> output
- Cost per 1K requests: $<N> at <model pricing>
- Monthly estimate at <volume>: $<N>

## Risks
- Hallucination surface: <description, mitigation>
- Prompt injection surface: <description, mitigation>
- PII exposure: <yes/no, handling>
```

## References

- RAGAS evaluation: https://docs.ragas.io/
- OpenAI evals: https://github.com/openai/evals
- HuggingFace MTEB: https://huggingface.co/spaces/mteb/leaderboard
- LangSmith: https://www.langchain.com/langsmith
- AI Safety via OWASP LLM Top 10: https://owasp.org/www-project-top-10-for-large-language-model-applications/
- Anthropic Responsible Scaling Policy
- Building LLMs for Production (Maxime Labonne)

---
name: rag-architect
description: Retrieval-Augmented Generation architecture. Embedding choice, chunking, vector stores, hybrid search, reranking, evaluation. Load when designing or reviewing RAG pipelines (e.g. in llmapps-main).
metadata:
  audience: architect, fullstack-lt, developer
---

# RAG Architect

## When to Use

- Designing a RAG pipeline
- Reviewing retrieval quality issues
- Choosing embedding model / vector store
- Adding reranking, hybrid search, or evaluation
- Working in `llmapps-main` AI agent paths

## Triggers

rag, retrieval, embedding, vector, similarity, semantic search, chunk, chunking, reranker, rerank, hybrid search, bm25, llm context, hallucination, ground truth

## RAG Pipeline Anatomy

```
INGEST                          QUERY
  source docs                     user query
    │                               │
    ├─ parse / clean                ├─ rewrite (optional)
    ├─ chunk                        │
    ├─ enrich (metadata)            ├─ embed
    ├─ embed                        │
    ├─ index (vector + lexical)     ├─ retrieve (k=20-50)
    │                               │
                                    ├─ rerank (top k=5-10)
                                    ├─ assemble context
                                    └─ generate (LLM)
```

## Chunking Strategy

| Strategy | When |
|---|---|
| **Fixed-size (e.g. 500 tokens)** | Default; simple |
| **Sentence/paragraph** | Natural prose; preserves semantics |
| **Recursive (by structure)** | Markdown/code; respects headings/blocks |
| **Semantic (split by embedding similarity)** | Heterogeneous content; more expensive |

- **Overlap**: 10-20% to avoid splitting context at boundaries.
- **Chunk size**: trade-off — small (precise but fragmented), large (context but dilute).
- **Metadata** on every chunk: source, page/section, timestamp, version. Used for filtering.

## Embedding Choice

| Model class | Use |
|---|---|
| **General multilingual** (e.g. `multilingual-e5-large`, OpenAI `text-embedding-3-*`) | Default |
| **Domain-specific** (legal, biomedical) | Specialized corpora |
| **Code embeddings** | Code search (`voyage-code`, `nomic-embed-code`) |
| **Small/local** (`bge-small`, `nomic-embed-text`) | Cost / latency / privacy |

- Match query and document embeddings — same model.
- Re-embed when you upgrade the model (and re-index).
- Pin model version explicitly. Embeddings are not interchangeable.

## Vector Store Choice

| Store | Strengths | When |
|---|---|---|
| **pgvector** | SQL co-located | Postgres-centric stack |
| **Qdrant** | Fast, filters, hybrid built-in | Most production needs |
| **Weaviate** | Schema, modules | Heavy filtering, multi-modal |
| **Milvus** | Scale | >100M vectors |
| **OpenSearch / Elasticsearch** | Lexical + vector | Hybrid first-class |
| **Pinecone** | Managed, simple | Vendor-locked OK |

For Pulzen (Postgres present): **pgvector** is the pragmatic default unless scale dictates otherwise.

## Hybrid Search

Combine **dense** (embeddings, semantic) + **sparse** (BM25, lexical):
- Sparse catches exact terms (IDs, names, jargon).
- Dense catches paraphrase and intent.
- Fuse with **Reciprocal Rank Fusion (RRF)** or weighted score.

```
final_score(d) = α * dense_rank(d) + (1-α) * sparse_rank(d)
```

Tune α on real queries.

## Reranking

After retrieval, rerank top-K with a cross-encoder:
- **Cross-encoder** (e.g. `bge-reranker-large`, Cohere Rerank) — order top 20 → top 5.
- Adds latency (50-300ms) but big quality lift.
- Especially valuable when retrieval recall is good but precision needs work.

## Context Assembly

- Order chunks by relevance, then by source coherence.
- **Citations** in context: include source IDs the LLM can reference.
- **Budget tokens** — leave room for the question and response.
- **Deduplicate near-identical chunks** before sending.

## Prompt Patterns

```
You answer using ONLY the provided context.
If the context does not contain the answer, say "I don't know".
Cite sources by their [id].

Context:
[1] <chunk text>
[2] <chunk text>
...

Question: <user query>
```

- Hard refusal on missing context to reduce hallucination.
- Force citations to ground the answer.
- Don't tell the model what role it is; tell it what to do.

## Evaluation

You cannot improve what you don't measure. Build an eval set early.

### Retrieval metrics
- **Recall@k**: did we retrieve any relevant doc in top k?
- **MRR**: rank of first relevant doc.
- **nDCG**: graded relevance.

### Generation metrics
- **Faithfulness**: answer supported by retrieved context.
- **Answer relevance**: does it answer the question?
- **Context precision**: % of retrieved chunks actually used.

### Build the eval set
- Curate 50-200 real questions with known good answers.
- Mix easy / hard / multi-hop / out-of-scope.
- Re-run on every model or chunking change.

Tools: Ragas, TruLens, DeepEval, custom LLM-as-judge.

## Common Failure Modes

| Symptom | Likely cause |
|---|---|
| "I don't know" too often | Recall low; tune k, chunk size, hybrid |
| Hallucinations with citations | Faithfulness loss; tighter prompts, reranker |
| Slow queries | No filter pre-pass; index missing; chunks too small (too many vectors) |
| Stale answers | No update on source; missing freshness in metadata/filter |
| Bad on jargon/IDs | Pure dense; add sparse hybrid |
| Bad on synonyms | Pure sparse; add dense |
| Mixed quality across sources | Skip source ranking; tag chunks by trust level |

## Cost / Latency Levers

- **Caching**: cache query → answer for FAQ patterns. Hit rates often >30%.
- **Sparse-only fast path** for short keyword queries.
- **Smaller embedder + reranker** beats huge embedder alone.
- **Async ingestion**; query path must be fast.
- **Filter by metadata first** to shrink the vector search space.

## Output Template

```
## Pipeline
- Source: <where docs come from>
- Parse: <library>
- Chunking: <strategy, size, overlap>
- Embedder: <model, version>
- Vector store: <name>
- Hybrid: <yes/no, fusion>
- Reranker: <model> | none

## Retrieval Params
- k_retrieve: N
- k_rerank: M
- Filters: <metadata fields>

## Generation
- LLM: <name>
- Prompt: <link or excerpt>
- Citations: <yes/no>

## Eval
- Dataset: <where, size>
- Metrics tracked: <list>
- Current baseline: <numbers>
```

## References

- *Retrieval-Augmented Generation for Large Language Models: A Survey* (2023)
- Ragas: https://docs.ragas.io
- LlamaIndex / LangChain docs (read selectively)
- MTEB leaderboard for embeddings: https://huggingface.co/spaces/mteb/leaderboard

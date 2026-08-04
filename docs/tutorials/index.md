# VantaDB Tutorials

Step-by-step guides for using VantaDB, ordered by increasing complexity.

## Learning Path

| Step | Tutorial | What you'll learn |
|------|----------|-------------------|
| 1 | [Building AI Agent Memory](01-ai-agent-memory.md) | Namespaces, `put()`, `search_memory()`, metadata filters, agent memory REPL |
| 2 | [Local RAG Pipeline](02-local-rag-pipeline.md) | Chunking, embedding, ingestion, retrieval-augmented generation with Ollama |
| 3 | [Hybrid Search](04-hybrid-search-basics.md) | Vector-only, BM25 (`text_query`), hybrid fusion, filters, tuning |
| 4 | [Embedding Providers](05-embedding-integrations.md) | OpenAI, Ollama, LiteLLM, and deterministic fallbacks — the BYO-vector model |

### Migration track

Already using another embedded vector database? Jump in here:

| Step | Tutorial | What you'll learn |
|------|----------|-------------------|
| M1 | [Migrating from ChromaDB](03-migrating-from-chromadb.md) | API mappings + one-command migration from ChromaDB |
| M2 | [Migrating from LanceDB](migration-from-lancedb.md) | Schema mapping, filters, and migration from LanceDB |
| M3 | [Migrating from Vectara](migrate-from-vectara.md) | Export Vectara corpora, convert to VantaDB format, re-embed |

## Prerequisites

- Python 3.11+ and `pip install vantadb-py`
- A basic understanding of vector embeddings and LLM APIs

## Suggested order

Start with **Building AI Agent Memory** — it covers the two primitives used everywhere (`put()` and `search_memory()`). Then build a **Local RAG Pipeline** on top. Once you're comfortable, the **Hybrid Search** and **Embedding Providers** tutorials cover production concerns: ranking quality, filter scoping, and provider choice.

The migration tutorials are standalone — read them when you have existing ChromaDB, LanceDB, or Vectara data to move.

## API reference

The full Python SDK reference lives at [Python SDK](../api/PYTHON_SDK.md).

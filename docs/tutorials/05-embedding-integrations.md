---
title: "Embedding Providers for VantaDB"
status: active
tags: [vantadb, tutorial, guide, embeddings, openai, ollama, litellm]
last_reviewed: 2026-08-02
aliases: []
---

# Embedding Providers for VantaDB

VantaDB is **BYO-vector**: it does not embed text for you. You compute embeddings with whatever model you like — OpenAI, Ollama, LiteLLM, a local transformer, or a deterministic hash — and pass the vectors to `put()` and `search_memory()`. This keeps VantaDB provider-agnostic and lets you switch embedding models without touching your data.

This tutorial shows the four integration patterns that matter in practice, all against the same namespace so you can mix and match.

## The pattern

Every provider integration is the same shape:

1. **Write:** `db.put(ns, key, payload, metadata={...}, vector=embed(text))`
2. **Read:** `db.search_memory(ns, embed(query), top_k=5)`

Only `embed()` changes between providers.

## 1. OpenAI (hosted)

```bash
pip install vantadb-py openai
```

```python
# vanta-skip: requires OPENAI_API_KEY — embed() calls the OpenAI embeddings API
from vantadb_py import VantaDB
import openai

db = VantaDB("app.db")

def embed(text: str) -> list[float]:
    resp = openai.embeddings.create(model="text-embedding-3-small", input=text)
    return resp.data[0].embedding

db.put("notes", "n1", "VantaDB fuses BM25 with HNSW vector search.",
       vector=embed("VantaDB fuses BM25 with HNSW vector search."))

hits = db.search_memory("notes", embed("hybrid search"), top_k=3)
```

Set `OPENAI_API_KEY` in the environment or configure the client as usual.

## 2. Ollama (local, offline)

```bash
pip install vantadb-py ollama
ollama pull nomic-embed-text
```

```python
# vanta-skip: requires a running Ollama service with nomic-embed-text pulled
from vantadb_py import VantaDB
import ollama

db = VantaDB("app.db")

def embed(text: str) -> list[float]:
    return ollama.embeddings(model="nomic-embed-text", prompt=text)["embedding"]

db.put("notes", "n1", "VantaDB fuses BM25 with HNSW vector search.",
       vector=embed("VantaDB fuses BM25 with HNSW vector search."))

hits = db.search_memory("notes", embed("hybrid search"), top_k=3)
```

Everything stays on your machine — no API keys, no network calls.

## 3. LiteLLM (provider router)

```bash
pip install vantadb-py litellm
```

```python
from vantadb_py import VantaDB
import litellm

db = VantaDB("app.db")

def embed(text: str) -> list[float]:
    resp = litellm.embedding(model="text-embedding-3-small", input=[text])
    return resp.data[0]["embedding"]

db.put("notes", "n1", "VantaDB fuses BM25 with HNSW vector search.",
       vector=embed("VantaDB fuses BM25 with HNSW vector search."))

hits = db.search_memory("notes", embed("hybrid search"), top_k=3)
```

LiteLLM routes to any of its 100+ providers behind one call — swap `model="ollama/nomic-embed-text"` to go local without changing your code.

## 4. Deterministic fallback (offline demos & tests)

When you need runnable examples without network access (tests, CI, tutorials), a stable hash embedding keeps vectors deterministic across processes:

```python
from vantadb_py import VantaDB
import hashlib

db = VantaDB(":memory:", backend="memory")

def embed(text: str, dim: int = 8) -> list[float]:
    out = []
    for i in range(dim):
        h = hashlib.sha256(f"{text}:{i}".encode()).digest()
        out.append(int.from_bytes(h[:4], "big") / 2**32)
    return out

db.put("notes", "n1", "VantaDB fuses BM25 with HNSW vector search.",
       vector=embed("VantaDB fuses BM25 with HNSW vector search."))
db.put("notes", "n2", "Graph edges build knowledge graphs over records.",
       vector=embed("Graph edges build knowledge graphs over records."))

for h in db.search_memory("notes", embed("vector search"), top_k=2):
    print(h.key, h.score)
```

> **Not for production ranking** — semantic quality is poor. Use it for tests and documentation examples where determinism matters more than accuracy.

## Consistency rules

1. **One model per namespace.** A HNSW index compares vectors in one metric space — mixing a 1536-dim OpenAI embedding with a 768-dim Ollama embedding in the same namespace breaks retrieval. Use separate namespaces per model if you must mix.
2. **Same dimension per write.** The index is built for a fixed dimensionality; consistent `embed()` output is required.
3. **Embed the query the same way as the documents.** `search_memory` embeds nothing for you — the query vector must come from the same model as the stored vectors.

## Managing cost and latency

- **Batch embeddings:** embed all chunks up front, then load with `put_batch()` — one call to VantaDB for thousands of records:

```python
texts = [f"Document {i} about hybrid vector search." for i in range(100)]
embedded_vectors = [[0.1] * 8 for _ in texts]   # from your embed() model
metadatas = [{"index": str(i), "source": "demo"} for i in range(len(texts))]

db.put_batch(
    None,                       # entries is a required positional arg
    keys=[f"doc-{i}" for i in range(len(texts))],
    vectors=embedded_vectors,
    payloads=texts,
    metadatas=metadatas,
    namespace="notes",
)
print(f"Batch-inserted {len(texts)} records")
```

Note: `metadatas` values must be strings (`str`) — convert numbers with `str(i)`.
- **Cache embeddings:** store the vector alongside the payload so re-ingestion doesn't re-embed.
- **Persist locally:** Ollama runs on-device; OpenAI and LiteLLM batch well for throughput.

## Next steps

- See [Hybrid Search](04-hybrid-search-basics.md) for `text_query` fusion on top of your vectors
- See [Local RAG Pipeline](02-local-rag-pipeline.md) for a complete offline ingestion + retrieval flow
- The integration packages (`vantadb-openai`, `vantadb-ollama`, `vantadb-litellm`, LangChain/LlamaIndex adapters) pre-wire these patterns for their ecosystems

---

**Key takeaway:** embeddings are pluggable — swap `embed()` and nothing else changes. Bring OpenAI, Ollama, LiteLLM, or a hash function; VantaDB stores, indexes, and searches whatever vectors you give it.

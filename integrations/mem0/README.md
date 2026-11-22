# VantaDB × Mem0

Mem0 adapter for [VantaDB](https://github.com/ness-e/Vantadb).

## Install

> **Published on PyPI at 0.9.0** (verified live 2026-10-20). The PyPI
> install pulls the published release; to run against the current tree use
> the repo-checkout path.

```bash
# From a repo checkout (current tree)
cd integrations/mem0 && pip install .
```

### Install from PyPI

```bash
pip install vantadb-mem0
```

## Quickstart

```python
from vantadb_mem0 import VantaDBVectorStore

store = VantaDBVectorStore(
    db_path="./my_data",
    namespace="memories",
)

store.add("VantaDB is an embedded vector database written in Rust.", user_id="alice")

# Option A: BM25 lexical search directly (no external embedding required)
bm25_results = store.search("vector database", top_k=5)

# Option B: vector similarity search when mem0 provides pre-computed vectors
query_vector = [0.1] * 384
store.insert([query_vector], payloads=[{"data": "VantaDB vector memory", "user_id": "alice"}], ids=["k1"])
results = store.search("vector database", query_vector, top_k=5)
for r in results:
    print(r.payload, r.score)   # OutputData exposes .id / .score / .payload
```

## API

- `add(text, user_id=None, metadata=None)` — store a memory (embeds internally
  when an embedding callable is set)
- `search(query, vectors=None, top_k=5, filters=None, text_query=None)` —
  search by a pre-computed dense vector (mem0 supplies the embedding), via
  `self.embedding(query)` when configured, or via BM25 lexical search when
  no vector is passed (pass `text_query=` alongside `vectors` for hybrid RRF);
  returns `OutputData` objects (`.id`, `.score`, `.payload`). `filters` scopes
  results exactly: flat `{"field": value}` / `{"$eq": v}` is pushed down to
  the engine; `$neq`/`$ne`, `$gt`, `$gte`, `$lt`, `$lte`, `$in`, `$nin` are
  evaluated as a Python post-filter; unknown operators fail closed (never
  broaden the result set) — this is what enforces mem0's `user_id`/`agent_id`
  isolation
- `delete(key)` — delete a memory by key
- `list(filters=None, top_k=100)` — list memories

## Why VantaDB?

- **Embedded & local-first:** the storage engine is a Rust library embedded
  in your process — no server to deploy, no network hop; data lives in your
  filesystem.
- **Persistent vector, BM25 & hybrid search:** HNSW vector search with
  pre-computed embeddings, BM25 lexical search when querying without vectors,
  and optional hybrid RRF fusion (`text_query=`) as the vector backend for
  mem0's memory layer, where mem0's default storage targets its hosted
  platform.
- **Zero-setup alternative to hosted stacks:** unlike Zep (requires a server)
  or Cognee (spins up its own knowledge-graph runtime), VantaDB is a plain
  library you import.

## Development

```bash
pip install -e .
```

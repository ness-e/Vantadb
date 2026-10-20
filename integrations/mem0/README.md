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

# mem0 owns the embedding layer: it computes the dense vector for the query
# and passes it in — this adapter does not embed. `query_vector` below is the
# vector mem0 already computed for "vector database".
query_vector = mem0_embeddings["vector database"]
results = store.search("vector database", query_vector, top_k=5)
for r in results:
    print(r.payload, r.score)   # OutputData exposes .id / .score / .payload
```

## API

- `add(text, user_id=None, metadata=None)` — store a memory (embeds internally
  when an embedding callable is set)
- `search(query, vectors, top_k=5, filters=None)` — search by a pre-computed
  dense vector (mem0 supplies the embedding); returns `OutputData` objects
  (`.id`, `.score`, `.payload`)
- `delete(key)` — delete a memory by key
- `list(filters=None, top_k=100)` — list memories

## Why VantaDB?

- **Embedded & local-first:** the storage engine is a Rust library embedded
  in your process — no server to deploy, no network hop; data lives in your
  filesystem.
- **Persistent hybrid search:** vectors + BM25 text search out of the box as
  the vector backend for mem0's memory layer, where mem0's default storage
  targets its hosted platform.
- **Zero-setup alternative to hosted stacks:** unlike Zep (requires a server)
  or Cognee (spins up its own knowledge-graph runtime), VantaDB is a plain
  library you import.

## Development

```bash
pip install -e .
```

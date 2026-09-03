# VantaDB × Mem0

Mem0 adapter for [VantaDB](https://github.com/ness-e/Vantadb).

## Install

> **Not on PyPI yet.** The `vantadb-mem0` package builds and passes
> `twine check` locally; it will go live with the first `adapters-v*` tag
> release. Until then, install from source.

```bash
# Today, from a repo checkout
cd integrations/mem0 && pip install .
```

### Install from PyPI (after first release)

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
results = store.search("vector database")
for r in results:
    print(r["payload"], r["score"])
```

## API

- `add(text, user_id=None, metadata=None)` — store a memory
- `search(query, k=4)` — search memories
- `delete(key)` — delete a memory by key
- `list(user_id=None, limit=100)` — list memories

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

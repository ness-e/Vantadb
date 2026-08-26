# VantaDB × Letta

Letta storage adapter for [VantaDB](https://github.com/ness-e/Vantadb).

## Install

```bash
pip install vantadb-letta
```

## Quickstart

```python
from vantadb_letta import VantaDBVectorStore

store = VantaDBVectorStore(
    db_path="./my_data",
    namespace="agent_memory",
)

store.insert("VantaDB is a vector database written in Rust.", source="docs")
results = store.search("vector database")
for r in results:
    print(r["text"])
```

## API

- `insert(text, source=None, metadata=None)` — store a memory record
- `search(query, k=4)` — search stored memories
- `delete(key)` — delete by key
- `list(limit=100)` — list all records

## Status: experimental

Letta is a stateful platform with its own memory layer and no public
vector-store contract; this adapter is a community convenience, not an
officially supported integration. The API may change without notice and it
is not exercised by Letta's own test suite. Prefer Letta's native memory
unless you specifically need to back it with an embedded local store.

## Why VantaDB?

- **Embedded & local-first:** the storage engine is a Rust library embedded
  in your process — no server to deploy, no network hop; data lives in your
  filesystem.
- **Persistent hybrid search:** vectors + BM25 text search out of the box,
  where Letta's own memory is designed for its server-side platform.
- **Zero-setup alternative to hosted stacks:** unlike Zep (requires a server)
  or Cognee (spins up its own knowledge-graph runtime), VantaDB is a plain
  library you import.

## Development

```bash
pip install -e .
```

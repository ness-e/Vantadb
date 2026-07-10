# VantaDB × Mem0

Mem0 adapter for [VantaDB](https://github.com/ness-e/Vantadb).

## Install

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

## Development

```bash
pip install -e .
```

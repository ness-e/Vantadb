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

## Development

```bash
pip install -e .
```

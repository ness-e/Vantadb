# VantaDB × Mem0 — Vector Store Backend

Use **VantaDB** as the vector store backend for [Mem0](https://github.com/mem0ai/mem0) — an intelligent memory layer for AI agents.

## What this does

This crate exposes a Python class that implements Mem0's `VectorStoreBase` ABC, allowing Mem0 to store and retrieve memory embeddings directly in VantaDB's embedded vector engine.

## How Mem0 users configure VantaDB

```python
from mem0 import Memory

config = {
    "vector_store": {
        "provider": "vantadb",
        "config": {
            "path": "~/.vantadb/mem0",
            "collection_name": "memories",
        },
    },
}

memory = Memory.from_config(config)
```

## Mem0 VectorStoreBase interface

Every backend implements the abstract methods from `mem0.vector_stores.base.VectorStoreBase`:

| Method | Description |
|--------|-------------|
| `create_col(name, vector_size, distance)` | Create a new collection |
| `insert(vectors, payloads=None, ids=None)` | Insert vectors with metadata |
| `search(query, vectors, top_k=5, filters=None)` | Semantic similarity search |
| `delete(vector_id)` | Delete a vector by ID |
| `update(vector_id, vector=None, payload=None)` | Update vector and/or payload |
| `get(vector_id)` | Retrieve a vector by ID |
| `list_cols()` | List all collections |
| `delete_col()` | Delete a collection |
| `col_info()` | Get collection metadata |
| `list(filters=None)` | List all memories with optional filters |
| `reset()` | Reset the entire store |

## Installation

```bash
pip install vantadb-mem0
```

Or install from source:

```bash
pip install git+https://github.com/ness-e/Vantadb.git#subdirectory=vantadb-mem0
```

## Development

```bash
cargo build --manifest-path vantadb-mem0/Cargo.toml
```

To skip the Python extension:

```bash
cargo build --manifest-path vantadb-mem0/Cargo.toml --no-default-features
```

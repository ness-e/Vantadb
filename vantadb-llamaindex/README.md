# VantaDB LlamaIndex Adapter

Use VantaDB as a drop-in vector store for LlamaIndex.

## Installation

```bash
pip install vantadb-llamaindex
```

## Usage

```python
from vantadb_llamaindex import VantaDBVectorStore

# Create a vector store
store = VantaDBVectorStore("/path/to/db")

# Add texts with embeddings
texts = ["Paris is the capital of France", "London is the capital of the UK"]
embeddings = [[0.1, 0.2, ...], [0.3, 0.4, ...]]
ids = store.add(texts, embeddings)

# Query
results = store.query([0.1, 0.2, ...], top_k=5)
for node in results:
    print(node["text"], node["score"])

# Delete
store.delete(ids)
```

## Development

Built with PyO3, uses the VantaDB Rust core for storage.

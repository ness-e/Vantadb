# VantaDB LangChain Adapter

Use VantaDB as a drop-in vector store for LangChain.

## Installation

```bash
pip install vantadb-langchain
```

## Usage

```python
from vantadb_langchain import VantaDBVectorStore

# Create a vector store
store = VantaDBVectorStore("/path/to/db")

# Add texts with embeddings
texts = ["Paris is the capital of France", "London is the capital of the UK"]
embeddings = [[0.1, 0.2, ...], [0.3, 0.4, ...]]
ids = store.add_texts(texts, embeddings)

# Search by vector
results = store.similarity_search_by_vector([0.1, 0.2, ...], k=5)
for doc in results:
    print(doc["text"], doc["score"])

# Delete
store.delete(ids)
```

## Development

Built with PyO3, uses the VantaDB Rust core for storage.

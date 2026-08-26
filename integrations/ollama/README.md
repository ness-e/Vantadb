# VantaDB × Ollama

Ollama embedding + storage adapter for [VantaDB](https://github.com/ness-e/Vantadb).

## Install

```bash
pip install vantadb-ollama
```

## Quickstart

```python
from vantadb_ollama import VantaDBOllama

store = VantaDBOllama(
    model="nomic-embed-text",
    db_path="./my_data",
)

store.add_texts(["VantaDB is an embedded vector database."])
results = store.similarity_search("vector database")
for doc in results:
    print(doc.page_content)
```

## API

- `add_texts(texts, metadatas=None, ids=None)` — embed and store texts
- `similarity_search(query, k=4)` — search by query text
- `delete(ids)` — delete by IDs

## Why VantaDB?

- **Embedded & local-first:** the storage engine is a Rust library embedded
  in your process — no server to deploy, no network hop; data lives in your
  filesystem.
- **Persistent hybrid search:** vectors + BM25 text search out of the box,
  on top of Ollama's local embeddings — fully offline.
- **Zero-setup alternative to hosted stacks:** unlike Zep (requires a server)
  or Cognee (spins up its own knowledge-graph runtime), VantaDB is a plain
  library you import.

## Development

```bash
pip install -e .
```

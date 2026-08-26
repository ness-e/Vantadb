# VantaDB × Haystack

Haystack DocumentStore adapter for [VantaDB](https://github.com/ness-e/Vantadb).

## Install

```bash
pip install vantadb-haystack
```

## Quickstart

```python
from vantadb_haystack import VantaDBDocumentStore

store = VantaDBDocumentStore(db_path="./my_data")

# Write documents
store.write_documents([
    {"id": "1", "content": "VantaDB is an embedded vector database written in Rust.",
     "meta": {"source": "docs"}},
])

# Filter documents
results = store.filter_documents(filters={"source": "docs"})
for doc in results:
    print(doc.content)
```

## API

- `write_documents(documents, policy)` — store documents
- `filter_documents(filters)` — retrieve by filter
- `delete_documents(filters)` — remove documents
- `count_documents(filters)` — count documents

## Why VantaDB?

- **Embedded & local-first:** the storage engine is a Rust library embedded
  in your process — no server to deploy, no network hop; data lives in your
  filesystem.
- **Persistent hybrid search:** vectors + BM25 text search out of the box,
  where Haystack's `InMemoryDocumentStore` covers only small single-process
  cases.
- **Zero-setup alternative to hosted stacks:** unlike Zep (requires a server)
  or Cognee (spins up its own knowledge-graph runtime), VantaDB is a plain
  library you import.

## Development

```bash
pip install -e .
```

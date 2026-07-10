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

## Development

```bash
pip install -e .
```

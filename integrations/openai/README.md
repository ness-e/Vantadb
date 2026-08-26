# VantaDB × OpenAI

OpenAI embedding + storage adapter for [VantaDB](https://github.com/ness-e/Vantadb).

## Install

```bash
pip install vantadb-openai
```

## Quickstart

```python
from vantadb_openai import VantaDBOpenAI

store = VantaDBOpenAI(
    api_key="sk-...",
    model="text-embedding-3-small",
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
  even when the embedding API is unavailable (keyword search keeps working).
- **Zero-setup alternative to hosted stacks:** unlike Zep (requires a server)
  or Cognee (spins up its own knowledge-graph runtime), VantaDB is a plain
  library you import.

## Development

```bash
pip install -e .
```

# VantaDB × OpenAI

OpenAI embedding + storage adapter for [VantaDB](https://github.com/ness-e/Vantadb).

## Install

> **Superseded — not part of the PyPI release (FIND-273, owner decision
> 2026-10-06).** The `vantadb-openai` PyPI name and `vantadb_openai` module
> belong to the canonical Rust provider
> [`providers/openai`](../../providers/openai/README.md); this Python adapter
> stays in-repo as a source-only reference and is **not** published by the
> `adapters-v*` lane.

```bash
# Source-only, from a repo checkout
cd integrations/openai && pip install .
```

### From PyPI (the canonical Rust provider)

```bash
pip install vantadb-openai   # installs the Rust provider (published since 0.9.0)
```

> **Name collision:** `pip install vantadb-openai` installs the **Rust
> provider**, not this Python adapter — and the two expose different APIs
> (both use the module name `vantadb_openai`). Pick the quickstart that
> matches what you installed.

## Quickstart

### Rust provider (`pip install vantadb-openai`)

```python
from vantadb_openai import VantaDBOpenAI

store = VantaDBOpenAI("/tmp/vantadb-openai", api_key="sk-...")
emb = store.embed(["VantaDB is an embedded vector database."])
rid = store.store("VantaDB is an embedded vector database.", emb[0], metadata={"lang": "en"})
results = store.search("openai_store", emb[0], top_k=5)
print(rid, results)
```

### Python adapter (source-only, this repo)

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

### Rust provider (`vantadb-openai` on PyPI)

- `embed(texts)` — generate embeddings via OpenAI's API
- `store(text, embedding, metadata=None, key=None)` — store a record; returns `"<namespace>:<key>"`
- `search(namespace, query_embedding, text_query=None, filters=None, distance_metric=None, top_k=10)` — hybrid vector + BM25 search
- `get(namespace, key)` · `list(namespace, limit=100, cursor=None)` · `delete(key, namespace=None)` · `list_namespaces()`

### Python adapter (source-only)

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

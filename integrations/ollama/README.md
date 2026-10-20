# VantaDB × Ollama

Ollama embedding + storage adapter for [VantaDB](https://github.com/ness-e/Vantadb).

## Install

> **Superseded — not part of the PyPI release (FIND-273, owner decision
> 2026-10-06).** The `vantadb-ollama` PyPI name and `vantadb_ollama` module
> belong to the canonical Rust provider
> [`providers/ollama`](../../providers/ollama/README.md); this Python adapter
> stays in-repo as a source-only reference and is **not** published by the
> `adapters-v*` lane.

```bash
# Source-only, from a repo checkout
cd integrations/ollama && pip install .
```

### From PyPI (the canonical Rust provider)

```bash
pip install vantadb-ollama   # installs the Rust provider (published since 0.9.0)
```

> **Name collision:** `pip install vantadb-ollama` installs the **Rust
> provider**, not this Python adapter — and the two expose different APIs
> (both use the module name `vantadb_ollama`). Pick the quickstart that
> matches what you installed.

## Quickstart

### Rust provider (`pip install vantadb-ollama`)

```python
from vantadb_ollama import VantaDBOllama

store = VantaDBOllama("/tmp/vantadb-ollama")  # base_url defaults to http://localhost:11434
emb = store.embed(["VantaDB is an embedded vector database."])
rid = store.store("VantaDB is an embedded vector database.", emb[0], metadata={"lang": "en"})
results = store.search("ollama_store", emb[0], top_k=5)
print(rid, results)
```

### Python adapter (source-only, this repo)

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

### Rust provider (`vantadb-ollama` on PyPI)

- `embed(texts)` — generate embeddings via Ollama's `/api/embed`
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
  on top of Ollama's local embeddings — fully offline.
- **Zero-setup alternative to hosted stacks:** unlike Zep (requires a server)
  or Cognee (spins up its own knowledge-graph runtime), VantaDB is a plain
  library you import.

## Development

```bash
pip install -e .
```

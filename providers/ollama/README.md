# VantaDB × Ollama

PyO3 cdylib that exposes a `VantaDBOllama` class — wraps Ollama's local embedding API with VantaDB for local vector storage and hybrid search. No API key required.

## Install

Requires Python 3.11+, a running Ollama server, and the Ollama SDK:

```bash
pip install ollama
```

Build the extension locally with [maturin](https://www.maturin.rs/):

```bash
maturin develop --release
```

## Quickstart

```python
from vantadb_ollama import VantaDBOllama

store = VantaDBOllama("/tmp/vantadb-ollama")  # base_url defaults to http://localhost:11434
emb = store.embed(["hello world"])
rid = store.store("hello world", emb[0], metadata={"lang": "en"})
results = store.search("ollama_store", emb[0], top_k=5)
print(rid, results)
```

## API

| Method | Signature | Description |
|--------|-----------|-------------|
| `embed` | `embed(texts) -> list[list[float]]` | Generate embeddings via Ollama's `/api/embed`. |
| `store` | `store(text, embedding, metadata=None) -> str` | Store a record; returns `"<namespace>:<key>"`. Metadata values must be `str`/`bool`/`int`/`float`. |
| `search` | `search(namespace, query_embedding, text_query=None, filters=None, distance_metric=None, top_k=10) -> list[dict]` | Hybrid vector + BM25 search (`distance_metric`: `"cosine"` or `"euclidean"`/`"l2"`). Returns `{id, text, score}` per hit. |
| `get` | `get(namespace, key) -> dict \| None` | Retrieve a single record, or `None` if not found. |
| `list` | `list(namespace, limit=100, cursor=None) -> dict` | Paginated listing: `{"records": [...], "cursor": str}` when a next page exists. |
| `delete` | `delete(key, namespace=None) -> bool` | Delete a record by key; defaults to the instance namespace. |
| `list_namespaces` | `list_namespaces() -> list[str]` | All namespaces containing at least one record. |

Constructor options: `VantaDBOllama(db_path, base_url="http://localhost:11434", model="nomic-embed-text", namespace="ollama_store", timeout=None)`.

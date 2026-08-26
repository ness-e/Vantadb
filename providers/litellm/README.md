# VantaDB × LiteLLM

PyO3 cdylib that exposes a `VantaDBLiteLLM` class — a unified embedding provider that generates embeddings through LiteLLM's `embedding()` gateway and stores/searches them in VantaDB.

## Install

Requires Python 3.11+ and the LiteLLM SDK:

```bash
pip install litellm
```

Build the extension locally with [maturin](https://www.maturin.rs/):

```bash
maturin develop --release
```

## Quickstart

```python
from vantadb_litellm import VantaDBLiteLLM

store = VantaDBLiteLLM("/tmp/vantadb-litellm", timeout=30)
emb = store.embed(["hello world"])
rid = store.store("hello world", emb[0], metadata={"lang": "en"})
results = store.search("litellm_store", emb[0], top_k=5)
print(rid, results)
```

## API

| Method | Signature | Description |
|--------|-----------|-------------|
| `embed` | `embed(texts) -> list[list[float]]` | Generate embeddings via `litellm.embedding()`; forwards `timeout` (seconds) when set. |
| `store` | `store(text, embedding, metadata=None) -> str` | Store a record; returns `"<namespace>:<key>"`. Metadata values must be `str`/`bool`/`int`/`float`. |
| `search` | `search(namespace, query_embedding, text_query=None, filters=None, distance_metric=None, top_k=10) -> list[dict]` | Hybrid vector + BM25 search (`distance_metric`: `"cosine"` or `"euclidean"`/`"l2"`). Returns full records plus `score`. |
| `get` | `get(namespace, key) -> dict \| None` | Retrieve a single record, or `None` if not found. |
| `list` | `list(namespace, limit=100, cursor=None) -> dict` | Paginated listing: `{"records": [...], "next_cursor": int \| None}`. |
| `delete` | `delete(key, namespace=None) -> bool` | Delete a record by key; defaults to the instance namespace. |
| `list_namespaces` | `list_namespaces() -> list[str]` | All namespaces containing at least one record. |

Constructor options: `VantaDBLiteLLM(db_path, api_key=None, model="text-embedding-3-small", namespace="litellm_store", timeout=None)`.

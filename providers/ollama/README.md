# VantaDB × Ollama

PyO3 cdylib that exposes a `VantaDBOllama` class — wraps Ollama's local embedding API with VantaDB for local vector storage and hybrid search. No API key required.

## Install

> **Published on PyPI at 0.9.0** (verified live 2026-10-20). The PyPI
> install pulls the published release; to build against the current tree use
> the maturin path below.

### From PyPI

```bash
pip install vantadb-ollama
```

The Ollama SDK is installed automatically (`ollama>=0.4,<1`).

### From source

Requires Python 3.11+, a running Ollama server, and the Ollama SDK. Build the
extension with [maturin](https://www.maturin.rs/):

```bash
pip install ollama
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
| `store` | `store(text, embedding, metadata=None, key=None) -> str` | Store a record; returns `"<namespace>:<key>"`. `key` upserts at a deterministic key, else auto-generated. Metadata values must be `str`/`bool`/`int`/`float`. |
| `search` | `search(namespace, query_embedding, text_query=None, filters=None, distance_metric=None, top_k=10) -> list[dict]` | Hybrid vector + BM25 search (`distance_metric`: `"cosine"` or `"euclidean"`/`"l2"`). Returns full records plus `score`. |
| `get` | `get(namespace, key) -> dict \| None` | Retrieve a single record, or `None` if not found. |
| `list` | `list(namespace, limit=100, cursor=None) -> dict` | Paginated listing: `{"records": [...], "next_cursor": int \| None}`. |
| `delete` | `delete(key, namespace=None) -> bool` | Delete a record by key; defaults to the instance namespace. |
| `list_namespaces` | `list_namespaces() -> list[str]` | All namespaces containing at least one record. |

Constructor options: `VantaDBOllama(db_path, base_url="http://localhost:11434", model="nomic-embed-text", namespace="ollama_store", timeout=None)`.

---
title: Migrating from ChromaDB to VantaDB
kind: tutorial
status: active
description: Move a ChromaDB persistent store into VantaDB with the bundled migration helper — commands, limits, and readback checks.
tags: [migration, chromadb, tutorial]
---

# Migrating from ChromaDB to VantaDB

> **Canonical reference.**

If you're using ChromaDB today, VantaDB gives you graph edges, MCP support, a WASM runtime, and hybrid search while keeping a similar embedded workflow. This tutorial shows the API mapping and the migration helper that ships in this repository.

## Side-by-side API comparison

| Operation | ChromaDB | VantaDB |
|-----------|----------|---------|
| Connect | `chromadb.PersistentClient(path)` | `Client(path)` from `vantadb` |
| Create/get collection | `client.get_or_create_collection(name)` | Lazy — namespaces are created on first `put()` |
| Insert documents | `collection.add(ids, documents, metadatas)` | `db.put(namespace, key, payload, ...)` |
| Semantic search | `collection.query(query_texts)` | `db.search(namespace, vector, ...)` |
| Get by ID | `collection.get(ids)` | `db.memory.get(namespace, key)` |
| Delete | `collection.delete(ids)` | `db.memory.delete(namespace, key)` |
| List records | `collection.get(limit=..., offset=...)` | `db.memory.list(namespace, limit=...)` (one page) |

## 1. Setup comparison

Use a VantaDB 0.9.x Python binding and install ChromaDB in the same environment:

```bash
pip install "vantadb-py==0.9.0" chromadb
```

Run the CLI from a complete checkout. Stop source writers and back up the source
before opening it: ChromaDB's `PersistentClient` is not a filesystem read-only
reader and can initialize or migrate its own storage. Prefer a backup copy when
the original must remain byte-for-byte unchanged.

**ChromaDB:**

```python
import chromadb

client = chromadb.PersistentClient(path="./chroma_data")
collection = client.get_or_create_collection(
    name="my_docs",
    metadata={"hnsw:space": "cosine"},
)
```

**VantaDB:**

These three-dimensional vectors illustrate the API only. For real semantic
retrieval, replace them with document and query vectors from the same encoder.

```python
from vantadb import Client

db = Client("./vantadb_data")
# Namespaces are created lazily on first put().
```

ChromaDB separates collections from their documents. VantaDB stores records under **namespaces** — the equivalent of a collection.

## 2. Inserting documents

**ChromaDB:**

```python
# vanta-skip: requires the chromadb package; first call downloads its ONNX embedding model
import chromadb

client = chromadb.PersistentClient(path="./chroma_data")
collection = client.get_or_create_collection(name="my_docs")

collection.add(
    ids=["doc1", "doc2"],
    documents=["VantaDB is an embedded vector database.", "It supports Python, TS, and Rust."],
    metadatas=[
        {"source": "docs", "page": 1},
        {"source": "docs", "page": 2},
    ],
)
```

**VantaDB:**

```python
db.put(
    "my_docs",                          # namespace (≈ collection name)
    "doc1",                             # key (≈ ChromaDB id)
    "VantaDB is an embedded vector database.",
    metadata={"source": "docs", "page": 1},
    vector=[1.0, 0.0, 0.0],
)
db.put(
    "my_docs",
    "doc2",
    "It supports Python, TS, and Rust.",
    metadata={"source": "docs", "page": 2},
    vector=[0.0, 1.0, 0.0],
)
```

Key differences:
- ChromaDB separates `ids`, `documents`, and `metadatas` into parallel arrays.
- VantaDB `put()` takes `(namespace, key, payload)` and optional `metadata` / `vector` / `ttl_ms`.
- To store pre-computed embeddings, pass them as `vector=[...]`.

## 3. Querying

**ChromaDB:**

```python
# vanta-skip: requires the chromadb package; first call downloads its ONNX embedding model
import chromadb

client = chromadb.PersistentClient(path="./chroma_data")
collection = client.get_or_create_collection(name="my_docs")

results = collection.query(
    query_texts=["embedded database"],
    n_results=5,
    where={"source": "docs"},
)
```

**VantaDB:**

```python
query_vector = [1.0, 0.0, 0.0]  # illustrative vector, not a model-generated embedding

results = db.search(
    "my_docs",
    query_vector,
    filters={"source": "docs"},
    top_k=5,
)
```

VantaDB returns `SearchHit` objects with attribute access (`hit.key`, `hit.payload`, `hit.metadata`, `hit.score`) instead of ChromaDB's dict-of-lists format.

## 4. Migration script

A ready-to-run CLI ships in this repository at
[`scripts/migrate_from_chroma.py`](../../../scripts/migrate_from_chroma.py).
It loads the migration helper **from this checkout** (`vantadb-python/vantadb_py/migrate/chroma.py`), so the published `vantadb-py` wheel and this repo version stay compatible: the wheel only needs to provide the `vantadb` binding (0.9.x). It reads every document through `collection.get()` — never `collection.query()` — so no Chroma auto-embedding runs during migration, and Chroma telemetry is disabled via `Settings(anonymized_telemetry=False)`.

```bash
# All collections → chroma/<collection> namespaces (default prefix)
python scripts/migrate_from_chroma.py --chroma-path ./chroma_data --vanta-path ./vanta_data

# Only one collection
python scripts/migrate_from_chroma.py --chroma-path ./chroma_data --vanta-path ./vanta_data \
    --collection my_docs

# Custom batch size and namespace prefix
python scripts/migrate_from_chroma.py --chroma-path ./chroma_data --vanta-path ./vanta_data \
    --batch-size 250 --namespace-prefix "imported/"
```

Flags:

| Flag | Default | Meaning |
|------|---------|---------|
| `--chroma-path` | `./chroma_data` | Path to the ChromaDB persistent store (must contain `chroma.sqlite3`) |
| `--vanta-path` | `./vanta_data` | Path where the VantaDB database will be created |
| `--collection` | `all` | Migrate only this collection |
| `--namespace-prefix` | `chroma/` | Prefix for derived namespaces |
| `--batch-size` | `500` | Records per batch (must be positive) |

Programmatic use (published wheel: same function, same positional signature; the `namespace_prefix`/`on_collection` keyword-only extension lives in this checkout):

```python
# vanta-skip: requires chromadb package for migration
from vantadb_py.migrate import migrate_from_chroma

count = migrate_from_chroma("./chroma_data", "./vanta_data")
print(f"Migrated {count} records")
```

## 5. What changes — and what doesn't

- **Vectors stay vectors** — but only if both stores use the same embedding **model and space**. Matching dimensions (e.g. 384d) is *not* enough: vectors from different models are not comparable even at equal length. Keep using your Chroma embedding model for queries, or re-embed.
- **Chroma has real persistence** — `PersistentClient` stores data on disk.
  Migration copies records rather than deleting them from Chroma. It is not a
  byte-level read-only snapshot; use a backup copy and quiesce writers first.
- **Chroma auto-embeds by default; VantaDB doesn't.** The migrator reads stored embeddings directly. For new writes you provide `vector=` yourself (local ONNX exports under `embeddings/models/`, an SDK, or any callable).
- **A batch is not a transaction.** If migration fails mid-run, the destination
  can hold a partial import. CLI progress counts completed collections, not
  every write in the failing collection. Re-running with the same namespace
  prefix updates the same keys, though internal version history may grow.

## 6. Post-migration checklist

```python
from vantadb import Client

db = Client("./vanta_data")

# Readback: verify records landed under the expected namespace.
rec = db.memory.get("chroma/my_docs", "doc1")
assert rec is not None

page = db.memory.list("chroma/my_docs")
print(len(page.records), "records in the first page, not the migration total")

db.flush()   # persist
db.close()   # release the writer lock
```

### Rollback

Keep an independent source backup. The migration does not delete Chroma records,
but opening Chroma is not a promise of byte-level immutability. If the destination
was newly created for this import, discard it only after confirming it contains
no unrelated data. Never delete an existing shared database as a rollback step.

## Known limitations

- **Collections → Namespaces**: ChromaDB collections are first-class objects with metadata. VantaDB namespaces are string prefixes on keys, created lazily on first `put()` — there is no `create_namespace()`.
- **No `peek()` equivalent**: use `memory.list()` with `limit` and optional `filters`.
- **No `where` document filter by content**: VantaDB metadata filters match the `metadata` field, not document text. Use `text_query` for payload search.
- **No `update` vs `upsert` distinction**: VantaDB `put()` is always upsert.
- **VantaDB is embedded**: records live next to your app; the optional HTTP server (for tooling) and the MCP launcher (`vanta-cli server --mcp --db <absolute-path>`, profile via `VANTADB_MCP_PROFILE`) are documented in `docs/api/MCP.md`.

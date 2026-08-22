---
title: "Migrating from ChromaDB to VantaDB"
status: active
tags: [vantadb, tutorial, guide, migration, chromadb]
last_reviewed: 2026-08-02
aliases: []
---

# Migrating from ChromaDB to VantaDB

> **Canonical reference.**

If you're using ChromaDB today, switching to VantaDB unlocks **graph edges**, **MCP protocol support**, **WASM browser runtime**, and **hybrid search** — while keeping your existing vector workflow. This tutorial shows the exact API mappings and provides a migration script you can run on your existing ChromaDB data.

## Side-by-side API comparison

| Operation | ChromaDB | VantaDB |
|-----------|----------|---------|
| Connect | `chromadb.PersistentClient(path)` | `VantaDB(path)` from `vantadb_py` |
| Create/get collection | `client.get_or_create_collection(name)` | Lazy — namespaces are created on first `put()` |
| Insert documents | `collection.add(ids, documents, metadatas)` | `db.put(namespace, key, payload, ...)` |
| Semantic search | `collection.query(query_texts)` | `db.search_memory(namespace, vector, ...)` |
| Get by ID | `collection.get(ids)` | `db.get_memory(namespace, key)` |
| Delete | `collection.delete(ids)` | `db.delete_memory(namespace, key)` |
| List all | `collection.get()` | `db.list_memory(namespace)` |

## 1. Setup comparison

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

```python
from vantadb_py import VantaDB

db = VantaDB("./vantadb_data")
# HNSW with cosine is the default — nothing extra to configure.
```

ChromaDB separates collections from their documents. VantaDB stores records under **namespaces** — the equivalent of a collection. Namespaces are created lazily on the first `put()`.

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
)
db.put(
    "my_docs",
    "doc2",
    "It supports Python, TS, and Rust.",
    metadata={"source": "docs", "page": 2},
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
query_vector = [0.1, 0.2, 0.3]  # embeddings from your embedding model

results = db.search_memory(
    "my_docs",
    query_vector,
    filters={"source": "docs"},
    top_k=5,
)
```

VantaDB returns `VantaSearchHit` objects with attribute access (`hit.key`, `hit.payload`, `hit.metadata`, `hit.score`) instead of ChromaDB's dict-of-lists format.

## 4. Migration script

A ready-to-run migration script ships with the Python SDK. It reads every document from a ChromaDB persistent store (all collections, or one) and imports them into VantaDB, preserving ids, documents, metadatas, and pre-computed embeddings.

```bash
# All collections → one VantaDB database
python -m vantadb_py.migrate.chroma --source ./chroma_data --dest ./vantadb_data

# Only one collection, mapped to a custom namespace
python -m vantadb_py.migrate.chroma \
    --source ./chroma_data \
    --dest ./vantadb_data \
    --collection-name my_docs \
    --namespace memories
```

Flags:

| Flag | Default | Meaning |
|------|---------|---------|
| `--source` | — (required) | Path to the ChromaDB persistent store |
| `--dest` | — (required) | Path where the VantaDB database will be created |
| `--collection-name` | all | Migrate only this collection |
| `--namespace` | collection name | Target VantaDB namespace |
| `--batch-size` | 500 | Records per batch |

Programmatic use:

```python
from vantadb_py.migrate import migrate_from_chroma

count = migrate_from_chroma("./chroma_data", "./vantadb_data")
print(f"Migrated {count} records")
```

## 5. Feature comparison: what you gain

| Feature | ChromaDB | VantaDB |
|---------|----------|---------|
| Vector search (HNSW) | ✅ | ✅ |
| Metadata filtering | ✅ | ✅ |
| **Hybrid search (vector + BM25)** | ❌ | ✅ |
| **Graph edges (knowledge graph)** | ❌ | ✅ |
| **MCP protocol support** | ❌ | ✅ |
| **WASM browser runtime** | ❌ | ✅ |
| Python SDK | ✅ | ✅ |
| TypeScript SDK | ❌ | ✅ |
| Rust SDK | ❌ | ✅ |
| Embedded (no server) | ✅ | ✅ |
| Auto-embedding | ❌ (manual) | ✅ (plug-in) |
| Single-node data model | ❌ (parallel arrays) | ✅ (rich objects) |

## 6. Post-migration checklist

After migrating, you can immediately start using VantaDB-specific features:

### Add graph edges between documents

```python
# Graph nodes take integer node IDs — link records via their node_id:
r1 = db.put("my_docs", "doc1", "VantaDB is an embedded vector database.",
            vector=[0.1, 0.2, 0.3])
r2 = db.put("my_docs", "doc2", "It supports Python, TS, and Rust.",
            vector=[0.4, 0.5, 0.6])

db.add_edge(r1.node_id, r2.node_id, "related")

# Traverse forward from a root node up to max_depth hops
path = db.graph_bfs([r1.node_id], 3)
print(path)   # [node_id of doc1, node_id of doc2]
```

### Enable hybrid search

```python
query_vector = [0.1, 0.2, 0.3]  # embeddings from your embedding model

# Hybrid (BM25 + HNSW) search on a text query:
results = db.search_memory(
    "my_docs",
    query_vector,          # still required; pass your query embedding
    text_query="your query",
    top_k=10,
)
```

### Expose via MCP

```python
# In your MCP server config:
# {
#   "vantadb": {
#     "path": "./vantadb_data"
#   }
# }
# Now any MCP-compatible host (Claude Desktop, etc.) can query your data.
```

### Run in the browser

```python
# With VantaDB WASM build, the same code works in a browser:
# import { VantaDB } from '@vantadb/wasm'
```

## Summary

| Task | ChromaDB equivalent | VantaDB equivalent |
|------|-------------------|--------------------|
| Connect | `PersistentClient(path)` | `VantaDB(path)` |
| Collection | `get_or_create_collection(name)` | namespace arg (lazy) |
| Insert | `collection.add(ids, docs, metas)` | `db.put(ns, key, payload, metadata=...)` |
| Query | `collection.query(query_texts)` | `db.search_memory(ns, vector, ...)` |
| Delete | `collection.delete(ids)` | `db.delete_memory(ns, key)` |
| Filter | `where={...}` | `filters={...}` |

Migration takes ~5 minutes and you keep all your existing data and embeddings. From there, the graph engine, MCP protocol, WASM runtime, and hybrid search are available with zero additional setup.

---

**Key takeaway:** VantaDB is a drop-in upgrade from ChromaDB — same mental model, richer feature set, and your migration script is one command.

## Pre-migration checklist

- [ ] VantaDB is installed (`pip install vantadb-py` or `cargo add vantadb`)
- [ ] ChromaDB collection data is accessible
- [ ] You have a text representation for each document (for `payload` field)
- [ ] Embedding dimensions match between ChromaDB and VantaDB

## Post-migration index management

```python
# Rebuild all indexes (HNSW + text + derived)
report = db.rebuild_index()

# Compact vector store for better page-fault locality
db.compact_wal()  # archive + start fresh WAL
```

## Known limitations

- **Collections → Namespaces**: ChromaDB collections are first-class objects with metadata. VantaDB namespaces are string prefixes on keys. There is no `create_namespace()` — namespaces are created lazily on first `put()`.
- **No `peek()` equivalent**: Use `list_memory()` with `limit` and optional `filters`.
- **No `where` document filter by content**: VantaDB metadata filters match on `metadata` field, not document text. Use `text_query` for payload search.
- **No `update` vs `upsert` distinction**: VantaDB `put()` is always upsert.
- **VantaDB is embedded only**: There is no VantaDB server to connect to remotely (the optional HTTP server is for localhost tooling).

## Rollback plan

Keep your ChromaDB data directory intact during migration. VantaDB does not modify or delete your ChromaDB data.

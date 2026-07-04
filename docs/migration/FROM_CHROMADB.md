---
title: Migrating from ChromaDB to VantaDB
type: guide
status: active
tags: [vantadb, migration]
last_reviewed: 2026-07-01
aliases: []
---

# Migrating from ChromaDB to VantaDB

## Why migrate?

VantaDB is purpose-built for **local-first AI agents** — your data stays on-device, with no cloud dependency, no API keys, and no network calls. ChromaDB, by contrast, requires a running server process for persistence and has no built-in durability for collections.

| Feature | ChromaDB | VantaDB |
|---|---|---|
| **Persistence** | Optional (RocksDB backend), server process required | Always-on, embedded — no server needed |
| **Hybrid search** | BM25 + vector requires separate index | Built-in BM25 + HNSW RRF fusion in `search()` |
| **TTL** | Not built-in | Native `ttl_ms` on every record |
| **Graph traversal** | Not supported | Native edges, BFS, DFS, topological sort |
| **Durability** | Risk of HNSW corruption on unclean shutdown | WAL + CRC32C auto-healing, crash recovery |
| **Batch operations** | N/A | `put_batch()` with Rayon parallelism |
| **Export/Import** | Manual | Built-in JSONL `export_namespace()` / `import_file()` |
| **Server** | Required for persistence | Embedded (no server) or optional axum HTTP server with Prometheus metrics |
| **Embedding** | Requires external embedding function | BYO vector (engine-agnostic), stores raw `Vec<f32>` |

## Pre-migration checklist

- [ ] VantaDB is installed (`pip install vantadb-py` or `cargo add vantadb`)
- [ ] ChromaDB collection data is accessible
- [ ] You have a text representation for each document (for `payload` field)
- [ ] Embedding dimensions match between ChromaDB and VantaDB

## Migration steps

### 1. Export data from ChromaDB

```python
import chromadb

client = chromadb.PersistentClient(path="./chroma_data")
collection = client.get_collection("my_collection")

# Get all documents with embeddings and metadata
all_data = collection.get(
    include=["embeddings", "documents", "metadatas"]
)
```

### 2. Transform to VantaDB JSONL format

Each ChromaDB document becomes a VantaDB memory record. The `namespace` maps to the collection name, and `key` maps to a stable document identifier.

```python
import json

records = []
for i in range(len(all_data["ids"])):
    record = {
        "namespace": "my_collection",
        "key": all_data["ids"][i],
        "payload": all_data["documents"][i] or "",
        "metadata": all_data["metadatas"][i] or {},
        "vector": all_data["embeddings"][i] if all_data["embeddings"] else None
    }
    records.append(record)

# Write JSONL file
with open("export.jsonl", "w") as f:
    for rec in records:
        f.write(json.dumps(rec) + "\n")
```

### 3. Import into VantaDB

```python
import vantadb

# Open or create the database
db = vantadb.VantaDB("./vantadb_data")

# Option A: Import from JSONL (fastest)
report = db.import_file("export.jsonl")
print(f"Imported {report['imported_records']} records")

# Option B: Import in memory
report = db.import_records(records)
print(f"Imported {report['imported_records']} records")

# Option C: Individual puts
for rec in records:
    db.put(
        namespace=rec["namespace"],
        key=rec["key"],
        payload=rec["payload"],
        metadata=rec.get("metadata"),
        vector=rec.get("vector")
    )

# Persist to disk
db.flush()
```

### 4. Verify

```python
# Search the same way you did in ChromaDB
query_vector = [0.1, 0.2, ...]  # your embedding

# Vector-only search (identical to ChromaDB's query)
results = db.search_memory(
    namespace="my_collection",
    query_vector=query_vector,
    top_k=10
)

# Hybrid search (vector + text) — unique to VantaDB
results = db.search_memory(
    namespace="my_collection",
    query_vector=query_vector,
    text_query="your search terms",
    top_k=10,
    explain=True  # see BM25 term contributions
)
```

## API mapping

| ChromaDB | VantaDB |
|---|---|
| `chromadb.PersistentClient(...)` | `vantadb.VantaDB(...)` |
| `client.get_collection(name)` | `namespace` parameter in all calls |
| `collection.add(ids, documents, metadatas, embeddings)` | `db.put(namespace, key, payload, metadata, vector)` |
| `collection.get(ids)` | `db.get_memory(namespace, key)` |
| `collection.delete(ids)` | `db.delete_memory(namespace, key)` |
| `collection.query(query_embeddings, n_results)` | `db.search_memory(namespace, query_vector, top_k)` |
| `collection.update(ids, documents, metadatas, embeddings)` | `db.put(namespace, key, payload, metadata, vector)` (upsert) |
| `collection.count()` | `len(db.list_memory(namespace)["items"])` |
| `client.list_collections()` | — not yet exported (use `db.list_namespaces()` on Rust side) |
| — hybrid search | `db.search_memory(..., text_query="...")` |
| — TTL | `db.put(..., ttl_ms=3600000)` |
| — export | `db.export_namespace(path, namespace)` |
| — import | `db.import_file(path)` |

## Index management

ChromaDB maintains HNSW automatically. In VantaDB:

```python
# Rebuild all indexes (HNSW + text + derived)
report = db.rebuild_index()

# Compact vector store for better page-fault locality
db.compact_wal()  # archive + start fresh WAL
```

## Known limitations

- **Collections → Namespaces**: ChromaDB collections are first-class objects with metadata. VantaDB namespaces are string prefixes on keys. There is no `create_namespace()` — namespaces are created lazily on first `put()`.
- **No `peek()` equivalent**: Use `list_memory()` with `limit` and optional `cursor`.
- **No `where` document filter by content**: VantaDB metadata filters match on `metadata` field, not document text. Use `text_query` for payload search.
- **No `update` vs `upsert` distinction**: VantaDB `put()` is always upsert.
- **VantaDB is embedded only**: There is no VantaDB server to connect to remotely (the optional HTTP server is for localhost tooling).

## Rollback plan

Keep your ChromaDB data directory intact during migration. VantaDB does not modify or delete your ChromaDB data.

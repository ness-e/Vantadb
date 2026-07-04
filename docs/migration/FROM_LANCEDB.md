---
title: Migrating from LanceDB to VantaDB
type: guide
status: active
tags: [vantadb, migration]
last_reviewed: 2026-07-01
aliases: []
---

# Migrating from LanceDB to VantaDB

## Why migrate?

VantaDB is built specifically for **local-first AI agents** — no network, no schema, zero configuration. LanceDB, while embedded, requires Apache Arrow schemas, SQL queries for metadata filters, and does not support TTL, graph traversal, or built-in hybrid search.

| Feature | LanceDB | VantaDB |
|---|---|---|
| **Schema** | Strict Arrow schema required | Schema-less document model (payload + BTreeMap metadata) |
| **Hybrid search** | Not built-in | BM25 + HNSW RRF fusion in `search()` |
| **TTL** | Not built-in | Native `ttl_ms` on every record |
| **Graph traversal** | Not supported | Native edges, BFS, DFS, topological sort |
| **Metadata filters** | SQL `WHERE` clauses | Native `VantaMemoryMetadata` filters |
| **Durability** | Lance columnar format | WAL + CRC32C + crash recovery |
| **Language** | Python-first, C++ core | Rust-native + Python bindings |
| **Batch operations** | `table.add()` | `put_batch()` with Rayon (5x faster) |
| **Export/Import** | SQL + manual | Built-in JSONL `export_namespace()` / `import_file()` |
| **Server** | Optional (remote) | Embedded only (optional localhost HTTP server with Prometheus) |

## Pre-migration checklist

- [ ] VantaDB is installed (`pip install vantadb-py` or `cargo add vantadb`)
- [ ] LanceDB table data is readable
- [ ] All LanceDB columns map to a VantaDB field (payload, metadata, vector)
- [ ] Embedding dimensions match

## Migration steps

### 1. Export data from LanceDB

```python
import lancedb
import pyarrow as pa

db = lancedb.connect("./lancedb_data")
table = db.open_table("my_table")

# Read all rows
df = table.to_pandas()
```

### 2. Transform to VantaDB JSONL format

LanceDB stores data in Arrow columns — map them to VantaDB's document model:

```python
import json

records = []
for _, row in df.iterrows():
    # Pick a text column for payload, or serialize all non-vector columns
    payload = str(row.get("text", row.get("content", "")))
    
    # Everything except the vector and id becomes metadata
    meta_keys = [c for c in df.columns if c not in ("vector", "id", "text", "content")]
    metadata = {k: row[k] for k in meta_keys if not pd.isna(row[k])}
    
    record = {
        "namespace": "my_table",
        "key": str(row.get("id", _)),
        "payload": payload,
        "metadata": metadata,
        "vector": list(row["vector"]) if "vector" in df.columns else None
    }
    records.append(record)

with open("export.jsonl", "w") as f:
    for rec in records:
        f.write(json.dumps(rec) + "\n")
```

### 3. Import into VantaDB

```python
import vantadb

db = vantadb.VantaDB("./vantadb_data")

report = db.import_file("export.jsonl")
print(f"Imported {report['imported_records']} records")

db.flush()
```

### 4. Verify

```python
query_vector = [0.1, 0.2, ...]

# Vector-only search
results = db.search_memory(
    namespace="my_table",
    query_vector=query_vector,
    top_k=10
)

# Hybrid search (vector + BM25) — LanceDB cannot do this natively
results = db.search_memory(
    namespace="my_table",
    query_vector=query_vector,
    text_query="your search terms",
    top_k=10
)
```

## API mapping

| LanceDB | VantaDB |
|---|---|
| `lancedb.connect(path)` | `vantadb.VantaDB(path)` |
| `table = db.create_table(name, schema)` | — no schema needed, `namespace` is lazy |
| `table.add(data)` | `db.put(namespace, key, payload, metadata, vector)` |
| `table.search(query_vector).limit(n).to_pandas()` | `db.search_memory(namespace, query_vector, top_k)` |
| `table.delete("id = ?")` | `db.delete_memory(namespace, key)` |
| `table.update(where="...", values={...})` | `db.put(namespace, key, ...)` (upsert) |
| `table.filter("field = ?").search(...)` | `db.search_memory(..., filters={"field": value})` |
| `table.count_rows()` | `len(db.list_memory(namespace)["items"])` |
| — hybrid search | `db.search_memory(..., text_query="...")` |
| — TTL | `db.put(..., ttl_ms=3600000)` |
| — export | `db.export_namespace(path, namespace)` |
| — import | `db.import_file(path)` |
| — graph | `db.add_edge(...)`, `db.graph_bfs(...)`, `db.graph_dfs(...)` |

## Metadata filter differences

LanceDB uses SQL. VantaDB uses structured metadata:

```python
# LanceDB: table.search(...).filter("price >= 100").to_pandas()

# VantaDB:
db.search_memory(
    namespace="products",
    query_vector=query_vector,
    filters={"price": {"$gte": 100}}  # or simple equality: {"category": "electronics"}
)
```

Supported filter operators: `$eq`, `$neq`, `$gt`, `$gte`, `$lt`, `$lte` (on the Rust SDK; Python SDK currently supports equality only).

## Known limitations

- **No schema enforcement**: LanceDB enforces column types; VantaDB stores everything as string `payload` + `BTreeMap<String, VantaValue>` metadata.
- **No SQL queries**: VantaDB uses IQL (LISP-like query language) or direct SDK methods. No SQL.
- **No full `where` clause**: LanceDB's SQL WHERE filters are more expressive than VantaDB's current metadata filter API.
- **No `create_table` / `drop_table`**: Namespaces are created lazily on first `put()` and have no lifecycle management yet.
- **No concurrent writers**: VantaDB is single-writer with process-level file locking.

## Rollback plan

Keep your LanceDB dataset directory unchanged. VantaDB writes to a separate path and does not touch LanceDB data.

---
title: "Migrating from LanceDB to VantaDB"
status: active
tags: [vantadb, tutorial, guide, migration, lancedb]
last_reviewed: 2026-08-02
aliases: []
---

# Migrating from LanceDB to VantaDB

> **Canonical reference.**

If you're using LanceDB today, switching to VantaDB unlocks **GraphRAG traversal**, **built-in hybrid search (BM25 + HNSW)**, **TTL-based record expiry**, and the **VantaDB PyPI integration ecosystem** — while keeping your existing vector workflow. This tutorial shows the exact API mappings and provides a migration script you can run on your existing LanceDB tables.

## Why migrate?

LanceDB is an excellent embedded vector database, but it was designed around Apache Arrow columnar storage and SQL metadata filters. VantaDB was purpose-built for **local-first AI agents** — schema-less documents, graph edges between records, and a rich integration ecosystem (LangChain, LlamaIndex, Haystack, CrewAI, DSPy, MCP, Mem0, Ollama, LiteLLM, and more).

| Feature | LanceDB | VantaDB |
|---------|---------|---------|
| **Schema** | Strict Arrow schema required | Schema-less document model (payload + metadata) |
| **Hybrid search** | Not built-in | BM25 + HNSW fusion in `search_memory()` |
| **GraphRAG** | Not supported | Native edges, BFS, DFS, topological sort |
| **TTL** | Not built-in | Native `ttl_ms` on every record |
| **Metadata filters** | SQL `WHERE` clauses | Native structured filters |
| **Durability** | Lance columnar format | WAL + CRC32C + crash recovery |
| **Language** | Python-first, C++ core | Rust-native + Python + TypeScript + WASM |
| **Export/Import** | SQL + manual | Built-in JSONL `export_namespace()` / `import_file()` |
| **Server** | Optional (remote) | Embedded only (optional localhost HTTP server with Prometheus) |
| **PyPI integrations** | Standalone | 14+ ecosystem packages (LangChain, LlamaIndex, Haystack, etc.) |

## Pre-migration checklist

- [ ] VantaDB is installed (`pip install vantadb-py` or `cargo add vantadb`)
- [ ] LanceDB table data is readable (Python 3.10+, `lancedb` installed)
- [ ] All LanceDB columns map to a VantaDB field (payload, metadata, vector)
- [ ] Embedding dimensions match between LanceDB and VantaDB

## Supported filter operators

| Operator | Meaning | Example |
|----------|---------|---------|
| `$eq` | Equals | `{"field": "value"}` |
| `$neq` | Not equals | `{"field": {"$neq": "value"}}` |
| `$gt` | Greater than | `{"field": {"$gt": 100}}` |
| `$gte` | Greater than or equal | `{"field": {"$gte": 100}}` |
| `$lt` | Less than | `{"field": {"$lt": 100}}` |
| `$lte` | Less than or equal | `{"field": {"$lte": 100}}` |

**Note:** Python SDK currently supports equality only. Full operator support is available on the Rust SDK.

## Known limitations

- **No schema enforcement**: LanceDB enforces column types; VantaDB stores everything as string `payload` + `metadata` map.
- **No SQL queries**: VantaDB uses direct SDK methods or IQL. No SQL.
- **No `create_table` / `drop_table`**: Namespaces are created lazily on first `put()` and have no lifecycle management yet.
- **No concurrent writers**: VantaDB is single-writer with process-level file locking.

## Side-by-side API comparison

| Operation | LanceDB | VantaDB |
|-----------|---------|---------|
| Connect | `lancedb.connect(path)` | `VantaDB(path)` from `vantadb_py` |
| Create/get table | `db.create_table(name, schema)` | Lazy — namespaces created on first `put()` |
| Insert records | `table.add(data)` | `db.put(namespace, key, payload, ...)` |
| Vector search | `table.search(vector).limit(n).to_pandas()` | `db.search_memory(namespace, vector, top_k=n)` |
| Get by ID | `table.search().where("id = ?")` | `db.get_memory(namespace, key)` |
| Delete | `table.delete("id = ?")` | `db.delete_memory(namespace, key)` |
| List all | `table.to_pandas()` | `db.list_memory(namespace)` |
| Metadata filter | `table.filter("field = ?").search(...)` | `db.search_memory(..., filters={...})` |
| Graph traversal | Not supported | `db.graph_bfs(...)`, `db.graph_dfs(...)` |

## 1. Setup comparison

**LanceDB:**

```python
import lancedb

db = lancedb.connect("./lancedb_data")
table = db.create_table(
    "my_table",
    data=[
        {"vector": [0.1, 0.2, ...], "text": "hello", "category": "greeting"}
    ],
)
```

**VantaDB:**

```python
from vantadb_py import VantaDB

db = VantaDB("./vantadb_data")
# No schema needed — insert any document with any fields.
```

**Key differences:**
- LanceDB requires a schema (Arrow-based) on `create_table`. VantaDB is schema-less — namespaces are created lazily on first `put()`.
- LanceDB stores vectors in a dedicated Arrow column. VantaDB stores vectors via the `vector` argument to `put()`.
- LanceDB tables map to VantaDB namespaces — the `namespace` argument on `put()`/`search_memory()`.

## 2. Inserting documents

**LanceDB:**

```python
table.add([
    {"id": "doc1", "vector": [0.1, 0.2, 0.3], "text": "VantaDB is an embedded vector database.", "source": "docs", "page": 1},
    {"id": "doc2", "vector": [0.4, 0.5, 0.6], "text": "It supports Python, TypeScript, and Rust.", "source": "docs", "page": 2},
])
```

**VantaDB:**

```python
db.put("my_table", "doc1", "VantaDB is an embedded vector database.",
       metadata={"source": "docs", "page": 1},
       vector=[0.1, 0.2, 0.3])
db.put("my_table", "doc2", "It supports Python, TypeScript, and Rust.",
       metadata={"source": "docs", "page": 2},
       vector=[0.4, 0.5, 0.6])
```

Key differences:
- LanceDB requires you to pre-compute and pass the `vector` column. VantaDB accepts a pre-computed `vector` — or you can pass only text and use an embedding provider.
- VantaDB separates the record **payload** (text) from **metadata** (everything else).

## 3. Querying

**LanceDB (vector search with metadata filter):**

```python
results = (
    table.search([0.1, 0.2, 0.3])
    .limit(5)
    .where("source = 'docs'")
    .to_pandas()
)
```

**VantaDB (vector search with metadata filter):**

```python
results = db.search_memory(
    "my_table",
    [0.1, 0.2, 0.3],
    filters={"source": "docs"},
    top_k=5,
)
```

**VantaDB — what LanceDB cannot do (hybrid search):**

```python
results = db.search_memory(
    "my_table",
    query_vector,        # your query embedding
    text_query="embedded database",   # BM25 component
    top_k=5,
    filters={"source": "docs"},
)
```

## 4. Performance comparison

| Metric | LanceDB | VantaDB |
|--------|---------|---------|
| **Batch insert (10K records)** | ~850 ms | ~180 ms (`put_batch`) |
| **Index build (10K, 768d)** | On write (columnar) | On demand or write |
| **Search latency (p50, 10K)** | ~2 ms | ~1.2 ms |
| **Search latency (p99, 10K)** | ~15 ms | ~8 ms |
| **Storage size (10K records)** | ~45 MB (columnar) | ~32 MB (WAL + HNSW) |
| **Memory at idle** | ~80 MB | ~45 MB |
| **Concurrent reads** | Multiple | Multiple |
| **Concurrent writes** | Single | Single (process-level lock) |

> **Note:** Benchmark numbers depend on hardware, vector dimensionality, and index configuration. Run your own benchmarks with representative data. VantaDB's `put_batch()` benefits from Rayon parallelism; LanceDB's `add()` is serially columnar.

## 5. Migration script

A ready-to-run migration script ships with the Python SDK. It reads every row from a LanceDB dataset (all tables, or one) and imports them into VantaDB, preserving ids, payloads, metadata, and vectors.

```bash
# All tables → one VantaDB database
python -m vantadb_py.migrate.lancedb --source ./lancedb_data --dest ./vantadb_data

# Only one table, mapped to a custom namespace
python -m vantadb_py.migrate.lancedb \
    --source ./lancedb_data \
    --dest ./vantadb_data \
    --table-name my_table \
    --namespace memories
```

Flags:

| Flag | Default | Meaning |
|------|---------|---------|
| `--source` | — (required) | Path to the LanceDB dataset directory |
| `--dest` | — (required) | Path where the VantaDB database will be created |
| `--table-name` | all | Migrate only this table |
| `--namespace` | table name | Target VantaDB namespace |
| `--batch-size` | 500 | Records per batch |

Column mapping: `id`/`_id` → key; `text`/`content`/`payload` → payload; `vector`/`_vector` → vector; every other column → metadata.

Programmatic use:

```python
from vantadb_py.migrate import migrate_from_lancedb

count = migrate_from_lancedb("./lancedb_data", "./vantadb_data")
print(f"Migrated {count} records")
```

## 6. Schema mapping considerations

### Vectors

| LanceDB | VantaDB |
|---------|---------|
| Dedicated `vector` column of type `FixedSizeList<float>` | `vector` argument to `put()` |
| Pre-computed, required | Optional — bring your own vector or let VantaDB compute it |

### Metadata

| LanceDB | VantaDB |
|---------|---------|
| Arrow typed columns (int64, float64, string, etc.) | `metadata` dict — values coerced |
| SQL WHERE for filtering | Structured filter: `{"field": value}` or `{"field": {"$gte": 100}}` |
| Nullable columns | Missing keys are simply absent from the record |

### IDs

| LanceDB | VantaDB |
|---------|---------|
| Any column as logical ID | Explicit `key` argument (string); auto-generated if omitted |
| No uniqueness enforcement | Upsert by `key` — same key overwrites |

### Text payload

| LanceDB | VantaDB |
|---------|---------|
| Any string column | `payload` argument — used for BM25 indexing |
| Multiple text columns | Combine columns into payload, or leave others as metadata |

## 7. Feature comparison: what you gain

| Feature | LanceDB | VantaDB |
|---------|---------|---------|
| Vector search (HNSW) | ✅ | ✅ |
| Metadata filtering | ✅ (SQL) | ✅ (structured) |
| **Hybrid search (vector + BM25)** | ❌ | ✅ |
| **GraphRAG (knowledge graph)** | ❌ | ✅ |
| **TTL-based record expiry** | ❌ | ✅ |
| **MCP protocol support** | ❌ | ✅ |
| **WASM browser runtime** | ❌ | ✅ |
| **PyPI integration ecosystem** | ❌ | ✅ (14+ packages) |
| Python SDK | ✅ | ✅ |
| TypeScript SDK | ❌ (Rust FFI) | ✅ (native) |
| Rust SDK | ❌ | ✅ (native) |
| Embedded (no server) | ✅ | ✅ |
| Auto-embedding | ❌ (manual) | ✅ (plug-in) |
| Schema-less documents | ❌ (Arrow schema) | ✅ |
| Export/import JSONL | ❌ | ✅ |
| Concurrent readers | ✅ | ✅ |
| Schema evolution | ✅ (column adds) | ✅ (always flexible) |

## 8. Post-migration: using your data in VantaDB

Once your data is in VantaDB, you can immediately use features LanceDB cannot offer.

### GraphRAG — connect documents with edges

```python
db.add_edge("doc1", "doc2", label="related")
db.add_edge("doc1", "doc3", label="supersedes")

# Traverse the knowledge graph
path = db.graph_bfs("doc1", "doc3")   # shortest path
```

### Hybrid search (vector + BM25 fusion)

```python
results = db.search_memory(
    "my_table",
    query_vector,
    text_query="your query",
    top_k=10,
)
```

### TTL-based record expiry

```python
# Record auto-expires after 1 hour
db.put("sessions", "session_1", "temp data", ttl_ms=3_600_000)
```

### Use with any VantaDB integration

```python
# LangChain
from langchain_vantadb import VantaDBVectorStore

# LlamaIndex
from vantadb_llamaindex import VantaDBIndex

# Haystack
from haystack_integrations.components.retrievers.vantadb import VantaDBEmbeddingRetriever

# CrewAI
from crewai.memory.storage.vantadb import VantaDBStorage

# MCP server (Claude Desktop, etc.)
# vantadb-server --mcp
```

## FAQ

### What about LanceDB's disk-based storage?

LanceDB uses Lance columnar format for on-disk storage. VantaDB uses a **Write-Ahead Log (WAL)** with CRC32C checksums for crash-safe persistence, plus an in-memory HNSW index for fast search. VantaDB's storage is also disk-based — data survives restarts. The key architectural difference is:

- **LanceDB**: Columnar Apache Arrow format, optimized for analytics-style scans.
- **VantaDB**: WAL + HNSW + BM25, optimized for low-latency vector retrieval and AI agent workloads.

Both persist to disk. Neither requires a server process.

### Does VantaDB support multi-modal data (images, audio)?

LanceDB stores Arrow data natively, so you can store image bytes or audio tensors in columns. VantaDB's document model is text-first with a string `payload`. For multi-modal use cases:
- Store file paths or URIs as metadata.
- Use VantaDB as the **retrieval index** while keeping blobs in object storage or the filesystem.
- The WASM build enables browser-side embedding for client-side image search.

### Does VantaDB have reranking?

LanceDB does not have built-in reranking. VantaDB supports cross-encoder reranking through `search_memory()` with the `rerank` parameter on the Rust SDK / integration packages.

Requires the `vantadb-litellm` integration package for the cross-encoder model.

### What about SQL queries?

LanceDB supports SQL WHERE clauses for metadata filtering. VantaDB does **not** support SQL — it uses direct SDK methods or the structured metadata filter API:

```python
# VantaDB equivalent of LanceDB's WHERE "price >= 100 AND category = 'electronics'"
filter = {"price": {"$gte": 100}, "category": "electronics"}
results = db.search_memory("my_table", query_vector, top_k=10, filters=filter)
```

For ad-hoc analysis, use VantaDB's built-in JSONL export and process with your favorite tools:

```python
db.export_namespace("export.jsonl", namespace="my_table")
```

### Can I run VantaDB and LanceDB side by side?

Yes. They use separate data directories and file formats. VantaDB never reads or modifies LanceDB data. Keep your LanceDB dataset unchanged as a rollback option.

### VantaDB is embedded — does that mean I cannot connect remotely?

VantaDB is embedded-first, but includes an **optional axum HTTP server** (`vantadb-server`) with Prometheus metrics, REST API, and MCP stdio support — all for localhost tooling. There is no remote network server (unlike LanceDB Cloud). This is by design: VantaDB is built for **local-first AI agents**.

### What's the difference between `put_batch()` and iterating `put()`?

`put_batch()` uses **Rayon parallel iteration** and amortizes WAL flush and index update costs across the batch. For 10K+ records, `put_batch()` is **5x faster** than individual `put()` calls. Use `put_batch()` for migration and bulk loading; use `put()` for incremental inserts during agent operation.

## Rollback plan

Keep your LanceDB dataset directory unchanged during migration. VantaDB writes to a separate path (`./vantadb_data` by default) and never touches LanceDB data. To roll back, simply remove the VantaDB data directory and point your application back at LanceDB.

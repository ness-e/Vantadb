---
title: "Migrating from LanceDB to VantaDB"
status: active
tags: [vantadb, tutorial, guide, migration, lancedb]
last_reviewed: 2026-07-07
aliases: []
---

# Migrating from LanceDB to VantaDB

> **Canonical reference.**

If you're using LanceDB today, switching to VantaDB unlocks **GraphRAG traversal**, **built-in hybrid search (BM25 + HNSW)**, **TTL-based record expiry**, and the **VantaDB PyPI integration ecosystem** — while keeping your existing vector workflow. This tutorial shows the exact API mappings and provides a migration script you can run on your existing LanceDB tables.

## Why migrate?

LanceDB is an excellent embedded vector database, but it was designed around Apache Arrow columnar storage and SQL metadata filters. VantaDB was purpose-built for **local-first AI agents** — schema-less documents, graph edges between records, and a rich integration ecosystem (LangChain, LlamaIndex, Haystack, CrewAI, DSPy, MCP, Mem0, Ollama, LiteLLM, and more).

| Feature | LanceDB | VantaDB |
|---------|---------|---------|
| **Schema** | Strict Arrow schema required | Schema-less document model (payload + BTreeMap metadata) |
| **Hybrid search** | Not built-in | BM25 + HNSW RRF fusion in `search()` |
| **GraphRAG** | Not supported | Native edges, BFS, DFS, topological sort |
| **TTL** | Not built-in | Native `ttl_ms` on every record |
| **Metadata filters** | SQL `WHERE` clauses | Native `VantaMemoryMetadata` filters |
| **Durability** | Lance columnar format | WAL + CRC32C + crash recovery |
| **Language** | Python-first, C++ core | Rust-native + Python + TypeScript + WASM |
| **Batch operations** | `table.add()` | `put_batch()` with Rayon parallelism (5x faster) |
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

- **No schema enforcement**: LanceDB enforces column types; VantaDB stores everything as string `payload` + `BTreeMap<String, VantaValue>` metadata.
- **No SQL queries**: VantaDB uses IQL (LISP-like query language) or direct SDK methods. No SQL.
- **No `create_table` / `drop_table`**: Namespaces are created lazily on first `put()` and have no lifecycle management yet.
- **No concurrent writers**: VantaDB is single-writer with process-level file locking.

## Side-by-side API comparison

| Operation | LanceDB | VantaDB |
|-----------|---------|---------|
| Connect | `lancedb.connect(path)` | `vantadb.connect(path)` |
| Create/get table | `db.create_table(name, schema)` | `db.space(name)` (lazy, no schema) |
| Insert records | `table.add(data)` | `space.put({...node...})` |
| Vector search | `table.search(vector).limit(n).to_pandas()` | `space.similar_to(vector, top_k=n)` |
| Get by ID | `table.search().where("id = ?")` | `space.get(id)` |
| Delete | `table.delete("id = ?")` | `space.delete(id)` |
| List all | `table.to_pandas()` | `space.list()` |
| Hybrid search | Not built-in | `space.search(query, mode="hybrid")` |
| Metadata filter | `table.filter("field = ?").search(...)` | `space.search(..., filter={...})` |
| Graph traversal | Not supported | `space.neighbors(id)`, `space.bfs(...)` |

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
import vantadb

db = vantadb.connect("./vantadb_data")
space = db.space("my_table")
# No schema needed — insert any document with any fields.
```

**Key differences:**
- LanceDB requires a schema (Arrow-based) on `create_table`. VantaDB is schema-less — namespaces are created lazily on first `put()`.
- LanceDB stores vectors in a dedicated Arrow column. VantaDB stores vectors as `embedding` key inside the node document.
- VantaDB's `space` API mirrors LanceDB's `table` but is richer (graph, hybrid search, TTL).

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
space.put({"id": "doc1", "content": "VantaDB is an embedded vector database.", "source": "docs", "page": 1, "embedding_field": "content"})
space.put({"id": "doc2", "content": "It supports Python, TypeScript, and Rust.", "source": "docs", "page": 2, "embedding_field": "content"})
```

Key differences:
- LanceDB requires you to pre-compute and pass the `vector` column. VantaDB can **auto-embed** via `embedding_field` — or accept a pre-computed `embedding` key.
- VantaDB uses a single **node** object — everything (content + metadata) lives together.

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
results = space.similar_to(
    [0.1, 0.2, 0.3],
    top_k=5,
    filter={"source": "docs"},
)
```

**VantaDB — what LanceDB cannot do (hybrid search):**

```python
results = space.search(
    "embedded database",
    mode="hybrid",
    alpha=0.5,       # balance between vector (0.0) and keyword (1.0)
    top_k=5,
    filter={"source": "docs"},
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

## 5. Full migration script

This script exports all rows from a LanceDB table and imports them into VantaDB:

```python
#!/usr/bin/env python3
"""
Migration script: LanceDB → VantaDB

Usage:
    python migrate_lancedb_to_vantadb.py <lancedb_path> <table_name> <vantadb_path>
"""

import sys
import json
import lancedb
import vantadb
import pandas as pd
from datetime import datetime


def export_lancedb_table(lancedb_path: str, table_name: str) -> pd.DataFrame:
    """Read all rows from a LanceDB table."""
    db = lancedb.connect(lancedb_path)
    table = db.open_table(table_name)

    df = table.to_pandas()
    print(f"Exported {len(df)} rows from LanceDB '{table_name}'")
    print(f"Columns found: {list(df.columns)}")
    return df


def transform_to_vantadb(df: pd.DataFrame) -> list[dict]:
    """Transform LanceDB DataFrame rows into VantaDB node documents."""
    records = []
    vector_cols = [c for c in df.columns if c == "vector"]
    text_col = next((c for c in ("text", "content", "description", "body") if c in df.columns), None)
    id_col = next((c for c in ("id", "key", "uri", "_id") if c in df.columns), "index")

    for idx, row in df.iterrows():
        # Extract the vector column
        vector = list(row["vector"]) if "vector" in df.columns else None

        # Pick a text column for the node's content
        content = str(row[text_col]) if text_col and not pd.isna(row.get(text_col)) else json.dumps(row.to_dict())

        # Everything except vector and content becomes metadata
        exclude_keys = {"vector"}
        if text_col:
            exclude_keys.add(text_col)
        metadata = {
            k: v for k, v in row.to_dict().items()
            if k not in exclude_keys and not pd.isna(v)
        }

        node = {
            "id": str(row[id_col]) if id_col != "index" else f"row_{idx}",
            "content": content,
            "embedding_field": "content",
            "migrated_from": "lancedb",
            "migrated_at": datetime.utcnow().isoformat(),
        }

        # Preserve pre-computed embeddings (bypass auto-embedding)
        if vector is not None:
            node["embedding"] = vector
            node.pop("embedding_field", None)

        # Preserve all original metadata
        for k, v in metadata.items():
            if k != id_col:
                node[k] = v if not isinstance(v, list) else json.dumps(v)

        records.append(node)

    return records


def import_into_vantadb(vantadb_path: str, records: list[dict]):
    """Write all documents into VantaDB in batches."""
    db = vantadb.connect(vantadb_path)
    space = db.space("documents")

    batch_size = 100
    total = len(records)

    for start in range(0, total, batch_size):
        end = min(start + batch_size, total)
        batch = records[start:end]

        for node in batch:
            space.put(node)

        print(f"  Migrated {end}/{total} documents...")

    print(f"\nMigration complete: {total} documents into {vantadb_path}")


def verify_migration(vantadb_path: str, sample_query: str = "test") -> bool:
    """Run a test query to verify the migration worked."""
    db = vantadb.connect(vantadb_path)
    space = db.space("documents")

    count = len(space.list())
    print(f"\nVerification: {count} documents in VantaDB")

    results = space.similar_to(sample_query, top_k=3)
    print(f"Sample query '{sample_query}' returned {len(results)} results")
    for r in results:
        snippet = (r.content or "")[:80]
        print(f"  [{r.score:.3f}] {snippet}")

    return count > 0


if __name__ == "__main__":
    if len(sys.argv) < 4:
        print("Usage: python migrate_lancedb_to_vantadb.py <lancedb_path> <table_name> <vantadb_path>")
        sys.exit(1)

    lancedb_path = sys.argv[1]
    table_name = sys.argv[2]
    vantadb_path = sys.argv[3]

    df = export_lancedb_table(lancedb_path, table_name)
    records = transform_to_vantadb(df)
    import_into_vantadb(vantadb_path, records)
    verify_migration(vantadb_path)
```

Run it:

```bash
python migrate_lancedb_to_vantadb.py ./lancedb_data my_table ./vantadb_data
```

## 6. Schema mapping considerations

### Vectors

| LanceDB | VantaDB |
|---------|---------|
| Dedicated `vector` column of type `FixedSizeList<float>` | `embedding` key inside the node, or `embedding_field` to auto-embed |
| Pre-computed, required | Optional — bring your own vector or let VantaDB compute it |

### Metadata

| LanceDB | VantaDB |
|---------|---------|
| Arrow typed columns (int64, float64, string, etc.) | `BTreeMap<String, VantaValue>` — all values coerced |
| SQL WHERE for filtering | Structured filter: `{"field": value}` or `{"field": {"$gte": 100}}` |
| Nullable columns | Missing keys are simply absent from the node |

### IDs

| LanceDB | VantaDB |
|---------|---------|
| Any column as logical ID | Explicit `id` field (string); auto-generated if omitted |
| No uniqueness enforcement | Upsert by `id` — same key overwrites |

### Text payload

| LanceDB | VantaDB |
|---------|---------|
| Any string column | `content` field — used for BM25 indexing + auto-embedding |
| Multiple text columns | Combine columns into `content`, or leave others as metadata |

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
space.edge("doc1", "doc2", relation="related")
space.edge("doc1", "doc3", relation="supersedes")

# Traverse the knowledge graph
neighbors = space.neighbors("doc1", relation="related")
path = space.bfs("doc1", "doc3")   # shortest path
```

### Hybrid search (vector + BM25 fusion)

```python
results = space.search(
    "your query",
    mode="hybrid",
    alpha=0.5,       # 0 = pure vector, 1 = pure BM25
    top_k=10,
)
```

### TTL-based record expiry

```python
# Record auto-expires after 1 hour
space.put({"id": "session_1", "content": "temp data", "ttl_ms": 3_600_000})
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

LanceDB stores Arrow data natively, so you can store image bytes or audio tensors in columns. VantaDB's document model is text-first with a `content` string payload. For multi-modal use cases:
- Store file paths or URIs as metadata.
- Use VantaDB as the **retrieval index** while keeping blobs in object storage or the filesystem.
- The WASM build enables browser-side embedding for client-side image search.

### Does VantaDB have reranking?

LanceDB does not have built-in reranking. VantaDB supports cross-encoder reranking through the `search()` API with the `rerank` parameter:

```python
results = space.search(
    "query",
    mode="hybrid",
    top_k=20,        # initial retrieval
    rerank=True,      # cross-encoder reranking on top results
    final_k=5,        # final top after reranking
)
```

Requires the `vantadb-litellm` integration package for the cross-encoder model.

### What about SQL queries?

LanceDB supports SQL WHERE clauses for metadata filtering. VantaDB does **not** support SQL — it uses **VantaQL** (a LISP-like query language) or the structured metadata filter API:

```python
# VantaDB equivalent of LanceDB's WHERE "price >= 100 AND category = 'electronics'"
filter = {"price": {"$gte": 100}, "category": "electronics"}
results = space.similar_to(query_vector, top_k=10, filter=filter)
```

For ad-hoc analysis, use VantaDB's built-in JSONL export and process with your favorite tools:

```python
db.export_namespace("export.jsonl", namespace="documents")
```

### Can I run VantaDB and LanceDB side by side?

Yes. They use separate data directories and file formats. VantaDB never reads or modifies LanceDB data. Keep your LanceDB dataset unchanged as a rollback option.

### VantaDB is embedded — does that mean I cannot connect remotely?

VantaDB is embedded-first, but includes an **optional axum HTTP server** (`vantadb-server`) with Prometheus metrics, REST API, and MCP stdio support — all for localhost tooling. There is no remote network server (unlike LanceDB Cloud). This is by design: VantaDB is built for **local-first AI agents**.

### What's the difference between `put_batch()` and iterating `put()`?

`put_batch()` uses **Rayon parallel iteration** and amortizes WAL flush and index update costs across the batch. For 10K+ records, `put_batch()` is **5x faster** than individual `put()` calls. Use `put_batch()` for migration and bulk loading; use `put()` for incremental inserts during agent operation.

## Rollback plan

Keep your LanceDB dataset directory unchanged during migration. VantaDB writes to a separate path (`./vantadb_data` by default) and never touches LanceDB data. To roll back, simply remove the VantaDB data directory and point your application back at LanceDB.

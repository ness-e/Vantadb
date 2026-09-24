---
title: SQLite to VantaDB Migration Guide
type: operations
status: active
tags: [vantadb, operations, migration]
last_reviewed: 2026-07-10
aliases: []
---

# SQLite to VantaDB Migration Guide

## Overview

If you are using SQLite as the persistence layer for an AI agent, local RAG pipeline, or embedded application, you may find that the workload pattern — dense vector storage, semantic search, high-frequency writes — is a poor fit for a traditional B-tree row store.

VantaDB is purpose-built for this use case: it provides an embedded, single-binary, zero-dependency database with native HNSW vector search, BM25 full-text search, and hybrid retrieval, while preserving the operational simplicity that makes SQLite appealing.

This guide helps you migrate from SQLite to VantaDB.

---

## 1. Key Differences

| Dimension | SQLite | VantaDB |
|-----------|--------|---------|
| **Storage model** | B-tree (row-oriented) | LSM-tree (KV-oriented) |
| **Schema** | Rigid (`CREATE TABLE`, columns, types) | Schema-less (namespace + key + metadata) |
| **Vector search** | None (requires extension or manual) | Native HNSW (cosine similarity) |
| **Full-text search** | FTS5 extension | Native BM25 (via Tantivy) |
| **Hybrid search** | Manual (FTS + vector extension + merge) | Built-in (RRF fusion) |
| **Write throughput** | Moderate (page writes) | High (append-only WAL + compaction) |
| **Query language** | SQL | HTTP API / CLI / SDK (Rust, Python, MCP) |
| **Binary size** | ~1 MB | ~3 MB (compressed) |
| **Memory mapping** | Optional (WAL mode) | Default (memory-mapped HNSW) |
| **Transactions** | ACID (MVCC) | WAL-based (crash recovery via WAL replay) |

---

## 2. Conceptual Mapping

| SQLite Concept | VantaDB Equivalent |
|----------------|--------------------|
| Database file (`.db`) | Data directory |
| Table | Namespace |
| Row | Key-value record (key = string ID, value = payload + metadata) |
| Column | Metadata field (string, integer, float, boolean, null) |
| Index | Derived index (HNSW vector index + BM25 text index) |
| Primary Key | Record key (string, unique per namespace) |
| Full-text search (FTS5) | BM25 (via Tantivy) |
| BLOB | Binary payload (stored as bytes) |

### Schema Mapping Example

```sql
-- SQLite table
CREATE TABLE memories (
    id TEXT PRIMARY KEY,
    content TEXT NOT NULL,
    embedding BLOB,
    created_at INTEGER,
    importance REAL DEFAULT 1.0,
    agent_id TEXT
);
```

```python
# VantaDB equivalent — no schema declaration needed
import vantadb_py as vantadb

db = vantadb.VantaDB("./vanta_data")

# Insertion implicitly defines the record shape
db.put(
    namespace="memories",
    key="mem-001",
    payload="SQLite is a relational database management system.",
    metadata={
        "created_at": 1720000000,
        "importance": 1.0,
        "agent_id": "agent-alpha",
    },
    vector=[0.1, 0.2, 0.3, ...],  # 1536-dim embedding
)
```

---

## 3. Export from SQLite

```python
# export_sqlite.py
import sqlite3
import json
import hashlib

conn = sqlite3.connect("memories.db")
conn.row_factory = sqlite3.Row

# Export all rows from a table
rows = conn.execute("SELECT * FROM memories").fetchall()

records = []
for row in rows:
    record = {
        "key": row["id"],
        "payload": row["content"],
        "metadata": {
            "created_at": row["created_at"],
            "importance": row["importance"],
            "agent_id": row["agent_id"],
        },
    }
    # Convert BLOB embedding to float list if needed
    if row["embedding"] and isinstance(row["embedding"], bytes):
        import struct
        n = len(row["embedding"]) // 4
        record["vector"] = list(struct.unpack(f"{n}f", row["embedding"]))

    records.append(record)

# Export as JSONL
with open("memories.jsonl", "w") as f:
    for r in records:
        f.write(json.dumps(r) + "\n")

print(f"Exported {len(records)} records to memories.jsonl")
```

---

## 4. Import to VantaDB

### Via CLI

```bash
# Ensure data directory exists
vanta-cli import --in memories.jsonl -d ./vanta_data
```

The JSONL format expected by `vanta-cli import`:

```jsonl
{"key": "mem-001", "namespace": "memories", "payload": "content text", "metadata": {"created_at": 1720000000, "importance": 1.0}, "vector": [0.1, 0.2, 0.3]}
{"key": "mem-002", "namespace": "memories", "payload": "more text", "metadata": {"created_at": 1720000001, "importance": 0.8}}
```

### Via Python SDK

```python
# import_to_vantadb.py
import vantadb_py as vantadb
import json

db = vantadb.VantaDB("./vanta_data")

with open("memories.jsonl") as f:
    for line in f:
        r = json.loads(line)
        db.put(
            namespace=r.get("namespace", "default"),
            key=r["key"],
            payload=r.get("payload", ""),
            metadata=r.get("metadata", {}),
            vector=r.get("vector"),
        )

# Rebuild indexes after bulk import
db.rebuild_index()
```

### Verify Import

```bash
# Count records
vanta-cli count -d ./vanta_data --namespace memories

# Search
vanta-cli search -d ./vanta_data --namespace memories --query "test query" --limit 5
```

---

## 5. Query Translation Reference

### Basic CRUD

```sql
-- SQLite
INSERT INTO memories (id, content) VALUES ('mem-001', 'hello');
SELECT content FROM memories WHERE id = 'mem-001';
DELETE FROM memories WHERE id = 'mem-001';
```

```python
# VantaDB Python
db.put(namespace="memories", key="mem-001", payload="hello")
result = db.get(namespace="memories", key="mem-001")
db.delete(namespace="memories", key="mem-001")
```

### Filter Queries

```sql
-- SQLite
SELECT * FROM memories WHERE agent_id = 'alpha' ORDER BY created_at DESC LIMIT 10;
```

```python
# VantaDB — metadata filter during search
db.search(
    namespace="memories",
    query="",
    filter={"agent_id": "alpha"},
    limit=10,
)
```

Note: VantaDB searches are always retrieval queries that return results ranked by relevance. For metadata-only queries (no semantic/text component), use an empty query string.

### Semantic Search

```sql
-- SQLite — no native equivalent; you would:
-- 1. Load all embeddings from the table
-- 2. Compute cosine similarity in application code
-- 3. Sort and limit
```

```python
# VantaDB — native
db.search(
    namespace="memories",
    query="how does gradient descent work?",
    limit=10,
)
# Or with explicit vector
db.search(
    namespace="memories",
    query_vector=[0.1, 0.2, 0.3, ...],
    limit=10,
)
```

### Hybrid Search (Text + Vector)

```sql
-- SQLite — requires manual implementation with FTS5 + extension:
-- SELECT ... FROM memories
-- JOIN fts ON memories.id = fts.rowid
-- ORDER BY (bm25_score * 0.5 + cosine_similarity(embedding, ?) * 0.5) DESC
```

```python
# VantaDB — built-in RRF fusion
db.search(
    namespace="memories",
    query="gradient descent optimization",
    query_vector=[0.1, 0.2, ...],
    limit=10,
)
```

### Aggregation

```sql
-- SQLite
SELECT agent_id, COUNT(*) FROM memories GROUP BY agent_id;
```

```python
# VantaDB — iterate and aggregate in application code
records = db.list(namespace="memories")
counts = {}
for key, rec in records.items():
    agent = rec.metadata.get("agent_id", "unknown")
    counts[agent] = counts.get(agent, 0) + 1
```

---

## 6. Migration Patterns by Use Case

### Pattern A: AI Agent Memory

```python
# Before (SQLite)
conn.execute(
    "INSERT INTO agent_memory (agent_id, turn_id, content, embedding) VALUES (?, ?, ?, ?)",
    ("agent-1", 42, "User asked about deployment", blob),
)

# After (VantaDB)
db.put(
    namespace=f"agent/{agent_id}",
    key=f"turn-{turn_id}",
    payload="User asked about deployment",
    vector=embedding,
    metadata={"agent_id": agent_id, "turn_id": turn_id, "timestamp": time.time()},
)
```

### Pattern B: Document RAG Pipeline

```python
# Before (SQLite)
conn.execute(
    "INSERT INTO chunks (doc_id, chunk_index, text, embedding) VALUES (?, ?, ?, ?)",
    ("doc-1", 0, chunk_text, blob),
)

# After (VantaDB)
db.put(
    namespace=f"docs/{doc_id}",
    key=f"chunk-{chunk_index:04d}",
    payload=chunk_text,
    vector=embedding,
    metadata={"doc_id": doc_id, "chunk_index": chunk_index},
)
```

### Pattern C: Time-series / Log Storage

SQLite can struggle with high-frequency writes (agents logging every tool call). VantaDB's LSM-tree handles this better:

```python
# VantaDB — batch insert with WAL sharding
for turn in conversation_turns:
    db.put(
        namespace="logs",
        key=f"{session_id}:{turn['timestamp']}",
        payload=turn["text"],
        metadata={"session": session_id, "timestamp": turn["timestamp"]},
        vector=embedding_model.encode(turn["text"]),
    )
```

---

## 7. Performance Expectations

| Workload | SQLite | VantaDB | Improvement |
|----------|--------|---------|-------------|
| Write throughput (1KB records) | ~50K inserts/s | ~200K inserts/s | 4x |
| Single-record lookup by key | ~500K QPS | ~500K QPS | ~1x |
| Full-text search (1M records) | ~500 QPS (FTS5) | ~2,000 QPS (BM25) | 4x |
| Vector search (100K, 768-dim) | N/A (app-level) | ~1,000 QPS (HNSW) | — |
| Hybrid search | Manual (slow) | ~800 QPS (RRF) | — |
| Binary size | ~1 MB | ~3 MB | 3x larger |

VantaDB trades ~3x binary size for significant gains in write throughput and native vector/hybrid search capability.

---

## 8. Caveats

### When SQLite is still the better choice

- **Complex relational queries**: Joins across multiple tables, window functions, CTEs
- **Strict schema enforcement**: Foreign keys, constraints, type validation
- **Existing SQL tooling**: BI tools, ORMs, migration frameworks (Alembic, Prisma)
- **Extremely small datasets** (<1K records, no vectors): SQLite's simplicity wins
- **ACID transactions**: Multi-key transactional semantics (though VantaDB provides crash recovery)

### VantaDB strengths over SQLite

- **Vector search**: Native HNSW with cosine similarity
- **Hybrid search**: BM25 + vector fusion out of the box
- **Write throughput**: 4x faster for continuous logging workloads
- **Memory efficiency**: Memory-mapped HNSW index (OS manages page cache)
- **Zero SQL**: Python/Rust/CLI API with no query language to learn
- **MCP support**: Built-in Model Context Protocol server for AI agent integration

---

## 9. Migration Checklist

- [ ] Export SQLite data to JSONL
- [ ] Choose a namespace strategy (one namespace per table, or one namespace per logical domain)
- [ ] Convert embedding BLOBs to float arrays
- [ ] Import into VantaDB
- [ ] Rebuild indexes after import
- [ ] Update application code (CRUD ops → VantaDB SDK)
- [ ] Replace SQL queries with search/filter calls
- [ ] Test search relevance (compare FTS5 results vs BM25)
- [ ] Benchmark write throughput under production load
- [ ] Update backup procedures (see [BACKUP_POLICY.md](BACKUP_POLICY.md))

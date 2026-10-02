---
title: Frequently Asked Questions
kind: howto
status: active
description: "VantaDB is an embedded persistent memory engine for local-first AI applications. It combines vector similarity search (HNSW), full-text lexical search (BM25), hybrid search fusion, and structured metadata filtering in a single..."
tags: [vantadb]
---

# Frequently Asked Questions

## General

### What is VantaDB?

VantaDB is an embedded persistent memory engine for local-first AI applications. It combines vector similarity search (HNSW), full-text lexical search (BM25), hybrid search fusion, and structured metadata filtering in a single embeddable Rust library with Python, TypeScript, and MCP bindings.

### How is VantaDB different from ChromaDB / LanceDB / Qdrant?

Unlike ChromaDB and Qdrant (client-server databases), VantaDB is fully embedded — zero external processes, no network dependency, and no Docker required. Compared to LanceDB, VantaDB offers synchronous-first APIs designed specifically for AI agent memory workloads, built-in graph traversal (BFS/DFS), TTL-based auto-expiry, and a pluggable storage backend (Fjall by default, with RocksDB and in-memory options).

### Is VantaDB production-ready?

VantaDB is at version 0.6.1 and under active development. The core engine, WAL durability, HNSW vector search, BM25 text indexing, and hybrid search are stable and covered by integration tests. Production use is encouraged with the understanding that the API is still evolving.

## Usage

### How do I install VantaDB?

**Rust:** Add to `Cargo.toml`:
```toml
[dependencies]
vantadb = "0.6.1"
```

**Python:**
```bash
pip install vantadb-py
```

<!-- **Homebrew (macOS/Linux):**
```bash
brew install vantadb
```
*(Homebrew formula — planned, not yet available)* -->

### How do I create a memory store?

```rust
use vantadb::{Config, Embedded};

let config = Config {
    storage_path: "./vanta_data".into(),
    ..Default::default()
};
let db = Embedded::open_with_config(config)?;
```

Or with defaults:
```rust
let db = Embedded::open("./vanta_data")?;
```

### How do I add vectors?

```rust
use vantadb::{MemoryInput, Value};

let record = db.put(MemoryInput {
    namespace: "chat".into(),
    key: "msg-1".into(),
    payload: "What is VantaDB?".into(),
    metadata: vec![("source".into(), Value::String("user".into()))]
        .into_iter()
        .collect(),
    vector: Some(vec![0.1, 0.2, 0.3, /* ... 384 dims */]),
    ..Default::default()
})?;
```

> **Type names (0.6.0):** the crate exports the un-prefixed names — `Embedded`,
> `Config`, `MemoryInput`, `MemorySearchRequest`, `NodeInput`, `Fields`, `Value`,
> `Error` (`src/lib.rs` re-exports). The old `VantaEmbedded` / `VantaConfig` /
> `VantaMemoryInput` / `VantaMemorySearchRequest` / `VantaNodeInput` /
> `VantaFields` / `VantaValue` / `VantaError` aliases were removed in 0.6.0
> (AST-010) and no longer resolve.

`MemoryInput` has further optional fields (`sparse_vector`, `valid_at_ms`,
`confidence_class`, `confidence`, `derived_from`); the `..Default::default()`
above keeps the example to the common case.

### How do I search?

```rust
use vantadb::MemorySearchRequest;

let results = db.search(MemorySearchRequest {
    namespace: "chat".into(),
    query_vector: vec![0.1, 0.2, 0.3, /* ... */],
    text_query: None,
    top_k: 10,
    ..Default::default()
})?;
```

### How do I use hybrid search?

Set both `query_vector` and `text_query` in the search request. VantaDB automatically fuses BM25 lexical results with HNSW vector results using Reciprocal Rank Fusion (RRF):

```rust
let results = db.search(MemorySearchRequest {
    namespace: "chat".into(),
    query_vector: vec![0.1, 0.2, 0.3, /* ... */],
    text_query: Some("What is VantaDB?".into()),
    top_k: 10,
    ..Default::default()
})?;
```

## Configuration

### What backends are supported?

| Backend | Enum Value | Description |
|---------|-----------|-------------|
| Fjall | `BackendKind::Fjall` | Default LSM-based embedded KV store |
| RocksDB | `BackendKind::RocksDb` | RocksDB backend (feature-gated) |
| InMemory | `BackendKind::InMemory` | Volatile in-memory store (no persistence) |

Set via `Config::backend_kind` or the `VANTADB_BACKEND` environment variable.

### How do I change the storage backend?

Set `backend_kind` in `Config` or use the `VANTADB_BACKEND` environment variable:

```rust
use vantadb::{Config, BackendKind};

let config = Config {
    storage_path: "./vanta_data".into(),
    backend_kind: BackendKind::RocksDb,
    ..Default::default()
};
```

Supported values: `fjall` (default LSM-based), `rocksdb` (feature-gated),
`in-memory` — also accepted as the legacy alias `memory` (volatile). An
unrecognized value logs a warning and falls back to `fjall`.

### How do I configure memory limits?

Set `memory_limit` in `Config` (in bytes), or set the `VANTADB_MEMORY_LIMIT`
environment variable — `Config::default()` does parse it (plain bytes or a
suffixed value like `2GB`; an invalid value is warned about and ignored). This
provides a budget hint for the backend and mmap selection — it is **not** a hard
RSS ceiling. An explicit constructor argument takes precedence over the env var.

Additionally, eviction weights (`eviction_weight_hits`, `eviction_weight_confidence`, `eviction_weight_importance`, `eviction_weight_recency`) control the eviction policy when memory pressure triggers.

### How do I enable metrics?

Metrics are exposed via Prometheus gauges registered in `METRICS_REGISTRY`. In server mode, they are available at `GET /metrics`. Key metrics include:

- `vanta_process_rss_bytes` — resident set size
- `vanta_hnsw_nodes_count` — HNSW index size
- `vanta_mmap_resident_bytes` — mmap pages in RAM

## Troubleshooting

### Why is my search returning no results?

Common causes:

1. **Empty index:** No vectors have been inserted yet, or the index was purged.
2. **Namespace mismatch:** The search namespace must match the namespace used during `put()`.
3. **Dimension mismatch:** Query vector dimensions must match the dimensions of inserted vectors.
4. **Filters too restrictive:** Metadata equality filters may exclude all candidates.
5. **TTL expiry:** Records may have expired if `ttl_ms` was set and the time has elapsed.

### What does error X mean?

| Error | Meaning |
|-------|---------|
| `Error::NodeNotFound` | The requested node ID does not exist |
| `Error::DimensionMismatch` | Vector dimensions do not match the index |
| `Error::DatabaseBusy` | Another process holds the `.vanta.lock` file |
| `Error::ResourceLimit` | Backpressure eviction threshold was exceeded |
| `Error::WALVersionMismatch` | WAL file was written by an incompatible engine version |
| `Error::Serialization` | Postcard/serde serialization failure (possible data corruption) |

### How do I recover from a corrupt WAL?

1. Stop all processes accessing the database.
2. Delete or move the `vanta.wal` file (the engine can replay from a clean state).
3. Run `db.rebuild_index()` to rebuild the HNSW, derived, and text indexes from canonical storage.
4. Call `db.compact_wal()` to start a fresh WAL.

If the backend KV store (Fjall/RocksDB) is also corrupt, restore from a JSONL backup created via `db.export_namespace()` or `db.export_all()`.

### How do I file a bug report?

Open an issue at [github.com/ness-e/Vantadb/issues](https://github.com/ness-e/Vantadb/issues) with:

- VantaDB version (`vantadb --version` or `Cargo.toml` version)
- Operating system and architecture
- Steps to reproduce (minimal code snippet)
- Expected vs actual behavior
- Relevant logs or error output

### Where can I get help?

- **GitHub Issues:** [github.com/ness-e/Vantadb/issues](https://github.com/ness-e/Vantadb/issues) — bug reports, feature requests
- **Documentation:** [docs.rs/vantadb](https://docs.rs/vantadb) — Rust API reference
- **SDK References:** `docs/api/EMBEDDED_SDK.md` for Rust, `docs/api/PYTHON_SDK.md` for Python
- **Operations Manual:** `docs/user/operations/CONFIGURATION.md` for all configuration knobs and CLI commands
- **Durability Guarantees:** `docs/user/operations/DURABILITY_GUARANTEES.md` for crash recovery and WAL behavior

### Can I use the Graph API?

Yes. VantaDB supports a low-level node-graph model with directed edges and traversal:

```rust
use vantadb::graph::TraversalDirection;
use vantadb::{Embedded, Fields, NodeInput};

let db = Embedded::open("./vanta_data")?;

// Insert nodes (NodeInput::new(id) pre-fills content/vector/fields)
db.insert_node(NodeInput {
    id: 1,
    content: Some("Root concept".into()),
    vector: None,
    fields: Fields::new(),
})?;
db.insert_node(NodeInput {
    id: 2,
    content: Some("Related idea".into()),
    vector: None,
    fields: Fields::new(),
})?;

// Add directed edges (weight and created_at_ms are both optional)
db.add_edge(1, 2, "relates_to", Some(0.8), None)?;

// Traverse (roots, max_depth, direction)
let bfs_order = db.graph_bfs(&[1], 3, TraversalDirection::Forward)?;
let dfs_order = db.graph_dfs(&[1], 3, TraversalDirection::Forward)?;
```

Available in both Rust and Python SDKs.

### How durable is VantaDB?

VantaDB uses a WAL-first architecture with CRC32C checksums on every record. Key guarantees:

- **No partial writes** — incomplete trailing WAL records are auto-healed on next open
- **Crash consistency** — after SIGKILL or power loss, all committed data is recoverable
- **Idempotent recovery** — replaying the same WAL records multiple times produces identical state
- **Atomic checkpoint** — `checkpoint_seq` ensures WAL replay only covers unflushed mutations
- **Multi-process isolation** — exclusive `.vanta.lock` prevents concurrent write access

Default sync mode is `Periodic` with flush threshold 1 — i.e., VantaDB **fsyncs after every write** by default (`wal.rs::maybe_sync`, `DEFAULT_PERIODIC_THRESHOLD = 1`), so a crash loses at most the record being written. To trade durability for throughput on write-heavy workloads, raise the flush threshold or switch sync modes in your config.

# Frequently Asked Questions

## General

### What is VantaDB?

VantaDB is an embedded persistent memory engine for local-first AI applications. It combines vector similarity search (HNSW), full-text lexical search (BM25), hybrid search fusion, and structured metadata filtering in a single embeddable Rust library with Python, TypeScript, and MCP bindings.

### How is VantaDB different from ChromaDB / LanceDB / Qdrant?

Unlike ChromaDB and Qdrant (client-server databases), VantaDB is fully embedded — zero external processes, no network dependency, and no Docker required. Compared to LanceDB, VantaDB offers synchronous-first APIs designed specifically for AI agent memory workloads, built-in graph traversal (BFS/DFS), TTL-based auto-expiry, and a pluggable storage backend (Fjall by default, with RocksDB and in-memory options).

### Is VantaDB production-ready?

VantaDB is at version 0.1.5 and under active development. The core engine, WAL durability, HNSW vector search, BM25 text indexing, and hybrid search are stable and covered by integration tests. Production use is encouraged with the understanding that the API is still evolving toward the v0.2.0 milestone.

## Usage

### How do I install VantaDB?

**Rust:** Add to `Cargo.toml`:
```toml
[dependencies]
vantadb = "0.1.5"
```

**Python:**
```bash
pip install vantadb-py
```

**Homebrew (macOS/Linux):**
```bash
brew install vantadb
```

### How do I create a memory store?

```rust
use vantadb::{VantaEmbedded, VantaConfig};

let config = VantaConfig {
    storage_path: "./vanta_data".into(),
    ..Default::default()
};
let db = VantaEmbedded::open_with_config(config)?;
```

Or with defaults:
```rust
let db = VantaEmbedded::open("./vanta_data")?;
```

### How do I add vectors?

```rust
use vantadb::{VantaMemoryInput, VantaValue};

let record = db.put(VantaMemoryInput {
    namespace: "chat".into(),
    key: "msg-1".into(),
    payload: "What is VantaDB?".into(),
    metadata: vec![("source".into(), VantaValue::String("user".into()))]
        .into_iter()
        .collect(),
    vector: Some(vec![0.1, 0.2, 0.3, /* ... 384 dims */]),
    ttl_ms: None,
})?;
```

### How do I search?

```rust
use vantadb::VantaMemorySearchRequest;

let results = db.search(VantaMemorySearchRequest {
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
let results = db.search(VantaMemorySearchRequest {
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

Set via `VantaConfig::backend_kind` or the `VANTA_BACKEND` environment variable.

### How do I change the storage backend?

Set `backend_kind` in `VantaConfig` or use the `VANTA_BACKEND` environment variable:

```rust
use vantadb::{VantaConfig, BackendKind};

let config = VantaConfig {
    storage_path: "./vanta_data".into(),
    backend_kind: BackendKind::RocksDb,
    ..Default::default()
};
```

Supported values: `fjall` (default LSM-based), `rocksdb` (feature-gated), `memory` (volatile).

### How do I configure memory limits?

Set `memory_limit` in `VantaConfig` (in bytes). This provides a budget hint for the backend and mmap selection. Additionally, eviction weights (`eviction_weight_hits`, `eviction_weight_confidence`, `eviction_weight_importance`, `eviction_weight_recency`) control the eviction policy when memory pressure triggers.

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
| `VantaError::NodeNotFound` | The requested node ID does not exist |
| `VantaError::DimensionMismatch` | Vector dimensions do not match the index |
| `VantaError::DatabaseBusy` | Another process holds the `.vanta.lock` file |
| `VantaError::ResourceLimit` | Backpressure eviction threshold was exceeded |
| `VantaError::WALVersionMismatch` | WAL file was written by an incompatible engine version |
| `VantaError::SerializationError` | Bincode/serde serialization failure (possible data corruption) |

### How do I recover from a corrupt WAL?

1. Stop all processes accessing the database.
2. Delete or move the `vanta.wal` file (the engine can replay from a clean state).
3. Run `db.rebuild_index()` to rebuild the HNSW, derived, and text indexes from canonical storage.
4. Call `db.compact_wal()` to start a fresh WAL.

If the backend KV store (Fjall/RocksDB) is also corrupt, restore from a JSONL backup created via `db.export_namespace()` or `db.export_all()`.

### How do I file a bug report?

Open an issue at [github.com/vantadb/vantadb/issues](https://github.com/vantadb/vantadb/issues) with:

- VantaDB version (`vantadb --version` or `Cargo.toml` version)
- Operating system and architecture
- Steps to reproduce (minimal code snippet)
- Expected vs actual behavior
- Relevant logs or error output

### Where can I get help?

- **GitHub Issues:** [github.com/vantadb/vantadb/issues](https://github.com/vantadb/vantadb/issues) — bug reports, feature requests
- **Documentation:** [docs.rs/vantadb](https://docs.rs/vantadb) — Rust API reference
- **SDK References:** `docs/api/EMBEDDED_SDK.md` for Rust, `docs/api/PYTHON_SDK.md` for Python
- **Operations Manual:** `docs/operations/CONFIGURATION.md` for all configuration knobs and CLI commands
- **Durability Guarantees:** `docs/operations/DURABILITY_GUARANTEES.md` for crash recovery and WAL behavior

### Can I use the Graph API?

Yes. VantaDB supports a low-level node-graph model with directed edges and traversal:

```rust
use vantadb::{VantaEmbedded, VantaNodeInput, VantaFields};

let db = VantaEmbedded::open("./vanta_data")?;

// Insert nodes
db.insert_node(VantaNodeInput {
    id: 1,
    content: Some("Root concept".into()),
    vector: None,
    fields: VantaFields::new(),
})?;
db.insert_node(VantaNodeInput {
    id: 2,
    content: Some("Related idea".into()),
    vector: None,
    fields: VantaFields::new(),
})?;

// Add directed edges
db.add_edge(1, 2, "relates_to", Some(0.8))?;

// Traverse
let bfs_order = db.graph_bfs(&[1], 3)?;
let dfs_order = db.graph_dfs(&[1], 3)?;
```

Available in both Rust and Python SDKs.

### How durable is VantaDB?

VantaDB uses a WAL-first architecture with CRC32C checksums on every record. Key guarantees:

- **No partial writes** — incomplete trailing WAL records are auto-healed on next open
- **Crash consistency** — after SIGKILL or power loss, all committed data is recoverable
- **Idempotent recovery** — replaying the same WAL records multiple times produces identical state
- **Atomic checkpoint** — `checkpoint_seq` ensures WAL replay only covers unflushed mutations
- **Multi-process isolation** — exclusive `.vanta.lock` prevents concurrent write access

Default sync mode is `Periodic` (fsync every 5s). For maximum durability, use `SyncMode::Always` at the cost of throughput.

---
title: VantaEmbedded SDK Reference
type: api
status: active
tags: [vantadb, api]
last_reviewed: 2026-07-01
---

# VantaEmbedded SDK Reference

> Core Rust SDK struct `VantaEmbedded` — the primary entry point for all embedded database operations. Used directly in Rust and exposed via [[pyo3|PyO3]] (Python), wasm-bindgen (TypeScript), and [[mcp|MCP]].

**Source:** `src/sdk.rs`

## Construction

```rust
use vantadb::{VantaEmbedded, VantaConfig};

// Open with defaults (path-based)
let db = VantaEmbedded::open("./vanta_data").unwrap();

// Open with full configuration
let config = VantaConfig {
    storage_path: "./vanta_data".into(),
    memory_limit: Some(512_000_000),
    read_only: false,
    ..Default::default()
};
let db = VantaEmbedded::open_with_config(config).unwrap();

// Wrap an existing StorageEngine handle
let db = VantaEmbedded::from_engine(engine);
```

| Method | Description |
|--------|-------------|
| `open(path)` | Open or create a database at the given filesystem path with default config |
| `open_with_config(config)` | Open or create with full `VantaConfig` |
| `from_engine(engine)` | Wrap an existing `Arc<StorageEngine>` handle |

## Memory (Namespace-scoped) API

CRUD operations for persistent memory records identified by `(namespace, key)` pairs.

| Method | Description |
|--------|-------------|
| `put(input: VantaMemoryInput)` | Insert or update a memory record. Returns `VantaMemoryRecord` |
| `put_batch(inputs: Vec<VantaMemoryInput>)` | Batch insert/update (parallel, up to 5x faster). Returns `Vec<VantaMemoryRecord>` |
| `get(namespace, key)` | Retrieve a record by namespace+key. Returns `Option<VantaMemoryRecord>` |
| `delete(namespace, key)` | Delete a record. Returns `bool` (true if existed) |
| `list(namespace, options)` | List records in a namespace with cursor pagination. Returns `VantaMemoryListPage` |
| `list_namespaces()` | List all namespaces. Returns `Vec<String>` |
| `search(request: VantaMemorySearchRequest)` | [[hybrid-search\|Hybrid]] (vector + lexical) search. Returns `Vec<VantaMemorySearchHit>` |
| `explain_memory_search(request)` | Search with detailed score breakdown. Returns `VantaSearchExplanation` |

### `VantaMemoryInput`

```rust
pub struct VantaMemoryInput {
    pub namespace: String,
    pub key: String,
    pub payload: String,
    pub metadata: VantaMemoryMetadata,  // BTreeMap<String, VantaValue>
    pub vector: Option<Vec<f32>>,       // 384-dim embedding
    pub ttl_ms: Option<u64>,            // auto-expiry in ms from now
}
```

### `VantaMemorySearchRequest`

```rust
pub struct VantaMemorySearchRequest {
    pub namespace: String,
    pub query_vector: Vec<f32>,       // empty = no vector search
    pub filters: VantaMemoryMetadata, // equality filter on metadata
    pub text_query: Option<String>,   // [[bm25|BM25]] lexical query
    pub top_k: usize,                 // default: 10
    pub distance_metric: DistanceMetric, // Cosine (default) or Euclidean
    pub explain: bool,                // include score breakdown
}
```

### `VantaMemoryRecord`

```rust
pub struct VantaMemoryRecord {
    pub namespace: String,
    pub key: String,
    pub payload: String,
    pub metadata: VantaMemoryMetadata,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
    pub version: u64,
    pub node_id: u64,
    pub vector: Option<Vec<f32>>,
    pub expires_at_ms: Option<u64>,
}
```

## Node / Graph API

Low-level operations on the node-graph model (numeric node IDs, edges, graph traversal).

| Method | Description |
|--------|-------------|
| `insert_node(input: VantaNodeInput)` | Insert a graph node with content, vector, and fields |
| `get_node(id: u64)` | Retrieve a node by numeric ID. Returns `Option<VantaNodeRecord>` |
| `delete_node(id, reason)` | Delete a node with auditable reason (tombstone) |
| `add_edge(source_id, target_id, label, weight)` | Add a directed edge between two nodes |
| `graph_bfs(roots, max_depth)` | BFS traversal. Returns `Vec<u64>` |
| `graph_dfs(roots, max_depth)` | DFS traversal. Returns `Vec<u64>` |
| `graph_topological_sort(roots)` | Topological sort. Returns `Vec<u64>` |
| `graph_is_dag(roots)` | Check if subgraph is a DAG. Returns `bool` |
| `search_vector(vector, top_k)` | Pure [[hnsw\|HNSW]] vector search. Returns `Vec<VantaSearchHit>` |
| `query(iql_query)` | Execute IQL query string. Returns `VantaQueryResult` |

### `VantaNodeInput`

```rust
pub struct VantaNodeInput {
    pub id: u64,
    pub content: Option<String>,
    pub vector: Option<Vec<f32>>,
    pub fields: VantaFields,  // BTreeMap<String, VantaValue>
}
```

### `VantaNodeRecord`

```rust
pub struct VantaNodeRecord {
    pub id: u64,
    pub fields: VantaFields,
    pub vector: Option<Vec<f32>>,
    pub vector_dimensions: usize,
    pub edges: Vec<VantaEdgeRecord>,
    pub confidence_score: f32,
    pub importance: f32,
    pub hits: u32,
    pub last_accessed: u64,
    pub epoch: u32,
    pub tier: VantaStorageTier,  // Hot / Cold
    pub is_alive: bool,
}
```

## Maintenance

| Method | Description |
|--------|-------------|
| `flush()` | Flush [[wal\|WAL]] + [[hnsw\|HNSW]] to disk for durability |
| `compact_wal()` | Archive [[wal\|WAL]] file and start fresh |
| `purge_expired()` | Delete TTL-expired records. Returns count purged |
| `rebuild_index()` | Rebuild ANN ([[hnsw\|HNSW]]), derived, and text indexes. Returns `VantaIndexRebuildReport` |
| `compact_layout()` | BFS-order physical compaction of vector store. Returns nodes compacted |

## Export / Import

| Method | Description |
|--------|-------------|
| `export_namespace(path, namespace)` | Export namespace as JSONL. Returns `VantaExportReport` |
| `export_all(path)` | Export all namespaces as JSONL. Returns `VantaExportReport` |
| `import_records(records)` | Import `Vec<VantaMemoryRecord>`. Returns `VantaImportReport` |
| `import_file(path)` | Import from JSONL file. Returns `VantaImportReport` |

## Text Index Diagnostics

| Method | Description |
|--------|-------------|
| `audit_text_index(namespace)` | Read-only structural audit. Returns `VantaTextIndexAuditReport` |
| `audit_text_index_deep(namespace)` | Deep audit (decodes TF, positions, DF, doc lengths). Returns `VantaTextIndexAuditReport` |
| `repair_text_index()` | Rebuild text index from canonical storage. Returns `VantaTextIndexRepairReport` |

## Observability

| Method | Description |
|--------|-------------|
| `operational_metrics()` | Snapshot of runtime metrics. Returns `VantaOperationalMetrics` |
| `capabilities()` | Stable runtime capabilities. Returns `VantaCapabilities` |
| `generate_snippet(payload, query, with_highlighting)` | Highlighted text snippet. Returns `Option<String>` |

## Lifecycle

| Method | Description |
|--------|-------------|
| `close()` | Flush and release the engine handle |
| `debug_memory_breakdown()` | *(debug-only)* Memory usage breakdown as JSON |

## Data Types

### `VantaValue`

```rust
pub enum VantaValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    DateTime(chrono::DateTime<chrono::Utc>),
    ListString(Vec<String>),
    ListInt(Vec<i64>),
    ListFloat(Vec<f64>),
    ListBool(Vec<bool>),
    ListDateTime(Vec<chrono::DateTime<chrono::Utc>>),
    Null,
}
```

### `VantaMemoryListOptions`

```rust
pub struct VantaMemoryListOptions {
    pub filters: VantaMemoryMetadata,  // equality filter on metadata fields
    pub limit: usize,                  // max records to return (default: 100)
    pub cursor: Option<usize>,         // pagination cursor from previous page
}
```

### `VantaMemoryListPage`

```rust
pub struct VantaMemoryListPage {
    pub records: Vec<VantaMemoryRecord>,
    pub next_cursor: Option<usize>,
}
```

### `VantaMemorySearchHit`

```rust
pub struct VantaMemorySearchHit {
    pub record: VantaMemoryRecord,
    pub score: f32,
    pub explanation: Option<VantaSearchExplanationHit>,
}
```

### `VantaEdgeRecord`

```rust
pub struct VantaEdgeRecord {
    pub target: u64,
    pub label: String,
    pub weight: f32,
}
```

### `VantaRuntimeProfile` / `VantaStorageTier`

```rust
pub enum VantaRuntimeProfile {
    Enterprise,
    Performance,
    LowResource,
}

pub enum VantaStorageTier {
    Hot,   // RAM-cached
    Cold,  // on-disk
}
```

### `VantaQueryResult`

```rust
pub enum VantaQueryResult {
    Read(Vec<VantaNodeRecord>),
    Write {
        affected_nodes: u64,
        message: String,
        node_id: Option<u64>,
    },
    StaleContext {
        node_id: u64,
    },
}
```

### `VantaSearchExplanation`

```rust
pub struct VantaSearchExplanation {
    pub route: String,                          // "hybrid", "text_only", "vector_only"
    pub hits: Vec<VantaSearchExplanationHit>,
    pub fusion_report: Option<VantaHybridFusionReport>,
}
```

### `VantaHybridFusionReport`

```rust
pub struct VantaHybridFusionReport {
    pub text_candidates: usize,
    pub vector_candidates: usize,
    pub fused_candidates: usize,
    pub rrf_k: usize,
}
```

### `VantaCapabilities`

```rust
pub struct VantaCapabilities {
    pub runtime_profile: VantaRuntimeProfile,  // Enterprise / Performance / LowResource
    pub persistence: bool,
    pub vector_search: bool,
    pub iql_queries: bool,
    pub read_only: bool,
}
```

### `VantaOperationalMetrics` (37 fields)

Metrics grouped by subsystem:

| Category | Fields |
|----------|--------|
| Startup | `startup_ms`, `wal_replay_ms`, `wal_records_replayed` |
| ANN rebuild | `ann_rebuild_ms`, `ann_rebuild_scanned_nodes` |
| Derived index | `derived_rebuild_ms`, `derived_prefix_scans`, `derived_full_scan_fallbacks` |
| Text index | `text_index_rebuild_ms`, `text_postings_written`, `text_index_repairs`, `text_lexical_queries`, `text_lexical_query_ms`, `text_candidates_scored`, `text_consistency_audits`, `text_consistency_audit_failures` |
| Hybrid planner | `hybrid_query_ms`, `hybrid_candidates_fused`, `planner_hybrid_queries`, `planner_text_only_queries`, `planner_vector_only_queries` |
| Export/Import | `records_exported`, `records_imported`, `import_errors` |
| Memory | `process_rss_bytes`, `process_virtual_bytes`, `hnsw_nodes_count`, `hnsw_logical_bytes`, `mmap_resident_bytes`, `volatile_cache_entries`, `volatile_cache_cap_bytes` |
| Jemalloc (heap) | `jemalloc_allocated_bytes`, `jemalloc_active_bytes`, `jemalloc_metadata_bytes`, `jemalloc_resident_bytes`, `jemalloc_mapped_bytes`, `jemalloc_retained_bytes` |

### `VantaSearchExplanationHit`

```rust
pub struct VantaSearchExplanationHit {
    pub identity: String,          // namespace/key
    pub score: f32,
    pub snippet: Option<String>,   // highlighted text
    pub matched_tokens: Vec<String>,
    pub matched_phrases: Vec<String>,
    pub bm25_terms: Vec<VantaBm25TermContribution>,  // per-term TF/DF/contribution
    pub rrf_text_rank: Option<usize>,
    pub rrf_vector_rank: Option<usize>,
}
```

### `VantaBm25TermContribution`

```rust
pub struct VantaBm25TermContribution {
    pub token: String,
    pub tf: u32,
    pub df: u64,
    pub doc_len: u32,
    pub contribution: f32,
}
```

## Report Types

### `VantaIndexRebuildReport`

```rust
pub struct VantaIndexRebuildReport {
    pub scanned_nodes: u64,
    pub indexed_vectors: u64,
    pub skipped_tombstones: u64,
    pub duration_ms: u64,
    pub derived_rebuild_ms: u64,
    pub index_path: String,
    pub success: bool,
}
```

### `VantaExportReport`

```rust
pub struct VantaExportReport {
    pub records_exported: u64,
    pub namespaces: Vec<String>,
    pub path: String,
    pub duration_ms: u64,
}
```

### `VantaImportReport`

```rust
pub struct VantaImportReport {
    pub inserted: u64,
    pub updated: u64,
    pub skipped: u64,
    pub errors: u64,
    pub duration_ms: u64,
}
```

### `VantaTextIndexAuditReport` (25 fields)

Key fields: `schema_version`, `tokenizer`, `namespaces_audited`, `records_scanned`, `expected_entries`, `actual_entries`, `missing_entries`, `unexpected_entries`, `value_mismatches`, `position_errors`, `tf_errors`, `df_errors`, `doc_len_errors`, `logical_corruptions`, `state_valid`, `passed`, `status`.

### `VantaTextIndexRepairReport`

```rust
pub struct VantaTextIndexRepairReport {
    pub record_count: u64,
    pub posting_entries: u64,
    pub doc_stats_entries: u64,
    pub term_stats_entries: u64,
    pub namespace_stats_entries: u64,
    pub duration_ms: u64,
    pub success: bool,
}
```

## Error Handling

All fallible methods return `Result<T, VantaError>` where `VantaError` is an enum covering:

- `VantaError::NodeNotFound(u64)` — node ID not found
- `VantaError::DuplicateNode(u64)` — duplicate node ID on insert
- `VantaError::DimensionMismatch { expected: usize, got: usize }` — vector dimension mismatch
- `VantaError::WalError(String)` — WAL operation failure
- `VantaError::WALVersionMismatch { expected: u32, found: u32, hint: String }` — incompatible WAL version
- `VantaError::SerializationError(String)` — bincode/serde failures
- `VantaError::IoError(std::io::Error)` — filesystem errors
- `VantaError::IncompatibleFormat { expected_magic, expected_version, found_magic, found_version, hint }` — incompatible binary format
- `VantaError::NotInitialized` — engine not open
- `VantaError::ResourceLimit(String)` — resource limit exceeded (backpressure)
- `VantaError::Execution(String)` — runtime errors (collisions, invariants)
- `VantaError::DatabaseBusy(String)` — database locked by another process

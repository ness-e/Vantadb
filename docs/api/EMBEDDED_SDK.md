---
title: VantaEmbedded SDK Reference
type: api
status: active
tags: [vantadb, api]
last_reviewed: 2026-09-01
aliases: []
---

# VantaEmbedded SDK Reference

> Core Rust SDK struct `VantaEmbedded` — the primary entry point for all embedded database operations. Used directly in Rust and exposed via [[pyo3|PyO3]] (Python), wasm-bindgen (TypeScript), and [[mcp|MCP]].

**Source:** `src/sdk/mod.rs`, `src/sdk/builder.rs`, `src/sdk/api/*.rs`, `src/sdk/graph.rs`, `src/sdk/search/mod.rs`

## Construction

```rust
use vantadb::{VantaEmbedded, VantaConfig, BackendKind};

// Open with defaults (path-based)
let db = VantaEmbedded::open("./vanta_data").unwrap();

// Open with full configuration
let config = VantaConfig {
    storage_path: "./vanta_data".into(),
    memory_limit: Some(512_000_000),
    read_only: false,
    backend_kind: BackendKind::Fjall,
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

### Audit Log

Set `VantaConfig.audit_log_path` (or `VANTADB_AUDIT_LOG_PATH`) to enable append-only JSONL audit logging of write/delete/export/import operations. Disabled by default (`None`).

```rust
let config = VantaConfig {
    storage_path: "./vanta_data".into(),
    audit_log_path: Some("audit/audit.jsonl".into()),
    ..Default::default()
};
let db = VantaEmbedded::open_with_config(config).unwrap();
```

Each line is one JSON object: `{"timestamp":"2026-08-02T12:34:56Z","op":"put","namespace":"docs","key":"a","outcome":"ok","reason":null}`. Ops: `put`, `put_batch`, `delete`, `delete_by_filter`, `export_namespace`, `export_all`, `import_file`, `bulk_import_file`, `bulk_import_stream`. Read-only ops are not audited. See `docs/operations/CONFIGURATION.md`.

## Memory (Namespace-scoped) API

CRUD operations for persistent memory records identified by `(namespace, key)` pairs.

| Method | Description |
|--------|-------------|
| `put(input: VantaMemoryInput)` | Insert or update a memory record. Returns `VantaMemoryRecord` |
| `put_batch(inputs: Vec<VantaMemoryInput>)` | Batch insert/update (parallel, up to 5x faster). Returns `Vec<VantaMemoryRecord>` |
| `get(namespace, key)` | Retrieve a record by namespace+key. Returns `Option<VantaMemoryRecord>` |
| `get_version(namespace, key, version)` | Retrieve the record as it was at the given version (VS-CORE-07). Returns `Option<VantaMemoryRecord>` — `None` if that version was never persisted (unknown key, purged by the retention cap, or deleted). Snapshot durability is best-effort post-commit, so a crash window can leave a version gap — degraded but never corrupt |
| `versions(namespace, key)` | List every retained version of a record, ascending (v1..vN) (VS-CORE-07). Returns `Vec<VantaMemoryRecord>` — empty if the key does not exist or has no history; expired versions are included as historical data until purged. `get_version(namespace, key, vN)` of the last element matches the live record |
| `delete(namespace, key)` | Delete a record. Returns `bool` (true if existed) |
| `delete_by_filter(namespace, filter)` | Delete all records in a namespace matching a metadata filter. Returns `u64` count of deleted records. Filter uses `VantaMemoryFilter` (equality + operators). Empty filter rejected to prevent accidental full-namespace deletion |
| `count(namespace, filter)` | Count memory records in a namespace, optionally filtered by metadata. Returns `u64`. Filter uses `VantaMemoryFilter` (equality + operators). `None` counts all records |
| `list(namespace, options)` | List records in a namespace with cursor pagination. Returns `VantaMemoryListPage` |
| `list_namespaces()` | List all namespaces. Returns `Vec<String>` |
| `search(request: VantaMemorySearchRequest)` | [[hybrid-search\|Hybrid]] (vector + lexical) search. Returns `Vec<VantaMemorySearchHit>` |
| `search_with_method(request, method)` | Same as `search` with an explicit index backend override for the dense-vector portion: `Some(IndexType::Ivf)` / `Some(IndexType::Scann)` / `Some(IndexType::Flat)` / `Some(IndexType::Hnsw)`. `None` (default) keeps automatic engine routing untouched; the shared engine config is never mutated (thread-safe, per-search override) |
| `search_multi(namespaces, request)` | Search across multiple namespaces, merging results by descending score, capped at `request.top_k`. Namespaces that produce no results or fail validation are silently skipped; an empty `namespaces` slice returns an empty `Vec` |
| `similar_to_key(namespace, key, top_k)` | Vector similarity search from an existing record's vector, post-filtered to `namespace`. Errors `NotFound` if the key does not exist and `NoVectorForKey` if the record carries no vector |
| `explain_memory_search(request)` | Search with detailed score breakdown. Returns `VantaSearchExplanation` |
| `namespace_stats(expiring_soon_window_ms)` | Per-namespace statistics: total records, records expiring within the window, already-expired records. Single full scan (no N paginated `count`/`list` calls). `None` uses the 24h default window. Returns `VantaNamespaceStatsMap` |
| `supersede(namespace, old_key, new_key)` | Mark `old_key` as superseded by `new_key`: the old record keeps its data (soft-dead, recoverable) but gains `superseded_by`/`superseded_at_ms`, and can be hidden from search/list with `exclude_superseded`. Errors if either key is missing, if `old_key == new_key`, or if the old record is already superseded (idempotency guard) |
| `purge_expired()` | Scan all memory records and physically delete those whose TTL has expired. Returns `u64` count of purged records |
| `bulk_import_file(path)` | Bulk-import from a binary `.vdbdump` file. Bypasses per-record validation for raw throughput; commits in batches sized by `bulk_commit_interval` (default 10000) |
| `bulk_import_stream(reader)` | Bulk-import records from a binary stream. Format: 8-byte magic `VDBJSON\n`, 1-byte version `0x01`, 8-byte LE record count, then serde_json-serialized `Vec<VantaMemoryInput>`. Same batching/validation behavior as `bulk_import_file` |

### Version History (VS-CORE-07)

Every `put`/`put_batch` snapshots the new record into a `Versions` partition keyed by `(namespace, key, version)`; `version` increments monotonically per key. `VantaConfig.version_history_limit` caps retained snapshots per key (default `Some(32)`, FIFO eviction of the oldest beyond the cap — see `docs/operations/CONFIGURATION.md`); `delete` and `purge_expired` purge the full history. Imports (`import_file`/`bulk_import_*`) do **not** write snapshots. Only the embedded (native) SDK and the CLI/bridge expose version history.

### Bulk Import

Bulk import operations bypass per-record validation for maximum throughput. Records are committed in batches sized by `VantaConfig::bulk_commit_interval` (default 10,000). The binary format (`.vdbdump`) uses magic `VDBJSON\n`, version `0x01`, LE record count, then JSON-serialized `Vec<VantaMemoryInput>`.

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
    pub text_query: Option<String>,   // BM25 lexical query
    pub top_k: usize,                 // default: 10
    pub distance_metric: DistanceMetric, // Cosine (default) or Euclidean
    pub explain: bool,                // include score breakdown
}
```
*Note: Lexical search uses the [[bm25|BM25]] algorithm.*

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
    pub node_id: u128,
    pub vector: Option<Vec<f32>>,
    pub expires_at_ms: Option<u64>,
}
```

### `VantaNamespaceStats`

```rust
pub struct VantaNamespaceStats {
    pub count: u64,          // total records in the namespace (includes not-yet-purged expired)
    pub expiring_soon: u64,  // records expiring within the window (never includes expired)
    pub expired: u64,        // records whose TTL has already passed
}

pub type VantaNamespaceStatsMap = BTreeMap<String, VantaNamespaceStats>;
```

`namespace_stats` returns one `VantaNamespaceStats` per namespace, keyed by namespace in sorted order. `count` counts all physical records (including expired ones not yet purged); the read-visible subset is available via `count`/`list` (lazy TTL eviction). A record is `expired` if `expires_at_ms <= now` and `expiring_soon` if `now < expires_at_ms <= now + window`; a record never counts as both. Records without a TTL count only toward `count`.

## Scene Nodes (L2 Memory Anchors) — Separate Module

> **Source:** `src/entity/scene.rs` (`entity::scene::SceneNodeStore`) — documented as part of the
> F5 memory pipeline closure (MEM-12 debt). This is a **separate module** from `VantaEmbedded`.

A **scene** is the L2 memory unit that groups an episode of conversation. The
scene NODE is the LLM-free anchor the L2 strategy reads/writes when it updates
the graph (CREATE/UPDATE/MERGE). Each node carries the META contract
`{created, updated, summary, heat}` plus its identity: `namespace`
(deployment/tenant), `session_id` (L0/L1 session), and `scene_name`.

Storage reuses the `EntityStore` partition pattern: a JSON record in the
`InternalMetadata` partition under key
`scene:{namespace}:{session_id}::{scene_name}`.

```rust
use vantadb::entity::scene::{SceneNode, SceneNodePage, SceneNodeStore};

let store = SceneNodeStore::new(&engine);

// Insert or replace (wholesale — caller computes META; store never mutates it)
let node = store.scene_node_set(
    "default",                    // namespace
    "session-42",                 // session_id
    "2026-08-21-10-00",           // scene_name
    "2026-08-21T10:00:00.000Z",   // created (RFC 3339 UTC)
    "2026-08-21T10:05:00.000Z",   // updated (bumped by the L2 strategy)
    "Debugging WAL replay",       // summary
    3,                            // heat (CREATE = 1, UPDATE = old + 1)
)?;

store.scene_node_get("default", "session-42", "2026-08-21-10-00")?; // Option<SceneNode>
let existed = store.scene_node_delete("default", "session-42", "2026-08-21-10-00")?;

// Paginated listing ordered by scene_name; total = full session size pre-pagination
let page: SceneNodePage = store.scene_node_list("default", "session-42", 20, 0)?;
```

**Semantics:** the store is intentionally dumb CRUD. META semantics
(preserve `created` on update, bump `updated`, heat increment) live in the L2
strategy (`vanta-memory/src/core/scene/scene_index.rs::upsert_scene`), not here.
Keys are validated (`validate_key` / `validate_scope`); invalid scope names are
rejected with `VantaError::InvalidInput`.

## Node / Graph API

Low-level operations on the node-graph model (numeric node IDs, edges, graph traversal).

| Method | Description |
|--------|-------------|
| `insert_node(input: VantaNodeInput)` | Insert a graph node with content, vector, and fields |
| `get_node(id: u128)` | Retrieve a node by numeric ID. Returns `Option<VantaNodeRecord>` |
| `delete_node(id, reason)` | Delete a node with auditable reason (tombstone) |
| `add_edge(source_id, target_id, label, weight, created_at_ms)` | Add a directed edge between two nodes with optional weight and creation timestamp. Automatically creates a reverse edge |
| `remove_edge(source_id, target_id, label)` | Remove all edges between two nodes with the given label (both directions) |
| `collect_graph_nodes()` | Collect every live non-memory-record node as `Vec<VantaNodeRecord>` (edges with labels resolved). Excludes nodes carrying a namespace field — those belong to the memory snapshot. Used by the WASM binding to persist the graph store alongside `db_state.json` |
| `restore_graph_nodes(records: Vec<VantaNodeRecord>)` | Restore graph nodes exported by `collect_graph_nodes` into the engine: re-interns edge labels, preserves weights, direction (`reverse`) and creation timestamps. Returns the number of nodes restored |
| `graph_bfs(roots, max_depth, direction)` | BFS traversal. Returns `Vec<u128>` |
| `graph_dfs(roots, max_depth, direction)` | DFS traversal. Returns `Vec<u128>` |
| `graph_bfs_filtered(roots, max_depth, direction, labels, time_range)` | BFS traversal with label filtering and optional temporal window. Only follows edges whose `label_id` is in `labels` (empty = no filter); `time_range: Option<(from_ms, to_ms)>` restricts to edges created within the window |
| `graph_dfs_filtered(roots, max_depth, direction, labels, time_range)` | DFS traversal with the same label/temporal filtering as `graph_bfs_filtered` |
| `graph_topological_sort(roots)` | Topological sort. Returns `Vec<u128>` |
| `graph_is_dag(roots)` | Check if subgraph is a DAG. Returns `bool` |
| `graph_create_accumulator()` | Create a new thread-safe `GraphAccumulator` shareable across worker threads for parallel graph algorithms (PageRank, centrality, etc.) |
| `graph_accumulator_add(acc, node_id, delta)` | Atomically add `delta` to the accumulator for `node_id`. Returns the previous value |
| `graph_accumulator_get(acc, node_id)` | Get the current value for `node_id`. Returns `Option<f64>` |
| `graph_accumulator_snapshot(acc)` | Capture a consistent snapshot of all accumulator values. Returns `HashMap<u128, f64>` |
| `search_vector(vector, top_k)` | Pure [[hnsw\|HNSW]] vector search. Returns `Vec<VantaSearchHit>` |
| `query(iql_query)` | Execute IQL query string. Returns `VantaQueryResult` |
| `vacuum()` | Purge tombstoned nodes from the HNSW index. Returns a `VacuumReport` with counts and timing |
| `pipeline(mode)` | Run the segment optimizer pipeline (vacuum → merge → reindex). Each phase is logged independently; a phase failure does not abort subsequent phases |
| `optimizer_config()` | Return the current segment optimizer configuration |
| `set_optimizer_config(config)` | Override the segment optimizer configuration. Takes effect on the next pipeline invocation |

### `VantaNodeInput`

```rust
pub struct VantaNodeInput {
    pub id: u128,
    pub content: Option<String>,
    pub vector: Option<Vec<f32>>,
    pub fields: VantaFields,  // BTreeMap<String, VantaValue>
}
```

### `VantaNodeRecord`

```rust
pub struct VantaNodeRecord {
    pub id: u128,
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

| `recover_archived_nodes(summary_id)` | Recover shadow-archived nodes that belonged to a summary node. Scans TombstoneStorage for nodes with a `belonged_to` edge targeting `summary_id`, re-activates them, and inserts them back into the active store. Returns `Vec<VantaNodeRecord>` |

## Threads API

Conversation threads with append-only messages and optional TTL expiry.

| Method | Description |
|--------|-------------|
| `create_thread(title, ttl_secs)` | Create a new conversation thread. Returns the thread's numeric ID. Pass `title` for display and optional `ttl_secs` for auto-expiry |
| `get_thread(thread_id)` | Retrieve a thread by its ID. Returns `Option<MessageThread>` |
| `list_threads(limit, offset)` | List threads with pagination. Returns `Vec<MessageThread>` |
| `delete_thread(thread_id)` | Delete a thread by its ID |
| `send_message(thread_id, role, content)` | Append a message to a thread. `role` is the message role (e.g., "user", "assistant") |
| `purge_expired_threads()` | Purge threads whose TTL has expired. Returns the number of threads removed |
| `recover_archived_nodes(summary_id)` | Recover shadow-archived nodes that belonged to a summary node. Scans TombstoneStorage for nodes with a `belonged_to` edge targeting `summary_id`, re-activates them, and inserts them back into the active store. Returns `Vec<VantaNodeRecord>` |

## Snapshots API

Instant filesystem snapshots via hard links (Unix) or copy (Windows).

| Method | Description |
|--------|-------------|
| `create_snapshot(name)` | Create an instant point-in-time snapshot. All data files in the storage directory are hard-linked into `<data_dir>/snapshots/<name>` (O(1)) |
| `list_snapshots()` | List all existing snapshot names. Returns `Vec<String>` |
| `restore_from(config, name)` | DESTRUCTIVE static associated function: replace the live database directory with the contents of snapshot `<name>`, then reopen. Close every open handle first (`db.close()?; let db = VantaEmbedded::restore_from(config.clone(), "name")?;`). The current data is staged aside (atomic rename, sibling of `data_dir`) with best-effort rollback on failure; all snapshots survive the swap. `name` must be a plain identifier (path traversal rejected). Indexes rebuild from storage on reopen |

### Restore flow

```rust,ignore
db.close()?;                                       // release the fs2 lock + handles
let db = VantaEmbedded::restore_from(config.clone(), "snap-1")?; // swap + reopen
```

Restore requires exclusive access to the database directory: on Windows an open
engine makes the swap fail loudly; on Unix an open handle would silently fork
state, so always close/drop handles first (the SDK wrapper reopens for you).

## Skills API — Separate Module

> **Source:** `vantadb-skills` crate (`skill_store`/`skill_versioning` modules) — this is a **separate module** from `VantaEmbedded`.

Versioned skill store (agent skills / memory skills) — port of the TDAM
`skill_store`/`skill_versioning` modules onto the entity pattern. Each skill
is append-only: every write appends a new version and demotes the previous
head. All writes use an optimistic lock: pass the current `version` you read,
or the write fails with `ExecutionConflict`.

| Method | Description |
|--------|-------------|
| `SkillStore::new(&engine)` | Wrap a storage engine reference |
| `create(input)` | Create a skill as version 1. Idempotent: same `(owner_agent, name)` + same content returns the existing head (`idempotent = true`) |
| `update(skill_id, expected_version, input)` | Append a new version replacing description/content. Fails with `ExecutionConflict` when `expected_version` is stale |
| `patch(skill_id, expected_version, input)` | Append a new version changing only the provided fields |
| `delete(skill_id, expected_version)` | Delete all versions plus the head index row. Returns `true` when the skill existed |
| `get_head(skill_id)` | Current head version, or `None` |
| `get_version(skill_id, version)` | A specific immutable version, or `None` |
| `list_versions(skill_id, limit, offset)` | All versions, newest first |
| `list(options)` | Skill heads with optional `owner_agent` / `name_prefix` filters, ordered by name |
| `cleanup_expired_versions(skill_id, now)` | Delete expired non-head versions, keeping the 3 most recent (TTL keep-recent=3) |

### Types

- **`SkillRecord`** — one immutable version: `skill_id`, `version`, `is_head`,
  `owner_agent`, `name`, `description`, `content`, `content_hash` (FNV-1a
  64-bit hex, idempotency only), `metadata` (map), `created_at`, `updated_at`,
  `expires_at`.
- **`SkillCreateInput`** — `name`, `description`, `content`, `owner_agent`,
  `metadata`, optional `ttl_secs`.
- **`SkillUpdateInput`** — `description`, `content`, optional `metadata`
  (`None` keeps the previous metadata).
- **`SkillPatchInput`** — optional `description`, `content`, `metadata`.
- **`SkillListOptions`** — optional `owner_agent`, `name_prefix`, `limit`
  (default 50), `offset`.
- **`SkillListPage`** — `items`, `total`.
- **`SkillWriteResult`** — `record`, `idempotent` (no version appended).

Name and `owner_agent` are immutable across versions; uniqueness is enforced
per `(owner_agent, name)` while a head exists. Content is stored as-is
(no FTS/vector index in v1 — listing is by name/owner, not semantic search).

## GraphRAG

| Method | Description |
|--------|-------------|
| `graphrag_search(namespace, query, query_vector)` | Run the GraphRAG pipeline: seed → expand → retrieve → generate context. Uses the default pipeline configuration (seed_k=10, hops=2, max=100, top_k=20). For custom settings, construct `GraphRagPipeline` directly. `query` and `query_vector` are optional |

## Maintenance

| Method | Description |
|--------|-------------|
| `flush()` | Flush [[wal\|WAL]] + [[hnsw\|HNSW]] to disk for durability |
| `compact_wal()` | Archive [[wal\|WAL]] file and start fresh |
| `vacuum()` | Purge tombstoned nodes from the [[hnsw\|HNSW]] index. Returns a `VacuumReport` with counts and timing |
| `pipeline()` | Run the segment optimizer pipeline (vacuum → merge → reindex). Each phase is logged independently; a phase failure does not abort subsequent phases |
| `optimizer_config()` | Return the current segment optimizer configuration |
| `set_optimizer_config(config)` | Override the segment optimizer configuration. Takes effect on the next pipeline invocation |
| `purge_expired()` | Delete TTL-expired records. Returns count purged |
| `rebuild_index()` | Rebuild ANN ([[hnsw\|HNSW]]), derived, and text indexes. Returns `VantaIndexRebuildReport` |
| `reindex_hnsw_from_text(namespace, page_size)` | Rebuild the vector index from text records using cursor-based pagination (`page_size` default 1000, max 1000). Safe alternative to unbounded enumeration |
| `compact_layout()` | BFS-order physical compaction of vector store. Returns nodes compacted |

## Export / Import

| Method | Description |
|--------|-------------|
| `export_namespace(path, namespace)` | Export namespace as JSONL. Returns `VantaExportReport` |
| `export_all(path)` | Export all namespaces as JSONL. Returns `VantaExportReport` |
| `import_file(path)` | Import from JSONL file. Returns `VantaImportReport` |
| `bulk_import_file(path)` | Bulk-import from a binary `.vdbdump` file. Bypasses per-record validation for raw throughput; commits in batches sized by `bulk_commit_interval` (default 10000) |
| `bulk_import_stream(reader)` | Bulk-import records from a binary stream. Format: 8-byte magic `VDBJSON\n`, 1-byte version `0x01`, 8-byte LE record count, then serde_json-serialized `Vec<VantaMemoryInput>`. Same batching/validation behavior as `bulk_import_file` |

Free-function helper (used by the MCP `import` tool to rebuild records from JSONL content received as a string): `vantadb::sdk::record_from_export_line(VantaMemoryExportLine) -> Result<VantaMemoryRecord>` — the inverse of [`export_line_from_record`](#export--import); recomputes the deterministic node id from namespace/key.

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
    pub target: u128,
    pub label: String,
    pub weight: f32,
    /// True for the auto-created reverse half of a bidirectional edge
    /// (`add_edge`). Load-bearing for directional traversal.
    /// `#[serde(default)]` — legacy JSON without these fields deserializes.
    pub reverse: bool,
    /// Logical creation timestamp (Unix-ms); `0` when unknown (legacy data).
    pub created_at_ms: u64,
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
        node_id: Option<u128>,
    },
    StaleContext {
        node_id: u128,
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

- `VantaError::NodeNotFound(u128)` — node ID not found
- `VantaError::DuplicateNode(u128)` — duplicate node ID on insert
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
- `VantaError::NodeIdCollision(u128)` — two nodes have colliding IDs
- `VantaError::CycleDetected` — cycle detected in graph operation
- `VantaError::ValidationError { field, reason }` — input validation failed
- `VantaError::Timeout { operation, duration_ms }` — operation exceeded time budget
- `VantaError::UnsupportedOperation { operation, detail }` — unsupported operation
- `VantaError::ExecutionConflict { resource, detail }` — concurrent modification conflict
- `VantaError::IqlError(ChainedError)` — IQL query processing error
- `VantaError::IqlParseError { msg, line, col }` — IQL parse error at a specific line/col
- `VantaError::CliError(ChainedError)` — CLI command processing error
- `VantaError::SearchError(ChainedError)` — search execution error
- `VantaError::RuntimeError(ChainedError)` — unexpected runtime error
- `VantaError::RestoreError(ChainedError)` — restore operation error
- `VantaError::BackupError(ChainedError)` — backup operation error
- `VantaError::BackendError(ChainedError)` — storage backend error
- `VantaError::InvalidInput(String)` — invalid input provided
- `VantaError::SchemaError(String)` — schema-related error
- `VantaError::NoVectorForKey(String)` — a record exists but does not carry a vector, so vector-based operations (e.g. `similar_to_key`) cannot proceed
- `VantaError::Generic(ChainedError)` — generic catch-all error

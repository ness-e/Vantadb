---
title: Embedded SDK Reference
kind: reference
status: active
description: "Source: src/sdk/mod.rs, src/sdk/builder.rs, src/sdk/api/.rs, src/sdk/graph.rs, src/sdk/search/mod.rs"
tags: [vantadb, api]
type: api
last_reviewed: "2026-09-01"
---

# Embedded SDK Reference

> Core Rust SDK struct `Embedded` — the primary entry point for all embedded database operations. Used directly in Rust and exposed via [PyO3](../user/glosario/pyo3.md) (Python), wasm-bindgen (TypeScript), and [MCP](./MCP.md).
>
> **Naming (ADR-0047 anti-stutter):** canonical names are `Embedded`, `Config`,
> `MemoryRecord`, `SearchHit`, `Error`, `MemoryInput`, `MemoryFilter`. Legacy
> `Vanta*` aliases (`VantaEmbedded`, `VantaConfig`, …) were removed in 0.6.0
> (AST-010). `Header` (on-disk format)
> and `VANTADB_*` error codes (wire) are intentionally unchanged.

**Source:** `src/sdk/mod.rs`, `src/sdk/builder.rs`, `src/sdk/api/*.rs`, `src/sdk/graph.rs`, `src/sdk/search/mod.rs`

## Construction

```rust
use vantadb::{Embedded, Config, BackendKind};

// Open with defaults (path-based)
let db = Embedded::open("./vanta_data").unwrap();

// Open with full configuration
let config = Config {
    storage_path: "./vanta_data".into(),
    memory_limit: Some(512_000_000),
    read_only: false,
    backend_kind: BackendKind::Fjall,
    ..Default::default()
};
let db = Embedded::open_with_config(config).unwrap();

// Wrap an existing StorageEngine handle
let db = Embedded::from_engine(engine);
```

| Method | Description |
|--------|-------------|
| `open(path)` | Open or create a database at the given filesystem path with default config |
| `open_with_config(config)` | Open or create with full `Config` |
| `from_engine(engine)` | Wrap an existing `Arc<StorageEngine>` handle |

### Audit Log

Set `Config.audit_log_path` (or `VANTADB_AUDIT_LOG_PATH`) to enable append-only JSONL audit logging of write/delete/export/import operations. Disabled by default (`None`).

```rust
let config = Config {
    storage_path: "./vanta_data".into(),
    audit_log_path: Some("audit/audit.jsonl".into()),
    ..Default::default()
};
let db = Embedded::open_with_config(config).unwrap();
```

Each line is one JSON object: `{"timestamp":"2026-08-02T12:34:56Z","op":"put","namespace":"docs","key":"a","outcome":"ok","reason":null}`. Ops: `put`, `put_batch`, `delete`, `delete_by_filter`, `export_namespace`, `export_all`, `import_file`, `bulk_import_file`, `bulk_import_stream`. Read-only ops are not audited. See `docs/user/operations/CONFIGURATION.md`.

## Memory (Namespace-scoped) API

CRUD operations for persistent memory records identified by `(namespace, key)` pairs.

| Method | Description |
|--------|-------------|
| `put(input: MemoryInput)` | Insert or update a memory record. Returns `MemoryRecord` |
| `put_batch(inputs: Vec<MemoryInput>)` | Batch insert/update (parallel, up to 5x faster). Returns `Vec<MemoryRecord>` |
| `get(namespace, key)` | Retrieve a record by namespace+key. Returns `Option<MemoryRecord>` |
| `get_version(namespace, key, version)` | Retrieve the record as it was at the given version (VS-CORE-07). Returns `Option<MemoryRecord>` — `None` if that version was never persisted (unknown key, purged by the retention cap, or deleted). Snapshot durability is best-effort post-commit, so a crash window can leave a version gap — degraded but never corrupt |
| `versions(namespace, key)` | List every retained version of a record, ascending (v1..vN) (VS-CORE-07). Returns `Vec<MemoryRecord>` — empty if the key does not exist or has no history; expired versions are included as historical data until purged. `get_version(namespace, key, vN)` of the last element matches the live record |
| `delete(namespace, key)` | Delete a record. Returns `bool` (true if existed) |
| `delete_by_filter(namespace, filter)` | Delete all records in a namespace matching a metadata filter. Returns `u64` count of deleted records. Filter uses `MemoryFilter` (equality + operators). Empty filter rejected to prevent accidental full-namespace deletion |
| `count(namespace, filter)` | Count memory records in a namespace, optionally filtered by metadata. Returns `u64`. Filter uses `MemoryFilter` (equality + operators). `None` counts all records |
| `list(namespace, options)` | List records in a namespace with cursor pagination. Returns `MemoryListPage` |
| `list_namespaces()` | List all namespaces. Returns `Vec<String>` |
| `search(request: MemorySearchRequest)` | [hybrid-search](../user/glosario/hybrid-search.md) (vector + lexical) search. Returns `Vec<MemorySearchHit>` |
| `search_page(request: MemorySearchRequest)` | Same pipeline as `search` with cursor-based pagination (WIRE-08). Returns `MemorySearchPage { hits, next_cursor, abstained, abstention_reason }`: `next_cursor` is `Some` only when the page is full (`hits.len() == top_k`); a short page is the last page. `abstained`/`abstention_reason` carry the selective-abstention signal (ADR-0046 §D2, SCH-07) when the configured `confidence_threshold` empties the page. Resume by passing the token back in `MemorySearchRequest::cursor` — the next page returns hits after the last returned hit's identity in the current ranking. Resume is **best-effort, not a snapshot**: a hit returned in a previous page can be returned again (or skipped) when interleaved writes reorder its rank across the anchor (BM25/IDF are recalculated corpus-wide on every write); writes that rank *before* the anchor are never duplicated. The token is bound to the request's plan fingerprint (mismatch → `SEARCH_CURSOR_INVALID`) and to the current process; pagination is rejected with `mmr`/`group_by`. See [SEARCH_PARITY](./SEARCH_PARITY.md) for the Milvus/Qdrant mapping |
| `search_with_method(request, method)` | Same as `search` with an explicit index backend override for the dense-vector portion: `Some(IndexType::Ivf)` / `Some(IndexType::Scann)` / `Some(IndexType::Flat)` / `Some(IndexType::Hnsw)`. `None` (default) keeps automatic engine routing untouched; the shared engine config is never mutated (thread-safe, per-search override) |
| `search_page_with_method(request, method)` | SCH-07: the page-shaped mirror of `search_with_method` — same hits as `search_with_method` plus the `MemorySearchPage` envelope (`next_cursor`, `abstained`, `abstention_reason`). Used by page-shaped transports (MCP `search_with_method`) so the abstention signal is never dropped at the `Vec` edge |
| `search_with_entity_boost(request, boost)` | Same as `search` with the opt-in deterministic entity-cluster boost (WIRE-05). Hits sharing an entity cluster with other fused candidates receive an additive delta (`weight × peers × 1/(rrf_k+1)`) before the final ranking. Returns `EntityBoostedSearch { hits, boost_report }` with per-hit provenance (cluster, peers, `base_score`, `delta`) — reversible, no stored data mutated; an empty `EntityBoost` is byte-identical to `search`. Single-channel routes (text-only/vector-only/sparse-only) are returned unchanged |
| `search_multi(namespaces, request)` | Search across multiple namespaces, merging results by descending score, capped at `request.top_k`. Namespaces that produce no results or fail validation are silently skipped; an empty `namespaces` slice returns an empty `Vec` |
| `similar_to_key(namespace, key, top_k)` | Vector similarity search from an existing record's vector, post-filtered to `namespace`. Errors `NotFound` if the key does not exist and `NoVectorForKey` if the record carries no vector |
| `explain_memory_search(request)` | Search with detailed score breakdown. Returns `SearchExplanation` |
| `namespace_stats(expiring_soon_window_ms)` | Per-namespace statistics: total records, records expiring within the window, already-expired records. Single full scan (no N paginated `count`/`list` calls). `None` uses the 24h default window. Returns `NamespaceStatsMap` |
| `supersede(namespace, old_key, new_key)` | Mark `old_key` as superseded by `new_key`: the old record keeps its data (soft-dead, recoverable) but gains `superseded_by`/`superseded_at_ms`, and can be hidden from search/list with `exclude_superseded`. Errors if either key is missing, if `old_key == new_key`, or if the old record is already superseded (idempotency guard) |
| `purge_expired()` | Scan all memory records and physically delete those whose TTL has expired. Returns `u64` count of purged records |
| `bulk_import_file(path)` | Bulk-import from a binary `.vdbdump` file. Bypasses per-record validation for raw throughput; commits in batches sized by `bulk_commit_interval` (default 10000) |
| `bulk_import_stream(reader)` | Bulk-import records from a binary stream. Format: 8-byte magic `VDBJSON\n`, 1-byte version `0x01`, 8-byte LE record count, then serde_json-serialized `Vec<MemoryInput>`. Same batching/validation behavior as `bulk_import_file` |

### Version History (VS-CORE-07)

Every `put`/`put_batch` snapshots the new record into a `Versions` partition keyed by `(namespace, key, version)`; `version` increments monotonically per key. `Config.version_history_limit` caps retained snapshots per key (default `Some(32)`, FIFO eviction of the oldest beyond the cap — see `docs/user/operations/CONFIGURATION.md`); `delete` and `purge_expired` purge the full history. Imports (`import_file`/`bulk_import_*`) do **not** write snapshots. Only the embedded (native) SDK and the CLI/bridge expose version history.

### Bulk Import

Bulk import operations bypass per-record validation for maximum throughput. Records are committed in batches sized by `Config::bulk_commit_interval` (default 10,000). The binary format (`.vdbdump`) uses magic `VDBJSON\n`, version `0x01`, LE record count, then JSON-serialized `Vec<MemoryInput>`.

### `MemoryInput`

```rust
pub struct MemoryInput {
    pub namespace: String,
    pub key: String,
    pub payload: String,
    pub metadata: MemoryMetadata,  // BTreeMap<String, Value>
    pub vector: Option<Vec<f32>>,       // 384-dim embedding
    pub ttl_ms: Option<u64>,            // auto-expiry in ms from now
    // v2 (ADR-046 §D2, SCH-02/SCH-05) — all optional, additive:
    pub valid_at_ms: Option<u64>,             // valid-time start (default: created_at)
    pub confidence_class: Option<ConfidenceClass>, // Asserted (default) | Derived
    pub confidence: Option<f32>,              // asserted only; derived scores are computed
    pub derived_from: Option<Vec<String>>,    // required non-empty when Derived
    pub quarantine: bool,                     // T1 write-time quarantine (default false)
}
```

Records written as `Derived` must declare `derived_from` (≥1 same-namespace parent
key) and must **not** declare `confidence` — the engine computes
`min(parents) × 0.9` (ADR-0046 §D4a/D4b; violations are `Validation` errors).

### `MemorySearchRequest`

```rust
pub struct MemorySearchRequest {
    pub namespace: String,
    pub query_vector: Vec<f32>,       // empty = no vector search
    pub query_sparse: Option<SparseVector>, // sparse-dot arm (fused with RRF)
    pub filters: MemoryMetadata, // equality filter on metadata
    pub text_query: Option<String>,   // BM25 lexical query
    pub top_k: usize,                 // default: 10
    pub distance_metric: DistanceMetric, // Cosine (default) or Euclidean
    pub explain: bool,                // include score breakdown
    pub exclude_superseded: bool,     // ADR-0028 soft-delete filter (+ ended windows, ADR-046 §D3-6)
    pub min_confidence: Option<f32>,  // SCH-04: opt-in confidence floor [0, 1]
    pub as_of_ms: Option<u64>,        // SCH-03: valid-time point (ADR-046 §D3)
    pub valid_window: Option<ValidWindow>, // SCH-03: { from_ms, to_ms } overlap
    pub include_quarantined: bool,    // SCH-05: quarantine view (default false)
    pub search_profile: Option<SearchProfileConfig>, // MEM-01 (mode, rrf_k, candidate_k)
    pub range: Option<RangeFilter>,   // WIRE-08: score bounds [min_score, max_score]
    pub group_by: Option<GroupByConfig>, // WIRE-08: { field, group_size }
    pub mmr: Option<MmrConfig>,       // WIRE-08: { lambda, fetch_k }
    pub cursor: Option<String>,       // WIRE-08: opaque token from `search_page`
}
```

*Note: Lexical search uses the [BM25](../user/glosario/bm25.md) algorithm.*

#### v2 search options (ADR-0046, SCH-03/04/05)

| Field | Contract |
|-------|----------|
| `as_of_ms: Option<u64>` | Valid-time point: keep only records valid at that instant, `valid_at_ms <= T < invalid_at_ms` (start inclusive, end exclusive). The **valid-time** axis — not transaction time. `None` = no filter (default unchanged) |
| `valid_window: ValidWindow { from_ms, to_ms }` | Valid-time window **overlap** filter: keep records whose `[valid_at_ms, invalid_at_ms)` intersects `[from_ms, to_ms)`. `from_ms < to_ms` is validated at the boundary (`SEARCH_OPTIONS_INVALID`). `None` = no filter |
| `min_confidence: Option<f32>` | Opt-in confidence floor: keep only hits whose record `confidence >= min_confidence` (finite, within `[0, 1]`; rejected otherwise — never clamped). Does **not** emit the abstention signal (that is `Config::confidence_threshold`) |
| `include_quarantined: bool` | Quarantine view: `false` (default) excludes quarantined records from search/list/retrieval; `true` includes them. `get` always returns a quarantined record with visible state — never a silent 404 |
| `exclude_superseded` (extended) | Beyond ADR-0028 supersession, also drops records whose validity window ended (`invalid_at_ms <= now`). The reference instant is read once per request |

#### WIRE-08 search options

| Field | Contract |
|-------|----------|
| `range: RangeFilter { min_score, max_score }` | Post-ranking score filter, **inclusive** bounds in score space (higher = more relevant, matching `MemorySearchHit::score`). `None` bound = unbounded. Rejected (`SEARCH_OPTIONS_INVALID`) when non-finite or `min_score > max_score`. Score-space equivalent of Milvus `radius`/`range_filter` and Qdrant `score_threshold` — see [SEARCH_PARITY](./SEARCH_PARITY.md). The fetch window deepens (bounded by 10 000) when the bounds can shorten the page |
| `group_by: GroupByConfig { field, group_size }` | Post-ranking group-by on a metadata field: walk hits in rank order, keep a hit while its group value has fewer than `group_size` selected hits (default `1`); `top_k` caps **total hits** (unlike Milvus/Qdrant, where `limit` caps the number of groups). Records missing the field form their own group. `field` must be non-empty and `group_size >= 1` |
| `mmr: MmrConfig { lambda, fetch_k }` | Maximal Marginal Relevance reranking: `lambda` in `[0, 1]` (default `0.5`; `1.0` = pure relevance = identity order, `0.0` = pure diversity). `fetch_k` is the candidate window (default `top_k * 5`, clamped to `[top_k, 16384]`). Relevance is min-max normalized inside the window so `lambda` is comparable across fusion routes; diversity uses cosine between the records' dense vectors (records without vectors compete on relevance only). Mutually exclusive with `cursor` |
| `cursor: Option<String>` | Opaque continuation token returned by `search_page`. Valid only for the same plan fingerprint (namespace, query, filters, metric, profile, range) and the same process — never persist or parse it. Resuming skips past the anchor hit's identity; the fetch window grows (bounded by 10 000) to compensate for writes that landed before the anchor. Mutually exclusive with `mmr`/`group_by` |

`search_page`/`search_page_with_method` return
`MemorySearchPage { hits: Vec<MemorySearchHit>, next_cursor: Option<String>, abstained: bool, abstention_reason: Option<AbstentionReason> }`.
When `Config::confidence_threshold` is set and every candidate falls below it,
`abstained = true` and `abstention_reason` carries the stable code
(`no_candidates_above_threshold` | `all_quarantined`) — never a silent empty
page. The signal also propagates to the single-namespace HTTP `SearchPageV2`
and the MCP search envelope (SCH-07); array-shaped binding APIs have no page.

### `MemoryRecord`

```rust
pub struct MemoryRecord {
    pub namespace: String,
    pub key: String,
    pub payload: String,
    pub metadata: MemoryMetadata,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
    pub version: u64,
    pub node_id: u128,
    pub vector: Option<Vec<f32>>,
    pub expires_at_ms: Option<u64>,
    // v2 (ADR-046 §D2, SCH-02/SCH-05) — all serde-defaulted:
    pub valid_at_ms: u64,                    // start of the validity window
    pub invalid_at_ms: Option<u64>,          // end (exclusive); None = open
    pub confidence_class: ConfidenceClass,   // Asserted | Derived
    pub confidence: f32,                     // [0, 1]; D_a = 1.0 for asserted
    pub last_validated_at_ms: Option<u64>,   // last successful re-validation
    pub derived_from: Vec<String>,           // parent keys (derived only)
    pub quarantined_at_ms: Option<u64>,      // Some(t) = quarantined since t
    pub quarantine_reason: Option<String>,   // explicit_write | unreviewed_import | ...
    pub quarantined_by: Option<String>,      // principal or system:<op>
    pub quarantine_review_due_ms: Option<u64>, // T3 review deadline (never auto-promotes)
}
```

### `NamespaceStats`

```rust
pub struct NamespaceStats {
    pub count: u64,          // total records in the namespace (includes not-yet-purged expired)
    pub expiring_soon: u64,  // records expiring within the window (never includes expired)
    pub expired: u64,        // records whose TTL has already passed
}

pub type NamespaceStatsMap = BTreeMap<String, NamespaceStats>;
```

`namespace_stats` returns one `NamespaceStats` per namespace, keyed by namespace in sorted order. `count` counts all physical records (including expired ones not yet purged); the read-visible subset is available via `count`/`list` (lazy TTL eviction). A record is `expired` if `expires_at_ms <= now` and `expiring_soon` if `now < expires_at_ms <= now + window`; a record never counts as both. Records without a TTL count only toward `count`.

## Scene Nodes (L2 Memory Anchors) — Separate Module

> **Source:** `src/entity/scene.rs` (`entity::scene::SceneNodeStore`) — documented as part of the
> F5 memory pipeline closure (MEM-12 debt). This is a **separate module** from `Embedded`.

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
rejected with `Error::InvalidInput`.

## Entity Linking + Entity Boost (Separate Module) — WIRE-05

> **Source:** `src/entity/linking.rs` (`entity::linking`) and
> `src/sdk/search/fusion.rs`. Deterministic multi-signal record linkage
> (Fellegi-Sunter + embeddings), **no LLM judge**, plus an opt-in
> entity-cluster boost for RRF fusion.

### Deterministic linking

`LinkSignal` carries one comparison signal per kind (`SignalKind::Name` /
`Email` / `Phone` / `Embedding`); a candidate is a `LinkEntity { id, signals }`.
The pair score is the classic Fellegi-Sunter log-likelihood ratio sum:

```text
W = prior_bits + Σ_i log2(m_i / u_i)              (signal i agrees)
                + Σ_i log2((1 - m_i) / (1 - u_i)) (signal i disagrees)
                + 0                                (signal absent on either side)
Pr(match) = 2^W / (1 + 2^W);  auto-link when W >= auto_link_bits,
                              review when W >= review_bits, else distinct.
```

Defaults are conservative: `auto_link_bits = 12` is above the strongest
single-signal agreement weight (≈ 10.95), so an automatic merge requires **at
least two independent signals** to agree; `review_bits = 4`;
`embedding_threshold = 0.90` (cosine); per-kind m/u weights are overridable
through `SignalWeight::new(m, u)` + `LinkConfig::with_signal_weight`. Manual
duplicate marking (`mark_duplicate`, MGR-05 step 1) is supported through
`ManualLink` — a manual pair merges regardless of score and is reported with
`manual: true`. Identical inputs always produce identical scores and reports;
`link_entities` never mutates stored data (clusters are a derived view).

```rust
use vantadb::entity::linking::{link_entities, LinkConfig, LinkEntity, LinkSignal, ManualLink, SignalKind};

let candidates = vec![
    LinkEntity::new("usr-1", vec![
        LinkSignal::text(SignalKind::Name, "Alice Smith"),
        LinkSignal::text(SignalKind::Email, "alice@example.com"),
    ]),
    LinkEntity::new("usr-2", vec![
        LinkSignal::text(SignalKind::Name, "alice  smith"),
        LinkSignal::text(SignalKind::Email, "ALICE@example.com"),
    ]),
];

// `manual` is optional: `mark_duplicate` pairs forced into one cluster.
let report = link_entities(&candidates, &LinkConfig::default(), &[ManualLink::new("usr-1", "usr-2")])?;
assert_eq!(report.clusters[0].canonical, "usr-1");
```

Entity fields can be mapped to signals with `signals_from_fields(&fields, &map)`
(`FieldValue::String` for text kinds, `FieldValue::ListFloat` for embeddings).

### Opt-in RRF entity boost

`EntityBoost` maps fused-record identities `(namespace, key)` to a cluster label
(typically `LinkCluster::canonical`). When two or more hits of a cluster
co-occur in one fused candidate set, each receives `weight × peers ×
1/(rrf_k+1)` — a fraction of the strongest single-channel RRF contribution
(`EntityBoost::DEFAULT_WEIGHT = 0.25`). The boost is opt-in and reversible
(`base_score` is exact; `score − delta ≈ base_score` within f32 rounding):
`EntityBoostReport` carries per-hit provenance (cluster, peers, `base_score`,
`delta`) and an empty boost is byte-identical to `search`.

```rust
use vantadb::sdk::EntityBoost;

let mut boost = EntityBoost::new();
for cluster in &report.clusters {
    for member in &cluster.members {
        boost = boost.link("my-namespace", member, cluster.canonical.as_str());
    }
}
let outcome = db.search_with_entity_boost(request, &boost)?; // EntityBoostedSearch
println!("{} hits, {} boosted", outcome.hits.len(), outcome.boost_report.len());
```

The boost applies to RRF-fused routes (hybrid, text+sparse, vector+sparse);
single-channel routes have no fusion score to boost. Measuring the boost is
tracked by the VER-08 harness (roadmap F5).

## Node / Graph API

Low-level operations on the node-graph model (numeric node IDs, edges, graph traversal).

| Method | Description |
|--------|-------------|
| `insert_node(input: NodeInput)` | Insert a graph node with content, vector, and fields |
| `get_node(id: u128)` | Retrieve a node by numeric ID. Returns `Option<NodeRecord>` |
| `delete_node(id, reason)` | Delete a node with auditable reason (tombstone) |
| `add_edge(source_id, target_id, label, weight, created_at_ms)` | Add a directed edge between two nodes with optional weight and creation timestamp. Automatically creates a reverse edge |
| `remove_edge(source_id, target_id, label)` | Remove all edges between two nodes with the given label (both directions) |
| `collect_graph_nodes()` | Collect every live non-memory-record node as `Vec<NodeRecord>` (edges with labels resolved). Excludes nodes carrying a namespace field — those belong to the memory snapshot. Used by the WASM binding to persist the graph store alongside `db_state.json` |
| `restore_graph_nodes(records: Vec<NodeRecord>)` | Restore graph nodes exported by `collect_graph_nodes` into the engine: re-interns edge labels, preserves weights, direction (`reverse`) and creation timestamps. Returns the number of nodes restored |
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
| `search_vector(vector, top_k)` | Pure [hnsw](../user/glosario/hnsw.md) vector search. Returns `Vec<SearchHit>` |
| `query(iql_query)` | Execute IQL query string. Returns `QueryResult` |
| `vacuum()` | Purge tombstoned nodes from the HNSW index. Returns a `VacuumReport` with counts and timing |
| `pipeline(mode)` | Run the segment optimizer pipeline (vacuum → merge → reindex). Each phase is logged independently; a phase failure does not abort subsequent phases |
| `optimizer_config()` | Return the current segment optimizer configuration |
| `set_optimizer_config(config)` | Override the segment optimizer configuration. Takes effect on the next pipeline invocation |

### `NodeInput`

```rust
pub struct NodeInput {
    pub id: u128,
    pub content: Option<String>,
    pub vector: Option<Vec<f32>>,
    pub fields: Fields,  // BTreeMap<String, Value>
}
```

### `NodeRecord`

```rust
pub struct NodeRecord {
    pub id: u128,
    pub fields: Fields,
    pub vector: Option<Vec<f32>>,
    pub vector_dimensions: usize,
    pub edges: Vec<EdgeRecord>,
    pub confidence_score: f32,
    pub importance: f32,
    pub hits: u32,
    pub last_accessed: u64,
    pub epoch: u32,
    pub tier: StorageTier,  // Hot / Cold
    pub is_alive: bool,
}
```

| `recover_archived_nodes(summary_id)` | Recover shadow-archived nodes that belonged to a summary node. Scans TombstoneStorage for nodes with a `belonged_to` edge targeting `summary_id`, re-activates them, and inserts them back into the active store. Returns `Vec<NodeRecord>` |

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
| `recover_archived_nodes(summary_id)` | Recover shadow-archived nodes that belonged to a summary node. Scans TombstoneStorage for nodes with a `belonged_to` edge targeting `summary_id`, re-activates them, and inserts them back into the active store. Returns `Vec<NodeRecord>` |

## Snapshots API

Instant filesystem snapshots via hard links (Unix) or copy (Windows).

| Method | Description |
|--------|-------------|
| `create_snapshot(name)` | Create an instant point-in-time snapshot. All data files in the storage directory are hard-linked into `<data_dir>/snapshots/<name>` (O(1)) |
| `list_snapshots()` | List all existing snapshot names. Returns `Vec<String>` |
| `restore_from(config, name)` | DESTRUCTIVE static associated function: replace the live database directory with the contents of snapshot `<name>`, then reopen. Close every open handle first (`db.close()?; let db = Embedded::restore_from(config.clone(), "name")?;`). The current data is staged aside (atomic rename, sibling of `data_dir`) with best-effort rollback on failure; all snapshots survive the swap. `name` must be a plain identifier (path traversal rejected). Indexes rebuild from storage on reopen |

### Restore flow

```rust,ignore
db.close()?;                                       // release the fs2 lock + handles
let db = Embedded::restore_from(config.clone(), "snap-1")?; // swap + reopen
```

Restore requires exclusive access to the database directory: on Windows an open
engine makes the swap fail loudly; on Unix an open handle would silently fork
state, so always close/drop handles first (the SDK wrapper reopens for you).

## Skills API — Separate Module

> **Source:** `vantadb-skills` crate (`skill_store`/`skill_versioning` modules) — this is a **separate module** from `Embedded`.

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
| `flush()` | Flush [wal](../user/glosario/wal.md) + [hnsw](../user/glosario/hnsw.md) to disk for durability |
| `compact_wal()` | Archive [wal](../user/glosario/wal.md) file and start fresh |
| `vacuum()` | Purge tombstoned nodes from the [hnsw](../user/glosario/hnsw.md) index. Returns a `VacuumReport` with counts and timing |
| `pipeline()` | Run the segment optimizer pipeline (vacuum → merge → reindex). Each phase is logged independently; a phase failure does not abort subsequent phases |
| `optimizer_config()` | Return the current segment optimizer configuration |
| `set_optimizer_config(config)` | Override the segment optimizer configuration. Takes effect on the next pipeline invocation |
| `purge_expired()` | Delete TTL-expired records. Returns count purged |
| `rebuild_index()` | Rebuild ANN ([hnsw](../user/glosario/hnsw.md)), derived, and text indexes. Returns `IndexRebuildReport` |
| `reindex_hnsw_from_text(namespace, page_size)` | Rebuild the vector index from text records using cursor-based pagination (`page_size` default 1000, max 1000). Safe alternative to unbounded enumeration |
| `compact_layout()` | BFS-order physical compaction of vector store. Returns nodes compacted |

## Export / Import

| Method | Description |
|--------|-------------|
| `export_namespace(path, namespace)` | Export namespace as JSONL. Returns `ExportReport` |
| `export_all(path)` | Export all namespaces as JSONL. Returns `ExportReport` |
| `import_file(path)` | Import from JSONL file. Returns `ImportReport` |
| `bulk_import_file(path)` | Bulk-import from a binary `.vdbdump` file. Bypasses per-record validation for raw throughput; commits in batches sized by `bulk_commit_interval` (default 10000) |
| `bulk_import_stream(reader)` | Bulk-import records from a binary stream. Format: 8-byte magic `VDBJSON\n`, 1-byte version `0x01`, 8-byte LE record count, then serde_json-serialized `Vec<MemoryInput>`. Same batching/validation behavior as `bulk_import_file` |

Free-function helper (used by the MCP `import` tool to rebuild records from JSONL content received as a string): `vantadb::sdk::record_from_export_line(MemoryExportLine) -> Result<MemoryRecord>` — the inverse of [`export_line_from_record`](#export--import); recomputes the deterministic node id from namespace/key.

Third-party importers (Mem0 / Zep / Letta → JSONL v2): `vantadb::sdk::importers::{mem0, zep, letta}` — each exposes `convert_str`/`convert_file` returning `Conversion { lines, stats }`, plus `Conversion::into_records` (import through the canonical transport) and `Conversion::write_jsonl` (the `vanta-cli import` input). The interchange contract, per-source mapping tables and declared discards live in [MEMORY_INTERCHANGE_FORMAT.md](./MEMORY_INTERCHANGE_FORMAT.md).

## Text Index Diagnostics

| Method | Description |
|--------|-------------|
| `audit_text_index(namespace)` | Read-only structural audit. Returns `TextIndexAuditReport` |
| `audit_text_index_deep(namespace)` | Deep audit (decodes TF, positions, DF, doc lengths). Returns `TextIndexAuditReport` |
| `repair_text_index()` | Rebuild text index from canonical storage. Returns `TextIndexRepairReport` |

## Observability

| Method | Description |
|--------|-------------|
| `operational_metrics()` | Snapshot of runtime metrics. Returns `OperationalMetrics` |
| `capabilities()` | Stable runtime capabilities. Returns `Capabilities` |
| `generate_snippet(payload, query, with_highlighting)` | Highlighted text snippet. Returns `Option<String>` |

## Lifecycle

| Method | Description |
|--------|-------------|
| `close()` | Flush and release the engine handle |
| `debug_memory_breakdown()` | *(debug-only)* Memory usage breakdown as JSON |

## Data Types

### `Value`

```rust
pub enum Value {
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

### `MemoryListOptions`

```rust
pub struct MemoryListOptions {
    pub filters: MemoryMetadata,  // equality filter on metadata fields
    pub filter_ops: Option<MemoryFilter>, // operator filters ($gt/$gte/...)
    pub limit: usize,                  // max records to return (default: 100)
    pub cursor: Option<usize>,         // pagination cursor from previous page
    pub exclude_superseded: bool,      // drop superseded + ended windows
    pub as_of_ms: Option<u64>,         // v2: valid-time point (SCH-03)
    pub valid_window: Option<ValidWindow>, // v2: { from_ms, to_ms } overlap
    pub include_quarantined: bool,     // v2: quarantine view (SCH-05)
    pub min_confidence: Option<f32>,   // v2: confidence floor [0, 1] (SCH-07)
}
```

### `MemoryListPage`

```rust
pub struct MemoryListPage {
    pub records: Vec<MemoryRecord>,
    pub next_cursor: Option<usize>,
}
```

### `MemorySearchHit`

```rust
pub struct MemorySearchHit {
    pub record: MemoryRecord,
    pub score: f32,
    pub explanation: Option<SearchExplanationHit>,
}
```

### `EdgeRecord`

```rust
pub struct EdgeRecord {
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

### `RuntimeProfile` / `StorageTier`

```rust
pub enum RuntimeProfile {
    Enterprise,
    Performance,
    LowResource,
}

pub enum StorageTier {
    Hot,   // RAM-cached
    Cold,  // on-disk
}
```

### `QueryResult`

```rust
pub enum QueryResult {
    Read(Vec<NodeRecord>),
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

### `SearchExplanation`

```rust
pub struct SearchExplanation {
    pub route: String,                          // "hybrid", "text_only", "vector_only"
    pub hits: Vec<SearchExplanationHit>,
    pub fusion_report: Option<HybridFusionReport>,
}
```

### `HybridFusionReport`

```rust
pub struct HybridFusionReport {
    pub text_candidates: usize,
    pub vector_candidates: usize,
    pub fused_candidates: usize,
    pub rrf_k: usize,
}
```

### `Capabilities`

```rust
pub struct Capabilities {
    pub runtime_profile: RuntimeProfile,  // Enterprise / Performance / LowResource
    pub persistence: bool,
    pub vector_search: bool,
    pub iql_queries: bool,
    pub read_only: bool,
}
```

### `OperationalMetrics` (37 fields)

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

### `SearchExplanationHit`

```rust
pub struct SearchExplanationHit {
    pub identity: String,          // namespace/key
    pub score: f32,
    pub snippet: Option<String>,   // highlighted text
    pub matched_tokens: Vec<String>,
    pub matched_phrases: Vec<String>,
    pub bm25_terms: Vec<Bm25TermContribution>,  // per-term TF/DF/contribution
    pub rrf_text_rank: Option<usize>,
    pub rrf_vector_rank: Option<usize>,
}
```

### `Bm25TermContribution`

```rust
pub struct Bm25TermContribution {
    pub token: String,
    pub tf: u32,
    pub df: u64,
    pub doc_len: u32,
    pub contribution: f32,
}
```

## Report Types

### `IndexRebuildReport`

```rust
pub struct IndexRebuildReport {
    pub scanned_nodes: u64,
    pub indexed_vectors: u64,
    pub skipped_tombstones: u64,
    pub duration_ms: u64,
    pub derived_rebuild_ms: u64,
    pub index_path: String,
    pub success: bool,
}
```

### `ExportReport`

```rust
pub struct ExportReport {
    pub records_exported: u64,
    pub namespaces: Vec<String>,
    pub path: String,
    pub duration_ms: u64,
}
```

### `ImportReport`

```rust
pub struct ImportReport {
    pub inserted: u64,
    pub updated: u64,
    pub skipped: u64,
    pub errors: u64,
    pub duration_ms: u64,
}
```

### `TextIndexAuditReport` (25 fields)

Key fields: `schema_version`, `tokenizer`, `namespaces_audited`, `records_scanned`, `expected_entries`, `actual_entries`, `missing_entries`, `unexpected_entries`, `value_mismatches`, `position_errors`, `tf_errors`, `df_errors`, `doc_len_errors`, `logical_corruptions`, `state_valid`, `passed`, `status`.

### `TextIndexRepairReport`

```rust
pub struct TextIndexRepairReport {
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

All fallible methods return `Result<T, Error>`. `Error` is a
`#[non_exhaustive]` enum defined in [`src/error.rs`](../../src/error.rs).

> **Canonical reference:** [`docs/api/ERROR_HANDLING.md`](ERROR_HANDLING.md)
> documents the contract that Python, TypeScript, MCP, and HTTP bindings
> normalize to: 10 codes (`VANTADB_VALIDATION_ERROR`, `VANTADB_NOT_FOUND`,
> `VANTADB_TIMEOUT`, `VANTADB_BUSY`, `VANTADB_RESOURCE_LIMIT`,
> `VANTADB_CORRUPT`, `VANTADB_INVALID_ARGUMENT`, `VANTADB_IO_ERROR`,
> `VANTADB_WASM_ERROR`, `VANTADB_CLOSED`), `is_retriable()` matrix,
> `recovery_hint()` guide. Read that first.

### Stable error codes

`Error::code() -> &'static str` (ERR-CORE-01) returns one of the ten
canonical `VANTADB_*` codes. Branch on `code()` — never on `Display` text,
which can change without a major bump. The HTTP error envelopes in
`docs/api/HTTP_API.md` carry the same value under the `"code"` JSON field.

### Variants

`Error` covers:

- `Error::NodeNotFound(u128)` — node ID not found
- `Error::DuplicateNode(u128)` — duplicate node ID on insert
- `Error::DimensionMismatch { expected: usize, got: usize }` — vector dimension mismatch
- `Error::Wal(ChainedError)` — WAL operation failure
- `Error::WALVersionMismatch { expected: u32, found: u32, hint: String }` — incompatible WAL version
- `Error::Serialization(#[source] Box<dyn StdError + Send + Sync>)` — postcard/serde failures
- `Error::Io(std::io::Error)` — filesystem errors
- `Error::IncompatibleFormat { expected_magic, expected_version, found_magic, found_version, hint }` — incompatible binary format
- `Error::NotInitialized` — engine not open
- `Error::ResourceLimit(String)` — resource limit exceeded (backpressure)
- `Error::VectorLenOverflow { id, len, limit }` — serialized vector length exceeds the u32 on-disk header field (ERR-CORE-01)
- `Error::EdgeCountOverflow { id, count, limit }` — edge count exceeds the u16 `DiskNodeHeader` field (ERR-CORE-01, hardens ERR-029)
- `Error::NodeIdCollision(u128)` — two nodes have colliding IDs
- `Error::CycleDetected` — cycle detected in graph operation
- `Error::Validation { field, reason }` — input validation failed
- `Error::Timeout { operation, duration_ms }` — operation exceeded time budget
- `Error::UnsupportedOperation { operation, detail }` — unsupported operation
- `Error::ExecutionConflict { resource, detail }` — concurrent modification conflict
- `Error::Iql(ChainedError)` — IQL query processing error
- `Error::IqlParse { msg, line, col }` — IQL parse error at a specific line/col
- `Error::Cli(ChainedError)` — CLI command processing error
- `Error::Search(ChainedError)` — search execution error
- `Error::Runtime(ChainedError)` — unexpected runtime error
- `Error::Restore(ChainedError)` — restore operation error
- `Error::Backup(ChainedError)` — backup operation error
- `Error::Backend(ChainedError)` — storage backend error
- `Error::InvalidInput(String)` — invalid input provided
- `Error::Schema(String)` — schema-related error
- `Error::DatabaseBusy(String)` — database locked by another process
- `Error::NoVectorForKey(String)` — a record exists but does not carry a vector, so vector-based operations (e.g. `similar_to_key`) cannot proceed
- `Error::Generic(ChainedError)` — generic catch-all error

### `is_retriable()` — retry classification

Defined in `src/error.rs:269`. Returns `true` for variants where retrying
after backoff is the correct response:

```rust
impl Error {
    pub fn is_retriable(&self) -> bool {
        matches!(
            self,
            Error::DatabaseBusy(_)
                | Error::Timeout { .. }
                | Error::ResourceLimit(_)
                | Error::Backend(_)
                | Error::Wal(_)
        )
    }
}
```

Use this when implementing retry policies at a binding boundary. Bindings
that surface `.retriable` (Python) or `code` class (TS) derive this from the
same matrix. Do **not** retry the other 25 variants — they indicate a caller
fix is required.

### `recovery_hint()` — actionable guidance

Defined in `src/error.rs:281`. Returns `Option<&'static str>` with
human-readable remediation:

```rust
impl Error {
    pub fn recovery_hint(&self) -> Option<&'static str> {
        match self {
            Error::DatabaseBusy(_) => Some("Wait for the lock to be released and retry"),
            Error::Timeout { .. } => Some("Increase the timeout or reduce system load"),
            Error::ResourceLimit(_) => Some("Reduce memory pressure or increase configured limits"),
            Error::IncompatibleFormat { .. } => Some("Delete the WAL or run dump/restore to migrate formats"),
            Error::Schema(_) => Some("Reinitialize the database or restore from backup"),
            Error::WALVersionMismatch { .. } => Some("The WAL was written by a different version of VantaDB"),
            Error::Restore(_) => Some("Check that the backup file exists and is readable"),
            Error::Backup(_) => Some("Ensure the backup directory is writable and has free space"),
            Error::NodeNotFound(_) => Some("The node may have been deleted or never existed"),
            Error::NotFound { .. } => Some("Verify that the namespace or identifier is spelled correctly"),
            _ => None,
        }
    }
}
```

Bindings should surface this in error `.hint` / `details.hint` so end-users
see actionable guidance instead of a generic error toast.

### `code()` — pending (`ERR-CORE-01`)

> **Upcoming:** Task `ERR-CORE-01` will add `pub fn code(&self) -> &'static str`
> returning the canonical 10 codes with the `VANTADB_` prefix
> (`VANTADB_VALIDATION_ERROR`, `VANTADB_NOT_FOUND`, …). Until that merges,
> use the TypeScript `ERROR_CODES` (see [`docs/api/TS_SDK.md`](TS_SDK.md)) as
> the contract surface and `is_retriable()` / `recovery_hint()` for
> classification.

### Example: idiomatic retry

```rust
use std::time::Duration;

fn with_retry<T, F: FnMut() -> Result<T, Error>>(mut op: F) -> Result<T, Error> {
    for attempt in 0..5u32 {
        match op() {
            Ok(v) => return Ok(v),
            Err(e) if e.is_retriable() && attempt < 4 => {
                std::thread::sleep(Duration::from_millis(100 * 2u64.pow(attempt)));
            }
            Err(e) => return Err(e),
        }
    }
    unreachable!()
}
```

For the full retry guidance and the `code()` table, see
[`docs/api/ERROR_HANDLING.md`](ERROR_HANDLING.md).

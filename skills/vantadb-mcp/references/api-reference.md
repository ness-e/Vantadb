# VantaDB API Reference

> Verified against the real SDK boundary: `src/sdk/types.rs`, `src/sdk/api.rs`, `src/sdk/builder.rs`, `src/index/graph.rs`, `src/error.rs`. Only symbols that exist in the code are documented here.

## MCP Tools (79)

> **This is the single source of truth for the VantaDB MCP contract.**
> Verified against `vantadb-mcp/src/`: exactly **79 tools** = 49 core
> (`handlers/tools.rs` `base_tools` — 49) + 6 `skill_*` (`skills.rs`) + 8 `code_*`
> (`code.rs`) + 6 `wiki_*` (`wiki.rs`) + 1 `context_assemble`
> (`context.rs`) + 3 `scene_*` (`scenes.rs`) + 6 `thread_*` (`threads.rs`). All seven sets are announced together
> in `tools/list` via extend (`handlers/tools.rs`).
> Last synced against code: 2026-09-02 — 79 tools = 49 core + 6 skill_* + 8 code_* + 6 wiki_* + 1 context_assemble + 3 scene_* + 6 thread_* (verified `vantadb-mcp/src/handlers/tools.rs` base_tools 49).

### Core — Memory / Search / Collections / Graph / IQL / GDS / Recovery (49)

| Tool | Purpose | Main params |
|------|---------|-------------|
| `memory_put` | Insert or update a memory record | `namespace`, `key`, `payload` (req); `vector`, `sparse_vector` (dim-id → weight object), `metadata`, `expires_at_ms` |
| `memory_put_batch` (MCP-19) | Store multiple records in one batch call; all-or-nothing validation, duplicate keys are upserts | `inputs` (array of `memory_put`-shaped objects, req) |
| `memory_get` | Retrieve a record by namespace + key | `namespace`, `key` |
| `memory_delete` | Delete a record | `namespace`, `key` |
| `memory_delete_by_filter` (MCP-18) | Batch-delete all records matching metadata filters; ≥1 filter item required | `namespace`, `filters` (same shape as `memory_list`, req); returns `{deleted_count}` |
| `memory_list` | List records with pagination + filters | `namespace`; `limit` (default 100), `cursor` (numeric offset), `filters` |
| `memory_list_namespaces` | List all namespaces | none |
| `memory_versions` (MOD-10) | List every retained version of a record, ascending v1..vN (snapshots drop supersession fields) | `namespace`, `key`; returns array of memory records (empty `[]` when missing) |
| `memory_supersede` (MOD-10) | Mark a record as superseded by another (durable soft-dead) | `namespace`, `old_key`, `new_key`; errors on missing key / same-key / already superseded; returns `{superseded: true}` |
| `query_iql` | Execute an IQL statement (reads + node mutations; LISP NOT supported) | `query` |
| `search_memory` | Hybrid vector + text search with filters/profile/explain | `namespace`; `query_vector`, `text_query`, `top_k` (10), `distance_metric` (cosine \| euclidean, per-request), `explain`, `filters`, `search_profile` `{mode, rrf_k, candidate_k}` |
| `search_semantic` | Raw HNSW vector search | `vector`, `k` (required in schema, defaults 5 if omitted); returns real distances (`1 − cosine_similarity`) ascending |
| `get_node_neighbors` | Inspect node's outgoing edges (alive targets only) | `node_id` (u128 decimal string) |
| `graph_page_rank` (MCP-21) | PageRank over the subgraph reachable from the roots | `roots` (array of u128 decimal strings, req); `max_iterations` (100), `damping_factor` (0.85), `tolerance` (1e-6); returns `{scores: {"<id>": rank}}` |
| `graph_degree_centrality` (MCP-21) | In/out degree counts for every reachable node | `roots` (req); returns `{degrees: {"<id>": {in, out}}}` |
| `graph_traverse` (MCP-22) | Multi-hop BFS/DFS traversal (plain or label/time-filtered) | `start` (array, req), `mode` (`bfs`\|`dfs`, req), `max_depth` (req); `direction` (`forward`\|`reverse`\|`both`, default forward), `filter` `{labels: [u32], time_range: [from_ms, to_ms]}`; returns `{visited: [...], count}` |
| `graph_topological_sort` (MCP-22) | Topological order of the subgraph; errors on cycles | `roots` (req); returns `{order: [...]}` |
| `graph_is_dag` (MCP-22) | Whether the subgraph reachable from roots is acyclic | `roots` (req); returns `{is_dag: bool}` |
| `remove_edge` (MOD-10) | Remove all edges between two nodes with the given label (both directions) | `source_id`, `target_id` (u128 decimal strings, req), `label` (req); returns `{removed: true}` |
| `inject_context` | Inject context into a thread for consolidation | `content`, `thread_id` (number) |
| `read_axioms` | Read active Iron Axioms | none |
| `collection_stats` | Stats for a namespace/collection | `namespace` |
| `collection_list` | List collections with metadata | none |
| `collection_delete` | Delete an entire namespace/collection | `namespace`, `confirm` (must be `"yes"`) |
| `rehydrate` | Recover shadow-archived nodes of a summary node | `summary_id` (u128 string) |
| `export` | Backup: export records as JSONL text (single namespace or all) | `namespace` (optional; omit = all); output capped 10 MB/call |
| `import` | Restore: import a JSONL string produced by `export` | `content` (JSONL, max 10 MB/call); returns `VantaImportReport` |
| `bulk_import_file` | Bulk-import a binary `.vdbdump` file from the host filesystem | `path`; returns `BulkImportReport`; raw nodes not addressable via memory_get |
| `bulk_import_stream` | Bulk-import inline NDJSON or raw `.vdbdump` payload | `content` (max 10 MB/call); returns `BulkImportReport` |
| `purge_expired` | Physically delete records past their TTL; returns purged count | none |
| `compact_wal` | Flush + archive current WAL, start fresh one | none |
| `flush` | Manual durability checkpoint (WAL + mmap flush) | none |
| `compact_layout` | Compact vector store in BFS order; returns bytes reclaimed | none |
| `vacuum` (MOD-10) | Purge tombstoned nodes from the HNSW index | none; returns `{scanned_nodes, removed_nodes, reclaimed_bytes, duration_ms, success}` |
| `rebuild_index` (MCP-20) | Rebuild HNSW + derived indexes + text index from scratch (recovery primitive) | none; returns `VantaIndexRebuildReport` `{scanned_nodes, indexed_vectors, skipped_tombstones, duration_ms, derived_rebuild_ms, index_path, success}` |
| `audit_text_index` (MCP-20) | Read-only integrity audit of the derived text index vs canonical records | `namespace` (optional), `deep` (bool, default false — adds value-level checks); returns audit report; `passed=true` + `status="ok"` mean no drift |
| `repair_text_index` (MCP-20) | Repair the text index by rebuilding it from canonical storage | none; returns `VantaTextIndexRepairReport` `{record_count, posting_entries, doc_stats_entries, term_stats_entries, namespace_stats_entries, duration_ms, success}` |
| `capabilities` (MCP-26) | Introspect engine features | none; returns `{runtime_profile, persistence, vector_search, iql_queries, read_only}` |
| `generate_snippet` (MCP-26) | Extract a snippet from a payload around matched query terms | `payload`, `text_query` (req); `with_highlighting` (bool, default false); returns `{snippet}` or `{snippet: null}` when the query yields no terms |
| `list_snapshots` (MCP-26) | List physical snapshot names under `<data_dir>/snapshots` | none; returns `{snapshots: [...]}` |

Detailed behavior notes (response envelope, error channels, IQL syntax, edge cases F4–F11): see [`SKILL.md`](../SKILL.md).

### Review-agent Skills (6 × `skill_*`)

Versioned agent skills over the core `SkillStore` (MEM-07).
**Precondition:** every call takes `owner_agent` as caller identity — skills owned by another agent respond exactly like missing ones (no existence leak). Writes require `expected_version` (optimistic lock).

| Tool | Purpose | Main params |
|------|---------|-------------|
| `skill_list` | List an agent's skills | `owner_agent` (req); `name_prefix`, `limit` (50), `offset` |
| `skill_view` | Read head or specific version incl. resource files | `skill_id`, `owner_agent`; `version` |
| `skill_create` | Create a skill (v1); idempotent on owner+name+content | `name`, `owner_agent`, `content`; `description`, `metadata`, `ttl_secs` |
| `skill_update` | Replace head content, appending a version | `skill_id`, `owner_agent`, `expected_version`, `content`; `description` |
| `skill_patch` | Substring replacement in skill content (TDAM-compatible) | `skill_id`, `owner_agent`, `expected_version`, `old_string`, `new_string`; `replace_all` (required when >1 occurrence) |
| `skill_files_write` | Write a resource file into the skill manifest (5 MB/file, 50 MB/skill) | `skill_id`, `owner_agent`, `expected_version`, `path` (relative only), `content`; `encoding` (utf-8 \| base64), `mime_type`, `is_executable` |

### Code Intelligence (8 × `code_*`)

Read-only wrappers over the **built-in** GraphRAG pipeline + graph traversal primitives (MEM-32; no external codegraph dependency).
**Precondition:** the target database must contain ingested graph nodes/edges first (via `query_iql` `INSERT NODE`, the SDK node API, or an ingestion pipeline). All are query-only; domain errors come back as `isError` content.

| Tool | Purpose | Main params |
|------|---------|-------------|
| `code_search` | GraphRAG search (seed → expand → retrieve → context) | `namespace`, `query` |
| `code_explore` | Node + direct neighborhood split into callers/callees | `node_id` |
| `code_callers` | Incoming edges (reverse traversal minus root) | `node_id` |
| `code_callees` | Outgoing edges (forward traversal minus root) | `node_id` |
| `code_impact` | Reachable subgraph within max_depth hops | `node_id`; `max_depth` (3, cap 10), `direction` (Forward \| Reverse \| Both) |
| `code_node` | Fetch one node as a full record | `node_id` |
| `code_status` | Operational-metrics snapshot of the backing engine | none |
| `code_files` | **Not supported** — built-in graphrag has no file-per-node concept; always errors by design | none |

### Wiki Knowledge (6 × `wiki_*`)

Query-only wrappers over the core `WikiStore` (MEM-33) plus async ingest (MEM-52). BM25-style ranking (k1=1.5, b=0.75, title terms ×5); `[[wikilink]]` edges extracted from page content.
**Precondition:** the wiki lifecycle must be in `ready` state (`pending → processing → ready | failed`); while not ready every tool refuses with the current state surfaced so the caller can retry.

| Tool | Purpose | Main params |
|------|---------|-------------|
| `wiki_search` | Full-text search over pages of a ready wiki | `namespace`, `slug`, `query`; `top_k` (10) |
| `wiki_read` | Read one page by canonical path (`locked:true` visible on managed pages) | `namespace`, `slug`, `path` |
| `wiki_list` | List every page ordered by canonical path | `namespace`, `slug` |
| `wiki_graph` | BFS multi-hop over wikilink edges (cap 200 visited nodes) | `namespace`, `slug`, `root_path`; `max_hops` (2, cap 10) |
| `wiki_ingest` | Start an async wiki build from local markdown; returns `run_id` immediately | `namespace`, `slug`, `root` (abs path, req) |
| `wiki_ingest_status` | Poll lifecycle state + progress of a build by run_id | `run_id` (req) |

### Context Engine (1 × `context_assemble`, MCP-31)

Read-only MCP exposure of the vanta-memory context engine (`assemble_with_recall` + the session auto-recall hook desktop consumes over IPC). Compressor internals are NOT exposed — assembly only.

**Precondition:** none for history compaction; session recall injects only when prior memory capture populated the session (L1 records / persona / scenes). An unknown session is not an error — assembly still runs on the provided history.

| Tool | Purpose | Main params |
|------|---------|-------------|
| `context_assemble` | Assemble a context window under a token budget: compacts the chat history and injects recall blocks (relevant L1 memories, user persona, scene navigation) | `session_key`, `token_budget` (req, > 0); `query`, `messages` (`{role, content, id?}`, optional); returns `{messages, report{mode,msgs_conserved,msgs_before,tokens_before,tokens_after}, mmd_injected, recall_injected}`. Note: when the protected final messages alone exceed the budget, output intentionally exceeds it (engine cursor guarantee) |

### Scenes API (3 × `scene_*`, MCP-30)

Read-only MCP exposure of the vanta-memory gateway scene handlers
(`vanta_memory::gateway` pure functions over `&VantaEmbedded`) — structured
scene navigation for external agents. Domain errors (unknown session, missing
scene) surface as error-content messages, never protocol errors. `scene_query`
ranks by keyword overlap only (no embedding hook in MCP).

| Tool | Purpose | Main params |
|------|---------|-------------|
| `scene_read` | Read one live scene block by name; soft-deleted and missing scenes are indistinguishable ("not found") | `session_key`, `scene_name` (req); returns `{scene:{scene_name, meta{created,updated,summary,heat}, content}}` |
| `scene_list` | List the session's scene index, heat descending, soft-deleted excluded | `session_key` (req); returns `{scenes:[{filename,summary,heat,created,updated}]}` where `filename` is the id for `scene_read` |
| `scene_query` | Keyword search over live scene blocks (term overlap vs summary+content, ties by heat) | `session_key`, `keyword` (req); `top_k` (optional, default 5); returns `{hits:[{scene_name,summary,heat,updated,score}]}` — load hits via `scene_read` |

## Python SDK

### Threads API (6 × `thread_*`, MCP-32)

Conversation history CRUD over the agentic thread store (`src/agentic/thread.rs` via the SDK builder). Thread ids are `u128` transported as JSON strings. Domain errors surface as error-content; malformed params are JSON-RPC `-32602`.

| Tool | Description |
|------|-------------|
| `thread_create` | `{title, ttl_secs?}` → `{thread_id}` (u128 string) |
| `thread_send` | `{thread_id, role, content}` → `{ok:true}` |
| `thread_get` | `{thread_id}` → full thread `{thread_id,title,messages:[{role,content,timestamp}],created_at,updated_at}`; missing → error_content "not found" |
| `thread_list` | `{limit?, offset?}` → `{threads:[...],count}` |
| `thread_delete` | `{thread_id}` → `{deleted:true}` — permanent, no undo |
| `thread_purge_expired` | `{}` → `{purged:n}` — removes TTL-expired threads |
### VantaDB Class

```python
from vantadb_py import VantaDB, AsyncVantaDB

db = VantaDB("./my_brain")        # persistent database directory
mem = VantaDB(":memory:")         # in-memory database
async_db = AsyncVantaDB("./my_brain")  # asyncio wrapper (thread-pool backed)
```

#### Methods

**put(namespace, key, payload, metadata=None, vector=None, ttl_ms=None)**
- Insert or update a memory record
- Returns: Record with created_at_ms, updated_at_ms

**get_memory(namespace, key)**
- Retrieve a memory record
- Returns: Record or None

**delete_memory(namespace, key)**
- Delete a memory record
- Returns: Boolean success status

**list_memory(namespace, limit=100, cursor=None, filters=None)**
- List records in namespace
- `filters` accepts BOTH formats (AUD-048 — unified with the CLI channel): flat `{"field": value}` (implicit `$eq`) **or** operator objects `{"field": {"$gt": value}}` (`$eq`, `$neq`, `$gt`, `$gte`, `$lt`, `$lte`)
- Returns: List of records

**list_namespaces()**
- List all namespaces
- Returns: List of namespace names

**search_memory(namespace, query_vector=None, text_query=None, top_k=10, filters=None, distance_metric=None, explain=False)**
- Hybrid vector + text search
- `distance_metric` (`"cosine"` | `"euclidean"`) is resolved **per request**; it changes the ranking and reported scores of that call. There is no global/server-side distance metric setting — each request selects its own metric. When omitted, `"cosine"` is used.
- `explain=True` appends a per-hit `explanation` object to each hit: `{identity, score, snippet, matched_tokens, matched_phrases, bm25_terms, rrf_text_rank, rrf_vector_rank}` (the per-source ranks behind the fused score).
- `filters` accepts flat values `{"field": value}` **or** explicit equality `{"field": {"$eq": value}}` (identical equality semantics). Range operators are NOT supported here — the search request has no operator slot; use `list_memory` for those (AUD-048).
- Returns: **JSON array** of hits — `[{record, score, explanation?}, ...]`. There is **no top-level `route` or `fusion_report`** on this method; those belong to `explain_memory_search()`, whose `fusion_report` is currently always `null`. Do not assert them on `search_memory` output.

**search(vector, top_k=10)**
- Pure HNSW vector search
- Returns: List of neighbors with distances

**query(iql_query)**
- Execute an IQL statement

**flush()**
- Flush data to disk

**close()**
- Close database connection

Other methods available on the class include `put_batch`, `export_namespace`, `import_file`, `operational_metrics`, `generate_snippet`, `explain_memory_search`, `capabilities`, `add_edge`, graph traversals (`graph_bfs`, `graph_dfs`, `graph_page_rank`), and more — see `vantadb-python/vantadb_py/__init__.py`.

## Rust SDK

### VantaEmbedded

```rust
use vantadb::VantaEmbedded;

let embedded = VantaEmbedded::open("./vantadb")?;  // default config
let embedded = VantaEmbedded::open_with_config(config)?;  // custom VantaConfig
let embedded = VantaEmbedded::from_engine(storage);  // wrap an existing StorageEngine
```

#### Methods (memory API)

**put(input: VantaMemoryInput) -> Result<VantaMemoryRecord>**
- Insert or update memory record

**put_batch(inputs: Vec<VantaMemoryInput>) -> Result<Vec<VantaMemoryRecord>>**
- Insert or update multiple memory records

**get(namespace: &str, key: &str) -> Result<Option<VantaMemoryRecord>>**
- Retrieve memory record

**delete(namespace: &str, key: &str) -> Result<bool>**
- Delete memory record

**list(namespace: &str, options: VantaMemoryListOptions) -> Result<VantaMemoryListPage>**
- List records with pagination

**list_namespaces() -> Result<Vec<String>>**
- List all namespaces

**search(request: VantaMemorySearchRequest) -> Result<Vec<VantaMemorySearchHit>>**
- Hybrid search (dense vector + sparse + BM25 text + metadata filters)

**search_vector(vector: &[f32], top_k: usize) -> Result<Vec<VantaSearchHit>>**
- Pure vector search against the HNSW index
- The core SDK reports the raw HNSW score in `VantaSearchHit.distance` (cosine **similarity**: identical → 1.0, orthogonal → 0.0; or negated euclidean distance). The MCP `search_semantic` tool converts this at the serialization boundary so its `distance` field is a real distance (`1 − similarity` for cosine) — see `SKILL.md`. Consumers of the core SDK directly must interpret the score per metric.

**count(namespace: &str, filter: Option<VantaMemoryFilter>) -> Result<u64>**
- Count records in a namespace

**delete_by_filter(namespace: &str, filter: VantaMemoryFilter) -> Result<u64>**
- Delete records matching a filter

#### Methods (node/graph API)

- `insert_node(input: VantaNodeInput)`, `get_node(id: u128)`, `delete_node(id: u128, reason: &str)`
- `add_edge(source_id, target_id, label, weight)`, `remove_edge(source_id, target_id, label)`
- `query(query: &str) -> Result<VantaQueryResult>` — execute an IQL statement

#### Methods (maintenance/metrics)

- `operational_metrics() -> VantaOperationalMetrics`
- `capabilities() -> VantaCapabilities`
- `flush()`, `vacuum()`, `compact_wal()`, `compact_layout()`, `purge_expired()`
- `rebuild_index()`, `reindex_hnsw_from_text(...)`
- `recover_archived_nodes(summary_id: u128)` — recover shadow-archived nodes (used by the MCP `rehydrate` tool)
- `bulk_import_file(path)`, `similar_to_key(...)`

## Data Structures

### VantaMemoryInput

```rust
pub struct VantaMemoryInput {
    pub namespace: String,
    pub key: String,
    pub payload: String,
    pub metadata: VantaMemoryMetadata,
    pub vector: Option<Vec<f32>>,
    pub sparse_vector: Option<SparseVector>,
    pub ttl_ms: Option<u64>,
}
```

### VantaMemoryRecord

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
    pub sparse_vector: Option<SparseVector>,
    pub expires_at_ms: Option<u64>,
}
```

### VantaMemoryMetadata

Type alias for the stable relational fields map:

```rust
pub type VantaFields = BTreeMap<String, VantaValue>;
pub type VantaMemoryMetadata = VantaFields;
```

### VantaValue

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

#### Wire format over MCP (F3)

`VantaValue` serializes asymmetrically over MCP:

- **Input (JSON arguments to tools):** plain JSON values — `{"priority": 2}`, `{"type": "preference"}`. The MCP handler parses them into `VantaValue` variants.
- **Output (payload inside `content[0].text`):** serde-tagged JSON — `{"priority": {"Int": 2}}`, `{"type": {"String": "preference"}}`, `{"active": {"Bool": true}}`.

Example — `memory_put` with `metadata: {"type": "preference", "priority": 2}` returns a record whose `metadata` serializes as:

```json
{
  "type": { "String": "preference" },
  "priority": { "Int": 2 }
}
```

Consumers should flatten single-key variant objects (`{"Int": 2}` → `2`) when reading metadata from tool responses. Both forms may be observed depending on the serialization path.

### VantaMemoryFilter

Advanced filter list combined with AND logic:

```rust
pub struct VantaMemoryFilterItem {
    pub field: String,
    pub op: VantaFilterOp,  // Eq, Neq, Gt, Lt, Gte, Lte
    pub value: VantaValue,
}
pub type VantaMemoryFilter = Vec<VantaMemoryFilterItem>;
```

### VantaMemoryListOptions

```rust
pub struct VantaMemoryListOptions {
    #[deprecated(note = "Use filter_ops instead")]
    pub filters: VantaMemoryMetadata,
    pub filter_ops: Option<VantaMemoryFilter>,
    pub limit: usize,
    pub cursor: Option<usize>,
}
```

### VantaMemoryListPage

```rust
pub struct VantaMemoryListPage {
    pub records: Vec<VantaMemoryRecord>,
    pub next_cursor: Option<usize>,
}
```

## Configuration

### VantaConfig

```rust
pub struct VantaConfig {
    pub storage_path: String,
    pub host: String,
    pub port: u16,
    pub llm_url: String,
    pub llm_model: String,
    pub llm_summarize_model: String,
    pub memory_limit: Option<u64>,
    pub read_only: bool,
    pub force_mmap: bool,
    pub mmap_hnsw: bool,
    pub prefetch_mode: PrefetchMode,
    pub rss_threshold: f64,
    pub backend_kind: BackendKind,
    pub max_blocking_threads: usize,
    pub max_connections: usize,
    pub sync_mode: SyncMode,
    pub api_key: Option<String>,
    pub require_auth: bool,
    pub rate_limit_rpm: u32,
    pub log_format: LogFormat,
    // ... plus WAL, TLS, encryption, audit-log, export, and hot-reload fields
}
```

`VantaConfig` has no `hnsw_config` field. HNSW parameters are engine-internal (see `HnswConfig` below) and are not part of `VantaConfig`; memory/search tuning knobs are configured via environment variables (see `configuration.md`).

### HnswConfig

```rust
pub struct HnswConfig {
    pub m: usize,
    pub m_max0: usize,
    pub ef_construction: usize,
    pub ef_search: usize,
    pub ml: f64,
    pub distance_metric: DistanceMetric,
    pub flat_threshold: Option<usize>,
    pub index_type: IndexType,
    pub auto_tune: bool,
}
```

Defaults: `m=32`, `m_max0=64`, `ef_construction=100`, `ef_search=100`, `ml=1/ln(32)`, `distance_metric=Cosine`, `flat_threshold=Some(10000)`, `index_type=Hnsw`, `auto_tune=false`.

## Error Handling

### VantaError

`VantaError` is a large non-exhaustive enum. Notable variants include:

```rust
pub enum VantaError {
    NodeNotFound(u128),
    DuplicateNode(u128),
    DimensionMismatch { expected: usize, got: usize },
    WalError(ChainedError),
    SerializationError(Box<dyn StdError + Send + Sync>),
    IoError(std::io::Error),
    ResourceLimit(String),
    NodeIdCollision(u128),
    IqlParseError { /* ... */ },
    NotFound { /* ... */ },
    ValidationError { /* ... */ },
    Timeout { /* ... */ },
    UnsupportedOperation { /* ... */ },
    ExecutionConflict { /* ... */ },
    IqlError(ChainedError),
    CliError(ChainedError),
    SearchError(ChainedError),
    RuntimeError(ChainedError),
    RestoreError(ChainedError),
    BackupError(ChainedError),
    Generic(ChainedError),
    BackendError(ChainedError),
    InvalidInput(String),
    SchemaError(String),
    DatabaseBusy(String),
    NoVectorForKey(String),
}
```

#### DimensionMismatch over MCP

`VantaError::DimensionMismatch` is **not** returned as a JSON-RPC error. The MCP
tools `search_memory` (when `query_vector` is provided) and `search_semantic`
validate the query vector dimension against the live HNSW index dimension and
deliver the failure as an **isError content result**:

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "isError": true,
    "content": [
      { "type": "text", "text": "Vector dimension mismatch: expected 4, got 3" }
    ]
  }
}
```

The message format is `Vector dimension mismatch: expected {expected}, got {got}`.
Clients that receive `isError: true` should surface `content[0].text` to the user;
do not expect a JSON-RPC `error` object for this case. On an empty index (no
vectors stored yet) the dimension check is skipped and the search returns an
empty result set.

## Performance Considerations

- Use namespace isolation to limit search scope
- Configure appropriate HNSW parameters for your dataset
- Implement periodic cleanup of old records (TTL, `purge_expired`)
- Use metadata filters to reduce search space
- Batch operations when possible (`put_batch`)

## Best Practices

1. **Namespace Strategy**
   - Use descriptive namespace names
   - Separate concerns by namespace
   - Use hierarchical naming (e.g., `agent/session-001`)

2. **Metadata Design**
   - Use consistent metadata keys
   - Include timestamps for temporal queries
   - Use type hints for filtering

3. **Vector Search**
   - Normalize vectors before storage
   - Use appropriate dimensionality
   - Tune HNSW parameters for your use case

4. **Memory Management**
   - Configure appropriate memory limits
   - Implement cleanup strategies
   - Monitor operational metrics

## More Information

- Full documentation: See ../../docs/
- API documentation: See ../../docs/api/
- Examples: See ../../examples/python/
- MCP Protocol: See mcp-protocol.md
- Configuration: See configuration.md
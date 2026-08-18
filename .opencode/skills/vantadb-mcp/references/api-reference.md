# VantaDB API Reference

> Verified against the real SDK boundary: `src/sdk/types.rs`, `src/sdk/api.rs`, `src/sdk/builder.rs`, `src/index/graph.rs`, `src/error.rs`. Only symbols that exist in the code are documented here.

## Python SDK

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
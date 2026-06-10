# VantaDB API Reference

## Python SDK

### VantaDB Class

```python
import vantadb_py as vantadb

db = vantadb.VantaDB(
    path: str,
    memory_limit_bytes: int = 512_000_000
)
```

#### Methods

**put(namespace, key, payload, metadata=None, vector=None)**
- Insert or update a memory record
- Returns: Record with created_at_ms, updated_at_ms

**get(namespace, key)**
- Retrieve a memory record
- Returns: Record or None

**delete(namespace, key)**
- Delete a memory record
- Returns: Boolean success status

**list(namespace, options)**
- List records in namespace
- Options: {"limit": int, "filters": dict, "cursor": int}
- Returns: List of records

**list_namespaces()**
- List all namespaces
- Returns: List of namespace names

**search_memory(namespace, query_vector=None, text_query=None, top_k=10, filters={})**
- Hybrid vector + text search
- Returns: List of hits with scores

**search_semantic(vector, k)**
- Pure HNSW vector search
- Returns: List of neighbors with distances

**flush()**
- Flush data to disk

**close()**
- Close database connection

## Rust SDK

### VantaEmbedded

```rust
use vantadb::VantaEmbedded;

let embedded = VantaEmbedded::open("./vantadb")?;
```

#### Methods

**put(input: VantaMemoryInput)**
- Insert or update memory record

**get(namespace: &str, key: &str)**
- Retrieve memory record

**delete(namespace: &str, key: &str)**
- Delete memory record

**list(namespace: &str, options: VantaMemoryListOptions)**
- List records with pagination

**list_namespaces()**
- List all namespaces

**search_memory(namespace, query_vector, text_query, top_k, filters)**
- Hybrid search

**search_semantic(vector, k)**
- Pure vector search

**operational_metrics()**
- Get operational metrics

**generate_snippet(payload, text_query, with_highlighting)**
- Generate text snippet with optional highlighting

## Data Structures

### VantaMemoryInput

```rust
pub struct VantaMemoryInput {
    pub key: String,
    pub namespace: String,
    pub payload: String,
    pub vector: Option<Vec<f32>>,
    pub metadata: VantaMemoryMetadata,
}
```

### VantaMemoryRecord

```rust
pub struct VantaMemoryRecord {
    pub key: String,
    pub namespace: String,
    pub payload: String,
    pub vector: Option<Vec<f32>>,
    pub metadata: VantaMemoryMetadata,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
}
```

### VantaMemoryMetadata

```rust
pub struct VantaMemoryMetadata {
    // HashMap<String, VantaValue>
}
```

### VantaValue

```rust
pub enum VantaValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}
```

### VantaMemoryListOptions

```rust
pub struct VantaMemoryListOptions {
    pub limit: usize,
    pub cursor: Option<usize>,
    pub filters: VantaMemoryMetadata,
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
    pub memory_limit_bytes: usize,
    pub max_blocking_threads: usize,
    pub read_only: bool,
    pub hnsw_config: HNSWConfig,
}
```

### HNSWConfig

```rust
pub struct HNSWConfig {
    pub dim: usize,
    pub m: usize,
    pub ef_construction: usize,
    pub ef_search: usize,
}
```

## Error Handling

### VantaError

```rust
pub enum VantaError {
    Io(String),
    Serialization(String),
    Validation(String),
    Execution(String),
    NotFound(String),
}
```

## Performance Considerations

- Use namespace isolation to limit search scope
- Configure appropriate HNSW parameters for your dataset
- Implement periodic cleanup of old records
- Use metadata filters to reduce search space
- Batch operations when possible

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

- Full documentation: https://docs.vantadb.io
- API docs: https://docs.vantadb.io/api
- Examples: https://github.com/your-org/vantadb/tree/main/examples

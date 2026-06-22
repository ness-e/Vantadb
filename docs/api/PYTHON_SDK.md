# Python SDK Documentation

## Installation

```bash
pip install vantadb-py
```

> **Note:** Requires Python 3.9+ and Rust toolchain (maturin) for building from source. Pre-built wheels are available for linux/amd64 and macOS (arm64/x86_64).

## Quick Start

```python
import vantadb_py as vantadb

db = vantadb.VantaDB("./vanta_data")

db.put(
    namespace="agent/main",
    key="memory-1",
    payload="The user prefers dark mode in all applications.",
)

results = db.search(
    namespace="agent/main",
    text="What display mode does the user prefer?",
)
print(results)
```

## API Reference

### Constructor

```python
vantadb.VantaDB(
    db_path: str,
    read_only: bool = False,
    memory_limit_bytes: Optional[int] = None,
    backend: Optional[str] = None,
) -> VantaDB
```

### Core Operations

#### `put()`
```python
db.put(
    namespace: str,
    key: str,
    payload: str,
    metadata: Optional[dict] = None,
    vector: Optional[List[float]] = None,
) -> dict
```
Insert or update a memory record. The `metadata` is a dict of scalar fields; `vector`, when provided, is a 384‑dimensional embedding.

Returns a dict like `{"key": "memory-1", "namespace": "agent/main", "created_at": "..."}`.

#### `search()`
```python
db.search(
    namespace: str,
    text: Optional[str] = None,
    query_vector: Optional[List[float]] = None,
    metadata: Optional[dict] = None,
    k: int = 10,
    include_vectors: bool = False,
    min_score: Optional[float] = None,
    mode: Optional[str] = None,
    extended_response: Optional[bool] = None,
) -> List[dict]
```
Hybrid (vector + lexical) search across the namespace. Returns a list of scored results.

- `namespace` (*str*) — required
- `text` — lexical query (tokenized, BM25-scored)
- `query_vector` — raw 384‑dim vector (bypasses LLM embedding call)
- `metadata` — filter by scalar fields
- `k` — max results (default 10)
- `include_vectors` — include embedding in response
- `min_score` — minimum similarity/BM25 threshold
- `mode` — `"lexical"` (for text-only search), `"vector"` (vector-only), `"hybrid"` (default)
- `extended_response` — include keyword extraction metadata

#### `get()`
```python
db.get(
    namespace: str,
    key: str,
) -> Optional[dict]
```
Fetch a single record by namespace + key. Returns `None` if not found.

#### `list()`
```python
db.list(
    namespace: str,
    limit: int = 1000,
    offset: int = 0,
    include_vectors: bool = False,
) -> dict
```
List records in a namespace. Returns `{"results": [...], "total": N, "has_more": bool}`.

#### `delete()`
```python
db.delete(
    namespace: str,
    key: str,
) -> bool
```
Delete a single record. Returns `True` if a record was removed.

#### `delete_namespace()`
```python
db.delete_namespace(
    namespace: str,
) -> bool
```
Delete an entire namespace and all its records. Returns `True` if the namespace existed.

#### `count()`
```python
db.count(
    namespace: str,
) -> int
```
Return the count of records in a namespace.

### Bulk Operations

#### `put_batch()`
```python
db.put_batch(
    namespace: str,
    records: List[dict],
    batch_size: int = 100,
) -> dict
```
Insert or update multiple records in a batch. Each dict requires `key` and `payload`, with optional `metadata` and `vector`.

Returns a dict with `successful` and `failed` record details.

#### `list_namespaces()`
```python
db.list_namespaces() -> List[str]
```
List all namespaces.

#### `get_namespace_info()`
```python
db.get_namespace_info(
    namespace: str,
) -> Optional[dict]
```
Return metadata about a namespace including record count and stats.

### Monitoring

#### `monitor_reset_window()`
```python
db.monitor_reset_window()
```
Reset the monitoring statistics window.

### Embeddings

#### `from_documents()`
```python
db.from_documents(
    documents: List[dict],
    namespace: str,
    key_field: str = "key",
    payload_field: str = "payload",
    batch_size: int = 100,
    verbose: bool = True,
) -> dict
```
Load a list of dict documents, chunk, embed, and store them. Each document must have a `key` and `payload` field by default.

### Update Operations

#### `update_payload()`
```python
db.update_payload(
    namespace: str,
    key: str,
    payload: str,
    metadata: Optional[dict] = None,
) -> Optional[dict]
```
Update the payload (and optionally metadata) of an existing record. Returns the updated record or `None` if not found.

#### `update_metadata()`
```python
db.update_metadata(
    namespace: str,
    key: str,
    metadata: dict,
) -> Optional[dict]
```
Merge metadata into an existing record. Returns the updated record or `None` if not found.

#### `update_importance()`
```python
db.update_importance(
    namespace: str,
    key: str,
    importance: float,
) -> Optional[dict]
```
Update the importance score of a record (used in consolidation/eviction).

#### `rename_key()`
```python
db.rename_key(
    namespace: str,
    old_key: str,
    new_key: str,
) -> Optional[dict]
```
Rename a record's key within the same namespace.

### Consolidation

#### `consolidate()`
```python
db.consolidate(
    namespace: str,
    strategy: str = "simple",
    min_score: float = 0.85,
    max_input_tokens: int = 2048,
    max_summary_tokens: int = 512,
    similarity_threshold: float = 0.85,
    batch_size: int = 10,
    verbose: bool = True,
) -> dict
```
Consolidate similar records by summarizing them. Returns a consolidation report.

- `strategy` — `"simple"` or `"clustering"`
- `min_score` — minimum similarity threshold for consolidation
- `max_input_tokens` — max tokens per summarization input
- `max_summary_tokens` — max tokens per output summary

#### `get_namespace_insights()`
```python
db.get_namespace_insights(
    namespace: str,
) -> Optional[dict]
```
Get insight data for a namespace, used for consolidation decisions.

### LLM / Knowledge

#### `query_ollama()`
```python
db.query_ollama(
    prompt: str,
    model: Optional[str] = None,
    system: Optional[str] = None,
) -> str
```
Query an Ollama LLM directly through VantaDB's HTTP client.

#### `knowledge()`
```python
db.knowledge(
    namespace: str,
    query: str,
    k: int = 5,
    include_sources: bool = False,
    min_score: Optional[float] = None,
) -> str
```
Relevant knowledge context string built from the top‑k search results in the namespace.

#### `knowledge_search()`
```python
db.knowledge_search(
    namespace: str,
    query: str,
    k: int = 5,
    include_sources: bool = False,
    min_score: Optional[float] = None,
) -> dict
```
Like `knowledge()` but returns structured results with scored hits.

#### `ask()`
```python
db.ask(
    namespace: str,
    query: str,
    k: int = 5,
    model: Optional[str] = None,
    system_prompt: Optional[str] = None,
    include_sources: bool = False,
    min_score: Optional[float] = None,
    extended_response: bool = False,
) -> dict
```
RAG query: retrieves context from namespace and answers using the configured LLM.

### Chat

#### `chat()`
```python
db.chat(
    namespace: str,
    messages: List[dict],
    model: Optional[str] = None,
    system_prompt: Optional[str] = None,
    k: int = 5,
    min_score: Optional[float] = None,
    extended_response: bool = False,
) -> dict
```
Multi‑turn RAG chat. Each message is `{"role": "user" | "assistant", "content": str}`.

### Node / Graph API

These methods operate on the low-level node-graph model (node IDs, edges, graph traversals), independent of the namespace-scoped memory API. They are part of the Rust `VantaEmbedded::insert_node` / `VantaEmbedded::get_node` system and correspond to the `VantaNodeInput` / `VantaNodeRecord` types.

#### `insert()`
```python
db.insert(
    id: int,
    content: str,
    vector: List[float],
    fields: Optional[dict] = None,
) -> None
```
Insert a graph node with content and an optional embedding vector. The `id` is a `u64` unique node identifier.

#### `get()`
```python
db.get(
    id: int,
) -> Optional[dict]
```
Retrieve a graph node by its numeric `id`. Returns `None` if not found. The returned dict contains `id`, `confidence_score`, `importance`, `hits`, `last_accessed`, `epoch`, `tier`, `is_alive`, `vector`, `vector_dims`, `fields`, and `edges`.

#### `delete()`
```python
db.delete(
    id: int,
    reason: str = "manual deletion",
) -> None
```
Delete a node by ID with an auditable reason (creates a tombstone).

#### `search()`
```python
db.search(
    vector: List[float],
    top_k: int = 10,
) -> List[Tuple[int, float]]
```
Pure vector K-NN search (bypasses memory namespace model). Returns `(node_id, distance)` tuples.

#### `search_batch()`
```python
db.search_batch(
    vectors: List[List[float]],
    top_k: int = 10,
) -> List[List[Tuple[int, float]]]
```
Batch vector K-NN search using Rayon parallel iteration. Searches multiple vectors at once.

#### `query()`
```python
db.query(
    iql_query: str,
) -> str
```
Execute an IQL or LISP query string against the node-graph model. Returns a formatted result string describing read/write results.

#### `add_edge()`
```python
db.add_edge(
    source_id: int,
    target_id: int,
    label: str,
    weight: Optional[float] = None,
) -> None
```
Add a labeled directed edge between two graph nodes (e.g., `"belongs_to"`, `"similar_to"`).

#### `graph_bfs()`
```python
db.graph_bfs(
    roots: List[int],
    max_depth: int = 999999,
) -> List[int]
```
Breadth-First Search from root node IDs, up to `max_depth`. Returns discovered node IDs.

#### `graph_dfs()`
```python
db.graph_dfs(
    roots: List[int],
    max_depth: int = 999999,
) -> List[int]
```
Depth-First Search from root node IDs, up to `max_depth`. Returns discovered node IDs.

#### `graph_topological_sort()`
```python
db.graph_topological_sort(
    roots: List[int],
) -> List[int]
```
Topological sort of the subgraph reachable from roots. Raises an error if a cycle is detected.

#### `graph_is_dag()`
```python
db.graph_is_dag(
    roots: List[int],
) -> bool
```
Check if the subgraph reachable from roots is a Directed Acyclic Graph.

### Maintenance & Diagnostics

#### `flush()`
```python
db.flush() -> None
```
Flush WAL and HNSW index to disk for durability. Releases the GIL during disk sync.

#### `compact_wal()`
```python
db.compact_wal() -> None
```
Compact the WAL: flush, archive `vanta.wal` as `vanta.wal.<timestamp>`, and start a fresh WAL.

#### `purge_expired()`
```python
db.purge_expired() -> int
```
Scan all memory records and physically delete TTL-expired ones. Returns the count of purged records.

#### `rebuild_index()`
```python
db.rebuild_index() -> dict
```
Rebuild ANN (HNSW), text, and derived indexes from canonical storage. Returns a rebuild report dict.

#### `compact_layout()`
```python
db.compact_layout() -> int
```
Compact the storage layout: reorder nodes in BFS order to improve locality and free unused pages. Returns the number of nodes compacted.

#### `export_namespace()`
```python
db.export_namespace(
    path: str,
    namespace: str,
) -> dict
```
Export a single namespace as a JSONL file. Returns a report with `records_exported`, `namespaces`, `path`, and `duration_ms`.

#### `export_all()`
```python
db.export_all(
    path: str,
) -> dict
```
Export all namespaces as JSONL files. Returns the same report shape as `export_namespace`.

#### `import_file()`
```python
db.import_file(
    path: str,
) -> dict
```
Import records from a VantaDB memory JSONL export file. Returns a report with `inserted`, `updated`, `skipped`, `errors`, and `duration_ms`.

#### `audit_text_index()`
```python
db.audit_text_index(
    namespace: Optional[str] = None,
    deep: bool = False,
) -> dict
```
Run a read-only structural audit of the derived text index. Returns a detailed audit report with schema version, tokenizer info, entry mismatches, and structural validity.

#### `repair_text_index()`
```python
db.repair_text_index() -> dict
```
Rebuild the text index from canonical storage as a repair primitive. Returns a repair report.

#### `operational_metrics()`
```python
db.operational_metrics() -> dict
```
Return operational metrics: startup duration, WAL replay (ms + records replayed), ANN/derived/text rebuild times, query counts, export/import stats, and per-subsystem memory breakdown (RSS, virtual, HNSW logical, mmap resident, volatile cache).

#### `capabilities()`
```python
db.capabilities() -> dict
```
Introspect the stable runtime capabilities. Returns a dict with `profile` (`ENTERPRISE`/`PERFORMANCE`/`LOW_RESOURCE`), `read_only`, `persistence`, `vector_search`, and `iql_queries`.

#### `hardware_profile()`
```python
db.hardware_profile() -> dict
```
Merged view of `capabilities()` + `operational_metrics()` memory telemetry (RSS, virtual bytes, HNSW logical bytes, mmap resident, cache entries).

#### `generate_snippet()`
```python
db.generate_snippet(
    payload: str,
    text_query: str,
    with_highlighting: bool = False,
) -> Optional[str]
```
Generate a text snippet from a payload, highlighting matched query terms. Returns `None` if the text index is not available.

#### `explain_memory_search()`
```python
db.explain_memory_search(
    namespace: str,
    query_vector: List[float],
    filters: Optional[dict] = None,
    text_query: Optional[str] = None,
    top_k: int = 10,
    distance_metric: Optional[str] = None,
) -> dict
```
Return a detailed breakdown of how a memory search arrives at its results: search route, fusion report, and per-hit explanation (matched tokens, BM25 term contributions, RRF ranks, snippet).

#### `close()`
```python
db.close() -> None
```
Flush and close the embedded engine handle. Releases WASM resources (in browser/TS context). In Python, it signals the engine to shut down cleanly.

### Chunking

#### `from_file()`
```python
db.from_file(
    file_path: str,
    namespace: str,
    key_prefix: Optional[str] = None,
    chunk_size: int = 1000,
    chunk_overlap: int = 200,
    verbose: bool = True,
) -> dict
```
Read a file (`.txt`, `.md`, `.json`, `.jsonl`), chunk its content, and store each chunk as a record.

#### `from_url()`
```python
db.from_url(
    url: str,
    namespace: str,
    key_prefix: Optional[str] = None,
    chunk_size: int = 1000,
    chunk_overlap: int = 200,
    verbose: bool = True,
) -> dict
```
Fetch a URL, extract text, chunk, and store.

#### `split_text()`
```python
db.split_text(
    text: str,
    chunk_size: int = 1000,
    chunk_overlap: int = 200,
) -> List[str]
```
Split text into chunks without storing.

## Data Types

### `VantaMemoryInput`
- `namespace: str` — namespace identifier
- `key: str` — unique identifier within namespace
- `payload: str` — text content
- `metadata: Optional[dict]` — arbitrary scalar metadata
- `vector: Optional[List[float]]` — optional pre-computed embedding

### Search Result
Each result is a dict with keys:
- `key` — record key
- `payload` — record text content
- `namespace` — namespace
- `score` — combined similarity score
- `score_parts` — breakdown of vector and lexical scores
- `metadata` — scalar metadata (if any)
- `vector` — embedding (if `include_vectors=True`)
- `created_at` — timestamp
- `importance` — importance score
- `extra` — keyword extraction metadata (if `extended_response=True`)

### Consolidation Report
Returned dict with:
- `summaries_created` — count of new summaries
- `records_consolidated` — count of records merged
- `tokens_used` — total tokens consumed
- `inference_calls` — LLM calls made

## Return Types

| Method | Return Type |
|--------|-------------|
| `put` | `dict` |
| `get` (memory) | `Optional[dict]` |
| `get` (node) | `Optional[dict]` |
| `delete` (memory) | `bool` |
| `delete` (node) | `None` |
| `search` (memory) | `List[dict]` |
| `search` (vector) | `List[Tuple[int, float]]` |
| `search_batch` | `List[List[Tuple[int, float]]]` |
| `list` | `dict` |
| `count` | `int` |
| `list_namespaces` | `List[str]` |
| `get_namespace_info` | `Optional[dict]` |
| `put_batch` | `dict` |
| `insert` | `None` |
| `add_edge` | `None` |
| `graph_bfs` | `List[int]` |
| `graph_dfs` | `List[int]` |
| `graph_topological_sort` | `List[int]` |
| `graph_is_dag` | `bool` |
| `query` | `str` |
| `flush` | `None` |
| `compact_wal` | `None` |
| `purge_expired` | `int` |
| `rebuild_index` | `dict` |
| `compact_layout` | `int` |
| `export_namespace` | `dict` |
| `export_all` | `dict` |
| `import_file` | `dict` |
| `audit_text_index` | `dict` |
| `repair_text_index` | `dict` |
| `operational_metrics` | `dict` |
| `capabilities` | `dict` |
| `hardware_profile` | `dict` |
| `generate_snippet` | `Optional[str]` |
| `explain_memory_search` | `dict` |
| `close` | `None` |
| `from_documents` | `dict` |
| `from_file` | `dict` |
| `from_url` | `dict` |
| `split_text` | `List[str]` |
| `update_payload` | `Optional[dict]` |
| `update_metadata` | `Optional[dict]` |
| `update_importance` | `Optional[dict]` |
| `rename_key` | `Optional[dict]` |
| `consolidate` | `dict` |
| `get_namespace_insights` | `Optional[dict]` |
| `knowledge` | `str` |
| `knowledge_search` | `dict` |
| `ask` | `dict` |
| `chat` | `dict` |
| `query_ollama` | `str` |
| `monitor_reset_window` | `None` |

## Error Handling

All methods raise `RuntimeError` with a descriptive message on failure. The underlying Rust errors are propagated through PyO3.

## Requirements

- **Python 3.9+**
- **OS**: Linux, macOS (ARM/x86), Windows
- **Dependencies**: none at the Python level (all dependencies are vendored via Rust)

## Development

```bash
git clone https://github.com/your-org/vantadb.git
cd vantadb-python
pip install maturin
maturin develop  # or: maturin develop --release
```

## Changelog

### 0.1.5
- `update_payload()`, `update_metadata()`, `update_importance()`, `rename_key()`
- `consolidate()`, `get_namespace_insights()`
- `knowledge()`, `knowledge_search()`, `ask()`, `chat()`
- `from_file()`, `from_url()`, `split_text()`
- `from_documents()` (batch document loading)
- `query_ollama()` (direct LLM query)

### 0.1.4
- `put_batch()`, `list_namespaces()`
- `get_namespace_info()`
- `monitor_reset_window()`
- `delete_namespace()`, `count()`

### 0.1.3
- `search()` with metadata filtering and `min_score`
- `mode` parameter for lexical/vector/hybrid search
- `extended_response` for keyword extraction metadata
- `min_score` support
- `embed_text()` method

### 0.1.2
- `put()`, `get()`, `search()`, `list()`, `delete()` operations
- Node graph API: `insert()`, `get()`, `delete()`, `search()`, `add_edge()`, `graph_bfs()`, `graph_dfs()`, `graph_topological_sort()`, `graph_is_dag()`
- Maintenance: `flush()`, `compact_wal()`, `purge_expired()`, `rebuild_index()`, `compact_layout()`
- Observability: `operational_metrics()`, `capabilities()`, `hardware_profile()`
- Utilities: `query()` (IQL), `generate_snippet()`, `explain_memory_search()`
- Export/Import: `export_namespace()`, `export_all()`, `import_file()`
- Text index: `audit_text_index()`, `repair_text_index()`
- Python 3.9+ support
- Windows support

### 0.1.1
- Initial release
- Rust SDK core bindings through PyO3
- Embedded vector database with HNSW indexing
- Namespace organization

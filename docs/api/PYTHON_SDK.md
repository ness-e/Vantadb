---
title: Python SDK Documentation
type: api
status: active
tags: [vantadb, api]
last_reviewed: 2026-09-01
aliases: []
---

# Python SDK Documentation

> **Stability:** the documented Python SDK API is covered by the [Versioning & Stability Policy](VERSIONING.md).

## Installation

```bash
pip install vantadb-py
```

> **Note:** Requires Python 3.11+ and Rust toolchain (maturin) for building from source. Pre-built wheels are available for linux/amd64, linux/arm64 (aarch64), macOS (arm64/x86_64), and Windows (amd64).

## Quick Start

```python
import vantadb

db = vantadb.VantaDB("./vanta_data")

db.put(
    namespace="agent/main",
    key="memory-1",
    payload="The user prefers dark mode in all applications.",
    vector=[0.1] * 384,  # VectorInput: List[float], VantaVector, or np.ndarray
)

# Hybrid search (memory API)
results = db.search_memory(
    namespace="agent/main",
    text_query="What display mode does the user prefer?",
    query_vector=[0.1] * 384,
)
print(results)

# Generate a snippet highlighting the match
if results and results.get("records"):
    payload = results["records"][0]["record"]["payload"]
    snippet = db.generate_snippet(
        payload=payload,
        text_query="display mode",
        with_highlighting=True
    )
    print(f"Snippet: {snippet}")
```
*Note: For more details on search execution, see [[hybrid-search|Hybrid Search]].*

## Import name

Use the canonical import:

```python
import vantadb
```

`import vantadb_py` still works (it points at the same compiled module) but
emits a `DeprecationWarning`. The legacy name will be removed in the next minor
release (0.6.0). The distribution on PyPI is `vantadb-py`; the importable
module is `vantadb`. See [ADR-030](../architecture/adr/ADR-030-brand-identity-naming-convention.md)
for the full brand-identity decision.

## Domain Sub-clients

Every flat method is also reachable through a **domain sub-client**: `db.memory`, `db.graph`, `db.system`, `db.wiki`. Sub-clients are pure organizational sugar over the flat API — each call forwards verbatim to the same-named method on the parent handle.

> **Backward-compat guarantee:** the flat API is unchanged. `db.memory.get_memory(...)` and `db.get_memory(...)` are the same call; existing code keeps working as-is. Canonical method→domain map: [BINDINGS_NAMESPACES.md](BINDINGS_NAMESPACES.md).

```python
# memory — namespace+key records, search, supersede, TTL
record = db.memory.put(namespace="ns", key="k", payload="...", vector=[0.1] * 384)
hits = db.memory.search_memory(namespace="ns", query_vector=[0.1] * 384)
db.memory.supersede(namespace="ns", old_key="draft-v1", new_key="draft-v2")

# graph — node/edge CRUD + traversals
# NOTE: insert/get/delete are NODE-level ops here (id: u128), unlike the
# memory-record semantics those names carry in the TS/WASM bindings.
db.graph.insert(id=42, content="...", vector=[0.1] * 384)
node = db.graph.get(id=42)
reachable = db.graph.graph_bfs(roots=[42], max_depth=3)
ranks = db.graph.graph_page_rank(roots=[42])

# wiki — summary-node archive recovery
nodes = db.wiki.recover_archived_nodes(summary_id="42")

# system — lifecycle, metrics, IQL, maintenance, import/export
print(db.system.capabilities())
result = db.system.query("(match (node :content \"rust\") (return node))")
db.system.flush()
```

Notes:

- Each attribute returns a lightweight delegate that holds a reference to the parent `VantaDB`; calls are forwarded with identical signatures and results.
- The full member lists per sub-client are fixed by [`BINDINGS_NAMESPACES.md`](BINDINGS_NAMESPACES.md) (Python section): memory 15 · graph 10 · system 17 · wiki 1.
- `AsyncVantaDB` does not expose sub-clients yet.

## API Reference

### Constructor

```python
vantadb.VantaDB(
    db_path: str,
    memory_limit_bytes: Optional[int] = None,
    read_only: bool = False,
    backend: Optional[str] = None,
) -> VantaDB
```

### Module-Level Functions

#### `connect()`

```python
vantadb.connect(
    path: str,
    memory_limit: Optional[int] = None,
    read_only: bool = False,
    backend: Optional[str] = None,
) -> VantaDB
```

Alternative constructor. Accepts a filesystem path, empty string `""`, or `":memory:"` for an in-memory database. This is equivalent to `VantaDB(db_path=path, memory_limit_bytes=memory_limit, read_only=read_only, backend=backend)`.

```python
import vantadb

# In-memory database
db = vantadb.connect(":memory:")

# Persistent database with memory limit
db = vanta.connect("./my_brain", memory_limit=256 * 1024 * 1024)
```

### Memory API (Namespace-Scoped)

#### `put()`
```python
db.put(
    namespace: str,
    key: str,
    payload: str,
    metadata: Optional[dict] = None,
    vector: Optional[VectorInput] = None,
    ttl_ms: Optional[int] = None,
) -> VantaMemoryRecord
```
Insert or update a memory record. The `metadata` is a dict of scalar fields.
#### `put_batch()`

```python
db.put_batch(
    keys: List[str],
    vectors: List[VectorInput],
    payloads: Optional[List[str]] = None,
    metadatas: Optional[List[Optional[dict]]] = None,
    namespace: Optional[str] = None,
    namespaces: Optional[List[str]] = None,
    ttls: Optional[List[Optional[int]]] = None,
) -> List[VantaMemoryRecord]
```
Insert or update multiple records in parallel.

**Keyword API** (preferred):
```python
db.put_batch(
    keys=["k1", "k2"],
    vectors=[[0.1]*384, [0.2]*384],
    payloads=["payload1", "payload2"],
    metadatas=[{"f": "v"}, None],
    namespace="agent/default",
    ttls=[None, 1000],
)
```

To route records of one batch into different namespaces, pass the parallel per-record column `namespaces` (length must equal `keys`); it overrides `namespace` for each record:
```python
db.put_batch(
    keys=["k1", "k2"],
    vectors=[[0.1]*384, [0.2]*384],
    namespaces=["ns1", "ns2"],
)
```

Returns a list of `VantaMemoryRecord` objects, up to ~5x faster than sequential `put()` for large batches.

#### `get_memory()`
```python
db.get_memory(
    namespace: str,
    key: str,
) -> Optional[VantaMemoryRecord]
```

#### `delete_memory()`
```python
db.delete_memory(
    namespace: str,
    key: str,
) -> bool
```

#### `list_memory()`
```python
db.list_memory(
    namespace: str,
    filters: Optional[dict] = None,
    limit: int = 100,
    cursor: Optional[int] = None,
) -> VantaListResult
```
Returns a `VantaListResult` object with `.records`, `.total_count`, and `.next_cursor`. Supports `__getitem__` for dict-style access (`result["records"]`, `result["next_cursor"]`) and `__iter__` for record iteration.

```python
page = db.list_memory("ns", limit=10)
for record in page:
    print(record.key, record.payload)

# Dict-style access
records = page["records"]
next_cursor = page["next_cursor"]
```
#### `search_memory()`

```python
db.search_memory(
    namespace: str,
    query_vector: VectorInput,
    filters: Optional[dict] = None,
    text_query: Optional[str] = None,
    top_k: int = 10,
    distance_metric: Optional[str] = None,
    method: Optional[str] = None,
    explain: bool = False,
    exclude_superseded: bool = False,
) -> List[VantaSearchHit]
```
Search namespace-scoped persistent memory records by vector + filters + text_query.

The `method` parameter accepts `"ivf"`, `"scann"`, `"flat"`, or `"hnsw"` to explicitly override the dense-vector index backend. `None` (default) keeps automatic engine routing.

The `exclude_superseded` parameter (default `False`) controls whether superseded records are filtered from results (ADR-028).
#### `explain_memory_search()`

```python
db.explain_memory_search(
    namespace: str,
    query_vector: VectorInput,
    filters: Optional[dict] = None,
    text_query: Optional[str] = None,
    top_k: int = 10,
    distance_metric: Optional[str] = None,
    method: Optional[str] = None,
) -> dict
```
Returns a detailed breakdown of how a memory search arrives at its results.

The `method` parameter accepts `"ivf"`, `"scann"`, `"flat"`, or `"hnsw"` to explicitly override the dense-vector index backend. `None` (default) keeps automatic engine routing.

#### `count()`
```python
db.count(
    namespace: str,
    filters: Optional[dict] = None,
) -> int
```
Count memory records in a namespace, optionally filtered by metadata. The
`filters` dict follows the canonical cross-SDK operator format: a flat value is
an implicit `$eq` (`{"category": "task"}`), and a nested dict selects an
operator per key (`{"score": {"$gte": 50}}`). Supported operators: `$eq`,
`$neq`, `$gt`, `$gte`, `$lt`, `$lte`. Omit `filters` (or pass `None`) to count
all records in the namespace. GIL-released.

```python
db.put("ns", "a", "alpha", metadata={"category": "task", "score": 10})
db.put("ns", "b", "beta", metadata={"category": "task", "score": 50})
db.put("ns", "c", "gamma", metadata={"category": "note", "score": 5})

db.count("ns")                          # 3
db.count("ns", {"category": "task"})    # 2
db.count("ns", {"score": {"$gte": 20}}) # 1
```

#### `delete_by_filter()`
```python
db.delete_by_filter(
    namespace: str,
    filters: dict,
) -> int
```
Delete all memory records in a namespace matching a metadata filter. The
`filters` dict uses the same operator format as `count()` (flat value →
implicit `$eq`, or `{"$op": value}` per key). Returns the number of records
deleted. **The filter must not be empty** — the core rejects an empty filter
with a `RuntimeError` to prevent accidental full-namespace deletion. Use
`delete_memory()` to remove individual records. GIL-released.

```python
deleted = db.delete_by_filter("ns", {"category": "draft"})
print(f"Removed {deleted} draft records")
```

#### `similar_to_key()`
```python
db.similar_to_key(
    namespace: str,
    key: str,
    top_k: int = 10,
) -> List[VantaSearchHit]
```
Search namespace-scoped memory records by vector similarity to an existing
key, without supplying a query vector. Resolves the record at `key`, reads its
embedding, and runs a vector search. The source record itself is excluded from
the results. GIL-released.

```python
hits = db.similar_to_key("agent/main", "task-1", top_k=5)
for hit in hits:
    print(hit.key, hit.score)
```

Raises `RuntimeError` if the source `key` does not exist or has no vector.

### Node / Graph API (Low-Level)

#### `insert()`
```python
db.insert(
    id: int,
    content: str,
    vector: VectorInput,
    fields: Optional[dict] = None,
) -> None
```
Insert a graph node with text content and an optional embedding vector. `fields` can contain additional metadata key-value pairs (supports `str`, `int`, `float`, `bool`, `datetime`, and homogeneous lists). GIL-released — allows Python threads to run during the insert.

```python
db.insert(
    id=42,
    content="VantaDB is a vector-graph database.",
    vector=[0.1] * 384,
    fields={"source": "docs", "year": 2026},
)
```

#### `get()`
```python
db.get(
    id: int,
) -> Optional[dict]
```
Retrieve a graph node by its numeric ID. Returns a dict with `id`, `vector`, `vector_dims`, `fields`, `edges`, `confidence_score`, `importance`, `hits`, `tier`, and `is_alive`, or `None` if not found. GIL-released.

```python
node = db.get(id=42)
if node:
    print(node["fields"], node["vector_dims"])
```

#### `delete()`
```python
db.delete(
    id: int,
    reason: str = "manual deletion",
) -> None
```
Delete a graph node by ID with an auditable reason (recorded as a tombstone). GIL-released.

```python
db.delete(id=42, reason="stale training data cleaned up")
```

#### `supersede()`
```python
db.supersede(
    namespace: str,
    old_key: str,
    new_key: str,
) -> None
```
Mark an existing memory record as superseded by another existing record (ADR-028). The old record keeps its data but gains `superseded_by`/`superseded_at_ms` and can be hidden from search/list with `exclude_superseded=True`. Raises `RuntimeError` if either key is missing, if `old_key == new_key`, or if the old record is already superseded. GIL-released.

```python
db.supersede(namespace="agents/summary", old_key="draft-v1", new_key="draft-v2")
```

#### `search()`
```python
db.search(
    vector: VectorInput,
    top_k: int = 10,
) -> List[Tuple[int, float]]
```
Pure vector K-NN search over all graph nodes. Returns a list of `(node_id, distance)` tuples sorted by ascending distance. GIL-released — HNSW traversal runs in Rust without blocking the Python thread.

```python
hits = db.search(vector=[0.1] * 384, top_k=5)
for node_id, distance in hits:
    print(f"node {node_id}: distance {distance:.4f}")
```

#### `search_batch()`
```python
db.search_batch(
    vectors: List[VectorInput],
    top_k: int = 10,
) -> List[List[Tuple[int, float]]]
```
Batch K-NN search over multiple query vectors. Each query returns its own list of `(node_id, distance)` tuples. Internally parallelized via Rayon for concurrent HNSW traversal. GIL-released.

```python
queries = [[0.1] * 384, [0.5] * 384, [0.9] * 384]
results = db.search_batch(vectors=queries, top_k=3)
for i, hits in enumerate(results):
    print(f"Query {i}: {len(hits)} hits")
```

#### `search_batch_requests()`
```python
db.search_batch_requests(
    requests: List[Union[Dict[str, Any], "SearchRequest"]],
    top_k: int = 10,
) -> List[List[SearchHit]]
```
Batch hybrid search over multiple request objects. Each element may be a
`dict` or a typed `SearchRequest` dataclass, and supports the fields
`namespace`, `query_vector`, `text_query`, `filters`, `distance_metric`, and
`top_k`. Requests are validated eagerly (raises `ValueError` on the first
invalid element), then executed in parallel via Rayon with the GIL released.
Returns one `[SearchHit]` list per request, in input order.

```python
requests = [
    {"namespace": "ns", "query_vector": [0.1] * 384},
    {"namespace": "ns", "text_query": "rust", "top_k": 5},
]
all_hits = db.search_batch_requests(requests)
```

#### `add_edge()`
```python
db.add_edge(
    source_id: int,
    target_id: int,
    label: str,
    weight: Optional[float] = None,
    created_at_ms: Optional[int] = None,
) -> None
```
Add a labeled, optionally weighted edge between two graph nodes. Useful for building knowledge graphs, relationships between entities, or graph-based RAG pipelines. GIL-released.

`created_at_ms` sets the edge's creation timestamp as Unix epoch milliseconds (forward and reverse edges share the same logical creation time). If omitted, the current wall-clock time is used. The timestamp is persisted with the edge and available to time-aware graph queries in the core engine.

```python
# Connect two nodes with a relationship edge
db.add_edge(source_id=42, target_id=17, label="references", weight=0.95)

# Record an edge as of a historical point in time
db.add_edge(source_id=42, target_id=17, label="references", created_at_ms=1700000000000)
```

#### `graph_bfs()`
```python
db.graph_bfs(roots: List[int], max_depth: int = 999999, direction: str = "Forward") -> List[int]
```
Breadth-First Search from root node IDs, up to `max_depth`. `direction` is one of `"Forward"`, `"Reverse"`, or `"Both"`. Returns discovered distinct node IDs. GIL-released.

```python
reachable = db.graph_bfs(roots=[42, 17], max_depth=3)
print(f"Reachable nodes: {reachable}")
```

#### `graph_dfs()`
```python
db.graph_dfs(roots: List[int], max_depth: int = 999999, direction: str = "Forward") -> List[int]
```
Depth-First Search from root node IDs, up to `max_depth`. `direction` is one of `"Forward"`, `"Reverse"`, or `"Both"`. Returns discovered distinct node IDs. GIL-released.

```python
reachable = db.graph_dfs(roots=[42], max_depth=5)
print(f"DFS reachable nodes: {reachable}")
```

#### `graph_topological_sort()`
```python
db.graph_topological_sort(roots: List[int]) -> List[int]
```
Topological sort of the subgraph reachable from the given roots. Raises `ValueError` if a cycle is detected. GIL-released.

```python
sorted_nodes = db.graph_topological_sort(roots=[1, 2, 3])
print(f"Topological order: {sorted_nodes}")
```

#### `graph_is_dag()`
```python
db.graph_is_dag(roots: List[int]) -> bool
```
Check whether the subgraph reachable from the given roots is a Directed Acyclic Graph (DAG). GIL-released.

```python
if db.graph_is_dag(roots=[1, 2]):
    print("Subgraph is a DAG — safe for topological sort")
```

#### `graph_page_rank()`
```python
db.graph_page_rank(
    roots: List[int],
    max_iterations: int = 100,
    damping: float = 0.85,
    tolerance: float = 1e-6,
) -> Dict[int, float]
```
Compute PageRank for the subgraph reachable from the given roots. Returns a dict mapping `node_id -> rank`. GIL-released - allows Python threads to run during PageRank computation.

```python
ranks = db.graph_page_rank(roots=[1, 2], max_iterations=100, damping=0.85)
for node_id, rank in sorted(ranks.items(), key=lambda kv: -kv[1]):
    print(node_id, rank)
```

#### `graph_degree_centrality()`
```python
db.graph_degree_centrality(
    roots: List[int],
) -> Dict[int, Tuple[int, int]]
```
Compute degree centrality (in/out degree counts) for the subgraph reachable from the given roots. Returns a dict mapping `node_id -> (in_degree, out_degree)`. GIL-released.

```python
centrality = db.graph_degree_centrality(roots=[1, 2])
for node_id, (in_deg, out_deg) in centrality.items():
    print(node_id, in_deg, out_deg)
```

#### `graph_bfs_filtered()`
```python
db.graph_bfs_filtered(
    roots: List[int],
    max_depth: int = 999999,
    direction: str = "Forward",
    labels: Optional[List[int]] = None,
    time_range: Optional[Tuple[int, int]] = None,
) -> List[int]
```
Breadth-First Search with optional edge label and time filtering from root node IDs, up to `max_depth`. `direction` is one of `"Forward"`, `"Reverse"`, or `"Both"`. `labels` is a list of edge label IDs to follow (empty list disables label filtering). `time_range` is an optional inclusive `(from_ms, to_ms)` window for edge creation time. Returns discovered distinct node IDs in BFS order. GIL-released.

```python
# BFS with no filtering (equivalent to graph_bfs)
reachable = db.graph_bfs_filtered(roots=[42, 17], max_depth=3)

# BFS following only edges with label IDs 1 and 2
reachable = db.graph_bfs_filtered(roots=[42], max_depth=5, labels=[1, 2])

# BFS within a time window (edges created between timestamps)
reachable = db.graph_bfs_filtered(
    roots=[42],
    max_depth=3,
    time_range=(1700000000000, 1800000000000)
)
```

### Advanced Operations

#### `query()`
```python
db.query(
    iql_query: str,
) -> str
```
Execute an IQL (Interactive Query Language) or LISP-style query string against the graph database. Returns a formatted result string describing the query outcome.

```python
# IQL query example
result = db.query("(match (node :content \"rust\") (return node))")
print(result)
```

#### `query_structured()`
```python
db.query_structured(
    iql_query: str,
) -> Dict[str, Any]
```
Execute an IQL or LISP-style query string and return a **structured dict** instead of a formatted string, so callers can consume the result as data. The dict carries a `kind` discriminator:

- `{"kind": "read", "nodes": [{"id": str, "tier": str, "confidence": float, "hits": int}, ...]}` for `SELECT`-style reads.
- `{"kind": "write", "affected_nodes": int, "message": str, "node_id": str | None}` for writes.
- `{"kind": "stale_context", "node_id": str}` when rehydration is required.

`u128` node ids are returned as strings to avoid precision loss.

```python
# IQL query example — structured
result = db.query_structured("(match (node :content \"rust\") (return node))")
print(result["kind"])   # "read"
print(result["nodes"])  # [{"id": "...", "tier": "...", "confidence": 0.5, "hits": 1}, ...]
```

> **Note:** `query_structured()` is additive — the legacy `query()` (which returns a formatted `str`) is unchanged.

#### `bulk_import()`
```python
db.bulk_import(
    path: str,
) -> Dict[str, Any]
```
Bulk-import records from a binary `.vdbdump` file. Returns a dict with `total_records`, `batches_committed`, `duration_ms`. GIL-released.

#### `bulk_import_bytes()`
```python
db.bulk_import_bytes(
    data: bytes,
) -> Dict[str, Any]
```
Bulk-import records from binary bytes (`.vdbdump` format). Returns a dict with `total_records`, `batches_committed`, `duration_ms`. GIL-released.

#### `recover_archived_nodes()`
```python
db.recover_archived_nodes(
    summary_id: str,
) -> List[dict]
```
Recover shadow-archived nodes that belonged to a summary node. Scans TombstoneStorage for nodes with a `belonged_to` edge targeting `summary_id`, re-activates them, and inserts them back into the active store. Returns a list of recovered node dictionaries. `summary_id` is the summary node ID as a decimal string (u128). GIL-released.

```python
nodes = db.recover_archived_nodes(summary_id="42")
for node in nodes:
    print(node["id"], node["fields"])
```

### Maintenance & Diagnostics

#### `flush()`
```python
db.flush() -> None
```
Flush the Write-Ahead Log (WAL) and HNSW index to disk for durability. Recommended before shutdown or after a batch of writes. GIL-released.

```python
db.put("ns", "k", "critical data", vector=[0.1]*384)
db.flush()  # ensure data is durably on disk
```

#### `compact_wal()`
```python
db.compact_wal() -> None
```
Flush, archive the current WAL as `vanta.wal.<timestamp>`, and start a fresh WAL. Prevents unbounded WAL growth under heavy write load. GIL-released.

```python
db.compact_wal()  # archive the current WAL
```

#### `purge_expired()`
```python
db.purge_expired() -> int
```
Scan all memory records and physically delete entries whose TTL has expired. Returns the number of records purged. GIL-released.

```python
purged = db.purge_expired()
print(f"Cleaned up {purged} expired records")
```

#### `rebuild_index()`
```python
db.rebuild_index() -> dict
```
Rebuild the ANN (HNSW) and all derived memory indexes from canonical storage. Useful after large bulk imports or data recovery. Returns a report dict with `scanned_nodes`, `indexed_vectors`, `duration_ms`, and `success`. GIL-released.

```python
report = db.rebuild_index()
print(f"Rebuilt {report['indexed_vectors']} vectors in {report['duration_ms']}ms")
```

#### `reindex_hnsw_from_text()`
```python
db.reindex_hnsw_from_text(namespace: str, page_size: int = 1000) -> None
```
Rebuild the HNSW vector index using the text content of all memory records in a namespace. Iterates through records in paginated batches (default 1000 per page) to prevent OOM on large databases. Each record's text is re-embedded and the vector index is reconstructed from canonical storage.

```python
db.reindex_hnsw_from_text("my-namespace", page_size=500)
print("Vector index rebuilt from text records")
```

#### `compact_layout()`
```python
db.compact_layout() -> int
```
Compact the storage layout by reordering nodes in BFS order to improve locality and free unused pages. Returns the number of nodes compacted. GIL-released.

```python
compacted = db.compact_layout()
print(f"Compacted {compacted} nodes")
```

#### `list_namespaces()`
```python
db.list_namespaces() -> List[str]
```
List all namespaces currently registered in the database.

```python
namespaces = db.list_namespaces()
print(f"Active namespaces: {namespaces}")
```

#### `export_namespace()`
```python
db.export_namespace(path: str, namespace: str) -> dict
```
Export a single namespace as a JSONL file. Returns a report dict with `records_exported`, `path`, and `duration_ms`. GIL-released.

```python
report = db.export_namespace("/tmp/export.jsonl", "agent/main")
print(f"Exported {report['records_exported']} records")
```

#### `export_all()`
```python
db.export_all(path: str) -> dict
```
Export all namespaces as a single JSONL file. Returns a report dict with `records_exported`, `namespaces`, and `duration_ms`. GIL-released.

```python
report = db.export_all("/tmp/full_backup.jsonl")
print(f"All-namespace export: {report}")
```

#### `import_file()`
```python
db.import_file(path: str) -> dict
```
Import records from a VantaDB memory JSONL export file. Returns a report dict with `inserted`, `updated`, `skipped`, `errors`, and `duration_ms`. GIL-released.

```python
report = db.import_file("/tmp/export.jsonl")
print(f"Imported: {report['inserted']} new, {report['updated']} updated")
```

#### `audit_text_index()`
```python
db.audit_text_index(namespace: Optional[str] = None, deep: bool = False) -> dict
```
Run a read-only structural audit of the derived text (BM25) index. With `deep=True`, also validates individual posting entries for positional and term-frequency consistency. Returns a detailed audit report. GIL-released.

```python
report = db.audit_text_index(namespace="agent/main", deep=False)
print(f"Text index audit: {report['status']}")
if not report['passed']:
    print(f"Mismatches: {report['mismatches']}")
```

#### `repair_text_index()`
```python
db.repair_text_index() -> dict
```
Rebuild the text index from canonical storage as a repair primitive. Useful when the audit report indicates corruption. Returns a report dict with `record_count`, `posting_entries`, and `duration_ms`. GIL-released.

```python
report = db.repair_text_index()
print(f"Repaired text index: {report['record_count']} records, {report['posting_entries']} postings")
```

#### `operational_metrics()`
```python
db.operational_metrics() -> dict
```
Return operational metrics: startup timing, WAL replay stats, ANN/text index rebuild times, query counts, memory breakdown (jemalloc, HNSW, mmap, cache). GIL-released.

```python
metrics = db.operational_metrics()
print(f"Process RSS: {metrics['process_rss_bytes'] / 1024**2:.1f} MB")
print(f"HNSW nodes: {metrics['hnsw_nodes_count']}")
```

#### `capabilities()`
```python
db.capabilities() -> dict
```
Introspect the stable runtime capabilities exposed by the SDK. Returns a dict with `profile` (`ENTERPRISE`, `PERFORMANCE`, or `LOW_RESOURCE`), `read_only`, `persistence`, `vector_search`, and `iql_queries`.

```python
caps = db.capabilities()
print(f"Runtime profile: {caps['profile']}")
print(f"Read-only: {caps['read_only']}")
```

#### `hardware_profile()`
```python
db.hardware_profile() -> dict
```
Return capabilities merged with system memory telemetry. Combines the `capabilities()` dict with memory metrics from `operational_metrics()` (RSS, HNSW, mmap, cache, jemalloc). Useful for deployment diagnostics.

```python
profile = db.hardware_profile()
print(f"Profile: {profile['profile']}, RSS: {profile.get('process_rss_bytes', 'N/A')}")
```

#### `generate_snippet()`
```python
db.generate_snippet(
    payload: str,
    text_query: str,
    with_highlighting: bool = False,
) -> Optional[str]
```
Generate a text snippet from a payload, highlighting matched query terms. Returns a context window around the best-matching passage, or `None` if no terms match.

```python
snippet = db.generate_snippet(
    payload="VantaDB is a high-performance vector database written in Rust.",
    text_query="vector database",
    with_highlighting=True,
)
print(snippet)  # e.g. "...**VantaDB** is a high-performance **vector database**..."
```

#### `close()`
```python
db.close() -> None
```
Flush and close the embedded engine handle, releasing all resources. The database can be re-opened by creating a new `VantaDB` instance. GIL-released.

```python
db.close()
```

#### `__enter__()` / `__exit__()` — Synchronous Context Manager
```python
db.__enter__() -> VantaDB
db.__exit__(exc_type, exc_val, exc_tb) -> None
```
Support for the synchronous context manager protocol (`with VantaDB(...) as db:`). `__enter__` returns the database handle; `__exit__` calls `close()` to flush and release resources. This ensures WAL is flushed even if an exception occurs within the `with` block. Available since 0.5.0 (RES-05).

```python
with VantaDB("./my_brain") as db:
    db.put("ns", "key", "payload", vector=[0.1]*384)
# db.close() is called automatically on exit
```

#### `put_batch_raw()`
```python
db.put_batch_raw(
    vectors: VectorInput,
    keys: List[str],
    payloads: Optional[List[str]] = None,
    metadatas: Optional[List[Optional[dict]]] = None,
    namespaces: Optional[List[str]] = None,
    ttls: Optional[List[Optional[int]]] = None,
) -> List[VantaMemoryRecord]
```
Batch insert with raw arrays (no tuple wrapping). Accepts `vectors` as a 2D NumPy array (shape `[N, D]`) for zero-copy buffer protocol input. Optimized for large batches with homogeneous vector dimensions. GIL-released.

```python
import numpy as np
vectors = np.array([[0.1]*384, [0.2]*384], dtype=np.float32)
records = db.put_batch_raw(
    vectors=vectors,
    keys=["k1", "k2"],
    payloads=["payload1", "payload2"],
    namespaces=["ns1", "ns2"],
)
```

#### `new()`
```python
VantaDB.__new__(cls, *args, **kwargs) -> VantaDB
```
Internal constructor — prefer the class constructor `VantaDB(db_path, ...)`.

### NumPy / Buffer Protocol

```python
db.get_array_interface() -> dict
db.get_search_hit_array_interface() -> dict
```
Return `__array_interface__`-compatible descriptors for zero-copy NumPy interop.

### Iteration Protocol

```python
db.__iter__() -> VantaDB    # iterator over search results / record lists
db.__next__() -> dict        # next record
db.__len__() -> int          # length of current result set
db.__getitem__(key) -> Any   # index into current result set
db.__getstate__() -> dict    # pickle serialization
db.__setstate__(state) -> None  # pickle deserialization
```

## Type Aliases

```python
VectorInput = Union[List[float], VantaVector, numpy.ndarray, memoryview]
```
Accepts plain Python lists, `VantaVector`, NumPy arrays, or any buffer-protocol object (zero-copy when possible).

## Data Types

### `VantaVector`

```python
vantadb.VantaVector(data: List[float]) -> VantaVector
```
Zero-copy vector wrapper backed by a `Box<[f32]>`. Exposes NumPy's `__array_interface__` for zero-copy `np.asarray()` conversion, and supports Python sequence iteration, indexing, and pickle serialization.

```python
vec = VantaVector([0.1, 0.2, 0.3])
arr = np.asarray(vec)  # zero-copy view
len(vec)               # 3
vec[0]                 # 0.1
```

### `VantaMemoryRecord`

```python
vantadb.VantaMemoryRecord
```

Each memory record is a typed object with property access and `__getitem__` support:

| Property | Type | Description |
|---|---|---|
| `namespace` | `str` | Namespace scope |
| `key` | `str` | Unique record key |
| `payload` | `str` | Record payload text |
| `metadata` | `dict` | Metadata key-value dict |
| `vector` | `Optional[numpy.ndarray \| VantaVector]` | Embedding vector |
| `created_at_ms` | `int` | Creation timestamp (ms) |
| `updated_at_ms` | `int` | Last update timestamp (ms) |
| `version` | `int` | Monotonic version counter |
| `node_id` | `int` | Internal node ID |
| `expires_at_ms` | `Optional[int]` | TTL expiration timestamp (ms) |

```python
record = db.put("ns", "k", "payload", vector=[0.1]*384)

# Property access
print(record.namespace, record.key, record.payload)

# Dict-style access (via __getitem__)
print(record["namespace"], record["key"], record["version"])
```

### Search Result
Each result is a `VantaSearchHit` object with properties:
- `namespace` — namespace of the matched record
- `key` — key of the matched record
- `payload` — payload text
- `metadata` — metadata dict
- `vector` — `VantaVector` or NumPy array
- `score` — relevance score (BM25, cosine similarity, or RRF fused)
- `id` / `node_id` — numeric node identifier
- `created_at_ms`, `updated_at_ms`, `version`, `expires_at_ms`

### `VantaListResult`

```python
vantadb.VantaListResult
```

Returned by `list_memory()`. Typed page of memory records with pagination.

| Property | Type | Description |
|---|---|---|
| `records` | `List[VantaMemoryRecord]` | Records in this page |
| `total_count` | `int` | Number of records in this page |
| `next_cursor` | `Optional[int]` | Cursor for the next page, or `None` |

Supports iteration, indexing, and dict-style access:

```python
page = db.list_memory("ns")
len(page)                  # total_count
for r in page:             # iterate records
    print(r.key)
page[0]                    # first record
page["records"]            # same as page.records
page["next_cursor"]        # same as page.next_cursor
```

## Async Support

`vantadb` provides an `AsyncVantaDB` class that exposes the same API using `asyncio.to_thread` to release the GIL.

```python
from vantadb import AsyncVantaDB

async with AsyncVantaDB("./my_brain") as db:
    record = await db.get_memory("ns", "key")
    results = await db.search_memory("ns", [1.0, 0.0, 0.0], top_k=5)
    # Query, diagnostics, and mutations are also async
    query_result = await db.query("(match (node :content \"rust\") (return node))")
    metrics = await db.operational_metrics()
    count = await db.purge_expired()
```

### Async Context Manager

`AsyncVantaDB` implements the async context manager protocol (`async with`):

```python
async def __aenter__(self) -> AsyncVantaDB
async def __aexit__(self, exc_type, exc_val, exc_tb) -> None
```

Returns the database handle on enter; calls `close()` on exit (which flushes WAL and releases resources). This ensures proper cleanup even if an exception occurs.

```python
async with AsyncVantaDB("./my_brain") as db:
    await db.put("ns", "key", "payload", vector=[0.1]*384)
# db.close() awaited automatically
```

All VantaDB methods are available on `AsyncVantaDB` with `async/await`, including `put()`, `put_batch()`, `insert()`, `delete_memory()`, `get_memory()`, `list_memory()`, `search_memory()`, `query()`, `flush()`, `compact_wal()`, `purge_expired()`, `rebuild_index()`, `export_namespace()`, `export_all()`, `import_file()`, `audit_text_index()`, `repair_text_index()`, `operational_metrics()`, `capabilities()`, `hardware_profile()`, `get()`, `delete()`, `search()`, `search_batch()`, `add_edge()`, `graph_bfs()`, `graph_dfs()`, `graph_topological_sort()`, `graph_is_dag()`, `compact_layout()`, `list_namespaces()`, `generate_snippet()`, `explain_memory_search()`, `count()`, `delete_by_filter()`, and `similar_to_key()`.

## ID limits

Node IDs are **u128** end-to-end: the core engine, WAL, and the Python binding
all use 128-bit unsigned integers.

- **Range:** `0 <= id <= 2^128 - 1` (`340282366920938463463374607431768211455`).
- **Passing IDs:** use a plain Python `int`. Python integers are arbitrary
  precision, so IDs larger than `u64::MAX` (`18446744073709551615`) work
  directly — there is **no u64 truncation**. (ERR-023: IDs beyond u64 were
  previously truncated or rejected by the binding; that limit no longer
  exists.)
- **Strings:** not required for the regular APIs. The only string-based path is
  `recover_archived_nodes()`, whose `summary_id` is a decimal string parsed to
  `u128`.
- **Out of range:** negative IDs or IDs greater than `u128::MAX` raise
  `OverflowError`; `recover_archived_nodes()` raises `ValueError` for a string
  it cannot parse.

> **JSON caution:** if IDs are transported through JSON (JSONL export/import,
> HTTP API), numbers beyond `2^53` lose precision in tools that decode them as
> IEEE-754 doubles. Keep large IDs as strings in JSON payloads.

## Error Handling

Every VantaDB error raised by this binding is an instance of `VantaError`,
which inherits from `RuntimeError`. This keeps existing `except RuntimeError` /
`except Exception` callers working while letting you catch the specific family:

```python
from vantadb_py import (
    VantaError,
    NotFoundError,
    ValidationError,
    CorruptError,
    StorageError,
    ConflictError,
    UnsupportedError,
    ResourceLimitError,
    BusyError,
    NoVectorError,
    TimeoutError,
)

try:
    db.supersede("ns", "missing", "k")
except NotFoundError as exc:
    print("record not found:", exc)
    if exc.retriable:
        schedule_retry()
```

### Hierarchy

```
VantaError (base, inherits RuntimeError)
├── NotFoundError          # VantaError::NodeNotFound, VantaError::NotFound
├── ValidationError        # VantaError::DimensionMismatch, DuplicateNode, ValidationError, InvalidInput, IqlParseError, NoVectorForKey, UnsupportedOperation, CycleDetected, NodeIdCollision
├── CorruptError           # VantaError::IncompatibleFormat, WALVersionMismatch, SerializationError, SchemaError, RestoreError, BackupError
├── StorageError           # VantaError::IoError, WalError, BackendError, CliError, SearchError, RuntimeError
├── ConflictError          # VantaError::ExecutionConflict
├── UnsupportedError       # VantaError::UnsupportedOperation (typed alias)
├── ResourceLimitError     # VantaError::ResourceLimit
├── BusyError              # VantaError::DatabaseBusy, VantaError::NotInitialized
├── NoVectorError          # VantaError::NoVectorForKey
└── TimeoutError           # VantaError::Timeout
```

Catch the base class to handle any VantaDB error uniformly:
`except VantaError:`.

### Canonical codes (10)

Every error raised through the typed hierarchy carries a `.code` attribute -
the exact `VANTADB_*` wire value produced by Rust `VantaError::code()`
(ERR-PY-01; identical strings as on the TS/MCP wire). **Branch on `.code` for
cross-binding logic; the variant class is for human-readable dispatch only.**

| Code | Python subclass(es) |
|------|---------------------|
| `VANTADB_NOT_FOUND` | `NotFoundError` |
| `VANTADB_VALIDATION_ERROR` | `ValidationError`, `ConflictError`, `UnsupportedError`, `NoVectorError` |
| `VANTADB_INVALID_ARGUMENT` | `ValidationError` (runtime IQL path) |
| `VANTADB_CORRUPT` | `CorruptError` |
| `VANTADB_IO_ERROR` | `StorageError`, `VantaError` base (`CliError`/`SearchError`/`RuntimeError`) |
| `VANTADB_RESOURCE_LIMIT` | `ResourceLimitError` |
| `VANTADB_BUSY` | `BusyError` |
| `VANTADB_TIMEOUT` | `TimeoutError` |
| `VANTADB_WASM_ERROR` | `VantaError` base (WASM `Generic` fallback) |
| `VANTADB_CLOSED` | (handle lifecycle, not raised via `code()`) |

> **Implemented (ERR-CORE-01 + ERR-PY-01):** `.code` carries the prefixed
> `VANTADB_*` form. The unprefixed strings in earlier drafts of this document
> are obsolete. See
> [`docs/api/ERROR_HANDLING.md`](ERROR_HANDLING.md) for the authoritative table.

### Attributes (0.5.0+)

```python
exc.code       # str - one of the 10 canonical VANTADB_* codes above
exc.retriable  # bool - equivalent to Rust `is_retriable()`
exc.hint       # str | None - recovery hint (mirrors Rust `recovery_hint()`)
str(exc)       # human-readable message (Display) - NOT for matching
```

> `.details` (structured variant fields) is **not** attached yet: PyO3
> `create_exception!` types carry no methods and variant-field extraction was
> out of ERR-PY-01 scope. Use `str(exc)` for the message; `.details` is
> tracked for a follow-up task.

Example — retry policy with `.retriable`:

```python
import time
from vantadb_py import VantaError, BusyError

def put_with_retry(db, **kwargs):
    for attempt in range(5):
        try:
            return db.put(**kwargs)
        except VantaError as exc:
            if exc.retriable and attempt < 4:
                time.sleep(0.1 * (2 ** attempt))
                continue
            raise
```

### `error_to_dict()` — cross-binding log correlation

The Python exception classes are built with PyO3 `create_exception!`, which
cannot carry methods - so the spec's `exc.to_dict()` is exposed as a
module-level helper instead (ERR-PY-01 decision). It returns the same plain
dict shape as the TypeScript `VantaError.toJSON()` so logs and traces line up
across Rust/Python/TS/MCP:

```python
import vantadb

try:
    db.get(...)
except vantadb.VantaError as exc:
    log.error("vanta_error", extra=vantadb.error_to_dict(exc))
    # {
    #   "name": "NotFoundError",
    #   "code": "VANTADB_NOT_FOUND",
    #   "message": "...",
    #   "retriable": false,
    #   "hint": "..."
    # }
```

Providers (`vantadb_openai`/`vantadb_litellm`/`vantadb_ollama`) raise the same
MOD-20 class names with `.code`/`.retriable`/`.hint` attached (ERR-PY-01):
`except vantadb_openai.TimeoutError` now works instead of the old
`KeyError`/`RuntimeError` bucket collapse. The provider classes are distinct
type objects from `vantadb_py`'s - catch them per module.

### Migration (0.5.0+)

The binding previously mapped core errors to standard-library exceptions
(`KeyError`, `ValueError`, `FileNotFoundError`, …). These now raise the typed
`VantaError` subclasses above. The one behavior change to be aware of:

| Before | After |
|--------|-------|
| missing key/node → `KeyError` | `NotFoundError` |
| validation / duplicate / dimension → `ValueError` | `ValidationError` |
| file not found / permission / OSError | `StorageError` |
| other engine errors → `RuntimeError` | `VantaError` (still a `RuntimeError`) |

`except RuntimeError` and `except Exception` remain fully compatible because
`VantaError` is a `RuntimeError`.

## Roadmap (not yet available)

The following methods are planned but **not yet available in the Python SDK** — tracked for future release:

*(none currently — `delete_by_filter()`, `similar_to_key()`, and `count()` shipped in 0.5.0.)*

## Development

```bash
git clone https://github.com/ness-e/Vantadb.git
cd vantadb-python
pip install maturin
maturin develop
```

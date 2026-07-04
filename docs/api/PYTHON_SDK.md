---
title: Python SDK Documentation
type: api
status: active
tags: [vantadb, api]
last_reviewed: 2026-07-01
aliases: []
---

# Python SDK Documentation

## Installation

```bash
pip install vantadb-py
```

> **Note:** Requires Python 3.11+ and Rust toolchain (maturin) for building from source. Pre-built wheels are available for linux/amd64, linux/arm64 (aarch64), and macOS (arm64/x86_64).

## Quick Start

```python
import vantadb_py as vantadb

db = vantadb.VantaDB("./vanta_data")

db.put(
    namespace="agent/main",
    key="memory-1",
    payload="The user prefers dark mode in all applications.",
    vector=[0.1] * 384, # Dummy vector for example
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

### Memory API (Namespace-Scoped)

#### `put()`
```python
db.put(
    namespace: str,
    key: str,
    payload: str,
    metadata: Optional[dict] = None,
    vector: Optional[List[float]] = None,
    ttl_ms: Optional[int] = None,
) -> dict
```
Insert or update a memory record. The `metadata` is a dict of scalar fields.

#### `put_batch()`
```python
db.put_batch(
    entries: List[Tuple[str, str, str, Optional[dict], Optional[List[float]], Optional[int]]]
) -> List[dict]
```
Insert or update multiple records in parallel. Each entry is `(namespace, key, payload, metadata, vector, ttl_ms)`.

#### `get_memory()`
```python
db.get_memory(
    namespace: str,
    key: str,
) -> Optional[dict]
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
) -> dict
```
Returns `{"records": [...], "next_cursor": Optional[int]}`.

#### `search_memory()`
```python
db.search_memory(
    namespace: str,
    query_vector: List[float],
    filters: Optional[dict] = None,
    text_query: Optional[str] = None,
    top_k: int = 10,
    distance_metric: Optional[str] = None,
    explain: bool = False,
) -> dict
```
Search namespace-scoped persistent memory records by vector + filters + text_query.

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
Returns a detailed breakdown of how a memory search arrives at its results.

### Node / Graph API (Low-Level)

#### `insert()`
```python
db.insert(
    id: int,
    content: str,
    vector: List[float],
    fields: Optional[dict] = None,
) -> None
```

#### `get()`
```python
db.get(
    id: int,
) -> Optional[dict]
```

#### `delete()`
```python
db.delete(
    id: int,
    reason: str = "manual deletion",
) -> None
```

#### `search()`
```python
db.search(
    vector: List[float],
    top_k: int = 10,
) -> List[Tuple[int, float]]
```
Pure vector K-NN search.

#### `search_batch()`
```python
db.search_batch(
    vectors: List[List[float]],
    top_k: int = 10,
) -> List[List[Tuple[int, float]]]
```

#### `add_edge()`
```python
db.add_edge(
    source_id: int,
    target_id: int,
    label: str,
    weight: Optional[float] = None,
) -> None
```

#### `graph_bfs()`
```python
db.graph_bfs(roots: List[int], max_depth: int = 999999) -> List[int]
```

#### `graph_dfs()`
```python
db.graph_dfs(roots: List[int], max_depth: int = 999999) -> List[int]
```

#### `graph_topological_sort()`
```python
db.graph_topological_sort(roots: List[int]) -> List[int]
```

#### `graph_is_dag()`
```python
db.graph_is_dag(roots: List[int]) -> bool
```

### Advanced Operations

#### `delete_by_filter()`
```python
db.delete_by_filter(
    namespace: str,
    filters: dict,
) -> int
```
Delete all records matching metadata filters in a namespace. Returns count deleted.

#### `similar_to_key()`
```python
db.similar_to_key(
    namespace: str,
    key: str,
    top_k: int = 10,
) -> List[dict]
```
Search by vector similarity from an existing key.

#### `count()`
```python
db.count(
    namespace: Optional[str] = None,
    filters: Optional[dict] = None,
) -> int
```
Count records, optionally filtered by namespace and metadata.

### Maintenance & Diagnostics

#### `flush()`
```python
db.flush() -> None
```

#### `compact_wal()`
```python
db.compact_wal() -> None
```

#### `purge_expired()`
```python
db.purge_expired() -> int
```

#### `rebuild_index()`
```python
db.rebuild_index() -> dict
```

#### `compact_layout()`
```python
db.compact_layout() -> int
```

#### `list_namespaces()`
```python
db.list_namespaces() -> List[str]
```

#### `export_namespace()`
```python
db.export_namespace(path: str, namespace: str) -> dict
```

#### `export_all()`
```python
db.export_all(path: str) -> dict
```

#### `import_file()`
```python
db.import_file(path: str) -> dict
```

#### `audit_text_index()`
```python
db.audit_text_index(namespace: Optional[str] = None, deep: bool = False) -> dict
```

#### `repair_text_index()`
```python
db.repair_text_index() -> dict
```

#### `operational_metrics()`
```python
db.operational_metrics() -> dict
```

#### `capabilities()`
```python
db.capabilities() -> dict
```

#### `hardware_profile()`
```python
db.hardware_profile() -> dict
```

#### `generate_snippet()`
```python
db.generate_snippet(
    payload: str,
    text_query: str,
    with_highlighting: bool = False,
) -> Optional[str]
```
Generate a text snippet from a payload, highlighting matched query terms.

#### `close()`
```python
db.close() -> None
```

## Data Types

### Memory Record
Each memory record is a dict with keys:
- `namespace`
- `key`
- `payload`
- `metadata`
- `vector`
- `created_at_ms`
- `updated_at_ms`
- `version`
- `node_id`
- `expires_at_ms`

### Search Result
Each result is a dict with keys:
- `score`
- `record`
- `explanation`

## Async Support

`vantadb_py` provides an `AsyncVantaDB` class that exposes the same API using `asyncio.to_thread` to release the GIL.

```python
from vantadb_py import AsyncVantaDB

async with AsyncVantaDB("./my_brain") as db:
    record = await db.get_memory("ns", "key")
    results = await db.search_memory("ns", [1.0, 0.0, 0.0], top_k=5)
```

## Error Handling

All methods raise `RuntimeError` with a descriptive message on failure.

## Development

```bash
git clone https://github.com/ness-e/Vantadb.git
cd vantadb-python
pip install maturin
maturin develop
```

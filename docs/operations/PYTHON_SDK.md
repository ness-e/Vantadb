# Python SDK Status

## Current State
- Local installation is supported through `pip install .` inside `vantadb-python/`.
- The Python binding now targets the stable Rust SDK boundary in `src/sdk.rs`.
- `vantadb-python/src/lib.rs` no longer needs direct access to `StorageEngine`, `Executor`, `UnifiedNode`, HNSW locks, or `HardwareCapabilities`.
- The Python import surface remains `import vantadb_py as vanta` for source installs in this cycle.

## Public Binding Boundary
- `VantaEmbedded` owns the embedded engine handle and stable open options.
- `VantaNodeInput`, `VantaNodeRecord`, `VantaQueryResult`, and `VantaCapabilities` form the contract for external SDKs.
- Vector search, node CRUD, edge insertion, query execution, flush, close, and capabilities all route through that boundary.
- Persistent memory APIs also route through the same boundary:
  `put`, `get_memory`, `delete_memory`, `list_memory`, `search_memory`,
  `rebuild_index`, `export_namespace`, `export_all`, `import_file`, and
  `operational_metrics`.

## Memory Flow

```python
import vantadb_py as vantadb

db = vantadb.VantaDB("./vanta_data")
db.put(
    "agent/main",
    "memory-1",
    "local durable memory",
    metadata={"kind": "note"},
    vector=[1.0, 0.0, 0.0],
)

record = db.get_memory("agent/main", "memory-1")
page = db.list_memory("agent/main", filters={"kind": "note"})
hits = db.search_memory("agent/main", [1.0, 0.0, 0.0], top_k=5)
text_hits = db.search_memory("agent/main", [], text_query="durable memory", top_k=5)
hybrid_hits = db.search_memory(
    "agent/main",
    [1.0, 0.0, 0.0],
    text_query="durable memory",
    top_k=5,
)
report = db.rebuild_index()
metrics = db.operational_metrics()
db.export_namespace("./agent-main.jsonl", "agent/main")
db.flush()
db.close()
```

Text-only `search_memory(..., query_vector=[], text_query="...")` uses BM25
lexical retrieval. Combining `text_query` with a non-empty vector uses Hybrid
Retrieval v1: BM25 and vector rankings are executed separately and fused with
RRF. `operational_metrics()` is diagnostic telemetry for startup, WAL replay,
rebuild, lexical queries, hybrid queries, planner routes, export, and import
behavior; it is not a public efficiency claim.

## Remaining Release Debt
- PyPI stays blocked until multiplatform wheels exist for Linux, macOS, and Windows.
- `vantadb-python` still uses a local path dependency on the core crate for in-repo builds.
- Release automation for `maturin build` and TestPyPI/PyPI publish is still pending.
- Public SDK API changes should remain additive until the Python package is distributed externally.

## Explicitly Deferred
- Package renaming, PyPI publication, and wheel distribution
- Signing, installers, and external distribution automation
- Any SDK API that would require exposing storage or executor internals

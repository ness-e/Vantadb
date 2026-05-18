# Python SDK Status

## Current State
- Local installation is supported through `pip install .` inside `vantadb-python/`.
- Package metadata is prepared as `vantadb-py` version `0.1.1`; the import name remains
  `vantadb_py`.
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
  `audit_text_index`, `repair_text_index`, and `operational_metrics`.

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
phrase_hits = db.search_memory("agent/main", [], text_query='"local durable"', top_k=5)
hybrid_hits = db.search_memory(
    "agent/main",
    [1.0, 0.0, 0.0],
    text_query="durable memory",
    top_k=5,
)
report = db.rebuild_index()
audit = db.audit_text_index("agent/main")
metrics = db.operational_metrics()
print(metrics["hnsw_logical_bytes"])
print(metrics["mmap_resident_bytes"])
db.export_namespace("./agent-main.jsonl", "agent/main")
db.flush()
db.close()
```

Text-only `search_memory(..., query_vector=[], text_query="...")` uses BM25
lexical retrieval. Combining `text_query` with a non-empty vector uses Hybrid
Retrieval v1: BM25 and vector rankings are executed separately and fused with
RRF. `operational_metrics()` is diagnostic telemetry for startup, WAL replay,
rebuild, HNSW logical bytes, mmap residency when available, lexical queries,
hybrid queries, planner routes, export, and import behavior; it is not a public
efficiency claim.

Quoted phrase queries such as `text_query='"local durable"'` are supported as
exact consecutive-token filters using the current `lowercase-ascii-alnum`
tokenizer. Hybrid planner/RRF explain and snippet inspection remains Rust
debug-build test support. It is not exposed as a stable Python SDK method.

`audit_text_index(namespace=None)` is a read-only diagnostic method. It reports
derived text-index drift against canonical memory records and recommends
`repair_text_index()` or `rebuild_index()` when the report does not pass. It
does not repair state, including when the database is opened with
`read_only=True`.

## Remaining Release Debt
- `.github/workflows/python_wheels.yml` builds Linux, macOS, and Windows wheels and smoke-installs the generated artifact.
- TestPyPI upload is prepared as an explicit manual workflow input guarded by `TEST_PYPI_API_TOKEN`.
- `vantadb-python/pyproject.toml` points to the canonical GitHub repository at `https://github.com/DevpNess/Vantadb`.
- PyPI production publication is tag-gated and remains an external manual release decision after TestPyPI validation.
- `vantadb-python` still uses a local path dependency on the core crate for in-repo builds.
- Public SDK API changes should remain additive until the Python package is distributed externally.

## TestPyPI Installation

When a TestPyPI release is available, install in a clean environment:

```bash
python -m venv .venv
source .venv/bin/activate
pip install --upgrade pip
pip install --index-url https://test.pypi.org/simple/ --extra-index-url https://pypi.org/simple/ vantadb-py
python -c "import vantadb_py; print(vantadb_py.__name__)"
```

Windows PowerShell:

```powershell
python -m venv .venv
.\.venv\Scripts\Activate.ps1
pip install --upgrade pip
pip install --index-url https://test.pypi.org/simple/ --extra-index-url https://pypi.org/simple/ vantadb-py
python -c "import vantadb_py; print(vantadb_py.__name__)"
```

> **Note**: The `--extra-index-url` fallback to production PyPI is required
> because TestPyPI may not host all transitive dependencies.

## Local Validation

Use the validation scripts to build, install, and test the SDK end-to-end:

```bash
# Unix/macOS
./dev-tools/validate_python_sdk.sh

# Windows PowerShell
.\dev-tools\validate_python_sdk.ps1

# Skip the wheel build and reuse an existing wheel
.\dev-tools\validate_python_sdk.ps1 -SkipBuild
```

## Explicitly Deferred
- Package renaming
- Actual PyPI publication from this task
- Native installers outside Python wheels
- Any SDK API that would require exposing storage or executor internals

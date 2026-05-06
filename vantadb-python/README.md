# VantaDB Python Bindings

`vantadb-py` provides source-installed Python bindings for the VantaDB embedded
persistent memory engine. The import name is `vantadb_py`.

The current package is prepared for wheel validation and TestPyPI checks, but
production PyPI publication remains deferred for the v0.1.x line.

## Install From Source

From the repository root:

```bash
pip install -e ./vantadb-python
```

## Minimal Usage

```python
import vantadb_py as vantadb

db = vantadb.VantaDB("./vanta_data", memory_limit_bytes=128_000_000)
db.put(
    "agent/main",
    "memory-1",
    "local durable memory",
    metadata={"kind": "note"},
    vector=[1.0, 0.0, 0.0],
)

hits = db.search_memory(
    "agent/main",
    [1.0, 0.0, 0.0],
    text_query="durable memory",
    top_k=5,
)
print(hits)
db.close()
```

See the repository README and
https://github.com/DevpNess/Vantadb/blob/main/docs/QUICKSTART.md for the
current MVP boundary and first-run workflows.

# Operations & Configuration Manual

This document tracks the current runtime knobs for the embedded core and the optional local server wrapper.

## 1. Python / Embedded Constructor

Current source-install usage goes through `vantadb_py`:

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `db_path` | `str` | required | Filesystem path for the embedded data directory. |
| `read_only` | `bool` | `False` | Opens the engine in read-only mode. |
| `memory_limit_bytes` | `int \| None` | `None` | Runtime budget hint used by backend and mmap selection logic. It is not currently a documented hard RSS ceiling. |

```python
import vantadb_py as vantadb

db = vantadb.VantaDB(
    "./vanta_data",
    read_only=False,
    memory_limit_bytes=512_000_000,
)
```

## 2. Embedded Runtime Notes

- Fjall is the default storage backend.
- RocksDB remains an explicit fallback path in the core.
- Vector search is cosine-based HNSW.
- Memory records use `namespace + key` identity with scalar metadata and optional vectors.
- Derived namespace/payload indexes are persisted and rebuilt from canonical records.
- Source-install is the supported Python distribution path for this release.

## 3. Embedded CLI

The CLI uses the embedded core directly and does not require the optional HTTP server.

```bash
vanta-cli put --db ./vanta_data --namespace agent/main --key memory-1 --payload "hello"
vanta-cli get --db ./vanta_data --namespace agent/main --key memory-1
vanta-cli list --db ./vanta_data --namespace agent/main
vanta-cli rebuild-index --db ./vanta_data
vanta-cli export --db ./vanta_data --namespace agent/main --out ./agent-main.jsonl
vanta-cli import --db ./vanta_data --in ./agent-main.jsonl
```

## 4. Server Wrapper Environment Variables

If you run the optional local HTTP wrapper:

| Variable | Description | Default |
|----------|-------------|---------|
| `VANTADB_HOST` | Bind address for the HTTP wrapper. | `127.0.0.1` |
| `VANTADB_PORT` | TCP port for the HTTP wrapper. | `8080` |
| `VANTADB_STORAGE_PATH` | Data directory path used by the embedded core. | `./vantadb_data` or server-specific default |
| `VANTADB_THREADS` | Tokio worker count override. | `auto` |
| `RUST_LOG` | Logging verbosity. | `info` |

## 5. Memory Telemetry Caveat

Current telemetry must be interpreted carefully:

- process memory and logical index memory are tracked separately
- process-scoped metrics do not equal mmap residency or page cache
- memory claims should use the contract in [MEMORY_TELEMETRY.md](MEMORY_TELEMETRY.md)

## 6. SIMD and Build Behavior

VantaDB still uses the runtime hardware profile to choose fast paths where available, but public claims should stay tied to validated behavior rather than to a specific SIMD tier alone.

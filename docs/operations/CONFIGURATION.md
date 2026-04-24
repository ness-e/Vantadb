# Operations & Configuration Manual

For DevOps, SREs, and Systems Engineers operating VantaDB in production via Docker or direct PyO3 instantiation.

VantaDB behaves similarly to SQLite: configuration parameters are primarily defined at runtime initialization but can also fall back to OS environment variables.

## 1. Constructor Initialization Params (Python SDK)

When orchestrating VantaDB directly inside your application code:

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `path` | `str` | `"./vanta_data"` | The local filesystem directory for Fjall (default) or RocksDB SSD physical persistence. |
| `read_only` | `bool` | `False` | Locks the FFI thread in Read-Only concurrency mode. Ideal for Uvicorn/Gunicorn multi-process read orchestration. |
| `memory_limit_bytes` | `int` | `1024_000_000` (1GB) | The absolute Cgroups ceiling constraint. Triggers the internal MMap swap (Resource Governance) when approaching runtime RAM panic limits. |

```python
import vantadb

db = vantadb.VantaDB(
    path="/mnt/volume/db",
    read_only=False,
    memory_limit_bytes=512_000_000 # 512MB Hard limit
)
```

## 2. Server Runtime (Environment Variables)

When deploying VantaDB as a standalone HTTP/Axum microservice using the official Docker container, pass the following ENV variables:

| Variable | Description | Default Target |
|----------|-------------|----------------|
| `VANTADB_HOST` | Bind address for the Rust HTTP layer. | `127.0.0.1` |
| `VANTADB_PORT` | Exposure TCP port. | `8080` |
| `VANTADB_STORAGE_PATH` | Equivalency to the `path` param. | `/data` |
| `VANTADB_THREADS` | Tokyo async worker count (defaults to Host vCPUs). | `auto` |
| `RUST_LOG` | Telemetry verbosity (`info`, `debug`, `trace`, `error`). | `info` |

## 3. Hardware Optimizations & SIMD

VantaDB natively compiles using `target-cpu=native`. This guarantees that your compiled binary leverages hardware-specific **SIMD / AVX-512** instruction sets present natively on your underlying x86/ARM motherboard when computing massive array multiplications during Vector Distance operations. No explicit ENV flags are required for optimization.

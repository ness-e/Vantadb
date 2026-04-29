<div align="center">
  <h1>VantaDB</h1>
  <p><b>Embedded persistent memory and vector retrieval engine for local-first AI applications</b></p>
  
  [![CI](https://github.com/DevpNess/Vantadb/actions/workflows/rust_ci.yml/badge.svg)](https://github.com/DevpNess/Vantadb/actions/workflows/rust_ci.yml)
  [![Release](https://img.shields.io/github/v/release/DevpNess/Vantadb?label=release)](https://github.com/DevpNess/Vantadb/releases)
  [![License](https://img.shields.io/github/license/DevpNess/Vantadb)](https://github.com/DevpNess/Vantadb/blob/main/LICENSE)
</div>

<br>

**VantaDB** is an embedded Rust engine for durable local memory. Today it focuses on persistent storage, HNSW vector retrieval, structured metadata, and crash recovery without requiring a separate database service.

`v0.1.0` is the current MVP cut. Docker and Ollama remain deferred or experimental, the Python package is still install-from-source, and lexical BM25/RRF hybrid search is not part of the current shipped surface. The repo now includes the `memory-mvp-core` surface: namespace-scoped memory CRUD/search, manual ANN rebuild, JSONL export/import, and embedded CLI flows.

## Current Product Boundary

VantaDB should currently be understood as:

- embedded-first and local-first
- durable memory with WAL-backed recovery
- cosine-based HNSW vector retrieval
- structured fields stored alongside vectors
- optional local server wrapper around the same core

VantaDB should **not** currently be understood as:

- a universal multimodel database
- full hybrid text + vector search with BM25/RRF
- a cloud or enterprise platform
- a PyPI-ready distributed product

## Technical Capabilities

The current repo supports the following production-facing capabilities:

| Engine | Mechanism | Details |
|--------|-----------|---------|
| **Persistent Core** | `StorageBackend` + VantaFile + WAL | Fjall is the default KV backend, RocksDB remains an explicit fallback, and index reconstruction is supported when the ANN artifact is missing. |
| **Vector Retrieval** | Native HNSW | Cosine similarity with configurable `M`, `ef_construction`, and `ef_search`. Internal validation covers 10K–100K synthetic datasets. |
| **Persistent Memory API** | `namespace + key` records | `put/get/delete/list/search` store UTF-8 payloads, scalar metadata, optional vectors, timestamps, versions, and deterministic node IDs behind the SDK boundary. |
| **Structured Data** | Derived namespace/payload indexes | Metadata is stored with each record and equality filters use persisted derived indexes that can be rebuilt from canonical records. |
| **Graph Edges** | Local adjacency lists | Directed edges with optional weights are stored in the same node model for local traversals and related lookups. |
| **Embedded SDK Surface** | Stable Rust boundary + PyO3 binding | The Python binding is now routed through `src/sdk.rs` instead of accessing storage and executor internals directly. |
| **Operational Flows** | Rebuild + JSONL | ANN rebuild and memory export/import are available through Rust SDK, Python SDK, and `vanta-cli`. |

## Install From Source

No separate cluster or external service is required.

```bash
pip install -e ./vantadb-python
```

```python
import vantadb_py as vantadb

db = vantadb.VantaDB("./vanta_data", memory_limit_bytes=512_000_000)

record = db.put(
    "agent/main",
    "memory-1",
    "In-process execution minimizes latency",
    metadata={"category": "architecture", "version": 1},
    vector=[0.12, 0.88, 0.54],
)

stored = db.get_memory("agent/main", "memory-1")
hits = db.search_memory("agent/main", [0.11, 0.89, 0.55], top_k=5)
caps = db.hardware_profile()
db.flush()
db.close()

print(record)
print(stored)
print(hits)
print(caps)
```

Embedded CLI smoke:

```bash
cargo run --bin vanta-cli -- put --db ./vanta_data --namespace agent/main --key memory-1 --payload "hello"
cargo run --bin vanta-cli -- list --db ./vanta_data --namespace agent/main
cargo run --bin vanta-cli -- export --db ./vanta_data --namespace agent/main --out ./memory.jsonl
cargo run --bin vanta-cli -- rebuild-index --db ./vanta_data
```

## Search Semantics Today

- The shipped ANN path is **cosine similarity only**.
- Namespace-scoped `list/search` use derived namespace and scalar metadata indexes, with canonical records remaining the source of truth.
- BM25, RRF, and true text+vector hybrid ranking are planned next-cycle work and are not claims for `v0.1.0`.
- SIFT1M remains useful as a stress/recovery scenario, not as a competitive benchmark while the engine stays cosine-only.

## Optional Server Mode

For local development or network exposure without Python, you can run the standalone binary. This wraps the embedded core; it is not the primary product identity for this release.

1. Download the `vanta-server-windows-amd64.exe` from the official [GitHub Releases](https://github.com/DevpNess/Vantadb/releases).
2. Open PowerShell or Command Prompt in the download directory and run:
   ```cmd
   .\vanta-server-windows-amd64.exe
   ```

**Defaults:**
- **Data Directory**: Creates a `vantadb_data` folder in the current execution directory.
- **Bind Address**: Listens on `127.0.0.1:8080` (safe localhost default).

**Exposing to the Network:**
If you want to expose the server to your LAN or the internet, override the host via environment variables before running:
```cmd
set VANTADB_HOST=0.0.0.0
.\vanta-server-windows-amd64.exe
```

> [!WARNING]
> **Windows SmartScreen Note (Unsigned Binary):** 
> When launching the binary, Windows SmartScreen may show an "Unrecognized Publisher" warning. This is expected because the current Windows release binaries are not yet digitally signed. Ensure you only execute binaries downloaded from the official GitHub repository.

## Documentation

The following documents track the current repo truth:

1. **[Architecture](docs/architecture/ARCHITECTURE.md)**: Current embedded core, durability, retrieval, and SDK boundary.
2. **[Operations & Configuration](docs/operations/CONFIGURATION.md)**: Runtime parameters and server wrapper configuration.
3. **[Memory Telemetry Contract](docs/operations/MEMORY_TELEMETRY.md)**: What process-memory metrics mean and what they do not mean.
4. **[Python SDK Status](docs/operations/PYTHON_SDK.md)**: Stable boundary, current binding surface, and deferred distribution work.
5. **[Reliability Gate](docs/operations/RELIABILITY_GATE.md)**: Allowed claims, trusted metrics, and the next cycle cut.
6. **[Memory MVP Baseline](docs/operations/MEMORY_MVP_BASELINE.md)**: Implemented memory surface, derived state, validation evidence, and explicit limits.

## 📄 License

VantaDB is licensed under the **Apache 2.0 License**. See `LICENSE` for details.

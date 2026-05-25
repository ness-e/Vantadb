<div align="center">
  <h1>VantaDB</h1>
  <p><b>Embedded Rust engine for durable local memory and hybrid vector retrieval.</b></p>

  [![CI](https://github.com/ness-e/Vantadb/actions/workflows/rust_ci.yml/badge.svg)](https://github.com/ness-e/Vantadb/actions/workflows/rust_ci.yml)
  [![Python Wheels](https://github.com/ness-e/Vantadb/actions/workflows/python_wheels.yml/badge.svg)](https://github.com/ness-e/Vantadb/actions/workflows/python_wheels.yml)
  [![Heavy Certification](https://github.com/ness-e/Vantadb/actions/workflows/heavy_certification.yml/badge.svg)](https://github.com/ness-e/Vantadb/actions/workflows/heavy_certification.yml)
  [![Release](https://img.shields.io/github/v/release/ness-e/Vantadb?label=Release&color=blue)](https://github.com/ness-e/Vantadb/releases)
  [![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)
  [![PyPI](https://img.shields.io/pypi/v/vantadb-py?label=PyPI&color=blue)](https://pypi.org/project/vantadb-py/)
  [![Python](https://img.shields.io/badge/Python-3.8+-blue.svg)](https://pypi.org/project/vantadb-py/)
  [![Rust](https://img.shields.io/badge/Rust-1.80+-orange.svg)](https://www.rust-lang.org/)
</div>

<br>

VantaDB is a local-first, embedded database engine designed for AI agents, local RAG pipelines, and edge applications. It provides persistent storage, crash-safe recovery via WAL, and native hybrid search (BM25 + HNSW) without requiring external services, containers, or network dependencies.

---

## Quick Links

| Need | Start here |
|------|------------|
| Understand the product boundary | [Product Boundary](#-product-boundary) |
| Try the MVP in five minutes | [5-Minute Quickstart](docs/QUICKSTART.md) |
| Install via pip | [Installation](#-installation) |
| Use the embedded CLI | [CLI Reference](#-embedded-cli) |
| Run as a local server | [Server Mode](#-optional-server-mode) |
| Read architecture docs | [Documentation](#-documentation) |
| Contribute safely | [CONTRIBUTING.md](CONTRIBUTING.md) |
| Report a vulnerability | [SECURITY.md](SECURITY.md) |
| Get support | [SUPPORT.md](SUPPORT.md) |

---

## 📦 Installation

VantaDB is distributed as a native Python package with pre-compiled wheels for Windows, macOS, and Linux.

```bash
pip install vantadb-py
```

> **Note:** The distribution name is `vantadb-py`, but the importable module uses an underscore due to Python naming conventions: `import vantadb_py`.

For development from source:
```bash
pip install -e ./vantadb-python
```

For Rust-native integration, add the crate to your `Cargo.toml`:
```toml
[dependencies]
vantadb = "0.1"
```

---

## 🚀 5-Minute Quickstart

Initialize a persistent memory store, save structured records with vectors, and execute hybrid retrieval in pure Python:

```python
import vantadb_py as vantadb

# 1. Open or create a local database (zero configuration)
db = vantadb.VantaDB("./vanta_data", memory_limit_bytes=512_000_000)

# 2. Store a memory record with payload, metadata, and embedding
record = db.put(
    "agent/main",
    "memory-001",
    "In-process execution minimizes latency for local AI agents.",
    metadata={"category": "architecture", "priority": 1},
    vector=[0.12, 0.88, 0.54],
)

# 3. Retrieve exact record by key
stored = db.get_memory("agent/main", "memory-001")

# 4. Hybrid Search (BM25 + Cosine Similarity fused via RRF)
hits = db.search_memory("agent/main", vector=[0.11, 0.89, 0.55], top_k=5)

# 5. Operational Telemetry & Safe Shutdown
caps = db.hardware_profile()
db.flush()
db.close()

print(record)
print(stored)
print(hits)
print(caps)
```

---

## 🔑 Core Capabilities

| Engine | Mechanism | Details |
|--------|-----------|---------|
| **Persistent Core** | `StorageBackend` + VantaFile + WAL | Fjall (default) or RocksDB fallback. Automatic crash recovery via Write-Ahead Log with CRC32C checksums. |
| **Hybrid Search** | BM25 + HNSW via RRF | Fuses lexical scoring and vector similarity using Reciprocal Rank Fusion. Automatically routed via query planner. |
| **Vector Retrieval** | Native HNSW | Cosine similarity with configurable `M`, `ef_construction`, and `ef_search`. Validated on 10K–100K synthetic datasets. |
| **Memory API** | `namespace + key` records | `put/get/delete/list/search` store UTF-8 payloads, scalar metadata, optional vectors, timestamps, versions, and deterministic node IDs. |
| **Structured Indexes** | Derived prefix-scan indexes | Equality filters use persisted metadata indexes that can be rebuilt from canonical records. |
| **Graph Edges** | Local adjacency lists | Directed edges with optional weights stored in the internal node model. Not a graph database claim. |
| **Operational Flows** | Rebuild + JSONL + Metrics | ANN rebuild, memory export/import, text-index repair, stale derived-index repair, and process telemetry exposed through the SDK boundary. |
| **Embedded Surface** | Rust Core + PyO3 Bindings | Zero-network overhead. Python bindings route through a stable `src/sdk.rs` boundary. |

No separate cluster, daemon, or external service is required. VantaDB runs in-process.

---

## Search Semantics

- The shipped ANN path uses **cosine similarity**.
- Namespace-scoped `list/search` use derived namespace and scalar metadata indexes, with canonical records remaining the source of truth.
- **Hybrid Search** is supported natively. The engine plans and executes lexical (BM25) and vector (Cosine) queries, fusing them using Reciprocal Rank Fusion (RRF).
- SIFT-1M remains useful as a stress/recovery scenario via the [Heavy Certification](https://github.com/ness-e/Vantadb/actions/workflows/heavy_certification.yml) workflow.

---

## 🛡️ Product Boundary

VantaDB should be understood as: embedded-first, local-first, durable memory with WAL-backed recovery, cosine-based HNSW vector retrieval, and an optional local server wrapper.

> **MVP = embedded memory + WAL + vector/BM25/hybrid + export/import + CLI/Python**

| Classification | Surface |
|---|---|
| **Production-facing** | Embedded SDK/CLI, memory CRUD/search, WAL/recovery, namespaces, metadata indexes, HNSW vector retrieval, BM25, Hybrid Retrieval v1, phrase filtering, rebuild/audit/repair, JSONL export/import |
| **Optional wrapper** | Local `vanta-server` around the embedded core |
| **Experimental / not MVP** | IQL/LISP/DQL, MCP, LLM/Ollama integration, governance and maintenance semantics, graph traversal beyond stored local edges |
| **Deferred** | Cloud/enterprise platform, HA/replication, distributed clustering, SQL/OLTP/warehouse/time-series, advanced ranking/snippets/tokenization, RBAC, multi-tenancy |

*VantaDB is an embedded memory engine, not a universal multimodel database or cloud platform.*

See [Experimental Features and Product Boundary](docs/operations/EXPERIMENTAL_FEATURES.md) for the operational classification of all repository surfaces.

---

## 💻 Embedded CLI

For local development, debugging, or pipeline automation without Python.

### 📥 One-Line Installation

Select the quickest method for your environment:

#### 1. Precompiled Binary (Recommended)

Download and install the CLI binary instantly in a single command without compiling:

- **Linux / macOS / WSL**:

  ```bash
  curl -fsSL https://raw.githubusercontent.com/ness-e/Vantadb/main/scripts/install.sh | sh
  ```

- **Windows (PowerShell)**:

  ```powershell
  irm https://raw.githubusercontent.com/ness-e/Vantadb/main/scripts/install.ps1 | iex
  ```

#### 2. Via Cargo (Rust Developers)

Installs and registers `vanta-cli` directly into your Cargo binary directory:

```bash
cargo install --git https://github.com/ness-e/Vantadb.git --bin vanta-cli
```

---

### 🚀 Usage Guide

Once installed and added to your `PATH`, use the global `vanta-cli` command:

```bash
vanta-cli put --db ./vanta_data --namespace agent/main --key mem-1 --payload "hello"
vanta-cli list --db ./vanta_data --namespace agent/main
vanta-cli export --db ./vanta_data --namespace agent/main --out ./memory.jsonl
vanta-cli rebuild-index --db ./vanta_data
vanta-cli audit-index --db ./vanta_data --namespace agent/main --json --deep
vanta-cli repair-text-index --db ./vanta_data
```

*(If you are developing locally inside this repository, you can also run directly from source using `cargo run --bin vanta-cli -- <command>`).*

---

## 🖥️ Optional Server Mode

For local development or network exposure without Python, you can run the standalone binary. This wraps the embedded core; it is not the primary product identity.

1. Download `vantadb-server-*` for your platform from [GitHub Releases](https://github.com/ness-e/Vantadb/releases).
2. Run the binary:

   ```bash
   ./vantadb-server-linux-amd64
   ```

**Defaults:**

- **Data Directory**: Creates a `vantadb_data` folder in the current execution directory.
- **Bind Address**: Listens on `127.0.0.1:8080` (safe localhost default).

**Exposing to the Network:** Override the host via environment variable:

```bash
export VANTADB_HOST=0.0.0.0
./vantadb-server-linux-amd64
```

> [!WARNING]
> **Windows SmartScreen Note (Unsigned Binary):** When launching the Windows binary, SmartScreen may show an "Unrecognized Publisher" warning. This is expected because the current release binaries are not yet digitally signed. Only execute binaries downloaded from the official [GitHub Releases](https://github.com/ness-e/Vantadb/releases).

---

## 📚 Documentation

| Resource | Description |
|----------|-------------|
| [Architecture](docs/architecture/ARCHITECTURE.md) | Core engine, durability model, retrieval mechanisms, and SDK boundary. |
| [Mutation & Recovery Protocol](docs/architecture/MUTATION_RECOVERY_PROTOCOL.md) | Canonical mutation order and WAL recovery behavior. |
| [Text Index Design](docs/architecture/TEXT_INDEX_DESIGN.md) | BM25, phrase positions, derived index repair, and Hybrid Retrieval v1 boundaries. |
| [Operations & Configuration](docs/operations/CONFIGURATION.md) | Runtime parameters and server wrapper configuration. |
| [Memory Telemetry](docs/operations/MEMORY_TELEMETRY.md) | Process-memory metrics contract and interpretation guidelines. |
| [Python SDK Status](docs/operations/PYTHON_SDK.md) | Stable boundary, current binding surface, and distribution policy. |
| [Python Release Policy](docs/operations/PYTHON_RELEASE_POLICY.md) | TestPyPI, production publishing, signing, release assets, and rollback. |
| [Reliability Gate](docs/operations/RELIABILITY_GATE.md) | Allowed claims, trusted metrics, and validation evidence. |
| [Memory MVP Baseline](docs/operations/MEMORY_MVP_BASELINE.md) | Implemented memory surface, derived state, and explicit limits. |
| [Experimental Features](docs/operations/EXPERIMENTAL_FEATURES.md) | Production, optional, experimental, and deferred surface classification. |
| [CI Policy](docs/operations/CI_POLICY.md) | Continuous integration strategy, profiles, and certification gates. |
| [Benchmarks](docs/BENCHMARKS.md) | Performance benchmark methodology and results. |
| [Changelog](docs/CHANGELOG.md) | Version history and release notes. |

---

## 🤝 Contributing & Security

- Contributions must follow [CONTRIBUTING.md](CONTRIBUTING.md).
- Security reports must follow [SECURITY.md](SECURITY.md).
- Support and triage channels are described in [SUPPORT.md](SUPPORT.md).

---

## 📜 License

VantaDB is licensed under the **Apache 2.0 License**. See [LICENSE](LICENSE) for details.

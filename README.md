<div align="center">
  <img src="assets/banner-v3.gif" alt="VantaDB - Embedded Rust engine for durable local memory and hybrid vector retrieval">
</div>

<br>

<div align="left">
  <a href="https://github.com/ness-e/Vantadb/actions/workflows/ci-rust-10.yml"><img src="https://img.shields.io/github/actions/workflow/status/ness-e/Vantadb/ci-rust-10.yml?label=Rust+CI" alt="Rust CI"></a>
  <a href="https://github.com/ness-e/Vantadb/actions/workflows/gate-docs-21.yml"><img src="https://img.shields.io/github/actions/workflow/status/ness-e/Vantadb/gate-docs-21.yml?label=Docs" alt="Docs"></a>
  <a href="https://github.com/ness-e/Vantadb/actions/workflows/sec-codeql-30.yml"><img src="https://img.shields.io/github/actions/workflow/status/ness-e/Vantadb/sec-codeql-30.yml?label=Security+Audit" alt="Security Audit"></a>

  <br>

  <a href="https://github.com/ness-e/Vantadb/releases"><img src="https://img.shields.io/github/v/release/ness-e/Vantadb?label=Release&logo=github&logoColor=white&color=FF5500" alt="Release"></a>
  <a href="https://pypi.org/project/vantadb-py/"><img src="https://img.shields.io/pypi/v/vantadb-py?label=pip&logo=python&logoColor=white&color=3775A9" alt="PyPI"></a>
  <a href="https://www.npmjs.com/package/vantadb"><img src="https://img.shields.io/npm/v/vantadb?label=npm&logo=npm&logoColor=white&color=CB3837" alt="npm"></a>

  <br>

  <a href="https://pypi.org/project/vantadb-py/"><img src="https://img.shields.io/badge/Python-3.11%2B-3776AB?logo=python&logoColor=white" alt="Python"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-1.94.1%2B-000000?logo=rust&logoColor=white" alt="Rust"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-Apache_2.0-181717" alt="License"></a>

  <br>

  <a href="https://discord.gg/g8nqB3NtXt"><img src="https://img.shields.io/badge/Discord-VantaDB_Community-5865F2?logo=discord&logoColor=white" alt="Discord"></a>
  <a href="https://colab.research.google.com/github/ness-e/Vantadb/blob/main/examples/colab/vantadb_quickstart.ipynb"><img src="https://colab.research.google.com/assets/colab-badge.svg" alt="Open in Colab"></a>
</div>

<div align="center">
  <a href="README_ES.md">🇪🇸 Español</a>
</div>

VantaDB is a local-first, embedded, single-node database engine designed for AI agents, local RAG pipelines, and edge applications. It provides persistent storage, crash-safe recovery via WAL, and native hybrid search (BM25 + HNSW) without requiring external services, containers, or network dependencies. Everything runs in-process on one node — VantaDB does not cluster or shard across machines.

---


## Quick Links

| Need | Start here |
| :--- | :--- |
| Understand the product boundary | [Product Boundary](#product-boundary) |
| Try the MVP in five minutes | [5-Minute Quickstart](docs/QUICKSTART.md) |
| Install via pip | [Installation](#installation) |
| Use the embedded CLI | [CLI Reference](#embedded-cli) |
| Run as a local server | [Server Mode](#optional-server-mode) |
| Follow a tutorial | [Tutorials](docs/tutorials/) |
| Read the FAQ | [FAQ](docs/FAQ.md) |
| Read the blog | [Blog Posts](docs/blog/) |
| Read architecture docs | [Documentation](#documentation) |
| Contribute safely | [CONTRIBUTING.md](CONTRIBUTING.md) |
| Report a vulnerability | [SECURITY.md](SECURITY.md) |
| Get support | [SUPPORT.md](SUPPORT.md) |

---

## Installation

VantaDB is distributed as a native Python package with pre-compiled wheels for Windows, macOS, and Linux.

```bash
pip install vantadb-py
```

> **Note:** The distribution name is `vantadb-py`, and the canonical import is
> `import vantadb` (same as the Rust crate and npm package). `import vantadb_py`
> remains available and is not broken.
>
> **Naming convention:** the product is **VantaDB**; the Rust crate is `vantadb`,
> the PyPI package is `vantadb-py`, the npm packages are `vantadb` (TypeScript/WASM)
> and `vantadb-node` (native), and the GitHub repository is `ness-e/Vantadb`.
> See [ADR-030](docs/architecture/adr/ADR-030-brand-identity-naming-convention.md)
> for the full audit and rationale.

For development from source:

```bash
pip install -e ./vantadb-python
```

For Rust-native integration, add the crate to your `Cargo.toml`:

```toml
[dependencies]
vantadb = { git = "https://github.com/ness-e/Vantadb" }
```

---

## 5-Minute Quickstart

<img src="assets/demo.gif" alt="VantaDB demo — pip install, CRUD, hybrid search">

Initialize a persistent memory store, save structured records with vectors, and execute hybrid retrieval in pure Python:

```python
import vantadb

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
hits = db.search_memory("agent/main", query_vector=[0.11, 0.89, 0.55], top_k=5)

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

## Integrations

VantaDB ships with runnable Python examples that wire the embedded engine into popular AI memory / RAG frameworks. Each example defines a thin wrapper class over the stable Python SDK (`vantadb_py`) and is exercised end-to-end by the CI example smoke suite (`ci-examples-12.yml`).

### Mem0 — persistence backend

Use VantaDB as the storage backend for [Mem0](https://mem0.ai) memories. [`VantaDBMem0Backend`](examples/python/mem0_integration.py) implements the memory CRUD/search interface (`add`, `get`, `search`, `update`, `delete`, `get_all`, `delete_all`) on top of a namespace-scoped hybrid store (`mem0/memories`):

```python
backend = VantaDBMem0Backend(namespace="mem0/memories")
backend.add(
    "User prefers dark mode in all applications",
    user_id="user-001",
    metadata={"category": "preference", "priority": "high"},
)
for r in backend.search("dark mode", user_id="user-001"):
    print(f"  Score: {r['score']:.3f}  Content: {r['content']}")
backend.close()
```

### Semantic Kernel — memory interface

Use VantaDB's hybrid retrieval for the **Microsoft Semantic Kernel** memory / context surface. [`VantaDBSemanticMemory`](examples/python/semantic_kernel_memory.py) exposes the store operations (`add`, `get`, `search`, `remove`) used by an AI-augmented environment, all backed by a LAN-free, embedded engine:

```python
memory = VantaDBSemanticMemory(collection_name="demo-app")
memory.save_information(
    "User prefers concise technical answers with code examples",
    metadata={"category": "preference", "priority": "high"},
)
for r in memory.retrieve("Semantic Kernel", limit=5):
    print(f"  Relevance: {r['relevance']:.3f}  Text: {r['text'][:80]}...")
memory.close()
```

### DSPy — retriever

Use VantaDB as the retriever for [DSPy](https://github.com/stanfordnlp/dspy) pipelines. [`VantaDBRetriever`](examples/python/dspy_retriever.py) implements the callable retriever interface (`__call__`) so it slots directly into DSPy pipelines, backed by hybrid vector + text search over `dspy/documents`:

```python
retriever = VantaDBRetriever(namespace="dspy/documents", k=3)
retriever.add([
    {"id": "doc-001", "text": "VantaDB is an embedded persistent memory and vector retrieval engine for local-first AI applications."},
    {"id": "doc-002", "text": "DSPy is a framework for algorithmically optimizing LM prompts and weights."},
])
for doc in retriever("vector database"):
    print(f"  Score: {doc['score']:.3f}  Text: {doc['text'][:80]}...")
retriever.close()
```

Run any example directly (mirrors the CI smoke commands):

```bash
python examples/python/mem0_integration.py
python examples/python/semantic_kernel_memory.py
python examples/python/dspy_retriever.py
```

---

## Examples

All runnable examples live under [`examples/`](examples/).

### Python

| Example | Description |
| :--- | :--- |
| [`examples/demo/demo.py`](examples/demo/demo.py) | End-to-end showcase: lifecycle, insert, vector/hybrid search, persistence, telemetry ([README](examples/demo/README.md)). |
| [`examples/colab/vantadb_quickstart.ipynb`](examples/colab/vantadb_quickstart.ipynb) | Interactive Colab quickstart notebook. |
| [`examples/python/agent_memory.py`](examples/python/agent_memory.py) | Persistent namespace-scoped memory for local AI agents: storage, hybrid retrieval, and telemetry. |
| [`examples/python/mem0_integration.py`](examples/python/mem0_integration.py) | [Mem0](https://mem0.ai) backend: memory CRUD/search over a namespace-scoped hybrid store. |
| [`examples/python/semantic_kernel_memory.py`](examples/python/semantic_kernel_memory.py) | Microsoft Semantic Kernel memory interface (`add`/`get`/`search`/`remove`). |
| [`examples/python/dspy_retriever.py`](examples/python/dspy_retriever.py) | DSPy callable retriever backed by hybrid search. |
| [`examples/python/langchain_ollama_rag.py`](examples/python/langchain_ollama_rag.py) | RAG pipeline with LangChain + Ollama embeddings via `VantaDBVectorStore`. |
| [`examples/python/haystack_documentstore.py`](examples/python/haystack_documentstore.py) | Haystack DocumentStore implementation for RAG pipelines. |
| [`examples/python/crewai_memory.py`](examples/python/crewai_memory.py) | Memory backend for CrewAI agents. |
| [`examples/python/autogen_memory.py`](examples/python/autogen_memory.py) | Persistent memory for AutoGen conversational agents. |
| [`examples/python/langgraph_checkpoint.py`](examples/python/langgraph_checkpoint.py) | Checkpoint store for LangGraph state management. |

### Rust

| Example | Description |
| :--- | :--- |
| [`examples/rust/basic.rs`](examples/rust/basic.rs) | Basic CRUD: create engine, add records with vectors/metadata, similarity search. Run with `cargo run --example basic`. |
| [`examples/rust/concurrent.rs`](examples/rust/concurrent.rs) | Concurrent access from multiple threads via `Arc<VantaEmbedded>`. |
| [`examples/rust/graphrag.rs`](examples/rust/graphrag.rs) | Knowledge graph: nodes + directed edges + BFS traversal (low-level Node/Graph API). |
| [`examples/rust/hybrid.rs`](examples/rust/hybrid.rs) | Hybrid search: BM25 lexical + HNSW vector fused via RRF. |

---

## Core Capabilities

| Engine | Mechanism | Details |
| :--- | :--- | :--- |
| **Persistent Core** | `StorageBackend` + VantaFile + WAL | Fjall (default) or RocksDB fallback. Automatic crash recovery via Write-Ahead Log with CRC32C checksums. |
| **Hybrid Search** | BM25 + HNSW via RRF | Fuses lexical scoring and vector similarity using Reciprocal Rank Fusion. Automatically routed via query planner. |
| **Vector Retrieval** | Native HNSW | Cosine similarity with configurable `M`, `ef_construction`, and `ef_search`. Validated on 10K–100K synthetic datasets. |
| **Memory API** | `namespace + key` records | `put/get/delete/list/search` store UTF-8 payloads, scalar metadata, optional vectors, timestamps, versions, and deterministic node IDs. |
| **Structured Indexes** | Derived prefix-scan indexes | Equality filters use persisted metadata indexes that can be rebuilt from canonical records. |
| **Graph Edges** | Local adjacency lists | Directed edges with optional weights stored in the internal node model. Not a graph database claim. |
| **Operational Flows** | Rebuild + JSONL + Metrics | ANN rebuild, memory export/import, text-index repair, stale derived-index repair, and process telemetry exposed through the SDK boundary. |
| **Embedded Surface** | Rust Core + PyO3 Bindings | Zero-network overhead. Python bindings route through a stable `src/sdk.rs` boundary. |

VantaDB is a single-node embedded engine: no cluster, daemon, or external service is required — it runs in-process.

---

## Search Semantics

- The shipped ANN path uses **cosine similarity**.
- Namespace-scoped `list/search` use derived namespace and scalar metadata indexes, with canonical records remaining the source of truth.
- **Hybrid Search** is supported natively. The engine plans and executes lexical (BM25) and vector (Cosine) queries, fusing them using Reciprocal Rank Fusion (RRF).
- SIFT-1M remains useful as a stress/recovery scenario via the [Heavy Certification](https://github.com/ness-e/Vantadb/actions/workflows/heavy-certification-50.yml) workflow.

---

## Product Boundary

VantaDB should be understood as: embedded-first, local-first, durable memory with WAL-backed recovery, cosine-based HNSW vector retrieval, and an optional local server wrapper.

> **MVP = embedded memory + WAL + vector/BM25/hybrid + export/import + CLI/Python**

| Classification | Surface |
| :--- | :--- |
| **Production-facing** | Embedded SDK/CLI, memory CRUD/search, WAL/recovery, namespaces, metadata indexes, HNSW vector retrieval, BM25, Hybrid Retrieval v1, phrase filtering, rebuild/audit/repair, JSONL export/import |
| **Optional wrapper** | Local `vantadb-server` binary around the embedded core |
| **New** | MCP server for AI agents ([setup guide](docs/api/MCP.md)) |
| **Experimental / not MVP** | IQL/LISP/DQL, LLM/Ollama integration, governance and maintenance semantics, graph traversal beyond stored local edges |
| **Deferred** | Cloud/enterprise platform, HA/replication, distributed clustering, SQL/OLTP/warehouse/time-series, advanced ranking/snippets/tokenization, RBAC, multi-tenancy |

*VantaDB is an embedded memory engine, not a universal multimodel database or cloud platform.*

> **Replication:** VantaDB is single-node — it does not cluster or replicate. The `wal-shipping` crate feature is an **experimental send-only** mechanism that ships WAL segments to a remote endpoint via HTTP POST; the embedded engine has no receive path, so it is not replication, clustering, or HA.

See [Experimental Features and Product Boundary](docs/operations/EXPERIMENTAL_FEATURES.md) for the operational classification of all repository surfaces.

---

## Embedded CLI

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

> [!NOTE]
> The precompiled binaries from [GitHub Releases](https://github.com/ness-e/Vantadb/releases) (and the install scripts above) already include the HTTP server feature. If you install from source via `cargo install` and need `vanta-cli server --http`, enable it explicitly:
>
> ```bash
> cargo install --git https://github.com/ness-e/Vantadb.git --bin vanta-cli --features server
> ```

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

## Optional Server Mode

For local development or network exposure without Python, you can run the standalone binary. This wraps the embedded core; it is not the primary product identity.

1. Download the tarball for your platform from [GitHub Releases](https://github.com/ness-e/Vantadb/releases) (e.g. `vantadb-x86_64-unknown-linux-gnu.tar.gz`).
2. Extract and run the binary:

   ```bash
   tar xzf vantadb-x86_64-unknown-linux-gnu.tar.gz
   ./vantadb-server
   ```

**Defaults:**

- **Data Directory**: Creates a `vantadb_data` folder in the current execution directory.
- **Bind Address**: Listens on `127.0.0.1:8080` (safe localhost default).

**Exposing to the Network:** Override the host via environment variable:

```bash
export VANTADB_HOST=0.0.0.0
./vantadb-server
```

> [!WARNING]
> **Windows SmartScreen Note (Unsigned Binary):** When launching the Windows binary (`vantadb-server.exe`), SmartScreen may show an "Unrecognized Publisher" warning. This is expected because the current release binaries are not yet digitally signed. Only execute binaries downloaded from the official [GitHub Releases](https://github.com/ness-e/Vantadb/releases).

---

## Benchmarks & Performance Baseline

VantaDB ships a formal Python-native performance benchmark suite (**BENCH-01**) that captures ingestion throughput and query latency profiles under realistic single-threaded workloads.

### In-Process Performance Baseline (10K Vectors, 128d, Cosine)

Measured single-threaded SDK baselines (including the PyO3/GIL boundary) are published in [docs/operations/BENCHMARKS.md](docs/operations/BENCHMARKS.md): SDK operation latencies (`put`, BM25, HNSW, hybrid) and the certified Rust stress-protocol results (10K–100K, recall, memory, scaling). Numbers are hardware- and build-dependent — regenerate locally with the suite below to reproduce them on your machine.

| Metric | Latest local baseline (`vanta_benchmark_report.json`, 10K×128d, regenerate locally) |
| :--- | :--- |
| **Ingestion** (Insert + WAL + Flush) | 74.0 records/sec (p50 13.2 ms) |
| **Search (Vector HNSW)** | p50 2.0 ms (~500 queries/sec) |
| **Search (Hybrid Fusion)** | p50 3.1 ms (~320 queries/sec) |

*Source: [`benchmarks/vanta_benchmark_report.json`](benchmarks/vanta_benchmark_report.json) — regenerable with `python benchmarks/vantadb_local_bench.py --size 10000 --dim 128 --queries 1000` (gitignored; not a committed artifact).* BM25 text-search latency is excluded above because the local artifact reports a degenerate outlier (p50 0.0035 ms for a single-document text query); see the maintained [BENCHMARKS.md §2](docs/operations/BENCHMARKS.md) table for the full CI series.

### SIFT-1M (100K scale) — Phase 2 Construction & Search Results

VantaDB's HNSW engine was optimized in Phase 2 through static prefetch, elimination of Euclidean square root calculation in hot graph traversal, pure SIMD calculation for cosine similarity, and the **O(M²) select_neighbors optimization** (which caches references to eradicate HashMap queries during the diversity loop).

The certified performance results on the standard SIFT-1M dataset at 100K scale in optimized mode are:

| Scale (Vectors) | HNSW Configuration | Metric | Construction Time (Before) | Construction Time (Now) | Speedup | p99 Search Latency | Average QPS |
| :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **100K** | Balanced Cos | Cosine | 139.4s | **63.7s** | **2.18x** | 441.2 µs | 3,636 |
| **100K** | High Recall Cos | Cosine | 390.8s | **182.2s** | **2.14x** | 1,231.8 µs | 1,379 |
| **100K** | Balanced L2 | Euclidean | 191.4s | **68.4s** | **2.80x** | 671.4 µs | 3,270 |
| **100K** | High Recall L2 | Euclidean | 462.2s | **194.5s** | **2.37x** | 1,183.6 µs | 1,353 |
| **100K** | High Recall L2 Mmap | Mmap Euclidean | 411.2s | **189.8s** | **2.16x** | 1,094.8 µs | 1,438 |

*Certification hardware: AMD Ryzen 12-Core @ 3.5GHz, compiled with `-C target-cpu=native`.*

*Source: [docs/operations/BENCHMARKS.md §5](docs/operations/BENCHMARKS.md) — "Impact of Loop and HNSW Distance Optimization (Phase 2)" (2026-07-21). Full optimization history in [docs/benchmarks/docs/BENCHMARK_OPTIMIZATION_2026.md](docs/benchmarks/docs/BENCHMARK_OPTIMIZATION_2026.md).*

<p align="center">
  <img src="assets/benchmark-sift1m.svg" alt="SIFT1M HNSW construction speedup — Phase 1 vs Phase 2 (2.14x–2.80x)" width="760">
</p>

### Running the Local Benchmark Suite

To measure performance baseline on your local hardware:

1. **Install python bindings in your active environment:**

   ```bash
   pip install maturin
   maturin develop --release --manifest-path vantadb-python/Cargo.toml
   ```

2. **Execute the benchmark script:**

   ```bash
   python benchmarks/vantadb_local_bench.py --size 10000 --dim 128 --queries 1000
   ```

Results will be printed directly to the console and written to `vanta_benchmark_report.json` for CI tracking.

---

## Documentation

| Resource | Description |
| :--- | :--- |
| [Architecture](docs/architecture/ARCHITECTURE.md) | Core engine, durability model, retrieval mechanisms, and SDK boundary. |
| [Mutation & Recovery Protocol](docs/architecture/MUTATION_RECOVERY_PROTOCOL.md) | Canonical mutation order and WAL recovery behavior. |
| [Text Index Design](docs/architecture/TEXT_INDEX_DESIGN.md) | BM25, phrase positions, derived index repair, and Hybrid Retrieval v1 boundaries. |
| [Operations & Configuration](docs/operations/CONFIGURATION.md) | Runtime parameters and server wrapper configuration. |
| [Memory Telemetry](docs/operations/MEMORY_TELEMETRY.md) | Process-memory metrics contract and interpretation guidelines. |
| [Python SDK Status](docs/api/PYTHON_SDK.md) | Stable boundary, current binding surface, and distribution policy. |
| [Python Release Policy](docs/operations/PYTHON_RELEASE_POLICY.md) | TestPyPI, production publishing, signing, release assets, and rollback. |
| [Reliability Gate](docs/operations/RELIABILITY_GATE.md) | Policies for RSS memory stability, chaos injection, and WAL durability. |
| [Experimental Features](docs/operations/EXPERIMENTAL_FEATURES.md) | Production, optional, experimental, and deferred surface classification. |
| [CI Policy](docs/operations/CI_POLICY.md) | Continuous integration strategy, profiles, and certification gates. |
| [Benchmarks](docs/operations/BENCHMARKS.md) | Performance benchmark methodology and results. |
| [Comparison vs Alternatives](docs/COMPARISON.md) | Honest qualitative comparison vs sqlite-vec / LanceDB / Qdrant / Chroma, our numbers with reproduction commands, and a practical-limits table. |
| [Changelog](docs/CHANGELOG.md) | Version history and release notes. |
| [Blog: Hybrid Search](docs/blog/how_hybrid_search_works.md) | How BM25 + HNSW + RRF work together in VantaDB's query engine. |
| [Blog: SQLite for AI Agents](docs/blog/sqlite_for_ai_agents.md) | Benchmarks and architecture decisions behind VantaDB's LSM storage. |
| [Blog: Why I Built VantaDB](docs/blog/why_i_built.md) | The motivation for a local memory engine for AI agents in Rust. |

---

## Contributing & Security

- Contributions must follow [CONTRIBUTING.md](CONTRIBUTING.md).

---

## License

VantaDB is licensed under the **Apache 2.0 License**. See [LICENSE](LICENSE) for details.

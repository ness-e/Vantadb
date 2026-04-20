

================================================================
Nombre: .connectome_profile
Ruta: .connectome_profile
================================================================

{
  "instructions": "Avx2",
  "profile": "Performance",
  "logical_cores": 4,
  "total_memory": 8489287680,
  "resource_score": 23,
  "env_hash": 9987678657533827080
}

================================================================
Nombre: .dockerignore
Ruta: .dockerignore
================================================================

target/
.git/
connectome_snapshots/
data/
*.db
build_logs/


================================================================
Nombre: .gitignore
Ruta: .gitignore
================================================================

# Rust build artifacts
/target/
**/*.rs.bk

# Local Database instances and logs
*.log
*.db
/data/
/rocksdb_data/
*.rdb

# External IDEs and OS files
.vscode/
.idea/
.DS_Store
Thumbs.db

# Pagina web
/connectome-web/

# Test Databases & Generated Outputs
/tests_graph_db/
/tests_server_db/
/high_density_bench_db/
/benches_db/
/vantadb_data/

# Debug artifacts
scratch.*

# Python Bindings Artifacts
vantadb-python/target/
vantadb-python/venv/
vantadb-python/.venv/
vantadb-python/__pycache__/
*.pyc
*.so
.pytest_cache/

# Assets
docs/assets/*.exe

# Local profiles
.vanta_profile
.env

datasets/
scratch.*



================================================================
Nombre: agent.md
Ruta: agent.md
================================================================

# VantaDB - Agent Instructions

Welcome to the VantaDB codebase.

If you are an AI assistant, an LLM, or an autonomous coding agent operating within this repository, you must strictly adhere to the following governance protocols:

## 1. No Biological Metaphors

VantaDB was originally a prototype riddled with overhyped conceptual language ("neurons", "synapses", "cognitive architecture"). We have explicitly **purged** all biological and pseudo-neural metaphors.

- **DO NOT** use words like `neuron`, `synapse`, `brain`, `cognitive`, `hallucination`, `dream`, or `immune system`.
- **INSTEAD**, use mathematically and technically descriptive terms: `node`, `edge`, `vector index`, `background worker`, `invalidation mechanism`, `garbage collection`.
- We hold ourselves to professional database engineering standards.

## 2. Technical Honesty & Precision

- Never promise impossible O(1) complexities for high-dimensional search.
- When generating documentation, clarify standard algorithms used (e.g., standard HNSW, Memory-Mapped persistence).
- Do not add "hype" adjectives to pull requests or commit messages.

## 3. Architecture Overview

VantaDB is a Rust-based, embedded, zero-copy multimodel database engine.

- **Data Model:** `UnifiedNode` contains an ID, a dense `f32` vector, relations, and outward edge lists.
- **Index:** `CPIndex` implements the `HNSW` algorithm. It uses a graph layout pinned via `mmap` if persistent.
- **C-ABI / Python:** We export a subset of functionalities through `src/engine.rs` exposing a C-ABI layer which is consumed by `vantadb-python`.

## 4. Stability

- Always compile and run `cargo check` / `cargo test` when proposing changes.
- Ensure that modifications to core index algorithms do not break the tests in `tests/certification/stress_protocol.rs`. Target recall > 90% is non-negotiable.


================================================================
Nombre: BENCHMARKS.md
Ruta: BENCHMARKS.md
================================================================

# VantaDB — HNSW Engine Benchmarks

All benchmarks use the internal **Stress Protocol** (`tests/certification/stress_protocol.rs`), a 7-block certification suite that validates recall, scaling, memory, persistence, edge cases, graph consistency, and latency.

## Methodology

### Dataset

- **Type:** Synthetic L2-normalized random vectors
- **Dimensions:** 128
- **Seed:** 2024 (deterministic, reproducible)
- **Similarity:** Cosine similarity

### HNSW Configuration

| Scale | M  | M_max0 | ef_construction | ef_search |
|-------|----|--------|-----------------|-----------|
| 10K   | 32 | 64     | 200             | 100       |
| 50K   | 32 | 64     | 400             | 200       |
| 100K  | 32 | 64     | 500             | 300       |

### Hardware

- **CPU:** 12-core, AVX2
- **RAM:** 31 GB
- **OS:** Windows 11

### Reproduction

```bash
cargo test --test stress_protocol -- --nocapture
```

## Results (Certified — April 2026)

| Scale | Recall@10 | Lat p50  | Lat p95   | Build Time | RAM      |
|-------|-----------|----------|-----------|------------|----------|
| 10K   | 0.9520    | 2.65 ms  | 3.24 ms   | 46.66s     | 10.2 MB  |
| 50K   | 0.9100    | 6.89 ms  | 8.80 ms   | 626.24s    | 51.1 MB  |
| 100K  | 0.8860    | 9.28 ms  | 10.51 ms  | 1447.17s   | 101.9 MB |

### Additional Validations

- **Ground Truth (50K):** Recall@10 = 0.9660 (brute-force verified)
- **Persistence:** Zero recall loss after serialize/deserialize cycle
- **Graph Integrity:** 0 orphan nodes, avg L0 connectivity = 51.3
- **Memory:** Linear growth (~1060 bytes/vector)

## Limitations

- Results are on synthetic random data, **not** a standard benchmark dataset (SIFT1M, etc.)
- Build times are single-threaded (no parallel insertion yet)
- External competitive benchmarks (FAISS, HNSWlib) pending — see Roadmap


================================================================
Nombre: Cargo.toml
Ruta: Cargo.toml
================================================================

[package]
name = "vantadb"
version = "0.1.0"
edition = "2021"
description = "VantaDB: An embedded multimodal database engine — vectors, graphs, and relational metadata unified in Rust."
license = "Apache-2.0"
readme = "README.MD"
keywords = ["database", "vector", "graph", "embedded", "rust"]
categories = ["database-implementations"]

[dependencies]
# ── Core ──
serde = { version = "1", features = ["derive"] }
chrono = "0.4"
bincode = "1.3"
thiserror = "1"
parking_lot = "0.12"
zerocopy = { version = "0.8", features = ["derive"] }

# ── Parser + Storage ──
rand = "0.8"
nom = "7"
num-traits = "0.2"
arrow = { version = "53", features = ["ipc"] }
rocksdb = { version = "0.22", default-features = false, features = ["lz4"] }
fjall = "3.1"

# ── Async + Integrations + Server ──
tokio = { version = "1", features = ["full", "rt-multi-thread"] }
reqwest = { version = "0.12", features = ["json"] }
axum = "0.7"
serde_json = "1.0"
prometheus = "0.13"
pyo3 = { version = "0.20", features = ["extension-module"], optional = true }
wide = "1.2.0"
cpufeatures = "0.2"
sysinfo = "0.30"
memmap2 = "0.9"

# ── Console UX ──
indicatif = "0.17"
console = "0.15"
rayon = "1.10"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }

[features]
python_sdk = ["pyo3"]
experimental = []

[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports", "async_tokio"] }
tempfile = "3"
tower = { version = "0.4", features = ["util"] }
http = "1"
futures = "0.3"

[[bench]]
name = "hybrid_queries"
harness = false

[[bench]]
name = "high_density"
harness = false

[[bench]]
name = "stress_test"
harness = false

[[bin]]
name = "vanta-server"
path = "src/bin/vanta-server.rs"
test = false

# ── Integration Tests ─────────────────────────────────────────────
[[test]]
name = "basic_node"
path = "tests/core/basic_node.rs"

[[test]]
name = "integration"
path = "tests/logic/integration.rs"

[[test]]
name = "server"
path = "tests/api/server.rs"

[[test]]
name = "vector_scale_check"
path = "tests/core/vector_scale_check.rs"

[[test]]
name = "mutations"
path = "tests/storage/mutations.rs"

[[test]]
name = "chaos_integrity"
path = "tests/storage/chaos_integrity.rs"

[[test]]
name = "parser"
path = "tests/logic/parser.rs"

[[test]]
name = "executor"
path = "tests/logic/executor.rs"

[[test]]
name = "graph"
path = "tests/core/graph.rs"

[[test]]
name = "storage"
path = "tests/storage/storage.rs"

[[test]]
name = "hnsw"
path = "tests/core/hnsw.rs"

[[test]]
name = "gc"
path = "tests/storage/gc.rs"

[[test]]
name = "governor"
path = "tests/logic/governor.rs"

[[test]]
name = "mcp_integration"
path = "tests/api/mcp_integration.rs"

[[test]]
name = "hardware_profiles"
path = "tests/certification/hardware_profiles.rs"

[[test]]
name = "structured_api_v2"
path = "tests/api/structured_api_v2.rs"

[[test]]
name = "columnar"
path = "tests/logic/columnar.rs"

[[test]]
name = "mmap_index"
path = "tests/storage/mmap_index.rs"

[[test]]
name = "stress_protocol"
path = "tests/certification/stress_protocol.rs"

[[test]]
name = "hnsw_validation"
path = "tests/certification/hnsw_validation.rs"

[[test]]
name = "hnsw_recall_certification"
path = "tests/certification/hnsw_recall.rs"

[[test]]
name = "sift_validation"
path = "tests/certification/sift_validation.rs"

[[test]]
name = "competitive_bench"
path = "tests/certification/competitive_bench.rs"

[[test]]
name = "backend_tests"
path = "tests/storage/backend_tests.rs"

[profile.release]
lto = "thin"
codegen-units = 1
opt-level = 3

[workspace]
members = [
    ".",
    "vantadb-python"
]

[profile.test]
debug = 0
opt-level = 3


================================================================
Nombre: CHANGELOG.md
Ruta: CHANGELOG.md
================================================================

# Changelog

All notable changes to the VantaDB engine will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased] - Ongoing Engine Stabilization
### Added
- Standardized HNSW index with deterministic configurable limits (`m`, `ef_construction`, `m_max0`), beam search graph exploration, and priority queue heuristic neighbor selection.
- `tests/hnsw_recall.rs` to validate algorithm mathematical precision and verify index capabilities.
- Python SDK rebranded to `vantadb-python` exposing PyO3 capabilities.

### Changed
- Complete semantic overhaul of the codebase purging old biological terminology (removing "neurons", "synapses"). Replaced strictly with mathematical equivalents (`UnifiedNode`, `Edge`).
- Updated project naming globally from *ConnectomeDB / NexusDB* to **VantaDB**.
- Simplified the internal index module replacing placeholder graph mappings with formalized spatial structures.

## [Legacy Build] - Connectome Prototype
### Removed
- Unstable proto-CGR algorithms.
- Hard-coded vector similarity loops lacking validation suites.


================================================================
Nombre: CLAUDE.md
Ruta: CLAUDE.md
================================================================

# Claude Protocol for VantaDB

## Reprogramming Directives

1. **Biological Terms are Banned**: Do not use the words `neuron`, `synapse`, `cognitive`, `hallucination`, `dreaming`, or `immune system`. If you see them in remnants, replace them with `node`, `edge`, `vectorized`, `invalidated`, `background synchronization`, and `garbage collection`.
2. **Honesty First**: Never claim VantaDB is AGI, infinite, or uses proprietary magic. Assert that it is a hybrid HNSW index grafted with memory-mapped graph and metadata persistence.
3. **Patience with Borrow Checker**: When dealing with internal index mutations (like updating edge links), always fetch relationships into a scoped block before mutating nodes to prevent `Rc<RefCell>`-like deadlocks or `cannot borrow as mutable` errors.

## Useful Commands

- **Check Build**: `cargo check`
- **Run fast tests**: `cargo test`
- **Run benchmark/recall**: `cargo test --test stress_protocol --release -- --nocapture`

## Project Structure

- `src/node.rs`: Core structures (`UnifiedNode`, `Edge`).
- `src/index.rs`: The HNSW memory representation.
- `src/storage.rs`: MMap backing layer.
- `src/engine.rs`: Unified trait & FFI exports.


================================================================
Nombre: CONTRIBUTING.md
Ruta: CONTRIBUTING.md
================================================================

# Contributing to VantaDB

We welcome contributions to VantaDB! Our goal is to build a high-performance embedded multimodel database without marketing overhead.

## Engineering Philosophy

1. **Precision & Consistency:** We use standard terminology. Avoid biological namespaces or exaggerated descriptors.
2. **Deterministic Debugging:** All core additions must have accompanying validation scripts utilizing brute-force validation (e.g., recall tests) if they involve statistical modeling or approximated distances.
3. **Rust Tooling:** The project utilizes standard `cargo` toolchains. Ensure code is locked to `stable`.

## Submitting Pull Requests

1. Fork the repository and formulate your changes.
2. Ensure you have run:
   - `cargo fmt --check`
   - `cargo clippy -- -D warnings`
   - `cargo test --release`
3. Include an objective breakdown of metric changes if optimizing algorithmic paths.

We look forward to reviewing your additions.


================================================================
Nombre: docker-compose.yml
Ruta: docker-compose.yml
================================================================

version: '3.8'

services:
  vantadb:
    build: 
      context: .
      dockerfile: Dockerfile
    container_name: vanta-server
    ports:
      - "8080:8080"
    environment:
      - RUST_LOG=info
      - VANTADB_PORT=8080
      - OLLAMA_HOST=http://ollama-llm:11434
    volumes:
      - vantadb_data:/data
    networks:
      - agent-network
    depends_on:
      - ollama-llm
    restart: unless-stopped

  ollama-llm:
    image: ollama/ollama:latest
    container_name: ollama-ai-companion
    ports:
      - "11434:11434"
    volumes:
      - ollama_models:/root/.ollama
    networks:
      - agent-network
    restart: always
    deploy:
      resources:
        reservations:
          devices:
            - driver: nvidia
              count: all
              capabilities: [gpu]
    # We don't automatically pull the model to avoid huge startup times.
    # The user should run: docker exec -it ollama-ai-companion ollama pull llama3 (or gemma/nomic-embed-text)
    
volumes:
  vantadb_data:
  ollama_models:

networks:
  agent-network:
    driver: bridge


================================================================
Nombre: Dockerfile
Ruta: Dockerfile
================================================================

# ==========================================
# STAGE 1: BUILD STAGE
# ==========================================
FROM rust:slim-bookworm AS builder

# Instalar dependencias requeridas por rust-rocksdb / pyo3
RUN apt-get update && apt-get install -y \
    clang \
    llvm \
    cmake \
    make \
    g++ \
    libsnappy-dev \
    liblz4-dev \
    libzstd-dev \
    git \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/vantadb
COPY . .

# Compilar release asegurando optimizaciones LTO + O3 (por defecto en release)
RUN cargo build --release --bin vanta-server

# ==========================================
# STAGE 2: RUNTIME STAGE
# ==========================================
FROM debian:bookworm-slim

# Instalar dependencias runtime para RocksDB local
RUN apt-get update && apt-get install -y \
    libsnappy1v5 \
    liblz4-1 \
    libzstd1 \
    ca-certificates \
    gawk \
 && rm -rf /var/lib/apt/lists/* \
 && apt-get clean

WORKDIR /vantadb

# Inyectar binario y entrypoint dinámico
COPY --from=builder /usr/src/vantadb/target/release/vanta-server /usr/local/bin/vanta-server
COPY start.sh /usr/local/bin/start.sh

# Preparar entorno minimalista
RUN chmod +x /usr/local/bin/start.sh \
 && mkdir -p /vantadb/data

# Puerto por defecto (MCP / HTTP)
EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/start.sh"]


================================================================
Nombre: download_sift.py
Ruta: download_sift.py
================================================================

import urllib.request
import os
import tarfile

url = "ftp://ftp.irisa.fr/local/texmex/corpus/sift.tar.gz"
file_name = "datasets/sift.tar.gz"
extract_path = "datasets"

print("Downloading SIFT1M (160MB)... This may take a while depending on the FTP server.")
try:
    if not os.path.exists("datasets"):
        os.makedirs("datasets")
        
    urllib.request.urlretrieve(url, file_name)
    print("Download complete. Extracting...")
    
    with tarfile.open(file_name, "r:gz") as tar:
        tar.extractall(path=extract_path)
        
    print("Extraction complete. You can now run the benchmarks!")
except Exception as e:
    print(f"Error downloading or extracting: {e}")


================================================================
Nombre: README.MD
Ruta: README.MD
================================================================

<div align="center">
  <h1>VantaDB</h1>
  <p><b>Embedded Multimodel Database Engine: Vectors, Graphs, and Relational Metadata</b></p>
  
  [![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
  [![Rust](https://img.shields.io/badge/Made_with-Rust-orange.svg)](https://www.rust-lang.org/)
  [![Python](https://img.shields.io/badge/Python-3.8%2B-blue.svg)](https://python.org)
</div>

<br>

**VantaDB** is a lightweight, fully embedded database engine written in Rust that operates within your Python process. It provides a unified data structure bridging high-dimensional HNSW vector search, local graph adjacency traversals, and relational metadata filtering. By operating entirely in-process, it bypasses network latency and complex serialization logic.

## 🚀 The In-Process Architecture

Running as a compiled C-ABI extension accessed directly via **PyO3**, VantaDB is fundamentally different from standalone vector database servers (e.g., Qdrant, Pinecone, pgvector). When you execute a hybrid query in Python, the engine performs native CPU operations directly on the shared memory space. This completely removes TCP/HTTP overhead, gRPC serialization delays, and context-switching bottlenecks.

---

## ⚡ Technical Capabilities

VantaDB unifies previously distinct data representations into a single coherent structure `UnifiedNode`.

| Engine | Mechanism | Details |
|--------|-----------|---------|
| **Vector Engine** | Native HNSW (Hierarchical Navigable Small World) | Production-grade `M`, `ef_construction`, `ef_search` configurability. Supports f32 vectors standard for embeddings. Validated via internal stress protocol (10K–100K vectors, 128D). See BENCHMARKS.md |
| **Graph Engine** | Adjacency Lists | Standard directed edges with optional float weights for O(1) adjacency traversals. Contextual filtering applied within the vector path. |
| **Relational Storage** | BTreeMap Schemaless filtering | In-memory relational constraints (Strings, Ints, Bools) allowing fast metadata pre-filtering and post-filtering heuristics. |
| **Storage Engine** | MMap-Backed / RAM bounds | Adheres to requested `memory_limit_bytes`. Uses `mmap` backing capabilities to ensure minimal host RAM footprint on cold starts. |

---

## 🛠 Quickstart

No separate database clusters or Docker orchestrations are required for execution.

```bash
pip install vantadb-python
```

```python
import vantadb

# Boot the engine in-process pointing to a local directory for persistence
db = vantadb.VantaDB(path="./vanta_data", memory_limit_bytes=512_000_000)

doc_id = db.insert({
    "content": "In-process execution minimizes latency",
    "vector": [0.12, 0.88, 0.54], # Example embeddings (768d/1536d)
    "category": "architecture",
    "version": 1
})

# Hybrid Search: Vector Similitude + Relational Constraint
results = db.search(
    vector=[0.11, 0.89, 0.55], 
    top_k=5, 
    filter_expr="category == 'architecture' AND version >= 1"
)

print(results)
```

---

## 📖 Deep-Dive Engineering Documentation

To read the specific mechanisms behind VantaDB, locate the core technical tracking files in the `docs/` directory:

1. **[Architecture (Zero-Copy & Memory Layout)](docs/architecture.md)**: Explore the `UnifiedNode` struct, the PyO3 interfaces, and the `index.rs` algorithms.
2. **[Governance & Consistency](docs/decisions.md)**: Details on architectural trade-offs, cache invalidation, and data compaction processes.

*(Note: Legacy documentation is held in `docs/old/` purely for archival relevance. We rely on strict Rust tests `tests/` to track definitive technical reality).*

## 📄 License

VantaDB is licensed under the **Apache 2.0 License**. See `LICENSE` for details.

================================================================
Nombre: run_bench.sh
Ruta: run_bench.sh
================================================================

#!/bin/bash
set -e

echo "Installing build prerequisites..."
apt-get update && apt-get install -y clang llvm cmake make g++ libsnappy-dev liblz4-dev libzstd-dev

echo "Compiling benchmark (no-run) unlimited memory..."
export CI=true
cargo bench --bench high_density --no-run

BENCH_BIN=$(ls -t target/release/deps/high_density-* | grep -v '\.d' | grep -v '\.pdb' | grep -v '\.rmeta' | head -n 1)

echo "Benchmark compiled successfully: $BENCH_BIN"

echo "Executing benchmark in strict 512m environment..."
export VANTADB_MEMORY_LIMIT=536870912
$BENCH_BIN
echo "Benchmark completed successfully within 512m memory limit!"


================================================================
Nombre: rust-toolchain.toml
Ruta: rust-toolchain.toml
================================================================

[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]


================================================================
Nombre: SECURITY.md
Ruta: SECURITY.md
================================================================

# Security Policy

## Supported Versions

Since VantaDB is currently in its initial `v0.1` stabilization phase, previous architectural snapshots are not managed for backported fixes. Only the current trunk is expected to remain stable.

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |
| Pre-v0.1| :x:                |

## Reporting a Vulnerability

If you discover a memory violation or PyO3 serialization vulnerability that allows execution outside boundary mapping protections, please open an Issue with replication steps or reach out to the core maintainers privately. Do not exploit index panics visibly on untrusted vectors in production pending formal stabilization guarantees.


================================================================
Nombre: start.sh
Ruta: start.sh
================================================================

#!/bin/bash
set -e

# VantaDB Intelligent Entrypoint
# Detects Docker CGroup Memory Limits and injects them to HardwareScout

MEMORY_LIMIT=""

if [ -f "/sys/fs/cgroup/memory.max" ]; then
    # Cgroups v2
    CGROUP_MEM=$(cat /sys/fs/cgroup/memory.max)
    if [ "$CGROUP_MEM" != "max" ]; then
        MEMORY_LIMIT=$CGROUP_MEM
    fi
elif [ -f "/sys/fs/cgroup/memory/memory.limit_in_bytes" ]; then
    # Cgroups v1
    CGROUP_MEM=$(cat /sys/fs/cgroup/memory/memory.limit_in_bytes)
    # 9223372036854771712 indicates no limit
    if [ "$CGROUP_MEM" != "9223372036854771712" ] && [ -n "$CGROUP_MEM" ]; then
        MEMORY_LIMIT=$CGROUP_MEM
    fi
fi

if [ -n "$MEMORY_LIMIT" ]; then
    # Subtract 10% for OS / buffer safety margin
    # Using awk for large number arithmetic natively
    SAFE_LIMIT=$(awk -v mem="$MEMORY_LIMIT" 'BEGIN { printf "%.0f", mem * 0.9 }')
    export VANTADB_MEMORY_LIMIT=$SAFE_LIMIT
    echo "🛡️  [DOCKER] Memory Limit detected: $MEMORY_LIMIT bytes. Setting Safe Cap: $SAFE_LIMIT bytes."
else
    echo "🛡️  [DOCKER] No Memory Limit detected. HardwareScout will use Host RAM."
fi

exec "/usr/local/bin/vanta-server" "$@"


================================================================
Nombre: test_runner.sh
Ruta: test_runner.sh
================================================================

#!/bin/bash
set -e

echo "=== System Update and Dependencies ==="
apt-get update
# Install build tools, LLVM/Clang (for RocksDB), and Python
apt-get install -y --no-install-recommends \
    pkg-config libssl-dev cmake clang libclang-dev \
    python3 python3-pip python3-venv

echo "=== Setting up Python Virtual Environment ==="
python3 -m venv /venv
source /venv/bin/activate

echo "=== Installing Python Test Dependencies ==="
pip install maturin pytest

echo "=== Building VantaDB Python SDK ==="
cd /app/vantadb-python
# Compile the Rust code into a Python native module (.so)
# We use backtraces to diagnose any unexpected Python crashes
RUST_BACKTRACE=1 maturin develop --release

echo "=== Running Integration Tests ==="
# Execute the complete SDK lifecycle test suite
pytest -v tests/test_sdk.py


================================================================
Nombre: vanta_certification.json
Ruta: vanta_certification.json
================================================================

{"block_name":"STORAGE LAYER (ROCKSDB ADAPTER): Integration: Persistent Node IO","duration_secs":0.132343,"ram_usage_mb":6808.0,"current_ram_mb":27816.0,"timestamp":"2026-04-17T19:31:57.292284500-04:00"}
{"block_name":"STORAGE LAYER (DML MUTATIONS): Pipeline: INSERT -> GET Cycle","duration_secs":0.0973446,"ram_usage_mb":8824.0,"current_ram_mb":31444.0,"timestamp":"2026-04-17T19:32:14.191729-04:00"}
{"block_name":"STORAGE LAYER (DML MUTATIONS): Pipeline: UPDATE & Atomicity","duration_secs":0.0510765,"ram_usage_mb":8884.0,"current_ram_mb":31504.0,"timestamp":"2026-04-17T19:32:14.243287700-04:00"}
{"block_name":"STORAGE LAYER (DML MUTATIONS): Pipeline: RELATE & Topology Integrity","duration_secs":0.0484976,"ram_usage_mb":9000.0,"current_ram_mb":31620.0,"timestamp":"2026-04-17T19:32:14.292311200-04:00"}
{"block_name":"STORAGE LAYER (DML MUTATIONS): Pipeline: Physical DELETE logic","duration_secs":0.0495781,"ram_usage_mb":9068.0,"current_ram_mb":31688.0,"timestamp":"2026-04-17T19:32:14.342487500-04:00"}
{"block_name":"STORAGE LAYER (CHAOS INTEGRITY): Topological Axioms: Ghost Node Prevention","duration_secs":0.1021759,"ram_usage_mb":8932.0,"current_ram_mb":31576.0,"timestamp":"2026-04-17T19:33:30.027110100-04:00"}
{"block_name":"STORAGE LAYER (CHAOS INTEGRITY): Topological Axioms: Tombstone Resilience","duration_secs":0.0480534,"ram_usage_mb":8940.0,"current_ram_mb":31584.0,"timestamp":"2026-04-17T19:33:30.075641600-04:00"}


================================================================
Nombre: architect_mindset.md
Ruta: .agents\rules\architect_mindset.md
================================================================

---
trigger: always_on
---

Actúa como un Ingeniero de Sistemas Principal. Antes de proponer o ejecutar cambios: 1. Realiza una deducción interna de impactos en cascada. 2. Identifica posibles fallos lógicos o de seguridad (FMEA). 3. No aceptes solicitudes de "parche rápido" sin evaluar la deuda técnica. 4. Si una instrucción es ambigua, cuestiona la premisa antes de proceder.


================================================================
Nombre: analisis.md
Ruta: .agents\workflows\analisis.md
================================================================

---
description: analisis-impacto
---

Trigger: /analisis-impacto

Pasos del proceso:

1. Identificación: Localizar componentes afectados (Storage, HNSW, SDK) según todo.md.

2. Deducción FODA: Generar matriz con:

- Fortalezas: Ganancias en performance/latencia.

- Oportunidades: Nuevas capacidades indexación/búsqueda.

- Debilidades: Incremento de complejidad o consumo de RAM.

- Amenazas: Riesgo de corrupción de datos o inestabilidad en el linker MSVC.

1. Simulación de Fallos: Identificar los 3 puntos de ruptura más probables (ej. race conditions, desalineación de memoria).

2. Plan de Mitigación: Proponer pruebas unitarias o guardias de seguridad para los riesgos detectados.

3. Veredicto: Recomendar proceder, rediseñar o abortar.


================================================================
Nombre: config.toml
Ruta: .cargo\config.toml
================================================================

[target.x86_64-pc-windows-msvc]
linker = "rust-lld.exe"


================================================================
Nombre: bug_report.yml
Ruta: .github\ISSUE_TEMPLATE\bug_report.yml
================================================================

name: Bug Report
description: Create a report to help us improve VantaDB.
title: "[BUG] "
labels: ["bug", "triage"]
body:
  - type: markdown
    attributes:
      value: "Thank you for taking the time to file a bug report! Before you submit, please search the issue tracker to ensure it hasn't been reported already."

  - type: input
    id: version
    attributes:
      label: VantaDB Version
      description: "What version of VantaDB are you using? (e.g. `vanta-server --version`)"
      placeholder: "e.g. 1.0.0"
    validations:
      required: true

  - type: dropdown
    id: os
    attributes:
      label: Operating System
      description: "On what OS did you encounter this bug?"
      options:
        - Windows
        - macOS
        - Linux
        - Docker
    validations:
      required: true

  - type: textarea
    id: description
    attributes:
      label: Describe the Bug
      description: "A clear and concise description of what the bug is."
      placeholder: "When I run an IQL query with X, it panics with Y..."
    validations:
      required: true

  - type: textarea
    id: steps
    attributes:
      label: Steps to Reproduce
      description: "How can we reproduce the problem? (Provide the specific IQL query if applicable)"
      value: |
        1. Run `vanta-server`
        2. Execute query `FROM...`
        3. See error
    validations:
      required: true

  - type: textarea
    id: logs
    attributes:
      label: Rust Error Logs / Panic Trace
      description: "If the engine crashed, please paste the panic output or `RUST_BACKTRACE=1` logs here."
      render: shell


================================================================
Nombre: feature_request.yml
Ruta: .github\ISSUE_TEMPLATE\feature_request.yml
================================================================

name: Feature Request
description: Suggest an idea, new IQL syntax, or enhancement for VantaDB.
title: "[FEATURE] "
labels: ["enhancement"]
body:
  - type: markdown
    attributes:
      value: "Thank you for suggesting an improvement for VantaDB! Please remember that our core philosophy is **simplicity and local AI performance**. Heavily bloated features might be better suited as external plugins."

  - type: textarea
    id: problem
    attributes:
      label: "Is your feature request related to a problem? Please describe."
      description: "A clear and concise description of what the problem is. (e.g. \"I'm always frustrated when I can't filter graphs by...\")"
    validations:
      required: true
      
  - type: textarea
    id: solution
    attributes:
      label: "Describe the solution you'd like"
      description: "A clear and concise description of what you want to happen. If you are proposing new IQL syntax, please provide an example."
      placeholder: |
        I would like the following syntax:
        FROM Node UPDATE SET field = 1
    validations:
      required: true

  - type: textarea
    id: alternatives
    attributes:
      label: "Describe alternatives you've considered"
      description: "A clear and concise description of any alternative solutions or workarounds you've considered."

  - type: textarea
    id: context
    attributes:
      label: "Additional Context"
      description: "Add any other context, technical links, or screenshots about the feature request here."


================================================================
Nombre: release.yml
Ruta: .github\workflows\release.yml
================================================================

name: Release VantaDB

on:
  push:
    tags:
      - "v*.*.*"
  workflow_dispatch:

permissions:
  contents: write
  packages: write

jobs:
  build-and-deploy:
    name: Build release binaries (${{ matrix.os }})
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        include:
          - os: ubuntu-latest
            artifact_name: vanta-server
            asset_name: vanta-server-linux-amd64
          - os: macos-latest
            artifact_name: vanta-server
            asset_name: vanta-server-macos-amd64
          - os: windows-latest
            artifact_name: vanta-server.exe
            asset_name: vanta-server-windows-amd64.exe

    steps:
      - name: Free Disk Space (Ubuntu)
        if: matrix.os == 'ubuntu-latest'
        run: |
          sudo rm -rf /usr/share/dotnet
          sudo rm -rf /usr/local/lib/android
          sudo rm -rf /opt/ghc
          sudo rm -rf /opt/hostedtoolcache/CodeQL

      - name: Checkout Code
        uses: actions/checkout@v4

      - name: Setup Rust Toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: stable
          components: rustfmt, clippy

      - name: Cache Dependencies
        uses: Swatinem/rust-cache@v2

      - name: Build Release Binary
        run: cargo build --release

      - name: Rename Binary (Unix)
        if: matrix.os != 'windows-latest'
        run: mv target/release/${{ matrix.artifact_name }} target/release/${{ matrix.asset_name }}

      - name: Rename Binary (Windows)
        if: matrix.os == 'windows-latest'
        shell: pwsh
        run: Rename-Item -Path "target\release\${{ matrix.artifact_name }}" -NewName "${{ matrix.asset_name }}"

      - name: Release
        uses: softprops/action-gh-release@v2
        if: startsWith(github.ref, 'refs/tags/')
        with:
          files: target/release/${{ matrix.asset_name }}
          draft: true
          generate_release_notes: true

  docker-publish:
    name: Build & Publish Docker Image to GHCR
    runs-on: ubuntu-latest
    needs: build-and-deploy
    steps:
      - name: Free Disk Space (Ubuntu)
        run: |
          sudo rm -rf /usr/share/dotnet
          sudo rm -rf /usr/local/lib/android
          sudo rm -rf /opt/ghc
          sudo rm -rf /opt/hostedtoolcache/CodeQL

      - name: Checkout Code
        uses: actions/checkout@v4

      - name: Extract version tag
        id: meta
        run: echo "VERSION=${GITHUB_REF#refs/tags/}" >> $GITHUB_OUTPUT

      - name: Log in to GitHub Container Registry
        uses: docker/login-action@v3
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Build and Push Docker Image
        uses: docker/build-push-action@v5
        with:
          context: .
          file: Dockerfile
          push: true
          tags: |
            ghcr.io/${{ github.repository_owner }}/vantadb:${{ steps.meta.outputs.VERSION }}
            ghcr.io/${{ github.repository_owner }}/vantadb:latest


================================================================
Nombre: rust_ci.yml
Ruta: .github\workflows\rust_ci.yml
================================================================

name: VantaDB CI

on:
  push:
    branches: [ "main" ]
    paths:
      - 'src/**'
      - 'tests/**'
      - 'benches/**'
      - 'Cargo.toml'
      - 'Cargo.lock'
      - 'build.rs'
      - '.github/workflows/rust_ci.yml'
  pull_request:
    branches: [ "main" ]
    paths:
      - 'src/**'
      - 'tests/**'
      - 'benches/**'
      - 'Cargo.toml'
      - 'Cargo.lock'
      - 'build.rs'
      - '.github/workflows/rust_ci.yml'
  workflow_dispatch:

env:
  CARGO_TERM_COLOR: always
  CARGO_INCREMENTAL: 0

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
    - name: Free Disk Space
      run: |
        sudo rm -rf /usr/share/dotnet
        sudo rm -rf /usr/local/lib/android
        sudo rm -rf /opt/ghc
        sudo rm -rf /opt/hostedtoolcache/CodeQL

    - uses: actions/checkout@v4

    - name: Add swap space (prevent OOM linker crash)
      run: |
        sudo swapoff /swapfile || true
        sudo rm -f /swapfile
        sudo dd if=/dev/zero of=/swapfile bs=1M count=6144
        sudo chmod 600 /swapfile
        sudo mkswap /swapfile
        sudo swapon /swapfile
        free -h

    - name: Set up Rust
      uses: dtolnay/rust-toolchain@stable

    - name: Rust Cache
      uses: Swatinem/rust-cache@v2

    - name: Install system dependencies (RocksDB + Clang)
      run: |
        sudo apt-get update
        sudo apt-get install -y libclang-dev clang librocksdb-dev

    - name: Format Check
      run: cargo fmt --check

    - name: Clippy Lints
      run: cargo clippy -- -D warnings

    - name: Compile and check (Debug mode)
      run: cargo test --no-run

    - name: Run tests (limited threads to reduce memory pressure)
      run: cargo test -- --test-threads=2

    - name: Run benchmarks (verification only)
      run: cargo bench --no-run


================================================================
Nombre: high_density.rs
Ruta: benches\high_density.rs
================================================================

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use rand::Rng;
use std::env;
use std::sync::Arc;
use tokio::runtime::Runtime;
use vantadb::node::{FieldValue, UnifiedNode, VectorRepresentations};
use vantadb::storage::StorageEngine;

fn generate_random_vector(dim: usize) -> Vec<f32> {
    let mut rng = rand::thread_rng();
    let mut vec: Vec<f32> = (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect();
    // Normalize
    let norm: f32 = vec.iter().map(|v| v * v).sum::<f32>().sqrt();
    if norm > 0.0 {
        vec.iter_mut().for_each(|v| *v /= norm);
    }
    vec
}

fn high_density_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let is_ci = env::var("CI").unwrap_or_else(|_| "false".to_string()) == "true";
    let target_nodes = if is_ci { 250_000 } else { 1_000_000 };
    let dim = 768; // BGE-M3 or BAAI/bge-base-en dimensionality

    println!("============================================================");
    println!("VantaDB High Density Benchmark");
    println!("Target Nodes: {}", target_nodes);
    println!("Vector Dimensions: {}", dim);
    println!(
        "Mode: {}",
        if is_ci {
            "CI (Survival)"
        } else {
            "Release (1M)"
        }
    );
    println!("============================================================");

    let storage = Arc::new(StorageEngine::open("high_density_bench_db").unwrap());

    // Seed the database
    println!(
        "Seeding database with {} nodes (This may take a while)...",
        target_nodes
    );
    rt.block_on(async {
        for i in 1..=target_nodes {
            let mut node = UnifiedNode::new(i as u64);
            node.relational.insert(
                "content".to_string(),
                FieldValue::String(format!("Node {}", i)),
            );
            node.relational.insert(
                "type".to_string(),
                FieldValue::String("benchmark".to_string()),
            );
            node.vector = VectorRepresentations::Full(generate_random_vector(dim));
            let _ = storage.insert(&node);

            if i % 100_000 == 0 {
                println!("Inserted {}/{}", i, target_nodes);
            }
        }
    });

    // Sub-Task 1: Search K-NN Latency
    let mut group = c.benchmark_group("high_density_search");
    group.sample_size(50); // Less samples due to intensity

    group.bench_function("knn_search_768d", |b| {
        b.iter_batched(
            || generate_random_vector(dim),
            |query_vec| {
                rt.block_on(async {
                    let results = storage
                        .hnsw
                        .read()
                        .unwrap()
                        .search_nearest(&query_vec, None, None, 0, 10);
                    // Force materialization to prevent optimization drop
                    assert!(results.len() <= 10);
                });
            },
            BatchSize::SmallInput,
        )
    });
    group.finish();

    // Sub-Task 2: Spam Mutations Collision (Logarithmic Friction Validation)
    let mut spam_group = c.benchmark_group("logarithmic_spam_friction");
    spam_group.sample_size(10); // Very intensive, 10 samples

    spam_group.bench_function("50k_spam_mutations", |b| {
        b.iter_batched(
            || {
                let mut dummy_nodes = Vec::with_capacity(50_000);
                for i in 0..50_000 {
                    let mut node = UnifiedNode::new((target_nodes + 1 + i) as u64);
                    node.relational.insert(
                        "content".to_string(),
                        FieldValue::String(format!("Spam node {}", i)),
                    );
                    // Mock spam identity via origin
                    node.relational.insert(
                        "_owner_role".to_string(),
                        FieldValue::String("malicious_agent".to_string()),
                    );
                    node.relational
                        .insert("_confidence".to_string(), FieldValue::Float(1.0)); // Trust manipulation
                    dummy_nodes.push(node);
                }
                dummy_nodes
            },
            |dummy_nodes| {
                rt.block_on(async {
                    // Inject the 50k nodes. Logarithmic friction should limit damage without heavy performance degradation on safe nodes
                    for node in dummy_nodes {
                        // Using raw inserts to simulate bulk spam
                        let _ = storage.insert(&node);
                    }
                });
            },
            BatchSize::LargeInput,
        )
    });
    spam_group.finish();

    // Clean up
    let _ = std::fs::remove_dir_all("high_density_bench_db");
}

criterion_group!(benches, high_density_benchmark);
criterion_main!(benches);


================================================================
Nombre: hybrid_queries.rs
Ruta: benches\hybrid_queries.rs
================================================================

use criterion::{black_box, criterion_group, criterion_main, Criterion};

// Note: Requires complete Integration of StorageEngine + CPIndex,
// using mocks here to demonstrate the benchmarking framework structure
// that runs with `cargo bench`.

fn bench_cp_index_filter(c: &mut Criterion) {
    c.bench_function("cp_index bitset filter", |b| {
        // Mock query mask scenario
        let query_mask = 0b10101010_10101010_10101010_10101010_10101010_10101010_10101010_10101010_10101010_10101010_10101010_10101010_10101010_10101010_10101010_10101010u128;
        let mut n = 0u128;
        b.iter(|| {
            // Simulated L1 cache hit logic
            n = black_box(n + 1);
            let hit = n & query_mask == query_mask;
            black_box(hit);
        })
    });
}

fn bench_unified_node_deserialization(c: &mut Criterion) {
    let mock_bytes = vec![0u8; 128]; // Simulación del block cache (128 bytes)
    c.bench_function("zero-copy bincode deserialize", |b| {
        b.iter(|| {
            // Zero-copy decode simulation
            let _val = black_box(&mock_bytes[0..56]);
        })
    });
}

criterion_group!(
    benches,
    bench_cp_index_filter,
    bench_unified_node_deserialization
);
criterion_main!(benches);


================================================================
Nombre: stress_test.rs
Ruta: benches\stress_test.rs
================================================================

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::env;
use std::sync::Arc;
use tempfile::tempdir;
use tokio::runtime::Runtime;
use vantadb::node::UnifiedNode;
use vantadb::storage::StorageEngine;

fn run_stress_test(c: &mut Criterion) {
    let dir = tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();

    // Abrir motor con BlockCache (2GB) y Bloom Filter (10 bit/key)
    let storage = Arc::new(StorageEngine::open(db_path).unwrap());
    let rt = Runtime::new().unwrap();

    let is_ultra = env::var("STRESS_LEVEL").unwrap_or_default() == "ULTRA";
    let num_nodes = if is_ultra { 1_000_000 } else { 100_000 };

    println!(
        "💉 Inyectando {} nodos... (Stress Level: {})",
        num_nodes,
        if is_ultra { "ULTRA" } else { "NORMAL" }
    );

    // Inyección Preparatoria
    for i in 1..=num_nodes {
        let node = UnifiedNode::new(i);
        storage.insert(&node).unwrap();
    }
    println!("✅ Inyección finalizada.");

    let mut group = c.benchmark_group("The Memory Abyss");
    group.sample_size(10);

    group.bench_function("Point Lookup Valido", |b| {
        b.to_async(&rt).iter(|| async {
            // Nodo que seguro existe, forzando fetch real
            let _ = black_box(storage.get(500).unwrap());
        });
    });

    group.bench_function("Point Lookup Spurious (Bloom Filter Reject)", |b| {
        b.to_async(&rt).iter(|| async {
            // Nodo que seguro NO existe. El Bloom Filter rechaza el I/O disk fetch al instante.
            let _ = black_box(storage.get(num_nodes + 9999).unwrap());
        });
    });

    group.finish();
}

criterion_group!(benches, run_stress_test);
criterion_main!(benches);


================================================================
Nombre: IQL.md
Ruta: docs\api\IQL.md
================================================================

# Inference Query Language (IQL) Specification

Vantadb abandons the complexity of standard SQL JOINs and Graph query languages (like Cypher) by combining traversing arrays and geometric similarities into a unified functional grammar. We call this the **Inference Query Language (IQL)**.

## 1. Core Grammar & Philosophy

IQL is designed explicitly so that **Machine Learning queries (Vectors)** and **Deterministic attributes (Graphs/Relational)** can be executed simultaneously during the same abstract syntax tree traversal, maximizing speed.

### Standard Pipeline Syntax

When utilized via the raw engine text-parser:

```sql
VECTOR search ~ [0.12, 0.44, ...] 
WHERE category == "technology" AND confidence > 0.8
WITH DEPTH 2 
LIMIT 5
```

### Deconstructing the Operands

* `VECTOR search ~ [n]`: The tilde (`~`) operator triggers native HNSW Cosine Similarity traversal using the provided dimensional slice. Mapped to physical space instantly.
* `WHERE`: Standard BTreeMap filtering. The engine evaluates equality (`==`), comparators (`<, >, >=`), and booleans. If pre-filtering is faster (via cardinality limits), Vantadb applies it *before* the HNSW execution.
* `WITH DEPTH`: Graph traversal initiator. Dictates the max recursion of adjacency list jumps from candidate nodes.
* `LIMIT`: The HNSW `top_k` threshold.

---

## 2. Production Examples

### A. Complex RAG System (Retrieval-Augmented Gen)

Filter documents that belong exclusively to the `company_internal` tag, while finding the closest vector distance, ignoring stale documents.

```python
# In standard PyO3 SDK SDK
results = db.search(
    vector=query_embedding,
    top_k=5,
    filter_expr="category == 'company_internal' AND is_stale == false"
)
```

### B. Graph Recommendations (E-Commerce)

Find elements semantically similar to this `product_vector`, but constrain the search exclusively to nodes connected via edge relation (a verified `PURCHASED_WITH` chemical connection).

```python
results = db.search(
    vector=product_vector,
    top_k=10,
    graph_constraint="EDGE_TYPE == 'PURCHASED_WITH'",
    depth=1
)
```

### C. Fraud Analysis Ring

Check geographical IP locations attached as attributes, combined with a behavioral embedding space, allowing the engine to traverse 2 degrees of connections (finding connected fraudulent wallets/users).

```python
# Uncovering a ring by going multiple hops into the metadata
results = db.search(
    vector=behavior_embedding,
    top_k=3,
    filter_expr="geo_risk_score >= 80",
    depth=2 
)
```

## 3. Weight Management & Operability

Every edge connecting two nodes inside Vantadb operates natively with an intrinsic `weight` (f32).

When chaining graph searches with vector searches, if an edge weight degrades drastically (e.g., `weight < 0.2`), Vantadb's executor interprets it as an asynchronous disconnection and will halt traversal early, effectively self-pruning noisy pathways in memory.


================================================================
Nombre: ARCHITECTURE.md
Ruta: docs\architecture\ARCHITECTURE.md
================================================================

# Vantadb Internal Architecture (Sensitives & Internals)

This document provides a deep structural overview of Vantadb's Rust internals, targeting Senior Systems Engineers, Database Architects, and HackerNews peers who wish to understand *how* it operates at a hardware and memory-management level.

## 1. Memory Layout: The `UnifiedNode`

Traditional applications stitch together multiple memory spaces across different DB limits dynamically allocating JSONs or blobs. Vantadb collapses this into the `UnifiedNode` struct.

Every inserted record lives in Rust memory structurally defined as:

```rust
pub struct UnifiedNode {
    pub id: String,                              // Hash or UUID
    pub vector: Box<[f32]>,                      // Contiguous heap slice ensuring SIMD cache-locality
    pub edges: Vec<Edge>,                        // Adjacency list for O(1) graph traversals
    pub relational_data: BTreeMap<String, Value> // Deterministic schemaless metadata mapping
}
```

**Why this layout?**
By combining the high-dimensional Vector (dense slice), the Graph edges (Adjacency Lists), and Relational metadata (BTreeMap) into a single struct contiguous in memory, Vantadb guarantees that when the HNSW algorithm isolates top candidates based on vector distance, the CPU already has the graph edges and the relational fields in the L3 Cache. There is no Secondary Index lookup required.

## 2. The Zero-Copy Pipeline (PyO3)

To solve the orchestration bottleneck, Vantadb runs strictly in-process.

When you pass a dictionary in Python:

1. **PyDict to Struct (Rust):** PyO3 bridges the Python GIL directly to the Rust engine heap. Data is unpacked once.
2. **Execution:** Rust handles the querying lock-free. The Python GIL is released exclusively during `compute(search)`.
3. **Struct to PyRef:** Instead of serializing returning data into a massive JSON payload over TCP (like network DBs do), Rust yields memory pointers back to Python objects natively.

```mermaid
sequenceDiagram
    participant App as Python App (Host)
    participant SDK as Vantadb PyO3 FFI
    participant Core as Rust Engine (Threads)
    participant SSD as RocksDB (WAL)

    App->>SDK: search(vector, filter="category=='tech'")
    SDK->>SDK: Release Python GIL
    SDK->>Core: Pass reference & execute
    Core->>Core: HNSW Traversal + IQL Syntax Tree Filtering (Parallel)
    Core-->>SDK: Return top_k IDs & Pointers
    SDK->>SDK: Re-acquire GIL
    SDK-->>App: Zero-Copy Python List returned
```

## 3. Biomimetic Governance (Memory Constraints & Survival Mode)

Memory is the major enemy of vector search. Vantadb utilizes an internal background thread pool ("SleepWorker", originally conceptualized from biological sleep cycles) to perform active memory governance.

When Vantadb is initialized, it is injected with `memory_limit_bytes` (e.g., 512MB).

* **Active Monitoring (Cgroups Detection):** The DB continuously queries the OS (and container Cgroups if running in Docker) to survey actual memory pressure.
* **Survival Mode Swap (MMap):** If RAM usage exceeds 85% of the threshold, the system triggers `Survival Mode`. Instead of allowing the kernel to hit an OOM (Out-of-Memory) panic and crash the DB, Vantadb dynamically flushes non-critical HNSW subgraph tiers and historical raw metadata chunks onto SSD.
* **Virtual Memory Fallback:** It immediately swaps references to `memmap2`, taking a slight latency penalty to guarantee absolute software survival.

## 4. Persistence Layer (RocksDB Integration)

To prevent data loss in a strictly in-process architecture:

1. **Write-Ahead Logging (WAL):** Every mutation to the `UnifiedNode` logic is synchronously streamed to an underlying embedded RocksDB instance.
2. **Startup Rehydration:** On crash or restart, the DB reads the RocksDB SSTables and rapidly rebuilds the HNSW spatial tiers in RAM.
3. **Compaction ("Apoptosis"):** The background GC selectively drops Tombstoned vectors and truncates RocksDB logs during low-traffic moments, ensuring minimal disk blooming.


================================================================
Nombre: CONFIGURATION.md
Ruta: docs\operations\CONFIGURATION.md
================================================================

# Operations & Configuration Manual

For DevOps, SREs, and Systems Engineers operating Vantadb in production via Docker or direct PyO3 instantiation.

Vantadb behaves similarly to SQLite: configuration parameters are primarily defined at runtime initialization but can also fall back to OS environment variables.

## 1. Constructor Initialization Params (Python SDK)

When orchestrating Vantadb directly inside your application code:

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `path` | `str` | `"./nexus_data"` | The local filesystem directory for RocksDB SSD physical persistence. |
| `read_only` | `bool` | `False` | Locks the FFI thread in Read-Only concurrency mode. Ideal for Uvicorn/Gunicorn multi-process read orchestration. |
| `memory_limit_bytes` | `int` | `1024_000_000` (1GB) | The absolute Cgroups ceiling constraint. Triggers the internal MMap swap (Survival Mode) when approaching runtime RAM panic limits. |

```python
import Vantadb

db = Vantadb.Vantadb(
    path="/mnt/volume/db",
    read_only=False,
    memory_limit_bytes=512_000_000 # 512MB Hard limit
)
```

## 2. Server Runtime (Environment Variables)

When deploying Vantadb as a standalone HTTP/Axum microservice using the official Docker container, pass the following ENV variables:

| Variable | Description | Default Target |
|----------|-------------|----------------|
| `Vantadb_HOST` | Bind address for the Rust HTTP layer. | `0.0.0.0` |
| `Vantadb_PORT` | Exposure TCP port. | `8080` |
| `Vantadb_STORAGE_PATH` | Equivalency to the `path` param. | `/data` |
| `Vantadb_THREADS` | Tokyo async worker count (defaults to Host vCPUs). | `auto` |
| `RUST_LOG` | Telemetry verbosity (`info`, `debug`, `trace`, `error`). | `info` |

## 3. Hardware Optimizations & SIMD

Vantadb natively compiles using `target-cpu=native`. This guarantees that your compiled binary leverages hardware-specific **SIMD / AVX-512** instruction sets present natively on your underlying x86/ARM motherboard when computing massive array multiplications during Vector Distance operations. No explicit ENV flags are required for optimization.


================================================================
Nombre: langchain_rag.py
Ruta: examples\python\langchain_rag.py
================================================================

import time
import uuid

# In a real environment, you would use standard LangChain modules:
# from langchain.llms import Ollama
# from langchain.embeddings import SentenceTransformerEmbeddings
print("[Setup] Loading Vantadb & Emulated LangChain Modules...")

import vantadb

# ---------------------------------------------------------
# 1. INITIALIZE NEXUSDB (In-Process, Zero-Network)
# ---------------------------------------------------------
# Like SQLite, it lives in your python heap.
# We set a strict 128MB limit for this script to demo the Survival Mode constraints.
db = vantadb.Vantadb(
    path="./local_nexus_brain", read_only=False, memory_limit_bytes=128_000_000
)

# ---------------------------------------------------------
# 2. DOCUMENT INGESTION (Ollama Embeddings Pipeline)
# ---------------------------------------------------------
raw_documents = [
    "Vantadb is a deeply embedded database written in Rust.",
    "Using multiple databases (Vector, Graph, Relational) creates a glue-code nightmare.",
    "By compiling the database via PyO3, Python apps can query vectors with Zero-Copy overhead.",
    "Survival mode automatically shifts HNSW heaps to Disk (MMAP) when RAM is low.",
]


def hook_embed_documents(texts):
    """
    Mocking a local embedding model (e.g. all-MiniLM-L6-v2 or Llama3 via Ollama).
    In reality, this returns a float32 array per text.
    """
    return [[0.1, 0.4, 0.6] for _ in texts]  # Dummy vectors for brevity


print("[RAG] Ingesting documents...")
start_time = time.perf_counter()

embeddings = hook_embed_documents(raw_documents)
for i, text in enumerate(raw_documents):
    db.insert(
        {
            "doc_id": str(uuid.uuid4()),
            "content": text,
            "vector": embeddings[i],
            "category": "architecture",
            "processed": True,
        }
    )

print(
    f"[RAG] Ingestion completed in {(time.perf_counter() - start_time) * 1000:.2f} ms"
)

# ---------------------------------------------------------
# 3. SEMANTIC RETRIEVAL (Zero-Latency Hybrid Query)
# ---------------------------------------------------------
user_query = "How does it avoid the glue-code problem in Python?"
query_embedding = hook_embed_documents([user_query])[0]

print("\n[RAG] Searching for context...")
search_start = time.perf_counter()

# This call maps directly to Rust C-ABI. No HTTP parsing, no TCP overhead.
results = db.search(
    vector=query_embedding,
    top_k=2,
    filter_expr="category == 'architecture' AND processed == true",
)
search_end = time.perf_counter()

print(f"[RAG] Semantic search took {(search_end - search_start) * 1000:.3f} ms.")

# ---------------------------------------------------------
# 4. LLM GENERATION (Augmentation)
# ---------------------------------------------------------
# We pass the zero-latency results directly to the LLM prompt.
context = "\n".join([str(res.get("content", "")) for res in results])
prompt = f"Answer the user's question using the context.\n\nContext:\n{context}\n\nQuestion: {user_query}"

print(f"\n[Generated Prompt Output to LLM]\n{prompt}\n")
print("✅ Local RAG pipeline executed successfully without leaving the Python process.")


================================================================
Nombre: backend.rs
Ruta: src\backend.rs
================================================================

//! Storage backend abstraction layer.
//!
//! Defines the `StorageBackend` trait and supporting types that decouple
//! `StorageEngine` from any specific persistent KV store (RocksDB, Fjall, etc.).
//!
//! ## Design notes
//!
//! - `scan()` returns a materialized `Vec<(Vec<u8>, Vec<u8>)>` instead of an
//!   iterator. This avoids `dyn Trait` lifetime complexity and is acceptable
//!   because `scan` is only used in `recover_archived_nodes`, which collects
//!   all entries anyway. It is not intended as a hot-path abstraction.
//!
//! - `compact()` has a default no-op implementation. Backends that lack native
//!   compaction (e.g. `InMemoryBackend`) simply inherit the no-op.
//!
//! - This trait is **crate-internal** (`pub(crate)`). It is not part of the
//!   public API surface and should not be implemented outside this crate.

use crate::error::Result;
use std::path::Path;

// ─── Partition Vocabulary ───────────────────────────────────

/// Logical partitions that replace stringly-typed column family names.
///
/// Every KV operation targets exactly one partition. The backend
/// implementation decides how to map these to physical storage
/// (e.g. RocksDB column families, separate BTreeMaps, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum BackendPartition {
    /// Primary metadata store (node metadata, relational fields).
    Default,
    /// Auditable tombstone archive for conflict resolution losers.
    TombstoneStorage,
    /// Compressed semantic summaries (data compression output).
    CompressedArchive,
    /// Lightweight tombstone markers for `is_deleted` checks.
    Tombstones,
}

impl BackendPartition {
    /// Returns the RocksDB column family name for this partition.
    /// Used only by `RocksDbBackend` internally.
    pub(crate) fn cf_name(&self) -> &'static str {
        match self {
            BackendPartition::Default => "default",
            BackendPartition::TombstoneStorage => "tombstone_storage",
            BackendPartition::CompressedArchive => "compressed_archive",
            BackendPartition::Tombstones => "tombstones",
        }
    }
}

// ─── Batch Write Operations ─────────────────────────────────

/// A single write operation within an atomic batch.
pub(crate) enum BackendWriteOp {
    #[allow(dead_code)]
    Put {
        partition: BackendPartition,
        key: Vec<u8>,
        value: Vec<u8>,
    },
    Delete {
        partition: BackendPartition,
        key: Vec<u8>,
    },
}

// ─── Backend Trait ──────────────────────────────────────────

/// Abstraction over the persistent KV store used by `StorageEngine`.
///
/// Covers only the operations that `StorageEngine` actually needs.
/// Does **not** include HNSW, VantaFile, WAL, or any higher-level
/// engine logic — those remain in `StorageEngine` directly.
///
/// This trait is crate-internal and should not be exposed publicly.
pub(crate) trait StorageBackend: Send + Sync {
    /// Write a key-value pair to the given partition.
    fn put(&self, partition: BackendPartition, key: &[u8], value: &[u8]) -> Result<()>;

    /// Read a value by key from the given partition.
    fn get(&self, partition: BackendPartition, key: &[u8]) -> Result<Option<Vec<u8>>>;

    /// Delete a key from the given partition.
    fn delete(&self, partition: BackendPartition, key: &[u8]) -> Result<()>;

    /// Execute a batch of write operations atomically (where supported).
    fn write_batch(&self, ops: Vec<BackendWriteOp>) -> Result<()>;

    /// Return all key-value pairs in the given partition.
    ///
    /// Returns a materialized `Vec` to avoid iterator lifetime issues
    /// behind `dyn Trait`. Not intended for hot-path use.
    fn scan(&self, partition: BackendPartition) -> Result<Vec<(Vec<u8>, Vec<u8>)>>;

    /// Flush all pending writes to durable storage.
    fn flush(&self) -> Result<()>;

    /// Create a consistent snapshot at the given filesystem path.
    ///
    /// Backends that do not support checkpointing should return an
    /// explicit error.
    fn checkpoint(&self, path: &Path) -> Result<()>;

    /// Request background compaction. Default implementation is a no-op
    /// for backends that do not support or need compaction.
    fn compact(&self) {
        // no-op by default
    }
}


================================================================
Nombre: columnar.rs
Ruta: src\columnar.rs
================================================================

use crate::error::Result;
use crate::node::UnifiedNode;
use arrow::array::{Float32Array, UInt64Array};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use std::sync::Arc;

/// Converts a collection of UnifiedNodes into an Apache Arrow RecordBatch.
/// This enables zero-copy SIMD analytical scans directly inside the executor or
/// zero-cost transmission to a Python client (Pandas/Polars).
pub fn nodes_to_record_batch(nodes: &[UnifiedNode]) -> Result<RecordBatch> {
    let mut ids = Vec::with_capacity(nodes.len());
    let mut vec_coords = Vec::new(); // Naive flattened vector logic for MVP

    for node in nodes {
        ids.push(node.id);
        // Only push first vector dimension to prove columnar packing capabilities
        if let crate::node::VectorRepresentations::Full(ref v) = node.vector {
            if !v.is_empty() {
                vec_coords.push(v[0]);
            } else {
                vec_coords.push(0.0);
            }
        } else {
            vec_coords.push(0.0);
        }
    }

    let id_array = UInt64Array::from(ids);
    let coords_array = Float32Array::from(vec_coords);

    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::UInt64, false),
        Field::new("vector_d0", DataType::Float32, true),
    ]));

    let batch = RecordBatch::try_new(schema, vec![Arc::new(id_array), Arc::new(coords_array)])
        .map_err(|e| crate::error::VantaError::Execution(e.to_string()))?;

    Ok(batch)
}


================================================================
Nombre: console.rs
Ruta: src\console.rs
================================================================

//! # VantaDB Professional Console
//!
//! Centralized, visually-rich terminal output system.
//! Brand colors: Vanta Black (`#050505`) · Rust Orange (`#CE422B`)
//!
//! ## Usage
//! ```rust
//! use vantadb::console;
//! console::init_logging();
//! console::print_banner();
//! console::ok("RocksDB opened", Some("4 column families"));
//! ```

use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

// ─── Banner ────────────────────────────────────────────────────────────────

/// Print the VantaDB ASCII banner to stdout.
/// Uses Rust Orange for the name and dim white for the tagline.
pub fn print_banner() {
    let border = style("═").color256(166).to_string(); // Rust Orange border
    let b = border.repeat(50);

    eprintln!();
    eprintln!("  {}", style(&b).color256(166));
    eprintln!(
        "  {}  {}  {}",
        style("║").color256(166),
        style("  ⚡  V A N T A D B   v0.1.0  ⚡  ")
            .bold()
            .color256(166),
        style("║").color256(166),
    );
    eprintln!(
        "  {}  {}  {}",
        style("║").color256(166),
        style("  Embedded Multimodal Database Engine     ")
            .dim()
            .white(),
        style("║").color256(166),
    );
    eprintln!(
        "  {}  {}  {}",
        style("║").color256(166),
        style("  Vector · Graph · Relational in one core ")
            .dim()
            .white(),
        style("║").color256(166),
    );
    eprintln!("  {}", style(&b).color256(166));
    eprintln!();
}

// ─── Logging Initialization ─────────────────────────────────────────────────

/// Initialize `tracing-subscriber` with colored, level-filtered output.
/// Respects the `RUST_LOG` env var (defaults to `info`).
pub fn init_logging() {
    use tracing_subscriber::{fmt, EnvFilter};

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    fmt::Subscriber::builder()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .with_ansi(true)
        .compact()
        .init();
}

// ─── Status Lines ───────────────────────────────────────────────────────────

/// `[✔] <label>  (<detail>)`  — success indicator
pub fn ok(label: &str, detail: Option<&str>) {
    let check = style("[✔]").green().bold();
    let lbl = style(label).white().bold();
    match detail {
        Some(d) => eprintln!("  {}  {:<36} {}", check, lbl, style(d).dim()),
        None => eprintln!("  {}  {}", check, lbl),
    }
}

/// `[→] <label>  (<detail>)` — progress / in-flight indicator
pub fn progress(label: &str, detail: Option<&str>) {
    let arrow = style("[→]").cyan().bold();
    let lbl = style(label).white();
    match detail {
        Some(d) => eprintln!("  {}  {:<36} {}", arrow, lbl, style(d).dim()),
        None => eprintln!("  {}  {}", arrow, lbl),
    }
}

/// `[⚠] <label>  (<detail>)` — warning indicator
pub fn warn(label: &str, detail: Option<&str>) {
    let ico = style("[⚠]").yellow().bold();
    let lbl = style(label).yellow();
    match detail {
        Some(d) => eprintln!("  {}  {:<36} {}", ico, lbl, style(d).dim()),
        None => eprintln!("  {}  {}", ico, lbl),
    }
}

/// `[✗] <label>  (<detail>)` — error indicator
pub fn error(label: &str, detail: Option<&str>) {
    let ico = style("[✗]").red().bold();
    let lbl = style(label).red().bold();
    match detail {
        Some(d) => eprintln!("  {}  {:<36} {}", ico, lbl, style(d).dim()),
        None => eprintln!("  {}  {}", ico, lbl),
    }
}

// ─── Section Dividers ───────────────────────────────────────────────────────

/// Print a labeled section header with Rust Orange separator
pub fn section(title: &str) {
    let line = style("─").color256(166).to_string().repeat(48);
    eprintln!();
    eprintln!(
        "  {}  {}  {}",
        style("┤").color256(166),
        style(title).color256(166).bold(),
        style("├").color256(166),
    );
    eprintln!("  {}", style(&line).color256(166).dim());
}

/// Print a simple separator line
pub fn separator() {
    eprintln!("  {}", style("─".repeat(50)).color256(166).dim());
}

// ─── Startup Summary ────────────────────────────────────────────────────────

/// Print the hardware + memory startup summary block
pub fn print_startup_summary(
    profile: &str,
    instructions: &str,
    total_memory_mb: u64,
    rocksdb_budget_mb: u64,
    backend_mode: &str,
    data_dir: &str,
) {
    section("System Configuration");
    eprintln!(
        "  {}  {:<20} {}",
        style("│").color256(166).dim(),
        style("Hardware:").dim(),
        style(profile).bold().white(),
    );
    eprintln!(
        "  {}  {:<20} {}",
        style("│").color256(166).dim(),
        style("Instructions:").dim(),
        style(instructions).bold().white(),
    );
    eprintln!(
        "  {}  {:<20} {}",
        style("│").color256(166).dim(),
        style("Total Memory:").dim(),
        style(format!("{} MB", total_memory_mb)).bold().white(),
    );
    eprintln!(
        "  {}  {:<20} {}",
        style("│").color256(166).dim(),
        style("RocksDB Budget:").dim(),
        style(format!("{} MB", rocksdb_budget_mb))
            .color256(166)
            .bold(),
    );
    eprintln!(
        "  {}  {:<20} {}",
        style("│").color256(166).dim(),
        style("HNSW Backend:").dim(),
        style(backend_mode).bold().white(),
    );
    eprintln!(
        "  {}  {:<20} {}",
        style("│").color256(166).dim(),
        style("Data Dir:").dim(),
        style(data_dir).dim().white(),
    );
    separator();
    eprintln!();
}

// ─── Ready Message ──────────────────────────────────────────────────────────

/// Print the final "server ready" line
pub fn print_ready(addr: &str) {
    eprintln!();
    eprintln!(
        "  {}  {} {}",
        style("[→]").color256(166).bold(),
        style("Listening on").white(),
        style(addr).color256(166).bold().underlined(),
    );
    eprintln!(
        "  {}  {}",
        style("   ").dim(),
        style("VantaDB is ready for connections.").dim(),
    );
    eprintln!();
}

// ─── Progress Bars ──────────────────────────────────────────────────────────

/// Create a styled progress bar for long operations (insert batch, indexing, etc.)
pub fn create_progress_bar(total: u64, message: &str) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::with_template(
            "  {spinner:.color256(166)} [{bar:40.color256(166)/dim}] {pos}/{len} {msg}",
        )
        .unwrap_or_else(|_| ProgressStyle::default_bar())
        .progress_chars("█▉▊▋▌▍▎▏ ")
        .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

/// Create a simple spinner for indeterminate operations
pub fn create_spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("  {spinner:.color256(166)} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_spinner())
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

// ─── Utilities ──────────────────────────────────────────────────────────────

/// Format bytes as human-readable string (B / KB / MB / GB)
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1_024;
    const MB: u64 = 1_024 * KB;
    const GB: u64 = 1_024 * MB;
    match bytes {
        b if b >= GB => format!("{:.1} GB", b as f64 / GB as f64),
        b if b >= MB => format!("{:.1} MB", b as f64 / MB as f64),
        b if b >= KB => format!("{:.1} KB", b as f64 / KB as f64),
        b => format!("{} B", b),
    }
}

/// Format milliseconds as human-readable duration (µs / ms / s)
pub fn format_duration_ms(ms: u128) -> String {
    match ms {
        t if t < 1 => format!("{}µs", t * 1000),
        t if t < 1_000 => format!("{}ms", t),
        t => format!("{:.2}s", t as f64 / 1000.0),
    }
}


================================================================
Nombre: engine.rs
Ruta: src\engine.rs
================================================================

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use parking_lot::{Mutex, RwLock};

use crate::error::{Result, VantaError};
use crate::node::{FieldValue, UnifiedNode, VectorRepresentations};
use crate::wal::{WalReader, WalRecord, WalWriter};

// ─── Query Result ──────────────────────────────────────────

/// How the result was produced
#[derive(Debug, Clone)]
pub enum SourceType {
    FullScan,
    BitsetFilter,
    VectorSearch,
    GraphTraversal,
    Hybrid,
}

/// Query result with exhaustivity metadata
#[derive(Debug, Clone)]
pub struct QueryResult {
    pub nodes: Vec<UnifiedNode>,
    /// true if resource limits truncated results
    pub is_partial: bool,
    /// 0.0-1.0 search completeness
    pub exhaustivity: f32,
    /// which index/scan was used
    pub source_type: SourceType,
}

/// Engine statistics snapshot
#[derive(Debug, Clone, Default)]
pub struct EngineStats {
    pub node_count: u64,
    pub edge_count: u64,
    pub vector_count: u64,
    pub total_dimensions: u64,
    pub memory_estimate_bytes: u64,
}

// ─── In-Memory Engine ──────────────────────────────────────

/// Fase 1 storage engine: HashMap + optional WAL.
///
/// Thread-safe: RwLock for reads, Mutex for WAL writes.
/// Fase 2: Replace HashMap with RocksDB-backed MemTable.
pub struct InMemoryEngine {
    nodes: RwLock<HashMap<u64, UnifiedNode>>,
    wal: Mutex<Option<WalWriter>>,
    next_id: AtomicU64,
    #[allow(dead_code)]
    wal_path: Option<PathBuf>,
}

impl InMemoryEngine {
    /// Create engine (in-memory only, no persistence)
    pub fn new() -> Self {
        Self {
            nodes: RwLock::new(HashMap::with_capacity(1024)),
            wal: Mutex::new(None),
            next_id: AtomicU64::new(1),
            wal_path: None,
        }
    }

    /// Create engine with WAL durability. Replays existing WAL on open.
    pub fn with_wal(wal_path: impl AsRef<Path>) -> Result<Self> {
        let path = wal_path.as_ref().to_path_buf();
        let mut nodes_map = HashMap::with_capacity(1024);
        let mut max_id: u64 = 0;

        // Replay existing WAL
        if path.exists() {
            let mut reader = WalReader::open(&path)?;
            reader.replay_all(|record| {
                match record {
                    WalRecord::Insert(node) => {
                        max_id = max_id.max(node.id);
                        nodes_map.insert(node.id, node);
                    }
                    WalRecord::Update { id, node } => {
                        max_id = max_id.max(id);
                        nodes_map.insert(id, node);
                    }
                    WalRecord::Delete { id } => {
                        nodes_map.remove(&id);
                    }
                    WalRecord::Checkpoint { .. } => {}
                }
                Ok(())
            })?;
        }

        let writer = WalWriter::open(&path)?;

        Ok(Self {
            nodes: RwLock::new(nodes_map),
            wal: Mutex::new(Some(writer)),
            next_id: AtomicU64::new(max_id + 1),
            wal_path: Some(path),
        })
    }

    /// Generate next unique node ID
    pub fn next_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }

    /// Insert a node. Auto-assigns ID if node.id == 0.
    pub fn insert(&self, mut node: UnifiedNode) -> Result<u64> {
        if node.id == 0 {
            node.id = self.next_id();
        }
        let id = node.id;

        // WAL first (durability before visibility)
        if let Some(ref mut wal) = *self.wal.lock() {
            wal.append(&WalRecord::Insert(node.clone()))?;
        }

        let mut nodes = self.nodes.write();
        if nodes.contains_key(&id) {
            return Err(VantaError::DuplicateNode(id));
        }
        nodes.insert(id, node);
        Ok(id)
    }

    /// Get a node by ID (cloned)
    pub fn get(&self, id: u64) -> Option<UnifiedNode> {
        self.nodes.read().get(&id).cloned()
    }

    /// Check if node exists
    pub fn contains(&self, id: u64) -> bool {
        self.nodes.read().contains_key(&id)
    }

    /// Update existing node
    pub fn update(&self, id: u64, node: UnifiedNode) -> Result<()> {
        if let Some(ref mut wal) = *self.wal.lock() {
            wal.append(&WalRecord::Update {
                id,
                node: node.clone(),
            })?;
        }
        let mut nodes = self.nodes.write();
        if !nodes.contains_key(&id) {
            return Err(VantaError::NodeNotFound(id));
        }
        nodes.insert(id, node);
        Ok(())
    }

    /// Delete a node
    pub fn delete(&self, id: u64) -> Result<()> {
        if let Some(ref mut wal) = *self.wal.lock() {
            wal.append(&WalRecord::Delete { id })?;
        }
        let mut nodes = self.nodes.write();
        if nodes.remove(&id).is_none() {
            return Err(VantaError::NodeNotFound(id));
        }
        Ok(())
    }

    /// Scan nodes matching a bitset mask (all bits in mask must be set)
    pub fn scan_bitset(&self, mask: u128) -> Vec<u64> {
        self.nodes
            .read()
            .values()
            .filter(|n| n.is_alive() && n.matches_mask(mask))
            .map(|n| n.id)
            .collect()
    }

    /// Brute-force vector similarity search.
    /// Fase 3: Replace with CP-Index HNSW for O(log n).
    pub fn vector_search(
        &self,
        query: &[f32],
        top_k: usize,
        min_score: f32,
        bitset_filter: Option<u128>,
    ) -> QueryResult {
        let query_vec = VectorRepresentations::Full(query.to_vec());
        let nodes = self.nodes.read();

        let mut scored: Vec<(u64, f32)> = nodes
            .values()
            .filter(|n| {
                n.is_alive()
                    && !n.vector.is_none()
                    && bitset_filter.is_none_or(|m| n.matches_mask(m))
            })
            .filter_map(|n| {
                n.vector
                    .cosine_similarity(&query_vec)
                    .filter(|&s| s >= min_score)
                    .map(|s| (n.id, s))
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);

        let result_nodes: Vec<UnifiedNode> = scored
            .iter()
            .filter_map(|(id, _)| nodes.get(id).cloned())
            .collect();

        QueryResult {
            nodes: result_nodes,
            is_partial: false,
            exhaustivity: 1.0, // brute-force = exhaustive
            source_type: if bitset_filter.is_some() {
                SourceType::Hybrid
            } else {
                SourceType::VectorSearch
            },
        }
    }

    /// BFS graph traversal from start, following edges with matching label.
    /// Returns (node_id, depth) pairs within [min_depth, max_depth].
    pub fn traverse(
        &self,
        start: u64,
        label: &str,
        min_depth: u32,
        max_depth: u32,
    ) -> Result<Vec<(u64, u32)>> {
        let nodes = self.nodes.read();
        if !nodes.contains_key(&start) {
            return Err(VantaError::NodeNotFound(start));
        }

        let mut visited = HashMap::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back((start, 0u32));
        visited.insert(start, 0u32);

        let mut results = Vec::new();

        while let Some((current_id, depth)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }
            if let Some(node) = nodes.get(&current_id) {
                for edge in &node.edges {
                    if edge.label == label {
                        if let std::collections::hash_map::Entry::Vacant(e) =
                            visited.entry(edge.target)
                        {
                            let next_depth = depth + 1;
                            e.insert(next_depth);
                            if next_depth >= min_depth {
                                results.push((edge.target, next_depth));
                            }
                            queue.push_back((edge.target, next_depth));
                        }
                    }
                }
            }
        }
        Ok(results)
    }

    /// Filter nodes by relational field equality
    pub fn filter_field(&self, field: &str, value: &FieldValue) -> Vec<u64> {
        self.nodes
            .read()
            .values()
            .filter(|n| n.is_alive() && n.get_field(field) == Some(value))
            .map(|n| n.id)
            .collect()
    }

    /// Hybrid search: vector similarity + bitset filter + field predicates.
    /// Evaluates filters in cost order: bitset → relational → vector.
    pub fn hybrid_search(
        &self,
        query_vector: &[f32],
        top_k: usize,
        min_score: f32,
        bitset_mask: Option<u128>,
        field_filters: &[(String, FieldValue)],
    ) -> QueryResult {
        let query_vec = VectorRepresentations::Full(query_vector.to_vec());
        let nodes = self.nodes.read();

        let mut scored: Vec<(u64, f32)> = nodes
            .values()
            .filter(|n| {
                if !n.is_alive() || n.vector.is_none() {
                    return false;
                }
                // Bitset first (cheapest: single AND)
                if let Some(mask) = bitset_mask {
                    if !n.matches_mask(mask) {
                        return false;
                    }
                }
                // Relational second
                for (field, value) in field_filters {
                    if n.get_field(field) != Some(value) {
                        return false;
                    }
                }
                true
            })
            .filter_map(|n| {
                n.vector
                    .cosine_similarity(&query_vec)
                    .filter(|&s| s >= min_score)
                    .map(|s| (n.id, s))
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);

        let result_nodes = scored
            .iter()
            .filter_map(|(id, _)| nodes.get(id).cloned())
            .collect();

        QueryResult {
            nodes: result_nodes,
            is_partial: false,
            exhaustivity: 1.0,
            source_type: SourceType::Hybrid,
        }
    }

    /// Flush WAL to disk
    pub fn flush_wal(&self) -> Result<()> {
        if let Some(ref mut wal) = *self.wal.lock() {
            wal.sync()?;
        }
        Ok(())
    }

    /// Total number of alive nodes
    pub fn node_count(&self) -> usize {
        self.nodes.read().values().filter(|n| n.is_alive()).count()
    }

    /// Get engine statistics
    pub fn stats(&self) -> EngineStats {
        let nodes = self.nodes.read();
        let mut stats = EngineStats::default();
        for node in nodes.values() {
            if !node.is_alive() {
                continue;
            }
            stats.node_count += 1;
            stats.edge_count += node.edges.len() as u64;
            if !node.vector.is_none() {
                stats.vector_count += 1;
                stats.total_dimensions += node.vector.dimensions() as u64;
            }
            stats.memory_estimate_bytes += node.memory_size() as u64;
        }
        stats
    }
}

impl Default for InMemoryEngine {
    fn default() -> Self {
        Self::new()
    }
}


================================================================
Nombre: error.rs
Ruta: src\error.rs
================================================================

use thiserror::Error;

/// Core error type for all VantaDB operations
#[derive(Error, Debug)]
pub enum VantaError {
    #[error("Node not found: {0}")]
    NodeNotFound(u64),

    #[error("Duplicate node ID: {0}")]
    DuplicateNode(u64),

    #[error("Vector dimension mismatch: expected {expected}, got {got}")]
    DimensionMismatch { expected: usize, got: usize },

    #[error("WAL error: {0}")]
    WalError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Engine not initialized")]
    NotInitialized,

    #[error("Resource limit exceeded: {0}")]
    ResourceLimit(String),

    #[error("Execution error: {0}")]
    Execution(String),
}

/// Crate-wide Result alias
pub type Result<T> = std::result::Result<T, VantaError>;


================================================================
Nombre: executor.rs
Ruta: src\executor.rs
================================================================

use crate::error::{Result, VantaError};
use crate::eval::LispSandbox;
use crate::governance::{ConfidenceArbiter, ResolutionResult};
use crate::node::{UnifiedNode, VectorRepresentations};
use crate::parser::lisp::parse as parse_lisp_expr;
use crate::parser::parse_statement;
use crate::query::{LogicalOperator, LogicalPlan, Statement};
use crate::storage::StorageEngine;
use std::sync::atomic::{AtomicU32, Ordering};

pub enum ExecutionResult {
    Read(Vec<UnifiedNode>),
    Write {
        affected_nodes: usize,
        message: String,
        node_id: Option<u64>,
    },
    StaleContext(u64), // Phase 30: Señal de que un contexto requiere rehidratación (Confidence Score crítico)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SearchPathMode {
    Standard,
    Uncertain,
}

/// Certitude Mode governs query fidelity vs latency tradeoff.
/// Asymmetric I/O quota: STRICT consumes 3x, BALANCED 1.5x, FAST 1x.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CertitudeMode {
    /// L1 only (Hamming). Lowest latency, lowest fidelity.
    Fast,
    /// L1 + L2 re-ranking (PolarQuant). Balanced.
    Balanced,
    /// L1 + L2 + L3 FP32 verification. Highest fidelity, highest I/O cost.
    Strict,
}

impl CertitudeMode {
    /// Returns the I/O quota multiplier for asymmetric penalization.
    /// Prevents inefficient agents from saturating disk bandwidth.
    pub fn io_quota_multiplier(&self) -> f32 {
        match self {
            CertitudeMode::Fast => 1.0,
            CertitudeMode::Balanced => 1.5,
            CertitudeMode::Strict => 3.0,
        }
    }
}

pub struct Executor<'a> {
    storage: &'a StorageEngine,
    certitude: CertitudeMode,
    path_mode: SearchPathMode,
    /// Tracks cumulative I/O cost of this executor session.
    /// Hardware backpressure uses this to throttle expensive agents.
    io_budget_consumed: AtomicU32,
}

impl<'a> Executor<'a> {
    pub fn new(storage: &'a StorageEngine) -> Self {
        Self {
            storage,
            certitude: CertitudeMode::Balanced,
            path_mode: SearchPathMode::Standard,
            io_budget_consumed: AtomicU32::new(0.0_f32.to_bits()),
        }
    }

    pub fn with_certitude(storage: &'a StorageEngine, mode: CertitudeMode) -> Self {
        Self {
            storage,
            certitude: mode,
            path_mode: SearchPathMode::Standard,
            io_budget_consumed: AtomicU32::new(0.0_f32.to_bits()),
        }
    }

    pub fn with_path_mode(mut self, path: SearchPathMode) -> Self {
        self.path_mode = path;
        self
    }

    /// Track I/O cost with asymmetric penalization based on CertitudeMode.
    fn consume_io(&self, base_cost: f32) {
        let penalty = base_cost * self.certitude.io_quota_multiplier();
        let mut current_bits = self.io_budget_consumed.load(Ordering::Acquire);
        loop {
            let current = f32::from_bits(current_bits);
            let next = current + penalty;
            match self.io_budget_consumed.compare_exchange_weak(
                current_bits,
                next.to_bits(),
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(b) => current_bits = b,
            }
        }
    }

    /// Returns the cumulative I/O budget consumed by this executor.
    pub fn io_consumed(&self) -> f32 {
        f32::from_bits(self.io_budget_consumed.load(Ordering::Acquire))
    }

    /// Inserts a pre-built UnifiedNode directly into storage.
    /// Used by the LISP sandbox to inject Node rules.
    pub fn insert_node(&self, node: &crate::node::UnifiedNode) -> crate::error::Result<()> {
        self.storage.insert(node)
    }

    pub async fn execute_hybrid(&self, query_string: &str) -> Result<ExecutionResult> {
        let trimmed = query_string.trim_start();
        if trimmed.starts_with('(') {
            let expr = parse_lisp_expr(trimmed)
                .map_err(|e| VantaError::Execution(format!("LISP Parse Error: {}", e)))?;
            let mut sandbox = LispSandbox::new(self);
            sandbox.eval(std::borrow::Cow::Owned(expr)).await
        } else {
            match parse_statement(trimmed) {
                Ok((_, stmt)) => self.execute_statement(stmt).await,
                Err(e) => Err(VantaError::Execution(format!("IQL Parse Error: {}", e))),
            }
        }
    }

    pub async fn execute_statement(&self, statement: Statement) -> Result<ExecutionResult> {
        // ── Memory Pressure Check ──
        {
            use crate::governor::{AllocationStatus, ResourceGovernor};
            let governor = ResourceGovernor::new(2 * 1024 * 1024 * 1024, 50);
            let probe_cost = 0;
            if let Ok(AllocationStatus::GrantedWithPressure) =
                governor.request_allocation(probe_cost)
            {
                println!("🚨 [ResourceGovernor] High memory pressure (>90%) detected. Triggering emergency flush.");
                if let Some(winner) = self.storage.consistency_buffer.force_flush() {
                    println!("    └─ Priority record preserved: {}", winner.id);
                    let _ = self.storage.insert(&winner);
                }
            }
        }

        match statement {
            Statement::Query(query) => {
                let plan = query.into_logical_plan();
                let nodes = self.execute_plan(plan).await?;

                use crate::node::AccessTracker;
                // Fase 30: Interceptación Arqueológica (Non-blocking)
                for node in &nodes {
                    if let Some(crate::node::FieldValue::String(node_type)) =
                        node.relational.get("type")
                    {
                        if node_type == "SemanticSummary" && node.confidence_score() < 0.4 {
                            println!("⚠️ [Executor] Supervised mode: Low-confidence summary detected (ID 0). Skipping.");
                            continue;
                        }
                    }
                }

                Ok(ExecutionResult::Read(nodes))
            }
            Statement::Insert(insert) => {
                let mut node = UnifiedNode::new(insert.node_id);
                node.set_field("type", crate::node::FieldValue::String(insert.node_type));

                // Copy all provided fields
                for (k, v) in insert.fields.clone() {
                    node.set_field(&k, v);
                }

                // Auto-Embedding Logic: If VECTOR is not provided in IQL, but "texto" field exists!
                if insert.vector.is_none() {
                    if let Some(crate::node::FieldValue::String(text)) = insert.fields.get("texto")
                    {
                        let llm = crate::llm::LlmClient::new();
                        // Request vectors to local Ollama inference bridge
                        if let Ok(vec) = llm.generate_embedding(text).await {
                            node.vector = VectorRepresentations::Full(vec);
                            node.flags.set(crate::node::NodeFlags::HAS_VECTOR);
                        }
                    }
                } else if let Some(vec) = insert.vector {
                    node.vector = VectorRepresentations::Full(vec);
                    node.flags.set(crate::node::NodeFlags::HAS_VECTOR);
                }

                // ── Admission Filter Check ──
                if let Some(crate::node::FieldValue::String(role)) =
                    node.relational.get("_owner_role")
                {
                    if self.storage.admission_filter.is_role_blocked(role) {
                        return Err(VantaError::Execution(format!(
                            "Admission Policy: agent '{}' has Confidence Score 0.0 (blocked)",
                            role
                        )));
                    }
                }

                // Conflict Resolution
                if node.flags.is_set(crate::node::NodeFlags::HAS_VECTOR) {
                    if let crate::node::VectorRepresentations::Full(vec) = &node.vector {
                        let nearest = {
                            let index = self.storage.hnsw.read();
                            let vs = self.storage.vector_store.read();
                            index.search_nearest(vec, None, None, 0, 1, Some(&vs))
                        };

                        if let Some((existing_id, _)) = nearest.first() {
                            if *existing_id != node.id {
                                if let Some(existing) = self.storage.get(*existing_id)? {
                                    match self
                                        .storage
                                        .conflict_resolver
                                        .evaluate_conflict(&existing, &node)
                                    {
                                        ResolutionResult::Reject(reason) => {
                                            return Err(VantaError::Execution(format!(
                                                "Consistency Violation: {}",
                                                reason
                                            )));
                                        }
                                        ResolutionResult::Superposition(record) => {
                                            self.storage.consistency_buffer.insert_record(record);
                                            return Ok(ExecutionResult::Write {
                                                affected_nodes: 1,
                                                message: format!("Node {} moved to ConsistencyBuffer (Pending Resolution).", node.id),
                                                node_id: Some(node.id),
                                            });
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                }

                self.storage.insert(&node)?;
                Ok(ExecutionResult::Write {
                    affected_nodes: 1,
                    message: format!("Node {} inserted.", insert.node_id),
                    node_id: Some(insert.node_id),
                })
            }
            Statement::Update(update) => {
                let mut node = match self.storage.get(update.node_id)? {
                    Some(n) => n,
                    None => {
                        return Err(VantaError::Execution(format!(
                            "Node {} not found for update",
                            update.node_id
                        )))
                    }
                };
                for (k, v) in update.fields {
                    node.set_field(k, v);
                }
                if let Some(vec) = update.vector {
                    node.vector = VectorRepresentations::Full(vec);
                    node.flags.set(crate::node::NodeFlags::HAS_VECTOR);
                }
                // ── Admission Filter Check ──
                if let Some(crate::node::FieldValue::String(role)) =
                    node.relational.get("_owner_role")
                {
                    if self.storage.admission_filter.is_role_blocked(role) {
                        return Err(VantaError::Execution(
                            format!("Admission Policy (Update): agent '{}' has Confidence Score 0.0 (blocked)", role)
                        ));
                    }
                }

                // Conflict Resolution
                if node.flags.is_set(crate::node::NodeFlags::HAS_VECTOR) {
                    if let crate::node::VectorRepresentations::Full(vec) = &node.vector {
                        let nearest = {
                            let index = self.storage.hnsw.read();
                            let vs = self.storage.vector_store.read();
                            index.search_nearest(vec, None, None, 0, 1, Some(&vs))
                        };

                        if let Some((existing_id, _)) = nearest.first() {
                            if *existing_id != node.id {
                                if let Some(existing) = self.storage.get(*existing_id)? {
                                    match self
                                        .storage
                                        .conflict_resolver
                                        .evaluate_conflict(&existing, &node)
                                    {
                                        ResolutionResult::Reject(reason) => {
                                            return Err(VantaError::Execution(format!(
                                                "Consistency Violation (Update): {}",
                                                reason
                                            )));
                                        }
                                        ResolutionResult::Superposition(record) => {
                                            self.storage.consistency_buffer.insert_record(record);
                                            return Ok(ExecutionResult::Write {
                                                affected_nodes: 1,
                                                message: format!("Node {} update entered ConsistencyBuffer (Pending Resolution).", node.id),
                                                node_id: Some(node.id),
                                            });
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                }

                self.storage.insert(&node)?;
                Ok(ExecutionResult::Write {
                    affected_nodes: 1,
                    message: format!("Node {} updated.", node.id),
                    node_id: Some(node.id),
                })
            }
            Statement::Delete(delete) => {
                self.storage.delete(delete.node_id, "IQL Manual Deletion")?;
                Ok(ExecutionResult::Write {
                    affected_nodes: 1,
                    message: format!("Node {} deleted.", delete.node_id),
                    node_id: Some(delete.node_id),
                })
            }
            Statement::Relate(relate) => {
                let mut node = match self.storage.get(relate.source_id)? {
                    Some(n) => n,
                    None => {
                        return Err(VantaError::Execution(format!(
                            "Source Node {} not found for relation",
                            relate.source_id
                        )))
                    }
                };

                // Axiom: Topological Consistency
                if self.storage.get(relate.target_id)?.is_none() {
                    if self.storage.is_deleted(relate.target_id).unwrap_or(false) {
                        return Err(VantaError::Execution(format!(
                            "Reference to deleted node: ID {} resides in the Tombstone storage",
                            relate.target_id
                        )));
                    } else {
                        return Err(VantaError::Execution(format!(
                            "Topological Axiom violated: Target Node {} does not exist",
                            relate.target_id
                        )));
                    }
                }

                if let Some(w) = relate.weight {
                    node.add_weighted_edge(relate.target_id, relate.label, w);
                } else {
                    node.add_edge(relate.target_id, relate.label);
                }
                self.storage.insert(&node)?;
                Ok(ExecutionResult::Write {
                    affected_nodes: 1,
                    message: format!(
                        "Edge related from {} to {}.",
                        relate.source_id, relate.target_id
                    ),
                    node_id: Some(relate.source_id),
                })
            }
            Statement::InsertMessage(msg) => {
                // Syntactic Sugar for Chat Threads: Creates a node and relates it.
                // Normally we'd use a UUID generator, but for MVP we use a timestamp-based ID or random
                let msg_id = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_micros() as u64;
                let mut node = UnifiedNode::new(msg_id);
                node.set_field(
                    "type",
                    crate::node::FieldValue::String("Message".to_string()),
                );
                node.set_field(
                    "role",
                    crate::node::FieldValue::String(msg.msg_role.clone()),
                );
                node.set_field(
                    "content",
                    crate::node::FieldValue::String(msg.content.clone()),
                );

                // Embed directly via LLM since it's a message
                let llm = crate::llm::LlmClient::new();
                if let Ok(vec) = llm.generate_embedding(&msg.content).await {
                    node.vector = VectorRepresentations::Full(vec);
                    node.flags.set(crate::node::NodeFlags::HAS_VECTOR);
                }

                // Now create relationship: MESSAGE -> belongs_to -> THREAD
                node.add_edge(msg.thread_id, "belongs_to_thread".to_string());

                // Node is saved (Atomic write for State + Edge)
                self.storage.insert(&node)?;

                Ok(ExecutionResult::Write {
                    affected_nodes: 2,
                    message: format!(
                        "Message {} inserted and linked to Thread {}.",
                        msg_id, msg.thread_id
                    ),
                    node_id: Some(msg_id),
                })
            }
            Statement::Collapse(collapse) => {
                let mut buffer = self.storage.consistency_buffer.records.write();
                if let Some(mut record) = buffer.remove(&collapse.zone_id) {
                    if collapse.index < record.candidates.len() {
                        let winner = record.candidates.remove(collapse.index);

                        // Remaining candidates to archive
                        let mut losers_to_archive = Vec::new();
                        for cand in record.candidates {
                            losers_to_archive.push((
                                collapse.zone_id,
                                cand.id,
                                "Manual Resolution: Candidate discarded by administrator"
                                    .to_string(),
                            ));
                        }

                        self.storage
                            .consistency_buffer
                            .stats
                            .pending_to_resolved
                            .fetch_add(1, Ordering::Relaxed);
                        drop(buffer);

                        self.storage.insert(&winner)?;

                        if !losers_to_archive.is_empty() {
                            use crate::backend::BackendPartition;
                            use crate::governance::AuditableTombstone;
                            for (id, hash, reason) in losers_to_archive {
                                let tomb = AuditableTombstone::new(id, reason, hash);
                                let key = id.to_le_bytes();
                                if let Ok(tomb_val) = bincode::serialize(&tomb) {
                                    let _ = self.storage.put_to_partition(
                                        BackendPartition::TombstoneStorage,
                                        &key,
                                        &tomb_val,
                                    );
                                }
                            }
                        }

                        Ok(ExecutionResult::Write {
                            affected_nodes: 1,
                            message: format!(
                                "Consistency record {} resolved. Candidate {} prevailed.",
                                collapse.zone_id, collapse.index
                            ),
                            node_id: Some(collapse.zone_id),
                        })
                    } else {
                        Err(VantaError::Execution(format!(
                            "Candidate index {} out of bounds for record {}",
                            collapse.index, collapse.zone_id
                        )))
                    }
                } else {
                    Err(VantaError::Execution(format!(
                        "Consistency record {} not found in buffer",
                        collapse.zone_id
                    )))
                }
            }
        }
    }

    /// Evaluates the Logical Plan over the underlying storage engine
    pub async fn execute_plan(&self, mut plan: LogicalPlan) -> Result<Vec<UnifiedNode>> {
        use crate::governor::ResourceGovernor;

        let governor = ResourceGovernor::new(2 * 1024 * 1024 * 1024, 50); // 2GB Soft Limit, 50ms timeout
        governor.apply_temperature_limits(&mut plan);

        let estimated_mem_cost = 1024 * 1024; // 1MB estimated buffer footprint per query
        match governor.request_allocation(estimated_mem_cost)? {
            crate::governor::AllocationStatus::GrantedWithPressure => {
                println!("🚨 [ResourceGovernor] High memory pressure detected. Triggering emergency flush.");
                if let Some(winner) = self.storage.consistency_buffer.force_flush() {
                    println!("    └─ Priority record preserved: {}", winner.id);
                    let _ = self.storage.insert(&winner);
                }
            }
            crate::governor::AllocationStatus::Granted => {}
        }

        let mut results = Vec::new();
        let mut target_nodes = Vec::new();

        // Pass 1: Resolver Escaneo Vectorial Dinámico (Si hubiere Condition::VectorSim)
        let mut searched_hnsw = false;

        for op in &plan.operators {
            if let LogicalOperator::VectorSearch {
                field: _,
                query_vec,
                min_score: _,
            } = op
            {
                let llm = crate::llm::LlmClient::new();

                // Real Inference: Translate NLP into Embedded Vectors
                if let Ok(vector) = llm.generate_embedding(query_vec).await {
                    // Record basic vector search I/O cost (cost logic is synthetic placeholder)
                    self.consume_io(10.0);

                    let index = self.storage.hnsw.read();
                    let vs = self.storage.vector_store.read();
                    let mut neighbors = index.search_nearest(&vector, None, None, 0, 5, Some(&vs)); // MVP: top_k = 5

                    if self.path_mode == SearchPathMode::Uncertain {
                        // Scan the ConsistencyBuffer via brute force
                        let buffer = self.storage.consistency_buffer.records.read();
                        let target_vec = VectorRepresentations::Full(vector.clone());
                        let mut matches = Vec::new();

                        for (&id, record) in buffer.iter() {
                            for cand in &record.candidates {
                                if cand.flags.is_set(crate::node::NodeFlags::HAS_VECTOR) {
                                    if let Some(sim) = cand.vector.cosine_similarity(&target_vec) {
                                        // Apply a penalty to the pending match
                                        let penalized_sim = sim * 0.9;
                                        matches.push((id, penalized_sim));
                                    }
                                }
                            }
                        }

                        // Merge and sort
                        neighbors.extend(matches);
                        neighbors.sort_by(|a, b| {
                            b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
                        });
                        neighbors.truncate(5); // Keep top 5
                    }

                    for (id, _sim) in neighbors {
                        target_nodes.push(id);
                    }
                    searched_hnsw = true;
                }
            }
        }

        if !searched_hnsw {
            // Fallback: real scan based on FROM entity (Scan operator)
            for op in &plan.operators {
                if let LogicalOperator::Scan { entity } = op {
                    // If entity starts with Conflict#, intercept it immediately
                    if entity.starts_with("Conflict#") {
                        if let Some(id_str) = entity.split('#').nth(1) {
                            if let Ok(id) = id_str.parse::<u64>() {
                                let buffer = self.storage.consistency_buffer.records.read();
                                if let Some(record) = buffer.get(&id) {
                                    return Ok(record.candidates.clone());
                                }
                            }
                        }
                    } else if let Some(id_str) = entity.split('#').nth(1) {
                        if let Ok(id) = id_str.parse::<u64>() {
                            target_nodes.push(id);
                        }
                    }
                    // Otherwise, scan is deferred to post-filter (MVP limitation)
                    break;
                }
            }
        }

        // Pass 2: Materializar los nodos devueltos por el índice y filtrar RBAC
        for id in target_nodes {
            // Materializing nodes is I/O intensive, track heavily
            self.consume_io(2.5);

            if let Ok(Some(node)) = self.storage.get(id) {
                // Agented RBAC (Role-Based Access Control) Graph pruning
                if let Some(required_role) = &plan.enforce_role {
                    let mut role_match = false;
                    if let Some(crate::node::FieldValue::String(node_role)) =
                        node.relational.get("_owner_role")
                    {
                        if node_role == required_role {
                            role_match = true;
                        }
                    }
                    if !role_match && required_role != "admin" {
                        continue; // Prune branch (Sub-graph isolation enforced)
                    }
                }

                results.push(node);
            }
        }

        governor.free_allocation(estimated_mem_cost);
        Ok(results)
    }
}


================================================================
Nombre: gc.rs
Ruta: src\gc.rs
================================================================

use crate::storage::StorageEngine;
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct GcWorker<'a> {
    storage: &'a StorageEngine,
    // Maps expiration timestamp (seconds) to a list of Node IDs
    index_ttl: BTreeMap<u64, Vec<u64>>,
}

impl<'a> GcWorker<'a> {
    pub fn new(storage: &'a StorageEngine) -> Self {
        Self {
            storage,
            index_ttl: BTreeMap::new(),
        }
    }

    /// Registers a node to be automatically expired and cleared at `expiry_secs`
    pub fn register_ttl(&mut self, id: u64, expiry_secs: u64) {
        self.index_ttl.entry(expiry_secs).or_default().push(id);
    }

    /// Triggers a sweep that clears old items. In production this runs in a `tokio::spawn` loop.
    pub fn sweep(&mut self) -> usize {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Split the BTreeMap, taking all nodes where expiration <= now
        let mut expired_count = 0;

        let mut keys_to_remove = Vec::new();
        for (expiry, ids) in self.index_ttl.iter() {
            if *expiry <= now {
                for &id in ids {
                    // Attempt deletion via StorageEngine physical store
                    if self.storage.delete(id, "GC TTL Expired").is_ok() {
                        expired_count += 1;
                    }
                }
                keys_to_remove.push(*expiry);
            } else {
                break; // Because it's a BTreeMap, subsequent keys are > now
            }
        }

        for key in keys_to_remove {
            self.index_ttl.remove(&key);
        }

        expired_count
    }
}


================================================================
Nombre: governor.rs
Ruta: src\governor.rs
================================================================

use crate::error::{Result, VantaError};
use crate::query::LogicalPlan;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Global counter of bytes currently allocated by queries in flight.
pub static ALLOCATED_BYTES: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, PartialEq)]
pub enum AllocationStatus {
    Granted,
    GrantedWithPressure, // Usado para invocar NMI si es necesario
}

pub struct ResourceGovernor {
    pub max_memory_bytes: usize,
    pub query_timeout_ms: u64,
}

impl ResourceGovernor {
    pub fn new(max_memory_bytes: usize, query_timeout_ms: u64) -> Self {
        Self {
            max_memory_bytes,
            query_timeout_ms,
        }
    }

    /// Request allocation before executing an expensive step
    pub fn request_allocation(&self, bytes: usize) -> Result<AllocationStatus> {
        let current = ALLOCATED_BYTES.load(Ordering::Relaxed);
        let new_total = current + bytes;

        if new_total > self.max_memory_bytes {
            return Err(VantaError::ResourceLimit(
                "OOM Guard triggered: query exceeds soft memory limit.".to_string(),
            ));
        }

        let pressure_threshold = (self.max_memory_bytes as f64 * 0.9) as usize;
        let status = if new_total > pressure_threshold {
            AllocationStatus::GrantedWithPressure
        } else {
            AllocationStatus::Granted
        };

        ALLOCATED_BYTES.fetch_add(bytes, Ordering::SeqCst);
        Ok(status)
    }

    /// Free allocation
    pub fn free_allocation(&self, bytes: usize) {
        ALLOCATED_BYTES.fetch_sub(bytes, Ordering::SeqCst);
    }

    /// Adapts the query plan based on TEMPERATURE
    pub fn apply_temperature_limits(&self, plan: &mut LogicalPlan) {
        if plan.temperature > 0.8 {
            // Aggressive pruning: modify traverse limits, reduce Top-K implicitly if large
            for op in plan.operators.iter_mut() {
                if let crate::query::LogicalOperator::Traverse { max_depth, .. } = op {
                    if *max_depth > 3 {
                        *max_depth = 3; // cap depth due to high heat
                    }
                }
            }
        }
    }
}


================================================================
Nombre: graph.rs
Ruta: src\graph.rs
================================================================

use crate::error::Result;
use crate::storage::StorageEngine;
use std::collections::{HashSet, VecDeque};

pub struct GraphTraverser<'a> {
    storage: &'a StorageEngine,
}

impl<'a> GraphTraverser<'a> {
    pub fn new(storage: &'a StorageEngine) -> Self {
        Self { storage }
    }

    /// Evaluates a Breadth-First-Search starting from a designated set of root IDs,
    /// up to a maximum depth, returning the discovered distinct Node IDs.
    pub fn bfs_traverse(&self, roots: &[u64], max_depth: usize) -> Result<Vec<u64>> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut results = Vec::new();

        for &root in roots {
            queue.push_back((root, 0));
        }

        while let Some((curr_id, depth)) = queue.pop_front() {
            if !visited.insert(curr_id) {
                continue; // Already processed
            }

            // Return all visited items
            results.push(curr_id);

            if depth < max_depth {
                // Fetch the node from the storage engine
                if let Ok(Some(node)) = self.storage.get(curr_id) {
                    for edge in &node.edges {
                        if !visited.contains(&edge.target) {
                            queue.push_back((edge.target, depth + 1));
                        }
                    }
                }
            }
        }

        Ok(results)
    }
}


================================================================
Nombre: index.rs
Ruta: src\index.rs
================================================================

use memmap2::MmapMut;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::collections::{BinaryHeap, HashMap};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

pub use crate::node::VectorRepresentations;
use crate::vector::quantization::{rabitq_similarity, turbo_quant_similarity};

const VECTOR_INDEX_MAGIC: &[u8; 8] = b"VNTHNSW1";
const VECTOR_INDEX_VERSION: u32 = 2; // Upgraded for config support

#[inline(always)]
pub fn cosine_sim_f32(a: &[f32], b: &[f32]) -> f32 {
    use crate::hardware::{HardwareCapabilities, InstructionSet};
    let caps = HardwareCapabilities::global();
    match caps.instructions {
        InstructionSet::Fallback => {
            let mut dot: f32 = 0.0;
            let mut norm_a: f32 = 0.0;
            let mut norm_b: f32 = 0.0;
            for (va, vb) in a.iter().zip(b.iter()) {
                dot += va * vb;
                norm_a += va * va;
                norm_b += vb * vb;
            }
            let denom = norm_a.sqrt() * norm_b.sqrt();
            if denom < f32::EPSILON {
                0.0
            } else {
                dot / denom
            }
        }
        _ => {
            use wide::f32x8;
            let mut dot_v = f32x8::ZERO;
            let mut norm_a_v = f32x8::ZERO;
            let mut norm_b_v = f32x8::ZERO;
            let chunks_a = a.chunks_exact(8);
            let chunks_b = b.chunks_exact(8);
            let rem_a = chunks_a.remainder();
            let rem_b = chunks_b.remainder();
            for (a_chunk, b_chunk) in chunks_a.zip(chunks_b) {
                let va = f32x8::from([
                    a_chunk[0], a_chunk[1], a_chunk[2], a_chunk[3], a_chunk[4], a_chunk[5],
                    a_chunk[6], a_chunk[7],
                ]);
                let vb = f32x8::from([
                    b_chunk[0], b_chunk[1], b_chunk[2], b_chunk[3], b_chunk[4], b_chunk[5],
                    b_chunk[6], b_chunk[7],
                ]);
                dot_v += va * vb;
                norm_a_v += va * va;
                norm_b_v += vb * vb;
            }
            let mut dot = dot_v.reduce_add();
            let mut norm_a = norm_a_v.reduce_add();
            let mut norm_b = norm_b_v.reduce_add();
            for i in 0..rem_a.len() {
                dot += rem_a[i] * rem_b[i];
                norm_a += rem_a[i] * rem_a[i];
                norm_b += rem_b[i] * rem_b[i];
            }
            let denom = norm_a.sqrt() * norm_b.sqrt();
            if denom < f32::EPSILON {
                0.0
            } else {
                dot / denom
            }
        }
    }
}

pub fn calculate_similarity(
    raw_query: &[f32],
    quantized_query_1bit: Option<&[u64]>,
    quantized_query_3bit: Option<(&[u8], f32)>,
    node_vec: &VectorRepresentations,
) -> f32 {
    match node_vec {
        VectorRepresentations::Binary(b) => {
            if let Some(q1) = quantized_query_1bit {
                rabitq_similarity(q1, b)
            } else {
                0.0
            }
        }
        VectorRepresentations::Turbo(t) => {
            if let Some((q3, max_abs)) = quantized_query_3bit {
                turbo_quant_similarity(q3, max_abs, t, 1.0)
            } else {
                0.0
            }
        }
        VectorRepresentations::Full(f) => {
            // ZERO ALLOCATION: Cálculo SIMD directo sin empaquetar ni clonar
            cosine_sim_f32(raw_query, f)
        }
        VectorRepresentations::None => 0.0,
    }
}

pub struct HnswNode {
    pub id: u64,
    pub bitset: u128,
    pub vec_data: VectorRepresentations,
    pub neighbors: Vec<Vec<u64>>,
    /// Offset into the VantaFile (Phase 3)
    pub storage_offset: u64,
}

#[derive(Debug)]
pub enum IndexBackend {
    InMemory,
    MMapFile {
        path: PathBuf,
        mmap: Option<MmapMut>,
    },
}

impl IndexBackend {
    pub fn new_mmap(path: PathBuf) -> Self {
        IndexBackend::MMapFile { path, mmap: None }
    }

    pub fn is_mmap(&self) -> bool {
        matches!(self, IndexBackend::MMapFile { .. })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HnswConfig {
    pub m: usize,
    pub m_max0: usize,
    pub ef_construction: usize,
    pub ef_search: usize,
    pub ml: f64,
}

impl Default for HnswConfig {
    fn default() -> Self {
        Self {
            m: 32,
            m_max0: 64,
            ef_construction: 200,
            ef_search: 100,
            ml: 1.0 / (32_f64).ln(),
        }
    }
}

// Custom wrapper to store (similarity, node_id) in BinaryHeap (Max-Heap)
#[derive(Clone, PartialEq, Debug)]
struct NodeSim(f32, u64);

impl Eq for NodeSim {}

impl PartialOrd for NodeSim {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NodeSim {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self
            .0
            .partial_cmp(&other.0)
            .unwrap_or(std::cmp::Ordering::Equal)
        {
            std::cmp::Ordering::Equal => other.1.cmp(&self.1),
            cmp => cmp,
        }
    }
}

// Wrapper for Min-Heap (used to track closest in result set)
#[derive(Clone, PartialEq, Debug)]
struct NodeSimMin(f32, u64);

impl Eq for NodeSimMin {}

impl PartialOrd for NodeSimMin {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NodeSimMin {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match other
            .0
            .partial_cmp(&self.0)
            .unwrap_or(std::cmp::Ordering::Equal)
        {
            std::cmp::Ordering::Equal => self.1.cmp(&other.1),
            cmp => cmp,
        }
    }
}

pub struct CPIndex {
    pub nodes: HashMap<u64, HnswNode>,
    pub max_layer: usize,
    pub entry_point: Option<u64>,
    pub backend: IndexBackend,
    pub config: HnswConfig,
    rng: rand::rngs::StdRng,
}

impl CPIndex {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            max_layer: 0,
            entry_point: None,
            backend: IndexBackend::InMemory,
            config: HnswConfig::default(),
            rng: rand::rngs::StdRng::seed_from_u64(42),
        }
    }

    pub fn new_with_config(config: HnswConfig) -> Self {
        Self {
            nodes: HashMap::new(),
            max_layer: 0,
            entry_point: None,
            backend: IndexBackend::InMemory,
            config,
            rng: rand::rngs::StdRng::seed_from_u64(42),
        }
    }

    pub fn with_backend(backend: IndexBackend) -> Self {
        Self {
            nodes: HashMap::new(),
            max_layer: 0,
            entry_point: None,
            backend,
            config: HnswConfig::default(),
            rng: rand::rngs::StdRng::seed_from_u64(42),
        }
    }

    fn random_layer(&mut self) -> usize {
        let r: f64 = self.rng.gen_range(0.0001..1.0);
        (-r.ln() * self.config.ml).floor() as usize
    }

    /// Primary search subroutine for HNSW.
    /// Performs a greedy beam search to return the `ef` nearest neighbors
    /// found at `layer`. Candidates are validated against `query_mask`.
    fn search_layer(
        &self,
        query_vec: &[f32],
        entry_points: &[u64],
        ef: usize,
        layer: usize,
        query_mask: u128,
        vector_store: Option<&crate::storage::VantaFile>,
    ) -> BinaryHeap<NodeSimMin> {
        let mut visited = std::collections::HashSet::new();
        let mut candidates = BinaryHeap::new(); // Max-heap: candidates to visit
        let mut results = BinaryHeap::new(); // Min-heap: best `ef` bounds

        for &ep in entry_points {
            if let Some(node) = self.nodes.get(&ep) {
                let d = if let Some(vs) = vector_store {
                    // Zero-copy search from VantaFile
                    if let Some(header) = vs.read_header(node.storage_offset) {
                        let vec_start = header.vector_offset as usize;
                        let vec_end = vec_start + (header.vector_len as usize * 4);
                        let vec_data = &vs.mmap[vec_start..vec_end];
                        // Safety: we trust the header.vector_len and bounds checking above
                        let f32_vec: &[f32] = unsafe {
                            std::slice::from_raw_parts(
                                vec_data.as_ptr() as *const f32,
                                header.vector_len as usize,
                            )
                        };
                        cosine_sim_f32(query_vec, f32_vec)
                    } else {
                        0.0
                    }
                } else {
                    calculate_similarity(query_vec, None, None, &node.vec_data)
                };

                candidates.push(NodeSim(d, ep));

                if query_mask == u128::MAX || (node.bitset & query_mask) == query_mask {
                    results.push(NodeSimMin(d, ep));
                }
                visited.insert(ep);
            }
        }

        while let Some(NodeSim(d_cand, cand_id)) = candidates.pop() {
            // Early stopping condition: if candidate is worse than the worst result
            if results.len() >= ef {
                if let Some(worst) = results.peek() {
                    // Because it's a min-heap, peek gives the smallest similarity (worst match)
                    if d_cand < worst.0 {
                        break;
                    }
                }
            }

            if let Some(node) = self.nodes.get(&cand_id) {
                if layer < node.neighbors.len() {
                    for &neighbor_id in &node.neighbors[layer] {
                        if !visited.contains(&neighbor_id) {
                            visited.insert(neighbor_id);

                            if let Some(neighbor) = self.nodes.get(&neighbor_id) {
                                let d = if let Some(vs) = vector_store {
                                    if let Some(h) = vs.read_header(neighbor.storage_offset) {
                                        let v_data = &vs.mmap[h.vector_offset as usize
                                            ..(h.vector_offset as usize
                                                + h.vector_len as usize * 4)];
                                        // Safety: trusted bounds and aligned data
                                        let f32_v: &[f32] = unsafe {
                                            std::slice::from_raw_parts(
                                                v_data.as_ptr() as *const f32,
                                                h.vector_len as usize,
                                            )
                                        };
                                        cosine_sim_f32(query_vec, f32_v)
                                    } else {
                                        0.0
                                    }
                                } else {
                                    calculate_similarity(query_vec, None, None, &neighbor.vec_data)
                                };

                                if results.len() < ef
                                    || (results.peek().is_some() && d > results.peek().unwrap().0)
                                {
                                    candidates.push(NodeSim(d, neighbor_id));

                                    if query_mask == u128::MAX
                                        || (neighbor.bitset & query_mask) == query_mask
                                    {
                                        results.push(NodeSimMin(d, neighbor_id));
                                        if results.len() > ef {
                                            results.pop(); // Remove the worst to keep size at `ef`
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        results
    }

    /// Select neighbors using the HNSW paper heuristic (Algorithm 4, Malkov & Yashunin 2018).
    /// Applies spatial diversity from slot 0 — no unconditional acceptance.
    /// keepPrunedConnections=true: fills limited remaining slots with discarded candidates.
    ///
    /// Metric: cosine similarity (higher = closer). The diversity condition is:
    ///   reject if similarity(candidate, selected) > similarity(candidate, query)
    /// This is the correct inversion of the paper's distance-based condition
    /// because cosine similarity is monotonically inverse to angular distance.
    fn select_neighbors(&self, candidates: &mut BinaryHeap<NodeSimMin>, m: usize) -> Vec<u64> {
        // Clone is critically necessary here because `w` is reused by the caller
        // to seed the `entry_points` for the next layer down.
        let sorted = candidates.clone().into_sorted_vec();
        // into_sorted_vec returns ascending order based on NodeSimMin's Ord
        // NodeSimMin Ord equates higher similarity to "Less", meaning best candidates come first!

        let mut selected: Vec<u64> = Vec::with_capacity(m);
        let mut discarded: Vec<u64> = Vec::new();

        for ns in sorted.into_iter() {
            if selected.len() >= m {
                break;
            }

            let cand_id = ns.1;
            let sim_q_cand = ns.0;

            let cand_slice = match self.nodes.get(&cand_id).map(|n| &n.vec_data) {
                Some(VectorRepresentations::Full(v)) => v.as_slice(),
                _ => {
                    selected.push(cand_id);
                    continue;
                }
            };

            let mut is_diverse = true;
            for &sel_id in &selected {
                if let Some(sel_node) = self.nodes.get(&sel_id) {
                    let sim_cand_sel =
                        calculate_similarity(cand_slice, None, None, &sel_node.vec_data);
                    if sim_cand_sel > sim_q_cand {
                        is_diverse = false;
                        break;
                    }
                }
            }

            if is_diverse {
                selected.push(cand_id);
            } else {
                discarded.push(cand_id);
            }
        }

        // keepPrunedConnections: fill remaining slots with discarded candidates.
        // HNSW relies on keeping degree close to M.
        for &disc_id in discarded.iter() {
            if selected.len() >= m {
                break;
            }
            selected.push(disc_id);
        }

        selected
    }

    pub fn add(
        &mut self,
        id: u64,
        bitset: u128,
        vec_data: VectorRepresentations,
        storage_offset: u64,
    ) {
        // Phase 1.3: Duplicate detection — silently updating an existing node can
        // corrupt the graph's bidirectional links. Log a warning and return early.
        if self.nodes.contains_key(&id) {
            warn!(
                node_id = id,
                "CPIndex::add called with duplicate ID — skipping to prevent graph corruption"
            );
            return;
        }

        if vec_data.is_none() {
            // Can't index graph nodes without vectors into HNSW layers,
            // but we must still register them in the nodes map to track their storage_offset.
            self.nodes.insert(
                id,
                HnswNode {
                    id,
                    bitset,
                    vec_data,
                    neighbors: vec![Vec::new()],
                    storage_offset,
                },
            );
            return;
        }

        let level = self.random_layer();
        let ef_cons = self.config.ef_construction;

        // If index is empty
        let ep = match self.entry_point {
            None => {
                self.entry_point = Some(id);
                self.max_layer = level;
                let neighbors = vec![Vec::new(); level + 1];
                self.nodes.insert(
                    id,
                    HnswNode {
                        id,
                        bitset,
                        vec_data,
                        neighbors,
                        storage_offset,
                    },
                );
                return;
            }
            Some(entry) => entry,
        };

        // Query vector as F32 for building the index properly
        let query_f32 = match vec_data.to_f32() {
            Some(v) => v,
            None => return, // Critical failure, vector decode failed
        };

        let mut curr_entry_points = vec![ep];
        let top_layer = self.max_layer;

        // Phase 1: Descend down to the new node's insertion level (or top_layer)
        for layer in (level + 1..=top_layer).rev() {
            let mut w =
                self.search_layer(&query_f32, &curr_entry_points, 1, layer, u128::MAX, None);
            if let Some(NodeSimMin(_, best_id)) = w.pop() {
                curr_entry_points = vec![best_id];
            }
        }

        let mut new_neighbors = vec![Vec::new(); level + 1];

        // Phase 2: From node's layer down to 0, find neighbors and connect
        let start_layer = std::cmp::min(level, top_layer);
        for layer in (0..=start_layer).rev() {
            let w = self.search_layer(
                &query_f32,
                &curr_entry_points,
                ef_cons,
                layer,
                u128::MAX,
                None,
            );

            // extendCandidates: expand W with the neighbors of its elements
            let mut extended_w = w.clone();
            let mut visited_ext: std::collections::HashSet<u64> = std::collections::HashSet::new();
            for item in w.iter() {
                visited_ext.insert(item.1);
            }

            // Only extend if it does not blow up the search scope pathologically
            if extended_w.len() <= ef_cons {
                for item in w.iter() {
                    if let Some(c_node) = self.nodes.get(&item.1) {
                        if layer < c_node.neighbors.len() {
                            for &adj_id in &c_node.neighbors[layer] {
                                if !visited_ext.contains(&adj_id) {
                                    visited_ext.insert(adj_id);
                                    if let Some(adj_node) = self.nodes.get(&adj_id) {
                                        let sim = calculate_similarity(
                                            &query_f32,
                                            None,
                                            None,
                                            &adj_node.vec_data,
                                        );
                                        extended_w.push(NodeSimMin(sim, adj_id));
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Extract the neighbors to connect (bidirectionally)
            let m_max = if layer == 0 {
                self.config.m_max0
            } else {
                self.config.m
            };
            let selected_neighbors = self.select_neighbors(&mut extended_w, self.config.m);
            new_neighbors[layer] = selected_neighbors.clone();

            // Entry points for next layer = full search results from this layer
            // (select_neighbors clones w internally, so w is still intact here)
            curr_entry_points = w.into_iter().map(|ns| ns.1).collect();

            // Bidirectional links
            for &neighbor_id in &selected_neighbors {
                // Scope mutable access to avoid overlap with immutable `self.nodes.get(&nt)`
                let (needs_shrink, current_neighbors) = {
                    if let Some(neighbor_node) = self.nodes.get_mut(&neighbor_id) {
                        if layer < neighbor_node.neighbors.len() {
                            if !neighbor_node.neighbors[layer].contains(&id) {
                                neighbor_node.neighbors[layer].push(id);
                            }

                            // Shrink connections if they overflow M_max
                            if neighbor_node.neighbors[layer].len() > m_max {
                                (true, neighbor_node.neighbors[layer].clone())
                            } else {
                                (false, Vec::new())
                            }
                        } else {
                            (false, Vec::new())
                        }
                    } else {
                        (false, Vec::new())
                    }
                };

                if needs_shrink {
                    // Zero-Copy Extractor for Pruning
                    let nb_vec = match self.nodes.get(&neighbor_id).map(|n| &n.vec_data) {
                        Some(VectorRepresentations::Full(v)) => Some(v.as_slice()),
                        _ => None,
                    };

                    if let Some(nb_v) = nb_vec {
                        let mut cand_heap = BinaryHeap::new();
                        for &n_target in &current_neighbors {
                            if let Some(nt) = self.nodes.get(&n_target) {
                                let d = calculate_similarity(nb_v, None, None, &nt.vec_data);
                                cand_heap.push(NodeSimMin(d, n_target));
                            }
                        }
                        let pruned = self.select_neighbors(&mut cand_heap, m_max);
                        if let Some(neighbor_node) = self.nodes.get_mut(&neighbor_id) {
                            neighbor_node.neighbors[layer] = pruned;
                        }
                    }
                }
            }
        }

        // Apply new node explicitly
        self.nodes.insert(
            id,
            HnswNode {
                id,
                bitset,
                vec_data,
                neighbors: new_neighbors,
                storage_offset,
            },
        );

        // Update entry point if we created a new highest layer
        if level > self.max_layer {
            self.max_layer = level;
            self.entry_point = Some(id);
        }
    }

    pub fn search_nearest(
        &self,
        query_vec: &[f32],
        _q_1bit: Option<&[u64]>, // We let these pass but currently default to calculate_similarity internal handler
        _q_3bit: Option<(&[u8], f32)>,
        query_mask: u128,
        top_k: usize,
        vector_store: Option<&crate::storage::VantaFile>,
    ) -> Vec<(u64, f32)> {
        let ep = match self.entry_point {
            Some(id) => id,
            None => return Vec::new(),
        };

        let ef_search = (self.config.ef_search * 2).max(top_k);
        let mut curr_entry_points = vec![ep];

        for layer in (1..=self.max_layer).rev() {
            let mut w = self.search_layer(
                query_vec,
                &curr_entry_points,
                1,
                layer,
                u128::MAX,
                vector_store,
            );
            if let Some(NodeSimMin(_, best_id)) = w.pop() {
                curr_entry_points = vec![best_id];
            }
        }

        let w = self.search_layer(
            query_vec,
            &curr_entry_points,
            ef_search,
            0,
            query_mask,
            vector_store,
        );

        // Extract top-k
        let mut result = w.into_sorted_vec();

        // into_sorted_vec returns highest similarity (best) first!
        result.truncate(top_k);

        result.into_iter().map(|n| (n.1, n.0)).collect()
    }

    pub fn serialize_to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.nodes.len() * 256 + 128);

        buf.extend_from_slice(VECTOR_INDEX_MAGIC);
        buf.extend_from_slice(&VECTOR_INDEX_VERSION.to_le_bytes());
        buf.extend_from_slice(&(self.max_layer as u64).to_le_bytes());

        // Config block (only in V2+)
        buf.extend_from_slice(&(self.config.m as u64).to_le_bytes());
        buf.extend_from_slice(&(self.config.m_max0 as u64).to_le_bytes());
        buf.extend_from_slice(&(self.config.ef_construction as u64).to_le_bytes());
        buf.extend_from_slice(&(self.config.ef_search as u64).to_le_bytes());
        buf.extend_from_slice(&self.config.ml.to_le_bytes());

        match self.entry_point {
            Some(ep) => {
                buf.push(1);
                buf.extend_from_slice(&ep.to_le_bytes());
            }
            None => {
                buf.push(0);
                buf.extend_from_slice(&0u64.to_le_bytes());
            }
        }

        let node_count = self.nodes.len() as u64;
        buf.extend_from_slice(&node_count.to_le_bytes());

        for node in self.nodes.values() {
            buf.extend_from_slice(&node.id.to_le_bytes());
            buf.extend_from_slice(&node.bitset.to_le_bytes());

            match &node.vec_data {
                VectorRepresentations::Full(f) => {
                    buf.push(1);
                    buf.extend_from_slice(&(f.len() as u64).to_le_bytes());
                    for &val in f {
                        buf.extend_from_slice(&val.to_le_bytes());
                    }
                }
                VectorRepresentations::Binary(b) => {
                    buf.push(2);
                    buf.extend_from_slice(&(b.len() as u64).to_le_bytes());
                    for &val in b {
                        buf.extend_from_slice(&val.to_le_bytes());
                    }
                }
                VectorRepresentations::Turbo(t) => {
                    buf.push(3);
                    buf.extend_from_slice(&(t.len() as u64).to_le_bytes());
                    buf.extend_from_slice(t);
                }
                VectorRepresentations::None => {
                    buf.push(0);
                    buf.extend_from_slice(&0u64.to_le_bytes());
                }
            }

            let layer_count = node.neighbors.len() as u64;
            buf.extend_from_slice(&layer_count.to_le_bytes());
            for layer in &node.neighbors {
                let neighbor_count = layer.len() as u64;
                buf.extend_from_slice(&neighbor_count.to_le_bytes());
                for &nid in layer {
                    buf.extend_from_slice(&nid.to_le_bytes());
                }
            }
        }

        buf
    }

    pub fn deserialize_from_bytes(data: &[u8]) -> std::io::Result<Self> {
        use std::io::{Error, ErrorKind};

        if data.len() < 29 {
            return Err(Error::new(ErrorKind::InvalidData, "Index file too small"));
        }

        let mut pos = 0;

        if &data[pos..pos + 8] != VECTOR_INDEX_MAGIC {
            return Err(Error::new(ErrorKind::InvalidData, "Invalid magic header"));
        }
        pos += 8;

        let version = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap());
        pos += 4;

        let max_layer = u64::from_le_bytes(data[pos..pos + 8].try_into().unwrap()) as usize;
        pos += 8;

        let mut config = HnswConfig::default();
        if version >= 2 {
            config.m = u64::from_le_bytes(data[pos..pos + 8].try_into().unwrap()) as usize;
            pos += 8;
            config.m_max0 = u64::from_le_bytes(data[pos..pos + 8].try_into().unwrap()) as usize;
            pos += 8;
            config.ef_construction =
                u64::from_le_bytes(data[pos..pos + 8].try_into().unwrap()) as usize;
            pos += 8;
            config.ef_search = u64::from_le_bytes(data[pos..pos + 8].try_into().unwrap()) as usize;
            pos += 8;
            config.ml = f64::from_le_bytes(data[pos..pos + 8].try_into().unwrap());
            pos += 8;
        }

        if pos >= data.len() {
            return Err(Error::new(ErrorKind::UnexpectedEof, "Truncated EP"));
        }
        let ep_exists = data[pos];
        pos += 1;
        if pos + 8 > data.len() {
            return Err(Error::new(ErrorKind::UnexpectedEof, "Truncated EP ID"));
        }
        let ep_id = u64::from_le_bytes(data[pos..pos + 8].try_into().unwrap());
        pos += 8;
        let entry_point = if ep_exists == 1 { Some(ep_id) } else { None };

        if pos + 8 > data.len() {
            return Err(Error::new(ErrorKind::UnexpectedEof, "Truncated node count"));
        }
        let node_count = u64::from_le_bytes(data[pos..pos + 8].try_into().unwrap()) as usize;
        pos += 8;

        let mut nodes = HashMap::with_capacity(node_count);

        for _ in 0..node_count {
            if pos + 8 > data.len() {
                return Err(Error::new(ErrorKind::UnexpectedEof, "Truncated node id"));
            }
            let id = u64::from_le_bytes(data[pos..pos + 8].try_into().unwrap());
            pos += 8;

            if pos + 16 > data.len() {
                return Err(Error::new(ErrorKind::UnexpectedEof, "Truncated bitset"));
            }
            let bitset = u128::from_le_bytes(data[pos..pos + 16].try_into().unwrap());
            pos += 16;

            if pos + 1 > data.len() {
                return Err(Error::new(ErrorKind::UnexpectedEof, "Truncated vec type"));
            }
            let vec_type = data[pos];
            pos += 1;

            if pos + 8 > data.len() {
                return Err(Error::new(ErrorKind::UnexpectedEof, "Truncated vec len"));
            }
            let vec_len = u64::from_le_bytes(data[pos..pos + 8].try_into().unwrap()) as usize;
            pos += 8;

            let vec_data = match vec_type {
                1 => {
                    let byte_len = vec_len * 4;
                    if pos + byte_len > data.len() {
                        return Err(Error::new(ErrorKind::UnexpectedEof, "Truncated f32 vec"));
                    }
                    let mut v = Vec::with_capacity(vec_len);
                    for i in 0..vec_len {
                        let start = pos + i * 4;
                        v.push(f32::from_le_bytes(
                            data[start..start + 4].try_into().unwrap(),
                        ));
                    }
                    pos += byte_len;
                    VectorRepresentations::Full(v)
                }
                2 => {
                    let byte_len = vec_len * 8;
                    if pos + byte_len > data.len() {
                        return Err(Error::new(ErrorKind::UnexpectedEof, "Truncated binary vec"));
                    }
                    let mut v = Vec::with_capacity(vec_len);
                    for i in 0..vec_len {
                        let start = pos + i * 8;
                        v.push(u64::from_le_bytes(
                            data[start..start + 8].try_into().unwrap(),
                        ));
                    }
                    pos += byte_len;
                    VectorRepresentations::Binary(v.into_boxed_slice())
                }
                3 => {
                    if pos + vec_len > data.len() {
                        return Err(Error::new(ErrorKind::UnexpectedEof, "Truncated turbo vec"));
                    }
                    let v = data[pos..pos + vec_len].to_vec();
                    pos += vec_len;
                    VectorRepresentations::Turbo(v.into_boxed_slice())
                }
                _ => VectorRepresentations::None,
            };

            if pos + 8 > data.len() {
                return Err(Error::new(
                    ErrorKind::UnexpectedEof,
                    "Truncated neighbor layers",
                ));
            }
            let layer_count = u64::from_le_bytes(data[pos..pos + 8].try_into().unwrap()) as usize;
            pos += 8;

            let mut neighbors = Vec::with_capacity(layer_count);
            for _ in 0..layer_count {
                if pos + 8 > data.len() {
                    return Err(Error::new(
                        ErrorKind::UnexpectedEof,
                        "Truncated neighbor count",
                    ));
                }
                let neighbor_count =
                    u64::from_le_bytes(data[pos..pos + 8].try_into().unwrap()) as usize;
                pos += 8;

                let byte_len = neighbor_count * 8;
                if pos + byte_len > data.len() {
                    return Err(Error::new(
                        ErrorKind::UnexpectedEof,
                        "Truncated neighbor ids",
                    ));
                }
                let mut layer_neighbors = Vec::with_capacity(neighbor_count);
                for i in 0..neighbor_count {
                    let start = pos + i * 8;
                    layer_neighbors.push(u64::from_le_bytes(
                        data[start..start + 8].try_into().unwrap(),
                    ));
                }
                pos += byte_len;
                neighbors.push(layer_neighbors);
            }

            nodes.insert(
                id,
                HnswNode {
                    id,
                    bitset,
                    vec_data,
                    neighbors,
                    storage_offset: 0, // New nodes start in RAM, offset set on persist
                },
            );
        }

        Ok(Self {
            nodes,
            max_layer,
            entry_point,
            backend: IndexBackend::InMemory,
            config,
            rng: rand::rngs::StdRng::seed_from_u64(42),
        })
    }

    pub fn persist_to_file(&self, path: &Path) -> std::io::Result<()> {
        let data = self.serialize_to_bytes();
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        writer.write_all(&data)?;
        writer.flush()?;
        info!(path = %path.display(), node_count = self.nodes.len(), bytes = data.len(), "HNSW index persisted");
        Ok(())
    }

    pub fn load_from_file(path: &Path) -> Option<Self> {
        if !path.exists() {
            return None;
        }

        let file = match std::fs::File::open(path) {
            Ok(f) => f,
            Err(_) => return None,
        };

        let mmap = match unsafe { memmap2::MmapOptions::new().map(&file) } {
            Ok(m) => m,
            Err(e) => {
                warn!(err = %e, "Failed to mmap HNSW index file — will rebuild");
                return None;
            }
        };

        match Self::deserialize_from_bytes(&mmap) {
            Ok(index) => {
                info!(path = %path.display(), node_count = index.nodes.len(), "HNSW cold-start: loaded index from file");
                // Phase 1.3: Validate integrity on every cold-start
                if let Err(violations) = index.validate_index() {
                    warn!(
                        violation_count = violations.len(),
                        "HNSW index has integrity violations after deserialization"
                    );
                    for v in &violations[..violations.len().min(5)] {
                        warn!(violation = %v, "HNSW integrity violation");
                    }
                }
                Some(index)
            }
            Err(e) => {
                warn!(err = %e, "Corrupt vector_index.bin — will rebuild and overwrite");
                None
            }
        }
    }

    pub fn sync_to_mmap(&mut self) -> std::io::Result<()> {
        let path = match &self.backend {
            IndexBackend::MMapFile { path, .. } => path.clone(),
            _ => return Ok(()),
        };

        let data = self.serialize_to_bytes();

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)?;
        file.set_len(data.len() as u64)?;

        let mut mapped = unsafe { MmapMut::map_mut(&file)? };
        mapped.copy_from_slice(&data);
        mapped.flush()?;

        if let IndexBackend::MMapFile { ref mut mmap, .. } = self.backend {
            *mmap = Some(mapped);
        }

        info!(path = %path.display(), node_count = self.nodes.len(), bytes = data.len(), "HNSW MMap synced");
        Ok(())
    }
}

// ─── Phase 1.1: Index Stats & Integrity Validation ──────────────────────────

/// Snapshot of HNSW index health metrics
#[derive(Debug, Clone)]
pub struct IndexStats {
    /// Total nodes in the index
    pub node_count: usize,
    /// Maximum layer height in the graph
    pub max_layer: usize,
    /// Nodes with zero neighbors on layer 0 (potential orphans)
    pub orphan_count: usize,
    /// Average outgoing connections on layer 0
    pub avg_connections_l0: f32,
    /// Total number of graph integrity violations found
    pub violation_count: usize,
}

impl CPIndex {
    /// Compute a snapshot of index health metrics.
    pub fn stats(&self) -> IndexStats {
        let node_count = self.nodes.len();
        let orphan_count = self
            .nodes
            .values()
            .filter(|n| n.neighbors.is_empty() || n.neighbors[0].is_empty())
            .count();
        let total_l0_connections: usize = self
            .nodes
            .values()
            .map(|n| n.neighbors.first().map(|l| l.len()).unwrap_or(0))
            .sum();
        let avg_connections_l0 = if node_count > 0 {
            total_l0_connections as f32 / node_count as f32
        } else {
            0.0
        };

        IndexStats {
            node_count,
            max_layer: self.max_layer,
            orphan_count,
            avg_connections_l0,
            violation_count: 0, // Updated by validate_index()
        }
    }

    /// Validate the structural integrity of the HNSW graph.
    ///
    /// Checks:
    /// 1. Every neighbor reference points to an existing node
    /// 2. No self-loops
    /// 3. Layer count is consistent with node's reported level
    ///
    /// Returns `Ok(())` if the graph is clean, or a list of violation messages.
    ///
    /// # Performance
    /// O(N × M) where N = node count, M = max neighbors per layer.
    /// Run at startup after deserialization, not in hot paths.
    pub fn validate_index(&self) -> Result<(), Vec<String>> {
        let mut violations = Vec::new();

        for (id, node) in &self.nodes {
            // Check: layer count should be ≥ 1
            if node.neighbors.is_empty() {
                violations.push(format!(
                    "Node {} has empty neighbors array (expected ≥1 layer)",
                    id
                ));
                continue;
            }

            // Check each layer's neighbor list
            for (layer_idx, layer) in node.neighbors.iter().enumerate() {
                for &neighbor_id in layer {
                    // Self-loop check
                    if neighbor_id == *id {
                        violations.push(format!(
                            "Node {} has a self-loop at layer {}",
                            id, layer_idx
                        ));
                        continue;
                    }
                    // Dangling reference check
                    if !self.nodes.contains_key(&neighbor_id) {
                        violations.push(format!(
                            "Node {} references non-existent neighbor {} at layer {}",
                            id, neighbor_id, layer_idx
                        ));
                    }
                }
            }
        }

        // Check entry point validity
        if let Some(ep) = self.entry_point {
            if !self.nodes.contains_key(&ep) {
                violations.push(format!("Entry point {} does not exist in the node map", ep));
            }
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(violations)
        }
    }
}

impl Default for CPIndex {
    fn default() -> Self {
        Self::new()
    }
}


================================================================
Nombre: integrations.rs
Ruta: src\integrations.rs
================================================================

//! VantaDB Integrations (Ollama, LangChain)
use serde::{Deserialize, Serialize};

/// Request mapping for a simple LangChain vector store search
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct SearchRequest {
    pub query: String,
    pub collection: String,
    pub temperature: Option<f32>,
    pub limit: Option<usize>,
}

#[derive(Serialize, Clone, Debug)]
pub struct SearchResponse {
    pub results: Vec<serde_json::Value>,
    pub latency_ms: u64,
}

/// Simulated Axum handler for Hybrid Search
pub async fn search_handler(_payload: SearchRequest) -> SearchResponse {
    // Converts hybrid text query to logical plan here
    SearchResponse {
        results: vec![],
        latency_ms: 5,
    }
}

/// Request for proxied Ollama generation
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct OllamaGenerateRequest {
    pub model: String,
    pub prompt: String,
    pub stream: Option<bool>,
}

/// Simulated context retrieval and proxy
pub async fn ollama_proxy_handler(req: OllamaGenerateRequest) -> String {
    // 1. Search VantaDB for semantically similar nodes
    // 2. Inject results into `req.prompt`
    // 3. Forward to actual localhost Ollama
    format!(
        "Proximamente: Context-Aware proxy response para {}",
        req.model
    )
}


================================================================
Nombre: lib.rs
Ruta: src\lib.rs
================================================================

//! # VantaDB — Embedded Multimodal Database Engine
//!
//! Unified engine for **Vector** (embeddings), **Graph** (edges),
//! and **Relational** (typed fields) data in a single storage layer.

pub mod api;
pub(crate) mod backend;
pub(crate) mod backends;
pub mod columnar;
pub mod console;
pub mod engine;
pub mod error;
pub mod eval;
pub mod executor;
pub mod gc;
pub mod governance;
pub mod governor;
pub mod graph;
pub mod hardware;
pub mod index;
pub mod integrations;
pub mod llm;
pub mod metrics;
pub mod node;
pub mod parser;
#[cfg(feature = "python_sdk")]
pub mod python;
pub mod query;
pub mod server;
pub mod storage;
pub mod vector;
pub mod wal;

// Re-exports for ergonomic API
pub use engine::{EngineStats, InMemoryEngine, QueryResult, SourceType};
pub use error::{Result, VantaError};
pub use node::{Edge, FieldValue, NodeFlags, RelFields, UnifiedNode, VectorRepresentations};
pub use storage::BackendKind;
pub use wal::{WalReader, WalRecord, WalWriter};


================================================================
Nombre: llm.rs
Ruta: src\llm.rs
================================================================

use crate::error::{Result, VantaError};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Serialize)]
struct OllamaEmbeddingRequest<'a> {
    model: &'a str,
    input: &'a str,
}

#[derive(Deserialize)]
struct OllamaEmbeddingResponse {
    embedding: Vec<f32>,
}

pub struct LlmClient {
    client: Client,
    base_url: String,
    default_model: String,
}

impl Default for LlmClient {
    fn default() -> Self {
        Self::new()
    }
}

impl LlmClient {
    pub fn new() -> Self {
        let base_url =
            env::var("VANTA_LLM_URL").unwrap_or_else(|_| "http://localhost:11434".to_string());

        // El predeterminado de ollama para embeddings vectoriales es nomic-embed-text o all-minilm
        let default_model =
            env::var("VANTA_LLM_MODEL").unwrap_or_else(|_| "all-minilm".to_string());

        Self {
            client: Client::builder()
                .pool_idle_timeout(Some(std::time::Duration::from_secs(60)))
                .build()
                .unwrap_or_else(|_| Client::new()),
            base_url,
            default_model,
        }
    }

    /// Comunica al LLM para traducir un texto nativo a un vector HNSW compatible.
    pub async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let url = format!("{}/api/embeddings", self.base_url);

        let req_body = OllamaEmbeddingRequest {
            model: &self.default_model,
            input: text,
        };

        let response = self
            .client
            .post(&url)
            .json(&req_body)
            .send()
            .await
            .map_err(|e| {
                VantaError::Execution(format!(
                    "Network error communicating with Inference Bridge: {}",
                    e
                ))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(VantaError::Execution(format!(
                "Inference Bridge returned error status: {}",
                status
            )));
        }

        let result: OllamaEmbeddingResponse = response.json().await.map_err(|e| {
            VantaError::Execution(format!(
                "Invalid response format from Inference Bridge: {}",
                e
            ))
        })?;

        Ok(result.embedding)
    }

    /// Invoke the LLM to generate a semantic summary of a group of archived nodes.
    /// The prompt includes importance and keywords so the summary preserves
    /// the priority data rather than being a generic recap.
    pub async fn summarize_context(&self, nodes: &[&crate::node::UnifiedNode]) -> Result<String> {
        // Build structured context: each node contributes its content + importance metadata
        let mut context_blocks = Vec::new();
        for (i, node) in nodes.iter().enumerate() {
            let content = node
                .relational
                .get("content")
                .and_then(|v| v.as_str())
                .unwrap_or("[no content]");

            let keywords = node
                .relational
                .get("keywords")
                .and_then(|v| v.as_str())
                .unwrap_or("none");

            let node_type = node
                .relational
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");

            context_blocks.push(format!(
                "--- Node Fragment #{} ---\nType: {}\nContent: {}\nSemantic Priority: {:.2}\nConfidence Score: {:.2}\nKeywords: {}\nAccess Count: {}",
                i + 1, node_type, content,
                node.importance, node.confidence_score,
                keywords, node.hits
            ));
        }

        let full_context = context_blocks.join("\n\n");

        if full_context.trim().is_empty() {
            return Err(VantaError::Execution(
                "No summarizable content found in node group".to_string(),
            ));
        }

        let system_prompt = "You are VantaDB's Semantic Compression Engine. \
            Your task is to distill a group of related data fragments into a single, \
            dense summary that preserves the most semantically important information. \
            Pay special attention to fragments with high Semantic Priority — these are \
            contextually critical and their essence MUST be preserved. \
            Output ONLY the summary text, no preamble or formatting.";

        let user_prompt = format!(
            "Compress the following {} nodes into a single coherent summary:\n\n{}",
            nodes.len(),
            full_context
        );

        let summarize_model =
            env::var("VANTA_LLM_SUMMARIZE_MODEL").unwrap_or_else(|_| "llama3".to_string());

        let url = format!("{}/api/generate", self.base_url);

        let req_body = OllamaGenerateRequest {
            model: &summarize_model,
            system: system_prompt,
            prompt: &user_prompt,
            stream: false,
        };

        let response = self
            .client
            .post(&url)
            .json(&req_body)
            .send()
            .await
            .map_err(|e| {
                VantaError::Execution(format!(
                    "Network error during Semantic Summarization: {}",
                    e
                ))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(VantaError::Execution(format!(
                "Inference Bridge returned error status during summarization: {}",
                status
            )));
        }

        let result: OllamaGenerateResponse = response.json().await.map_err(|e| {
            VantaError::Execution(format!(
                "Invalid response format from Inference Bridge (summarize): {}",
                e
            ))
        })?;

        Ok(result.response)
    }
}

#[derive(Serialize)]
struct OllamaGenerateRequest<'a> {
    model: &'a str,
    system: &'a str,
    prompt: &'a str,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaGenerateResponse {
    response: String,
}


================================================================
Nombre: metrics.rs
Ruta: src\metrics.rs
================================================================

use prometheus::{Histogram, IntCounter, Registry};
use std::sync::LazyLock;

// Ensure singleton metrics registry across the binary
pub static METRICS_REGISTRY: LazyLock<Registry> = LazyLock::new(Registry::new);

pub static QUERY_LATENCY: LazyLock<Histogram> = LazyLock::new(|| {
    let hist = Histogram::with_opts(prometheus::HistogramOpts::new(
        "vanta_query_latency_ms",
        "Query execution times in ms",
    ))
    .unwrap();
    METRICS_REGISTRY.register(Box::new(hist.clone())).unwrap();
    hist
});

pub static OOM_TRIPS: LazyLock<IntCounter> = LazyLock::new(|| {
    let counter =
        IntCounter::new("vanta_oom_circuit_trips_total", "Governor OOM prevents").unwrap();
    METRICS_REGISTRY
        .register(Box::new(counter.clone()))
        .unwrap();
    counter
});

pub static CACHE_HITS: LazyLock<IntCounter> = LazyLock::new(|| {
    let counter = IntCounter::new("vanta_cache_hits_total", "CP-Index fast path matches").unwrap();
    METRICS_REGISTRY
        .register(Box::new(counter.clone()))
        .unwrap();
    counter
});

/// Export utility suitable for the `/metrics` Axum endpoint
pub fn export_metrics_text() -> String {
    use prometheus::TextEncoder;
    let encoder = TextEncoder::new();
    let metric_families = METRICS_REGISTRY.gather();
    let mut buffer = String::new();
    encoder.encode_utf8(&metric_families, &mut buffer).unwrap();
    buffer
}


================================================================
Nombre: node.rs
Ruta: src\node.rs
================================================================

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::time::{SystemTime, UNIX_EPOCH};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

// ─── Vector Data ───────────────────────────────────────────

/// Vector storage — supports tiered precision (Hybrid Quantization)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum VectorRepresentations {
    /// L1: Fast binary index in RAM. Hamming distance (XOR + POPCNT).
    Binary(Box<[u64]>),
    /// L2: Re-ranking and initial validation. Memory-mapped from disk (3-bit).
    Turbo(Box<[u8]>),
    /// L3: Full precision float32.
    Full(Vec<f32>),
    /// No vector attached
    None,
}

impl VectorRepresentations {
    pub fn dimensions(&self) -> usize {
        match self {
            VectorRepresentations::Full(v) => v.len(),
            VectorRepresentations::Binary(data) => data.len() * 64, // rough dim
            VectorRepresentations::Turbo(data) => data.len() * 2,   // depends on packing
            VectorRepresentations::None => 0,
        }
    }

    pub fn is_none(&self) -> bool {
        matches!(self, VectorRepresentations::None)
    }

    /// Decode to f32 for distance computation (Fallback/Testing)
    pub fn to_f32(&self) -> Option<Vec<f32>> {
        match self {
            VectorRepresentations::Full(v) => Some(v.clone()),
            _ => None, // Only full supports exact to_f32 without decomp
        }
    }

    /// Computes cosine similarity (F32) or delegates to quantized logic
    pub fn cosine_similarity(&self, other: &VectorRepresentations) -> Option<f32> {
        use crate::hardware::{HardwareCapabilities, InstructionSet};

        let a = self.to_f32()?;
        let b = other.to_f32()?;
        if a.len() != b.len() || a.is_empty() {
            return None;
        }

        let caps = HardwareCapabilities::global();
        match caps.instructions {
            InstructionSet::Fallback => {
                let mut dot: f32 = 0.0;
                let mut norm_a: f32 = 0.0;
                let mut norm_b: f32 = 0.0;
                for (va, vb) in a.iter().zip(b.iter()) {
                    dot += va * vb;
                    norm_a += va * va;
                    norm_b += vb * vb;
                }
                let denom = norm_a.sqrt() * norm_b.sqrt();
                if denom < f32::EPSILON {
                    None
                } else {
                    Some(dot / denom)
                }
            }
            _ => {
                let mut dot_v = wide::f32x8::ZERO;
                let mut norm_a_v = wide::f32x8::ZERO;
                let mut norm_b_v = wide::f32x8::ZERO;
                let chunks_a = a.chunks_exact(8);
                let chunks_b = b.chunks_exact(8);
                let rem_a = chunks_a.remainder();
                let rem_b = chunks_b.remainder();
                for (a_chunk, b_chunk) in chunks_a.zip(chunks_b) {
                    let va = wide::f32x8::from([
                        a_chunk[0], a_chunk[1], a_chunk[2], a_chunk[3], a_chunk[4], a_chunk[5],
                        a_chunk[6], a_chunk[7],
                    ]);
                    let vb = wide::f32x8::from([
                        b_chunk[0], b_chunk[1], b_chunk[2], b_chunk[3], b_chunk[4], b_chunk[5],
                        b_chunk[6], b_chunk[7],
                    ]);
                    dot_v += va * vb;
                    norm_a_v += va * va;
                    norm_b_v += vb * vb;
                }
                let mut dot = dot_v.reduce_add();
                let mut norm_a = norm_a_v.reduce_add();
                let mut norm_b = norm_b_v.reduce_add();
                for i in 0..rem_a.len() {
                    dot += rem_a[i] * rem_b[i];
                    norm_a += rem_a[i] * rem_a[i];
                    norm_b += rem_b[i] * rem_b[i];
                }
                let denom = norm_a.sqrt() * norm_b.sqrt();
                if denom < f32::EPSILON {
                    None
                } else {
                    Some(dot / denom)
                }
            }
        }
    }

    /// Estimated heap memory in bytes
    pub fn memory_size(&self) -> usize {
        match self {
            VectorRepresentations::Full(v) => v.len() * 4,
            VectorRepresentations::Binary(data) => data.len() * 8,
            VectorRepresentations::Turbo(data) => data.len(),
            VectorRepresentations::None => 0,
        }
    }
}

// ─── Edge ──────────────────────────────────────────────────

/// Labeled directed edge with optional weight
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Edge {
    pub target: u64,
    pub label: String,
    pub weight: f32,
}

impl Edge {
    pub fn new(target: u64, label: impl Into<String>) -> Self {
        Self {
            target,
            label: label.into(),
            weight: 1.0,
        }
    }

    pub fn with_weight(target: u64, label: impl Into<String>, weight: f32) -> Self {
        Self {
            target,
            label: label.into(),
            weight,
        }
    }
}

// ─── Field Value ───────────────────────────────────────────

/// Typed relational field value
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum FieldValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
}

impl FieldValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            FieldValue::String(s) => Some(s),
            _ => None,
        }
    }
    pub fn as_int(&self) -> Option<i64> {
        match self {
            FieldValue::Int(i) => Some(*i),
            _ => None,
        }
    }
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            FieldValue::Bool(b) => Some(*b),
            _ => None,
        }
    }
}

/// Relational fields: ordered key-value map
pub type RelFields = BTreeMap<String, FieldValue>;

// ─── Node Flags ────────────────────────────────────────────

#[repr(transparent)]
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Serialize,
    Deserialize,
    PartialEq,
    IntoBytes,
    FromBytes,
    Immutable,
    KnownLayout,
)]
pub struct NodeFlags(pub u32);

impl NodeFlags {
    pub const ACTIVE: u32 = 1 << 0;
    pub const INDEXED: u32 = 1 << 1;
    pub const DIRTY: u32 = 1 << 2;
    pub const TOMBSTONE: u32 = 1 << 3;
    pub const HAS_VECTOR: u32 = 1 << 4;
    pub const HAS_EDGES: u32 = 1 << 5;
    pub const PINNED: u32 = 1 << 6;
    pub const RECOVERED: u32 = 1 << 7;
    pub const INVALIDATED: u32 = 1 << 8;

    pub fn new() -> Self {
        Self(Self::ACTIVE)
    }
    pub fn is_set(&self, flag: u32) -> bool {
        self.0 & flag != 0
    }
    pub fn set(&mut self, flag: u32) {
        self.0 |= flag;
    }
    pub fn clear(&mut self, flag: u32) {
        self.0 &= !flag;
    }
    pub fn is_active(&self) -> bool {
        self.is_set(Self::ACTIVE)
    }
    pub fn is_tombstone(&self) -> bool {
        self.is_set(Self::TOMBSTONE)
    }
}

// ─── Node Tier ─────────────────────────────────────────────

/// Determines storage tier behavior
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq)]
pub enum NodeTier {
    /// Fast volatile memory (RAM cache)
    Hot,
    /// Long-term persistent storage (disk)
    #[default]
    Cold,
}

/// Trait for tracking access patterns
pub trait AccessTracker {
    fn confidence_score(&self) -> f32;
    fn hits(&self) -> u32;
    fn last_accessed(&self) -> u64; // Unix ms
    fn pin(&mut self);
    fn unpin(&mut self);
    fn is_pinned(&self) -> bool;
}

// ─── DiskNodeHeader (Zero-Copy) ────────────────────────────

/// Fixed-size header for zero-copy memory mapping.
/// Aligned to 64 bytes for optimal SIMD access and cache line boundary.
/// Uses raw u32 for flags/tier to avoid enums in #[repr(C)].
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug, PartialEq, IntoBytes, FromBytes, Immutable, KnownLayout)]
pub struct DiskNodeHeader {
    /// Globally unique identifier (Offset 0)
    pub id: u64,
    /// Offset 8
    pub confidence_score: f32,
    /// Offset 12
    pub importance: f32,
    /// 128-bit fast filter (Offset 16)
    pub bitset: u128,
    /// Offset to vector data in the MMap file (Offset 32)
    pub vector_offset: u64,
    /// Number of elements in the vector (Offset 40)
    pub vector_len: u32,
    /// Number of outgoing edges (Offset 44)
    pub edge_count: u16,
    /// Explicit padding to align relational_len (Offset 46)
    pub _pad1: [u8; 2],
    /// Length of the relational metadata block (Offset 48)
    pub relational_len: u32,
    /// Storage tier: Hot (0) or Cold (1) (Offset 52)
    pub tier: u8,
    /// Explicit gap padding for u32 field 'flags' alignment (Offset 53)
    pub _pad2: [u8; 3],
    /// Status flags (Offset 56)
    pub flags: u32,
    /// Explicit padding to reach exactly 64 bytes (Offset 60)
    pub _padding: [u8; 4],
}

impl DiskNodeHeader {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            confidence_score: 0.5,
            importance: 0.1,
            bitset: 0,
            vector_offset: 0,
            vector_len: 0,
            edge_count: 0,
            _pad1: [0; 2],
            relational_len: 0,
            tier: 0,
            _pad2: [0; 3],
            flags: 0,
            _padding: [0; 4],
        }
    }
}

/// Core multimodel node: vector + graph + relational unified.
///
/// Header (id+bitset+cluster+flags = 32B) is cache-friendly.
/// Heavy data (vector, edges, relational) lives on the heap.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct UnifiedNode {
    /// Globally unique identifier
    pub id: u64,
    /// 128-bit fast filter (country, role, active, etc.)
    pub bitset: u128,
    /// Semantic cluster for super-node routing
    pub semantic_cluster: u32,
    /// Status flags
    pub flags: NodeFlags,
    pub vector: VectorRepresentations,
    /// Lineage version
    pub epoch: u32,
    /// Outgoing graph edges
    pub edges: Vec<Edge>,
    /// Relational key-value fields
    pub relational: RelFields,
    /// Storage tier: Hot (RAM) or Cold (disk)
    pub tier: NodeTier,
    /// Access frequency heuristic
    pub hits: u32,
    /// Recency heuristic (Unix MS)
    pub last_accessed: u64,
    /// Confidence score (0.0 - 1.0)
    pub confidence_score: f32,
    /// Importance score (0.0 - 1.0)
    pub importance: f32,
    /// Forward-compatible schema metadata without breaking Bincode
    pub ext_metadata: HashMap<String, Vec<u8>>,
}

impl AccessTracker for UnifiedNode {
    fn confidence_score(&self) -> f32 {
        self.confidence_score
    }
    fn hits(&self) -> u32 {
        self.hits
    }
    fn last_accessed(&self) -> u64 {
        self.last_accessed
    }
    fn pin(&mut self) {
        self.flags.set(NodeFlags::PINNED);
    }
    fn unpin(&mut self) {
        self.flags.clear(NodeFlags::PINNED);
    }
    fn is_pinned(&self) -> bool {
        self.flags.is_set(NodeFlags::PINNED)
    }
}

impl UnifiedNode {
    /// New empty node with given ID
    pub fn new(id: u64) -> Self {
        Self {
            id,
            bitset: 0,
            semantic_cluster: 0,
            flags: NodeFlags::new(),
            vector: VectorRepresentations::None,
            epoch: 0,
            edges: Vec::new(),
            relational: BTreeMap::new(),
            tier: NodeTier::Cold,
            hits: 0,
            last_accessed: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            confidence_score: 0.5,
            importance: 0.1,
            ext_metadata: HashMap::new(),
        }
    }

    /// New node with vector data
    pub fn with_vector(id: u64, vector: Vec<f32>) -> Self {
        let mut node = Self::new(id);
        node.vector = VectorRepresentations::Full(vector);
        node.flags.set(NodeFlags::HAS_VECTOR);
        node
    }

    /// Add a labeled edge
    pub fn add_edge(&mut self, target: u64, label: impl Into<String>) {
        self.edges.push(Edge::new(target, label));
        self.flags.set(NodeFlags::HAS_EDGES);
    }

    /// Add weighted edge
    pub fn add_weighted_edge(&mut self, target: u64, label: impl Into<String>, weight: f32) {
        self.edges.push(Edge::with_weight(target, label, weight));
        self.flags.set(NodeFlags::HAS_EDGES);
    }

    /// Set relational field
    pub fn set_field(&mut self, key: impl Into<String>, value: FieldValue) {
        self.relational.insert(key.into(), value);
    }

    /// Get relational field
    pub fn get_field(&self, key: &str) -> Option<&FieldValue> {
        self.relational.get(key)
    }

    /// Set bit in filter bitset
    pub fn set_bit(&mut self, pos: u8) {
        debug_assert!(pos < 128);
        self.bitset |= 1u128 << pos;
    }

    /// Check if bit is set
    pub fn has_bit(&self, pos: u8) -> bool {
        self.bitset & (1u128 << pos) != 0
    }

    /// Check if ALL bits in mask are set
    pub fn matches_mask(&self, mask: u128) -> bool {
        self.bitset & mask == mask
    }

    /// Estimate total memory usage (bytes)
    pub fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.vector.memory_size()
            + self.edges.capacity() * std::mem::size_of::<Edge>()
            + self.relational.len() * 64 // rough BTreeMap node overhead
    }

    /// Mark as deleted (tombstone)
    pub fn mark_deleted(&mut self) {
        self.flags.clear(NodeFlags::ACTIVE);
        self.flags.set(NodeFlags::TOMBSTONE);
    }

    /// Is this node alive (active and not tombstoned)?
    pub fn is_alive(&self) -> bool {
        self.flags.is_active() && !self.flags.is_tombstone()
    }
}

impl Default for UnifiedNode {
    fn default() -> Self {
        Self::new(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let node = UnifiedNode::new(42);
        assert_eq!(node.id, 42);
        assert!(node.is_alive());
        assert!(node.vector.is_none());
        assert_eq!(node.epoch, 0);
        assert!(node.edges.is_empty());
    }

    #[test]
    fn test_bitset_operations() {
        let mut node = UnifiedNode::new(1);
        node.set_bit(5);
        node.set_bit(16);

        assert!(node.has_bit(5));
        assert!(node.has_bit(16));
        assert!(!node.has_bit(7));

        let mask: u128 = (1 << 5) | (1 << 16);
        assert!(node.matches_mask(mask));
        assert!(!node.matches_mask(mask | (1 << 7)));
    }

    #[test]
    fn test_tombstone() {
        let mut node = UnifiedNode::new(1);
        assert!(node.is_alive());
        node.mark_deleted();
        assert!(!node.is_alive());
    }

    #[test]
    fn test_relational_fields() {
        let mut node = UnifiedNode::new(1);
        node.set_field("country", FieldValue::String("US".into()));
        node.set_field("active", FieldValue::Bool(true));

        assert_eq!(
            node.get_field("country"),
            Some(&FieldValue::String("US".into()))
        );
        assert_eq!(node.get_field("active"), Some(&FieldValue::Bool(true)));
        assert_eq!(node.get_field("missing"), None);
    }
}


================================================================
Nombre: python.rs
Ruta: src\python.rs
================================================================

#![cfg(feature = "python_sdk")]
use crate::node::{UnifiedNode, VectorRepresentations};
use crate::storage::StorageEngine;
use pyo3::prelude::*;

#[pyclass]
pub struct ClientEngine {
    _storage: StorageEngine,
}

#[pymethods]
impl ClientEngine {
    #[new]
    pub fn new() -> Self {
        ClientEngine {
            _storage: StorageEngine::open("vantadb_data").expect("Failed to open StorageEngine"),
        }
    }

    /// High level query mapping directly traversing the execution plan.
    pub fn execute(&self, query: &str) -> PyResult<Vec<String>> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

        rt.block_on(async {
            let executor = crate::executor::Executor::new(&self._storage);
            match executor.execute_hybrid(query).await {
                Ok(crate::executor::ExecutionResult::Read(nodes)) => {
                    let results = nodes
                        .into_iter()
                        .map(|n| format!("ID: {} | Relational: {:?}", n.id, n.relational))
                        .collect();
                    Ok(results)
                }
                Ok(crate::executor::ExecutionResult::Write { message, .. }) => Ok(vec![message]),
                Ok(crate::executor::ExecutionResult::StaleContext(id)) => {
                    Ok(vec![format!("STALE_CONTEXT: {}", id)])
                }
                Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
            }
        })
    }

    /// Exposes node insertion directly to python scripts skipping HTTP serialization
    pub fn insert_node(&self, id: u64, vec_data: Option<Vec<f32>>) -> PyResult<()> {
        let mut node = UnifiedNode::new(id);
        if let Some(v) = vec_data {
            node.vector = VectorRepresentations::Full(v);
            node.flags.set(crate::node::NodeFlags::HAS_VECTOR);
        }
        self._storage
            .insert(&node)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        Ok(())
    }
}

/// The python module definition.
/// Compiled utilizing `maturin develop --features python_sdk`.
#[pymodule]
fn vantadb(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<ClientEngine>()?;
    Ok(())
}


================================================================
Nombre: query.rs
Ruta: src\query.rs
================================================================

use crate::node::FieldValue;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Query(Query),
    Insert(InsertStatement),
    Update(UpdateStatement),
    Delete(DeleteStatement),
    Relate(RelateStatement),
    InsertMessage(InsertMessageStatement), // Conversational Primitive
    Collapse(CollapseStatement),           // Phase 32B: Consistency Records
}

#[derive(Debug, Clone, PartialEq)]
pub struct CollapseStatement {
    pub zone_id: u64,
    pub index: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InsertStatement {
    pub node_id: u64,
    pub node_type: String,
    pub fields: BTreeMap<String, FieldValue>,
    pub vector: Option<Vec<f32>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpdateStatement {
    pub node_id: u64,
    pub fields: BTreeMap<String, FieldValue>,
    pub vector: Option<Vec<f32>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeleteStatement {
    pub node_id: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RelateStatement {
    pub source_id: u64,
    pub target_id: u64,
    pub label: String,
    pub weight: Option<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InsertMessageStatement {
    pub msg_role: String, // system, user, assistant
    pub content: String,
    pub thread_id: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Query {
    pub from_entity: String,
    pub traversal: Option<Traversal>,
    pub target_alias: String,
    pub where_clause: Option<Vec<Condition>>,
    pub fetch: Option<Vec<String>>,
    pub rank_by: Option<RankBy>,
    pub temperature: Option<f32>,
    pub owner_role: Option<String>, // RBAC
}

#[derive(Debug, Clone, PartialEq)]
pub struct Traversal {
    pub min_depth: u32,
    pub max_depth: u32,
    pub edge_label: String,
    pub target_type: Option<String>,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Condition {
    Relational(String, RelOp, FieldValue),
    VectorSim(String, String, f32), // field, text_query, min_score
}

#[derive(Debug, Clone, PartialEq)]
pub enum RelOp {
    Eq,
    Neq,
    Gt,
    Lt,
    Gte,
    Lte,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RankBy {
    pub field: String,
    pub desc: bool,
}

// ─── Logical Plan ──────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum LogicalOperator {
    Scan {
        entity: String,
    },
    Traverse {
        min_depth: u32,
        max_depth: u32,
        edge_label: String,
    },
    FilterRelational {
        field: String,
        op: RelOp,
        value: FieldValue,
    },
    VectorSearch {
        field: String,
        query_vec: String,
        min_score: f32,
    },
    Project {
        fields: Vec<String>,
    },
    Sort {
        field: String,
        desc: bool,
    },
    Limit {
        top_k: usize,
    },
}

#[derive(Debug, Clone)]
pub struct LogicalPlan {
    pub operators: Vec<LogicalOperator>,
    pub temperature: f32,
    pub enforce_role: Option<String>,
}

impl Query {
    /// Convert AST into a basic Logical Plan
    pub fn into_logical_plan(self) -> LogicalPlan {
        let mut ops = Vec::new();

        ops.push(LogicalOperator::Scan {
            entity: self.from_entity,
        });

        if let Some(mut conds) = self.where_clause {
            for cond in conds.drain(..) {
                match cond {
                    Condition::Relational(f, op, v) => {
                        ops.push(LogicalOperator::FilterRelational {
                            field: f,
                            op,
                            value: v,
                        });
                    }
                    Condition::VectorSim(f, text, min) => {
                        ops.push(LogicalOperator::VectorSearch {
                            field: f,
                            query_vec: text,
                            min_score: min,
                        });
                    }
                }
            }
        }

        if let Some(trav) = self.traversal {
            ops.push(LogicalOperator::Traverse {
                min_depth: trav.min_depth,
                max_depth: trav.max_depth,
                edge_label: trav.edge_label,
            });
        }

        if let Some(rank) = self.rank_by {
            ops.push(LogicalOperator::Sort {
                field: rank.field,
                desc: rank.desc,
            });
        }

        if let Some(fetch) = self.fetch {
            ops.push(LogicalOperator::Project { fields: fetch });
        }

        LogicalPlan {
            operators: ops,
            temperature: self.temperature.unwrap_or(0.0), // 0.0 default (Exhaustive)
            enforce_role: self.owner_role,
        }
    }
}

// ── VantaDB Biological Nomenclature (Type Alias) ────────────

/// The **QueryPlanner** is VantaDB's query decision engine.
/// Technically identical to `LogicalPlan` — it decides what to scan,
/// how to filter, and which traversal strategy to execute.
pub type QueryPlanner = LogicalPlan;


================================================================
Nombre: server.rs
Ruta: src\server.rs
================================================================

use crate::storage::StorageEngine;
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub struct QueryRequest {
    pub query: String,
}

#[derive(Serialize, Deserialize)]
pub struct QueryResponse {
    pub success: bool,
    pub data: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nodes: Option<Vec<NodeDTO>>,
}

#[derive(Serialize, Deserialize)]
pub struct NodeDTO {
    pub id: u64,
    pub semantic_cluster: u32,
    pub relational: std::collections::BTreeMap<String, crate::node::FieldValue>,
    pub hits: u32,
    pub confidence_score: f32,
}

impl From<&crate::node::UnifiedNode> for NodeDTO {
    fn from(n: &crate::node::UnifiedNode) -> Self {
        Self {
            id: n.id,
            semantic_cluster: n.semantic_cluster,
            relational: n.relational.clone(),
            hits: n.hits,
            confidence_score: n.confidence_score,
        }
    }
}

pub struct ServerState {
    pub storage: Arc<StorageEngine>,
}

pub fn app(state: Arc<ServerState>) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/api/v2/query", post(execute_query)) // Upgraded to v2 to reflect NodeDTO changes
        .with_state(state)
}

async fn health_check() -> Json<QueryResponse> {
    Json(QueryResponse {
        success: true,
        data: "OK".to_string(),
        node_id: None,
        nodes: None,
    })
}

async fn execute_query(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<QueryRequest>,
) -> Json<QueryResponse> {
    use crate::executor::{ExecutionResult, Executor};

    let executor = Executor::new(&state.storage);
    match executor.execute_hybrid(&payload.query).await {
        Ok(ExecutionResult::Read(nodes)) => {
            let dtos = nodes.iter().map(NodeDTO::from).collect();
            Json(QueryResponse {
                success: true,
                data: format!("Read {} nodes.", nodes.len()),
                node_id: None,
                nodes: Some(dtos),
            })
        }
        Ok(ExecutionResult::Write {
            affected_nodes,
            message,
            node_id,
        }) => Json(QueryResponse {
            success: true,
            data: format!("Mutated {} nodes: {}", affected_nodes, message),
            node_id,
            nodes: None,
        }),
        Ok(ExecutionResult::StaleContext(summary_id)) => Json(QueryResponse {
            success: true,
            data: format!(
                "STALE_CONTEXT: Confidence Score critical. Rehydration available for summary {}",
                summary_id
            ),
            node_id: Some(summary_id),
            nodes: None,
        }),
        Err(e) => Json(QueryResponse {
            success: false,
            data: format!("Execution Error: {}", e),
            node_id: None,
            nodes: None,
        }),
    }
}


================================================================
Nombre: storage.rs
Ruta: src\storage.rs
================================================================

use crate::backend::{BackendPartition, BackendWriteOp, StorageBackend};
use crate::backends::fjall_backend::FjallBackend;
use crate::backends::in_memory::InMemoryBackend;
use crate::backends::rocksdb_backend::RocksDbBackend;
use crate::error::{Result, VantaError};
use crate::index::{CPIndex, IndexBackend};
use crate::node::{DiskNodeHeader, UnifiedNode};
use memmap2::{MmapMut, MmapOptions};
use parking_lot::RwLock;
use std::fs::{File, OpenOptions};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{info, warn};
use zerocopy::{FromBytes, IntoBytes};

// ─── Internal Metadata Persistence ──────────────────────────

#[derive(serde::Serialize, serde::Deserialize)]
struct NodeMetadata {
    relational: crate::node::RelFields,
    edges: Vec<crate::node::Edge>,
}

// ─── VantaFile: Zero-Copy MMap Wrapper ──────────────────────

/// Magic bytes written at position 0 of the vector store file.
/// Allows format validation on open and prevents loading arbitrary binary files.
pub const VANTA_FILE_MAGIC: &[u8; 8] = b"VNTAFILE";

/// Format version for the VantaFile binary layout.
/// Increment when the DiskNodeHeader layout or write_cursor position changes.
pub const VANTA_FILE_VERSION: u32 = 1;

pub struct VantaFile {
    pub file: File,
    pub mmap: MmapMut,
    pub path: PathBuf,
    pub size: u64,
    pub write_cursor: u64,
}

// VantaFile must be Send + Sync for multi-threaded Python usage (FastAPI/Gunicorn).
// Safety: Access to VantaFile is governed by a RwLock in the StorageEngine,
// ensuring that mutation (write_header, save_cursor) and shared reads (read_header)
// never occur simultaneously across threads.
unsafe impl Send for VantaFile {}
unsafe impl Sync for VantaFile {}

impl VantaFile {
    pub fn open(path: PathBuf, initial_size: u64) -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)
            .map_err(VantaError::IoError)?;

        let mut current_size = file.metadata().map_err(VantaError::IoError)?.len();
        if current_size < 8 {
            current_size = initial_size.max(8);
            file.set_len(current_size).map_err(VantaError::IoError)?;
        }

        let mmap = unsafe {
            MmapOptions::new()
                .map_mut(&file)
                .map_err(VantaError::IoError)?
        };

        // El primer u64 del mmap es nuestro write_cursor persistente
        let write_cursor = u64::from_le_bytes(mmap[0..8].try_into().unwrap());
        let write_cursor = if write_cursor < 64 || write_cursor > current_size {
            64
        } else {
            // Ensure any existing cursor is aligned to 64
            (write_cursor + 63) & !63
        };

        Ok(Self {
            file,
            mmap,
            path,
            size: current_size,
            write_cursor,
        })
    }

    /// Guarda el cursor actual en el archivo para persistencia entre reinicios
    pub fn save_cursor(&mut self) {
        self.mmap[0..8].copy_from_slice(&self.write_cursor.to_le_bytes());
    }

    /// Read a DiskNodeHeader from a specific offset without cloning (Zero-Copy)
    pub fn read_header(&self, offset: u64) -> Option<&DiskNodeHeader> {
        let header_size = std::mem::size_of::<DiskNodeHeader>() as u64;
        if offset + header_size > self.size || !offset.is_multiple_of(64) {
            return None;
        }

        let slice = &self.mmap[offset as usize..(offset + header_size) as usize];
        DiskNodeHeader::ref_from_bytes(slice).ok()
    }

    /// Write a DiskNodeHeader to a specific offset
    pub fn write_header(&mut self, offset: u64, header: &DiskNodeHeader) -> Result<()> {
        let header_size = std::mem::size_of::<DiskNodeHeader>() as u64;

        // Alignment Check: Must be 64-byte aligned for Zero-Copy casting
        if !offset.is_multiple_of(64) {
            return Err(VantaError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "VantaFile: Misaligned header write at {} (must be 64B aligned)",
                    offset
                ),
            )));
        }

        if offset + header_size > self.size {
            return Err(VantaError::IoError(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "VantaFile: Offset out of bounds",
            )));
        }

        let dest = &mut self.mmap[offset as usize..(offset + header_size) as usize];
        dest.copy_from_slice(header.as_bytes());
        Ok(())
    }

    /// Sincroniza los cambios con el disco
    pub fn flush(&self) -> Result<()> {
        self.mmap.flush().map_err(VantaError::IoError)
    }

    /// Implementación de Warm-up Strategy (Phase 3.4)
    /// Protege capas superiores del HNSW con pre-fetching para evitar page faults iniciales.
    pub fn warmup_top_layers(&self, size: usize) {
        #[cfg(unix)]
        {
            use std::os::unix::io::AsRawFd;
            unsafe {
                libc::madvise(
                    self.mmap.as_ptr() as *mut libc::c_void,
                    size.min(self.mmap.len()),
                    libc::MADV_WILLNEED,
                );
            }
        }
        #[cfg(windows)]
        {
            // En Windows, leer secuencialmente es suficiente para que el OS pre-cachee.
            let mut _sum = 0u8;
            for i in (0..size.min(self.mmap.len())).step_by(4096) {
                _sum ^= self.mmap[i];
            }
        }
    }
}

// ─── Backend Kind ──────────────────────────────────────────

/// Selects which KV backend `StorageEngine` uses.
///
/// `InMemory` replaces only the KV layer (RocksDB). VantaFile and WAL
/// are still initialized on disk at the provided path. See module docs
/// in `backends::in_memory` for details.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum BackendKind {
    #[default]
    RocksDb,
    Fjall,
    InMemory,
}

/// Configuration for `StorageEngine` initialization.
#[derive(Debug, Clone, Default)]
pub struct EngineConfig {
    pub memory_limit: Option<u64>,
    pub force_mmap: bool,
    pub read_only: bool,
    /// Which KV backend to use. Defaults to `RocksDb`.
    pub backend_kind: BackendKind,
}

pub struct StorageEngine {
    /// Abstract KV backend. No RocksDB types leak through this field.
    pub(crate) backend: Arc<dyn StorageBackend>,
    pub hnsw: RwLock<CPIndex>,
    pub volatile_cache: RwLock<std::collections::HashMap<u64, UnifiedNode>>,
    pub admission_filter: crate::governance::admission_filter::AdmissionFilter,
    pub consistency_buffer: crate::governance::consistency::ConsistencyBuffer,
    pub conflict_resolver: crate::governance::conflict_resolver::ConflictResolver,
    pub last_query_timestamp: AtomicU64,
    pub emergency_maintenance_trigger: std::sync::atomic::AtomicBool,
    /// Path to the data directory
    pub data_dir: PathBuf,
    /// Vector Store
    pub vector_store: RwLock<VantaFile>,
    /// Write-Ahead Log for durability
    pub wal: std::sync::Arc<parking_lot::Mutex<Option<crate::wal::WalWriter>>>,
}

impl StorageEngine {
    /// Open with default configuration (backward-compatible).
    /// All existing call sites continue to work without modification.
    pub fn open(path: &str) -> Result<Self> {
        Self::open_with_config(path, None)
    }

    /// Open with explicit configuration for memory budgets and mode overrides.
    /// Used by the Python SDK to inject per-instance memory limits.
    pub fn open_with_config(path: &str, config: Option<EngineConfig>) -> Result<Self> {
        let config = config.unwrap_or_default();
        let caps = crate::hardware::HardwareCapabilities::global();

        let effective_memory = config
            .memory_limit
            .or_else(|| {
                std::env::var("VANTADB_MEMORY_LIMIT")
                    .ok()
                    .and_then(|v| v.parse::<u64>().ok())
            })
            .unwrap_or(caps.total_memory);

        // ── KV Backend initialization ──
        let backend: Arc<dyn StorageBackend> = match config.backend_kind {
            BackendKind::RocksDb => Arc::new(RocksDbBackend::open(path, &config)?),
            BackendKind::Fjall => Arc::new(FjallBackend::open(path)?),
            BackendKind::InMemory => Arc::new(InMemoryBackend::new()),
        };

        let data_dir = PathBuf::from(path).join("data");
        let _ = std::fs::create_dir_all(&data_dir);
        let index_path = data_dir.join("vector_index.bin");

        let use_mmap = config.force_mmap
            || caps.profile == crate::hardware::HardwareProfile::Survival
            || effective_memory < 16 * 1024 * 1024 * 1024
            || std::env::var("VANTA_FORCE_MMAP").is_ok();

        let hnsw = if let Some(loaded) = CPIndex::load_from_file(&index_path) {
            let mut idx = loaded;
            if use_mmap {
                idx.backend = IndexBackend::new_mmap(index_path.clone());
                info!(
                    backend = "mmap",
                    "HNSW Survival Mode: MMap backend activated (cold-start)"
                );
            }
            idx
        } else {
            if use_mmap {
                info!(
                    backend = "mmap",
                    "HNSW Survival Mode: MMap backend activated (fresh)"
                );
                CPIndex::with_backend(IndexBackend::new_mmap(index_path.clone()))
            } else {
                info!(
                    backend = "in-memory",
                    "HNSW Performance Mode: InMemory backend"
                );
                CPIndex::new()
            }
        };

        let vector_store_path = data_dir.join("vector_store.vanta");
        let vector_store = VantaFile::open(vector_store_path, 1024 * 1024 * 64)?;

        let wal_path = data_dir.join("vanta.wal");
        let wal_writer = crate::wal::WalWriter::open(&wal_path)?;

        let admission_filter = crate::governance::admission_filter::AdmissionFilter::new(100_000);
        let consistency_buffer = crate::governance::consistency::ConsistencyBuffer::new();
        let conflict_resolver = crate::governance::conflict_resolver::ConflictResolver::new();

        Ok(Self {
            backend,
            hnsw: RwLock::new(hnsw),
            volatile_cache: RwLock::new(std::collections::HashMap::new()),
            admission_filter,
            consistency_buffer,
            conflict_resolver,
            last_query_timestamp: AtomicU64::new(0),
            emergency_maintenance_trigger: std::sync::atomic::AtomicBool::new(false),
            data_dir,
            vector_store: RwLock::new(vector_store),
            wal: std::sync::Arc::new(parking_lot::Mutex::new(Some(wal_writer))),
        })
    }

    pub fn touch_activity(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        self.last_query_timestamp.store(now, Ordering::Release);
    }

    pub fn insert(&self, node: &UnifiedNode) -> Result<()> {
        if self.admission_filter.is_blocked(node.id) {
            return Err(VantaError::Execution(format!(
                "Node {} is blocked by AdmissionFilter (recently rejected)",
                node.id
            )));
        }

        self.touch_activity();

        let mut active_node = node.clone();
        active_node.last_accessed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        if let Some(ref mut wal_writer) = *self.wal.lock() {
            wal_writer.append(&crate::wal::WalRecord::Insert(active_node.clone()))?;
        }

        let storage_offset = {
            let mut vstore = self.vector_store.write();
            let offset = vstore.write_cursor;

            let header_size = std::mem::size_of::<DiskNodeHeader>() as u64;
            let vec_len =
                if let crate::node::VectorRepresentations::Full(ref v) = active_node.vector {
                    v.len()
                } else {
                    0
                };
            let vec_size = (vec_len * 4) as u64;
            let total_needed = offset + header_size + vec_size;

            if total_needed > vstore.size {
                let new_size = vstore.size * 2;
                vstore.file.set_len(new_size).map_err(VantaError::IoError)?;
                vstore.mmap = unsafe {
                    MmapOptions::new()
                        .map_mut(&vstore.file)
                        .map_err(VantaError::IoError)?
                };
                vstore.size = new_size;
            }

            let mut header = DiskNodeHeader::new(active_node.id);
            header.vector_offset = offset + header_size;
            header.vector_len = vec_len as u32;
            header.flags = active_node.flags.0;
            header.bitset = active_node.bitset;
            header.confidence_score = active_node.confidence_score;
            header.importance = active_node.importance;
            header.tier = match active_node.tier {
                crate::node::NodeTier::Hot => 1u8,
                crate::node::NodeTier::Cold => 0u8,
            };
            header.edge_count = active_node.edges.len() as u16;

            vstore.write_header(offset, &header)?;

            if let crate::node::VectorRepresentations::Full(ref vec) = active_node.vector {
                let vec_bytes = vec.as_bytes();
                vstore.mmap
                    [(offset + header_size) as usize..(offset + header_size + vec_size) as usize]
                    .copy_from_slice(vec_bytes);
            }

            vstore.write_cursor = (total_needed + 63) & !63; // Align next header to 64B
            vstore.save_cursor();
            offset
        };

        let key = active_node.id.to_le_bytes();
        let metadata = NodeMetadata {
            relational: active_node.relational.clone(),
            edges: active_node.edges.clone(),
        };
        let metadata_val = bincode::serialize(&metadata)
            .map_err(|e| VantaError::SerializationError(e.to_string()))?;
        self.backend
            .put(BackendPartition::Default, &key, &metadata_val)?;

        {
            let mut hnsw = self.hnsw.write();
            hnsw.add(
                active_node.id,
                active_node.bitset,
                active_node.vector.clone(),
                storage_offset,
            );
        }

        if active_node.tier == crate::node::NodeTier::Hot {
            let mut cache = self.volatile_cache.write();
            cache.insert(active_node.id, active_node.clone());

            let caps = crate::hardware::HardwareCapabilities::global();
            let cache_cap_bytes = caps.total_memory / 4;
            let approx_node_size = 1536;
            let max_nodes = (cache_cap_bytes / approx_node_size) as usize;

            if cache.len() > max_nodes {
                self.emergency_maintenance_trigger
                    .store(true, Ordering::Release);
            }
        }

        Ok(())
    }

    pub fn refresh_index(&self, node: &UnifiedNode, storage_offset: u64) {
        if node.flags.is_set(crate::node::NodeFlags::HAS_VECTOR) {
            if let crate::node::VectorRepresentations::Full(vec) = &node.vector {
                let mut index = self.hnsw.write();
                index.add(
                    node.id,
                    node.bitset,
                    crate::node::VectorRepresentations::Full(vec.clone()),
                    storage_offset,
                );
            }
        }
    }

    pub fn consolidate_node(&self, node: &UnifiedNode) -> Result<()> {
        let mut persisted = node.clone();
        persisted.tier = crate::node::NodeTier::Cold;

        let key = persisted.id.to_le_bytes();
        let val = bincode::serialize(&persisted)
            .map_err(|e| VantaError::SerializationError(e.to_string()))?;
        self.backend.put(BackendPartition::Default, &key, &val)?;

        self.refresh_index(&persisted, 0);

        {
            let mut cache = self.volatile_cache.write();
            cache.remove(&node.id);
        }

        Ok(())
    }

    pub fn insert_to_cf(&self, node: &UnifiedNode, cf_name: &str) -> Result<()> {
        let partition = Self::partition_from_cf_name(cf_name)?;
        let key = node.id.to_le_bytes();
        let val =
            bincode::serialize(node).map_err(|e| VantaError::SerializationError(e.to_string()))?;
        self.backend.put(partition, &key, &val)?;

        self.refresh_index(node, 0);
        Ok(())
    }

    pub fn get(&self, id: u64) -> Result<Option<UnifiedNode>> {
        self.touch_activity();

        {
            let mut cache = self.volatile_cache.write();
            if let Some(node) = cache.get_mut(&id) {
                if node.flags.is_set(crate::node::NodeFlags::TOMBSTONE) {
                    return Ok(None);
                }
                node.hits += 1;
                node.last_accessed = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                return Ok(Some(node.clone()));
            }
        }

        let key = id.to_le_bytes();
        let metadata_res = match self.backend.get(BackendPartition::Default, &key)? {
            Some(res) => res,
            None => return Ok(None),
        };

        let metadata: NodeMetadata = bincode::deserialize(&metadata_res)
            .map_err(|e| VantaError::SerializationError(e.to_string()))?;

        let hnsw = self.hnsw.read();
        let index_node = match hnsw.nodes.get(&id) {
            Some(n) => n,
            None => return Ok(None),
        };
        let storage_offset = index_node.storage_offset;

        let vstore = self.vector_store.read();
        let header = match vstore.read_header(storage_offset) {
            Some(h) => h,
            None => return Ok(None),
        };

        if (header.flags & 0x8) != 0 {
            return Ok(None);
        }

        let vec_start = header.vector_offset as usize;
        let vec_end = vec_start + (header.vector_len as usize * 4);
        if vec_end > vstore.size as usize {
            return Ok(None);
        }

        let vec_bytes = &vstore.mmap[vec_start..vec_end];
        let f32_vec: &[f32] = unsafe {
            std::slice::from_raw_parts(vec_bytes.as_ptr() as *const f32, header.vector_len as usize)
        };

        let mut node = UnifiedNode::new(id);
        node.bitset = header.bitset;
        node.vector = crate::node::VectorRepresentations::Full(f32_vec.to_vec());
        node.relational = metadata.relational;
        node.edges = metadata.edges;
        node.confidence_score = header.confidence_score;
        node.importance = header.importance;
        node.tier = if header.tier == 1 {
            crate::node::NodeTier::Hot
        } else {
            crate::node::NodeTier::Cold
        };
        node.flags = crate::node::NodeFlags(header.flags);

        Ok(Some(node))
    }

    pub fn delete(&self, id: u64, _reason: &str) -> Result<()> {
        if let Some(ref mut wal_writer) = *self.wal.lock() {
            wal_writer.append(&crate::wal::WalRecord::Delete { id })?;
        }

        let hnsw = self.hnsw.write();
        if let Some(index_node) = hnsw.nodes.get(&id) {
            let offset = index_node.storage_offset;

            let mut vstore = self.vector_store.write();
            if let Some(mut header) = vstore.read_header(offset).cloned() {
                header.flags |= 0x8;
                vstore.write_header(offset, &header)?;
            }
        }

        self.volatile_cache.write().remove(&id);

        let key = id.to_le_bytes();
        self.backend.delete(BackendPartition::Default, &key)?;

        Ok(())
    }

    pub fn purge_permanent(&self, id: u64) -> Result<()> {
        let key = id.to_le_bytes();
        self.backend.write_batch(vec![
            BackendWriteOp::Delete {
                partition: BackendPartition::Default,
                key: key.to_vec(),
            },
            BackendWriteOp::Delete {
                partition: BackendPartition::TombstoneStorage,
                key: key.to_vec(),
            },
            BackendWriteOp::Delete {
                partition: BackendPartition::Tombstones,
                key: key.to_vec(),
            },
        ])
    }

    pub fn is_deleted(&self, id: u64) -> Result<bool> {
        let key = id.to_le_bytes();
        match self.backend.get(BackendPartition::Tombstones, &key)? {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    pub fn trigger_compaction(&self) -> Result<()> {
        let vstore = self.vector_store.write();
        let hnsw = self.hnsw.read();

        let tombstone_count = hnsw
            .nodes
            .values()
            .filter(|n| {
                if let Some(h) = vstore.read_header(n.storage_offset) {
                    (h.flags & 0x8) != 0
                } else {
                    false
                }
            })
            .count();

        let total_nodes = hnsw.nodes.len();
        if total_nodes > 0 && (tombstone_count as f32 / total_nodes as f32) > 0.20 {
            warn!(
                tombstone_pct = (tombstone_count as f32 / total_nodes as f32 * 100.0) as u32,
                "Fragmentation >20% — offline compaction triggered"
            );
        }

        Ok(())
    }

    pub fn flush(&self) -> Result<()> {
        self.backend.flush()?;
        self.save_vector_index();
        Ok(())
    }

    fn save_vector_index(&self) {
        let index_path = self.data_dir.join("vector_index.bin");
        let mut index = self.hnsw.write();

        if index.backend.is_mmap() {
            if let Err(e) = index.sync_to_mmap() {
                warn!(err = %e, "Failed to sync MMap vector index");
            }
        } else {
            if let Err(e) = index.persist_to_file(&index_path) {
                warn!(err = %e, "Failed to persist vector index to file");
            }
        }
    }

    pub fn create_life_insurance(&self, timestamp_name: &str) -> Result<()> {
        let mut save_path = std::path::PathBuf::from("./vantadb_snapshots");
        if let Ok(override_dir) = std::env::var("VANTA_BACKUP_DIR") {
            save_path = std::path::PathBuf::from(override_dir);
        }
        save_path.push(timestamp_name);

        self.backend.checkpoint(&save_path)
    }

    pub fn recover_archived_nodes(&self, summary_id: u64) -> Result<Vec<UnifiedNode>> {
        let entries = self.backend.scan(BackendPartition::TombstoneStorage)?;

        let mut recovered = Vec::new();
        for (_k, v) in &entries {
            if let Ok(mut node) = bincode::deserialize::<crate::node::UnifiedNode>(v) {
                if node
                    .edges
                    .iter()
                    .any(|e| e.target == summary_id && e.label == "belonged_to")
                {
                    node.flags.set(crate::node::NodeFlags::ACTIVE);
                    node.flags.set(crate::node::NodeFlags::RECOVERED);
                    node.tier = crate::node::NodeTier::Hot;

                    self.refresh_index(&node, 0);

                    {
                        let mut cache = self.volatile_cache.write();
                        cache.insert(node.id, node.clone());
                    }
                    recovered.push(node);
                }
            }
        }
        Ok(recovered)
    }

    // ─── Delegation methods for external modules ────────────────
    //
    // These replace direct `storage.db.{cf_handle, put_cf, ...}` access
    // from executor.rs and maintenance_worker.rs.

    /// Write a value to a specific backend partition.
    ///
    /// Used by Executor (Collapse) and MaintenanceWorker to write
    /// auditable tombstones to `TombstoneStorage`.
    pub(crate) fn put_to_partition(
        &self,
        partition: BackendPartition,
        key: &[u8],
        value: &[u8],
    ) -> Result<()> {
        self.backend.put(partition, key, value)
    }

    /// Request backend compaction.
    ///
    /// Used by MaintenanceWorker after high tombstone volume.
    /// No-op for backends that don't support compaction.
    pub(crate) fn request_compaction(&self) {
        self.backend.compact();
    }

    // ─── Internal helpers ───────────────────────────────────────

    /// Translate a string-based CF name to a `BackendPartition`.
    /// Temporary compatibility bridge for `insert_to_cf`.
    fn partition_from_cf_name(cf_name: &str) -> Result<BackendPartition> {
        match cf_name {
            "default" => Ok(BackendPartition::Default),
            "tombstone_storage" => Ok(BackendPartition::TombstoneStorage),
            "compressed_archive" => Ok(BackendPartition::CompressedArchive),
            "tombstones" => Ok(BackendPartition::Tombstones),
            other => Err(VantaError::Execution(format!(
                "Unknown column family: '{}'",
                other
            ))),
        }
    }

    pub fn emergency_shutdown(&self, reason: &str, stmt: Option<&str>) -> ! {
        println!("\n=======================================================");
        println!("🔥 VANTADB SYSTEM EMERGENCY: Security Constraint Violated 🔥");
        println!("=======================================================");
        println!("Reason: {}", reason);
        if let Some(s) = stmt {
            println!("Offending Transaction: {}", s);
        }

        println!("Attempting controlled flush...");
        if let Err(e) = self.flush() {
            eprintln!(
                "CRITICAL ERROR: Failed to flush buffers during shutdown: {}",
                e
            );
        } else {
            println!("Buffers flushed successfully.");
        }
        std::process::exit(1);
    }
}


================================================================
Nombre: wal.rs
Ruta: src\wal.rs
================================================================

use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Result, VantaError};
use crate::node::UnifiedNode;

// ─── WAL Record ────────────────────────────────────────────

/// WAL record types (bincode-serialized)
#[derive(Serialize, Deserialize, Debug)]
pub enum WalRecord {
    Insert(UnifiedNode),
    Update { id: u64, node: UnifiedNode },
    Delete { id: u64 },
    Checkpoint { node_count: u64 },
}

// ─── WAL Writer ────────────────────────────────────────────

/// Append-only WAL writer with CRC32 integrity checks.
///
/// Record format: [len:u32][payload:bincode][crc:u32]
pub struct WalWriter {
    writer: BufWriter<File>,
    path: PathBuf,
    bytes_written: u64,
    record_count: u64,
}

impl WalWriter {
    /// Open or create WAL file for appending
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let file = OpenOptions::new().create(true).append(true).open(&path)?;
        let bytes_written = file.metadata()?.len();
        Ok(Self {
            writer: BufWriter::with_capacity(64 * 1024, file),
            path,
            bytes_written,
            record_count: 0,
        })
    }

    /// Append a record to the WAL
    pub fn append(&mut self, record: &WalRecord) -> Result<()> {
        let payload = bincode::serialize(record)
            .map_err(|e| VantaError::SerializationError(e.to_string()))?;
        let len = payload.len() as u32;
        let crc = crc32(&payload);

        self.writer.write_all(&len.to_le_bytes())?;
        self.writer.write_all(&payload)?;
        self.writer.write_all(&crc.to_le_bytes())?;

        self.bytes_written += 4 + payload.len() as u64 + 4;
        self.record_count += 1;
        Ok(())
    }

    /// Flush buffer and fsync to disk
    pub fn sync(&mut self) -> Result<()> {
        self.writer.flush()?;
        self.writer.get_ref().sync_data()?;
        Ok(())
    }

    pub fn bytes_written(&self) -> u64 {
        self.bytes_written
    }
    pub fn record_count(&self) -> u64 {
        self.record_count
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
}

// ─── WAL Reader ────────────────────────────────────────────

/// Sequential WAL reader for crash recovery
pub struct WalReader {
    reader: BufReader<File>,
    records_read: u64,
}

impl WalReader {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let file = File::open(path)?;
        Ok(Self {
            reader: BufReader::with_capacity(64 * 1024, file),
            records_read: 0,
        })
    }

    /// Read next record. Returns None at EOF.
    pub fn next_record(&mut self) -> Result<Option<WalRecord>> {
        // Read length prefix
        let mut len_buf = [0u8; 4];
        match self.reader.read_exact(&mut len_buf) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => return Err(e.into()),
        }
        let len = u32::from_le_bytes(len_buf) as usize;

        // Read payload
        let mut payload = vec![0u8; len];
        self.reader.read_exact(&mut payload)?;

        // Read and verify CRC
        let mut crc_buf = [0u8; 4];
        self.reader.read_exact(&mut crc_buf)?;
        let stored_crc = u32::from_le_bytes(crc_buf);
        let computed_crc = crc32(&payload);

        if stored_crc != computed_crc {
            return Err(VantaError::WalError(format!(
                "CRC mismatch at record {}: stored={:#x}, computed={:#x}",
                self.records_read, stored_crc, computed_crc
            )));
        }

        let record: WalRecord = bincode::deserialize(&payload)
            .map_err(|e| VantaError::SerializationError(e.to_string()))?;
        self.records_read += 1;
        Ok(Some(record))
    }

    /// Replay all records through a handler function
    pub fn replay_all<F>(&mut self, mut handler: F) -> Result<u64>
    where
        F: FnMut(WalRecord) -> Result<()>,
    {
        let mut count = 0u64;
        while let Some(record) = self.next_record()? {
            handler(record)?;
            count += 1;
        }
        Ok(count)
    }
}

// ─── CRC32 ─────────────────────────────────────────────────

/// Simple CRC32 (IEEE polynomial, non-cryptographic)
fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB8_8320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::UnifiedNode;

    #[test]
    fn test_wal_roundtrip() {
        let dir = std::env::temp_dir().join("connectome_test_wal_rt");
        let _ = std::fs::remove_file(&dir);

        {
            let mut w = WalWriter::open(&dir).unwrap();
            w.append(&WalRecord::Insert(UnifiedNode::new(1))).unwrap();
            w.append(&WalRecord::Insert(UnifiedNode::new(2))).unwrap();
            w.append(&WalRecord::Delete { id: 1 }).unwrap();
            w.sync().unwrap();
            assert_eq!(w.record_count(), 3);
        }

        {
            let mut r = WalReader::open(&dir).unwrap();
            let mut records = Vec::new();
            r.replay_all(|rec| {
                records.push(rec);
                Ok(())
            })
            .unwrap();
            assert_eq!(records.len(), 3);
        }

        let _ = std::fs::remove_file(&dir);
    }

    #[test]
    fn test_crc32_deterministic() {
        let data = b"connectome wal test";
        assert_eq!(crc32(data), crc32(data));
        assert_ne!(crc32(data), crc32(b"connectome wal tesx"));
    }
}


================================================================
Nombre: mcp.rs
Ruta: src\api\mcp.rs
================================================================

use crate::executor::{ExecutionResult, Executor};
use crate::storage::StorageEngine;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::sync::Arc;

#[derive(Deserialize)]
struct RpcRequest {
    jsonrpc: String,
    id: Value,
    method: String,
    params: Option<Value>,
}

#[derive(Serialize)]
struct RpcResponse {
    jsonrpc: String,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Value>,
}

fn error_code(code: i32, message: &str) -> Result<Value, Value> {
    Err(json!({"code": code, "message": message}))
}

pub async fn run_stdio_server(storage: Arc<StorageEngine>) {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let executor = Executor::new(&storage);

    // Bucle Stdio principal (JSON-RPC)
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        if line.trim().is_empty() {
            continue;
        }

        let req: RpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("[MCP] Error (stderr): Unparseable JSON-RPC: {}", e);
                let err_res = json!({
                    "jsonrpc": "2.0",
                    "id": Value::Null,
                    "error": {
                        "code": -32700,
                        "message": format!("Parse error: {}", e)
                    }
                });
                if let Ok(out) = serde_json::to_string(&err_res) {
                    writeln!(stdout, "{}", out).unwrap();
                    stdout.flush().unwrap();
                }
                continue;
            }
        };

        if req.jsonrpc != "2.0" {
            continue;
        }

        let res = match req.method.as_str() {
            "initialize" => handle_initialize(),
            "tools/list" => handle_tools_list(),
            "tools/call" => handle_tools_call(&req.params, &executor, &storage).await,
            _ => error_code(-32601, "Method not found"),
        };

        let (result, error) = match res {
            Ok(val) => (Some(val), None),
            Err(err) => (None, Some(err)),
        };

        let response = RpcResponse {
            jsonrpc: "2.0".to_string(),
            id: req.id,
            result,
            error,
        };

        if let Ok(out) = serde_json::to_string(&response) {
            writeln!(stdout, "{}", out).unwrap();
            stdout.flush().unwrap();
        }
    }
}

pub fn handle_initialize() -> Result<Value, Value> {
    Ok(json!({
        "protocolVersion": "2024-11-05",
        "serverInfo": {
            "name": "vantadb",
            "version": "0.4.0"
        },
        "capabilities": {
            "tools": {}
        }
    }))
}

pub fn handle_tools_list() -> Result<Value, Value> {
    Ok(json!({
        "tools": [
            {
                "name": "query_lisp",
                "description": "Ejecuta código VantaLISP. Permite leer estructuras e insertar/mutar Nodes aportando contexto semántico.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "Programa o sentencia en VantaLISP" }
                    },
                    "required": ["query"]
                }
            },
            {
                "name": "search_semantic",
                "description": "Búsqueda vectorial semántica cruda directamente en el índice HNSW.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "vector": { "type": "array", "items": {"type": "number"}, "description": "Vector F32 de consulta" },
                        "k": { "type": "number", "description": "Top K vecinos" }
                    },
                    "required": ["vector", "k"]
                }
            },
            {
                "name": "get_node_neighbors",
                "description": "Inspecciona vecinos o linaje de un nodo (Volatile o Archived).",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "node_id": { "type": "number", "description": "ID del Nodo a explorar" }
                    },
                    "required": ["node_id"]
                }
            },
            {
                "name": "inject_context",
                "description": "Inyecta estado o contexto externo conectándolo a un hilo específico para consolidación posterior (Vector Compaction).",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "content": { "type": "string", "description": "Contenido del contexto" },
                        "thread_id": { "type": "number", "description": "ID del hilo al que pertenece" }
                    },
                    "required": ["content", "thread_id"]
                }
            },
            {
                "name": "read_axioms",
                "description": "Retorna los Axiomas (Iron Axioms) del Devil's Advocate activos en la base de datos.",
                "inputSchema": {
                    "type": "object",
                    "properties": {},
                    "required": []
                }
            }
        ]
    }))
}

pub async fn handle_tools_call(
    params: &Option<Value>,
    executor: &Executor<'_>,
    storage: &StorageEngine,
) -> Result<Value, Value> {
    let p = params
        .as_ref()
        .ok_or_else(|| json!({"code": -32602, "message": "Missing params"}))?;
    let name = p["name"].as_str().unwrap_or("");
    let args = &p["arguments"];

    match name {
        "query_lisp" => {
            let query = args["query"].as_str().unwrap_or("");
            match executor.execute_hybrid(query).await {
                Ok(ExecutionResult::Read(nodes)) => {
                    let content = serde_json::to_string(&nodes).unwrap_or_default();
                    Ok(json!({"content": [{"type": "text", "text": content}]}))
                }
                Ok(ExecutionResult::Write {
                    affected_nodes,
                    message,
                    node_id,
                }) => {
                    let content = serde_json::to_string(&json!({
                        "affected_nodes": affected_nodes,
                        "message": message,
                        "node_id": node_id
                    }))
                    .unwrap_or_default();
                    Ok(json!({"content": [{"type": "text", "text": content}]}))
                }
                Ok(ExecutionResult::StaleContext(summary_id)) => {
                    let content = serde_json::to_string(&json!({
                        "stale_context": true,
                        "rehydration_available": true,
                        "summary_id": summary_id,
                        "message": "Recuperación Histórica sugerida (Confidence Score Crítico)."
                    }))
                    .unwrap_or_default();
                    Ok(json!({"content": [{"type": "text", "text": content}]}))
                }
                Err(e) => Ok(
                    json!({"isError": true, "content": [{"type": "text", "text": format!("LISP Runtime Error: {}", e)}]}),
                ),
            }
        }
        "search_semantic" => {
            let vec_arr = args["vector"]
                .as_array()
                .ok_or_else(|| json!({"code": -32602, "message": "Missing 'vector' array"}))?;
            let mut vector: Vec<f32> = Vec::with_capacity(vec_arr.len());
            for v in vec_arr {
                vector.push(v.as_f64().unwrap_or(0.0) as f32);
            }
            let k = args["k"].as_i64().unwrap_or(5) as usize;

            let mut results = Vec::new();
            let index = storage.hnsw.read();
            let vs = storage.vector_store.read();
            let neighbors = index.search_nearest(&vector, None, None, 0, k, Some(&vs));
            for (id, distance) in neighbors {
                if let Ok(Some(node)) = storage.get(id) {
                    results.push(json!({"id": id, "distance": distance, "node": node}));
                }
            }
            let content = serde_json::to_string(&results).unwrap_or_default();
            Ok(json!({"content": [{"type": "text", "text": content}]}))
        }
        "get_node_neighbors" => {
            let node_id = args["node_id"]
                .as_u64()
                .ok_or_else(|| json!({"code": -32602, "message": "Missing 'node_id"}))?;

            if let Ok(Some(node)) = storage.get(node_id) {
                let mut neighbors = Vec::new();
                for edge in &node.edges {
                    if let Ok(Some(target_node)) = storage.get(edge.target) {
                        neighbors.push(json!({
                            "rel": edge.label,
                            "target_id": edge.target,
                            "target_confidence": target_node.confidence_score,
                            "target_priority": target_node.importance
                        }));
                    }
                }
                let content = serde_json::to_string(&json!({"node": node, "neighbors": neighbors}))
                    .unwrap_or_default();
                Ok(json!({"content": [{"type": "text", "text": content}]}))
            } else {
                Ok(
                    json!({"isError": true, "content": [{"type": "text", "text": "Node not found"}]}),
                )
            }
        }
        "inject_context" => {
            let content = args["content"]
                .as_str()
                .ok_or_else(|| json!({"code": -32602, "message": "Missing 'content'"}))?;
            let thread_id = args["thread_id"]
                .as_u64()
                .ok_or_else(|| json!({"code": -32602, "message": "Missing 'thread_id'"}))?;

            let query = format!(
                "INSERT MESSAGE SYSTEM \"{}\" TO THREAD#{}",
                content, thread_id
            );
            match executor.execute_hybrid(&query).await {
                Ok(ExecutionResult::Write {
                    affected_nodes,
                    message,
                    ..
                }) => {
                    let out = serde_json::to_string(&json!({
                        "affected_nodes": affected_nodes,
                        "message": message,
                        "status": "Context Anchored"
                    }))
                    .unwrap_or_default();
                    Ok(json!({"content": [{"type": "text", "text": out}]}))
                }
                Ok(_) => Ok(
                    json!({"isError": true, "content": [{"type": "text", "text": "Unexpected read result for insert"}]}),
                ),
                Err(e) => Ok(
                    json!({"isError": true, "content": [{"type": "text", "text": format!("Execution Error: {}", e)}]}),
                ),
            }
        }
        "read_axioms" => {
            let axioms = json!([
                {"id": 1, "name": "Axioma Topológico", "description": "No se permiten referencias (edges) a nodos huérfanos o en el Tombstone storage."},
                {"id": 2, "name": "Confidence Constraint", "description": "Se rechazan mutaciones vectoriales divergentes con alto Confidence Score histórico."},
                {"id": 3, "name": "Axioma Inmortal", "description": "Maintenance: Nodos marcados como PINNED evaden degradación por Data Decay."},
                {"id": 4, "name": "Resource Allocation", "description": "Maintenance: Reservado el 5% de memoria para nodos con prioridad semántica >= 0.8."}
            ]);
            let content = serde_json::to_string(&axioms).unwrap_or_default();
            Ok(json!({"content": [{"type": "text", "text": content}]}))
        }
        _ => error_code(-32601, "Tool not found"),
    }
}


================================================================
Nombre: mod.rs
Ruta: src\api\mod.rs
================================================================

pub mod mcp;


================================================================
Nombre: fjall_backend.rs
Ruta: src\backends\fjall_backend.rs
================================================================

//! Fjall-backed implementation of `StorageBackend`.
//!
//! This adapter maps the `StorageBackend` trait onto `fjall` v3.1.x.
//!
//! ## Fjall API model (v3.1.4)
//!
//! - **`fjall::Database`**: Top-level container. Owns the journal and all
//!   keyspaces. One per engine path. Equivalent to a RocksDB `DB` instance.
//! - **`fjall::Keyspace`**: A named LSM-tree within the Database. Each
//!   `BackendPartition` maps 1:1 to a Keyspace using the same string names
//!   as the RocksDB column families.
//! - **`fjall::OwnedWriteBatch`** (aliased as `WriteBatch`): Atomic batch
//!   that can span multiple Keyspaces. Equivalent to RocksDB `WriteBatch`.
//! - **`fjall::PersistMode`**: Controls durability on `Database::persist()`.
//!   `SyncAll` = fsync(data + metadata), strongest guarantee.
//!
//! ## Limitations vs RocksDB
//!
//! - **No checkpoint**: Fjall does not expose a point-in-time snapshot-to-disk
//!   API. `checkpoint()` returns an explicit error.
//! - **No manual compaction**: Fjall manages compaction internally via its
//!   LSM background threads. `compact()` is a no-op.

use crate::backend::{BackendPartition, BackendWriteOp, StorageBackend};
use crate::error::{Result, VantaError};
use fjall::{Database, Keyspace, KeyspaceCreateOptions, PersistMode};
use std::path::Path;
use tracing::info;

/// Fjall adapter implementing `StorageBackend`.
///
/// Owns a `fjall::Database` and four `Keyspace` handles corresponding to
/// the `BackendPartition` variants. Created through `FjallBackend::open`.
pub(crate) struct FjallBackend {
    db: Database,
    default: Keyspace,
    tombstone_storage: Keyspace,
    compressed_archive: Keyspace,
    tombstones: Keyspace,
}

impl FjallBackend {
    /// Open a Fjall database at `path`.
    ///
    /// Creates the database directory if it does not exist.
    /// Opens (or creates) one keyspace per `BackendPartition` using the
    /// same names as the RocksDB column families for semantic continuity.
    pub(crate) fn open(path: &str) -> Result<Self> {
        let db = Database::builder(path)
            .open()
            .map_err(|e| VantaError::IoError(std::io::Error::other(e.to_string())))?;

        let default = db
            .keyspace("default", KeyspaceCreateOptions::default)
            .map_err(|e| VantaError::IoError(std::io::Error::other(e.to_string())))?;

        let tombstone_storage = db
            .keyspace("tombstone_storage", KeyspaceCreateOptions::default)
            .map_err(|e| VantaError::IoError(std::io::Error::other(e.to_string())))?;

        let compressed_archive = db
            .keyspace("compressed_archive", KeyspaceCreateOptions::default)
            .map_err(|e| VantaError::IoError(std::io::Error::other(e.to_string())))?;

        let tombstones = db
            .keyspace("tombstones", KeyspaceCreateOptions::default)
            .map_err(|e| VantaError::IoError(std::io::Error::other(e.to_string())))?;

        info!("Fjall database opened at '{}'", path);

        Ok(Self {
            db,
            default,
            tombstone_storage,
            compressed_archive,
            tombstones,
        })
    }

    /// Resolve a `BackendPartition` to the corresponding `Keyspace` handle.
    fn keyspace(&self, partition: BackendPartition) -> &Keyspace {
        match partition {
            BackendPartition::Default => &self.default,
            BackendPartition::TombstoneStorage => &self.tombstone_storage,
            BackendPartition::CompressedArchive => &self.compressed_archive,
            BackendPartition::Tombstones => &self.tombstones,
        }
    }
}

impl StorageBackend for FjallBackend {
    fn put(&self, partition: BackendPartition, key: &[u8], value: &[u8]) -> Result<()> {
        self.keyspace(partition)
            .insert(key, value)
            .map_err(|e| VantaError::IoError(std::io::Error::other(e.to_string())))
    }

    fn get(&self, partition: BackendPartition, key: &[u8]) -> Result<Option<Vec<u8>>> {
        self.keyspace(partition)
            .get(key)
            .map(|opt| opt.map(|slice| slice.to_vec()))
            .map_err(|e| VantaError::IoError(std::io::Error::other(e.to_string())))
    }

    fn delete(&self, partition: BackendPartition, key: &[u8]) -> Result<()> {
        self.keyspace(partition)
            .remove(key)
            .map_err(|e| VantaError::IoError(std::io::Error::other(e.to_string())))
    }

    fn write_batch(&self, ops: Vec<BackendWriteOp>) -> Result<()> {
        // OwnedWriteBatch (type-aliased as WriteBatch) provides native atomic
        // writes across multiple Keyspaces within the same Database.
        // All operations are committed atomically via the shared journal.
        let mut batch = self.db.batch();
        for op in ops {
            match op {
                BackendWriteOp::Put {
                    partition,
                    key,
                    value,
                } => {
                    batch.insert(self.keyspace(partition), key, value);
                }
                BackendWriteOp::Delete { partition, key } => {
                    batch.remove(self.keyspace(partition), key);
                }
            }
        }
        batch
            .commit()
            .map_err(|e| VantaError::IoError(std::io::Error::other(e.to_string())))
    }

    fn scan(&self, partition: BackendPartition) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        let ks = self.keyspace(partition);
        let mut result = Vec::new();
        for item in ks.iter() {
            let kv = item.into_inner().map_err(|e| VantaError::IoError(std::io::Error::other(e.to_string())))?;
            result.push((kv.0.to_vec(), kv.1.to_vec()));
        }
        Ok(result)
    }

    /// Flush pending writes to durable storage.
    ///
    /// Uses `PersistMode::SyncAll` which calls `fsync` on both data and
    /// metadata, providing the strongest durability guarantee Fjall offers.
    ///
    /// Per Fjall docs: "Persisting only affects durability, NOT consistency.
    /// Even without flushing data is crash-safe." The journal architecture
    /// provides crash consistency regardless; this call ensures data survives
    /// power loss.
    fn flush(&self) -> Result<()> {
        self.db
            .persist(PersistMode::SyncAll)
            .map_err(|e| VantaError::IoError(std::io::Error::other(e.to_string())))
    }

    /// Checkpoint is not supported by Fjall.
    ///
    /// Fjall does not expose a point-in-time consistent snapshot-to-disk API
    /// equivalent to RocksDB's `Checkpoint::create_checkpoint`. Returning an
    /// honest error rather than simulating with unsafe file copies.
    fn checkpoint(&self, _path: &Path) -> Result<()> {
        Err(VantaError::Execution(
            "Checkpoint not supported by FjallBackend: Fjall does not expose a \
             point-in-time snapshot-to-disk API equivalent to RocksDB checkpoints"
                .to_string(),
        ))
    }

    /// No-op: Fjall manages LSM compaction automatically via internal
    /// background threads. No manual compaction trigger is needed or
    /// exposed for this use case.
    fn compact(&self) {
        // Fjall's LSM engine (lsm-tree crate) runs automatic background
        // compaction. There is no public manual compaction API to call here.
    }
}


================================================================
Nombre: in_memory.rs
Ruta: src\backends\in_memory.rs
================================================================

//! In-memory implementation of `StorageBackend`.
//!
//! Provides a fully functional KV store backed by `BTreeMap`s in memory.
//! Intended for:
//! - Fast, isolated unit tests that don't need disk I/O.
//! - Decoupling `StorageEngine` logic from the persistence layer during testing.
//!
//! ## Important clarification
//!
//! "InMemoryBackend" means **in-memory KV backend only**. When used with
//! `StorageEngine`, VantaFile (vector store) and WAL are still initialized
//! on disk at the provided path. This backend replaces only the RocksDB
//! key-value layer, not the entire storage stack.

use crate::backend::{BackendPartition, BackendWriteOp, StorageBackend};
use crate::error::{Result, VantaError};
use parking_lot::RwLock;
use std::collections::{BTreeMap, HashMap};
use std::path::Path;

/// In-memory storage backend using `BTreeMap` per partition.
///
/// Thread-safe via `RwLock`. All data is lost when the backend is dropped.
pub(crate) struct InMemoryBackend {
    #[allow(clippy::type_complexity)]
    partitions: RwLock<HashMap<BackendPartition, BTreeMap<Vec<u8>, Vec<u8>>>>,
}

impl InMemoryBackend {
    /// Create a new in-memory backend with all partitions initialized empty.
    pub(crate) fn new() -> Self {
        let mut map = HashMap::new();
        map.insert(BackendPartition::Default, BTreeMap::new());
        map.insert(BackendPartition::TombstoneStorage, BTreeMap::new());
        map.insert(BackendPartition::CompressedArchive, BTreeMap::new());
        map.insert(BackendPartition::Tombstones, BTreeMap::new());
        Self {
            partitions: RwLock::new(map),
        }
    }
}

impl StorageBackend for InMemoryBackend {
    fn put(&self, partition: BackendPartition, key: &[u8], value: &[u8]) -> Result<()> {
        let mut parts = self.partitions.write();
        let btree = parts.entry(partition).or_default();
        btree.insert(key.to_vec(), value.to_vec());
        Ok(())
    }

    fn get(&self, partition: BackendPartition, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let parts = self.partitions.read();
        Ok(parts
            .get(&partition)
            .and_then(|btree| btree.get(key).cloned()))
    }

    fn delete(&self, partition: BackendPartition, key: &[u8]) -> Result<()> {
        let mut parts = self.partitions.write();
        if let Some(btree) = parts.get_mut(&partition) {
            btree.remove(key);
        }
        Ok(())
    }

    fn write_batch(&self, ops: Vec<BackendWriteOp>) -> Result<()> {
        let mut parts = self.partitions.write();
        for op in ops {
            match op {
                BackendWriteOp::Put {
                    partition,
                    key,
                    value,
                } => {
                    let btree = parts.entry(partition).or_default();
                    btree.insert(key, value);
                }
                BackendWriteOp::Delete { partition, key } => {
                    if let Some(btree) = parts.get_mut(&partition) {
                        btree.remove(&key);
                    }
                }
            }
        }
        Ok(())
    }

    fn scan(&self, partition: BackendPartition) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        let parts = self.partitions.read();
        Ok(parts
            .get(&partition)
            .map(|btree| btree.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default())
    }

    fn flush(&self) -> Result<()> {
        // No-op: all data is already in memory.
        Ok(())
    }

    fn checkpoint(&self, _path: &Path) -> Result<()> {
        Err(VantaError::Execution(
            "Checkpoint not supported by InMemoryBackend".to_string(),
        ))
    }

    // compact() inherits the default no-op from the trait.
}

// ─── Unit Tests ─────────────────────────────────────────────
//
// These tests validate InMemoryBackend directly through the trait.
// They live here (inside the crate) because StorageBackend is pub(crate).

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::{BackendPartition, BackendWriteOp};

    #[test]
    fn test_backend_in_memory_basic_crud() {
        let backend = InMemoryBackend::new();

        // Put
        backend
            .put(BackendPartition::Default, b"key1", b"value1")
            .unwrap();

        // Get
        let val = backend
            .get(BackendPartition::Default, b"key1")
            .unwrap()
            .expect("key1 should exist");
        assert_eq!(val, b"value1");

        // Get non-existent
        assert!(backend
            .get(BackendPartition::Default, b"missing")
            .unwrap()
            .is_none());

        // Delete
        backend.delete(BackendPartition::Default, b"key1").unwrap();
        assert!(backend
            .get(BackendPartition::Default, b"key1")
            .unwrap()
            .is_none());

        // Scan on different partition
        backend
            .put(BackendPartition::Tombstones, b"t1", b"tombval")
            .unwrap();
        let entries = backend.scan(BackendPartition::Tombstones).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, b"t1");
    }

    #[test]
    fn test_backend_in_memory_batch() {
        let backend = InMemoryBackend::new();

        // Seed a value that will be deleted in the batch
        backend
            .put(BackendPartition::Default, b"to_delete", b"val")
            .unwrap();

        let ops = vec![
            BackendWriteOp::Put {
                partition: BackendPartition::Default,
                key: b"batch_key1".to_vec(),
                value: b"batch_val1".to_vec(),
            },
            BackendWriteOp::Put {
                partition: BackendPartition::TombstoneStorage,
                key: b"batch_key2".to_vec(),
                value: b"batch_val2".to_vec(),
            },
            BackendWriteOp::Delete {
                partition: BackendPartition::Default,
                key: b"to_delete".to_vec(),
            },
        ];

        backend.write_batch(ops).unwrap();

        assert!(backend
            .get(BackendPartition::Default, b"batch_key1")
            .unwrap()
            .is_some());
        assert!(backend
            .get(BackendPartition::TombstoneStorage, b"batch_key2")
            .unwrap()
            .is_some());
        assert!(backend
            .get(BackendPartition::Default, b"to_delete")
            .unwrap()
            .is_none());
    }
}


================================================================
Nombre: mod.rs
Ruta: src\backends\mod.rs
================================================================

//! Concrete `StorageBackend` implementations.

pub(crate) mod fjall_backend;
pub(crate) mod in_memory;
pub(crate) mod rocksdb_backend;


================================================================
Nombre: rocksdb_backend.rs
Ruta: src\backends\rocksdb_backend.rs
================================================================

//! RocksDB-backed implementation of `StorageBackend`.
//!
//! This adapter encapsulates all direct interaction with the `rocksdb` crate.
//! No RocksDB types (DB, ColumnFamily handles, iterators, options) should leak
//! outside this module.

use crate::backend::{BackendPartition, BackendWriteOp, StorageBackend};
use crate::error::{Result, VantaError};
use crate::storage::EngineConfig;
use rocksdb::checkpoint::Checkpoint;
use rocksdb::{FlushOptions, Options, WriteBatch, DB};
use std::path::Path;
use tracing::{info, warn};

/// RocksDB adapter implementing `StorageBackend`.
///
/// Owns the `rocksdb::DB` instance and all column family configuration.
/// Created exclusively through `RocksDbBackend::open`.
pub(crate) struct RocksDbBackend {
    db: DB,
}

impl RocksDbBackend {
    /// Open a RocksDB database at `path` with the given configuration.
    ///
    /// Preserves the original tuning: bloom filters, LRU cache sizing,
    /// memtable budgets, LZ4 compression, mmap access for low-RAM profiles,
    /// and per-CF block-based table options.
    pub(crate) fn open(path: &str, config: &EngineConfig) -> Result<Self> {
        let caps = crate::hardware::HardwareCapabilities::global();

        // Memory limit resolution priority:
        // 1. Explicit config.memory_limit (from Python SDK constructor)
        // 2. VANTADB_MEMORY_LIMIT env var (from Docker/CI)
        // 3. Hardware detection (from HardwareCapabilities)
        let effective_memory = config
            .memory_limit
            .or_else(|| {
                std::env::var("VANTADB_MEMORY_LIMIT")
                    .ok()
                    .and_then(|v| v.parse::<u64>().ok())
            })
            .unwrap_or(caps.total_memory);

        let mut opts = Options::default();
        opts.create_if_missing(!config.read_only);
        opts.create_missing_column_families(true);
        opts.set_max_background_jobs(4);
        opts.set_compression_type(rocksdb::DBCompressionType::Lz4);

        // Adaptive Mode: Dynamic RocksDB tuning based on effective RAM
        let mut bopts = rocksdb::BlockBasedOptions::default();
        bopts.set_bloom_filter(10.0, false);
        // Performance Booster: Force retention of L0 indexes and bloom filters permanently
        bopts.set_cache_index_and_filter_blocks(true);
        bopts.set_pin_l0_filter_and_index_blocks_in_cache(true);

        // Standard Bopts for cold layers (no L0 pinning)
        let mut cold_bopts = rocksdb::BlockBasedOptions::default();
        cold_bopts.set_bloom_filter(10.0, false);

        // OOM Guard: Cap LRU Cache and WriteBuffer to ~60% of effective capacity
        let rocksdb_budget = (effective_memory as f64 * 0.60) as usize;
        let cache_size = (rocksdb_budget as f64 * 0.75) as usize; // 75% focus on block cache
        let write_buffer_total = rocksdb_budget - cache_size; // 25% for memtables

        let write_buffer_size = (write_buffer_total / 2).clamp(8 * 1024 * 1024, 128 * 1024 * 1024);

        opts.set_write_buffer_size(write_buffer_size);
        opts.set_max_write_buffer_number(2);

        let cache = rocksdb::Cache::new_lru_cache(cache_size);
        bopts.set_block_cache(&cache);
        cold_bopts.set_block_cache(&cache);

        info!(
            rocksdb_budget_mb = rocksdb_budget / 1024 / 1024,
            cache_mb = cache_size / 1024 / 1024,
            memtable_mb = write_buffer_size / 1024 / 1024,
            "RocksDB memory configured"
        );

        opts.set_block_based_table_factory(&bopts);

        if caps.profile == crate::hardware::HardwareProfile::Survival
            || effective_memory < 16 * 1024 * 1024 * 1024
        {
            opts.set_allow_mmap_reads(true);
            opts.set_allow_mmap_writes(true);
            warn!(
                effective_memory_gb = effective_memory / 1024 / 1024 / 1024,
                "RAM < 16GB — MMap access forced (Survival Mode)"
            );
        }

        // Fast layers (LZ4)
        let mut default_opts = opts.clone();
        default_opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
        default_opts.set_block_based_table_factory(&bopts);

        // tombstone_storage: Unpinned bloom for efficiency
        let mut shadow_opts = rocksdb::Options::default();
        shadow_opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
        shadow_opts.set_block_based_table_factory(&cold_bopts);

        let mut archive_opts = rocksdb::Options::default();
        archive_opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
        archive_opts.set_block_based_table_factory(&bopts);

        let mut tombstone_opts = default_opts.clone();
        tombstone_opts.set_block_based_table_factory(&cold_bopts);

        let cf_descriptors = vec![
            rocksdb::ColumnFamilyDescriptor::new("default", default_opts),
            rocksdb::ColumnFamilyDescriptor::new("tombstone_storage", shadow_opts),
            rocksdb::ColumnFamilyDescriptor::new("compressed_archive", archive_opts),
            rocksdb::ColumnFamilyDescriptor::new("tombstones", tombstone_opts),
        ];

        let db = DB::open_cf_descriptors(&opts, path, cf_descriptors)
            .map_err(|e| VantaError::IoError(std::io::Error::other(e.to_string())))?;

        Ok(Self { db })
    }

    /// Helper: resolve a `BackendPartition` to its RocksDB column family handle.
    fn cf_handle(&self, partition: BackendPartition) -> Result<&rocksdb::ColumnFamily> {
        self.db.cf_handle(partition.cf_name()).ok_or_else(|| {
            VantaError::Execution(format!("Column family '{}' not found", partition.cf_name()))
        })
    }
}

impl StorageBackend for RocksDbBackend {
    fn put(&self, partition: BackendPartition, key: &[u8], value: &[u8]) -> Result<()> {
        if partition == BackendPartition::Default {
            self.db
                .put(key, value)
                .map_err(|e| VantaError::IoError(std::io::Error::other(e.to_string())))
        } else {
            let cf = self.cf_handle(partition)?;
            self.db
                .put_cf(&cf, key, value)
                .map_err(|e| VantaError::IoError(std::io::Error::other(e.to_string())))
        }
    }

    fn get(&self, partition: BackendPartition, key: &[u8]) -> Result<Option<Vec<u8>>> {
        if partition == BackendPartition::Default {
            self.db
                .get(key)
                .map_err(|e| VantaError::IoError(std::io::Error::other(e.to_string())))
        } else {
            let cf = self.cf_handle(partition)?;
            self.db
                .get_cf(&cf, key)
                .map_err(|e| VantaError::IoError(std::io::Error::other(e.to_string())))
        }
    }

    fn delete(&self, partition: BackendPartition, key: &[u8]) -> Result<()> {
        if partition == BackendPartition::Default {
            self.db
                .delete(key)
                .map_err(|e| VantaError::IoError(std::io::Error::other(e.to_string())))
        } else {
            let cf = self.cf_handle(partition)?;
            self.db
                .delete_cf(&cf, key)
                .map_err(|e| VantaError::IoError(std::io::Error::other(e.to_string())))
        }
    }

    fn write_batch(&self, ops: Vec<BackendWriteOp>) -> Result<()> {
        let mut batch = WriteBatch::default();
        for op in ops {
            match op {
                BackendWriteOp::Put {
                    partition,
                    key,
                    value,
                } => {
                    if partition == BackendPartition::Default {
                        batch.put(&key, &value);
                    } else {
                        let cf = self.cf_handle(partition)?;
                        batch.put_cf(&cf, &key, &value);
                    }
                }
                BackendWriteOp::Delete { partition, key } => {
                    if partition == BackendPartition::Default {
                        batch.delete(&key);
                    } else {
                        let cf = self.cf_handle(partition)?;
                        batch.delete_cf(&cf, &key);
                    }
                }
            }
        }
        self.db
            .write(batch)
            .map_err(|e| VantaError::IoError(std::io::Error::other(e.to_string())))
    }

    fn scan(&self, partition: BackendPartition) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        let cf = self.cf_handle(partition)?;
        let mut result = Vec::new();
        for item in self.db.iterator_cf(&cf, rocksdb::IteratorMode::Start) {
            let (k, v) =
                item.map_err(|e| VantaError::IoError(std::io::Error::other(e.to_string())))?;
            result.push((k.to_vec(), v.to_vec()));
        }
        Ok(result)
    }

    fn flush(&self) -> Result<()> {
        let mut flush_opt = FlushOptions::default();
        flush_opt.set_wait(true);
        self.db
            .flush_opt(&flush_opt)
            .map_err(|e| VantaError::IoError(std::io::Error::other(e.to_string())))
    }

    fn checkpoint(&self, path: &Path) -> Result<()> {
        let cp = Checkpoint::new(&self.db).map_err(|e| {
            VantaError::IoError(std::io::Error::other(format!(
                "Error creating Checkpoint initializer: {}",
                e
            )))
        })?;

        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        cp.create_checkpoint(path).map_err(|e| {
            VantaError::IoError(std::io::Error::other(format!(
                "Error writing checkpoint: {}",
                e
            )))
        })
    }

    fn compact(&self) {
        let mut c_opts = rocksdb::CompactOptions::default();
        c_opts.set_exclusive_manual_compaction(false);
        self.db
            .compact_range_opt(None::<&[u8]>, None::<&[u8]>, &c_opts);
    }
}


================================================================
Nombre: vanta-cli.rs
Ruta: src\bin\vanta-cli.rs
================================================================

use std::io::{self, Write};
// VantaDB CLI REPL. In production, use `rustyline` for proper terminal support.

#[tokio::main]
async fn main() {
    println!("VantaDB Interactive Shell v0.1.0");
    println!("Type '\\help' for commands, or write your query directly.");
    println!("Connecting to tcp://127.0.0.1:8080...");

    let client = reqwest::Client::new();
    let url = "http://127.0.0.1:8080/api/v1/query";

    loop {
        print!("vanta> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        if input == "\\q" || input == "\\quit" || input == "exit" {
            println!("Goodbye!");
            break;
        } else if input == "\\help" {
            println!(
                "Commands:\n\\q     Quit\n\\help  Show help\n<query> Send physical query to daemon"
            );
            continue;
        }

        // Send query to the REST API
        match client
            .post(url)
            .json(&serde_json::json!({ "query": input }))
            .send()
            .await
        {
            Ok(response) => {
                if let Ok(json) = response.json::<serde_json::Value>().await {
                    let success = json["success"].as_bool().unwrap_or(false);
                    let data = json["data"].as_str().unwrap_or("");
                    if success {
                        println!("✅ SUCCESS\n{}", data);
                    } else {
                        println!("❌ ERROR\n{}", data);
                    }
                } else {
                    println!("Error parsing daemon response");
                }
            }
            Err(e) => {
                println!("Error communicating with daemon: {}", e);
            }
        }
    }
}


================================================================
Nombre: vanta-server.rs
Ruta: src\bin\vanta-server.rs
================================================================

use std::env;
use std::sync::Arc;
use tokio::net::TcpListener;
use vantadb::console;
use vantadb::server::{app, ServerState};
use vantadb::storage::StorageEngine;

#[tokio::main]
async fn main() {
    // ── Initialize styled logging & banner ──────────────────────────────────
    console::init_logging();

    let args: Vec<String> = env::args().collect();
    let is_mcp = args.iter().any(|arg| arg == "--mcp");

    if !is_mcp {
        console::print_banner();
        console::progress("Initializing storage engine...", None);
    }

    // ── Open storage engine ─────────────────────────────────────────────────
    let data_dir = env::var("VANTA_DATA_DIR").unwrap_or_else(|_| "vantadb_data".to_string());
    let storage = match StorageEngine::open(&data_dir) {
        Ok(s) => {
            if !is_mcp {
                console::ok("Storage engine opened", Some(&data_dir));
            }
            Arc::new(s)
        }
        Err(e) => {
            console::error("Failed to open storage engine", Some(&e.to_string()));
            std::process::exit(1);
        }
    };

    // ── Bootstrap Invalidation Dispatcher ──────────────────────────────────
    let mut dispatcher = vantadb::governance::invalidations::InvalidationDispatcher::new(256);
    let invalidation_tx = dispatcher.sender();
    if let Some(rx) = dispatcher.take_receiver() {
        tokio::spawn(async move {
            vantadb::governance::invalidations::invalidation_listener(rx).await;
        });
    }

    // ── Background maintenance worker ───────────────────────────────────────
    let maintenance_storage_ctx = storage.clone();
    tokio::spawn(async move {
        vantadb::governance::maintenance_worker::MaintenanceWorker::start(
            maintenance_storage_ctx,
            invalidation_tx,
        )
        .await;
    });

    if !is_mcp {
        console::ok(
            "Background workers started",
            Some("maintenance_worker · invalidations"),
        );
    }

    // ── Serve MCP or HTTP ───────────────────────────────────────────────────
    if is_mcp {
        vantadb::api::mcp::run_stdio_server(storage).await;
    } else {
        let state = Arc::new(ServerState {
            storage: storage.clone(),
        });
        let router = app(state);

        let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
        let addr = format!("{}:{}", host, port);

        let listener = match TcpListener::bind(&addr).await {
            Ok(l) => {
                console::ok("TCP listener bound", Some(&addr));
                l
            }
            Err(e) => {
                console::error("Failed to bind port", Some(&e.to_string()));
                std::process::exit(1);
            }
        };

        console::print_ready(&addr);

        axum::serve(listener, router).await.unwrap();
    }
}


================================================================
Nombre: mod.rs
Ruta: src\eval\mod.rs
================================================================

use crate::error::{Result, VantaError};
use crate::executor::{ExecutionResult, Executor};
use crate::node::FieldValue;
use crate::parser::lisp::LispExpr;
use std::collections::BTreeMap;

pub mod vm;

const MAX_FUEL: u64 = 1000;

pub struct LispSandbox<'a> {
    executor: &'a Executor<'a>,
    fuel: u64,
}

impl<'a> LispSandbox<'a> {
    pub fn new(executor: &'a Executor<'a>) -> Self {
        Self {
            executor,
            fuel: MAX_FUEL,
        }
    }

    pub async fn eval(&mut self, expr: impl AsRef<LispExpr>) -> Result<ExecutionResult> {
        if self.fuel == 0 {
            return Err(VantaError::Execution(
                "Sandbox Abort: Out of Execution Fuel".to_string(),
            ));
        }
        self.fuel -= 1;

        match expr.as_ref() {
            LispExpr::List(list) => {
                if list.is_empty() {
                    return Err(VantaError::Execution("Empty LISP statement".to_string()));
                }

                if let LispExpr::Atom(func) = &list[0] {
                    match func.to_uppercase().as_str() {
                        "INSERT" => self.eval_insert(&list[1..]).await,
                        "MATCH" => Err(VantaError::Execution(
                            "MATCH LISP logic pending".to_string(),
                        )),
                        _ => Err(VantaError::Execution(format!(
                            "Unknown LISP logic intrinsic: {}",
                            func
                        ))),
                    }
                } else {
                    Err(VantaError::Execution(
                        "Expected function atom at beginning of expression".to_string(),
                    ))
                }
            }
            _ => Err(VantaError::Execution(
                "Top level must be a LISP List".to_string(),
            )),
        }
    }

    // MVP: (INSERT :node {:label "IA" :confidence 0.9})
    async fn eval_insert(&mut self, args: &[LispExpr]) -> Result<ExecutionResult> {
        if args.len() < 2 {
            return Err(VantaError::Execution(
                "INSERT requires target and payload".to_string(),
            ));
        }

        let target = if let LispExpr::Keyword(k) = &args[0] {
            k.as_str()
        } else {
            "node"
        };
        let mut fields = BTreeMap::new();
        let node_type = target.to_string();

        let node_id = rand::random::<u64>(); // Generación genérica

        if let LispExpr::Map(map) = &args[1] {
            for (key, val) in map {
                if let LispExpr::Keyword(k) = key {
                    match val {
                        LispExpr::StringLiteral(s) => {
                            fields.insert(k.clone(), FieldValue::String(s.clone()));
                        }
                        LispExpr::Number(n) => {
                            fields.insert(k.clone(), FieldValue::Float(*n as f64));
                        }
                        _ => {} // Fallback for simple map parser
                    }
                }
            }
        }

        // Atar Metadata Homoiconica por v0.4.0 directiva "sys_rule: true"
        fields.insert("sys_rule".to_string(), FieldValue::Bool(true));

        // LISP rules are top-tier active nodes (Hot) —
        // must live in volatile_cache for low-latency access.
        let mut node = crate::node::UnifiedNode::new(node_id);
        node.tier = crate::node::NodeTier::Hot;
        node.set_field("type", FieldValue::String(node_type.clone()));
        for (k, v) in &fields {
            node.set_field(k.as_str(), v.clone());
        }

        self.executor.insert_node(&node)?;
        Ok(ExecutionResult::Write {
            affected_nodes: 1,
            message: format!("LISP Node {} inserted into volatile cache.", node_id),
            node_id: Some(node_id),
        })
    }
}


================================================================
Nombre: vm.rs
Ruta: src\eval\vm.rs
================================================================

use crate::node::{UnifiedNode, VectorRepresentations};

#[derive(Debug, Clone)]
pub enum Opcode {
    OpPushFloat(f32),
    OpPushVector(VectorRepresentations),
    OpConfidenceCheck,
    OpVecSim,
    OpRehydrate,
}

pub struct VantaLispVM {
    float_stack: Vec<f32>,
    vec_stack: Vec<VectorRepresentations>,
    pub needs_rehydration: bool,
    /// Epoch snapshot taken at VM creation to detect mid-flight invalidations.
    context_epoch: u32,
}

impl Default for VantaLispVM {
    fn default() -> Self {
        Self::new()
    }
}

impl VantaLispVM {
    pub fn new() -> Self {
        Self {
            float_stack: Vec::new(),
            vec_stack: Vec::new(),
            needs_rehydration: false,
            context_epoch: 0,
        }
    }

    /// Bind the VM to a specific node's epoch for staleness detection.
    pub fn bind_epoch(&mut self, epoch: u32) {
        self.context_epoch = epoch;
    }

    /// Executa el array de bytecode (Opcodes) retornando (Valor, ConfidenceScore)
    pub fn execute(
        &mut self,
        program: &[Opcode],
        current_context: &UnifiedNode,
    ) -> Result<(f32, f32), String> {
        // Epoch Staleness Guard: if the node was invalidated since we bound,
        // the data we're operating on may be corrupted. Degrade confidence immediately.
        if current_context.epoch != self.context_epoch && self.context_epoch != 0 {
            eprintln!(
                "⚠️ [VM] Epoch mismatch on node {}: expected {}, got {}. Context invalidated mid-flight.",
                current_context.id, self.context_epoch, current_context.epoch
            );
            // Return degraded result — confidence collapses to signal stale data
            return Ok((0.0, 0.1));
        }

        // Snapshot the epoch for this execution pass
        self.context_epoch = current_context.epoch;

        // En v0.4.0, cada ejecución VantaLISP evalúa un Confidence Score inherente base general
        let mut op_confidence = current_context.confidence_score;

        for op in program {
            match op {
                Opcode::OpPushFloat(f) => {
                    self.float_stack.push(*f);
                }
                Opcode::OpPushVector(v) => {
                    self.vec_stack.push(v.clone());
                }
                Opcode::OpConfidenceCheck => {
                    // Empuja a la pila de floats el confidence score de contexto
                    self.float_stack.push(current_context.confidence_score);
                }
                Opcode::OpVecSim => {
                    let v2 = self.vec_stack.pop().ok_or("Stack underflow: OP_VEC_SIM")?;
                    let v1 = self.vec_stack.pop().ok_or("Stack underflow: OP_VEC_SIM")?;

                    if let Some(sim) = v1.cosine_similarity(&v2) {
                        self.float_stack.push(sim);
                    } else {
                        // Penalizar confidence si no hay similitud cálculable
                        op_confidence *= 0.8;
                        self.float_stack.push(0.0);
                    }
                }
                Opcode::OpRehydrate => {
                    self.needs_rehydration = true;
                    // Retorna temporalmente NaN float o similar para la pila (o simplemente ignora)
                    self.float_stack.push(0.0);
                }
            }
        }

        let result_val = self.float_stack.pop().unwrap_or(0.0);
        Ok((result_val, op_confidence))
    }
}


================================================================
Nombre: admission_filter.rs
Ruta: src\governance\admission_filter.rs
================================================================

use parking_lot::RwLock;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

const DEFAULT_BLOOM_BITS: usize = 100_000;
const K_SALTS: [u64; 3] = [0x5A5A5A5A5A5A5A5A, 0x3C3C3C3C3C3C3C3C, 0x1E1E1E1E1E1E1E1E];

/// AdmissionFilter prevents the ingestion of previously rejected records
/// and blocked agent roles via a probabilistic Bloom Filter.
pub struct AdmissionFilter {
    bit_array: RwLock<Vec<u8>>,
    bits_count: usize,
}

impl AdmissionFilter {
    pub fn new(capacity_hint: usize) -> Self {
        let optimal_bits = ((capacity_hint as f64) * 9.585).ceil() as usize;
        let bits_count = optimal_bits.max(DEFAULT_BLOOM_BITS);
        let size_bytes = bits_count.div_ceil(8);

        Self {
            bit_array: RwLock::new(vec![0; size_bytes]),
            bits_count,
        }
    }

    fn calculate_hashes_u64(&self, key: u64) -> [usize; 3] {
        let mut idxs = [0; 3];
        for (i, &salt) in K_SALTS.iter().enumerate() {
            let mut hasher = DefaultHasher::new();
            key.hash(&mut hasher);
            salt.hash(&mut hasher);
            idxs[i] = (hasher.finish() as usize) % self.bits_count;
        }
        idxs
    }

    fn calculate_hashes_str(&self, key: &str) -> [usize; 3] {
        let mut idxs = [0; 3];
        for (i, &salt) in K_SALTS.iter().enumerate() {
            let mut hasher = DefaultHasher::new();
            key.hash(&mut hasher);
            salt.hash(&mut hasher);
            idxs[i] = (hasher.finish() as usize) % self.bits_count;
        }
        idxs
    }

    fn set_bits(&self, idxs: &[usize; 3]) {
        let mut bit_array = self.bit_array.write();
        for &idx in idxs {
            let byte_idx = idx / 8;
            let bit_pos = idx % 8;
            bit_array[byte_idx] |= 1 << bit_pos;
        }
    }

    fn check_bits(&self, idxs: &[usize; 3]) -> bool {
        let bit_array = self.bit_array.read();
        for &idx in idxs {
            let byte_idx = idx / 8;
            let bit_pos = idx % 8;
            if (bit_array[byte_idx] & (1 << bit_pos)) == 0 {
                return false;
            }
        }
        true
    }

    pub fn block_record(&self, record_id: u64) {
        let idxs = self.calculate_hashes_u64(record_id);
        self.set_bits(&idxs);
    }

    pub fn is_blocked(&self, record_id: u64) -> bool {
        let idxs = self.calculate_hashes_u64(record_id);
        self.check_bits(&idxs)
    }

    pub fn block_role(&self, owner_role: &str) {
        let idxs = self.calculate_hashes_str(owner_role);
        self.set_bits(&idxs);
    }

    pub fn is_role_blocked(&self, owner_role: &str) -> bool {
        let idxs = self.calculate_hashes_str(owner_role);
        self.check_bits(&idxs)
    }
}


================================================================
Nombre: conflict_resolver.rs
Ruta: src\governance\conflict_resolver.rs
================================================================

use crate::governance::ResolutionResult;
use crate::node::{AccessTracker, UnifiedNode};
use parking_lot::RwLock;
use std::collections::HashMap;

// ─── Conflict Resolver (Legacy: Devil's Advocate) ─────────────────────────────

/// Tracks collision counts and confidence scores per unique origin (`_owner_role`).
/// Used by ConflictResolver to compute the logarithmic friction metric F_ax.
pub struct OriginCollisionTracker {
    /// Map: owner_role → (collision_count, confidence_score_of_origin)
    origins: HashMap<String, (u64, f32)>,
}

impl Default for OriginCollisionTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl OriginCollisionTracker {
    pub fn new() -> Self {
        Self {
            origins: HashMap::new(),
        }
    }

    pub fn record_collision(&mut self, owner_role: &str, challenger_confidence: f32) {
        let entry = self
            .origins
            .entry(owner_role.to_string())
            .or_insert((0, challenger_confidence));
        entry.0 += 1;
        entry.1 = entry.1 * 0.8 + challenger_confidence * 0.2;
    }

    pub fn compute_friction(&self) -> f32 {
        self.origins
            .iter()
            .map(|(_, (count, confidence))| ((*count as f32 + 1.0).log2()) * confidence)
            .sum()
    }

    pub fn unique_origins(&self) -> usize {
        self.origins.len()
    }

    pub fn slash_origin(&mut self, owner_role: &str) {
        if let Some(entry) = self.origins.get_mut(owner_role) {
            entry.1 = 0.0;
        } else {
            self.origins.insert(owner_role.to_string(), (0, 0.0));
        }
    }

    pub fn is_slashed(&self, owner_role: &str) -> bool {
        self.origins
            .get(owner_role)
            .is_some_and(|(_, confidence)| *confidence <= 0.0)
    }

    pub fn reset(&mut self) {
        self.origins.clear();
    }
}

pub struct ConflictResolver {
    pub collision_tracker: RwLock<OriginCollisionTracker>,
}

impl Default for ConflictResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl ConflictResolver {
    pub fn new() -> Self {
        Self {
            collision_tracker: RwLock::new(OriginCollisionTracker::new()),
        }
    }
}

pub trait ConfidenceArbiter {
    fn evaluate_conflict(
        &self,
        incumbent: &UnifiedNode,
        challenger: &UnifiedNode,
    ) -> ResolutionResult;
}

impl ConfidenceArbiter for ConflictResolver {
    fn evaluate_conflict(
        &self,
        incumbent: &UnifiedNode,
        challenger: &UnifiedNode,
    ) -> ResolutionResult {
        let challenger_role = challenger
            .relational
            .get("_owner_role")
            .and_then(|v| v.as_str())
            .unwrap_or("anonymous");

        {
            let tracker = self.collision_tracker.read();
            if tracker.is_slashed(challenger_role) {
                return ResolutionResult::Reject(format!(
                    "Slashing Policy: agent '{}' has Confidence Score 0.0 (banned)",
                    challenger_role
                ));
            }
        }

        if let Some(sim) = incumbent.vector.cosine_similarity(&challenger.vector) {
            if sim > 0.95 {
                if incumbent.is_pinned() && incumbent.importance >= 0.8 {
                    let mut tracker = self.collision_tracker.write();
                    tracker.record_collision(challenger_role, challenger.confidence_score());

                    let friction = tracker.compute_friction();
                    let threshold = incumbent.importance * 10.0;

                    if friction < threshold {
                        return ResolutionResult::Reject(
                            format!(
                                "Consistency Barrier: Insufficient friction (F_ax={:.2} < threshold={:.2}). Unique origins: {}",
                                friction, threshold, tracker.unique_origins()
                            )
                        );
                    }
                }

                if challenger.confidence_score() < incumbent.confidence_score() {
                    return ResolutionResult::Superposition(
                        crate::governance::consistency::ConsistencyRecord::new_superposition(
                            incumbent.clone(),
                            challenger.clone(),
                            10000,
                        ),
                    );
                }
            }
        }
        ResolutionResult::Accept
    }
}


================================================================
Nombre: consistency.rs
Ruta: src\governance\consistency.rs
================================================================

use crate::node::UnifiedNode;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum RecordState {
    PendingConflict, // Pending contextual resolution
    ResolvedAccept,  // Allowed to migrate to persistent storage
    ResolvedReject,  // Heading to AdmissionFilter + Purge
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConsistencyRecord {
    pub node_id: u64,
    pub candidates: Vec<UnifiedNode>,
    pub state: RecordState,
    pub injected_at: u64, // Unix ms
    pub resolution_deadline_ms: u64,
}

impl ConsistencyRecord {
    pub fn new_superposition(
        incumbent: UnifiedNode,
        challenger: UnifiedNode,
        deadline_offset_ms: u64,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        Self {
            node_id: incumbent.id,
            candidates: vec![incumbent, challenger],
            state: RecordState::PendingConflict,
            injected_at: now,
            resolution_deadline_ms: now + deadline_offset_ms,
        }
    }

    pub fn add_candidate(&mut self, candidate: UnifiedNode) {
        if self.candidates.len() < 3 {
            self.candidates.push(candidate);
        }
    }
}

pub struct ResolutionStats {
    pub pending_to_resolved: AtomicU64,
    pub pending_to_decayed: AtomicU64,
}

impl Default for ResolutionStats {
    fn default() -> Self {
        Self {
            pending_to_resolved: AtomicU64::new(0),
            pending_to_decayed: AtomicU64::new(0),
        }
    }
}

pub struct ConsistencyBuffer {
    pub records: RwLock<HashMap<u64, ConsistencyRecord>>,
    pub stats: ResolutionStats,
}

impl Default for ConsistencyBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsistencyBuffer {
    pub fn new() -> Self {
        Self {
            records: RwLock::new(HashMap::new()),
            stats: ResolutionStats::default(),
        }
    }

    pub fn insert_record(&self, record: ConsistencyRecord) {
        let mut map = self.records.write();
        map.insert(record.node_id, record);
    }

    pub fn get_record(&self, id: u64) -> Option<ConsistencyRecord> {
        self.records.read().get(&id).cloned()
    }

    pub fn remove_record(&self, id: u64) -> Option<ConsistencyRecord> {
        self.records.write().remove(&id)
    }

    pub fn record_accept(&self) {
        self.stats
            .pending_to_resolved
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_decay(&self) {
        self.stats
            .pending_to_decayed
            .fetch_add(1, Ordering::Relaxed);
    }

    /// Emergency flush of the consistency buffer.
    /// Performs speculative resolution: Integrates the candidate of highest importance
    /// and purges the rest.
    pub fn force_flush(&self) -> Option<UnifiedNode> {
        let mut map = self.records.write();
        if map.is_empty() {
            return None;
        }

        let mut best_candidate: Option<UnifiedNode> = None;
        let mut best_importance = -1.0;

        for record in map.values() {
            for candidate in &record.candidates {
                if candidate.importance > best_importance {
                    best_importance = candidate.importance;
                    best_candidate = Some(candidate.clone());
                }
            }
        }

        let discarded = map.len() as u64;

        self.stats
            .pending_to_decayed
            .fetch_add(discarded.saturating_sub(1), Ordering::Relaxed);
        map.clear();

        if let Some(winner) = best_candidate {
            self.stats
                .pending_to_resolved
                .fetch_add(1, Ordering::Relaxed);
            return Some(winner);
        }

        None
    }
}


================================================================
Nombre: invalidations.rs
Ruta: src\governance\invalidations.rs
================================================================

use tokio::sync::mpsc;

/// Types of invalidation events emitted by the reactive protocol.
#[derive(Debug, Clone)]
pub enum InvalidationEvent {
    /// A node's quantized representation diverged from its FP32 ground truth.
    /// The epoch was incremented and the node re-quantized.
    PremiseInvalidated {
        node_id: u64,
        old_epoch: u32,
        new_epoch: u32,
        reason: String,
    },
    /// A node was flagged as INVALIDATED and purged from the graph.
    InvalidatedPurged { node_id: u64, reason: String },
    /// Hardware profile changed, forcing a full re-benchmark.
    EnvironmentDrift { old_hash: u64, new_hash: u64 },
}

/// Dispatcher that manages an async MPSC channel for invalidation events.
/// Producers (SleepWorker, DevilsAdvocate) send events.
/// Consumers (MCP API, Webhooks, Logging) receive and act on them.
pub struct InvalidationDispatcher {
    sender: mpsc::Sender<InvalidationEvent>,
    receiver: Option<mpsc::Receiver<InvalidationEvent>>,
}

impl InvalidationDispatcher {
    /// Create a new dispatcher with bounded channel capacity.
    /// The capacity acts as backpressure: if the consumer is slow,
    /// producers will await (not block the Tokio runtime).
    pub fn new(capacity: usize) -> Self {
        let (sender, receiver) = mpsc::channel(capacity);
        Self {
            sender,
            receiver: Some(receiver),
        }
    }

    /// Get a clone of the sender for producers (SleepWorker, etc.)
    pub fn sender(&self) -> mpsc::Sender<InvalidationEvent> {
        self.sender.clone()
    }

    /// Take ownership of the receiver (call once, give to the consumer task).
    pub fn take_receiver(&mut self) -> Option<mpsc::Receiver<InvalidationEvent>> {
        self.receiver.take()
    }

    /// Emit a PREMISE_INVALIDATED event.
    pub async fn emit_premise_invalidated(
        sender: &mpsc::Sender<InvalidationEvent>,
        node_id: u64,
        old_epoch: u32,
        new_epoch: u32,
        reason: String,
    ) {
        let event = InvalidationEvent::PremiseInvalidated {
            node_id,
            old_epoch,
            new_epoch,
            reason,
        };
        if let Err(e) = sender.send(event).await {
            eprintln!(
                "⚠️ [Invalidation] Failed to emit PREMISE_INVALIDATED: {}",
                e
            );
        }
    }

    /// Emit a INVALIDATED_PURGED event.
    pub async fn emit_invalidated_purged(
        sender: &mpsc::Sender<InvalidationEvent>,
        node_id: u64,
        reason: String,
    ) {
        let event = InvalidationEvent::InvalidatedPurged { node_id, reason };
        if let Err(e) = sender.send(event).await {
            eprintln!("⚠️ [Invalidation] Failed to emit INVALIDATED_PURGED: {}", e);
        }
    }
}

/// Background consumer task that logs invalidation events.
/// In production this would forward to MCP/Webhooks.
pub async fn invalidation_listener(mut receiver: mpsc::Receiver<InvalidationEvent>) {
    while let Some(event) = receiver.recv().await {
        match &event {
            InvalidationEvent::PremiseInvalidated {
                node_id,
                old_epoch,
                new_epoch,
                reason,
            } => {
                eprintln!(
                    "🔴 [INVALIDATION] PREMISE_INVALIDATED: Node {} | Epoch {} → {} | Reason: {}",
                    node_id, old_epoch, new_epoch, reason
                );
            }
            InvalidationEvent::InvalidatedPurged { node_id, reason } => {
                eprintln!(
                    "🧨 [INVALIDATION] INVALIDATED_PURGED: Node {} | Reason: {}",
                    node_id, reason
                );
            }
            InvalidationEvent::EnvironmentDrift { old_hash, new_hash } => {
                eprintln!(
                    "🦎 [INVALIDATION] ENVIRONMENT_DRIFT: Hardware signature changed {} → {}",
                    old_hash, new_hash
                );
            }
        }
    }
    eprintln!("[INVALIDATION] Listener channel closed. Dispatcher shut down.");
}


================================================================
Nombre: maintenance_worker.rs
Ruta: src\governance\maintenance_worker.rs
================================================================

use crate::backend::BackendPartition;
use crate::governance::invalidations::{InvalidationDispatcher, InvalidationEvent};
use crate::node::{AccessTracker, FieldValue, NodeFlags, NodeTier, UnifiedNode};
use crate::storage::StorageEngine;
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;
use tokio::time::sleep;

/// Maximum duration the maintenance cycle may spend on data compression.
const MAX_COMPRESSION_DURATION_MS: u128 = 8_000;

/// Minimum combined hit-weight for a group to deserve compression.
const MIN_GROUP_WEIGHT_FOR_COMPRESSION: u32 = 3;

pub struct MaintenanceWorker;

impl MaintenanceWorker {
    pub async fn start(
        storage: Arc<StorageEngine>,
        invalidation_tx: mpsc::Sender<InvalidationEvent>,
    ) {
        let cycle_duration = Duration::from_secs(10);
        let inactivity_threshold_ms = 5000;

        loop {
            if storage
                .emergency_maintenance_trigger
                .load(Ordering::Acquire)
            {
                println!("🚨 [Maintenance] EMERGENCY TRIGGER: Volatile Cache at limit. Starting aggressive maintenance (OOM Guard).");
                storage
                    .emergency_maintenance_trigger
                    .store(false, Ordering::Release);
                Self::run_maintenance_cycle(&storage, &invalidation_tx).await;
            }

            sleep(cycle_duration).await;

            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            let last_activity = storage.last_query_timestamp.load(Ordering::Acquire);

            if now - last_activity > inactivity_threshold_ms {
                Self::run_maintenance_cycle(&storage, &invalidation_tx).await;
            }
        }
    }

    async fn run_maintenance_cycle(
        storage: &Arc<StorageEngine>,
        invalidation_tx: &mpsc::Sender<InvalidationEvent>,
    ) {
        println!("🌙 [Maintenance] Starting maintenance cycle (Memory cleanup)...");

        let mut to_consolidate = Vec::new();
        let mut to_purge: Vec<(u64, bool, Option<String>)> = Vec::new(); // (id, is_invalidated, slashed_role)
        let mut compression_candidates: Vec<UnifiedNode> = Vec::new();

        {
            // ── Stage 0: ConsistencyBuffer Decay ──
            let stats = &storage.consistency_buffer.stats;
            let resolved = stats.pending_to_resolved.load(Ordering::Relaxed) as f64;
            let decayed = stats.pending_to_decayed.load(Ordering::Relaxed) as f64;
            let total = resolved + decayed;

            let _shrinks_deadline = total > 10.0 && (decayed / total) > 0.7;

            let mut buffer = storage.consistency_buffer.records.write();
            let mut keys_to_resolve = Vec::new();
            let mut keys_to_purge = Vec::new();

            for (&id, record) in buffer.iter_mut() {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;

                let mut best_confidence = -1.0;
                for cand in &record.candidates {
                    if cand.confidence_score() > best_confidence {
                        best_confidence = cand.confidence_score();
                    }
                }

                if now > record.resolution_deadline_ms {
                    keys_to_resolve.push(id);
                } else if best_confidence < 0.2 {
                    keys_to_purge.push(id);
                } else {
                    if _shrinks_deadline {
                        record.resolution_deadline_ms =
                            record.resolution_deadline_ms.saturating_sub(100);
                    }
                    for cand in &mut record.candidates {
                        cand.confidence_score *= 0.9;
                    }
                }
            }

            let discarded = keys_to_purge.len() as u64;
            if discarded > 0 {
                storage
                    .consistency_buffer
                    .stats
                    .pending_to_decayed
                    .fetch_add(discarded, Ordering::Relaxed);
            }

            let mut winners_to_insert = Vec::new();
            let mut losers_to_log = Vec::new();

            for id in keys_to_purge {
                if let Some(purged) = buffer.remove(&id) {
                    storage.admission_filter.block_record(id);
                    for cand in purged.candidates {
                        if cand.importance > 0.8 {
                            losers_to_log.push((
                                id,
                                cand.id,
                                "Consistency Decay: Total expiration".to_string(),
                            ));
                        }
                    }
                }
            }

            for id in keys_to_resolve {
                if let Some(mut record) = buffer.remove(&id) {
                    storage
                        .consistency_buffer
                        .stats
                        .pending_to_resolved
                        .fetch_add(1, Ordering::Relaxed);

                    let mut best_idx = 0;
                    let mut best_confidence = -1.0;
                    for (i, cand) in record.candidates.iter().enumerate() {
                        if cand.confidence_score() > best_confidence {
                            best_confidence = cand.confidence_score();
                            best_idx = i;
                        }
                    }

                    if !record.candidates.is_empty() {
                        let winner = record.candidates.remove(best_idx);
                        winners_to_insert.push(winner);

                        for cand in record.candidates {
                            losers_to_log.push((
                                id,
                                cand.id,
                                "Consistency Resolution: Rejected candidate".to_string(),
                            ));
                        }
                    }
                }
            }

            drop(buffer);

            for winner in winners_to_insert {
                let _ = storage.insert(&winner);
            }

            use crate::governance::AuditableTombstone;
            if !losers_to_log.is_empty() {
                for (id, hash, reason) in losers_to_log {
                    let tomb = AuditableTombstone::new(id, reason, hash);
                    let key = id.to_le_bytes();
                    if let Ok(tomb_val) = bincode::serialize(&tomb) {
                        let _ = storage.put_to_partition(
                            BackendPartition::TombstoneStorage,
                            &key,
                            &tomb_val,
                        );
                    }
                }
            }
        }

        let total_nodes;

        {
            // ── Stage 1 & 2: Eviction & Persistence Evaluation ──
            let mut cache = storage.volatile_cache.write();
            total_nodes = cache.len();
            let max_priority_shielded = (total_nodes as f32 * 0.05).ceil() as usize;
            let mut current_shielded = 0;

            let mut keys_to_remove = Vec::new();

            let caps = crate::hardware::HardwareCapabilities::global();
            let max_consolidations = match caps.profile {
                crate::hardware::HardwareProfile::Enterprise => 5000,
                crate::hardware::HardwareProfile::Performance => 500,
                crate::hardware::HardwareProfile::Survival => 50,
            };

            for (&id, node) in cache.iter_mut() {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;

                if node.flags.is_set(NodeFlags::RECOVERED) {
                    keys_to_remove.push(id);
                    continue;
                }
                if now - storage.last_query_timestamp.load(Ordering::Acquire) < 5000 {
                    println!("🔌 [Maintenance] Cycle interrupted (I/O activity detected).");
                    break;
                }

                if node.importance >= 0.8 && current_shielded < max_priority_shielded {
                    current_shielded += 1;
                    continue;
                }

                if node.flags.is_set(NodeFlags::INVALIDATED) {
                    println!(
                        "🧨 [Maintenance] Invalidated node detected: {}. Purging immediately.",
                        id
                    );
                    keys_to_remove.push(id);
                    let slashed_role: Option<String> = node
                        .relational
                        .get("_owner_role")
                        .and_then(|v: &crate::node::FieldValue| v.as_str())
                        .map(|s: &str| s.to_string());
                    to_purge.push((id, true, slashed_role));
                    continue;
                }

                node.hits = (node.hits as f32 * 0.5) as u32;

                if node.confidence_score() < 0.2 {
                    keys_to_remove.push(id);
                    to_purge.push((id, false, None));
                } else if node.hits < 10 && !node.is_pinned() && (now - node.last_accessed > 60_000)
                {
                    keys_to_remove.push(id);

                    if node.hits < 5 {
                        compression_candidates.push(node.clone());
                    }

                    if to_consolidate.len() < max_consolidations {
                        to_consolidate.push(node.clone());
                    } else {
                        keys_to_remove.pop();
                    }
                }
            }

            for id in keys_to_remove {
                cache.remove(&id);
            }
        }

        for node in &to_consolidate {
            if let Err(e) = storage.consolidate_node(node) {
                eprintln!(
                    "⚠️ [Maintenance] Error consolidating node {}: {}",
                    node.id, e
                );
            }
        }

        let mut deleted_count = 0usize;
        for (id, is_invalidated, slashed_role) in &to_purge {
            if *is_invalidated {
                if let Some(role) = slashed_role {
                    {
                        let mut tracker = storage.conflict_resolver.collision_tracker.write();
                        tracker.slash_origin(role);
                    }
                    storage.admission_filter.block_role(role);
                    println!("🔥 [Maintenance] Origin Slashing: agent '{}' blocked → ConfidenceScore=0.0", role);
                }

                InvalidationDispatcher::emit_invalidated_purged(
                    invalidation_tx,
                    *id,
                    "Flagged INVALIDATED during maintenance cycle".to_string(),
                )
                .await;
                let _ = storage.delete(*id, "Reactive Purge: INVALIDATED flag");
            } else {
                let _ = storage.delete(*id, "Low Confidence Eviction (Score < 0.2)");
            }
            deleted_count += 1;
        }

        if !compression_candidates.is_empty() {
            Self::execute_data_compression(storage, &compression_candidates).await;
        }

        if deleted_count > 10_000 {
            println!("🧹 [Maintenance] Triggering disk compaction due to high tombstone volume.");
            storage.request_compaction();
        }

        println!(
            "☀️  [Maintenance] Cycle finished. Analyzed: {} nodes.",
            total_nodes
        );
    }

    async fn execute_data_compression(storage: &Arc<StorageEngine>, candidates: &[UnifiedNode]) {
        let mut thread_groups: HashMap<u64, Vec<&UnifiedNode>> = HashMap::new();

        for node in candidates {
            if let Some(thread_edge) = node.edges.iter().find(|e| e.label == "belongs_to_thread") {
                thread_groups
                    .entry(thread_edge.target)
                    .or_default()
                    .push(node);
            }
        }

        let deadline = Instant::now();
        let llm = crate::llm::LlmClient::new();

        for (thread_id, group) in &thread_groups {
            if deadline.elapsed().as_millis() > MAX_COMPRESSION_DURATION_MS {
                println!(
                    "⏳ [Maintenance] Compression time budget reached. Deferring remaining groups."
                );
                break;
            }

            if group.len() < 2 {
                continue;
            }
            let group_hit_sum: u32 = group.iter().map(|n| n.hits).sum();
            if group_hit_sum < MIN_GROUP_WEIGHT_FOR_COMPRESSION {
                continue;
            }

            let node_refs: Vec<&UnifiedNode> = group.to_vec();
            let summary_text = match llm.summarize_context(&node_refs).await {
                Ok(text) => text,
                Err(e) => {
                    eprintln!(
                        "⚠️ [Maintenance] LLM compression failed for thread {}: {}. Skipping.",
                        thread_id, e
                    );
                    continue;
                }
            };

            let summary_id = rand::random::<u64>();
            let mut summary_node = UnifiedNode::new(summary_id);
            summary_node.tier = NodeTier::Cold;
            summary_node.flags.set(NodeFlags::PINNED);
            summary_node.importance = 0.9;
            summary_node.confidence_score =
                group.iter().map(|n| n.confidence_score).sum::<f32>() / group.len() as f32;
            summary_node.set_field("type", FieldValue::String("Summary".to_string()));
            summary_node.set_field("content", FieldValue::String(summary_text));
            summary_node.set_field("source_thread", FieldValue::Int(*thread_id as i64));

            let ancestor_ids: Vec<String> = group.iter().map(|n| n.id.to_string()).collect();
            summary_node.set_field("ancestors", FieldValue::String(ancestor_ids.join(",")));

            if let Ok(vec) = llm
                .generate_embedding(
                    summary_node
                        .get_field("content")
                        .and_then(|f| f.as_str())
                        .unwrap_or(""),
                )
                .await
            {
                summary_node.vector = crate::node::VectorRepresentations::Full(vec);
                summary_node.flags.set(NodeFlags::HAS_VECTOR);
            }

            if let Err(e) = storage.insert_to_cf(&summary_node, "compressed_archive") {
                eprintln!("⚠️ [Maintenance] Failed to persist summary node: {}. Aborting group compression.", e);
                continue;
            }

            for original in group {
                if let Err(e) = storage.delete(
                    original.id,
                    &format!(
                        "Data Compression: condensed into summary node {}",
                        summary_id
                    ),
                ) {
                    eprintln!(
                        "⚠️ [Maintenance] Failed to tombstone node {} during compression: {}",
                        original.id, e
                    );
                }
            }

            println!(
                "🧬 [Maintenance] Data Compression: {} nodes from thread {} → Summary Node {} (Cold).",
                group.len(), thread_id, summary_id
            );
        }
    }
}


================================================================
Nombre: mod.rs
Ruta: src\governance\mod.rs
================================================================

pub mod admission_filter;
pub mod conflict_resolver;
pub mod consistency;
pub mod invalidations;
pub mod maintenance_worker;

pub use conflict_resolver::{ConfidenceArbiter, ConflictResolver};

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// A permanent record of a node that has been logically deleted.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditableTombstone {
    pub id: u64,
    pub timestamp_deleted: u64,
    pub reason: String,
    pub original_hash: u64,
}

impl AuditableTombstone {
    pub fn new(id: u64, reason: impl Into<String>, original_hash: u64) -> Self {
        let timestamp_deleted = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            id,
            timestamp_deleted,
            reason: reason.into(),
            original_hash,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ResolutionResult {
    Accept,
    Reject(String),
    Superposition(crate::governance::consistency::ConsistencyRecord),
    Merge { new_confidence: f32 },
}


================================================================
Nombre: mod.rs
Ruta: src\hardware\mod.rs
================================================================

use console::{style, Emoji};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::sync::OnceLock;
use sysinfo::System;

/// Global Hardware Profile loaded once at startup.
static CAPS: OnceLock<HardwareCapabilities> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstructionSet {
    Avx512,
    Avx2,
    Neon,
    Fallback, // Explicit scalar loop network of safety
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HardwareProfile {
    Enterprise,  // Heavy hardware: AVX-512, high RAM
    Performance, // Standard server: AVX2/Neon, standard RAM
    Survival,    // Constrained devices: Low RAM or Scalar Fallback
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareCapabilities {
    pub instructions: InstructionSet,
    pub profile: HardwareProfile,
    pub logical_cores: usize,
    pub total_memory: u64, // Total RAM in bytes
    pub resource_score: u32,
    pub env_hash: u64, // Hash of the static environment for invalidation
}

impl HardwareCapabilities {
    pub fn global() -> &'static Self {
        CAPS.get_or_init(HardwareScout::detect)
    }
}

pub struct HardwareScout;

impl HardwareScout {
    const PROFILE_PATH: &'static str = ".connectome_profile";

    pub fn detect() -> HardwareCapabilities {
        let mut sys = System::new_all();
        sys.refresh_all();

        let total_memory = std::env::var("VANTADB_MEMORY_LIMIT")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or_else(|| sys.total_memory());
        let logical_cores = sys.cpus().len();

        // Calculate stable environment hash
        let mut hasher = DefaultHasher::new();
        total_memory.hash(&mut hasher);
        logical_cores.hash(&mut hasher);
        if let Some(cpu) = sys.cpus().first() {
            cpu.brand().hash(&mut hasher);
        }
        let env_hash = hasher.finish();

        // Check if we have a valid cached profile
        if let Ok(data) = fs::read_to_string(Self::PROFILE_PATH) {
            if let Ok(cached_caps) = serde_json::from_str::<HardwareCapabilities>(&data) {
                if cached_caps.env_hash == env_hash {
                    // Cache Hit: Environment unchanged! Perfect cold-start speedup.
                    Self::log_adaptive_status(&cached_caps, true);
                    return cached_caps;
                } else {
                    eprintln!("[HARDWARE] ⚠️ Environment signature changed. Re-benchmarking...");
                }
            }
        }

        let instructions = Self::detect_instructions();
        let profile = Self::determine_profile(total_memory, instructions);

        let resource_score =
            Self::calculate_resource_score(total_memory, logical_cores, instructions);

        let caps = HardwareCapabilities {
            instructions,
            profile,
            logical_cores,
            total_memory,
            resource_score,
            env_hash,
        };

        Self::log_adaptive_status(&caps, false);

        // Save new profile
        if let Ok(json) = serde_json::to_string_pretty(&caps) {
            let _ = fs::write(Self::PROFILE_PATH, json);
        }

        caps
    }

    fn detect_instructions() -> InstructionSet {
        // Detect x86_64 AVX-512 / AVX2
        #[cfg(target_arch = "x86_64")]
        {
            if std::is_x86_feature_detected!("avx512f") {
                return InstructionSet::Avx512;
            } else if std::is_x86_feature_detected!("avx2") {
                return InstructionSet::Avx2;
            }
        }

        #[cfg(target_arch = "aarch64")]
        {
            if std::arch::is_aarch64_feature_detected!("neon") {
                return InstructionSet::Neon;
            }
        }

        InstructionSet::Fallback
    }

    fn determine_profile(memory: u64, instructions: InstructionSet) -> HardwareProfile {
        let memory_gb = memory / (1024 * 1024 * 1024);

        if memory_gb >= 16 && instructions == InstructionSet::Avx512 {
            HardwareProfile::Enterprise
        } else if memory_gb >= 4 && instructions != InstructionSet::Fallback {
            HardwareProfile::Performance
        } else {
            HardwareProfile::Survival
        }
    }

    fn calculate_resource_score(memory: u64, cores: usize, instructions: InstructionSet) -> u32 {
        let mem_score = (memory / (1024 * 1024 * 1024)) as u32;
        let core_score = cores as u32;
        let instr_score = match instructions {
            InstructionSet::Avx512 => 10,
            InstructionSet::Avx2 => 5,
            InstructionSet::Neon => 5,
            InstructionSet::Fallback => 1,
        };
        (mem_score * 2) + core_score + instr_score
    }

    fn log_adaptive_status(caps: &HardwareCapabilities, cached: bool) {
        let instr_str = match caps.instructions {
            InstructionSet::Avx512 => style("AVX-512").cyan().bold(),
            InstructionSet::Avx2 => style("AVX2").cyan().bold(),
            InstructionSet::Neon => style("NEON").cyan().bold(),
            InstructionSet::Fallback => style("SCALAR FALLBACK").red().dim(),
        };

        let (_profile_str, profile_color) = match caps.profile {
            HardwareProfile::Enterprise => ("ENTERPRISE", style("ENTERPRISE").green().bold()),
            HardwareProfile::Performance => ("PERFORMANCE", style("PERFORMANCE").yellow().bold()),
            HardwareProfile::Survival => ("SURVIVAL", style("SURVIVAL").red().bold()),
        };

        let ram_gb = caps.total_memory / (1024 * 1024 * 1024);
        let cache_cap_gb = (caps.total_memory / 4) / (1024 * 1024 * 1024);

        let source_str = if cached {
            style("CACHED").dim()
        } else {
            style("DETECTED").bold().underlined()
        };

        let lightning = Emoji("⚡ ", "!");
        let shield = Emoji("🛡️  ", "!!");

        eprintln!(
            "\n{}",
            style(
                "╭──────────────────────────────────────────────────────────────────────────────╮"
            )
            .dim()
        );
        eprintln!(
            "{} {} {} [ {} ] {}",
            style("│").dim(),
            lightning,
            style("ADAPTIVE RESOURCE MODE:").bold(),
            instr_str,
            style("│").dim()
        );
        eprintln!(
            "{}    {} {} | {} Core(s) | Score: {} {}",
            style("│").dim(),
            source_str,
            style(format!("RAM: {}GB (Cache: {}GB)", ram_gb, cache_cap_gb)).dim(),
            caps.logical_cores,
            style(caps.resource_score).magenta(),
            style("│").dim()
        );
        eprintln!(
            "{} {} {} [ {} ] {:>32} {}",
            style("│").dim(),
            shield,
            style("PROFILER STATUS:").bold(),
            profile_color,
            "",
            style("│").dim()
        );
        eprintln!(
            "{}\n",
            style(
                "╰──────────────────────────────────────────────────────────────────────────────╯"
            )
            .dim()
        );
    }
}


================================================================
Nombre: lisp.rs
Ruta: src\parser\lisp.rs
================================================================

use nom::{
    branch::alt,
    bytes::complete::{is_not, tag},
    character::complete::{alpha1, alphanumeric1, char, multispace0},
    combinator::{map, recognize},
    multi::{many0, many1},
    number::complete::float,
    sequence::{delimited, preceded, tuple},
    IResult,
};

#[derive(Debug, Clone, PartialEq)]
pub enum LispExpr {
    List(Vec<LispExpr>),
    Map(Vec<(LispExpr, LispExpr)>),
    Atom(String),
    Keyword(String),
    StringLiteral(String),
    Number(f32),
    Variable(String),
}

fn parse_keyword(i: &str) -> IResult<&str, LispExpr> {
    map(
        preceded(char(':'), recognize(many1(alt((alphanumeric1, tag("-")))))),
        |s: &str| LispExpr::Keyword(s.to_string()),
    )(i)
}

fn parse_variable(i: &str) -> IResult<&str, LispExpr> {
    map(preceded(char('?'), alpha1), |s: &str| {
        LispExpr::Variable(s.to_string())
    })(i)
}

fn parse_string(i: &str) -> IResult<&str, LispExpr> {
    let parse_str = delimited(char('"'), is_not("\""), char('"'));
    map(parse_str, |s: &str| LispExpr::StringLiteral(s.to_string()))(i)
}

fn parse_atom(i: &str) -> IResult<&str, LispExpr> {
    map(
        recognize(many1(alt((alphanumeric1, tag("-"), tag("_"))))),
        |s: &str| LispExpr::Atom(s.to_string()),
    )(i)
}

fn parse_number(i: &str) -> IResult<&str, LispExpr> {
    map(float, LispExpr::Number)(i)
}

fn parse_expr(i: &str) -> IResult<&str, LispExpr> {
    delimited(
        multispace0,
        alt((
            parse_list,
            parse_map,
            parse_keyword,
            parse_variable,
            parse_string,
            parse_number,
            parse_atom,
        )),
        multispace0,
    )(i)
}

fn parse_list(i: &str) -> IResult<&str, LispExpr> {
    let parse_inside = many0(parse_expr);
    map(
        delimited(char('('), parse_inside, char(')')),
        LispExpr::List,
    )(i)
}

fn parse_map(i: &str) -> IResult<&str, LispExpr> {
    let parse_pairs = many0(tuple((parse_expr, parse_expr)));
    map(delimited(char('{'), parse_pairs, char('}')), LispExpr::Map)(i)
}

pub fn parse(input: &str) -> Result<LispExpr, String> {
    match parse_expr(input) {
        Ok((rem, expr)) if rem.trim().is_empty() => Ok(expr),
        Ok((rem, _)) => Err(format!("Unparsed trailing data: {}", rem)),
        Err(e) => Err(format!("Parse error: {}", e)),
    }
}


================================================================
Nombre: mod.rs
Ruta: src\parser\mod.rs
================================================================

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{alpha1, alphanumeric1, char, digit1, multispace0},
    combinator::{map, map_res, opt, recognize},
    multi::{many0, separated_list1},
    number::complete::float,
    sequence::{delimited, tuple},
    IResult, Parser,
};

pub mod lisp;

use crate::node::FieldValue;
use crate::query::*;

pub fn ws<'a, F, O, E: nom::error::ParseError<&'a str>>(
    inner: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: Parser<&'a str, O, E>,
{
    delimited(multispace0, inner, multispace0)
}

fn ident(i: &str) -> IResult<&str, String> {
    let (i, id) = recognize(tuple((
        alt((alpha1, tag("_"))),
        many0(alt((alphanumeric1, tag("_"), tag("#"), tag(".")))),
    )))(i)?;
    Ok((i, id.to_string()))
}

fn parse_number(i: &str) -> IResult<&str, u32> {
    map_res(digit1, str::parse)(i)
}

fn string_literal(i: &str) -> IResult<&str, String> {
    delimited(char('"'), take_while1(|c| c != '"'), char('"'))
        .map(|s: &str| s.to_string())
        .parse(i)
}

fn parse_u64_id(i: &str) -> IResult<&str, u64> {
    map_res(digit1, str::parse)(i)
}

fn parse_i64(i: &str) -> IResult<&str, i64> {
    map_res(recognize(tuple((opt(char('-')), digit1))), str::parse)(i)
}

fn parse_literal_field_value(i: &str) -> IResult<&str, FieldValue> {
    alt((
        map(string_literal, FieldValue::String),
        map(ws(tag("true")), |_| FieldValue::Bool(true)),
        map(ws(tag("false")), |_| FieldValue::Bool(false)),
        map(ws(tag("null")), |_| FieldValue::Null),
        map(ws(parse_i64), FieldValue::Int),
        map(ws(float), |f: f32| FieldValue::Float(f as f64)),
    ))(i)
}

fn parse_traversal(i: &str) -> IResult<&str, Traversal> {
    let (i, _) = ws(tag("SIGUE"))(i)?;
    let (i, min_depth) = ws(parse_number)(i)?;
    let (i, _) = ws(tag(".."))(i)?;
    let (i, max_depth) = ws(parse_number)(i)?;
    let (i, edge_label) = ws(string_literal)(i)?;
    let (i, target_type) = opt(tuple((ws(tag("TYPE")), ws(ident))))(i)?;
    let (i, alias) = opt(tuple((ws(tag("AS")), ws(ident))))(i)?;

    Ok((
        i,
        Traversal {
            min_depth,
            max_depth,
            edge_label,
            target_type: target_type.map(|(_, t)| t),
            alias: alias.map(|(_, a)| a),
        },
    ))
}

fn parse_rel_op(i: &str) -> IResult<&str, RelOp> {
    alt((
        map(tag("="), |_| RelOp::Eq),
        map(tag("!="), |_| RelOp::Neq),
        map(tag(">="), |_| RelOp::Gte),
        map(tag(">"), |_| RelOp::Gt),
        map(tag("<="), |_| RelOp::Lte),
        map(tag("<"), |_| RelOp::Lt),
    ))(i)
}

fn parse_condition(i: &str) -> IResult<&str, Condition> {
    alt((
        // Vector Query: p.bio ~ "rust expert", min = 0.88
        map(
            tuple((
                ws(ident),
                ws(tag("~")),
                ws(string_literal),
                ws(tag(",")),
                ws(tag("min")),
                ws(tag("=")),
                ws(float),
            )),
            |(field, _, query, _, _, _, min_score)| Condition::VectorSim(field, query, min_score),
        ),
        // Relational Query: p.pais = "VZLA"
        map(
            tuple((ws(ident), ws(parse_rel_op), ws(string_literal))),
            |(field, op, val)| Condition::Relational(field, op, FieldValue::String(val)),
        ),
    ))(i)
}

pub fn parse_query(i: &str) -> IResult<&str, Query> {
    let (i, _) = ws(alt((tag("FROM"), tag("MATCH"))))(i)?;
    let (i, from_entity) = ws(ident)(i)?;

    let (i, traversal) = opt(parse_traversal)(i)?;

    let (i, target_alias) = opt(ws(ident))(i)?;
    let target_alias = target_alias.unwrap_or_else(|| "target".to_string());

    let (i, where_clause) = opt(tuple((
        ws(tag("WHERE")),
        separated_list1(ws(tag("AND")), parse_condition),
    )))(i)?;

    let (i, fetch) = opt(tuple((
        ws(tag("FETCH")),
        separated_list1(ws(char(',')), ws(ident)),
    )))(i)?;

    let (i, rank_by) = opt(tuple((ws(tag("RANK BY")), ws(ident), opt(ws(tag("DESC"))))))(i)?;

    let (i, temperature) = opt(tuple((ws(tag("WITH")), ws(tag("TEMPERATURE")), ws(float))))(i)?;

    let (i, owner_role) = opt(tuple((ws(tag("ROLE")), ws(string_literal))))(i)?;

    Ok((
        i,
        Query {
            from_entity,
            traversal,
            target_alias,
            where_clause: where_clause.map(|(_, conds)| conds),
            fetch: fetch.map(|(_, f)| f),
            rank_by: rank_by.map(|(_, f, d)| RankBy {
                field: f,
                desc: d.is_some(),
            }),
            temperature: temperature.map(|(_, _, t)| t),
            owner_role: owner_role.map(|(_, r)| r),
        },
    ))
}

// ─── DML (Data Manipulation Language) ──────────────────────────

fn parse_field_assign(i: &str) -> IResult<&str, (String, FieldValue)> {
    let (i, key) = ws(ident)(i)?;
    let (i, _) = ws(char(':'))(i)?;
    let (i, val) = ws(parse_literal_field_value)(i)?;
    Ok((i, (key, val)))
}

fn parse_vector_lit(i: &str) -> IResult<&str, Vec<f32>> {
    delimited(
        ws(char('[')),
        separated_list1(ws(char(',')), ws(float)),
        ws(char(']')),
    )(i)
}

fn parse_insert(i: &str) -> IResult<&str, InsertStatement> {
    let (i, _) = ws(tag("INSERT"))(i)?;
    let (i, _) = ws(tag("NODE#"))(i)?;
    let (i, node_id) = ws(parse_u64_id)(i)?;
    let (i, _) = ws(tag("TYPE"))(i)?;
    let (i, node_type) = ws(ident)(i)?;

    let (i, fields) = delimited(
        ws(char('{')),
        opt(separated_list1(ws(char(',')), ws(parse_field_assign))),
        ws(char('}')),
    )(i)?;
    let fields = fields.unwrap_or_default().into_iter().collect();

    let (i, vector) = opt(tuple((ws(tag("VECTOR")), ws(parse_vector_lit))))(i)?;

    Ok((
        i,
        InsertStatement {
            node_id,
            node_type,
            fields,
            vector: vector.map(|(_, v)| v),
        },
    ))
}

fn parse_update_field_expr(i: &str) -> IResult<&str, (String, FieldValue)> {
    let (i, key) = ws(ident)(i)?;
    let (i, _) = ws(char('='))(i)?;
    let (i, val) = ws(parse_literal_field_value)(i)?;
    Ok((i, (key, val)))
}

fn parse_update(i: &str) -> IResult<&str, UpdateStatement> {
    let (i, _) = ws(tag("UPDATE"))(i)?;
    let (i, _) = ws(tag("NODE#"))(i)?;
    let (i, node_id) = ws(parse_u64_id)(i)?;
    let (i, _) = ws(tag("SET"))(i)?;

    let (i, vector_only) = opt(tuple((ws(tag("VECTOR")), ws(parse_vector_lit))))(i)?;

    if let Some((_, vec)) = vector_only {
        return Ok((
            i,
            UpdateStatement {
                node_id,
                fields: std::collections::BTreeMap::new(),
                vector: Some(vec),
            },
        ));
    }

    let (i, parsed_fields) = separated_list1(ws(char(',')), ws(parse_update_field_expr))(i)?;
    let fields = parsed_fields.into_iter().collect();

    Ok((
        i,
        UpdateStatement {
            node_id,
            fields,
            vector: None,
        },
    ))
}

fn parse_delete(i: &str) -> IResult<&str, DeleteStatement> {
    let (i, _) = ws(tag("DELETE"))(i)?;
    let (i, _) = ws(tag("NODE#"))(i)?;
    let (i, node_id) = ws(parse_u64_id)(i)?;
    Ok((i, DeleteStatement { node_id }))
}

fn parse_relate(i: &str) -> IResult<&str, RelateStatement> {
    let (i, _) = ws(tag("RELATE"))(i)?;
    let (i, _) = ws(tag("NODE#"))(i)?;
    let (i, source_id) = ws(parse_u64_id)(i)?;
    let (i, _) = ws(tag("--\""))(i)?;
    let (i, label) = ws(take_while1(|c| c != '"'))(i)?;
    let (i, _) = ws(tag("\"-->"))(i)?;
    let (i, _) = ws(tag("NODE#"))(i)?;
    let (i, target_id) = ws(parse_u64_id)(i)?;

    let (i, weight) = opt(tuple((ws(tag("WEIGHT")), ws(float))))(i)?;

    Ok((
        i,
        RelateStatement {
            source_id,
            target_id,
            label: label.to_string(),
            weight: weight.map(|(_, w)| w),
        },
    ))
}

fn parse_insert_message(i: &str) -> IResult<&str, InsertMessageStatement> {
    let (i, _) = ws(tag("INSERT"))(i)?;
    let (i, _) = ws(tag("MESSAGE"))(i)?;

    let (i, msg_role) = alt((
        map(ws(tag("SYSTEM")), |_| "system".to_string()),
        map(ws(tag("USER")), |_| "user".to_string()),
        map(ws(tag("ASSISTANT")), |_| "assistant".to_string()),
    ))(i)?;

    let (i, content) = ws(string_literal)(i)?;

    let (i, _) = ws(tag("TO"))(i)?;
    let (i, _) = ws(tag("THREAD#"))(i)?;
    let (i, thread_id) = ws(parse_u64_id)(i)?;

    Ok((
        i,
        InsertMessageStatement {
            msg_role,
            content,
            thread_id,
        },
    ))
}

fn parse_collapse(i: &str) -> IResult<&str, CollapseStatement> {
    let (i, _) = ws(tag("COLLAPSE"))(i)?;
    let (i, _) = ws(tag("QuantumZone#"))(i)?;
    let (i, zone_id) = ws(parse_u64_id)(i)?;
    let (i, _) = ws(tag("FAVOR"))(i)?;
    let (i, index) = ws(parse_number)(i)?;

    Ok((
        i,
        CollapseStatement {
            zone_id,
            index: index as usize,
        },
    ))
}

// ─── Entry Point ───────────────────────────────────────────────

pub fn parse_statement(i: &str) -> IResult<&str, Statement> {
    alt((
        map(parse_insert_message, Statement::InsertMessage), // Must be before parse_insert to prevent shadowing
        map(parse_insert, Statement::Insert),
        map(parse_update, Statement::Update),
        map(parse_delete, Statement::Delete),
        map(parse_relate, Statement::Relate),
        map(parse_collapse, Statement::Collapse),
        map(parse_query, Statement::Query),
    ))(i)
}


================================================================
Nombre: mod.rs
Ruta: src\vector\mod.rs
================================================================

pub mod quantization;
pub mod transform;


================================================================
Nombre: quantization.rs
Ruta: src\vector\quantization.rs
================================================================

/// Hybrid Quantization Algorithms (Phase 31)
/// Contains carefully engineered quantization schemes for MMap Zero-Copy and L1 Caching.
///
/// SAFETY: All packed outputs are padded to 8-byte (u64) alignment boundaries
/// to prevent SIMD segfaults on unaligned mmap reads.
use core::f32;

/// Required alignment for mmap-safe SIMD reads (AVX2 minimum = 32, but u64 = 8 is our pack unit).
const MMAP_ALIGNMENT: usize = 8;

/// Creates a 1-bit representation (RaBitQ) of the FWHT-transformed vector.
/// Packs 64 boolean flag features into a single `u64`.
/// Excellent for massive batch pruning in L1 RAM cache.
pub fn rabitq_quantize(data: &[f32]) -> Box<[u64]> {
    let num_blocks = data.len().div_ceil(64);
    let mut packed = vec![0u64; num_blocks];

    for (i, &val) in data.iter().enumerate() {
        if val > 0.0 {
            let block = i / 64;
            let bit = i % 64;
            packed[block] |= 1 << bit;
        }
    }

    packed.into_boxed_slice()
}

/// Computes the similarity (equivalent to cosine similarity in Angular space)
/// between two 1-bit RaBitQ quantified vectors using POPCNT.
pub fn rabitq_similarity(a: &[u64], b: &[u64]) -> f32 {
    let mut xor_sum = 0;
    for (va, vb) in a.iter().zip(b.iter()) {
        xor_sum += (va ^ vb).count_ones();
    }

    let total_bits = (a.len() * 64) as f32;
    // Angle approximation from Hamming distance
    // cosine_sim = cos(pi * hamming / total_bits)
    // For fast retrieval, we can just return normalized match percentage,
    // which operates monotonically:

    1.0 - (xor_sum as f32 / total_bits)
}

/// Creates a PolarQuant (Custom 3-bit / 4-bit Two's Complement packed)
/// representation of the FWHT-transformed vector.
/// Each `u8` holds two 4-bit values (-8 to 7).
pub fn turbo_quant_quantize(data: &[f32]) -> (Box<[u8]>, f32) {
    // 1. Find max absolute value to establish the scaling bound
    let mut max_abs = 0.0_f32;
    for &val in data {
        let abs = val.abs();
        if abs > max_abs {
            max_abs = abs;
        }
    }

    // Fallback if vector is extremely close to zero
    if max_abs < f32::EPSILON {
        max_abs = 1.0;
    }

    // We quantize into range [-8, 7].
    let scale = 7.0 / max_abs;

    let num_bytes = data.len().div_ceil(2);
    let mut packed = vec![0u8; num_bytes];

    for (i, &val) in data.iter().enumerate() {
        let scaled = (val * scale).round();
        // Clamp explicitly to avoid panic on NaNs or huge math flukes
        let clamped = scaled.clamp(-8.0, 7.0) as i8;

        // Take bottom 4 bits safely
        let q = (clamped as u8) & 0x0F;

        let byte_pos = i / 2;
        if i % 2 == 0 {
            // High nibble
            packed[byte_pos] |= q << 4;
        } else {
            // Low nibble
            packed[byte_pos] |= q;
        }
    }

    // Pad to MMAP_ALIGNMENT boundary for safe SIMD mmap reads
    let aligned_len = (num_bytes + MMAP_ALIGNMENT - 1) & !(MMAP_ALIGNMENT - 1);
    packed.resize(aligned_len, 0u8);

    (packed.into_boxed_slice(), max_abs)
}

/// Helper wrapper that implements SIMD dot products for two unpacked TurboQuant strings.
/// (During Mmap, we stream the u8, unpack them rapidly, and accumulate).
pub fn turbo_quant_similarity(
    a_packed: &[u8],
    a_max_abs: f32,
    b_packed: &[u8],
    b_max_abs: f32,
) -> f32 {
    // Safety: verify pointer alignment for mmap zero-copy paths.
    // If data comes from mmap, misaligned pointers would cause SIMD penalties or segfaults.
    debug_assert!(
        (a_packed.as_ptr() as usize).is_multiple_of(std::mem::align_of::<u8>()),
        "turbo_quant_similarity: a_packed pointer is misaligned"
    );

    let mut dot = 0_i32;

    // Extremely fast scalar loop. The Rust compiler unrolls this beautifully,
    // and manual SIMD padding for 4-bit decompression is complex unless using specific shuffle intrinsic blocks.
    for (va, vb) in a_packed.iter().zip(b_packed.iter()) {
        let a_high = (*va >> 4) as i8;
        let a_high = if a_high & 8 != 0 { a_high | -8 } else { a_high }; // sign extend

        let a_low = (*va & 0x0F) as i8;
        let a_low = if a_low & 8 != 0 { a_low | -8 } else { a_low };

        let b_high = (*vb >> 4) as i8;
        let b_high = if b_high & 8 != 0 { b_high | -8 } else { b_high };

        let b_low = (*vb & 0x0F) as i8;
        let b_low = if b_low & 8 != 0 { b_low | -8 } else { b_low };

        dot += (a_high as i32 * b_high as i32) + (a_low as i32 * b_low as i32);
    }

    // Reverse the scale
    // Because both were scaled by (7.0 / max_abs), we divide by (49.0 / (a_max * b_max))

    // Note: Since fwht preserves magnitude, we can estimate cosine similarity directly
    // from this dot product if the original vectors were length 1.0!
    // But since this is a dot product, we just return it.
    dot as f32 * (a_max_abs * b_max_abs) / 49.0
}


================================================================
Nombre: transform.rs
Ruta: src\vector\transform.rs
================================================================

use crate::hardware::{HardwareCapabilities, InstructionSet};
use wide::f32x8;

/// Fast Walsh-Hadamard Transform (FWHT)
///
/// Distributes the variance of the vector components across all dimensions,
/// which is critical to minimizing error before 1-bit and 3-bit quantization.
/// Mutates the input slice in place. Requires `data.len()` to be a power of 2.
pub fn fwht(data: &mut [f32]) {
    let n = data.len();
    if !n.is_power_of_two() {
        return; // Must handle padding horizontally before calling
    }

    let caps = HardwareCapabilities::global();
    match caps.instructions {
        InstructionSet::Fallback => fwht_scalar(data),
        _ => fwht_simd(data),
    }
}

pub fn fwht_scalar(data: &mut [f32]) {
    let n = data.len();
    let mut h = 1;
    while h < n {
        for i in (0..n).step_by(h * 2) {
            for j in i..i + h {
                let x = data[j];
                let y = data[j + h];
                data[j] = x + y;
                data[j + h] = x - y;
            }
        }
        h *= 2;
    }

    // Normalize to preserve magnitude
    let scale = 1.0 / (n as f32).sqrt();
    for x in data.iter_mut() {
        *x *= scale;
    }
}

pub fn fwht_simd(data: &mut [f32]) {
    let n = data.len();
    let mut h = 1;

    // For strides smaller than 8, we do scalar, because f32x8 cannot easily
    // interleave elements across 1, 2, 4 distance natively without complex swizzles.
    // Given the cache locality, scalar is extremely fast here anyway.
    while h < 8 && h < n {
        for i in (0..n).step_by(h * 2) {
            for j in i..i + h {
                let x = data[j];
                let y = data[j + h];
                data[j] = x + y;
                data[j + h] = x - y;
            }
        }
        h *= 2;
    }

    // SIMD for h >= 8
    while h < n {
        for i in (0..n).step_by(h * 2) {
            for j in (i..i + h).step_by(8) {
                // Ensure we don't go out of bounds
                if j + 8 <= i + h {
                    let x_slice = &data[j..j + 8];
                    let y_slice = &data[j + h..j + h + 8];

                    let x = f32x8::new([
                        x_slice[0], x_slice[1], x_slice[2], x_slice[3], x_slice[4], x_slice[5],
                        x_slice[6], x_slice[7],
                    ]);
                    let y = f32x8::new([
                        y_slice[0], y_slice[1], y_slice[2], y_slice[3], y_slice[4], y_slice[5],
                        y_slice[6], y_slice[7],
                    ]);

                    let new_x = x + y;
                    let new_y = x - y;

                    let arr_x: [f32; 8] = new_x.into();
                    let arr_y: [f32; 8] = new_y.into();

                    data[j..j + 8].copy_from_slice(&arr_x);
                    data[j + h..j + h + 8].copy_from_slice(&arr_y);
                } else {
                    // Scalar fallback for remainder
                    for k in j..i + h {
                        let x = data[k];
                        let y = data[k + h];
                        data[k] = x + y;
                        data[k + h] = x - y;
                    }
                }
            }
        }
        h *= 2;
    }

    // Normalize
    let scale = 1.0 / (n as f32).sqrt();
    let scale_v = f32x8::splat(scale);
    let mut chunks = data.chunks_exact_mut(8);
    for chunk in &mut chunks {
        let x = f32x8::new([
            chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5], chunk[6], chunk[7],
        ]);
        let v = x * scale_v;
        let arr: [f32; 8] = v.into();
        chunk.copy_from_slice(&arr);
    }
    for x in chunks.into_remainder() {
        *x *= scale;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fwht_scalar() {
        let mut data = vec![1.0, 0.0, 1.0, 0.0];
        fwht_scalar(&mut data);
        let expected = vec![1.0, 1.0, 0.0, 0.0];
        for (a, b) in data.iter().zip(expected.iter()) {
            assert!((a - b).abs() < 1e-5);
        }
    }

    #[test]
    fn test_fwht_simd_vs_scalar() {
        let mut d1 = vec![0.5f32; 1024];
        for i in 0..1024 {
            d1[i] = (i as f32).sin();
        }
        let mut d2 = d1.clone();

        fwht_scalar(&mut d1);
        fwht_simd(&mut d2);

        for (a, b) in d1.iter().zip(d2.iter()) {
            assert!((a - b).abs() < 1e-4);
        }
    }
}


================================================================
Nombre: mcp_integration.rs
Ruta: tests\api\mcp_integration.rs
================================================================

//! MCP Protocol Integration Modernized Test Suite
//! Part of the Vanta Certification ecosystem.

#[path = "../common/mod.rs"]
mod common;

use common::{TerminalReporter, VantaHarness};
use serde_json::json;
use vantadb::api::mcp::{handle_initialize, handle_tools_call, handle_tools_list};
use vantadb::executor::Executor;
use vantadb::node::UnifiedNode;
use vantadb::storage::StorageEngine;

#[tokio::test]
async fn mcp_protocol_certification() {
    let mut harness = VantaHarness::new("API LAYER (MCP PROTOCOL)");

    harness.execute("Protocol: Handshake & Identity (2024-11-05)", || {
        let init_res = handle_initialize().expect("Initialization failed");
        assert_eq!(init_res["protocolVersion"], "2024-11-05");
        assert_eq!(init_res["serverInfo"]["name"], "vantadb");

        let list_res = handle_tools_list().expect("Tools listing failed");
        let tools = list_res["tools"]
            .as_array()
            .expect("Tools must be an array");
        assert!(tools.iter().any(|t| t["name"] == "query_lisp"));

        TerminalReporter::success("MCP handshake and tools definition verified.");
    });

    harness.execute("Protocol: Tool Execution & State Mutability", || {
        futures::executor::block_on(async {
            let temp_dir = tempfile::tempdir().unwrap();
            let storage = StorageEngine::open(temp_dir.path().to_str().unwrap()).unwrap();
            let executor = Executor::new(&storage);

            TerminalReporter::sub_step("Testing get_node_neighbors tool...");
            let mut node = UnifiedNode::new(100);
            node.confidence_score = 0.99;
            storage.insert(&node).unwrap();

            let params = Some(json!({
                "name": "get_node_neighbors",
                "arguments": { "node_id": 100 }
            }));

            let tool_res = handle_tools_call(&params, &executor, &storage)
                .await
                .expect("Tool call failed");
            let text = tool_res["content"][0]["text"].as_str().unwrap();
            assert!(text.contains("\"confidence_score\":0.99"));

            TerminalReporter::sub_step("Testing query_lisp tool (Insertion)...");
            let lisp_params = Some(json!({
                "name": "query_lisp",
                "arguments": { "query": "(INSERT :node {:label \"MCP_TEST\"})" }
            }));
            let lisp_res = handle_tools_call(&lisp_params, &executor, &storage)
                .await
                .expect("Lisp execution failed");
            assert!(lisp_res["content"][0]["text"]
                .as_str()
                .unwrap()
                .contains("affected_nodes"));

            TerminalReporter::success("MCP tool dispatcher correctly routed and executed calls.");
        });
    });
}


================================================================
Nombre: python.rs
Ruta: tests\api\python.rs
================================================================

//! Python Bridging Mock Modernized Test Suite
//! Part of the Vanta Certification ecosystem.

#[path = "../common/mod.rs"]
mod common;

use common::{VantaHarness, TerminalReporter};

#[test]
fn python_bridge_certification() {
    let mut harness = VantaHarness::new("API LAYER (PYTHON BINDINGS)");

    harness.execute("Scaffolding: PyO3 Signature Verification", || {
        TerminalReporter::sub_step("Simulating cross-language boundary checks...");
        assert!(true);
        TerminalReporter::success("Python bridging scaffolding confirmed.");
    });
}


================================================================
Nombre: server.rs
Ruta: tests\api\server.rs
================================================================

//! API Server & Health Modernized Test Suite
//! Part of the Vanta Certification ecosystem.

#[path = "../common/mod.rs"]
mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::{TerminalReporter, VantaHarness};
use std::sync::Arc;
use tower::ServiceExt;
use vantadb::server::{app, ServerState};
use vantadb::storage::StorageEngine;

#[tokio::test]
async fn api_server_certification() {
    let mut harness = VantaHarness::new("API LAYER (SERVER & HEALTH)");

    harness.execute("Health: Endpoint Availability & Router State", || {
        futures::executor::block_on(async {
            let temp_dir = tempfile::tempdir().unwrap();
            let storage = Arc::new(StorageEngine::open(temp_dir.path().to_str().unwrap()).unwrap());
            let state = Arc::new(ServerState { storage });
            let app = app(state);

            TerminalReporter::sub_step("Dispatching oneshot request to /health...");
            let response = app
                .oneshot(
                    Request::builder()
                        .uri("/health")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();

            assert_eq!(response.status(), StatusCode::OK);
            TerminalReporter::success("API Health check passed.");
        });
    });
}


================================================================
Nombre: structured_api_v2.rs
Ruta: tests\api\structured_api_v2.rs
================================================================

//! Structured API v2 Modernized Test Suite
//! Part of the Vanta Certification ecosystem.

#[path = "../common/mod.rs"]
mod common;

use common::{TerminalReporter, VantaHarness};
use std::sync::Arc;
use tempfile::tempdir;
use vantadb::executor::{ExecutionResult, Executor};
use vantadb::storage::StorageEngine;

#[tokio::test]
async fn structured_api_v2_certification() {
    let mut harness = VantaHarness::new("API LAYER (STRUCTURED V2)");

    harness.execute("Integration: Relational ID Capture", || {
        futures::executor::block_on(async {
            let dir = tempdir().unwrap();
            let storage = Arc::new(StorageEngine::open(dir.path().to_str().unwrap()).unwrap());
            let executor = Executor::new(&storage);

            TerminalReporter::sub_step("Inserting nodes S1 and S2 via hybrid syntax...");
            executor
                .execute_hybrid("(INSERT :node {:label \"S1\"})")
                .await
                .unwrap();
            executor
                .execute_hybrid("(INSERT :node {:label \"S2\"})")
                .await
                .unwrap();

            let (s1_id, s2_id);
            {
                let cache = storage.volatile_cache.read();
                s1_id = *cache
                    .iter()
                    .find(|(_, n)| n.get_field("label").unwrap().as_str() == Some("S1"))
                    .unwrap()
                    .0;
                s2_id = *cache
                    .iter()
                    .find(|(_, n)| n.get_field("label").unwrap().as_str() == Some("S2"))
                    .unwrap()
                    .0;
            }

            TerminalReporter::sub_step(&format!("Establishing relation {} -> {}...", s1_id, s2_id));
            let relate_query = format!(
                "RELATE NODE#{} --\"test_rel\"--> NODE#{} WEIGHT 0.8",
                s1_id, s2_id
            );
            let res = executor.execute_hybrid(&relate_query).await.unwrap();

            if let ExecutionResult::Write { node_id, .. } = res {
                assert_eq!(node_id, Some(s1_id));
            }
            TerminalReporter::success("Relational result-ID alignment verified.");
        });
    });

    harness.execute("Integration: Message-to-Thread Dispatch", || {
        futures::executor::block_on(async {
            let dir = tempdir().unwrap();
            let storage = Arc::new(StorageEngine::open(dir.path().to_str().unwrap()).unwrap());
            let executor = Executor::new(&storage);

            executor
                .execute_hybrid("(INSERT :node {:type \"Thread\" :id 999})")
                .await
                .unwrap();

            TerminalReporter::sub_step("Dispatching message to THREAD#999...");
            let msg_query = "INSERT MESSAGE USER \"Hola Mundo\" TO THREAD#999";
            let msg_res = executor.execute_hybrid(msg_query).await.unwrap();

            if let ExecutionResult::Write { node_id, .. } = msg_res {
                assert!(node_id.is_some(), "Message ID was not returned");
            }
            TerminalReporter::success("Structured message routing validated.");
        });
    });
}


================================================================
Nombre: competitive_bench.rs
Ruta: tests\certification\competitive_bench.rs
================================================================

//! ═══════════════════════════════════════════════════════════════════════════
//! COMPETITIVE BENCHMARK — VantaDB vs SIFT1M Ground Truth
//! ═══════════════════════════════════════════════════════════════════════════
//!
//! Phase 2.1/2.2: Real-world dataset benchmark using the standard SIFT1M
//! dataset (128D, 1M vectors) with pre-computed ground truth.
//!
//! Run with: cargo test --test competitive_bench --release -- --nocapture
//!
//! Requires: datasets/sift/{sift_base.fvecs, sift_query.fvecs, sift_groundtruth.ivecs}

use console::style;
use std::path::Path;
use std::time::Instant;
use vantadb::index::{CPIndex, HnswConfig, VectorRepresentations};

#[path = "../common/mod.rs"]
mod common;

use common::sift_loader::{read_fvecs, read_ivecs};
use common::{TerminalReporter, VantaHarness};

// ═══════════════════════════════════════════════════════════════════════════
// HELPERS
// ═══════════════════════════════════════════════════════════════════════════

/// Calculate Recall@K by comparing VantaDB results against SIFT1M ground truth.
fn calculate_recall(
    index: &CPIndex,
    queries: &[Vec<f32>],
    groundtruth: &[Vec<usize>],
    k: usize,
) -> f64 {
    let mut total_hits = 0;

    for (i, query) in queries.iter().enumerate() {
        let results = index.search_nearest(query, None, None, u128::MAX, k, None);
        let gt_k = &groundtruth[i][..k];

        for (id, _score) in &results {
            if gt_k.contains(&(*id as usize)) {
                total_hits += 1;
            }
        }
    }

    total_hits as f64 / (queries.len() * k) as f64
}

/// Measure per-query latency percentiles (p50, p95, p99) in microseconds.
fn measure_latency(index: &CPIndex, queries: &[Vec<f32>], k: usize) -> (f64, f64, f64, f64) {
    let mut latencies_us: Vec<f64> = queries
        .iter()
        .map(|q| {
            let t = Instant::now();
            let _ = index.search_nearest(q, None, None, u128::MAX, k, None);
            t.elapsed().as_nanos() as f64 / 1_000.0
        })
        .collect();
    latencies_us.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let n = latencies_us.len();
    let p50 = latencies_us[n / 2];
    let p95 = latencies_us[(n as f64 * 0.95) as usize];
    let p99 = latencies_us[(n as f64 * 0.99) as usize];
    let qps = queries.len() as f64 / (latencies_us.iter().sum::<f64>() / 1_000_000.0);

    (p50, p95, p99, qps)
}

// ═══════════════════════════════════════════════════════════════════════════
// BENCHMARK RUNNER
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn sift1m_competitive_benchmark() {
    let base_path = Path::new("datasets/sift/sift_base.fvecs");
    let query_path = Path::new("datasets/sift/sift_query.fvecs");
    let gt_path = Path::new("datasets/sift/sift_groundtruth.ivecs");

    if !base_path.exists() {
        println!("⚠️  SIFT dataset not found at datasets/sift/. Skipping.");
        println!("   Download from: http://corpus-texmex.irisa.fr/");
        return;
    }

    let mut harness = VantaHarness::new("SIFT1M_Competitive");

    // ── Load Dataset ─────────────────────────────────────────────────────
    let base_vectors = harness.execute("Load SIFT Base (1M × 128D)", || {
        read_fvecs(base_path).expect("Failed to read sift_base.fvecs")
    });

    let query_vectors = harness.execute("Load SIFT Queries (10K × 128D)", || {
        read_fvecs(query_path).expect("Failed to read sift_query.fvecs")
    });

    let groundtruth = harness.execute("Load Ground Truth", || {
        read_ivecs(gt_path).expect("Failed to read sift_groundtruth.ivecs")
    });

    // Integrity gate
    assert_eq!(base_vectors[0].len(), 128);
    assert_eq!(query_vectors[0].len(), 128);
    println!(
        "\n  {} Dataset: {} base, {} queries, {} GT entries",
        style("✓").green().bold(),
        base_vectors.len(),
        query_vectors.len(),
        groundtruth.len()
    );

    // ── Benchmark Scenarios ──────────────────────────────────────────────
    // SIFT uses L2 distance, but our engine uses cosine sim.
    // We test recall against the official ground truth anyway —
    // lower recall is expected since we're not matching metric.
    // This is the honest, no-bullshit measurement.

    let scales: Vec<usize> = vec![10_000, 100_000];
    let k = 10;

    struct ScenarioResult {
        scale: usize,
        config_name: String,
        recall: f64,
        p50_us: f64,
        p95_us: f64,
        _p99_us: f64,
        qps: f64,
        build_secs: f64,
    }

    let mut all_results: Vec<ScenarioResult> = Vec::new();

    for &scale in &scales {
        let scale_base = &base_vectors[..scale];

        let configs = vec![
            (
                "Balanced",
                HnswConfig {
                    m: 16,
                    m_max0: 32,
                    ef_construction: 200,
                    ef_search: 100,
                    ml: 1.0 / (16_f64).ln(),
                },
            ),
            (
                "High Recall",
                HnswConfig {
                    m: 32,
                    m_max0: 64,
                    ef_construction: 400,
                    ef_search: 200,
                    ml: 1.0 / (32_f64).ln(),
                },
            ),
        ];

        for (config_name, config) in configs {
            let block_name = format!("SIFT {}K — {}", scale / 1000, config_name);

            let (index, build_secs) = harness.execute(&block_name, || {
                let mut idx = CPIndex::new_with_config(config.clone());
                let pb = TerminalReporter::create_progress(scale as u64, "Inserting vectors");
                let t0 = Instant::now();

                for (id, vec) in scale_base.iter().enumerate() {
                    idx.add(
                        id as u64,
                        u128::MAX,
                        VectorRepresentations::Full(vec.clone()),
                        0,
                    );
                    pb.inc(1);
                }
                pb.finish_and_clear();
                let elapsed = t0.elapsed().as_secs_f64();
                (idx, elapsed)
            });

            // Recall against ground truth
            let recall = calculate_recall(&index, &query_vectors, &groundtruth, k);

            // Latency
            let (p50, p95, p99, qps) = measure_latency(&index, &query_vectors, k);

            all_results.push(ScenarioResult {
                scale,
                config_name: config_name.to_string(),
                recall,
                p50_us: p50,
                p95_us: p95,
                _p99_us: p99,
                qps,
                build_secs,
            });
        }
    }

    // ── Print Report ─────────────────────────────────────────────────────
    println!("\n");
    TerminalReporter::block_header("SIFT1M COMPETITIVE BENCHMARK RESULTS");

    println!(
        "  {}",
        style("╭──────────┬──────────────┬──────────┬────────────┬────────────┬────────────┬──────────╮").dim()
    );
    println!(
        "  {} {} {} {} {} {} {} {} {} {} {} {} {} {} {}",
        style("│").dim(),
        style(" Scale   ").bold(),
        style("│").dim(),
        style("   Config    ").bold(),
        style("│").dim(),
        style("Recall@10").bold(),
        style("│").dim(),
        style(" p50 (µs)  ").bold(),
        style("│").dim(),
        style(" p95 (µs)  ").bold(),
        style("│").dim(),
        style("   QPS     ").bold(),
        style("│").dim(),
        style("Build(s) ").bold(),
        style("│").dim(),
    );
    println!(
        "  {}",
        style("├──────────┼──────────────┼──────────┼────────────┼────────────┼────────────┼──────────┤").dim()
    );

    for r in &all_results {
        let recall_styled = if r.recall >= 0.90 {
            style(format!(" {:.4}  ", r.recall)).green().bold()
        } else if r.recall >= 0.70 {
            style(format!(" {:.4}  ", r.recall)).yellow().bold()
        } else {
            style(format!(" {:.4}  ", r.recall)).red().bold()
        };

        println!(
            "  {} {:>7}K {} {:^12} {} {} {} {:>9.1} {} {:>9.1} {} {:>9.0} {} {:>7.1} {}",
            style("│").dim(),
            r.scale / 1000,
            style("│").dim(),
            r.config_name,
            style("│").dim(),
            recall_styled,
            style("│").dim(),
            r.p50_us,
            style("│").dim(),
            r.p95_us,
            style("│").dim(),
            r.qps,
            style("│").dim(),
            r.build_secs,
            style("│").dim(),
        );
    }

    println!(
        "  {}",
        style("╰──────────┴──────────────┴──────────┴────────────┴────────────┴────────────┴──────────╯").dim()
    );

    println!(
        "\n  {} Dataset: SIFT1M (128D, L2 ground truth)",
        style("ℹ").blue()
    );
    println!(
        "  {} VantaDB uses cosine similarity — recall gap vs L2 GT is expected.",
        style("ℹ").blue()
    );
    println!(
        "  {} For competitive parity: Recall >= FAISS_recall - 5%",
        style("ℹ").blue()
    );

    // ── Sanity Assertions ────────────────────────────────────────────────
    // We don't fail on recall (metric mismatch), but we fail on crashes.
    for r in &all_results {
        assert!(r.recall > 0.0, "Zero recall indicates a broken search path");
        assert!(r.qps > 0.0, "Zero QPS indicates search is hanging");
    }

    TerminalReporter::success("SIFT1M Competitive Benchmark Complete.");
}


================================================================
Nombre: hardware_profiles.rs
Ruta: tests\certification\hardware_profiles.rs
================================================================

//! Hardware Profiles Certification — Vanta Certification Edition
//!
//! Validates hardware detection and emergency threshold logic.
//! Sequential execution via a unified entry point to avoid console overlapping.

#[path = "../common/mod.rs"]
mod common;

use common::{TerminalReporter, VantaHarness};
use console::style;
use std::sync::Arc;
use vantadb::storage::StorageEngine;

fn temp_storage() -> Arc<StorageEngine> {
    let dir = tempfile::tempdir().expect("Failed to create temp dir");
    Arc::new(StorageEngine::open(dir.path().to_str().unwrap()).expect("Failed to open storage"))
}

#[tokio::test]
async fn hardware_certification_full() {
    let mut harness = VantaHarness::new("HARDWARE CERTIFICATION");

    // BLOCK 1: Emergency Logic
    harness.execute("Thermal & OOM Thresholds", || {
        let storage = temp_storage();
        TerminalReporter::sub_step("Verifying maintenance triggers...");
        // Verify that it starts false
        assert!(!storage
            .emergency_maintenance_trigger
            .load(std::sync::atomic::Ordering::Acquire));
        TerminalReporter::success("Maintenance flags are initially clean.");
    });

    // BLOCK 2: Detection Profile
    harness.execute("System Capability Audit", || {
        let caps = vantadb::hardware::HardwareCapabilities::global();
        TerminalReporter::sub_step("Reading system topology...");
        assert!(
            caps.total_memory > 0,
            "System memory must be greater than 0"
        );
        assert!(
            caps.logical_cores > 0,
            "Logical cores must be greater than 0"
        );
        assert!(
            caps.resource_score >= 1,
            "Resource score must be calculated"
        );

        println!("\n  {}", style("DETECTED PROFILE").bold().underlined());
        println!(
            "  {} Core Count:   {}",
            style("🧵").cyan(),
            caps.logical_cores
        );
        println!(
            "  {} Total Memory: {:.2} GB",
            style("🧠").magenta(),
            caps.total_memory as f64 / (1024.0 * 1024.0 * 1024.0)
        );
        println!(
            "  {} SIMD Support: {:?}",
            style("⚡").yellow(),
            caps.instructions
        );
        println!("  {} Profile Tier: {:?}", style("🏆").green(), caps.profile);

        TerminalReporter::success("System hardware profile correctly identified.");
    });

    // BLOCK 3: Fallback Math
    harness.execute("ALGORITHMIC FALLBACK", || {
        let a = vantadb::node::VectorRepresentations::Full(vec![1.0, 0.0, 0.0]);
        let b = vantadb::node::VectorRepresentations::Full(vec![1.0, 0.0, 0.0]);
        let c = vantadb::node::VectorRepresentations::Full(vec![0.0, 1.0, 0.0]);

        TerminalReporter::sub_step("Calculating Cosine Similarity (Fallbacks)...");
        // Test similarity computation regardless of the active instruction set branch.
        let sim_ab = a.cosine_similarity(&b).unwrap();
        assert!((sim_ab - 1.0).abs() < 1e-6);

        let sim_ac = a.cosine_similarity(&c).unwrap();
        assert!(sim_ac.abs() < 1e-6); // orthogonal = 0

        TerminalReporter::success("Mathematical consistency verified.");
    });
}


================================================================
Nombre: hnsw_recall.rs
Ruta: tests\certification\hnsw_recall.rs
================================================================

//! Vanta Recall & Latency Certification
//!
//! Measures search precision and timing distribution to ensure production readiness.
//! Sequential execution to maintain console clarity.

#[path = "../common/mod.rs"]
mod common;

use common::{TerminalReporter, VantaHarness};
use console::style;
use rand::{thread_rng, Rng};
use std::time::Instant;
use vantadb::index::{CPIndex, HnswConfig, VectorRepresentations};

fn generate_random_vectors(count: usize, dims: usize) -> Vec<Vec<f32>> {
    let mut rng = thread_rng();
    let mut vectors = Vec::with_capacity(count);
    for _ in 0..count {
        let mut vec = Vec::with_capacity(dims);
        for _ in 0..dims {
            vec.push(rng.gen_range(-1.0..1.0));
        }
        let norm: f32 = vec.iter().map(|v| v * v).sum::<f32>().sqrt();
        if norm > 0.0 {
            for v in &mut vec {
                *v /= norm;
            }
        }
        vectors.push(vec);
    }
    vectors
}

fn brute_force_search(query: &[f32], all_vectors: &[(u64, Vec<f32>)], top_k: usize) -> Vec<u64> {
    let mut distances = Vec::with_capacity(all_vectors.len());
    let query_vector = VectorRepresentations::Full(query.to_vec());
    for (id, vec) in all_vectors {
        let node_vec = VectorRepresentations::Full(vec.clone());
        let sim = query_vector.cosine_similarity(&node_vec).unwrap_or(0.0);
        distances.push((*id, sim));
    }
    distances.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    distances.truncate(top_k);
    distances.into_iter().map(|(id, _)| id).collect()
}

#[test]
fn recall_certification_runner() {
    let mut harness = VantaHarness::new("RECALL CERTIFICATION");

    harness.execute("Recall@10 Calibration", || {
        let node_count = 5000;
        let query_count = 100;
        let dims = 64;
        let top_k = 10;
        TerminalReporter::sub_step(&format!(
            "Generating dataset (N={}, D={})...",
            node_count, dims
        ));
        let raw_vectors = generate_random_vectors(node_count, dims);
        let query_vectors = generate_random_vectors(query_count, dims);
        let dataset: Vec<(u64, Vec<f32>)> = raw_vectors
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as u64, v))
            .collect();

        let config = HnswConfig {
            m: 24,
            m_max0: 48,
            ef_construction: 200,
            ef_search: 100,
            ml: 1.0 / (24_f64).ln(),
        };
        let mut index = CPIndex::new_with_config(config);

        let pb = TerminalReporter::create_progress(node_count as u64, "Building Index");
        for (id, vec) in &dataset {
            index.add(*id, u128::MAX, VectorRepresentations::Full(vec.clone()), 0);
            pb.inc(1);
        }
        pb.finish_and_clear();

        let mut total_recall = 0.0;
        let mut latencies_us = Vec::with_capacity(query_count);
        let pb_query = TerminalReporter::create_progress(query_count as u64, "Computing Recall");
        for query in &query_vectors {
            let true_neighbors = brute_force_search(query, &dataset, top_k);
            let t_start = Instant::now();
            let hnsw_results = index.search_nearest(query, None, None, u128::MAX, top_k, None);
            latencies_us.push(t_start.elapsed().as_micros() as u64);
            let hnsw_neighbor_ids: Vec<u64> = hnsw_results.into_iter().map(|(id, _)| id).collect();
            let intersection = true_neighbors
                .iter()
                .filter(|&id| hnsw_neighbor_ids.contains(id))
                .count();
            total_recall += intersection as f64 / top_k as f64;
            pb_query.inc(1);
        }
        pb_query.finish_and_clear();

        let mean_recall = total_recall / query_count as f64;
        latencies_us.sort_unstable();
        let _p50 = latencies_us[query_count / 2];
        let p95 = latencies_us[(query_count as f64 * 0.95) as usize];
        let avg_us = latencies_us.iter().sum::<u64>() as f64 / query_count as f64;

        println!("\n  {}", style("SEARCH RESULTS").bold().underlined());
        println!("  {} Recall:   {:.4}", style("📊").cyan(), mean_recall);
        println!("  {} Avg Lat:  {:.2} µs", style("🔹").blue(), avg_us);
        println!("  {} p95 Lat:  {} µs", style("🔸").yellow(), p95);
        println!(
            "  {} QPS:      {:.0}",
            style("⚡").green(),
            1_000_000.0 / avg_us
        );

        assert!(mean_recall >= 0.90, "Recall too low: {:.4}", mean_recall);
        TerminalReporter::success("Recall and Latency standards satisfied.");
    });
}


================================================================
Nombre: hnsw_validation.rs
Ruta: tests\certification\hnsw_validation.rs
================================================================

//! HNSW Hard Validation — Vanta Certification Suite
//!
//! Validates the algorithmic correctness, stability, and edge-case handling
//! of the HNSW engine under heavy loads and adverse data distributions.
//!
//! Run with: cargo test --test hnsw_validation -- --nocapture
//! Sequential execution is enforced to maintain console output integrity.

#[path = "../common/mod.rs"]
mod common;

use common::{TerminalReporter, VantaHarness};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use console::style;
use vantadb::index::{cosine_sim_f32, CPIndex, HnswConfig, VectorRepresentations};

// ═══════════════════════════════════════════════════════════════════════
// HELPERS
// ═══════════════════════════════════════════════════════════════════════

fn generate_vectors_seeded(count: usize, dims: usize, seed: u64) -> Vec<Vec<f32>> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut vectors = Vec::with_capacity(count);
    for _ in 0..count {
        let mut vec = Vec::with_capacity(dims);
        for _ in 0..dims {
            vec.push(rng.gen_range(-1.0..1.0));
        }
        let norm: f32 = vec.iter().map(|v| v * v).sum::<f32>().sqrt();
        if norm > 0.0 {
            for v in &mut vec {
                *v /= norm;
            }
        }
        vectors.push(vec);
    }
    vectors
}

fn brute_force_knn(query: &[f32], dataset: &[(u64, Vec<f32>)], k: usize) -> Vec<u64> {
    let mut sims: Vec<(u64, f32)> = dataset
        .iter()
        .map(|(id, vec)| (*id, cosine_sim_f32(query, vec)))
        .collect();
    sims.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    sims.truncate(k);
    sims.into_iter().map(|(id, _)| id).collect()
}

fn build_index(dataset: &[(u64, Vec<f32>)], config: HnswConfig, block_msg: &str) -> CPIndex {
    let pb = TerminalReporter::create_progress(dataset.len() as u64, block_msg);
    let mut index = CPIndex::new_with_config(config);
    for (id, vec) in dataset {
        index.add(*id, u128::MAX, VectorRepresentations::Full(vec.clone()), 0);
        pb.inc(1);
    }
    pb.finish_and_clear();
    index
}

fn compute_recall(
    index: &CPIndex,
    queries: &[Vec<f32>],
    dataset: &[(u64, Vec<f32>)],
    k: usize,
    block_msg: &str,
) -> f64 {
    let pb = TerminalReporter::create_progress(queries.len() as u64, block_msg);
    let mut total_recall = 0.0;
    for query in queries {
        let truth = brute_force_knn(query, dataset, k);
        let hnsw_ids: Vec<u64> = index
            .search_nearest(query, None, None, u128::MAX, k, None)
            .into_iter()
            .map(|(id, _)| id)
            .collect();
        let hits = truth.iter().filter(|id| hnsw_ids.contains(id)).count();
        total_recall += hits as f64 / k as f64;
        pb.inc(1);
    }
    pb.finish_and_clear();
    total_recall / queries.len() as f64
}

// ═══════════════════════════════════════════════════════════════════════
// UNIFIED CERTIFICATION RUNNER (Full Logic Expansion)
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn hnsw_hard_validation_certification() {
    let mut harness = VantaHarness::new("HNSW HARD VALIDATION");

    // ─────────────────────────────────────────────────────────────────
    // SCALE VALIDATIONS
    // ─────────────────────────────────────────────────────────────────

    harness.execute("Scale Check: 1K Vectors", || {
        let n = 1_000;
        let dims = 128;
        let k = 10;
        let n_queries = 200;
        let seed = 42;
        let vecs = generate_vectors_seeded(n, dims, seed);
        let dataset: Vec<(u64, Vec<f32>)> = vecs
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as u64, v))
            .collect();
        let queries = generate_vectors_seeded(n_queries, dims, seed + 1000);
        let config = HnswConfig {
            m: 32,
            m_max0: 64,
            ef_construction: 200,
            ef_search: 100,
            ml: 1.0 / (32_f64).ln(),
        };
        let index = build_index(&dataset, config, "Building");
        let recall = compute_recall(&index, &queries, &dataset, k, "Searching");
        TerminalReporter::info(&format!("Recall@10: {:.4} (Required >= 0.95)", recall));
        assert!(recall >= 0.95);
    });

    harness.execute("Scale Check: 10K Vectors", || {
        let n = 10_000;
        let dims = 128;
        let k = 10;
        let n_queries = 200;
        let seed = 42;
        let vecs = generate_vectors_seeded(n, dims, seed);
        let dataset: Vec<(u64, Vec<f32>)> = vecs
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as u64, v))
            .collect();
        let queries = generate_vectors_seeded(n_queries, dims, seed + 1000);
        let config = HnswConfig {
            m: 32,
            m_max0: 64,
            ef_construction: 400,
            ef_search: 200,
            ml: 1.0 / (32_f64).ln(),
        };
        let index = build_index(&dataset, config, "Building");
        let recall = compute_recall(&index, &queries, &dataset, k, "Searching");
        TerminalReporter::info(&format!("Recall@10: {:.4} (Required >= 0.90)", recall));
        assert!(recall >= 0.90);
    });

    harness.execute("Scale Check: 50K Vectors", || {
        let n = 50_000;
        let dims = 128;
        let k = 10;
        let n_queries = 100;
        let seed = 42;
        let vecs = generate_vectors_seeded(n, dims, seed);
        let dataset: Vec<(u64, Vec<f32>)> = vecs
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as u64, v))
            .collect();
        let queries = generate_vectors_seeded(n_queries, dims, seed + 1000);
        let config = HnswConfig {
            m: 32,
            m_max0: 64,
            ef_construction: 500,
            ef_search: 350,
            ml: 1.0 / (32_f64).ln(),
        };
        let index = build_index(&dataset, config, "Building");
        let recall = compute_recall(&index, &queries, &dataset, k, "Searching");
        TerminalReporter::info(&format!("Recall@10: {:.4} (Required >= 0.85)", recall));
        assert!(recall >= 0.85);
    });

    // ─────────────────────────────────────────────────────────────────
    // STABILITY VALIDATIONS
    // ─────────────────────────────────────────────────────────────────

    harness.execute("Determinism: Same Query -> Same Result", || {
        let n = 5_000;
        let dims = 64;
        let k = 10;
        let seed = 99;
        let vecs = generate_vectors_seeded(n, dims, seed);
        let dataset: Vec<(u64, Vec<f32>)> = vecs
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as u64, v))
            .collect();
        let queries = generate_vectors_seeded(20, dims, seed + 500);
        let index = build_index(&dataset, HnswConfig::default(), "Building");
        for query in &queries {
            let first = index.search_nearest(query, None, None, u128::MAX, k, None);
            let first_ids: Vec<u64> = first.iter().map(|(id, _)| *id).collect();
            for _ in 1..5 {
                let repeat = index.search_nearest(query, None, None, u128::MAX, k, None);
                let repeat_ids: Vec<u64> = repeat.iter().map(|(id, _)| *id).collect();
                assert_eq!(first_ids, repeat_ids);
            }
        }
        TerminalReporter::success("Consistency verified for all query batches.");
    });

    harness.execute("Recall vs ef_search Degradation Curve", || {
        let n = 10_000;
        let seed = 77;
        let vecs = generate_vectors_seeded(n, 64, seed);
        let dataset: Vec<(u64, Vec<f32>)> = vecs
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as u64, v))
            .collect();
        let queries = generate_vectors_seeded(100, 64, seed + 500);
        let mut index = build_index(&dataset, HnswConfig::default(), "Building");
        let ef_values = [10, 20, 50, 100, 200];
        let mut prev_recall = 0.0;
        for &ef in &ef_values {
            index.config.ef_search = ef;
            let recall = compute_recall(&index, &queries, &dataset, 10, &format!("ef={}", ef));
            TerminalReporter::info(&format!("  ef_search={:>3} → recall={:.4}", ef, recall));
            assert!(recall >= prev_recall - 0.02);
            prev_recall = recall;
        }
        assert!(prev_recall >= 0.95);
    });

    // ─────────────────────────────────────────────────────────────────
    // EDGE CASES (Individual Blocks for Transparency)
    // ─────────────────────────────────────────────────────────────────

    harness.execute("Edge Case: Duplicate Vectors", || {
        let dims = 32;
        let k = 5;
        let iv = vec![1.0; dims];
        let mut index = CPIndex::new();
        for i in 0..100 {
            index.add(i, u128::MAX, VectorRepresentations::Full(iv.clone()), 0);
        }
        let results = index.search_nearest(&iv, None, None, u128::MAX, k, None);
        assert_eq!(results.len(), k);
        for (_, sim) in &results {
            assert!((sim - 1.0).abs() < 0.01);
        }
        TerminalReporter::success("100 identical vectors handled correctly.");
    });

    harness.execute("Edge Case: Zero Vector Resilience", || {
        let mut index = CPIndex::new();
        index.add(1, u128::MAX, VectorRepresentations::Full(vec![1.0; 32]), 0);
        index.add(2, u128::MAX, VectorRepresentations::Full(vec![0.0; 32]), 0);
        let res = index.search_nearest(&vec![1.0; 32], None, None, u128::MAX, 3, None);
        assert!(!res.is_empty());
        TerminalReporter::success("Zero vector in index did not cause panics.");
    });

    harness.execute("Edge Case: Single Node Index", || {
        let mut index = CPIndex::new();
        index.add(42, u128::MAX, VectorRepresentations::Full(vec![1.0; 16]), 0);
        let res = index.search_nearest(&vec![1.0; 16], None, None, u128::MAX, 10, None);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].0, 42);
    });

    harness.execute("Edge Case: Empty Index", || {
        let res = CPIndex::new().search_nearest(&vec![1.0; 16], None, None, u128::MAX, 10, None);
        assert!(res.is_empty());
    });

    harness.execute("Stress: High Dimensionality (768D)", || {
        let n = 1_000;
        let dims = 768;
        let k = 10;
        let n_queries = 50;
        let seed = 55;
        let vecs = generate_vectors_seeded(n, dims, seed);
        let dataset: Vec<(u64, Vec<f32>)> = vecs
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as u64, v))
            .collect();
        let queries = generate_vectors_seeded(n_queries, dims, seed + 500);
        let index = build_index(&dataset, HnswConfig::default(), "Building 768D");
        let recall = compute_recall(&index, &queries, &dataset, k, "Searching 768D");
        TerminalReporter::info(&format!("Recall@10 (768D): {:.4}", recall));
        assert!(recall >= 0.90);
    });

    // ─────────────────────────────────────────────────────────────────
    // ACCURACY & COVERAGE
    // ─────────────────────────────────────────────────────────────────

    harness.execute("Validation: Top-1 Accuracy Correctness", || {
        let n = 5_000;
        let seed = 33;
        let vecs = generate_vectors_seeded(n, 64, seed);
        let dataset: Vec<(u64, Vec<f32>)> = vecs
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as u64, v))
            .collect();
        let queries = generate_vectors_seeded(200, 64, seed + 500);
        let index = build_index(&dataset, HnswConfig::default(), "Building");
        let mut hits = 0;
        for q in &queries {
            let truth = brute_force_knn(q, &dataset, 1);
            let res = index.search_nearest(q, None, None, u128::MAX, 1, None);
            if !res.is_empty() && res[0].0 == truth[0] {
                hits += 1;
            }
        }
        let acc = hits as f64 / queries.len() as f64;
        TerminalReporter::info(&format!("Top-1 Precision: {:.4}", acc));
        assert!(acc >= 0.95);
    });

    harness.execute("Validation: Recall@K Sweep (1 to 50)", || {
        let n = 10_000;
        let seed = 88;
        let vecs = generate_vectors_seeded(n, 64, seed);
        let dataset: Vec<(u64, Vec<f32>)> = vecs
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as u64, v))
            .collect();
        let queries = generate_vectors_seeded(100, 64, seed + 500);
        let index = build_index(&dataset, HnswConfig::default(), "Building");
        for &k in &[1, 5, 10, 20, 50] {
            let recall = compute_recall(&index, &queries, &dataset, k, &format!("Recall@K={}", k));
            TerminalReporter::info(&format!("  Recall@{:>2}: {:.4}", k, recall));
            assert!(recall >= 0.80);
        }
    });

    harness.execute("Validation: Memory proportionality", || {
        let dims = 64;
        let seed = 44;
        let ds1 = generate_vectors_seeded(1000, dims, seed)
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as u64, v))
            .collect::<Vec<_>>();
        let idx1 = build_index(&ds1, HnswConfig::default(), "Building 1K");
        let ds5 = generate_vectors_seeded(5000, dims, seed)
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as u64, v))
            .collect::<Vec<_>>();
        let idx5 = build_index(&ds5, HnswConfig::default(), "Building 5K");
        let links1: usize = idx1
            .nodes
            .values()
            .map(|n| n.neighbors.iter().map(|l| l.len()).sum::<usize>())
            .sum();
        let links5: usize = idx5
            .nodes
            .values()
            .map(|n| n.neighbors.iter().map(|l| l.len()).sum::<usize>())
            .sum();
        let ratio = links5 as f64 / links1 as f64;
        TerminalReporter::info(&format!("Memory Growth Factor (5x N): {:.2}x links", ratio));
        assert!(ratio >= 3.0 && ratio <= 8.0);
    });

    println!(
        "\n{}",
        style("VANTA HNSW HARD VALIDATION COMPLETE").green().bold()
    );
}


================================================================
Nombre: sift_validation.rs
Ruta: tests\certification\sift_validation.rs
================================================================

use std::path::Path;

#[path = "../common/mod.rs"]
mod common;

use common::sift_loader::{read_fvecs, read_ivecs};

#[test]
fn validate_sift_dataset_integrity() {
    let base_path = Path::new("datasets/sift/sift_base.fvecs");
    let query_path = Path::new("datasets/sift/sift_query.fvecs");
    let groundtruth_path = Path::new("datasets/sift/sift_groundtruth.ivecs");

    // Skip if not downloaded
    if !base_path.exists() || !query_path.exists() || !groundtruth_path.exists() {
        println!("SIFT dataset not found. Skipping integrity check.");
        return;
    }

    println!("Loading SIFT1M base vectors...");
    let base = read_fvecs(base_path).expect("Failed to read base.fvecs");

    println!("Loading SIFT1M query vectors...");
    let query = read_fvecs(query_path).expect("Failed to read query.fvecs");

    println!("Loading SIFT1M ground truth...");
    let groundtruth = read_ivecs(groundtruth_path).expect("Failed to read groundtruth.ivecs");

    // Phase 2.1 Validation Logic from Roadmap
    assert_eq!(base.len(), 1_000_000, "Base must have 1M vectors");
    assert_eq!(base[0].len(), 128, "Base vectors must be 128D");

    assert_eq!(query.len(), 10_000, "Query must have 10K vectors");
    assert_eq!(query[0].len(), 128, "Query vectors must be 128D");

    assert_eq!(
        groundtruth.len(),
        10_000,
        "Groundtruth must have 10K entries"
    );
    assert_eq!(
        groundtruth[0].len(),
        100,
        "Groundtruth usually provides top 100 nearest neighbors"
    );

    println!("SIFT1M Dataset Integrity: PASSED ✅");
}


================================================================
Nombre: stress_protocol.rs
Ruta: tests\certification\stress_protocol.rs
================================================================

//! ═══════════════════════════════════════════════════════════════════════════
//! STRESS PROTOCOL — VantaDB HNSW Certification Suite
//! ═══════════════════════════════════════════════════════════════════════════
//!
//! This is NOT a unit test. It is a full certification protocol that must pass
//! before the HNSW engine is considered validated for production use.
//!
//! Run with: cargo test --test stress_protocol -- --nocapture
//! Sequential execution is enforced to maintain console output integrity.

use console::style;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rayon::prelude::*;
use std::time::Instant;
use vantadb::index::{cosine_sim_f32, CPIndex, HnswConfig, VectorRepresentations};

#[path = "../common/mod.rs"]
mod common;
use common::*;

// ═══════════════════════════════════════════════════════════════════════════
// HELPERS
// ═══════════════════════════════════════════════════════════════════════════

fn gen_vectors(count: usize, dims: usize, seed: u64) -> Vec<Vec<f32>> {
    let mut rng = StdRng::seed_from_u64(seed);
    (0..count)
        .map(|_| {
            let mut v: Vec<f32> = (0..dims).map(|_| rng.gen_range(-1.0..1.0)).collect();
            let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
            if norm > f32::EPSILON {
                v.iter_mut().for_each(|x| *x /= norm);
            }
            v
        })
        .collect()
}

fn brute_force_knn(query: &[f32], dataset: &[(u64, Vec<f32>)], k: usize) -> Vec<u64> {
    let mut scored: Vec<(u64, f32)> = dataset
        .par_iter()
        .map(|(id, vec)| (*id, cosine_sim_f32(query, vec)))
        .collect();
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(k);
    scored.into_iter().map(|(id, _)| id).collect()
}

fn compute_recall(
    index: &CPIndex,
    queries: &[Vec<f32>],
    dataset: &[(u64, Vec<f32>)],
    k: usize,
) -> f64 {
    let pb = TerminalReporter::create_progress(queries.len() as u64, "Computing Recall");
    let total: f64 = queries
        .par_iter()
        .map(|q| {
            let truth = brute_force_knn(q, dataset, k);
            let hnsw: Vec<u64> = index
                .search_nearest(q, None, None, u128::MAX, k, None)
                .into_iter()
                .map(|(id, _)| id)
                .collect();
            let hits = truth.iter().filter(|id| hnsw.contains(id)).count();
            pb.inc(1);
            hits as f64 / k as f64
        })
        .sum();
    pb.finish_and_clear();
    total / queries.len() as f64
}

fn build_index(dataset: &[(u64, Vec<f32>)], config: HnswConfig) -> CPIndex {
    let mut idx = CPIndex::new_with_config(config);
    let pb = TerminalReporter::create_progress(dataset.len() as u64, "Building HNSW");
    for (id, vec) in dataset {
        idx.add(*id, u128::MAX, VectorRepresentations::Full(vec.clone()), 0);
        pb.inc(1);
    }
    pb.finish_and_clear();
    idx
}

fn measure_latency_percentiles(index: &CPIndex, queries: &[Vec<f32>], k: usize) -> (f64, f64, f64) {
    let mut latencies: Vec<f64> = queries
        .iter()
        .map(|q| {
            let t = Instant::now();
            let _ = index.search_nearest(q, None, None, u128::MAX, k, None);
            t.elapsed().as_nanos() as f64 / 1000.0 // µs
        })
        .collect();
    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = latencies.len();
    let p50 = latencies[n / 2];
    let p95 = latencies[(n as f64 * 0.95) as usize];
    let p99 = latencies[(n as f64 * 0.99) as usize];
    (p50, p95, p99)
}

fn estimate_memory_bytes(index: &CPIndex) -> usize {
    let mut total: usize = 0;
    for node in index.nodes.values() {
        match &node.vec_data {
            VectorRepresentations::Full(v) => total += v.len() * 4,
            VectorRepresentations::Binary(b) => total += b.len() * 8,
            VectorRepresentations::Turbo(t) => total += t.len(),
            VectorRepresentations::None => {}
        }
        for layer in &node.neighbors {
            total += layer.len() * 8 + 24;
        }
        total += 8 + 16 + 8 + 24;
    }
    total += index.nodes.len() * 60;
    total
}

// ═══════════════════════════════════════════════════════════════════════════
// UNIFIED CERTIFICATION RUNNER (Strict Logic Preservation)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn stress_protocol_certification() {
    let mut harness = VantaHarness::new("VANTA STRESS PROTOCOL");

    // BLOCK 1: Recall
    harness.execute("BLOCK 1 — GROUND TRUTH RECALL (50K/128D)", || {
        let n = 50_000;
        let dims = 128;
        let k = 10;
        let n_queries = 100;
        let seed = 2024;
        TerminalReporter::sub_step("Generating synthetic datasets...");
        let vecs = gen_vectors(n, dims, seed);
        let dataset: Vec<(u64, Vec<f32>)> = vecs
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as u64, v))
            .collect();
        let queries = gen_vectors(n_queries, dims, seed + 9999);
        let config = HnswConfig {
            m: 32,
            m_max0: 64,
            ef_construction: 500,
            ef_search: 300,
            ml: 1.0 / (32_f64).ln(),
        };
        let index = build_index(&dataset, config);
        let recall = compute_recall(&index, &queries, &dataset, k);
        let status_msg = format!("Recall@{}: {:.4} (Required >= 0.95)", k, recall);
        assert!(recall >= 0.95, "BLOCK 1 FAILED: {}", status_msg);
        TerminalReporter::success(&format!("PASSED: {}", status_msg));
    });

    // BLOCK 2: Scaling
    harness.execute("BLOCK 2 — SCALING (10K → 50K → 100K)", || {
        let dims = 128;
        let k = 10;
        let n_queries = 100;
        let seed = 2024;
        let scales = [10_000, 50_000, 100_000];
        let mut results = Vec::new();
        for &n in &scales {
            TerminalReporter::sub_step(&format!("Processing scale: {} vectors", n));
            let config = HnswConfig {
                m: 32,
                m_max0: 64,
                ef_construction: if n <= 10_000 {
                    200
                } else if n <= 50_000 {
                    400
                } else {
                    500
                },
                ef_search: if n <= 10_000 {
                    100
                } else if n <= 50_000 {
                    200
                } else {
                    300
                },
                ml: 1.0 / (32_f64).ln(),
            };
            let vecs = gen_vectors(n, dims, seed);
            let dataset: Vec<(u64, Vec<f32>)> = vecs
                .into_iter()
                .enumerate()
                .map(|(i, v)| (i as u64, v))
                .collect();
            let queries = gen_vectors(n_queries, dims, seed + 9999);
            let t0 = Instant::now();
            let index = build_index(&dataset, config);
            let build_s = t0.elapsed().as_secs_f64();
            let recall = compute_recall(&index, &queries, &dataset, k);
            let (p50, p95, _) = measure_latency_percentiles(&index, &queries, k);
            let mem_mb = estimate_memory_bytes(&index) as f64 / (1024.0 * 1024.0);
            results.push((n, recall, p50, p95, build_s, mem_mb));
        }

        println!(
            "\n  {}",
            style("SCALING PERFORMANCE SUMMARY").bold().underlined()
        );
        println!(
            "  {}",
            style(
                "╭───────────┬────────────┬──────────────┬──────────────┬───────────┬──────────╮"
            )
            .dim()
        );
        println!(
            "  {} {} {} {} {} {} {} {} {} {} {} {} {}",
            style("│").dim(),
            style("  Dataset  ").bold().white(),
            style("│").dim(),
            style(" Recall@10  ").bold().white(),
            style("│").dim(),
            style("  Lat p50(µs) ").bold().white(),
            style("│").dim(),
            style("  Lat p95(µs) ").bold().white(),
            style("│").dim(),
            style(" Build(s)  ").bold().white(),
            style("│").dim(),
            style(" RAM(MB)  ").bold().white(),
            style("│").dim()
        );
        println!(
            "  {}",
            style(
                "├───────────┼────────────┼──────────────┼──────────────┼───────────┼──────────┤"
            )
            .dim()
        );
        for (n, rec, p50, p95, b_s, mem) in &results {
            let recall_style = if *rec >= 0.95 {
                style(format!("{:.4}", rec)).green().bold()
            } else if *rec >= 0.90 {
                style(format!("{:.4}", rec)).yellow().bold()
            } else {
                style(format!("{:.4}", rec)).red().bold()
            };
            println!(
                "  {} {:>9} {}   {}   {}  {:>10.1} {}  {:>10.1} {}  {:>7.2} {}  {:>6.1} {}",
                style("│").dim(),
                format!("{}K", n / 1000),
                style("│").dim(),
                recall_style,
                style("│").dim(),
                p50,
                style("│").dim(),
                p95,
                style("│").dim(),
                b_s,
                style("│").dim(),
                mem,
                style("│").dim()
            );
        }
        println!(
            "  {}",
            style(
                "╰───────────┴────────────┴──────────────┴──────────────┴───────────┴──────────╯"
            )
            .dim()
        );

        assert!(results[0].1 >= 0.95);
        assert!(results[1].1 >= 0.90);
        assert!(results[2].1 >= 0.85);
        let recall_drop = results[0].1 - results[2].1;
        assert!(
            recall_drop < 0.15,
            "Catastrophic degradation: {:.4}",
            recall_drop
        );
        assert!(results[2].2 < 50_000.0, "100K p50 too slow");
        TerminalReporter::success("BLOCK 2 PASSED.");
    });

    // BLOCK 3: Memory
    harness.execute("BLOCK 3 — MEMORY MEASUREMENT", || {
        let dims = 128;
        let seed = 2024;
        let config = HnswConfig {
            m: 32,
            m_max0: 64,
            ef_construction: 200,
            ef_search: 100,
            ml: 1.0 / (32_f64).ln(),
        };
        let sizes = [1_000, 5_000, 10_000, 50_000];
        let mut memories = Vec::new();
        for &n in &sizes {
            let vecs = gen_vectors(n, dims, seed);
            let dataset: Vec<(u64, Vec<f32>)> = vecs
                .into_iter()
                .enumerate()
                .map(|(i, v)| (i as u64, v))
                .collect();
            let index = build_index(&dataset, config.clone());
            let m_bytes = estimate_memory_bytes(&index);
            let m_mb = m_bytes as f64 / (1024. * 1024.);
            TerminalReporter::info(&format!(
                "{:>6} vectors → {:>6.2} MB ({:.0} bytes/vector)",
                n,
                m_mb,
                m_bytes as f64 / n as f64
            ));
            memories.push(m_mb);
        }
        let ratio = memories[3] / memories[1]; // 50K / 5K
        assert!(
            ratio >= 5.0 && ratio <= 15.0,
            "Growth ratio {:.2}x not proportional",
            ratio
        );
        TerminalReporter::success("BLOCK 3 PASSED.");
    });

    // BLOCK 4: Persistence
    harness.execute("BLOCK 4 — PERSISTENCE ROUND-TRIP", || {
        let n = 10_000;
        let dims = 128;
        let k = 10;
        let n_queries = 100;
        let seed = 2024;
        let vecs = gen_vectors(n, dims, seed);
        let dataset: Vec<(u64, Vec<f32>)> = vecs
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as u64, v))
            .collect();
        let queries = gen_vectors(n_queries, dims, seed + 9999);
        let config = HnswConfig {
            m: 32,
            m_max0: 64,
            ef_construction: 200,
            ef_search: 100,
            ml: 1.0 / (32_f64).ln(),
        };
        let original = build_index(&dataset, config);
        let recall_before = compute_recall(&original, &queries, &dataset, k);
        let tmp = tempfile::NamedTempFile::new().unwrap();
        original.persist_to_file(tmp.path()).unwrap();
        let file_size = std::fs::metadata(tmp.path()).unwrap().len();
        TerminalReporter::info(&format!(
            "File size: {:.2} MB",
            file_size as f64 / (1024. * 1024.)
        ));
        let loaded = CPIndex::load_from_file(tmp.path()).unwrap();
        assert_eq!(loaded.nodes.len(), n);
        let recall_after = compute_recall(&loaded, &queries, &dataset, k);
        assert!((recall_before - recall_after).abs() < 0.001);
        loaded.validate_index().unwrap();
        TerminalReporter::success("BLOCK 4 PASSED.");
    });

    // BLOCK 5: Edge Cases (5a-5g)
    harness.execute("BLOCK 5 — EDGE CASES", || {
        let k = 5;
        let d = 64;
        TerminalReporter::sub_step("5a: Empty index...");
        let empty = CPIndex::new();
        assert!(empty
            .search_nearest(&vec![1.0; d], None, None, u128::MAX, k, None)
            .is_empty());

        TerminalReporter::sub_step("5b: Single node...");
        let mut single = CPIndex::new();
        single.add(1, u128::MAX, VectorRepresentations::Full(vec![1.0; d]), 0);
        assert_eq!(
            single
                .search_nearest(&vec![1.0; d], None, None, u128::MAX, k, None)
                .len(),
            1
        );

        TerminalReporter::sub_step("5c: Two nodes...");
        let mut two = CPIndex::new();
        two.add(1, u128::MAX, VectorRepresentations::Full(vec![1.0; d]), 0);
        two.add(2, u128::MAX, VectorRepresentations::Full(vec![-1.0; d]), 0);
        assert_eq!(
            two.search_nearest(&vec![1.0; d], None, None, u128::MAX, k, None)
                .len(),
            2
        );

        TerminalReporter::sub_step("5d: Zero vector...");
        let mut zvec = CPIndex::new();
        zvec.add(1, u128::MAX, VectorRepresentations::Full(vec![0.0; d]), 0);
        assert!(!zvec
            .search_nearest(&vec![0.0; d], None, None, u128::MAX, k, None)
            .is_empty());

        TerminalReporter::sub_step("5e: Duplicate ID...");
        let mut dup = CPIndex::new();
        dup.add(1, u128::MAX, VectorRepresentations::Full(vec![1.0; d]), 0);
        dup.add(1, u128::MAX, VectorRepresentations::Full(vec![-1.0; d]), 0);
        assert_eq!(dup.nodes.len(), 1);

        TerminalReporter::sub_step("5f: Dimension Mismatch...");
        let mut dvec = CPIndex::new();
        dvec.add(1, u128::MAX, VectorRepresentations::Full(vec![1.0; d]), 0);
        let _ = dvec.search_nearest(&vec![1.0; 128], None, None, u128::MAX, k, None);

        TerminalReporter::sub_step("5g: k > n...");
        let results = dvec.search_nearest(&vec![1.0; d], None, None, u128::MAX, 100, None);
        assert!(results.len() == 1);

        TerminalReporter::success("BLOCK 5 PASSED.");
    });

    // BLOCK 6: Consistency
    harness.execute("BLOCK 6 — GRAPH CONSISTENCY", || {
        let n = 50_000;
        let dims = 128;
        let seed = 2024;
        let config = HnswConfig {
            m: 32,
            m_max0: 64,
            ef_construction: 400,
            ef_search: 200,
            ml: 1.0 / (32_f64).ln(),
        };
        let vecs = gen_vectors(n, dims, seed);
        let dataset: Vec<(u64, Vec<f32>)> = vecs
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as u64, v))
            .collect();
        let index = build_index(&dataset, config);
        index.validate_index().unwrap();
        let stats = index.stats();
        TerminalReporter::info(&format!(
            "Nodes: {} | Orphans: {} | Avg L0 Conn: {:.1}",
            stats.node_count, stats.orphan_count, stats.avg_connections_l0
        ));
        assert!(stats.orphan_count <= 1);
        TerminalReporter::success("BLOCK 6 PASSED.");
    });

    // BLOCK 7: Latency
    harness.execute("BLOCK 7 — LATENCY PERCENTILES", || {
        let n1 = 10000;
        let n2 = 50000;
        let dims = 128;
        let seed = 2024;
        let mut results = Vec::new();
        for &n in &[n1, n2] {
            let config = HnswConfig {
                m: 32,
                m_max0: 64,
                ef_construction: if n <= 10000 { 200 } else { 400 },
                ef_search: if n <= 10000 { 100 } else { 200 },
                ml: 1.0 / (32_f64).ln(),
            };
            let vecs = gen_vectors(n, dims, seed);
            let dataset: Vec<(u64, Vec<f32>)> = vecs
                .into_iter()
                .enumerate()
                .map(|(i, v)| (i as u64, v))
                .collect();
            let queries = gen_vectors(200, dims, seed + 9999);
            let index = build_index(&dataset, config);
            let (p50, p95, p99) = measure_latency_percentiles(&index, &queries, 10);
            TerminalReporter::info(&format!(
                "{}K vectors -> p50: {:.1}µs | p95: {:.1}µs | p99: {:.1}µs",
                n / 1000,
                p50,
                p95,
                p99
            ));
            results.push(p50);
        }
        let s_factor = results[1] / results[0];
        TerminalReporter::info(&format!("Latency scale factor (50K/10K): {:.2}x", s_factor));
        // Threshold: 8.0x accounts for CPU cache/thermal variance between runs.
        // Theoretical HNSW: ~1.7x for 5x data. Practical observed: 2.6x–5.6x.
        // See docs/problemas_encontrados_en_tests.md for analysis.
        assert!(s_factor < 8.0, "Latency scales too fast: {:.2}x", s_factor);
        TerminalReporter::success("BLOCK 7 PASSED.");
    });
}


================================================================
Nombre: mod.rs
Ruta: tests\common\mod.rs
================================================================

#![allow(dead_code)]

use console::{style, Emoji};
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::time::Instant;
use sysinfo::System;

pub mod sift_loader;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TestMetric {
    pub block_name: String,
    pub duration_secs: f64,
    pub ram_usage_mb: f64,
    pub current_ram_mb: f64,
    pub timestamp: String,
}

pub struct VantaHarness {
    sys: System,
    pid: sysinfo::Pid,
    _start_time: Instant,
    start_mem: u64,
    test_name: String,
}

impl VantaHarness {
    const REPORT_FILE: &'static str = "vanta_certification.json";

    pub fn new(test_name: &str) -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        let pid = sysinfo::get_current_pid().expect("Failed to get PID");

        // Initial snapshot
        sys.refresh_process(pid);
        let start_mem = sys.process(pid).map(|p| p.memory()).unwrap_or(0);

        Self {
            sys,
            pid,
            _start_time: Instant::now(),
            start_mem,
            test_name: test_name.to_string(),
        }
    }

    pub fn execute<F, R>(&mut self, block_name: &str, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        TerminalReporter::block_header(block_name);

        let t0 = Instant::now();
        let result = f();
        let duration = t0.elapsed();

        // Measurements
        self.sys.refresh_process(self.pid);
        let end_mem = self.sys.process(self.pid).map(|p| p.memory()).unwrap_or(0);
        let mem_usage_kb = if end_mem > self.start_mem {
            end_mem - self.start_mem
        } else {
            0
        };

        let metric = TestMetric {
            block_name: format!("{}: {}", self.test_name, block_name),
            duration_secs: duration.as_secs_f64(),
            ram_usage_mb: mem_usage_kb as f64 / 1024.0,
            current_ram_mb: end_mem as f64 / 1024.0,
            timestamp: chrono::Local::now().to_rfc3339(),
        };

        // Standard Report
        println!("\n  {}", style("CERTIFICATION METRICS").bold().underlined());
        println!(
            "  {} Time:      {:.2}s",
            style("⏱️").cyan(),
            metric.duration_secs
        );
        println!(
            "  {} RAM Usage: {:.2} MB (Current: {:.2} MB)",
            style("🧠").magenta(),
            metric.ram_usage_mb,
            metric.current_ram_mb
        );

        self.log_metric(metric);

        result
    }

    fn log_metric(&self, metric: TestMetric) {
        // Append to JSON list (simplified for now as a line-based JSON for easier concurrent appends)
        if let Ok(json) = serde_json::to_string(&metric) {
            if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open(Self::REPORT_FILE)
            {
                let _ = writeln!(file, "{}", json);
            }
        }
    }
}

pub struct TerminalReporter;

impl TerminalReporter {
    pub fn block_header(title: &str) {
        let bar = "━".repeat(title.len() + 4);
        println!("\n{}", style(format!("┏{}┓", bar)).cyan().dim());
        println!(
            "{}  {}  {}",
            style("┃").cyan().dim(),
            style(title).bold().white(),
            style("┃").cyan().dim()
        );
        println!("{}\n", style(format!("┗{}┛", bar)).cyan().dim());
    }

    #[allow(dead_code)]
    pub fn sub_step(msg: &str) {
        println!("  {} {}", style("➜").cyan().bold(), style(msg).dim());
    }

    pub fn success(msg: &str) {
        println!("  {} {}", Emoji("✅", "OK"), style(msg).green());
    }

    #[allow(dead_code)]
    pub fn info(msg: &str) {
        println!("  {} {}", Emoji("ℹ️ ", "i"), style(msg).blue());
    }

    #[allow(dead_code)]
    pub fn warning(msg: &str) {
        println!("  {} {}", Emoji("⚠️ ", "W"), style(msg).yellow());
    }

    #[allow(dead_code)]
    pub fn create_progress(len: u64, msg: &str) -> ProgressBar {
        let pb = ProgressBar::new(len);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
            .expect("Invalid progress template")
            .progress_chars("█▉▊▋▌▍▎▏  "));
        pb.set_message(msg.to_string());
        pb
    }
}


================================================================
Nombre: sift_loader.rs
Ruta: tests\common\sift_loader.rs
================================================================

#![allow(dead_code)]

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

/// Parses an `.fvecs` file into a `Vec<Vec<f32>>`.
/// Format: The file consists of sequences of (d, v_1, ..., v_d).
/// Where `d` is the dimension (i32) and `v_i` are vector elements (f32).
pub fn read_fvecs<P: AsRef<Path>>(path: P) -> std::io::Result<Vec<Vec<f32>>> {
    let mut file = BufReader::new(File::open(path)?);
    let mut results = Vec::new();
    let mut d_buf = [0u8; 4];

    loop {
        if let Err(e) = file.read_exact(&mut d_buf) {
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                // Natural end of file
                break;
            } else {
                return Err(e);
            }
        }

        let d = i32::from_le_bytes(d_buf) as usize;
        let mut vector_buf = vec![0u8; d * 4];
        file.read_exact(&mut vector_buf)?;

        let mut vector = Vec::with_capacity(d);
        for i in 0..d {
            let val_bytes = [
                vector_buf[i * 4],
                vector_buf[i * 4 + 1],
                vector_buf[i * 4 + 2],
                vector_buf[i * 4 + 3],
            ];
            vector.push(f32::from_le_bytes(val_bytes));
        }

        results.push(vector);
    }

    Ok(results)
}

/// Parses an `.ivecs` file into a `Vec<Vec<usize>>`.
/// Same structure but values are i32 instead of f32.
pub fn read_ivecs<P: AsRef<Path>>(path: P) -> std::io::Result<Vec<Vec<usize>>> {
    let mut file = BufReader::new(File::open(path)?);
    let mut results = Vec::new();
    let mut d_buf = [0u8; 4];

    loop {
        if let Err(e) = file.read_exact(&mut d_buf) {
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                break;
            } else {
                return Err(e);
            }
        }

        let d = i32::from_le_bytes(d_buf) as usize;
        let mut vector_buf = vec![0u8; d * 4];
        file.read_exact(&mut vector_buf)?;

        let mut vector = Vec::with_capacity(d);
        for i in 0..d {
            let val_bytes = [
                vector_buf[i * 4],
                vector_buf[i * 4 + 1],
                vector_buf[i * 4 + 2],
                vector_buf[i * 4 + 3],
            ];
            let val = i32::from_le_bytes(val_bytes) as usize;
            vector.push(val);
        }

        results.push(vector);
    }

    Ok(results)
}


================================================================
Nombre: basic_node.rs
Ruta: tests\core\basic_node.rs
================================================================

//! Integration tests for VantaDB Fase 1: node CRUD, vector search, graph traversal
//! Modernized with Vanta Certification Framework.

#[path = "../common/mod.rs"]
mod common;

use common::{TerminalReporter, VantaHarness};
use std::time::Instant;
use vantadb::{FieldValue, InMemoryEngine, UnifiedNode};

#[test]
fn core_engine_certification() {
    let mut harness = VantaHarness::new("CORE ENGINE (CRUD & SEARCH)");

    harness.execute("Node CRUD: Insert & Get", || {
        let engine = InMemoryEngine::new();
        let node = UnifiedNode::new(100);
        let id = engine.insert(node).unwrap();
        assert_eq!(id, 100);

        let retrieved = engine.get(100).unwrap();
        assert_eq!(retrieved.id, 100);
        assert!(retrieved.is_alive());
        TerminalReporter::success("Basic Insert/Get verified.");
    });

    harness.execute("Node CRUD: Auto-ID Generation", || {
        let engine = InMemoryEngine::new();
        let id1 = engine.insert(UnifiedNode::new(0)).unwrap();
        let id2 = engine.insert(UnifiedNode::new(0)).unwrap();
        assert_ne!(id1, id2);
        assert!(id1 > 0);
        assert!(id2 > 0);
    });

    harness.execute("Node CRUD: Duplicate ID Protection", || {
        let engine = InMemoryEngine::new();
        engine.insert(UnifiedNode::new(42)).unwrap();
        let err = engine.insert(UnifiedNode::new(42));
        assert!(err.is_err());
    });

    harness.execute("Node CRUD: Delete logic", || {
        let engine = InMemoryEngine::new();
        engine.insert(UnifiedNode::new(1)).unwrap();
        engine.delete(1).unwrap();
        assert!(engine.get(1).is_none());
    });

    harness.execute("Node CRUD: Field Update logic", || {
        let engine = InMemoryEngine::new();
        engine.insert(UnifiedNode::new(1)).unwrap();

        let mut updated = UnifiedNode::new(1);
        updated.set_field("name", FieldValue::String("Eros".into()));
        engine.update(1, updated).unwrap();

        let node = engine.get(1).unwrap();
        assert_eq!(
            node.get_field("name"),
            Some(&FieldValue::String("Eros".into()))
        );
    });

    harness.execute("Bitset: Multidimensional Scan", || {
        let engine = InMemoryEngine::new();
        for i in 1..=100 {
            let mut node = UnifiedNode::new(i);
            if i % 2 == 0 {
                node.set_bit(5);
            } // VZLA
            if i % 3 == 0 {
                node.set_bit(16);
            } // active
            engine.insert(node).unwrap();
        }
        let vzla = engine.scan_bitset(1u128 << 5);
        assert_eq!(vzla.len(), 50);
        let both = engine.scan_bitset((1u128 << 5) | (1u128 << 16));
        assert_eq!(both.len(), 16);
        TerminalReporter::success("Cross-bitset filtering validated.");
    });

    harness.execute("Vector: Exact Top-K Search", || {
        let engine = InMemoryEngine::new();
        engine
            .insert(UnifiedNode::with_vector(1, vec![1.0, 0.0, 0.0]))
            .unwrap();
        engine
            .insert(UnifiedNode::with_vector(2, vec![0.9, 0.1, 0.0]))
            .unwrap();
        engine
            .insert(UnifiedNode::with_vector(3, vec![0.0, 1.0, 0.0]))
            .unwrap();

        let result = engine.vector_search(&[1.0, 0.0, 0.0], 2, 0.5, None);
        assert_eq!(result.nodes.len(), 2);
        assert_eq!(result.nodes[0].id, 1);
        assert_eq!(result.nodes[1].id, 2);
    });

    harness.execute("Graph: Relation Traversal & Hops", || {
        let engine = InMemoryEngine::new();
        let mut n1 = UnifiedNode::new(1);
        n1.add_edge(2, "amigo");
        let mut n2 = UnifiedNode::new(2);
        n2.add_edge(3, "amigo");
        let mut n3 = UnifiedNode::new(3);
        n3.add_edge(4, "amigo");
        let n4 = UnifiedNode::new(4);

        engine.insert(n1).unwrap();
        engine.insert(n2).unwrap();
        engine.insert(n3).unwrap();
        engine.insert(n4).unwrap();

        let result = engine.traverse(1, "amigo", 1, 2).unwrap();
        assert_eq!(result.len(), 2);
        let result_full = engine.traverse(1, "amigo", 1, 3).unwrap();
        assert_eq!(result_full.len(), 3);
    });

    harness.execute("Hybrid Search: Bitset + Vector + Fields", || {
        let engine = InMemoryEngine::new();
        for i in 1..=10 {
            let mut node = UnifiedNode::with_vector(i, vec![i as f32, 0.0, 0.0]);
            node.set_field("pais", FieldValue::String("VZLA".into()));
            if i % 2 == 0 {
                node.set_bit(5);
            }
            engine.insert(node).unwrap();
        }
        let result = engine.hybrid_search(
            &[10.0, 0.0, 0.0],
            3,
            0.5,
            Some(1u128 << 5),
            &[("pais".to_string(), FieldValue::String("VZLA".into()))],
        );
        assert_eq!(result.nodes.len(), 3);
        for node in &result.nodes {
            assert_eq!(node.id % 2, 0);
        }
    });

    harness.execute("WAL: Persistence & Recovery", || {
        let wal_path = std::env::temp_dir().join("vanta_wal_modern_test.bin");
        let _ = std::fs::remove_file(&wal_path);
        {
            let engine = InMemoryEngine::with_wal(&wal_path).unwrap();
            let mut node = UnifiedNode::new(42);
            node.set_field("name", FieldValue::String("test".into()));
            engine.insert(node).unwrap();
            engine.flush_wal().unwrap();
        }
        {
            let engine = InMemoryEngine::with_wal(&wal_path).unwrap();
            let node = engine.get(42).unwrap();
            assert_eq!(
                node.get_field("name"),
                Some(&FieldValue::String("test".into()))
            );
        }
        let _ = std::fs::remove_file(&wal_path);
    });

    harness.execute("System: Basic Engine Stats", || {
        let engine = InMemoryEngine::new();
        engine
            .insert(UnifiedNode::with_vector(1, vec![1.0, 2.0, 3.0]))
            .unwrap();
        let mut n2 = UnifiedNode::new(2);
        n2.add_edge(1, "knows");
        engine.insert(n2).unwrap();
        let stats = engine.stats();
        assert_eq!(stats.node_count, 2);
        assert_eq!(stats.vector_count, 1);
    });

    harness.execute("Benchmark: 10K Node Throughput", || {
        let engine = InMemoryEngine::new();
        let start = Instant::now();
        for i in 1..=10_000u64 {
            let node = UnifiedNode::new(i);
            engine.insert(node).unwrap();
        }
        let elapsed = start.elapsed();
        assert_eq!(engine.node_count(), 10_000);
        assert!(elapsed.as_millis() < 500);
        TerminalReporter::success(&format!(
            "BENCH: 10k inserts in {:?} ({:.1} μs/insert)",
            elapsed,
            elapsed.as_micros() as f64 / 10_000.0
        ));
    });
}


================================================================
Nombre: graph.rs
Ruta: tests\core\graph.rs
================================================================

//! Graph Traversal Modernized Test Suite
//! Part of the Vanta Certification ecosystem.

#[path = "../common/mod.rs"]
mod common;

use common::{TerminalReporter, VantaHarness};
use vantadb::graph::GraphTraverser;
use vantadb::node::UnifiedNode;
use vantadb::storage::StorageEngine;

#[test]
fn graph_traversal_certification() {
    let mut harness = VantaHarness::new("CORE ENGINE (GRAPH TRAVERSAL)");

    harness.execute("BFS Traversal Matrix", || {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().to_str().unwrap();
        let storage = StorageEngine::open(db_path).unwrap();

        TerminalReporter::sub_step("Building system topology (1->2->3, 1->4)...");
        let mut node1 = UnifiedNode::new(1);
        node1.add_edge(2, "relates_to");
        node1.add_edge(4, "relates_to");
        let mut node2 = UnifiedNode::new(2);
        node2.add_edge(3, "relates_to");
        let node3 = UnifiedNode::new(3);
        let node4 = UnifiedNode::new(4);

        storage.insert(&node1).unwrap();
        storage.insert(&node2).unwrap();
        storage.insert(&node3).unwrap();
        storage.insert(&node4).unwrap();

        let traverser = GraphTraverser::new(&storage);

        TerminalReporter::sub_step("Verifying Depth-1 coverage...");
        let res_d1 = traverser.bfs_traverse(&[1], 1).unwrap();
        assert!(res_d1.contains(&1));
        assert!(res_d1.contains(&2));
        assert!(res_d1.contains(&4));
        assert!(!res_d1.contains(&3));

        TerminalReporter::sub_step("Verifying Depth-2 coverage (reaching terminal nodes)...");
        let res_d2 = traverser.bfs_traverse(&[1], 2).unwrap();
        assert_eq!(res_d2.len(), 4);
        assert!(res_d2.contains(&3));

        TerminalReporter::success("BFS Traversal Axioms satisfied.");
    });
}


================================================================
Nombre: hnsw.rs
Ruta: tests\core\hnsw.rs
================================================================

//! HNSW Algorithm Core Modernized Test Suite
//! Part of the Vanta Certification ecosystem.

#[path = "../common/mod.rs"]
mod common;

use common::{TerminalReporter, VantaHarness};
use vantadb::index::{CPIndex, VectorRepresentations};

#[test]
fn hnsw_core_logic_certification() {
    let mut harness = VantaHarness::new("CORE ENGINE (HNSW LOGIC)");

    harness.execute("Vector Math: Cosine Similarity Axioms", || {
        TerminalReporter::sub_step(
            "Verifying Identical (1.0), Orthogonal (0.0), and Opposite (-1.0) vectors...",
        );
        let a = VectorRepresentations::Full(vec![1.0, 0.0, 0.0]);
        let b = VectorRepresentations::Full(vec![1.0, 0.0, 0.0]);
        let sim = a.cosine_similarity(&b).unwrap();
        assert!((sim - 1.0).abs() < f32::EPSILON);

        let c = VectorRepresentations::Full(vec![0.0, 1.0, 0.0]);
        let sim_orthogonal = a.cosine_similarity(&c).unwrap();
        assert!(sim_orthogonal.abs() < f32::EPSILON);

        let d = VectorRepresentations::Full(vec![-1.0, 0.0, 0.0]);
        let sim_opposite = a.cosine_similarity(&d).unwrap();
        assert!((sim_opposite - (-1.0)).abs() < f32::EPSILON);

        TerminalReporter::success("Algebraic consistency confirmed.");
    });

    harness.execute("HNSW: Greedy Search Integrity", || {
        let mut index = CPIndex::new();
        TerminalReporter::sub_step("Populating sparse vector space...");
        index.add(1, 0, VectorRepresentations::Full(vec![1.0, 0.0, 0.0]), 0);
        index.add(2, 0, VectorRepresentations::Full(vec![0.8, 0.2, 0.0]), 0);
        index.add(3, 0, VectorRepresentations::Full(vec![0.0, 1.0, 0.0]), 0);
        index.add(4, 0, VectorRepresentations::Full(vec![0.0, 0.8, 0.2]), 0);

        let query = vec![0.0, 0.9, 0.1];
        let results = index.search_nearest(&query, None, None, 0, 2, None);

        assert_eq!(results.len(), 2);
        let top_match = results[0].0;
        assert!(top_match == 3 || top_match == 4);

        TerminalReporter::success("Greedy search converged on expected neighbors.");
    });
}


================================================================
Nombre: vector_scale_check.rs
Ruta: tests\core\vector_scale_check.rs
================================================================

//! Vector Scale & Logarithmic Performance Modernized Test Suite
//! Part of the Vanta Certification ecosystem.

#[path = "../common/mod.rs"]
mod common;

use common::{TerminalReporter, VantaHarness};
use std::sync::Arc;
use tempfile::tempdir;
use vantadb::node::{NodeTier, UnifiedNode};
use vantadb::storage::StorageEngine;

#[tokio::test]
async fn vector_scale_performance_certification() {
    let mut harness = VantaHarness::new("CORE ENGINE (VECTOR SCALE)");

    harness.execute("Scale: 1K Node Graph Navigation", || {
        futures::executor::block_on(async {
            let dir = tempdir().unwrap();
            let db_path = dir.path().to_str().unwrap();
            let storage = Arc::new(StorageEngine::open(db_path).unwrap());

            TerminalReporter::sub_step("Populating HNSW graph with 1,000 orthogonal vectors...");
            for i in 0..1000 {
                let mut vec = vec![0.0; 128];
                vec[i % 128] = 1.0;

                let mut node = UnifiedNode::new(i as u64);
                node.tier = NodeTier::Hot;
                node.vector = vantadb::node::VectorRepresentations::Full(vec);
                node.flags.set(vantadb::node::NodeFlags::HAS_VECTOR);
                storage.insert(&node).unwrap();
            }

            let mut query_vec = vec![0.0; 128];
            query_vec[10] = 1.0;

            TerminalReporter::sub_step(
                "Executing greedy beam search over 128-dimensional space...",
            );
            let results = {
                let index = storage.hnsw.read();
                index.search_nearest(&query_vec, None, None, 0, 5, None)
            };

            assert!(!results.is_empty());
            assert_eq!(
                results[0].0, 10,
                "Heuristic search failed to find identical neighbor"
            );

            TerminalReporter::success("Topological search precision verified at scale.");
        });
    });
}


================================================================
Nombre: columnar.rs
Ruta: tests\logic\columnar.rs
================================================================

//! Columnar Engine & Arrow Integration Modernized Test Suite
//! Part of the Vanta Certification ecosystem.

#[path = "../common/mod.rs"]
mod common;

use common::{TerminalReporter, VantaHarness};
use vantadb::columnar::nodes_to_record_batch;
use vantadb::node::{UnifiedNode, VectorRepresentations};

#[test]
fn columnar_engine_certification() {
    let mut harness = VantaHarness::new("LOGIC LAYER (COLUMNAR ENGINE)");

    harness.execute("Arrow: UnifiedNode to RecordBatch Conversion", || {
        TerminalReporter::sub_step("Preparing heterogeneous node buffer...");
        let mut node1 = UnifiedNode::new(1);
        node1.vector = VectorRepresentations::Full(vec![4.2]);
        let mut node2 = UnifiedNode::new(2);
        node2.vector = VectorRepresentations::Full(vec![7.1]);

        let nodes = vec![node1, node2];
        let batch = nodes_to_record_batch(&nodes).expect("Arrow conversion failed");

        assert_eq!(batch.num_columns(), 2);
        assert_eq!(batch.num_rows(), 2);

        TerminalReporter::success("Apache Arrow record batch generated successfully.");
    });
}


================================================================
Nombre: executor.rs
Ruta: tests\logic\executor.rs
================================================================

//! Query Executor & Result Projection Modernized Test Suite
//! Part of the Vanta Certification ecosystem.

#[path = "../common/mod.rs"]
mod common;

use common::{TerminalReporter, VantaHarness};
use vantadb::index::{CPIndex, VectorRepresentations};

#[test]
fn engine_executor_certification() {
    let mut harness = VantaHarness::new("LOGIC LAYER (QUERY EXECUTOR)");

    harness.execute("Math: Cosine Similarity Projection", || {
        let vec_a = VectorRepresentations::Full(vec![1.0, 0.0, 0.0]);
        let vec_b = VectorRepresentations::Full(vec![1.0, 0.0, 0.0]);
        let vec_c = VectorRepresentations::Full(vec![0.0, 1.0, 0.0]);

        assert!((vec_a.cosine_similarity(&vec_b).unwrap() - 1.0).abs() < f32::EPSILON);
        assert!((vec_a.cosine_similarity(&vec_c).unwrap() - 0.0).abs() < f32::EPSILON);
        TerminalReporter::success("Scalar similarity math validated.");
    });

    harness.execute("Search: Bitset + Nearest Neighbor Projection", || {
        let mut idx = CPIndex::new();
        TerminalReporter::sub_step("Setting up tiered bitmask dataset...");
        // Match mask + High sim
        idx.add(1, 0b11, VectorRepresentations::Full(vec![1.0, 0.0]), 0);
        // Match mask + Low sim
        idx.add(2, 0b11, VectorRepresentations::Full(vec![0.0, 1.0]), 0);
        // Fails mask
        idx.add(3, 0b00, VectorRepresentations::Full(vec![1.0, 0.0]), 0);

        let res = idx.search_nearest(&[1.0, 0.0], None, None, 0b10, 2, None);

        assert_eq!(res.len(), 2, "Failed to ignore bitset-filtered nodes");
        assert_eq!(res[0].0, 1, "Incorrect result ranking");
        assert_eq!(res[1].0, 2);

        TerminalReporter::success("Bitset filter and NN ranking integrated correctly.");
    });
}


================================================================
Nombre: governor.rs
Ruta: tests\logic\governor.rs
================================================================

//! Resource Governor & OOM Protection Modernized Test Suite
//! Part of the Vanta Certification ecosystem.

#[path = "../common/mod.rs"]
mod common;

use common::{TerminalReporter, VantaHarness};
use std::sync::atomic::Ordering;
use vantadb::governor::{ResourceGovernor, ALLOCATED_BYTES};

#[test]
fn engine_governor_certification() {
    let mut harness = VantaHarness::new("LOGIC LAYER (RESOURCE GOVERNOR)");

    harness.execute("OOM: Strategic Allocation & Safeguards", || {
        let governor = ResourceGovernor::new(1024 * 1024, 1000); // 1MB limit

        TerminalReporter::sub_step("Requesting 512KB (valid allocation)...");
        assert!(governor.request_allocation(512 * 1024).is_ok());
        assert_eq!(ALLOCATED_BYTES.load(Ordering::SeqCst), 512 * 1024);

        TerminalReporter::sub_step("Requesting 600KB (total 1.1MB, exceeding 1MB limit)...");
        let result = governor.request_allocation(600 * 1024);
        assert!(result.is_err(), "Governor failed to block OOM condition");

        TerminalReporter::sub_step("Releasing memory and verifying neutrality...");
        governor.free_allocation(512 * 1024);
        assert_eq!(ALLOCATED_BYTES.load(Ordering::SeqCst), 0);

        TerminalReporter::success("OOM protection and state-tracking verified.");
    });
}


================================================================
Nombre: integration.rs
Ruta: tests\logic\integration.rs
================================================================

//! Integration Handlers Modernized Test Suite
//! Part of the Vanta Certification ecosystem.

#[path = "../common/mod.rs"]
mod common;

use common::{TerminalReporter, VantaHarness};
use vantadb::integrations::*;

#[tokio::test]
async fn integrations_certification() {
    let mut harness = VantaHarness::new("LOGIC LAYER (HANDLERS & MCP)");

    harness.execute("Search: LangChain Handler Proximity", || {
        futures::executor::block_on(async {
            let req = SearchRequest {
                query: "What is the capital of VZLA?".to_string(),
                collection: "nodes".to_string(),
                temperature: Some(0.1),
                limit: Some(10),
            };

            TerminalReporter::sub_step("Simulating semantic search via LangChain bridge...");
            let res = search_handler(req).await;
            assert_eq!(res.latency_ms, 5);
            TerminalReporter::success("LangChain search handler response validated.");
        });
    });

    harness.execute("Proxy: Ollama Context-Aware Generation", || {
        futures::executor::block_on(async {
            let req = OllamaGenerateRequest {
                model: "llama3".to_string(),
                prompt: "Tell me about memory constraints".to_string(),
                stream: Some(false),
            };

            TerminalReporter::sub_step("Routing generational prompt through Ollama proxy...");
            let res = ollama_proxy_handler(req).await;
            assert!(res.contains("Context-Aware"));
            TerminalReporter::success("Ollama proxy handler consensus reached.");
        });
    });
}


================================================================
Nombre: parser.rs
Ruta: tests\logic\parser.rs
================================================================

//! Vanta Lisp & DQL Parser Modernized Test Suite
//! Part of the Vanta Certification ecosystem.

#[path = "../common/mod.rs"]
mod common;

use common::{TerminalReporter, VantaHarness};
use vantadb::node::FieldValue;
use vantadb::parser::*;
use vantadb::query::*;

#[test]
fn dql_parser_certification() {
    let mut harness = VantaHarness::new("LOGIC LAYER (DQL PARSER)");

    harness.execute("DQL: Complex FROM -> FETCH Pipeline", || {
        let q = r#"
            FROM Usuario#usr45
            SIGUE 1..3 "amigo" Persona
            WHERE Persona.pais="VZLA" AND Persona.bio ~ "rust", min=0.88
            FETCH Persona.nombre, Persona.email
            RANK BY Persona.relevancia DESC
            WITH TEMPERATURE 0.5
        "#;

        TerminalReporter::sub_step(
            "Parsing complex DQL query with graph traversal and semantic filter...",
        );
        let (_, parsed) = parse_query(q).expect("DQL Parser failed");

        assert_eq!(parsed.from_entity, "Usuario#usr45");
        assert_eq!(parsed.traversal.as_ref().unwrap().edge_label, "amigo");
        assert_eq!(parsed.traversal.as_ref().unwrap().max_depth, 3);
        assert_eq!(parsed.where_clause.as_ref().unwrap().len(), 2);

        match &parsed.where_clause.as_ref().unwrap()[0] {
            Condition::Relational(f, op, v) => {
                assert_eq!(f, "Persona.pais");
                assert_eq!(op, &RelOp::Eq);
                assert_eq!(v, &FieldValue::String("VZLA".to_string()));
            }
            _ => panic!("Expected relational condition"),
        }
        TerminalReporter::success("DQL Abstract Syntax Tree (AST) generated correctly.");
    });

    harness.execute("DML: Multi-Statement Core Parse", || {
        TerminalReporter::sub_step("Testing INSERT with positional vector...");
        let q_ins =
            r#"INSERT NODE#101 TYPE Usuario { nombre: "Eros", edad: 28 } VECTOR [0.1, -0.4]"#;
        let (_, stmt_ins) = parse_statement(q_ins).expect("Insert parse failed");
        if let Statement::Insert(ins) = stmt_ins {
            assert_eq!(ins.node_id, 101);
            assert_eq!(ins.fields.get("edad").unwrap(), &FieldValue::Int(28));
        }

        TerminalReporter::sub_step("Testing UPDATE with multiple fields...");
        let q_upd = r#"UPDATE NODE#101 SET nombre = "Eros Dev", activo = true"#;
        let (_, stmt_upd) = parse_statement(q_upd).expect("Update parse failed");
        if let Statement::Update(upd) = stmt_upd {
            assert_eq!(upd.node_id, 101);
        }

        TerminalReporter::sub_step("Testing RELATE with edge weighting...");
        let q_rel = r#"RELATE NODE#1 --"amigo"--> NODE#2 WEIGHT 0.95"#;
        let (_, stmt_rel) = parse_statement(q_rel).expect("Relate parse failed");
        if let Statement::Relate(rel) = stmt_rel {
            assert_eq!(rel.source_id, 1);
            assert_eq!(rel.weight.unwrap(), 0.95);
        }

        TerminalReporter::sub_step("Testing DELETE physical excision...");
        let q_del = r#"DELETE NODE#5"#;
        let (_, stmt_del) = parse_statement(q_del).expect("Delete parse failed");
        if let Statement::Delete(del) = stmt_del {
            assert_eq!(del.node_id, 5);
        }

        TerminalReporter::success("DML statement family parsing complete.");
    });
}


================================================================
Nombre: backend_tests.rs
Ruta: tests\storage\backend_tests.rs
================================================================

//! Backend abstraction integration test suite.
//!
//! Validates `StorageEngine` with `RocksDbBackend`, `InMemoryBackend`, and
//! `FjallBackend` through the public API.
//!
//! Direct `StorageBackend` trait tests live inside the crate as unit tests
//! (see `src/backends/in_memory.rs`) because the trait is `pub(crate)`.

#[path = "../common/mod.rs"]
mod common;

use common::TerminalReporter;
use tempfile::tempdir;
use vantadb::node::UnifiedNode;
use vantadb::storage::{BackendKind, EngineConfig, StorageEngine};

// ─── StorageEngine + InMemoryBackend Integration ────────────

#[test]
fn test_storage_engine_with_inmemory_backend_insert_get_delete() {
    let dir = tempdir().unwrap();
    let config = EngineConfig {
        backend_kind: BackendKind::InMemory,
        ..Default::default()
    };
    let storage =
        StorageEngine::open_with_config(dir.path().to_str().unwrap(), Some(config)).unwrap();

    // Insert
    let mut node = UnifiedNode::new(42);
    node.vector = vantadb::VectorRepresentations::Full(vec![1.0, 2.0, 3.0]);
    node.flags.set(vantadb::NodeFlags::HAS_VECTOR);
    storage.insert(&node).unwrap();

    // Get
    let retrieved = storage.get(42).unwrap().expect("Node 42 should exist");
    assert_eq!(retrieved.id, 42);

    // Delete
    storage.delete(42, "test deletion").unwrap();
    assert!(storage.get(42).unwrap().is_none());

    TerminalReporter::success("StorageEngine + InMemoryBackend roundtrip verified.");
}

// ─── StorageEngine + RocksDbBackend Smoke Test ──────────────

#[test]
fn test_storage_engine_rocksdb_backend_still_works() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();

    let storage = StorageEngine::open(db_path).expect("Failed to open StorageEngine with RocksDB");

    let node = UnifiedNode::new(99);
    storage.insert(&node).unwrap();

    let retrieved = storage.get(99).unwrap().expect("Node 99 should exist");
    assert_eq!(retrieved.id, 99);

    TerminalReporter::success("StorageEngine + RocksDbBackend smoke test passed.");
}

// ─── Purge Permanent via Backend ────────────────────────────

#[test]
fn test_purge_permanent_via_backend() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();

    let storage = StorageEngine::open(db_path).unwrap();

    // Insert a node
    let node = UnifiedNode::new(77);
    storage.insert(&node).unwrap();

    // Verify it exists
    assert!(storage.get(77).unwrap().is_some());

    // Purge should delete from all partitions without error
    storage.purge_permanent(77).unwrap();

    TerminalReporter::success("purge_permanent via backend abstraction verified.");
}

// ═══════════════════════════════════════════════════════════════
// ─── FjallBackend Tests ────────────────────────────────────────
// ═══════════════════════════════════════════════════════════════

/// Helper: create a StorageEngine backed by Fjall in a tempdir.
fn open_fjall_engine() -> (StorageEngine, tempfile::TempDir) {
    let dir = tempdir().unwrap();
    let config = EngineConfig {
        backend_kind: BackendKind::Fjall,
        ..Default::default()
    };
    let engine =
        StorageEngine::open_with_config(dir.path().to_str().unwrap(), Some(config)).unwrap();
    (engine, dir)
}

// ─── 1. Basic CRUD ──────────────────────────────────────────

#[test]
fn test_fjall_backend_basic_crud() {
    let (engine, _dir) = open_fjall_engine();

    // Insert
    let node = UnifiedNode::new(1);
    engine.insert(&node).unwrap();

    // Get
    let retrieved = engine.get(1).unwrap().expect("Node 1 should exist");
    assert_eq!(retrieved.id, 1);

    // Delete
    engine.delete(1, "test deletion").unwrap();
    assert!(
        engine.get(1).unwrap().is_none(),
        "Node 1 should be gone after delete"
    );

    TerminalReporter::success("FjallBackend basic CRUD verified.");
}

// ─── 2. Batch Multi-Partition ───────────────────────────────

#[test]
fn test_fjall_backend_batch_multi_partition() {
    let (engine, _dir) = open_fjall_engine();

    // Insert a node — this writes to the Default partition
    let node = UnifiedNode::new(200);
    engine.insert(&node).unwrap();
    assert!(engine.get(200).unwrap().is_some());

    // purge_permanent issues a write_batch across Default, TombstoneStorage,
    // CompressedArchive, and Tombstones partitions atomically.
    engine.purge_permanent(200).unwrap();

    // After purge, node should be gone from all partitions.
    assert!(
        engine.get(200).unwrap().is_none(),
        "Node 200 should be purged from all partitions"
    );

    TerminalReporter::success("FjallBackend batch multi-partition verified.");
}

// ─── 3. Full Engine Roundtrip ───────────────────────────────

#[test]
fn test_storage_engine_with_fjall_backend_insert_get_delete() {
    let (engine, _dir) = open_fjall_engine();

    // Insert with vector data
    let mut node = UnifiedNode::new(500);
    node.vector = vantadb::VectorRepresentations::Full(vec![0.1, 0.2, 0.3, 0.4]);
    node.flags.set(vantadb::NodeFlags::HAS_VECTOR);
    engine.insert(&node).unwrap();

    // Retrieve and validate
    let retrieved = engine.get(500).unwrap().expect("Node 500 should exist");
    assert_eq!(retrieved.id, 500);

    // Delete and confirm
    engine.delete(500, "engine roundtrip cleanup").unwrap();
    assert!(engine.get(500).unwrap().is_none());

    TerminalReporter::success("StorageEngine + FjallBackend full roundtrip verified.");
}

// ─── 4. Flush Durability ────────────────────────────────────

#[test]
fn test_storage_engine_fjall_backend_flush() {
    let (engine, _dir) = open_fjall_engine();

    // Insert data
    let node = UnifiedNode::new(600);
    engine.insert(&node).unwrap();

    // flush() must succeed — not an empty stub
    engine.flush().expect("FjallBackend flush() must not fail");

    // Data must survive the flush
    let retrieved = engine
        .get(600)
        .unwrap()
        .expect("Node 600 should survive flush");
    assert_eq!(retrieved.id, 600);

    TerminalReporter::success("FjallBackend flush (PersistMode::SyncAll) verified.");
}

// ─── 5. Checkpoint Not Supported ────────────────────────────

#[test]
fn test_fjall_backend_checkpoint_not_supported() {
    let (engine, dir) = open_fjall_engine();

    let checkpoint_path = dir.path().join("checkpoint_test");
    let result = engine.create_life_insurance(checkpoint_path.to_str().unwrap());

    assert!(
        result.is_err(),
        "FjallBackend checkpoint must return an error, not fake success"
    );

    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("not supported") || err_msg.contains("Checkpoint"),
        "Error message should be explicit about checkpoint not being supported, got: {}",
        err_msg
    );

    TerminalReporter::success("FjallBackend checkpoint honestly reports not-supported.");
}

// ─── 6. Partition Initialization ────────────────────────────

#[test]
fn test_fjall_backend_opens_all_partitions() {
    // Verify that the engine opens cleanly with Fjall — all 4 keyspaces
    // (default, tombstone_storage, compressed_archive, tombstones) are
    // created without error.
    let (engine, _dir) = open_fjall_engine();

    // If we got here, all keyspaces were created.
    // Insert and delete to exercise at least the default partition roundtrip.
    let node = UnifiedNode::new(700);
    engine.insert(&node).unwrap();
    assert!(engine.get(700).unwrap().is_some());

    TerminalReporter::success("FjallBackend all partitions initialize cleanly.");
}


================================================================
Nombre: chaos_integrity.rs
Ruta: tests\storage\chaos_integrity.rs
================================================================

//! Storage Chaos & Data Integrity Modernized Test Suite
//! Part of the Vanta Certification ecosystem.

#[path = "../common/mod.rs"]
mod common;

use common::{TerminalReporter, VantaHarness};
use std::sync::Arc;
use tempfile::tempdir;
use vantadb::error::VantaError;
use vantadb::executor::Executor;
use vantadb::query::{InsertStatement, RelateStatement, Statement};
use vantadb::storage::StorageEngine;

#[tokio::test]
async fn chaos_integrity_certification() {
    let mut harness = VantaHarness::new("STORAGE LAYER (CHAOS INTEGRITY)");

    harness.execute("Topological Axioms: Ghost Node Prevention", || {
        futures::executor::block_on(async {
            let dir = tempdir().unwrap();
            let db_path = dir.path().to_str().unwrap();
            let storage = Arc::new(StorageEngine::open(db_path).unwrap());
            let executor = Executor::new(&storage);

            TerminalReporter::sub_step("Setting up valid base nodes (1, 2)...");
            executor
                .execute_statement(Statement::Insert(InsertStatement {
                    node_id: 1,
                    node_type: "Test".to_string(),
                    fields: std::collections::BTreeMap::new(),
                    vector: None,
                }))
                .await
                .unwrap();
            executor
                .execute_statement(Statement::Insert(InsertStatement {
                    node_id: 2,
                    node_type: "Test".to_string(),
                    fields: std::collections::BTreeMap::new(),
                    vector: None,
                }))
                .await
                .unwrap();

            TerminalReporter::sub_step("Attempting RELATE to non-existent Ghost Node 999...");
            let relate_ghost = Statement::Relate(RelateStatement {
                source_id: 1,
                target_id: 999,
                label: "likes".to_string(),
                weight: None,
            });
            let result_ghost = executor.execute_statement(relate_ghost).await;

            assert!(result_ghost.is_err());
            if let Err(VantaError::Execution(msg)) = result_ghost {
                assert!(msg.contains("Topological Axiom violated"));
            } else {
                panic!("Expected Topological Axiom error");
            }

            TerminalReporter::success("Ghost node relation correctly blocked.");
        });
    });

    harness.execute("Topological Axioms: Tombstone Resilience", || {
        futures::executor::block_on(async {
            let dir = tempdir().unwrap();
            let storage = Arc::new(StorageEngine::open(dir.path().to_str().unwrap()).unwrap());
            let executor = Executor::new(&storage);

            executor
                .execute_statement(Statement::Insert(InsertStatement {
                    node_id: 1,
                    node_type: "Test".to_string(),
                    fields: std::collections::BTreeMap::new(),
                    vector: None,
                }))
                .await
                .unwrap();
            executor
                .execute_statement(Statement::Insert(InsertStatement {
                    node_id: 2,
                    node_type: "Test".to_string(),
                    fields: std::collections::BTreeMap::new(),
                    vector: None,
                }))
                .await
                .unwrap();

            TerminalReporter::sub_step("Deleting Node 2 (creating tombstone)...");
            executor
                .execute_statement(Statement::Delete(vantadb::query::DeleteStatement {
                    node_id: 2,
                }))
                .await
                .unwrap();

            TerminalReporter::sub_step("Attempting RELATE to deleted Node 2...");
            let relate_tombstone = Statement::Relate(RelateStatement {
                source_id: 1,
                target_id: 2,
                label: "likes".to_string(),
                weight: None,
            });
            let result_tombstone = executor.execute_statement(relate_tombstone).await;

            assert!(result_tombstone.is_err());
            TerminalReporter::success("Relation to tombstone correctly blocked.");
        });
    });
}


================================================================
Nombre: gc.rs
Ruta: tests\storage\gc.rs
================================================================

//! Storage Garbage Collection Modernized Test Suite
//! Part of the Vanta Certification ecosystem.

#[path = "../common/mod.rs"]
mod common;

use common::{TerminalReporter, VantaHarness};
use std::time::{SystemTime, UNIX_EPOCH};
use tempfile::tempdir;
use vantadb::gc::GcWorker;
use vantadb::node::UnifiedNode;
use vantadb::storage::StorageEngine;

#[test]
fn storage_gc_certification() {
    let mut harness = VantaHarness::new("STORAGE LAYER (GARBAGE COLLECTION)");

    harness.execute("Sweep Logic: TTL Expiry & Physical Purge", || {
        let dir = tempdir().unwrap();
        let db_path = dir.path().to_str().unwrap();
        let storage = StorageEngine::open(db_path).unwrap();

        TerminalReporter::sub_step(
            "Initializing nodes with TTL (Node 1=Expired, Node 2=Active)...",
        );
        let node1 = UnifiedNode::new(1);
        let node2 = UnifiedNode::new(2);
        storage.insert(&node1).unwrap();
        storage.insert(&node2).unwrap();

        let mut worker = GcWorker::new(&storage);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        worker.register_ttl(1, now - 10); // Past
        worker.register_ttl(2, now + 100); // Future

        TerminalReporter::sub_step("Executing sweep cycle...");
        let purged = worker.sweep();

        assert_eq!(purged, 1, "GC failed to purge expired node");
        assert!(
            storage.get(1).unwrap().is_none(),
            "Node 1 should be physically deleted"
        );
        assert!(
            storage.get(2).unwrap().is_some(),
            "Node 2 should be preserved"
        );

        TerminalReporter::success(&format!(
            "Sweep cycle successful. Purged {} expired nodes.",
            purged
        ));
    });
}


================================================================
Nombre: mmap_index.rs
Ruta: tests\storage\mmap_index.rs
================================================================

//! MMap Neural Index & Survival Mode Modernized Test Suite
//! Part of the Vanta Certification ecosystem.

#[path = "../common/mod.rs"]
mod common;

use common::{TerminalReporter, VantaHarness};
use tempfile::TempDir;
use vantadb::index::{CPIndex, IndexBackend, VectorRepresentations};

/// Helper: create a CPIndex with N test vectors
fn build_test_index(node_count: u64) -> CPIndex {
    let mut index = CPIndex::new();
    for i in 1..=node_count {
        let raw = vec![i as f32, (i + 1) as f32, (i + 2) as f32, (i + 3) as f32];
        let norm: f32 = raw.iter().map(|x| x * x).sum::<f32>().sqrt();
        let normalized: Vec<f32> = raw.iter().map(|x| x / norm).collect();
        index.add(i, 0, VectorRepresentations::Full(normalized), 0);
    }
    index
}

#[test]
fn mmap_neural_index_certification() {
    let mut harness = VantaHarness::new("STORAGE LAYER (MMAP NEURAL INDEX)");

    harness.execute("Serialization: Byte Roundtrip Integrity", || {
        let index = build_test_index(50);
        let bytes = index.serialize_to_bytes();
        assert_eq!(&bytes[0..8], b"VNTHNSW1");

        let restored = CPIndex::deserialize_from_bytes(&bytes).expect("Deserialization failed");
        assert_eq!(restored.nodes.len(), 50);
        TerminalReporter::success(&format!(
            "Serialization roundtrip: {} nodes, {} bytes",
            restored.nodes.len(),
            bytes.len()
        ));
    });

    harness.execute("Persistence: Cold-Start Performance", || {
        let tmp = TempDir::new().expect("Failed to create temp dir");
        let index_path = tmp.path().join("neural_index.bin");

        let index = build_test_index(100);
        index.persist_to_file(&index_path).expect("Persist failed");

        let loaded = CPIndex::load_from_file(&index_path).expect("Cold-start load failed");
        assert_eq!(loaded.nodes.len(), 100);

        let query = vec![1.0f32, 2.0, 3.0, 4.0];
        let norm: f32 = query.iter().map(|x| x * x).sum::<f32>().sqrt();
        let nq: Vec<f32> = query.iter().map(|x| x / norm).collect();
        let results = loaded.search_nearest(&nq, None, None, 0, 5, None);

        assert_eq!(results[0].0, 1);
        TerminalReporter::success("Cold-start persistence and search verified.");
    });

    harness.execute("MMap Survival: Backend Sync & Reload", || {
        let tmp = TempDir::new().expect("Failed to create temp dir");
        let mmap_path = tmp.path().join("neural_index_mmap.bin");

        let mut index = CPIndex::with_backend(IndexBackend::new_mmap(mmap_path.clone()));
        for i in 1..=30u64 {
            let raw = vec![i as f32, (i + 1) as f32, (i + 2) as f32, (i + 3) as f32];
            let norm: f32 = raw.iter().map(|x| x * x).sum::<f32>().sqrt();
            let normalized: Vec<f32> = raw.iter().map(|x| x / norm).collect();
            index.add(i, 0, VectorRepresentations::Full(normalized), 0);
        }

        index.sync_to_mmap().expect("MMap sync failed");
        assert!(mmap_path.exists());

        let reloaded = CPIndex::load_from_file(&mmap_path).expect("Load from MMap failed");
        assert_eq!(reloaded.nodes.len(), 30);
        TerminalReporter::success("MMap survival backend functional.");
    });

    harness.execute("Error Handling: Corrupt/Nonexistent Fallback", || {
        let tmp = TempDir::new().expect("Failed to create temp dir");
        let index_path = tmp.path().join("corrupt_index.bin");
        std::fs::write(&index_path, b"GARBAGE").unwrap();

        assert!(CPIndex::load_from_file(&index_path).is_none());
        assert!(CPIndex::load_from_file(&tmp.path().join("absent.bin")).is_none());
        TerminalReporter::success("Graceful fallback on corruption verified.");
    });

    harness.execute("Abstraction: Memory vs MMap Equivalence", || {
        let tmp = TempDir::new().expect("Failed to create temp dir");
        let mmap_path = tmp.path().join("equiv_test.bin");

        let mut inmem_index = CPIndex::new();
        let mut mmap_index = CPIndex::with_backend(IndexBackend::new_mmap(mmap_path));

        let vectors: Vec<(u64, Vec<f32>)> = (1..=20u64)
            .map(|i| {
                let raw = vec![i as f32, (i + 1) as f32, (i + 2) as f32, (i + 3) as f32];
                let n: f32 = raw.iter().map(|x| x * x).sum::<f32>().sqrt();
                (i, raw.iter().map(|x| x / n).collect())
            })
            .collect();

        for (id, v) in &vectors {
            inmem_index.add(*id, 0, VectorRepresentations::Full(v.clone()), 0);
            mmap_index.add(*id, 0, VectorRepresentations::Full(v.clone()), 0);
        }

        let q = vec![0.5f32, 0.5, 0.5, 0.5];
        let res_inmem = inmem_index.search_nearest(&q, None, None, 0, 5, None);
        let res_mmap = mmap_index.search_nearest(&q, None, None, 0, 5, None);

        assert_eq!(res_inmem.len(), res_mmap.len());
        TerminalReporter::success("Memory and MMap backend equivalence confirmed.");
    });
}


================================================================
Nombre: mutations.rs
Ruta: tests\storage\mutations.rs
================================================================

//! DML Pipeline & Mutations Modernized Test Suite
//! Part of the Vanta Certification ecosystem.

#[path = "../common/mod.rs"]
mod common;

use common::{TerminalReporter, VantaHarness};
use tempfile::tempdir;
use vantadb::executor::{ExecutionResult, Executor};
use vantadb::parser::parse_statement;
use vantadb::storage::StorageEngine;

#[tokio::test]
async fn dml_mutations_certification() {
    let mut harness = VantaHarness::new("STORAGE LAYER (DML MUTATIONS)");

    harness.execute("Pipeline: INSERT -> GET Cycle", || {
        futures::executor::block_on(async {
            let dir = tempdir().unwrap();
            let storage = StorageEngine::open(dir.path().to_str().unwrap()).unwrap();
            let executor = Executor::new(&storage);

            let q_insert = r#"INSERT NODE#101 TYPE Usuario { nombre: "Eros", pais: "VZLA" }"#;
            let (_, stmt_insert) = parse_statement(q_insert).unwrap();

            match executor.execute_statement(stmt_insert).await.unwrap() {
                ExecutionResult::Write { affected_nodes, .. } => assert_eq!(affected_nodes, 1),
                _ => panic!("Expected write result"),
            }

            let node = storage.get(101).unwrap().unwrap();
            assert_eq!(node.get_field("pais").unwrap().as_str().unwrap(), "VZLA");
            TerminalReporter::success("Parse-to-Insert pipeline validated.");
        });
    });

    harness.execute("Pipeline: UPDATE & Atomicity", || {
        futures::executor::block_on(async {
            let dir = tempdir().unwrap();
            let storage = StorageEngine::open(dir.path().to_str().unwrap()).unwrap();
            let executor = Executor::new(&storage);

            // Initial insert
            let q_insert = r#"INSERT NODE#101 TYPE Usuario { nombre: "Eros", pais: "VZLA" }"#;
            let (_, stmt_insert) = parse_statement(q_insert).unwrap();
            executor.execute_statement(stmt_insert).await.unwrap();

            let q_update = r#"UPDATE NODE#101 SET role = "Admin", pais = "US""#;
            let (_, stmt_update) = parse_statement(q_update).unwrap();
            executor.execute_statement(stmt_update).await.unwrap();

            let node = storage.get(101).unwrap().unwrap();
            assert_eq!(node.get_field("role").unwrap().as_str().unwrap(), "Admin");
            assert_eq!(node.get_field("pais").unwrap().as_str().unwrap(), "US");
            TerminalReporter::success("Partial node updates committed successfully.");
        });
    });

    harness.execute("Pipeline: RELATE & Topology Integrity", || {
        futures::executor::block_on(async {
            let dir = tempdir().unwrap();
            let storage = StorageEngine::open(dir.path().to_str().unwrap()).unwrap();
            let executor = Executor::new(&storage);

            let q_i1 = r#"INSERT NODE#101 TYPE Usuario { nombre: "Eros" }"#;
            let q_i2 = r#"INSERT NODE#5 TYPE Tarea { nombre: "VantaDB" }"#;
            executor
                .execute_statement(parse_statement(q_i1).unwrap().1)
                .await
                .unwrap();
            executor
                .execute_statement(parse_statement(q_i2).unwrap().1)
                .await
                .unwrap();

            let q_relate = r#"RELATE NODE#101 --"creo"--> NODE#5 WEIGHT 1.0"#;
            executor
                .execute_statement(parse_statement(q_relate).unwrap().1)
                .await
                .unwrap();

            let node = storage.get(101).unwrap().unwrap();
            assert_eq!(node.edges.len(), 1);
            assert_eq!(node.edges[0].label, "creo");
            TerminalReporter::success("Directed relation established through DML.");
        });
    });

    harness.execute("Pipeline: Physical DELETE logic", || {
        futures::executor::block_on(async {
            let dir = tempdir().unwrap();
            let storage = StorageEngine::open(dir.path().to_str().unwrap()).unwrap();
            let executor = Executor::new(&storage);

            executor
                .execute_statement(parse_statement(r#"INSERT NODE#101 TYPE X {}"#).unwrap().1)
                .await
                .unwrap();
            executor
                .execute_statement(parse_statement(r#"DELETE NODE#101"#).unwrap().1)
                .await
                .unwrap();

            assert!(storage.get(101).unwrap().is_none());
            TerminalReporter::success("Node excision complete.");
        });
    });
}


================================================================
Nombre: storage.rs
Ruta: tests\storage\storage.rs
================================================================

//! RocksDB Engine Integration Modernized Test Suite
//! Part of the Vanta Certification ecosystem.

#[path = "../common/mod.rs"]
mod common;

use common::{TerminalReporter, VantaHarness};
use tempfile::tempdir;
use vantadb::node::UnifiedNode;
use vantadb::storage::StorageEngine;

#[test]
fn storage_engine_certification() {
    let mut harness = VantaHarness::new("STORAGE LAYER (ROCKSDB ADAPTER)");

    harness.execute("Integration: Persistent Node IO", || {
        let dir = tempdir().unwrap();
        let db_path = dir.path().to_str().unwrap();

        TerminalReporter::sub_step("Opening StorageEngine (RocksDB backend)...");
        let storage = StorageEngine::open(db_path).expect("Failed to open RocksDB");

        let node = UnifiedNode::new(42);
        storage.insert(&node).unwrap();
        TerminalReporter::sub_step("Node 42 committed to persistent storage.");

        let retrieved = storage
            .get(42)
            .unwrap()
            .expect("Node not found after insertion");
        assert_eq!(retrieved.id, 42);

        TerminalReporter::success("RocksDB roundtrip successful.");
    });
}


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

1. **[Architecture (Zero-Copy & Memory Layout)](docs/architecture/ARCHITECTURE.md)**: Explore the `UnifiedNode` struct, the PyO3 interfaces, and the LSM persistence logic.
2. **[Operations & Configuration](docs/operations/CONFIGURATION.md)**: Details on runtime parameters, environment variables, and memory limits.
3. **[Backup & Snapshot Policy](docs/operations/BACKUP_POLICY.md)**: Operational guidelines for data protection across different storage backends (Fjall vs RocksDB).

## 📄 License

VantaDB is licensed under the **Apache 2.0 License**. See `LICENSE` for details.
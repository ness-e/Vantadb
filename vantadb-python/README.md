# 🐍 VantaDB Python SDK

Official Python bindings for **VantaDB**, an embedded, native-Rust database engine designed for **persistent memory, hybrid retrieval and graph queries** in local-first AI applications.

## Why VantaDB instead of a plain vector store?

Most embedded vector databases (e.g. ChromaDB) index vectors and stop there. VantaDB ships the missing pieces agents actually need:

- **Hybrid search with RRF fusion** — dense vector ANN (HNSW) and lexical BM25 run together and fuse via Reciprocal Rank Fusion, so semantic misses get caught by keyword matches (and vice versa). One call (`search_memory`), one ranked result set.
- **Graph and memory in one engine** — namespace-scoped memory records live next to a property graph with typed edges: BFS/DFS traversals, PageRank, cycle detection and topological sort, all queryable through IQL (`query` / `query_structured`).
- **Explicit memory lifecycle** — per-record TTL expiry (`purge_expired`) and atomic fact replacement (`supersede`) without delete/reinsert races.
- **Built-in migration paths** — `bulk_import` / `bulk_import_bytes` for fast ingestion, `export_namespace` / `export_all` for backup, and `reindex_hnsw_from_text` to rebuild indexes from stored payloads after schema or index changes.

## 📦 Installation

### From PyPI (Recommended)
```bash
pip install vantadb-py
```

> **Note:** The distribution name is `vantadb-py` and the canonical import is `import vantadb` (same as the Rust crate and the npm package). `import vantadb_py` remains available and is not broken.

### From TestPyPI (Pre-release testing)
```bash
pip install --index-url https://test.pypi.org/simple/ --extra-index-url https://pypi.org/simple/ vantadb-py
```

### From Source (Development)
Requires [Rust](https://rustup.rs/) and [Maturin](https://github.com/PyO3/maturin).
```bash
# Clone the repository
git clone https://github.com/ness-e/Vantadb.git
cd Vantadb/vantadb-python

# Compile and install into the active virtual environment
pip install maturin
maturin develop --release
```

## 🚀 Quickstart

```python
import vantadb

# 1. Open or create an embedded database
db = vantadb.VantaDB("./my_agent_memory", memory_limit_bytes=128 * 1024 * 1024)

# 2. Store persistent memory (payload + vector + metadata)
db.put(
    namespace="agent/session_1",
    key="fact_001",
    payload="The user prefers direct, technical answers.",
    metadata={"source": "chat", "priority": "high"},
    vector=[0.1, 0.2, 0.3, 0.4]  # Dense vector (e.g. embedding from a local model)
)

# 3. Retrieve the exact record
record = db.get_memory("agent/session_1", "fact_001")
print(record["payload"])

# 4. Hybrid search (vector + lexical)
# Note: The query vector must match the dimensionality of the stored vectors
query_vector = [0.15, 0.25, 0.35, 0.45]
results = db.search_memory(
    namespace="agent/session_1",
    query_vector=query_vector,
    text_query="user preferences",
    top_k=5
)

for hit in results:
    print(f"Key: {hit.key}, Score: {hit.score:.4f}")

# 5. Resource monitoring (critical for local agents)
stats = db.operational_metrics()
print(f"Logical usage: {stats['hnsw_logical_bytes'] / 1024:.2f} KB")
print(f"Physical RSS: {stats['process_rss_bytes'] / 1024:.2f} KB")

# 6. Clean shutdown
db.close()
```

## 🔢 Real Embeddings

The vectors above are toy examples. VantaDB stores and searches any dense
vector but does not generate embeddings — bring your own client (local
[Ollama](https://ollama.com) or the OpenAI API):

```python
import json, urllib.request

def embed(text: str) -> list[float]:
    req = urllib.request.Request(
        "http://localhost:11434/api/embed",
        data=json.dumps({"model": "nomic-embed-text", "input": text}).encode(),
        headers={"Content-Type": "application/json"},
    )
    return json.load(urllib.request.urlopen(req))["embeddings"][0]

db.put(
    namespace="agent/session_1",
    key="fact_002",
    payload="The user prefers direct, technical answers.",
    metadata={"source": "chat"},
    vector=embed("user tone preferences"),
)
```

Use **one embedding model per namespace** — stored and query vectors must
share the same dimensionality. Full walkthrough:
[QUICKSTART → Real Embeddings](../docs/QUICKSTART.md#4-real-embeddings-optional).

## Cross-SDK Search Parity

VantaDB exposes the same search capabilities across bindings, but **the `search()`
name carries different semantics per SDK**. Read this before porting code between
Python and TypeScript. The canonical method→domain map lives in
[`docs/api/BINDINGS_NAMESPACES.md`](../docs/api/BINDINGS_NAMESPACES.md).

| Capability | Python SDK | TypeScript SDK |
|---|---|---|
| `search()` meaning | **Pure vector ANN** (K-NN) → returns `(node_id, distance)` | **Hybrid** search (vector + text) → returns `SearchHit[]` |
| Pure vector ANN | `search(vector, top_k=10)` | `searchVector(vector, topK?)` |
| Hybrid (vector + text) | `search_memory(namespace, query_vector, text_query=...)` | `search({ namespace, query_vector, text_query })` |
| Namespace scoping | `search_memory(namespace=...)` (`search()` is global over nodes) | `search({ namespace })` |
| Filters | `search_memory(filters=...)` | `search({ filters })` |
| `top_k` | `search(top_k=)` / `search_memory(top_k=)` | `search({ top_k })` / `searchVector(v, topK)` |
| `distance_metric` | `search_memory(distance_metric="cosine"/"euclidean")` | `search({ distance_metric: "Cosine"/"Euclidean" })` |
| `text_query` | `search_memory(text_query=...)` | `search({ text_query })` |
| Explain | `search_memory(explain=True)` + `explain_memory_search()` | `search({ explain })` + `explainSearch()` |
| Batch search | `search_batch(vectors)` / `search_batch_requests(requests)` — **Python-only** | — |
| Hybrid method / profile override | `search_memory(method=...)` — **Python-only** | — |

> **Porting hazard:** `search()` in Python and `search()` in TypeScript do **different
> things**. To get hybrid search in Python use `search_memory()`; to get pure vector
> ANN in TypeScript use `searchVector()`.

## 🤖 Use Case: Memory for AI Agents

VantaDB is optimized to act as **long-term memory** for local autonomous agents (Claude, Gemini, LLaMA, etc.):

- **Zero-Copy Persistence**: Data survives agent restarts with no serialization overhead.
- **Hybrid RRF search**: Combines semantic similarity (vectors) with lexical matching (BM25) for precise context retrieval.
- **Explicit Memory Control**: `memory_limit_bytes` prevents the agent from collapsing the host device's RAM.
- **Embedded**: No external servers, no Docker, no network latency. Ideal for edge and offline devices.

## 🛠️ Development and Testing

```bash
# Run the SDK test suite
pytest tests/test_sdk.py -v

# Format Python code
black tests/ vantadb_python/
```

## 📜 License
Distributed under the VantaDB main project license. See the `LICENSE` at the repository root.

# 🐍 VantaDB Python SDK

Official Python bindings for **VantaDB**, an embedded, native-Rust database engine designed for **persistent memory and vector retrieval** in local-first AI applications.

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

# 1. Abrir o crear una base de datos embebida
db = vantadb.VantaDB("./my_agent_memory", memory_limit_bytes=128 * 1024 * 1024)

# 2. Almacenar memoria persistente (payload + vector + metadatos)
db.put(
    namespace="agent/session_1",
    key="fact_001",
    payload="El usuario prefiere respuestas técnicas y directas.",
    metadata={"source": "chat", "priority": "high"},
    vector=[0.1, 0.2, 0.3, 0.4]  # Vector denso (ej. embedding de un modelo local)
)

# 3. Recuperar memoria exacta
record = db.get_memory("agent/session_1", "fact_001")
print(record["payload"])

# 4. busqueda-hibrida (Vectorial + Léxica)
# Nota: Requiere un vector de consulta del mismo tamaño que los almacenados
query_vector = [0.15, 0.25, 0.35, 0.45]
results = db.search_memory(
    namespace="agent/session_1",
    query_vector=query_vector,
    text_query="preferencias usuario",
    top_k=5
)

for hit in results:
    print(f"Key: {hit.key}, Score: {hit.score:.4f}")

# 5. Monitoreo de recursos (Crítico para agentes locales)
stats = db.operational_metrics()
print(f"Uso lógico: {stats['hnsw_logical_bytes'] / 1024:.2f} KB")
print(f"RSS físico: {stats['process_rss_bytes'] / 1024:.2f} KB")

# 6. Cierre seguro
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
    payload="El usuario prefiere respuestas técnicas y directas.",
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

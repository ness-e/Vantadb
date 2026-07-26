# VantaDB Demo App

End-to-end showcase of VantaDB's embedded persistent memory engine for local AI agents, vector search, and hybrid retrieval.

## What it demonstrates

| Feature | Detail |
|---------|--------|
| **Database lifecycle** | Create, open, flush, close, reopen |
| **Document insertion** | Text + metadata + embedding vectors |
| **Dense vector search** | ANN (approximate nearest neighbor) via HNSW |
| **Hybrid search** | Dense vector + BM25 text fusion |
| **Persistence** | Data survives close/reopen cycle |
| **Operational metrics** | Memory usage, HNSW stats, cache telemetry |
| **Hardware profile** | Capabilities + runtime resource introspection |

## Requirements

- Python ≥ 3.11
- [vantadb-py](https://test.pypi.org/project/vantadb-py/) ≥ 0.4
- _Optional:_ `sentence-transformers` for real embeddings

## Installation

```bash
# 1. Install VantaDB Python SDK
pip install --index-url https://test.pypi.org/simple/ \
  --extra-index-url https://pypi.org/simple/ \
  vantadb-py

# 2. (Optional) Enable real embeddings
pip install sentence-transformers

# 3. Run the demo
python examples/demo/demo.py
```

### Build from source (development)

```bash
# Requires Rust toolchain
cd vantadb-python
pip install maturin
maturin develop --release
cd ..
python examples/demo/demo.py
```

## Expected output

```
════════════════════════════════════════════════════════
   VantaDB  —  Embedded Vector-Graph Database
   Demo App
════════════════════════════════════════════════════════

────────────────────────────────────────────────────────
  1. Create / open database
────────────────────────────────────────────────────────
  Storage  : /tmp/vantadb_demo
  Profile  : desktop
  Backend  : fjall
  Vector   : hnsw
  Persist  : aio

────────────────────────────────────────────────────────
  2. Insert documents (text + metadata + vectors)
────────────────────────────────────────────────────────
  [embed] all-MiniLM-L6-v2 loaded
  ✓ alice        dim=384  meta={'person': 'Alice', 'activity': 'hiking', 'season': 'summer'}
  ✓ bob          dim=384  meta={'person': 'Bob', 'profession': 'engineer', 'topic': 'data'}
  ...
```

## Files

| File | Purpose |
|------|---------|
| `demo.py` | Main demo script |
| `requirements.txt` | Python dependencies |
| `README.md` | This file |

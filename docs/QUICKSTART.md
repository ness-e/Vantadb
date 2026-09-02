---
title: VantaDB 5-Minute Quickstart
type: documentation
status: active
tags: [vantadb]
last_reviewed: 2026-07-01
aliases: []
---

# VantaDB 5-Minute Quickstart

This quickstart validates the current v0.4.x MVP boundary from a clean local
checkout. It uses the embedded CLI for operational flows and the source-installed
Python binding for vector, text, and hybrid memory search.

No external database service, Docker container, Ollama runtime, or network LLM is
required.

## 1. Prerequisites

- Rust stable toolchain
- Python 3.11 or newer
- `pip`
- Platform build tools needed by Rust dependencies

On Ubuntu, install the native dependencies used by CI:

```bash
sudo apt-get update
sudo apt-get install -y libclang-dev clang librocksdb-dev
```

## 2. Clone and Build the CLI

```bash
git clone https://github.com/ness-e/Vantadb.git
cd Vantadb
cargo run --bin vanta-cli -- --help
```

## 3. Put and Read Memory with the CLI

```bash
cargo run --bin vanta-cli -- put \
  --db ./quickstart_data \
  --namespace agent/main \
  --key memory-1 \
  --payload "local durable memory"

cargo run --bin vanta-cli -- get \
  --db ./quickstart_data \
  --namespace agent/main \
  --key memory-1

cargo run --bin vanta-cli -- list \
  --db ./quickstart_data \
  --namespace agent/main
```

Expected result: `get` prints `local durable memory`, and `list` shows
`memory-1`.

## 4. Install the Python Binding from Source

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip maturin pytest
python -m pip install -e ./vantadb-python
```

On Windows PowerShell:

```powershell
python -m venv .venv
.\.venv\Scripts\Activate.ps1
python -m pip install --upgrade pip maturin pytest
python -m pip install -e .\vantadb-python
```

### Alternative: Install from a Pre-built Wheel

If a wheel is available from the GitHub Actions `Python Wheels` workflow or a
GitHub Release, install it directly without needing the Rust toolchain:

```bash
python -m venv .venv
source .venv/bin/activate
pip install --upgrade pip pytest
pip install ./path/to/vantadb_py-0.1.1-*.whl
```

### Alternative: Install from TestPyPI

When a TestPyPI release is available:

```bash
python -m venv .venv
source .venv/bin/activate
pip install --upgrade pip
pip install --index-url https://test.pypi.org/simple/ --extra-index-url https://pypi.org/simple/ vantadb-py
```

> **Note**: TestPyPI availability depends on the `TEST_PYPI_API_TOKEN` secret
> being configured in the repository. Production PyPI is not yet available.

## 5. Search by Vector, Text, and Hybrid Retrieval

Create `quickstart_memory.py`:

```python
import vantadb_py as vantadb

db = vantadb.VantaDB("./quickstart_data", memory_limit_bytes=128_000_000)

db.put(
    "agent/main",
    "vector",
    "HNSW vector retrieval works in-process",
    metadata={"kind": "note"},
    vector=[1.0, 0.0, 0.0],
)
db.put(
    "agent/main",
    "text",
    "BM25 lexical retrieval finds durable local memory",
    metadata={"kind": "note"},
    vector=[0.0, 1.0, 0.0],
)
db.put(
    "agent/main",
    "hybrid",
    "Hybrid Retrieval v1 fuses BM25 and vector rankings",
    metadata={"kind": "note"},
    vector=[0.9, 0.1, 0.0],
)

vector_hits = db.search_memory("agent/main", [1.0, 0.0, 0.0], top_k=3)
text_hits = db.search_memory("agent/main", [], text_query="durable memory", top_k=3)
hybrid_hits = db.search_memory(
    "agent/main",
    [1.0, 0.0, 0.0],
    text_query="Hybrid Retrieval",
    top_k=3,
)

print("vector:", [hit.key for hit in vector_hits])
print("text:", [hit.key for hit in text_hits])
print("hybrid:", [hit.key for hit in hybrid_hits])

db.flush()
db.close()
```

Run it:

```bash
python quickstart_memory.py
```

## 6. Export and Audit

```bash
cargo run --bin vanta-cli -- export \
  --db ./quickstart_data \
  --namespace agent/main \
  --out ./quickstart-agent-main.jsonl

cargo run --bin vanta-cli -- audit-index \
  --db ./quickstart_data \
  --namespace agent/main \
  --json
```

Expected result: export reports records written, and audit reports
`"passed": true`.

## 7. Optional: Local Embeddings (`embed-local`)

VantaDB is BYO-vector by default — you pass `vector=[...]` as above. For fully offline use, enable the optional `embed-local` feature (no Ollama, no network):

```bash
# 1. Download the default model (384d, 220 MB ONNX, 691 MB total)
python embeddings/download.py --only multilingual-e5-small

# 2. Run with embed-local (Rust + CLI + MCP + SQL)
cargo run --features embed-local --bin vanta-cli -- --help
WANTA_EMBEDDING_PROVIDER=local WANTA_LOCAL_MODEL=embeddings/models/multilingual-e5-small/onnx \
  cargo run --features embed-local --bin vanta-cli -- put --db ./quickstart_data --namespace agent/main --key hello --payload "hola mundo"

# 3. SQL auto-embed now works offline: VECTOR_SEARCH('hola mundo') → LocalOnnxProvider
# 4. Verify without downloading (CI-friendly)
python embeddings/verify.py --check
```

`embed-local` is **Optional** (not Experimental): `LocalOnnxProvider` via `ort`+`tokenizers` (9 models, default `multilingual-e5-small` 384d, `embeddings/manifest.json` as source of truth, `embed-batch` via `EmbeddingProvider::embed_batch`). See `docs/api/EMBEDDINGS.md` and `docs/tutorials/05-embedding-integrations.md` for the full 9-model matrix and the one-model-per-namespace rule.

## Current Boundary

This quickstart covers the production-facing MVP: embedded storage, WAL-backed
recovery, namespaces, metadata-bearing memory records, HNSW vector retrieval,
BM25 text retrieval, Hybrid Retrieval v1, JSONL export, and text-index audit.

> **MVP = embedded memory + WAL + vector/BM25/hybrid + export/import + CLI/Python**

It does not cover IQL/LISP/DQL, MCP, enterprise features, cloud, plugins, or graph database behavior.
Remote Ollama/LLM integration remains an external optional path; the preferred offline path is now `embed-local` (no external service). See `docs/operations/EXPERIMENTAL_FEATURES.md` for the full boundary.

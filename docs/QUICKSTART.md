# VantaDB 5-Minute Quickstart

This quickstart validates the current v0.1.x MVP boundary from a clean local
checkout. It uses the embedded CLI for operational flows and the source-installed
Python binding for vector, text, and hybrid memory search.

No external database service, Docker container, Ollama runtime, or network LLM is
required.

## 1. Prerequisites

- Rust stable toolchain
- Python 3.8 or newer
- `pip`
- Platform build tools needed by Rust dependencies

On Ubuntu, install the native dependencies used by CI:

```bash
sudo apt-get update
sudo apt-get install -y libclang-dev clang librocksdb-dev
```

## 2. Clone and Build the CLI

```bash
git clone https://github.com/DevpNess/Vantadb.git
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

print("vector:", [hit["record"]["key"] for hit in vector_hits])
print("text:", [hit["record"]["key"] for hit in text_hits])
print("hybrid:", [hit["record"]["key"] for hit in hybrid_hits])

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

## Current Boundary

This quickstart covers the production-facing MVP: embedded storage, WAL-backed
recovery, namespaces, metadata-bearing memory records, HNSW vector retrieval,
BM25 text retrieval, Hybrid Retrieval v1, JSONL export, and text-index audit.

It does not cover IQL/LISP/DQL, MCP, Ollama/LLM integration, enterprise
features, cloud, plugins, or graph database behavior.

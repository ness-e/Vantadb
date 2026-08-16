---
title: VantaDB 5-Minute Quickstart
type: documentation
status: active
tags: [vantadb]
last_reviewed: 2026-07-01
aliases: []
---

# VantaDB 5-Minute Quickstart

This quickstart validates the current v0.5.0 MVP boundary from a clean
environment: pre-built Python wheel and npm package, then a first hybrid query
in under 5 minutes.

No external database service, Docker container, Ollama runtime, or network LLM is
required.

## 1. Prerequisites

- Python 3.11 or newer
- `pip`
- Node.js 18+ (only for the TypeScript SDK section)

No external database service, Docker container, Ollama runtime, or network LLM is
required.

## 2. Install the Python Binding (Fastest Path)

The pre-built wheel installs in seconds — no Rust toolchain needed:

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
pip install vantadb-py
```

On Windows PowerShell:

```powershell
python -m venv .venv
.\.venv\Scripts\Activate.ps1
python -m pip install --upgrade pip
pip install vantadb-py
```

> **Note**: The distribution name is `vantadb-py`; the importable module uses an
> underscore: `import vantadb_py`.

### Alternative: Install from Source (Development)

Requires the Rust stable toolchain and platform build tools (on Ubuntu:
`sudo apt-get install -y libclang-dev clang librocksdb-dev`):

```bash
git clone https://github.com/ness-e/Vantadb.git
cd Vantadb
python -m pip install --upgrade pip maturin pytest
python -m pip install -e ./vantadb-python
```

### Alternative: Install from a Pre-built Wheel Artifact

If a wheel is available from the GitHub Actions `Python Wheels` workflow or a
GitHub Release, install it directly without needing the Rust toolchain:

```bash
python -m venv .venv
source .venv/bin/activate
pip install --upgrade pip pytest
pip install ./path/to/vantadb_py-0.5.0-*.whl
```

## 3. First Query with Python

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

## 4. TypeScript SDK (Node.js 18+)

The npm package ships the WASM engine — no build step:

```bash
npm install vantadb
```

Create `quickstart.ts`:

```ts
import { VantaDB } from "vantadb";

const db = VantaDB.create();

await db.put({
  namespace: "memories",
  key: "greeting",
  payload: "Hello, world!",
  metadata: { lang: { String: "en" } },
  vector: [0.1, 0.2, 0.3],
});

const hits = await db.search({
  namespace: "memories",
  query_vector: [0.1, 0.2, 0.3],
  top_k: 10,
});

console.log(hits[0].record.payload); // "Hello, world!"

db.close();
```

Run it:

```bash
npx tsc quickstart.ts && node quickstart.js
```

## 5. Export and Audit (Optional CLI)

The embedded CLI (`vanta-cli`) covers operational flows. Build it from source
(requires the Rust toolchain) or install the precompiled binary, then:

```bash
vanta-cli export \
  --db ./quickstart_data \
  --namespace agent/main \
  --out ./quickstart-agent-main.jsonl

vanta-cli audit-index \
  --db ./quickstart_data \
  --namespace agent/main \
  --json
```

Expected result: export reports records written, and audit reports
`"passed": true`.

## Measured Time-to-First-Query

Measured locally (2026-08-16, Windows, Python 3.11.9 / Node 24.16.0) against the
published 0.5.0 packages — install + first query, cold cache:

| SDK | Install | First query | Total |
| :--- | :--- | :--- | :--- |
| **Python** (`pip install vantadb-py`) | 5.5 s | 0.7 s | **~6 s** |
| **TypeScript** (`npm install vantadb`) | 1.3 s | 0.3 s | **~2 s** |

Both paths stay well under the 5-minute target. The dominant friction in the
original docs was broken code samples (wrong hit access / metadata shape), not
install time — fixed above.

## ID limits

Node IDs are **u128** end-to-end (core engine, WAL, and Python binding):

- **Range:** `0 <= id <= 2^128 - 1` (`340282366920938463463374607431768211455`).
- **Python:** pass any ID as a plain `int`. Python integers are arbitrary
  precision, so IDs larger than `u64::MAX` (`18446744073709551615`) work
  directly — there is **no u64 truncation** (ERR-023 fixed the previous
  u64-only limit in the binding).
- **Strings:** not required for the regular APIs; `recover_archived_nodes()`
  is the only method that takes an ID as a decimal string.
- **Out of range:** negative IDs or IDs above `u128::MAX` raise
  `OverflowError`.
- **JSON:** numbers beyond `2^53` lose precision in IEEE-754 JSON decoders —
  keep large IDs as strings in JSON payloads.

## Current Boundary

This quickstart covers the production-facing MVP: embedded storage, WAL-backed
recovery, namespaces, metadata-bearing memory records, HNSW vector retrieval,
BM25 text retrieval, Hybrid Retrieval v1, JSONL export, and text-index audit.

> **MVP = embedded memory + WAL + vector/BM25/hybrid + export/import + CLI/Python**

It does not cover IQL/LISP/DQL, MCP, Ollama/LLM integration, enterprise
features, cloud, plugins, or graph database behavior.

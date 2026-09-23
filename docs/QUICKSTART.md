---
title: VantaDB 5-Minute Quickstart
type: documentation
status: active
tags: [vantadb]
last_reviewed: 2026-09-15
aliases: []
---

# VantaDB 5-Minute Quickstart

This quickstart validates the current v0.6.1 boundary from pre-built artifacts
(no clone, no Rust toolchain required): install `vantadb-py` from PyPI /
`vantadb` from npm, or use the embedded CLI. The from-source path for
contributors is kept in §1–§4.

No external database service, Docker container, Ollama runtime, or network LLM is
required.

## 0. Install without cloning (one-liner)

No clone, no Rust toolchain. Installs the pre-compiled `vanta-cli` and chains
to the interactive setup wizard (model + MCP block per client + proxy
default-on; skip with `--no-wizard` / `-NoWizard`):

- **Linux / macOS / WSL**:

  ```bash
  curl -fsSL https://raw.githubusercontent.com/ness-e/Vantadb/main/scripts/install.sh | sh
  ```

- **Windows (PowerShell)**:

  ```powershell
  irm https://raw.githubusercontent.com/ness-e/Vantadb/main/scripts/install.ps1 | iex
  ```

Preview the installer→wizard chain without effects (or verify manually by
downloading the script first and comparing its hash with the published
`.sha256` release asset):

```bash
curl -fsSL https://raw.githubusercontent.com/ness-e/Vantadb/main/scripts/install.sh -o install.sh
sh install.sh --dry-run
```

```powershell
irm https://raw.githubusercontent.com/ness-e/Vantadb/main/scripts/install.ps1 -OutFile install.ps1
pwsh -NoProfile -File install.ps1 -DryRun
```

> **Note**: the Python examples in §5 use `vantadb-py>=0.6.1` from PyPI
> (`pip install vantadb-py==0.6.1` — verified 2026-09-23: vector/text/hybrid
> search green in a clean venv, no Rust toolchain).
> TypeScript examples live in [`vantadb-ts/examples/`](../vantadb-ts/examples/)
> (LangChain, LlamaIndex, Vercel AI SDK — indexed in [examples/README.md](../examples/README.md)).
> The steps below (§1-§4) are the from-source path for contributors.

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

> **Recommended (no Rust toolchain):** install the published wheel instead:
>
> ```powershell
> python -m venv .venv
> .\.venv\Scripts\Activate.ps1
> python -m pip install --upgrade pip
> pip install vantadb-py==0.6.1
> ```
>
> From-source install (contributors):

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
pip install ./dist/vantadb_py-0.6.1-*.whl
```

Wheels are attached to each GitHub Release (`release-wheels-60.yml`) and built
locally with `maturin build --out dist --manifest-path ./vantadb-python/Cargo.toml`
(`vantadb-python/dist/` when building from that directory).

### Alternative: Install from TestPyPI

When a TestPyPI release is available:

```bash
python -m venv .venv
source .venv/bin/activate
pip install --upgrade pip
pip install --index-url https://test.pypi.org/simple/ --extra-index-url https://pypi.org/simple/ vantadb-py
```

> **Note**: TestPyPI carries the same version (`0.6.1`). Both registries publish
> via OIDC trusted publishing (no API tokens). Production PyPI is available
> since 0.6.1 — prefer it over TestPyPI.

## 5. Search by Vector, Text, and Hybrid Retrieval

Create `quickstart_memory.py`:

```python
import vantadb

db = vantadb.Client("./quickstart_data", memory_limit_bytes=128_000_000)

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

vector_hits = db.search("agent/main", [1.0, 0.0, 0.0], top_k=3)
text_hits = db.search("agent/main", [], text_query="durable memory", top_k=3)
hybrid_hits = db.search(
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
`"passed": true`. On a fresh database audit may report
`"status": "repair_recommended"` until the text index state is built —
run `cargo run --bin vanta-cli -- rebuild-index --db ./quickstart_data`
and re-run audit (verified 2026-09-15: `passed: true` after rebuild).

## 7. Optional: Local Embeddings (`embed-local`)

VantaDB is BYO-vector by default — you pass `vector=[...]` as above. For fully offline use, enable the optional `embed-local` feature (no Ollama, no network):

```bash
# 1. Download the default model (384d, 220 MB ONNX, 691 MB total)
python embeddings/download.py --only multilingual-e5-small

# 2. Run with embed-local (Rust + CLI + MCP + SQL)
cargo run --features embed-local --bin vanta-cli -- --help
VANTADB_EMBEDDING_PROVIDER=local VANTADB_LOCAL_MODEL=embeddings/models/multilingual-e5-small/onnx \
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

> For runnable examples beyond this quickstart, see [examples/](../examples/README.md) (demo + Colab).

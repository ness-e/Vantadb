---
title: "Embedding Integrations — BYO-Vector + embed-local"
type: tutorial
status: active
tags: [vantadb, tutorial, embeddings, embed-local, byo-vector]
last_reviewed: 2026-08-28
aliases: []
---

# Embedding Integrations — BYO-Vector + `embed-local`

VantaDB **does not embed text for you** by default. This is intentional (BYO-vector): you control the embedding model, dimension, and lifecycle. Bring your own `vector=[...]` on `put()` and `query_vector` on `search_memory()`.

`embed-local` adds an **opt-in** local path: 9 ONNX+HF models (8 ≤3 GB + 1 exception) via `LocalOnnxProvider` (`ort`+`tokenizers`, `embed-local` feature). No breaking changes.

> **Source of truth:** `embeddings/manifest.json` (rev pinned). One-liner: `python embeddings/download.py --only multilingual-e5-small`.

## BYO-vector (default) — you embed, VantaDB stores + searches

```python
import vantadb_py as vantadb

db = vantadb.VantaDB("./vanta_data", memory_limit_bytes=512_000_000)

# You choose the model (e.g., sentence-transformers, OpenAI, Ollama) and pass vectors:
db.put("agent/main", "k1", "hola mundo",  vector=[0.12]*384)
db.put("agent/main", "k2", "hello world", vector=[0.11]*384)

hits = db.search_memory("agent/main", query_vector=[0.12]*384, text_query="hola", top_k=5)
```

Works with any provider. You are responsible for `dim` consistency.

## `embed-local` (optional) — offline ONNX, no network

```bash
# 1. Download default (EN+ES 16+ langs, 384d)
python embeddings/download.py --only multilingual-e5-small

# 2. Rust / CLI / SQL / MCP with embed-local
cargo run --features embed-local --bin vanta-cli -- --help
WANTA_EMBEDDING_PROVIDER=local WANTA_LOCAL_MODEL=embeddings/models/multilingual-e5-small/onnx \
  cargo run --features embed-local --bin vanta-cli -- put --db ./vanta_data --namespace agent/main --key k3 --payload "hola mundo"

# 3. Verify (offline check, no download)
python embeddings/verify.py --check         # dims, revs, 3/3/3 balance
python embeddings/download.py --check      # manifest v1, 9 models
python benchmarks/embed_bench.py --models multilingual-e5-small,bge-m3 --dataset tiny-en-es-1k
```

Rust:

```rust
use vantadb::llm::{EmbeddingProvider, LocalOnnxProvider, get_embedding_provider};

let provider = LocalOnnxProvider::new("embeddings/models/multilingual-e5-small/onnx")
    .unwrap_or_else(|_| LocalOnnxProvider::new_dummy(384));
let v = provider.embed("hola mundo")?;               // 384d, L2-normalized
let batch = provider.embed_batch(&["hola mundo".into(), "hello world".into()])?;

let via_env: Box<dyn EmbeddingProvider> = get_embedding_provider(); // VANTA_EMBEDDING_PROVIDER=local
```

Env:

```bash
VANTA_EMBEDDING_PROVIDER=local        # local | ollama | openai
VANTA_LOCAL_MODEL=embeddings/models/multilingual-e5-small/onnx
```

See `docs/api/EMBEDDINGS.md` for `LocalOnnxProvider` internals (mean-pool + L2, `load-dynamic`, dummy fallback).

## Model matrix — 9 models (8 ≤3 GB + 1 exception >3 GB)

> All sizes are **per-format**; total per model = ONNX+HF. All accept EN or ES.

| # | id (manifest) | Repo HF | Dim | Params | ONNX | HF | Total | Group | MTEB* | Langs | Licence | Role |
|---|----------------|---------|-----|--------|------|----|-------|-------|-------|---------|----------|-----|
| 1 | `bge-small-en-v1.5` | `BAAI/bge-small-en-v1.5` | 384 | 33M | 120 MB | 133 MB | **253 MB** ≤3 GB ✅ | **EN** | 62.2 | EN | MIT | baseline rápido |
| 2 | `all-MiniLM-L6-v2` | `sentence-transformers/all-MiniLM-L6-v2` | 384 | 22M | 80 MB | 90 MB | **170 MB** ✅ | **EN** | 56.8 | EN | Apache-2.0 | ultra-ligero |
| 3 | `bge-base-en-v1.5` | `BAAI/bge-base-en-v1.5` | 768 | 109M | 440 MB | 438 MB | **878 MB** ✅ | **EN** | 64.8 | EN | MIT | EN balance |
| 4 | `jina-es-v2-base` | `jinaai/jina-embeddings-v2-base-es` | 768 | 161M | 1100 MB | 1100 MB | **2.20 GB** ✅ | **ES** | 64.5 multi | ES+EN | Apache-2.0 | **ES optimizado** |
| 5 | `paraphrase-multilingual-MiniLM-L12-v2` | `sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2` | 384 | 118M | 470 MB | 471 MB | **941 MB** ✅ | **ES** | 63.0 multi | ES+EN 50+ | Apache-2.0 | ES multi ligero |
| 6 | `distiluse-multilingual` | `sentence-transformers/distiluse-base-multilingual-cased-v1` | 512 | 135M | 540 MB | 539 MB | **1.08 GB** ✅ | **ES** | 62.8 multi | ES+EN 15+ | Apache-2.0 | ES multi base |
| 7 | **`multilingual-e5-small`** | `intfloat/multilingual-e5-small` | 384 | 118M | 220 MB | 471 MB | **691 MB** ✅ | **Combined** | 64.1 multi | **ES+EN 16+** | MIT | **DEFAULT** |
| 8 | `bge-m3` | `BAAI/bge-m3` | 1024 | 568M | 1.20 GB (int8) | 2.27 GB | **3.47 GB*** ✅ | **Combined** | 69.1 | ES+EN 100+ | MIT | SOTA local ≤3 GB int8 |
| 9 | `qwen3-embedding-8b` | `Qwen/Qwen3-Embedding-8B` | 4096 | 8B | — (HF only) | **16.0 GB** | **16.0 GB** >3 GB ⚠️ | **Combined** | **75.1** | ES+EN 100+ | Apache-2.0 | **EXCEPCIÓN >3 GB — MTEB #1 Jun 2026, Matryoshka, GPU** |

\* `bge-m3` fp32 ONNX 2.3 GB + HF 2.27 GB = 4.57 GB >3 GB total; pinned `model_int8.onnx` (1.20 GB) → total 3.47 GB.

\* MTEB Retrieval EN / multi, Jun 2026. Latency `e5-small` ONNX ~10 ms/512t CPU M1, `bge-m3` ~80 ms, `Qwen3` GPU.

**Balance:** 3 EN + 3 ES + 3 Combined — tantos para español como para inglés y combinados.

**One-model-per-namespace (regla de oro):** **un modelo por namespace** — misma `dim` para writes y query. Cross-model search rompe HNSW (dim mismatch → `DimensionMismatch` error). En benchmarks usar `namespace="bench-{model}"`. Ver bench `benchmarks/embed_bench.py` (ingest 1k EN+ES, `--skip-exception` vs `--include-exception`).

## Choosing a model

| Need | Pick | Why |
|------|------|-----|
| Offline default EN+ES, smallest that works | `multilingual-e5-small` **DEFAULT** | 384d, 691 MB total, EN+ES 16+ |
| Best EN only, tiny | `all-MiniLM-L6-v2` | 170 MB total |
| Best ES optimized | `jina-es-v2-base` | ES+EN specialist, 2.2 GB |
| Balanced multi 50+ langs, still 384d | `paraphrase-multilingual-MiniLM-L12-v2` | 941 MB |
| SOTA local ≤3 GB | `bge-m3` int8 | 1024d, 100+ langs, Matryoshka-capable |
| MTEB #1, GPU, Matryoshka 4096→1024 | `qwen3-embedding-8b` **EXCEPCIÓN** | 16 GB, `trust_remote_code=True` |

## Offline wiring (§3 consumers)

| Layer | With `embed-local` |
|-------|--------------------|
| `vanta-memory` L1 | `L1DedupConfig::with_local_provider(Arc<dyn EmbeddingProvider>)` uses `LocalOnnxProvider` instead of hash dim=8 |
| `src/physical_plan/vector.rs` | `VECTOR_SEARCH('hola mundo')` offline via `LocalOnnxProvider::embed(query_text)` (was no-op without `remote-inference`) |
| `vantadb-mcp` | `embed_texts { texts, model? } -> { embeddings: float[][] }` via `embed_batch` (budgeted 25k tokens) |
| Python/TS | Still BYO-vector; Rust `embed-local` via `benchmarks/embed_bench.py` validation |

## Verification

```bash
python -m py_compile embeddings/download.py && python embeddings/download.py --help  # --only
python embeddings/verify.py --check
cargo check -p vantadb --features embed-local
cargo test --features embed-local -- llm::tests::local_embed_multilingual  # 384, self>0.99, multi>0.60
python benchmarks/embed_bench.py --models multilingual-e5-small,bge-m3 --dataset tiny-en-es-1k --output benchmarks/embed_bench_report.json
```

See also `embeddings/README.md`, `docs/api/EMBEDDINGS.md`, `docs/operations/BENCHMARKS.md` §8, `embeddings/manifest.json`.

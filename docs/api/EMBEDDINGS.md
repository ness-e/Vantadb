---
title: Embeddings — BYO-Vector + embed-local
kind: reference
status: active
description: VantaDB is BYO-vector by default (“does not embed text for you”) and offers an opt-in local embedding path via the embed-local Cargo feature. No breaking changes — EmbeddingProvider is additive
tags: [vantadb, embeddings, api]
---

# Embeddings — BYO-Vector + `embed-local`

VantaDB is **BYO-vector by default** (“does not embed text for you”) and offers an **opt-in** local embedding path via the `embed-local` Cargo feature. No breaking changes — `EmbeddingProvider` is additive.

## Feature flag

```toml
[features]
embed-local = ["dep:ort", "dep:tokenizers"]  # ort 2.0 load-dynamic, no bundled onnxruntime.so
remote-inference = ["dep:reqwest"]           # Ollama / OpenAI (orthogonal to embed-local)
```

Enable:

```bash
cargo check -p vantadb --features embed-local
cargo test --features embed-local -- llm::tests::local_embed_multilingual
```

> `embed-local` is **Optional** (offline, local-first), not Experimental. See `docs/user/operations/EXPERIMENTAL_FEATURES.md`.

## Trait `EmbeddingProvider`

Source: `src/llm.rs:26`

```rust
pub trait EmbeddingProvider: Send + Sync {
    fn embed(&self, text: &str) -> Result<Vec<f32>>;
    fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        // default: loop over embed()
    }
}
```

| Provider | Feature | Env | Notes |
|---|---|---|---|
| `LocalOnnxProvider` | `embed-local` | `VANTADB_EMBEDDING_PROVIDER=local`, `VANTADB_LOCAL_MODEL=embeddings/models/<id>/onnx` | ONNX via `ort`+`tokenizers`, mean-pool + L2. In strict mode (default), fails closed if runtime or model weights are missing; dummy fallback requires `VANTADB_ALLOW_DUMMY_EMBEDDINGS=true` |
| `OllamaProvider` | `remote-inference` | `VANTADB_LLM_URL` (default `http://localhost:11434`), `VANTADB_LLM_MODEL` (default `all-minilm`) | `POST {VANTADB_LLM_URL}/api/embed` (Ollama's native embeddings endpoint — `src/llm.rs`, `OllamaProvider::embed`) |
| `OpenAIProvider` | `remote-inference` | `VANTADB_OPENAI_API_KEY` (required), `VANTADB_OPENAI_MODEL` (default `text-embedding-3-small`) | `POST https://api.openai.com/v1/embeddings` |

Factory `src/llm.rs:39` `get_embedding_provider()`:

```rust
// VANTADB_EMBEDDING_PROVIDER=local|multilingual-e5-small|ollama|openai
let provider = vantadb::llm::get_embedding_provider();
let vec = provider.embed("hola mundo")?;
let batch = provider.embed_batch(&["hola mundo".into(), "hello world".into()])?;
```

- The default `VANTADB_EMBEDDING_PROVIDER` is `ollama` in **every** build: `src/config.rs` reads the env var and falls back to the literal `"ollama"` unconditionally, so the `embed-local` feature does **not** change the default. To use the local ONNX path, set `VANTADB_EMBEDDING_PROVIDER=local` explicitly; the model directory then defaults to `embeddings/models/multilingual-e5-small/onnx` [384d] (768d for `embeddinggemma-2`, selectable via `vanta config set embeddings.model embeddinggemma-2`).
- Without `embed-local`, `local` is not a selectable provider — the only implementations are `ollama` and `openai` (`src/llm.rs`, `try_embedding_provider`).
- `LocalOnnxProvider::new("embeddings/models/<id>/onnx")` validates session loading and enforces strict embeddings when configured. Synthetic fallback is permitted only when the `strict_embeddings` config field is off — i.e. `VANTADB_STRICT_EMBEDDINGS=false`, or the legacy opt-out `VANTADB_ALLOW_DUMMY_EMBEDDINGS=true` (`src/config.rs`, `strict_embeddings` resolver). There is **no** `--allow-dummy-embeddings` CLI flag in any binary.
- **Dimension Isolation & Namespacing:** Embedding models with differing dimensions (e.g., 384d for `multilingual-e5-small` vs. 768d for `embeddinggemma-2`) cannot share a database: the vector index holds vectors of **one dimension**, and a `put` whose vector length differs from the index dimension is rejected with `Error::DimensionMismatch { expected, got }` (FIND-411). The rule is **one model per database**, not per namespace: a dedicated namespace is not enough, because the index is shared across namespaces. When switching models, use a dedicated database path.
- **Runtime Dylib Discovery:** `ort` loads the native ONNX runtime from `ORT_DYLIB_PATH`. When unset, VantaDB auto-probes local Python virtualenvs (`.venv/Lib/site-packages/onnxruntime/capi/onnxruntime.dll` / `.so` / `.dylib`) before falling back to system PATH.

## `LocalOnnxProvider` internals

`src/llm.rs` `pub struct LocalOnnxProvider { session: Option<Mutex<Session>>, tokenizer: Option<Tokenizer>, dim: usize, family: EmbedFamily, model_dir: String, allow_dummy: bool }`

- `dim` auto-detected from `embeddings/manifest.json` (by `model_dir` substring) → `config.json:hidden_size` → default `384`.
- `try_load_tokenizer` probes `model_dir/tokenizer.json`, `../tokenizer.json`, `../../tokenizer.json`.
- `try_load_session` calls `ort::init().commit()` once (`load-dynamic`) then probes `model.onnx`, `onnx/model.onnx`, `model_int8.onnx`, or any `*.onnx` recursively.
- `run_onnx(text)` tokenizes → `input_ids` + `attention_mask` `[1, seq]` → `Session::run` (dynamic input names) → `last_hidden_state [1, seq, dim]` → mean-pool with mask → L2 normalize. Returns `None` on any missing piece → `embed()` then either **fails closed** with `Error::Embedding` (strict mode, the default) or falls back to `deterministic_embed` (only when `strict_embeddings` is off).
- `deterministic_embed` is hash-seeded L2-normalized (ensures `cosine("hola mundo","hola mundo")>0.99` and multilingual `cosine("hola mundo","hello world")>0.60` even offline).

Methods:

```rust
impl LocalOnnxProvider {
    pub fn new(model_dir: &str) -> Result<Self>
    pub fn new_dummy(dim: usize) -> Self
}
impl EmbeddingProvider for LocalOnnxProvider {
    fn embed(&self, text: &str) -> Result<Vec<f32>>       // rejects empty text
    fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>
}
```

## `embeddings/` folder (Option B lazy download)

Repo stays light; `embeddings/models/` is gitignored (~26 GB for all 10 × ONNX+HF, per `embeddings/manifest.json`: the sum of `size_onnx_mb` + `size_hf_mb` over the 10 entries is 26 832 MB). Source of truth: `embeddings/manifest.json` (rev pinned).

```
embeddings/
├── README.md          # table, licences, one-liners
├── manifest.json      # 10 models, rev pinned, dim, onnx path, langs, licence, group
├── manifest.lock      # shas after first download (commitable, reproducible)
├── download.py        # huggingface_hub snapshot_download --only / --all / --check / --skip-exception
├── verify.py          # manifest structure + ONNX session/tokenizer smoke, writes verify.log
└── models/            # gitignored
```

One-liners:

```bash
python embeddings/download.py --only multilingual-e5-small
python embeddings/download.py --all --skip-exception   # CI-friendly (9 models — all but the qwen3 exception, ~10.6 GB)
python embeddings/download.py --check
python embeddings/verify.py --check
python -m py_compile embeddings/download.py && python embeddings/download.py --help
```

`.gitignore` delta:

```gitignore
/embeddings/models/
embeddings/models/**
/embeddings/*.onnx
/embeddings/*.safetensors
embeddings/**/*.bin
!embeddings/README.md
!embeddings/manifest.json
!embeddings/manifest.lock
!embeddings/download.py
!embeddings/verify.py
```

Env for local:

```bash
VANTADB_EMBEDDING_PROVIDER=local
VANTADB_LOCAL_MODEL=embeddings/models/multilingual-e5-small/onnx
```

> The legacy `VANTA_EMBEDDING_PROVIDER` / `VANTA_LOCAL_MODEL` names are **no longer
> read** — the `VANTADB_*` names above are the only ones the code consults (see
> `docs/user/operations/CONFIGURATION.md` § Environment Variables Outside `VantaConfig`).

## Model matrix — 10 models (8 ≤3 GB + 1 int8-pinned + 1 exception)

> Source: `embeddings/manifest.json` §4 plan. Total per model = ONNX+HF.

| # | id | Repo HF | Dim | ONNX | HF | Total | Group | Langs | Licence | Role |
|---|----|---------|-----|------|----|-------|-------|-------|---------|------|
| 1 | `bge-small-en-v1.5` | `BAAI/bge-small-en-v1.5` | 384 | 120 MB | 133 MB | 253 MB | EN | EN | MIT | baseline rápido |
| 2 | `all-MiniLM-L6-v2` | `sentence-transformers/all-MiniLM-L6-v2` | 384 | 80 MB | 90 MB | 170 MB | EN | EN | Apache-2.0 | ultra-ligero |
| 3 | `bge-base-en-v1.5` | `BAAI/bge-base-en-v1.5` | 768 | 440 MB | 438 MB | 878 MB | EN | EN | MIT | EN balance |
| 4 | `jina-es-v2-base` | `jinaai/jina-embeddings-v2-base-es` | 768 | 1100 MB | 1100 MB | 2.20 GB | ES | ES+EN | Apache-2.0 | ES optimizado |
| 5 | `paraphrase-multilingual-MiniLM-L12-v2` | `sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2` | 384 | 470 MB | 471 MB | 941 MB | ES | ES+EN 50+ | Apache-2.0 | ES multi ligero |
| 6 | `distiluse-multilingual` | `sentence-transformers/distiluse-base-multilingual-cased-v1` | 512 | 540 MB | 539 MB | 1.08 GB | ES | ES+EN 15+ | Apache-2.0 | ES multi base |
| 7 | **`multilingual-e5-small`** | `intfloat/multilingual-e5-small` | **384** | **220 MB** | 471 MB | **691 MB** | **combined** | **ES+EN 16+** | **MIT** | **DEFAULT** |
| 8 | `bge-m3` | `BAAI/bge-m3` | 1024 | 1.20 GB (int8) | 2.27 GB | 3.47 GB | combined | ES+EN 100+ | MIT | SOTA local, ONNX pinneado a int8 (3.47 GB > 3 GB) |
| 9 | `embeddinggemma-2` | `onnx-community/embeddinggemma-2-ONNX` | 768 | 550 MB | 600 MB | 1.15 GB | combined | ES+EN+code 100+ | Gemma | moderno 768d |
| 10 | `qwen3-embedding-8b` | `Qwen/Qwen3-Embedding-8B` | 4096 | — (HF only) | 16.0 GB | 16.0 GB | combined | ES+EN 100+ | Apache-2.0 | EXCEPCIÓN >3 GB — MTEB #1 75.1, GPU, Matryoshka |

- Balance: **3 EN + 3 ES + 4 Combined** — tantos para español como para inglés y combinados.
- `bge-m3` fp32 ONNX 2.3 GB + HF 2.27 GB = 4.57 GB >3 GB; se pinnea `model_int8.onnx` (1.20 GB), lo que deja el total en 3.47 GB.
- `qwen3-embedding-8b` sin ONNX (`onnx: null`), solo HF (`trust_remote_code=True`), GPU-only, Matryoshka 4096→1024. Incluir con `download.py --include-exception` (CI usa `--skip-exception`).

## One-model-per-namespace (regla de oro)

`docs/user/tutorials/05-embedding-integrations.md:126` — **un modelo por namespace**: misma `dim` para writes y query. Cross-model search rompe HNSW (dim mismatch → error). Usar `namespace="bench-{model}"` en benchmarks.

## Integrations

| Layer | Status with `embed-local` |
|-------|---------------------------|
| **Core HNSW** `src/index/` | consumes vectors; no auto-embed (BYO still works) |
| **Hybrid** `src/search/` | `query_vector` + `text_query` → RRF; dim must match docs |
| **SQL** `src/physical_plan/vector.rs` (`PhysicalVectorSearch`) | The IQL vector condition `<field> ~ "hola mundo", min = 0.88` now embeds offline via `LocalOnnxProvider` (was `remote-inference` only → empty). There is **no** `VECTOR_SEARCH(...)` function in the IQL grammar (`src/parser/grammar.rs`) |
| **vanta-memory L1** | `L1DedupConfig::with_local_provider(Arc<dyn EmbeddingProvider>)` uses `LocalOnnxProvider` instead of hash dim=8 |
| **MCP** `vantadb-mcp` | `embed_texts { texts: string[], model?: string } -> { embeddings: float[][] }` reuses `embed_batch` |
| **Python/TS SDK** | BYO-vector remains; `embed-local` is Rust-side (Python can call via `maturin` + bench `benchmarks/embed_bench.py`) |

## Verification (contract §7)

```bash
python -m py_compile embeddings/download.py && python embeddings/download.py --help  # --only
cargo check -p vantadb --features embed-local
cargo test --features embed-local -- llm::tests::local_embed_multilingual  # 384, self>0.99, multi>0.60
python embeddings/verify.py --check         # structural check (10 models, dims, rev pinned, en≥3/es≥3/combined≥3) + ONNX session/tokenizer smoke
python embeddings/download.py --only multilingual-e5-small --dry-run
python benchmarks/embed_bench.py --models multilingual-e5-small,bge-m3 --dataset tiny-en-es-1k
grep -r "embed-local" docs/   # ≥3 hits (this file + QUICKSTART + tutorial + EXPERIMENTAL_FEATURES)
```

See also `embeddings/README.md`, `docs/user/tutorials/05-embedding-integrations.md`, `docs/user/operations/BENCHMARKS.md` §8, `benchmarks/embed_bench.py`.

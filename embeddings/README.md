# embeddings/ — Modelos locales VantaDB (ONNX + HF)

> **Opción B descarga lazy** — repo liviano, `embeddings/models/` gitignored (~22 GB si los 9 × ONNX+HF). Mantener BYO-vector; `embed-local` es opt-in.

## Quick start (1-liner)

```bash
# default multilingual (384d, 220 MB ONNX, 691 MB total) — EN+ES 16+ idiomas
python embeddings/download.py --only multilingual-e5-small

# 3 modelos recomendados (multi + SOTA int8)
python embeddings/download.py --only multilingual-e5-small,bge-m3,paraphrase-multilingual-MiniLM-L12-v2

# todos salvo excepción >3GB (CI-friendly)
python embeddings/download.py --all --skip-exception

# verificar sin red
python embeddings/download.py --check
python embeddings/verify.py --check
```

## Modelos (9) — source of truth: `manifest.json` (rev pinned)

| # | id | Repo HF | Dim | ONNX | HF | Total | Grupo | Idiomas | Licencia | Rol |
|---|----|---------|-----|------|----|-------|-------|---------|----------|-----|
| 1 | `bge-small-en-v1.5` | `BAAI/bge-small-en-v1.5` | 384 | 120 MB | 133 MB | 253 MB | EN | EN | MIT | baseline rápido |
| 2 | `all-MiniLM-L6-v2` | `sentence-transformers/all-MiniLM-L6-v2` | 384 | 80 MB | 90 MB | 170 MB | EN | EN | Apache-2.0 | ultra-ligero |
| 3 | `bge-base-en-v1.5` | `BAAI/bge-base-en-v1.5` | 768 | 440 MB | 438 MB | 878 MB | EN | EN | MIT | EN balance |
| 4 | `jina-es-v2-base` | `jinaai/jina-embeddings-v2-base-es` | 768 | 1100 MB | 1100 MB | 2.20 GB | ES | ES+EN | Apache-2.0 | ES optimizado |
| 5 | `paraphrase-multilingual-MiniLM-L12-v2` | `sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2` | 384 | 470 MB | 471 MB | 941 MB | ES | ES+EN 50+ | Apache-2.0 | ES multi ligero |
| 6 | `distiluse-multilingual` | `sentence-transformers/distiluse-base-multilingual-cased-v1` | 512 | 540 MB | 539 MB | 1.08 GB | ES | ES+EN 15+ | Apache-2.0 | ES multi base |
| 7 | **`multilingual-e5-small`** | `intfloat/multilingual-e5-small` | **384** | **220 MB** | 471 MB | **691 MB** | **combined** | **ES+EN 16+** | **MIT** | **DEFAULT** |
| 8 | `bge-m3` | `BAAI/bge-m3` | 1024 | 1.20 GB (int8) | 2.27 GB | 3.47 GB | combined | ES+EN 100+ | MIT | SOTA local ≤3 GB int8 |
| 9 | `qwen3-embedding-8b` | `Qwen/Qwen3-Embedding-8B` | 4096 | — | 16.0 GB | 16.0 GB | combined | ES+EN 100+ | Apache-2.0 | **EXCEPCIÓN >3 GB — MTEB #1 75.1, GPU, Matryoshka** |

- **Balance:** 3 EN + 3 ES + 3 Combined — tantos para español como para inglés y combinados.
- **Formato:** ONNX (`onnx/model.onnx` o `model_int8.onnx` para bge-m3) + HF pytorch (`*.safetensors`). Comparar Rust `ort` vs Python `sentence-transformers`.
- **Regla de oro:** un modelo por namespace (misma dim para writes y query; cross-model rompe HNSW). Ver `docs/tutorials/05-embedding-integrations.md:126`.
- **bge-m3 int8:** fp32 ONNX 2.3 GB + HF 2.27 GB = 4.57 GB >3 GB; por eso se pinnea `model_int8.onnx` (1.20 GB).
- **Qwen3:** sin ONNX oficial; solo HF (`trust_remote_code=True`), `onnx=null`, GPU-only, Matryoshka 4096→1024.

## Comandos

```bash
python -m py_compile embeddings/download.py   # contrato EMB-01
python -m py_compile embeddings/verify.py
python embeddings/download.py --help           # muestra --only
python embeddings/download.py --check          # valida manifest sin red
python embeddings/verify.py --check            # valida dims/rev sin modelos

# descarga real (requiere huggingface_hub)
pip install huggingface_hub
python embeddings/download.py --only multilingual-e5-small

# verificación ONNX (requiere ort + tokenizers)
pip install onnxruntime tokenizers
python embeddings/verify.py --only multilingual-e5-small
```

## Estructura

```
embeddings/
├── README.md          # esta tabla + one-liners
├── manifest.json      # source-of-truth (rev pinned)
├── manifest.lock      # shas fijados tras primera descarga (commitable)
├── download.py        # huggingface_hub snapshot_download (lazy)
├── verify.py          # ort+tokenizers dim+cosine checks
└── models/            # gitignored — creado por download.py
```

## Licencias

MIT: bge-*, multilingual-e5-small, bge-m3. Apache-2.0: all-MiniLM, jina-es, paraphrase, distiluse, Qwen3.

## Excepción >3 GB — Qwen3-Embedding-8B

16 GB, GPU-only, MTEB v2 #1 Jun 2026 (75.1), Matryoshka 4096→1024. Solo HF; `download.py --include-exception` para incluirlo. En CI usar `--skip-exception`.

## Env vars (para EMB-02)

`VANTA_EMBEDDING_PROVIDER=local` y `VANTA_LOCAL_MODEL=embeddings/models/multilingual-e5-small/onnx` (cuando `embed-local` feature esté disponible).

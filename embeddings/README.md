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
- **Descarga recortada (FIND-71):** `download.py` usa `ALLOW_PATTERNS = ["*.json", "*.txt", "tokenizer*", "onnx/*", "*.safetensors"]` — sin `*.bin` global (duplicaba pesos cuando el repo trae ambos formatos → 3-4× lo declarado). Si un modelo resulta bin-only, añadir entrada explícita en `MODEL_PATTERNS` por id (`get_allow_patterns()`), nunca re-ampliar el global.
- **Sizes re-medidos 2026-09-15 desde `manifest.json` (`size_onnx_mb + size_hf_mb`):** 253 / 170 / 878 / 2200 / 941 / 1079 / 691 / 3470 / 16000 MB — la tabla coincide con el manifest (fuente, Regla 11).
- **Regla de oro:** un modelo por namespace (misma dim para writes y query; cross-model rompe HNSW). Ver `docs/tutorials/05-embedding-integrations.md:126`.
- **bge-m3 int8:** fp32 ONNX 2.3 GB + HF 2.27 GB = 4.57 GB >3 GB; por eso se pinnea `model_int8.onnx` (1.20 GB).
- **Qwen3:** sin ONNX oficial; solo HF (`trust_remote_code=True`), `onnx=null`, GPU-only, Matryoshka 4096→1024.

## Comandos

```bash
python -m py_compile embeddings/download.py   # contrato EMB-01
python -m py_compile embeddings/verify.py
python embeddings/download.py --help           # muestra --only
python embeddings/download.py --check          # valida manifest sin red (global o --only <id> por modelo + lock repo/rev)
python embeddings/verify.py --check            # SMOKE: valida dims/rev sin modelos (no verificacion numerica)

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
├── download.py        # huggingface_hub snapshot_download (lazy, patterns recortados + check por modelo)
├── verify.py          # SMOKE ort+tokenizers (abre sesion, hints cosine; numerico real en sanity_embed.py)
└── models/            # gitignored — creado por download.py
```

## Licencias

MIT: bge-*, multilingual-e5-small, bge-m3. Apache-2.0: all-MiniLM, jina-es, paraphrase, distiluse, Qwen3.

## Excepción >3 GB — Qwen3-Embedding-8B

> **Solo HF — 16 GB VRAM mínimo — GPU-only — `onnx: null`**

- **Modelo:** `Qwen/Qwen3-Embedding-8B` — 8B params, dim **4096** (Matryoshka 4096→1024), MTEB v2 #1 Jun 2026 (**75.1**), EN+ES 100+ idiomas, Apache-2.0. Balance 3/3/3: 3 Combined incluye esta excepción.
- **Formato:** solo HF pytorch (`*.safetensors`, `size_hf_mb: 16000`), **sin ONNX** (`onnx: null`, `size_onnx_mb: null`) — ONNX no recomendado / no oficial. Requiere `trust_remote_code=True` en `sentence_transformers` / `transformers` (Qwen3 usa código remoto custom).
- **Requisitos:** GPU con ≥16 GB VRAM para inferencia. No apto para `ort` CPU; solo `sentence_transformers.SentenceTransformer(..., trust_remote_code=True)`.
- **Descarga:** `python embeddings/download.py --include-exception` (o `--only qwen3-embedding-8b --include-exception`). Por defecto `--all` incluye la excepción; para CI liviano usar `--skip-exception` o `--all --skip-exception` (8 modelos ≤3 GB). Equivalente: `python embeddings/download.py --only qwen3-embedding-8b` (16 GB, requiere red + HF token si repo gated).
- **Verificación:** `python embeddings/verify.py --only qwen3-embedding-8b` — HF-only check: `dim==4096`, `cosine("hola mundo","hello world")>0.70` (multi), `onnx=null`, `exception: ">3GB"`. Sin ONNX vs HF cosine (HF-only). Ver `verify.log` con `HF-only`.
- **Bench:** solo con `python benchmarks/embed_bench.py --include-exception` (o `--models qwen3-embedding-8b --include-exception`). Por defecto bench omite Qwen3 (`--skip-exception` implícito si `--models all` sin flag) para no OOM en CI. Con flag: 9 modelos.
- **Matryoshka:** embeddings 4096d pueden truncarse a 1024/2048 sin re-entrenar (Matryoshka Representation Learning) — útil para RRF híbrido con dims menores. Ver `docs/api/EMBEDDINGS.md` y `benchmarks/embed_bench.py` multi cosine.

## Env vars — providers (verificado EMB-19, fuentes: `src/config.rs:203-209,936-957`, `src/llm.rs:17,58-93`, `vanta-mcp-local.ps1:1-99`)

Tres proveedores, selección solo por env (sin recompilar):

| Proveedor | Activación | Requisito | Estado verificado (EMB-19 `a5d549af`) |
|-----------|------------|-----------|----------------------------------------|
| `local` (ONNX, default recomendado) | `VANTADB_EMBEDDING_PROVIDER=local` + `VANTADB_LOCAL_MODEL=<absoluta>/embeddings/models/<id>/onnx` + `ORT_DYLIB_PATH=<onnxruntime>=1.27.dll` — o lanzador `.\vanta-mcp-local.ps1 -DbPath <db>` (setea ambas + autodetecta ORT) | Modelo en disco (`python embeddings/download.py --only <id>`) + onnxruntime ≥1.27 (`%LOCALAPPDATA%/VantaDB/onnxruntime/onnxruntime.dll`, EMB-11; System32 1.17.1 aborta — FIND-100) | REAL ejecutado: `multilingual-e5-small` dim 384 `fallback:false`, `s(par)=0.9158` vs `0.8427/0.8423` gap `0.0732` (fuente: `docs/tasks/EMB-19.md` Step 1) |
| `ollama` (default del core si no se setea nada) | `VANTADB_EMBEDDING_PROVIDER=ollama` (lee servidor `localhost:11434`) | Servidor Ollama vivo + modelo con embeddings descargado (`ollama pull nomic-embed-text`) | DEGRADACIÓN AVISADA ejecutada: servidor v0.33.2 vivo pero `/api/tags` → `{"models":[]}` → `fallback:true` + `warning` con `404`, exit 0 sin crash (fuente: `docs/tasks/EMB-19.md` Step 2). Embedding real: documentado-no-ejecutado (precedente FIND-69) |
| `openai` | `VANTADB_EMBEDDING_PROVIDER=openai` + `VANTADB_OPENAI_API_KEY=<key>` (solo env de sesión, NUNCA a disco ni al script) + opcional `VANTADB_OPENAI_MODEL` | Key válida con crédito | DEGRADACIÓN AVISADA ejecutada (sin key: `fallback:true` + `warning="VANTADB_OPENAI_API_KEY must be set"`, fuente: `docs/tasks/EMB-19.md` Step 2). Embedding real: documentado-no-ejecutado sin key (precedente FIND-69) |

- Default del core sin env: `ollama` (fuente: `src/config.rs:956`). El lanzador fija `local` explícito para no depender del default.
- Path del modelo: usar ABSOLUTA (el default relativo es frágil por CWD; fuente: plan §Verificación + EMB-13 nota CWD).
- Secrets: keys solo env de sesión (contrato EMB-11/12; review lo audita).

## Cuándo usar cada modelo (guía operativa — dims/idiomas/tamaños de `manifest.json`)

| # | id | Cuándo usar |
|---|----|-------------|
| 1 | `bge-small-en-v1.5` (384d, 120+133=253 MB) | Baseline rápido solo-inglés, CPU modesta; alternativa ligera al default cuando no hay español. |
| 2 | `all-MiniLM-L6-v2` (384d, 80+90=170 MB) | Ultra-ligero solo-inglés (el más chico); CI/smoke y switching por nombre (`model` param EMB-17). Sin prefijos e5 (familia MiniLM). |
| 3 | `bge-base-en-v1.5` (768d, 440+438=878 MB) | Inglés con más calidad que small, si la base es 768d desde el inicio. Requiere descarga (`--only bge-base-en-v1.5`, no está en los 3 en disco). |
| 4 | `jina-es-v2-base` (768d, 1.10+1.10=2.20 GB) | Español optimizado 768d; solo si la base nace 768d y hay disco/RAM para 2.2 GB. |
| 5 | `paraphrase-multilingual-MiniLM-L12-v2` (384d, 470+471=941 MB) | Multilingüe ligero 384d compatible en dim con el default (cambio sin re-ingerir si la base ya es 384d). En disco (verificado EMB-19). |
| 6 | `distiluse-multilingual` (512d, 540+539=1.08 GB) | Multilingüe base 512d; base nueva 512d o nada (no mezclar con 384d). |
| 7 | **`multilingual-e5-small` (384d, 220+471=691 MB) DEFAULT** | Default: ES+EN 16+ idiomas, dim 384, e5 con prefijos `query:/passage:` (EMB-16, margen asimétrico 0.1204 vs simétrico 0.0812, fuente: `docs/tasks/EMB-16.md`). En disco. Primera opción salvo razón explícita. |
| 8 | `bge-m3` (1024d, int8 1.20+2.27=3.47 GB) | SOTA local ≤3 GB (int8 `model_int8.onnx`); base nueva 1024d, disco ≥4 GB. Multilingüe 100+. |
| 9 | `qwen3-embedding-8b` (4096d, 16.0 GB HF-only, `onnx:null`) | EXCEPCIÓN GPU-only (≥16 GB VRAM, `trust_remote_code=True`, Matryoshka 4096→1024). Fuera de `ort` CPU. Solo `--include-exception`. |

- En disco verificado (EMB-19): `all-MiniLM-L6-v2`, `multilingual-e5-small`, `paraphrase-multilingual-MiniLM-L12-v2` (los 3 × 384d). Resto: bajo demanda con `python embeddings/download.py --only <id>` (aviso Q2: tamaño/tiempo antes de descargar).
- Cambiar de modelo con distinta dim exige base nueva o re-ingerir (ver regla abajo). Misma dim (384d: 1/2/5/7) = switch sin re-ingerir.

## Regla una-dim-por-base (Q4: bloquear+guiar — fuente: `docs/tasks/EMB-18.md`, verificado EMB-19)

Una base = una dimensión. Si la base contiene vectores de dim distinta al vector entrante (modelo cambiado), el put/search es RECHAZADO con error que dice dim esperada + obtenida + comando exacto de regeneración. Nunca auto-reindex silencioso.

- Base vacía: sin gate (la primera escritura con vector define la dim). Puts solo-texto (sin vector): nunca gateados.
- Guía honesta del error: re-embeder con la dim original O re-ingerir todo con el modelo nuevo + `rebuild_index` (tool MCP) / `reindex_hnsw_from_text(ns, page_size)` (SDK). `rebuild_index` solo NO cambia dims (reconstruye desde vectores almacenados).
- Ver `docs/tutorials/05-embedding-integrations.md:126` (regla de oro citada) y `docs/tasks/EMB-18.md` (spec de decisiones).

## Nota `embed_texts` real (verificado EMB-19 `a5d549af`, fuentes: `docs/tasks/EMB-19.md` Steps 1-2 + `docs/tasks/EMB-13.md`)

`embed_texts` ya no es dummy: con modelo real devuelve vectores con señal semántica; sin modelo devuelve fallback determinista AVISADO (nunca silencioso, nunca error duro — Q5).

- Respuesta: `{embeddings, model, dim, dimensions, count, fallback, warning?, truncated, next_cursor}`. `fallback:false` = vectores reales; `fallback:true` + `warning` = hash sin semántica (revisar env/modelo/ORT).
- Medición EMB-19 (binario final rebuild, `VANTADB_EMBEDDING_PROVIDER=local`, modelo `multilingual-e5-small`): `s(D0-D1 par)=0.9158` vs `s(D0-D2)=0.8427` vs `s(D0-D3)=0.8423`, gap `0.0732`, `dim=384`, `fallback:false`, `model=multilingual-e5-small` (precedentes: EMB-13 `0.9282 vs 0.8427/0.8366`; EMB-10 `pares≥0.91 vs impares≤0.78`).
- `model` param (EMB-17): `model:"<id del manifest>"` usa ESE modelo (caché por modelo, tope `MAX_CACHED_LOCAL_MODELS=2` + evicción avisada). Id desconocido → `invalid_params` con lista válida (9 ids). Id conocido sin archivos → error con `python embeddings/download.py --only <id>` + tamaño. `model:null` = proveedor activo por env (comportamiento EMB-13 intacto).
- Budgeting intacto: 128 items / 25k tokens + paginación `cursor`/`next_cursor` (fuente: `vantadb-mcp/src/config.rs:75-78,111-112`).
- Auto-embed (EMB-14): `memory_put`/`put_batch` sin vector guarda CON vector del proveedor activo (`memory_get` lo muestra, len == dim); vector provisto se respeta; fallo de proveedor → guarda sin vector + aviso.
- Query mismo-proveedor (EMB-15): `search_memory`/`memory_recall` con texto embebe la query con el proveedor activo; sinónimos verificados `keys=["d1","d0","d2"]` + recall `hybrid` con D0 (fuente: EMB-19 Step 1, idéntico a EMB-15).
- Prefijos e5 (EMB-16): familia e5 usa `query:` (queries) / `passage:` (documentos); MiniLM y resto ninguno. Margen medido: asimétrico `0.1204` vs simétrico `0.0812` (+48% relativo; fuente: `docs/tasks/EMB-16.md`).
- e2e puesto en EMB-19: `memory_put` (d0,d1,d2 sin vector) → `memory_get d0` con `vector` len 384; `search_memory {text_query:"felino descansando"}` (0 palabras comunes con D0) → `keys=["d1","d0","d2"]`; focado recall `mode="hybrid" recalled=[felino…, gato…, cuántica…]` 4/4.

---
title: "Deep audit W7 - Benchmarks and market research"
kind: review
status: active
description: "Auditoria de los 23 benches Rust + 21 ficheros en benchmarks/ + evals + datasets, y research de mercado con URLs: que benchmarkea hoy el sector, que publica cada competidor y que benchmarks unicos deberia construir VantaDB."
tags: [audit, reviews, benchmarks]
---

# Deep audit W7 - Benchmarks + research de mercado

> **Wave:** W7 · **Fecha:** 2026-10-20 · **Modo:** read-only (research web con URLs, sin inventar)
> **Especificacion:** `docs/dev/plans/2026-10-20-revision-profunda-integral-vantadb.md` §6 (W7)
> **Ejecucion:** subagentes `vanta-tuner` (benchmarks locales) + `vanta-research` (web) no disponibles
> (sin tool `task`). El orquestador ejecuto ambos sub-scope.

## Scope

**Cubierto:** `benches/` (23 `.rs` + `benches/common/`), `benchmarks/` (21 ficheros), `evals/` (10),
`datasets/` (SIFT1M, GloVe-100-angular, longmemeval), `scripts/{bench_regression.py,north_star_metric.py,pre_bench_check.ps1,download_benchmark_datasets.*,verify_datasets.*}`,
`docs/user/operations/BENCHMARKS.md` (baseline canonico), `docs/user/benchmarks/` (6 ficheros).

**Excluido con motivo:** `datasets/*.hdf5` (artefactos binarios; se verifico su presencia y checksum
de verificacion, no su contenido).

## Metodologia

- Inventario de los 23 benches Rust y los 21 ficheros de `benchmarks/`, con su invocabilidad.
- Trazado de que benches se invocan en que workflow (`.github/workflows/*.yml`).
- Verificacion del estado de versionado de cada artefacto de resultado (`git ls-files` vs `git check-ignore`).
- Lectura del baseline canonico en `docs/user/operations/BENCHMARKS.md` (entorno, comando, fecha).
- **Research web** con el router en cascada de la skill `coordinated-web-search`. El tool `websearch`
  nativo devolvio HTTP 403; se escalo al nivel 1 de la cascada via MCP `websearch.search` (motor
  DuckDuckGo, keyless) y se leyeron las paginas fuente con `websearch.fetchWebContent`. Todas las
  URLs de este reporte fueron visitadas el **2026-10-20**.

## Hallazgos

| ID | Severidad | Hallazgo | Archivo:línea | Evidencia | Fix propuesto | Esfuerzo |
|---|---|---|---|---|---|---|
| `FIND-393` | Medium | **Los resultados de recall (BEIR) y los de north-star/memoria no tienen artefacto versionado** | `benchmarks/beir_recall_report.json`, `evals/memory_bench_report.json`, `evals/memory_harness_report.json`, `evals/memory_harness_report_full500.json` | `git check-ignore -v` los marca como **ignorados**; `git ls-files` no los lista. El unico resultado de benchmark **versionado** en todo el arbol es `benchmarks/criterion_baseline.json` (tracked) + `benchmarks/python_baseline.json` (tracked) + `benchmarks/wasm_threads_results*.json` (tracked) + `embeddings/sanity_report.json` (tracked). Los que faltan son precisamente los de **calidad de recuperacion**: BEIR recall y las metricas de north-star. Consecuencia (Regla 11): las cifras de recall/nDCG que aparezcan en docs no son reproducibles por un tercero porque el artefacto que las respalda no existe en el repo, y el comando que las genera (`python benchmarks/beir_recall_bench.py`) necesita red + modelos + datasets. | Versionar un subconjunto **pequeno y determinista** del reporte BEIR (por ejemplo el SciFact ya medido, ver `FIND-315`) y el resumen de north-star, con la convencion `<!-- BENCHMARK_METRICS_START/END -->` que ya usa `BENCHMARKS.md`. Dejar el reporte completo en gitignore. | 4-8h |
| `FIND-394` | Medium | **`canonical_p99` -el benchmark canonico de la Regla 9- nunca se mide en el gate de PR: solo se compila** | `.github/workflows/nightly.yml:330`, `.github/workflows/nightly.yml:417`, `.github/workflows/ci-rust.yml` | `nightly.yml:330` = `cargo bench -p vantadb --bench canonical_p99 --no-run` (**compile-gate**, no mide). `nightly.yml:417` = `cargo bench -p vantadb --bench canonical_p99 -- --nocapture` (medicion real, pero en el **nightly** = Heavy tier, no en el gate de PR). En `ci-rust.yml` (Fast Gate) **no hay ninguna invocacion de `cargo bench`**. La linea 378 de `BENCHMARKS.md` documenta el baseline: "Measured timed run 2026-09-25: **p50 2.2388 / p95 4.248 / p99 4.9987 ms** ... machine-readable baseline `benchmarks/criterion_baseline.json`". Es decir: el baseline tiene **~2 meses** y **nadie lo re-mide antes de mergear**. La Regla 9 exige before/after contra este baseline, pero un PR que rompa el hot path no produce ninguna señal hasta el nightly. | Anadir al Fast Gate un step que ejecute `canonical_p99` con un **budget** (no un `--no-run`): comparar el p99 del run contra `benchmarks/criterion_baseline.json` con un margen (p.ej. +15 %) y fallar si se excede. El harness `scripts/bench_regression.py` ya existe: revisar si puede correr en modo comparativo sobre el baseline versionado. | 1d |
| `FIND-395` | Medium | **13 de los 23 benches Rust no se ejecutan en ningun workflow** | `benches/acorn_filtered_search.rs`, `benches/backend_compare.rs`, `benches/batch_existing_check.rs`, `benches/crash_recovery.rs`, `benches/hnsw_recall_ef.rs`, `benches/list_window.rs`, `benches/param_sweep.rs`, `benches/purge_expired.rs`, `benches/scores_semantics.rs`, `benches/sparse_hot_path.rs`, `benches/tokenizer_bench.rs`, `benches/vfile_search.rs`, `benches/wal_throughput.rs` | `.github/workflows/certification.yml` invoca 10 benches: `hnsw_pure` (403), `hybrid_queries` (406), `stress_test` (409), `bench_concurrent` (412), `ingestion_concurrent` (419), `canonical_p99` (422), `memory_budget` (425), `incremental_bench` (428), `ivf_bench` (431), `high_density` (463). `nightly.yml` solo añade `canonical_p99`. Los 13 restantes no aparecen en ningun workflow. `docs/user/operations/BENCHMARKS.md:272` documenta `cargo bench -p vantadb --bench scores_semantics` y `:411` documenta `memory_budget` con `MEMORY_BUDGET_SCALE=lite` - o sea, **estos benches son la instrumentacion de caracteristicas documentadas** (semantica de scores, presupuesto de memoria, busqueda dispersa, TTL/purge, WAL throughput) y son los que nadie mide. `wal_throughput` en particular es el unico bench que mediria la耐久 de la WAL, que es el producto de VantaDB. | Clasificar los 13 en: (a) mover a Heavy si son caros, (b) anadir al Fast Gate si son <2 min y relevant, (c) **borrar/documentar como no mantenidos** si son redundantes con otro. `param_sweep` y `backend_compare` son candidatos a Heavy; `scores_semantics`, `sparse_hot_path`, `purge_expired`, `wal_throughput` a Fast Gate. | 1-2d |
| `W7-04` | Info | El baseline canonico esta correctamente documentado y con artefacto versionado (Regla 11 cumplida) | `docs/user/operations/BENCHMARKS.md:380`, `benchmarks/criterion_baseline.json` | `BENCHMARKS.md:380`: "**Baseline canonico (Regla 9):** `benches/canonical_p99.rs` - 100k vectors x 1536d, seed 42, insert + search p50/p95/p99. Measured timed run 2026-09-25: **p50 2.2388 / p95 4.248 / p99 4.9987 ms** ... machine-readable baseline `benchmarks/criterion_baseline.json`". `BENCHMARKS.md:355-365` publica el entorno (Windows 11 NT 10.0.26200, i5-1235U 10c/12t, 31.78 GB, dataset seed 42, `VEC_SEED=42` hardcoded). `benchmarks/criterion_baseline.json` esta **trackeado** en git. La §2 (SDK Python) publica entorno, comando y fecha, y declara explicitamente el dataset **unseeded** y por que se excluyo la fila BM25 (degenerate). Este es el nivel de rigor que el resto del proyecto deberia alcanzar. | Sin accion. Es el modelo a replicar en el resto (ver FIND-393). | - |
| `W7-05` | Info | 12 datasets/artefactos de benchmark presentes con verificacion reproducible | `datasets/sift/{sift_base.fvecs,sift_query.fvecs,sift_groundtruth.ivecs}`, `datasets/glove-100-angular.hdf5`, `datasets/sift-128-euclidean.hdf5`, `datasets/longmemeval/longmemeval_s_cleaned.json`, `scripts/download_benchmark_datasets.{ps1,sh}`, `scripts/verify_datasets.{ps1,sh}` | Hay un gate de pre-test (`verify_datasets.*`) que se inyecta como pre-step en el workflow de certificacion pesada, con contrato identico bash/PowerShell. `datasets/longmemeval/` esta descargado (dataset de memoria de agente). `datasets/sift/` tiene los tres ficheros fvecs/ivecs. La obtencion es reproducible por comando. | Sin accion. | - |
| `W7-06` | Info | El north-star de memoria existe pero su reporte no es versionado | `evals/northstar.mjs`, `scripts/north_star_metric.py`, `evals/memory_harness.py` | Hay 3 artefactos de north-star: el evaluador JS (`evals/northstar.mjs`), el script de metrica (`scripts/north_star_metric.py`) y el harness de memoria (`evals/memory_harness.py` con `memory_harness_report.json` y `memory_harness_report_full500.json`, ambos gitignored). El "north star" del proyecto es precisamente "¿el agente recuerda?" y **no hay numero publicado ni artefacto**. | Ver FIND-393. Publicar el north-star como tabla versionada con su comando. | 4h |
| `W7-07` | Info | `benchmarks/update_markdown.py` genera markdown automaticamente: verificar que no escribe en `docs/` | `benchmarks/update_markdown.py` | Es un entry point del grafo (`get_architecture` -> entry_points). `benchmarks/vantadb_local_bench.py` escribe `benchmarks/vanta_benchmark_report.json` (gitignored) y `benchmarks/update_markdown.py` presumably inyecta las tablas en `BENCHMARKS.md` entre los markers `<!-- BENCHMARK_METRICS_START -->` / `<!-- BENCHMARK_METRICS_END -->` que se observaron en `BENCHMARKS.md:53-58`. Si escribe en `docs/`, el gate de `check-docs.mjs` y el `git diff` de un PR son la unica red. | Confirmar el destino de escritura en una revision del script. | 30min |

### Research de mercado (con URLs, consultadas 2026-10-20)

**A. Benchmarks que usa hoy el sector**

| Benchmark | Que mide | Fuente |
|---|---|---|
| `ann-benchmarks` | ANN competitive: recall vs latency/throughput, 30+ algoritmos en el mismo hardware | <https://ann-benchmarks.com/index.html> · <https://github.com/erikbern/ann-benchmarks> |
| BIGANN / Deep1B | ANN a escala de miles de millones (SIFT, SimSearchNet++, Deep) | <https://big-ann-benchmarks.com/neurips21.html> · <https://github.com/harsha-simhadri/big-ann-benchmarks> |
| MTEB (v2) | Text embeddings, 8 categorias, 112+ idiomas, mean task score | <https://huggingface.co/spaces/mteb/leaderboard> · <https://leaderboard.mteb.org/benchmarks> |
| BEIR | Recuperacion heterogenea, nDCG@10, 18 dominios | <https://github.com/beir-cellar/beir> |
| RULER | Long-context efectivo (13 tareas, 4 categorias) en vez del NIAH trivial | <https://github.com/NVIDIA/RULER> · <https://arxiv.org/abs/2404.06654> |
| GraphRAG-Bench | Pipeline GraphRAG completo: construccion + recuperacion + generacion, 16 disciplinas | <https://graphrag-bench.github.io/> · <https://arxiv.org/pdf/2506.02404> · <https://github.com/GraphRAG-Bench/GraphRAG-Benchmark> |
| GraphRAG de Microsoft (datasets) | HotpotQA multi-hop para GraphRAG | <https://github.com/microsoft/graphrag-benchmarking-datasets> |
| LongMemEval / LoCoMo / BEAM | Memoria conversacional de largo plazo | <https://mem0.ai/research> · <https://agentmemoryleaderboard.ai/> |
| Revision empirica de 7 vector DBs | FAISS, Qdrant, Milvus, Weaviate, Chroma, pgvector, LanceDB sobre 6 datasets, >4M vectores | <https://arxiv.org/abs/2608.12812> |

**B. Que publica cada competidor (y con que rigor)**

| Competidor | Publica | Hardware | Reproducible |
|---|---|---|---|
| Qdrant | Pagina de benchmarks **comparativos** con la razon explicita: "We do comparative benchmarks, which means we focus on relative numbers rather than absolute numbers. We use affordable hardware, so that you can reproduce the results easily" | Razonablemente specified | Parcial - el codigo del harness es publico, el dataset tambien |
| Milvus / Knowhere | Aparece en `ann-benchmarks` (comparacion con el resto de algoritmos) | El del ann-benchmarks | Si |
| Weaviate | Aparece en `ann-benchmarks` (`weaviate`) | El del ann-benchmarks | Si |
| Chroma | Aparece en la revision academica <https://arxiv.org/abs/2608.12812> y en wrappers Python nativos | Variable | Parcial |
| LanceDB | Idem revision academica; ademas el repo de VantaDB ya lo midio (ver `docs/user/COMPARISON.md:109`) | Variable | Parcial |
| Mem0 | Paper arXiv con tabla LOCOMO contra 6 categorias de baseline; pagina de research con 92.5 LoCoMo / 94.4 LongMemEval | No declarado en el abstract | **No** - numeros de marketing |
| Zep / Graphiti | Publica scores comparables en leaderboards externos | No declarado | No |
| Letta / LangMem | Publican via BEAM y similares | No declarado | No |

**Lectura de mercado (lo que nadie mide)**: la busqueda de "coste total" (build time + RSS + p99 con
recall) **no aparece** en ningun competidor publicado. Qdrant publica throughput/recall relativos;
Mem0/Zep publican accuracy de memoria sin coste. **La combinacion recall + p99 + RSS + tiempo de build
+ coste por ingestión no la publica nadie**, y VantaDB tiene las piezas para medirla.

**C. EmbeddingGemma 2 - verificado contra la fuente oficial (2026-10-20)**

| Atributo | Valor verificado | Fuente |
|---|---|---|
| Existe la v2 | **Si** - `google/embeddinggemma-2` | <https://huggingface.co/google/embeddinggemma-2> |
| Dim de salida | **768** (MRL: 128/256/512) | <https://gemma.ai/models/embeddinggemma/model-card-v2/> |
| Parametros | 740M (270M texto + 170M vision + 300M audio) | idem |
| **Licencia** | **Apache 2.0** (no "Gemma") | idem + <https://huggingface.co/onnx-community/embeddinggemma-2-ONNX> |
| ONNX | Si, via `onnx-community/embeddinggemma-2-ONNX` | <https://huggingface.co/onnx-community/embeddinggemma-2-ONNX> |
| Multilingue | 100+ idiomas | model card |
| MTEB multilingue v2 | **61.36** (vs 61.15 de EmbeddingGemma 1) | model card |
| MTEB codigo v1 (nDCG@10) | **78.68** (vs 68.76 de la v1) | model card |
| Contexto | 8 192 tokens | model card |

**Consecuencia para el manifest**: el repo **ya lo tiene** (`embeddings/manifest.json:120-131`) con
`dim: 768` y `langs: 100+` correctos, pero con la licencia equivocada y el tamano equivocado
(`FIND-382`, `FIND-384`). **Si debe entrar**: si, pero **no como default** mientras el default
siga siendo `multilingual-e5-small` (384d, MIT, 220 MB) - EmbeddingGemma 2 gana en recall pero cuesta
1 034 MB y es multimodal (3 encoders que VantaDB no usa). Recomendacion: mantenerlo como **modelo de
referencia** y añadir un `--model` a `benchmarks/beir_recall_bench.py` para publicarlo como fila
comparativa en el BEIR canonico. La comparacion exigida (bge-m3, multilingual-e5-*, nomic-embed-text,
jina-embeddings-v3, snowflake-arctic-embed, qwen3-embedding) **no puede hacerse con numeros inventados**:
requiere ejecutar BEIR en el entorno, que esta BLOCKED en esta sesion (sin red de datasets y sin
`cargo`).

**D. Propuesta de benchmarks unicos para VantaDB (P0/P1/P2)**

| # | Nombre | Dataset | Modelo | Metrica | Competidor comparable | Tier | Coste |
|---|---|---|---|---|---|---|---|
| P0-1 | `hybrid_fusion_quality` | BEIR-13 (NLI + MS MARCO + FEVER) | e5-small vs bge-m3 vs embeddinggemma-2 | nDCG@10 / recall@10 de RRF vs BM25-solo vs HNSW-solo | Ninguno publica la descomposicion de la fusion | Fast (dataset pequeno) | 1d + dataset |
| P0-2 | `memory_northstar` | `datasets/longmemeval/longmemeval_s_cleaned.json` | n/a (memoria) | QA accuracy + coste en tokens + latencia de recall | Mem0/Zep publican accuracy sin coste | Fast | 1-2d |
| P0-3 | `total_cost_of_ownership` | SIFT1M (`datasets/sift/`) | e5-small | build time + RSS + p99 + recall@10 en un solo numero | **Nadie lo publica** (gap de mercado) | Heavy | 2-3d |
| P0-4 | `sdk_score_parity` | 1 000 vectores sinteticos seed 42 | - | identidad byte a byte de `scores` entre python/node/ts/wasm | - (es interno, pero bloquea la promesa de paridad) | Fast | 4h |
| P1-1 | `time_travel_query` | dataset sintetico con snapshots | - | exactitud de "estado en T" | No existe benchmark estandar | Fast | 2d |
| P1-2 | `memory_lifecycle_decay` | longmemeval | - | recall vs TTL/decay/compactacion | Ninguno | Heavy | 2d |
| P1-3 | `graphrag_memory_quality` | HotpotQA + 2WikiMultihopQA + MuSiQue | e5-small | accuracy multi-hop + coherence (metricas de GraphRAG-Bench) | GraphRAG-Bench | Heavy | 3-5d |
| P1-4 | `multitenant_isolation` | sinteticow + N tenants | - | 0 fugas cross-tenant + overhead % | Ninguno | Fast | 1d |
| P1-5 | `wasm_cold_start` | - | - | ms hasta primera query + bundle KB | Puro WASM | Fast | 4h |
| P2-1 | `wal_recovery_determinism` | - | - | hash del estado post-recovery (bit-identico) | Ninguno | Heavy | 2d |
| P2-2 | `rbac_overhead` | sintetico | - | overhead % de la comprobacion por rol | Ninguno | Fast | 4h |

**E. Benches locales a mover/fusionar/eliminar**: `param_sweep` y `backend_compare` -> Heavy;
`acorn_filtered_search` y `batch_existing_check` -> fusionar en un unico bench de "camino caliente";
`crash_recovery` -> duplica `tests/certification/stress_protocol.rs` (bloque de crash persistence) y
deberia fusionarse con el; los 13 de `FIND-395` necesitan una decision explicita.

## Veredicto

**FAIL**

El subsistema de benchmarks tiene la **disciplina documental mas alta del proyecto** (el baseline
canonico de `canonical_p99` tiene entorno, comando, seed, fecha y artefacto versionado - la Regla 11
cumplida de verdad) pero **tres huecos estructurales** que la hacen inoperante como puerta de calidad:
el benchmark canonico **nunca se mide antes de mergear** (solo se compila), **13 de 23 benches no se
ejecutan en ningun workflow** (incluidos los que miden scores, memoria, WAL y TTL, es decir, las
caracteristicas que VantaDB vende), y **los unicos resultados de calidad de recuperacion (BEIR,
north-star) no tienen artefacto versionado**. La research de mercado confirma que existe un hueco
real y no explotado - el "coste total" (build + RSS + p99 con recall) no lo publica ningun
competidor - y que VantaDB tiene las piezas para medirlo. Sobre EmbeddingGemma 2: **la v2 existe y es
Apache 2.0**, el repo ya la tiene en el manifest pero con la licencia y el tamano equivocados.

## Indice de hallazgos

| ID | Severidad | Archivo:línea |
|---|---|---|
| `FIND-393` | Medium | `benchmarks/beir_recall_report.json:1` |
| `FIND-394` | Medium | `.github/workflows/nightly.yml:330` |
| `FIND-395` | Medium | `benches/param_sweep.rs:1` |
| `W7-04` | Info | `docs/user/operations/BENCHMARKS.md:380` |
| `W7-05` | Info | `scripts/verify_datasets.ps1:1` |
| `W7-06` | Info | `evals/northstar.mjs:1` |
| `W7-07` | Info | `benchmarks/update_markdown.py:1` |

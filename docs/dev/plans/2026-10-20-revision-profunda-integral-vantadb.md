---
title: "Prompt de revisión profunda integral de VantaDB (multi-subagente, background)"
kind: plan
status: completed
description: "Fan-out de subagentes para auditar paquetes, SDKs, bindings, providers, integrations, MCP, server, skills, docs, tests, benchmarks, dev-tools y el harness .opencode/"
tags: [audit, reviews, benchmarks, tests, harness]
---

# Prompt de revisión profunda integral de VantaDB (multi-subagente, background)

> **Campaign ID:** _(asignar al iniciar con `/pipeline plan`)_
> **Inicio:** 2026-10-20
> **Estado:** COMPLETADO — 8 subagentes, 12 áreas, **141 hallazgos** (2 P0, 39 P1, 59 P2, 41 P3)
> **Alcance:** revisión **read-only** del sistema completo. No implementa fixes.
>
> **Resultados:**
> - Informe: [`docs/dev/reviews/2026-10-20-auditoria-profunda-v0.9.0.md`](../reviews/2026-10-20-auditoria-profunda-v0.9.0.md)
> - Ledger (una línea por hallazgo): [`docs/dev/reviews/2026-10-20-auditoria-profunda-v0.9.0-ledger.md`](../reviews/2026-10-20-auditoria-profunda-v0.9.0-ledger.md)
> - Áreas **sin** pase profundo: `vanta-proxy`/providers, el interior de `.opencode/`, y los
>   workflows no tocados por la rama.
> - Contexto del release en el que corrió: [`docs/dev/reviews/2026-10-20-release-v0.9.0-postmortem.md`](../reviews/2026-10-20-release-v0.9.0-postmortem.md)

---

## 0. Objetivo

Ejecutar una revisión profunda, con múltiples subagentes en paralelo, de **todo** el sistema
VantaDB, y dejar el resultado en disco como evidencia auditable:

1. **Paquetes, módulos, SDKs, funciones, features** del core Rust y de cada binding.
2. **Embeddings, providers, integrations (adapters)**.
3. **MCP server, HTTP server, proxy, memoria**.
4. **Skills** que usarán los usuarios en sus agentes (`skills/vantadb`, `skills/vantadb-mcp`) y las
   skills internas del harness (`.opencode/skills/`, 198 dirs).
5. **Documentación**: todos los `.md` de `docs/` y los de la raíz, **validados contra el código**
   (no contra otros docs). Badges de README, estructura de README, estructura de `docs/`.
6. **Todo el sistema de `.opencode/`** (harness: commands, agents, skills, rules, references,
   task-system, plugins, mcp, configs).
7. **Auditoría de tests**: qué hay, qué falta, qué sobra, qué cambiar/optimizar/automatizar.
8. **Auditoría de benchmarks + investigación web profunda** sobre qué se usa en el mercado, qué usa
   la competencia, qué benchmarks únicos debería construir VantaDB, con qué datasets y con qué
   modelos de embedding (incluido **EmbeddingGemma**).
9. **Auditoría de dev-tools, examples, Formula, integrations, providers, scripts, skills**.

**Entregable final:** un informe de síntesis con veredicto por dominio, hallazgos priorizados
(Critical/High/Medium/Low) con `archivo:línea` y fix propuesto, y filas `FIND-*` en el backlog.

---

## 1. Precondiciones (obligatorias, en este orden)

1. Leer `.opencode/AGENTS.md` completo (reglas duras 1-11, tabla de roles, path resolution).
2. Leer `.opencode/references/definition-of-done.md` y `.opencode/references/backlog-format.md`.
3. Leer `.opencode/references/clean-code-clean-architecture.md` (mínimo Apéndice V).
4. Cargar la skill `progreso` (chequeo de WIP del backlog).
5. Cada subagente, según su área, carga la **regla del área** de `.opencode/rules/` (lazy-loading
   obligatorio) + las references de su dominio antes de actuar. Sin lectura previa, no hay acción.
6. `git status --short` + `git log --oneline -5` para fijar el estado base del repo.

---

## 2. Reglas duras (no negociables)

| # | Regla |
|---|---|
| R1 | **Read-only.** Esta revisión NO implementa fixes, NO toca código de producción, NO borra archivos, NO commitea, NO pushea, NO crea branches, NO taguea. Solo escribe: reportes en `docs/dev/reviews/`, filas en `docs/dev/reports/INDEX.md` y filas `FIND-*` en `docs/dev/Backlog.md`. |
| R2 | **Regla dura `-p`:** nunca `cargo test`/`cargo nextest` desde la raíz sin `-p <crate>`. `--workspace` solo para certificación deliberada — **no se usa aquí** (el CI de 0.9.0 está corriendo). |
| R3 | **No Heavy Certification** (`stress_protocol`, `hnsw validation`, `SIFT`, `competitive_bench`, `chaos_integrity`, `wal_resilience`): hasta 2 h y colisiona con el CI. Marcar como **NOT EXECUTED** con el comando exacto para correrlo. |
| R4 | **No `continue-on-error`**, no `--no-verify`, no Relax de gates. |
| R5 | **`.opencode/` es repo separado** (remoto privado, ignorado por VantaDB). Esta revisión **no escribe** en `.opencode/`; los cambios de harness se reportan como propuesta, se aplican después con Gate H (`/harness` → `vanta-harness`). |
| R6 | **Regla 0 (impacto antes de borrar):** nada se elimina; todo "sobra" se reporta con grep de referencias entrantes/salientes. |
| R7 | **Toda afirmación sobre el código se verifica contra el código** (no contra docs). Cada claim documental se etiqueta: `VERIFIED` / `STALE` / `WRONG` / `UNVERIFIABLE`. |
| R8 | **Cero números inventados.** Todo claim de performance/benchmark cita archivo + comando exacto + entorno (CPU/RAM/OS) + fecha. Si no se puede reproducir, no se publica (Regla 11). |
| R9 | **Toda afirmación externa (web) cita URL.**.Router obligatorio: skill `coordinated-web-search` (keyless → metasearch → argus → fallbacks). No se inventa un resultado de búsqueda. |
| R10 | Los subagentes `vanta-*` son **leaf nodes** (`subagent: deny`): el orquestador los lanza **directamente**, nunca les pide que lanzen otros. |
| R11 | Cada subagente devuelve un digest acotado (< 500 palabras) + `RESULTADO` block, y escribe su reporte completo en disco. El orquestador no re-lee todo: merge por índice. |

---

## 3. Orden de herramientas (obligatorio)

```
1. codegraph_codegraph_explore("<símbolo|pregunta>")   ← búsqueda estructural + blast radius
2. codebase-memory-mcp_* (search_graph / trace_path / query_graph / get_architecture)
   ↳ cargar skill `codebase-memory` (incluye mantenimiento del índice)
3. grep / glob / read                                ← solo para texto literal o lo no indexado
4. web: coordinated-web-search router                 ← metasearch / argus / webfetch
```

Nunca: `grep` para encontrar definiciones (CodeGraph ya lo sabe). Nunca re-verificar con grep lo que
CodeGraph devolvió.

---

## 4. Plan de fan-out (waves)

El orquestador lanza **todas** las waves de abajo en paralelo (máx. ~6 simultáneas por tanda para no
matar el contexto), y hace merge al final. Subagentes disponibles: `explore`, `general`,
`vanta-arch`, `vanta-audit`, `vanta-chaos`, `vanta-docs`, `vanta-engine`, `vanta-harness`,
`vanta-lead`, `vanta-research`, `vanta-review`, `vanta-tuner`, `vanta-worker`.

| Wave | Scope | Subagente(s) | Entregable |
|---|---|---|---|
| **W0** | **Harness `.opencode/`**: `commands/` (14), `agents/` (11), `skills/` (198), `rules/` (15), `references/` (25), `task-system/` (prompts, mcp, enforcement, memory, sandbox, workflows, self-modification, traces), `plugins/ponytail.ts`, `mcp/`, `configs/`, `mcp-profiles/`, `locks/`, `opencode-loop/`, `package.json`/`bun.lock`. Coherencia comando↔skill↔agent↔rule, duplicados, skills huérfanas, refs rotas, permisos TSYS-11 reales vs tabla. | `vanta-harness` | `audit-deep-w0-harness-*.md` |
| **W1** | **Core Rust `src/`** (75 entradas): storage/WAL/LSM/backends, índices (HNSW/IVF/BM25/text/sparse/graph), engine/planner/executor, query IQL, memory (vanta-memory), scheduler, telemetry, CLI. Dead code, complejidad, unsafe, O(n²), dual API, error handling, `[non_exhaustive]`. | `vanta-arch` (arquitectura/coupling/dead-code) + `vanta-engine` (índices/algoritmos) + `vanta-audit` (correctness/seguridad) | 3 reportes: `w1a-arquitectura`, `w1b-indices`, `w1c-correctness` |
| **W2** | **Bindings/SDKs**: `vantadb-python` (PyO3), `vantadb-node` (napi), `vantadb-ts`, `vantadb-wasm` (+OPFS), `vantadb-ffi-core`, `vanta-proxy`. Paridad de API entre SDKs, stubs `.pyi` anti-drift, semver, `VantaError`/`VantaErrorCode`, FFI/UB (raw pointers, `__array_interface__`), zero-copy vs serialización, ergonomía. | `vanta-audit` (FFI/UB) + `vanta-review` (paridad/contrato) | 2 reportes: `w2a-ffi`, `w2b-paridad-sdk` |
| **W3** | **Server + MCP**: `vantadb-server` (axum), `vantadb-mcp`, `.vantadb-mcp/`, `docs/api/openapi.yaml` + `scripts/check_openapi_parity.mjs`, auth/RBAC/rotación de keys, SSE/streaming, rate limit, health, `docs/api/MCP.md` vs código. | `vanta-audit` + `vanta-review` | 2 reportes: `w3a-server-mcp`, `w3b-openapi-paridad` |
| **W4** | **Embeddings + Providers + Integrations**: `embeddings/` (manifest, `download.py`, `verify.py`, `sanity_embed.py`, `embeddings/models/`), `providers/{openai,ollama,litellm,shared_py.rs}`, `integrations/{langchain,llamaindex,crewai,dspy,haystack,letta,mem0,ollama,openai,vantadb_shared}` + `integrations/test_pins.py`. Feature `remote-inference`, fallback offline, paridad entre providers, pins de versiones, cobertura por framework, licencia de modelos. | `vanta-engine` (calidad de embedding/recall) + `vanta-audit` (contratos/licencias/pins) | 2 reportes: `w4a-embeddings-providers`, `w4b-integrations` |
| **W5** | **Docs + READMEs + skills de usuario**: todos los `.md` de `docs/` (`api/`, `user/`, `dev/`, `specs/`, `reports/`) y de la raíz (`README.md`, `README_ES.md`, `CHANGELOG.md`, `CONTRIBUTING.md`, `SECURITY.md`, `SUPPORT.md`, `CODE_OF_CONDUCT.md`, `CITATION.cff`, `SKILLS-MANIFEST.md`, `AGENTS.md`). Validar **cada claim contra el código**. Badges (CI, crates.io, npm, PyPI, license, coverage, docs), links rotos, frontmatter, `kind`↔ruta, `docs/index.md`/`llms.txt` generados, `scripts/docs/*.mjs`. **Skills de usuario**: `skills/vantadb/SKILL.md` (+`scripts/`), `skills/vantadb-mcp/SKILL.md` (+`assets/`, `references/`, `scripts/`) — ¿el agente de un usuario puede seguirlos? ¿los comandos existen? | `vanta-docs` | `audit-deep-w5-docs-*.md` |
| **W6** | **Auditoría de tests** (ver §5) | `vanta-review` (cobertura/gaps/redundancia) + `vanta-chaos` (flaky/estrés/concurrencia) + `vanta-tuner` (coste de la suite, tiempo de CI) | 3 reportes: `w6a-cobertura`, `w6b-flaky-estres`, `w6c-tiempo-suite` |
| **W7** | **Auditoría de benchmarks + research web** (ver §6) | `vanta-tuner` (benchmarks locales) + `vanta-research` (investigación de mercado, read-only, web) | 2 reportes: `w7a-benchmarks-actuales`, `w7b-benchmark-strategy` (con URLs) |
| **W8** | **Tooling y periferia**: `dev-tools/` (23: `verify*.ps1`, `floor-guard.ps1`, `coverage-budget.ps1`, `run-all-tests.ps1`, `nocturnal_suite.ps1`, `chaos_loop.ps1`, `audit-all.ps1`, `parity-compare.mjs`, `validate_doc_snippets.py`), `scripts/` (27: `docs/`, `bench_regression.py`, `north_star_metric.py`, `pre_bench_check.ps1`, `setup-embeddinggemma.ps1`, `download_benchmark_datasets.*`, `verify_datasets.*`, `install.*`), `examples/` (8), `Formula/vantadb.rb` (Homebrew), `e2e/`, `fuzz/`, `datasets/`, `evals/`, `.github/workflows/`. Consistencia Windows/Linux, idempotencia, exit codes, paths hardcodeados, scripts duplicados. | `vanta-audit` (o `explore` si es solo inventario) + `vanta-review` | 2 reportes: `w8a-devtools-scripts`, `w8b-tooling-periferia` |
| **W9** | **Cross-cutting**: seguridad/UB/FFI (`unsafe` sin `// SAFETY:`, Miri), concurrencia (deadlock, lock order, `dashmap`/`parking_lot`/Tokio), memory budget (RSS/OOM guard), API contract (semver, deprecations), paridad de **scores** entre SDKs, `docs/api/scores.md` vs implementación, `SEARCH_PARITY.md`. | `vanta-audit` + `vanta-chaos` | 2 reportes: `w9a-seguridad-concurrencia`, `w9b-contrato-api` |
| **W10** | **Revisión adversarial de la síntesis** (P2-01): validar los hallazgos Critical/High de W0-W9 contra el código, descartar falsos positivos, detectar hallazgos duplicados. | `vanta-review` | `audit-deep-w10-verificacion-*.md` |
| **W11** | **Síntesis global** (hecha por el orquestador, no por subagente) | orquestador | `audit-deep-sintesis-<timestamp>.md` |

### Reglas de fan-out

- Tanda 1: W0, W1, W2, W3 (paralelo). Tanda 2: W4, W5, W8, W9. Tanda 3: W6, W7. Tanda 4: W10.
- W7 depende de W6/W1 solo para **contexto**, no para bloquearse: puede arrancar en tanda 2.
- Cada subagente escribe su reporte en disco y devuelve un digest. El orquestador **no** re-lee los
  reportes completos: extrae el índice de hallazgos del final de cada archivo.
- Si un subagente devuelve `blocked`, el orquestador re-lanza con scope más pequeño o lo resuelve en
  el siguiente turno (nunca quedan waves sin cubrir).
- Cobertura obligatoria: cada archivo del scope asignado está revisado o explícitamente listado como
  "fuera de scope" en el reporte.

---

## 5. W6 — Auditoría de tests (detalle)

**Inventario real a cubrir:** `tests/` (72 entradas: `api/`, `certification/`, `common/`, `core/`,
`fixtures/`, `logic/`, `memory/`, `parity/`, `security/`, `snapshots/`, `storage/` + 62 archivos
raíz), tests por crate (`vantadb-python/tests/`, `vantadb-server/tests/`, `vantadb-mcp/tests/`,
`vantadb-wasm/tests/`, `vantadb-node/test/`, `vanta-memory/tests/`), `#[cfg(test)]` dentro de `src/`,
`e2e/`, `fuzz/`, `evals/`, `integrations/test_pins.py`, `dev-tools/validate_doc_snippets.py`.

Preguntas obligatorias:

1. **Cobertura por módulo**: qué módulo público de `src/` y qué función pública de cada binding
   **no** tiene test. Cruzar `codebase-memory-mcp` (complejidad, `loop_depth`, `linear_scan_in_loop`,
   `alloc_in_loop`, `recursion_in_loop`) contra la existencia de tests: **las funciones más complejas
   y menos cubiertas son la prioridad**. Reportar como tabla `módulo:fn | complejidad | ¿test? | riesgo`.
2. **Áreas sin tests**: paths de `src/` sin ningún test que los toque (grafo, IQL avanzado,
  时间和 time-travel, sparse vectors, MVCC, quota, RBAC, certified delete, snapshot consistency,
   WAL chain verify, importadores, wiki ingestors, embeddings fallback, multi-namespace, etc.).
3. **Tests que sobran**: redundantes, tautológicos (assert que re-asserta el impl), brittle
   (timing/orden/FS), tests que no fallan si el bug se reintroduce (mutation-testing mental),
   tests de "smoke" que nunca podrían fallar, snapshots obsoletos.
4. **Tests a cambiar/optimizar**: los más lentos (miden tiempo), los que dependen de orden, los que
   dependen de red/FS local, los que requieren features opcionales, los que no corren en CI
   (orphan tests: existen pero ningún workflow los invoca).
5. **Automatizar**: qué se puede convertir de manual a determinista (p.ej. `nocturnal_suite.ps1`,
   `chaos_loop.ps1`, `coverage-budget.ps1`), qué falta en CI (`.github/workflows/`), qué puede ir a
   `just` targets.
6. **Flaky**: candidatos a flaky (timing, puertos, orden de iteración, `SystemTime`, threads), tests
   marcados `#![ignore]`, `#[should_panic]` con Tolerance, quarantine (`tests/quarantine_containment.rs`),
   `continue-on-error` en CI (los 8 existentes: ¿justificados?).
7. **Calidad de aserciones**: tests que solo chequean que no explota vs. que chequean el invariante.
8. **Miri / UB**: cobertura real de `tests/miri_unsafe.rs` y `fuzz/` (fuera del workspace: requiere
   nightly + Linux).

Entregable: matriz de cobertura + lista priorizada de tests a **agregar**, **arreglar** y
**eliminar**, con esfuerzo estimado, sin escribirlos todavía (W6 es auditoría; la implementación
va en un plan posterior).

---

## 6. W7 — Auditoría de benchmarks + research web (detalle)

### 6.1 Inventario local (auditar)

`benches/` (25: `canonical_p99.rs` — canónico según Regla 9, `backend_compare`, `bench_concurrent`,
`crash_recovery`, `high_density`, `hnsw_pure`, `hnsw_recall_ef`, `hybrid_queries`,
`incremental_bench`, `ingestion_concurrent`, `ivf_bench`, `list_window`, `memory_budget`,
`param_sweep`, `purge_expired`, `scores_semantics`, `sparse_hot_path`, `stress_test`,
`tokenizer_bench`, `vfile_search`, `wal_throughput`, `acorn_filtered_search`,
`batch_existing_check`, `prefetch_benchmark`), `benchmarks/` (Python: `competitive_bench.py`,
`beir_recall_bench.py` + `beir_recall_report.json`, `embed_bench.py`, `vantadb_local_bench.py`,
`batch_vs_sequential_bench.py`, `prefetch_comparison.py`, `compare_baseline.py`,
`graphrag_bench.rs`, `wasm_bench.mjs`, `wasm_threads_bench.mjs`), `evals/` (`northstar.mjs`,
`dora.mjs`, `eval-metrics.mjs`, `memory_harness.py`, `memory_bench.py`, `calibration.py`),
`datasets/` (`sift/`, `longmemeval/`, `glove-100-angular.hdf5`, `sift-128-euclidean.hdf5`),
`scripts/bench_regression.py`, `scripts/north_star_metric.py`, `scripts/pre_bench_check.ps1`,
`scripts/download_benchmark_datasets.{ps1,sh}`, `scripts/verify_datasets.{ps1,sh}`,
`docs/user/operations/BENCHMARKS.md` (baseline canónico) y
`docs/user/benchmarks/COMPETITIVE_SDK_BENCH.md` + `competitive_sdk_bench{,_milvus}.json`.

Preguntas: ¿qué bench existe y qué mide realmente? ¿el dataset está fijado (seed, versión, checksum)?
¿el resultado es comparable entre runs (mismo hardware, warm/cold, warmup)? ¿hay baseline versionado
o solo artefactos locales ignorados por git (Regla 11: artefactos locales regenerables **no** son
fuente válida)? ¿qué bench NO debería estar en el Fast Gate (<5 min)? ¿qué bench falta para cubrir
las features que hoy no se miden (hybrid fusion, GraphRAG, time-travel, memory lifecycle,
multi-tenant/RBAC, MCP, WASM browser, SDK parity, WAL recovery, RSS/memory budget)?

### 6.2 Research web (obligatoria, con URLs)

**Mercado — qué benchmarks se usan hoy** (investigar y citar):
- Recuperación vectorial / ANN: BEIR, MTEB y MTEB (multilingual), SIFT1M, GIST1M, BIGANN/Deep1B,
  `ann-benchmarks`, ANN-benchmarks de NeurIPS/SIGIR, DiskANN/FreshDiskANN, DiskANN paper benchmarks.
- Híbrido: BEIR, "hybrid search benchmark" (BGE-M3, jina-colbert), Cohere/Elastic benchmarks.
- RAG / GraphRAG: RAGAS, RAGBench, CRAG, FRAMES, 2WikiMultihopQA, HotpotQA, MuSiQue, GraphRAG-Bench,
  GraphRAG de Microsoft (evaluaciones publicadas), HippoRAG, LightRAG, ColBERT/PISA.
- Memoria de agentes: LongMemEval, LoCoMo, MemoryAgentBench, MemBench, Mem0/Zep/Letta/LangMem
  papers-benchmarks, "agent memory benchmark" landscape 2025-2026.
- Operacionales: throughput/latencia P99, recall@k vs ef/HNSW, recall vs build time, coste por
  ingestión, RSS/memory, cold start WASM, coste de build vs query (rebuild-from-scratch).
- Datasets: MS MARCO, NQ, SQuAD, FEVER, BEIR-13, Wikipedia (dpr), LoCoMo, LongMemEval, HotpotQA,
  Wikidata5M, GloVe-100-angular, SIFT128-euclidean.

**Competencia — qué publica cada uno** (investigar y citar): Chroma, LanceDB, Qdrant, Weaviate,
Milvus/Zilliz, Pinecone, Turbopuffer, Elasticsearch/Lucene, Redis Vector, Mem0, Zep/Graphiti,
Letta, LangChain/LangMem, LlamaIndex. Qué datasets, qué hardware publican, qué formato de reporte,
si publican reproducibles o solo marketing. Qué metricas ("nDCG@10", "recall@10", "QPS p99",
"build time", "$/1M vectors") y qué nadie mide todavía (brecha = oportunidad de VantaDB).

**Propuesta de benchmark único para VantaDB** (lo que nadie mide y VantaDB sí puede):
matriz `benchmark | dataset | modelo | métrica | competitor comparable | tier CI | coste` para al menos:
- calidad de fusión híbrida (RRF/weighted) vs BM25 solo vs HNSW solo (nDCG@10/recall@10);
- GraphRAG sobre memoria (vs vector-only y vs full-context) con HotpotQA/2Wiki/MuSiQue;
- **time-travel / historial** (consulta "estado en T") — no existe benchmark estándar;
- **memory lifecycle** (decay/TTL/resumen/compactación) y su efecto en recall;
- **aislamiento multi-tenant + RBAC** (correctness + overhead);
- **agentic memory north-star** (¿el agente recuerda? `evals/northstar.mjs`);
- **paridad de SDK** (¿los 4 bindings devuelven el mismo resultado? mismo `scores.md`?);
- **durabilidad** (WAL recovery determinista, snapshot consistency);
- **WASM/browser** (cold start, bundle size, OPFS);
- **coste total** (build time + RSS + query p99) — el número que nadie publica junto a recall.

**Modelos de embedding** (obligatorio, con verificación de fuente oficial):
- `google/embeddinggemma-300M` y la variante **"EmbeddingGemma 2"** (verificar si existe la v2 a
  fecha 2026-10 — el repo ya tiene `scripts/setup-embeddinggemma.ps1` y `embeddings/manifest.json`
  con `multilingual-e5-small` por defecto): dims, licencia (Gemma license vs Apache/MIT), soporte
  ONNX para el runtime local, cobertura multilingüe, throughput, recall vs `bge-*`/`e5-*` en el
  dataset canónico, y si debe entrar al `manifest.json` como default o como modelo de referencia.
- Comparar con `bge-m3`, `multilingual-e5-*`, `nomic-embed-text`, `jina-embeddings-v3`,
  `snowflake-arctic-embed`, `qwen3-embedding` (verificar estado actual en la web).

### 6.3 Entregable W7

- Auditoría de cada bench local (una línea: qué mide, cómo se corre, si es reproducible, si está
  en Fast Gate o Heavy, si su baseline está versionado).
- Tabla de benchmarks de mercado + de la competencia, con **URL** y **fecha de consulta**.
- **Propuesta priorizada de benchmarks nuevos** (P0/P1/P2), con: nombre, archivo, harness, dataset,
  modelo, métrica, tiempo de ejecución, tier de CI, y qué artefacto se versiona como baseline.
- Lista de benchmarks locales a **eliminar, fusionar o mover a Heavy** (justificada).
- Matriz de datasets/modelos con licencia, tamaño, y命令 de descarga reproducible.

---

## 7. Contrato de salida

### 7.1 Por wave

Cada subagente escribe **un** archivo en `docs/dev/reviews/`:

```
docs/dev/reviews/audit-deep-<wave-slug>-<YYYYMMDD-HHMMSS>.md
```

Con frontmatter mínimo (`title`, `kind: review`, `status`, `description`) y cuerpo:

1. `## Scope` — archivos/paths cubiertos (lista explícita) y excluidos con motivo.
2. `## Metodología` — tools usadas, qué se pudo verificar y qué no.
3. `## Hallazgos` — tabla: `| ID | Severidad | Hallazgo | Archivo:línea | Evidencia | Fix propuesto | Esfuerzo |`.
   Severidades: `Critical` / `High` / `Medium` / `Low` / `Info`.
4. `## Veredicto` — `PASS` / `PASS CON OBSERVACIONES` / `FAIL` + 3 líneas de resumen.
5. `## Índice de hallazgos` — lista plana `ID | Severidad | archivo:línea` para el merge.

IDs de hallazgo: prefijo **FIND-** (ver §7.2). Un hallazgo = una fila; **no** re-ticketear lo ya
registrado (revisar `docs/dev/Backlog.md` antes de emitir un ID nuevo; si ya existe, referenciarlo).

### 7.2 Backlog (regla canónica)

Esquema de **10 columnas, obligatorio** (`.opencode/references/backlog-format.md`):

```
| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |
```

- ID: `` `FIND-NN` `` en backticks en la primera celda. **Verificar el siguiente NN libre** contra
  `docs/dev/Backlog.md` (no colisionar con filas existentes, ni con `AUD-*`/`REVIEW-*` legacy).
- Sección: `## Hallazgos pendientes de reportes` de `docs/dev/Backlog.md`.
- Celda vacía → `-`. Pipes escapados en links.
- `Relaciones` → `Origen: docs/dev/reviews/<archivo>.md` + `Ver: <IDs relacionados>`.
- Solo hallazgos `Medium`+ entran al backlog. `Low`/`Info` se quedan en el reporte.
- **No** crear prefijos nuevos (`AUD-`, `BENCH-`, etc. están prohibidos sin registrarlos en la
  taxonomía de `.opencode/AGENTS.md`).

### 7.3 Índice de reportes

Una fila por reporte en `docs/dev/reports/INDEX.md`:

```
| 20261020-HHMMSS | deep | `docs/dev/reviews/audit-deep-<slug>-<timestamp>.md` | - | C/H/M/L/I | vigente | <resumen> |
```

### 7.4 Síntesis final (W11, la escribe el orquestador)

`docs/dev/reviews/audit-deep-sintesis-<YYYYMMDD-HHMMSS>.md` con:

1. Tablero por wave: `wave | scope | veredicto | #Critical | #High | #Medium | #Low`.
2. **Top 20 hallazgos** del sistema completo, ordenados por severidad × esfuerzo.
3. **Mapa de cobertura**: qué se revisó a fondo, qué se revisó por muestreo, qué no se tocó.
4. **FODA** (Fortalezas, Oportunidades, Defensas, Amenazas) del sistema VantaDB.
5. **Rutas de remediación**: 3 caminos (rápido / medio / estructural) con los IDs Finder.
6. **Veredicto global** de la revisión + estado de preparación de v0.9.0 (con la salvedad de que la
   revisión corre en background y no bloquea el CI).
7. Índice consolidado de todos los hallazgos (todos los IDs, agrupados por severidad).

Actualizar `docs/last-audit-state.json`:
`{timestamp, mode: "deep", veredicto: PASS|FAIL, findings_critical: N, report_file: "<síntesis>"}`.

---

## 8. Definition of Done de la revisión

- [ ] Las 12 waves (W0-W11) tienen reporte en disco o están explícitamente `NOT EXECUTED` con motivo.
- [ ] Cada archivo de cada scope está revisado o listado como excluido con motivo.
- [ ] Cada claim documental tiene veredicto `VERIFIED/STALE/WRONG/UNVERIFIABLE` con evidencia.
- [ ] Todo hallazgo tiene `archivo:línea` real (verificado con Read/CodeGraph, no inferido).
- [ ] Cero findings con número de performance sin archivo+comando+entorno.
- [ ] Toda afirmación de web tiene URL.
- [ ] Hallazgos `Medium+` están en `docs/dev/Backlog.md` como `FIND-*` con las 10 columnas.
- [ ] `docs/dev/reports/INDEX.md` tiene una fila por reporte.
- [ ] `docs/last-audit-state.json` actualizado.
- [ ] Cero cambios de código, cero commits, cero push, cero escrituras en `.opencode/`.
- [ ] Checks de documentación en verde si se escribió cualquier `.md`:
      `node scripts/docs/check-links.mjs && node scripts/docs/check-docs.mjs && node scripts/docs/gen-index.mjs --write`
- [ ] `skill progreso` ejecutado al cerrar (migra completados a `docs/dev/avance/`).

---

## 9. Restricciones operativas (CI v0.9.0 corriendo)

- No ejecutar suites pesadas: nada que tarde >10 min. `cargo nextest` solo con `-p <crate>` y
  `--build-jobs 2`.
- No tocar `target/` compartido con builds masivos.
- No usar `git push` (política del owner). Commits locales: **no** (revisión read-only; los reportes
  los commitea el owner o `vanta-lead` cuando lo pida).
- Si un comando falla por entorno (missing tool, sin red, MSVC), reportar `BLOCKED` con el comando
  exacto y el error; no inventar el resultado.

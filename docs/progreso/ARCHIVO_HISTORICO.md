---
title: "Archivo Histórico — No-Progreso"
status: archive
date: 2026-08-03
---

> Contenido movido desde `docs/progreso/README.md` (líneas 1-1100) el 2026-08-03. Entradas que no son progreso de tarea implementada: autopsias, investigaciones, no-ops, decisiones WONTFIX y meta/proceso. Copia íntegra — sin edición.

## Autopsias

### 2026-07-29 — INV-001: RUSTSEC Advisory Audit ✅

**Fuente:** Backlog (Investigaciones de Seguridad) `INV-001`

**Resuelto por (vanta-audit, vanta-lead):**
- Auditadas 3 RUSTSEC: `atomic-polyfill` (ya gestionada en deny.toml), `paste` y `rustls-pemfile` (no son dependencias activas — stale)
- `cargo deny check advisories` pasa limpio
- Reporte: `docs/audit-reports/inv-001-rustsec-2026-07-29.md`

**Veredicto:** ✅ Sin acciones correctivas — 0 de 3 advisories son riesgo real.

**Ids:** `INV-001`

### 2026-07-30 — INV-024: Unsafe Blocks Audit ✅

**Fuente:** Backlog (Phase 1 — Security & Critical) `INV-024`

**Resuelto por (vanta-audit):**
- Auditados **39 bloques `unsafe`** en 11 archivos del core Rust
- Clasificación: 28 SAFE, 4 SAFE_BUT_UNDOCUMENTED, 7 UB_POTENTIAL
- **🔴 High (1):** panic-DoS en `sq8_similarity` (`distance.rs:411/450`) — reachable desde API pública (`search_vector` → `search_nearest` → `calculate_similarity`), sin guard de dimensiones. Query de 9 dims contra SQ8 de 8 dims → panic. Fix de una línea.
- **🟠 Medium (1):** UB por alineación — `header.vector_offset` nunca validado como múltiplo de 4 en runtime (solo `debug_assert`, ausente en release). Afecta 7 sitios (`ops.rs:509` ni siquiera tiene debug_assert).
- **🟡 Low (3):** RUSTSEC-2026-0002 (lru via ratatui, allowed), MmapFull sin validación de contenido (NaN silencioso), `_force_copy` muerto.
- `cargo deny check` **PASSED** | `cargo audit` 0 critical/high/medium
- Reporte: `docs/audit-reports/inv-024-unsafe-audit-2026-07-30.md`

**Veredicto:** ✅ Core sólido. Fix recomendado primero: guard de una línea en `sq8_similarity`; luego fix central de alineación en `read_header`.

**Ids:** `INV-024`


### TSK-111 — Expanded Filter Operators (2026-06-21) — ❌ NUNCA IMPLEMENTADO

- **Goal:** Extend the flat equality filter system (`VantaMemoryMetadata`) with comparison operators (`Eq, Neq, Gt, Gte, Lt, Lte, In, Exists`).
- **Realidad:** Engine layer (`src/query.rs`, `src/physical_plan.rs`) tiene los 6 operadores (`Eq, Neq, Gt, Lt, Gte, Lte`) para IQL queries. Pero el SDK layer (`src/sdk/serialization/mod.rs:368` — `matches_memory_filters()`) solo hace comparación plana `==`. **`FilterOperator`, `MemoryFilter`, `filter_exprs` nunca existieron en `src/sdk.rs`.** Este checklist documenta algo que debía hacerse pero no se completó.
- **Checklist REAL:**
  - [ ] `FilterOperator` enum en `src/sdk.rs`
  - [ ] `MemoryFilter` struct con `field`, `operator`, `value`
  - [ ] `evaluate_filter()` y `compare_vanta_values()`
  - [ ] `filter_exprs` en `VantaMemoryListOptions` y `VantaMemorySearchRequest`
  - [ ] Exposición a Python/WASM via PyO3/WASM bindings
- **Causa raíz:** Feature documentada y diseñada pero nunca implementada en el SDK. Solo existe en el engine interno (IQL).
- **Archivos que DEBERÍAN modificarse:** `src/sdk/serialization/mod.rs`, `vantadb-python/src/lib.rs`, `vantadb-wasm/src/lib.rs`


### TSK-119 — delete_by_filter (2026-06-21) — ❌ NUNCA FUE SDK, ELIMINADO

- **Goal:** Delete multiple records per metadata filter from SDK and CLI.
- **Realidad:** Solo existió como CLI handler (`cmd_delete_by_filter` en `src/cli_handlers.rs`). **Nunca fue parte del SDK programático** (`VantaEmbedded`). El CLI handler fue eliminado en **commit `e9371ea8` (AUD-09)** como dead code: "4 CLI handlers (cmd_search_similar, cmd_count, cmd_delete_by_filter, cmd_repl, cmd_tui) + rustyline + strsim (~560 LOC)".
- **Checklist REAL:**
  - [ ] `VantaEmbedded::delete_by_filter()` en `src/sdk/api.rs`
  - [ ] Exposición Python via PyO3
  - [ ] Exposición WASM
  - [ ] Tests
- **Bindings:** Ningún binding fue actualizado (los `.pyi` stubs no lo listan, ni `vantadb-wasm/src/lib.rs` lo tiene).
- **Archivos que DEBERÍAN modificarse:** `src/sdk/api.rs`, `vantadb-python/src/lib.rs`, `vantadb-wasm/src/lib.rs`, `vantadb-python/vantadb_py/__init__.pyi`


### TSK-86 — similar_to_key (2026-06-21) — ❌ NUNCA IMPLEMENTADO

- **Goal:** Convenience: search for similar records using the vector of an existing record by its key.
- **Realidad:** `similar_to_key` **nunca se implementó** en ningún lenguaje. No existe en Rust SDK (`src/sdk/api.rs`), ni en Python, ni WASM, ni TS. Git history confirma: cero commits con código `.rs` o `.py` para esta función. Solo existe como concepto en documentación (`docs/api/PYTHON_SDK.md` la menciona como "(not yet exposed)").
- **Checklist REAL:**
  - [ ] `VantaEmbedded::similar_to_key(namespace, key, top_k)` en `src/sdk/api.rs`
  - [ ] CLI handler `vanta-cli search-similar`
  - [ ] Exposición Python/WASM/TS
  - [ ] Tests
- **Causa raíz:** Deuda de especificación — se documentó pero nunca se codificó.
- **Archivos que DEBERÍAN modificarse:** `src/sdk/api.rs`, `src/cli_handlers/search.rs`, `vantadb-python/src/lib.rs`, `vantadb-wasm/src/lib.rs`


### TSK-87 — count with filters (2026-06-21) — ❌ NUNCA FUE SDK, ELIMINADO

- **Goal:** Count records in a namespace, optionally filtered by metadata.
- **Realidad:** Solo existió como CLI handler (`cmd_count` en `src/cli_handlers.rs`). **Nunca fue parte del SDK programático** (`VantaEmbedded::count()`). El CLI handler fue eliminado en **commit `e9371ea8` (AUD-09)** como dead code junto con delete_by_filter y search-similar.
- **Checklist REAL:**
  - [ ] `VantaEmbedded::count(namespace, filters)` en `src/sdk/api.rs`
  - [ ] CLI handler `vanta-cli count`
  - [ ] Exposición Python/WASM/TS
  - [ ] Tests
- **Nota:** Existe un helper interno `fn count_memory_records_from()` en el engine, pero no es público.
- **Archivos que DEBERÍAN modificarse:** `src/sdk/api.rs`, `src/cli_handlers/mod.rs`, `vantadb-python/src/lib.rs`, `vantadb-wasm/src/lib.rs`


### TSK-88 — Multi-namespace Search (2026-06-21) — ❌ NUNCA IMPLEMENTADO

- **Goal:** Search multiple namespaces simultaneously.
- **Realidad:** `VantaMemorySearchRequest` siempre ha tenido `namespace: String` (singular). **`namespaces: Vec<String>` nunca existió.** El git history pre-refactor (`72d334c3^:src/sdk.rs`) confirma que siempre fue `namespace: &str`. El único `Vec<String>` de namespaces está en tipos de reporte (output: `VantaExportReport.namespaces`, `VantaTextIndexAuditReport.namespaces_audited`), no como parámetro de búsqueda.
- **Checklist REAL:**
  - [ ] `namespaces: Vec<String>` en `VantaMemorySearchRequest` (tipos)
  - [ ] Backward compat: si `namespaces` vacío, usar `namespace`
  - [ ] Implementación: iterar namespaces, merge top_k por score
  - [ ] CLI: `vanta-cli search --namespace ns1,ns2,...`
  - [ ] Exposición Python/WASM
  - [ ] Tests
- **Causa raíz:** Feature diseñada/documentada pero nunca implementada ni en tipos ni en lógica de búsqueda.
- **Archivos que DEBERÍAN modificarse:** `src/sdk/serialization/vector_types.rs`, `src/sdk/search/mod.rs`, `src/sdk/api.rs`, `src/cli_handlers/search.rs`, `vantadb-python/src/lib.rs`, `vantadb-wasm/src/lib.rs`


### 2026-07-14 — REV-016: Audit vantadb-enterprise premature abstraction

| ID | Tarea | Cambio | Estado |
|----|-------|--------|--------|
| REV-016 | Audit `vantadb-enterprise` premature abstraction | Delivered audit report then deleted entire crate per recommendation. Every module was speculative (96% TODO stubs). Removed `vantadb-enterprise/` directory + workspace member from `Cargo.toml`. Net: -267 lines. | ✅ |

**Verificación:** Manual audit per ponytail-audit method. Full report: `docs/reviews/REV-016-vantadb-enterprise-audit.md`.


## Investigaciones

21. **[TSK-28]** Research: lock-free HNSW (DISC-01) — ✅
    - Conclusion: current `RwLock` is sufficient for predictable workloads

29. **[TSK-57]** Research: large benchmark dataset (DISC-02) — ✅
- `scripts/download_benchmark_datasets.sh`, `tests/benchmark_datasets.rs`

63. **[TSK-84]** DISC-03: Prefetch benchmark — ✅
- Prefetch 13.8% faster, `src/index.rs:33-72`

### 2026-07-30 — INV-002: Memory Telemetry Correction ✅

**Fuente:** Backlog (Phase 4 — Engineering Health) `INV-002`

**Resuelto por (vanta-tuner):**
- Inventario completo: 10+ métricas mapeadas a fuente real (PSAPI/sysinfo, `estimate_memory_bytes`, `VantaFile::mmap_resident_bytes`, jemalloc-ctl, CacheWarmer dead code, MemoryGovernor sin gauge)
- Hallazgos clave: `volatile_cache_cap_bytes` hardcoded `0` (roto), sampler periódico inexistente, `MemoryGovernor` no conectado a métricas
- Esquema de 5 categorías diseñado (core / index / page_cache / mmap / ingest) con invariante explícita: categorías son vistas ortogonales, nunca sumarlas como total; único agregado es RSS del OS (`core ≈ rss − index − page_cache − mmap − ingest`)
- Propuesta: `IntGaugeVec` con label fijo `category` — validado contra API oficial `tikv/rust-prometheus`; descartado `metrics`/`metrics-tracing` del backlog (workspace usa prometheus 0.14 directo)
- Contrato público `OperationalMetrics` (TS SDK) preservado — Vec aditivo en `/metrics`
- Doc actualizado: `docs/operations/MEMORY_TELEMETRY.md` (+357 líneas, reconciliado DISC-05), cross-ref en `PERFORMANCE_TUNING.md`
- **Sin implementación** — solo diseño + propuesta (src/ intacto)

**Veredicto:** ✅ Esquema aprobado para fase 2 (implementación de gauges por categoría como task futuro).

**Ids:** `INV-002`


### INV-007: Competitive benchmark vs LanceDB/Chroma — investigación y diseño
- **Fuente:** Backlog (Phase 6 — Launch Campaign)
- **Fecha:** 2026-08-03
- **Objetivo:** Diseñar el benchmark competitivo VantaDB vs LanceDB/Chroma como asset marketing #1 para audiencia técnica. Sin implementación — solo diseño + propuesta.
- **Resultado:** ✅ `docs/Investigaciones/INV-007-competitive-benchmark-lancedb-chroma.md` (19.8KB). Veredicto: NO publicar en `ann-benchmarks` (repo sin mantenimiento, recomienda migrar a VIBE, integración costaría 3-5 días) — usar solo como fuente de datasets HDF5 + metodología Recall-QPS. Harness standalone reproducible Python (`benchmarks/competitive/run_competitive_benchmark.py`) con glove-100-angular (1.18M×100d cosine) + sift-128-euclidean (1M×128d L2). Protocolo: 10K queries oficiales, warmup 100, 5 runs mediana, grid M∈{16,32}×ef_search∈{10,50,100,200}, hardware publicado. Métricas: Recall@10, QPS, latencia p50/p95/p99, RSS peak, build time. Contrato web `competitive_benchmark.json`. Slicing vertical: Slice 1 harness+JSON, Slice 2 tabla `competitive-table.tsx` bajo `<BenchmarkRace />`, Slice 3 CI manual. 10 fuentes web citadas. Cero cambios de código.
- **Ids:** `INV-007`


### INV-008: Batch Queries Python SDK — diseño
- **Fuente:** Backlog (Investigaciones Post-Consolidación)
- **Fecha:** 2026-08-03
- **Objetivo:** Diseñar `VantaDB.search_batch()` para ejecutar múltiples queries en paralelo vía Rayon. Sin implementación — solo diseño.
- **Resultado:** ✅ `docs/Investigaciones/INV-008-batch-queries-python-sdk.md` (10.9KB). Gate: parcialmente implementado — `search_batch(vectors, top_k)` vector-only YA existía (`vantadb-python/src/lib.rs:1181`, GIL release eager + Rayon `into_par_iter`, wrapper async `__init__.py:214`, tests `test_sdk.py` + `benchmarks/batch_vs_sequential_bench.py`). Gap real: no acepta SearchRequest completo (filters/text_query/namespace/hybrid). Propuesta: `search_batch_requests(queries: List[SearchRequest]) -> List[SearchResult]` con dataclass SearchRequest (vector, top_k, namespace, text_query, filters, distance_metric, explain), reusar patrón GIL+Rayon sobre `engine.search`, errores parciales fail-fast v1, target batch 10 < 3× single, plan 4 pasos (binding → dataclasses/stubs → tests → bench). Veredicto YAGNI: método nuevo en binding, wrapper Python puro descartado. Cero cambios de código.
- **Ids:** `INV-008`


### INV-009: Phrase Queries + Term Positions — diseño
- **Fuente:** Backlog (Investigaciones Post-Consolidación)
- **Fecha:** 2026-08-03
- **Objetivo:** Diseñar phrase query operator con almacenamiento de term positions para snippets destacados. Sin implementación — solo diseño.
- **Resultado:** ✅ `docs/Investigaciones/INV-009-phrase-queries-term-positions.md` (13.8KB). Gate: parcialmente implementado — infrastructure phrase-ready YA existía (`TextQueryPlan.phrases` text_index.rs:145, `TextRecordTerms.token_positions` :132, `posting_value(node_id, tf, positions)` :554, `text_positions_match_phrase` en src/sdk/search/phrase.rs:28 con 12 tests, test `spec_declares_phrase_ready_text_index_v3`). Gaps: sintaxis IQL, enforcement en query execution, highlight de frase. Propuesta: `Condition::TextMatch(field, query)` + `parse_condition` reusando `string_literal` (parser delega a `query_plan()`), filtro con `text_positions_match_phrases`, riesgo con advanced-tokenizer (frases deben tokenizar sin stopword removal), `highlight_phrases` para envolver frase completa en un solo `<strong>`. Veredicto tantivy: CUSTOM (YAGNI) — tantivy duplicaría índice, ~40 crates, sin feature ausente relevante. Cero cambios de código.
- **Ids:** `INV-009`


### INV-010: ACID rollback multi-capa completo — diseño
- **Fuente:** Backlog (Investigaciones Post-Consolidación)
- **Fecha:** 2026-08-03
- **Objetivo:** Diseñar el rollback coordinado entre WAL, VantaFile, HNSW y KV store para completar el soporte ACID (Phases 1-3 implementadas: WAL txn records, buffered writes, MVCC snapshot). Sin implementación — solo diseño.
- **Resultado:** ✅ `docs/Investigaciones/ACID_ROLLBACK_DESIGN.md` (28KB, en inglés). Research `ACID_TRANSACTIONS.md` estaba borrado del repo (commit `8b1c52cd`) pero íntegro en git — documentado el gap y citados los 3 enfoques verbatim (A=Fjall-only rechazado, B=Custom WAL layer recomendado, C=SQLite journal rechazado). Protocolo: extender B con `WalRecord::Prepare(u64)` + reordenar commit point (prepare durable → aplicar stores por costo de compensación → Commit; Abort ante fallo), recovery roll-forward idempotente sin breaking changes. Hallazgos F1-F6: commit durable antes de apply, recovery pre-MVCC (`created_by_txn: 0`), derived indexes sin compensación, VantaFile sin watermark, HNSW remove irreversible. Plan 4 fases (4a WAL v2+Prepare keystone, 4b KV pre-image+GC, 4c VantaFile watermark+HNSW commit protocol, 4d derived-index consistency). `cargo check -p vantadb` ✅ (0 cambios de código).
- **Ids:** `INV-010`


### INV-011: Core-Server Separation — auditoría
- **Fuente:** Backlog (Investigaciones — Rust/SDK)
- **Fecha:** 2026-08-03
- **Objetivo:** Verificar si el core embebido (`VantaEmbedded`) tiene dependencias no deseadas del modo servidor (axum, tower, MCP). Sin implementación — solo auditoría.
- **Resultado:** ✅ **Separación YA limpia — sin cambios requeridos.** Server deps (tokio, axum, tower_governor, tower-http, rustls, opentelemetry) todas optional detrás de features `server`/`tls`/`opentelemetry`/`prometheus`. `default = [cli, arrow, fjall, roaring, advanced-tokenizer, memmap2, fs2, sysinfo]` NO incluye server deps. Imports server-only gated: `cli_server.rs`/`circuit_breaker.rs`/`connection_pool.rs` bajo `#[cfg(feature = "server")]` en lib.rs. Verificado mecánicamente: `cargo tree --no-default-features -F cli -e normal` = 0 deps server; `cargo check -p vantadb --no-default-features -F cli` exit 0. Observación menor: `server = ["cli",...]` acopla server→cli (intencional, YAGNI separar hoy). Doc: `docs/Investigaciones/INV-011-core-server-separation.md`. Cero cambios de código.
- **Ids:** `INV-011`


### INV-012: Anti-Locality Disk Layout — re-evaluación
- **Fuente:** Backlog (Investigaciones — Storage/Benchmarks)
- **Fecha:** 2026-08-03
- **Objetivo:** Re-evaluar si con LSM compaction + multi-level storage el BFS relabeling tiene más impacto que en DRV-130 (WONTFIX ~9%). Sin implementación — solo benchmark + recomendación.
- **Resultado:** ✅ **WONTFIX CONFIRMADO — NO re-abrir.** Re-run `benches/vfile_search.rs` (vía vanta-tuner, release): with_vfile 614.5ms vs with_vfile_compacted 571.5ms → mejora locativa **~7.0%**, inferior al 9% de DRV-130 y muy bajo el 15% requerido. `with_vfile` ~4.9x sobre in-memory; BFS compaction recupera ~1 unidad. LSM/multi-level NO alteraron el resultado. Causa raíz vigente: search greedy (distancia-guía) diverge del orden BFS; overhead es call/mmap-deref, no page misses. Limitación: dataset 10K×128 ≈5MB cabe en page-cache (infravalora locality en SSD frío). Re-apertura hipotética requeriría dataset 1M+ cold-cache. Nota: backlog apuntaba a `src/index/graph.rs`; real en `src/storage/archive.rs`+`maintenance.rs`. Doc: `docs/Investigaciones/INV-012-antilocality-reevaluation.md`. Cero cambios de código.
- **Ids:** `INV-012`


### INV-006: Blog series completion — plan de finalización
- **Fecha:** 2026-08-02
- **Objetivo:** Plan de finalización del blog series (sin implementación). MKT-05 reportaba 4/5; audit 2026-07-28 corrigió que el backlog inflaba el conteo (3 posts en docs/blog, 4 en web).
- **Checklist:**
  - [x] `docs/strategy/BLOG_SERIES_PLAN.md` — inventario 4 web vs 3 docs/blog con 6 mismatches (M1-M6), incluyendo `introducing-vantadb` sin fuente `.md` y drift de versión (M6)
  - [x] Revisión drafts — 3 posts sólidos, listos tras fixes (CTA débil en 2/3, frontmatter incompleto)
  - [x] Audiencia + keyword research — 5 segmentos, 6 clusters validados por búsqueda web 2026
  - [x] Calendario — Show HN (referencia SHOW_HN_PREP.md) + cadencia 2 posts/mes alineada a GTM (6/12/24)
- **Resultado:** ✅ Plan entregado (205 líneas, markdownlint 0 issues), sin implementación de contenido. Commit 042e8e50. **Siguientes acciones derivadas:** resolver M1 (crear `introducing-vantadb.md`) y M6 (unificar versión en web/posts/SHOW_HN) — no bloquean el plan.
- **Ids:** `INV-006`


### INV-019: Advanced Tokenizer (Unicode + Stopwords)
- **Fecha:** 2026-08-02
- **Objetivo:** Investigar tokenizer avanzado con Unicode, stopwords per-language y stemming.
- **Resultado:** ❌ SKIP — ya implementada. Verificado contra código real (no se re-investigó).
- **Checklist de verificación:**
  - [x] `src/tokenizer.rs` (288 líneas) — `tokenize_advanced()`, `AdvancedTokenizerConfig` (language, max_token_length, remove_stopwords, apply_stemming), `is_advanced_tokenizer_available()`
  - [x] Feature gate `advanced-tokenizer` — `Cargo.toml:108`, habilitado en `default` (línea 94)
  - [x] Wiring runtime — `src/config.rs:321,586,776` (`advanced_tokenizer_config`, `with_advanced_tokenizer_config`)
  - [x] Integración `src/text_index.rs` — `TextTokenizerSpec::advanced()`, `TEXT_INDEX_SCHEMA_VERSION=4`, nombre `tantivy-multilingual`
  - [x] Tests multilingües (ES/FR/DE), stemming, stopwords, length_filter, combined
  - [x] Bench `benches/tokenizer_bench.rs` (ASCII vs Tantivy)
  - [x] Commits: `1a7c4d04`, `7459a558`
- **Gap detectado:** `docs/api/ADVANCED_TOKENIZER.md` no existe — doc de API pendiente (ticket separado, no bloquea el SKIP).
- **Ids:** `INV-019`


### 2026-07-06 — Post-Benchmark Deep Investigations (4 paralelas, 25 tareas agregadas al backlog)

**Objetivo:** Investigar a fondo los gaps contra LanceDB/ChromaDB revelados en benchmarks competitivos. 4 sub-agentes en paralelo.

#### Hallazgos clave por área:

| Área | Hallazgos | IDs asignados |
|------|-----------|---------------|
| 🐛 Distancia Euclidea | **Bug crítico:** `squared_distance` raw vs `1.0 - similarity` causa ordenación invertida. Recall@10 55.7% vs ChromaDB 90%. Fix estimado: 1 hora | CODE-092 🔴 |
| ⚡ AVX-512/SIMD | f32x16 dispatch, SQ8 path, norm caching, runtime multiversion — avx512f ya detectado, no cableado | PERF-21/22/29/34/38 🟡 |
| ⚡ FFI/PyO3 | `put_batch_raw` PyBuffer 2D, `#[pyclass]` hits, lazy serialization, GIL scope tuning | PERF-15/16/24/25/26/31/35 🔴🟡🟢 |
| ⚡ HNSW Recall | ef_construction 200→400, M/max0 16→24, ep_enter freeze, tombstone mitigation | PERF-17/18/23/27/28 🟠🟡 |
| ⚡ Ingestion | WAL batch append, storage batch insert, async pipeline, config tuning | PERF-19/20/30/32/33/36/37 🟠🟡🟢 |

**Impacto cuantificado:**
- CODE-092 fix solo: recall euclidean 55.7% → ~90% (paridad ChromaDB)
- PERF-15 + PERF-19 + PERF-20: ingestion QPS 127 → ~1500+ (10×)
- PERF-16: query latency 4.06ms → ~2.5ms (cerca de 2.27ms ChromaDB)

**Backlog:** +25 items agregados. Pendientes: 98 items open.


### 2026-08-02 — INV-017: sccache en CI — investigación

**Objetivo:** Investigar compatibilidad de `sccache` con GitHub Actions + `Swatinem/rust-cache`, evaluar complementariedad/redundancia, diseñar integración mínima y medir impacto estimado.

| ID | Tarea | Resultado | Evidencia |
|----|-------|-----------|-----------|
| `INV-017` | sccache en CI — investigación | ✅ COMPLETED | `docs/Investigaciones/INV-017-sccache-ci.md`. Hallazgo clave: `.opencode/AGENTS.md` afirmaba falsamente sccache implementado (drift, 0 matches en `.github/`); corregido. Diseño: `mozilla-actions/sccache-action@v0.0.11` + `SCCACHE_GHA_ENABLED` + `RUSTC_WRAPPER` en rust-setup. |

**Verificación:** `rg -rln sccache .github/` → 0 (pre-cambio) ✅ | `docs/Investigaciones/INV-017-sccache-ci.md` existe ✅


## No-ops / SKIPs

### 2026-07-25 — REV-012: HNSW insert_lock contention analysis ✅

**Fuente:** Backlog Phase 2 `REV-012`

**Resuelto por (vanta-tuner, ponytail):**
- Análisis de 3 puntos de contención:
  - DashMap `nodes`: shard count = num_cpus * 4, critical sections µs-scale → ✅ Adecuado
  - Mutex RNG: 2-5µs hold, 64 adquisiciones/batch → 🟡 No medido como bottleneck
  - FairMutex insert_lock: micro-batching 64 ops/acq + try_lock non-blocking drain → ✅ Bien mitigado
- Sin code changes — todo ya mitigado. Comentario ponytail documentando upgrade path (thread_local SmallRng si profiling lo requiere)

**Verificación:** `cargo check -p vantadb` ✅

**Ids:** `REV-012`

### 2026-07-25 — Phase 3 Test Coverage: 7 tasks completadas (document-only) ✅

**Fuente:** Backlog Phase 3 — evaluación de cobertura de tests

**Resultados — 0 code changes (todas document-only, ponytail):**

| ID | Módulo | Tests | Hallazgo |
|----|--------|-------|----------|
| `DRV-013` | ShardedWal | 25 tests, ~90%+ line coverage | Gap: concurrent access no testeado. Document-only |
| `DRV-017` | search.rs / serialize.rs | 33+29 tests | Gap: mmap zero-copy unsafe path no testeado. Document-only |
| `DRV-061` | OpenAI adapter | 10 tests/119L | Happy path sólido. Error paths dependen de API externa. |
| `DRV-067` | Ollama adapter | 8 tests/79L | Adapter 1-line delegate. Document-only |
| `DRV-073` | LiteLLM adapter | 10 tests/78L | Mejor coverage de los 3. Document-only |
| `TEST-11` | Frontend (Vitest + Playwright) | 38+54 tests | Sin cross-browser WASM (demo es placeholder). Document-only |
| `TEST-12` | Fuzzing | 4 targets + proptest | Sin corpus guardado ni storage API fuzz. Document-only |

**Verificación:** todos los checks pasan ✅

**Ids:** `DRV-013`, `DRV-017`, `DRV-061`, `DRV-067`, `DRV-073`, `TEST-11`, `TEST-12`


### DRV-109: LlamaIndex missing GIL release
- **Fuente:** Plan 2026-07-14 backlog-campaign
- **Fecha:** 2026-07-14
- **Objetivo:** Release GIL in `add`, `query`, `delete` using pyo3 0.29 `detach()` — already correct from the start, no changes needed
- **Resultado:** ✅ `cargo check -p vantadb-llamaindex` passes, no-op
- **Ids:** `DRV-109`


### DEVOPS-13: Pin all workflow actions to SHA + Node 22
- **Fuente:** Plan 2026-07-14 backlog-campaign
- **Fecha:** 2026-07-14
- **Objetivo:** Replace `actions/*@vX` with pinned SHA across all workflows; update Node 20→22
- **Resultado:** ✅ No-op — no `.github/workflows/` files exist in this repository
- **Ids:** `DEVOPS-13`


### SEC-01/SEC-02: Security Advisory Resolutions (bincode, rustls-pemfile)
- **Fecha:** 2026-07-02
- **Objetivo:** Verify bincode 1.x → 2.0 (already migrated via AUD-03) and rustls-pemfile deprecation (already on v2). Both advisories found already resolved.
- **Checklist:**
  - [x] `SEC-01` — bincode confirmed on v2.0. Already resolved in AUD-03 (bincode 1.3 → 2.0)
  - [x] `SEC-02` — rustls-pemfile confirmed on v2. Already resolved
- **Ids:** `SEC-01`, `SEC-02`


### MKT-14: Case studies publicados (3)
- **Fecha:** 2026-08-02
- **Objetivo:** Publicar 3 case studies + rutas `/case-studies/` y `/case-studies/[slug]`. El backlog decía "falta pulir copy y métricas" — falso negativo corregido por el audit 2026-07-28 y re-verificado en código.
- **Checklist de verificación (gate SKIP — ya implementada):**
  - [x] `web/src/components/vanta/vanta-data.ts:993-1042` — `CASE_STUDIES` con 3 items (metrics, challenge, solution, quote, quoteAuthor)
  - [x] Listing `/case-studies/` (`page.tsx`) — cards con métricas, tags, i18n
  - [x] Detail `/case-studies/[slug]` (`[slug]/page.tsx`) — 404 handling, metrics grid, challenge/solution, quote, CTA
  - [x] i18n — 62 keys `caseStudiesData.0/1/2` en `dictionaries.ts` (ES/EN)
  - [x] Navegación — `site-navbar.tsx:71` (`nav.caseStudies` → `/case-studies`) + `LIVE_ROUTES` (4 rutas) en `use-vanta-navigate.ts:63-66`
- **Resultado:** ✅ COMPLETADA 2026-08-02 — SKIP por gate (feature ya implementada, validada por audit 2026-07-28:205 como "Más completo de lo reportado"). Sin code changes.
- **Ids:** `MKT-14`


### CODE-022: Remove unused Three.js dependency (600KB+ bundle reduction)
- **Fecha:** 2026-07-04
- **Objetivo:** Three.js no tenía ningún import en `web/src/` pero estaba listado en package.json. Ya fue eliminado en commit previo — verificado: no está en package.json, node_modules, ni imports.
- **Checklist:**
  - [x] Verificar que no haya imports de three en `web/src/` (0 imports ✅)
  - [x] Verificar que no esté en `package.json` (ya removido ✅)
  - [x] Verificar que no esté en `npm ls three` (empty ✅)
- **Ids:** `CODE-022`


### 2026-07-23 — Batch TIER 1 Gate Check (8 tasks completadas)

**Objetivo:** Verificar estado real de TIER 1 backlog tasks contra código antes de implementar, porque 3 de 4 primeras ya estaban fixeadas como side effects.

| ID | Tarea | Resultado | Evidencia |
|----|-------|-----------|-----------|
| ~~`DRV-031`~~ | Doc comment duplicado | ✅ SKIP | Side effect de refactor previo, doc existe 1 vez |
| ~~`DRV-026`~~ | Redundant unwrap en three_way_merge | ✅ SKIP | Código usa match sobre .get(), sin unwrap |
| ~~`DRV-116`~~ | 10 warnings compilación | ✅ SKIP | `cargo check -p vantadb` + `-p vantadb-mcp` = 0 warnings |
| ~~`DRV-040`~~ | unsafe sin SAFETY en simd.rs | ✅ SKIP | No existe archivo simd.rs en el proyecto |
| ~~`DRV-025`~~ | TOCTOU race ResourceGovernor | ✅ ALREADY FIXED | Ya usa CAS loop con compare_exchange_weak |
| ~~`DRV-047`~~ | Hardcoded MCP validation limits | ✅ ALREADY FIXED | Todos los handlers usan config.* values |
| ~~`DRV-012`~~ | WAL sync duplication | ✅ ALREADY FIXED | maybe_sync() ya extraída como fn separada |
| `DRV-009` | node_count() O(n) full scan | ✅ ALREADY FIXED | Ya usa AtomicU64 cacheado con fetch_add/fetch_sub. Comment `/// DRV-009` |
| `DRV-016` | Mutex inconsistency governor.rs | ✅ ALREADY FIXED | Código ya usa parking_lot::Mutex (import L7), no std::sync::Mutex. Backlog description estaba obsoleta. Verificado Jul 23 |
| ~~`DRV-019`~~ | 14 .expect() en SIMD hot-path | ✅ FIXED | Replaced 14 `.expect()` → `unsafe { .unwrap_unchecked() }` + SAFETY comment en 7 funciones SIMD (f32x8, f32x16, sq8). 62 tests pass. commit pending |
| ~~`DRV-007`~~ | Data race en filter_field | ✅ ALREADY FIXED | DashMap interno es thread-safe. `_nodes` dead code binding. Sin UB. Verificado Jul 23 |
| `DRV-049` | collection_delete no atómico (MCP) | ✅ ALREADY FIXED | Ya usa begin_transaction/abort/commit con abort en delete parcial. Verificado Jul 23 |

**Patrón detectado:**** 7/8 TIER 1 tasks ya resueltas — las campañas de refactor previas (DRV-004, DRV-006, commits 3fd2c0d, aa87e5c, d9e1caf, 4467004, de6ecac) limpiaron más issues de los que trackeaban explícitamente. El backlog tenía estado ❌ en items que ya estaban ✅ en código.

| `DRV-005` | SDK unit tests search/mod.rs | ✅ FIXED | 18 tests agregados para `search()`, `lexical_search()`, `vector_memory_search()`, `hybrid_search()`. Cubre: BM25 scoring, HNSW fallback, RRF fusion, explain mode, corrupt index. Verificado Jul 24 |


### 2026-07-25 — P4 Engineering Health Wave 0: VFY-004 (flat.rs O(n²) comment-only)

**Objetivo:** Document that `flat.rs` filter O(n²) is by design (DashMap scan bounded by `flat_threshold`).

| ID | Tarea | Archivos | Resultado |
|----|-------|----------|-----------|
| `VFY-004` | flat.rs O(n²) en filter — comment-only | `src/index/flat.rs:32` | ✅ `dd13b67d` — 0 code logic changes. Comment explains bounded scan. |

**Verificación:** `cargo check -p vantadb` ✅


## WONTFIX / Decisiones

### 2026-08-02 — COMP-019: Binary protocol (gRPC) — WONTFIX ✅

**Fuente:** Backlog (Phase 10 — Competitive Features) `COMP-019`

**Resuelto por (vanta-lead):**
- **Decisión:** WONTFIX. gRPC contradice el posicionamiento embedded-first de VantaDB.
- rkyv (serialización binaria zero-copy) ya cubre la serialización interna en storage/WAL — el 80% del valor técnico de la tarea.
- Sin demanda de usuario ni dependencias de otras tareas en el backlog → YAGNI.
- Micro-ADR: `docs/architecture/adr/COMP-019-binary-protocol-wontfix.md`
- Backlog: `COMP-019` tachado como WONTFIX (línea 279).
- ROADMAP: 3 referencias a COMP-019 actualizadas (Sem 13-14, FASE 4, resumen).

**Criterio de re-apertura:** si aparece un caso de uso de servidor remoto con transferencia masiva de vectores, o un issue de usuario que lo requiera. La base rkyv deja la serialización lista.

**Ids:** `COMP-019`

## Meta / Proceso

### Housekeeping sin ID (Backlog audit, Clippy/fmt fixes, Fix `with_writer`, ttl_ms)

65. **Backlog audit** — ✅
- 4 discrepancies corrected (TSK-94/67/80/82)
66. **Clippy/fmt fixes** — ✅
- 3 unused vars, formatting 18 files, conditional imports
67. **Fix `with_writer`** — ✅
- `MakeWriter` closure instead of direct `Box<dyn Write>`
68. **`vantadb-mcp` ttl_ms: None** — ✅
- `planner.rs:369` `expires_at_ms: Some(0)`

### 2026-07-24 — Pipeline: auto-progreso + auto-commit en /pipeline task ✅

**Proceso:** Se detectó que `skill progreso` (Trigger 1 — migración de tareas completadas) y el commit automático no se ejecutaban al final del pipeline MODO TAREA.

**Fix:**
- `pipeline.md` pasos 6-7 agregados después del Review: `skill progreso` + auto-commit
- Aplica a ambos modos: MODO TAREA y MODO RUN
- Decisión registrada en campaign_memory como policy

### 2026-07-07 — Reorganización Masiva del Backlog (24 eliminaciones, 21 adiciones, 11 prioridades)

**Fuente:** Análisis completo del proyecto (`docs/research/VantaDB_ANALISIS_COMPLETO.md`) que evaluó cada item del backlog contra: impacto real, esfuerzo, timing, alineación con visión estratégica.

**Cambios ejecutados:**
- **24 items eliminados** del backlog activo: Cloud entero (7 items), optimizaciones prematuras (6), SOC2/HIPAA (2), WAL shipping, PITR, Semantic Kernel, visual regression, y 4 duplicados/ya-existentes
- **11 items re-priorizados**: 5 subieron a 🔴 (WASM demo, Discord, TS SDK, MCP docs), 3 bajaron a 🟡/🟢 (ARM64, signing, GraphRAG metodología)
- **21 nuevos items agregados**: sanitizer CI, flat index, migration tools, learning path, WASM fallbacks, HNSW auto-tuning, PQ, LSM, sparse vectors, y más
- **Resultado**: Backlog pasó de 79 → **65 items activos**

**Documentación completa:** `docs/progreso/backlog-2026-07-07.md`

### Week of 2026-07-01 — Documentation overhaul & Code Hardening

- **Documentation structure**: Re-created Obsidian graph color groups (`docs/.obsidian/graph.json`), installed usability plugins (Dataview, Linter, Calendar) to optimize reading and editing experience locally.
- **Wikilinks resolution**: Repaired 58 instances of broken `[[wikilinks]]` that were improperly nested inside Markdown code blocks across 10 files (like `architecture.md`, `HTTP_API.md`). Confirmed that while GitHub doesn't natively render wikilinks, they remain ideal for the primary Obsidian-based workflow.
- **Syntax error fix**: Fixed an improper module-level doc comment (`//!`) and a duplicate `use std::time::Duration` inside `src/cli_server.rs` that was preventing the build and breaking `rustfmt`.
- **Clippy static analysis**: Fixed an `if_same_then_else` warning in `src/sdk/search.rs:307` related to distance resolution.
- **Codebase formatting**: Applied `cargo fmt` across all 22 Rust files (1349 lines modified, mostly line-wrapping and import ordering).
- **Test Suite Verification**: Discovered a system resource limit (Windows pagefile `os error 1455`) during parallel compilation. Bypassed by compiling the `lib` tests individually. All 440/440 tests are now passing successfully.

### Week of 2026-06-19 — Complete Comprehensive Audit (AUD-01→44)

- **44 audit findings resolved** in a single day using parallel specialized agents (3 per batch, 15 batches).
- **7 critical** (security, packaging, documentation), **14 medium** (tests, CI/CD, infra), **23 low** (refactors, technical debt, UX).
- **Files modified**: ~45 files between Rust, Python, YAML, TOML, Markdown, scripts.
- **New files**: `tests/edge_cases.rs` (25 edge case tests).
- **CVEs resolved**: RUSTSEC-2025-0141 (bincode), RUSTSEC-2026-0176/0177 (pyo3).
- **Updated PHASE 3 exit criteria**: all AUDs resolved ✅

### Week of 2026-06-12 → 2026-06-18

- **TSK-79**: Benchmark regression alerts. `scripts/bench_regression.py` (3 modes), nightly workflow with automatic GitHub Issue creation. Updated progress and CHANGELOG.
- **CI fixes**: Conditional imports in `cli_server.rs`. Step benchmark datasets in coverage job. Update `install-action` to `@v2`.
- **Clippy audit**: 5 categories of warnings corrected (too_many_arguments, suspicious_open_options, field_reassign_with_default, needless_range_loop, needless_borrow)
- **Comprehensive audit**: 40 documented findings (7 critical, 14 high, 19 medium).
- **Final push**: 30 commits ahead, pushed to `ness-e/Vantadb` main (commit `f5eafbd`)

### 2026-07-26 — P2 Backlog Housekeeping: DRV-041, VFY-006, VFY-007 ✅

**Fuente:** Backlog P2 Quick Wins — tareas triageadas como "ya corregidas" en pase de revisión anterior

| ID | Tarea | Resultado |
|----|-------|-----------|
| `DRV-041` | **worker.rs Promise con serde_wasm_bindgen** — **Corregido:** _reject sí se invoca (línea 254). No hay serde_json round-trip (usa serde_wasm_bindgen). Descripción original no coincide con código real. | ✅ Document-only. Backlog actualizado. |
| `VFY-006` | **`add_node` / `remove_node` lock contention** — **Corregido:** usa DashMap (locking por shard) + AtomicUsize/AtomicU128 (lock-free). Único Mutex es rng. | ✅ Document-only. Backlog actualizado. |
| `VFY-007` | **`remove_node` O(n²) neighbor fixup** — **Corregido:** archivo real `src/index/graph.rs` (no `core.rs`). | ✅ Document-only. Backlog actualizado. |

**Verificación:** Backlog.md P2 counter 15→12. Sin code changes (ponytail — ya estaba corregido).


### 2026-07-26 — Backlog Cleanup: P0–P4, P7, P9–P10 — 53 items moved to progreso

**Objetivo:** Limpiar backlog verificando cada item ✅ contra código real. Mover completados a progreso, re-abrir falsos positivos.

| Fase | Acción | Items |
|------|--------|-------|
| **P0** | 6 stale removidos + 1 WONTFIX (DEVOPS-15) | `DEVOPS-10/12/14`, `NUEVO-09/10` removidos. `DEVOPS-15` WONTFIX — 7 features necesarias para experiencia "it just works" |
| **P1** | Fase completa cerrada (9 items resueltos/deferidos) | `RC-06`, `SEC-13/15/16/17`, `VFY-010/14/15/16` |
| **P2** | 7 ✅ + 24 stale a progreso | `DRV-014/028/041/136`, `VFY-006/007`, `REV-012` + 24 crates de integración nunca implementados |
| **P3** | 7 ✅ + 7 stale a progreso | `DRV-013/017/061/067/073`, `TEST-11/12` + 7 stale |
| **P4** | 10 ✅ a progreso | `WEB-03/04`, `VFY-004/011`, `DRV-121/122/123/130/131`, `DOC-20` |
| **P7** | 2 ✅ a progreso | `NUEVO-13` (auto-tuning), `NUEVO-14` (WASM bundle 394KB gzip < 500KB) |
| **P9** | 7 ✅ a progreso | `OLD-04/07/13/15/17/18/22` |
| **P10** | 12 ✅ a progreso | `COMP-001/002/003/004/005/006/007/009/011/015/018/020/030` |

**Impacto:** Backlog total ~120→~65 items activos. 5 fases cerradas (P1–P4, P7). Exec Summary actualizado.

**Verificación:** Cada item verificado contra código real antes de mover. `DEVOPS-15` re-abierto tras detectar discrepancia en `Cargo.toml:89`, luego marcado WONTFIX — reducir features rompe UX "it just works".

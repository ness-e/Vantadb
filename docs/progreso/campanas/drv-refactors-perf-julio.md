# Serie DRV — refactors y performance (julio 24-25)

> **Migrado desde** `docs/progreso/README.md` (split GOV-D2, 2026-08-22). Contenido histórico sin cambios salvo dedup indicado.

### 2026-07-25 — DRV-014: ShardedWal::batch_append sin clonación de WalRecords ✅

**Fuente:** Backlog Phase 2 `DRV-014`

**Problema original:** `batch_append()` creaba `Vec<Vec<WalRecord>>` por shard y clonaba cada record con `record.clone()` — overhead de alloc en batches grandes.

**Resuelto por (vanta-worker, ponytail):**
- Eliminado el batch vector intermedio y `record.clone()` por completo
- Reemplazado con loop directo `append()` round-robin por shard
- -10 líneas de código, 0 allocs intermedios, misma semántica

**Verificación:** `cargo check -p vantadb` ✅ | 25/25 tests wal_sharded ✅

**Ids:** `DRV-014`

### 2026-07-25 — DRV-136: Medición del tamaño del bundle de vantadb-wasm + fix de rustflags LTO ✅

**Fuente:** Backlog Phase 2 `DRV-136`

**Resuelto por (vanta-worker):**
- Medido bundle: 1,158 KB raw / 433 KB gzipped — dentro de rango normal para DB embebida en WASM
- Todos los levers de optimización ya activos: `opt-level = "s"`, `wasm-opt -Oz`, `lto = "thin"`, `codegen-units = 1`
- Fix en `.cargo/config.toml`: removido `-C lto=yes` de rustflags que rompía build WASM (`tracing-wasm` lib crate rechaza LTO)
- Recomendación: fat LTO (`--config 'profile.release.lto="fat"'`) opcional para ~5-10% extra

**Verificación:** `cargo check -p vantadb-wasm` ✅ | `cargo build -p vantadb-wasm --target wasm32-unknown-unknown --release` ✅

**Ids:** `DRV-136`

<!-- movido a ARCHIVO_HISTORICO.md -->
<!-- movido a ARCHIVO_HISTORICO.md -->
### 2026-07-25 — DRV-054: read_axioms extraído a const + resolve_axioms() con fallback ✅

**Fuente:** Backlog `DRV-054` — 4 axioms inline, no sync con metadata

**Problema original:** `AXIOMS` en `vantadb-mcp/src/lib.rs:77-82` era un string JSON literal sin posibilidad de override desde storage.

**Resuelto por (vanta-worker):**
- Renombrado `AXIOMS` → `HARDCODED_AXIOMS` (semántica de fallback)
- Agregadas constantes `SYSTEM_NAMESPACE` y `AXIOMS_STORAGE_KEY` para lookup en storage
- Creada `resolve_axioms()`: intenta `embedded.get("_system", "axioms")`, fallback a const
- Handler `read_axioms` actualizado a `resolve_axioms(storage)`
- Lookup best-effort (fallback safe en error: not found, parse error, engine no init)

**Verificación:** `cargo check -p vantadb-mcp` ✅ | `cargo clippy -p vantadb-mcp -- -D warnings` ✅

**Ids:** `DRV-054`

### 2026-07-25 — Tanda de Refactor: DRV-036, DRV-038, DRV-029, DRV-032, DRV-055 ✅

**Fuente:** Backlog `DRV-036`, `DRV-038`, `DRV-029`, `DRV-032`, `DRV-055`

**Resuelto por:**
- **`DRV-036` (TypeScript SDK):** `_mapRecord` actualizado para usar `isMemoryRecord(r)` de `guards.ts` para validación de tipo exhaustiva.
- **`DRV-038` (TypeScript SDK):** `MemoryRecord` actualizado para permitir `string | number` en campos de tiempo/versión/ID para paridad total con Rust/Python `u64`.
- **`DRV-029` (Python SDK):** `py_dict_to_metadata` optimizado con retorno inmediato en diccionarios vacíos y generación de cache-key sin allocs de strings intermedios.
- **`DRV-032` (Python SDK):** Documentación explícita y estructura limpia para las firmas PyO3 con `too_many_arguments`.
- **`DRV-055` (MCP):** `test_mcp_invalid_json` refactorizado para testear estrictamente la respuesta del protocolo MCP JSON-RPC (`-32700` y `-32602`) en vez de `serde_json` interno.

**Verificaciones:** `npx tsc --noEmit` ✅ | `cargo check -p vantadb_py` ✅ | `cargo test --package vantadb-mcp` ✅ (25/25 passed)

**Ids:** `DRV-036`, `DRV-038`, `DRV-029`, `DRV-032`, `DRV-055`

### 2026-07-25 — DRV-034: Refactorización de bloques try-catch en TypeScript SDK ✅

**Fuente:** Backlog `DRV-034` — 38 bloques try-catch repetidos en `vantadb-ts/src/vantadb.ts`

**Problema original:** Cada método de instancia de la clase `VantaDB` en el SDK TypeScript contenía un bloque try-catch idéntico `try { ... } catch(e) { throw wrapWasmError(e, "X"); }`, generando ~200 líneas de boilerplate sin valor.

**Resuelto por:**
- Introducido método privado genérico `_wasm<T>(method: string, fn: () => T): T` en la clase `VantaDB`.
- Refactorizados 35 bloques try-catch en métodos de instancia a llamadas `_wasm()`.
- 3 factory methods estáticos (`connect`, `create`, `open`) conservan su try-catch propio (no tienen `this`).
- `close()` preserva su `try/finally` por el side-effect crítico de `_closed = true`.
- Hallazgo colateral: `reindex_hnsw_from_text` no estaba en el binario WASM instalado — se documentó con `VantaError` explícito hasta que el WASM sea actualizado.

**Verificación:** `npx tsc --noEmit` ✅ 0 errores.

**Ids:** `DRV-034`

### 2026-07-25 — DRV-030: Refactor de conversores _to_pydict vía Macro Rust ✅

**Fuente:** Backlog `DRV-030` — Conversores de reportes a PyDict duplicados en PyO3

**Problema original:** `vantadb-python/src/convert.rs` contenía 12 conversores de reportes a diccionarios de Python (`PyDict`) con ~180 líneas de código repetitivo de PyO3 (`PyDict::new(py)`, `.set_item(...)`, `Ok(dict.unbind().into())`).

**Resuelto por:**
- Definida la macro declarativa `pydict_set!` en `convert.rs`.
- Refactorizadas las funciones `rebuild_report_to_pydict`, `export_report_to_pydict`, `import_report_to_pydict`, `text_index_repair_report_to_pydict`, `text_index_audit_report_to_pydict`, y `operational_metrics_to_pydict` usando la macro.
- Código reducido en ~180L manteniendo 100% la compatibilidad y firma de retorno.

**Verificación:** `cargo check -p vantadb_py` ✅

**Ids:** `DRV-030`

### 2026-07-25 — DRV-050: Sanitización de Inyección de Consultas en MCP ✅

**Fuente:** Backlog `DRV-050` — MCP injection: LISP query via string interpolation

**Problema original:** `query_lisp()` en `vantadb-mcp/src/lib.rs` no desinfectaba ni validaba adecuadamente las cadenas de entrada recibidas desde clientes MCP, dejando un vector de inyección de código/comandos o caracteres nulos descontrolados.

**Resuelto por:**
- Agregada validación de entradas vacías y detección/rechazo de caracteres de byte nulo (`\0`) en la tool `query_lisp` del servidor MCP.
- Implementado recorte y desinfección previa al envío a `executor.execute_hybrid()`.
- Añadida suite de pruebas en `vantadb-mcp/tests/mcp_tests.rs` (`test_mcp_query_lisp_sanitization`).

**Verificación:** `cargo test --package vantadb-mcp` ✅

**Ids:** `DRV-050`

### 2026-07-25 — OLD-05: Calidad de Búsqueda v2 (Unicode + snippets) ✅

**Fuente:** Backlog `OLD-05` — Search Quality v2 (Unicode + snippets)

**Problema original:** La extracción de snippets y el resaltado de términos (`generate_snippet_with_highlighting` y `highlight_terms`) en el core dependían de `to_ascii_lowercase()` y `eq_ignore_ascii_case()`, fallando al buscar o resaltar coincidencias insensibles a diacríticos/acentos (`café` vs `cafe`, `rápido` vs `rapido`).

**Resuelto por:**
- Implementado Unicode accent folding (`fold_char` y `fold_str`) en `src/sdk/search/snippet.rs` para insensibilidad a diacríticos en la búsqueda de posición de snippet y resaltado `<strong>`.
- Preservados los caracteres y diacríticos originales del texto fuente dentro de las etiquetas `<strong>`.
- Añadidos unit tests `unicode_folding_snippet_accent_match` y `unicode_folding_snippet_unaccented_query`.

**Verificación:** `cargo test --package vantadb --lib sdk::search::snippet::tests` ✅

**Ids:** `OLD-05`

### 2026-07-26 — DRV-130 fix T1: SearchProfile gated tras #[cfg(debug_assertions)] ✅

**Fuente:** Backlog P4 `DRV-130` (refinimiento T1)

**Problema:** SearchProfile original tenía `if vector_store.is_some()` inline en hot path. ThinLTO no podía especializar entre `None` vs `Some(vfile)`, causando 23% overhead solo por tener el parámetro (bench `pass_none`: 506ms vs in_memory 412ms).

**Resuelto por:**
- SearchProfile partido en dos: struct real con tracking (`#[cfg(debug_assertions)]`) y ZST no-op (`#[cfg(not(debug_assertions))]`)
- Métodos extraídos: `record_vfile_entry`, `record_vfile_candidate`, `start_compute`/`end_compute`
- `if vector_store.is_some()` eliminado del hot path — siempre se llama al método, en release es no-op

**Resultado:** `pass_none` overhead eliminado (506ms → 282ms). `in_memory -26%`, `with_vfile -24%`. 1515 tests pasan.

**Ids:** `DRV-130`

### 2026-07-25 — DRV-130: Cuello de botella de búsqueda SIFT 1M — SearchProfile + auditoría de prefetch ✅

**Fuente:** Backlog P4 `DRV-130`

**Problema original:** `search_nearest` usa HNSW sin optimización SSD-locality. SIFT 1M high-recall en 127s.

**Resuelto por (vanta-tuner, ponytail):**
- **T1 ✅ SearchProfile:** Nuevo `SearchProfile` struct en `src/index/search.rs` con `vfile_reads`, `unique_pages`, `compute_ns`, `candidates_seen`. Instrumentado en hot paths de `search_layer`.
- **T2 ✅ WONTFIX:** `prefetch_mmap_vector` ya implementa `madvise(MADV_WILLNEED)` / `PrefetchVirtualMemory`. Prefetch ya activo.
- **T3 ❌ WONTFIX:** Node reordering investigado y descartado. Benchmark con `compact_layout` (BFS reorder) mostró solo ~9% de mejora (2,440→2,221 ms). Search sigue greedy distance-guided path, no BFS order. Overhead es de function calls y bounds checks, no page misses. <20% threshold → cerrado como WONTFIX.

**Verificación:** `cargo bench --bench vfile_search` — in_memory: 783ms, with_vfile: 2,440ms, with_vfile_compacted: 2,221ms (~9% improvement). `cargo check --benches` ✅.

**Ids:** `DRV-130`

### 2026-07-24 — DRV-022: Eliminado código muerto governance/ (1235L) ✅

**Fuente:** Backlog stabilization plan — Phase 2, Task 11

**Problema original:** `src/governance/` contenía 5 archivos (1235L total) con `#![allow(dead_code)]`, gated tras feature `governance` no-default sin consumidores. Feature nunca activada, dependía de `sync_ext` que hacía compilación inviable incluso si se activara.

**Resuelto por:** Eliminación completa del directorio `src/governance/` y feature `governance` de `Cargo.toml`. Conservados: `DuplicatePreventionFilter`, `OriginCollisionTracker`, `compute_confidence_friction` — ya extraídos en `src/utils/` como production-ready, re-exportados vía `pub use utils::compute_confidence_friction`.

**Verificación:** `cargo check -p vantadb` ✅ (0 errores). `cargo clippy -p vantadb -- -D warnings` ✅.

**Ids:** `DRV-022`

### 2026-07-24 — DRV-059/065/071/087/091/096: RwLock<String> → String (6 tareas ✅)

**Fuente:** Backlog DRV Hallazgos — adapters/providers, `review-deep` Wave 0 (quick)

**Problema original:** 6 adapters usaban `RwLock<String>` para `namespace` que nunca se escribía, solo se leía (0 `.write()`, múltiples `.read().unwrap().clone()`). Overhead innecesario de lock + alloc.

**Resuelto por:** La **Adapter Restructure** (commit `accbfa8`) reestructuró todo el sistema:
- **DRV-059** (OpenAI): `providers/openai/src/python.rs:70` → `namespace: String` plano
- **DRV-065** (Ollama): `providers/ollama/src/python.rs:41` → `namespace: String` plano
- **DRV-071** (LiteLLM): `providers/litellm/src/python.rs:72` → `namespace: String` plano
- **DRV-087** (CrewAI): Migró de Rust PyO3 a Python puro (`integrations/crewai/`). `RwLock` no existe en Python.
- **DRV-091** (DSPy): Migró de Rust PyO3 a Python puro (`integrations/dspy/`). Idem.
- **DRV-096** (Haystack): Migró de Rust PyO3 a Python puro (`integrations/haystack/`). Idem.

**Verificación:** `grep -r "RwLock" providers/` → 0 matches. `grep -r "RwLock" integrations/` → 0 matches.

**Ids:** `DRV-059`, `DRV-065`, `DRV-071`, `DRV-087`, `DRV-091`, `DRV-096`

### 2026-07-24 — DRV-070/086/092/098/103/110: Metadata no-string ignorado (6 tareas ✅)

**Fuente:** Backlog DRV Hallazgos — adapters/providers

**Problema original:** `v.extract::<String>()` en Rust PyO3 descartaba silenciosamente Bool/Int/Float. Usuario pasaba `metadata={"count": 5}` y el valor desaparecía sin warning.

**Resuelto por adapter restructure + migración a Python:**
- **DRV-070** (LiteLLM): `providers/litellm/src/python.rs:282-288` — fallthrough chain `String→bool→i64→f64`
- **DRV-086** (CrewAI): Migrado a Python puro (`integrations/crewai/`). Commit `b83f0f9`
- **DRV-092** (DSPy): Migrado a Python puro (`integrations/dspy/`). Commit `b83f0f9`
- **DRV-098** (Haystack): Migrado a Python puro (`integrations/haystack/`). `dict(doc.meta)` maneja todos los tipos
- **DRV-103** (LangChain): Migrado a Python puro (`integrations/langchain/`). Commit `b83f0f9`
- **DRV-110** (LlamaIndex): Migrado a Python puro (`integrations/llamaindex/`). Commit `b83f0f9`

**Verificación:**
- LiteLLM Rust provider: `grep "extract::<String>" providers/litellm/src/python.rs` → solo en key extraction (correcto), metadata usa fallthrough chain
- 5 adapters Python: no hay `v.extract::<String>()` en código Python

**Ids:** `DRV-070`, `DRV-086`, `DRV-092`, `DRV-098`, `DRV-103`, `DRV-110`

### 2026-07-24 — DRV-068/069/074/079/085/107/112: Misc GIL + paginación + cobertura de tests (7 tareas ✅)

**Fuente:** Backlog DRV Hallazgos — Bug C + Bug D

- **DRV-068/069** (LiteLLM GIL + store()): Resuelto por adapter restructure.
  `search()` ahora usa `py.detach()` (L243), `store()` acepta `py: Python` (L263)
- **DRV-074/079/085** (Paginación solo página 1): Resuelto por migración a Python puro.
  Mem0: `delete_col()` usa `delete_namespace()` atómico. Letta: `list()` acepta custom limit. CrewAI: `list()` soporta cursor
- **DRV-107** (LangChain test coverage): 5 tests/43L → **25 tests/256L**
- **DRV-112** (LlamaIndex delete malformed IDs): Migrado a Python puro. `delete()` cursor-based con `rec.key`
- **Ids:** `DRV-068`, `DRV-069`, `DRV-074`, `DRV-079`, `DRV-085`, `DRV-107`, `DRV-112`

### 2026-07-24 — TIER 4: REV-010/DRV-023/DRV-044/DRV-046 resueltos (4 tareas ✅)

**Fuente:** Backlog TIER 4 — refactors medianos, verificación contra código actual

| ID | Resolución |
|----|------------|
| **REV-010** | `src/sdk/serialization/mod.rs` (1827L) ya split en **8 archivos**: mod.rs 1051L + 7 submodulos (conversions, graph_types, impl_export, impl_index, impl_rebuild, impl_text_index, vector_types). Total 4223L en 8 archivos. |
| **DRV-023** | `ResourceGovernor` ya tiene callers: `execute_plan`/`execute_statement` en planner.rs + integration test `engine_governor_certification` en `tests/logic/governor.rs`. `ALLOCATED_BYTES` trackeado. |
| **DRV-044** | `vantadb-server/src/main.rs` reescrito (58L). Flujo async: `run_stdio_server(storage).await` → flush → exit natural. Eliminado `process::exit(0)` en shutdown path. |
| **DRV-046** | `vantadb-mcp/src/lib.rs` usa `tokio::io::AsyncBufReadExt::lines()` (L364) no bloqueante + `tokio::sync::Semaphore` (L329) + graceful shutdown via `AtomicBool`. |

**Ids:** `REV-010`, `DRV-023`, `DRV-044`, `DRV-046`

### 2026-07-24 — DRV-027: Refactor del módulo God vantadb-python/src/lib.rs (1991L → 4 archivos) ✅

**Fuente:** Backlog DRV Hallazgos — Python SDK (TIER 2, `review-deep` Wave 0)

**Problema original:** `vantadb-python/src/lib.rs` tenía 1991 líneas mezclando VantaDB pyclass (~35 métodos), 22 funciones de conversión `*_to_pydict`, LRU cache, VantaVector/VantaVectorIter, VantaPySearchHit, error mapping, y helpers vectoriales. God module con responsabilidades no separadas.

**Resuelto por:** División en 4 archivos especializados:
- **`convert.rs`** (28,799 B) — todas las funciones de conversión Python↔VantaValue + LRU cache + error mapping
- **`vector.rs`** (3,269 B) — VantaVector, VantaVectorIter pyclasses
- **`types.rs`** (11,142 B) — existente, se le agregó VantaPySearchHit
- **`lib.rs`** (40,131 B, ~900L) — solo VantaDB pyclass, connect(), módulo setup

**Verificación:** `cargo check -p vantadb_py` → ✅ 0 errores. `cargo clippy -p vantadb_py` → ✅ 0 issues. `cargo fmt` aplicado. Sin cambios en API pública.

**Ids:** `DRV-027`

<!-- movido a ARCHIVO_HISTORICO.md -->
### 2026-07-24 — VFY-010: ACID Fase 2 — Transacciones de Escritura Bufferizadas ✅

**Fuente:** Backlog (VFY Hallazgos)

**Problema original:** Cada `insert()`/`delete()` escribía a stores y WAL inmediatamente — N fsyncs por transacción. Sin buffer, `abort()` no podía descartar writes ya enviados a stores.

**Resuelto por:** Buffer writes in-memory durante la transacción. I/O a stores y WAL diferido hasta `commit()`. En `abort()` los buffers se descartan sin escribir nada.

**Cambios:**
- `src/storage/engine/mod.rs` — `BufferedWrite` enum (Insert/Delete), `active_txn_id`, `txn_buffers`
- `src/storage/engine/init.rs` — inicialización de nuevos campos
- `src/storage/engine/ops.rs` — `begin_transaction()` setea txn_id (sin WAL); `insert()`/`delete()` bufferan si hay txn activa; `commit_transaction()` drena buffer → WAL batch → apply stores; `abort_transaction()` descarta buffer; `get()` chequea buffer primero (read-your-writes)
- `src/storage/engine/tests.rs` — 6 tests nuevos (commit_persists, abort_rolls_back, delete_abort, read_your_writes, empty_commit, double_commit_error)

**Verificación:** `cargo check -p vantadb` ✅. 3 tests existentes ✅. 6 tests nuevos ✅. Cero regresiones.

**Ids:** `VFY-010`

### 2026-07-23 — DRV-001: Refactor del god file search.rs (1162L → 845L, 5 sub-módulos) ✅

**Fuente:** Backlog DRV Hallazgos — SDK, `review-deep` Wave 0

**Objetivo:** Dividir `src/sdk/search.rs` (1162L, 4+ responsabilidades) en sub-módulos con responsabilidad única, agregando tests unitarios donde sea viable.

**6 pasos completados:**
- **Step 1 (phrase.rs):** `text_positions_match_phrase`, `text_positions_match_phrases` → free fns + 13 tests
- **Step 2 (snippet.rs):** `generate_snippet_with_highlighting`, `highlight_terms` → free fns + 9 tests
- **Step 3 (debug.rs):** 5 debug helpers extraídos del impl block (rank_map, explain_hit, identities, bm25_terms, matched_phrases). 15 call sites actualizados.
- **Step 4 (text_index.rs):** 4 helpers de audit/repair/ensure_text_index extraídos. mod.rs solo retiene 4 wrappers de delegación (1-6 líneas c/u).
- **Step 5:** 22 unit tests totales (phrase + snippet). debug.rs y text_index.rs requieren StorageEngine mock — cubiertos por tests de integración.
- **Step 6:** fmt, clippy --deny, nextest (1598/1599 ✅) — 1 pre-existing flaky. Sin breaking changes en API pública.

**Estructura final:** `src/sdk/search/{mod,phrase,snippet,debug,text_index}.rs`

**Ids:** `DRV-001`

### 2026-07-19 — DRV-002 + DRV-003: Eliminación de duplicación SDK y fix de perf ✅

**Fuente:** Backlog DRV Hallazgos — SDK, `review-deep` Wave 0

- **DRV-002:** `put_batch` duplicaba ~50 líneas de `put()`. Se extrajo método privado `put_one()` — ambos métodos delegan a él. [`f029d42`](`fix: Bug Fix Phase 1`)
- **DRV-003:** `purge_expired` llamaba `replace_derived_indexes` O(n) veces por nodo → reemplazado por `derived_delete_ops` (selectivo). [`d9e1caf`](`perf(DRV-003)`)

**Ids:** `DRV-002`, `DRV-003`

### 2026-07-19 — Fixes Diferidos Post-RC (DEF-01 → DEF-05) ✅

**Fuente:** Investigación de sub-agentes (vanta-tuner, vanta-engine, vanta-worker, vanta-audit) sobre 7 items diferidos post-feature-freeze. Item 3 (WAL) omitido, item 7 (missing_docs) verificado como non-issue.

**5 fixes implementados y verificados (commit `aee17f9`):**
- **DEF-01 (SendPtr → `Arc<Mmap>`):** Reemplazado `*const f32` wrapper por `Option<Arc<Mmap>>` en `VectorRepresentations::MmapFull`. Elimina UB cuando mmap se re-mappea. Archivos: `src/node.rs`, `src/index/graph.rs`, `src/index/serialize.rs`, `src/index/distance.rs`, `src/storage/engine/maintenance.rs`.
- **DEF-02 (text_stats_cache bounds + read path):** Cache-aside en `load_text_term_stats` y `load_text_namespace_stats`. Watermark eviction al 100% del límite. 3 constantes globales en `src/config.rs` (`MAX_TEXT_STATS_CACHE=100k`, `MAX_TEXT_NS_CACHE=1k`, `MAX_CARDINALITY_PAIRS=10k`). Archivos: `src/config.rs`, `src/sdk/serialization/impl_text_index.rs`, `src/sdk/api.rs`, `src/sdk/serialization/impl_index.rs`, `src/storage/engine/ops.rs`, `src/storage/engine/stats.rs`.
- **DEF-03 (scan_prefix streaming):** Nuevo método `scan_prefix_iter` en `Backend` trait que retorna `Box<dyn Iterator>`. Implementado en fjall, rocksdb, in_memory. 3 callers migrados. Elimina materialización `Vec<(Vec<u8>,Vec<u8>)>`. Archivos: `src/backend.rs`, `src/backends/fjall_backend.rs`, `src/backends/rocksdb_backend.rs`, `src/backends/in_memory.rs`, `src/storage/engine/partition.rs`, `src/sdk/serialization/impl_export.rs`, `src/sdk/search.rs`.
- **DEF-04 (HNSW ahash + pre-alloc):** XxHash64 (~30-50ns) → ahash (~5-10ns) en `search_layer`. HashSet pre-allocado con `with_capacity_and_hasher`. `ahash` agregado a `Cargo.toml`. Archivos: `src/Cargo.toml`, `src/index/search.rs`, `src/index/graph.rs`, `src/index/flat.rs`, `src/index/serialize.rs`.
- **DEF-05 (lexical_search with_capacity):** `HashMap::new()` → `with_capacity(safe_estimate)`. `node.clone()` → `&UnifiedNode` en `memory_record_from_node`. 12+ callers actualizados. Archivos: `src/sdk/search.rs`, `src/sdk/serialization/mod.rs`, `src/sdk/api.rs`, `src/sdk/serialization/impl_export.rs`, `src/sdk/serialization/impl_rebuild.rs`, `src/sdk/serialization/impl_text_index.rs`.

**Verificación:** `cargo check --workspace` ✅, `cargo clippy -D warnings` ✅, `cargo fmt --check` ✅, `cargo nextest run --profile audit --workspace` ✅ (550/550 tests).

<!-- movido a ARCHIVO_HISTORICO.md -->
### 2026-07-14 — REV-011: Descomponer función monolítica insert_hnsw de 177L

- **REV-011 (✅ completado):** Extraído `connect_layer_neighbors()` de `insert_hnsw` en `src/index/graph.rs:595-619`. El bucle anidado de 3 niveles para la conexión bidireccional de vecinos ahora es un método privado con nombre. `insert_hnsw` reducido de ~135→112 líneas. Sin cambio de comportamiento.
- **Hallazgos colaterales:** 2 errores pre-existentes en `src/sdk/serialization/impl_index.rs` (acceso de fn privada a métodos de `impl_text_index.rs`). No relacionados con REV-011.

### 2026-07-14 — REV-009: Optimizar compilación del workspace con default-members

- **REV-009 (✅ completado):** Removido `--workspace` de las 9 invocaciones de `cargo check/clippy/nextest` en `ci-rust-10.yml` (ahora usan `default-members`). Añadido `[workspace] default-members = [...]` a `Cargo.toml` listando solo 5 paquetes core, excluyendo 12 adapter crates de los rebuilds de desarrollo.

### 2026-07-08 — WASM Demo + Quick Wins (NUEVO-03/04) + Ruta Demo

- **WASM-03 (completado):** Ruta `/demo` creada con chat interactivo (Transformers.js + mock embedder + fallback in-memory). Fixes: `vector: [vector]` double-wrap, `@wasm` alias resuelto copiando `pkg/` a `web/src/wasm/`, `vite-plugin-wasm` configurado, `cssMinify: "esbuild"` para compatibilidad Tailwind v4. Demo completamente funcional.
- **NUEVO-03 (✅ completado):** `llms.txt` ya existía en raíz del repo (describe el proyecto para AI crawlers). `web/public/llms.txt` es específico del sitio web. Backlog actualizado.
- **NUEVO-04 (✅ completado):** `CONTRIBUTING.md` ya estaba en raíz. `CODE_OF_CONDUCT.md` copiado de `.github/` a raíz. Ambos archivos detectables por GitHub.
- **MKT-13 (⏳ en progreso):** Ruta `/demo` funcional y diseñada con brand VantaDB. Pendiente: enlace "Try in browser" desde la hero + deploy a Vercel.
- **Rediseño visual demo:** CSS reescrito con hard corners, amber accent, dark surfaces, JetBrains Mono, hard shadows — consistente con el design system VantaDB.
- **Backlog:** NUEVO-02/03/04 + COM-01 movidos a ✅. MKT-13 marcado como ⏳ (solo falta hero link). Total pendiente: 60 ❌ + 2 ⏳ = 62 open.
- **Tokens file:** Creado `.env.tokens.example` con documentación de todos los tokens/secrets del proyecto. `.env.tokens` (real) en `.gitignore`. `.env.tokens.example` (template) trackeado.
- **INT-01/02 adapters fix:** LangChain y LlamaIndex adapters reparados para usar la API actual de `vantadb-py` (propiedades en vez de dicts). Tests: ✅ 5/5 LangChain, ✅ 5/5 LlamaIndex. Dep `vantadb-py>=0.3` corregida a `>=0.2`. Ya están listos para publicar.

### 2026-07-03 — Tanda Masiva de Adapters, WASM, Rendimiento, Seguridad, DX y Clippy (26 tareas completadas)

**fix: clippy warnings (commit `b11c0e7`):** Se resolvieron las 22 advertencias de `dead_code` en el código scaffolding (PERF-02/07/08/10, SEC-05, vfile sigbus, ops auxiliares, wal recovery) mediante `#[allow(dead_code)]`. Se corrigió un type mismatch en `rkyv_archives.rs` (`Vec<Vec<u64>>` → `Vec<NeighborVec>`). `cargo clippy` ahora emite 0 warnings y 342/342 tests pasan.

Se completan 25 tareas en una gran tanda pre-lanzamiento que abarca 7 áreas críticas:

- **Framework Adapters (7):** MEM-02 (vantadb-letta), TSK-89 (vantadb-crewai), TSK-91 (vantadb-dspy), TSK-92 (vantadb-haystack), TSK-95 (vantadb-litellm), TSK-116 (vantadb-openai), TSK-117 (vantadb-ollama)
- **WASM (3):** WASM-03 (demo Transformers.js + OPFS), WASM-04 (bundle 394.5 KB gzip), WASM-05 (SIMD f32x4 cosine distance)
- **MCP (2):** MCP-04 (collection management tools), MCP-05 (25 tests)
- **Performance (6):** PERF-02 (Sharded WAL), PERF-04 (typed error variants), PERF-05 (module split), PERF-07 (edge index + referential integrity), PERF-08 (secondary scalar indexes), PERF-10 (memory governor + eviction metrics)
- **Developer Experience (3):** DX-01 (connect()), DX-02 (Python SDK latency — LRU cache, buffer reuse), DX-04 (55 TS tests)
- **Security (4):** SEC-04 (auth hardening — subtle::ConstantTimeEq, rate limiting, /metrics auth), SEC-05 (RBAC design), SEC-06 (SBOM workflow), SEC-07 (CodeQL + cargo-deny CI)

### 2026-07-02 — Pulido Frontend Web, Hardening de Seguridad, Estabilización MCP, Infraestructura Docker

- **Web tasks (6 completed):**
  - **WEB-15/WEB-16** — Refinamientos visuales del homepage (text-align left, H1 font-weight 700, fondo de Nav a warm paper)
  - **WEB-09** — Librerías de animación consolidadas: removido AnimeJS, toda la animación refactorizada a GSAP (~155KB+ de reducción de bundle)
  - **WEB-13** — URLs canónicas SEO, OG tags y datos estructurados JSON-LD en los 25 archivos de rutas
  - **WEB-12** — Creado componente reutilizable `<VsTable>` reemplazando 7+ implementaciones manuales de tablas
  - **WEB-10** — Code splitting con `React.lazy()` para 4 páginas pesadas (Engine, Architecture, Docs, Changelog)
  - **WEB-11** — Optimización con `React.memo` + `useMemo` en 10 componentes para prevenir rerenders innecesarios
- **Security (2 advisories verified resolved):**
  - **SEC-01** — Migración bincode 1.x→2.0 confirmada como ya completada (vía AUD-03 previo)
  - **SEC-02** — rustls-pemfile confirmado ya en v2
- **MEM-01** — Creado crate PyO3 `vantadb-mem0/` para la integración Mem0 VectorStoreBackend
- **MCP-02** — Servidor MCP estabilizado a readiness GA: config, error handling, timeouts, graceful shutdown, metrics, docs por IDE
- **DX-03** — Docker Compose "Local LLM Stack": Dockerfile + docker-compose.yml + .dockerignore
- **Compilación:** Rust pasa limpio (sin warnings/errors), TypeScript pasa limpio (con fix aplicado para dead code en archivos de rutas stripped)

### 2026-07-02 — Infraestructura de Testing, Persistencia WASM, Rendimiento Backend y Hardening de Seguridad (6 tareas)

- **WASM-02** — Persistencia OPFS (Origin Private File System) para vantadb-wasm. Habilita persistencia browser crash-safe sobre almacenamiento InMemory
- **WEB-07** — Infraestructura de tests frontend: Vitest + React Testing Library + Playwright E2E configurados con 23 component tests en 3 archivos
- **TEST-01** — Suite de tests WASM: 45 tests en `vantadb-wasm/tests/wasm_tests.rs` cubriendo embedding, search, persistence, error handling
- **TEST-02** — Component tests frontend: 23 tests en 3 archivos usando Vitest + RTL
- **TEST-03** — Suite de tests de seguridad: 30 tests cubriendo fuzzing de inyección IQL, intentos de bypass de auth, payloads malformados
- **PERF-01** — Cargador KV por lotes (`get_many`) en el trait StorageBackend. Eliminados 5 patrones N+1: graph.rs BFS/DFS, physical_plan.rs PhysicalScan, vector search post-filter, hybrid search explain
- **SEC-03** — Evolución del schema de almacenamiento físico: headers versionados, migration runner en la CLI vanta-cli
- **Verificación:** Rust compila limpio (sin warnings/errors), todos los tests pasan, TypeScript compila limpio
- **Backlog:** Backlog.md actualizado — tareas removidas de secciones activas, verdict scores actualizados

<!-- movido a ARCHIVO_HISTORICO.md -->
<!-- movido a ARCHIVO_HISTORICO.md -->
<!-- movido a ARCHIVO_HISTORICO.md -->

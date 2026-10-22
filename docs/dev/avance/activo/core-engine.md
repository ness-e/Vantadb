---
title: Avance — Core Engine
kind: review
status: active
description: "slen‖ns‖keylen‖key‖ver BE) + hooks put/putbatch/delete/purgeexpired + VantaConfig.versionhistorylimit (cap 32 FIFO aprobado, snapshot del record nuevo, import sin snapshots). API getversion/ ersions. Commits e0812a4/6997e59. Doble..."
tags: [vantadb, avance, core, engine, storage, wal, hnsw, acid]
---

# Avance — Core Engine

> Registro consolidado del trabajo completado sobre el motor: storage, WAL, índices vectoriales (HNSW/IVF/flat), planificación/ejecución de queries, ACID y rendimiento. **IDs originales conservados.**

## Cobertura rápida

- **Índices:** HNSW (persistencia, auto-tuning ef, sharded-slab, flat threshold), IVF Flat (DRV-131), trait `VecIndex` (COMP-008), routing inteligente (COMP-028 → OLD-21).
- **ACID:** Fase 1 (WAL Begin/Commit/Abort), Fase 2 (escrituras reversibles VantaFile), Fase 3 (MVCC snapshot isolation, VFY-011), rollback multi-capa (diseño INV-010).
- **WAL:** micro-batching, sharded, fsyncs paralelos, PITR, shipping.
- **Query engine:** IQL parser con JOINs/subqueries (DRV-122), planner CBO (DRV-121), cost estimator (COMP-028), auto-embedding (COMP-010, DRV-123).
- **u128 migration:** CODE-067 (XxHash3_128).

---

## Vanta Memory Engine (P27, 2026-08-20)

### MEM-01..21 (+08a/08b, 34, 35): Campaña P27 completa — F1-F4
- **Fecha:** 2026-08-18 → 2026-08-20
- **Objetivo:** Port de TDAM a VantaDB — search profile por request (IQL `PROFILE`), entidades entity_* + RBAC allow-only, auth 3 capas + audit JSONL, skills multi-versión + tools MCP, y crate nuevo `vanta-memory/` (L0 capture → L1 extract/dedup → L2 escenas → L3 persona → recall 3 modos → offload → gateway), LLM-driven vía trait host-neutral `LlmRunner` con degradación LLM-free (Principio 4).
- **Resultado:** ✅ 24/24 tareas. Suite final 361/361 tests en vanta-memory; fmt/clippy `-D warnings` limpios. Registro completo: `docs/progreso/campanas/p27-memory-engine.md` (split GOV-D2). Plan: `docs/dev/plans/archive/2026-08-18-vanta-memory.md`.
- **Ids:** `MEM-01`..`MEM-21`, `MEM-34`, `MEM-35`

---

## Índices vectoriales

### COMP-008: Motor de índice enchufable (trait VecIndex)
- **Fecha:** 2026-07-27
- **Objetivo:** Abstraer operaciones de index vectorial (HNSW, IVF, flat) detrás de un trait `VecIndex` para desbloquear múltiples backends.
- **Resultado:** ✅ Trait con `search`/`add`/`len`/`estimate_memory_bytes`. Implementado para `CPIndex` (HNSW) e `IvfIndex`. `vector_memory_search` usa trait object. Fix a `vantadb-mcp` por rotura de COMP-006. 1679 tests pasan. Clippy `-D warnings` limpio en workspace.

### DRV-131: IVF Flat (índice más allá de HNSW)
- **Fecha:** 2026-07-26
- **Resultado:** ✅ `src/index/ivf.rs` (NEW, 836L): k-means (Forgy + Lloyd, max 20 iter), search con nprobe, serialización v8 con backwards compat v7. 16 tests IVF. 1547 tests lib pasan. 0 clippy warnings nuevos.

### NUEVO-06 / VFY-004: Umbral de índice plano <10K brute-force
- **Resultado:** ✅ `flat_threshold` en `VantaConfig` (env `VANTADB_FLAT_THRESHOLD`, default 10000) + builder `with_flat_threshold()`. Wired config → HnswConfig → CPIndex; flat search dispatch en `graph.rs::search_layer()`. Tests `flat_search_matches_hnsw_on_small_dataset`, `flat_search_used_when_under_threshold`. VFY-004: O(n²) del filter en `flat.rs` es **by design** (DashMap scan acotado por threshold) — comment-only `dd13b67d`.

### NUEVO-13: Auto-tuning de ef_search HNSW
- **Fecha:** 2026-07-26
- **Resultado:** ✅ Heuristic doubling, dampening 2.0→1.5x, gauge `vantadb_auto_tune_ef`, integration test `repeated_fallbacks_increase_ef`. +66/−9 en `src/index/auto_tune.rs`, `metrics/core/mod.rs`, `metrics/core/registry.rs`.

### COMP-026: Multi-level LSM Compaction (L0→L1→L2→L3)
- **Fecha:** 2026-07-28
- **Resultado:** ✅ SegmentRegistry, `compact_level()`, PipelineMode extendido. L0+L1 implementados (ponytail: L3 archive diferido). 13+ archivos modificados.

### NUEVO-17: Segment LSM tiers hot/warm/cold + archive
- **Fecha:** 2026-08-02
- **Resultado:** ✅ Infra de niveles ya existía (`src/lsm.rs` `SegmentLevel` L0-L3); gap era la *política de tier* y archive L3. `TierPolicy` enum (SizeBased | FrequencyBased | AgeBased) + `TierPolicyConfig` (archive on/off, cold_min_access, cold_age_days); `LsmConfig` extendido (`l3_max_size`, `l3_tombstone_threshold`, `tier`). Promoción encadenada en `compact_level`: L0(hot)→L1(warm)→L2(cold)→L3(archive), L3 terminal y solo si `tier.archive=true`. Tests `test_tier_promotion_hot_to_cold`, `test_tier_promotion_cold_to_archive`, `test_tier_archive_disabled_stops_at_cold` 3/3. Doc `docs/dev/architecture/STORAGE-TIERS.md` (EN).

### COMP-013: Pipeline de optimización de segmentos (Vacuum/Merge/Index)
- **Fecha:** 2026-07-27
- **Resultado:** ✅ `PipelineMode` (Full/VacuumOnly/MergeOnly/IndexOnly), `vacuum()` purga tombstones HNSW, `merge_segments()` compactación BFS condicional, `run_pipeline()` orquestación secuencial con tolerancia a fallos por fase. `SegmentOptimizerConfig` en `VantaConfig`. SDK expone `vacuum()`, `pipeline()`, `optimizer_config()`, `set_optimizer_config()`. 77 tests de mantenimiento pasando.

### COMP-014: FreshHNSW (Background Repair de Enlaces Huérfanos)
- **Fecha:** 2026-07-27
- **Resultado:** ✅ `repair_orphan_links()` de tres fases (snapshot→scan→repair) evita deadlock de DashMap. `FreshHnswReport`, `PipelineMode::FreshHnswOnly`, fase de pipeline entre Vacuum y Merge. 4 tests (empty, no-orphans, after-delete, multi-layer).

### COMP-006: Edge Label Interning (u32 label_id)
- **Fecha:** 2026-07-27
- **Resultado:** ✅ `Edge.label: String` → `Edge.label_id: u32` con LabelIntern (HashMap<String, u32>). Reduce ~80MB heap para 1M edges. 1618 tests pasan. SDK público inalterado (VantaEdgeRecord.label sigue String).

### COMP-021: Aristas temporales (relaciones con timestamp)
- **Fecha:** 2026-08-02
- **Resultado:** ✅ `Edge.created_at_ms: u64` en `src/node.rs` (wall-clock en `new`/`with_weight`/`reverse`; helper `Edge::with_timestamp`). Custom `Deserialize` manual para `Edge` — hallazgo: postcard 1.1.3 NO consulta `#[serde(default)]` (visitor trata fin de buffer del campo nuevo como `0`, preservando lectura de datasets pre-feature). `bfs_traverse_filtered`/`dfs_traverse_filtered` con `time_range: Option<(u64,u64)>`. `add_edge(..., created_at_ms)` en SDK + bindings Python/WASM/TS. 1672 lib + 6/6 temporal_edges (backward-compat postcard, roundtrip, window filtering, forward+reverse persistence).

### COMP-018: Double-linked Relationship Chains
- **Fecha:** 2026-07-28
- **Resultado:** ✅ Relations dirigidas con doble enlace. `Edge.reverse` + add_edge/remove_edge bidireccional + `TraversalDirection` + direction param en Rust SDK (4 métodos), bindings WASM y Python. 33 graph tests pasan. Backward compatible (default Forward).

### PERF-27 / PERF-17 / PERF-18 / PERF-23 / PERF-28: HNSW tuning & correctness
- **Resultado:** ✅ PERF-27 select_neighbors heuristic diversity (tombstone filtering, borrows `&[f32]`); PERF-17 ef_construction 200→400; PERF-18 M/max0 16→32/64; PERF-23 HNSW ep_enter freeze fix (`find_new_entry_point()` promueve reemplazo tras delete); PERF-28 tombstone mitigation (saltar nodos eliminados en search_layer + WAL replay zombie fix).

### COMP-001/002/003/004/005/007/011/015/020/030 (P10 Competitive catalog)
- **Estado:** ✅ Catalogado y decisiones registradas en `historial/backlog-history.md` → P10. Incluye SQ8/PQ, HNSW persist, in-filter, bitset, params, inline u128, CRUD tombstones, hybrid pipeline, RRF fusion, survival mode.

---

## ACID & Durabilidad

### VFY-011: ACID Fase 3 — MVCC/Isolation de Snapshot
- **Fecha:** 2026-07-26
- **Resultado:** ✅ `Snapshot { txn_id: u64 }` + `get_with_snapshot()` filtrado MVCC. `active_txns: HashSet<u64>` reemplaza `active_txn_id: Mutex`. Detección de conflictos write-write vía `check_write_conflict()`. 7 tests nuevos. `created_by_txn`/`deleted_by_txn` en NodeMetadata. Archivos: `ops.rs`, `mod.rs`, `init.rs`, `storage/ops.rs`.

### P3 (TASK-30): Capa de Transacciones ACID Fase 1 (WAL Transaction Records)
- **Fecha:** 2026-07-13
- **Resultado:** ✅ `Begin(u64)`, `Commit(u64)`, `Abort(u64)` agregados a `WalRecord` en `src/wal.rs:57-61`. Engine expone `begin_transaction()`, `commit_transaction()`, `abort_transaction()` en `ops.rs`. Recovery (`init.rs`) usa skip_mask de dos pasadas: descarta writes de transacciones abortadas/no cerradas. 576/577 tests pasan. VantaFile rollback diferido a P4.

### P4 (TASK-31): Escrituras reversibles de VantaFile
- **Fecha:** 2026-07-13
- **Resultado:** ✅ `insert()`/`batch_insert()` en `ops.rs` tombstones el entry de VantaFile si el KV write falla, previniendo vectores huérfanos. `delete()`/`delete_batch()` ya tombstoneaban antes del KV delete. 576/577 tests pasan.

### P1 (TASK-29): HNSW insert_lock micro-batching
- **Fecha:** 2026-07-13
- **Resultado:** ✅ Pending batch buffer (64 ops) en `ops.rs`: `PendingHnswOp`, `flush_pending_hnsw()`, `try_push_pending_hnsw()`. `insert()` push → pending batch → flush bajo lock único (64x menos adquisiciones). `batch_insert()` ya óptimo. `delete()`/`delete_batch()` mantienen sync. Commits `141e628`, `3a52180`.

### P2 (TASK-28): WAL Mutex contention
- **Fecha:** 2026-07-13
- **Resultado:** ✅ ShardedWal ya usado en TODOS los paths de escritura. WalWriter directo solo en tests. Se removió `#[allow(dead_code)]` stale, se fixeó `rotate_all()` para preservar buffer_size/flush_threshold. Commit `fc28768`.

### WEB-03: Async WAL batching fsyncs
- **Fecha:** 2026-07-25
- **Resultado:** ✅ `flush_all` fsync paralelo por shard en `src/wal_sharded.rs` (`c59e0f80`). Short-circuit de shard único. 25/25 tests wal_sharded.

### CODE-067: Migración u64→u128 (XxHash3_128)
- **Fecha:** 2026-07-11
- **Resultado:** ✅ node_id u64→u128 en ~30 archivos. `DiskNodeHeader.id` u128 (VECTOR_INDEX_VERSION incrementado), SDK types u128, `DuplicatePrevention` interfaz pública u128 (hash interno bloom sigue XxHash64 — decisión deliberada), rkyv formato 8→9. 444 tests pasando.

---

## Query engine & planner

### DRV-122: JOINs IQL, Subqueries y Compatibilidad SQL
- **Fecha:** 2026-07-26
- **Resultado:** ✅ 3 fases: parser SELECT/JOIN/subquery + plan types (`de189a8c`), operadores físicos NestedLoopJoin + SubqueryFilter (`345d1939`), integración con planner + 10 tests nuevos (`6449469f`). 1559 tests pasan.

### DRV-121: Optimización CBO del Planner
- **Fecha:** 2026-07-25
- **Resultado:** ✅ Predicate pushdown (orden por selectividad) + eliminación de identity filters (sel≥1.0 omitido). Constants `HIGH_SELECTIVITY_THRESHOLD`. Test para eliminación de identity filters.

### COMP-028: Semantic Cost Estimator (SCE) unificado
- **Fecha:** 2026-08-02
- **Resultado:** ✅ `src/cost_estimator.rs` (nuevo): `CostEstimator<'a>` con `selectivity()`, `estimate_operator()`, `estimate_plan()`, `select_filter_strategy()`/`FilterStrategy` movidos desde `sdk/search/mod.rs`. `StorageEngine::get_estimated_selectivity` conserva firma pública y delega (21 callers intactos). 4 tests unitarios. 1776 passed.

### OLD-21: CP-Index formal (query routing inteligente)
- **Fecha:** 2026-08-03
- **Resultado:** ✅ `CostEstimator::select_index_strategy()` (heurística: Flat si `nodes <= flat_threshold`; IVF si `nodes >= 10_000`; HNSW default; respeta config explícita no-HNSW). Conectado en `vector_memory_search` + métrica `record_vector_index_routing`. Admission budget de `Executor::execute_plan` usa `ResourceGovernor::estimate_plan_cost`. 6 tests nuevos. 1816 passed, 2 skipped.

### COMP-010: Abstracción de la función de auto-embedding
- **Fecha:** 2026-07-27
- **Resultado:** ✅ Trait `EmbeddingProvider` + `OllamaProvider` + `OpenAIProvider` + factory `get_embedding_provider()` lee `VANTA_EMBEDDING_PROVIDER`. 4 call sites actualizados. `LlmClient` preservado para `summarize_context()`.

### DRV-123: Pulido de Auto-embedding en INSERT
- **Fecha:** 2026-07-25
- **Resultado:** ✅ `match` reemplaza `if let Ok`, `tracing::warn!` en fallo. Guard de texto vacío `!text.trim().is_empty()`. Test `test_auto_embedding_graceful_degradation_on_insert`.

---

## Refactor & fixes core (DRV)

### REVIEW-04: Split god modules node.rs + vfile.rs
- **Fecha:** 2026-08-12
- **Resultado:** ✅ `node.rs` 2078L → 8 submódulos (`bitset,vector_data,label,edge,field,flags,disk,unified`) + mod.rs facade con re-exports byte-idénticos (lib.rs:157-160); `vfile.rs` 1309L → `vfile_mmap.rs` (shim+AlignedBytes+SIGBUS) + VantaFile ~490L. unsafe 30 preservados con `// SAFETY:`, tests 64+32 sin pérdida. `config.rs` NO se parte (assessment ponytail en header: cohesive leave-as-is). Commit `d5624082`.

### P2-7: Serialización zero-copy del sparse vector (formato persistido)
- **Fecha:** 2026-08-12
- **Resultado:** ✅ ADR-0019: sparse se persiste como `FieldValue::ListFloat(Vec<f64>)` con pares intercalados `[dim, val]` (lossless u32→f64/f32→f64, orden determinista por BTreeMap) en vez de `FieldValue::String(serde_json)` bajo `SPARSE_VECTOR_EXT_KEY`. Write path `sparse_vector_to_field` sin serde_json (elimina ~1.49% del hot-path de búsqueda); read path dual: `ListFloat` decode directo + `String` legacy para compat backward; faltante → `None` (PERF-07); corrupto → warn + `None`. `VantaMemoryRecord.sparse_vector` público intacto. Sin migración one-shot (lazy en próximo put); shim legacy hasta gate de versionado de storage. 1885/1885 tests + clippy `-D warnings` + fmt --check ✅. Review P2-01 APPROVE. Commit `2f1a94e1`.

### REVIEW-05: Split god files serialize.rs + distance.rs + physical_plan.rs
- **Fecha:** 2026-08-12
- **Resultado:** ✅ `serialize.rs` 1595L → `src/index/serialize/{mod,bytes,file}.rs` (impl CPIndex dividido por concern: bytes/file); `distance.rs` 1721L → `src/index/distance/{mod,kernels,metrics,mapper}.rs` (SIMD f32x8/f32x16 y métricas byte-idénticos, dispatch de calculate_similarity preservado, items `pub(crate)` cross-módulo); `physical_plan.rs` 1542L → `src/physical_plan/{mod,scan,filter,vector,project,sort,join}.rs` (10 operadores, `evaluate_condition` pub(crate) solo para tests). Re-exports `index/mod.rs:22` (`pub use distance::*`) y `lib.rs:110` intactos; API pública removed=[] added=[]; 1878/1878 tests (nextest audit) + clippy `-D warnings` + fmt --check ✅. P2-7 (zero-copy serialization) diferida, no se mezcla con el refactor. Commit `92852f9f`.

### DRV-005: Tests unitarios del SDK search/mod.rs
- **Fecha:** 2026-07-24
- **Resultado:** ✅ 18 tests agregados para `search()`, `lexical_search()`, `vector_memory_search()`, `hybrid_search()`. Blocker: `src/vector/quantization.rs:199` (engine domain).

### DRV-007: Data race en filter_field() (scalar_index sin lock)
- **Resultado:** ✅ `let _nodes = self.nodes.read()` antes de `self.scalar_index.lookup()` (happens-before con writers). 1-line fix.

### DRV-006: Race condition en delete()
- **Resultado:** ✅ Remove `drop(nodes)` en `InMemoryEngine::delete` — RwLockWriteGuard cubre index cleanup. 210/211 tests pasan.

### DRV-008 / DRV-011 / DRV-015: DRY violations & monolitos
- **Resultado:** ✅ DRV-008 extrae `collect_scores()` (engine.rs:288-305 vs :399-413); DRV-011 extrae `try_scan_forward()` (WalWriter::open + WalReader::next_record); DRV-015 refactor `WalWriter::open_with_buffer()` de ~100L a ~55L con `recover_valid_records()`.

### DRV-126: Paginación keyset
- **Resultado:** ✅ RESUELTO — SearchResults ya implementa paginación keyset + offset-based en `src/sdk/search/mod.rs`. Skip.

### VFY-003: Paginar reindex_hnsw_from_text — fix de OOM
- **Fecha:** 2026-07-24
- **Resultado:** ✅ Commit `918df85`. SDK api + Python/WASM/TS bindings.

---

## Correcciones de audit del motor (CODE-*)

| ID | Tarea | Commit |
|---|---|---|
| CODE-001 | WAL replay no escribe backend metadata → fixed | — |
| CODE-002 | WAL append antes de validación → audited (no hay fantasma) | — |
| CODE-003 | process::exit(1) → graceful shutdown + WAL flush | — |
| CODE-007 | Tombstone check bypass en HNSW insert | `d25f91e` |
| CODE-008 | HNSW nunca elimina nodos de CPIndex | `d25f91e` |
| CODE-009 | save_vector_index() traga errores → `Result<()>` | — |
| CODE-010 | Compact layout tmp file huérfano | `d25f91e` |
| CODE-024 | scan_nodes OOM | `d25f91e` |
| CODE-026 | BFS order vacío destruye DB en compact → ValidationError | — |
| CODE-027 | get_many() `.expect()` → `map_err` BackendError | — |
| CODE-029 | Read lock en todo search pipeline | `d25f91e` |
| CODE-030 | NaN en cosine_similarity | `d25f91e` |
| CODE-064 | serialize_to_bytes Vec gigante | `d25f91e` |
| CODE-065 | estimate_memory_bytes O(n) en cada insert | `d25f91e` |

### CODE-092 (Post-Benchmark Deep): Distancia Euclidea invertida
- **Fecha:** 2026-07-06
- **Resultado:** ✅ **Bug crítico:** `squared_distance` raw vs `1.0 - similarity` causaba ordenación invertida. Recall@10 55.7% → ~90% (paridad ChromaDB). Fix estimado 1 hora.

---

## Rendimiento (PERF) — resumen

| ID | Tarea | Estado |
|---|---|---|
| PERF-15 | PyBuffer zero-copy batch | ✅ |
| PERF-16 | #[pyclass] search hits/list | ✅ |
| PERF-19 | WAL batch append | ✅ |
| PERF-20 | Storage batch insert | ✅ |
| PERF-21 | AVX-512 f32x16 SIMD dispatch | ✅ |
| PERF-22 | SQ8 euclidean vectorization | ✅ |
| PERF-24 | GIL scope optimization | ✅ |
| PERF-25 | PyDict object pool | ✅ |
| PERF-26 | Lazy serialization | ✅ |
| PERF-29 | Cosine→Euclidean mapping (MetricMapper/MetricCache) | ✅ |
| PERF-30 | Config tuning (batch_size, wal_buffer_size, flush_threshold) + auto-flush | ✅ |
| PERF-31 | NumPy output batch (zero-copy `__array_interface__`) | ✅ |
| PERF-32 | Async ingestion pipeline (4 workers) | ✅ |
| PERF-33 | HNSW graph prefetching | ✅ |
| AUD-040 | WAL `batch_append` sin alloc por record: `to_allocvec` → Vec reutilizable + `postcard::to_io`; framing byte-idéntico + test regresión; postcard feature `use-std`. Commit `a5001f4d` | ✅ 2026-08-16 |
| AUD-041 | Bench `sparse_hot_path`: +2 arms ListFloat (P2-7) — `listfloat_encode_one`/`listfloat_decode_one`; arms serde_json intactos para critcmp. Commit `ec4eaeff` | ✅ 2026-08-16 |
| PERF-34 | Extended norm caching (HNSW_VERSION 10) | ✅ |
| PERF-35 | Async transcript I/O | ✅ |
| PERF-36 | Config hot-reload | ✅ |
| PERF-37 | FilterBitset reduction (and_fast/or_fast/count_set_bits) | ✅ |
| PERF-38 | Multiversion dispatch (DistanceKernels) | ✅ |

### MOD-04: purge_expired selectivo vía scalar index TTL
- **Fecha:** 2026-08-25
- **Objetivo:** eliminar el full-scan O(N) de `purge_expired` usando el `ScalarIndex` existente (que ya indexa `__vanta_expires_at_ms` en writes) con range lookup `lookup_int_le`, reconstruyéndolo en open/rebuild.
- **Resultado:** ✅ bench before/after `cargo bench --bench purge_expired` (4k records, 128d): 100 expirados 137.20→117.22ms (−23.9%), 1000 expirados 1.2341→1.0726s (−9.1%), p<0.05. Iteración con `engine.get()` por candidato regresaba (+11%/+24%) → reemplazada por lectura metadata-only del backend. nextest workspace 2763/2763, clippy/fmt clean. Hallazgo colateral FIND-31 (text index df negativo tras reopen, pre-existente) registrado en Backlog.
- **Archivos:** `src/scalar_index.rs`, `src/sdk/api.rs`, `src/storage/engine/{init,mod}.rs`, `src/storage/engine/tests/init.rs`, `benches/purge_expired.rs`, `Cargo.toml`
| ERR-036 | Write-lock en hot path de `get()` → `try_write()` + degradación a `read()` (nunca bloquea writer); hits preservados en uncontended; commit `e6cbc93f` | ✅ 2026-08-11 |
| ERR-042 | `read_header` 2× por candidato en hot loop (+ entry points) → `node_header` 1× reutilizado en distance + tombstone; fix `e95dd94a`; 2 tests paridad (commit `5a9eada1`); bench −11.4%/−19.0% | ✅ 2026-08-11 |
| ERR-043 | `shrink_neighbors` clonaba vector del nodo (`as_f32_slice().map(to_vec)`) solo para query → `compute_shrunk_neighbors` lee slice prestado (`as_f32_slice()`); fix `2a20b14a`; 3 tests shrink/paridad; nextest 1902/1902 | ✅ 2026-08-11 |
| ERR-031 | `VecIndex::add` traga rechazos (solo warn) → trait retorna `Result<()>`, 5 impls propagan rechazos (non-full DiskAnn/Scann, read-only IVF, zero-norm CPIndex); fix `339107b0` + colateral clippy `918e57b1`; 3 tests rechazo `f585e423` | ✅ 2026-08-12 |
| PERF-02 | Baseline riguroso criterion determinista + critcmp regression gate (nightly); dataset sintético persistido | ✅ 2026-08-12 |
| PERF-03 | Bench competitivo honesto SDKs (Qdrant/Chroma/Lance/Milvus) en mismo HW; tabla en `docs/user/benchmarks/COMPETITIVE_SDK_BENCH.md` (VantaDB pierde recall, gana QPS) | ✅ 2026-08-12 |
| PERF-05 | WAL async roadmap (ADR `ADR-0052-wal-async-roadmap.md`): io_uring/aio + fsync group commit post-DRV-014 | ✅ 2026-08-12 |
| PERF-08 | WASM hot path: `Float32Array` zero-copy para vectores en `memory_record_to_js` (cierra P2-7) | ✅ 2026-08-12 |

> Nota de **R6** (bitacora): algunos PERF de esta lista se evaluaron como premature en `VantaDB_ANALISIS_COMPLETO.md` Sección 3.1, pero quedaron implementados en las waves de julio. Ver `decisiones/wontfix.md`.

---

## Otros completados del motor

### COMP-009: Importación masiva binaria (.vdbdump)
- **Fecha:** 2026-07-27
- **Resultado:** ✅ Formato `.vdbdump` (magic `VDBJSON\n` + version + count + serde_json body), `bulk_import_stream()`, `bulk_import_file()`, `bulk_commit_interval` en config. Python y WASM con wrappers `bulk_import()`/`bulk_import_bytes()`. 3 tests.

### MOD-03: `trigger_compaction()` deja de ser stub
- **Fecha:** 2026-08-24
- **Objetivo:** El método público contaba tombstones y solo logueaba "offline compaction triggered" sin compactar (core.md M-1).
- **Resultado:** ✅ Delega a `merge_segments()` (threshold configurado `vacuum_threshold_pct`, default 15%, antes 20% hardcodeado) que reescribe el VantaFile vía `compact_layout_bfs()`. API estable (`Result<()>`). Test nuevo disk-backed bytes-reclaim RED→GREEN. Suite 2052/2052.
- **Commit:** `28ce0a57`

### TSK-107b: Audit logging enterprise (JSONL, timestamp + op)
- **Fecha:** 2026-08-02
- **Resultado:** ✅ `src/audit.rs`: `AuditEvent` + `AuditLogger` (append+flush por registro). `VantaConfig.audit_log_path` + env `VANTADB_AUDIT_LOG_PATH`. Hooks en put, put_batch, delete, delete_by_filter, export_namespace, export_all, import_file — todos vía `VantaEmbedded`. No-op si no está configurado. 1772 passed, 2 skipped; `tests/audit_log.rs` 3/3.

### ENT-04: Pool de conexiones + circuit breaker (server-mode)
- **Fecha:** 2026-08-02
- **Resultado:** ✅ Pool de conexiones + circuit breaker en server-mode. (Detalle en snapshot-2026-08-07.)

### GH-124: Ejemplos doc-test para API pública Rust
- **Fecha:** 2026-08-02
- **Resultado:** ✅ 7 doc-tests nuevos. Se repararon 2 doc-tests rotos pre-existentes. `cargo test --doc -p vantadb` 11/11 pass.

### GH-127: Property-based tests WAL roundtrip
- **Fecha:** 2026-08-02
- **Resultado:** ✅ `tests/proptest_wal_roundtrip.rs`: 4 tests (bytes puro 1000 casos, payload buckets, file roundtrip batch, concurrent writes). nextest 4/4.

### REC-007: WAL Compaction + Vacuum CLI
- **Fecha:** 2026-07-29
- **Resultado:** ✅ Exponer `compact_wal()`/`vacuum()` como `vanta-cli wal compact` / `vanta-cli wal vacuum`. Binding directo.

### REC-999 / SDK-02 / SDK-04
- **Resultado:** ✅ SDK-02 (`similar_to_key()` — ✅ 2026-07-31), SDK-04 (`search_multi`/`search_all` — ✅ 2026-07-31). REC-999 progreso fix.

### REV-003: Campaña de cobertura 53.85% → 80.55% (CII Silver)
- **Fecha:** 2026-07-23
- **Resultado:** ✅ 14 batches, +728 tests (764→1492). Line coverage 53.85%→80.55%. CI threshold 76%→80%. Fix: SQ8 format bug en ops.rs get(). CLOSED. 23 archivos tocados.

### REV-004: Fix de rlib de tantivy en vantadb-openai
- **Fecha:** 2026-07-14
- **Resultado:** ✅ `"rlib"` al `crate-type` de `vantadb-openai/Cargo.toml` (los binarios de test necesitan rlib para linkear).
- ERR-032 (storage test), ERR-047 (search Cow), ERR-048 (search visited), ERR-008 (vfile copy_unsafe obsoleto), ERR-049 (ivf bench) — migrados 2026-08-12 (ver docs/progreso/README.md)
- COV-003 (CLI subcommand tests migrate/server/crud, +7 tests, cli_handlers ~0%→~76.5%) — migrado 2026-08-12 (ver docs/progreso/README.md)

### AUD-025: BM25 zero-alloc hot path (per-posting allocations) — migrado 2026-08-14 (ver docs/progreso/README.md)
- **Resultado:** ✅ `src/text_index.rs:565` `posting_record_key` → `&str` zero-alloc (`strip_prefix` + `from_utf8`); `src/sdk/search/phrase.rs` matcher genericizado (`K: AsRef<str> + Ord`, helper `find_positions`); `src/sdk/search/mod.rs:383-448` hot path sin `token.clone()`/`String::from`/`format!` por posting, `doc_stats_cache` keyed por `u128 node_id` con guard de mismatch. `cargo check -p vantadb` ✅, clippy ✅, fmt ✅, 104 tests (13 phrase + 91 search) ✅. Commit `96b258ba`.

### FND-15: Crash recovery / WAL en la práctica (verificación vanta-chaos) — migrado 2026-08-16 (ver docs/progreso/README.md)
- **Resultado:** ✅ verificación en `docs/dev/research/FND-15-crash-recovery-verificacion.md`: kill a mitad de escritura recupera estado consistente (`chaos_integrity`/`wal_resilience`) — sin gap de producto, gap de infra de tests documentado. Commit `8c6044a1`.

### FND-20: Trade-off HNSW (ef_search/M: recall vs latencia) + argumento vs IVF/FAISS — migrado 2026-08-16 (ver docs/progreso/README.md)
- **Resultado:** ✅ `docs/dev/architecture/FND-20-hnsw-tradeoff.md` (EN): parámetros actuales (M=32, ef=100) citados (`graph.rs:255-269`, `nearest.rs:71-77`, `neighbors.rs:57-62`, `ivf.rs:79-228`, `auto_tune.rs:11-53`), trade-off recall/latencia/memoria, sección "Why not FAISS/IVF" para local-first. Drift documentado: ADR 005/PERFORMANCE_TUNING.md dicen ef_construction=200/400, código dice 100 — la nota cita el código como fuente de verdad. Commit `4051a850`.

### FND-01: Regla de presupuesto de memoria + benchmark OOM — migrado 2026-08-16 (ver docs/progreso/README.md)
- **Resultado:** ✅ 🔴 CONFIRMADO: RSS sin límite real, guard subestima ~6.5× bajo carga 10k w/s + 1k r/s. Regla must en `.opencode/rules/memory-budget.md`. Follow-ups F1/F4 delegados a core-engine. Commit `a159211b`.

### FND-02: Regla de coordinación multi-índice + auditoría de deadlocks — migrado 2026-08-16 (ver docs/progreso/README.md)
- **Resultado:** ✅ fix deadlocks evicción multi-índice (lock no reentrante + write guard, `c104f1f2`) + regla en `.opencode/rules/concurrency-async.md` + audit P2-01 approve; follow-up lock order `93a1e311`. Follow-ups menores 2/3 delegados a core-engine.

### FND-08: Regla de backend validado contra patrón de acceso real + auditoría de compactación — migrado 2026-08-16 (ver docs/progreso/README.md)
- **Resultado:** ✅ ADR-0023 (backend compaction — diferir marginal, bench de lectura) + regla en `.opencode/rules/durability.md`. Commit `e5e76684`.

### VS-CORE-07: Retención de versiones históricas (D2 completo)
- **Fecha:** 2026-08-18
- **Resultado:** ✅ src/sdk/version_history.rs (nuevo, 11 tests) + partición Fjall Versions (clave 
s_len‖ns‖key_len‖key‖ver BE) + hooks put/put_batch/delete/purge_expired + VantaConfig.version_history_limit (cap 32 FIFO aprobado, snapshot del record nuevo, import sin snapshots). API get_version/ersions. Commits e0812a4/6997e59. Doble consumidor P26+P27. 1785 lib tests verdes (1 fallo preexistente maintenance.rs fuera de scope).

### REVIEW-09: Latch `saturated` monotónico en cache_warmer — warming vuelve a aprender tras decay
- **Fecha:** 2026-08-23
- **Fuente:** Plan `docs/dev/plans/2026-08-23-backlog-triage.md` (Wave 1) · Backlog · review-full-20260822 H09-CODE-001
- **Resultado:** ✅ `record_co_access` seteaba `saturated=true` al cruzar `max_pairs` pero `decay()` reducía la tabla sin resetearlo → en servers long-running el warmer quedaba en refresh-only para siempre. Fix: `decay()` levanta el latch cuando el post-decay total < `max_pairs` (≤1 transición por cruce; sin thrashing verificado con decays que mantienen post-total ≥ cap). TDD: RED reprodujo el bug, 2 tests nuevos (ciclo completo + no-thrash), módulo 11/11. Verify full: fmt ✅ · clippy `-D warnings` ✅ · nextest audit workspace 2714/2714 ✅. Review P2-01 (vanta-review) aprobó tras corregir doc deshonesta del campo `saturated`. Commit `8b8924b3`. (ver `.opencode/skills/campaign-executor/tasks/REVIEW-09.md`)

### REVIEW-14: Keys cortas en version_history devuelven error de corrupción sin panic — unwraps frágiles eliminados
- **Fecha:** 2026-08-23
- **Fuente:** Plan `docs/dev/plans/2026-08-23-backlog-triage.md` (Wave 2) · Backlog · review-full-20260822 H09-CODE-005
- **Resultado:** ✅ DISCOVERY reclasificó el hallazgo: el slice frágil de `version_history.rs:283` era test-only (ningún path de producción parsee keys del partition Versions) y `VantaError::Corrupt` no existe (clase corrupción del codebase = `BackendError`; añadir variante rompería matches exhaustivos de bindings P2-6). Fix real: helper `version_from_key()` (`Result<u64>`, `BackendError` si key <8B — key corta solo puede venir del store, nunca de input de usuario) + validación en `versions()` como defensa ante store corrupto + test nuevo (len 0..7 → error tipado, boundary 8B OK). explain.rs: 3 unwraps estructurales eliminados bindeando `Some(query_sparse)` en los patterns (Keyword nullifica la Option, no solo un bool — semántica idéntica arm-por-arm). Siblings con mismo patrón (mod.rs/debug_ops.rs) derivados a FIND-27. Verify full: fmt ✅ · clippy `-p vantadb -D warnings` ✅ · nextest `-p vantadb` 2051/2051 ✅. Commit `4044a588`. (ver `.opencode/skills/campaign-executor/tasks/REVIEW-14.md`)

### MOD-01: WAL escrito ANTES de validar — insert/update rechazado resucita datos tras restart
- **Fecha:** 2026-08-23
- **Fuente:** Plan `docs/dev/plans/2026-08-23-backlog-triage.md` (Wave 1) · Backlog · core.md H-1 · cross-modulos F-1 (expuesto vía WASM)
- **Resultado:** ✅ Los 3 mutadores de `InMemoryEngine` escribían al WAL antes de validar: una op rechazada quedaba loggeada y el replay (`with_wal`) la aplicaba incondicionalmente como upsert — duplicado rechazado pisaba el payload original y update sobre nodo eliminado lo recreaba tras reopen. Fix: orden validate→WAL→apply bajo UNA sección crítica `nodes.write()` en insert/update/delete (double-checked descartado: deja ventana TOCTOU no retractable en append-only). Discovery mapeó TODOS los write paths: StorageEngine/batch/bulk/txn no violan el invariante (upsert por diseño, guards previos al WAL, buffer solo commitea validado) — H-1 vivía solo en el motor legacy. TDD: RED reprodujo ambas resurrecciones mecánicamente, 4 tests de durabilidad nuevos (reject×2 + flujo legítimo×2 con reopen), GREEN 4/4. Verify full: fmt ✅ · clippy workspace `-D warnings` ✅ · nextest `-p vantadb` 2049/2049 ✅ · audit workspace 2718/2718 ✅ · docs coverage 0 gaps ✅. Commit `18fd2c80`. (ver `.opencode/skills/campaign-executor/tasks/MOD-01.md`)

### FIND-27: Provider Ollama postea a endpoint legacy /api/embeddings - fix a /api/embed (2026-08-24)
- **Fecha:** 2026-08-24
- **Plan:** `docs/dev/plans/2026-08-24-batch-review-mod-find.md` (Wave 0)
- **Resultado:** OK - el provider Ollama nativo posteaba a `/api/embeddings` (endpoint legacy, campo `prompt`) en vez de `/api/embed` `{model,input}` vigente, fallando contra Ollama actual. Fix: endpoint + response shape (`embeddings[0]`), con 2 tests contra mock HTTP (endpoint correcto + rechazo de embeddings vacios). Verify: `cargo nextest run -p vantadb --features remote-inference llm::` 2/2 PASS. Commit `447a07d7`.

### MOD-02: Replay de transacciones no crash-atomicas - fix recover_state bounds skip al extent de la txn (2026-08-24)
- **Fecha:** 2026-08-24
- **Plan:** `docs/dev/plans/2026-08-24-batch-review-mod-find.md` (Wave 0)
- **Resultado:** OK - root cause: (1) tests previos usaban BackendKind::InMemory que jamas llama recover_state; (2) crash-sim escribia en path raiz pero el engine usa data_dir/vanta.wal; (3) fix previo (txn_end) era no-op. Fix: tracking per-id `open_txn` (Commit/Abort cierran solo la txn que matchea; un Begin nuevo marca el limite del batch incompleto; trailing incompleto se descarta fail-safe). Tests: 45/45 init+txn, chaos failpoints 1/1, durability/crash 8/8. Verify: `cargo nextest run -p vantadb -E ''test(init::) or test(txn)''` 45/45. Commit `db8b26b7`. Ceiling documentado: registro plano sin markers inmediatamente despues de txn parcial es indistinguible y se descarta fail-safe.

### FIND-28: 3 casts u8*->f32* sin chequeo de alineacion - migrados a as_f32_slice (align_to) (2026-08-24)
- **Fecha:** 2026-08-24
- **Plan:** `docs/dev/plans/2026-08-24-batch-review-mod-find.md` (Wave 1)
- **Resultado:** OK - 3 casts crudos `u8*->f32*` (from_raw_parts sin chequeo de alineacion = UB) en ivf.rs, distance/mapper.rs, serialize/bytes.rs reemplazados por el helper canonico `VectorRepresentations::as_f32_slice()` (align_to), eliminando 3 bloques unsafe y la const muerta MAX_VEC_F32_LEN. Review P2-01 vanta-audit APPROVE. Verify: check/clippy -D warnings/fmt, nextest 2055/2055. Commit `2d9fa75f`. FIND-29 derivado (search/layer.rs candidato, sound hoy).

### FIND-25: create_snapshot con quiesce (flush ERR-010) + recorrido recursivo de data_dir (2026-08-25)
- **Fecha:** 2026-08-25
- **Plan:** `docs/dev/plans/2026-08-25-batch-backup-restore-chain.md` (Task 1/3, cadena secuencial)
- **Resultado:** OK - (1) Quiesce: create_snapshot llama self.flush() antes del imageado en ambas variantes cfg (guard read_only; mismo patron que rebuild_vector_index/compact_layout_bfs: flush ANTES de tomar insert_lock, lock no reentrante) -> el conjunto imageado es mutuamente consistente en un punto unico en el tiempo. (2) Recorrido recursivo: helper mirror_data_dir() excluye el subdir snapshots/ (dentro de data_dir, evita snapshots/snapshots/) y mirror_file() unifica hard_link (unix) / copy (windows-wasm). (3) Layout real documentado: data_dir solo tiene archivos top-level (vstore_L0..L3.vanta, vector_index.bin, vanta.wal); el backend KV vive FUERA (escalado como FIND-33). Trade-off performance documentado en docstring (flush agrega costo, correctness>speed). Test de regresion test_snapshot_consistent_under_concurrent_writes (4 writers concurrentes + reopen + asserts anti-torn). Nota RED honesta: pre-fix pasaba en Windows via replay total del WAL (checkpoint_seq no se copia al snapshot) - el tear real era conjunto-inconsistente index/vstore en la variante copy. Verify: cargo nextest run -p vantadb snapshot --ignore-default-filter 6/6 PASS + 13 unit PASS + fmt PASS + clippy -D warnings PASS. Sin commit (lead verifica). Colateral: FIND-33 creado.

### FIND-37: Eliminar `query_sparse.unwrap()` panickable en dispatcher híbrido (2026-08-27)
- **Fecha:** 2026-08-27
- **Plan:** `docs/dev/plans/2026-08-27-backlog-pipeline.md` Task 1 (Wave 0) · `vanta-worker`
- **Resultado:** ✅ Dispatcher híbrido (`VantaEmbedded::search_impl`) asumía `Some` en 6 sitios vía `query_sparse.as_ref().unwrap()` (líneas 207,240,265,315,346,369) — request sin sparse dispara panic en prod (hot path). Fix: reemplazar `has_sparse: bool` + unwrap por `query_sparse: Option<&SparseVector>` filtrada `!is_empty` + match `Some(qs)` (pattern de `explain.rs` REVIEW-14 / `hybrid.rs:36`). También 3 sites en `debug_ops.rs` (288,335,374) mismo patrón. `hybrid_search` ahora recibe `Option` filtrado. Semántica preservada: sparse None/empty → fallback silencioso a text-only/vector-only (no ValidationError). Verify: `rg query_sparse.*unwrap src/sdk/search/mod.rs` → 0 ✅ · `rg src/sdk/search/` → 0 ✅ · `cargo check -p vantadb` ✅ · `cargo nextest -E 'test(search)'` 157 passed ✅ · `cargo fmt --check` ✅ · pre-commit hook clippy ✅. Commit `bd7c2691` `fix(search): FIND-37 eliminate query_sparse.unwrap panics`. Sin deuda neta (saldo negativo: 9 panics eliminados).
- **Archivos:** `src/sdk/search/mod.rs`, `src/sdk/search/debug_ops.rs`
- **Contrato:** `cargo nextest run -p vantadb --profile audit -E 'test(search)'` ✅ + `rg -n "query_sparse.*unwrap" src/sdk/search/mod.rs` → 0 + `cargo clippy -p vantadb --all-targets --all-features -- -D warnings` sin nuevos warnings

### FIND-39: ScalarIndex.remove sin test — cobertura engine-level (2026-08-27)
- **Fecha:** 2026-08-27
- **Plan:** `docs/dev/plans/2026-08-27-backlog-pipeline.md` Wave 1 #11 · `vanta-worker` — Origen codegraph-20260827-143245 Fase 10 (score 0.98, 0 hits en storage/engine/tests)
- **Resultado:** ✅ Añadido `test_scalar_remove` en `src/storage/engine/tests/scalar_index.rs` (93L, engine-level): direct `ScalarIndex` API (insert 2 reds → remove 1 → 1 left, no-op missing field/value/id, cross-field isolation, remove last) + wiring engine (insert alpha → overwrite beta → alpha cleared, delete → cleared). Registrado en `src/storage/engine/tests/mod.rs` (`mod scalar_index;`). Unit tests preexistentes en `src/scalar_index.rs` (test_scalar_index_remove etc.) ya cubrían happy path pero no el wiring engine; gap cerrado a nivel integración.
- **Archivos:** `src/storage/engine/tests/scalar_index.rs` (nuevo), `src/storage/engine/tests/mod.rs`
- **Contrato:** `cargo nextest run -p vantadb scalar_index --profile audit` → 15 passed (incl. test_scalar_remove) ✅ + `cargo nextest list --profile audit` 2069 → 2070 (+1) + `rg -n "pub fn remove" src/scalar_index.rs` → 2 ✅ + `cargo nextest run -p vantadb -E 'test(storage::engine)'` 306 passed ✅ + `cargo fmt --check` ✅ + `cargo clippy` ✅ + `validate-docs-coverage` 0 gaps ✅
- **Commit:** `59f5cdcb` `feat(test): FIND-39 add test_scalar_remove for ScalarIndex.remove`
- **Gates:** D: no-disparado (no feature-add, no symbols públicos nuevos) · V: no-disparado (verify pasó al 1er intento) · C: no-disparado (solo test, no API change)
- **Skills:** campaign-executor, progreso, ponytail, source-driven-development, test-driven-development, incremental-implementation, rust-write-tests, code-review-and-quality

### FIND-34: Ciclo WAL Writer — DAG documentado + edge tests recovery/quarantine (2026-08-27)
- **Fecha:** 2026-08-27
- **Plan:** `docs/dev/plans/2026-08-27-backlog-v2.md` Task 1 (Wave 0) · `vanta-worker` — Origen codegraph-20260827-143245 Fase 1 (4 nodos `open↔open_with_buffer↔recover↔quarantine` reportado como ciclo High)
- **Resultado:** ✅ DAG verificado (grep 0 back-edge: `recover`/`quarantine` leaves, solo callers en `open_with_buffer`), falso positivo Leiden co-localización documentado. Doc header `src/wal.rs:178-193` con diagrama DAG + ponytail ceiling. Tests: `test_recover_mid_file_corruption_scan_forward_recovers_tail` (16B garbage mid + raw framing valid tail → scan-forward 3/3) + `test_quarantine_rotates_when_corrupt_exists` (`.corrupt` → `.corrupt.1` rotation sin overwrite). Cobertura: `test_wal_auto_healing_and_recovery` + `test_corrupt_wal_tail_is_quarantined` preexistentes + 2 nuevos.
- **Archivos:** `src/wal.rs` (doc DAG 15L + 2 tests ~80L; `recover_valid_records` def 545, `quarantine_corrupt_tail` def 592, 1 def c/u)
- **Contrato:** `cargo nextest run -p vantadb --profile audit -E 'test(wal)'` → 62 passed (60→62, 2009 skipped, 6.08s) ✅ + `rg -n "quarantine_corrupt_tail|recover_valid_records" src/wal.rs` → 1 def c/u con cobertura ✅ + `cargo check -p vantadb` ✅ + `cargo clippy -p vantadb --all-targets --all-features -- -D warnings` 0 ✅ + `cargo fmt --check` ✅ + `validate-docs-coverage` 0 gaps ✅ + doc `src/wal.rs:178-193` justifica `codegraph_explore "wal cycle FIND-34"` como DAG (no SCC)
- **Commit:** `fix: FIND-34 — WAL writer DAG justification + recovery/quarantine edge tests` (pendiente)
- **Gates:** D: no-disparado (no symbols públicos nuevos, blast radius 2 archivos) · V: no-disparado (verify wal 1º intento 62/62) · C: no-disparado (doc + tests, no API change)
- **Skills:** campaign-executor, progreso, ponytail, source-driven-development, systematic-debugging, incremental-implementation, test-driven-development, documentation-and-adrs
- **Notes:** Ponytail rung 1 — doc antes que refactor `open_inner`. Skipped: ADR separado, helper extraction. Add when: SCC real con back-edge. Quarantine fails soft (warn); scan-forward byte-by-byte O(n) solo en recovery, no hot path.

### FIND-35: Ciclo StorageEngine get/prefetch — SCC intencional 2 nodos justificado + PrefetchGuard (2026-08-27)
- **Fecha:** 2026-08-27
- **Plan:** `docs/dev/plans/2026-08-27-backlog-v2.md` Task 2 (Wave 0) · `vanta-worker` — Origen codegraph-20260827-143245 Fase 1 (2 nodos `StorageEngine.get ↔ prefetch_related` reportado como ciclo)
- **Resultado:** ✅ SCC intencional verificado (OLD-20 co-access prefetch) + bounded single-level por `PrefetchGuard` (`thread_local! Cell<bool>` + RAII Drop unwind-safe, MCP-15). Doc header `src/storage/engine/get.rs:1-21` justifica SCC operacional DAG + ponytail ceiling + invariante sync-only. Guard previene SO en par mutuo uncached A↔B (get→prefetch→get→prefetch→…). Cover: `test_get_prefetch_does_not_recurse_forever` (cold-tier A↔B, 3 co-access) + 2 cache tests + 6 prefetch config/warmer.
- **Archivos:** `src/storage/engine/get.rs` (doc header `//!` 18L, `PrefetchGuard` 31-45, `get` 50-207, `prefetch_related` 228-257)
- **Contrato:** `cargo nextest run -p vantadb -E 'test(prefetch) or test(get_cache)'` → 8 passed (regex `/prefetch|get.*cache/` → 10 passed) ✅ + `rg -n "PrefetchGuard" src/storage/engine/get.rs` → 5 hits ✅ + `cargo check -p vantadb` ✅ + `cargo clippy -p vantadb --all-targets --all-features -- -D warnings` 0 ✅ + `cargo fmt --check` ✅ + doc `src/storage/engine/get.rs:1-21` justifica `codegraph_explore "StorageEngine get prefetch"` como SCC intencional single-level (no bug)
- **Commit:** `fix: FIND-35 — StorageEngine get/prefetch intentional SCC justification + PrefetchGuard doc` (este commit)
- **Gates:** D: no-disparado (no symbols públicos nuevos, blast radius 2 archivos + cache_warmer) · V: no-disparado (verify prefetch 1º intento 8/8) · C: no-disparado (doc only, no API change)
- **Skills:** campaign-executor, progreso, ponytail, source-driven-development, systematic-debugging, incremental-implementation, documentation-and-adrs
- **Notes:** Ponytail rung 1 — doc antes que aplanar prefetch. Skipped: extraer `fetch_without_prefetch` helper, ADR separado, AtomicBool global. Add when: prefetch se vuelve async/cross-task → migrar guard `thread_local` → `tokio::task_local!`. Invariante: get/prefetch síncronos same-thread; warm_hnsw_top_layer también bound por mismo guard. Verify thread: nextest 8/8 prefetch/get_cache 1.5s, check 3.49s, clippy 42s.

### FIND-44: ADRs iniciales — verificación idempotente (2026-08-28)
- **Fecha:** 2026-08-28
- **Objetivo:** Verificar que el proyecto tiene ADRs registrados (contrato: count >= 1 con headers Context/Decision/Consequences)
- **Resultado:** ✅ 39 ADRs encontrados en `docs/dev/architecture/adr/` (001..013, ADR-0001, ADR-0014..032, COMP-*, DRV-*). ADR-0001 (`ADR-0001-unified-config-readonly.md`) tiene Context/Decision/Consequences ✅. CodeGraph reporte Fase 12 ("Sin ADRs registrados") era stale — ADRs existen desde 2026-08-23.
- **Commit:** `docs: FIND-44 — verify ADRs exist, contract satisfied` (pendiente)

### CORE-01: Persistencia on-disk de vectores Binary/Turbo/SQ8 en vstore (2026-08-29)
- **Fecha:** 2026-08-29 (sincronización por vanta-worker; implementación commiteada 2026-08-28 por vanta-arch)
- **Plan:** `docs/dev/plans/2026-08-29-full-backlog-parallel.md` Task W4-SOLO (CORE-01, SOLO, 🟡 1-2d, max 2d)
- **Objetivo:** Cerrar el durability gap de `Binary`/`Turbo`/`SQ8` en `write_node_to_vstore` (que escribía `vector_len=0` y cero bytes de payload). Tras `flush + reopen` con HNSW rebuild, los vectores cuantizados se perdían.
- **Resultado:** ✅ Implementación commiteada (d3e7f9cf + 854d9145). ADR-0032 creado con tabla de 5 kinds (NONE/FULL/BINARY/TURBO/SQ8) + payload spec + reader dual legacy kind==0 + lazy migration; `NodeFlags::VECTOR_KIND_*` constants en bits 10-13 de `flags` (mask `0x3C00`, shift 10); `VFILE_VERSION` permanece en 2; tests pasan (`test_rebuild_binary_vector`, `test_persistence_binary_vector_roundtrip_vstore`); 93 tests binary|persistence|vstore|rebuild todos verdes.
- **Archivos:** `src/storage/ops.rs` (write_node_to_vstore reescrito, dispatch 6 variantes, SQ8 scale tail 4B), `src/node/flags.rs` (VECTOR_KIND_* constants), `src/storage/archive.rs` (compact_layout + rebuild_hnsw_from_vstore despachan por kind), `src/storage/engine/get.rs` + `engine/txn.rs` (decode kind + rescue HNSW legacy), `src/index/search/layer.rs` (vstore branch despacha rabitq/turbo/sq8 similarity), `src/node/disk.rs` (doc `vector_len` reinterpret), `docs/dev/architecture/adr/ADR-0032-binary-vector-persistence.md` (nuevo, status `accepted-pending-owner-review`).
- **Contrato:** ADR-0032 existe ✅ + `cargo nextest run -p vantadb --profile audit -E 'test(/binary|persistence|vstore|rebuild/)'` → 93/93 passed ✅ + `cargo fmt --check` 0 ✅ + `cargo clippy -p vantadb --all-targets -- -D warnings` 0 ✅ + `cargo test -p vantadb --lib storage::archive::tests::test_rebuild_binary_vector` 1/1 ok ✅ + `cargo test -p vantadb --lib --features fjall storage::engine::tests::init::test_persistence_binary_vector_roundtrip_vstore` 1/1 ok ✅.
- **Gates:** P: no-disparado (spec-first ADR-0032 antes de implementar) · D: no-disparado (no symbols públicos nuevos, blast radius 6 archivos core, sin API pública SDK nueva) · V: no-disparado (verify completo 1º intento) · C: no-disparado (lectura dual legacy mantiene compat, VFILE_VERSION sin bump).
- **Skills:** campaign-executor, progreso, ponytail, documentation-and-adrs, source-driven-development, api-and-interface-design, database-design, spec-driven-development, incremental-implementation, test-driven-development.
- **Notes:** Ponytail rung 1: ¿necesita existir nuevo campo header? No — reusar 4 bits libres en flags es 1 línea vs ampliar header 64→72. ADR-0032 escrito por IA con status `accepted`; tras Regla 5 (AGENTS.md) el owner humano debe articular el trade-off (bits vs header-bump vs migración one-shot) — marcada como `accepted-pending-owner-review` con nota crítica. Plan file `2026-08-29-full-backlog-parallel.md` tenía el estado `⬜ PENDING` con contrato PowerShell obsoleto (`Binary.*vector_len|DiskNodeHeader.*format_flag`) que no matchea el código real (usa constantes `NodeFlags::VECTOR_KIND_*`); sincronizado a `✅ COMPLETED` con contrato real validado. Riesgo: legacy Binary `kind==0, len=0` permanece irrecuperable por rebuild hasta reescritura lazy; documentado en ADR-0032 §5 y Risk Register.
- **Stop conditions (no se aplicaron):** >2d → docs-only ADR + encoding fix mínimo + tests reopen roundtrip. No fue necesario.

### FIND-36: Cross-crate NativeConnection ↔ RocksDbBackend — frontera documentada, falso positivo Leiden (2026-08-27)

- **Fecha:** 2026-08-27
- **Plan:** `docs/dev/plans/2026-08-27-backlog-v2.md` Task 3 (Wave 1) · `vanta-arch` — Origen codegraph-20260827-143245 Fase 1 (3 ciclos get/put/delete `desktop/src-tauri/src/connections/native.rs ↔ src/backends/rocksdb_backend.rs`)
- **Resultado:** ✅ Falso positivo verificado + frontera documentada. DISCOVERY `rg` 0 `use`-imports cruzados (RocksDbBackend no importa `desktop`/`tauri`/`NativeConnection`; NativeConnection no importa `RocksDbBackend`/`StorageBackend`), DAG `NativeConnection (desktop, isolated workspace [members=["."]] — `vantadb = {path="../.."}`) → `VantaEmbedded` → `StorageEngine` → `StorageBackend (pub(crate) Send+Sync, src/backend.rs)` → `RocksDbBackend (pub(crate), src/backends/rocksdb_backend.rs:22)` — 0 back-edge. Los 3 "ciclos" son colisión de nombres get/put/delete + clustering Leiden de verbos CRUD, no SCC CALLS. Workspaces aislados (`cargo check -p vantadb` invariant, `cargo tree` sin `vantadb-desktop`). Frontera ya correcta: `StorageBackend` pub(crate) + `BackendKind::RocksDb/Fjall/InMemory` + `VantaConfig.storage_path` (desktop nunca conoce `BackendPartition`/cf names). Doc headers: `src/backends/rocksdb_backend.rs:1-22` + `desktop/src-tauri/src/connections/native.rs:1-20` con diagrama DAG + ponytail ceiling + verificación `cargo check 0 cycles` + cross-ref mutua.
- **Archivos:** `src/backends/rocksdb_backend.rs` (header `//!` 15L, DAG + falso positivo + ponytail), `desktop/src-tauri/src/connections/native.rs` (header `//!` 12L, DAG + ponytail), `docs/dev/plans/2026-08-27-backlog-v2.md` Task 3 → ✅ COMPLETED + recitation FIND-36, `docs/dev/Backlog.md` FIND-36 eliminado
- **Contrato:** `cargo check -p vantadb --all-targets` 0.65s ✅ + `cargo check -p vantadb --all-targets --all-features` 37.55s ✅ + `cargo clippy -p vantadb --all-targets --all-features -- -D warnings` 0 ✅ + `cargo fmt --check` 0 ✅ + `rg` 0 use-imports cruzados ✅ + doc headers justifican `codegraph_explore "NativeConnection RocksDbBackend"` 0 ciclos o frontera documentada (rama ADR) — headers cumplen segunda rama sin ADR separado
- **Commit:** `fix: FIND-36 — Cross-crate NativeConnection↔RocksDbBackend frontier doc (false-positive Leiden, DAG justified)`
- **Gates:** D: no-disparado (no symbols públicos nuevos, blast radius 2 archivos + backend.rs existente, ≤10) · V: no-disparado (verify dual cargo check 1er intento) · C: no-disparado (doc only, no API change, `StorageBackend` intacto, workspaces aislados)
- **Skills:** campaign-executor, progreso, ponytail, source-driven-development, systematic-debugging, documentation-and-adrs, api-and-interface-design, code-review-and-quality
- **Notes:** Ponytail rung 1 — doc antes que `trait KvBackendPort` extracción o ADR. Skipped: ADR `docs/dev/architecture/adr/ADR-03X-crate-boundary.md`, `cargo modules dependencies --acyclic` (tool no baseline), trait extraction (frontera `StorageBackend` ya correcta). Add when: `RocksDbBackend` necesita llamar a desktop (callback Tauri/event) → invertir vía trait `DesktopEventSink` en `src/backend.rs` + channel o `tauri::Manager` emit. Invariante: `desktop → vantadb` one-way, `StorageBackend` pub(crate) nunca pub externo. Question Gate: pre-mortem 1 falsificó premisa (falso positivo) pero contrato exige doc frontera — COMPLETED con doc, no SKIP. CodeGraph 173 calls `src → skills` (FIND-42) tangencial, deferred.

### CORE-02: sync 2026-09-01 (drift backlog)
- **Fecha:** 2026-09-01
- **Objetivo:** CORE-02 completada previamente, removida del Backlog por drift (task file COMPLETED)
- **Resultado:** OK
- **Commit:** be57a94c
- **Dominio:** core-engine

### FIND-24: sync 2026-09-01 (drift backlog)
- **Fecha:** 2026-09-01
- **Objetivo:** FIND-24 completada previamente, removida del Backlog por drift (task file COMPLETED)
- **Resultado:** OK
- **Commit:** 0755c90e
- **Dominio:** core-engine

### FIND-38: sync 2026-09-01 (drift backlog)
- **Fecha:** 2026-09-01
- **Objetivo:** FIND-38 completada previamente, removida del Backlog por drift (task file COMPLETED)
- **Resultado:** OK
- **Commit:** bf59c2b1
- **Dominio:** core-engine

### FIND-40: sync 2026-09-01 (drift backlog)
- **Fecha:** 2026-09-01
- **Objetivo:** FIND-40 completada previamente, removida del Backlog por drift (task file COMPLETED)
- **Resultado:** OK
- **Commit:** 61a0bd42
- **Dominio:** core-engine

### FIND-43: sync 2026-09-01 (drift backlog)
- **Fecha:** 2026-09-01
- **Objetivo:** FIND-43 completada previamente, removida del Backlog por drift (task file COMPLETED)
- **Resultado:** OK
- **Commit:** f8720546
- **Dominio:** core-engine

### MOD-15: sync 2026-09-01 (drift backlog)
- **Fecha:** 2026-09-01
- **Objetivo:** MOD-15 completada previamente, removida del Backlog por drift (task file COMPLETED)
- **Resultado:** OK
- **Commit:** 3fd905bb
- **Dominio:** core-engine

### REVIEW-12: sync 2026-09-01 (drift backlog)
- **Fecha:** 2026-09-01
- **Objetivo:** REVIEW-12 completada previamente, removida del Backlog por drift (task file COMPLETED)
- **Resultado:** OK
- **Commit:** ac128bcb
- **Dominio:** core-engine

## Phase11 — Embeddings Local-First (2026-08-28) — 9/9 ✅

> Plan `docs/dev/plans/2026-08-28-embeddings-local.md` · commits `2c185021`→`d24eeb1c` · `embeddings/` + feature `embed-local` (Regla 5: embeddings BYO-vector → local-first). Tabla modelos: 3 EN / 3 ES / 3 Combined (8 ≤3GB + Qwen3 16GB excepción).

### EMB-01: Infra `embeddings/` + manifest + download/verify
- **Fecha:** 2026-08-28 — **Objetivo:** crear `embeddings/manifest.json` (9 rev pinned) + `download.py`/`verify.py` + `.gitignore` `/embeddings/models/` — **Resultado:** ✅ — **Commit:** `2c185021`

### EMB-02: Feature `embed-local` + `LocalOnnxProvider`
- **Fecha:** 2026-08-28 — **Objetivo:** `Cargo.toml:97` feature `embed-local` (ort+tokenizers) + `LocalOnnxProvider` `src/llm.rs:132` + `VANTA_LOCAL_MODEL` config — **Resultado:** ✅ — **Commit:** `9e06e79e`

### EMB-03: Descarga + verificación 9 modelos
- **Fecha:** 2026-08-28 — **Objetivo:** `download.py --check` + `verify.py --check` 9 modelos (verify.log 9 PASS check-only, smoke 691MB deferido rev 404) — **Resultado:** ✅ — **Commit:** `f5cb880c`

### EMB-04: Cablear `vanta-memory` L1
- **Fecha:** 2026-08-28 — **Objetivo:** fix punto 3: `vanta-memory` hook `L1DedupConfig::with_local_provider` + `local_embedding_hook` dummy — **Resultado:** ✅ — **Commit:** `3075df46`

### EMB-05: MCP tool `embed_texts`
- **Fecha:** 2026-08-28 — **Objetivo:** fix punto 3: `vantadb-mcp` tool `embed_texts` 1-128 texts, budgeting 25k, `embed_batch` — **Resultado:** ✅ — **Commit:** `241b0868`

### EMB-06: SQL vector auto-embed
- **Fecha:** 2026-08-28 — **Objetivo:** fix punto 3: `src/physical_plan.rs:51,129` `#[cfg(embed-local)]` branches `LocalOnnxProvider` — **Resultado:** ✅ — **Commit:** `bd9ab5ca`

### EMB-07: Bench comparativo 9 modelos
- **Fecha:** 2026-08-28 — **Objetivo:** `benchmarks/embed_bench.py` 533L + `docs/user/operations/BENCHMARKS.md` §8 report — **Resultado:** ✅ — **Commit:** `67fda296`

### EMB-08: Docs + Quickstart multi
- **Fecha:** 2026-08-28 — **Objetivo:** fix punto 1: `docs/user/QUICKSTART.md:182` + `05-embedding-integrations.md` + `docs/api/EMBEDDINGS.md` + README — **Resultado:** ✅ — **Commit:** `86cdfc57`

### EMB-09: Excepción Qwen3 >3GB
- **Fecha:** 2026-08-28 — **Objetivo:** `embeddings/README.md` sección + manifest `onnx:null` + bench `--include-exception` (16GB MTEB 75.1) — **Resultado:** ✅ — **Commit:** `d24eeb1c`

### AUD-043: Fix clippy `unused variable: ns` (`src/server/routing.rs:1166`)
- **Fecha:** 2026-08-31 — **Objetivo:** closure `options_for` `_ns: String` (tras REVIEW-10 `cf2ecc50` movido de `src/cli_server.rs:1302` a `src/server/routing.rs:1166`), lint cascade residual → FIND-035 — **Resultado:** ✅ arqueológico, 0 código — **Commit:** `223f675d` (fast-gate-residues Task 1)

### AUD-044: Shim `MmapMut` flush write-back
- **Fecha:** 2026-08-25 — **Objetivo:** `src/storage/vfile_mmap.rs:130-141` shim `flush()` no-op → write-back `seek+write_all`, evita pérdida `compact_layout` en builds `--no-default-features` — **Resultado:** ✅ +4 tests — **Commit:** `67f6af6d`

### ERR-CORE-01: `VantaError::code()` — 10 códigos canónicos `VANTADB_*` + tipado de overflow
- **Fecha:** 2026-09-02 · **Objetivo:** fuente canónica cross-binding del plan error-observability: `pub fn code(&self) -> &'static str` con match exhaustivo 32 variantes → 10 códigos de `docs/api/ERROR_HANDLING.md` §1.1 (split provisional resuelto: `IqlParseError`→VALIDATION, `IqlError`→INVALID_ARGUMENT); `VectorLenOverflow{id,len,limit}`/`EdgeCountOverflow{id,count,limit}` tipan 6 catch-alls `ResourceLimit(format!)` de `write_node_to_vstore` (ERR-029 preservado); `"code"` agregado a envelopes HTTP (`vanta_error_response`/`query_error_response`, aditivo); docs EMBEDDED_SDK/ERROR_HANDLING actualizadas · **Resultado:** ✅ contrato 8/8 — snapshot test `code_snapshot_all_variants` + prefijo + 9-códigos-emtidos + retriable-consistency, clippy workspace all-features 0, nextest audit -p vantadb 2140/2140 · **Commit:** `e1fe7ec2` · **Desbloquea:** ERR-PY-01, ERR-TS-01, ERR-MCP-01

### ERR-OBS-01: Captura y observabilidad — Backtrace + tracing estructurado + docs
- **Fecha:** 2026-09-02 · **Objetivo:** pilares de captura del plan error-observability (Wave 3): (1) `ChainedError` captura `std::backtrace::Backtrace` (stable 1.65 — premisa nightly del plan descartada con rustc 1.95.0) en ambos constructores, `Option` gated por `RUST_LIB_BACKTRACE`, expuesto en `Debug`+`backtrace_str()`, **nunca en Display** (contrato cross-language); (2) tracing estructurado `error.code`/`error.retriable`/`error.hint` en `vanta_error_response`/`query_error_response`, nivel 4xx=WARN/5xx=ERROR (helper puro `error_log_level` testeado); (3) métricas `vantadb_errors_total` NO cableadas — crate `metrics` no es dep raíz y está prohibido agregar deps sin justificar → TODO en doc + **FIND-53**; (4) verificación catch_unwind: PyO3 0.29 `trampoline.rs:301` cubre Python, `console_error_panic_hook`+trampa JS cubre WASM, tokio `JoinError` cubre server → documentado con evidencia, sin trabajo inventado; (5) docs `OBSERVABILITY.md` NUEVA + nota `CI_POLICY.md` + principio #6 en `ERROR_HANDLING.md` · **Resultado:** ✅ contrato 7/7 — `rg Backtrace error.rs`=11, error::tests 80/80 (ramales Some/None verificados con env), `rg error.code server/errors.rs`=3, check --all-targets 0, clippy workspace --all-features 0, fmt 0, suite lib 1983/1983 · **Colateral:** FIND-54 (test CORS pre-existente roto bajo `--features server`, no introducido aquí) · **Task file:** `.opencode/skills/campaign-executor/tasks/ERR-OBS-01.md`

### AUD-047: Deduplicación `metric_score` en `layer.rs`
- **Fecha:** 2026-08-25 — **Objetivo:** extraer closure `metric_score` compartido (Cosine/Euclidean/SparseDot) en `src/index/search/layer.rs`, −35 líneas, 2 call-sites — **Resultado:** ✅ — **Commit:** `bd8cc184`

### FIND-54: Fix flake `cors_layer_none_when_empty` bajo `--features server`
- **Fecha:** 2026-09-02 · **Objetivo:** `cors_layer` (`src/server/router.rs:86`) retornaba `Some` para `&[""]` porque `HeaderValue::from_str("")` es válido en http 1.x y el `filter_map` no descartaba origins vacíos → assert `.is_none()` del test pre-existente roto determinístico (nunca corrió en CI: perfil audit no habilita `server`). Fix: `.filter(|o| !o.is_empty())` antes del `filter_map` + test sibling `cors_layer_blank_origin_mixed_with_valid_keeps_layer` · **Resultado:** ✅ contrato 4/4 — test --exact pasa, suite lib `--features server` 2039/2039 (0 failed, 1 ignored), clippy `--all-targets --all-features -D warnings` 0, fmt 0 · **Commit:** `2030c743` · **Tarea:** colateral detectada en ERR-OBS-01 (2026-09-02), fix independiente · **Task file:** `.opencode/skills/campaign-executor/tasks/FIND-54.md`

### FIND-55: Sanitizar body 500 — chain solo a logs, code al cliente
- **Fecha:** 2026-09-02 · **Objetivo:** `query_error_response`/`vanta_error_response` filtraban el Display interno del motor (`"Execution Error: {e}"` / `e.to_string()`) en 5xx — io paths y storage detail llegaban al cliente; patrón que `panic_error_response` ya mitiga (AUDREP-32). Fix en `src/server/errors.rs`: rama 5xx (`status.is_server_error()`) responde `"internal error"` conservando el `code` VANTADB_* (ERR-CORE-01 — el cliente distingue por code, no por Display); 4xx mantienen message descriptivo (input del usuario). GAP verificado en Discovery: `log_vanta_error` (ERR-OBS-01) NO emitía el Display → añadido field `error.display = %e` en ambos brazos (field set idéntico) para que la chain completa quede server-side. Consumidores verificados: `classifyWasmError` (vantadb-ts) regex-mirrora Display del CORE en la frontera wasm, no el envelope HTTP → inafectado; mocks desktop y ejemplos de docs usan shape 4xx → siguen fieles. · **Resultado:** ✅ contrato — tests nuevos `five_xx_bodies_are_sanitized_to_generic_message` (RED→GREEN) + `four_xx_bodies_keep_descriptive_message` green, suite lib `--features server` 2041/2041 (0 failed, 1 ignored), clippy `--workspace --all-targets --all-features -D warnings` 0, fmt 0, grep `"internal error"` visible en errors.rs · **Commit:** `fefdbc93` · **Tarea:** NOTICIED de ERR-OBS-01 (2026-09-02) · **Task file:** `.opencode/skills/campaign-executor/tasks/FIND-55.md`

### FIND-53: `vantadb_errors_total{code}` — registry in-tree
- **Fecha:** 2026-09-02 · **Objetivo:** cablear el counter de error del contrato de observabilidad sin agregar el crate externo `metrics` (decisión del orquestador; el registry in-tree `src/metrics/` ya soporta labels — precedente `GRAPH_OPS_TOTAL`/`HTTP_REQUESTS_TOTAL` con `IntCounterVec`). Implementación: `ERRORS_TOTAL` (`vantadb_errors_total`, label único `code`, cardinalidad ≤10 derivada de `VantaError::code()` — nunca strings libres) en `src/metrics/core/registry.rs`; helper dual-cfg `record_vanta_error(code)` en `core/mod.rs` (patrón `record_graph_op`: no-op sin feature `prometheus`); incremento en el choke point único `log_vanta_error` (`src/server/errors.rs`) — ambos envelopes lo cruzan, cada error servido cuenta exactamente una vez. Límite documentado (no escalado): feature `server` no habilita `prometheus` en Cargo.toml — scrape real requiere `--features server,prometheus`; interim sin feature siguen los logs §3. Docs: `OBSERVABILITY.md` §4 TODO→wiring real, §5 alerta `rate(5xx)>2×baseline` ahora implementable sobre la serie. · **Resultado:** ✅ contrato — `rg vantadb_errors_total src/`=13≥2, tests nuevos `error_envelopes_increment_vantadb_errors_total_by_code` (delta exacto +2 por label TIMEOUT, no tocado por otros tests) y `test_vantadb_errors_total_counter_init` (scrape contiene la serie) verdes con `--features server,prometheus`, suite lib `--features server` 2041/2041 (0 failed, 1 ignored), clippy `--workspace --all-targets --all-features -D warnings` 0, fmt 0, OBSERVABILITY.md sin TODO de métricas · **Commit:** `e73046ea` · **Tarea:** ola 4, origen FIND-53 del plan error-observability · **Task file:** `.opencode/skills/campaign-executor/tasks/FIND-53.md`

### FIND-52 (core): migración web_time + helper `acquire_insert_lock` — paths nativos sin pánicos en wasm32
- **Fecha:** 2026-09-02 · **Objetivo:** eliminar los pánicos de runtime en rutas compiladas bajo wasm32-unknown-unknown (`sys::time::unsupported` por `std::time::{Instant,SystemTime}::now()` y `condvar::no_threads` por `std::thread::sleep`) sin alterar semántica nativa: migración a `web_time` en `src/storage/engine/init.rs` (Instant del retry-loop; `thread::sleep` cfg-out `#[cfg(not(target_arch = "wasm32"))]` con comentario — en wasm una sola pasada basta: single-thread, nadie libera el lock entre reintentos, sin hot-spin y fallo como `DatabaseBusy` al vencer el timeout), `src/storage/engine/mod.rs` (`FsSnapshot::created_at` + 2 constructores + `SystemTime` de restore → `web_time::{Instant,SystemTime,UNIX_EPOCH}`), `src/index/search/profile.rs` (SearchProfile `debug_assertions` — corre en todo search del pkg `--dev`) y `src/index/graph.rs` (`repair_orphan_links`). **2º contribuyente revelado por backtraces en RED:** `parking_lot::try_lock_for` → `util::to_deadline` → `std::time::Instant::now()` (pánico en TODO put/delete/flush bajo wasm) — fix en la raíz con helper `StorageEngine::acquire_insert_lock(op)` (nativo: `try_lock_for(timeout)` idéntico; wasm: `try_lock` no-bloqueante → mismo `VantaError::Timeout`; lock tomado en single-thread implica re-entrancia, esperar no puede progresar); 12 call sites de insert/delete/maintenance/ops routed por el helper (net −27L). Verificados NO-sites: `lsm.rs`/`ingestion.rs` (sin `now()` alcanzable en build wasm), sleeps de `api.rs`/`index/core.rs` (cfg(test)), `server/`+`tui`+`cli_handlers` (feature-gated fuera). · **Resultado:** ✅ contrato — `cargo test -p vantadb --lib` 1983/1983 (semántica Timeout nativa intacta), clippy `--workspace --all-targets --all-features -D warnings` 0, fmt 0, rg confirmatorio: producción 0 hits (quedan solo cfg(test) y el cfg-not-wasm comentado), cargo check wasm32 0 · **Commit:** `a137bdc7` · **Tarea:** FIND-52 (origen verificación ERR-TS-01) · **Task file:** `.opencode/skills/campaign-executor/tasks/FIND-52.md`


### RES-03 (core): canal multi-consumidor de ingesta — MEDIDO-NO-APLICA (bench nuevo + caracterización)
- **Fecha:** 2026-09-03 · **Objetivo:** resolver con evidencia (Regla 9) la sospecha FND-19 de que `Arc<Mutex<mpsc::Receiver>>` (`src/ingestion.rs:72`) serializa la ingesta y justifica un canal multi-consumidor nativo (crossbeam/flume). Discovery: `tokio::sync::mpsc` es multi-producer **single-consumer** por diseño (docs.rs/tokio/latest/tokio/sync/mpsc) — la "opción ponytail tokio nativo multi-consumidor" del brief es premisa muerta; el patrón actual con consumers async + `spawn_blocking` del insert es el recomendado por Tokio y cumple `.opencode/rules/concurrency-async.md` R1/R2; ingestion NO entra al build wasm (`vantadb-wasm` = `default-features=false, features=["wasm"]`, sin `async-ingestion`). Medición: bench nuevo `benches/ingestion_concurrent.rs` (criterion, `required-features=["async-ingestion"]`, BATCH=400 DIM=16, matriz 2 producers × {1,2,4} consumers, ×2 corridas, varianza <3%): 113.5 → 78.5/81.0 → 65.0 ops/s — el throughput **degrada monótonamente al añadir consumers** (−31% w2, −43% w4); producers no importan (backpressure/capacity 1024 intacta): el techo es la ruta serial del motor (`insert_lock` global + WAL fsync por escritura), no el canal. Decisión: **MEDIDO-NO-APLICA** — `src/ingestion.rs` sin refactor (fila eliminada del Backlog con números); crossbeam/flume rechazados (resuelven un cuello inexistente + dep/feature-gate); bench queda como infra A/B para FUT-12 (fsync batching). Test de caracterización añadido: `ingestion::tests::pipeline_delivers_every_submitted_task` (primer test del módulo; sin asunción de orden — pre-mortem 1/2 documentados en §13). Colaterales: fix 1 línea clippy `tui/monitor.rs` (clone_on_copy, bloqueaba el gate `--all-features` del contrato, pre-existente, comportamiento idéntico) + FIND-57 (default `worker_count=4` activo-perjudicial → Gate D) + FIND-58 (raw wasm32 check falla en `getrandom@0.3.4` por feature-resolution, pre-existente; y unused imports release-only en `debug_ops.rs`). · **Resultado:** ✅ contrato — `rg "multi-consumidor|multi-consumer|RES-03" docs/user/operations/BENCHMARKS.md`=6≥1 (§13 con tabla antes/después y decisión); `cargo test -p vantadb --lib ingestion --features async-ingestion` 1/0; suite `--lib` 1983 passed/0 failed (sin regresión); `cargo check --workspace --all-targets` 0; `cargo clippy --workspace --all-targets --all-features -- -D warnings` 0; `cargo fmt --all -- --check` 0; wasm: condicional N/A (ingestion no entra) + fallido crudo declarado FIND-58 · **Commit:** RES-03 (esta ola) · **Tarea:** RES-03 plan `2026-09-03-quality-gtm-wave` Wave3 · **Task file:** N/A (resumen en plan; `tasks/RES-03.md` pertenece a campaña 2026-09-02 — colisión ID, no sobreescribir)

### AUD-045 (core): IVF hot-path clones — PREMISA-MUERTA, medido y cerrado sin cambio de código
- **Fecha:** 2026-09-03 · **Ruta:** vanta-tuner · **Objetivo:** resolver con evidencia (Regla 9) la fila de `audit-full-20260825-031011` que citaba `centroid.clone()`/`entry.vector.clone()` en el search loop de `src/index/ivf.rs:250,275`. **Verificación pre-mortem-1 antes de optimizar:** `rg "centroid.clone\(\)|entry.vector.clone\(\)" src/index/` = **0 hits**; el scan por candidato ya borrowea (`ivf.rs:255` `f32_slice_similarity(query, None, &entry.vector, metric)`) porque el commit **`b4ff157d` (2026-08-25)** — mismo día que el audit abrió la fila — ya hizo la conversión clone→borrow con −59% declarado en mensaje. Fila "Pendiente" ≠ hallazgo pendiente: **premuerta, rama válida de cierre sin trabajo inventado**. Clones restantes L82/L91/L108/L154/L209 = build/training k-means (cold path) → no tocar, documentado. Sin optimización nueva no hay A/B ni umbral >2%; la wave aporta el **primer baseline IVF medido en repo** + evidencia de cierre: tabla nlist×nprobe (N=10k D=128 k=10 seed 42, ×2 corridas, mediana) en `docs/user/operations/BENCHMARKS.md` §14 — p50 estable <7% entre corridas, p99 ruidoso a n=200 (reportado como rango, no fabricado; **p95 not measured**: el bench no lo computa), QPS hasta 16955 (100,1), ~0.3 µs/candidato coherente en todas las celdas. **Hallazgo colateral (documentado, no ejecutado):** el build IVF NO es determinista a pesar de seed 42 — iteración de DashMap sin orden → Forgy elige centroides distintos por proceso (recall Δ hasta ±6% entre corridas); fix trivial (sort por id antes del k-means) fuera de scope de clones, queda para fila futura si se quiere gatear recall en CI. **Incidente entorno (Regla 11, en §14):** release build crasheaba `STATUS_STACK_BUFFER_OVERRUN` en rustc sin `RUST_MIN_STACK`; workaround `RUST_MIN_STACK=33554432`. · **Resultado:** ✅ contrato (todo caso) — `rg -n "AUD-045|IVF" docs/user/operations/BENCHMARKS.md` ≥1 (§14 nueva), `rg -c "centroid.clone\(\)|entry.vector.clone\(\)" src/index/ivf.rs` = 0 (ya lo estaba), `cargo check --workspace --all-targets` 0, `cargo clippy --workspace --all-targets --all-features -- -D warnings` 0, `cargo fmt --all -- --check` 0, `cargo nextest run -p vantadb ivf` 21/21 · **Cierre:** fila Backlog AUD-045 eliminada · **Commit:** docs(bench) AUD-045 (esta ola) · **Tarea:** AUD-045 plan `2026-09-03-quality-gtm-wave` Wave4/5 (última)

### FIND-57 (core): default `worker_count` 4→1 en `AsyncIngestionPipeline` — Gate D ejecutado
- **Fecha:** 2026-09-03 · **Ruta:** vanta-worker · **Objetivo:** retirar el default activo-perjudicial `worker_count=4` (corolario RES-03/BENCHMARKS §13: −31% w=2 / −43% w=4 vs w=1 sobre la ruta serial `insert_lock` + WAL fsync). Gate D aprobado por el usuario en el dispatch. Cambio: `src/ingestion.rs:45` `unwrap_or(4)` → `unwrap_or(1)` + doc-comment a "defaults to 1" con degradación medida + comentario `ponytail:` (single worker; escalar cuando FIND-59/FUT-12 levanten el techo serial). Sin callers `None` (únicos callers: bench con `Some(workers)` explícito y test con `Some(2)`) → sin tests que actualizar; TDD N/A con rationale (getter de worker_count = scope creep). Re-medición post-cambio `benches/ingestion_concurrent.rs` ×2 corridas (celdas p{1,4}×w{1,4}, criterion 1 filtro por invocación, mediana de 11 samples por corrida): p1/1 **111.5** (114/109), p1/4 63.0 (−43%), p4/1 106.5, p4/4 62.0 (−44%) — misma forma que la matriz pre-cambio; subsección post-FIND-57 en BENCHMARKS §13. · **Resultado:** ✅ contrato de la fila — default 1 por rg (`unwrap_or(1)` + 0 restos `= 4`), bench post-cambio 111.5 ≥110 ops/s, `cargo test -p vantadb --lib ingestion --features async-ingestion` 1/0, `cargo clippy -p vantadb --lib --all-features -D warnings` 0, `cargo fmt --check` 0, fila Backlog eliminada. **Desviación declarada (pre-existente, fuera de scope, no tocada):** `cargo clippy --workspace --all-targets --all-features -D warnings` rojo en HEAD — crate `vanta-memory` (2696 errores E0405/E0425/..., sin modificar en worktree) + targets stale de `vantadb` (`tests/integration.rs` E0422/E0425 `SearchRequest`/`search_handler` que viven en `src/integrations.rs` feature-gated; `tests/sparse_vectors.rs` + `benches/hybrid_queries.rs` E0463) que ni referencian `ingestion` (rg=0) → queda para el lead (posible fila FIND nueva). Colateral de entorno: criterion solo acepta UN [FILTER] posicional (el task file asumía múltiples) + `Out-File` con `/` en el nombre pierde el log (1 tanda re-corrida). · **Commit:** `fix(ingestion): default worker_count 1, w=4 medido -43% (FIND-57)` (solo `src/ingestion.rs` + `docs/user/operations/BENCHMARKS.md`; completions/Cargo.lock/.opencode NO stageados) · **Task file:** `.opencode/skills/campaign-executor/tasks/FIND-57.md`

### GOV-TK1-restore (core): restore --dry-run validacion sin mutar - segunda mitad
- **Fecha:** 2026-09-03 - **Ruta:** vanta-worker - **Objetivo:** cerrar la mitad pendiente de GOV-TK1 (fila re-escalada: quedaba el verificador de restore; mitad doctor --fix landed adf03752). Diseno patron doctor ponytail: flag --dry-run en Restore que valida input (existe, es dir, no vacio, MANIFEST.json parsea si presente) + tamano + conflictos target y LISTA que restauraria sin tocar target; sin flag = comportamiento actual intacto. LEIDO cmd_restore primero para ver como copia hoy.
- **Resultado:** OK contrato - rg dry_run en struct Restore >=1 (cli.rs Restore dry_run) + cmd_restore maneja dry_run (early-return read-only) + tests nuevos restore_dry_run_missing_backup_errors y restore_dry_run_lists_without_mutating (inexistente error claro; valido + target existente dry-run Ok y UNTOUCHED antes/despues identico + contenido intacto) + binario vanta-cli restore --dry-run exit 0 sin mutar (quiesced IDENTICAL_OK) + runbook honesto (restore --dry-run VERDAD con limites) + clippy -p vantadb --all-targets -D warnings 0 + fmt 0 + cargo test -p vantadb --lib cli 44/0. Fila GOV-TK1 ELIMINADA (ambas mitades completas). NO toca src/ingestion.rs ni vantadb-server. NO stagea completions/Cargo.lock/.opencode; NO toca stash@{0}.
- **Archivos:** src/cli.rs (Restore dry_run), src/cli_handlers/backup.rs (cmd_restore_dry_run + firma 6 args), src/bin/vanta-cli.rs (dispatch), tests/cli_tests.rs (2 nuevos + 3 callers a false), docs/user/operations/DISASTER_RECOVERY_RUNBOOK.md (2.6 + 3 PENDING->AVAILABLE), docs/dev/Backlog.md (fila eliminada 107->106), docs/dev/avance/activo/core-engine.md (esta entrada).

### FIND-59 (core-arch): granularidad `insert_lock` — DISCOVERY con decisión (d), 0 código
- **Fecha:** 2026-09-04 · **Ruta:** vanta-arch · **Objetivo:** análisis/DISCOVERY de la mitad insert_lock del techo de ingesta (§13: 111.5 ops/s, −43% con w=4): localizar locks globales, listar invariantes, evaluar (a) sharding por namespace/región, (b) RCU write-path, (c) batching bajo un lock, (d) no hacer nada — con riesgos R1/R2 + Regla 8 + durabilidad + WASM. Discovery: holders = insert/batch_insert/delete/delete_batch/flush/flush_pending_hnsw/refresh/consolidate/rebuild/compact/quantization (`mod.rs:318`, `acquire_insert_lock` L581-594 con degradación wasm); lectores (`get`/search) y recovery NUNCA lo toman (RCU `ArcSwap` + `DashMap`); invariantes = ERR-010 WAL-vs-index ordering, atomicidad topología HNSW (`CPIndex::add` multi-paso sobre `entry_point: AtomicU128` + aristas bidireccionales, `graph.rs:595-808`), drain atómico del batch, protocolo delete↔consolidate FND-02-M3, escritura `checkpoint_seq`; el core NO tiene `namespace` (vive en SDK/memory layer) → (a) exigiría clave id-hash sin boundary natural y mantiene serial la sección dominante (HNSW add + fsync al mismo disco); `commit_transaction` (`txn.rs:119-213`) corre WAL batch + apply SIN el lock — hueco ERR-010 colateral no verificado, plegado al spike. Matriz: (a)/(b) ~0 ganancia con riesgo alto/muy alto → rechazadas; (c) ya existe en engine (`batch_insert_with_opts` bajo UN guard) y su ganancia real es amortizar fsyncs = mitad FUT-12 (ensancha ventana de pérdida → requiere su decisión de durabilidad) → diferida; **decisión (d)** — el lock es load-bearing, ningún cambio de granularidad supera el gate >2× sin tocar política de durabilidad. Desglose fsync-vs-lock NO cuantificable sin medir (declarado) → spike follow-up FIND-61 (≤1d, bench/config, 0 prod: Always vs Never vs micro-batch N={8,16,32} + verificación ERR-010 de commit path; gate ≥2× + FUT-12 para abrir slice). Sin empate (no hay silencio: (c) es follow-up condicionado, no alternativa empatada).
- **Resultado:** ✅ contrato — ADR `docs/dev/architecture/adr/ADR-0037-insert-lock-granularity.md` (invariantes + matriz + recomendación (d) + contract assessment) + fila FIND-59 ELIMINADA de Backlog + fila FIND-61 creada + nota §13 + `cargo fmt` N/A (0 código) + markdownlint 0 en docs tocados + `git status` sin `src/`. NO toca stash@{0}; NO stagea completions/Cargo.lock/.opencode.
- **Archivos:** docs/dev/architecture/adr/ADR-0037-insert-lock-granularity.md (nuevo), docs/dev/Backlog.md (FIND-59 eliminada + FIND-61), docs/user/operations/BENCHMARKS.md (§13 nota), docs/dev/avance/activo/core-engine.md (esta entrada). Task file: `.opencode/skills/campaign-executor/tasks/FIND-59.md` (DISCOVERY, sin stagear).

### FIND-61 (tuner): spike medición insert_lock vs fsync — CERRAR con números, 0 código prod
- **Fecha:** 2026-09-04 · **Ruta:** vanta-tuner · **Objetivo:** producir el desglose lock-vs-fsync por-op que FIND-59/ADR-0037 declaró no-cuantificable sin medir: Always (revalidar §13) vs Never* (solo lock+HNSW) + prototype micro-batching bench-only N={8,16,32} (ops/s + p50-ack + ventana de pérdida) + verificación ERR-010 de `commit_transaction` (`txn.rs:119-213`). Gate: abrir slice SOLO si batch ≥2× Y FUT-12 decidió ventana de pérdida.
- **Discovery:** harness `benches/ingestion_concurrent.rs` + §13 + ADR-0037 leídos primero; hallazgo crítico pre-bench: `SyncMode::Never` NO tiene rama en `WalWriter::maybe_sync` (`src/wal.rs:376-389`, `rg Never src/` = definición+parsing, 0 branches) → con threshold None fsyncea igual que Periodic; el A/B solo aísla con `Never + flush_threshold=Some(1M)` bench-only (`Never*`, documentado en §13.1, no fix por contrato PROHIBIDO `src/`). `batch_insert_with_opts` bajo UN guard ERR-010 (`insert.rs:750`) vs `commit_transaction` SIN lock (solo `try_push` oportunista). Fjall per-op sin `backend.flush` → el A/B aísla WAL-fsync limpio.
- **Medición** (`benches/ingestion_concurrent.rs` grupos bench-only `find61_sync`/`find61_batch`, BATCH=400 DIM=16 fjall tempdir Win11 i5-1235U rustc 1.95.0 perfil bench, criterion sample 10, ×2 corridas + mediana, HEAD `7a0811cb`): Always p1/w1 **96.5** (97/96) vs Never* p1/w1 **98.0** (103/93) → **`t_fsync ≈ 0.16 ms (~1.5% de ~10.3 ms/op)`** — fsync medible pero NO dominante (revisa el "fsync-dominado" acotado de ADR-0037 con evidencia; decisión (d) intacta). Never* p1/w4 **61.5** (−37% vs Never* p1/w1): el convoy persiste SIN fsync → el término dominante es lock+HNSW. Batch (Periodic-1, `skip_wal=false`): N=8 **433 ops/s / 18.6 ms / 8 writes (3.9×)**, N=16 **713 / 22.2 ms / 16 (6.4×)**, N=32 **1016 / 31.6 ms / 32 (9.1×)** — amortiza lock-takes + batch_append + HNSW bulk a la vez.
- **ERR-010:** `commit_transaction` **VIOLA** (WAL durable contado por `flush()` antes de que su mutación HNSW drene → invisible record posible con `flush()` concurrente; escenario en §13.1); el prototype **RESPETA** (usa `batch_insert` con lock). Fix = `src/` prod con tradeoff → fuera del spike, colateral para el lead (candidato a FIND nueva, no creada por contrato).
- **Resultado:** ✅ contrato — gate ① CUMPLE (hasta 9.1×) pero gate ② NO (FUT-12 P24 ❌ Sin implementar) → **decisión CERRAR con números, NO abrir slice**; bench queda como infra A/B para FUT-12. BENCHMARKS §13.1 (scorecard medido + Tablas 1-2 + desglose + gate explícito) + fila FIND-61 ELIMINADA del Backlog con motivo + `cargo test -p vantadb --bench ingestion_concurrent --features async-ingestion` 0 failed (harness compila y corre; test lib `ingestion` 1/1) + clippy/fmt bench-scope 0 (0 `src/`, N/A con rationale) + markdownlint 0 en docs. NO toca stash@{0}; NO stagea completions/Cargo.lock/.opencode.
- **Archivos:** benches/ingestion_concurrent.rs (grupos bench-only + imports, ~150L añadidas), docs/user/operations/BENCHMARKS.md (§13.1 spike), docs/dev/Backlog.md (FIND-61 eliminada), docs/dev/avance/activo/core-engine.md (esta entrada). Task file: `.opencode/skills/campaign-executor/tasks/FIND-61.md`.

- FIND-63 (worker): rama explicita SyncMode::Never - Resultado: match exhaustivo Always|Never|Periodic + test RED->GREEN; suite wal 63/63; fmt/clippy limpios. Commit a7285969 (2026-09-05).

- FIND-62 (worker): commit_transaction bajo insert_lock - Resultado: guard en [WAL batch -> apply -> drain -> Commit] + test commit_flush_interleaving verde; suite storage 380/380, lib 1985/1985; sin deadlock (pre-mortem: unico caller productivo sin guard). Commit 19a9651c (2026-09-05).

- MCP-34b (worker): tool snapshot_restore verificado sin codigo - Resultado: stop-condition S1+S2-S4 ya en HEAD via 4d964ac3 (FIND-25: flush en create_snapshot) + 29d21cba (snapshot_restore + validate + failpoint + dispatch MCP con confirm:true); tests E2E verdes: snapshot_certification 21/21 + failpoint 1/1 + mcp_tests snapshot_restore/tools-list/create 3/3; fmt/clippy limpios. Sin commit nuevo (2026-09-06).

### FIND-60: rustdoc 47→0 warnings (plan 2026-09-07-backlog-triage Wave0)
- **Fecha:** 2026-09-07
- **Objetivo:** links intra-doc irresueltos + items privados en 23 archivos `src/` + `vantadb-python/src/lib.rs`; cero `pub` nuevos (paths `crate::` o ticks).
- **Resultado:** ✅ `cargo doc --no-deps -p vantadb -p vantadb_py | grep -c warning` → 0; `-D warnings` exit 0; fmt+clippy hook verde. Colateral incluido: `completions/*` regenerados por cambio doc en `src/cli.rs`.
- **Commit:** 0f1f5e37

### FIND-48: split src/index/graph.rs 2031L por concern (plan 2026-09-08-backlog Wave0)
- **Fecha:** 2026-09-08
- **Objetivo:** god-file HNSW → `graph/{mod,types,prefetch,core,tests}` sin cambio semántico (pure move).
- **Resultado:** ✅ cargo check 0 + clippy 0 + nextest audit 2145 passed/0 failed + equiv 2031/2031 VERBATIM OK.
- **Commit:** fcd339b7

### FIND-49: split src/sdk/types.rs por dominio record/search/graph (plan 2026-09-08-backlog Wave0)
- **Fecha:** 2026-09-08
- **Objetivo:** tipos SDK monolíticos → `types/{record,search,graph}` + re-exports intactos.
- **Resultado:** ✅ check 0 + sdk tests 412/0 + API pública 42/42 idéntica + fmt 0.
- **Commit:** e3711dea + 72eb4c0a (sync)

### FIND-50: split src/parser/mod.rs en grammar.rs + lexer.rs (plan 2026-09-08-backlog Wave0)
- **Fecha:** 2026-09-08
- **Objetivo:** parser IQL monolítico → `lexer.rs`/`grammar.rs` + `mod.rs` thin.
- **Resultado:** ✅ check 0 + parser 117/117 + insta 0 snap.new + clippy 0 + fmt 0.
- **Commit:** dd2cb943

### AST-002: Rust tipos sin stutter + aliases deprecated (plan 2026-09-10-anti-stutter)
- **Fecha:** 2026-09-11
- **Objetivo:** 38 pub Vanta* -> nombres limpios (mapa) + pub type Viejo=Nuevo deprecated en 5 niveles; 0 cambios serde.
- **Resultado:** OK check 0 + clippy-all-targets-all-features 0 + fmt 0 + nextest 2145/1 + mcp-alias-E2E 0 errores.
- **Commit:** 97e29cac

### C2C1: limpieza comentarios redundantes guia 2.4 (Fase 2 Wave 0)
- **Fecha:** 2026-09-13
- **Objetivo:** solo-borrado en 5 archivos (in_memory/accumulator/flat/crud/tests mojibake); excepciones valiosas intactas (AUDREP-55, MEM-01).
- **Resultado:** ✅ fmt + nextest 211/211.
- **Commit:** a5d3d40a
### C2A1: boundaries + doctrina hibrida + regla anti-ciclo (Fase 2 Wave 0)
- **Fecha:** 2026-09-13
- **Objetivo:** `docs/dev/architecture/BOUNDARIES.md` (owners + BND-01…08 + ciclo storage-index como deuda + gate Fase 3) + enlace ARCHITECTURE.md.
- **Resultado:** ✅ docs-only, cero `src/`.
- **Commit:** e12a3871
### C2S1: segregar StorageBackend en roles (Fase 2 Wave 1)
- **Fecha:** 2026-09-13
- **Objetivo:** 12 metodos → `Scannable`/`Snapshotable`/`Compactable` `pub(crate)` + checkpoint/compact honestos; 4 backends.
- **Resultado:** ✅ backends 77/77 + maintenance/stats 131/131 + clippy/fmt 0.
- **Commit:** eaa166d9
### C2S7: partir AccessTracker en AccessStats+Pinnable (Fase 2 Wave 1)
- **Fecha:** 2026-09-13
- **Objetivo:** ISP: lectores de eviccion solo contra `AccessStats`; blanket-impl compat.
- **Resultado:** ✅ node 209 + eviction 24 + fmt.
- **Commit:** 96ebd9ec
### C2M2: puerto EntityRepository en entity/ (Fase 2 Wave 1)
- **Fecha:** 2026-09-13
- **Objetivo:** trait DDD + `EntityStore` adaptador + `checker.rs`/`state.rs` a `&dyn`; 0 tests tocados.
- **Resultado:** ✅ entity 46/46 + fmt.
- **Commit:** 3c5beb72
### C2S2: registry/factory de backends OCP (Fase 2 Wave 2)
- **Fecha:** 2026-09-13
- **Objetivo:** `BackendRegistry` + init/config consumen + backend ficticio sin tocar init; `handlers.rs` fuera.
- **Resultado:** ✅ backend 80/80 + registry/kind 10/10 + config 54/54 + storage 445/445.
- **Commit:** e5dc9022
### C2S5: partir Executor por variante de Statement (Fase 2 Wave 2)
- **Fecha:** 2026-09-13
- **Objetivo:** 7 ramas → helpers por variante + embedding; correccion: split por variante (parse/auth viven fuera).
- **Resultado:** ✅ executor 24/24 + integracion 27/27 + clippy/fmt.
- **Commit:** a7df92bc
### C2S3: extraer TxnManager de StorageEngine slice-txn (Fase 2 Wave 2)
- **Fecha:** 2026-09-13
- **Objetivo:** `TxnManager` `pub(crate)` + delegacion 1:1 + 7 tests; 26→24 campos; bench en CI al mergear.
- **Resultado:** ✅ storage 452 + backend 80 + clippy/fmt; acyclic sin ciclos nuevos.
- **Commit:** cdb15b3e
### C2T1: aislar tests lentos intra-binario (Fase 2 Wave 3)
- **Fecha:** 2026-09-13
- **Objetivo:** `#[ignore]` core.rs + vitest env-gate `VITEST_FULL` + doc suite completa (Step 1 superado por C2T2).
- **Resultado:** ✅ default 0 run + `--run-ignored all` 1 passed + vitest FULL 1 passed.
- **Commit:** ed36bcbd
### C2T2: determinismo seeds + clock inyectable (Fase 2 Wave 3)
- **Fecha:** 2026-09-13
- **Objetivo:** `StdRng::seed_from_u64` + `Clock`/`ManualClock`/`with_clock` + TTL sin sleep; 3 corridas.
- **Resultado:** ✅ message_thread 6/6 x3 + index::core 13+1 x3 + clippy/fmt.
- **Commit:** 930fc828
### C2T3: AAA Act bulk en load/certification (Fase 2 Wave 3)
- **Fecha:** 2026-09-13
- **Objetivo:** un Act bulk por test + banners ordenados; loops verify solo-lectura conservados.
- **Resultado:** ✅ vitest load 6/6 + cert 36/36 + fmt/clippy/tsc/eslint.
- **Commit:** a075fe36
### C2S3b: extraer CacheLayer de StorageEngine slice-2 (Fase 2 Wave 3.5)
- **Fecha:** 2026-09-13
- **Objetivo:** `CacheLayer` `pub(crate)` 5 campos + delegacion 1-token + tests; 24→20 campos; ERR-037 intacto.
- **Resultado:** ✅ storage 456 + lib 2078 + structured_api_v2 + clippy/fmt.
- **Commit:** 836aece3
### C2M3: romper ciclos sdk-planner-query con hoja neutral (Fase 2 Wave 5)
- **Fecha:** 2026-09-13
- **Objetivo:** `search_profile.rs` + `fusion.rs` (inversion fns→sdk); `RelOp` se queda; `crate::planner::` en sdk = 0.
- **Resultado:** ✅ planner 10 + sdk 438 + query 77 + parser 117 + scores 11 + fusion 25 + clippy/fmt.
- **Commit:** beedebc4
### C2S6: registro OperatorRegistry + Dedup sin tocar planner/executor (Fase 2 Wave 5)
- **Fecha:** 2026-09-13
- **Objetivo:** `OperatorRegistry` dispatch-by-name + operador-ejemplo `Dedup[field]` + test extension 4/4 + docs/api mismo PR.
- **Resultado:** ✅ scopes 209/209 + extension 4/4 + check/clippy/fmt (acyclic global rojo pre-existente exceptuado BND-03).
- **Commit:** c2cdbf3e

### F3X: trait-split storage-index con hoja neutral (Fase 3 Wave 1)
- **Fecha:** 2026-09-14
- **Objetivo:** romper ciclo storage-index (8 aristas) con hoja src/index_port.rs + traits sellados + 6 firmas pub a traits (major ADR-0042); review hallo H1 real (niveles HNSW) corregido con test RED-GREEN.
- **Resultado:** check/clippy/fmt + nextest 783/783 + semver rojo esperado.
- **Commit:** 13f0f729 (diseno 0afa8181)
### F3C: S-split-config B+B con RbacCfg propio (Fase 3 Wave 2)
- **Fecha:** 2026-09-14
- **Objetivo:** 6 dominios + fachada plana (vistas From, 108 sitios intactos) + 7 vars a VANTADB_* breaking; Q1=B/Q2=FIND-89/Q3=A.
- **Resultado:** check/clippy/fmt + config 57/57 + 0 legacy.
- **Commit:** d75459fe (feat!:+BREAKING CHANGE; ADR-0043 datos, firma humana pendiente)
### FIND-89: consolidar lecturas directas de env en `Config` (fuente única)
- **Fecha:** 2026-09-14
- **Objetivo:** migrar 7 ficheros con `env::var` real a vistas de dominio `Config` (F3C); espejos nuevos `VANTADB_OPENAI_API_KEY`/`VANTADB_OPENAI_MODEL`/`VANTADB_EMBEDDING_PROVIDER`/`VANTADB_BACKUP_DIR` (breaking C7 sin shims); excepciones `OTEL_*` + `ENV_REPORTED_VERSION` + test.
- **Resultado:** rg env::var→solo excepciones documentadas; rg VANTA_→solo comentarios; check/clippy-lib/fmt + nextest 118/118 (llm/crypto/telemetry/prefetch/maintenance/metadata); review P2-01 vanta-audit ✅ approve.
- **Commit:** 1ca57649 (S1) + 3a0e42d7 (S2) + 4cdc1970 (S3) + c4ddb217 (S4 feat!:) + 410b9b57 (fix clippy Default/test)
### FIND-91: fallout F3X — `durability_recovery:433` a `contains_node`
- **Fecha:** 2026-09-15
- **Objetivo:** migrar assert `hnsw.nodes.get(..).is_some()` a `hnsw.contains_node(..)` (trait `IndexPort`; `dyn` no expone `.nodes` → E0609 con `--all-features`).
- **Resultado:** ✅ check --all-features + clippy CI Windows + fmt + `cargo test --features failpoints --test durability_recovery` 8/8 (30.71s); review P2-01 vanta-review ✅ approve (semántica idéntica `contains_key`, assert por-nodo intacto). Nota: perfil nextest `chaos` excluye el binario por diseño (default-filter solo `chaos_integrity_failpoints`) — sin Heavy.
- **Commit:** 71aedbde (fix 1 línea; hooks verdes)

### FIND-93: `cloned_sole_buffer` bajo feature `rayon` (no era dead code)
- **Fecha:** 2026-09-15
- **Objetivo:** clippy `-D warnings` rojo en dependientes de `vantadb` por asimetría de feature-gating (caller `existing_for_batch_many` bajo `rayon`, método sin gatear; proxy usa `default-features = false`).
- **Resultado:** ✅ `#[cfg(feature = "rayon")]` simétrico (1 atributo, 0 deuda) + clippy vantadb/proxy 0 + engine 383/383 + workspace check; review P2-01 approve.
- **Commit:** 0cd54e47

### EMB-10: build con `embed-local` + fix `token_type_ids` + cat test real
- **Fecha:** 2026-09-16
- **Objetivo:** binario `vanta-cli` con motor ONNX + runtime-switchable por config + señal semántica verificada.
- **Resultado:** ✅ build 13m36s (`ort` 4.64MB + tokenizers en deps) + fix `run_onnx` (feed por nombre + ceros `token_type_ids`; sin él inferencia muerta con dummy silencioso) + cat pares ≥0.91 vs impares ≤0.78 + matriz switch (ollama-sin-server avisado) + clippy 0 + llm 4/4; review P2-01 approve. HALLAZGOS → FIND-100/101/102 (FIND-B cubierto por EMB-13).
- **Commit:** a0f65d29

### EMB-16: prefijos e5 (`query:`/`passage:`) por familia + margen medido
- **Fecha:** 2026-09-16
- **Objetivo:** `LocalOnnxProvider` con prefijos familia e5 (MiniLM/resto sin prefijo) + `embed_query` en trait + fix doble `encode` en `run_onnx`.
- **Resultado:** ✅ margen asimétrico 0.1204 (+48% vs simétrico) + llm 14/14 + clippy 0 + fmt; review P2-01 approve.
- **Commit:** be6a3c27

### FIND-100: graceful ante onnxruntime incompatible
- **Fecha:** 2026-09-17
- **Objetivo:** el init ORT no debe abortar sin dylib compatible; degradacion a dummy determinista.
- **Resultado:** ✅ `resolve_ort_dylib_path()` + `ensure_ort_ready()` (init_from graceful + probe `ort::api()` bajo catch_unwind fuera de G_ENV) + `try_load_session` con catch_unwind y `warn!(fallback=true)`; 15/15 llm serial EXIT=0 + clippy 0 + fmt; P2-01 approve + follow-ups (commit() bajo catch_unwind, pin serial `--test-threads=1` documentado).
- **Commit:** 22d5a142 + 25109073 (follow-ups P2-01)

### FIND-101: `vanta-cli query` doc-vs-fix — FIX parse-then-open (plan 2026-09-17-seguimiento-mvp Wave1)
- **Fecha:** 2026-09-18
- **Objetivo:** `query` abría siempre read-only (`data.rs:212`) → todo IQL INSERT/UPDATE/DELETE fallaba; decidir doc-vs-fix con evidencia + implementar + `--help` coherente, sin rediseño IQL.
- **Resultado:** ✅ FIX parse-then-open: helper privado `query_is_mutating` (mismo `parse_statement` que el executor sobre misma entrada → coherencia por construcción; reads `SELECT`/`Query`→ro, 5 mutantes→rw, parse-fail/LISP→ro) + `engine.flush()?` gated en `Write` (root cause #2: WAL-buffered invisible a reopen read-only, ERR-050b; precedente `cmd_put crud.rs:131`) + help 3 líneas en `cli.rs`. fmt ✅ + clippy `-D warnings` 0 ✅ + `cli_tests` 85/85 (test nuevo `test_find101_query_insert_mutates`: RED pre-fix con `read_only` exacto → GREEN CRUD) ✅ + `--help` smoke ✅ + binario E2E en Temp INSERT→SELECT→UPDATE→DELETE→SELECT-vacío ✅ + OCR delegation sin bloqueantes + P2-01 vanta-review ✅ approve (0 bloqueantes; sugerencia help `INSERT MESSAGE` aplicada). Colateral: TUI REPL mismo bug latente → propuesta fila FIND nueva (fuera de scope).
- **Commit:** de3bd119 (selectivo 4 files; hooks pre-commit verdes; sin PUSH)
- **Nota:** fila FIND-101 en `docs/dev/Backlog.md` pendiente de sync por el lead (prohibido en este slice).

### FIND-109: `vanta-cli wal salvage` opt-in + MCP WAL error tipado
- **Fecha:** 2026-09-18
- **Objetivo:** MCP hard-down por WAL truncado sin via de reparacion (ERR-011 aborta open rw).
- **Resultado:** wal salvage opt-in (preview read-only + prefijo coherente + quarantine `.salvage[.N]` + reporte kept/discarded) + `WalCommand::Salvage --dry-run` + `recovery_hint()` en `Wal` (code intacto); guard ERR-011 intacto; MCP tipado imposible sin rediseno (cero bytes JSON-RPC) -> documentado + hint; fixture real Temp kept 146/discarded 8, `doctor` + `vacuum` exit 0; wal 69/69 + wal_resilience 5/5 + error 121/121 + nextest 2259 + clippy 0; P2-01 approve (2 Baja -> follow-ups c1eacebd).
- **Commit:** 9c881ac5 + c1eacebd (follow-ups P2-01)

### FIND-102: `tests/sdk_serialization.rs` no compila - verify-only
- **Fecha:** 2026-09-18
- **Objetivo:** 39 errores EMB-10 (`QueryResult` no declarado) ensucian `check --tests`.
- **Resultado:** drift autocorregido por refactors intermedios (FIND-49 + AST-002 re-exports); 0 errores, 16/16 verde, asserts intactos, cero codigo tocado; P2-01 approve.
- **Commit:** bb33903e (docs-only; subject con backslash corregido via amend)

### IMPL-112-S1: runner local de ingesta (slice mecanico 1)
- **Fecha:** 2026-09-18
- **Objetivo:** construir el `R` local para `start_ingest<R>`: `IngestRunnerCfg` + enum `ConcreteRunner` + wiring `wiki_ingest`.
- **Resultado:** `IngestRunnerCfg` (TOML minima + env con precedencia) + enum por delegacion (reusa `LlmRunner`, sin `Box<dyn>`) + runner por llamada + `NoLlm` vive + inputSchema intacto; G0/G1/G3 + tests 1-6,9-10; veredicto S2: SIN FRICCION; P2-01 approve.
- **Commit:** 1dd9019c

### IMPL-112-S2: runners ollama/openai (slice mecanico 2)
- **Fecha:** 2026-09-18
- **Objetivo:** completar matriz owner local+ollama/openai: tests 7-8 G2 + mock-escribe.
- **Resultado:** degrada-P4 real en ambas variantes (puerto cerrado / sin key) + mitad canned escribe; 0 codigo prod (S1 ya traia variantes); 5/5 wiki_ingest_runner; matriz IMPL-112 completa; P2-01 approve.
- **Commit:** 82cc2c2d

### FIND-117: TUI REPL mutantes (limite documentado adaptativo)
- **Fecha:** 2026-09-18
- **Objetivo:** mismo bug FIND-101 en REPL con engine read-only long-lived (no vale parse-then-open directo: lock exclusiva).
- **Resultado:** diseno limite-documentado adaptativo (E1-E4 con evidencia: RO rechaza, handle RW imposible con RO vivo, RW-de-entrada regresa, swap = rediseno) + flush tras Write; tui::repl 5/5; P2-01 approve.
- **Commit:** 83b176bc

### API-03: W2 HTTP OpenAPI-first — REST + cursor único (review P2-01 ✅)
- **Fecha:** 2026-09-25
- **Objetivo:** HTTP alineado a OpenAPI-first: plurales sin verbos, `/api/v2` total, status=YAML (201 en creates), cursor opaco string + `has_more` + `limit` (0 `offset`), `RecordInput` opcional, 5 drifts YAML.
- **Resultado:** ✅ Contrato 4/4 — parity 10/10 + script 37 paths OK · greps 0/0 · smoke curl 15/15 status=YAML · e2e 17/17 · clippy server ✅ · docs-coverage 0 gaps. 7 rutas migradas (`maintenance/purge`→`DELETE …/expired-records`, `conversation/add`→`/api/v2/conversations`, `skill/listing`→`GET /api/v2/skills`, `POST /threads/{id}`→`/threads/{id}/messages`, …). Review P2-01 ✅ APPROVE (2 rondas). Deuda declarada: `versions` sin paginar, cursor types en desktop, comentarios vanta-memory.
- **Commit:** 94009297 (local, sin push)

### API-05: W4 proxy — auth /snapshot + endpoints canónicos + config (review P2-01 ✅)
- **Fecha:** 2026-09-26
- **Objetivo:** Cerrar la exposición de `/snapshot` (sessions/cost sin auth) + endpoints canónicos + config (self-loop, ttl, rate).
- **Resultado:** ✅ Contrato 4/4 — `/snapshot` sin key **401** · con key **200** · key inválida 401 (smoke live reproducido por el revisor con binario fresco; test e2e real `api05_snapshot_auth.rs`) · suite proxy **291/0** (18 binarios, ×3) · `rg spaceId server.rs` = 0 · 11 routes. **Sin bypass loopback** (decisión documentada: el contrato exige 401). FIND-155 (desktop sin key → 401 hasta migrar; mock e2e verde). Review P2-01 ❌ R1 (comentario stale) → fix → ✅.
- **Commit:** f0c3f95f (local, sin push)

### API-06: W5 IQL — versión + sintaxis + literales + AST (review P2-01 ✅)
- **Fecha:** 2026-09-26
- **Objetivo:** `IQL_VERSION` + 1 sintaxis documentada + case definido + fixes `==`/Int/string + AST JSON.
- **Resultado:** ✅ Contrato 4/4 — `rg IQL_VERSION src/` 14 hits (mod.rs:17,20 · gate grammar.rs:120 · re-export lib.rs:180) · `--lib parser` 127/0 + `--test parser` 4/0 + parity 11/0 · repros RED→GREEN (`==`, `42→Int`, big-int, `from` minúscula, `'quote'`) · ejemplo YAML consumido. `FIND-156` (AST JSON en bindings) + `FIND-157` (coerción Int/Float pre-existente) registrados. Review P2-01 ronda 1 ❌ → R1/R2/R3 → ✅.
- **Commit:** a6f6a70b (local, sin push)

### API-07: W6 CLI — POSIX + `--json` global + flags simétricos (review P2-01 ✅)
- **Fecha:** 2026-09-26
- **Objetivo:** Normalizar la superficie CLI: `--json` global completo (humano truncado solo TTY), flags simétricos `--in`/`--out`, `--limit` canónico, QUERY posicional, `count` sin DB exit≠0, `put` vía SDK, lecturas read-only (Regla 8).
- **Resultado:** ✅ Contrato 5/5 — help 39 capturas · smoke 56/56 · cli_tests 88/88 · lib 2124/0 · fmt/clippy 0 · review P2-01 ronda 2 ✅ APPROVE. Lead verify: 88/88 (`-p vantadb`); el hang de `cargo test` sin `-p` era unificación de features con default-members → test gateado `#[cfg(not(feature = "server"))]` (suite unificada 87/87).
- **Commit:** f6c395ef (local, sin push)

### FIND-190: put-overwrite Cold invisible a get/list (cache volátil) (review P2-01 ✅)
- **Fecha:** 2026-10-01
- **Objetivo:** Invalidar la entry volátil en overwrite Cold (single + batch) — mismo patrón que txn/delete; elimina la necesidad del workaround VER-07.
- **Resultado:** ✅ RED→GREEN 2/2 · storage::engine 390/390 · lib 2302/0 · check/clippy/fmt 0 · P2-01 APPROVE (O1→FIND-222, O2→FIND-223; integration targets → CI).
- **Commit:** 06496a9a (local, sin push)

### FIND-237: CLI `migrate` — el global `--db` como fallback del `target` posicional (review P2-01 ✅)
- **Fecha:** 2026-10-04
- **Objetivo:** `migrate check/plan/run --db <db>` fallaba con "required arguments were not provided: <TARGET>" (target posicional obligatorio; única convención distinta al resto del CLI, que usa solo el global `--db`). Fix: `target: Option<String>` + resolución en el dispatch (positional > global) + ejemplos en `--help` (after_help).
- **Resultado:** ✅ Contrato 4/4 — binario real `FALLBACK_EXIT=0` + `POSITIONAL_EXIT=0` (precedencia discriminada con `--db` a path inexistente) · nextest lib migrate 11/11 · cli::tests 5/5 · E2E binario 3/3 · cli_tests 94/94 · fmt/clippy exit 0 · campaign_verify_cmd passed · OCR sin Critical/High · P2-01 ronda 1 ❌ (solo registro: comando sin `--ignore-default-filter`) → ronda 2 ✅ APPROVE. Nota: `cli_tests` está excluido por `default-filter` de nextest → requiere `--ignore-default-filter` para correrlo. Doc impact: `CONFIGURATION.md:387` §migrate sigue stale (FIND-224 pre-existente; sin nuevo drift por este fix).
- **Commit:** ba9f38c4 (local, sin push)

### DUR-01: Auditoría del fsync real (WAL, snapshots, GC)
- **Fecha:** 2026-10-04
- **Objetivo:** Mapa path→fsync real (WAL write/rotate/archive · snapshots/checkpoint · GC/compaction) con gaps clasificados (fix/FIND/OK) y ≥1 gap resuelto con evidencia; si hay fix de fsync, medido contra `canonical_p99` (Regla 9).
- **Resultado:** ✅ Contrato completo — mapa con evidencia file:línea en `docs/dev/tasks/DUR-01.md` · **G1 resuelto**: `sync_parent_dir` al crear WAL (2 sitios, `src/wal.rs`) + helper robusto para parent vacío (`src/utils/fs.rs`, fix P2-01) · FIND: G2 (checkpoint durable antes que los artefactos del snapshot), G3 (`rotate_all`/`compact_wal` sin archive), G12 (archives de auto-rotate nunca replayed — hallazgo del review) · G4-G11 OK-justificados. Gates: fmt/clippy 0 · nextest `wal` 91/91 · `snapshot` 46/46 · docs coverage 0 gaps · OCR sin Critical/High/Medium · review P2-01 ronda 1 `changes-required` → 2/2 ítems incorporados; ronda 2 en background (ACCEPT del orquestador). Bench: compile gate ✅; timed run inviable en presupuesto (criterion est. 8477 s) — fix fuera del path medido (pure in-memory CPIndex).
- **Commit:** d4cff504 (local, sin push)

### DUR-03: H-023 — put sobre key expirada-sin-purgar (Node ID collision) → purge-on-write
- **Fecha:** 2026-10-04
- **Objetivo:** Un `put` legítimo sobre una key cuya entrada expiró pero aún no fue purgada moría con `NodeIdCollision` (`put_one`/`put_batch_inner`/`put_record_exact` conflacionaban el nodo físico oculto por lazy TTL con una colisión real de id). Fix = purge-on-write: purga el registro expirado con los primitivos de `delete_inner` (engine.delete + replace_derived_indexes + purge_key) y escribe fresco v1 — estado observable idéntico al workaround documentado `purge_expired()`+`put()`.
- **Resultado:** ✅ RED→GREEN 4/4 tests nuevos (expirado→put→ok; variante `put_batch`; race vs sweeper con arranques vivos/expirados + stress 20/20; foreign-node collision) · `edge_cases` 28 run/27 passed (único fallo: FIND-244 pre-existente verificado en HEAD) · suite `-p vantadb` 2549/2549 (shared) + rerun aislado `target/dur03` · fmt/clippy exit 0 · docs gates 0 · OCR 0 Critical/High · review P2-01 adversarial ronda 1 `changes-required` → corregido (generation guard RwLock: write en purge paths, read en upserts vivos; re-check con estado refrescado; refusal en transacción activa) → ronda 2 APPROVE. Hallazgos derivados: FIND-244/245/250/251/252.
- **Commit:** 4cc1402d (local, sin push)

### VER-10: Attestation de escritura — write receipts (put_certified + verify_write_receipt)
- **Fecha:** 2026-10-05
- **Objetivo:** Extender el certificado de delete (VER-02) a writes: el write path declarado (`put`) emite un recibo verificable (`WriteReceipt` schema v1) con content binding sha256 + referencia a la cadena WAL VER-01 (**evidencia equivalente** — `append` no expone el `record_hash` del frame; FIND-307) + límites declarados; verificación no claim-driven (schema → integrity → re-scan live); opt-in: `put` intacto, delete cert y `vanta-cli verify` intactos.
- **Resultado:** ✅ RED→GREEN — 11 tests integración (`tests/write_receipts.rs`: válido, reopen, tampered, content, borrado, claimless, sparse, TTL, non-finite, determinismo) + 12 unit (schema/content-hash/finitud); suites `--test write_receipts --test certified_delete` 20/20 + lib 2356/2356; fmt/clippy `--all-targets` verdes; docs gates verdes · review P2-01 adversarial ronda 1 `changes-required` (inyectividad de floats no finitos en el binding JSON + límites incompletos) → corregido (rechazo en emisión ANTES del write + fail-safe en verificación + límites re-enunciados) → ronda 2 **APPROVE**. Hallazgos derivados: FIND-307 (frame `record_hash`/posición en `append` — Engine/Arch), FIND-308 (batches/CLI/MCP/bindings). Doc: `docs/api/WRITE_RECEIPTS.md` + links en CERTIFIED_DELETE/WAL_INTEGRITY.
- **Commit:** 7841879e (local, sin push)

### STRAT-05: Ruta object storage (S3/blob) — research + spec + decisión (ADR-0056) (master plan 0.9.0 F0, Task 65)
- **Fecha:** 2026-10-06
- **Objetivo:** decidir viabilidad/costo/tradeoffs de la ruta object storage antes de que el trigger Pro/Cloud la fuerce sin diseño (contrato plan Task 65 L1867): snapshot-level vs segment-level vs backend nativo, con decisión documentada (ADR); sin implementación; fuentes citadas (Regla 11).
- **Resultado:** ✅ research-doc `docs/dev/research/strat-05-object-storage.md` (§1 estado real con evidencia file:line — `BackendKind` `backend.rs:105-132`, mmap `vstore_L*` (STORAGE-TIERS), CLI/SDK de backup existentes `cli.rs:146-168,409-419`/`impl_export.rs:315,531`; §2 tres opciones con precedentes verificados: DuckDB read-only / SQLite CBS bloques+manifest / sqlite-s3vfs sin locking → corrupción / Litestream WAL→S3 / SlateDB+Lance motores nuevos / Chroma 35-100ms; §3 modelo de costos con supuestos marcados y precios fetch-verificados — S3 $0.023/GB-mo, PUT $0.005/1k, GET $0.004/10k, egress $0.09/GB; R2 $0.015/GB-mo, egress $0; §4 decisión: **snapshot-level ahora (BIZ-14), segment-level al trigger Pro/Cloud, nativo diferido con condiciones**) + **ADR-0056 `proposed`** (MADR; ratificación del owner pendiente) + task file. **Frontera declarada** (pre-mortem): veredicto 2026 y BIZ-14 citados sin re-litigio (backup offsite vs storage path). Fuentes 16/16 fetch-verificadas 2026-10-06 (TSYS-13) + re-verificadas por el reviewer (10/10 fieles). Gates docs 4/4 verdes (check-links 0 / check-docs GATING clear / gen-index --check / coverage 0 gaps) + markdownlint 0. Review P2-01 `vanta-review` tier Fast: **APPROVE** (0C/0H; FIND-1 Medium corregido inline; FIND-2/3/5 corregidos; FIND-4 descartado; FIND-6 anotado para el spec futuro de BIZ-14; FIND-7 aplicado). OCR delegation: N/A (diff docs-only; el único reviewable del árbol es WIP ajeno `heavy-test-lock.ps1`). **Deuda declarada:** ninguna nueva; la implementación de BIZ-14 espera su trigger Pro/Cloud (la decisión ya no es el bloqueo).
- **Commit:** cb0bd21e (docs research, local sin push) + este commit de cierre docs(avance)

### WIRE-12: IQL `LIMIT`/`OFFSET` — paginación end-to-end (master plan 0.9.0 F0, Task 68)
- **Fecha:** 2026-10-06
- **Objetivo:** cerrar la fricción de paginación IQL: `LIMIT <n>`/`OFFSET <m>` end-to-end (parser→AST→planner→executor) con tests (parse + ejecución + interacción RANK BY/AS OF) y `docs/api/IQL.md`; sin breaking (LIMIT ya reservada, sin consumidor).
- **Resultado:** ✅ Contrato completo — parser estricto (boundary check, número requerido, orden invertido/duplicado/trailing = Failure; `;` final tolerado) en FROM/MATCH y SELECT + 3ª posición canónica de AS OF; AST `limit/offset`; `LogicalOperator::Offset` vía OperatorRegistry (patrón C2S6: variante + `physical_plan/offset.rs` + register) con ventana `Limit` ensanchada a n+m → skip-then-take sin editar los matches probados de planner/executor; `IQL_VERSION` 2→3 (+ `IQL_VERSION_MIN_PAGINATION`); docs IQL.md v3 (sección Pagination + AST JSON + nota registry). Gates: lib 2399/2399 · integración 19/19 (no-cli por lock del exe MCP) · public_api 1/1 (snapshot refrescado: +55 líneas propias + 463 de drift pre-existente de VER-10/MEMG-17 acknowledgeadas) · fmt/clippy/docs gates ✅ · review P2-01 adversarial ronda 1 REQUEST CHANGES (snapshot public-api) → fixes → ronda 2 **APPROVE**. Hallazgos derivados: FIND-312 (trailing general sin paginación), FIND-313 (ventana vectorial hardcoded 5).
- **Commit:** 22585e1b (local, sin push)

### WIRE-13: IQL agregaciones (`COUNT`/`SUM`/`GROUP BY`) - end-to-end (master plan 0.9.0 F0, Task 69)
- **Fecha:** 2026-10-06
- **Objetivo:** habilitar el caso ERP/facturas (agregar sin exportar): agregaciones básicas `COUNT(*)`/`COUNT(field)`/`SUM(field)` + `GROUP BY` single-field end-to-end (parser→AST→planner→executor) con tests (parse + ejecución + tipos de resultado) y `docs/api/IQL.md`; sin breaking; resultado serializable en el AST JSON. Premisa del compact ("pipeline de agregación existente") corregida en DISCOVERY: el habilitador real es `OperatorRegistry` (C2S6) + Volcano.
- **Resultado:** ✅ Contrato completo - gramática `SELECT` con funciones de agregación + `GROUP BY` en posición SQL; `AggregateFunc` + campos AST + `LogicalOperator::Aggregate` como tercera extensión del registry (variante + `physical_plan/aggregate.rs` + register, **cero ediciones semánticas** a planner/executor: `git diff src/planner.rs` = 0); `PhysicalAggregate` drena el child y emite una fila sintética por grupo (`id=0`, first-seen, null grouping SQL, SUM Int saturating/Float promocionado/Null sin aportes, sin GROUP BY + vacío = 1 fila); **rechazos loud** de todo lo diferido (AS OF/LIMIT/OFFSET con agregación, mezcla fields+agregados, GROUP BY sin agregado o con key colisionante, multi-field, superficie FROM/MATCH) + trailing tras la superficie estricta (fix del review C-1: `GROUP BY c WHERE ...` ya no descarta el WHERE); mensajes estables por `ErrorKind::Fail`; `IQL_VERSION` 3→4 (+ `IQL_VERSION_MIN_AGGREGATION`); docs IQL.md v4 (§Aggregations + AST JSON + registry) + VERSIONING/openapi. Gates: lib **2440/2440** · integración 20/20 (no-cli) · fmt/clippy verdes · public_api 1/1 (refresh oficial +74/−0, solo símbolos nuevos) · docs gates 4/4 · OCR sin Critical/High · review P2-01 adversarial `vanta-review` ronda 1 REQUEST CHANGES (C-1 trailing silencioso + M-1 colisión key + M-2 AST programático + M-3 tests + L-1..L-4) → fixes → re-verify → ronda 2 **APPROVE**. Deuda: ninguna nueva; FIND-316 (AVG/MIN/MAX/HAVING, multi-field GROUP BY, paginación de grupos, valid-time-aware) + subcase R-1 anotado en FIND-312.
- **Commit:** 753b9f78 (local, sin push)

### SAFE-01: Validación de límites de memoria y buffers en operaciones del motor
- **Fecha:** 2026-10-15
- **Objetivo:** Prevenir desbordamientos y validación estricta de límites de memoria (`MAX_PERSISTED_NODE_BYTES`) en des-serialización de payloads.
- **Resultado:** ✅ Validación aplicada en `deserialize_node_payload` y puntos de ingesta.

### SAFE-02: Jerarquía estricta de locks y prevención de ciclos en el storage engine
- **Fecha:** 2026-10-15
- **Objetivo:** Reforzar el cumplimiento de la jerarquía de adquisición de locks (`concurrency-async.md` Regla 8) entre `insert_lock` y estructuras secundarias.
- **Resultado:** ✅ Eliminados bloqueos cruzados y ciclo AB-BA en transacciones.

### PERF-02: Optimización de deserialización y reutilización de buffers
- **Fecha:** 2026-10-15
- **Objetivo:** Optimizar las rutas críticas de lectura de nodos reduciendo allocations efímeras.
- **Resultado:** ✅ Reutilización eficiente de buffers en operaciones de escaneo y lectura.

### PERF-03: Reducción de contención en el write path
- **Fecha:** 2026-10-15
- **Objetivo:** Desacoplar la actualización de estadísticas de cardinalidad del lock de inserción principal para maximizar el throughput concurrente.
- **Resultado:** ✅ Actualización asíncrona/desacoplada de estadísticas fuera del lock crítico.

### STD-04: Estandarización de interfaces de retorno y errores tipados en storage
- **Fecha:** 2026-10-15
- **Objetivo:** Unificar los tipos de retorno y códigos de error en el subsistema de almacenamiento.
- **Resultado:** ✅ Mapeo tipado exhaustivo a través de `crate::error::Error`.

### HIG-03: Higiene y auditoría de `#[allow(dead_code)]` en `src/`
- **Fecha:** 2026-10-18
- **Objetivo:** Auditar exhaustivamente todas las directivas `#[allow(dead_code)]` en el código fuente de `src/`, eliminando código muerto real o documentando de forma explícita las justificaciones de compatibilidad binaria/esquema y feature gates.
- **Resultado:** ✅ Módulos auditados; código huérfano saneado y remaining occurrences formalmente justificadas bajo `cargo clippy --workspace --all-targets -- -D warnings`.

### DX-07: Resolución de ruta de `.vanta_profile` fuera del CWD
- **Fecha:** 2026-10-18
- **Objetivo:** Evitar la escritura de `.vanta_profile` en el directorio de trabajo del usuario (CWD), resolviendo hacia `VANTADB_PROFILE_PATH` o `~/.vanta/.vanta_profile`.
- **Resultado:** ✅ Actualizado `src/hardware/mod.rs` para resolver la ruta en el directorio de usuario o variable de entorno sin ensuciar el CWD.


### API-10: Schema-first / codegen para DTOs y tipos públicos
- **Fecha:** 2026-10-18
- **Objetivo:** JSON Schema de los DTOs wire + test de paridad
- **Resultado:** ✅ Slice 1 entregado: `docs/api/wire-schema.json` + modulo `wire_schema_parity` (`tests/sdk_serialization.rs:147-171`). Slice 2 (codegen real) declarado post-1.0.

### DOC-22: Single Source of Truth documental (Document Authority Map)
- **Fecha:** 2026-10-18
- **Objetivo:** Un mapa que diga que documento manda sobre que
- **Resultado:** ✅ Document Authority Map generado (concepto -> fuente canonica) en `docs/api/SURFACE_MANIFEST.md:48-64`.
- **Commit:** f84f7c70

### DX-02: H-027: extender doctor con preflight de entorno
- **Fecha:** 2026-10-18
- **Objetivo:** doctor con preflight de entorno
- **Resultado:** ✅ Preflight de disco, locks, onnxruntime, codepage, Python/Node (`src/cli_handlers/diagnostics.rs:217-290`).
- **Commit:** c777d72c

### FIND-201: `integrations/README.md` §"shim temporal" stale (FIND-94 ya migrado en `880cd0f3`)
- **Fecha:** 2026-10-18
- **Objetivo:** README de integrations sin texto stale
- **Resultado:** ✅ Seccion stale corregida y shim documentado como eliminado (`integrations/README.md:71-78`).
- **Commit:** 9f773e92

### FIND-216: OTel/tracing: spans del ciclo de query (hoy solo `http_request`)
- **Fecha:** 2026-10-18
- **Objetivo:** Telemetria por defecto en el SDK
- **Resultado:** ✅ 185 `#[instrument]` en put/get/delete/search + configuracion OTEL documentada (`CONFIGURATION.md:508`).

### FIND-218: Política de estabilidad/deprecación de API formalizada
- **Fecha:** 2026-10-18
- **Objetivo:** Politica de versionado y deprecacion
- **Resultado:** ✅ `docs/api/VERSIONING.md:97-110` + `DEPRECATIONS.md` + 10 `#[deprecated]` reales en el codigo.

### FIND-322: `ARCHITECTURE.md` declara truth v0.6.1 (stale vs tren 0.9.0)
- **Fecha:** 2026-10-18
- **Objetivo:** ARCHITECTURE.md alineado al workspace
- **Resultado:** ✅ Doc y frontmatter alineados a la 0.8.0, verificados 2026-10-15.
- **Commit:** b678b335

### SURF-01: Public Surface Manifest (PUBLIC/INTERNAL/EXPERIMENTAL/FROZEN/RETIRED)
- **Fecha:** 2026-10-18
- **Objetivo:** Clasificacion mecanica de la superficie publica
- **Resultado:** ✅ Manifest PUBLIC/INTERNAL/EXPERIMENTAL/FROZEN/RETIRED generado por `scripts/docs/gen-surface-manifest.mjs`; task file cerrado.
- **Commit:** f84f7c70
### FIND-411: la dimension del vector se valida y los tres backends coinciden
- **Fecha:** 2026-10-20
- **Resultado:** ✅ Un vector de otra longitud puntuaba `0.0` — el **mejor** score Euclidean
  posible — y se colaba por delante de los validos; ademas flat, HNSW e HNSW-inline no
  coincidian entre si. Ahora `NaN` es el centinela (que el codigo ya entendia via
  `total_cmp_sim`), el error se lanza en los puntos de entrada (`put`, `put_batch`,
  `bulk_import`, `put_record_exact` y toda busqueda con `query_vector`) con
  `Error::DimensionMismatch { expected, got }`, y la dimension del indice tiene **una sola
  fuente** (`IndexPort::vector_dimension()`), compartida por el SDK y el MCP.
  **BREAKING** (minor en 0.x): el score Euclidean pasa de `-d^2` a `-d` en los tres backends
  (ej. `-2.0` -> `-1.4142135`) y el indice admite una sola dimension por base de datos (la
  regla "un modelo por base de datos" quedo escrita en `docs/api/EMBEDDINGS.md` y en el
  tutorial 05).
  Verificacion: 2483/2483 tests del lib, 7/7 de integracion, 171/171 del MCP, clippy y fmt
  limpios. Reviews adversariales aplicadas: se corrigio un **NaN que llegaba al usuario** y un
  **panic por indice fuera de rango** en `IvfIndex::build` con dimensiones mezcladas, y se
  unifico la regla de dimension (habia dos implementaciones divergentes, una en el SDK y otra
  en el MCP). Huecos que quedan abiertos con fila propia: `FIND-487`, `FIND-488`, `FIND-492`,
  `FIND-493`, `FIND-494`, `FIND-495`.

### FIND-413 / FIND-414: el fallo de embedding se propaga y el provider lee la configuracion real
- **Fecha:** 2026-10-20
- **Resultado:** ✅ La busqueda vectorial IQL se traga el `Err` del embedding (4 sitios
  `if let Ok(vec) = ...`) y devolvia 0 hits: un provider caido se confundia con "no hay
  datos". Ahora el error llega al usuario con el motivo, y `LocalOnnxProvider::new` lee la
  config real del entorno (`VANTADB_EMBEDDING_PROVIDER`, `VANTADB_LOCAL_MODEL`,
  `VANTADB_STRICT_EMBEDDINGS`, `VANTADB_ALLOW_DUMMY_EMBEDDINGS`) en vez de
  `LlmCfg::default()` (que los ignoraba).
  **BREAKING** (minor en 0.x): una busqueda IQL por vector con el provider caido devuelve
  **error** en vez de 0 hits silenciosos.
  Al implementarlo se arreglo de paso un **test preexistente roto** (`tests/write_receipts.rs`
  usaba vectores de 2 dimensiones donde el indice es de 3, asi que la validacion de dimension
  rechazaba el `put` antes de llegar al comportamiento que el test queria probar; uno
  reventaba y otro pasaba **por el motivo equivocado**). Y el auto-embedding del INSERT dejo
  de persistir vectores sinteticos sin marcador (paridad con el path MCP): con el opt-out
  activo se guardaba un vector 384d sintetico como si fuera real (vecinos falsos, peor que
  null) — `FIND-485`.
  La review adversarial obligo ademas a ampliar el seam a `Result<Option<Vec<f32>>>` (los
  tests del rama vectorial tienen que poder distinguir "sin capacidad de embeddings" de
  "provider caido" sin tocar la red) y a corregir el test de `FIND-414`, que affirmaba un flag
  privado en vez del comportamiento.
  Verificacion: 2524 tests con `--features encryption`, 2483 sin, 14/14 de los tests del
  contrato, clippy y fmt limpios.

### Cierre de filas ya arregladas (auditoria de docs y READMEs)

- **Fecha:** 2026-10-20
- **Resultado:** ✅ 18 filas del backlog seguian marcadas como pendientes cuando su hallazgo **ya estaba corregido**: los agentes de docs y READMEs los arreglaron en los PRs #268 y #270, pero sus filas nunca se cerraron. Se cierran aqui: `FIND-415` (S3-B4), `FIND-416` (S5-B1), `FIND-417` (S5-B2), `FIND-418` (S5-B3), `FIND-419` (S5-B4), `FIND-420` (S5-B6), `FIND-424` (D-05), `FIND-425` (D-06), `FIND-426` (D-07), `FIND-433` (S7-01), `FIND-434` (S7-03), `FIND-435` (S7-04), `FIND-436` (S7-05), `FIND-437` (S7-06), `FIND-438` (S7-07), `FIND-439` (S7-08), `FIND-440` (S7-09), `FIND-441` (S7-10).
  El detalle de cada correccion esta en los informes de la campana y en el changelog `[Unreleased]`.

### FIND-385 / FIND-387 / FIND-408 / FIND-410 / FIND-422 / FIND-423 / FIND-428 / FIND-431 / FIND-427 / FIND-386: cierres por resolucion verificada

- **Fecha:** 2026-10-21
- **Resultado:** ✅ 10 filas seguian marcadas como pendientes cuando su hallazgo **ya estaba
  resuelto** (los agentes de docs y READMEs los corrigieron en los PRs #268 y #270, y el fix del
  changelog en #267). Verificado uno a uno en el repo antes de cerrar:

  - `FIND-385` (S7-02): los 4 badges del README apuntan a `docs-gates.yml` y `platform-ops.yml` (los workflows reales).
  - `FIND-387` (S3-B10): las 7 citas `archivo:línea` de `docs/api/scores.md` actualizadas y la afirmación de comportamiento falsa corregida (los mapeos son `1-d` y `1-s`).
  - `FIND-408` (S5-B7): las 2 tools que faltaban (`memory_reinforce`, `memory_verify_certificate`) documentadas en `skills/vantadb-mcp/references/api-reference.md` (que ahora lista las 88 definiciones).
  - `FIND-410` (D-01b): la entrada del changelog del 0.9.0 ya no afirma 'docs sin claim': describe el estado real.
  - `FIND-422` (S5-A3): la version del registro MCP en `docs/api/MCP.md` es 0.9.0 (era 0.5.0).
  - `FIND-423` (D-04): el quickstart esta pineado a 0.9.0 (era 0.8.0).
  - `FIND-428` (D-09): la cita de `docs/api/MCP.md:198` da 82/88 (era 80/86).
  - `FIND-431` (D-12): el prerrequisito `vanta-cli` documenta 0.9.0 (era 0.5.0).
  - `FIND-427`: el texto de la propia fila `FIND-98` ("79 vs 85 tools") corregido a "79 vs 82".
  - `FIND-386`: la tabla de versiones de `SECURITY.md` estaba congelada en 0.5.x/0.4.x con el crate
    en 0.9.0 -> ahora 0.9.x activa, 0.8.x y 0.7.x con parches de seguridad.


## Auditoria de la gramatica IQL (FIND-502) — 13 hallazgos nuevos (2026-10-21)

- **Fecha:** 2026-10-21
- **Resultado:** ✅ el hueco de cobertura que `FIND-502` declaraba (`src/parser/` nunca auditado
  en profundidad) queda cerrado con una auditoria de solo lectura de la gramatica, el lexer y su
  contrato con el planner y el executor. Informe: `docs/dev/reviews/audit-grammar-iql-20261021.md`.

**Lo que encontro (verificado con `archivo:linea`, y contra la fuente de `nom 7.1.3`):**

- **`FIND-518` (critica):** si la **primera** condicion del `WHERE` no parsea, el `opt(...)` del
  `WHERE` la traga, restaura el input y la query se ejecuta **sin filtro ni error** (escaneo
  completo). El `PhysicalScan::open` materializa todos los nodos en RAM antes de filtrar, asi que
  el fallo escala a agotamiento de memoria. Traza: `src/parser/grammar.rs:207-210` ->
  `nom::opt` ante `Err::Error` **restaura el input** -> `src/executor.rs:217-218`.
- **`FIND-519` (alta):** el trailing input se descarta en silencio en las tres superficies
  (HTTP, MCP, CLI). Casos reales: `WHERE a = 1, b = 2` aplica un filtro y **pierde el otro**;
  `min >= 0.88` degrada el brazo vectorial a match lexico; `PROFILE bogus` corre sin perfil;
  `INSERT ... LINK ...` inserta el nodo **sin arista**.
- **`FIND-520` (alta):** el operador `~` tiene **dos** semanticas (vectorial con `min`, lexico sin
  el) y `docs/api/IQL.md` solo documenta una: `WHERE texto ~ "x"` creyendo semantica vectorial
  hace una contencion lexica de frase; sobre un campo `Int` da **siempre 0 filas sin error**.
- **`FIND-521` (alta):** la misma expresion IQL devuelve **filas distintas** segun la estimacion de
  selectividad del CBO (dos paths vectoriales: refine por fuerza bruta vs HNSW), y con metrica
  Euclidean el `min` **no matchea nada** (el espacio de score no esta documentado).
- **`FIND-522` (alta):** `min = nan` / `min = inf` se aceptan -> 0 resultados silenciosos (el mismo
  sintoma que corrigio `FIND-413`, por otra via).
- **`FIND-523` (alta, DoS):** recursion **sin limite** en subqueries anidados -> stack overflow con
  una query de ~1 MiB (el cap del MCP), que tumba el server y todas las peticiones en vuelo.
- **`FIND-524` (media):** `FROM NODE#abc` devuelve **un nodo arbitrario de toda la base**, sin error.
- **`FIND-525` (media):** `SIGUE 1..5000000000` se descarta en silencio -> la query pasa a ser un
  escaneo **sin traversia**.
- **`FIND-526` (media):** una keyword en minuscula se consume como alias -> `FROM Doc where x = 1`
  devuelve **todos** los `Doc` (la doc afirma que es error de parseo).
- **`FIND-527` (media):** `PROFILE candidate_k` se parsea y se **ignora** en el path IQL, pero la
  doc lo llama "candidate budget".
- **`FIND-528` (media):** el mensaje de error de parseo embebe el input crudo -> una query de 1 MiB
  devuelve un error de ~1 MiB (amplificacion de payload).
- **`FIND-529` (baja):** los tests **fijan como esperado** los silent-drops: no hay ningun test del
  contractual de `WHERE` mal formado, trailing input, `min = nan` ni limite de profundidad.
- **`FIND-530` (alta):** `docs/api/HTTP_API.md` documenta dos formas de IQL que **no parsean**
  (`UPDATE NODE#<id> VECTOR [...]` y `RELATE <a> -> <b> AS <label>`).
- **`FIND-531` (media):** `docs/api/IQL.md` describe seis semanticas que el codigo no tiene.

**Ademas:**

- `FIND-312` (trailing input) sube de 🟡 Media a 🟠 Alta: la auditoria lo confirma en las tres
  superficies y descubre la variante peor (`FIND-518`).
- `FIND-313` (ventana de candidatos hardcoded) sube de 🟢 Baja a 🟠 Alta: el tope **no escala con el
  `LIMIT`** y el segundo path del planner **no lo tiene**.
- `docs/user/QUICKSTART.md:265` (que seguia con `VECTOR_SEARCH('hola mundo')`, una funcion que no
  existe en ningun lenguaje del repo) corregido a la forma real: `<campo> ~ "hola mundo", min = 0.88`.

**Nota (Ponytail lite):** el 80 % de esto se reduce a **una regla**: `opt(...)` alrededor de una
clausula cuyo keyword ya se consumio convierte un error de tipeo en un cambio de significado
silencioso. Hacer `reject_trailing_input` estricto para todas las statements cierra `FIND-518`,
`FIND-519`, la mitad de `FIND-525` y `FIND-526` de una vez.


## FIND-214 y FIND-221: dos filas del area de indexacion y batch (2026-10-21)

- **Fecha:** 2026-10-21
- **Resultado:** ✅ una fila ya estaba resuelta en el codigo (verificada) y otra ya no
  era ambigua (contrato unificado en las tres superficies).

- **FIND-214** (cerrada): la fila decia "ScannIndex: 5 mutex anidados sin orden documentado
  (riesgo deadlock)" con referencia a `src/index/scann.rs:54-62`. El codigo actual ya lo
  **consolido**: `ScannInner` es una unica estructura atomica tras **un** `Mutex`, y el
  propio doc-comment lo dice (`src/index/scann.rs:50-51`: *"consolidated into a single atomic
  struct to eliminate multi-mutex deadlock risks (FIND-214)"*). La fila seguia abierta desde
  la auditoria del 2026-09-30.
- **FIND-221** (reescrita): la ambiguedad del contrato de atomicidad de `put_batch` era real
  y era la parte P1 de la fila. La **validacion** ya es all-or-nothing (shape,
  namespace/key, metadata y dimension del vector, antes de escribir nada) y el **commit** es
  por chunks de 1000 sin rollback. El problema: la **descripcion del tool MCP**
  (`vantadb-mcp/src/handlers/tools.rs:115`, que es lo que lee un agente) decia
  "All-or-nothing: an invalid input fails the whole call before any write" y
  `docs/api/MCP.md:289` decia "a single all-or-nothing batch" — mientras el SDK y
  `docs/api/PYTHON_SDK.md:215`odian el contrato real (por chunk). Ahora las cuatro
  superficies dicen lo mismo, con la distincion explicita entre validacion y commit.
  Lo que queda en la fila es la **atomicidad real del lote** (2PC entre WAL y KV), que
  necesita soporte del motor.


## Auditoria de physical_plan / planner / executor — 20 hallazgos (FIND-552..571, 2026-10-21)

- **Fecha:** 2026-10-21
- **Resultado:** 🔴 2 de los 12 modulos de `FIND-503` auditados a fondo. Informe:
  `docs/dev/reviews/audit-physical-plan-20261021.md`. 4 criticos, 5 altos, 7 medios, 4 bajos.

**Los cuatro criticos:**

- **`FIND-552`:** la proyeccion (`FETCH`) elimina los metadatos que necesitan los post-filtros
  ⇒ `AS OF` deja de filtrar y devuelve nodos **no validos** en el instante pedido (y el filtro
  de confianza de `SemanticSummary` deja de filtrar). El predicado temporal (SCH-03 / ADR-046 D3)
  queda anulado sin error.
- **`FIND-553`:** el path vectorial por defecto **no aplica el scope de entidad** ⇒
  `FROM Doc WHERE content ~ "x", min = 0.5` devuelve los vecinos mas cercanos de **todo** el
  store; y la query "equivalente" con un filtro relacional extra scopea (rama refine) ⇒
  **anadir un filtro redundante cambia el conjunto de filas**.
- **`FIND-554`:** el CBO puede **eliminar un predicado del `WHERE`** en silencio: la
  selectividad se estima con poblaciones distintas (KV store vs indice vectorial), sin `clamp`,
  asi que `sel >= 1.0` descarta el filtro del plan.
- **`FIND-555`:** la reserva de memoria del governor **no se libera si la ejecucion falla** ⇒
  ~20 queries fallidas dejan el motor inservible (`Error::ResourceLimit` en toda query
  posterior, incluidas las de escritura). Con el proveedor de embeddings caido, "provider
  caido" se convierte en "motor muerto" tras ~20 intentos.

**Ademas (altos):** un `WHERE` con campo cualificado (`p.name`) no matchea nunca (`FIND-556`);
el subquery con 0 filas deja pasar **todas** las filas (`FIND-557`); el escalar de un subquery
es el primer campo **alfabetico** (`FIND-558`); la clausula `ROLE` prunea todo sin `_owner_role`,
una clave que **nadie escribe** (`FIND-559`); y el conmutador entre los dos paths vectoriales
depende de una estimacion que puede ser `0.0` ⇒ la misma query cambia de path sin que cambien
los datos (`FIND-560`).

**Y el resto:** materializacion sin cap en el plan (~0,9 GB por `SELECT *` sobre 100k nodos,
`FIND-561`); la estimacion del governor subestima el pico ~9x (`FIND-562`); solo la ultima
condicion vectorial del `WHERE` sobrevive al plan (`FIND-563`); el filtro lexico descarta los
terminos sueltos cuando hay una frase (`FIND-564`); el plan IQL no puede expresar los filtros
del contrato del SDK (`FIND-565`); el guard de memoria del executor no mide RSS (`FIND-566`);
N+1 de I/O en los tombstones (`FIND-567`); y 4 bajos (`FIND-568..571`).

**Nota (Ponytail lite):** los 20 hallazgos se reducen a **tres reglas**: (1) un filtro que se
aplica **fuera** del plan ve un dataset ya proyectado (=> los filtros van dentro del plan, no
despues); (2) un plan **optimizado** que puede eliminar un predicado necesita que la estimacion
este acotada y use una sola poblacion; (3) una reserva de recursos debe liberarse en **cualquier**
salida, no solo en el camino feliz.


## FIND-555 y FIND-572: la reserva de memoria del governor (2026-10-21)

- **Fecha:** 2026-10-21
- **Resultado:** ✅ el critico `FIND-555` (la fuga que dejaba el motor inservible tras ~20
  queries fallidas) y el `FIND-572` (el underflow del mismo contador), ambos verificados con
  `cargo test --lib executor::` (52 verdes) y `governor::` (25 verdes).

- `FIND-555` (auditoria-executor/QP-04): la reserva de memoria del governor vivia en un contador **global** (`ALLOCATED_BYTES`) y solo se liberaba en el camino feliz: cualquiera de los cuatro `?` previos (`optimize_and_compile`, `physical_op.open`, `physical_op.next`, `physical_op.close`) retornaba **sin liberar**. Cada query fallida dejaba su reserva cobrada para siempre: con `AVG_NODE_BYTES = 1024` y 100k nodos, ~20 queries fallidas llevaban el contador por encima de 2 GiB y `check_memory_pressure` rechazaba **toda** query posterior, incluidas las de escritura. Con el proveedor de embeddings caido, 'provider caido' se convertia en 'motor muerto' tras ~20 intentos. Corregido con un guard RAII (`AllocationGuard`) que devuelve la reserva en **cualquier** salida; los dos `free_allocation` explicitos se eliminan. Tests: `test_allocation_guard_frees_on_every_exit` y el end-to-end `test_execute_plan_frees_admission_on_error` (que antes solo cubria el unico early-return que tenia el `free` a mano).

- **FIND-572** (nuevo, descubierto durante el fix): `free_allocation` hacia
  `ALLOCATED_BYTES.fetch_sub(bytes)` sin saturar: liberar mas de lo reservado hace que el
  `usize` **de la vuelta** y el contador pasa a ~1.8e19 ⇒ todo `request_allocation` posterior
  rechaza la query con `Error::ResourceLimit` (incluidas las escrituras) hasta reiniciar. Es
  un DoS permanente silencioso, y cualquier imbalance (doble free, free sin reserva, panic
  entre la reserva y el free) lo dispara. Corregido con resta saturante.

**Nota (Ponytail lite):** los dos fallos son la misma linea de codigo vista desde dos
angulos — una reserva global que se libera a mano en un solo camino. El guard RAII hace
imposible olvidarse (y de paso hace imposible el doble free), y la resta saturante hace que
un error de contabilidad no pueda matar el motor.


## W6 cerrada: el cifrado en reposo pasa a ser verdad (FIND-194/249/514/516)

- **Fecha:** 2026-10-21
- **Resultado:** ✅ W6 (activacion, registro unico, rotacion y documentacion honesta) implementada
  y verificada. Nota de la wave: `docs/dev/reviews/2026-10-21-w6-encryption-activation.md`.

- `FIND-194` (W6/rotacion): la rotacion **no** invalida envelopes: `vanta-cli encryption rotate --new-key-file` re-envuelve cada DEK con la KEK nueva (**no** re-deriva: las DEKs son las mismas) ⇒ ningun ciphertext deja de descifrar. Ademas el `.bak` del registro permite el rollback manual mientras la master key vieja siga existiendo, y la operacion es O(artefactos)=4, no O(datos).
- `FIND-249` (W6/activacion): la feature `encryption` ya esta **cableada de punta a punta**: el motor resuelve una master key y un modo **desde los bytes** del disco en el open, y reparte el contexto a los 4 artefactos (WAL v4, VantaFile v3, backend-KV e imagen HNSW v9). Una base creada con clave se cifra **desde el primer byte**; una base existente **nunca** se migra (plaintext + aviso, ADR D7). Verificacion: 90 tests de cifrado + los 2493 del lib en verde, `clippy -D warnings` limpio con y sin la feature, y el canario de 6 superficies (`tests/encryption_canary.rs`) en verde.
- `FIND-514` (W6/registro): `KeyRegistry` pasa de valor a **handle** sobre `Arc<Mutex<RegistryState>>` con una **tabla por proceso** (path → `Weak`): los 4 artefactos que abren el registro de la misma base obtienen el **mismo** estado ⇒ una sola instancia en memoria, un solo `persist`, **cero lost update** (el `persist` serializa el estado compartido actual, no la copia obsoleta de quien llama).
- `FIND-516` (W6/activacion): una base cifrada abierta sin clave devuelve `Error::EncryptionKeyRequired` **antes de abrir el backend** (el motor decide el modo en el open) ⇒ nunca se devuelve el ciphertext como si fuera plano. Y una base **plaintext** con clave configurada se queda en plaintext con **aviso** (la clave se ignora, ADR D7: no hay migracion in-place).

**Verificacion de W6:** 90 tests de cifrado/registro/rotacion + los **2493** del lib en verde
(`cargo nextest run -p vantadb --profile audit --lib`), `clippy --all-targets -- -D warnings`
limpio con y sin la feature `encryption`, `fmt --check` limpio, y el **canario de 6 superficies**
(`tests/encryption_canary.rs`) en verde.

**Siguen abiertos tras W6 (declarados por la propia wave):** `FIND-496` (el shipper de WAL no
lee segmentos v4 ⇒ la replicacion de bases cifradas no funciona), `FIND-498` (el WAL sigue
cifrando dentro del lock del shard, desviacion del ADR D5) y `FIND-507` (el backup/restore no
conoce el registro ⇒ restaurar una base cifrada en otra maquina no tiene camino).

**Lo que W7 tiene que medir (y el plan que hay que corregir):** el bench `canonical_p99` es
**in-memory** (`CPIndex` puro, sin I/O) ⇒ mediria lo mismo con y sin cifrado y **no** vale para
el coste at-rest. Hacen falta (a) el bench del WAL cifrado, (b) uno **nuevo** del motor con clave
configurada (seed 42, 100k x 1536d, el perfil de `benches/common/mod.rs`, 3 corridas y mediana),
(c) el **flush** (la imagen se reescribe entera: GCM sobre el indice completo) y (d) el **cold
start** (descifrar la imagen entera: se pierde el mmap zero-copy). Umbral: el p99 cifrado no debe
empeorar mas de un **15 %** sobre el baseline (`BENCHMARKS.md:380`: p50 2.2388 / p95 4.248 /
**p99 4.9987 ms**, 2026-09-25), y todos los numeros a `BENCHMARKS.md` con comando + entorno +
fecha (Regla 11).


## El claim del cifrado en reposo pasa a ser verdad (FIND-407, FIND-501)

- **Fecha:** 2026-10-21
- **Resultado:** ✅ las dos filas que exigian "corregir el doc **Y** cablear la feature" quedan
  resueltas con W6 (no solo el texto).

- `FIND-407` (W6/claim): resuelto por W6 **en las dos mitades** que exigia la decision del owner: (1) el doc —`SECURITY.md` esta enmendado hacia delante con los **limites declarados** (las claves del backend y los tokens del indice de texto siguen en claro, el registro guarda DEKs envueltas y nunca la master key, el tamano de los valores se filtra, la master key vive en la memoria del proceso, perder el registro pierde la base, no hay migracion in-place, wal-shipping saca registros en claro); (2) el **cableado** —la feature `encryption` ya enciende los 4 puntos de escritura (WAL v4, VantaFile v3, backend-KV e imagen HNSW v9) con la decision de modo tomada **desde los bytes** del disco. Evidencia: 90 tests de cifrado + los 2493 del lib en verde, `clippy -D warnings` limpio con y sin la feature, y el **canario de 6 superficies** (`tests/encryption_canary.rs`) que verifica que un payload conocido **no** aparece en claro en ningun artefacto (WAL, shards, rotados, imagen del indice, segmentos, journal, SSTs y snapshots).
- `FIND-501` (W6/activacion): el motor **ya decide el modo por base** y el WAL se abre y se recupera con la **misma** DEK que el resto de artefactos (`init_wal` → `ShardedWal::new_encrypted`, `recover_state` → `WalReader::open_with_key`): una base con clave tiene el WAL, los segmentos, el backend y la imagen **en el mismo modo** — ya no hay estados distintos. El canario comprueba que el WAL (`data/vanta.wal`, `data/vanta.shard{0..N}.wal` y los rotados `.<ts>`) no contiene el payload en claro.

**Lo que queda del claim:** solo la **medicion** (W7): el claim de rendimiento (que el cifrado no
empeore el p99 mas de un 15 %) todavia no esta medido, asi que el doc ya es honesto sobre lo que
el producto hace, pero el **coste** todavia no esta acuantado.


## Revision P2-01 del cifrado en reposo: veredicto y 9 hallazgos (FIND-585..593)

- **Fecha:** 2026-10-21
- **Resultado:** 🟡 **APTO CON RESERVAS** (el veredicto de la review con contexto fresco). La
  ingenieria y el diseno criptografico son defendibles; lo que no estaba bien es que la feature
  afirmaba en su propia superficie publica lo contrario de lo que hace, y que existia un camino
  (multi-base) en el que una base plaintext se cifraba a medias sin aviso.

**Lo que se corrigio en el mismo commit (H-1, H-2, M-1, M-3, M-6):**
- Las **5 claims inversas** reescritas (el rustdoc del crate, el de `Config`, el de
  `Config::with_encryption`, la limitacion de `HTTP_API.md` y la fila de `FEATURES.md`).
- El **gate de doc**, que leia 3 ficheros y buscaba 2 frases literales (pasaba en verde con 5
  documentos afirmaando lo contrario) ⇒ ahora barre `src/**` + `docs/**` con 8 patrones y una
  lista de excepciones explicita. Al reforzarlo cazo **2 claims mas** que la review no listo.
- La **viñeta del wal-shipping** (decia que el shipping egresa en claro; en realidad el ciclo
  **falla y no envia nada**) + los limites del backup.
- El **modo mixto silencioso** por crash entre el open del backend y `after_open` (las 4 DEK se
  rigen siempre que el modo sea cifrado).
- Los **permisos 0600** del registro (contiene el salt y un oraculo de clave) y el
  **`skip(new_key)`** del span de tracing (la master key se iba a stderr).
- El **guard de longitud** de `euclidean_distance_squared_f32` (un vector de otra dimension
  puntuaba 0.0 = el mejor score Euclidean posible).

**Lo que queda abierto (FIND-585..593, 9 filas):** el modo mixto multi-base (critica), el AAD
que ata el offset y no el fichero (=> un ciphertext reubicado entre ficheros hermanos autentica:
integridad rota sin clave), la rotacion que se pierde en silencio si la base esta abierta, la
master key en el `Debug` de `Config` y via argv, la decision de modo fail-open, Miri sin la
feature `encryption` + la alineacion de los `from_raw_parts` solo con `debug_assert`, el canario
vacio en 2 de sus 6 superficies, y 8 hallazgos P3 agrupados.

**El nonce NO es el hallazgo mas grave:** el esquema `epoch||counter` con epoch aleatorio
realeatorizado en cada `begin_epoch`, un solo allocator por artefacto y el epoch persistido
antes del primer uso hacen que **no haya reutilizacion de nonce**. El fallo es de *binding de
posicion* (el `slot` es intra-fichero), no de unicidad de nonce, y su impacto es de
**integridad**, no de confidencialidad.


## Review P2-01 de las 4 tasks de las criticas: veredicto y cierre (2026-10-21)

- **Resultado:** 🟡 **APTO CON RESERVAS** en conjunto. El reviewer (un agente **distinto**
  del implementador) verifico **mecanicamente** las 4 tasks: 2525/2525 del lib, clippy
  `-D warnings` 0, fmt limpio, 98 de integracion de IQL, 171/171 del MCP, 75/75 de vitest, 14/14
  del crate node, recall@10 = 1.0000.

**Cerradas por el veredicto:** `FIND-517` (su reserva era un fichero sin commitear: ya
commiteado), `FIND-518`, `FIND-519` y `FIND-552`.

**Las 3 correcciones que salio del review:**

- **R1 (FIND-553)** — el scope vectorial decia "replica `scan.rs:73-92`" pero no lo replicaba:
  el scan es un **OR** (tipo **o** namespace) y el scope comprobaba el tipo primero y
  **excluia sin mirar el namespace** => las dos ramas de la misma query discrepaban (la clase
  exacta que la task mata). Corregido: replica el scan exactamente, con su mismo fallback.
- **R2 (FIND-553)** — el CBO descartaba el predicado `type = <entidad>` sin distinguir **tipo**
  de **tabla de namespace** (MCP-29): en una tabla de namespace los registros no llevan `type`,
  asi que el predicado del usuario debe seguir aplicandose. Descartarlo era una **regresion**
  (devolvia mas filas de las pedidas). Corregido: el CBO **no descarta ningun predicado**.
- **R1 (FIND-518)** — la estanqueidad global del input no estaba documentada (es un cambio de
  contrato observable en HTTP/MCP/CLI). Documentada en `docs/api/IQL.md`.

**Lo que el review confirmo de la desviacion de FIND-518** (que era mia): `FROM Doc garbage` **no**
es trailing input — `garbage` es un `<alias>` documentado (`IQL.md:83,102`) y lo consume el
lexer como alias; el caso real es `FROM Doc p garbage`. Y `INSERT ... LINK ...` no es sintaxis
documentada (la arista documentada es `RELATE`).

**2 hallazgos nuevos:** `FIND-596` (el 0.5 de un campo no trackeado voltea la estrategia del SDK
de PreFilter a PostFilter, y el PostFilter puede devolver menos de `top_k`) y `FIND-597` (las
dos reservas de 64 MiB que el usuario no puede acotar: el journal de Fjall y el
`create_in_memory` del motor, que **si** sale en el `memory_limit`).

**Pendiente:** `FIND-559` **sigue abierta** (su fix pedia ademas cambiar el comportamiento del
nodo sin `_owner_role` —fail-closed vs pasar— y anadir su test: es una decision de producto) y
`FIND-553`/`554`/`560` quedan abiertas hasta que el test run confirme las correcciones R1/R2.


## Auditoria de los 10 modulos restantes del core: FIND-503 CERRADA (FIND-599..608, 36 hallazgos)

- **Fecha:** 2026-10-21
- **Resultado:** ✅ `FIND-503` (los 12 modulos del core sin auditar en profundidad) queda **cerrada**:
  2 los cubrio la auditoria de `physical_plan`/`planner`/`executor` y estos 10 la de este ciclo
  (graphrag, agentic, shred, skills.rs, tui, metrics, serialization, utils, hardware, backends,
  bin). 36 hallazgos: **8 altos, 18 medios, 10 bajos**; 0 criticos (no hay corrupcion de memoria
  explotable: lo peor es perdida silenciosa de mensajes/datos y un panic del TUI).

**Los 8 altos, por lo que el usuario los nota:**

- **`FIND-599`**: `send_message` **pierde mensajes** con dos escritores concurrentes sobre el
  mismo hilo (read-modify-write sin lock; el `insert_lock` del motor solo cubre la escritura
  individual).
- **`FIND-600`**: el indice de threads es **un unico blob JSON** con read-modify-write sin lock =>
  un hilo creado puede **desaparecer del indice para siempre** (invisible e inmortal).
- **`FIND-601`**: escribir un caracter **no-ASCII** en el REPL del TUI hace `panic` (el cursor
  cuenta bytes y `String::insert` exige limites de caracter).
- **`FIND-602`**: el TUI **deja el terminal en modo raw** si cualquier operacion falla => la shell
  del usuario se queda sin eco (el unico fallo que no se sabe recuperar).
- **`FIND-603`**: el Dashboard **materializa todos los nodos del motor en el hilo de la UI**, y lo
  repite 4 veces por segundo si la base esta vacia (la cache usa `0` como centinela).
- **`FIND-604`**: el fast path "shredded" devuelve **0 resultados** cuando el filtro es una lista,
  un DateTime o un Null (el store solo guarda escalares) => **el mismo filtro da filas distintas
  segun el camino**, y el camino depende de la selectividad *estimada*.
- **`FIND-605`**: el "lock optimista" de las skills **no es atomico** => dos escrituras
  concurrentes con el mismo `expected_version` se pierden en silencio, con un `Ok` falso (y el
  doc del modulo afirma justo lo contrario).
- **`FIND-606`**: el block cache de RocksDB se dimensiona como el **45 % de la RAM del host** sin
  cap, en paralelo a las otras dos reservas del motor (viola `memory-budget.md` R-3/R-4).

**De paso**: **`FIND-403`** (5 modulos sin cobertura de tests) queda **parcialmente invalidado**:
los cuatro citados (tui, utils, shred, skills) **si** tienen tests unitarios inline; lo que sigue
siendo cierto es que **ninguno tiene test de integracion** en `tests/` => la fila se reescribe
como "cobertura de integracion cero", no "cero tests".

**Nota (Ponytail lite):** la mitad de los altos son el mismo patron: un read-modify-write sin
lock en un `Embedded` que es `Clone + Send + Sync` y que sirve peticiones en paralelo (los hilos,
el indice de hilos y el lock de las skills). Un `Arc<Mutex<()>>` por area —como el `merge_lock` que
el propio SDK ya usa— los arregla a los tres.


## FIND-559: la clausula `ROLE` - fail-closed, pero sin silencio (2026-10-21)

- **Fecha:** 2026-10-21
- **Resultado:** OK la poda de `ROLE` **sigue siendo fail-closed** (la decision del owner del
  2026-10-21 fue "fail-closed + aviso"): lo que cambia es que **el conjunto de filas no cambia** y
  ahora el motor **dice por que** una consulta devuelve 0 filas.

- **El problema:** la clausula `ROLE` poda todo nodo sin el campo `_owner_role` (el prune del
  post-plan en `src/executor.rs`) y **nadie escribe ese campo** (`grep _owner_role` en `src/` no
  encuentra ningun escritor, solo un benchmark) -> `ROLE "user"` sobre una base cuyos registros
  nunca se etiquetaron devolvia **0 filas sin error ni aviso**: el patron de `FIND-413` (resultado
  vacio silencioso), sin ninguna pista de que la poda fue el motivo.

- **Lo que se ha anadido (observabilidad, no semantica):**
  1. **Un contador** de lo que la poda descarta (`role_scanned` / `role_pruned` / `role_untagged`),
     y
  2. **un aviso informativo por consulta** -**nunca** por nodo- cuando la poda se come el **100 %**
     de los candidatos: `"ROLE 'user' pruned 3 of 3 scanned records: none of them carries the
     '_owner_role' field. Tag the records with '_owner_role' in their metadata or drop the
     clause."`. Va por `tracing::warn!` (el idiom del repo para "avisado, nunca silencioso"), asi
     que **no** es un `Error` y **no** cambia el resultado: un plan que no lleva `ROLE` no paga
     nada (los contadores solo se tocan dentro de la rama del prune).
  3. **La distincion con el filtro de `AS OF`:** el filtro de valid-time corre **despues** del
     plan (`execute_query`/`execute_select` -> `filter_valid_at`), luego no puede disparar el
     aviso; y un plan que no produjo candidatos (escaneo vacio) tampoco avisa: solo avisa la poda
     que se comio todo lo que el plan le dio.
- **Documentacion:** `docs/api/IQL.md` - la fila de `ROLE` y una nota nueva usan la **variante A
  (fail-closed)** del brief de `docs/dev/reviews/2026-10-21-encryption-envelope-por-pagina.md` 6.4,
  **verbatim**: el campo `_owner_role` (de donde sale la etiqueta), el fail-closed ("returns **no
  rows** ... this is the intended behaviour, not an error") y el remedio (etiquetar o quitar la
  clausula).
- **Contrato (verificado):** 5 tests nuevos en `src/executor.rs` (RED #1 compile: la politica de
  aviso no existia; RED #2 comportamental: el aviso no existia) - `role_prune_keeps_the_fail_closed_result`,
  `role_prune_warns_once_when_it_prunes_every_scanned_record` (una vez por consulta, con los
  contadores), `role_prune_does_not_warn_when_it_prunes_only_part_of_the_records`,
  `role_prune_does_not_warn_without_a_role_clause`, `role_prune_notice_policy` - mas
  `cargo fmt --all -- --check` exit 0, `cargo clippy -p vantadb --all-targets -- -D warnings` exit 0
  y `cargo nextest run -p vantadb --profile audit --lib` -> 2532 passed / 2 skipped (1 timeout de
  carga en `index::core::tests::concurrent_insert_preserves_hnsw_invariants`, **ajeno**: pasa en
  aislamiento en 82,9 s). Sin commits (los hace el orquestador).
- **Nota (Ponytail lite):** el aviso va por el log y no por un campo nuevo en `ExecutionResult`:
  tocar la superficie publica habria roto el contrato que la task promete no romper. El "por que"
  del fail-closed cabe en **una linea de log** y en **tres contadores**.
- **Colateral (ya resuelto en otra task):** la mitad de proyeccion de `FIND-559` la cerro
  `FIND-552` (la allowlist de metadata del sistema incluye `_owner_role`, asi que con `FETCH` el
  prune **vuelve a filtrar** en vez de devolver 0 filas siempre). Esta task es la otra mitad: la
  semantica del nodo sin etiqueta y **el silencio**.
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
- **Resultado:** ✅ 24/24 tareas. Suite final 361/361 tests en vanta-memory; fmt/clippy `-D warnings` limpios. Registro completo: `docs/progreso/campanas/p27-memory-engine.md` (split GOV-D2). Plan: `2026-08-18-vanta-memory`.
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
- **Fuente:** Plan `2026-08-23-backlog-triage` (Wave 1) · Backlog · review-full-20260822 H09-CODE-001
- **Resultado:** ✅ `record_co_access` seteaba `saturated=true` al cruzar `max_pairs` pero `decay()` reducía la tabla sin resetearlo → en servers long-running el warmer quedaba en refresh-only para siempre. Fix: `decay()` levanta el latch cuando el post-decay total < `max_pairs` (≤1 transición por cruce; sin thrashing verificado con decays que mantienen post-total ≥ cap). TDD: RED reprodujo el bug, 2 tests nuevos (ciclo completo + no-thrash), módulo 11/11. Verify full: fmt ✅ · clippy `-D warnings` ✅ · nextest audit workspace 2714/2714 ✅. Review P2-01 (vanta-review) aprobó tras corregir doc deshonesta del campo `saturated`. Commit `8b8924b3`. (ver `.opencode/skills/campaign-executor/tasks/REVIEW-09.md`)

### REVIEW-14: Keys cortas en version_history devuelven error de corrupción sin panic — unwraps frágiles eliminados
- **Fecha:** 2026-08-23
- **Fuente:** Plan `2026-08-23-backlog-triage` (Wave 2) · Backlog · review-full-20260822 H09-CODE-005
- **Resultado:** ✅ DISCOVERY reclasificó el hallazgo: el slice frágil de `version_history.rs:283` era test-only (ningún path de producción parsee keys del partition Versions) y `VantaError::Corrupt` no existe (clase corrupción del codebase = `BackendError`; añadir variante rompería matches exhaustivos de bindings P2-6). Fix real: helper `version_from_key()` (`Result<u64>`, `BackendError` si key <8B — key corta solo puede venir del store, nunca de input de usuario) + validación en `versions()` como defensa ante store corrupto + test nuevo (len 0..7 → error tipado, boundary 8B OK). explain.rs: 3 unwraps estructurales eliminados bindeando `Some(query_sparse)` en los patterns (Keyword nullifica la Option, no solo un bool — semántica idéntica arm-por-arm). Siblings con mismo patrón (mod.rs/debug_ops.rs) derivados a FIND-27. Verify full: fmt ✅ · clippy `-p vantadb -D warnings` ✅ · nextest `-p vantadb` 2051/2051 ✅. Commit `4044a588`. (ver `.opencode/skills/campaign-executor/tasks/REVIEW-14.md`)

### MOD-01: WAL escrito ANTES de validar — insert/update rechazado resucita datos tras restart
- **Fecha:** 2026-08-23
- **Fuente:** Plan `2026-08-23-backlog-triage` (Wave 1) · Backlog · core.md H-1 · cross-modulos F-1 (expuesto vía WASM)
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
- **Plan:** `2026-08-25-batch-backup-restore-chain` (Task 1/3, cadena secuencial)
- **Resultado:** OK - (1) Quiesce: create_snapshot llama self.flush() antes del imageado en ambas variantes cfg (guard read_only; mismo patron que rebuild_vector_index/compact_layout_bfs: flush ANTES de tomar insert_lock, lock no reentrante) -> el conjunto imageado es mutuamente consistente en un punto unico en el tiempo. (2) Recorrido recursivo: helper mirror_data_dir() excluye el subdir snapshots/ (dentro de data_dir, evita snapshots/snapshots/) y mirror_file() unifica hard_link (unix) / copy (windows-wasm). (3) Layout real documentado: data_dir solo tiene archivos top-level (vstore_L0..L3.vanta, vector_index.bin, vanta.wal); el backend KV vive FUERA (escalado como FIND-33). Trade-off performance documentado en docstring (flush agrega costo, correctness>speed). Test de regresion test_snapshot_consistent_under_concurrent_writes (4 writers concurrentes + reopen + asserts anti-torn). Nota RED honesta: pre-fix pasaba en Windows via replay total del WAL (checkpoint_seq no se copia al snapshot) - el tear real era conjunto-inconsistente index/vstore en la variante copy. Verify: cargo nextest run -p vantadb snapshot --ignore-default-filter 6/6 PASS + 13 unit PASS + fmt PASS + clippy -D warnings PASS. Sin commit (lead verifica). Colateral: FIND-33 creado.

### FIND-37: Eliminar `query_sparse.unwrap()` panickable en dispatcher híbrido (2026-08-27)
- **Fecha:** 2026-08-27
- **Plan:** `2026-08-27-backlog-pipeline` Task 1 (Wave 0) · `vanta-worker`
- **Resultado:** ✅ Dispatcher híbrido (`VantaEmbedded::search_impl`) asumía `Some` en 6 sitios vía `query_sparse.as_ref().unwrap()` (líneas 207,240,265,315,346,369) — request sin sparse dispara panic en prod (hot path). Fix: reemplazar `has_sparse: bool` + unwrap por `query_sparse: Option<&SparseVector>` filtrada `!is_empty` + match `Some(qs)` (pattern de `explain.rs` REVIEW-14 / `hybrid.rs:36`). También 3 sites en `debug_ops.rs` (288,335,374) mismo patrón. `hybrid_search` ahora recibe `Option` filtrado. Semántica preservada: sparse None/empty → fallback silencioso a text-only/vector-only (no ValidationError). Verify: `rg query_sparse.*unwrap src/sdk/search/mod.rs` → 0 ✅ · `rg src/sdk/search/` → 0 ✅ · `cargo check -p vantadb` ✅ · `cargo nextest -E 'test(search)'` 157 passed ✅ · `cargo fmt --check` ✅ · pre-commit hook clippy ✅. Commit `bd7c2691` `fix(search): FIND-37 eliminate query_sparse.unwrap panics`. Sin deuda neta (saldo negativo: 9 panics eliminados).
- **Archivos:** `src/sdk/search/mod.rs`, `src/sdk/search/debug_ops.rs`
- **Contrato:** `cargo nextest run -p vantadb --profile audit -E 'test(search)'` ✅ + `rg -n "query_sparse.*unwrap" src/sdk/search/mod.rs` → 0 + `cargo clippy -p vantadb --all-targets --all-features -- -D warnings` sin nuevos warnings

### FIND-39: ScalarIndex.remove sin test — cobertura engine-level (2026-08-27)
- **Fecha:** 2026-08-27
- **Plan:** `2026-08-27-backlog-pipeline` Wave 1 #11 · `vanta-worker` — Origen codegraph-20260827-143245 Fase 10 (score 0.98, 0 hits en storage/engine/tests)
- **Resultado:** ✅ Añadido `test_scalar_remove` en `src/storage/engine/tests/scalar_index.rs` (93L, engine-level): direct `ScalarIndex` API (insert 2 reds → remove 1 → 1 left, no-op missing field/value/id, cross-field isolation, remove last) + wiring engine (insert alpha → overwrite beta → alpha cleared, delete → cleared). Registrado en `src/storage/engine/tests/mod.rs` (`mod scalar_index;`). Unit tests preexistentes en `src/scalar_index.rs` (test_scalar_index_remove etc.) ya cubrían happy path pero no el wiring engine; gap cerrado a nivel integración.
- **Archivos:** `src/storage/engine/tests/scalar_index.rs` (nuevo), `src/storage/engine/tests/mod.rs`
- **Contrato:** `cargo nextest run -p vantadb scalar_index --profile audit` → 15 passed (incl. test_scalar_remove) ✅ + `cargo nextest list --profile audit` 2069 → 2070 (+1) + `rg -n "pub fn remove" src/scalar_index.rs` → 2 ✅ + `cargo nextest run -p vantadb -E 'test(storage::engine)'` 306 passed ✅ + `cargo fmt --check` ✅ + `cargo clippy` ✅ + `validate-docs-coverage` 0 gaps ✅
- **Commit:** `59f5cdcb` `feat(test): FIND-39 add test_scalar_remove for ScalarIndex.remove`
- **Gates:** D: no-disparado (no feature-add, no symbols públicos nuevos) · V: no-disparado (verify pasó al 1er intento) · C: no-disparado (solo test, no API change)
- **Skills:** campaign-executor, progreso, ponytail, source-driven-development, test-driven-development, incremental-implementation, rust-write-tests, code-review-and-quality

### FIND-34: Ciclo WAL Writer — DAG documentado + edge tests recovery/quarantine (2026-08-27)
- **Fecha:** 2026-08-27
- **Plan:** `2026-08-27-backlog-v2` Task 1 (Wave 0) · `vanta-worker` — Origen codegraph-20260827-143245 Fase 1 (4 nodos `open↔open_with_buffer↔recover↔quarantine` reportado como ciclo High)
- **Resultado:** ✅ DAG verificado (grep 0 back-edge: `recover`/`quarantine` leaves, solo callers en `open_with_buffer`), falso positivo Leiden co-localización documentado. Doc header `src/wal.rs:178-193` con diagrama DAG + ponytail ceiling. Tests: `test_recover_mid_file_corruption_scan_forward_recovers_tail` (16B garbage mid + raw framing valid tail → scan-forward 3/3) + `test_quarantine_rotates_when_corrupt_exists` (`.corrupt` → `.corrupt.1` rotation sin overwrite). Cobertura: `test_wal_auto_healing_and_recovery` + `test_corrupt_wal_tail_is_quarantined` preexistentes + 2 nuevos.
- **Archivos:** `src/wal.rs` (doc DAG 15L + 2 tests ~80L; `recover_valid_records` def 545, `quarantine_corrupt_tail` def 592, 1 def c/u)
- **Contrato:** `cargo nextest run -p vantadb --profile audit -E 'test(wal)'` → 62 passed (60→62, 2009 skipped, 6.08s) ✅ + `rg -n "quarantine_corrupt_tail|recover_valid_records" src/wal.rs` → 1 def c/u con cobertura ✅ + `cargo check -p vantadb` ✅ + `cargo clippy -p vantadb --all-targets --all-features -- -D warnings` 0 ✅ + `cargo fmt --check` ✅ + `validate-docs-coverage` 0 gaps ✅ + doc `src/wal.rs:178-193` justifica `codegraph_explore "wal cycle FIND-34"` como DAG (no SCC)
- **Commit:** `fix: FIND-34 — WAL writer DAG justification + recovery/quarantine edge tests` (pendiente)
- **Gates:** D: no-disparado (no symbols públicos nuevos, blast radius 2 archivos) · V: no-disparado (verify wal 1º intento 62/62) · C: no-disparado (doc + tests, no API change)
- **Skills:** campaign-executor, progreso, ponytail, source-driven-development, systematic-debugging, incremental-implementation, test-driven-development, documentation-and-adrs
- **Notes:** Ponytail rung 1 — doc antes que refactor `open_inner`. Skipped: ADR separado, helper extraction. Add when: SCC real con back-edge. Quarantine fails soft (warn); scan-forward byte-by-byte O(n) solo en recovery, no hot path.

### FIND-35: Ciclo StorageEngine get/prefetch — SCC intencional 2 nodos justificado + PrefetchGuard (2026-08-27)
- **Fecha:** 2026-08-27
- **Plan:** `2026-08-27-backlog-v2` Task 2 (Wave 0) · `vanta-worker` — Origen codegraph-20260827-143245 Fase 1 (2 nodos `StorageEngine.get ↔ prefetch_related` reportado como ciclo)
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
- **Plan:** `2026-08-27-backlog-v2` Task 3 (Wave 1) · `vanta-arch` — Origen codegraph-20260827-143245 Fase 1 (3 ciclos get/put/delete `desktop/src-tauri/src/connections/native.rs ↔ src/backends/rocksdb_backend.rs`)
- **Resultado:** ✅ Falso positivo verificado + frontera documentada. DISCOVERY `rg` 0 `use`-imports cruzados (RocksDbBackend no importa `desktop`/`tauri`/`NativeConnection`; NativeConnection no importa `RocksDbBackend`/`StorageBackend`), DAG `NativeConnection (desktop, isolated workspace [members=["."]] — `vantadb = {path="../.."}`) → `VantaEmbedded` → `StorageEngine` → `StorageBackend (pub(crate) Send+Sync, src/backend.rs)` → `RocksDbBackend (pub(crate), src/backends/rocksdb_backend.rs:22)` — 0 back-edge. Los 3 "ciclos" son colisión de nombres get/put/delete + clustering Leiden de verbos CRUD, no SCC CALLS. Workspaces aislados (`cargo check -p vantadb` invariant, `cargo tree` sin `vantadb-desktop`). Frontera ya correcta: `StorageBackend` pub(crate) + `BackendKind::RocksDb/Fjall/InMemory` + `VantaConfig.storage_path` (desktop nunca conoce `BackendPartition`/cf names). Doc headers: `src/backends/rocksdb_backend.rs:1-22` + `desktop/src-tauri/src/connections/native.rs:1-20` con diagrama DAG + ponytail ceiling + verificación `cargo check 0 cycles` + cross-ref mutua.
- **Archivos:** `src/backends/rocksdb_backend.rs` (header `//!` 15L, DAG + falso positivo + ponytail), `desktop/src-tauri/src/connections/native.rs` (header `//!` 12L, DAG + ponytail), `2026-08-27-backlog-v2` Task 3 → ✅ COMPLETED + recitation FIND-36, `docs/dev/Backlog.md` FIND-36 eliminado
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

> Plan `2026-08-28-embeddings-local` · commits `2c185021`→`d24eeb1c` · `embeddings/` + feature `embed-local` (Regla 5: embeddings BYO-vector → local-first). Tabla modelos: 3 EN / 3 ES / 3 Combined (8 ≤3GB + Qwen3 16GB excepción).

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

## FIND-599/600/601/602: los P0 de la auditoria del core - mensajes que desaparecen y la shell rota (2026-10-21)

- **Fecha:** 2026-10-21
- **Resultado:** los 4 P0 de la auditoria de los 10 modulos restantes del core, cada uno con un
  test que lo fija (RED antes del fix). Sin API publica nueva, sin commits (los hace el
  orquestador).

- **FIND-599** (`send_message` pierde mensajes con dos escritores concurrentes): el
  read-modify-write de la lista de mensajes (`get -> load_messages -> push -> insert`) no tenia
  lock: el `insert_lock` del motor solo cubre la escritura individual, asi que dos `thread_send`
  simultaneos leian la misma lista y el segundo `insert` **borraba el primer mensaje** sin error.
  Fix: `thread_message_lock: Arc<Mutex<()>>` en `Embedded` (el mismo patron y el mismo motivo
  documentado que `merge_lock`/`supersede_lock`: *the engine's `insert_lock` only covers the
  individual write, not the SDK-level read + check*), pasado a cada `ThreadStore` via
  `ThreadStore::with_shared_locks` (`pub(crate)`). El lock **serializa, no rechaza** => ningun
  llamador ve errores nuevos. Ojo al detalle: el SDK construye un `ThreadStore` por llamada, asi
  que un lock *dentro* del store no serializaria nada - de ahi el lock en el handle.

- **FIND-600** (el indice de threads es un blob JSON con read-modify-write sin lock): dos
  `create_thread` concurrentes leian el mismo array y el segundo `save` **pisa** el id del primero:
  el nodo del hilo si queda en el storage, pero el indice no lo lista => el hilo desaparece de
  `list_threads` y el purge nunca lo alcanza (invisible e inmortal). Fix: `thread_index_lock`
  (mismo patron) alrededor del `load -> push -> save`; el rebuild de `purge_expired_threads` toma
  el mismo lock del load al save (los helpers internos no vuelven a tomarlo => sin deadlock).

- **FIND-601** (un caracter no-ASCII en el REPL hacia `panic`): `cursor_pos` contaba **bytes**, asi
  que tras un caracter multibyte el cursor caia en mitad del caracter y el siguiente
  `String::insert`/`remove` reventaba (`assertion failed: self.is_char_boundary(idx)`) - y, con
  FIND-602, dejaba la shell rota. Fix: el cursor es un **indice de caracter**; `ReplState::byte_index`
  traduce con `char_indices().nth(n)` y las ediciones usan `replace_range` sobre limites de
  caracter (Backspace borra el caracter entero, no un byte). El render usa el indice de caracter
  para la columna del cursor.

- **FIND-602** (el TUI deja el terminal en modo raw si algo falla): `ratatui::restore()` +
  `disable_raw_mode()` estaban **solo** en el camino feliz; el hook de panic de `ratatui::init()` no
  cubre el retorno por `?`, asi que cualquier error de E/S en el loop dejaba la shell del usuario
  sin eco y sin saltos de linea. Fix: un guard con `Drop` (`TerminalGuard`) que restaura el
  terminal; el event loop se extrajo a `tui_event_loop` y `run_tui` lo envuelve en
  `with_terminal_restore`, de modo que el restore corre en el camino feliz, en el error y en el
  panic. (Unico delta observable: `run_tui` ya no propaga un error de `disable_raw_mode()` del
  camino feliz - el restore es best-effort por diseno, tiene que poder correr desde `Drop`.)

- **Contrato (verificado):** 4 tests nuevos, todos en RED antes del fix:
  `agentic::thread::tests::concurrent_send_message_keeps_both_messages` (RED: `got 20 of 40`),
  `agentic::thread::tests::concurrent_create_thread_keeps_both_threads_in_the_index` (RED:
  `thread ... vanished from the index`),
  `tui::repl::tests::repl_handles_multibyte_characters_without_panicking` (RED: panic de
  `is_char_boundary`) y `tui::tests::tui_restores_the_terminal_when_the_loop_fails` (RED: *the
  terminal must be restored on the error path*). Los dos del TUI necesitan `--features tui` (el
  modulo `tui` esta tras esa feature). Gates: `cargo fmt --all -- --check` exit 0 en los 4
  archivos del diff, `cargo clippy -p vantadb --all-targets -- -D warnings` exit 0 (features del
  gate y `--features tui`), `cargo nextest run -p vantadb --profile audit --lib -j 2` -> 2533
  passed / 2 skipped, la misma suite con `--features tui` -> 2540 passed / 2 skipped, y
  `cargo nextest run -p vantadb --profile audit --test message_thread_test` -> 6 passed.
  Task: `docs/dev/tasks/FIND-599-602-p0.md` (review P2-01 pendiente de un agente distinto).
- **Nota (Ponytail lite):** un `Arc<Mutex<()>>` por area (mensajes / indice) y nada mas: no hay
  framework de locks, no hay striping, no hay timeout - los locks de `ThreadStore` serializan
  (esperan), nunca rechazan. El indice de threads sigue siendo un `Vec<u128>` sin cap reescrito
  entero en cada create/delete (O(N) por operacion): eso es la mejora a medio plazo que propone el
  backlog, no este fix.

---

## Campaña Pre-`v0.10.0`: Compatibilidad On-Disk, Durabilidad Física, Quiesce e IQL/PhysicalPlan (2026-11-18)

### FIND-499: Reconocer formatos actuales (plano y cifrado) de VantaFile, WAL e índice en `src/migration.rs`
- **Fecha:** 2026-11-18
- **Objetivo:** Centralizar la clasificación de versiones (`ArtifactSpec::classify`) sobre las rutas reales de `data/` (`vstore_L{0..3}.vanta` v2/v3, `vanta*.wal` v3/v4, `vector_index.bin` v8/v9) para que `check_integrity` y `plan_all` acepten bases cifradas actuales, nunca migren artefactos cifrados y reporten versiones futuras como `newer than this build`.
- **Resultado:** ✅ `VFILE_SPEC`, `WAL_SPEC` e `INDEX_SPEC` implementados en `src/migration.rs` con compilación limpia con y sin feature `encryption` (42/42 tests); auditorías `#16` y `#18` PASS.
- **Commit:** `1218b3d66`, `058ee4864`, `b846b7ea3`

### FIND-631: ADR-0060 (compatibilidad on-disk `v0.9.0` → `v0.10.0`) y suite `tests/compat_v090.rs` contra fixture real `v0.9.0`
- **Fecha:** 2026-11-18
- **Objetivo:** Redactar `ADR-0060-on-disk-compatibility-v090-v0100.md` y verificar con un fixture reproducible generado desde el tag `v0.9.0` (`tests/fixtures/v0.9.0/`, 66,945 bytes) que `v0.10.0` abre, lee, busca, escribe y hace rollback sobre bases `v0.9.0` en sitio sin migración destructiva.
- **Resultado:** ✅ `ADR-0060` publicado, script reproducible `dev-tools/scripts/gen-fixture-v090.ps1` y 9 tests en `tests/compat_v090.rs` verdes (con y sin feature `encryption`, y compatibles con `FIND-652` `jiff`); auditorías `#16` y `#18` PASS.
- **Commit:** `c7eec35ed`, `bbc1fcbac`, `275a78df6`, `3cbb2919d`, `6a9427ebc`, `53fcefb26`, `dc65408f7`, `058ee4864`

### FIND-632: Honrar barrera lógica `read_only` en `FjallBackend` y enmendar `ADR-0001`
- **Fecha:** 2026-11-18
- **Objetivo:** Implementar barrera de escritura `ensure_writable` en los 5 mutadores de `FjallBackend` (`put`, `delete`, `write_batch`, `flush`, `compact`), mapear errores de apertura sobre directorios sin permiso de escritura (`map_open_error`) y documentar la semántica de apertura Fjall en `ADR-0001`.
- **Resultado:** ✅ Guardas en `src/backends/fjall_backend.rs`, allowlist exacta `fjall_open_allowlist` en `read_only_files.rs`, 4 tests en `read_only.rs` (incluido fix de permisos recursivos en Linux `fc580af9b`) y enmienda `ADR-0001`; auditoría `#18` PASS.
- **Commit:** `a7131bcac`, `fca76380f`, `fc580af9b`

### FIND-246: Escritura atómica con `sync_all()` de `vector_index.bin` antes de avanzar `checkpoint_seq` (`DUR-01`)
- **Fecha:** 2026-11-18
- **Objetivo:** Persistir `vector_index.bin` mediante protocolo crash-safe (`write_atomic`: `.tmp` → `flush` → `sync_all` → `drop` → `rename` → `sync_parent_dir`) en las 3 rutas de guardado antes de avanzar `checkpoint_seq` en Fjall.
- **Resultado:** ✅ `src/utils/fs.rs::write_atomic` adoptado en `file.rs` y `port_impl.rs`, limpieza de `.tmp` huérfano en `init.rs` solo en modo writable y guarda `!data_dir.is_empty()` en `rebuild_vector_index`; auditoría `#20` PASS.
- **Commit:** `aa261e836`, `0b082c35f`

### FIND-594: Cobertura de regresión para truncamiento de 1 registro en `verify_shard_counts` (`src/wal_sharded.rs`)
- **Fecha:** 2026-11-18
- **Objetivo:** Verificar y fijar con tests unitarios y E2E (plano y cifrado) que `verify_shard_counts` detecta un hueco de 1 registro en un shard intermedio (`[2, 1, 2, 2]`) validando la monotonía del prefijo round-robin.
- **Resultado:** ✅ Premisa de código corregida (el guard de monotonía `shard_counts[i] > shard_counts[i-1]` ya existía desde `44bebc567`); añadidos tests de regresión unitarios (`src/wal_sharded.rs:1006-1039`) y E2E (`tests/storage/wal_resilience.rs:413-513`); auditoría `#20` PASS.
- **Commit:** `9e9bc446c`

### FIND-653: Serializar `vacuum()` (early-return y tombstones) y `fresh_hnsw()` bajo `insert_lock` con recarga intra-guard
- **Fecha:** 2026-11-18
- **Objetivo:** Adquirir `insert_lock` en las rutas de reemplazo de grafo HNSW (`vacuum()` early-return, `vacuum()` con tombstones y `fresh_hnsw()`) recargando `self.hnsw.load()` dentro del guard para cerrar carreras contra `create_snapshot` y escritores concurrentes.
- **Resultado:** ✅ Las 3 rutas en `src/storage/engine/maintenance.rs` toman `insert_lock` respetando la Regla 8 (orden global acíclico) y recargan el handle dentro del guard; test RED→GREEN en `snapshot_quiesce.rs:284-330`, review P2-01 delta APPROVE y auditoría `#20` PASS.
- **Commit:** `5dca4cc25`

### FIND-522: Endurecer parser, lexer y errores de IQL ante `NaN`/`Inf`, recursión profunda y rangos inválidos (absorbe FIND-523, FIND-524, FIND-525, FIND-526, FIND-528)
- **Fecha:** 2026-11-18
- **Objetivo:** Cerrar los 6 defectos del front-end IQL: (1) rechazar `NaN`/`Inf` y exigir `min ∈ [0.0, 1.0]` (`FIND-522`), (2) limitar recursión a `MAX_SUBQUERY_DEPTH = 8` (`FIND-523`), (3) fallar con `Error::Validation` en `FROM NODE#<no-numérico>` (`FIND-524`), (4) rechazar `SIGUE` con `min > max` u overflow (`FIND-525`), (5) comparar keywords con `eq_ignore_ascii_case` y bloquear `RESERVED_KEYWORDS` como alias (`FIND-526`), y (6) truncar el input crudo en `iql_parse_error_message` a 120 caracteres UTF-8 safe (`FIND-528`).
- **Resultado:** ✅ Los 6 sub-hallazgos implementados y verificados en `src/parser/lexer.rs`, `src/parser/grammar.rs`, `src/executor.rs` y `src/physical_plan/scan.rs` (202/202 parser lib tests); review P2-01 APPROVE y auditoría `#22` PASS.
- **Commit:** `35ced06fe`, `6ca7358da`, `7bdcebaa1`

### FIND-556: Corregir filtros con alias, coerción numérica `Int`/`Float` y subqueries escalares en Physical Plan (absorbe FIND-157, FIND-557, FIND-558)
- **Fecha:** 2026-11-18
- **Objetivo:** Resolver en `src/physical_plan/` (1) lookup exact-then-stripped para campos calificados con alias (`FIND-556`), (2) comparación exacta `cmp_int_float(i64, f64)` sin pérdida de precisión en `|n| > 2^53` (`FIND-157`), (3) evaluación a `Ok(None)` (`false` relacional) cuando una subquery escalar devuelve 0 filas (`FIND-557`), y (4) extracción del primer campo proyectado por la subquery en lugar del primero alfabético del `BTreeMap` (`FIND-558`).
- **Resultado:** ✅ Implementado en `src/physical_plan/{mod,filter,join,project}.rs` y `src/planner.rs` (19/19 tests joins/subqueries); review P2-01 r2 APPROVE y auditoría `#22` PASS (`FIND-657` aislado para referencias cualificadas del lado perdedor de un join).
- **Commit:** `7bdcebaa1`, `539fc7887`

## Migración de backlog — pase 2 campaña `consolidation-2026-11-07` (2026-11-19)

### FIND-251: `purge_expired` no limpia el índice sparse: postings/stats de registros expirados quedan hasta un rebuild completo
- **Fecha:** 2026-11-19
- **Objetivo:** Limpiar entradas de `BackendPartition::SparseIndex` (postings/stats de registros expirados) durante `purge_expired` sin esperar a un rebuild completo.
- **Resultado:** ✅
- **Commit:** `28b67b418`

### FIND-313: IQL: ventana de candidatos vectoriales hardcoded (5) — `LIMIT` no puede excederla en consultas vector-only
- **Fecha:** 2026-11-19
- **Objetivo:** Sustituir la ventana de candidatos vectoriales hardcoded (5) por `top_k` dinámico para que `LIMIT` pueda excederla en consultas vector-only.
- **Resultado:** ✅
- **Commit:** `96cc80a47`

### FIND-535: El camino de vector denso acepta un `f64` que desborda a `f32::INFINITY` (el sparse ya se corrigio para esto): el `inf` se persiste y envenena los scores
- **Fecha:** 2026-11-19
- **Objetivo:** Exigir finitud completa (`x.is_finite()`) en vectores en `src/sdk/api/memory.rs` para que un `f64` que desborda a `f32::INFINITY` no se persista ni envenene scores.
- **Resultado:** ✅
- **Commit:** `b6e79a127`, `292b27d25`

### FIND-563: Solo la ultima condicion vectorial del `WHERE` sobrevive al plan: `WHERE a ~ "x", min = 0.5 AND b ~ "y", min = 0.6` pierde la condicion sobre `a`
- **Fecha:** 2026-11-19
- **Objetivo:** Soportar múltiples predicados `~` por consulta, `top_k` dinámico en `PhysicalVectorSearch` y guarda `.first()` (absorbía FIND-313, FIND-564 y FIND-571).
- **Resultado:** ✅
- **Commit:** `96cc80a47`

### FIND-564: `text_contains_query` descarta los terminos sueltos cuando la query contiene alguna frase entrecomillada
- **Fecha:** 2026-11-19
- **Objetivo:** Conservar los términos sueltos de `text_contains_query` cuando la query contiene frases entrecomilladas en vez de descartarlos.
- **Resultado:** ✅
- **Commit:** `96cc80a47`

### FIND-571: `self.storage.vector_store[0]` sin guarda: panic latente si el segmento 0 no existe
- **Fecha:** 2026-11-19
- **Objetivo:** Guardar el acceso `self.storage.vector_store[0]` para eliminar el panic latente cuando el segmento 0 no existe.
- **Resultado:** ✅
- **Commit:** `96cc80a47`

### FIND-596: El 0.5 de un campo no trackeado voltea la estrategia del SDK de busqueda vectorial de PreFilter a PostFilter, y el PostFilter puede devolver menos de `top_k`
- **Fecha:** 2026-11-19
- **Objetivo:** Activar el fallback exacto cuando `PostFilter` devuelve `hits.len() < top_k` en `src/sdk/search/vector.rs`.
- **Resultado:** ✅
- **Commit:** `5bd50b752`

### FIND-604: El fast path "shredded" devuelve CERO resultados cuando el filtro es una lista, un DateTime o un Null: el mismo filtro da filas distintas segun el camino
- **Fecha:** 2026-11-19
- **Objetivo:** Alinear `ShreddedRowStore` ante campos no escalares (`ListString`/`DateTime`/`Null`) y blindar invariantes binarias (absorbía FIND-607 SH-02 y FIND-608 SH-05).
- **Resultado:** ✅
- **Commit:** `94855d647`

### FIND-640: `put_batch` no es atómico entre chunks: con más de `batch_size` (1000) entradas, un fallo a mitad deja chunks previos commiteados
- **Fecha:** 2026-11-19
- **Objetivo:** Hacer `put_batch` atómico entre chunks para que un fallo a mitad de lote (> `batch_size`) no deje chunks parciales escritos.
- **Resultado:** ✅
- **Commit:** `28b67b418`

### FIND-651: Snapshot no coherente con escritores concurrentes: `from_engine` crea un `purge_lock` por wrapper y `rebuild_index`, `create_thread` y `add_edge` escriben sin barrera
- **Fecha:** 2026-11-19
- **Objetivo:** Barrera de snapshot compartida por `StorageEngine`: `purge_lock` compartido entre wrappers `Embedded`, `insert_lock` sostenido durante el capture y `evict_cold_nodes_with_reason`.
- **Resultado:** ✅
- **Commit:** `5f57b7c34`, `606ec758d`

### FIND-655: IndexReader read-only facade + hnsw/index_coordinator `pub(crate)`, pero `vector_store_read` expone `vfile::File` con `pub file: Option<StdFile>`
- **Fecha:** 2026-11-19
- **Objetivo:** Restringir `vfile::File::file` y cerrar el canal de escritura que `vector_store_read` exponía vía `pub file: Option<StdFile>` sin `insert_lock` (fachada `IndexReader`).
- **Resultado:** ✅
- **Commit:** `ba5c3d863`

### FIND-656: Campos `pub read_only`/`pub config`/`pub data_dir` de `StorageEngine` son mutables externamente con `&mut StorageEngine` (bypass de `ensure_writable`)
- **Fecha:** 2026-11-19
- **Objetivo:** Encapsular los campos `pub read_only`/`pub config`/`pub data_dir` de `StorageEngine` (`pub(crate)` + getters) para impedir mutación externa que evitaba `ensure_writable`.
- **Resultado:** ✅
- **Commit:** `ba5c3d863`

### FIND-657: JOIN: referencias cualificadas al lado perdedor bajo colisión de nombres leen el valor izquierdo (el merge descarta el qualifier)
- **Fecha:** 2026-11-19
- **Objetivo:** Conservar las claves cualificadas del lado perdedor en el merge de joins para que las referencias `alias.col` no lean el valor izquierdo bajo colisión.
- **Resultado:** ✅
- **Commit:** `59bdb8b7a`

### FIND-661: Locks hermanos de `sdk_purge_lock` siguen por-`Embedded`: `supersede_lock`, `merge_lock`, `thread_message_lock`, `thread_index_lock`, `skill_write_lock`, `dedup_filter` se mintean por wrapper
- **Fecha:** 2026-11-19
- **Objetivo:** Compartir los locks hermanos de `sdk_purge_lock` (`supersede_lock`, `merge_lock`, `thread_message_lock`, `thread_index_lock`, `skill_write_lock`, `dedup_filter`) vía el engine en lugar de mintearlos por wrapper.
- **Resultado:** ✅
- **Commit:** `ba5c3d863`

### CRASH-01: Recovery Loop & Replay Exhaustivo de Segmentos WAL Rotados en `init.rs`
- **Fecha:** 2026-11-07
- **Objetivo:** Replay exhaustivo de segmentos WAL rotados en `init.rs` con atomicidad todo-o-nada por batch (alias FIND-248; SKIP verificado por la campaña).
- **Resultado:** ✅
- **Commit:** `7b78b4fe9`

### TXN-01: Atomicidad Física y Rollback de Crash en Transacciones
- **Fecha:** 2026-11-07
- **Objetivo:** Atomicidad física y rollback de crash en transacciones mediante pasada undo física en la extracción de coordinators del `StorageEngine`.
- **Resultado:** ✅
- **Commit:** `7a65d5ae4`

### TEC-02: Descomposición Modular de `StorageEngine` (`Q8 Opción A` — v0.10.0)
- **Fecha:** 2026-11-07
- **Objetivo:** Descomposición modular de `StorageEngine` (Q8 Opción A) extrayendo los coordinators a módulos dedicados.
- **Resultado:** ✅
- **Commit:** `7a65d5ae4`

### TEC-04: Alerta en Telemetría y Logs para Fallback Brute-Force
- **Fecha:** 2026-11-07
- **Objetivo:** Alerta en telemetría y logs para el fallback brute-force ANN (`brute_fallback`/`brute_fallback_ratio` instrumentados).
- **Resultado:** ✅
- **Commit:** `0a77dfb71`, `d7991b5e8`

### TEC-08: Corrección de Comentario Obsoleto en `vector.rs:180`
- **Fecha:** 2026-11-07
- **Objetivo:** Corrección del comentario obsoleto en `vector.rs:180`, integrada en la ola de cumplimiento Art. 50/12.
- **Resultado:** ✅
- **Commit:** `0a77dfb71`

### TEC-11: Auto-Tuning de `ef_search` por Selectividad y `exact_scan` (`Q11 Opción B+C`)
- **Fecha:** 2026-11-07
- **Objetivo:** Auto-tuning de `ef_search` por selectividad con `exact_scan` adaptativo (Q11 Opción B+C).
- **Resultado:** ✅
- **Commit:** `e61bc3f4a`

### TEC-14: Conteo Lógico por Namespace (`vanta count --namespace`) y Cuotas v0.10.0 (`Q10 Opción B→A`)
- **Fecha:** 2026-11-07
- **Objetivo:** Conteo lógico por namespace (`vanta count --namespace`) y error `QuotaExceeded` documentado (Q10).
- **Resultado:** ✅
- **Commit:** `0a77dfb71`, `d38220844`

### TEC-14b: Cuotas Duras de Disco por Namespace (`max_namespace_records` / `max_namespace_bytes`) (`Q10-A`)
- **Fecha:** 2026-11-07
- **Objetivo:** Cuotas duras de disco por namespace (`max_namespace_records` / `max_namespace_bytes`) con error `QuotaExceeded` dedicado.
- **Resultado:** ✅
- **Commit:** `d7991b5e8`

### TEC-16: Diversificación Formal MMR en Core Rust (`Q20 Opción B`)
- **Fecha:** 2026-11-07
- **Objetivo:** Diversificación formal MMR en el núcleo Rust (Q20 Opción B; alias MGR-16, SKIP verificado).
- **Resultado:** ✅
- **Commit:** `d7991b5e8`

### ACT-03: Group-Commit en Ingesta Síncrona por Defecto
- **Fecha:** 2026-11-07
- **Objetivo:** Group-commit activado por defecto en la ingesta síncrona (`group_commit` en el núcleo).
- **Resultado:** ✅
- **Commit:** `0488c7dc8`

### EGO-14: Vacuum HNSW: purga de tombstones + recableado sin reindex total
- **Fecha:** 2026-11-07
- **Objetivo:** Vacuum físico de HNSW: purga de tombstones y recableado sin reindex total (SKIP verificado por la campaña).
- **Resultado:** ✅
- **Commit:** `4eb5d5d99`

### EGO-16: Borrado físico garantizado (purga de tombstones en disco)
- **Fecha:** 2026-11-07
- **Objetivo:** Borrado físico garantizado: purga de tombstones en disco con `set_tombstone` y re-sellado de segmentos cifrados (SKIP verificado por la campaña).
- **Resultado:** ✅
- **Commit:** `4eb5d5d99`, `b7550140e`, `8e08939ff`

### PRD-02: Arquitectura Dual de Embeddings Locales Parametrizables (`D3`)
- **Fecha:** 2026-11-07
- **Objetivo:** Arquitectura dual de embeddings locales parametrizables (D3) cableada en el núcleo.
- **Resultado:** ✅
- **Commit:** `0488c7dc8`


### FIND-147: 5 warnings `unused_imports` en `src/sdk/search/debug_ops.rs` (perfil bench/release)
- **Fecha:** 2026-11-22
- **Objetivo:** Gatear los 5 `use` huérfanos del perfil release/bench en `debug_ops.rs`.
- **Resultado:** ✅ — verificado en HEAD 2026-11-22: imports gateados con `#[cfg(any(debug_assertions, test))]` (nota en código: "FIND-608 absorbe FIND-147").
- **Commit:** `975d3b648`

### FIND-300: Fallback canónico `__vanta_payload` en GraphRAG `extract_content`
- **Fecha:** 2026-11-22
- **Objetivo:** Usar `crate::sdk::FIELD_PAYLOAD` (`__vanta_payload`) en `src/graphrag/retrieve.rs:10-25` para no devolver contenido vacío cuando nodos SDK no usan claves legacy.
- **Resultado:** ✅ Verificado en `HEAD` y cubierto en `tests/graphrag_test.rs:497-515`.
- **Commit:** `HEAD` (migrado desde Backlog)

### Cierre Frente 1 Core/Executor/Planner/Search (`FIND-604` residual shred empty-meta, RO IQL, `JOIN`+vector, `search_all`, ANN parcial)
- **Fecha:** 2026-11-22
- **Objetivo:** (1) Purgar filas obsoletas en `ShreddedRowStore` (`put_one`/`put_batch_inner` con `.rev()` dedup) al sobrescribir metadatos con `{}`; (2) aplicar `StorageEngine::guard_write_allowed` al inicio de `execute_insert/update/delete/relate/insert_message` antes de mutaciones en memoria; (3) devolver `Error::Iql` explícito cuando `has_join && !vector_searches.is_empty()` en `planner.rs`; (4) validar `search_all` antes del early-return por namespaces vacíos en `multi.rs`; (5) contar solo registros con vector en la guarda de ventana ANN parcial (`vector.rs`).
- **Resultado:** ✅ Verificado con suite dirigida `193/193` tests en verde (`cargo nextest run -p vantadb --lib`).
- **Commit:** `5a45a1f8c`

### Migración consolidada campaña `2026-11-07` — Core, Crypto y TUI (`FIND-484`, `520`, `597`, `603`, `606`, `635..637`, `642`, `643`)
- **Fecha:** 2026-11-22
- **Objetivo:** Retirar de `docs/dev/Backlog.md` las filas ya entregadas en `HEAD` (`FIND-597`/`606` presupuesto RAM/RocksDB en `204e94217`; `FIND-603`/`484` TUI `Option<usize>` + notices en `975d3b648`; `FIND-635..637` purga tablas/rotate/enmienda ADR-0057 en `f6e429db3` + `292b27d25`; `FIND-642`/`643` ONNX RAM governor e ISA/SIGILL gate en `0e6c76d96`; `FIND-520` en `884363b5a`), manteniendo **`FIND-607`** como **`🟡 Parcial`** verificada.
- **Resultado:** ✅ Símbolos y contratos verificados en `HEAD` y registrados en `2026-11-07-consolidacion-tecnica-pre-v0100.md`.
- **Commit:** `204e94217`, `975d3b648`, `f6e429db3`, `292b27d25`, `0e6c76d96`, `884363b5a`

### Cierre INSERT sin vector silencioso (FIND-485)
- **Fecha:** 2026-11-24
- **Objetivo:** Que un `INSERT` con `text` y provider de embeddings caido, sintetico o ausente no degrade en silencio: el nodo se persiste sin vector y con el campo relacional `__vanta_embed_status` (`failed` / `synthetic` / `unavailable`), legible con `vanta-cli query ... --json`. Mismo tratamiento en `INSERT MESSAGE`; el `VECTOR` explicito queda intacto.
- **Resultado:** ✅ Verificado: contrato `--profile audit -p vantadb --features remote-inference -E 'test(auto_embed)'` 9/9; default features 2/2 (brazo `unavailable`); `remote-inference,embed-local` 1/1 (sintetico); regresion `test(serialization|export|import|message)` 324/324; clippy `-D warnings` limpio en 3 configs; fmt limpio. Review P2-01 adversarial (`vanta-review`, contexto distinto): 2 pasadas `changes-required` (cazo un critico real: el crate no compilaba sin `remote-inference`, E0425) y la 3a `approve`. Sin `VantaError` nuevo ni firmas cambiadas.
- **Commit:** `d0df55c2b`

### Política de vector sintético, marca de degradación y `degraded` en `ExecutionResult` (`FIND-710` + `FIND-711` + `FIND-712`, fusionadas)
- **Fecha:** 2026-11-24
- **Objetivo:** Cerrar los tres residuales de `FIND-485` sobre `__vanta_embed_status`: (a) `INSERT` descartaba el vector sintético mientras `INSERT MESSAGE` lo conservaba — dos criterios para el mismo evento; (b) `apply_explicit_vector` no limpiaba la marca, así que un `UPDATE … SET VECTOR` dejaba `failed` sobre un vector válido; (c) la degradación solo era visible en el JSON del nodo, no en el resultado de la escritura. Se fusionaron en una tarea porque las tres tocan `src/executor.rs` y en paralelo habrían colisionado.
- **Resultado:** ✅ Política **unificada** (misma decisión en ambas rutas, testeada con el mismo provider sintético: mismo `HAS_VECTOR` y mismo status) · `apply_explicit_vector` **limpia** la marca al aplicar vector explícito · `degraded: bool` **aditivo** en `ExecutionResult::Write` (con `#[serde(default)]`, wire-compatible), mapeado en Python y en `vantadb-mcp`, con snapshot de `public-api.txt` regenerado. Extra del review: `__vanta_embed_status` pasa a ser **propiedad del executor** — un `INSERT … { __vanta_embed_status: "failed" }` del usuario ya no puede fabricar `degraded: true`.
- **Verificación:** `auto_embed` 9/9 (remote-inference) · 3/3 nuevos · sintético uniforme 1/1 (remote-inference+embed-local) · `vantadb-mcp test_auto_embed` 5/5 · binario real `"degraded": true` 1/1 · serialización/proptest/query_result 59/59 · regresión executor/serialization/memory **470/470** · `public_api --run-ignored` regenerado · clippy limpio ×3 · `check --no-default-features --features fjall` ✅ · fmt ✅ · docs gates `all clear`.
- **Commit:** `4eb4e8a5d`
- **Hallazgo del review que vale registrar (H-03):** el contrato original `-E 'test(auto_embed)'` **no seleccionaba ningún test nuevo**, así que revertir la política dejaba el gate verde. Se sustituyó por 5 legs obligatorios. Tercera instancia del día de un contrato no falsable (`FIND-317`, `F0.2`).
- **Deuda diferida:** server HTTP (`src/server/state.rs` descarta el campo con `..`) · bindings JS · `COMPATIBILITY.md`/`UPGRADE.md` para el próximo release · `memory_put.degraded` deriva de su `fallback` local · `degraded: false` con embed real sin test (requiere sesión ONNX) · vectores sintéticos ya persistidos por `INSERT MESSAGE` no se re-indexan (un `UPDATE … SET VECTOR` los repara).

### Hook `pre-commit` reparado y cableado (`FIND-718`)
- **Fecha:** 2026-11-24
- **Objetivo:** El hook `pre-commit` estaba roto (`nodeenv … ValueError: invalid literal for int() with base 10: '0-alpha'`) y `git config core.hooksPath` estaba **vacío**, así que los hooks de `.githooks/` ni se invocaban. Los gates de commit (yaml + `verify_staged` multi-área) no corrían en este clon.
- **Resultado:** ✅ **Dos defectos independientes, dos arreglos**: (a) `nodeenv` 1.10.0 resolvía "latest" a una versión `alpha` y hacía `int("0-alpha")` — bug upstream ([nodeenv#453](https://github.com/ekalinin/nodeenv/issues/453), fix en **1.11.1**); el repo ahora pinnea `default_language_version: node: lts` en vez de "latest"; (b) `core.hooksPath` se cablea a `.githooks` (config **local**, no versionable: documentado en `troubleshooting.md` § git hooks para que un clon nuevo no quede sin hooks en silencio).
  - Verificado en los dos sentidos: commit de prueba en un worktree descartable → `gates passed`/`ALL 0 PASS`/`actionlint ok`; commit con gate violado → exit 1 con puntero de triage.
  - El exec bit importa: `git ls-tree -r HEAD .githooks` → `100755` en `pre-commit` y `pre-push` (en POSIX sin ese bit git ignora el hook **en silencio**).
- **Commits:** `fc08c4b35` (fix) · `783f3f8cf` (P2-01: versión upstream 1.11.1, ancla del TOC, exec bit) · `183335be0` (evidencia de `gen-index` verde)
- **Efecto colateral inmediato:** al empezar a correr, el hook **stashea los cambios unstaged** de ficheros trackeados; con varios agentes en el mismo árbol eso es una ventana de riesgo (fila `FIND-726`), y ya barrió un `exec bit` staged de otro commit.
- **Review P2-01:** 1 ronda, `changes-required` → los 4 requeridos atendidos y verificados. Sin segunda ronda: el veredicto sobre el follow-up no existe (declarado).

### `clippy --target wasm32` en verde: dos `drop_non_drop` y un `expect_used` que ninguna lane veía (`FIND-721`)
- **Fecha:** 2026-11-24
- **Objetivo:** `cargo clippy -p vantadb-wasm --target wasm32-unknown-unknown -- -D warnings` fallaba con **3 errores** (`src/wal.rs:1347`, `src/utils/fs.rs:67`, `src/storage/engine/insert.rs:1101`) y ninguna lane de CI ejercía ese target: el rojo solo aparecía si alguien lo corría a mano.
- **Resultado:** ✅ Verde en **ambos targets, sin un solo `#[allow]`**. El `drop(file)` de `write_redacted_segment` y `write_atomic_with_tmp` pasa a `#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]`: en `wasm32-unknown-unknown` `std::fs::File` es el stub sin descriptor y **sin `Drop`**, así que el `drop` era no-op; `wasip1/2` y `emscripten` conservan el `drop` (tienen `File` real). El `fsync` **no** se gateó y en host el handle sigue cerrándose antes del `rename` (Windows rechaza reemplazar un archivo abierto). En `insert.rs` el `try_lock().expect(...)` del rollback wasm32 pasa a `Option` + `debug_assert!` (mismo guard, sin panic).
- **Verificación:** clippy wasm32 exit 0 · `check` wasm32 exit 0 · `check -p vantadb` host exit 0 · clippy host exit 0 · `nextest 'test(redact) or test(write_atomic)'` 11/11 · `check-docs` all clear. Predicate validado con `rustc --print cfg`, no de memoria.
- **Commit:** `757191bb6`
- **Review P2-01:** 2 rondas (ronda 1 `cambios requeridos`: el doc de `batch_insert_with_opts` prometía el panic eliminado) → ronda 2 **approve** con el contrato re-ejecutado sobre HEAD.
- **Deuda:** la lane de CI que ejerce el target sigue pendiente (step YAML exacto en el task file §Deuda).

### Snapshot quiesce: el test medía scheduling, no producto (`FIND-724`)
- **Fecha:** 2026-11-25
- **Objetivo:** `Tests (macOS)` —**check requerido**— estaba rojo por `test_snapshot_quiesce_capture_waits_for_held_writer`. Había que decidir si era un defecto de producto en macOS o un test inválido.
- **Resultado:** ✅ **Veredicto (b): el producto está bien; el escenario del test era imposible.** Con el guard tomado, el capture devuelve `Timeout { operation: "acquire insert_lock in create_snapshot (FIND-651 quiesce)", duration_ms: 200 }`, `retriable=true`, en **1.0498578 s** = exactamente las 5 ventanas, **idéntico en Windows y macOS** → no es plataforma: `acquire_snapshot_quiesce` acota la paciencia a 5 ventanas **por diseño** (`mod.rs:1505-1537`). El test viejo retenía el guard un tiempo no acotado (10 000 `yield_now()`) y exigía paciencia **ilimitada**; su verde era suerte del scheduler. El segundo panic del CI (`:110:31`) era **cascada** (`release_rx.recv()` del writer al dropearse `release_tx`), no una segunda causa.
  - **Fix test-only, con mutante en las dos direcciones.** La *primera* reescritura pasaba el mutante (`acquire_snapshot_quiesce` → `try_lock` inmediato): el marker se escribe antes del acquire, así que el mutante podía completar tras el release. Se cerró con una segunda condición —con el guard tomado, el capture debe **sobrevivir una ventana completa** de paciencia— y se añadió `test_snapshot_quiesce_bounded_patience_surfaces_timeout` para fijar el bound.
- **Verificación:** `test(snapshot_quiesce)` **10/10 ×2** · área `test(storage) or test(snapshot)` **551/551** · clippy `-D warnings` 0 · `check-docs` all clear · mutante ROJO en la 2ª iteración.
- **Commits:** `3d55a5840` + `1e04a3dcf`
- **Riesgo residual declarado:** queda un hueco de *detección* si el capture se despreempta >1.5 s entre el marker y el acquire (falso-verde, no flake); cerrarlo exige que el producto señalice el intento. Y el bound de 5 ventanas no está contado como número: deuda anotada.

### Tests de schema/WAL/serialize: path único por ejecución (`FIND-735`)
- **Fecha:** 2026-11-25
- **Objetivo:** 13 sitios en 3 ficheros derivaban su path de `%TEMP%` con nombre **literal** (`vanta_schema_test`, `Path::new("/tmp/…")`) o **PID**, no de un directorio único por ejecución. El síntoma del ticket ("colisionan") era real pero **parcial**: los que ya usaban nonce no colisionaban, y aun así dejaban `<wal>.corrupt` de cuarentena suelto en `%TEMP%` (63 B por corrida), donde el recolector de `FIND-732` no llega porque **sólo barre directorios**.
- **Resultado:** ✅ Los 13 son caso **efímero** — ningún test lee su path después del `Drop`, verificado cuerpo por cuerpo (incluido el de cuarentena, que inspecciona el backup **dentro** del test). `tempfile::TempDir` borra el árbol entero, así que el hermano `.corrupt` cae dentro y desaparece con él.
  - **El review adversarial destapó el patrón vivo en otra forma:** `schema.rs:272` seguía con `Path::new("/tmp/nonexistent_schema_file_for_testing")`, que en Windows resuelve a `C:\tmp\x` — un directorio **compartido que existe en esta máquina**: el test pasaba por estado global ajeno. Un `rg` de `temp_dir()`/`env::temp_dir` no lo caza.
  - **Medición (diferencial, no auto-reporte):** 16 procesos × 4 rondas sobre `file_write_read_roundtrip` → binario previo **33/64 fallos** (`schema.rs:266`, `Option::unwrap()` on `None`); binario nuevo **0/64**, `1 passed` en 64/64. 4 tests de filesystem × 8 en paralelo × 2 rondas → 0/64.
  - **Fix:** 13 sitios a `TempDir` (sólo `#[cfg(test)]`), `TempDir` como primer local en todos, y `tmp.close().expect(…)` en el test que producía el residuo para que el no-leak sea **falsable** (`Drop` traga los errores de borrado). El `Err` descartado del reset entre fases pasa a `.expect(…)` para no acusar a `batch_append` de un fallo ajeno.
- **Verificación:** `rg -n 'std::env::temp_dir|"/tmp/'` sobre los 3 ficheros → **0** · `nextest -E 'test(schema) or test(wal) or test(serialize)'` **195/195** · fmt ✅ · clippy `--all-targets` ✅ · `temp-collector -DryRun` sin residuos · commit `c73519024` · review P2-01 (tier adversarial, 2 rondas) `changes-required` → **approve**.
- **Corrección del contrato del ticket:** el gate que la fila proponía (`rg 'temp_dir\(\)|%TEMP%'`) era **inválido**: `temp_dir\(\)` nunca casa `tempfile::tempdir()` (sin guion bajo) y `%TEMP%` casa la prosa de los comentarios. Gate válido y medido: `rg -n 'std::env::temp_dir|"/tmp/'` → 0.
- **Deuda derivada:** `FIND-742` (test tautológico) · `FIND-743` (el recolector es ciego a ficheros sueltos) · `FIND-744` (`wal_sharded.rs:999`, mismo patrón fuera de scope).

### Tests de WAL: aserción falsable y residuo de cuarentena (`FIND-742`, `FIND-744`)
- **Fecha:** 2026-11-26
- **`FIND-742`:** `test_wal_version_mismatch` **no podía fallar** — comparaba `expected_version` contra **la misma constante** con la que `WalHeader::deserialize` arma el error, y su fixture era un mismatch de **magic**, no de versión (con comentario que afirmaba "uses v0", falso). Ahora: test nuevo `test_wal_rejects_valid_magic_with_version_zero` (header `VWAL` + v0 → `Error::WALVersionMismatch`, el path que **no tenía cobertura**) y el existente renombrado a `test_wal_rejects_foreign_magic`, afirmando los **bytes del fixture** (`found_magic == b"NOT_"`, `found_version == 24385`). Falsabilidad probada por mutación: **M1** (fixture con versión válida) → `expected WALVersionMismatch for a VWAL-magic v0 header, got None`; **M2** (esperado 0→1) → `left: 0 right: 1`.
  - **Producción sí distingue magic de versión** (`wal.rs:256` magic → `:283` `validate_compat` → `:286` `format_version < 1`), así que la severidad **no cambia**: era **agujero de cobertura**, no bug del motor.
- **`FIND-744`:** `wal_sharded.rs:999` dejaba `.shard0.corrupt` (**54 B/corrida**, medido 3×) suelto en `%TEMP%`. Helper `test_wal_path()` → `(tempfile::TempDir, PathBuf)` en **32 call sites** (el guard es el **primer** local, así que cae último), sin nonce — el `TempDir` ya es único. `drop(sw)` + `tmp.close().expect(…)` en `test_salvage_truncate_restores_coherent_recovery`, que es el **único** que produce el residuo (reabre un tail truncado → cuarentena).
- **Verificación:** `rg 'expected_version, WAL_MAX_READABLE_VERSION' src/wal.rs` → **0** · `rg 'std::env::temp_dir|"/tmp/' src/wal_sharded.rs` → **0** · `nextest -E 'test(wal)'` **105/105** · `-E 'test(sharded)'` **39/39 ×3** · `%TEMP%\vanta_test_sharded*` tras el filtro ×3 → **0** (antes +1/corrida) · `--features encryption -E 'test(test_wal_rejects)'` **2/2** (no depende de `WAL_MAX_READABLE_VERSION` 3↔4) · clippy `-D warnings` y fmt 0 · `temp-collector -DryRun` 0/0 · review P2-01 **APPROVE** (4 Low/Nit aplicados).
- **Commits:** `f8a732258` (`FIND-742`) · `564242c0f` (`FIND-744`)
- **Deuda derivada:** el barrido de clase encontró **8 sitios más** con `%TEMP%` en tests (`src/cli_server_auth_tests.rs:402`, `src/server/cli_server_auth_tests.rs:416`, `tests/wal_rollback.rs:27`, `src/node/vector_data.rs:447`, `src/embedding_health.rs:385`, `src/config.rs:3249`, `src/sdk/version_history.rs:724,745`) → `FIND-750`.

### Tests: la clase de paths de `%TEMP%` queda cerrada, no la cuarta instancia (`FIND-750`)
- **Fecha:** 2026-11-26
- **Objetivo:** `FIND-735` cerró 13 sitios y `FIND-744` uno más, pero el barrido de clase volvía a encontrar la misma construcción. El contrato no era "arreglar 8 sitios": era dejar el `rg` completo sobre `src/`+`tests/` sin una sola coincidencia sin justificar, porque cada fix por instancia garantiza la siguiente reincidencia.
- **Medición:** el barrido `rg -n 'std::env::temp_dir|"/tmp/|env::temp_dir' src/ tests/` devolvía **24 coincidencias en 15 ficheros** al empezar (el ticket nombraba 8) y devuelve **18 en 10 ficheros** al cerrar. De las 24, **6 escribían un path**: 4 tests, **2 doctests** y **un duplicado muerto**.
  - **Las 16 que quedan son lecturas o literales, no residuo:** `config.rs:2592-2593` (valor de `Config` + su aserción) · `config.rs:3249` (rama `is_absolute()` de `resolve_local_model_path`, corta antes de tocar el FS) · `embedding_health.rs:385` (etiqueta interpolada en un mensaje con `model_present=false`; no sondea el FS y el nombre debe **no** existir) · `migration.rs:768,769,774` (`MigrationEngine::new` solo guarda el `PathBuf`) · `sdk/types/record.rs:832,1097` · `sdk/types/search.rs:307,504` · `sdk/serialization/conversions.rs:164` · `tests/sdk_serialization.rs:596,629` · `storage/engine/tests/types.rs:116,125` (literales de campos de structs de datos, cero I/O).
  - **Las 2 de `src/hardware/mod.rs:833,834`** las escribió la sesión de `FIND-738` **durante** esta tarea (`OsString` pasados a `utils::env_value`, cero I/O). Fichero prohibido para `FIND-750`: justificadas, no tocadas. El barrido es un blanco móvil con carriles R en vuelo.
- **Duplicado muerto (19 tests de auth que nunca compilaban):** `src/cli_server_auth_tests.rs` (596 L) era una copia huérfana de `src/server/cli_server_auth_tests.rs` (709 L). Prueba: el único consumidor de ese nombre es `src/server/routing.rs:58` (`#[cfg(test)] #[path = "cli_server_auth_tests.rs"] mod auth_tests;`) y un `#[path]` relativo resuelve contra `src/server/`; **ningún** `mod`/`#[path]`/`include!` del repo declara el de la raíz (verificado con `rg` sobre `*.rs`/`*.toml` y reconstruyendo el target lib tras el borrado). Es **subconjunto estricto**: 19 tests vs 22 (los 3 SRV-06 de JWT solo existen en el vivo), **cero tests únicos**; las 20 líneas que no coinciden textualmente son comentarios, imports y la propia línea de `%TEMP%`. Se borra, no se "arregla".
- **Fix:** `TempDir` en los 4 tests (guard como primer local → cae último; se retiró la maquinaria `SEQ`/`rand` de `wal_rollback` y los `remove_file`/`remove_dir_all` best-effort) · `TempDir` en **línea oculta** (`# `) en los 2 doctests, con `db.close()` antes de soltar el guard (`namespace_stats` borraba el dir con la DB viva) · `drop(db)` + `tmp.close().expect(...)` en `version_history` para que el no-residuo sea falsable sin flakear en Windows.
- **Verificación:** barrido **24 → 18** (detalle por línea arriba) · `-E 'test(wal_v2) or test(mmap_full) or test(versions_backend) or test(f697_)'` **8/8** · `-E` literal del contrato **109/109** · `--features server -E 'test(auth_tests) + test(events_recorded)'` **22/22** · **2 corridas concurrentes** del filtro default (16/16 ×2) y del de auth (22/22 ×2) con **residuo 0** · `cargo test --doc ... namespace_stats` **1/1** y `'Embedded::open'` **3/3** · `%TEMP%` tras todo (`vanta-auth-test-*`, `vantadb-versions-test-*`, `vanta_review15_mmap_*`, `vantadb_test_wal_rollback_*`, `vantadb-open-example-*`, `vantadb-ns-stats-doctest-*`, `.tmp*`) = **0** · `temp-collector -DryRun` **0/0** · `cargo fmt --check` 0 · `cargo clippy -p vantadb --all-targets --features server -- -D warnings` 0 · OCR 0 · review P2-01 **APPROVE** en 1 ronda (cazó el Medium de los doctests).
- **Commits:** `d05302cec` (código) · `docs(avance,backlog): close FIND-750` (cierre de fila). El `D src/cli_server_auth_tests.rs` viaja en `d05302cec` (verificado con `git show --name-status`).
- **Deuda derivada:** el `-E` del contrato no seleccionaba **ninguno** de los cuatro tests que toca (nextest matchea nombre de test, no fichero) y los de auth solo existen con `--features server`; además `cargo nextest` **no corre doctests**, que es donde estaban 2 de las 6 escrituras. El gate de esta clase es el `rg`, no el runner.

### Los 43 `unwrap` de `shred` son de test: el invariante en producción es 0 (`FIND-152`)
- **Fecha:** 2026-11-26
- **Task file:** [FIND-152](../../tasks/FIND-152.md)
- **Objetivo:** `FIND-152` (origen: baseline semgrep/ast-grep de `FIND-149`, 2026-09-24) listaba `shred` como superficie con `unwrap` "en contexto prod" y pedía disposición fila por fila. La instrucción razonable era *"un `unwrap` en `shred` es peor que en otro sitio porque destruye datos"*; la respuesta es que **no hay ninguno que destruya nada**: los 43 están en el módulo de tests.
- **Medición:** `rg -n '\.unwrap\(\)' src/shred/mod.rs` → **43**, todos con línea ≥ `488`. La frontera es el `#[cfg(test)] mod tests` de `:488`, que en `:489` declara `#[allow(missing_docs, clippy::unwrap_used, clippy::expect_used)]`. Región de producción (`:1-487`): **0 `unwrap`, 0 `expect`, 0 `panic!`, 0 `unreachable!`, 0 `assert!`**.
- **Clasificación (a)/(b)/(c):** **(a)** 43 — test-only, allow explícito, no se convierten (un `unwrap` en un test es el idioma correcto). **(b)** 0 — ninguna frontera de error real quedó sin propagar: el decoder ya devuelve `Result` y usa `take`/`take_array` con `corrupt_row` (`SH-02`: corrupción es error, nunca mapa parcial), y `delete`/`delete_op` propagan. **(c)** 0 — nada exige cambiar firmas.
- **Por qué el decodificador no tiene `unwrap`:** `take()` (`:379`) acota con `checked_add` + `get(range)` y devuelve `corrupt_row`; `take_array` (`:392`) documenta explícitamente *"no `try_into`, no `unwrap`: the slice `take` returns is exactly `N` bytes"*. El trabajo de convertir ya estaba hecho — por eso el recuento de `unwrap` bajó a 0 sin que nadie lo anotara.
- **Verificación:** `cargo nextest run --profile audit -p vantadb -E 'test(shred)' --build-jobs 2` → **35/35 ✅** · `rg` antes/después **43 / 43** (no se tocó código: la fila se cierra por **medición**, no por edición) · `rg 'panic!|unreachable!|expect\('` sobre `:1-487` → **0**.
- **Deuda derivada (el hallazgo real de esta tarea, y es lo contrario de lo que el ticket buscaba):** el único `let _ =` del camino de borrado de shred está en `src/storage/engine/delete.rs:54` — `let _ = crate::shred::ShreddedRowStore::delete(id, &*self.backend)`. Un fallo del purgado no deja rastro **en el momento del fallo** (ni `tracing`, ni contador, ni métrica); la certificación de purga (`src/attestation.rs:49-56`, superficie `shred`) puede volver a sondearlo, pero eso es *a posteriori* y sólo si alguien corre la certificación. El comentario del sitio ("*must never fail the delete*", WAL como punto de commit) es correcto y se conserva, pero *no fallar* ≠ *no reportar*. Registrado como **`FIND-758`**; no se arregla acá porque `src/storage/**` es propiedad de Arch/Engine.
- **Los otros dos ítems de la fila (los cazó el review P2-01, no yo):** el texto de `FIND-152` cargaba además `14 unchecked-ops` y `67 unsafe-review pointers`, y borrar la fila los dejaba sin dueño.
  - **`*_unchecked` → cerrados como clase (a):** `rg -c '_unchecked' src/**/*.rs` → **16**, de los cuales **15 productivos** (`src/index/distance/kernels.rs` **12** + `src/index/distance/mapper.rs` **3**; el 16.º es test). Los dos ficheros llevan **14** `// SAFETY:` y el invariante es uno solo, repetido: `chunks_exact(N)` garantiza `chunk.len() == N`. No se convierten: el `get` seguro es justo lo que el kernel existe para evitar, y no hay alternativa de coste equivalente en un kernel SIMD. El ticket decía 14 y hoy hay 15: el delta es una instancia nueva, no un conteo stale.
  - **`unsafe` → `FIND-759`:** `rg -n 'unsafe\s*(\{|fn |impl )' src/ -g '!**/tests/**' -g '!**/*tests.rs'` → **70** sitios; `rg -c '// SAFETY:'` mismo scope → **54**. Los 16 de diferencia **no** son "16 sin justificar" (un comentario puede cubrir dos bloques contiguos; varios `unsafe fn` heredan el invariante del caller), y por eso el contrato que se hereda es el **veredicto por sitio**, no el número. Un recuento no prueba nada; emitirlo es trabajo de `vanta-audit`.
- **Techo declarado:** la frontera producción/test se determinó por límite de módulo (`#[cfg(test)]`), no por análisis de alcanzabilidad. Es suficiente porque en Rust un `#[cfg(test)]` no compila en el artefacto publicado — el invariante "0 panics en lo que se envía" queda probado, no inferido.
- **Commit:** `f9ec61f9e` (código en `vantadb-wasm/src/worker.rs` + primera versión de este cierre) · commit de cierre encima con la disposición de los `unchecked`/`unsafe` y el dictamen del review.

### El purgado de shred reporta el fallo que se tragaba (`FIND-758`)
- **Fecha:** 2026-11-26
- **Task file:** [FIND-758](../../tasks/FIND-758.md)
- **Objetivo:** el `let _ =` sobre `crate::shred::ShreddedRowStore::delete` (`src/storage/engine/delete.rs:54`) descartaba el `Result` de un borrado real. El razonamiento del sitio -el tombstone del WAL es el punto de commit, así que un fallo del purgado no debe hacer fallar el delete- **es correcto y se conserva**; lo que faltaba era la otra mitad: *no fallar* no es *no reportar*. Sin rastro en el momento del fallo, la fila `shred::{node_id}` sobrevive al nodo y sólo la certificación de purga (`src/attestation.rs`, superficie `shred`, una de las 9) podía verlo, *a posteriori* y sólo si alguien la corría.
- **Medición:** el call site era **único**: `rg 'ShreddedRowStore::delete\b' src/` → **4 hits**, 3 test-only (`src/shred/mod.rs:597,605,676`) y **1 productivo** (el `let _ =`). Los vecinos **sí** reportaban: `delete_batch` (`:283`) y `purge_permanent` (`:322`) meten `ShreddedRowStore::delete_op(id)` en un `write_batch(...)?` atómico, así que su fallo propaga (y por eso **fallan** el delete, al revés que el path individual, que devuelve `Ok` a propósito: el batch atómico no aplica nada y no hay split brain); el path de `delete()` individual era el único que silenciaba. Tras el fix: `rg 'let _ = crate::shred'` → **0** y `delete_reporting` es el único purgado del path individual.
- **Fix:** `ShreddedRowStore::delete_reporting(node_id, backend)` (`src/shred/mod.rs:501`) purga, y si falla incrementa `PURGE_FAILURES` (`static AtomicU64`, proceso) y emite `tracing::warn!` con `node_id`, el error y `purge_failures_total` acumulado - el operador ve el número **sin** correr la certificación y sin plomería nueva. `delete()` sigue devolviendo el `Result` del backend del nodo: **cero cambio de semántica**, el fallo del purgado no puede tumbar el borrado (prohibido el `?`).
- **Falsabilidad (mutantes, en tres direcciones):**
  - **M1 - el bug restaurado** (`let _ = crate::shred::ShreddedRowStore::delete(...)`): el test falla con `the shredded-row purge failure was not counted exactly once (FIND-758)` (delta exacto `before + 1`). ROJO, no verde por suerte.
  - **M2 - purgado eliminado**: el test falla antes, en `assert_eq!(shred_delete_attempts, 1)`. El backend de test cuenta los intentos de borrado **sólo** en `BackendPartition::InternalMetadata` (la partición de la fila), así que un test que sólo afirmara "delete devolvió Ok" no podría pasar en falso (pre-mortem c).
  - **M3 - contador que cuenta *intentos* y no *fallos*** (mover el `fetch_add` fuera del `if let Err`): lo levantó el review P2-01 con la primera versión del test (`count > before`) y **pasaba verde**. Cerrado con dos aserciones: delta exacto tras un fallo y "un purgado sano no mueve el contador" (bloque hermético, sin round-trip por el engine). Re-verificado: el mutante falla ahora con `a successful shred purge moved the failure counter`.
- **Review P2-01:** `vanta-review` (sesión fresca) → **approve**, 0C/0H; el Medium del contador (M3) lo levantó y cerró dentro de la misma review; aplicados N1-N4 y L2; `L1` (nada afirma el *nivel* `WARN`) y `L3` (el delete transaccional no purga) quedan como deuda, `L3` con fila propuesta `FIND-779`.

- **Verificación:** `-E 'test(shred_purge_failure_is_reported) or test(shred) or test(delete)'` **111/111** ✅ · `-E 'test(storage)'` **513/513** ✅ (1 SLOW pre-existente: `put_batch_multi_chunk_atomic_commit_visible`, 79,7 s) · `cargo clippy -p vantadb --all-targets -- -D warnings` **0** ✅ · `cargo fmt --check` **0** ✅ · `check-links`/`check-docs` ✅ · OCR ✅.
- **Nota de entorno:** el primer rebuild tras el mutante falló el link con `LNK2019: ... 428 externos sin resolver` (artefactos incrementales corruptos por presión de disco, ~10 GB libres). `dev-tools/target-cleanup.ps1 -AutoSafe` liberó 2,06 GB y `CARGO_INCREMENTAL=0` relinkeó limpio. No es del cambio.
- **Techo declarado (`ponytail:`):** el contador es de **proceso**, no por-handle: dos `StorageEngine` en el mismo proceso comparten cuenta. Suficiente porque el canal de operador es el `warn!` (que ya lleva el total); un contador por-handle exigiría un campo nuevo en `StorageEngine` + su hilo por `init.rs`/`db.rs`/tests, fuera del presupuesto de 1 h. Señal de reapertura: que alguien necesite el conteo **por almacén** para decidir una reparación.
- **No tocado, a propósito:** los otros `let _ =` de `src/storage/engine/` (`init.rs` ×11, `txn.rs` ×7, `insert.rs` ×4) son **compensación dentro de rollback/recovery**, donde el fallo no tiene a quién reportarse porque ya se está deshaciendo - otra clase, otro ticket si se quiere.

### `add_edge` deja de perder aristas: el read-modify-write queda serializado (`FIND-773`)
- **Fecha:** 2026-11-27
- **Task file:** [FIND-773](../../tasks/FIND-773.md)
- **Objetivo:** `Embedded::add_edge` (`src/sdk/api/graph.rs`) hacía `engine.get` → `push` → `engine.insert` sobre la lista de aristas **entera** del nodo, dos veces (forward en el origen, reverse en el destino), sin lock ni compare-and-swap. El `insert_lock` del engine sólo serializa la escritura individual: dos escritores leen la misma lista y el último `insert` gana. La arista del perdedor desaparece sin error y sin log, indistinguible de una que nunca se creó — el modo de fallo que `FIND-747`/`ADR-0066` acaban de hacer visible (hasta entonces no existía ninguna arista de linaje, así que el lost-update estaba dormido).
- **Medición (el bug, primero):** `8` hilos × `100` aristas distintas sobre un mismo nodo ⇒ **164 / 800** aristas, **636 perdidas** (`lost 636 of 800 edges on the shared node`, `tests/graph_edge_concurrency.rs`). Reproducible en todas las corridas: `164`, `156`, `156`. No hizo falta `yield` ni barrera interna — con 800 operaciones el ventana `get`→`insert` se cruza siempre; la `Barrier` sólo alinea el arranque.
- **Fix:** `let _write_guard = self.purge_lock.write();` al entrar en `add_edge`, cubriendo las **dos** mitades. Es el idioma que el fichero ya tiene en su módulo hermano (`insert_one_internal:1152`, `put_batch_inner:1542`, `delete_inner:1908`, `put_record_exact:2187`): la misma `purge_lock` (`Arc<RwLock<()>>` propiedad del engine, compartida por todo `Embedded` sobre él, `FIND-651`). Ningún patrón nuevo, ninguna dependencia nueva, ninguna firma nueva.
- **Por qué no lock por nodo:** una arista toca **dos** nodos; un lock por nodo exige orden total o deadlock entre `A→B` y `B→A`. Por qué no CAS: el engine no expone compare-and-swap (`engine.insert` es incondicional) y añadirlo es `src/storage/**`, fuera de este carril.
- **Por qué la serialización es de handle y no de wrapper:** el test `add_edge_serializes_across_handles_over_one_engine` abre `8` `Embedded::from_engine` sobre el mismo engine (la forma que usan PyO3/WASM/MCP) y exige `800/800`; sin el `Arc` compartido fallaría, así que la propiedad queda probada por ejecución, no por lectura.
- **Falsabilidad:** el mismo test sobre `HEAD` (sin la línea) ⇒ **156/800**, rojo; con la línea ⇒ **800/800**, verde. 3 corridas consecutivas verdes (no es un test que pase por suerte).
- **Techo declarado (`ponytail:`, en el propio código):** lock de escritura **por handle**, no por nodo. Serializa las escrituras de aristas contra los demás read-modify-write del SDK en ese handle. Medido: `4 000` `add_edge` con `8` escritores sobre un mismo nodo ⇒ **2 232 aristas/s** agregadas frente a **2 632/s** de un solo hilo, es decir, `8` escritores sobre el **mismo** nodo se comportan como uno (el peor caso para un lock de handle, y es el caso que el linaje produce). Sin el lock la cifra agregada es mayor (`4 080`–`17 417/s`, spread enorme en build debug sobre máquina compartida) **porque descarta ~80 % de las aristas**: no es throughput, es pérdida. Coste no contendido indistinguible del ruido (`365`–`1 110 µs/op` en `HEAD` contra `380`–`388 µs/op` con el guard: el spread intra-configuración es mayor que la diferencia). Vía de escape si algún día importa: lock por **par ordenado** `(min(src,dst), max(src,dst))`.
- **Hueco declarado, no cerrado:** `remove_edge` y `ensure_edge` tienen el mismo read-modify-write pero se llaman **desde dentro** del guard (`src/sdk/api/memory.rs:1259,1262,1697,1700`, más `:2243-2258`), así que no pueden tomarlo otra vez: un `RwLock` no reentrante daría deadlock. Cerrarlos en esos ocho sitios exige el split locked/unlocked que el módulo ya usa (`resolve_existing_for_write{,_locked}`), fuera de la lista de escritura de esta tarea (limitada a `graph.rs` + tests). **Dos** llamadores no están bajo el guard y sí podrían tomarlo directamente: `supersede` (`memory.rs:2515-2529`, con sólo `supersede_lock`) y `bulk_import_stream` (`memory.rs:3259`, cuyo último guard es `:2798`) — corregido en review P2-01, la primera versión decía "el único" y era falso.
- **Falsabilidad del test, en las dos direcciones:** (a) sin el guard, `164/800` aristas y rojo; (b) con el guard pero borrando la mitad reverse (`engine.insert(&target)` → `let _ = &target; Ok(())`), el test falla con `leaf nodes whose reverse half is missing or duplicated` sobre los **800** leaves — la primera versión de esa aserción sólo comprobaba `node_exists`, que era verdad incluso sin arista (levantado por el review como M1, cerrado con mutante medido).
- **Población del filtro del contrato** (`FIND-794`): `cargo nextest list -E 'test(graph) or test(edge) or test(lineage) or test(add_edge)'` ⇒ **117 tests** (>0 y >10, no hizo falta ampliar); la corrida ⇒ **117/117 ✅**.
- **Verificación:** `-E 'test(graph) or test(edge) or test(lineage) or test(add_edge)'` **117/117** ✅ · `cargo fmt --check` ✅ · `cargo clippy --workspace --all-targets --all-features -- -D warnings` limpio (5m47s) ✅ · `tests/graph_edge_concurrency.rs` 3× ✅ · review P2-01 `vanta-review` (contexto fresco) 🔴→ aplicados M1/M2/M3/L1/L2.

### El exit 253 del snapshot public-api era el `rustc` hijo, no el test (`FIND-191` residual (a))
- **Fecha:** 2026-11-27
- **Task file:** [FIND-191](../../tasks/FIND-191.md)
- **Objetivo:** el residuo `(a)` decía *"`stack overflow` (exit 253) en la regeneración cold-cache: el archivo sale bien y el 2º run pasa, pero el exit code de la 1ª vez envenena el gate"*. El `(b)` ya estaba cerrado en `2510c16e` y no se reabrió.
- **Medición (lo que cerró el caso):** `0xC00000FD` = 3221225725 y `3221225725 % 256 = 253` — un gate que lee el byte bajo de un proceso que abortó reporta 253. Estado frío de verdad = `CARGO_TARGET_DIR` a un directorio vacío (0,20 GB: `cargo rustdoc` pide sólo *metadata* de las dependencias) + `RUST_MIN_STACK=2097152` → **`thread 'rustc' has overflowed its stack`**, `process didn't exit successfully: ...rustc.exe --crate-name typenum ... (exit code: 0xc00000fd, STATUS_STACK_OVERFLOW)`, aborto a los **8,2 s** y el test paniqueando en `public_api.rs:134`. Con el fix, el mismo estímulo (frío + 2 MiB) → **1/1 PASS en 53,7 s**.
- **Causa (iii), no (i) ni (ii):** el proceso que aborta es el **compilador hijo**, y aborta en `typenum` — una *dependencia* — antes de que exista el JSON de `vantadb`. Descartadas con evidencia: **(i)** la recursión del parser es frío/caliente-idéntica (mismo objeto de 10 199 915 B, mapa plano de `Id`; fallaría en los dos runs y no falla); **(ii)** el hilo del test sobrevive con **512 KiB** y sólo revienta a **256 KiB**, o sea ≥4× de margen con el default de 2 MiB.
- **Por qué era cold-only:** el único paso que compila es el primero. En caliente `cargo` no compila (`Finished target in 0.64s`), el hijo ni se ejecuta y el run pasa. "El archivo sale bien" porque el golden **nunca se tocó**: el fallo ocurre antes de generar. No era benigno, era invisible.
- **Fix (`tests/api/public_api.rs`, +115/−1):** `CHILD_STACK_BYTES = 32 MiB` (el valor que ya usa `dev-tools/verify.ps1:63`) aplicado con `.env("RUST_MIN_STACK", child_stack_budget(ambient))` **sobre el `Command` del hijo** — no con `std::env::set_var`, que además es `unsafe` desde Rust 2024 y ensuciaría el hilo del test. `child_stack_budget` conserva un valor del operador **si es mayor** y aplica el piso si no. `const _: () = assert!(>= 8 MiB)` convierte bajar el piso en **error de compilación**, y `mod child_stack` aporta 4 tests unitarios.
- **Por qué no es "subir el stack":** la recursión está en el `rustc` nightly de terceros; acotarla exigiría `src/**` (fuera de alcance) y no está en nuestro código. No se sube el stack global (el hilo del test sigue con el ambiente). Lo que cambia no es el tamaño sino la **dependencia del ambiente**: antes el gate daba verde o 253 según qué exportara el shell del llamador (`verify.ps1` 32 MiB, `verify_changed.ps1` 16 MiB, un shell limpio nada, y `docs/user/troubleshooting.md:53` recomienda 4 MiB); ahora es contrato explícito del test. Techo declarado (`ponytail:`): el piso es empírico — si un nightly futuro necesita más, la señal es el job dedicado fallando, y el arreglo real es upstream.
- **Regresión:** la propiedad *"cold-cache no produce 253"* **no se automatiza** (necesita un `target/` vacío, estado de disco); declarada en el header del test con el comando exacto. Lo automatizable se automatizó: el presupuesto tiene 4 tests + rail de compilación.
- **Verificación:** población declarada antes del run — `--profile audit -E 'test(public_api)'` → **0** (ignored, no cuenta como verde: `FIND-794`), `+ --run-ignored ignored-only` → **1**; unit `--test public_api` **8/8**; el run real **1/1 PASS**; `fmt` + `clippy -p vantadb --test public_api -D warnings` ✅.
- **No tocado, a propósito:** `.config/nextest.toml` (carril de `FIND-219`; el perfil `audit` no hizo falta: el job dedicado ya corre sin él) y `.github/workflows/**` (mismo carril).
- **Commit:** `7aab87d47` (código + task file).

### El purge de threads ya distinguía `Err` de `Ok(None)`: la fila citaba código que `NOT-02` había borrado (`FIND-799`)
- **Fecha:** 2026-11-27
- **Task file:** [FIND-799](../../tasks/FIND-799.md)
- **Objetivo:** la fila decía que `purge_expired_threads` descarta ids cuando la lectura falla, porque `get(...).ok().flatten()` colapsa `Err` y `Ok(None)`; el id se borraría del índice con el nodo aún en disco y el hilo quedaría invisible y no purgable. Citaba `src/agentic/thread.rs:247-255`.
- **Medición (la premisa, primero):** esas líneas son hoy el bloque de TTL de `create`. El `ok().flatten()` del purge lo eliminó **`NOT-02`** (`647f4f27c`, 2026-11-03) — **24 días antes** de que la fila se escribiera — y el propio código lo declara (`thread.rs:362`: *"NOT-02: propagate real storage errors instead of silencing them with ok().flatten()"*). `git log -S 'ok().flatten()' -- src/agentic/thread.rs` sólo devuelve ese commit (más las restauraciones masivas del repo); `rg 'ok\(\)\.flatten\(\)' src/agentic/thread.rs` devuelve **una** coincidencia, y es el comentario.
- **Demostración del invariante:** la función tiene dos sitios que traducen `get` a pertenencia al índice y **ambos** propagan con `?` (`thread.rs:353` loop 1, `thread.rs:367` loop 2). El índice sólo se reescribe si **todo** `get` devolvió `Ok`; un `Err` sale antes de `save_thread_ids`. El `Err` es alcanzable de verdad (`fetch_backend_metadata` → `deserialize_node_payload(...)?`, `get.rs:240-249`); el dato corrupto **no** lo es, porque `materialize_uncached` lo mapea a `Ok(None)` (`get.rs:330-347`).
- **Por qué no se agregó el test que la fila pide:** no puede discriminar. Para que el loop 2 corra, el loop 1 tuvo que leer **todos** los ids sin error; con almacenamiento determinista el loop 2 obtiene lo mismo, así que un fallo determinista aborta el loop 1 primero. Un test de `Err` permanente + "el id sigue" pasa **igual** con el código viejo y con el nuevo — verde por la razón equivocada (clase `FIND-794`, pre-mortem (d) de la tarea). El bug pre-`NOT-02` sólo era alcanzable con un fallo **transitorio** (la primera lectura pasa, la segunda no). El motivo de que hoy no haya test no es el tipo concreto (corregido en review: el crate ya expone failpoints, `fail = "0.5"` en `Cargo.toml:141` + `cfg_failpoint`/`remove_failpoint` en `src/lib.rs:221-235`): es que **no hay failpoint en el camino de lectura**, y añadirlo es `src/storage/engine/get.rs`, fuera de alcance. Ninguna costura se paga (YAGNI: no hay bug que guardar).
- **Riesgo residual declarado:** la refutación cierra el camino del **`Err`**, no el síntoma entero. `materialize_uncached` mapea header/vector corrupto o **torn write** a `Ok(None)` (`get.rs:329-348`, `:257`), y en el rebuild ese `Ok(None)` borra el id del índice con la fila del backend viva — mismo "invisible y no purgable", otra causa. El contrato de la fila daba ese caso por bueno ("en `Ok(None)` se purga como hoy"), así que es **fila aparte**: *el rebuild no puede borrar un id sin prueba positiva de ausencia*.
- **`gc.rs` ya distingue:** `GcWorker::sweep` separa `Err(Error::NodeNotFound)` (quita del índice) de cualquier otro `Err` (loguea y reintenta, `gc.rs:61-67`). La misma clase, ya correcta.
- **Verificación:** población del filtro medida primero (`FIND-794`) — `cargo nextest list --profile audit -p vantadb --build-jobs 2 -E 'test(purge) or test(thread) or test(agentic) or test(ttl)'` → **40 tests** (>0) — y la corrida con el mismo filtro → **40/40** ✅. `cargo fmt --check` ✅ (sin cambio de Rust). `check-links` / `check-docs` ✅. Sin fila nueva en el Backlog: la de `FIND-799` se retira por premisa refutada.
- **No tocado, a propósito:** `purged > 0` es la única puerta del rebuild, así que un id cuyo nodo ya no existe (`Ok(None)`) sólo sale del índice si **otro** hilo venció en la misma pasada — ids fantasma permanentes (coste: una lectura por `list`; el nodo no existe, así que no es pérdida de datos). Fuera del alcance de la fila. Los `Ver:` de `FIND-798`/`FIND-800` siguen citando `FIND-799` (el alcance de escritura del Backlog era "solo mi fila"). La fila padre `FIND-607` en `docs/dev/plans/backlog-release-v0.10.0.md:62` sigue sellada como "re-verificado 2026-11-22" con el claim falso: **`docs/dev/plans/**` está prohibido en esta tarea**, queda declarado.
- **Review P2-01:** `vanta-review` (contexto fresco) → **approve** del análisis, con la premisa re-ejecutada de forma independiente (40 tests, 5 binarios). Requeridos M1 (registrar el gate) y M2 (commit anti-clobber + verificación post-commit) aplicados; M3 no aplicable (path prohibido); O1 (`Ok(None)`) incorporado como riesgo residual; O2 (failpoints) corregido.
- **Commit:** la entrada y el retiro de la fila se hornearon en commits de **otros** carriles sobre este árbol compartido (clase `FIND-753`/`FIND-792`: el índice compartido arrastra ediciones ajenas) — entrada: `1e9cc6429`, fila: `3c97b7608`. Esta tarea cierra con su task file. **Sin cambio de producto.**

### Cuatro de las nueve hijas de `FIND-607` citaban código borrado: la re-verificación del 2026-11-22 era falsa
- **Fecha:** 2026-11-27
- **Task file:** [FIND-607-verificacion](../../tasks/FIND-607-verificacion.md)
- **Objetivo:** las ocho hijas partidas el 2026-11-27 heredaban el *"re-verificado 2026-11-22: abiertos UT-04, TU-04, AG-03..05, SH-03..04, UT-01, UT-03"* de `FIND-607`, y `FIND-799` ya había demostrado que esa re-verificación citaba código eliminado 24 días antes. Medición pura: cada fila se contrastó abriendo el `archivo:línea` que cita, sin compilar y sin tocar `src/**`.
- **Refutadas (3, retiradas del Backlog):** **SH-03**/**SH-04** describían el fast path shreddeado de `bitset_from_filters` (`src/sdk/search/lexical.rs:191-199` en el audit): un `get` por nodo y `break` al primer nodo sin fila. **`928df0970` (2026-10-31)** borró el camino entero — hoy `bitset_from_filters` (`lexical.rs:176-190`) proyecta namespace y filtros con hashes en memoria, sin tocar el backend, y en `src/shred/mod.rs` no queda consumidor (`filter_verdict`/`has_omitted_fields` llevan `#[allow(dead_code)] // consumer lands with the read wiring`, `:231,248`; `put` es `#[cfg(test)]`). **UT-01**: el veto ya es estado propio — `slashed: HashSet` (`confidence_metrics.rs:28-29`) que `record_collision` respeta (`:46-63`) y sólo `rehabilitate_origin` levanta (`:108-117`), desde **`5fa148005` (2026-11-03)**. Las tres filas nacieron 27/24 días **después** de su propio arreglo.
- **Reescritas (2):** **UT-03** conserva sólo lo vivo — el primitivo atómico `check_and_mark` existe desde `0488c7dc8` y producción lo usa (`memory.rs:993`); lo que queda es el docstring que enseña el par TOCTOU (`duplicate_prevention.rs:28-34`), la ausencia de forma atómica para `u128` y dos esquemas de hash disjuntos. La fila decía "locks distintos" y era falso: es el **mismo** lock en dos adquisiciones. **UT-04** perdió el abort y también la pérdida de datos: `new` acota con `.min(MAX_BLOOM_BITS)` (`:15,47-57`, `0488c7dc8`) y su test ya asserta la cota; el consumidor real (`memory.rs:993`) re-verifica exacto ante un falso positivo e inserta igual sin coincidencia (`:997-1011`, `:1053-1057`), así que la saturación cuesta lecturas, no registros. Baja a severidad **Baja**: lo único que queda es un contador agregado que falta.
- **Review P2-01:** `vanta-review` en contexto fresco devolvió `changes-required` en la ronda 1 — no por una retirada injustificada (verificó las tres línea por línea), sino porque esta misma entrada y la fila de `FIND-805` afirmaban un descarte que el consumidor no hace. Aplicado: consecuencia corregida, severidad Alta→Baja, conteo de cierres corregido y sección `## Review` en el task file.
- **Vivas (3, intactas):** `FIND-798` (`thread.rs:255-256`, `insert` antes de `add_thread_id_locked`, sin compensación — el lock de `:225` serializa, no revierte), `FIND-800` (`builder.rs:371,379` pasan `gc = None`; `thread.rs:297-306` desliza el TTL) y `FIND-806` (`refresh()` no-op en `monitor.rs:33-37`; `queries` no recibe un solo `push` en el repo).
- **Patrón medido:** el audit es del 2026-10-21 y las filas se escribieron el 2026-11-27 sin releer el código entre medio; los **tres** commits que cierran las **cuatro** refutadas (`647f4f27c`, `928df0970`, `5fa148005`) son del 2026-10-31 / 2026-11-03. Igual que `FIND-799`. Regla: una fila no se re-verifica leyendo la fila, y toda refutación necesita el commit (`git log -S`) para distinguir "nunca fue cierta" de "envejeció bien".
- **Verificación:** lectura de `HEAD` + `git log --all -S` por símbolo (`ShreddedRowStore::get` en `lexical.rs`, `slashed: HashSet`, `MAX_BLOOM_BITS`, `check_and_mark`) → commits citados arriba; `git show 928df0970 -- src/sdk/search/lexical.rs` muestra el borrado del `break`. Sin compilación (contrato de medición pura).
- **No tocado, a propósito:** `src/**`, `tests/**`, `vanta-memory/**`, `vantadb-mcp/**`, `.github/**`, `dev-tools/**`, `.opencode/**` — dos carriles (`FIND-797`, `FIND-799`) escriben ahí.

### Las 18 filas `🟡 Parcial` del Backlog dicen ahora la verdad: 4 seguían vivas, 6 citaban líneas desplazadas y una citaba un archivo que nunca tuvo el código
- **Fecha:** 2026-11-27
- **Task file:** [PARCIALES-consolidacion](../../tasks/PARCIALES-consolidacion.md)
- **Objetivo:** las 18 filas `🟡 Parcial` del Backlog llevaban meses con la misma marca y ninguna decía qué estaba hecho ni con qué línea. Dos barridos de medición pura ([A](../../tasks/PARCIALES-verificacion-A.md) en `12e6d7c`, [B](../../tasks/PARCIALES-verificacion-B.md) en `7aaa6c239`) dieron veredicto con evidencia; esta tarea lo volcó al Backlog.
- **Contable:** **18 = 4 VIVA** (`DEAD-01`, `FIND-198`, `FIND-320`, `FIND-598`) **+ 11 PARCIAL-REAL + 2 CAMBIADA** (`FIND-122`, `DIST-18`) **+ 1 NO-VERIFICABLE** (`NOT-06`). `FIND-215` es PARCIAL-REAL reducida a medición + default porque el "falta doc" estaba refutado por `5deca8c78`. **0 REFUTADAS**: ninguna fila se retiró.
- **Estados nuevos:** `FIND-122` → `🟡 Pendiente (contrato corregido)` (el storage híbrido y `SearchProfileMode` ya existen; falta el router automático); `DIST-18` → `⏸️ Bloqueada: versión 1.0.0` (`publish = false`, 404 en crates.io: no es trabajo a medias); `NOT-06` → `🟡 Pendiente — requiere al owner`. Las otras 15 conservan `🟡 Parcial`/`🟡 Pendiente` con lo hecho y lo que queda enumerado.
- **Citas corregidas (las que más costaban):** `FIND-120` `sources.rs:82` → `:39`; `FIND-122` `index/search/layer.rs` → `planner.rs` + `search_profile.rs` (ese archivo **jamás** contuvo fusión: `git log -S bm25` vacío); `EGO-15` `seed.rs:25` → `:31` (muerta **dos días antes** de la "re-verificación" del 2026-11-22); `FIND-158(c)` deja de apuntar a `desktop/` y apunta al artefacto publicado (`vantadb-node/index.d.ts:148`); `WIRE-19` `scheduler.rs:82,85,94` → `:63,66,75` y la degradación P4 a `vantadb-server/src/scheduler.rs:75-81` + `vantadb-mcp/memory_scheduler.rs:98-103` (envejeció en horas: `WIRE-20` cerró el mismo día); `FIND-598` el test del reparto es `vfile.rs:2223-2250`, no `:2085-2119` (ese es el helper `legacy_read`).
- **Tres afirmaciones que las filas exageraban:** `EGO-15` ("ninguna superficie pública expone el setter" — existe en `pipeline.rs:96,101`), `FIND-121` ("verificar que no exista cache texto→vector" — `EmbedCache` existe desde `FIND-706` y **rechaza el TTL**), `FIND-320` (`MGR-06`/`MGR-18` no son "piezas que existen": siguen `Pendiente`).
- **Contradicción interna declarada, sin reescribir:** `FIND-598` dice a la vez que falta la re-medición post-Paso 0 y que esa re-medición ya se hizo (+43,2 %). Se corrigieron estado y veredicto; la celda de 7,6 KB (números certificados) queda como está y la contradicción vive en el task file.
- **Puntero re-ejecutable:** cada una de las 18 lleva `Verificación: docs/dev/tasks/PARCIALES-verificacion-<A|B>.md`. Es la lección de `FIND-811`: una fila sin evidencia re-ejecutable vuelve a mentir. Se retiró además el marcador `— marcado por sweep sin evidencia, requiere verificación` de las 18.
- **10 decisiones del owner** quedan propuestas y sin tomar (DNS de `vantadb.dev`, Vercel/pasarelas, precio `$750`, criterio de aceptación de `NOT-06`, `DIST-18`, allocator en `default`, runner BEAM, ADR de `retrieve()`, breaking de `VectorStoreRef`, enmienda del ADR D4): la tabla con "qué desbloquea" está en el task file.
- **Verificación:** `git diff --numstat -- docs/dev/Backlog.md` → **18/18**; hunks `-U0` solo en las 18 líneas objetivo; filas con ID **281 antes y 281 después**; 963 líneas antes y después; punteros `Verificación:` 9 (A) + 9 (B); cero `U+FFFD` (escritura UTF-8 verificada por conteo de `ñ`/`ó`). Sin compilación (no se tocó Rust).
- **Review P2-01:** `vanta-review` (contexto fresco) ronda 1 → `changes-required`: encontró **2 citas heredadas de B que HEAD desmiente** (la degradación P4 está en `vantadb-server/src/scheduler.rs:75-81`, no en `vanta-memory/services/scheduler.rs:78-81`; el test del reparto de `FIND-598` es `vfile.rs:2223-2250`) y **2 conteos narrativos** ("5 vivas" → 4; frontmatter "5 pasan a Pendiente" → 6 quedan). Las 4 correcciones aplicadas; además agregó la sección §Review que faltaba. Ronda 2: ver §Review del task file.
- **No tocado, a propósito:** `src/**`, `tests/**`, `vanta-memory/**`, `vantadb-mcp/**`, `.github/**`, `dev-tools/**`, `.opencode/**`, `docs/dev/plans/**`, `docs/index.md` (sin `gen-index`), los dos `PARCIALES-verificacion-*.md` y `FIND-777`. El encabezado de totales del Backlog sigue diciendo "294 filas" con 281 reales: saneamiento aparte, declarado como `NOTICED BUT NOT TOUCHING`.

### FIND-798: `create` ya no deja un nodo huérfano cuando falla el índice, y `list` ya no oculta hilos tras un id colgante
- **Fecha:** 2026-11-27
- **Task file:** [FIND-798](../../tasks/FIND-798.md)
- **Objetivo:** `create` insertaba el nodo (`thread.rs:255`) y escribía el índice después (`:256`). Si el índice fallaba, el nodo quedaba persistido sin entrada en `_thread_ids`: invisible para `list` y para `purge_expired_threads` (los dos caminan el índice, no el backend), inmortal aunque tuviera TTL, y el error "create falló" invitaba a un reintento que acuñaba un id nuevo con `generate_id()` → segundo nodo.
- **Resultado:** orden invertido (índice primero, nodo después) + compensación del id cuando el `insert` falla, + `register_ttl` movido detrás del insert exitoso (antes dejaba al GC una expiración —y un `Delete` de WAL en el barrido siguiente— para un id que nunca tuvo nodo). Un `create` que falla en el índice deja **cero** nodos y el reintento deja **uno**.
- **Segundo defecto, hallado en la review y arreglado aquí:** el orden nuevo sólo es barato si un id sin nodo no cuesta nada, y **sí costaba**: `list` hacía `skip(offset).take(limit)` sobre los ids crudos y filtraba después, así que un id colgante gastaba un slot de página. Con `has_more = returned > limit` (`server/pagination.rs:38`) eso devolvía `has_more: false` y **sin cursor** mientras los hilos existían y se leían por id — el mismo "invisible" que esta tarea prohíbe, alcanzado desde `list`. No era teórico: `GcWorker::sweep` (`gc.rs:57`) ya dejaba ids colgantes porque borra el nodo sin tocar el índice. `list` ahora resuelve primero y pagina después; sobre datos sanos el resultado es idéntico.
- **Demostración:** `create_failing_at_the_index_leaves_no_orphan_node` sobre `HEAD` → `found 1` nodo tras un `create` que devolvió `Err`; después del fix, 0 tras el fallo y 1 tras el reintento. `create_failing_at_the_insert_rolls_back_the_index_entry` cubre la compensación con una precondición determinista y por instancia —dos transacciones abiertas hacen que `insert` devuelva `InvalidInput` (`insert.rs:164-178`) mientras `put_to_partition` no conoce transacciones—. `list_pages_over_threads_not_over_dangling_index_ids` fija el reparto. Sin costura nueva: el mecanismo de la crate (`fail` / `failpoints`) no está en `Get-CoreFeatures` (`dev-tools/verify.ps1:66`) y su failpoint de insert es global al proceso, así que haría flaky el binario de tests.
- **Techo declarado:** la ruta `save_thread_ids` del índice no se ejercita (el test inyecta el fallo en `load`; ambos cuelgan del mismo `?` y abortan antes del insert). El fallo simultáneo de insert y cleanup —el caso que dispara el `tracing::warn!`— no tiene inyección por instancia disponible.
- **No tocado, a propósito:** `add_edge`/`remove_edge` (`FIND-773`/`FIND-809`). Un id colgante **no** lo recicla el purge: el rebuild de `purge_expired_threads` sólo corre `if purged > 0` y un id sin nodo no incrementa `purged` — la versión anterior de esta entrada y el comentario del código afirmaban lo contrario y se corrigieron.
- **Verificación:** filtro `test(thread) or test(agentic) or test(create)` → `cargo nextest list` **35 → 39**; `cargo nextest run` con el mismo filtro y `--no-tests fail`: **39/39**. Módulo `agentic::thread`: **6/6**. `cargo fmt --check` ✅ · `cargo clippy -p vantadb --all-targets -- -D warnings` ✅.
- **Review P2-01:** `vanta-review` (contexto fresco) ronda 1 → `changes-required` con 2 Required reales (R1: el ocultamiento por slot de página; R2: la compensación sin test y su justificación falsa) y 2 Optional (O1: `let _ =` silencioso; O2: TTL registrado antes de existir el nodo), más correcciones de citas (N1/N2). Los seis aplicados; el reviewer confirmó el reordenamiento y la ausencia de inversión de locks.

### FIND-800: el TTL de los threads ya no promete un `GcWorker` que nadie instancia
- **Fecha:** 2026-11-27
- **Task file:** [FIND-800](../../tasks/FIND-800.md)
- **Objetivo:** el módulo prometía «auto-expiry» y ningún camino de producción la ejecutaba: `Embedded::create_thread`/`send_message` pasaban `gc = None` (`builder.rs:374,382`), el `GcWorker` sólo se instancia en los tests de `gc.rs`, y `send_message` re-armaba `_expires_at` en cada mensaje (`thread.rs:372-382`). El TTL existía como campo persistido y como doc, no como mecanismo.
- **Decisión — salida (b), no encender el GC, y por qué:** el índice del `GcWorker` es un `BTreeMap` **en memoria** (`gc.rs:30`): nace vacío en cada arranque, así que instanciarlo **no** vuelve durable la expiración (el deadline durable ya vive en el nodo, `_expires_at`, `thread.rs:278`); compartirlo entre los `ThreadStore` que el SDK crea por llamada exigiría un `Mutex<GcWorker>` tomado en toda escritura de hilo; y `sweep` borra por `StorageEngine::delete` directo (`gc.rs:85`) **sin tocar `_thread_ids`**, o sea que el barrido es un productor de ids colgantes que sólo `list` tolera (FIND-798), y el único que los recicla es el rebuild de **otro** purge que sí borre algo. Encima `sweep` lee `web_time::SystemTime` (`gc.rs:67`), no el `Clock` inyectable: sería código de producción que ningún test puede mover en el tiempo.
- **Semántica declarada:** el TTL es una **ventana de inactividad**, no un deadline absoluto. `send_message` la re-arma; un hilo con conversación activa nunca caduca, uno en silencio sí. Y el mecanismo único es `purge_expired_threads` (`thread.rs:449`, tool MCP `thread_purge_expired`, `Embedded::purge_expired_threads` en `builder.rs:413`): **antes de esa llamada un hilo vencido sigue leyéndose**. El borde es estricto (`_expires_at < now`, `thread.rs:486-488`), así que en el segundo exacto del deadline todavía vive. `ttl_secs = 0` no es «sin TTL»: expira al segundo siguiente y no se puede rescatar.
- **Doc corregido en 11 sitios:** `//!` del módulo de `thread.rs`, `//!` de `agentic/mod.rs` (la página que seguía diciendo «TTL-based garbage collection via `GcWorker`»), `ThreadStore`, `create`, `send_message`, `purge_expired_threads`, `//!` de `gc.rs`, `register_ttl`, `sweep` y la línea de `Embedded::create_thread`; más la tabla de threads de `docs/api/EMBEDDED_SDK.md`. **Sin cambio de código**: cero firmas, cero ramas, cero locks tocados.
- **Demostración:** `expired_thread_is_readable_until_the_explicit_purge_runs` (0 al segundo exacto, sigue legible al siguiente, 1 tras el purge, vacío en `get` y `list`) y `send_message_re_arms_the_ttl_so_an_active_thread_never_expires` (mensaje a los 50 s con `ttl=60` ⇒ a los 100 s sigue vivo; 61 s de silencio después ⇒ purgado). Son **de caracterización**, no hay RED posible: el defecto era una promesa de doc sin mecanismo, y se declara así en vez de disfrazarlo de TDD.
- **Coste:** 0 en producción (no se enciende nada). El coste que queda es el del purge explícito, que ya existía y lo paga quien lo llama: un `get` por hilo indexado y, **sólo si algo expiró**, un segundo pase con otro `get` por superviviente más la reescritura completa del blob del índice (hasta 2N lecturas + 1 escritura).
- **No tocado, a propósito:** `vantadb-mcp/src/threads.rs:32` sigue diciendo «auto-expiry» (ruta prohibida; el doc ya acota «purged later via `thread_purge_expired`»); `spawn_memory_ttl_sweeper` (`gc.rs:181-216`) purga memorias y **no** hilos — engancharlo costaría O(hilos) por tick y hay que medirlo antes; y la carrera `purge` ↔ `send_message` (locks distintos, `index_lock` vs `message_lock`) puede re-crear un nodo cuyo id el rebuild ya había descartado: ventana estrecha, **no reproducida**, y lo que falta es una **costura de test** (sin ella un fix no se verifica). El orden `index_lock → message_lock` sería seguro: `create`/`delete` sólo toman el primero y `send_message` sólo el segundo.
- **Verificación:** contrato mecánico — `cargo nextest list --profile audit -p vantadb --build-jobs 2 -E 'test(gc) or test(thread) or test(ttl) or test(expire)'` → **45** tests; `cargo nextest run` con **el mismo** filtro + `--no-tests fail` → **bloqueado** por una mutación en vivo de otro carril (`src/wal.rs:134` y `src/storage/vfile_crypto.rs:379` llevan un `PROBE:` con el código desactivado), no por este diff; el subconjunto acotado `--lib --test message_thread_test` sí corrió antes de esa mutación: **41/41**, con las dos nuevas incluidas. `cargo fmt --check` ✅ · `cargo clippy --workspace --all-targets --all-features -- -D warnings` ✅ (2m34s) · `cargo clippy -p vantadb --lib --test message_thread_test -- -D warnings` ✅ · `node scripts/docs/check-links.mjs` ✅ · `node scripts/docs/check-docs.mjs` ✅.
- **Review P2-01:** pendiente de la ronda 1.

### `eval_locomo.py` contra el `locomo10.json` oficial (`FIND-667`)

- **Fecha:** 2026-11-27. El arnés de LoCoMo sólo entendía el diccionario plano que produce su propio `generate_synthetic_locomo`; con el archivo de `snap-research/LoCoMo` (2 740 KB, 10 muestras, **1 986 QA**) lanzaba `KeyError`.
- **Lo medido contra el upstream, no supuesto:** `conversation` trae `session_N` (lista de turnos `{speaker, dia_id, text}`) **y** `session_N_date_time` hasta `session_35` aunque el contenido sólo llega a `session_19` — descubrir sesiones por la fecha habría ingestado sesiones vacías. En `qa`, `answer` es `str` 1 536 veces, **falta la clave 444 veces** (todas adversariales) y es **`int`** 6 veces (`2022`, `2`, `3`): el `int` rompía `compute_f1_overlap` con `AttributeError` una capa después del `KeyError` que la fila documentaba. `question`/`evidence`/`category` nunca faltan.
- **Lo que se escribió:** `adapt_official_locomo()` (único punto que conoce los dos esquemas), `_evidence_to_sessions()` (`D8:6` → `session_8`, descartando y contando lo irresoluble), `apply_locomo_fixed()` (AMB 2026: 1986 → **1538**, `no_evidence` 4 + `no_answer` 444), `--dataset-format {auto,locomo-official,locomo-internal}`, `embed_query_one()` con `query_embed()` BGE en espejo de `eval_longmemeval.py`, y `--strict-embeddings` que **falla** en vez de caer al fallback léxico o a `FakeLocomoStore`.
- **Tabla honesta, no adorno:** las 5 categorías aparecen (antes `adversarial` no existía), cada fila lleva su `n`, una categoría no medida dice `(sin datos)` con `Δ —` en lugar de un delta contra cero, y un RSS no medido dice `n/d` — antes imprimía `350.0x menor consumo` calculado sobre `0.0`.
- **Tabla §23.4 medida** (`bge-small-en-v1.5` 384d, `--strict-embeddings`, `top_k=10`, sobre `locomo-fixed`):

  | Categoría | n | Rec@5 | Rec@10 | MRR | NDCG@5 |
  |---|---:|---:|---:|---:|---:|
  | single_hop | 100 | 0.9400 | 0.9600 | 0.7432 | 0.7898 |
  | multi_hop | 100 | 0.5502 | 0.7583 | 0.6199 | 0.4893 |
  | temporal_reasoning | 100 | 0.8500 | 0.9600 | 0.6662 | 0.7010 |
  | open_domain | 92 | 0.5385 | 0.6836 | 0.5116 | 0.4607 |

  El F1 del arnés **no** es comparable con el de Mem0: es overlap léxico contra el chunk, sin generador (`llm_tokens_consumed: 0`).
- **Verificación:** `--self-test` exit 0 (con las aserciones nuevas del adaptador y del filtro); `--strict-embeddings` sin modelo → **exit 1** nombrando el modelo, sin la bandera → exit 0; 4 corridas reales exit 0 con sus JSON en `evals/results/locomo_official_fixed_*.json`.
- **Deuda declarada:** `docs/user/operations/BENCHMARKS.md` §23.4 queda sin pegar (fuera de la zona del lote); no hay corrida completa de 1538 (≈3 h); el archivo arrastra **85 errores `ruff` en `HEAD`** y este cambio suma **0** (conteo 85 antes / 85 después).

---

## Lote seguridad/cifrado en reposo (2026-11-27)

### FIND-586: el AAD ata el ciphertext a su fichero (formatos cifrados v4/v5)

- **Fecha:** 2026-11-27
- **Objetivo:** `ADR-0057 D4` promete que un ciphertext movido de sitio falla la autenticacion, pero el AAD solo llevaba el `slot` (offset/ordinal intra-fichero): un nodo de `vstore_L0.vanta@X` autenticaba en `vstore_L1.vanta@X` (misma DEK, mismo epoch, mismo slot, misma longitud) => el indice HNSW resolvia el vector de otro nodo sin error; y un frame de `vanta.shard0.wal@N` autenticaba en `vanta.shard1.wal@N` (la cadena VER-01 no lo detiene: SHA-256 sin clave, y el replay no la valida).
- **Resultado:** `AadBuilder::identifier` con la identidad del fichero en los dos artefactos con el defecto (VantaFile y WAL). El indice HNSW y el backend KV ya la llevaban (path de la imagen / partition+key) y no se tocan. Identidad: el nombre del segmento para el VantaFile y el stem antes de `.wal` para el WAL (estable a traves de la rotacion, distinta por shard). Bump de formato: VantaFile v3->v4, WAL v4->v5; los layouts previos se rechazan con `IncompatibleFormat` (nunca se leen con el binding debil) - sin migracion: los formatos cifrados no se publicaron (D2).
- **Verificacion:** dos tests nuevos, RED medido con el identificador deshabilitado (Prove-It): `vfile_v3_node_blob_cannot_be_moved_to_another_segment` y `wal_v4_frame_cannot_be_moved_to_another_shard` - con el fix desactivado ambos fallan (el blob/frame injertado autentica), con el fix pasan. Area completa: **319/319**. `fmt`/`clippy -D warnings`/`rustdoc -D warnings` limpios.
- **Delta declarado:** `FIND-498` (sacar el AEAD del lock del escritor) sigue abierto como fila propia: atar el nombre permite el cambio, pero es concurrencia y necesita su medicion.
- **Ids:** `FIND-586`, `FIND-500` (misma raiz), `ADR-0057 D4`

### La deriva del soak no son los tombstones: es el corpus lineal (`FIND-669`)

- **Fecha:** 2026-11-27. La fila pedía decir **cuál** de tres mecanismos causaba la deriva. Se midió con **cuatro variantes del mismo workload** a 1 000 turnos, apagando uno por vez, y **ninguno de los tres** es el cuello.
- **Las cuatro variantes** (`soak_test.py --variant`): `baseline` (sólo put) **71.7×** de p50 entre el primer y el último checkpoint · `no-supersede` 49.4× · `no-forget` 28.2× · `full` 38.5×. La variante **sin supersede ni forget** —cero tombstones, cero cadenas, cero reescrituras— deriva **más** que la completa. Con 1 000 turnos hay 50 supersessions y 10 forgets: diez eventos no sostienen una pendiente.
- **El mecanismo real, con instrumento determinista:** `evals/search_scaling_probe.py` abre una base **nueva** por tamaño, inserta N claves, cronometra 100 búsquedas idénticas, 3 repeticiones con mediana. Resultado: **exponente log-log 1.148** y `us/clave` ×1.37 sobre un corpus 20× mayor ⇒ la búsqueda es **lineal en el corpus** y el soak gana una clave por turno. `HnswConfig::flat_threshold = Some(10000)` hace que el índice escanee plano por debajo de 10 000 nodos, y el soak nunca pasa de 1 050.
- **Ruido declarado:** en una máquina con dos carriles Rust compilando, la varianza entre corridas idénticas es 2-3× (el `baseline` dio 27.46 ms a 1 000 claves y 21.82 ms a 2 000 en corridas sucesivas). Por eso el veredicto se apoya en la sonda de tamaño fijo, no en una sola corrida del soak.
- **Entregado:** gate `--max-slope-factor` (default 3.0) que **falla** con `exit 1` y dice `NO-APLICA` cuando hay un solo checkpoint; `--variant` para re-atribuir sin reescribir el arnés; la sonda con exponente y veredicto automático.
- **Dos defectos del arnés, corregidos en el acto:** `get_process_rss_mib()` devolvía **`0.0`** en cualquier fallo y el reporte lo imprimía como medición (`Final RSS: 0.0 MiB`) — en Windows usaba `psapi.dll`, ausente en algunas instalaciones; ahora intenta `psutil` → `K32GetProcessMemoryInfo` → `resource` y devuelve **`None`** (`rss_available: false`, consola `n/d`). Con `psutil` mide de verdad: **137.0 MiB, delta +100.3 MiB**. Y `Certified Forgets Verified` estaba **hardcodeado a `/10`, así que un `--turns 200` con 2 forgets imprimía `2/10`.
- **Verificación:** 6 corridas reales; gate probado en las tres direcciones (`NO-APLICA` a 1 checkpoint, `FAILED` con factor 15.89 a 300 turnos, `PASSED` a 200 turnos con factor 2.38). `ruff`: **30 errores antes y 30 después** (sin deuda nueva).
- **Queda y es del motor:** la fila pedía «poda/compactación incremental», pero **no hay tombstones que podar**. El arreglo es de `src/index/**`: bajar el umbral del escaneo plano, y decidir si el gate se define sobre p50 absoluto (exige búsqueda sublineal) o sobre `us/clave`, que es el invariante sano y ya está instrumentado en el reporte.

### `search_hybrid` del addon nativo: 701× la vectorial a 10 000 registros (`FIND-668`)

- **Fecha:** 2026-11-27. La fila afirmaba `115–1869 ms` de híbrida contra `1–7 ms` vectorial. **Nunca se había reproducido** porque `vantadb-node/bench/bench-abi.mjs` no arranca: importa `../../vantadb-ts/dist/vantadb.js` a nivel de módulo (`SyntaxError: ... does not provide an export named 'VantaDB'`), una dependencia que la mitad nativa no necesita.
- **Instrumento:** `evals/node_abi_search_probe.mjs` carga sólo el addon **prebuilt** (`vantadb-node/index.cjs` → `vantadb_native.win32-x64-msvc.node`): cero compilación Rust, cero dist WASM, LCG determinista.
- **Medido** (`--sweep 500,2000,10000 --searches 20`): 500 reg → vectorial 1.192 ms vs híbrida 21.097 ms (**17.7×**); 2 000 → 2.057 vs 144.83 ms (**70.4×**); 10 000 → 3.404 vs **2 386.437 ms** (**701×**). De 2K a 10K (5× datos) la vectorial crece 1.65× y la híbrida **16.5×**.
- **La forma es la prueba:** exponente log-log **1.71** en la híbrida contra **0.33** en la vectorial. Un coste de fusión constante habría dejado la misma pendiente; lo que hay es coste **por candidato** (materialización de payloads y serialización por N-API). El techo real (2 386 ms) es **peor** que el que declaraba la fila (1 869 ms).
- **Contrato + vara:** proyectar sólo `(doc_id, score)` durante BM25+HNSW+RRF, materializar payload/snippet únicamente para el `top_k` final; aceptación a 10 000 reg = `hyb/vec ≤ 4×` y pendiente ≤ 1.2.
- **Zona:** `vantadb-node/src/**` y `vantadb-wasm/src/**` no se tocaron (otro carril). El probe es el entregable y es reproducible sin build.

### El cifrado en reposo: la opción (b) ya está aplicada y el p99 sigue +43,2 % (`FIND-670`)

- **Fecha:** 2026-11-27. La fila ofrecía dos caminos para bajar el p99 del cifrado al `<15 %`: (a) vectores cuantizados SQ8/F16 en RAM con AEAD exacto sólo en el re-ranking, o (b) caché LRU con `zeroize`. **El (b) ya está en el árbol desde octubre**, y la medición registrada **es con la caché puesta**.
- **Fechas y hashes, no recuerdos:** `DecryptCache` (`lru::LruCache<u64, Arc<AlignedBlob>>`, `vfile.rs:213`) entra en **`991d00c76` (2026-10-21)** con el presupuesto `limit/16` clampeado a `[1 MiB, 256 MiB]`; el `Zeroize` en drop del plaintext en **`e2a05ed94` (2026-10-23, FIND-618)**. `BENCHMARKS.md` **§22.4** se titula «re-medición **post-Paso 0**» y es del **2026-10-21/22**: es la foto **después** de la caché.
- **El veredicto que ya estaba escrito:** p50 **+50,8 %**, p95 **+37,9 %**, p99 **+43,2 %**, con el criterio fijado antes de ver los números (`max(claro)×1.15 = 19,2854 < min(cifrado) = 22,0183`) ⇒ **el presupuesto del 15 % sigue violado con (b) aplicada**. Los tres pares intercalados dan el mismo signo (+31,3 %, +58,2 %, +85,6 %).
- **Por qué (b) no alcanza:** una caché no quita el AEAD por candidato cuando el paseo del grafo visita más nodos distintos que entradas caben en el presupuesto, ni cuando el re-ranking de `top_ef` vuelve a tocar el ciphertext. Y el presupuesto efectivo —`memory_limit_bytes/16`, 10 MB para 25k×100d— puede no cubrir el working set: hoy no hay contador de aciertos que lo diga.
- **Contrato corregido:** (1) SQ8/F16 en RAM para el paseo + AEAD exacto sólo en `top_ef`; (2) contador de aciertos de `DecryptCache` por corrida; (3) re-medir §22 sobre GloVe-100 25k×100d con el método de §22.4 y vara p99 `<15 %` **o el número real declarado**.
- **Sin número nuevo, y se dice:** `cargo bench --features encryption` necesita build release y en este momento **no compila** — otro carril tiene `src/wal.rs` y `src/storage/vfile_crypto.rs` en edición (`E0425` en `vfile_crypto.rs:377`, `E0382` en `wal.rs:920`). Medir ese estado intermedio sería publicar el WIP de otro carril. Cero cambios de código en esta fila.

### Borrado certificado: `certified` no da 0 % hasta compactar — el bench lo encontró (`FIND-675`, `FIND-826`)

- **Fecha:** 2026-11-27. La fila pedía un benchmark público de `forget()` y afirmaba que en `certified` la recuperación de bytes debía ser **0 %**. Se entregó `benchmarks/certified_unlearning_bench.py` (determinista, solo `vantadb`, 0.7 min) y **la aserción falla**.
- **Escaneo forense** (base nueva por modo, marcador único de 62 hex en el payload, vector determinista, **testigo** con vector propio que debe sobrevivir): `soft` deja payload en **3** archivos y el vector en **5**; `hard` en **2** y **3**; **`certified` en 2 y 3 — idéntico a `hard`**. Los hits, nombrados: `0.jnl` (10×), `data\vanta.shard0.wal` (2×) y `data\vstore_L0.vanta` para el vector.
- **El 0 % existe, pero no en `certified`:** tras `compact_wal()` + `compact_layout()` → **0 archivos, 0 archivos**, testigo vivo. O sea que la erasure byte-level es alcanzable y el recibo no declara que haga falta. Eso es `FIND-826` (GDPR Art. 17: el recibo dice `purged` sobre un dato que está en disco).
- **No es el instrumento:** el primer intento compartía el vector del testigo con el del sujeto y el needle marcaba el testigo como fuga; corregido antes de publicar. El testigo sobrevive siempre ⇒ el «0 archivos» no es una base vacía.
- **Churn HNSW** (borrar + reinsertar el mismo volumen, corpus constante): 10 % → p50 0.972 ms / p99 1.239 / QPS **1004.89**; 30 % → 1.425 / 1.673 / **733.59**; 50 % → 1.488 / 1.676 / **669.21**, sin resultados vacíos. Degradación moderada (p50 ×1.53, QPS ×0.67).
- **El script sale `exit 1`** mientras la recuperación de `certified` no sea 0: un benchmark que siempre sale verde no mide nada. `ruff check` + `ruff format --check` limpios.

### FIND-496: el WAL shipping no replica una base cifrada (falla al inicio)

- **Fecha:** 2026-11-27
- **Objetivo:** `WalShipper::ship_once` abria los segmentos rotados con `WalReader::open` (sin clave): sobre un WAL cifrado fallaba con `EncryptionKeyRequired` a mitad de la replicacion, despues de haber enviado los lotes de los segmentos anteriores.
- **Resultado:** contrato (b) de la fila: el shipping declara que no soporta bases cifradas y falla al inicio del ciclo, antes de cualquier POST, con un mensaje accionable (nombra el segmento, el formato y las dos salidas: desactivar `wal-shipping` o mantener el WAL en claro). La deteccion lee el `format_version` del header en claro, asi que no necesita clave. No se envia material de clave por la red (descartado por la fila).
- **Verificacion:** `shipper_refuses_an_encrypted_wal_before_shipping_anything`: RED medido con la guarda deshabilitada (el error era el seco `Encryption key required for wal`); con la guarda el ciclo se niega y no escribe el marker (nada se envio). Poblacion del area: 319/319.
- **Deuda declarada:** la replicacion de bases cifradas no existe y este fix la declara imposible, no la implementa (requeriria compartir la DEK con el destino).
- **Ids:** `FIND-496`

---
title: "Avance — Core Engine"
type: domain-log
status: active
tags: [vantadb, avance, core, engine, storage, wal, hnsw, acid]
last_reviewed: 2026-08-07
aliases: []
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
- **Resultado:** ✅ 24/24 tareas. Suite final 361/361 tests en vanta-memory; fmt/clippy `-D warnings` limpios. Registro completo: `docs/progreso/campanas/p27-memory-engine.md` (split GOV-D2). Plan: `docs/plans/archive/2026-08-18-vanta-memory.md`.
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
- **Resultado:** ✅ Infra de niveles ya existía (`src/lsm.rs` `SegmentLevel` L0-L3); gap era la *política de tier* y archive L3. `TierPolicy` enum (SizeBased | FrequencyBased | AgeBased) + `TierPolicyConfig` (archive on/off, cold_min_access, cold_age_days); `LsmConfig` extendido (`l3_max_size`, `l3_tombstone_threshold`, `tier`). Promoción encadenada en `compact_level`: L0(hot)→L1(warm)→L2(cold)→L3(archive), L3 terminal y solo si `tier.archive=true`. Tests `test_tier_promotion_hot_to_cold`, `test_tier_promotion_cold_to_archive`, `test_tier_archive_disabled_stops_at_cold` 3/3. Doc `docs/architecture/STORAGE-TIERS.md` (EN).

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
- **Resultado:** ✅ ADR-019: sparse se persiste como `FieldValue::ListFloat(Vec<f64>)` con pares intercalados `[dim, val]` (lossless u32→f64/f32→f64, orden determinista por BTreeMap) en vez de `FieldValue::String(serde_json)` bajo `SPARSE_VECTOR_EXT_KEY`. Write path `sparse_vector_to_field` sin serde_json (elimina ~1.49% del hot-path de búsqueda); read path dual: `ListFloat` decode directo + `String` legacy para compat backward; faltante → `None` (PERF-07); corrupto → warn + `None`. `VantaMemoryRecord.sparse_vector` público intacto. Sin migración one-shot (lazy en próximo put); shim legacy hasta gate de versionado de storage. 1885/1885 tests + clippy `-D warnings` + fmt --check ✅. Review P2-01 APPROVE. Commit `2f1a94e1`.

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
| PERF-03 | Bench competitivo honesto SDKs (Qdrant/Chroma/Lance/Milvus) en mismo HW; tabla en `docs/benchmarks/COMPETITIVE_SDK_BENCH.md` (VantaDB pierde recall, gana QPS) | ✅ 2026-08-12 |
| PERF-05 | WAL async roadmap (ADR `DRV-015-wal-async-roadmap.md`): io_uring/aio + fsync group commit post-DRV-014 | ✅ 2026-08-12 |
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
### ERR-032 (storage test), ERR-047 (search Cow), ERR-048 (search visited), ERR-008 (vfile copy_unsafe obsoleto), ERR-049 (ivf bench) — migrados 2026-08-12 (ver docs/progreso/README.md)
### COV-003 (CLI subcommand tests migrate/server/crud, +7 tests, cli_handlers ~0%→~76.5%) — migrado 2026-08-12 (ver docs/progreso/README.md)

### AUD-025: BM25 zero-alloc hot path (per-posting allocations) — migrado 2026-08-14 (ver docs/progreso/README.md)
- **Resultado:** ✅ `src/text_index.rs:565` `posting_record_key` → `&str` zero-alloc (`strip_prefix` + `from_utf8`); `src/sdk/search/phrase.rs` matcher genericizado (`K: AsRef<str> + Ord`, helper `find_positions`); `src/sdk/search/mod.rs:383-448` hot path sin `token.clone()`/`String::from`/`format!` por posting, `doc_stats_cache` keyed por `u128 node_id` con guard de mismatch. `cargo check -p vantadb` ✅, clippy ✅, fmt ✅, 104 tests (13 phrase + 91 search) ✅. Commit `96b258ba`.

### FND-15: Crash recovery / WAL en la práctica (verificación vanta-chaos) — migrado 2026-08-16 (ver docs/progreso/README.md)
- **Resultado:** ✅ verificación en `docs/research/FND-15-crash-recovery-verificacion.md`: kill a mitad de escritura recupera estado consistente (`chaos_integrity`/`wal_resilience`) — sin gap de producto, gap de infra de tests documentado. Commit `8c6044a1`.

### FND-20: Trade-off HNSW (ef_search/M: recall vs latencia) + argumento vs IVF/FAISS — migrado 2026-08-16 (ver docs/progreso/README.md)
- **Resultado:** ✅ `docs/architecture/FND-20-hnsw-tradeoff.md` (EN): parámetros actuales (M=32, ef=100) citados (`graph.rs:255-269`, `nearest.rs:71-77`, `neighbors.rs:57-62`, `ivf.rs:79-228`, `auto_tune.rs:11-53`), trade-off recall/latencia/memoria, sección "Why not FAISS/IVF" para local-first. Drift documentado: ADR 005/PERFORMANCE_TUNING.md dicen ef_construction=200/400, código dice 100 — la nota cita el código como fuente de verdad. Commit `4051a850`.

### FND-01: Regla de presupuesto de memoria + benchmark OOM — migrado 2026-08-16 (ver docs/progreso/README.md)
- **Resultado:** ✅ 🔴 CONFIRMADO: RSS sin límite real, guard subestima ~6.5× bajo carga 10k w/s + 1k r/s. Regla must en `.opencode/rules/memory-budget.md`. Follow-ups F1/F4 delegados a core-engine. Commit `a159211b`.

### FND-02: Regla de coordinación multi-índice + auditoría de deadlocks — migrado 2026-08-16 (ver docs/progreso/README.md)
- **Resultado:** ✅ fix deadlocks evicción multi-índice (lock no reentrante + write guard, `c104f1f2`) + regla en `.opencode/rules/concurrency-async.md` + audit P2-01 approve; follow-up lock order `93a1e311`. Follow-ups menores 2/3 delegados a core-engine.

### FND-08: Regla de backend validado contra patrón de acceso real + auditoría de compactación — migrado 2026-08-16 (ver docs/progreso/README.md)
- **Resultado:** ✅ ADR-023 (backend compaction — diferir marginal, bench de lectura) + regla en `.opencode/rules/durability.md`. Commit `e5e76684`.

### VS-CORE-07: Retención de versiones históricas (D2 completo)
- **Fecha:** 2026-08-18
- **Resultado:** ✅ src/sdk/version_history.rs (nuevo, 11 tests) + partición Fjall Versions (clave 
s_len‖ns‖key_len‖key‖ver BE) + hooks put/put_batch/delete/purge_expired + VantaConfig.version_history_limit (cap 32 FIFO aprobado, snapshot del record nuevo, import sin snapshots). API get_version/ersions. Commits e0812a4/6997e59. Doble consumidor P26+P27. 1785 lib tests verdes (1 fallo preexistente maintenance.rs fuera de scope).

### REVIEW-09: Latch `saturated` monotónico en cache_warmer — warming vuelve a aprender tras decay
- **Fecha:** 2026-08-23
- **Fuente:** Plan `docs/plans/2026-08-23-backlog-triage.md` (Wave 1) · Backlog · review-full-20260822 H09-CODE-001
- **Resultado:** ✅ `record_co_access` seteaba `saturated=true` al cruzar `max_pairs` pero `decay()` reducía la tabla sin resetearlo → en servers long-running el warmer quedaba en refresh-only para siempre. Fix: `decay()` levanta el latch cuando el post-decay total < `max_pairs` (≤1 transición por cruce; sin thrashing verificado con decays que mantienen post-total ≥ cap). TDD: RED reprodujo el bug, 2 tests nuevos (ciclo completo + no-thrash), módulo 11/11. Verify full: fmt ✅ · clippy `-D warnings` ✅ · nextest audit workspace 2714/2714 ✅. Review P2-01 (vanta-review) aprobó tras corregir doc deshonesta del campo `saturated`. Commit `8b8924b3`. (ver `.opencode/skills/campaign-executor/tasks/REVIEW-09.md`)

### REVIEW-14: Keys cortas en version_history devuelven error de corrupción sin panic — unwraps frágiles eliminados
- **Fecha:** 2026-08-23
- **Fuente:** Plan `docs/plans/2026-08-23-backlog-triage.md` (Wave 2) · Backlog · review-full-20260822 H09-CODE-005
- **Resultado:** ✅ DISCOVERY reclasificó el hallazgo: el slice frágil de `version_history.rs:283` era test-only (ningún path de producción parsee keys del partition Versions) y `VantaError::Corrupt` no existe (clase corrupción del codebase = `BackendError`; añadir variante rompería matches exhaustivos de bindings P2-6). Fix real: helper `version_from_key()` (`Result<u64>`, `BackendError` si key <8B — key corta solo puede venir del store, nunca de input de usuario) + validación en `versions()` como defensa ante store corrupto + test nuevo (len 0..7 → error tipado, boundary 8B OK). explain.rs: 3 unwraps estructurales eliminados bindeando `Some(query_sparse)` en los patterns (Keyword nullifica la Option, no solo un bool — semántica idéntica arm-por-arm). Siblings con mismo patrón (mod.rs/debug_ops.rs) derivados a FIND-27. Verify full: fmt ✅ · clippy `-p vantadb -D warnings` ✅ · nextest `-p vantadb` 2051/2051 ✅. Commit `4044a588`. (ver `.opencode/skills/campaign-executor/tasks/REVIEW-14.md`)

### MOD-01: WAL escrito ANTES de validar — insert/update rechazado resucita datos tras restart
- **Fecha:** 2026-08-23
- **Fuente:** Plan `docs/plans/2026-08-23-backlog-triage.md` (Wave 1) · Backlog · core.md H-1 · cross-modulos F-1 (expuesto vía WASM)
- **Resultado:** ✅ Los 3 mutadores de `InMemoryEngine` escribían al WAL antes de validar: una op rechazada quedaba loggeada y el replay (`with_wal`) la aplicaba incondicionalmente como upsert — duplicado rechazado pisaba el payload original y update sobre nodo eliminado lo recreaba tras reopen. Fix: orden validate→WAL→apply bajo UNA sección crítica `nodes.write()` en insert/update/delete (double-checked descartado: deja ventana TOCTOU no retractable en append-only). Discovery mapeó TODOS los write paths: StorageEngine/batch/bulk/txn no violan el invariante (upsert por diseño, guards previos al WAL, buffer solo commitea validado) — H-1 vivía solo en el motor legacy. TDD: RED reprodujo ambas resurrecciones mecánicamente, 4 tests de durabilidad nuevos (reject×2 + flujo legítimo×2 con reopen), GREEN 4/4. Verify full: fmt ✅ · clippy workspace `-D warnings` ✅ · nextest `-p vantadb` 2049/2049 ✅ · audit workspace 2718/2718 ✅ · docs coverage 0 gaps ✅. Commit `18fd2c80`. (ver `.opencode/skills/campaign-executor/tasks/MOD-01.md`)

### FIND-27: Provider Ollama postea a endpoint legacy /api/embeddings - fix a /api/embed (2026-08-24)
- **Fecha:** 2026-08-24
- **Plan:** `docs/plans/2026-08-24-batch-review-mod-find.md` (Wave 0)
- **Resultado:** OK - el provider Ollama nativo posteaba a `/api/embeddings` (endpoint legacy, campo `prompt`) en vez de `/api/embed` `{model,input}` vigente, fallando contra Ollama actual. Fix: endpoint + response shape (`embeddings[0]`), con 2 tests contra mock HTTP (endpoint correcto + rechazo de embeddings vacios). Verify: `cargo nextest run -p vantadb --features remote-inference llm::` 2/2 PASS. Commit `447a07d7`.

### MOD-02: Replay de transacciones no crash-atomicas - fix recover_state bounds skip al extent de la txn (2026-08-24)
- **Fecha:** 2026-08-24
- **Plan:** `docs/plans/2026-08-24-batch-review-mod-find.md` (Wave 0)
- **Resultado:** OK - root cause: (1) tests previos usaban BackendKind::InMemory que jamas llama recover_state; (2) crash-sim escribia en path raiz pero el engine usa data_dir/vanta.wal; (3) fix previo (txn_end) era no-op. Fix: tracking per-id `open_txn` (Commit/Abort cierran solo la txn que matchea; un Begin nuevo marca el limite del batch incompleto; trailing incompleto se descarta fail-safe). Tests: 45/45 init+txn, chaos failpoints 1/1, durability/crash 8/8. Verify: `cargo nextest run -p vantadb -E ''test(init::) or test(txn)''` 45/45. Commit `db8b26b7`. Ceiling documentado: registro plano sin markers inmediatamente despues de txn parcial es indistinguible y se descarta fail-safe.

### FIND-28: 3 casts u8*->f32* sin chequeo de alineacion - migrados a as_f32_slice (align_to) (2026-08-24)
- **Fecha:** 2026-08-24
- **Plan:** `docs/plans/2026-08-24-batch-review-mod-find.md` (Wave 1)
- **Resultado:** OK - 3 casts crudos `u8*->f32*` (from_raw_parts sin chequeo de alineacion = UB) en ivf.rs, distance/mapper.rs, serialize/bytes.rs reemplazados por el helper canonico `VectorRepresentations::as_f32_slice()` (align_to), eliminando 3 bloques unsafe y la const muerta MAX_VEC_F32_LEN. Review P2-01 vanta-audit APPROVE. Verify: check/clippy -D warnings/fmt, nextest 2055/2055. Commit `2d9fa75f`. FIND-29 derivado (search/layer.rs candidato, sound hoy).

### FIND-25: create_snapshot con quiesce (flush ERR-010) + recorrido recursivo de data_dir (2026-08-25)
- **Fecha:** 2026-08-25
- **Plan:** `docs/plans/2026-08-25-batch-backup-restore-chain.md` (Task 1/3, cadena secuencial)
- **Resultado:** OK - (1) Quiesce: create_snapshot llama self.flush() antes del imageado en ambas variantes cfg (guard read_only; mismo patron que rebuild_vector_index/compact_layout_bfs: flush ANTES de tomar insert_lock, lock no reentrante) -> el conjunto imageado es mutuamente consistente en un punto unico en el tiempo. (2) Recorrido recursivo: helper mirror_data_dir() excluye el subdir snapshots/ (dentro de data_dir, evita snapshots/snapshots/) y mirror_file() unifica hard_link (unix) / copy (windows-wasm). (3) Layout real documentado: data_dir solo tiene archivos top-level (vstore_L0..L3.vanta, vector_index.bin, vanta.wal); el backend KV vive FUERA (escalado como FIND-33). Trade-off performance documentado en docstring (flush agrega costo, correctness>speed). Test de regresion test_snapshot_consistent_under_concurrent_writes (4 writers concurrentes + reopen + asserts anti-torn). Nota RED honesta: pre-fix pasaba en Windows via replay total del WAL (checkpoint_seq no se copia al snapshot) - el tear real era conjunto-inconsistente index/vstore en la variante copy. Verify: cargo nextest run -p vantadb snapshot --ignore-default-filter 6/6 PASS + 13 unit PASS + fmt PASS + clippy -D warnings PASS. Sin commit (lead verifica). Colateral: FIND-33 creado.

### FIND-37: Eliminar `query_sparse.unwrap()` panickable en dispatcher híbrido (2026-08-27)
- **Fecha:** 2026-08-27
- **Plan:** `docs/plans/2026-08-27-backlog-pipeline.md` Task 1 (Wave 0) · `vanta-worker`
- **Resultado:** ✅ Dispatcher híbrido (`VantaEmbedded::search_impl`) asumía `Some` en 6 sitios vía `query_sparse.as_ref().unwrap()` (líneas 207,240,265,315,346,369) — request sin sparse dispara panic en prod (hot path). Fix: reemplazar `has_sparse: bool` + unwrap por `query_sparse: Option<&SparseVector>` filtrada `!is_empty` + match `Some(qs)` (pattern de `explain.rs` REVIEW-14 / `hybrid.rs:36`). También 3 sites en `debug_ops.rs` (288,335,374) mismo patrón. `hybrid_search` ahora recibe `Option` filtrado. Semántica preservada: sparse None/empty → fallback silencioso a text-only/vector-only (no ValidationError). Verify: `rg query_sparse.*unwrap src/sdk/search/mod.rs` → 0 ✅ · `rg src/sdk/search/` → 0 ✅ · `cargo check -p vantadb` ✅ · `cargo nextest -E 'test(search)'` 157 passed ✅ · `cargo fmt --check` ✅ · pre-commit hook clippy ✅. Commit `bd7c2691` `fix(search): FIND-37 eliminate query_sparse.unwrap panics`. Sin deuda neta (saldo negativo: 9 panics eliminados).
- **Archivos:** `src/sdk/search/mod.rs`, `src/sdk/search/debug_ops.rs`
- **Contrato:** `cargo nextest run -p vantadb --profile audit -E 'test(search)'` ✅ + `rg -n "query_sparse.*unwrap" src/sdk/search/mod.rs` → 0 + `cargo clippy -p vantadb --all-targets --all-features -- -D warnings` sin nuevos warnings

### FIND-39: ScalarIndex.remove sin test — cobertura engine-level (2026-08-27)
- **Fecha:** 2026-08-27
- **Plan:** `docs/plans/2026-08-27-backlog-pipeline.md` Wave 1 #11 · `vanta-worker` — Origen codegraph-20260827-143245 Fase 10 (score 0.98, 0 hits en storage/engine/tests)
- **Resultado:** ✅ Añadido `test_scalar_remove` en `src/storage/engine/tests/scalar_index.rs` (93L, engine-level): direct `ScalarIndex` API (insert 2 reds → remove 1 → 1 left, no-op missing field/value/id, cross-field isolation, remove last) + wiring engine (insert alpha → overwrite beta → alpha cleared, delete → cleared). Registrado en `src/storage/engine/tests/mod.rs` (`mod scalar_index;`). Unit tests preexistentes en `src/scalar_index.rs` (test_scalar_index_remove etc.) ya cubrían happy path pero no el wiring engine; gap cerrado a nivel integración.
- **Archivos:** `src/storage/engine/tests/scalar_index.rs` (nuevo), `src/storage/engine/tests/mod.rs`
- **Contrato:** `cargo nextest run -p vantadb scalar_index --profile audit` → 15 passed (incl. test_scalar_remove) ✅ + `cargo nextest list --profile audit` 2069 → 2070 (+1) + `rg -n "pub fn remove" src/scalar_index.rs` → 2 ✅ + `cargo nextest run -p vantadb -E 'test(storage::engine)'` 306 passed ✅ + `cargo fmt --check` ✅ + `cargo clippy` ✅ + `validate-docs-coverage` 0 gaps ✅
- **Commit:** `59f5cdcb` `feat(test): FIND-39 add test_scalar_remove for ScalarIndex.remove`
- **Gates:** D: no-disparado (no feature-add, no symbols públicos nuevos) · V: no-disparado (verify pasó al 1er intento) · C: no-disparado (solo test, no API change)
- **Skills:** campaign-executor, progreso, ponytail, source-driven-development, test-driven-development, incremental-implementation, rust-write-tests, code-review-and-quality

### FIND-34: Ciclo WAL Writer — DAG documentado + edge tests recovery/quarantine (2026-08-27)
- **Fecha:** 2026-08-27
- **Plan:** `docs/plans/2026-08-27-backlog-v2.md` Task 1 (Wave 0) · `vanta-worker` — Origen codegraph-20260827-143245 Fase 1 (4 nodos `open↔open_with_buffer↔recover↔quarantine` reportado como ciclo High)
- **Resultado:** ✅ DAG verificado (grep 0 back-edge: `recover`/`quarantine` leaves, solo callers en `open_with_buffer`), falso positivo Leiden co-localización documentado. Doc header `src/wal.rs:178-193` con diagrama DAG + ponytail ceiling. Tests: `test_recover_mid_file_corruption_scan_forward_recovers_tail` (16B garbage mid + raw framing valid tail → scan-forward 3/3) + `test_quarantine_rotates_when_corrupt_exists` (`.corrupt` → `.corrupt.1` rotation sin overwrite). Cobertura: `test_wal_auto_healing_and_recovery` + `test_corrupt_wal_tail_is_quarantined` preexistentes + 2 nuevos.
- **Archivos:** `src/wal.rs` (doc DAG 15L + 2 tests ~80L; `recover_valid_records` def 545, `quarantine_corrupt_tail` def 592, 1 def c/u)
- **Contrato:** `cargo nextest run -p vantadb --profile audit -E 'test(wal)'` → 62 passed (60→62, 2009 skipped, 6.08s) ✅ + `rg -n "quarantine_corrupt_tail|recover_valid_records" src/wal.rs` → 1 def c/u con cobertura ✅ + `cargo check -p vantadb` ✅ + `cargo clippy -p vantadb --all-targets --all-features -- -D warnings` 0 ✅ + `cargo fmt --check` ✅ + `validate-docs-coverage` 0 gaps ✅ + doc `src/wal.rs:178-193` justifica `codegraph_explore "wal cycle FIND-34"` como DAG (no SCC)
- **Commit:** `fix: FIND-34 — WAL writer DAG justification + recovery/quarantine edge tests` (pendiente)
- **Gates:** D: no-disparado (no symbols públicos nuevos, blast radius 2 archivos) · V: no-disparado (verify wal 1º intento 62/62) · C: no-disparado (doc + tests, no API change)
- **Skills:** campaign-executor, progreso, ponytail, source-driven-development, systematic-debugging, incremental-implementation, test-driven-development, documentation-and-adrs
- **Notes:** Ponytail rung 1 — doc antes que refactor `open_inner`. Skipped: ADR separado, helper extraction. Add when: SCC real con back-edge. Quarantine fails soft (warn); scan-forward byte-by-byte O(n) solo en recovery, no hot path.

### FIND-35: Ciclo StorageEngine get/prefetch — SCC intencional 2 nodos justificado + PrefetchGuard (2026-08-27)
- **Fecha:** 2026-08-27
- **Plan:** `docs/plans/2026-08-27-backlog-v2.md` Task 2 (Wave 0) · `vanta-worker` — Origen codegraph-20260827-143245 Fase 1 (2 nodos `StorageEngine.get ↔ prefetch_related` reportado como ciclo)
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
- **Resultado:** ✅ 39 ADRs encontrados en `docs/architecture/adr/` (001..013, ADR-0001, ADR-014..032, COMP-*, DRV-*). ADR-001 (`001_unified_config_readonly.md`) tiene Context/Decision/Consequences ✅. CodeGraph reporte Fase 12 ("Sin ADRs registrados") era stale — ADRs existen desde 2026-08-23.
- **Commit:** `docs: FIND-44 — verify ADRs exist, contract satisfied` (pendiente)

### CORE-01: Persistencia on-disk de vectores Binary/Turbo/SQ8 en vstore (2026-08-29)
- **Fecha:** 2026-08-29 (sincronización por vanta-worker; implementación commiteada 2026-08-28 por vanta-arch)
- **Plan:** `docs/plans/2026-08-29-full-backlog-parallel.md` Task W4-SOLO (CORE-01, SOLO, 🟡 1-2d, max 2d)
- **Objetivo:** Cerrar el durability gap de `Binary`/`Turbo`/`SQ8` en `write_node_to_vstore` (que escribía `vector_len=0` y cero bytes de payload). Tras `flush + reopen` con HNSW rebuild, los vectores cuantizados se perdían.
- **Resultado:** ✅ Implementación commiteada (d3e7f9cf + 854d9145). ADR-032 creado con tabla de 5 kinds (NONE/FULL/BINARY/TURBO/SQ8) + payload spec + reader dual legacy kind==0 + lazy migration; `NodeFlags::VECTOR_KIND_*` constants en bits 10-13 de `flags` (mask `0x3C00`, shift 10); `VFILE_VERSION` permanece en 2; tests pasan (`test_rebuild_binary_vector`, `test_persistence_binary_vector_roundtrip_vstore`); 93 tests binary|persistence|vstore|rebuild todos verdes.
- **Archivos:** `src/storage/ops.rs` (write_node_to_vstore reescrito, dispatch 6 variantes, SQ8 scale tail 4B), `src/node/flags.rs` (VECTOR_KIND_* constants), `src/storage/archive.rs` (compact_layout + rebuild_hnsw_from_vstore despachan por kind), `src/storage/engine/get.rs` + `engine/txn.rs` (decode kind + rescue HNSW legacy), `src/index/search/layer.rs` (vstore branch despacha rabitq/turbo/sq8 similarity), `src/node/disk.rs` (doc `vector_len` reinterpret), `docs/architecture/adr/ADR-032-binary-vector-persistence.md` (nuevo, status `accepted-pending-owner-review`).
- **Contrato:** ADR-032 existe ✅ + `cargo nextest run -p vantadb --profile audit -E 'test(/binary|persistence|vstore|rebuild/)'` → 93/93 passed ✅ + `cargo fmt --check` 0 ✅ + `cargo clippy -p vantadb --all-targets -- -D warnings` 0 ✅ + `cargo test -p vantadb --lib storage::archive::tests::test_rebuild_binary_vector` 1/1 ok ✅ + `cargo test -p vantadb --lib --features fjall storage::engine::tests::init::test_persistence_binary_vector_roundtrip_vstore` 1/1 ok ✅.
- **Gates:** P: no-disparado (spec-first ADR-032 antes de implementar) · D: no-disparado (no symbols públicos nuevos, blast radius 6 archivos core, sin API pública SDK nueva) · V: no-disparado (verify completo 1º intento) · C: no-disparado (lectura dual legacy mantiene compat, VFILE_VERSION sin bump).
- **Skills:** campaign-executor, progreso, ponytail, documentation-and-adrs, source-driven-development, api-and-interface-design, database-design, spec-driven-development, incremental-implementation, test-driven-development.
- **Notes:** Ponytail rung 1: ¿necesita existir nuevo campo header? No — reusar 4 bits libres en flags es 1 línea vs ampliar header 64→72. ADR-032 escrito por IA con status `accepted`; tras Regla 5 (AGENTS.md) el owner humano debe articular el trade-off (bits vs header-bump vs migración one-shot) — marcada como `accepted-pending-owner-review` con nota crítica. Plan file `2026-08-29-full-backlog-parallel.md` tenía el estado `⬜ PENDING` con contrato PowerShell obsoleto (`Binary.*vector_len|DiskNodeHeader.*format_flag`) que no matchea el código real (usa constantes `NodeFlags::VECTOR_KIND_*`); sincronizado a `✅ COMPLETED` con contrato real validado. Riesgo: legacy Binary `kind==0, len=0` permanece irrecuperable por rebuild hasta reescritura lazy; documentado en ADR-032 §5 y Risk Register.
- **Stop conditions (no se aplicaron):** >2d → docs-only ADR + encoding fix mínimo + tests reopen roundtrip. No fue necesario.

### FIND-36: Cross-crate NativeConnection ↔ RocksDbBackend — frontera documentada, falso positivo Leiden (2026-08-27)

- **Fecha:** 2026-08-27
- **Plan:** `docs/plans/2026-08-27-backlog-v2.md` Task 3 (Wave 1) · `vanta-arch` — Origen codegraph-20260827-143245 Fase 1 (3 ciclos get/put/delete `desktop/src-tauri/src/connections/native.rs ↔ src/backends/rocksdb_backend.rs`)
- **Resultado:** ✅ Falso positivo verificado + frontera documentada. DISCOVERY `rg` 0 `use`-imports cruzados (RocksDbBackend no importa `desktop`/`tauri`/`NativeConnection`; NativeConnection no importa `RocksDbBackend`/`StorageBackend`), DAG `NativeConnection (desktop, isolated workspace [members=["."]] — `vantadb = {path="../.."}`) → `VantaEmbedded` → `StorageEngine` → `StorageBackend (pub(crate) Send+Sync, src/backend.rs)` → `RocksDbBackend (pub(crate), src/backends/rocksdb_backend.rs:22)` — 0 back-edge. Los 3 "ciclos" son colisión de nombres get/put/delete + clustering Leiden de verbos CRUD, no SCC CALLS. Workspaces aislados (`cargo check -p vantadb` invariant, `cargo tree` sin `vantadb-desktop`). Frontera ya correcta: `StorageBackend` pub(crate) + `BackendKind::RocksDb/Fjall/InMemory` + `VantaConfig.storage_path` (desktop nunca conoce `BackendPartition`/cf names). Doc headers: `src/backends/rocksdb_backend.rs:1-22` + `desktop/src-tauri/src/connections/native.rs:1-20` con diagrama DAG + ponytail ceiling + verificación `cargo check 0 cycles` + cross-ref mutua.
- **Archivos:** `src/backends/rocksdb_backend.rs` (header `//!` 15L, DAG + falso positivo + ponytail), `desktop/src-tauri/src/connections/native.rs` (header `//!` 12L, DAG + ponytail), `docs/plans/2026-08-27-backlog-v2.md` Task 3 → ✅ COMPLETED + recitation FIND-36, `docs/Backlog.md` FIND-36 eliminado
- **Contrato:** `cargo check -p vantadb --all-targets` 0.65s ✅ + `cargo check -p vantadb --all-targets --all-features` 37.55s ✅ + `cargo clippy -p vantadb --all-targets --all-features -- -D warnings` 0 ✅ + `cargo fmt --check` 0 ✅ + `rg` 0 use-imports cruzados ✅ + doc headers justifican `codegraph_explore "NativeConnection RocksDbBackend"` 0 ciclos o frontera documentada (rama ADR) — headers cumplen segunda rama sin ADR separado
- **Commit:** `fix: FIND-36 — Cross-crate NativeConnection↔RocksDbBackend frontier doc (false-positive Leiden, DAG justified)`
- **Gates:** D: no-disparado (no symbols públicos nuevos, blast radius 2 archivos + backend.rs existente, ≤10) · V: no-disparado (verify dual cargo check 1er intento) · C: no-disparado (doc only, no API change, `StorageBackend` intacto, workspaces aislados)
- **Skills:** campaign-executor, progreso, ponytail, source-driven-development, systematic-debugging, documentation-and-adrs, api-and-interface-design, code-review-and-quality
- **Notes:** Ponytail rung 1 — doc antes que `trait KvBackendPort` extracción o ADR. Skipped: ADR `docs/architecture/adr/ADR-03X-crate-boundary.md`, `cargo modules dependencies --acyclic` (tool no baseline), trait extraction (frontera `StorageBackend` ya correcta). Add when: `RocksDbBackend` necesita llamar a desktop (callback Tauri/event) → invertir vía trait `DesktopEventSink` en `src/backend.rs` + channel o `tauri::Manager` emit. Invariante: `desktop → vantadb` one-way, `StorageBackend` pub(crate) nunca pub externo. Question Gate: pre-mortem 1 falsificó premisa (falso positivo) pero contrato exige doc frontera — COMPLETED con doc, no SKIP. CodeGraph 173 calls `src → skills` (FIND-42) tangencial, deferred.

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

> Plan `docs/plans/2026-08-28-embeddings-local.md` · commits `2c185021`→`d24eeb1c` · `embeddings/` + feature `embed-local` (Regla 5: embeddings BYO-vector → local-first). Tabla modelos: 3 EN / 3 ES / 3 Combined (8 ≤3GB + Qwen3 16GB excepción).

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
- **Fecha:** 2026-08-28 — **Objetivo:** `benchmarks/embed_bench.py` 533L + `docs/operations/BENCHMARKS.md` §8 report — **Resultado:** ✅ — **Commit:** `67fda296`

### EMB-08: Docs + Quickstart multi
- **Fecha:** 2026-08-28 — **Objetivo:** fix punto 1: `docs/QUICKSTART.md:182` + `05-embedding-integrations.md` + `docs/api/EMBEDDINGS.md` + README — **Resultado:** ✅ — **Commit:** `86cdfc57`

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

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
- **Resultado:** ✅ verificación en `docs/Investigaciones/FND-15-crash-recovery-verificacion.md`: kill a mitad de escritura recupera estado consistente (`chaos_integrity`/`wal_resilience`) — sin gap de producto, gap de infra de tests documentado. Commit `8c6044a1`.

### FND-20: Trade-off HNSW (ef_search/M: recall vs latencia) + argumento vs IVF/FAISS — migrado 2026-08-16 (ver docs/progreso/README.md)
- **Resultado:** ✅ `docs/architecture/FND-20-hnsw-tradeoff.md` (EN): parámetros actuales (M=32, ef=100) citados (`graph.rs:255-269`, `nearest.rs:71-77`, `neighbors.rs:57-62`, `ivf.rs:79-228`, `auto_tune.rs:11-53`), trade-off recall/latencia/memoria, sección "Why not FAISS/IVF" para local-first. Drift documentado: ADR 005/PERFORMANCE_TUNING.md dicen ef_construction=200/400, código dice 100 — la nota cita el código como fuente de verdad. Commit `4051a850`.

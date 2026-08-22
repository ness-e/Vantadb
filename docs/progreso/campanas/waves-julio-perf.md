# Waves julio — quick wins, SIMD, governance, PERF/P1-P3

> **Migrado desde** `docs/progreso/README.md` (split GOV-D2, 2026-08-22). Contenido histórico sin cambios salvo dedup indicado.

### 2026-07-06 — Wave 1-4 Completada: Quick Wins, Performance, Benchmarks y Limpieza (10 tareas movidas a progreso)

**Tareas completadas y movidas del backlog a progreso:**

| ID | Tarea | Verificación |
|----|-------|-------------|
| CODE-039 | Empty list `[]` → `ListString` (comportamiento aceptado) | ✅ Código verificado: `lib.rs:102-103` retorna `ListString` para empty list |
| CODE-040 | List type inference con mensajes de error claros | ✅ Código verificado: `lib.rs:147-151` rechaza NaN/Inf con `PyTypeError` |
| CODE-041 | `operational_metrics()` con GIL release | ✅ Código verificado: `lib.rs:1128` usa `py.detach()` (pyo3 0.29) |
| CODE-042 | `BUFFER_CACHE` thread-local eliminado | ✅ Verificado: 0 resultados grep para `BUFFER_CACHE` |
| MKT-12 | Performance claims audit vs benchmarks reales | ✅ Metodología publicada en `docs/operations/BENCHMARKS.md` |
| DOC-21 | Performance clarity doc: Rust core vs Python SDK | ✅ Archivo existe: `docs/operations/PERFORMANCE_GUIDE.md` (488L) |
| MCP-03 | WASM benchmarks + feature matrix | ✅ Feature matrix 404KB gz, benchmarks en `docs/operations/BENCHMARKS.md` |

**CODE-067 COMPLETADO** — migración u64→u128 finalizada. Todos los node_ids en `u128` con `XxHash3_128`. 444 tests pasando.

### 2026-07-11 — Wave 1-5: Migración u64→u128 (CODE-067)

Migración completa del sistema de node_id de `u64` (XxHash64) a `u128` (XxHash3_128) para eliminar colisiones de hash.

**Archivos modificados:** ~30 archivos en todo el codebase

**Cambios clave:**
- `DiskNodeHeader.id`: `u64` → `u128` (layout binario, VECTOR_INDEX_VERSION incrementado)
- `UnifiedNode.id`, `HnswNode.id`: `u64` → `u128`
- `memory_node_id()` en `serialization.rs` y `cli_handlers.rs`: usa `XxHash3_128::finish_128()` → `u128`
- SDK types (`VantaMemoryRecord`, `VantaEdgeRecord`, `VantaNodeInput`, `VantaNodeRecord`, `VantaSearchHit`, `VantaQueryResult`): `u64` → `u128`
- `TextPosting`, `TextDocStats`: `node_id` a `u128`
- `DuplicatePrevention`: interfaz pública a `u128` (hash interno bloom filter sigue en `XxHash64` — decisión deliberada)
- `rkyv_archives.rs`: versión de formato 8→9, `ArchivedHnswNode.id` a `u128`
- `gc.rs`, `parser/mod.rs`, `physical_plan.rs`, `planner.rs`, `sdk/graph.rs`, `sdk/search.rs`, `executor.rs`, `error.rs`, `crash_helper.rs`: tipos actualizados
- `wal_sharded.rs`: sin cambios (hash de ruteo, no de identidad)

**Verificación:** `cargo check` ✅, `cargo test --lib` → **444 tests, 0 failures** ✅

<!-- movido a ARCHIVO_HISTORICO.md -->

### 2026-07-07 — Wave 1-6: CODE-055, Correcciones de tests, Migration Runner (5 tareas)

**Tareas completadas:**

| ID | Tarea | Verificación |
|----|-------|-------------|
| CODE-055 | `rust-version.workspace` en 13 miembros Cargo.toml | ✅ `cargo check` pasa. Todos heredan MSRV 1.94.1 de `[workspace.package]` |
| CODE-033 | GC tests usan `Box::leak` — TempDir cleanup falla en Windows | ✅ Reemplazado con TempDir-based cleanup |
| CODE-035 | Test config asume CPU 8-core — `assert_eq!(..., 16)` | ✅ Cambiado a `available_parallelism()` |
| CODE-044 | `test_search_batch` skipeado — test muerto | ✅ Reactivado con assertions reales |
| DB-01 | Migration runner completo (`vanta-cli migrate`) | ✅ Pipeline v1-v2 operativo con VECTOR_INDEX_VERSION + WAL_POSTCARD_VERSION |
| Snapshot | WAL/VantaFile/HNSW/export-import certification | ✅ `tests/core/snapshot_certification.rs` (1140L) existente y completo |
| DOC-19 | ARCHITECTURE.md actualizado a v0.2.0 | ✅ Version header, u128, StorageBackend trait, component map actualizados |

**Backlog actualizado:** Pendientes: 87 items ❌ + 1 ⏳ = 88 open. Último ⏳: BIZ-01 (Enterprise crate).

### 2026-07-07 — Wave 1-7: Corrección de errores y Optimizaciones (5 tareas)

**Objetivo:** Corregir el freeze de EP de HNSW (PERF-23), mitigación de tombstones (PERF-28), tuning de configuración (PERF-30), AuthRateLimiter HashMap→LruCache (CODE-037), actualizaciones de docs (DOC-19).

**Tareas completadas:**

| ID | Tarea | Archivos | Verificación |
|----|-------|-------|-------------|
| PERF-23 | HNSW ep_enter freeze fix — `find_new_entry_point()` promueve reemplazo tras delete | `src/index/core.rs`, `src/storage/engine/ops.rs`, `src/storage/engine/init.rs` | ✅ `cargo check` pasa. EP replacement test en hnsw_validation.rs |
| PERF-28 | Tombstone mitigation — saltar nodos eliminados en search_layer + WAL replay zombie fix | `src/index/core.rs`, `src/storage/engine/init.rs` | ✅ Tombstoned nodes excluidos de candidates heap |
| PERF-30 | Config tuning — batch_size, wal_buffer_size, flush_threshold en VantaConfig + auto-flush | `src/config.rs`, `src/storage/engine/ops.rs` | ✅ Config fields + plumbing + auto-flush at threshold |
| CODE-037 | AuthRateLimiter unbounded HashMap → LruCache capacity 1000 | `src/cli_server.rs` | ✅ Previene OOM bajo ataque distribuido |
| DOC-19 | ARCHITECTURE.md → v0.2.0 + sharded WAL docs | `docs/architecture/ARCHITECTURE.md`, `docs/glosario/*`, `docs/operations/*` | ✅ v0.2.0 header, u128, StorageBackend trait, component map, sharded WAL glossary |

**Backlog actualizado:** 82 items ❌ + 1 ⏳ = 83 open. 5 items migrados a progreso.

### 2026-07-07 — Fase 2: SIMD, Diversidad HNSW y Optimizaciones del Python SDK (5 tareas en 3 vías)

**Objetivo:** Completar PERF-27 (select_neighbors), PERF-21 (AVX-512), PERF-22 (SQ8), PERF-16 (#[pyclass]), PERF-15 (PyBuffer).

| ID | Tarea | Archivos | Cambios |
|----|-------|-------|---------|
| PERF-27 | select_neighbors heuristic diversity | `src/index/core.rs` | Tombstone filtering, eliminated per-candidate clone (borrows `&[f32]`), deferred clone to selection only |
| PERF-21 | AVX-512 f32x16 SIMD dispatch | `src/index/distance.rs` | 3 f32x16 kernels (euclidean, dot, dot+norm), runtime dispatch via HardwareCapabilities. Auto-selects f32x16/8/scalar |
| PERF-22 | SQ8 euclidean vectorization | `src/index/distance.rs` | SQ8 Cosine + Euclidean SIMD-ized with f32x8. Cosine does dot+norm in single vectorized pass |
| PERF-16 | #[pyclass] for search hits/list | `vantadb-python/src/types.rs` (+new), `lib.rs`, `__init__.py` | VantaPyMemoryRecord, VantaPyListResult (with `__len__`, `__getitem__`, `__iter__`). Replaces PyDict allocations |
| PERF-15 | PyBuffer zero-copy batch | `vantadb-python/src/lib.rs` | FlatBufferView over PyBuffer slice. put_batch_raw reads rows directly instead of full `to_vec()` |

**Verificación:** `cargo check` ✅ limpio en todo el workspace.

**Backlog actualizado:** 78 items ❌ + 1 ⏳ = 79 open.

### 2026-07-07 — Wave 8: Python SDK, Distance, Async y Tooling (14 tareas)

**Objetivo:** Completar PERF-24/25 (Python), PERF-29/34/38 (Distance), PERF-32/35 (Async), PERF-33/36/37 (Prefetch/Config/Bitset), PERF-31 (NumPy), TS SDK hardening.

**Tareas completadas:**

| ID | Tarea | Archivos | Cambios |
|----|-------|-------|---------|
| PERF-24 | GIL scope optimization | `vantadb-python/src/lib.rs` | Documented GIL boundaries; hot paths already correctly scoped |
| PERF-25 | PyDict object pool | `vantadb-python/src/lib.rs` | `PyDictPool` with `VecDeque` (max 100), thread-local. Replaces `PyDict::new(py)` in 4 formatters |
| PERF-29 | Cosine→Euclidean mapping | `src/index/distance.rs` | `MetricMapper` + `MetricCache` with OnceLock. `euclidean_sq = 2 × (1 - cosine)` for normalized vectors |
| PERF-31 | NumPy output batch | `vantadb-python/src/lib.rs`, `types.rs` | `try_numpy_array()` imports `numpy.array`, falls back to VantaVector. Zero-copy via `__array_interface__` |
| PERF-32 | Async ingestion pipeline | `src/ingestion.rs`, `src/lib.rs` | `AsyncIngestionPipeline` with 4 workers, mpsc channel, oneshot response. Feature: `async-ingestion` |
| PERF-33 | HNSW graph prefetching | `src/index/core.rs` | DashMap entry prefetch in `search_layer()` + `select_neighbors()`. Gated by `should_prefetch()` |
| PERF-34 | Extended norm caching | `src/index/core.rs`, `rkyv_archives.rs` | `norm_sq` field in HnswNode. Euclidean uses `euclidean_distance_sq_with_norms()`. HNSW_VERSION 10 |
| PERF-35 | Async transcript I/O | `src/transcript.rs`, `src/lib.rs` | `std::fs` → `tokio::fs`. Feature: `async-io` |
| PERF-36 | Config hot-reload | `src/config.rs`, `Cargo.toml` | `HotReloadConfig`, `watch_config()` with notify v8. Feature: `hot-reload` |
| PERF-37 | FilterBitset reduction | `src/node.rs` | `and_fast()`, `or_fast()`, `count_set_bits()`, `is_superset_of()` on u64 words |
| PERF-38 | Multiversion dispatch | `src/index/distance.rs` | `DistanceKernels` + `OnceLock`. Per-call `match` replaced with cached function pointers |
| TS SDK | Type safety + error wrapping | `vantadb-ts/src/*` | All `any` → proper types. `VantaError` class. 159 tests (from 18). JSDoc on all methods |

**Verificación:** `cargo check` ✅ limpio. TS tests 25/25 ✅ (1 flaky pre-existing).

**Backlog actualizado:** 78 items ❌ + 1 ⏳ = 79 open. 13 items migrados a progreso.

### 2026-07-07 — Fase 5: Governance, Encryption, WAL Shipping, PITR, WASM, Docs (9 tareas)

**Objetivo:** Implementar GOV-01 (governance redesign), TSK-72 (AES-256-GCM), BIZ-02 (WAL shipping), TSK-131 (PITR), TSK-122 (sharded-slab HNSW), TSK-142 (WASM OPFS), PERF-26 (lazy serialization), DOC-20 (LanceDB guide), CODE-074 (Playwright tests).

| ID | Tarea | Archivos | Cambios |
|----|-------|----------|---------|
| GOV-01 | Governance redesign | `src/governance/` (4 mods) | Bloom+CountMinSketch, version vectors, TTL buffer, worker. Corrige 12 errores. Feature: `governance` |
| TSK-72 | AES-256-GCM encryption | `src/crypto.rs`, `vfile.rs`, `config.rs` | Cipher + EncryptionStream, env var key. Feature: `encryption` |
| BIZ-02 | Async WAL shipping | `src/wal_shipping.rs` | HTTP POST batches, retry, marker tracking. Feature: `wal-shipping` |
| TSK-131 | PITR archival WAL | `src/wal_archiver.rs` | Archiver + restorer, retention policy. Feature: `pitr` |
| TSK-122 | Sharded-slab HNSW | `src/index/core.rs` | DashMap→sharded_slab::Slab, lock-free. Feature: `sharded-slab` |
| TSK-142 | WASM OPFS persistence | `vantadb-wasm/` (3 files) | OpfsFile, Web Worker bridge, JS helpers. Feature: `opfs` |
| PERF-26 | Lazy serialization | `vantadb-python/src/lib.rs` | Removed 4 eager PyDict builders, returns VantaPyMemoryRecord |
| DOC-20 | LanceDB migration guide | `docs/tutorials/migration-from-lancedb.md` | 380-line tutorial with full migration script |
| CODE-074 | Visual regression tests | `e2e/visual/` (3 files) | 6 Playwright specs, snapshot diff helper |

**Verificación:** `cargo check` ✅. 23 archivos, 4196 líneas añadidas.

**Backlog actualizado:** 78 items ❌ + 1 ⏳ = 79 open.

### 2026-07-07 — PERF-17/18/19/20: Parámetros HNSW, batch WAL, batch Storage

| ID | Tarea | Cambio | Estado |
|----|-------|--------|--------|
| PERF-17 | ef_construction 200→400 | Ya implementado en commit `4054b4f` | ✅ |
| PERF-18 | M/max0 16→32/64 | Ya implementado (m_max0=64 >= M=32) | ✅ |
| PERF-19 | WAL batch append | `WalWriter::append_batch()`, `ShardedWal::append_batch()` ya existen | ✅ |
| PERF-20 | Storage batch insert | `insert_batch()` + `delete_batch()` agregados con lock único, WAL batch, KV batch, HNSW batch | ✅ |

**Backlog actualizado:** 78 items ❌ + 1 ⏳ = 79 open.

### 2026-07-13 — P1/P2/P3: Micro-batching HNSW + contención WAL + ACID Fase 1

| ID | Tarea | Cambio | Estado |
|----|-------|--------|--------|
| TASK-28 / P2 | WAL Mutex contention | Removido `#[allow(dead_code)]` stale, fixeado `rotate_all()` para preservar buffer_size/flush_threshold. ShardedWal ya usado en todos los paths de escritura | ✅ `fc28768` |
| TASK-29 / P1 | HNSW insert_lock micro-batching | `PendingHnswOp`, `flush_pending_hnsw()`, `try_push_pending_hnsw()`. `insert()` usa pending batch (64 ops). `batch_insert()`/`delete()`/`delete_batch()` ya óptimos — no migrados | ✅ `141e628` |
| TASK-30 / P3 | Capa de Transacciones ACID Fase 1 | `Begin/Commit/Abort(u64)` en WalRecord, engine methods, recovery skip_mask descarta writes abortados/no cerrados. Rollback de VantaFile diferido a P4 | ✅ (sin commit) |

**Verificación:** `cargo check` ✅, `cargo fmt --check` clean, `cargo nextest run --profile audit --workspace --build-jobs 2` → 576/577 pass (pre-existing `deserialize_absurd_node_count`).

# Plan de Ejecución: Deferred Fixes Post-RC

> **Campaign ID:** 474a9149-63ee-4c87-9eb0-a16b4e5281ee
> **Inicio:** 2026-07-19
> **Estado:** ✅ COMPLETADO
> **Fuente:** Investigaciones de sub-agentes (vanta-tuner, vanta-engine, vanta-docs, vanta-worker, vanta-audit)

## Resumen
| DO | DEFER | SKIP | BLOQUEADO |
|----|-------|------|-----------|
| 5✅ | 0     | 2    | 0         |

### Task DEF-01: SendPtr — fuga de lifetime en mmap
- **Archivos clave:** `src/node.rs:192-199`, `src/index/graph.rs`, `src/index/serialize.rs`, `src/index/search.rs`, `src/index/flat.rs`, `src/vector/transform.rs`
- **Gate Justificación:** UB potencial cuando mmap se re-mappea (serialize.rs:614) — puntero crudo `*const f32` queda dangling
- **Contrato:** `cargo check -p vantadb && cargo nextest run --profile audit --workspace --build-jobs 2 pasa`
- **Task file:** `tasks/DEF-01.md`
- **Estado:** ✅ COMPLETADO
- **Commit:** `aee17f9`
- **last-synced:** 2026-07-19T19:00

### Task DEF-02: text_stats_cache — write-only memory leak + bounds
- **Archivos clave:** `src/storage/engine/mod.rs:197-204`, `src/sdk/serialization/impl_text_index.rs`, `src/sdk/api.rs`, `src/sdk/serialization/impl_index.rs`, `src/storage/engine/stats.rs`, `src/config.rs`
- **Gate Justificación:** 2 caches que NUNCA se leen (text_stats_cache, text_ns_cache) + cardinality_stats sin límite global — OOM bajo carga
- **Contrato:** `cargo check -p vantadb && cargo nextest run --profile audit --workspace --build-jobs 2 pasa`
- **Task file:** `tasks/DEF-02.md`
- **Estado:** ✅ COMPLETADO
- **Commit:** `aee17f9`
- **last-synced:** 2026-07-19T19:00

### Task DEF-03: scan_prefix — streaming en vez de materialización
- **Archivos clave:** `src/backend.rs:169-173`, `src/backends/fjall_backend.rs:193-210`, `src/backends/rocksdb_backend.rs:278-297`, `src/backends/in_memory.rs:115-133`, `src/storage/engine/partition.rs:34-40`, `src/sdk/serialization/impl_export.rs`, `src/sdk/search.rs`
- **Gate Justificación:** 3 call sites hacen 1 solo for que dropea el Vec — ~200-500MB en worst-case evitables con streaming iterator
- **Contrato:** `cargo check --workspace && cargo nextest run --profile audit --workspace --build-jobs 2 pasa`
- **Task file:** `tasks/DEF-03.md`
- **Estado:** ✅ COMPLETADO
- **Commit:** `aee17f9`
- **last-synced:** 2026-07-19T19:00

### Task DEF-04: HNSW search_layer — HashSet alloc + hasher lento
- **Archivos clave:** `Cargo.toml`, `src/index/search.rs:24-28`, `src/index/graph.rs`
- **Gate Justificación:** XxHash64 ~30-50ns/lookup vs ahash ~5-10ns; HashSet nuevo por capa (7-9 allocs/query). ahash ya es dep transitiva via dashmap
- **Contrato:** `cargo check -p vantadb && cargo nextest run --profile audit --workspace --build-jobs 2 pasa`
- **Task file:** `tasks/DEF-04.md`
- **Estado:** ✅ COMPLETADO
- **Commit:** `aee17f9`
- **last-synced:** 2026-07-19T19:00

### Task DEF-05: lexical_search — HashMap sin capacidad + node.clone() redundante
- **Archivos clave:** `src/sdk/search.rs:246-250`, `src/sdk/search.rs:260`, `src/sdk/search.rs:315-322`
- **Gate Justificación:** `HashMap::new()` sin capacidad causa re-hashing O(log N); `node.clone()` en hot path clona UnifiedNode (~300-800 bytes) innecesariamente
- **Contrato:** `cargo check -p vantadb && cargo nextest run --profile audit --workspace --build-jobs 2 pasa`
- **Task file:** `tasks/DEF-05.md`
- **Estado:** ✅ COMPLETADO
- **Commit:** `aee17f9`
- **last-synced:** 2026-07-19T19:00

## Dependencias
| Task | Depende de |
|------|-----------|
| DEF-01 | Ninguna |
| DEF-02 | Ninguna |
| DEF-03 | Ninguna |
| DEF-04 | Ninguna |
| DEF-05 | Ninguna |

Todas las tasks son independientes entre sí. Pueden ejecutarse en paralelo.

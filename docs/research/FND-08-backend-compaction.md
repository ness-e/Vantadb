# FND-08 — Backend validado contra patrón de acceso real + auditoría de compactación

> **Fecha:** 2026-08-16 · **Tipo:** auditoría de config backend (P20a) · **Estado:** ✅ COMPLETO
> **Task file:** `.opencode/skills/campaign-executor/tasks/FND-08.md` · **ADR:** `ADR-023-backend-compaction.md`

## Objetivo

Verificar que la config de compactación del backend (Fjall default, RocksDB
opt-in) está tuneada para el patrón real de VantaDB — **escrituras pequeñas
frecuentes + reads random** — y no con defaults pensados para escritura
secuencial masiva (bulk-load). Complementa FND-02 (locks multi-índice ya
commiteado; no se tocó `src/storage/engine/`).

## Patrón de acceso real (verificado en código)

| Operación | Dónde | Tamaño típico |
|-----------|-------|---------------|
| Write: `put`/`write_batch` de NodeMetadata (relational, edges) | `src/storage/engine/get.rs:79`, `src/storage/engine/partition.rs` | < 1 KiB |
| Write: postings de índices derivados (text/payload/namespace/sparse) | `src/text_index.rs:728-840`, `sdk/serialization/impl_text_index.rs` | < 1 KiB |
| Write: tombstones | `BackendPartition::Tombstones` | pequeño |
| Read: `get(id)` point por key u128 LE (16 B) | `src/storage/engine/get.rs:79` | 16 B key |
| Read: `get_many` (multi_get nativo en ambos backends) | `rocksdb_backend.rs:182-221`, `fjall_backend.rs:141-156` | batch |
| Read: `scan_prefix` sobre índices derivados | `rocksdb_backend.rs:278-303`, `fjall_backend.rs:200-223` | prefijo |

**Nota clave:** los vectores NO viven en el backend KV — viven en
`VantaFile`/mmap (L0..L3) y HNSW. El backend guarda metadata + índices
derivados. El working set del backend es proporcional a nodos × payloads,
no a la dimensión vectorial.

## Config actual (archivo:línea)

### Fjall (default, `Cargo.toml:97`)

- `src/backends/fjall_backend.rs:54-57` — `Database::builder(path).open()`:
  **todos los defaults del builder**.
- `:60-93` — `KeyspaceCreateOptions::default` en las 9 particiones.
- Defaults efectivos (validados contra source `fjall-3.1.8`, `docs.rs/fjall`):

| Opción | Default | Fuente |
|--------|---------|--------|
| `cache_size` | **32 MiB fijo** (NO escala a RAM) | `fjall-3.1.8/src/db_config.rs:90` |
| `worker_threads` | `min(#cores, 4)` | `db_config.rs:70` |
| `max_journaling_size` | 512 MiB | `db_config.rs:77` |
| `manual_journal_persist` | `false` (flush al OS automático; durabilidad real la da el WAL de VantaDB) | `db_config.rs:80` |
| `max_memtable_size` (keyspace) | 64 MiB | `keyspace/options.rs:91` |
| `data_block_size` (keyspace) | **4 KiB** | `keyspace/options.rs:95` |
| Compaction | Leveled default | `keyspace/options.rs:124` |

### RocksDB (opt-in, `Cargo.toml:45` + feature)

- `src/backends/rocksdb_backend.rs:32-142`:
  - Bloom filter 10 bits (`:48`), `cache_index_and_filter_blocks` +
    `pin_l0_filter_and_index_blocks_in_cache` (`:50-51`) — **correcto para
    random reads**.
  - LRU block cache = 75% de 60% del budget RAM (`:58-60`), write buffer
    clamp 8–128 MiB / max 2 (`:62-65`), LZ4 (`:44`), `max_background_jobs(4)`
    (`:43`), mmap si RAM < 16 GiB (`:80-89`).
  - **NO setea**: `level_compaction_dynamic_level_bytes`, `target_file_size_base`,
    `max_bytes_for_level_base`, `level0_file_num_compaction_trigger`,
    `num_levels`, `optimize_for_point_lookup`.

## Defaults de la crate vs patrón real

| Aspecto | Defaults de la crate | ¿Penaliza el patrón real? |
|---------|----------------------|---------------------------|
| Fjall block 4 KiB | Docs fjall: "for point read heavy workloads a sensible default is 4–8 KiB" | **No** — alineado con random reads |
| Fjall memtable 64 MiB | Rango recomendado 8–64 MiB | **No** — amortiza writes pequeñas frecuentes |
| Fjall cache 32 MiB fijo | Docs: recomienda 20–25% RAM | **Marginal** — suficiente mientras working set ≤ 32 MiB; corto para DBs grandes con payloads pesados |
| RocksDB bloom + pin L0 | Default rocksdb: sin bloom | **No** — el bloom 10 bits + pin L0 es exactamente el tool para point reads |
| RocksDB sin level tuning | Default: `dynamic_level_bytes=false`, triggers 4 | **Marginal** — aumenta write amp bajo carga mixta sostenida; RocksDB es opt-in |

## Clasificación de gaps

- **gap-real:** NINGUNO. Ni Fjall ni RocksDB están configurados con defaults
  de bulk-load que penalicen el patrón real. Fjall hereda defaults ya
  point-read/write-friendly; RocksDB ya tiene las opciones dominantes
  (bloom, cache, pinning).
- **marginal (2):**
  1. Fjall `cache_size` fijo en 32 MiB — no escala a RAM ni a working set.
  2. RocksDB sin `level_compaction_dynamic_level_bytes` / triggers de nivel.
- **ok (todo lo demás):** block 4 KiB, memtable 64 MiB, worker threads,
  journal policy, bloom+pin+LRU, mmap governance.

## Decisión (ADR-023)

**Mantener la config actual. Diferir ambos gaps marginales.** Sin cambio de
código. Rationale:

1. **Regla 9 (no optimizar sin medir):** los benches existentes
   (`backend_compare.rs` 5k records; `canonical_p99.rs` búsqueda vectorial)
   usan datasets muy por debajo de la cache de 32 MiB — un before/after de
   `cache_size` o level tuning mediría ruido, no el gap. Cambiar config sin
   un bench que pueda demostrar la diferencia es especulación.
2. **Working set no medido:** el backend guarda metadata + índices (no
   vectores); el working set real aún no está medido. Escalar cache al
   20–25% RAM sin ese dato arriesga desperdiciar memoria.
3. **RocksDB es opt-in:** su falta de level tuning es un matiz del fallback,
   no un defecto del path default.

## Bench

- **Referenciado (no creado — ponytail: existe > nuevo):**
  `benches/backend_compare.rs:70-94` — `random_get` con seed 42, 500 queries
  sobre 5k records, p50/p95/p99 (`bench_get_latency_distribution:125-155`).
  Verificado que **compila** con `cargo check -p vantadb --bench backend_compare
  --features rocksdb` (2026-08-16, ✅).
- **Señal de reapertura (ADR-023):** aplicar tuning cuando (a) un bench con
  working set > 32 MiB muestre regresión de latencia vs baseline, o (b) RocksDB
  pase a primario con write amplification medida. Entonces: `cache_size` desde
  `effective_memory` (20–25% RAM) y `level_compaction_dynamic_level_bytes(true)`.

## Entregables

| Archivo | Acción |
|---------|--------|
| `docs/architecture/adr/ADR-023-backend-compaction.md` | ✅ nuevo (numerado tras ADR-022) |
| `.opencode/rules/durability.md` | ✅ regla agregada (config backend = patrón de acceso documentado) |
| `.opencode/skills/campaign-executor/tasks/FND-08.md` | ✅ nuevo |
| `benches/backend_compare.rs` | referenciado, verificado que compila |
| `src/backends/*`, `src/storage/engine/*` | **sin cambios** (diferimiento justificado) |

## Fuentes

- docs.rs/fjall — `DatabaseBuilder`, `KeyspaceCreateOptions` (2026-08-16)
- fjall-3.1.8 source local (`.cargo/registry/src/.../fjall-3.1.8/`)
- RocksDB Tuning Guide / Compaction Options (web, 2026-08-16)
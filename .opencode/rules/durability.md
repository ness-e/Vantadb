# Durability & WAL — Reglas

> **Scope:** `src/wal.rs`, `wal_sharded.rs`, `wal_archiver.rs` (pitr), `wal_shipping.rs` (wal-shipping), `storage/engine/`, `backends/`, `storage/vfile.rs`, `storage/archive.rs`, `gc.rs`, `lsm.rs`, `schema.rs`, `migration.rs`, `binary_header.rs`, `shred/`
> **No tocar aquí:** índices (`indexes.md`), concurrencia/async (`concurrency-async.md`), API pública (`api-contract.md`)
> **Status:** 🟡 En revisión
> **Fuentes:** DRV-014, DRV-133

## Reglas

> **Pendiente de decisión (INV-012, requiere aprobación humana):** INV-012 re-evaluó la anti-localidad y concluyó **WONTFIX** — mantener LSM + multi-level storage (2N/3N de I/O en cold reads es aceptado a cambio de simplicidad y determinismo; `benches/vfile_search.rs` re-ejecutado 2026-08-04 confirma el tradeoff). Cuando se decida formalmente, documentar aquí como regla: NO reintroducir un buffer cache complejo ni layout de datos con localidad física sin una medición que justifique el costo de complejidad. Fuente: INV-012 §2 (Rendimiento) y §3 (Complejidad vs localidad).

> **Config de backend = patrón de acceso documentado (ADR-023, FND-08):** la
> config del backend KV (Fjall default, RocksDB opt-in) DEBE justificarse contra
> el patrón de acceso real de VantaDB: **escrituras pequeñas frecuentes + reads
> random** (metadata + índices derivados; los vectores viven en `VantaFile`/mmap,
> NO en el backend). Must: (a) NO cambiar opciones de compactación/memtable/cache
> sin un bench before/after contra `benches/backend_compare.rs` (random_get, seed
> 42) con dataset que exceda la cache en juego — si el dataset del bench no
> supera la cache, el delta medido es ruido (Regla 9); (b) cualquier cambio de
> config backend requiere ADR que cite el patrón de acceso y el bench; (c) señal
> de reapertura documentada: Fjall `cache_size` fijo en 32 MiB y RocksDB sin
> `level_compaction_dynamic_level_bytes` son gaps marginales aceptados mientras
> el working set < 32 MiB y RocksDB sea opt-in (ADR-023). Por qué: los defaults
> de Fjall (block 4 KiB, memtable 64 MiB) ya están alineados con point-reads y
> writes pequeñas; cambiar sin medición es especulación.

<!-- Referencias cruzadas: → ver concurrency-async.md, indexes.md -->

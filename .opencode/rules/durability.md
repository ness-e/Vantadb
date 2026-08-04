# Durability & WAL — Reglas

> **Scope:** `src/wal.rs`, `wal_sharded.rs`, `wal_archiver.rs` (pitr), `wal_shipping.rs` (wal-shipping), `storage/engine/`, `backends/`, `storage/vfile.rs`, `storage/archive.rs`, `gc.rs`, `lsm.rs`, `schema.rs`, `migration.rs`, `binary_header.rs`, `shred/`
> **No tocar aquí:** índices (`indexes.md`), concurrencia/async (`concurrency-async.md`), API pública (`api-contract.md`)
> **Status:** 🟡 En revisión
> **Fuentes:** DRV-014, DRV-133

## Reglas

> **Pendiente de decisión (INV-012, requiere aprobación humana):** INV-012 re-evaluó la anti-localidad y concluyó **WONTFIX** — mantener LSM + multi-level storage (2N/3N de I/O en cold reads es aceptado a cambio de simplicidad y determinismo; `benches/vfile_search.rs` re-ejecutado 2026-08-04 confirma el tradeoff). Cuando se decida formalmente, documentar aquí como regla: NO reintroducir un buffer cache complejo ni layout de datos con localidad física sin una medición que justifique el costo de complejidad. Fuente: INV-012 §2 (Rendimiento) y §3 (Complejidad vs localidad).

<!-- Referencias cruzadas: → ver concurrency-async.md, indexes.md -->

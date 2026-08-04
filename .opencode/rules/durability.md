# Durability & WAL — Reglas

> **Scope:** `src/wal.rs`, `wal_sharded.rs`, `wal_archiver.rs` (pitr), `wal_shipping.rs` (wal-shipping), `storage/engine/`, `backends/`, `storage/vfile.rs`, `storage/archive.rs`, `gc.rs`, `lsm.rs`, `schema.rs`, `migration.rs`, `binary_header.rs`, `shred/`
> **No tocar aquí:** índices (`indexes.md`), concurrencia/async (`concurrency-async.md`), API pública (`api-contract.md`)
> **Status:** 🟡 En revisión
> **Fuentes:** DRV-014, DRV-133

## Reglas

<!-- Pendiente: reglas de Durability & WAL. Ver README.md → Reglas para las reglas (formato R2). -->

<!-- Referencias cruzadas: → ver concurrency-async.md, indexes.md -->

---
title: "Durabilidad en VantaDB"
kind: concept
status: active
description: "WAL, fsync, checkpoints, recuperación ante crash y cifrado en reposo: qué garantiza el motor y dónde están sus límites."
---

# Durabilidad en VantaDB

Audiencia: operadores y evaluadores que necesitan saber qué sobrevive a un crash. La referencia canónica de garantías es [DURABILITY_GUARANTEES](../../user/operations/DURABILITY_GUARANTEES.md); esta página explica el mecanismo y enlaza fuentes.

## Componentes

`[HECHO]` El WAL es público: `WalWriter` ([`src/wal.rs:648`](../../../src/wal.rs#L648)), `WalReader` ([`:2087`](../../../src/wal.rs#L2087)), `WalRecord` ([`:144`](../../../src/wal.rs#L144)) y las versiones `WAL_FORMAT_VERSION` ([`:26`](../../../src/wal.rs#L26))/`WAL_POSTCARD_VERSION` ([`:74`](../../../src/wal.rs#L74)) se re-exportan (`src/lib.rs:172`,`:221-222`); variantes internas `src/wal_sharded.rs` y `src/storage/wal.rs`. ADR-0002 registra CRC32C + auto-healing con truncado de cola corrupta.

`[HECHO]` Las mutaciones se escriben al WAL antes de aplicarse al backend KV; el backend por defecto es Fjall (ADR-0020) con RocksDB opcional tras la feature `rocksdb` (`Cargo.toml:160-161`; `src/backend.rs`, `src/backends/`). `[INFERENCIA]` El acuse durable efectivo depende del modo de fsync configurado; ADR-0038 registra fsync-batching opt-in como *spec propuesta*, no como comportamiento por defecto.

`[HECHO]` Existe `wal_shipping` para réplica asíncrona del WAL tras la feature `wal-shipping` (`src/lib.rs:178-180`, `src/wal_shipping.rs`) y endpoints de snapshots en HTTP (`docs/api/HTTP_API.md` §snapshots). `[HECHO]` El formato on-disk y sus versiones (WAL, VantaFile v3, imagen HNSW, registro de claves) se documentan en [STORAGE_FORMATS](../../api/STORAGE_FORMATS.md); la integridad/cadena hash del WAL en [WAL_INTEGRITY](../../api/WAL_INTEGRITY.md).

## Recuperación

`[HECHO]` Hay tests de recuperación registrados: `tests/storage/wal_resilience.rs`, `tests/durability_recovery.rs`, `tests/index_reconstruction.rs`, `tests/text_index_recovery.rs`, `tests/storage/crash_injection.rs`, `tests/fjall_cold_copy_restore.rs` y el harness de caos `tests/storage/chaos_integrity.rs` (feature `failpoints`) — todos en `Cargo.toml` secciones `[[test]]`. `[INFERENCIA]` Esas suites están diseñadas para verificar la recuperación; su resultado no se ejecutó en esta sesión — las garantías textuales son las de DURABILITY_GUARANTEES.

## Cifrado en reposo

`[HECHO]` Tras `encryption` (opt-in), AES-256-GCM cubre cuatro rutas de escritura — WAL (v4), segmentos VantaFile (v3), KV del backend (envelope por valor) e imagen HNSW (envelope de fichero) — y el modo se decide por los bytes en disco, no por la config (doc de módulo, `src/lib.rs:54-64`; ADR-0057). Los límites declarados (qué queda en claro, qué se pierde, registro de claves) están en `.github/SECURITY.md`.

## Mantenimiento

`[HECHO]` Operaciones expuestas: `flush`, `compact_wal`, `compact_layout`, `rebuild_index`, `reindex_hnsw_from_text`, `repair_text_index`, `audit_text_index`/`audit_text_index_deep` en los cuatro bindings ([BINDINGS_NAMESPACES](../../api/BINDINGS_NAMESPACES.md)) y como endpoints HTTP `POST /api/v2/maintenance/{flushes,compactions,index-rebuilds}` y `DELETE /api/v2/maintenance/expired-records` ([HTTP_API](../../api/HTTP_API.md)).

## Límites

`[HECHO]` `put` individual, batch y transacción tienen contratos distintos. Límite de esta página: no se afirma la atomicidad exacta de `commit_batch_locked`/`apply_insert` — el contrato vive en `src/storage/engine/{ops,txn}.rs` y no se audita aquí. `[INFERENCIA]` Exportar JSONL (`export_all`/`export_namespace`) es backup lógico, no físico: la copia física consistente exige cerrar/quiescer el escritor o usar snapshots/checkpoints del backend (ver [persistence](../user/persistence.md)).

## Fuentes

`src/wal.rs` · `src/storage/engine/maintenance.rs` · `src/storage/vfile*.rs` · `src/wal_shipping.rs` · `src/lib.rs:54-64` · ADR-0002/0020/0022/0038/0057 · [DURABILITY_GUARANTEES](../../user/operations/DURABILITY_GUARANTEES.md) · [WAL_INTEGRITY](../../api/WAL_INTEGRITY.md) · [STORAGE_FORMATS](../../api/STORAGE_FORMATS.md) · [BACKUP_RESTORE](../../user/operations/BACKUP_RESTORE.md)

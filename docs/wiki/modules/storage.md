---
title: "Módulo: storage (`storage`, `wal`, backends)"
kind: concept
status: active
description: "Motor de almacenamiento, WAL, backends KV (Fjall/RocksDB/memoria), VantaFile y versionado on-disk: estructura y fuentes."
---

# Módulo: storage (`storage`, `wal`, backends)

Responsabilidad: persistencia física — WAL, segmentos VantaFile, backend KV y recuperación.

## Estructura

`[HECHO]` `src/storage/` incluye `engine/` (`mod`, `init`, `insert`, `delete`, `get`, `ops`, `txn`, `maintenance`, `cache`, `partition`, `stats` + `tests/`), `vfile.rs`, `vfile_mmap.rs`, `vfile_crypto.rs`, `ops.rs`, `archive.rs`, `lock.rs`, `wal.rs`, `mod.rs`. `[HECHO]` WAL: `src/wal.rs` público (`src/lib.rs:172`), `src/wal_sharded.rs` interno, `src/storage/wal.rs`, y `src/wal_shipping.rs` tras feature (`src/lib.rs:179-180`). `[HECHO]` Backends: `src/backend.rs` + `src/backends/{mod,registry,fjall_backend,rocksdb_backend,in_memory,kv_crypto}.rs` (`pub(crate)`, `src/lib.rs:73-74`); `BackendKind` público (`src/lib.rs:218`); LSM interno `src/lsm.rs`; warm-up `src/cache_warmer.rs` (`src/lib.rs:77`).

`[HECHO]` Tipos públicos de storage: `VFILE_VERSION` (`src/lib.rs:217`), `MergeReport`, `PipelineMode`, `PipelineReport`, `VacuumReport` (`src/lib.rs:218`). `[HECHO]` Formato binario común: `src/binary_header.rs` (`Header`/`VantaHeader`, `src/lib.rs:190-192`); versionado de schema `src/schema.rs`; metadata columnar JSON-shredded `src/shred/`; export Arrow `src/columnar.rs` (feature `arrow`); ingestión asíncrona `src/ingestion.rs` (feature `async-ingestion`).

## Contratos y límites

`[HECHO]` Single-writer por fichero lock (`src/storage/lock.rs`; tests `multi_process_lock`, `file_locking_stress`); la apertura/recuperación del engine es [`src/storage/engine/init.rs:24`](../../../src/storage/engine/init.rs#L24) y el WAL público vive en [`src/wal.rs`](../../../src/wal.rs) (`WalWriter` en `:648`). `[HECHO]` El modo cifrado se decide por los bytes en disco, no por config (`src/lib.rs:54-64`). Límite de esta página: no se afirma atomicidad exacta de `commit_batch_locked`/`apply_insert`/`drain_hnsw_batch_locked` — el contrato vive en `src/storage/engine/{ops,txn,insert}.rs` y no se audita aquí.

## Tests existentes

`[HECHO]` `storage`, `mutations`, `core_invariants`, `tombstone_ann_vstore`, `backend_tests`, `wal_resilience`, `mmap_index`, `antilocality_layout`, `chaos_integrity`, `crash_injection`, `gc`, `property_durability` (`Cargo.toml [[test]]`).

## Fuentes

`src/storage/` · `src/wal*.rs` · `src/backends/` · `src/lsm.rs` · [STORAGE_FORMATS](../../api/STORAGE_FORMATS.md) · [STORAGE-TIERS](../../dev/architecture/STORAGE-TIERS.md) · [STORAGE_VERSIONING](../../dev/architecture/STORAGE_VERSIONING.md) · ADR-0020/0023/0049

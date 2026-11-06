---
title: "Persistencia y backups"
kind: howto
status: active
description: "Modos de persistencia, backups lógicos vs físicos y recuperación: qué garantiza el motor y dónde están los límites."
---

# Persistencia y backups

## Modos

`[HECHO]` Persistente en disco (ruta en `Config.storage_path`/`backend_kind`), en memoria (`":memory:"` + backend `InMemory`, ejemplo en el docstring de [`Config` en `src/config.rs:709-716`](../../../src/config.rs#L709-L716)), read-only (`read_only`), y mmap (`mmap_hnsw`/`force_mmap`/`prefetch_mode`, `src/config.rs:761-773`). En WASM la persistencia depende del runtime: OPFS/IDB ([WASM_PERSISTENCE](../../api/WASM_PERSISTENCE.md)).

## Backup

`[HECHO]` Lógico: `export_namespace`/`export_all`/`export_namespace_filtered` (JSONL v2, [MEMORY_INTERCHANGE_FORMAT](../../api/MEMORY_INTERCHANGE_FORMAT.md)); HTTP `POST /api/v2/export`/`/import`. `[HECHO]` Físico: snapshots (`GET|POST /api/v2/snapshots`) y checkpoints del backend. `[INFERENCIA]` Una copia de directorio con escritor activo no garantiza consistencia aunque se haga `flush()` antes: la copia física exige cerrar/quiescer el escritor y copiar el conjunto completo según [BACKUP_POLICY](../../user/operations/BACKUP_POLICY.md) y [BACKUP_RESTORE](../../user/operations/BACKUP_RESTORE.md).

## Recuperación y formatos

`[HECHO]` WAL con CRC32C + auto-healing (ADR-0002); versiones on-disk en [STORAGE_FORMATS](../../api/STORAGE_FORMATS.md); garantías en [DURABILITY_GUARANTEES](../../user/operations/DURABILITY_GUARANTEES.md) y disaster recovery en [DISASTER_RECOVERY_RUNBOOK](../../user/operations/DISASTER_RECOVERY_RUNBOOK.md). `[INFERENCIA]` Export JSONL no restaura índices físicos — tras import, considerar `rebuild_index`/`repair_text_index`.

## Fuentes

[persistence architecture](../architecture/durability.md) · [STORAGE_FORMATS](../../api/STORAGE_FORMATS.md) · [WAL_INTEGRITY](../../api/WAL_INTEGRITY.md) · [UPGRADE](../../user/operations/UPGRADE.md)

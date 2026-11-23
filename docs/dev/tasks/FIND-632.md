---
title: "FIND-632 — Honrar `read_only` en el backend Fjall (ADR-0001)"
kind: task
---

# FIND-632 — Honrar `read_only` en el backend Fjall (ADR-0001)

- **Estado:** ✅ COMPLETED
- **Plan:** `docs/dev/plans/2026-11-07-consolidacion-tecnica-pre-v0100.md` — Task 49
- **Ruta:** vanta-engine (revisión vanta-audit, tier adversarial: `src/storage/**`)
- **Contrato:** `cargo nextest run --profile audit -p vantadb --features encryption --locked --build-jobs 2 -E "test(read_only)"`
- **SDP:** `systematic-debugging`, `test-driven-development`, `source-driven-development` (API Fjall verificada contra `fjall-3.1.12` en registry local)
- **Decisión owner:** opción E — barrera LÓGICA honesta (no fallo cerrado general)

## Spec

| Decisión | Valor | Justificación |
|---|---|---|
| Semántica RO en Fjall | Barrera lógica: sin escrituras del motor/registry/cifrado; solo metadatos Fjall de allowlist | fjall 3.1.12 no tiene modo RO (fjall#313); hechos verificados por el owner |
| Errores de apertura | `fjall::Error::Locked` → `Error::DatabaseBusy`; `Io(PermissionDenied|ReadOnlyFilesystem)` en `read_only` → `Error::Io` con mensaje accionable | variantes existentes de `src/error.rs:176,329`; ninguna otra encaja |
| Escrituras backend en RO | `Error::Validation{field:"read_only"}` (misma variante que `guard_write_allowed`, stats.rs:16-24) | paridad de contrato con el engine |
| Allowlist | Medida empíricamente: snapshot (ruta, tamaño, sha256) antes/después de apertura RO | orden del owner: medir primero, decidir después |
| Helper compartido | `src/storage/engine/tests/read_only_files.rs` (snapshot + allowlist); consumido por `encryption.rs` y `read_only.rs` | prohibido duplicar |

## Impacto mapeado (Regla 0)

- **Leídos completos:** `src/backends/fjall_backend.rs`, `src/storage/engine/tests/encryption.rs`, `src/storage/engine/tests/mod.rs`, `src/storage/engine/stats.rs` (guard), `src/storage/engine/init.rs` (init_storage, crypto wiring RO), `src/error.rs` (variantes Io/DatabaseBusy/Validation), `fjall-3.1.12/src/{locked_file.rs,db.rs,error.rs,journal/writer.rs}`
- **Referencias hacia dentro:** `FjallBackend::open` ← `backends/registry.rs:33`, `backends/kv_crypto.rs` (tests); `files_under`/`is_backend_marker` ← encryption.rs:385-417
- **Referencias entrantes:** `Error::DatabaseBusy` ya usado por `.vanta.lock` (init.rs:356); `Error::Validation{field:"read_only"}` usado por engine/vfile — paridad
- **Veredicto:** cambio localizado en el adapter Fjall + tests; no toca trait `StorageBackend` ni formatos on-disk

## Steps

- [x] S1 — Medición: snapshot before/after de apertura RO sobre Fjall → clasificar mutaciones
- [x] S2 — Helper `read_only_files.rs` (snapshot sha256 + allowlist citada) registrado en `mod.rs`
- [x] S3 — `FjallBackend`: flag `read_only`, `map_open_error` (Locked→DatabaseBusy, PermissionDenied→Io accionable), `ensure_writable` en put/delete/write_batch/flush/compact
- [x] S4 — Tests: `fjall_read_only_open_never_mutates_files`, lock-taken, not-writable (unix, skip root), backend-write-rejected; sustituir exclusión general en `read_only_open_never_writes_the_registry` por la allowlist
- [x] S5 — RED demostrado (mutación inyectada → fallo → revertida) + contrato verde
- [x] S6 — Enmienda ADR-0001 (semántica por backend, fecha 2026-11-09, fjall#313, FIND-648, nota MCP-35) + markdownlint
- [x] S7 — Review P2-01 (vanta-audit, adversarial) + commits

## Invariantes

- `read_only` sigue abriendo bases Fjall (no fail-closed general)
- Ningún fichero de datos (SST/segmentos/blobs/registry/VantaFile/WAL/índice) entra en la allowlist
- Escrituras RO → error tipado, nunca pánico
- Commits locales con pathspec; prohibido push/amend/rebase/stash

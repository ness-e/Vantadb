---
title: Espejar recursivamente keyspaces/ de Fjall en create_snapshot y restaurar backend/ en snapshot_restore
kind: task
status: completed
description: "cargo nextest run --profile audit -p vantadb --features failpoints -E \"test(snapshot)\" --locked --build-jobs 2 (+ suite de certificación snapshotcertification en Windows y Linux)"
id: FIND-319
type: fix
plan: docs/dev/plans/2026-11-07-consolidacion-tecnica-pre-v0100.md (Task 18)
---

# FIND-319: Espejar recursivamente `keyspaces/` de Fjall en `create_snapshot` y restaurar `backend/` en `snapshot_restore` (absorbe FIND-287)

## Metadata
- **Plan file:** `docs/dev/plans/2026-11-07-consolidacion-tecnica-pre-v0100.md`
- **Fuente:** `docs/dev/Backlog.md` (L280 `FIND-287` + L310 `FIND-319`) / Plan Task 18
- **Esfuerzo:** 🟡 4h
- **Prioridad:** 🔴
- **Tipo:** Rust (`src/storage/engine/mod.rs`, `src/storage/engine/init.rs`, `src/backends/rocksdb_backend.rs`, `vanta-memory/src/utils/backup.rs`)
- **Turns estimados:** 40
- **Creado:** 2026-11-16T15:00
- **last-synced:** 2026-11-18
- **Estado:** ✅ COMPLETED
- **Commits:** `08b0cf03b`, `2e0e3016a`, `f91d44a73` (+ CI/certificación `15e5f6cc8`, `0b082c35f`, `4929d8d9a`)
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps de ejecución restantes

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `Embedded::create_snapshot`, `Embedded::restore_snapshot`, `vantadb-mcp` (`snapshot_create`, `snapshot_restore`), `vanta-memory::utils::backup` |
| Callees | `StorageEngine::create_snapshot` (`src/storage/engine/mod.rs:711-1013`), `StorageEngine::snapshot_restore` (`:1882-2124`), `StorageEngine::schedule_pending_restore` (`:1707-1880`), `RocksDbBackend` (`src/backends/rocksdb_backend.rs:137-156`) |
| Implicaciones | Garantiza que los snapshots incluyan recursivamente `keyspaces/` (SSTs compactados de Fjall) o el checkpoint nativo de RocksDB, y que `snapshot_restore` restaure atómicamente tanto `data/` como `backend/` con staging y rollback dual. Absorbe `FIND-287`. |

## Impacto mapeado (Regla 0)
- **Archivos leídos (completos):** `src/storage/engine/mod.rs`, `src/storage/engine/init.rs`, `src/backends/rocksdb_backend.rs`, `vanta-memory/src/utils/backup.rs`, `tests/core/snapshot_certification.rs`.
- **Archivos referenciados hacia dentro:** `utils::fs::write_atomic_unique`, `BackendKind`, `KeyRegistry`.
- **Archivos que referencian a los editados:** `vantadb-mcp/src/handlers/tools.rs`, `vanta-memory/tests/backup_snapshot.rs`, `tests/core/snapshot_certification.rs`.
- **Veredicto impacto:** alto (rediseña el formato y protocolo de backup/restore físico del motor manteniendo retrocompatibilidad con snapshots legacy `data/`-only).

## Contrato
`cargo nextest run --profile audit -p vantadb --features failpoints -E "test(snapshot)" --locked --build-jobs 2` (+ suite de certificación `snapshot_certification` en Windows y Linux).

## Fase 1 — Evidencia de Debugging (GATE — tipo Bug)
- **Repro:** `src/storage/engine/mod.rs:643` hacía `if path.is_file()` ignorando el subdirectorio `keyspaces/` de Fjall (donde viven los SSTs tras compactación), y `snapshot_restore` (`:926-975`) solo restauraba `data_dir` ignorando `<snap_dir>/backend`, dejando KV/BM25/metadatos desincronizados respecto a los segmentos vectoriales (`FIND-287`).
- **Hipótesis:** Espejar recursivamente `keyspaces/` con allowlist estricta (copiando mutables `current`/`v*`/`.jnl` activos y hard-linkeando solo `tables/`/`blobs/` y `.jnl` sellados, verificando `backend_currents_match` con 3 reintentos) y restaurar mediante staging dual (`data.pre_restore_*` + `backend.pre_restore_*`) con rollback E/S dual y apertura fail-closed ante residuos cierra la pérdida de datos y la desincronización.
- **1 variable controlada:** Espejado recursivo + staging/restore dual de `data/` y `backend/`.
- **Test RED:** Tras forzar compactación Fjall y mutar/borrar registros post-snapshot, `snapshot_restore` perdía los SSTs o conservaba las mutaciones posteriores; pasa a GREEN con KV + BM25 + HNSW exactos al instante del snapshot.

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** Nunca incluir `.vanta.lock` en el snapshot; nunca hard-linkear archivos mutables (`current`, `v*`, `.jnl` activo); validar `snapshot.format` y `backend.manifest` con `deny_unknown_fields`; en MCP (`schedule_pending_restore`) persistir `.restore_pending` atómicamente vía `write_atomic_unique` bajo `.restore_pending.lock` y aplicarlo en el siguiente `open` writable; en RocksDB incluir el descriptor de `sparse_index` CF (`src/backends/rocksdb_backend.rs:137-156`).
- **Comandos de verificación:** `cargo nextest run --profile audit --cargo-profile ci -p vantadb --locked --build-jobs 2 --test-threads 1 --ignore-default-filter --test snapshot_certification --features failpoints,encryption -E "test(snapshot) or test(pending_restore) or test(schedule)"`
- **Deuda pendiente:** La barrera compartida frente a escritores concurrentes se separó en `FIND-651` (Task 63) por decisión del owner (Opción B).

## Deuda técnica (Regla 6 — MUST)
**Saldo neto de deuda por PR:** ≤ 0 (cierra `FIND-319` y `FIND-287`, añade descriptor faltante `sparse_index` en RocksDB y lane de certificación Linux `snapshot-certification.yml`).

## Definition of Done (contrato multi-nivel — P2-08)
| Nivel | Gate |
|-------|------|
| **Task** | Contrato local + certificación `snapshot_certification` 41/41 sin encryption, 43/43 con encryption, 1/1 RocksDB nativo, 7/7 fases en CI Linux (runs `44183458267` y `44200732891`) |
| **Commit** | `08b0cf03b`, `2e0e3016a`, `f91d44a73` |
| **Release** | Snapshots físicos y restore coherentes en `v0.10.0` |

## Herramientas necesarias
- cargo-mcp / nextest

**Skills cargadas (SDP):** `systematic-debugging`, `test-driven-development`, `doubt-driven-development`.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03
| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 0 |
| % completado | 100% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)
- [x] **SECURITY** — Validación estricta de manifiestos (`deny_unknown_fields`), rechazo de path traversal y apertura fail-closed si quedan directorios `.pre_restore_*` de un fallo previo.
- [x] **PERFORMANCE** — Conserva hard-links $O(1)$ para tablas SST inmutables (`tables/`, `blobs/`) y journals sellados en Unix; `[profile.test.package.vantadb] opt-level = 2` evita timeouts espurios en HNSW durante tests de certificación.

## Steps
- [x] Step 1: Implementar espejo recursivo de `keyspaces/` y checkpoint nativo de RocksDB en `src/storage/engine/mod.rs:711-1013` + `src/backends/rocksdb_backend.rs:137-156`.
- [x] Step 2: Implementar staging dual (`data.pre_restore_*` + `backend.pre_restore_*`), `sync_restored_tree`, rollback E/S dual y compatibilidad legacy en `src/storage/engine/mod.rs:1882-2124`.
- [x] Step 3: Implementar `.restore_pending` diferido atómico (`mod.rs:1707-1880`, `init.rs:45,58-63`) para MCP.
- [x] Step 4: Certificar localmente (Windows) y en CI Linux (`snapshot-certification.yml`, 7/7 fases verdes).

## Dependencias
- Wave 0 completa.

## Review (GATE — agente distinto, P2-01)
- **Revisor:** `vanta-review` (`e7c1b42e`, fresh-context) + re-auditoría `audit-w1-durability` (#20).
- **Enfoque:** Correcto. El espejo recursivo con allowlist y `backend_currents_match`, más el staging dual con rollback y apertura fail-closed, cierran tanto `FIND-319` como `FIND-287`.
- **Cómo se probó:** Certificación local con `--cargo-profile ci` (41/41 sin encryption, 43/43 con encryption, 1/1 RocksDB nativo) + CI Linux run `44183458267` y `44200732891` (7/7 fases: 34/34, 39/39, 40/40, 42/42, rocksdb 1/1, vanta-memory 7/7, MCP 4/4).
- **Checklist anti-hábitos tóxicos:**
  - [x] No inventar salidas de comandos/herramientas que no se ejecutaron.
  - [x] No saltarse la clarificación por "ya sé qué quiere".
  - [x] No declarar done sin verificar contra los acceptance criteria.
  - [x] No ignorar fallos ni reportar "todo OK" cuando hubo fallo parcial.
  - [x] No hacer un solo intento de búsqueda y darlo por saturado.
  - [x] No copiar sin citar ni presentar supuestos propios como evidencia.
  - [x] No reintentar en bucle sin diagnóstico.
  - [x] No dejar huérfanos los pasos: cada paso conectado al objetivo.
  - [x] No degradar el chequeo de errores en paths de dinero/seguridad.
  - [x] No gastar presupuesto infinito; paradas explícitas.
  - [x] Verificar cobertura SDP (v3).
- **Veredicto:** ✅ approve

## Notas
- Creado y sincronizado en Pasada 1 de auditoría (`2026-11-18`).

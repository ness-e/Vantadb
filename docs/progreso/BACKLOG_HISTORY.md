---
title: "Backlog History — Items Removed & Migrated"
type: tracking
status: active
tags: [vantadb, backlog, history]
last_reviewed: 2026-08-03
aliases: []
---

# Backlog History — Items Removed & Migrated

> Historial narrativo de los items que salieron del catálogo activo (`docs/Backlog.md`): completados, removidos por stale, resueltos o cerrados WONTFIX. Este archivo documenta el *por qué*; el catálogo solo lista lo que queda por hacer.

## Items removidos totales

**71+ items removidos:** ~25 originales + 6 P0 stale + 9 P1 resueltos + 24 P2 stale + 7 P3 stale + 10 P4 completados + 7 P9 completados + 11 P10 completados + 1 P7 completado + 24 crates de integración nunca implementados.

## Por fase

### P0 — Release Blockers (7 removidos + 1 WONTFIX)

- `DEVOPS-10` — deferido
- `DEVOPS-12` — PyPI signing
- `DEVOPS-14` — ✅
- `NUEVO-09` — ✅
- `NUEVO-10` — ✅
- `DEVOPS-15` — ❌ **WONTFIX** (remover `cli, memmap2, fs2, sysinfo` rompe UX "it just works"; las 7 features mantienen experiencia completa)
- `META-001` — queda como único P0 activo en su momento

### P1 — Security & Critical (9 resueltos)

Todos los items P1 originales resueltos/deferidos en campañas anteriores.

### P2 — Quick Wins Técnicos (31 removidos)

`DRV-014` ✅, `DRV-028` ✅, `DRV-041` ✅, `VFY-006` ✅, `VFY-007` ✅, `REV-012` ✅, `DRV-136` ✅ + 24 stale items de la auditoría original.

> ⚠️ **Nota DRV-014:** el fix fue revertido por `cae92db3` — ver `docs/architecture/adr/DRV-014-wal-batch-tradeoff.md`. Tradeoff de performance posterior, no deuda pendiente.

### P3 — Test Coverage (14 removidos)

`DRV-013` ✅, `DRV-017` ✅, `DRV-061` ✅, `DRV-067` ✅, `DRV-073` ✅, `TEST-11` ✅, `TEST-12` ✅ + 7 stale de auditoría original.

### P4 — Engineering Health (10 completados)

`WEB-03` ✅, `WEB-04` ✅, `VFY-004` ✅, `VFY-011` ✅, `DRV-121` ✅, `DRV-122` ✅, `DRV-123` ✅, `DRV-130` ✅, `DRV-131` ✅, `DOC-20` ✅ — movidos a `docs/progreso/README.md`.

### P7 — WASM & Performance (5 removidos)

`NUEVO-11`/`NUEVO-12` (WASM IndexedDB + multi-tab coordinación — ✅ implementados), `NUEVO-14` (bundle 394KB gzip < 500KB — ✅ en WASM-04), `NUEVO-19` (`SourceDesign/` no existe), `BENCH-01` (solo mención en backlog, sin script ni dataset).

### P8 — Post-Launch & Enterprise (1 removido)

`NUEVO-20` (Dockerfile ya existe en raíz del repo — multi-stage, Rust 1.94).

### P9 — Old Docs Rescue (8 removidos a progreso)

`OLD-04` (OpenTelemetry), `OLD-07` (AutoHot/Cold tiering), `OLD-13` (Explainable ranking), `OLD-15` (Euclidean SIMD), `OLD-16` (WAL rotation 256MB), `OLD-17` (Migration guides), `OLD-18` (TEMPERATURE param), `OLD-22` (Arrow columnar export).

### P10 — Competitive Features (12 removidos a progreso)

`COMP-001` (SQ8/PQ), `COMP-002` (HNSW persist), `COMP-003` (in-filter), `COMP-004` (bitset), `COMP-005` (params), `COMP-006` (Edge Label Interning), `COMP-007` (inline u128), `COMP-010` (auto-embedding), `COMP-011` (CRUD tombstones), `COMP-015` (hybrid pipeline), `COMP-018` (Double-linked chains), `COMP-020` (RRF fusion), `COMP-030` (survival mode).

## Historial de verificación del catálogo

- **2026-07-27:** vanta-lead ejecutó 8 tareas de P5/P6/P8.
- **2026-07-28:** 5 sub-agentes explore validaron 69 items contra código real — ver `docs/audit-reports/backlog-validation-2026-07-28.md`.
- **2026-07-29:** 19 items INVESTIGACION agregados (INV-001 a INV-017) tras verificación de consolidación de 4 sub-agentes vs código real.

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

## Completados de `docs/strategy/` (2026-08-05)

Verificación de los 5 documentos de `docs/strategy/` (ROADMAP, GO_TO_MARKET, SHOW_HN_PREP, BLOG_SERIES_PLAN, REDDIT_POSTS) contra backlog + `docs/progreso/README.md` + git history. Items que strategy lista como tarea y **ya están completados** — registrados aquí porque no tenían fila activa en el backlog:

### Release & packaging (Fase 0 roadmap)

- `REL-01` — bump a v0.2.0 → **superado: repo en v0.5.0** (`vantadb-ts/package.json`, SHOW_HN_PREP). Commit `0b3b8353` (fase 4 release engineering).
- `REL-02` — publicar `vantadb-ts` en npm → **✅ completado** — commit explícito `cb9589db release(REL-02): publicar vantadb-ts en npm`; `vantadb@0.5.0` en package.json.
- `DEVOPS-05` — pipeline CI a PyPI para adapters → **✅ completado** — task file borrado `DEVOPS-05.md` + commits `2ac6b033`, `1e986b68`.

### Integraciones (GTM Tier 1-2)

- `INT-01` — LangChain adapter → **✅ completado** — task file borrado `INT-01.md`; adapter en `integrations/langchain-vantadb`.
- `INT-02` — LlamaIndex adapter → **✅ completado** — task file borrado `INT-02.md`.
- `INT-03→09` — 7 adapters Python puros (Mem0, CrewAI, DSPy, Haystack, Letta, OpenAI, Ollama) → **✅ completados** — commit `60c7b3e7`.
- `TSK-90` (CrewAI), `TSK-91` (DSPy) → **✅ completados** — commit `23a40320` (7 framework integration adapter crates).

### Web / Marketing / Legal

- `WEB-02` — corregir claims falsos del landing (50x→40x, SQL, auto-embeddings, cloud) → **✅ completado** — commit `e84e3c40 fix(web): correct landing page claims`; `vanta-data.ts` hoy muestra 2.80x/2.18x/2.14x.
- `MKT-13` — WASM demo funcional → **✅ completado** — `/demo` existe (`web/src/app/demo`) + commit `ee310422 feat(WEB-001): run real WASM in playground`.
- `MKT-17` — página de comparación competitiva → **✅ completado** — commit `e898b47b` (Fase 1 cierres).
- `LEG-01` — trademark → **✅ cerrado** — commit `e898b47b` (cierre en backlog-validation).

### SDK / Platform

- `TSK-61` — feature gates + build profiles → **✅ completado** — `docs/progreso/README.md:123` (✅).
- `TSK-68` — Python SDK latency <20ms / zero-copy NumPy → **✅ completado** — commit `0c1962b2 feat(python): zero-copy NumPy FFI via buffer protocol (TSK-68)`.
- `TSK-100` — Homebrew formula macOS → **✅ completado** — `Formula/vantadb.rb` existe + task file `DEVOPS-HOMEBREW.md` + `docs/progreso/README.md:1409`.
- `TSK-101` — ARM64 Linux wheels → **✅ completado** — `docs/progreso/README.md:1407` + `release-binaries-63.yml` (aarch64-apple-darwin).

### Enterprise / Governance (Q1-Q2 2027 GTM)

- `TSK-72` — AES-256 at-rest encryption → **✅ completado** — commit `b78a9b5a feat: Phase 5 complete — governance, encryption, WAL shipping, PITR`.
- `TSK-107b` — audit logging → **✅ completado** — task file archivado `tasks/complete/TSK-107b.md` + commit `cc095774`.
- `BIZ-02` — async WAL shipping → **✅ completado** — commit `b78a9b5a` (Phase 5, WAL shipping + PITR).
- `BIZ-03` — pricing page → **✅ completado** — commit `c73e8a4a docs: move BIZ-03, DOC-11 and DOC-12 to progress log`.

### Seguridad

- `SEC-13` — CSP + HSTS + nonce → **✅ completado** — task file borrado `SEC-13.md`; ARCHIVO_HISTORICO P1 lo lista cerrado.
- `SEC-14` — cargo-deny / licencias → **✅ RESUELTO** — `docs/progreso/README.md:2763` (`cargo deny check` pasa en CI).

### Engine / Engineering Health (roadmap Sem 5)

- `DRV-001` — split `search.rs` god file → **✅ completado** — task file archivado `tasks/complete/DRV-001.md`.
- `DRV-002` — `put_batch` duplica `put()` DRY → **✅ completado** — task file archivado `tasks/complete/DRV-002.md`.
- `DRV-003` — `purge_expired` O(n) index rebuilds → **✅ completado** — commit `d9e1caf9 perf(DRV-003): replace O(n) index rebuild with selective removal`.

### Nota — pendientes de strategy SIN evidencia de completado (no registrar como ✅)

`CLD-01/02/04` (cloud beta, pitch deck, case study), `OLD-001`, `VFY-008` (WAL fsync batching), `DRV-115` (MSVC linker), `DRV-117` (advisory ignores), `DRV-119` (ACID 0) — aparecen solo como **menciones** en ROADMAP/GO_TO_MARKET/backlog-guide; no hay task file, commit de fix, ni fila de progreso que demuestre completado. Siguen pendientes o sin trackear.

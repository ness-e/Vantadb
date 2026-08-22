---
title: "Avance — Meta / Proceso"
type: meta
status: active
tags: [vantadb, avance, meta, proceso, housekeeping]
last_reviewed: 2026-08-22
aliases: []
---

# Avance — Meta / Proceso

> Cambios de proceso, housekeeping del backlog, decisiones de documentación y mejoras de pipeline. IDs originales conservados.

## Contrato del mirror `activo/`

Los archivos de `docs/avance/activo/` **se actualizan al cierre de cada campaña** (no daily); los dominios del mirror = **crates activos** del workspace. Un crate nuevo ⇒ archivo de dominio nuevo en el mismo cierre. Verificación: muestreo cruzado `git log --grep MEM-` ↔ archivos de dominio (GOV-D1, 2026-08-22).

## Backlog housekeeping

### 2026-07-26 — Backlog Cleanup P0–P4, P7, P9–P10 (53 items → progreso)
- **Objetivo:** Limpiar backlog verificando cada item ✅ contra código real.
- **Resumen:** P0 6 stale removidos + 1 WONTFIX (DEVOPS-15); P1 fase completa cerrada (9); P2 7 ✅ + 24 stale; P3 7 ✅ + 7 stale; P4 10 ✅; P7 2 ✅; P9 7 ✅; P10 12 ✅.
- **Impacto:** Backlog ~120 → ~65 items activos. 5 fases cerradas (P1–P4, P7).
- **Verificación:** cada item verificado contra código real antes de mover.

### 2026-07-07 — Reorganización Masiva del Backlog (24 eliminaciones, 21 adiciones, 11 prioridades)
- Fuente: `docs/research/VantaDB_ANALISIS_COMPLETO.md`.
- 24 items eliminados (Cloud entero, optimizaciones prematuras, SOC2/HIPAA, WAL shipping, PITR, Semantic Kernel, visual regression, duplicados).
- 11 re-priorizados; 21 nuevos agregados. Backlog 79 → **65 items activos**.
- Documentación: `docs/progreso/backlog-2026-07-07.md`.

### Housekeeping sin ID
- **Backlog audit:** 4 discrepancias corregidas (TSK-94/67/80/82) ✅
- **Clippy/fmt fixes:** 3 unused vars, 18 archivos formateados, conditional imports ✅
- **Fix `with_writer`:** MakeWriter closure en vez de `Box<dyn Write>` ✅
- **`vantadb-mcp` ttl_ms:** `planner.rs:369` `expires_at_ms: Some(0)` ✅

### P2 Backlog Housekeeping: DRV-041, VFY-006, VFY-007 (2026-07-26)
- Document-only: 3 tareas triageadas como ya corregidas (ver `historial/no-ops.md`). Backlog P2 counter 15→12.

### ECO-001: Eliminar hooks muertos de Claude Code
- **Fecha:** 2026-07-28
- **Resultado:** ✅ Hooks muertos de Claude Code eliminados. Detalle en snapshot-2026-08-07.

## Proceso / Pipeline

### 2026-08-22 — GOV-D1: catch-up del mirror + dominios faltantes
- El mirror `activo/` estaba congelado al 20/08 y sin los crates creados después: `vanta-proxy`, context engine, y las campañas P29/P30/P31 sin registrar (MEM-43 `a0bcb112` / MEM-44 `785db22c` ausentes).
- **Fix:** 3 archivos de dominio nuevos por campaña (no commit-por-commit): `activo/vanta-memory.md` (P27 F1-F4 + P29 + P31), `activo/vanta-proxy.md` (P30 F6-F7: MEM-25..33), `activo/context-engine.md` (MEM-22/23/24/37 + wiring `a0bcb112`). Contrato del mirror actualizado (sección arriba).

### 2026-07-24 — auto-progreso + auto-commit en /pipeline task
- **Proceso:** `skill progreso` (Trigger 1) y el commit automático no se ejecutaban al final del pipeline MODO TAREA.
- **Fix:** `pipeline.md` pasos 6-7 después del Review: `skill progreso` + auto-commit. Aplica a MODO TAREA y MODO RUN. Decisión en campaign_memory como policy.

### 2026-08-07 — Migración de `docs/progreso/` → `docs/avance/`
- Reorganización del README único en árbol por dominio (este índice).
- 0 info perdida: snapshot completo 2026-08-03 en `historial/snapshot-2026-08-03.md`; fuentes originales conservadas hasta validar equivalencia de IDs.
- **Re-sync post-validación:** detectadas entradas del 04-08..07-08 ausentes en los archivos de dominio → `snapshot-2026-08-07.md` (copia íntegra del README actual) + `activo/desktop.md` (DESKTOP-01..11) + entradas añadidas: NUEVO-17, COMP-021, COMP-029, ENT-04 (core-engine/bindings), CI-01, REVIEW-02/03/05 (ci-cd), AUDIT-01/02, P13 (seguridad), ECO-001 (meta).

## Documentación

### Week 2026-07-01 — Documentation overhaul & Code Hardening
- Re-creado Obsidian graph color groups; plugins (Dataview, Linter, Calendar).
- 58 wikilinks rotos reparados (10 archivos).
- Fix syntax error `cli_server.rs` (//! + duplicate use).
- Clippy `if_same_then_else` en `src/sdk/search.rs:307`.
- `cargo fmt` en 22 archivos (1349 líneas).
- Windows pagefile os error 1455 → compilación lib tests individual. 440/440 tests pasan.

### Week 2026-06-19 — Comprehensive Audit (AUD-01→44)
- 44 hallazgos resueltos en un día con agentes paralelos (3 por batch, 15 batches).
- 7 críticos, 14 medios, 23 bajos. ~45 archivos modificados.
- CVEs resueltos: RUSTSEC-2025-0141 (bincode), RUSTSEC-2026-0176/0177 (pyo3).
- PHASE 3 exit criteria actualizados: todos AUDs resueltos ✅.

### 2026-08-07 — Auditoría y reclasificación de docs (C-*)
- **ECO-002:** Contradicción `--no-verify` en AGENTS.md (Regla 1 vs Regla 7) → Regla B eliminada; queda solo prohibición en línea 791. `.antigravity/AGENTS.md` idéntico.
- Reclasificación de archivos: `vectara-competitive-research` y `meta-001-root-cause-analysis` → `docs/Investigaciones/`; `backlog-validation`, `progreso-readme-part1/2/3`, `progreso-sistema` → `docs/reviews/archive/`.

## Skills ecosystem

### S1: Consolidar skills duplicadas (~40% waste, ~80 a remover de ~190)
- Duplicados identificados: `minimalist-skill`=`minimalist-ui`, `redesign-skill`=`redesign-existing-projects`, `stitch-skill`=`stitch-design-taste`, `soft-skill`=`high-end-visual-design`, `threejs` local=`threejs-*` global, `prisma` basic=`prisma-expert`, `browser-use`=`agent-browser`+Playwright MCP, `gpt-taste`=`impeccable`+`design-taste-frontend`.
- Eliminar: Venice.ai suite (5 stubs), Fal.ai stub suite (10 de 14), `imagen` (5th image gen), `design-taste-frontend-v1` (migrar a v2).
- Referencia: `docs/reviews/FINAL-REVIEW.md` (Core 50).

### S2: Empty skill directories
- 9 dirs en `.claude/skills/` sin `SKILL.md`: cargo-nextest, github-repo-management, m10-performance, markdown-documentation, python-packaging, rust-ffi, rust-write-tests, test-reporting, vector-database-engineer. Poblar o limpiar.

## Fuentes
- `docs/progreso/ARCHIVO_HISTORICO.md` §Meta/Proceso
- `docs/progreso/README.md` §Housekeeping y C-*
- `docs/progreso/bitacora.md` §SKILLS ECOSYSTEM
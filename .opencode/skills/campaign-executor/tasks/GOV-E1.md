# Task: GOV-E1 — Documento de propuestas de limpieza de artefactos

- **Plan:** `docs/plans/2026-08-22-doc-governance-plan.md` (NO editable)
- **Appetite:** max 45min · 🟢 · Prioridad 🟢
- **Estado:** ⏳ IN PROGRESS
- **Restricciones del orquestador:** PROHIBIDO git commit, PROHIBIDO borrar/mover archivos existentes. Solo se crea 1 documento nuevo.

## Steps

1. ✅ DISCOVERY — Regla 0 sobre los 7 candidatos (verificado con Test-Path, git ls-files, rg)
2. ✅ ACT — escribir `docs/reviews/propuesta-limpieza-artefactos-2026-08-22.md`
3. ✅ VERIFY — markdownlint exit 0 + git status sin borrados/movimientos

## Impacto mapeado (Regla 0)

| # | Candidato | Existe | Tracked en git | Ignorado (.gitignore) | Refs entrantes |
|---|---|---|---|---|---|
| 1 | `docs/book/book/` | sí (83 files) | **NO** (0 tracked) | sí (:15) | auditoría, master-index (mención), snapshots históricos |
| 2 | `docs/examples/__pycache__/` | sí (1 .pyc) | **NO** | sí (:43) | collect_code.ps1 (exclusión genérica), auditoría |
| 3 | `docs/TDAM-VANTADB/` | sí (vacía) | **NO** | n/a | auditoría, master-index |
| 4 | `docs/benchmarks/_run_stdout.md` | sí (8KB) | **SÍ** | no | solo auditoría-documentación |
| 5 | `docs/web/DESIGN_RULES.md` (16.5KB) vs `docs/web/standards/design-rules.md` (10.5KB) | ambos tracked | SÍ | no | `docs/master-index.md:293`; menciones históricas bitácora/sesiones |
| 6 | `docs/.obsidian/` (18 items) | sí | **NO** | sí (:161) | master-index (exclusión deliberada), research RESEARCH.md |
| 7 | stubs `book/src/blog` + `case_studies` | sí (7 files) | SÍ | no | book TOC (SUMMARY.md) |

**Correcciones a la auditoría V1+II (hallazgos de esta sesión):**
- Item 1 NO está "commiteado": `git ls-files docs/book/book` = 0. Ya ignorado. Es artefacto local → acción = borrado local opcional, NO `git rm --cached`.
- Items 2/3/6 igualmente no trackeados → acciones puramente locales.
- Item 5 NO es duplicado literal: contenido distinto (tutorial español de visualización vs reglas CSS Tailwind). Propuesta ajustada: extraer único→standards, archivar.
- Item 7 RESUELTO hoy por GOV-B1 (commit 8b21733): case_studies = stubs ARCHIVED, blog = `{{#include}}` a fuentes reales.

## Context Save Point

- Doc creado: `docs/reviews/propuesta-limpieza-artefactos-2026-08-22.md`
- Verify: markdownlint-cli2 exit 0; git status pre-existente sucio (vanta-proxy ×5, MEM-50) — NO tocado por esta tarea.

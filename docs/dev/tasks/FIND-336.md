---
title: "FIND-336: Mover SPEC.md + CONSTRAINTS.md a docs/specs/ + fix refs"
kind: task
status: active
description: "Higiene raíz: SPEC y CONSTRAINTS a docs/specs con fix de refs vivas"
tags: [higiene, docs]
---

# FIND-336: Mover SPEC.md + CONSTRAINTS.md a docs/specs/ + fix refs

## Metadata
- **Plan file:** `docs/dev/plans/2026-10-15-pre-mercado-estabilizacion.md` (Task 1)
- **Creado:** 2026-10-15
- **last-synced:** 2026-10-15
- **Estado:** ✅ COMPLETED
- **Skills:** `documentation-skill` (obligatoria docs), `codebase-memory` (N/A — refs literales, grep es la herramienta correcta), `progreso`, `ponytail` (full)
- **Decisión owner (Gate D):** mover + fix todo en el mismo commit (verificado: ~100 matches, 0 markdown-links a estos archivos, check-links excluye `archive/`)

## Blast Radius
- **Entrantes (quién los cita):** ~100 matches en `docs/**` (PROXY, EXPERIMENTAL_FEATURES, VISION, DISTRIBUTION, ADR-0047, SKILLS-MANIFEST, docs/index, research, tasks, avance, reviews, CHANGELOG), `scripts/north_star_metric.py`, `src/embedding_health.rs`, `vantadb-mcp/tests/embedding_visibility.rs`, `dev-tools/floor-guard.ps1` (lógica), `.opencode/**` (repo separado — excluido).
- **Salientes (qué citan ellos):** SPEC.md autocontenido (sin refs a CONSTRAINTS); CONSTRAINTS.md autocitas + checklist interno (mudados intactos, rename puro 97%/98%).
- **Implicaciones:** `floor-guard.ps1` pineaba `CONSTRAINTS.md` en raíz por ruta (`git diff -- CONSTRAINTS.md`) — actualizado a `docs/specs/CONSTRAINTS.md` y verificado exit 0. Cero markdown-links rotos (verificado: ninguna ocurrencia `](...SPEC.md)` / `](...CONSTRAINTS.md)`).

## Alcance (Regla 0)
- **SE TOCARON (vivas + lógica + comentarios):** `docs/specs/` (nueva), `docs/api/PROXY.md`, `docs/user/operations/EXPERIMENTAL_FEATURES.md`, `docs/dev/vision/VISION.md`, `docs/dev/strategy/DISTRIBUTION.md`, `docs/dev/architecture/adr/ADR-0047-anti-stutter.md`, `SKILLS-MANIFEST.md`, `docs/index.md` (vía gen-index, no a mano), `scripts/north_star_metric.py`, `src/embedding_health.rs`, `vantadb-mcp/tests/embedding_visibility.rs`, `dev-tools/floor-guard.ps1`, plan file (esta task).
- **NO SE TOCARON (histórico/propiedad/repo ajeno):** `docs/CHANGELOG.md` (release-plz), `docs/dev/tasks/**` (actas DONE), `docs/dev/avance/**`, `docs/dev/reviews/**`, `docs/dev/plans/archive/**`, `docs/dev/research/**` (docs fechados, solo backticks, sin links), `.opencode/**` (repo separado configOpencode — follow-up con Gate H).

## Contrato
`git status --short` solo muestra moves+edits esperados + `node scripts/docs/check-links.mjs` exit 0 + `node scripts/docs/check-docs.mjs` exit 0 + `git diff --check` limpio + `cargo fmt --all -- --check` exit 0

## Herramientas
- `git mv`, `edit` (replaceAll por archivo), `node scripts/docs/*.mjs`, `cargo fmt --check`, `dev-tools/verify_changed.ps1`, `campaign_verify_cmd` si disponible

## Steps
### Step 1: Mover archivos (rename puro, sin editar contenido)
- **Archivos:** `SPEC.md`, `CONSTRAINTS.md` → `docs/specs/`
- **Acción:** `mkdir docs/specs; git mv SPEC.md docs/specs/SPEC.md; git mv CONSTRAINTS.md docs/specs/CONSTRAINTS.md`
- **Verify:** `Test-Path docs/specs/SPEC.md, docs/specs/CONSTRAINTS.md` + `git status --short` muestra `R` (renames)
- **Estado:** ✅ COMPLETED

### Step 2: Actualizar lógica floor-guard.ps1
- **Archivos:** `dev-tools/floor-guard.ps1`
- **Acción:** `CONSTRAINTS.md` → `docs/specs/CONSTRAINTS.md` (12 ocurrencias: paths de `git diff` + mensajes)
- **Verify:** `dev-tools/floor-guard.ps1` → `FLOOR GUARD PASSED (all 5 checks)`, exit 0, check [5/5] vigila la ruta nueva
- **Estado:** ✅ COMPLETED

### Step 3: Actualizar refs en docs vivas + comentarios (backtick-scoped)
- **Archivos:** PROXY.md (1), EXPERIMENTAL_FEATURES.md (3), VISION.md (1), DISTRIBUTION.md (5), ADR-0047 (1), SKILLS-MANIFEST.md (2), north_star_metric.py (1), embedding_health.rs (1), embedding_visibility.rs (1) + frontmatter `kind: concept` en los 2 movidos + plan + este task file
- **Acción:** `` `SPEC.md` `` → `` `docs/specs/SPEC.md` `` ; `CONSTRAINTS.md` → `docs/specs/CONSTRAINTS.md`
- **Verify:** `rg` residual solo en excluidos + `git diff --check` limpio + `cargo fmt` exit 0 + check-links 0 broken
- **Estado:** ✅ COMPLETED

### Step 4: Gates docs + fmt + review P2-01 + commit local
- **Archivos:** plan file (Estado + Iteraciones)
- **Acción:** check-links + check-docs + gen-index --write + fmt + `verify_changed.ps1`; review `vanta-review` (contexto distinto); commit local `docs: ... (FIND-336)`; `skill progreso` NO migra (no es fila de Backlog — es FIND de higiene; se registra avance en plan)
- **Verify:** contrato completo verde + commit existe + plan actualizado
- **Estado:** ✅ COMPLETED

## Dependencias
- Ninguna (Wave 0). Bloquea a FIND-337/338/339 solo por orden de diff limpio, no por contenido.

## Deuda / follow-up (no bloquea cierre)
- `.opencode/AGENTS.md:41` + `.opencode/references/floor-guard.md` + `.opencode/rules/*` citan `CONSTRAINTS.md` en raíz → commit separado en repo configOpencode con Gate H (`/harness`). Registrar como FIND nuevo si se quiere trackear.
- `docs/dev/research/**`, `tasks/**`, `avance/**`, `reviews/**` conservan citas históricas a raíz (actas fechadas, a propósito).
- `check-docs.mjs` sigue con `missingFrontmatter=1` preexistente (`docs/dev/historial/borrada-2026-10-10.md`, intacta en HEAD, fuera de alcance).
- `gen-index.mjs --check` solo converge post-commit (compara blobs commiteados vs generación) — re-verificar tras el commit.

## Notas
- `docs/index.md:1015` (fila WEB-DOCS-MOVE) conserva texto "SPEC.md (contexto)": es índice GENERADO desde el task file histórico (no se edita a mano por norma); gate-safe (texto plano, no link).

## Context Save Point
- **Fecha:** 2026-10-15
- **Branch:** develop
- **CI pendiente:** no
- **Decisiones:** mover+fix-todo (owner, Gate D) · excluidos = histórico/propiedad/repo-ajeno · rename puro de los 2 archivos (cuerpos idénticos 153/153 y 179/179) · frontmatter `kind: concept` por deriveKind · plan/task con frontmatter `plan`/`task`
- **Problemas conocidos:** ninguno abierto
- **Próxima tarea:** FIND-337 (plan Task 2)

## Review (Gate P2-01)
- **Revisor:** vanta-review (sesión ses_ebecfacb0ffeZiucKAsnzyxj0E — contexto distinto al implementador)
- **Veredicto:** ✅ APPROVE — renames 97%/98% + cuerpos idénticos + check-links 0 broken + check-docs sin nuevos missing + fmt exit 0 + diff --check limpio + floor-guard exit 0 + status sin excluidos. Sin fixes.

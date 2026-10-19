---
title: "FIND-337: Mover CLA a .github/cla/ + fix links"
kind: task
status: active
description: "Higiene raíz: CLAs a .github/cla con fix de clabot-config"
tags: [higiene, gobernanza]
---

# FIND-337: Mover CLA a .github/cla/ + fix links

## Metadata
- **Plan file:** `docs/dev/plans/2026-10-15-pre-mercado-estabilizacion.md` (Task 2)
- **Creado:** 2026-10-15
- **last-synced:** 2026-10-15
- **Estado:** ✅ COMPLETED
- **Skills:** `documentation-skill` (N/A real — `.github/*.md` fuera de `docs/`), `progreso`, `ponytail` (full)
- **Decisión owner:** `.github/cla/` + fix links en el mismo commit

## Blast Radius (Regla 0)
- **Leídos completos:** `CLA_INDIVIDUAL.md` (58L), `CLA_CORPORATE.md` (74L) — texto legal intacto (R100 + `fc /b` idénticos).
- **Entrantes:** `.github/clabot-config.json:2` (links rotos `.github/CLA_*.md` → fix a `.github/cla/*`). Cross-refs entre ambos CLA por nombre pelado (mismo dir, intactas). `CONTRIBUTING.md`: cero refs (grep confirmado).
- **Excluidos (histórico):** avance/historial, reviews/archive, research/validacion, tasks/GOV-F1, Manual Estrategico (prosa "en el repo", sigue verdadera).
- **Implicaciones:** bot clabot pasa de links rotos a válidos. Sin código, workflows ni gates afectados.

## Contrato
`Test-Path .github/cla/CLA_INDIVIDUAL.md, .github/cla/CLA_CORPORATE.md` + `git status` muestra `R` (renames) + `check-links.mjs` exit 0 + `git diff --check` limpio

## Herramientas
- `git mv`, `edit`, `node scripts/docs/check-links.mjs`

## Steps
### Step 1: Mover archivos (rename puro, texto legal intacto)
- **Archivos:** `CLA_INDIVIDUAL.md`, `CLA_CORPORATE.md` → `.github/cla/`
- **Acción:** `mkdir .github/cla; git mv ...`
- **Verify:** R100 ambos + `fc /b` sin diferencias + Test-Path raíz False
- **Estado:** ✅ COMPLETED

### Step 2: Fix clabot-config.json
- **Archivos:** `.github/clabot-config.json`
- **Acción:** 2 rutas en `message` → `.github/cla/*`
- **Verify:** diff muestra solo esas 2 líneas + Test-Path destinos True
- **Estado:** ✅ COMPLETED

### Step 3: Gates + review P2-01 + commit local
- **Archivos:** plan file (Estado + Iteraciones)
- **Acción:** check-links exit 0 + review `vanta-review` + commit local; sin push (Regla 7)
- **Verify:** contrato verde + commit existe + plan actualizado
- **Estado:** ✅ COMPLETED

## Dependencias
- Ninguna (Wave 0).

## Deuda / follow-up
- Ninguna nueva.

## Notas

## Context Save Point
- **Fecha:** 2026-10-15
- **Branch:** develop
- **CI pendiente:** no
- **Decisiones:** mover+fix-links (owner); texto legal intocable (R100); CONTRIBUTING sin refs
- **Problemas conocidos:** ninguno
- **Próxima tarea:** FIND-338 (plan Task 3)

## Review (Gate P2-01)
- **Revisor:** vanta-review (sesión ses_ebec0c2e7ffes94rC0MYmV0BZu — contexto distinto)
- **Veredicto:** ✅ APPROVE — R100 ambos CLA, `fc /b` idénticos, clabot solo 2 rutas, CONTRIBUTING cero, check-links exit 0, diff --check limpio, diff con solo los 3 archivos permitidos.

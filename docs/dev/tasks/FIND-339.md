---
title: "FIND-339: .editorconfig + hooks unificados en pre-commit"
kind: task
status: active
description: "Higiene: .editorconfig nuevo y pre-commit como sistema único"
tags: [higiene, hooks]
---

# FIND-339: .editorconfig + hooks unificados en pre-commit

## Metadata
- **Plan file:** `docs/dev/plans/2026-10-15-pre-mercado-estabilizacion.md` (Task 4)
- **Creado:** 2026-10-15
- **last-synced:** 2026-10-15
- **Estado:** ✅ COMPLETED
- **Skills:** `progreso`, `ponytail` (full)
- **Decisión owner:** unificar en uno solo → `pre-commit` sistema único, `.githooks` shim

## Blast Radius (Regla 0)
- **Leídos:** `.githooks/pre-commit` (87L), `.githooks/pre-push` (75L, INTACTO), `.pre-commit-config.yaml` (44L), `CONTRIBUTING.md:50-68`, `.gitattributes`, grep hooksPath (CI no usa pre-commit).
- **Diseño:** yaml = única definición de gates deterministas (+clippy local); shim delega + actionlint-condicional + fallback fmt; pre-push nativo intacto (barrera SIPP irreproducible en el framework).
- **Excluidos:** `.githooks/pre-push`, CI workflows, `.opencode/**`.

## Contrato
`Test-Path ./.editorconfig` + `pre-commit run` hooks propios en verde + shim LF + `sh -n` exit 0 + `git diff --check` limpio

## Herramientas
- `write`, `edit`, `pre-commit`, `git diff --check`

## Steps
### Step 1: .editorconfig + yaml ✅ COMPLETED
- **Verify:** Test-Path + yaml parse OK + clippy entry idéntico al hook viejo + MD028 del plan corregido
- **Estado:** ✅ COMPLETED

### Step 2: Shim + CONTRIBUTING ✅ COMPLETED
- **Verify:** CR 0 + `sh -n` exit 0 + párrafo setup/anti-install + mención muerta SKIP_CLIPPY removida (fix review 🟡#1)
- **Estado:** ✅ COMPLETED

### Step 3: Gates + review P2-01 + commit local ✅ COMPLETED
- **Verify:** cargo-fmt ✅ + cargo-clippy ✅ + markdownlint nuestros .md ✅ (MD028 plan fixed) + ruff falla SOLO en WIP ajeno `vantadb-python/**` (56 errores preexistentes, 0 archivos nuestros, sin tocar) + review APPROVE
- **Estado:** ✅ COMPLETED

## Dependencias
- Ninguna (Wave 0).

## Deuda / follow-up
- `pre-commit`/`ruff` binarios fuera de PATH como exe (usar `python -m pre_commit`); ruff 56 errores en WIP ajeno sin stagear — de otra sesión, no tocar.
- Review 🟡#2 (MD028 preexistente en CONTRIBUTING, fuera de scope) — commit separado si se quiere.

## Notas

## Context Save Point
- **Fecha:** 2026-10-15
- **Branch:** develop
- **CI pendiente:** no
- **Decisiones:** pre-commit único (owner); pre-push nativo intacto; actionlint-condicional en shim
- **Problemas conocidos:** ninguno
- **Próxima tarea:** SURF-01 (plan Task 5)

## Review (Gate P2-01)
- **Revisor:** vanta-review (sesión ses_ebeb40f17ffejQKxeCvz9iCygs — contexto distinto)
- **Veredicto:** ✅ APPROVE — staged quirúrgico (WIP ajeno excluido), pre-push intacto, yaml válido, editorconfig espeja gitattributes, shim LF + sh-syntax OK, nuestros hooks en verde, ruff rojo solo ajeno. 2 🟡 opcionales (#1 aplicada, #2 fuera de scope).

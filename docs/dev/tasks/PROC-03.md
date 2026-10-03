---
title: "TASK PROC-03: Descartar las 3 alertas de secret scanning (falsos positivos de test)"
kind: task
description: "Resolución used_in_tests de las alertas #14/#10/#8 (fixtures de test de scripts/docs/) — contrato: open=0"
---

# TASK PROC-03: Descartar las 3 alertas de secret scanning (falsos positivos de test)

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-02-post-release-0.8.0.md` (Task 2, Wave 0)
- **Fuente:** `docs/dev/Backlog.md` (PROC-03)
- **Esfuerzo:** 🟢 15min | **Appetite:** max 1h
- **Prioridad:** 🟡
- **Tipo:** release-CI (API ops — sin cambios de código)
- **Creado:** 2026-10-03T01:52Z | **last-synced:** 2026-10-03T02:12Z
- **Estado:** ✅ COMPLETED (review P2-01 APPROVE)
- **Campaign ID:** post-release-0.8.0-20261002

## Blast Radius

Sin ediciones de archivos (operación vía API de GitHub). Evidencia leída: `scripts/docs/check-secrets.mjs` (L323/L325) y `scripts/docs/probe-secrets.mjs` (L21) — claves de ejemplo que testean el propio detector de secretos.

## Impacto mapeado (Regla 0)

- **Archivos leídos (relevantes):** `scripts/docs/check-secrets.mjs:323,325`, `scripts/docs/probe-secrets.mjs:21`
- **Referencias entrantes:** los scripts corren en gates de docs/CI; las alertas apuntan al commit `86173d5f`.
- **Veredicto impacto:** NULO — solo cambió el estado remoto de 3 alertas; ningún archivo del repo se modificó.

## Contrato

1. `gh api '/repos/ness-e/Vantadb/secret-scanning/alerts?state=open' --jq 'length'` **= 0** → ✅ verificado 2026-10-03T01:53Z
2. Alertas 14/10/8 → `state=resolved`, `resolution=used_in_tests`, `resolution_comment` presente → ✅ (PATCHes 200)

## Steps

- [x] **Step 1 — Validar API contra docs oficiales:** `PATCH /repos/{owner}/{repo}/secret-scanning/alerts/{n}` con `state=resolved` + `resolution=used_in_tests` + `resolution_comment` (enums verificados en docs.github.com) — ✅
- [x] **Step 2 — Resolver 14/10/8:** PATCH con comentario citando archivo:línea del fixture — ✅ (3/3 → `used_in_tests`)
- [x] **Step 3 — Verify contrato:** open count = 0 — ✅
- [x] **Step 4 — Review P2-01 (vanta-review):** ✅ APPROVE (evidencia re-ejecutada por el reviewer)

## Review (P2-01)

| Campo | Valor |
|-------|-------|
| Reviewer | vanta-review (sesión fresca, sin participación en la implementación) |
| reviewer_context | `ses_f008a2213ffeVIqkk3p7sKvA0i` |
| Verdict | ✅ **APPROVE** — contrato re-ejecutado (open=0); 3 fixtures confirmados (keyboard-walk / secuencias fabricadas, `validity: unknown`, únicos en esas 3 líneas); sanity adversarial negativo |

## DoD (3 niveles)

- **task:** 0 alertas open verificadas por comando ✅
- **commit:** n/a (operación API)
- **release:** n/a

## Deuda técnica (Regla 6)

Ninguna introducida. Nota (pre-mortem #2): si los fixtures se re-suben, las alertas pueden reabrirse — evaluar ignore de paths de test en el scanner si recurre (≥2 reaperturas).

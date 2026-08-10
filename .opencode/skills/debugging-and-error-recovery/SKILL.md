---
name: debugging-and-error-recovery
description: DEPRECATED — delegate to systematic-debugging. Systematic root-cause debugging (Stop-the-Line loop). Use when tests fail, builds break, behavior doesn't match expectations, or you encounter any unexpected error.
---

# Debugging and Error Recovery — DEPRECATED

> **Este skill fue unificado con `systematic-debugging` (EVAL-03, 2026-08-10).**
> Existe **un solo** skill de debugging canónico: **`systematic-debugging`**
> (`.agents/skills/systematic-debugging/SKILL.md` — 4 fases, Iron Law:
> *NO FIXES WITHOUT ROOT CAUSE INVESTIGATION FIRST*).

## Acción

**No seguir este archivo.** Cargá `systematic-debugging` y seguí sus 4 fases:

1. **Root Cause Investigation** (repro determinista antes de cualquier fix)
2. **Pattern Discovery**
3. **Hypothesis & Test**
4. **Implementation & Verification**

El contenido histórico que estaba aquí (bucle STOP→PRESERVE→DIAGNOSE→FIX→GUARD→RESUME,
triage checklist, regla Stop-the-Line) queda **subsumido** en las fases del skill canónico:
- *Reproduce first* ↔ Phase 1 paso 2 ("Reproduce Consistently")
- *Localize* ↔ Phase 1 pasos 1-4 (leer errores, boundaries, instrumentar)
- *Guard against recurrence* ↔ Phase 4 + test de regresión
- *Tratar error output como datos no confiables* ↔ preservado en la sección correspondiente de `systematic-debugging`

## Si algo apunta acá

- Doc o prompt que referencia `debugging-and-error-recovery` → debe apuntar a `systematic-debugging`.
- Un agente que cargó este skill por error → cargue `systematic-debugging` en su lugar.
# Plan de Ejecución: P1 disciplina de proceso — pre-mortem, riesgos, retrospectiva, guardrails, gates

> **Inicio:** 2026-08-10
> **Estado:** ✅ COMPLETADO (2026-08-10)
> **Fuente:** docs/research/2026-08-10-agent-engineering/REPORTE-FINAL.md (§3.7 P1-1..P1-7)

## Resumen

| DO | DEFER | SKIP | BLOQUEADO |
|----|-------|------|-----------|
| 7  | 0     | 0    | 0         |

Ejecuta el paquete **P1** (disciplina de proceso) de la investigación agent-engineering.
P0 (harness, estados, debugging, path resolution) ya está completo y committeado.
Estas tareas son independientes entre sí → se distribuyen en sub-agentes paralelos.

### Task 1: P1-01 — Pre-mortem + stop conditions en plan.md
- **Esfuerzo:** 🟢 | **Prioridad:** 🔴 | **Ruta:** vanta-docs
- **Archivos clave:** `prompts/plan.md`
- **Gate Justificación:** La técnica con mejor ROI cognitivo; expone rabbit holes ANTES del commitment fuerte (REPORTE-FINAL §3.3 punto 2).
- **Gate Result:** ✅ DO
- **Contrato:** `rg -c "pre-mortem\|stop condition" prompts/plan.md` → ≥2. El Paso 0 de plan.md exige: (a) sección "¿por qué fracasaría?" explícita y (b) stop conditions (appetite/circuit breaker: cuándo se CANCELA la tarea).
- **Task file:** `skills/campaign-executor/tasks/P1-01.md`
- **Estado:** ✅ COMPLETED

### Task 2: P1-02 — Risk register en el template de plan
- **Esfuerzo:** 🟢 | **Prioridad:** 🔴 | **Ruta:** vanta-docs
- **Archivos clave:** `prompts/plan.md`
- **Gate Justificación:** La decisión DRV-131 se documenta por disciplina del autor, no porque el formato lo exija (REPORTE-FINAL §3.3 gap-02 punto 2).
- **Gate Result:** ✅ DO
- **Contrato:** `rg -c "risk register\|riesgos\|Prob×Impacto" prompts/plan.md` → ≥1. El template de plan incluye tabla de riesgos: Prob×Impacto, respuesta, trigger/due, máx 5-8 riesgos vivos.
- **Task file:** `skills/campaign-executor/tasks/P1-02.md`
- **Estado:** ✅ COMPLETED

### Task 3: P1-03 — Retrospectiva con 1 acción medida
- **Esfuerzo:** 🟢 | **Prioridad:** 🔴 | **Ruta:** vanta-docs
- **Archivos clave:** `prompts/pipeline-run.md`, `.opencode/skills/progreso/SKILL.md`
- **Gate Justificación:** `progreso` migra tareas pero sin Start/Stop/Continue + una acción medida contra baseline (REPORTE-FINAL §3.3 punto 5).
- **Gate Result:** ✅ DO
- **Contrato:** `rg -c "Start/Stop/Continue\|retrospectiva" prompts/pipeline-run.md` ≥1 AND `rg -c "Start/Stop/Continue\|retrospectiva" .opencode/skills/progreso/SKILL.md` ≥1.
- **Task file:** `skills/campaign-executor/tasks/P1-03.md`
- **Estado:** ✅ COMPLETED

### Task 4: P1-04 — Endurecer denied de ACT/RESEARCH
- **Esfuerzo:** 🟢 | **Prioridad:** 🔴 | **Ruta:** vanta-worker
- **Archivos clave:** `.opencode/task-system/config/state-tools.mjs:16-17,30-34`
- **Gate Justificación:** `ACT.denied = []` no prohíbe nada; `RESEARCH.denied` incluye `bash` read-only (inconsistencia: diagnosticar sin bash es difícil) (REPORTE-FINAL §3.4 punto 1).
- **Gate Result:** ✅ DO
- **Contrato:** `state-tools.mjs` tiene `ACT.denied` no vacío (editar/write fuera de scope) y `RESEARCH` PERMITE `bash` read-only (solo niega `edit`/`write`); las 10 estados de la lista NO cambian; `node .opencode/task-system/config/parity-check.mjs` sigue exit 0.
- **Task file:** `skills/campaign-executor/tasks/P1-04.md`
- **Estado:** ✅ COMPLETED

### Task 5: P1-05 — Gate de proceso para bug fixes
- **Esfuerzo:** 🟡 | **Prioridad:** 🔴 | **Ruta:** vanta-worker
- **Archivos clave:** `prompts/task.md`, `RULES.md`
- **Gate Justificación:** "el sistema exige que el método sea correcto, no solo que el test pase" — para cualquier `fix:`, contrato con evidencia de Phase 1 (repro + hipótesis escrita + 1 variable) antes del cambio (REPORTE-FINAL §3.7 P1-5).
- **Gate Result:** ✅ DO
- **Contrato:** `rg -c "repro\|hipótesis\|Phase 1" prompts/task.md RULES.md` → ≥4 repartidos entre ambos; el gate exige evidencia escrita de repro + hipótesis y 1 variable controlada ANTES del fix.
- **Task file:** `skills/campaign-executor/tasks/P1-05.md`
- **Estado:** ✅ COMPLETED

### Task 6: P1-06 — Instrumentar métricas North Star
- **Esfuerzo:** 🟡 | **Prioridad:** 🔴 | **Ruta:** vanta-worker
- **Archivos clave:** `evals/` (script nuevo), `docs/reports/`
- **Gate Justificación:** Solo se mide qué se colecta; sin primer-intento/falsos positivos normalizados el North Star no es verificable (REPORTE-FINAL §5.3 punto 3).
- **Gate Result:** ✅ DO
- **Contrato:** script `evals/northstar.mjs` lee plan files + `verify-log.jsonl` y emite `docs/reports/northstar.md` con tasa de completado primer intento, falsos positivos, regresión; `node evals/northstar.mjs` exit 0.
- **Task file:** `skills/campaign-executor/tasks/P1-06.md`
- **Estado:** ✅ COMPLETED

### Task 7: P1-07 — Activar pre-push barrier
- **Esfuerzo:** 🟡 | **Prioridad:** 🔴 | **Ruta:** vanta-worker
- **Archivos clave:** `.opencode/skills/unified-review/templates/pre-push.ps1`
- **Gate Justificación:** El pre-push barrier (SIPP) existe como template pero NO está instalado (AGENTS.md "Nota: hooks NO instalados").
- **Gate Result:** ✅ DO
- **Contrato:** existe hook real en `.git/hooks/pre-push` (o `core.hooksPath` apuntando a un dir con pre-push) que ejecuta el template; `git config core.hooksPath` retorna el path si se usa; instrucciones de instalación documentadas en el manual.
- **Task file:** `skills/campaign-executor/tasks/P1-07.md`
- **Estado:** ✅ COMPLETED
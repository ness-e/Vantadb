# Pipeline Evaluation Report

> Generado por `evals/eval-metrics.mjs` (EVAL-01) — 2026-08-11T20:24:42.680Z
> Datos: `.opencode/task-system/enforcement/verify-log.jsonl` (2 invocaciones de verify) + `docs/plans/*.md`

## North Star (RULES.md)

| Métrica | Threshold | Actual | Status |
|---|---|---|---|
| Tasa completado primer intento | >90% | 0.0% | 🚩 |
| Falsos positivos (COMPLETED con verify fallido) | 0 | 0 | ✅ |
| Regresión silenciosa (verify falla tras pasar) | 0 | 1 | 🚩 |

## Por tipo de tarea

| Tipo | Tareas |
|---|---|
| other | 1 |

## Skills → primer intento (P3-rem)

| Skill | Tareas con skill | Primer intento ✅ | Tasa |
|---|---|---|---|
| source-driven-development | 1 | 0 | 0.0% |
| doubt-driven-development | 1 | 0 | 0.0% |
| ponytail (full) | 1 | 0 | 0.0% |
| campaign-executor | 1 | 0 | 0.0% |

> Tareas con skills registradas: 1 / 1 (el resto no tenía "Archivos clave" derivables o log previo a P3-rem).

## Detalle por tarea

| Task | Plan | Verify ok | Verify fail | Primer intento | Regresiones | Estado final |
|---|---|---|---|---|---|---|
| T1-residuo-consolidado | — | 1 | 0 | ❌ | 1 | — |

## Notas
- Un "Primer intento" = la primera invocación de verify de la tarea pasó.
- "Regresiones" cuenta verifies que fallaron después de haber pasado para la misma tarea.
- El log se alimenta automáticamente desde `campaign_verify_cmd`; este reporte es la referencia del threshold de RULES.md.

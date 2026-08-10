# Pipeline Evaluation Report

> Generado por `evals/eval-metrics.mjs` (EVAL-01) — 2026-08-10T21:55:58.637Z
> Datos: `.opencode/task-system/enforcement/verify-log.jsonl` (0 invocaciones de verify) + `docs/plans/*.md`

## North Star (RULES.md)

| Métrica | Threshold | Actual | Status |
|---|---|---|---|
| Tasa completado primer intento | >90% | 0.0% | 🚩 |
| Falsos positivos (COMPLETED con verify fallido) | 0 | 0 | ✅ |
| Regresión silenciosa (verify falla tras pasar) | 0 | 0 | ✅ |

## Por tipo de tarea

| Tipo | Tareas |
|---|---|


## Skills → primer intento (P3-rem)

> ⚠️ **Sin datos aún** — el verify-log no tiene registros con `skills` (telemetría recolectada desde P3-rem en `campaign_verify_cmd`). La correlación skill → primer intento se poblará con los próximos verifies.

> Tareas con skills registradas: 0 / 0 (el resto no tenía "Archivos clave" derivables o log previo a P3-rem).

## Detalle por tarea

| Task | Plan | Verify ok | Verify fail | Primer intento | Regresiones | Estado final |
|---|---|---|---|---|---|---|
| _(sin datos de verify aún)_ | | | | | | |

## Notas
- Un "Primer intento" = la primera invocación de verify de la tarea pasó.
- "Regresiones" cuenta verifies que fallaron después de haber pasado para la misma tarea.
- El log se alimenta automáticamente desde `campaign_verify_cmd`; este reporte es la referencia del threshold de RULES.md.

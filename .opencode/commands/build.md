---
description: "DEPRECATED alias — use /pipeline task. /build now delegates to the unified pipeline with TDD + incremental-implementation skills."
---

> **CONSOLIDATED 2026-08-25** — `/build` ya no es un orquestador propio.
> Era un sistema paralelo a `/pipeline` con formato de plan incompatible
> (`tasks/plan.md`, que nunca existió) y estado propio (`last-build-state.json`,
> nunca escrito). Todo su valor útil (selección de skills TDD/incremental,
> modo bug "prove") vive ahora dentro del pipeline.

## Routing

| Invocación histórica | Equivalente actual |
|----------------------|--------------------|
| `/build` (siguiente tarea) | `/pipeline task <ID>` (o `/pipeline run` para el plan completo) |
| `/build auto` | `/pipeline run` |
| `/build prove` (bug: reproducir→fix) | `/pipeline task <ID>` — el task file tipo Bug exige la sección "Fase 1 — Evidencia de Debugging" (repro + hipótesis + test RED) vía skill `systematic-debugging`. Mismo contrato Prove-It, mismo gate. |

## Qué conserva el pipeline de este comando

Al ejecutar `/pipeline task`, cargá además estas skills si el task file no las lista:

- `test-driven-development` — lógica nueva o fix con test RED→GREEN
- `incremental-implementation` — slices verticales delgados (~100 líneas/step)
- `browser-testing-with-devtools` — si el cambio corre en navegador (`web/`)
- `source-driven-development` — si involucra frameworks/libs nuevas

El estado del build lo maneja la recitation canónica del campaign MCP
(`campaign_update_task_state`) — no hay archivo de estado paralelo.

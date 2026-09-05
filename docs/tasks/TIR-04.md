# TIR-04: Dead-letter queue

## Metadata
- **Plan file:** ninguno activo — fuente `docs/Backlog.md` P18 (línea 450)
- **Fuente:** backlog P18 — re-verificado 2026-08-12
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟡
- **Tipo:** Investigación/Decisión (NO implementación directa)
- **Turns estimados:** 5-10
- **Creado:** 2026-08-17T00:00
- **Estado:** ✅ COMPLETED — ver Context Save Point (commit: ninguno — lead commitea)
- **Incógnitas (uphill):** 1 — ¿basta un contenedor de tareas fallidas citado desde el plan vs infraestructura nueva?
- **Pendientes (downhill):** 3 steps

## Blast Radius
| Dirección | Módulos |
|-----------|---------|
| Callers | retry/recuperación de sub-agentes (SARL ESCALATE) |
| Callees | `agent-02-task-execution.md §8.2`, `pipeline-run.md` (SARL), `campaign_update_task_state` |
| Implicaciones | Decisión de diseño del task-system — sin cambios de código del producto |

## Contrato
"`docs/Investigaciones/TIR-04-dead-letter-queue.md` existe con: (1) análisis de qué preserva hoy la escalera ESCALATE (estado/traceId de tarea fallida) y qué perdería sin DLQ; (2) comparación contenedor-de-tareas-fallidas-citado-desde-plan vs infraestructura nueva; (3) recomendación EXPLÍCITA (implementar / WONTFIT / deferir)."

## Herramientas
- Read, grep

## Steps
### Step 1: Leer fuentes
- **Archivos:** `docs/Investigaciones/2026-08-10-agent-engineering/agent-02-task-execution.md §8.2`, `.opencode/task-system/prompts/pipeline-run.md` (SARL), `subagent-recovery.md`
- **Acción:** leer cómo se manejan hoy las tareas que agotan retries (ESCALATE a humano); qué estado se preserva
- **Verify:** flujo actual documentado
- **Estado:** ✅ COMPLETED — leído §8.2 (dead-letter queue como patrón), SARL Nivel 4 (documentar+commit WIP+state failed+SARL trace), pipeline-run §6.h/j (clasificación no-DONE, FAIL_MODE, NO_PROGRESS_LIMIT), pipeline-full (task file como estado durable), campaign-server (traceId en budget.json, task.failed event), tracer.mjs (traces/*.jsonl), tasks/closed/ (DEVOPS-10/15).

### Step 2: Comparar opciones
- **Acción:** contenedor de tareas fallidas citado desde el plan (task files + Context Save Point existentes) vs DLQ como infraestructura nueva (cola con estado/traceId). ¿Qué gana/pierde cada una?
- **Verify:** comparación documentada
- **Estado:** ✅ COMPLETED — tabla §3 del doc: A (contenedor) gana por estado ya durable + costo 🟢; B (DLQ nueva) pierde por duplicar estado y sobredimensionamiento (NO_PROGRESS_LIMIT ya acota fallos).

### Step 3: Escribir doc + recomendación
- **Archivos:** crear `docs/Investigaciones/TIR-04-dead-letter-queue.md`
- **Acción:** doc con análisis + recomendación explícita
- **Verify:** archivo existe con sección "Recomendación"
- **Estado:** ✅ COMPLETED — doc creado; sección §4 "Recomendación: **IMPLEMENTAR** — contenedor citado desde el plan (Opción A). WONTFIT infraestructura nueva (Opción B)". Sin commit (lead commitea).

## Dependencias
- Ninguna

## Notas
- Tarea read-only de investigación. NO commitear (el lead commitea).
- SECURITY/PERFORMANCE: no aplica — justificado.
- Review: lead (orquestador).

## Context Save Point

- **2026-08-17 (vanta-arch):** todos los steps ✅. Doc creado: `docs/Investigaciones/TIR-04-dead-letter-queue.md` — §2 (qué preserva ESCALATE: task file + plan recitation + traceId/traces + WIP commits; gap real = descubribilidad/re-procesamiento, no pérdida de estado), §3 (tabla A vs B), §4 (recomendación: IMPLEMENTAR contenedor Opción A, WONTFIT DLQ infraestructura nueva). Sin commit — lead commitea. Próximo: lead aplica la decisión (docs en subagent-recovery.md/pipeline-run.md, tarea separada) + cierra fila backlog P18 TIR-04 vía skill progreso.
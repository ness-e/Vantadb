# TIR-04 — Decisión: ¿dead-letter queue para tareas fallidas?

> Investigación/decisión. Origen: `docs/Backlog.md` P18 (línea 450), re-verificado 2026-08-12.
> Tipo: Investigación/Decisión — NO implementación directa. Estado: RESUELTO 2026-08-17 (por vanta-arch; commit del lead).
> Alcance: task-system del pipeline (`.opencode/task-system/`) — sin impacto en código del producto.

## 1. Fuentes analizadas

| Fuente | Qué dice |
|--------|----------|
| `agent-02-task-execution.md` §8.2 (líneas 306-312) | Patrones de resiliencia: idempotencia, circuit breaker, **dead-letter queue** ("los mensajes/tareas que agotaron retries van a una cola de inspección humana, no se pierden"), escalación humana. |
| `subagent-recovery.md` §2 (SARL) | Escalera RESUME→RETRY→STRATEGY→**ESCALATE**. Nivel 4: documentar intentos, commit WIP, `campaign_update_task_state "failed"`, aplicar FAIL_MODE. §3.7: SARL trace obligatorio (`campaign_session_track` con `sarlRung`/`outcome`/`reason`). |
| `pipeline-run.md` §6.h/j | Clasificación de resultados no-DONE; tras agotar escalera → `campaign_update_task_state "failed"`, FAIL_MODE stop/skip, NO_PROGRESS_LIMIT (3 no-DONE seguidos → pausar y preguntar humano). |
| `pipeline-full.md` §7 | Contrato de retorno `RESULTADO`; el task file (steps ✅/⬜ + Context Save Point) es el estado durable para reanudar. |
| `campaign-server.mjs` (updateTaskStateCore) | `newState="failed"` persiste estado + recitation en el plan file; genera `task.failed` trace event; traceId por tarea persiste en `<plan>.budget.json` → `tasks[<taskId>].traceId` (P2-05). |
| `tracer.mjs` | Traces persistidos en `traces/<campaignId>.jsonl` en la raíz del proyecto (incluye SARL trace con sarlRung/outcome/reason). |
| `tasks/closed/` (DEVOPS-10, DEVOPS-15) | Convención existente (documentada en AGENTS.md path resolution): tareas cerradas sin resolver se mueven a `tasks/closed/` con discovery completo + estado + rationale. |

## 2. Qué preserva hoy la escalera ESCALATE y qué se perdería sin DLQ

**Lo que NO se pierde hoy** — el estado de una tarea fallida ya es durable en 4 lugares:

| Artefacto | Qué conserva | Ubicación |
|-----------|--------------|-----------|
| **Task file** | Steps ✅/⬜, Context Save Point (dónde quedó, próximo step), blast radius, contrato | `.opencode/skills/campaign-executor/tasks/<ID>.md` (nunca se borra al fallar) |
| **Plan file** | Fila de la tarea en `❌ FAILED` + recitation (activeGoal, lastAction, result, nextAction, contract/evidencia/artefactos) | `docs/plans/<plan>.md` vía `campaign_update_task_state` |
| **traceId + traces** | traceId por tarea (P2-05) + eventos `task.failed` + SARL trace (sarlRung, outcome, reason) | `<plan>.budget.json` + `traces/<campaignId>.jsonl` |
| **WIP commits** | Trabajo parcial commiteado antes de escalar | git history |

**Qué se perdería sin DLQ (el gap real):**
- **Nada se borra** — el gap NO es pérdida de estado sino de *descubribilidad y re-procesamiento*:
  1. **No hay índice único** de "todas las tareas fallidas" cruzando planes; el humano debe grepear plan files / task files manualmente.
  2. **`❌ FAILED` es terminal** — nada re-encola automáticamente a `⬜ PENDING` tras revisión humana; la re-apertura es manual (`campaign_update_task_state "pending"`).
  3. **`tasks/closed/` es convención no formalizada** — solo 2 ejemplos; no hay regla que obligue a mover el task file ahí al escalar.
- Sin ningún contenedor, una tarea fallida quedaría solo como fila `❌` en el plan: el contexto (por qué falló, qué se intentó, próximo paso) sobrevive en la recitation del plan, pero el detalle operativo (steps, Context Save Point) quedaría huérfano si el task file se moviera/limpiara.

## 3. Comparación de opciones

| Dimensión | **A. Contenedor citado desde el plan** (task files + Context Save Point + `tasks/closed/`) | **B. DLQ como infraestructura nueva** (cola con estado/traceId) |
|-----------|----------------|----------------|
| Estado durable | Ya existe 100% (task file, plan recitation, traceId, traces) | Habría que duplicarlo en un formato/cola nueva → segunda fuente de verdad |
| Costo | 🟢 Docs + convención (~10-20 líneas en subagent-recovery.md/pipeline-run.md) | 🟡🔴 Tooling nuevo (server/archivo + MCP tooling), sync con state machine C0, plan sync |
| Re-procesamiento | Manual pero trivial: humano abre plan → ve `❌` → lee task file Context Save Point → `campaign_update_task_state "pending"` → pipeline-full.md reanuda desde primer step ⬜ | Automático posible (re-drive), pero añade complejidad de ack/requeue |
| Inspección humana | Multi-archivo (plan + task file + trace) | Un solo lugar consolidado |
| Riesgo | Bajo: no toca tooling; la convención ya existe | Medio: estado duplicado puede divergir; mantenimiento permanente |
| Escalabilidad | Suficiente: fallos son raros (NO_PROGRESS_LIMIT detiene el pipeline a los 3 no-DONE y pregunta al humano) | Sobredimensionada para la tasa real de fallos |

## 4. Recomendación: **IMPLEMENTAR — contenedor de tareas fallidas citado desde el plan (Opción A). WONTFIT la infraestructura nueva (Opción B).**

**Qué implementar (docs + convención, sin tooling nuevo):**
1. **Formalizar `tasks/closed/` como el contenedor de tareas fallidas**: al alcanzar ESCALATE (SARL Nivel 4), el orquestador DEBE mover el task file a `tasks/closed/<ID>.md` (con estado `❌ FAILED` + SARL trace) y el plan conserva la fila `❌ FAILED` con recitation — el plan **cita** el task file cerrado (ruta explícita en la fila de la tarea).
2. **Añadir regla de re-procesamiento**: tras revisión humana, `campaign_update_task_state "pending"` + mover el task file de vuelta a `tasks/` → el flujo normal de `pipeline-full.md` reanuda desde el primer step ⬜ PENDING (el Context Save Point ya lo permite).
3. **Documentar el índice de fallos** en la operación del lead: `rg -l "❌ FAILED" docs/plans/*.md` + `tasks/closed/` como vistas de inspección.

**Por qué no DLQ nueva (WONTFIT):** escalera ponytail rung 2 — el contenedor *ya existe* en el codebase (task files + Context Save Point + `tasks/closed/` + traceId). La infraestructura nueva (Opción B) duplicaría estado que ya persiste en 4 lugares, agregaría una segunda fuente de verdad que puede divergir, y es sobredimensionada para una tasa de fallos que el propio pipeline ya acota (NO_PROGRESS_LIMIT → pausa y pregunta al humano). El patrón es idéntico a TIR-03 (Fase 0.5): cerrar un gap con doctrina/convención sobre infraestructura existente, no con tooling.

**Criterio de re-evaluación** (cuándo volver a considerar DLQ): si la tasa de fallos real supera ~1 tarea fallida por plan Y la revisión humana no alcanza (fallos recurrentes del mismo tipo sin diagnóstico), el siguiente escalón sería un script que liste `tasks/closed/` con traceIds para el lead — no una cola.

## 5. Revisión

- Revisión P2-01: por vanta-review (el orquestador la delega al cerrar; este doc es la evidencia).
- Nota: este doc NO implementa los cambios de §4 — es la decisión. La implementación (docs en subagent-recovery.md/pipeline-run.md) la ejecuta el lead en una tarea separada.
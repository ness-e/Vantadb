---
description: Gestionar campañas: crear plan desde backlog, definir tarea, ejecutar
---

> **ENTRY POINT — Campaign Command**
> El agente DEBE leer este archivo cuando el usuario envía un mensaje que empieza con `/campaign`.
> Path resolution: `prompts/X.md` → `.opencode/task-system/prompts/X.md`
> Skills: `skills/X` → `.opencode/skills/X/`
> Instrucciones: cargar cada prompt con Read tool y ejecutar secuencialmente.
> Al finalizar: mostrar comando recomendado para el siguiente paso.

Cargá las skills campaign-executor, brainstorming, writing-plans, progreso, ponytail (full).

Entrada: $1
Si no se especificó entrada, usá `docs/Backlog.md` en modo plan.

## Router: detectar modo según el argumento

- Si el argumento es `pipeline` → **MODO PIPELINE**: mostrá el comando exacto para arrancar `/loop-goal` con el prompt `prompts/pipeline-full.md`. Este modo ejecuta UNA TAREA COMPLETA por iteración (discovery → impl → verify → commit → progreso).
- Si el argumento es `run` → **MODO EJECUCIÓN**: mostrá el comando para arrancar el harness sobre el plan file más reciente.
- Si el argumento empieza con `task ` → **MODO TAREA**: extraé el ID después de `task ` y ejecutá el prompt de `prompts/task.md` para definir esa tarea a profundidad.
- Si el argumento es una ruta de archivo o está vacío → **MODO PLAN**: ejecutá el prompt de `prompts/plan.md` para crear un plan desde ese backlog.

---

## MODO PLAN — Crear plan de campaña desde backlog

Backlog: $1 (o docs/Backlog.md si vacío)

Aplicá el triage gate del campaign-executor a CADA tarea en el backlog.
Resultados posibles: ✅ DO, 🟡 DEFER, ❌ SKIP, 🔴 BLOQUEADO.

### Reglas del gate
1. Bug ya inexistente o feature ya implementada → SKIP
2. Cosmético sin queja de usuario → DEFER
3. Esfuerzo >> impacto → DEFER o SKIP
4. Dependencia no lista → BLOQUEADO
5. Prioridad original es sugerencia, no orden

### Formato del plan file
Creá `docs/plans/<FECHA>-<nombre>.md` con:
- Solo tareas ✅ DO, ordenadas por prioridad real
- Gate Justificación para cada una
- Contrato verificable (condición booleana que un comando puede verificar)
- Task file: `skills/campaign-executor/tasks/<ID>.md` (aún no existe)
- Estado inicial ⬜ PENDING
- Tabla resumen al inicio (DO / DEFER / SKIP / BLOQUEADO)
- Fuente del backlog

### Auto-detección de formato
- Si el backlog tiene estructura conocida → parseá determinísticamente
- Si no → el agente interpreta con LLM para extraer tareas

---

## MODO TAREA — Definir tarea a profundidad

Task ID: {id extraído después de "task "}

Buscá la tarea en:
1. El plan file más reciente en `docs/plans/` (si existe)
2. El backlog del proyecto (`docs/Backlog.md`)
3. Si no se encuentra en ninguna, preguntale al usuario

Después de encontrar la tarea:
- Cargá `prompts/task.md` y ejecutá sus fases
- `codegraph_explore` para blast radius
- Creá `.opencode/skills/campaign-executor/tasks/<ID>.md`
- Actualizá el plan file si existe (agregá link al task file)

---

## MODO PIPELINE — Ejecución completa (una tarea por iteración)

Modo principal. Ideal para ejecutar el backlog completo tarea por tarea.

**Recomendado (chat, backlog completo sin parar):**
```
/pipeline run
```
Ejecuta TODAS las tareas automáticamente usando sub-agentes. Cada sub-agente procesa una tarea completa con contexto fresco. No requiere re-ejecutar manualmente.

**Alternativa (chat, una tarea a la vez):**
```
/loop-goal "Ejecutá UNA TAREA COMPLETA siguiendo `.opencode/task-system/prompts/pipeline-full.md`"
```
Cada iteración procesa UNA TAREA COMPLETA (discovery → implementación → verify → commit → skill progreso).
Requiere re-ejecutar `/loop-goal` manualmente para cada tarea.

Al terminar todas las tareas, ejecuta `skill progreso` y reporta campaña completada.

## MODO EJECUCIÓN — Arrancar ejecución (MCP, paso a paso)

Buscá el plan file más reciente con `campaign_get_next_task` (MCP) o en `docs/plans/`.
Mostrá AMBOS métodos:

**Chat (paso a paso, con auto-skill-loading):**
```
/loop-goal "Ejecutá UNA iteración de campaña siguiendo `.opencode/task-system/prompts/iter-loop-tools.md`"
```
Usa `campaign_get_next_task`/`campaign_update_task_state`/`campaign_verify_cmd` (MCP)
+ `campaign_load_skills`/`campaign_detect_task_type` para carga automática de skills.
No lee el plan file completo — usa MCP tools.
Cada iteración procesa UN PASO (no una tarea completa). Útil para depuración.

**Fallback (harness PowerShell, para terminal dedicada):**
```
.\harness-executor.ps1 -PlanFile docs\plans\<plan-más-reciente>.md -Interval 10
```
El flag `-Yes` auto-responde sí a todas las confirmaciones (git dirty, stall, etc.).

Si hay múltiples planes activos, listalos y pedí al usuario que elija.

---

## Al final de cualquier modo

Después de la acción correspondiente, mostrá el comando exacto para lo que sigue:

| Después de... | Mostrar... |
|--------------|------------|
| Crear plan | `/pipeline run` *(chat, backlog completo sin parar, recomendado)* o `/loop-goal "Ejecutá UNA TAREA COMPLETA siguiendo \`.opencode/task-system/prompts/pipeline-full.md\`"` *(chat, una tarea por vez)* o `.\harness-executor.ps1 -PlanFile ... -Interval 10` *(terminal)* |
| Definir tarea | Task file creado. Para ejecutar: `/pipeline run` *(backlog completo)* o `/pipeline task <ID>` *(solo esta tarea)* o `.\harness-executor.ps1 -PlanFile ... -SingleTask <ID>` |
| Ejecutar pipeline | `/pipeline run` *(backlog completo sin parar, recomendado)* o `/loop-goal "...pipeline-full.md"` *(una tarea por vez)* o `.\harness-executor.ps1` *(terminal)* |
| Cualquier modo | **También disponible:** `/pipeline plan`, `/pipeline task <ID>`, `/pipeline run` — comandos unificados con los mismos 3 modos. `/pipeline run` ejecuta backlog completo sin parar usando sub-agentes, sin necesidad de re-ejecutar `/loop-goal` manualmente. |

---

## Apéndice: Prompt Templates (Referencia Rápida)

| # | Propósito | Prompt / Comando |
|---|-----------|-----------------|
| 0 | **Iniciar campaña** (triage + gate + crear plan) | `/campaign docs/Backlog.md` → aplica triage gate, crea `docs/plans/<fecha>-<nombre>.md` con solo ✅ DO |
| 1 | **Una iteración** (paso a paso con MCP) | `/loop-goal "Ejecutá UNA iteración siguiendo \`.opencode/task-system/prompts/iter-loop-tools.md\`"` |
| 2 | **Una tarea completa** (discovery → impl → verify → commit) | `/pipeline task <ID>` o `/loop-goal "Ejecutá UNA TAREA siguiendo \`.opencode/task-system/prompts/pipeline-full.md\`"` |
| 3 | **Backlog completo** (sub-agentes, auto) | `/pipeline run` — ejecuta todas las tareas, contexto fresco por tarea |
| 4 | **FAIL_MODE=skip** (no parar en fallos) | `/pipeline run` con FAIL_MODE=skip (editar variable en pipeline-run.md) |
| 5 | **FAIL_MODE=parallel** (waves paralelas) | `/pipeline run` con FAIL_MODE=parallel (editar variable en pipeline-run.md) |

---
description: "Pipeline unificado: crear plan desde backlog, definir tarea, ejecutar backlog completo. Modos: plan | task | run | interactive | pipeline | ejecución"
---

> **ENTRY POINT — Pipeline Command**
> El agente DEBE leer este archivo cuando el usuario envía un mensaje que empieza con `/pipeline`.
> Path resolution: `prompts/X.md` → `.opencode/task-system/prompts/X.md`
> Skills: `skills/X` → `.opencode/skills/X/`
> Tasks: `tasks/ID.md` → `.opencode/skills/campaign-executor/tasks/ID.md`
> Instrucciones: cargar cada prompt con Read tool y ejecutar secuencialmente.
> Al finalizar: handoff y stop (no continuar sin que el usuario lo pida).

Cargá las skills campaign-executor, brainstorming, writing-plans, planning-and-task-breakdown, progreso, ponytail (full).

Si el modo es `plan` (crear plan desde backlog), cargá también `spec-driven-development` — backlog items pueden necesitar specs antes de partir en tareas.

Entrada: $1
Si no se especificó entrada, usá `docs/Backlog.md` en modo plan.

## Router: detectar modo según el argumento

- Si el argumento es `plan` + ruta → **MODO PLAN**: creá plan desde backlog.

**Agents:** Pipeline no spawns agentes directamente — delega a `/build` (que usa vanta-worker/vanta-engine) y `/audit` (que usa vanta-audit/vanta-tuner) para tareas concretas. El plan se ejecuta vía sub-agentes genéricos o prompts, no agentes especializados.
- Si el argumento empieza con `task ` → **MODO TAREA**: extraé el ID y definí/ejecutá esa tarea.
- Si el argumento es `run` + plan opcional → **MODO RUN**: ejecutá backlog completo sin parar.
- Si el argumento es `pipeline` → **MODO PIPELINE**: ejecutá una tarea completa por iteración vía `/loop-goal`.
- Si el argumento es `ejecución` o `mcp` → **MODO EJECUCIÓN**: paso a paso con MCP tools o harness PowerShell.
- Si no hay argumento → **MODO INTERACTIVO**: detectá estado actual y sugerí próximo paso.

Cross-command flow: pipeline → build → audit → ship → rollback
  - `/pipeline` planifica y ejecuta tareas
  - `/build` implementa tareas individuales (RED→GREEN→refactor)
  - `/audit` certifica la calidad antes de ship
  - `/ship` decide GO/NO-GO con fan-out a 3 personas
  - `/rollback` si el ship falla

---

## MODO PLAN — Crear plan desde backlog

```
/pipeline plan docs/Backlog.md
```

**Paso 0 — Leer spec existente.** Buscá `SPEC.md` en raíz, `docs/SPEC.md`, o archivos en `spec/`. Si existe, úsalo como contexto. Si no, continuá igual.

**Enter plan mode — read only, no code changes.**

Cargá `prompts/plan.md` con `{{BACKLOG_PATH}}` = ruta del backlog.

Aplicá triage gate (✅ DO / 🟡 DEFER / ❌ SKIP / 🔴 BLOQUEADO).

### Reglas del gate
1. Bug ya inexistente o feature ya implementada → SKIP
2. Cosmético sin queja de usuario → DEFER
3. Esfuerzo >> impacto → DEFER o SKIP
4. Dependencia no lista → BLOQUEADO
5. Prioridad original es sugerencia, no orden

### Proceso de planificación
1. Identificá el grafo de dependencias entre componentes
2. Sliced vertical: un path completo por tarea (no capas horizontales)
3. Cada tarea debe tener acceptance criteria y verification steps
4. Agregá checkpoints entre fases

### Formato del plan file
Creá `docs/plans/<FECHA>-<nombre>.md` con:
- Solo tareas ✅ DO, ordenadas por prioridad real
- Gate Justificación para cada una
- Contrato verificable (condición booleana que un comando puede verificar)
- Task file: `skills/campaign-executor/tasks/<ID>.md` (aún no existe)
- Estado inicial ⬜ PENDING
- Tabla resumen al inicio (DO / DEFER / SKIP / BLOQUEADO)
- Fuente del backlog

Opcional: también guardá en `tasks/plan.md` y `tasks/todo.md` para compatibilidad con `/build auto`.

### Auto-detección de formato
- Si el backlog tiene estructura conocida → parseá determinísticamente
- Si no → el agente interpreta con LLM para extraer tareas

**Al finalizar, mostrá:**
```
Plan creado en docs/plans/<FECHA>-<nombre>.md
Próximo paso recomendado:
  /pipeline run                    → ejecutar backlog completo sin parar
  /pipeline task <ID>              → definir/ejecutar una tarea específica
  /build                           → implementar primera tarea (RGD)
```

---

## MODO TAREA — Definir/ejecutar tarea compleja

```
/pipeline task DRV-068
```

Task ID: {id extraído después de "task "}

1. Buscá la tarea en:
   - El plan file más reciente (`docs/plans/`)
   - `docs/Backlog.md`
   - Si no se encuentra en ninguna, preguntale al usuario
2. Si no existe task file → cargá `prompts/task.md` y ejecutá sus 4 fases:
   - Auto-detect type → codegraph_explore → blast radius → web research → atomic steps
   - Creá `.opencode/skills/campaign-executor/tasks/<ID>.md`
3. Si ya existe task file → ejecutá la tarea usando `pipeline-full.md` internamente
4. Si el usuario quiere ejecutar AHORA → cargá `prompts/pipeline-full.md` y ejecutá la tarea completa

**Al finalizar, mostrá:**
```
Task file creado: .opencode/skills/campaign-executor/tasks/<ID>.md
Para ejecutar:
  /pipeline run                      → backlog completo
  /build                             → implementar (RED→GREEN→refactor)
  /build prove                       → si es un bug (reproducir→fix)
```

---

## MODO RUN — Ejecutar backlog completo (non-stop)

```
/pipeline run [docs/plans/mi-plan.md]
```

Ejecutá TODAS las tareas del plan file una por una hasta completar.
Usá `prompts/pipeline-run.md` con el plan file correspondiente.

> Si no se especifica plan file, detectá automáticamente el más reciente en `docs/plans/`.
> Si no hay plan file, mostrá error: "No hay plan file. Usá `/pipeline plan docs/Backlog.md` primero."

### FAIL_MODE
Por defecto, el pipeline se detiene al primer error. Se puede modificar:
- **FAIL_MODE=skip** — no parar en fallos, marcar como failed y continuar
- **FAIL_MODE=parallel** — waves paralelas de tareas independientes
  (editar variable en `prompts/pipeline-run.md`)

**FAIL_MODE=parallel en detalle:**
1. Identificá tareas sin dependencias entre sí
2. Agrupalas en waves según el grafo de dependencias
3. Wave 0: tareas sin dependencias → N sub-agentes paralelos
4. Wave 1: tareas que dependen de Wave 0 → N sub-agentes
5. MAX_CONCURRENT = min(4, tareas_en_wave)
6. Cada tarea paralela es su propio sub-agente (contexto fresco)
7. Si una tarea de una wave falla, las tareas dependientes en waves posteriores quedan BLOQUEADAS

Al terminar todas las tareas, ejecutá `skill progreso` y reportá campaña completada.

---

## MODO PIPELINE — Ejecución completa (una tarea por iteración)

Modo ideal para ejecutar el backlog completo tarea por tarea con contexto fresco.

**Recomendado (backlog completo sin parar):**
```
/pipeline run
```
Ejecuta TODAS las tareas automáticamente usando sub-agentes. Cada sub-agente procesa una tarea completa con contexto fresco. No requiere re-ejecutar manualmente.

**Alternativa (una tarea a la vez):**
```
/loop-goal "Ejecutá UNA TAREA COMPLETA siguiendo `.opencode/task-system/prompts/pipeline-full.md`"
```
Cada iteración procesa UNA TAREA COMPLETA (discovery → implementación → verify → commit → skill progreso).
Requiere re-ejecutar `/loop-goal` manualmente para cada tarea.

---

## MODO EJECUCIÓN — Arrancar ejecución (MCP, paso a paso)

Buscá el plan file más reciente con `campaign_get_next_task` (MCP) o en `docs/plans/`.
Mostrá AMBOS métodos:

**Chat (paso a paso, con auto-skill-loading):**
```
/loop-goal "Ejecutá UNA iteración siguiendo `.opencode/task-system/prompts/iter-loop-tools.md`"
```
Usa `campaign_get_next_task`/`campaign_update_task_state`/`campaign_verify_cmd` (MCP)
+ `campaign_load_skills`/`campaign_detect_task_type` para carga automática de skills.
No lee el plan file completo — usa MCP tools.
Cada iteración procesa UN PASO (no una tarea completa). Útil para depuración.

**Harness PowerShell (terminal dedicada o via MCP):**
```
.opencode\task-system\harness\harness-executor.ps1 -PlanFile docs\plans\<plan-más-reciente>.md -Interval 10
```
El flag `-Yes` auto-responde sí a todas las confirmaciones (git dirty, stall, etc.).
También se puede invocar desde el chat via MCP: `campaign_mount_harness` con los mismos parámetros.
Usar `campaign_session_track` (MCP) para tracking de sesión y persistencia entre iteraciones.

Si hay múltiples planes activos, listalos y pedí al usuario que elija.

---

## MODO INTERACTIVO — Sin argumentos

Detectá el estado actual:

0. **Primero: buscá checkpoint** `docs/pipeline-state.json`
   - Si existe y tiene `inProgress` → mostrá "Pipeline en pausa en task {inProgress}. Usá `/pipeline run` para continuar."
   - Si existe y no hay `inProgress` pero hay completed/failed → mostrá resumen y recomendá `/pipeline run`
1. Buscá plan files en `docs/plans/`
2. Si hay un plan file ⏳ EN PROGRESO:
   - Leé el resumen (completados/pendientes/failed)
   - Mostrá: "Tienes un plan en progreso: N/M completadas. Usá `/pipeline run` para continuar."
3. Si hay plan file pero no iniciado (solo ⬜ PENDING):
   - Mostrá: "Plan listo con N tareas. Usá `/pipeline run` para empezar."
4. Si no hay plan file:
   - Mostrá: "No hay plan activo. Usá `/pipeline plan docs/Backlog.md` para crear uno."
5. Si hay tareas en progreso sin plan:
   - Mostrá las tareas y recomendá `/pipeline plan` o `/pipeline task <ID>`

---

## Al final de cualquier modo

Mostrá el comando exacto para lo que sigue:

| Después de... | Mostrar... |
|--------------|------------|
| Crear plan | `/pipeline run` *(backlog completo)* o `/build` *(primera tarea, RED→GREEN)* o `.opencode\task-system\harness\harness-executor.ps1` *(terminal)* |
| Definir tarea | `/build` *(implementar, RED→GREEN)* o `/pipeline run` *(backlog completo)* |
| Ejecutar pipeline | `/audit quick` *(verificar calidad)* o `/ship` *(preparar release)* o `/status` *(dashboard)* |
| Cualquier modo | También disponible: `/audit`, `/ship`, `/rollback`, `/status` |

---

## Mapa rápido

| Comando | Qué hace | Llama a |
|---------|----------|---------|
| `/pipeline plan backlog.md` | Crear plan | `prompts/plan.md` |
| `/pipeline task ID` | Definir tarea | `prompts/task.md` → `prompts/pipeline-full.md` |
| `/pipeline run [plan]` | Ejecutar backlog completo | `prompts/pipeline-run.md` |
| `/pipeline` | Detectar estado y sugerir | auto-detect |
| `/pipeline pipeline` | Una tarea por iteración | `/loop-goal` + `prompts/pipeline-full.md` |
| `/pipeline ejecución` | Paso a paso con MCP | `/loop-goal` + `prompts/iter-loop-tools.md` |

---

## Apéndice: Prompt Templates (Referencia Rápida)

| # | Propósito | Prompt / Comando |
|---|-----------|-----------------|
| 0 | **Iniciar pipeline** (triage + gate + crear plan) | `/pipeline plan docs/Backlog.md` |
| 1 | **Una iteración** (paso a paso con MCP) | `/loop-goal "Ejecutá UNA iteración siguiendo \`.opencode/task-system/prompts/iter-loop-tools.md\`"` |
| 2 | **Una tarea completa** (discovery → impl → verify → commit) | `/pipeline task <ID>` o `/build` |
| 3 | **Backlog completo** (sub-agentes, auto) | `/pipeline run` |
| 4 | **FAIL_MODE=skip** (no parar en fallos) | `/pipeline run` con FAIL_MODE=skip |
| 5 | **FAIL_MODE=parallel** (waves paralelas) | `/pipeline run` con FAIL_MODE=parallel |

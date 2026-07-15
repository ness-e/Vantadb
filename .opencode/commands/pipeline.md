---
description: Pipeline unificado: crear plan, ejecutar tarea compleja, ejecutar backlog completo
---

Cargá las skills campaign-executor, progreso, ponytail (full).

Entrada: $1

## Router: detectar modo según el argumento

- Si el argumento es `plan` + ruta → **MODO PLAN**: creá plan desde backlog.
- Si el argumento empieza con `task ` → **MODO TAREA**: extraé el ID y definí/ejecutá esa tarea.
- Si el argumento es `run` + plan opcional → **MODO RUN**: ejecutá backlog completo sin parar.
- Si no hay argumento → **MODO INTERACTIVO**: detectá estado actual y sugerí próximo paso.

Los 3 modos se llaman entre sí:
  - `plan` crea plan file y luego recomienda `/pipeline run`
  - `task` define task file y luego recomienda `/pipeline run` como próximo paso
  - `run` por cada tarea sin task file, llama internamente a `/pipeline task <ID>`

---

## MODO PLAN — Crear plan desde backlog

```
/pipeline plan docs/Backlog.md
```

Cargá `prompts/plan.md` con `{{BACKLOG_PATH}}` = ruta del backlog.
Aplicá triage gate (✅ DO / 🟡 DEFER / ❌ SKIP / 🔴 BLOQUEADO).
Creá `docs/plans/<FECHA>-<nombre>.md`.

**Al finalizar, mostrá:**
```
Plan creado en docs/plans/<FECHA>-<nombre>.md
Próximo paso recomendado:
  /pipeline run                    → ejecutar backlog completo sin parar
  /pipeline task <ID>              → definir/ejecutar una tarea específica
```

---

## MODO TAREA — Definir/ejecutar tarea compleja

```
/pipeline task DRV-068
```

Task ID: {id extraído después de "task "}

1. Buscá la tarea en el plan file más reciente (`docs/plans/`) o en `docs/Backlog.md`
2. Si no existe task file → cargá `prompts/task.md` y ejecutá sus 4 fases:
   - Auto-detect type → codegraph_explore → blast radius → web research → atomic steps
   - Creá `.opencode/skills/campaign-executor/tasks/<ID>.md`
3. Si ya existe task file → ejecutá la tarea usando `pipeline-full.md` internamente
4. Si el usuario quiere ejecutar AHORA → cargá `prompts/pipeline-full.md` y ejecutá la tarea completa

**Al finalizar, mostrá:**
```
Task file creado: .opencode/skills/campaign-executor/tasks/<ID>.md
Para ejecutar esta y el resto del backlog:
  /pipeline run
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

## Mapa rápido

| Comando | Qué hace | Llama a |
|---------|----------|---------|
| `/pipeline plan backlog.md` | Crear plan | `prompts/plan.md` |
| `/pipeline task ID` | Definir tarea | `prompts/task.md` → `prompts/pipeline-full.md` |
| `/pipeline run [plan]` | Ejecutar backlog completo | `prompts/pipeline-run.md` |
| `/pipeline` | Detectar estado y sugerir | auto-detect |

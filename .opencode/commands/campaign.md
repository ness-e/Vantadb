---
description: Gestionar campañas: crear plan desde backlog, definir tarea, ejecutar
---

Cargá las skills campaign-executor, brainstorming, writing-plans, progreso, ponytail (full).

Entrada: $1
Si no se especificó entrada, usá `docs/Backlog.md` en modo plan.

## Router: detectar modo según el argumento

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

## MODO EJECUCIÓN — Arrancar ejecución

Buscá el plan file más reciente en `docs/plans/` con Estado ⏳ EN PROGRESO.
Mostrá AMBOS métodos:

**Recomendado (vía custom tools, en el chat):**
```
/loop-goal "Ejecutá UNA iteración de campaña siguiendo `.opencode/prompts/iter-loop-tools.md`"
```
Usa `campaign_get_task`/`campaign_update_task`/`campaign_verify` — no lee el plan file completo.

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
| Crear plan | `/loop-goal "Ejecutá UNA iteración de campaña siguiendo \`.opencode/prompts/iter-loop-tools.md\`"` *(chat, recomendado)* o `.\harness-executor.ps1 -PlanFile ... -Interval 10` *(terminal)* |
| Definir tarea | Task file creado. Para ejecutar: `/loop-goal "..."` o `.\harness-executor.ps1 -PlanFile ... -SingleTask <ID>` |
| Ejecutar tools loop | Monitoréalo en el chat. Si algo falla: revisá `campaign_verify` output, corregí, y repetí el `/loop-goal`. |

Cargá las skills campaign-executor, progreso, ponytail (full).

INSTRUCCIONES — EJECUTAR BACKLOG COMPLETO:

Procesás TODAS las tareas del plan file en una sola sesión.
Usás sub-agentes para mantener contexto fresco.

## Flujo

1. LLAMÁ a `campaign_get_task` (sin args)

2. MIENTRAS `hasTask == true`:
   a. Leé `id`, `name`, `state`, `contract`, `recitation`
   b. Spawn UN sub-agente via `task` tool:
      Prompt: "Ejecutá UNA TAREA COMPLETA siguiendo
               `.opencode/prompts/pipeline-full.md`
               Task ID: {id}, Plan file: {plan_path}"
   c. Esperá resultado del sub-agente
   d. Si `result == "completed"`:
      - `campaign_update_task` con `"completed"` y recitation apuntando a próxima
      - `campaign_verify` con el contrato
      - Si verify pasa → continuar. Si no → marcar como stalled
   e. Si `result == "failed"`:
      - Anotar en notas del plan file
      - `campaign_update_task` con `"failed"`
   f. LLAMÁ a `campaign_get_task` de nuevo

3. CUANDO `hasTask == false`:
   - Reportá campaña completada: N/M ✅, K ❌
   - Ejecutá `skill progreso` (migración masiva de todas las completadas)
   - Detenete

REGLAS:
- Budget: máximo 20 iteraciones totales
- Si 3 sub-agentes consecutivos fallan → pausar y preguntar al usuario
- No cambiar scope, no implementar tareas no planificadas
- El sub-agente NO tiene acceso al plan file completo — solo a su task

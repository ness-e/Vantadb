Cargá las skills campaign-executor, progreso, ponytail (full).

INSTRUCCIONES — EJECUTAR BACKLOG COMPLETO:

Procesás TODAS las tareas del plan file en una sola sesión.
Usás sub-agentes para mantener contexto fresco.

Antes de empezar, llamá `campaign_get_next_task` (MCP) para obtener resumen
del plan + próxima tarea. Usá `campaign_stalled_tasks` (MCP) si hay tareas
estancadas.

## Flujo

1. DETECTAR plan file activo con MCP:
   - Llamá `campaign_get_next_task` (MCP) — devuelve plan file + resumen + próxima tarea
   - Si no hay plan file → mostrá error y detenete

2. LEER resumen del plan con `campaign_get_next_task` (MCP):
   - completed/failed/pending count
   - Recitation block (si existe)
   - Próxima tarea pendiente con sus datos

3. ENCONTRAR próxima tarea pendiente vía MCP:
   - `campaign_get_next_task` devuelve la primera ⬜ PENDING o null
   - Si no hay → **campaña completada**. Ejecutá `skill progreso`, detenete.

4. MIENTRAS haya tareas pendientes:
   a. Identificá: `id`, `name`, `contract`, `archivos clave`
   b. Auto-cargá skills para el sub-agente:
      Llamá `campaign_load_skills` (MCP) con los archivos clave para obtener
      los skills exactos a cargar. Incluilos en el prompt del sub-agente.
   c. Spawn UN sub-agente via `task` tool:
      Prompt mínimo (inline, no leas pipeline-full.md):
      "Ejecutá UNA TAREA COMPLETA:
       Task ID: {id}
       Archivos: {archivos clave}
       Contrato: {contract}
       Descripción: {name}

       Skills a cargar: {skills de campaign_load_skills}

       Flujo:
       1. Cargá skills con skill <nombre> (TODOS los listados)
       2. codegraph_explore para blast radius (nombrá los archivos clave)
       3. Si es bug → systematic-debugging
       4. Si es security → doubt-driven-development
       5. Si tiene lógica nueva → test-driven-development
       6. Implementá ~100 líneas, un step
       7. Verify: campaign_verify_cmd (MCP) o cargo check
       8. Si verify falla: retry ladder (feedback → fresh context → strategy distinta → FAILED)
       9. Si pasa: git add -A && git commit -m \"fix({id}): {name}\"
       10. Devolvé: resultado (✅/❌), commit hash, qué se hizo"
   d. Esperá resultado del sub-agente
   e. Si `✅ completed`:
      - `campaign_update_task_state` con `"completed"` y recitation apuntando a próxima
      - `campaign_verify_cmd` con el contrato
      - Si verify pasa → continuar. Si no → marcar como stalled
   f. Si `❌ failed`:
      - Intentá retry con estrategia distinta (máx 1 retry)
      - Si vuelve a fallar → `campaign_update_task_state` con `"failed"`
      - Incrementar contador de consecutivos
   g. Stagnation Detection:
      - Si 3 sub-agentes consecutivos fallan (aún con retry) → NO_PROGRESS_LIMIT
      - Llamá `campaign_stalled_tasks` (MCP) para revisar estado
      - Pausá y preguntá al usuario
   h. Budget ceilings:
      - Max 20 sub-agentes totales (HARD STOP a los 20)
      - Max 3 consecutive fails (HARD STOP → preguntar)
      - Cada sub-agente: max 8 tool calls, ~2 min timeout
   i. ACTUALIZAR checkpoint `docs/pipeline-state.json`:
      ```json
      { "plan": "ruta", "totalCompleted": N, "totalFailed": K, "total": M, "consecutiveFails": C, "lastSync": "ISO" }
      ```
   j. Leer plan file con `campaign_get_next_task` (MCP) y buscar próxima ⬜ PENDING

5. CUANDO no haya más ⬜ PENDING:
   - Reportá campaña completada: N/M ✅, K ❌, stalled: S
   - Ejecutá `skill progreso` (migración masiva de todas las completadas)
   - Detenete

REGLAS:
- Budget: máximo 20 sub-agentes totales, 3 consecutive fails → stall
- Cada sub-agente: máximo 8 tool calls internas — si no responde en ~2 min, killed
- Si 3 sub-agentes consecutivos fallan (aún con retry) → pausar y preguntar al usuario
- No cambiar scope, no implementar tareas no planificadas
- El sub-agente NO tiene acceso al plan file completo — solo a su task
- Tasks sin dependencias ni archivos compartidos → spawn 2 sub-agentes en paralelo
- Stall detection: si 3 tareas consecutivas fallan con mismo error, detener el pipeline

Cargá las skills campaign-executor, progreso, ponytail (full).

INSTRUCCIONES — EJECUTAR BACKLOG COMPLETO:

Procesás TODAS las tareas del plan file en una sola sesión.
Usás sub-agentes para mantener contexto fresco.

Usá `campaign_get_next_task` (MCP) para obtener la próxima tarea, o leé el plan file directamente si necesitás más contexto.

## Flujo

1. DETECTAR plan file activo:
   - Si se especificó `[plan]` como argumento → usá ese
   - Si no → buscá el más reciente en `docs/plans/` por mtime
   - Si no hay plan file → mostrá error y detenete

2. LEER plan file completo con `Read tool`

3. ENCONTRAR próxima tarea pendiente:
   - Buscá la primera tarea con `**Estado:** ⬜ PENDING`
   - Si no hay → **campaña completada**. Ejecutá `skill progreso`, detenete.

4. MIENTRAS haya tareas pendientes:
   a. Identificá: `id`, `name`, `contract`, `archivos clave`
   b. Spawn UN sub-agente via `task` tool:
      Prompt mínimo (inline, no leas pipeline-full.md):
      "Ejecutá UNA TAREA COMPLETA:
       Task ID: {id}
       Archivos: {archivos clave}
       Contrato: {contract}
       Descripción: {name}

       Flujo:
       1. codegraph_explore para blast radius (nombrá los archivos clave)
       2. Si es Rust → skill source-driven-development
       3. Si es bug → skill systematic-debugging
       4. Si es security → skill doubt-driven-development
       5. Si tiene lógica nueva → skill test-driven-development
       6. Implementá ~100 líneas, un step
       7. Verify: cargo check -p <crate>
       8. Si verify falla: retry ladder (feedback → fresh context → strategy distinta → FAILED)
       9. Si pasa: git add -A && git commit -m \"fix({id}): {name}\"
       10. Devolvé: resultado (✅/❌), commit hash, qué se hizo"
   c. Esperá resultado del sub-agente
   d. Si `✅ completed`:
      - `campaign_update_task_state` con `"completed"` y recitation apuntando a próxima
      - `campaign_verify_cmd` con el contrato
      - Si verify pasa → continuar. Si no → marcar como stalled
      - Actualizá el plan file: cambiar Estado a ✅ COMPLETED
   e. Si `❌ failed`:
      - Intentá retry con estrategia distinta (máx 1 retry)
      - Si vuelve a fallar → `campaign_update_task_state` con `"failed"`
      - Anotá en plan file: Estado → ❌ FAILED + nota del error
   f. ACTUALIZAR checkpoint `docs/pipeline-state.json`:
      ```json
      { "plan": "ruta", "completed": [...], "failed": [...], "total": N, "lastSync": "ISO" }
      ```
   g. Leer plan file de nuevo (refresh) y buscar próxima ⬜ PENDING

5. CUANDO no haya más ⬜ PENDING:
   - Reportá campaña completada: N/M ✅, K ❌
   - Ejecutá `skill progreso` (migración masiva de todas las completadas)
   - Detenete

REGLAS:
- Budget: máximo 20 sub-agentes totales
- Cada sub-agente: máximo 8 tool calls internas — si no responde en ~2 min, killed
- Si 3 sub-agentes consecutivos fallan (aún con retry) → pausar y preguntar al usuario
- No cambiar scope, no implementar tareas no planificadas
- El sub-agente NO tiene acceso al plan file completo — solo a su task
- Tasks sin dependencias ni archivos compartidos → spawn 2 sub-agentes en paralelo

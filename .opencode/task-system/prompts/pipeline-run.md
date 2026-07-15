Cargá las skills campaign-executor, progreso, ponytail (full).

INSTRUCCIONES — EJECUTAR BACKLOG COMPLETO:

Procesás TODAS las tareas del plan file en una sola sesión.
Usás sub-agentes para mantener contexto fresco.

Parámetros:
- FAIL_MODE: `stop` | `skip` | `parallel` (default: `stop`)
  - `stop`: se para ante la primera falla
  - `skip`: registra fallo y sigue
  - `parallel`: ejecuta tareas independientes en paralelo vía waves

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

3. PROBES DE INTEGRIDAD (antes de empezar):
   - Validá: (a) plan file existe y tiene tasks, (b) recitation block es legible,
     (c) última tarea no es la misma dos veces seguidas sin progreso,
     (d) plan file no tiene harness PID activo de otra sesión,
     (e) git status está limpio o los cambios son del pipeline actual
   - Si alguna probe falla → preguntá al usuario antes de continuar

4. ENCONTRAR próxima tarea pendiente vía MCP:
   - `campaign_get_next_task` devuelve la primera ⬜ PENDING o null
   - Si no hay → **campaña completada**. Ejecutá `skill progreso`, detenete.

5. MIENTRAS haya tareas pendientes:
   a. Identificá: `id`, `name`, `contract`, `archivos clave`
   b. Si FAIL_MODE=parallel y hay ≥2 tareas independientes → saltá a **Paso 6 (waves paralelas)**
   c. Auto-cargá skills para el sub-agente:
      Llamá `campaign_load_skills` (MCP) con los archivos clave para obtener
      los skills exactos a cargar. Incluilos en el prompt del sub-agente.
    d. RESEARCH ISOLATION: Si la tarea requiere leer muchos archivos (3+)
       o documentación extensa, spawné PRIMERO un sub-agente de research:
       Prompt: "research-agent.md (lee {archivos clave} o la documentación necesaria,
       devolvé solo un Digest en el formato especificado)"
       → Guardá el digest en memoria o pasalo al sub-agente siguiente
    e. Spawn UN sub-agente via `task` tool:
       Prompt mínimo (inline, no leas pipeline-full.md):
       "Ejecutá UNA TAREA COMPLETA:
        Task ID: {id}
        Archivos: {archivos clave}
        Contrato: {contract}
        Descripción: {name}
       Context Save: sí (escribe ## Context Save al final del task file)

        Research Digest: {si hay digest del paso research, incluílo aquí}

        Skills a cargar: {skills de campaign_load_skills}

        Flujo:
        1. Cargá skills con skill <nombre> (TODOS los listados)
       2. codegraph_explore para blast radius (nombrá los archivos clave)
       3. Zero-code planning: describí solución en ≤3 viñetas primero
       4. Si es bug → systematic-debugging
       5. Si es security → doubt-driven-development
       6. Si tiene lógica nueva → test-driven-development
       7. Implementá ~100 líneas, un step
       8. Verify: campaign_verify_cmd (MCP)
       9. Si verify falla: retry ladder (feedback → fresh context → strategy → FAILED)
       10. Si pasa: git add -A && git commit -m \"{type}({id}): {name}\"
       11. Escribí Context Save Point en task file (decisiones, problemas)
       12. Devolvé: resultado (✅/❌), commit hash, qué se hizo, archivos tocados"
   e. Esperá resultado del sub-agente
   f. Si `✅ completed`:
      - `campaign_update_task_state` con `"completed"` y recitation apuntando a próxima
      - `campaign_verify_cmd` con el contrato (doble verificación)
      - Si verify pasa → incrementar totalCompleted, reset consecutiveFails
      - Si verify NO pasa → marcar como stalled, no contar como completed
      - **Revisión cada 5 tareas:** si totalCompleted % 5 == 0, releé el plan file
        completo y verificá: (a) estados consistentes, (b) recitation legible,
        (c) no hay duplicados en progreso. Anotá "Review N/5: OK" en el plan.
   g. Si `❌ failed`:
      - Intentá retry con estrategia distinta (máx 1 retry)
      - Si vuelve a fallar → `campaign_update_task_state` con `"failed"`
      - Incrementar totalFailed y consecutiveFails
      - Si FAIL_MODE=stop → detener el pipeline
      - Si FAIL_MODE=skip → registrar y continuar
      - Si consecutiveFails >= 3 → FAIL_MODE pasa a "stop" forzosamente
   h. Stagnation Detection:
      - Si 3 sub-agentes consecutivos fallan (aún con retry) → NO_PROGRESS_LIMIT
      - Llamá `campaign_stalled_tasks` (MCP) para revisar estado
      - Pausá y preguntá al usuario
   i. Budget ceilings:
      - Max 20 sub-agentes totales (HARD STOP a los 20)
      - Max 3 consecutive fails (HARD STOP → preguntar)
      - Cada sub-agente: max 8 tool calls, ~2 min timeout
   j. ACTUALIZAR checkpoint `docs/pipeline-state.json`:
      ```json
      { "plan": "ruta", "totalCompleted": N, "totalFailed": K, "total": M, "consecutiveFails": C, "failMode": "stop|skip", "lastSync": "ISO" }
      ```
   k. Leer plan file con `campaign_get_next_task` (MCP) y buscar próxima ⬜ PENDING

6. WAVES PARALELAS (FAIL_MODE=parallel):
   a. Construí DAG de dependencias entre las N tareas pendientes:
      - Tarea A tiene `depende de X` → arista X → A
      - codegraph_explore en archivos de cada tarea para detectar conflictos
      - Si dos tareas tocan archivos diferentes y no hay arista → paralelizable
   b. Agrupá por waves:
      ```
      Wave 0: tareas sin dependencias
      Wave 1: tareas que dependen de Wave 0
      Wave 2: tareas que dependen de Wave 1
      ```
   c. MAX_CONCURRENT = min(4, tareas_en_wave)
   d. Por cada wave: spawn N sub-agentes en paralelo (task tool), esperá que
      todos terminen, procesá resultados individuales (mismo que paso 5.f/5.g)
   e. Si una tarea falla en parallel → las demás de la wave terminan, waves
      siguientes NO arrancan. Reporte parcial.

7. CUANDO no haya más ⬜ PENDING:
   - Reportá campaña completada: N/M ✅, K ❌, stalled: S
   - Ejecutá `skill progreso` (migración masiva de todas las completadas)
   - Si FAIL_MODE=parallel: verificá que no haya conflictos entre ramas paralelas
     (`git log --oneline` después del último commit secuencial)
   - Detenete

REGLAS:
- FAIL_MODE=stop: primera falla → detener
- FAIL_MODE=skip: fallas registradas, sigue. Si 3 consecutivas → pasa a stop forzoso
- FAIL_MODE=parallel: waves con MAX_CONCURRENT=4, DAG de dependencias
- Budget: máximo 20 sub-agentes totales, 3 consecutive fails → stall
- Cada sub-agente: máximo 8 tool calls internas — si no responde en ~2 min, killed
- Si 3 sub-agentes consecutivos fallan (aún con retry) → pausar y preguntar al usuario
- No cambiar scope, no implementar tareas no planificadas
- El sub-agente NO tiene acceso al plan file completo — solo a su task
- Revisión cada 5 tareas: checkpoint de consistencia de artefactos
- Stall detection: si 3 tareas consecutivas fallan con mismo error, detener el pipeline
- Context Save Point: cada sub-agente lo escribe al final de su task file

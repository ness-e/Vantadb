> **ACTIVE INSTRUCTION — Execute Full Backlog**
> Activado por `/pipeline run [plan]`.
> Path resolution: `skills/X` → `.opencode/skills/X/`, `prompts/X.md` → `.opencode/task-system/prompts/X.md`
> Procesar TODAS las tareas del plan file en una sesión usando sub-agentes.
> **Profundidad UNIFICADA con `/pipeline task`:** cada tarea pasa por
> DISCOVERY (task file con steps atómicos, blast radius, skills) → EJECUCIÓN → CIERRE
> vía `pipeline-full.md`. NUNCA prompts inline de 7 líneas.
> Cada sub-agente se ejecuta con `subagent_type` según el campo `Ruta` del plan.
> Si un sub-agente devuelve resultado incompleto/fallido/detenido → `prompts/subagent-recovery.md` (SARL).
> Al finalizar: skill progreso, checkpoint, detenerse.
> Si FAIL_MODE=stop: detener en primera falla. Si skip: registrar y seguir.

Las skills base (campaign-executor, progreso, ponytail) se cargan automáticamente vía MCP — no las cargues manualmente.

INSTRUCCIONES — EJECUTAR BACKLOG COMPLETO:

Procesás TODAS las tareas del plan file en una sola sesión.
Usás sub-agentes para mantener contexto fresco.

Parámetros:
- FAIL_MODE: `stop` | `skip` | `parallel` (default: `parallel`)
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

3. TRACKING DE SESIÓN:
   - Llamá `campaign_session_track` (MCP) con `action: "create"` y `sessionId` único al inicio
   - En cada tarea completada → `campaign_session_track` con `action: "update"` para registrar progreso
   - Al finalizar → `campaign_session_track` con `action: "update"` y estado final

4. PROBES DE INTEGRIDAD (antes de empezar):
- Validá: (a) plan file existe y tiene tasks, (b) recitation block es legible,
      (c) última tarea no es la misma dos veces seguidas sin progreso,
      (d) git status está limpio o los cambios son del pipeline actual
   - Si alguna probe falla → preguntá al usuario antes de continuar

5. ENCONTRAR próxima tarea pendiente vía MCP:
   - `campaign_get_next_task` devuelve la primera ⬜ PENDING o null
   - Si no hay → **campaña completada**. Ejecutá `skill progreso`, detenete.

6. MIENTRAS haya tareas pendientes — por cada tarea (profundidad completa):

   a. **Identificá:** `id`, `name`, `contract`, `archivos clave`, y el campo `Ruta` (sub-agente destino).
      Si la tarea YA tiene task file (`.opencode/skills/campaign-executor/tasks/<id>.md`),
      leelo para saber dónde quedó (steps ✅ / ⬜ / Context Save Point).

   b. Si FAIL_MODE=parallel y hay ≥2 tareas independientes → saltá a **Paso 7 (waves paralelas)**.

   c. **Warmer de skills (orquestador):**
      Llamá `campaign_load_skills` (MCP) con los archivos clave → skills + checks exactos.
      Incluilos en el prompt del sub-agente (el sub-agente también se los auto-carga
      dentro de `pipeline-full.md` — no dupliques carga de skills, solo pasalos como contexto).

   d. **RESEARCH ISOLATION:** si la tarea requiere leer muchos archivos (3+) o documentación
      extensa, spawné PRIMERO un sub-agente de research que devuelva un Digest:
      Prompt: "research-agent.md (lee {archivos clave} o la documentación necesaria,
      devolvé solo un Digest en el formato especificado)"
      → Guardá el digest y pasalo al sub-agente de ejecución (evita robarle contexto).

   e. **ROUTING por `Ruta` — nunca sub-agentes genéricos.**
      Mapeá el campo `Ruta` del plan a `subagent_type`:

      | `Ruta` en el plan | `subagent_type` |
      |---|---|
      | vanta-worker | `vanta-worker` |
      | vanta-tuner | `vanta-tuner` |
      | vanta-engine | `vanta-engine` |
      | vanta-arch | `vanta-arch` |
      | vanta-audit | `vanta-audit` |
      | vanta-chaos | `vanta-chaos` |
      | vanta-docs | `vanta-docs` |
      | vanta-lead / CI / release / packaging | `vanta-lead` |
      | (sin Ruta o desconocida) | `campaign_detect_task_type` (MCP) → rust: `vanta-worker`, frontend: `general`, python/ts: `vanta-worker`, server/security: `vanta-audit`, docs: `vanta-docs`, default: `general` |

      Si la `Ruta` trae paréntesis (ej: "vanta-worker (con revisión vanta-audit)"),
      tomá SOLO el primer token. La revisión secundaria la hace tu post-check (paso i).

   f. **Spawn UN sub-agente de EJECUCIÓN** via `task` tool cuyo prompt SIEMPRE referencia
      `pipeline-full.md` — misma profundidad que `/pipeline task`:

      ```
      "Cargá .opencode/task-system/prompts/pipeline-full.md y ejecutá UNA TAREA COMPLETA:
       Task ID: {id}
       Plan file: {ruta del plan}
       Archivos clave: {archivos clave}
       Contrato: {contract}
       Descripción: {name}
       Task file: .opencode/skills/campaign-executor/tasks/{id}.md
       Research Digest: {si se generó en d, incluilo aquí}
       Skills a cargar: {skills de campaign_load_skills}

       Reglas del prompt:
       1. Seguí pipeline-full.md al pie de la letra (DISCOVERY → EJECUCIÓN → CIERRE).
       2. Si el task file no existe → crealo en DISCOVERY (steps atómicos, blast radius,
          web research si hay ambigüedad, contrato, herramientas, Context Save Point).
       3. Si el task file ya existe → NO re-hagas steps ya ✅; continuá desde el primer
          step ⬜ PENDING. No pierdas trabajo hecho.
       4. Cada step: ~100 líneas, verify mecánico con campaign_verify_cmd.
       5. Cierre: verify full (fmt/clippy/nextest/docs) → commit conventional con task ID.
       6. Al final devolvé SIEMPRE el bloque RESULTADO estructurado (ver § Resultado).
       7. Si no podés terminar en el presupuesto, devolvé el trabajo hecho + el próximo
          step — nunca te detengas en silencio."
      ```

   g. Esperá resultado del sub-agente.

   h. **CLASIFICÁ el resultado** según `prompts/subagent-recovery.md` (SARL):
      - `✅ COMPLETO` → pasá a (i)
      - `🟡 INCOMPLETO` / `❌ FALLIDO` / `⚠️ SIN-FORMATO` / salida vacía / "se detuvo solo"
        → aplicá la escalera SARL en orden:
        1. **RESUME** misma sesión: `task(task_id=<id del sub-agente>, subagent_type=<mismo>,
           prompt="<feedback procesado + próximo step ⬜ PENDING del task file + pedí RESULTADO>")`
        2. **RETRY** fresco: nuevo sub-agente del mismo tipo con digest ~200 tokens.
        3. **STRATEGY** distinta (puede escalar con `campaign_mom_escalate`).
        4. **ESCALATE** a humano: documentar, commit WIP, `campaign_update_task_state "failed"`.
      - Cada recovery consume `campaign_budget_consume resource="fail"`.
      - Si 3 resultados no-DONE seguidos en la misma tarea → pausá y preguntá al usuario.

   i. Si `✅ completed` (post-recovery):
      - `campaign_update_task_state` con `"completed"` y recitation apuntando a próxima
      - `campaign_verify_cmd` con el contrato (doble verificación — sin verify no cuenta)
      - Si verify pasa → incrementar totalCompleted, reset consecutiveFails
      - Si verify NO pasa → tratalo como INCOMPLETE y volvé a (h)
      - Revisión secundaria según la `Ruta` (ej: "con revisión vanta-audit") → podés forkear
        `vanta-audit` con `task` para validar el diff sobre el resultado antes de cerrar.
      - **Revisión cada 5 tareas:** si totalCompleted % 5 == 0, releé el plan file
        completo y verificá: (a) estados consistentes, (b) recitation legible,
        (c) no hay duplicados en progreso. Anotá "Review N/5: OK" en el plan.

   j. Si agotaste la escalera con ❌:
      - `campaign_update_task_state` con `"failed"`
      - Incrementar totalFailed y consecutiveFails
      - Si FAIL_MODE=stop → detener el pipeline
      - Si FAIL_MODE=skip → registrar y continuar
      - Si consecutiveFails >= 3 → FAIL_MODE pasa a "stop" forzosamente

   k. Stagnation Detection:
      - Si 3 sub-agentes consecutivos fallan (aún con SARL) → NO_PROGRESS_LIMIT
      - Llamá `campaign_stalled_tasks` (MCP) para revisar estado
      - Pausá y preguntá al usuario

   l. Budget ceilings:
      - Max 20 sub-agentes totales (HARD STOP a los 20) — cada intento de la escalera cuenta
      - Max 3 consecutive fails (HARD STOP → preguntar)
      - Cada sub-agente: max 8 tool calls, ~2 min timeout

   m. ACTUALIZAR checkpoint `docs/pipeline-state.json`:
      ```json
      { "plan": "ruta", "totalCompleted": N, "totalFailed": K, "total": M, "consecutiveFails": C, "failMode": "stop|skip", "lastSync": "ISO" }
      ```
   n. Leer plan file con `campaign_get_next_task` (MCP) y buscar próxima ⬜ PENDING

7. WAVES PARALELAS (FAIL_MODE=parallel):
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
   c. MAX_CONCURRENT = min(3, tareas_en_wave)  # 3 por RAM en Windows (mismo límite que iter-loop-tools.md)
   d. Por cada wave: spawn N sub-agentes en paralelo (task tool) con el MISMO prompt de
      profundidad completa del paso 6.f (pipeline-full.md) y routing del 6.e.
      Esperá que todos terminen, clasificá cada resultado con SARL (6.h) individualmente.
   e. Si una tarea falla en parallel → las demás de la wave terminan, waves siguientes NO
      arrancan. Reporte parcial. La escalera SARL se aplica por sub-agente dentro de la wave.

8. CUANDO no haya más ⬜ PENDING:
   - Reportá campaña completada: N/M ✅, K ❌, stalled: S
   - Ejecutá `skill progreso` (migración masiva de todas las completadas)
   - **RETROSPECTIVA de cierre del plan/milestone** (obligatoria antes de detenerte):
     - Produci la retrospectiva **Start/Stop/Continue** + 1 acción medida:
       - **Start** (seguir haciendo): qué funcionó y se mantiene
       - **Stop** (dejar de hacer): qué no funcionó y se elimina
       - **Continue** (continuar): qué se sigue haciendo igual
       - **UNA acción de mejora de proceso medible** con métrica contra baseline.
         Ej: "reducir verify retries de 3 a 1 por tarea" (métrica: retries/tarea).
         Cuando aplique, usá como baseline natural la North Star de
         `.opencode/skills/campaign-executor/RULES.md`: tasa de completado >90%
         en primer intento, falsos positivos 0, regresión 0.
     - Registrala en el plan y pasala a `progreso` (Trigger 1.D2) para que quede
       archivada junto con el plan, no solo movidas las tareas.
   - Si FAIL_MODE=parallel: verificá que no haya conflictos entre ramas paralelas
     (`git log --oneline` después del último commit secuencial)
   - Detenete

REGLAS:
- **Profundidad unificada:** cada sub-agente sigue `pipeline-full.md` (DISCOVERY → EJECUCIÓN → CIERRE).
  NUNCA prompts inline. El task file es el estado durable — si existe, continuá, no re-creees.
- **Routing por `Ruta`:** `subagent_type` sale del campo `Ruta` del plan (tabla 6.e).
- **Recuperación:** cualquier resultado no-DONE → `prompts/subagent-recovery.md` (RESUME misma sesión
  → RETRY fresco → STRATEGY → ESCALATE). Nunca tratar INCOMPLETE como FAILED.
- **Preservación:** nunca borrar/rehacer trabajo del sub-agente. El Context Save Point + steps del
  task file son la fuente de verdad para reanudar.
- FAIL_MODE=stop: primera falla → detener
- FAIL_MODE=skip: fallas registradas, sigue. Si 3 consecutivas → pasa a stop forzoso
- FAIL_MODE=parallel: waves con MAX_CONCURRENT=3, DAG de dependencias
- Budget: máximo 20 sub-agentes totales (cada intento de la escalera cuenta), 3 consecutive fails → stall
- Cada sub-agente: máximo 8 tool calls internas — si no responde en ~2 min, killed
- Si 3 sub-agentes consecutivos fallan (aún con retry) → pausar y preguntar al usuario
- No cambiar scope, no implementar tareas no planificadas
- El sub-agente NO tiene acceso al plan file completo — solo a su tarea
- Revisión cada 5 tareas: checkpoint de consistencia de artefactos
- Stall detection: si 3 tareas consecutivas fallan con mismo error, detener el pipeline
- Context Save Point: cada sub-agente lo escribe al final de su task file
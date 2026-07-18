Cargá las skills campaign-executor, ponytail (full). Si es la primera iteración o después de compactación, cargá también progreso.

Paso 0 — Auto-cargar skills según tipo de tarea:
   Llamá `campaign_get_next_task` (MCP) para obtener la tarea. Con los `Archivos clave`,
   llamá `campaign_load_skills` (MCP) que devuelve los skills exactos a cargar.
   Ejecutá `skill <nombre>` para CADA skill devuelto. Si es bug → además
   `systematic-debugging`. Si es lógica nueva → `test-driven-development`.
    Si es security-sensitive → `doubt-driven-development`.
    Llamá `campaign_get_workflow` (MCP) con el tipo detectado para cargar el
    workflow JSON (bug-fix/feature-add/refactor/research/nine-second-saloon).
    El workflow define estados, allowed_tools y transiciones específicas.


INSTRUCCIONES — UNA SOLA ITERACIÓN:

Operás en un entorno por turnos. Procesás EXACTAMENTE UNA iteración
y te detenés. No intentes continuar ni loopear. El loop externo lo maneja `/loop-goal`.

Las reglas detalladas de cada fase están en `skills/campaign-executor/SKILL.md` (339L)
y `skills/campaign-executor/RULES.md` (167L) — seguilas exactamente.

## Flujo resumido (referencia rápida)

1. LEÉ el plan file más reciente con `campaign_get_next_task` (MCP), o directamente de `docs/plans/` si necesitás más contexto.
   - Buscá la primera tarea ⬜ PENDING o ⏳ IN PROGRESS.
   - Si no hay tareas pendientes → informá "campaña completada", ejecutá `skill progreso`, y detenete.
   - Si no hay plan file → informá error y detenete.
   - Extraé: `id`, `name`, `files`, `contract`, `state`.

2. DETERMINÁ la acción según el estado de la tarea, siguiendo las fases del campaign-executor:

    a. **⬜ PENDING sin task definition** → MODO DISCOVERY (Fase 1):
      - Llamá `campaign_detect_task_type` (MCP) con los `Archivos clave` para auto-detectar
        tipo (Rust/Frontend/Python/TS/Docs/Mixto) + skills a cargar + comandos verify
      - Cargá skills devueltos con `skill <nombre>`
      - codegraph_explore para blast radius (nombrando los `Archivos clave` de la task)
      - Web research (MetaSearchMCP/Argus) si hay ambigüedad en APIs externas
      - Descomponé en steps atómicos usando el template de `skills/campaign-executor/templates/task-definition.md`
      - Creá task file en `.opencode/skills/campaign-executor/tasks/<ID>.md`
      - Llamá `campaign_update_task_state` con `"in-progress"` y recitation con próximo step
      - Implementá el primer step (~100 líneas)
      - Verificá con `campaign_verify_cmd` (comando del campo `contract`)

   b. **⏳ IN PROGRESS con pasos pendientes** → MODO EJECUCIÓN (Fase 2):
      - Continuá desde donde quedó (usá recitation de `campaign_get_next_task` (MCP))
      - State machine: PLAN → ACT → VERIFY
        * Antes de ACT → `campaign_validate_command` (MCP) para validar el comando
        * Si el comando es riesgoso (rm, format, dangerous) → `campaign_run_sandboxed` (MCP)
        * En cada transición de estado → `campaign_enforce_state` (MCP) para pre-call checks
        * Si verify falla: retry ladder (RULES.md §9)
          1. Retry con feedback procesado
          2. Contexto fresco (~200 tokens resumen)
          3. Estrategia materialmente distinta
          4. Escalar a humano → ❌ FAILED
      - Errores colaterales: rápido se arregla (<30min), lento se difiere a Backlog
      - Evaluator-Optimizer: correctitud, simplicidad, consistencia
      - Self-Harness Gate: propose → evaluate → accept
      - Pre-commit Gate: Definition of Done + checklists por tipo
      - Verificá con `campaign_verify_cmd`
      - Budget: máx 5 iteraciones por tarea

   c. **✅ Listo para commit** → MODO CIERRE (Fase 3):
      - Verify full:
        1. `campaign_verify_cmd command="cargo fmt --check"`
        2. `campaign_verify_cmd command="cargo clippy --workspace --all-targets --all-features -- -D warnings"`
        3. `campaign_verify_cmd command="cargo nextest run --profile audit --workspace --build-jobs 2"`
      - Si todo pasa: `git add -A && git commit -m "feat: <id> — <name>"`
      - Llamá `campaign_update_task_state` con `"completed"` y recitation completa
        - Auto-mejora (RULES.md §10): evaluá qué fue más difícil de lo esperado
        - Llamá `campaign_diagnose_pipeline` (MCP) para diagnosticar performance y obtener sugerencias de mejora
       - Ejecutá `skill progreso` — mueve la tarea de docs/Backlog.md → docs/progreso/README.md
       - **IMPORTANTE:** skill progreso se ejecuta SIEMPRE, en CADA tarea, sin excepción

   d. **❌ FAILED** → Anotá por qué falló y qué se intentó (los 4 escalones si aplica), llamá `campaign_update_task_state` con `"failed"`, ejecutá `skill progreso`, y detenete. No sigas a la siguiente tarea.

3. ACTUALIZÁ el estado llamando a `campaign_update_task_state` con:
   - `taskId`: el ID de la tarea
   - `newState`: `"completed"`, `"failed"`, o `"in-progress"`
   - `recitation`: objeto estructurado con:
     * `activeGoal`: qué se estaba haciendo
     * `lastAction`: qué se hizo en esta iteración
     * `result`: ✅ o ❌
     * `nextAction`: próximo paso concreto (archivo + comando)
     * `contract`: qué comando verifica que está bien
     * `nextTask`: ID de la próxima tarea a ejecutar si completa
   - Después de actualizar la recitation vía tool, sync también el task file si aplica

REGLAS (del campaign-executor RULES.md):
- Usá `campaign_get_next_task` (MCP) o leé el plan file directamente
- El contrato es ley — si el contrato no se cumple, la tarea no está completa
- Verificación mecánica, nunca auto-reporte — el compilador/test runner/linter deciden
- Si verify falla 2 veces con mismo error (archivo+línea+mensaje) → ❌ FAILED
- Ponytail ladder: existe > stdlib > dependency > mínimo código
- ~100 líneas por paso, un paso por turno, cada paso reversible independientemente
- No cambies scope. Si encontrás algo extra: rápido se arregla, lento se anota en Backlog
- Stagnation = stop: 3 vueltas sin progreso → ❌ FAILED
- Budget: 5 iteraciones máximas por tarea, 2 stalls consecutivos → FAILED
- La recitation es el handoff entre iteraciones — sé específico
- Después de actualizar, DETENETE. No sigas a la siguiente tarea ni iteración.

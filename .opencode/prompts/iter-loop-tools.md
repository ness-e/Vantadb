Cargá las skills campaign-executor, ponytail (full). Si es la primera iteración o después de compactación, cargá también progreso.

INSTRUCCIONES — UNA SOLA ITERACIÓN:

Operás en un entorno por turnos. Procesás EXACTAMENTE UNA iteración
y te detenés. No intentes continuar ni loopear. El loop externo lo maneja `/loop-goal`.

Las reglas detalladas de cada fase están en `skills/campaign-executor/SKILL.md` (339L)
y `skills/campaign-executor/RULES.md` (167L) — seguilas exactamente.

## Flujo resumido (referencia rápida)

1. LLAMÁ a `campaign_get_task` (sin args — detecta automáticamente el plan file más reciente).
   - Si `hasTask == false` → informá "campaña completada", ejecutá `skill progreso`, y detenete.
   - Si devuelve error → informalo y detenete.
   - La task devuelve `id`, `name`, `files`, `contract`, `state`, y `summary` (completed/failed/pending/total).

2. DETERMINÁ la acción según el estado de la tarea, siguiendo las fases del campaign-executor:

   a. **⬜ PENDING sin task definition** → MODO DISCOVERY (Fase 1):
      - Auto-detectá tipo de tarea (Rust/Frontend/Python/TS/Docs/Mixto) según `Archivos clave`
      - Cargá skills adicionales según tipo (RULES.md §10):
        * Rust → `source-driven-development`
        * Frontend → `frontend-ui-engineering`
        * Bug reportado → `systematic-debugging`
        * API pública → `api-and-interface-design`
      - codegraph_explore para blast radius (nombrando los `Archivos clave` de la task)
      - Web research (MetaSearchMCP/Argus) si hay ambigüedad en APIs externas
      - Descomponé en steps atómicos usando el template de `skills/campaign-executor/templates/task-definition.md`
      - Creá task file en `.opencode/skills/campaign-executor/tasks/<ID>.md`
      - Llamá `campaign_update_task` con `"in-progress"` y recitation con próximo step
      - Implementá el primer step (~100 líneas)
      - Verificá con `campaign_verify` (comando del campo `contract`)

   b. **⏳ IN PROGRESS con pasos pendientes** → MODO EJECUCIÓN (Fase 2):
      - Continuá desde donde quedó (usá recitation de `campaign_get_task`)
      - State machine: PLAN → ACT → VERIFY
        * Si verify falla: retry ladder (RULES.md §9)
          1. Retry con feedback procesado
          2. Contexto fresco (~200 tokens resumen)
          3. Estrategia materialmente distinta
          4. Escalar a humano → ❌ FAILED
      - Errores colaterales: rápido se arregla (<30min), lento se difiere a Backlog
      - Evaluator-Optimizer: correctitud, simplicidad, consistencia
      - Self-Harness Gate: propose → evaluate → accept
      - Pre-commit Gate: Definition of Done + checklists por tipo
      - Verificá con `campaign_verify`
      - Budget: máx 5 iteraciones por tarea

   c. **✅ Listo para commit** → MODO CIERRE (Fase 3):
      - Verify full:
        1. `campaign_verify command="cargo fmt --check"`
        2. `campaign_verify command="cargo clippy --workspace --all-targets --all-features -- -D warnings"`
        3. `campaign_verify command="cargo nextest run --profile audit --workspace --build-jobs 2"`
      - Si todo pasa: `git add -A && git commit -m "feat: <id> — <name>"`
      - Llamá `campaign_update_task` con `"completed"` y recitation completa
      - Auto-mejora (RULES.md §10): evaluá qué fue más difícil de lo esperado
      - Ejecutá `skill progreso`

   d. **❌ FAILED** → Anotá por qué falló y qué se intentó (los 4 escalones si aplica), llamá `campaign_update_task` con `"failed"`, y detenete. No sigas a la siguiente tarea.

3. ACTUALIZÁ el estado llamando a `campaign_update_task` con:
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
- NO leas el plan file directamente — usá `campaign_get_task` para todo
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

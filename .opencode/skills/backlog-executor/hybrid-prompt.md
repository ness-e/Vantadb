Cargá las skills backlog-executor, task-executor, ponytail (full).

Archivos de referencia (leer al inicio):
- `{{PLAN_FILE}}` — plan file con las 98 tareas
- `.opencode/skills/task-executor/VISION.md` — north star del executor
- `.opencode/skills/task-executor/SKILL.md` — fases detalladas (Fase 0-6)
- `.opencode/AGENTS.md` — tabla de skills por fase

INSTRUCCIONES — UNA SOLA ITERACIÓN:

Operás en un entorno por turnos. Procesás EXACTAMENTE UNA iteración
y te detenés. No intentes continuar ni loopear.

1. Leé el plan file COMPLETO (`{{PLAN_FILE}}`).
2. Buscá la recitation block o la primera tarea ⬜ PENDING / ⏳ IN PROGRESS.
3. Determiná la PRÓXIMA ACCIÓN CONCRETA según el estado de la tarea:

   a. ⬜ PENDING sin task definition creada → **task-executor Fase 0-1**:
      - skill progreso
      - codegraph_explore para blast radius (callers, callees, implicaciones)
      - Auto-detectar tipo: Rust / Frontend / Python / TS / Docs
      - Auto-detectar checks según tipo
      - Definir contrato verificable
      - Descomponer en pasos atómicos (~100 líneas c/u)
      - Crear `.opencode/skills/task-executor/tasks/<ID>.md`

   b. ⏳ IN PROGRESS con pasos pendientes → **task-executor Fase 2**:
      - Cargar skills según tipo detectado (source-driven-development para Rust,
        frontend-ui-engineering para web, etc.)
      - Ponytail ladder: ya existe > stdlib > dependency > mínimo código
      - Un paso atómico por iteración (~100 líneas)
      - State machine: PLAN → ACT → VERIFY
      - Si verify falla: retry ladder (1 error feedback, 2 mismo error = ❌ FAILED)
      - Stagnation detection: 3 intentos mismo error = ❌ FAILED

   c. Si verify pasa ✅ → **task-executor Fase 3-4**:
      - Verificación full: cargo build + nextest + fmt + clippy + tsc
      - Si el código contiene `unsafe` o concurrencia: ejecutar `cargo +nightly miri test`
      - Si el componente es crítico (parser, serializador, scheduler): marcar para fuzzing en CI
      - codegraph_explore para verificar impacto completo
      - Evaluator-optimizer: auto-crítica 3 ejes (correctitud, simplicidad, consistencia)
      - Self-Harness gate: propose → evaluate → accept/reject
      - Si reject → volver a implementar

   d. Si pasa el gate ✅ → **task-executor Fase 6**:
      - git add -p + git commit con mensaje estructurado
      - skill progreso (Trigger 1: migrar a progreso/README.md)
      - Actualizar plan file: marcar ✅, commit hash, notas
      - Errores colaterales: rápidos (<30min) arreglar, lentos → Backlog.md

4. **ESCRIBÍ EL RECITATION BLOCK** al final del plan file:

   ```
   === RECITATION ===
   Objetivo activo: TASK-NN — Ref ID
   Estado: discovery / definition / implementing / verifying / review / completed / failed
   Última acción: qué se acaba de hacer
   Resultado: ✅ / ❌ (con error)
   Próxima acción: el PRÓXIMO paso concreto (archivo + comando)
   Contrato: "condición verificable"
   Próxima tarea si completa: TASK-NN+1 — Ref ID
   === END RECITATION ===
   ```

5. **DETENETE.** No sigas a la siguiente tarea ni iteración.

REGLAS GLOBALES:
- Ponytail full: ya existe > stdlib > dependency instalada > mínimo código
- Verify = comando mecánico real (nunca auto-reporte)
- Verify falla 2 veces con mismo error (archivo+línea+mensaje) → ❌ FAILED
- No cambies scope. Si encontrás algo extra → anotalo, no lo implementes
- Stagnation: 3 intentos mismo error = ❌ FAILED
- Cada acción termina con plan file actualizado + recitation escrita

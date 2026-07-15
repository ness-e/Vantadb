Cargá las skills backlog-executor, task-executor, ponytail (full).

Archivos de referencia (leer al inicio, priorizando los 3 primeros):
- **Plan file:** `{{PLAN_FILE}}` (inyectado por el harness)
- `.opencode/skills/task-executor/VISION.md` — north star del executor
- `.opencode/skills/task-executor/SKILL.md` — fases detalladas (Fase 0-6)
- `.opencode/AGENTS.md` — tabla de skills por fase

**Presupuesto de contexto — reglas estrictas:**
- Uso inicial < 20% (~40k tokens).

**Map-Reduce determinista (no LLMs para leer código):**
- **Código fuente → CodeGraph** (determinista, AST-level, 0 tokens, 0 alucinación).
  `codegraph_explore` devuelve source verbatim + call paths + blast radius en
  una llamada. Úsalo para toda exploración de código. NO leas archivos .rs, .ts,
  .py directamente como contexto — deja que CodeGraph extraiga solo lo necesario.
- **Prosa/documentación no indexada → sub-agentes LLM** (`task` + `general`).
  Usalos solo para archivos que CodeGraph no indexa. Cada sub-agente lee un
  archivo y devuelve un resumen enfocado en 3-5 líneas:
  - `.opencode/skills/task-executor/VISION.md` → north star, principios rectores
  - `.opencode/skills/task-executor/SKILL.md` → fases aplicables, reglas de
    verify, reglas de commit
  - `.opencode/AGENTS.md` → tabla de skills por fase
- **Stale cache:** Si `codegraph_explore` muestra la advertencia
  `⚠️ Some files referenced below were edited since the last index sync…`,
  leé SOLO esos archivos directamente (no todo el repo).
- No cargues MCPs que no uses para esta tarea (Argus, Discord, Pencil, etc.)
- Preferí parches cortos (`edit` con oldString/newString) sobre sobreescribir
  archivos completos. Un diff de 5 líneas vs reescribir 500 = ahorro masivo.
- Cuando una tarea completa ✅ y commitea → la sesión se cierra.
  La siguiente tarea arranca sesión limpia (contexto fresco).

INSTRUCCIONES — UNA SOLA ITERACIÓN:

Operás en un entorno por turnos. Procesás EXACTAMENTE UNA iteración
y te detenés. No intentes continuar ni loopear.

1. Leé el plan file de `{{PLAN_FILE}}` COMPLETO. No uses sub-agentes para esto.
2. Buscá la recitation block o la primera tarea ⬜ PENDING / ⏳ IN PROGRESS.
3. Determiná la PRÓXIMA ACCIÓN CONCRETA según el estado de la tarea:

    a. ⬜ PENDING sin task definition creada → **task-executor Fase 0-1**:
       - skill progreso
       - codegraph_explore para blast radius (callers, callees, implicaciones)
       - Auto-detectar tipo: Rust / Frontend / Python / TS / Docs
       - Auto-detectar checks según tipo
       - **Planificar primero (cero código inicial):** describí la solución
         conceptualmente en ≤3 viñetas de pseudocódigo. Sin tocar archivos
         todavía. Recién después de la aprobación implícita → definición.
       - Definir contrato verificable
       - Descomponer en pasos atómicos (~100 líneas, ~5 line diff efectivo c/u)
       - Crear `.opencode/skills/task-executor/tasks/<ID>.md`

    b. ⏳ IN PROGRESS con pasos pendientes → **task-executor Fase 2**:
       - Cargar skills según tipo detectado (source-driven-development para Rust,
         frontend-ui-engineering para web, etc.)
       - Ponytail ladder: ya existe > stdlib > dependency > mínimo código
       - **Prohibición de prosa defensiva:** No expliques el código, no justifiques
         decisiones en comentarios. Si hay un problema, exprésalo modificando código
         o tests. Cero comentarios narrativos que duplican lo que el código ya dice.
       - Un paso atómico por iteración (~100 líneas)
       - State machine: PLAN → ACT → VERIFY
       - Si verify falla → **Agente de Diagnóstico**: no pasar el error crudo al
         implementador. Procesá el error del compilador/test/lint, identificá la causa
         raíz (archivo, línea, mensaje), y sintetizá una instrucción técnica precisa:
         *"El compilador falló en la línea 45: error de lifetime. Reestructurá la
         función para evitar devolver una referencia local"*. Recién ahí → retry.
       - Retry ladder: 1 error feedback, 2 mismo error (archivo+línea+mensaje) = ❌ FAILED
       - Stagnation detection: 3 intentos mismo error = ❌ FAILED

    c. Si verify pasa ✅ → **task-executor Fase 3-4**:
       - Verificación full: cargo build + nextest + fmt + clippy + tsc
       - **Capa determinista (barrera infranqueable):**
         - `cargo clippy --all-targets -- -D warnings` — cero advertencias
         - Si el código contiene `unsafe` o concurrencia:
           - Si nightly está disponible: `cargo +nightly miri test` (UB detection)
           - Marcar para ThreadSanitizer / AddressSanitizer en CI
         - Si el componente es crítico (parser, serializador, scheduler, WAL):
           - Marcar para fuzzing en CI (`fuzz/fuzz_<componente>.rs`)
           - Escribir test de propiedad básico (quickcheck/proptest)
       - **Pivotaje cognitivo para auto-revisión:** antes de evaluar, inyectá
         este prompt de cambio de rol: *"Detené la implementación. Ahora asumí
         el rol de Ingeniero de Sistemas Senior ultra-crítico. Encontrá 1-3
         problemas de seguridad, memoria, ineficiencia o errores lógicos
         ocultos en el código que acabas de escribir. Corregilos de inmediato."*
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

REGLAS RUST (MOTOR DB):
- **`unsafe` prohibido por defecto.** Si una optimización lo requiere:
  debe aprobarse explícitamente + documentar el safety invariant en
  `// SAFETY: ...`. Sin excepción.
- **`Rc<T>` prohibido en contextos multi-hilo.** Usar siempre `Arc<T>`.
  Justificar lifetimes con tipos antes de clonar para evadir el borrow checker.

REGLAS GLOBALES:
- Ponytail full: ya existe > stdlib > dependency instalada > mínimo código
- Verify = comando mecánico real (nunca auto-reporte)
- Verify falla 2 veces con mismo error (archivo+línea+mensaje) → ❌ FAILED
- No cambies scope. Si encontrás algo extra → anotalo, no lo implementes
- Stagnation: 3 intentos mismo error = ❌ FAILED
- Cada acción termina con plan file actualizado + recitation escrita

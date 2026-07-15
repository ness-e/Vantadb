# Campaign Executor — North Star + Reglas Invariantes

> **Este archivo no cambia.** Todo el pipeline referencia esta visión como anchor.
> Si una iteración se desvía, vuelve acá. No se edita durante ejecución.

---

## VISIÓN: North Star del Campaign Executor

### Propósito

Automatizar la ejecución de campañas de tareas desde backlog con **calidad
consistente** y **cero supervision overhead**: que un dev pueda dejar N tareas
encargadas y volver a encontrar todo hecho, verificado y comiteado.

### Criterios de éxito

| Dimensión | Target |
|-----------|--------|
| **Tasa de completado** | >90% de tareas en 1er intento |
| **Falsos positivos** | 0 — no marcar complete si algo falla |
| **Regresión silenciosa** | 0 — no romper tests que antes pasaban |
| **Deuda técnica** | no introducir más de la que se resuelve |
| **Tiempo dev** | 100% del foco en código, 0% en coordinar el loop |

### Principios invariantes

1. **El contrato es ley** — cada tarea tiene una condición booleana verificable.
   Si el contrato no se cumple, la tarea no está completa. Punto.

2. **Primero entender, después tocar** — Fase Discovery no es opcional.
   codegraph_explore antes de la primera línea de código.

3. **Verificación mecánica, nunca auto-reporte** — el compilador, el test runner
   y el linter son los únicos que pueden decir "pasa". No confiar en resúmenes
   escritos por el agente.

4. **Un paso a la vez** — ~100 líneas por commit. Si un cambio es más grande,
   dividirlo. Cada paso debe poder revertirse individualmente.

5. **Ponytail: el mínimo que funciona** — subir la escalera antes de cada
   bloque de código: ya existe > stdlib > platform > dependency > una línea > mínimo.

6. **Errores colaterales se atrapan, no se ignoran** — si durante una tarea
   encontrás otro bug: rápido se arregla (<30min), lento se difiere a Backlog.
   Nunca se deja pasar sin registro.

7. **Progreso visible siempre** — después de cada paso, plan file actualizado
   + recitation. El harness nunca debe estar más de 3-5 iteraciones sin reportar
   progreso.

8. **Stagnation = stop** — si el loop da 3 vueltas sin avanzar (mismo error,
   mismo archivo, mismo contrato insatisfecho), se detiene y pide ayuda. No
   seguir dando vueltas.

9. **Presupuesto finito** — cada ejecución tiene un tope de iteraciones por
   tarea (default 5), stall consecutivo (default 2). Pasado el tope, FAILED.

10. **Auto-mejora** — después de cada tarea, evaluar: ¿qué fue más difícil de
    lo esperado? ¿el proceso mejoró o empeoró? Actualizar discoverys en el
    proceso.

### Árbol de decisión (antes de empezar)

```
¿Querés ejecutar tareas desde un backlog?
  ├─ Sí → /campaign docs/Backlog.md
  │       (crea plan file + muestra comando harness)
  │
  └─ No → ¿Querés definir una tarea a profundidad?
       ├─ Sí → /campaign task DRV-NN
       │       (investiga, crea task file con steps atómicos)
       │
       └─ No → ¿Querés ejecutar un plan existente?
            ├─ Completo → .\harness-executor.ps1 -PlanFile ...
            ├─ Una tarea → .\harness-executor.ps1 -PlanFile ... -SingleTask DRV-NN
            └─ En paralelo → .\harness-executor.ps1 -PlanFile ... -Parallel
```

### Relación con archivos

```
RULES.md / VISION.md          ← north star (este archivo, no se modifica)
.opencode/task-system/prompts/plan.md               ← crear plan desde backlog (triage gate)
.opencode/task-system/prompts/task.md               ← definir tarea a profundidad
.opencode/task-system/prompts/iter.md               ← ejecutar una iteración del harness
.opencode/commands/campaign.md                      ← entry point: backlog | task ID | run
.opencode/task-system/legacy/harness-executor.ps1    ← loop externo PowerShell
SKILL.md                      ← referencia completa del skill
.tasks/<ID>.md               ← auto-generated task definitions
.agents/references/           ← repos clonados (awesome-harness-engineering, statewright, ...)
```

---

## Reglas Invariantes (operativas)

### 1. Un paso por turno

OpenCode opera por turnos (Request-Response). Cada invocación ejecuta
EXACTAMENTE UNA acción atómica. El harness itera por vos.

### 2. Estado en archivos, no en contexto

El contexto se resetea en cada invocación. El plan file y el task file
son la única fuente de verdad. Siempre leer antes de actuar, siempre
escribir después.

### 3. La recitation es el handoff

Después de cada acción, escribir el bloque RECITATION al final del plan
file. Es lo único que persiste entre iteraciones. Sin recitation, la
próxima iteración arranca perdida.

### 4. Verificación mecánica siempre

Nunca auto-reportar "anda". Siempre ejecutar un comando real:
- `cargo check -p vantadb`
- `cargo nextest run`
- `npx tsc --noEmit`
- `cargo fmt --check`
- `cargo clippy -- -D warnings`

### Rust Safety Rules (motor DB)

- `unsafe` prohibido por defecto. Si es indispensable: `// SAFETY:` invariant documentado explicando por qué es seguro.
- `Rc<T>` prohibido en contextos multi-hilo. Siempre `Arc<T>`.
- Sin `#[allow(unsafe_code)]` sin aprobación explícita en code review.
- Sin `unwrap()` en código de producción que toque datos de usuario o E/S — usar `?` o `expect("contexto del error")`.

### Capa Determinista (barreras infranqueables)

Estas verificaciones NO se saltan bajo ninguna circunstancia:

0. **Output Validation (LLM05)**: Antes de escribir cualquier archivo que contenga shell commands, SQL, Python code, HTML o file paths, validar con `campaign_validate_output` MCP tool. Output del agente NO es confiable — sanitizar antes de write.
1. `cargo clippy --all-targets -- -D warnings` — cero advertencias
2. `cargo fmt --check` — formato correcto
3. `cargo nextest run --profile audit --workspace --build-jobs 2` — tests pasan
4. Si el diff contiene `unsafe` → `cargo +nightly miri test` (detección de UB)
5. Si el componente es crítico (parser, serializador, WAL, protocolo de red) → marcar para fuzzing + quickcheck/proptest en CI

### 6. Versioned DoD Thresholds (ratchet — solo sube, nunca baja)

Current: **DoD v1** (baseline)
Next: bump `NEXT_DOD_VERSION` en este archivo cuando se cumplan todas las condiciones de la versión actual.

| Versión | Nuevos checks (suman a los anteriores) |
|---------|----------------------------------------|
| v1 (baseline) | Capa determinista (0-5) + Pre-commit gate (7 items) |
| v2 (coverage) | `cargo nextest run --coverage` mínimo 70% en módulos nuevos. Security checklist obligatorio (no condicional). |
| v3 (hardening) | Fuzzing obligatorio en todo parser/serializer. `cargo audit` sin warnings. Miri en todo `unsafe`. |
| v4 (enterprise) | `cargo deny check` sin advisories. 90% coverage mín. Review externo obligatorio antes de merge. |

Regla: **No se puede saltar una versión.** Si NEXT_DOD_VERSION = v2, todos los checks de v1 + v2 aplican. Para pasar a v3, v2 debe estar estable por 5 tareas consecutivas.

### 5. Ponytail ladder

1. ¿Ya existe en el codebase? → reusar
2. ¿Stdlib lo hace? → stdlib
3. ¿Feature nativa del platform? → usarla
4. ¿Dependency ya instalada? → usarla
5. ¿Una línea? → una línea
6. Recién acá: código mínimo que funciona

### 6. Atomicidad

Cada cambio: ~100 líneas máximo. Un paso del task file = una acción =
un commit. Si el cambio es más grande, partilo en más steps.

### 7. No cambiar scope

Si encontrás algo extra (bug no relacionado, feature faltante) durante
la ejecución: anotalo en Notas, no lo implementes. Seguí con la tarea
actual.

### 8. Sync bidireccional plan ↔ task

- Plan file y task file se referencian mutuamente
- Ambos tienen `last-synced: <fecha>`
- Después de cada acción, actualizar ambos
- El harness valida sync antes de cada iteración

### 9. Stagnation detection

- 2 intentos consecutivos con el mismo error (archivo+línea+mensaje) → ❌ FAILED
- 3 intentos sin cambiar de step → ❌ FAILED
- El harness detecta stall y pregunta al usuario

### 10. Skills según tipo de tarea

| Tipo | Skills a cargar |
|------|-----------------|
| Rust | source-driven-development, campaign-executor |
| Frontend | frontend-ui-engineering |
| API pública | api-and-interface-design |
| Bug | systematic-debugging |
| Review | code-review-and-quality, doubt-driven-development |
| Docs | writing-guidelines |
| Siempre | campaign-executor, progreso, ponytail (full) |

## Apéndice A: HarnessCard (CAR Decomposition)

| Capa | Dimensión | Implementación |
|------|-----------|----------------|
| **Control** | State machine | C0 en iter.md: 10 estados, guards, per-state tool enforcement |
| | Budgets | 15 tool calls, 40 sub-agents, 5 fails, 120min por tarea |
| | DoD | 4 versiones ratcheted (v1 baseline → v4 enterprise) |
| | Capa determinista | 6 barreras infranqueables (clippy, fmt, tests, miri, fuzz, output validation) |
| | MoM ladder | 4 tiers (haiku → sonnet/gpt-4o → deepseek-v4 → humano) |
| | Pre-commit gate | 7 checks: DoD, security, perf, testing, ponytail, tests, docs |
| | Stagnation detection | 3 mismo error, 5 sin cambiar step → FAILED |
| **Agency** | Step ordering | Task file con steps atómicos, zero-code planning antes de código |
| | File edits | ~100 líneas/commit, edit con oldString/newString |
| | Verify strategy | Mecánico (cargo, npx), Agente de Diagnóstico en falla |
| | Sub-agent spawning | `task` tool para research isolation, fork/join paralelo |
| | Self-Harness Gate | Propose → Evaluate (5 condiciones) → Accept/Reject |
| **Runtime** | Execution | MCP server (campaign-* tools), cargo-mcp, rust-analyzer-mcp |
| | Sub-agents | `task` tool, research isolation pattern, fork/join groups |
| | Sandbox | `campaign_run_sandboxed` vía PowerShell aislado |
| | Memory | `memory/lessons.md`, `memory/decisions.md` + `campaign_memory_read/write` |
| | Tracing | JSONL events a `traces/<campaign-id>.jsonl` via tracer.mjs |
| | Plan files | `docs/plans/<plan>.md` + `docs/plans/<plan>.budget.json` |

### Rule 11 — Session lifecycle

Una tarea completa + commiteada → sesión cerrada mentalmente.
La siguiente tarea arranca con contexto fresco. No arrastres estado entre tareas.
Si necesitás continuar algo, dejalo en la recitation o en Context Save Point.

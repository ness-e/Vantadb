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
prompts/plan.md               ← crear plan desde backlog (triage gate)
prompts/task.md               ← definir tarea a profundidad
prompts/iter.md               ← ejecutar una iteración del harness
commands/campaign.md          ← entry point: backlog | task ID | run
harness-executor.ps1          ← loop externo PowerShell
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

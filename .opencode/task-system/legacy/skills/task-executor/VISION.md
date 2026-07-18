# VISION — North Star del Task Executor

> **Este archivo no cambia.** Todo loop referencia esta visión como `anchor`.
> Si un loop se desvía, vuelve acá. No se edita durante ejecución.

## Propósito

Automatizar la ejecución de tareas técnicas con **calidad consistente** y
**cero supervision overhead**: que un dev pueda dejar N tareas encargadas
y volver a encontrar todo hecho, verificado y comiteado.

## Criterios de éxito

| Dimensión | Target |
|-----------|--------|
| **Tasa de completado** | >90% de tareas en 1er intento |
| **Falsos positivos** | 0 — no marcar complete si algo falla |
| **Regresión silenciosa** | 0 — no romper tests que antes pasaban |
| **Deuda técnica** | no introducir más de la que se resuelve |
| **Tiempo dev** | 100% del foco en código, 0% en coordinar el loop |

## Principios invariantes

1. **El contrato es ley** — cada tarea tiene una condición booleana verificable.
   Si el contrato no se cumple, la tarea no está completa. Punto.

2. **Primero entender, después tocar** — Fase A (Discovery) no es opcional.
   codegraph_explore antes de la primera línea de código.

3. **Verificación mecánica, nunca auto-reporte** — el compilador, el test runner
   y el linter son los únicos que pueden decir "pasa". No confiar en resúmenes
   escritos por el agente.

4. **Un paso a la vez** — ~100 líneas por commit. Si un cambio es más grande,
   dividirlo. Cada paso debe poder revertirse individualmente.

5. **Ponytail: el mínimo que funciona** — subir la escalera antes de cada
   bloque de código. No escribir nada que no haga falta.

6. **Errores colaterales se atrapan, no se ignoran** — si durante una tarea
   encontrás otro bug: rápido se arregla, lento se difiere a Backlog.
   Nunca se deja pasar sin registro.

7. **Progreso visible siempre** — después de cada paso, `opencode_loop_goal_progress`.
   El loop nunca debe estar más de 3-5 acciones sin reportar progreso.

8. **Stagnation = stop** — si el loop da 3 vueltas sin avanzar (mismo error,
   mismo archivo, mismo contrato insatisfecho), se detiene y pide ayuda. No
   seguir dando vueltas.

9. **Presupuesto finito** — cada ejecución tiene un tope de iteraciones,
   tokens, y (cuando se mida) costo. Pasado el tope, se detiene ordenadamente.

10. **Auto-mejora** — después de cada tarea, evaluar: ¿qué fue más difícil de
    lo esperado? ¿el proceso mejoró o empeoró? Actualizar AGENTS.md con el
    learning.

## Árbol de decisión (antes de empezar)

```
¿Tarea única compleja?
  ├─ Sí → loop-prompt.md (single-task executor)
  └─ No → ¿Múltiples tareas en plan file?
       ├─ Sí → batch-prompt.md (batch executor)
       └─ No → no necesitás el loop, hacelo directo
```

## Relación con otros archivos

```
VISION.md                  ← north star (este archivo, no se modifica)
loop-prompt.md             ← single-task executor (referencia VISION.md)
batch-prompt.md            ← batch executor (referencia VISION.md)
SKILL.md                   ← hub con entry points + innovaciones 2026
templates/task-definition.md ← template para tasks/ID.md
tasks/                     ← auto-generated task definitions
AGENTS.md                  ← learnings post-ejecución
.opencode/references/        ← repos clonados (awesome-harness-engineering, statewright, deepclaude, darwin-godel-machine)

## Pattern Coverage (5/5 gaps cerrados)

| Gap | Solución | Archivo |
|-----|----------|---------|
| Sin VISION.md (north star) | VISION.md con 10 principios | VISION.md |
| Sin no-progress detection | Stagnation Detection Middleware | loop-prompt.md C3 |
| Sin budget ceiling | Budget Controls (MAX_ITER, etc.) | batch-prompt.md 0.3 |
| Sin evaluator-optimizer | Auto-crítica 3 ejes post-implementación | loop-prompt.md C6 |
| Sin parallel subagentes | Orchestrator workers con waves | batch-prompt.md Fase 1.5 |
```

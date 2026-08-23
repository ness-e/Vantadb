> **CANONICAL SPEC — Routing de Hallazgos (fuente única)**
> Referenciado por: pipeline-full.md (Gate C + regla rápido/lento) · question-gates.md
> (Gate C). Los prompts REFERENCIAN este archivo; nunca lo redefinen.

# Hallazgos — bugs/errores/fallas descubiertos durante ejecución

Un **hallazgo** es cualquier bug, error o falla descubierto mientras se ejecuta
una tarea/plan que NO es el objetivo de esa tarea (el bug de la propia tarea se
arregla vía `bug-workflow.md` + `systematic-debugging`, no es un hallazgo).

## Regla única

**Todo hallazgo que no se arregle inline nace como fila en `docs/Backlog.md`
EN EL MOMENTO del discovery — no al cierre.** El plan/task file solo lo
referencia. Prohibido dejar hallazgos anotados solo en notas de plan,
recitations o commits (hoy se pierden cuando el plan se archiva).

### Inline vs ticket — criterio libre del agente

El agente decide caso por caso si arregla ya o genera ticket. Única restricción:
si arregla inline, aplica `bug-workflow.md` completo (root cause obligatorio,
sin refactor "while I'm here") y queda en el commit `fix:`. Si duda → ticket
(el ticket es barato; el hallazgo perdido no).

### Formato de ticket derivado

```
| FIND-<n> | <descripción corta> | ref: <plan-id>#<task-id> o commit | 🟡 | ... |
```

- **ID:** prefijo `FIND-` con numerador incremental (continuar del mayor
  existente en Backlog: `rg -o "FIND-\d+" docs/Backlog.md | sort | tail -1`)
- **Origen obligatorio:** `ref:` apuntando al plan/tarea/commit donde se descubrió
- **Contrato:** igual que toda fila del backlog (condición verificable)
- Prioridad 🟢 por default; el triage del próximo `/pipeline plan` la ajusta

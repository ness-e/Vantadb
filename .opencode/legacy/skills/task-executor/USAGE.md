# USAGE — Task Executor: Cómo generar y ejecutar prompts

## Formato Único

```bash
/loop-goal --prompt-file .opencode/skills/task-executor/loop-prompt.md "TASK=REV-004" "DESC=Fix tantivy rlib test build"
```

## Cómo generar prompts desde cualquier archivo

### De Backlog.md / review files → a prompts

Para cada fila ❌ en el backlog, armá:

```
/loop-goal --prompt-file .opencode/skills/task-executor/loop-prompt.md "TASK=<ID>" "DESC=<nombre corto>"
```

### Batch mode (varias tareas de un plan file)

```bash
/loop-goal --prompt-file .opencode/skills/task-executor/batch-prompt.md \
  "PLAN_FILE=docs/plans/mi-plan.md" "FAIL_MODE=skip" "FILTER=REV-"
```

| Flag | Qué hace |
|------|----------|
| `PLAN_FILE=` | Ruta al plan file con tareas |
| `FAIL_MODE=stop` | Para en la primera falla |
| `FAIL_MODE=skip` | Salta fallos y sigue |
| `FAIL_MODE=parallel` | Tareas independientes en paralelo |
| `FILTER=REV-` | Solo tareas que matcheen el regex |

## Mapa rápido: variables del loop-prompt.md

| Variable | Qué espera | Ejemplo |
|----------|-----------|---------|
| `TASK` | ID único | `REV-004`, `VFY-001`, `INT-01` |
| `DESC` | Descripción corta | `"Fix tantivy rlib test build"` |

El prompt file internamente hace:
- **A2:** busca la tarea por ID en `docs/plans/*.md`
- **A3:** `codegraph_explore` con el nombre de la tarea → blast radius
- **A4:** detecta si es Rust/Frontend/Python/TS/docs según los archivos
- **B1:** crea `.opencode/skills/task-executor/tasks/${TASK}.md` con plan atómico
- **C0-C9:** ejecuta con state machine, ponytail, stagnation detection, self-harness gate

## Ejemplos concretos

```bash
# De docs/reviews/2026-07-13-full-review.md → REV items
/loop-goal --prompt-file .opencode/skills/task-executor/loop-prompt.md "TASK=REV-003" "DESC=Add code coverage job to CI"

# De docs/Backlog.md → VFY items
/loop-goal --prompt-file .opencode/skills/task-executor/loop-prompt.md "TASK=VFY-001" "DESC=TS SDK: fix empty catch blocks"

# De docs/reviews/PROJECT_FULL_REVIEW.md → RC items (primero agregar al plan)
/loop-goal --prompt-file .opencode/skills/task-executor/loop-prompt.md "TASK=RC1-RC4" "DESC=Add lock_rwlock helper, replace 23 expects"

# Multi-tarea batch (filtrando solo REV)
/loop-goal --prompt-file .opencode/skills/task-executor/batch-prompt.md \
  "PLAN_FILE=docs/plans/2026-07-13-verified-backlog-execution.md" \
  "FAIL_MODE=skip" "FILTER=REV-"
```

## Flujo típico

```bash
# 1. Crear plan file con las tareas que querés ejecutar
# 2. Ejecutar single-task o batch
/loop-goal --prompt-file .opencode/skills/task-executor/batch-prompt.md \
  "PLAN_FILE=docs/plans/mi-plan.md" "FAIL_MODE=skip"
# 3. El skill ejecuta cada tarea:
#    Discovery → Definition → Execution (state machine) → Verify → Commit
# 4. Repite hasta completar el plan
```

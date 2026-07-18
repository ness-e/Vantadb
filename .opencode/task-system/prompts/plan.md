> **ACTIVE INSTRUCTION — Create Plan from Backlog**
> Cargado por `commands/pipeline.md` (modo PLAN).
> Path resolution: skills por nombre → `.opencode/skills/<nombre>/`
> Aplicar triage gate (✅ DO / 🟡 DEFER / ❌ SKIP / 🔴 BLOQUEADO) a cada tarea.
> Crear `docs/plans/<FECHA>-<nombre>.md` solo con tareas ✅ DO.
> Al finalizar: mostrar comando recomendado (`/pipeline run` o `/pipeline task <ID>`).

Cargá las skills brainstorming, writing-plans, idea-refine, progreso, ponytail (full).

Backlog: {{BACKLOG_PATH}}
Si no se especificó ruta, usá `docs/Backlog.md`.

## INSTRUCCIONES — CREAR PLAN DE CAMPAÑA

Aplicá el **triaje gate** del campaign-executor a CADA tarea en el backlog.
Resultados posibles: ✅ DO, 🟡 DEFER, ❌ SKIP, 🔴 BLOQUEADO.

### Reglas del gate

1. Bug ya inexistente o feature ya implementada → SKIP
2. Cosmético sin queja de usuario → DEFER
3. Esfuerzo >> impacto → DEFER o SKIP
4. Dependencia no lista → BLOQUEADO
5. Prioridad original es sugerencia, no orden

### Para cada tarea ✅ DO

Registrá en el plan file con:

- **ID** único (ej: DRV-068)
- **Descripción** corta (máx 80 chars)
- **Esfuerzo:** 🟢 1h | 🟡 1d | 🔴 2-3d
- **Prioridad:** 🔴 | 🟠 | 🟡 | 🟢
- **Archivos clave:** paths relevantes
- **Gate Justificación:** por qué pasó el gate
- **Contrato:** condición verificable por comando mecánico
- **Estado inicial:** ⬜ PENDING
- **Task file:** `skills/campaign-executor/tasks/ID.md` (aún no existe — se creará bajo demanda)

### Auto-detección de formato

Si el backlog tiene estructura conocida (TIER, Estado ❌, etc.) → parseá determinísticamente.
Si no reconoce el formato → el agente interpreta con LLM para extraer tareas.

### Formato del plan file

```markdown
# Plan de Ejecución: [Nombre]

> **Inicio:** YYYY-MM-DD
> **Estado:** ⏳ EN PROGRESO
> **Fuente:** [ruta al backlog]

## Resumen

| Resultado | Count |
|-----------|-------|
| ✅ DO | N |
| 🟡 DEFER | N |
| ❌ SKIP | N |
| 🔴 BLOQUEADO | N |

## Tasks

### Task 1: ID — Descripción

- **Esfuerzo:** 🟢 | 🟡 | 🔴
- **Prioridad:** 🔴 | 🟠 | 🟡 | 🟢
- **Archivos clave:** `path/to/file.rs`
- **Gate Justificación:** por qué pasó
- **Gate Result:** ✅ DO
- **Contrato:** "comando mecánico para verificar"
- **Task file:** `skills/campaign-executor/tasks/ID.md`
- **Estado:** ⬜ PENDING
- **Branch:**
- **Commit:**

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**

### Task 2: ...
```

### Al finalizar

Mostrá el comando exacto para ejecutar:

```
.opencode\task-system\harness\harness-executor.ps1 -PlanFile docs\plans\YYYY-MM-DD-<nombre>.md -Interval 10
```

---
name: task-executor
description: >
  Ejecutor dual: tarea única (`loop-prompt.md`) o batch multi-tarea (`batch-prompt.md`).
  Auto-discovery → auto-definition → execution rigurosa vía `/loop-goal --prompt-file`.
  Ponytail full + skills auto-seleccionadas + verificación mecánica.
---

# Task Executor — Ejecutor Dual

> Dos modos de operación según necesites:

| Modo | Prompt file | Para qué |
|------|-------------|----------|
| **Single-Task** | `loop-prompt.md` | Una tarea compleja que necesita toda la atención (features nuevas, refactors grandes) |
| **Batch (secuencial)** | `batch-prompt.md` | N tareas en secuencia desde un plan file |
| **Batch (paralelo)** | `batch-prompt.md` | FAIL_MODE=parallel — tareas independientes en waves concurrentes |
| **State Machine** | `loop-prompt.md` C0 | Guardrails formales Estado→Transición, bloquea saltos inválidos |
| **Self-Harness Gate** | `loop-prompt.md` C6.5 | Propose→Evaluate→Accept antes de commit. 2 rejections = block |

**Anchor común:** `VISION.md` — north star con principios invariantes. Leer antes de cualquier ejecución.

---

## Entry Points

### Single-Task (loop-prompt.md)
```
/loop-goal --prompt-file .opencode/skills/task-executor/loop-prompt.md "TASK=NUEVO-13" "DESC=HNSW PID loop"
```
Fases internas: Discovery → Definition → Execution.
Crea automáticamente `.opencode/skills/task-executor/tasks/ID.md` con el plan atómico.

### Batch (batch-prompt.md)
```
/loop-goal --prompt-file .opencode/skills/task-executor/batch-prompt.md \
  "PLAN_FILE=docs/plans/mi-plan.md" "FAIL_MODE=skip" "FILTER=NUEVO-"
```
Orquesta single-task executor para cada tarea pendiente en el plan file.

---

## Innovaciones 2026 Incorporadas

Basadas en investigación de harness engineering y agent loops (julio 2026):

| Innovación | Fuente | Dónde |
|------------|--------|-------|
| **VISION.md (north star)** | Steinberger | Archivo raíz del skill |
| **Stagnation detection** | Anthropic: no-progress detection | `loop-prompt.md` C3 |
| **Budget ceilings (3 hard stops)** | explainx.ai | `batch-prompt.md` 0.3 |
| **Evaluator-optimizer** | Lilian Weng: agent self-review | `loop-prompt.md` C6 |
| **State machine guardrails** | Statewright (415⭐) — reduce errores 80% | `loop-prompt.md` C0 |
| **Self-Harness propose-evaluate-accept** | Self-Harness: cierra círculo calidad | `loop-prompt.md` C6.5 |
| **Parallel orchestrator workers** | Anthropic: building effective agents (2026) | `batch-prompt.md` Fase 1.5 |
| **Auto-type discovery** | TaskWeaver (9k⭐): plan-and-execute | `loop-prompt.md` A4 |
| **Ponytail (shortest path)** | awesome-harness-engineering | Ambas fases |

### Comunidad: recursos referenciados (6/6)

| Recurso | Estrellas | Qué aporta | Estado |
|---------|-----------|------------|--------|
| [awesome-harness-engineering](https://github.com/walkinglabs/awesome-harness-engineering) | 3.6k⭐ | Catálogo patrones + herramientas | ✅ Clonado local |
| [Statewright](https://github.com/statewright/statewright) | 415⭐ | State machine guardrails, Rust | ✅ Clonado local + integrado (C0) |
| [TaskWeaver](https://github.com/microsoft/TaskWeaver) | 9k⭐ | Plan-and-execute concurrent planner | 📖 Referenciado (A4, 1.5) |
| [deepclaude](https://github.com/aattaran/deepclaude) | ~1k⭐ | Loop engine interchangeable backends | ✅ Clonado local |
| [Darwin Gödel Machine](https://github.com/lemoz/darwin-godel-machine) | ~500⭐ | Harness evolution (20→50% SWE-bench) | ✅ Clonado local |
| [Self-Harness](https://github.com/anthropics/self-harness) | — | Propose-evaluate-accept loop | ✅ Integrado (C6.5) |

### Referencias locales (repos clonados)

Los repos clonados están en `.opencode/references/` para consulta directa:

   ```
   .opencode/references/
  awesome-harness-engineering/   ← catálogo patrones (walkinglabs, 3.6k⭐)
  statewright/                   ← state machine guardrails en Rust (415⭐)
  deepclaude/                    ← loop engine interchangeable (1k⭐)
  darwin-godel-machine/          ← harness evolution research (~500⭐)
```

---

## Contrato de Calidad

Toda ejecución debe cumplir estas condiciones antes de marcar complete:

```
CONTRATO: "cargo build && cargo nextest run --profile audit --workspace --build-jobs 2 pasa,
           y el comportamiento específico de [tarea] funciona según spec"
```

---

## Fase 0: Preparación Profunda

Cargá skills según la fase. Guía completa en `AGENTS.md` → Skill Loading Guide.

| Fase | Skills |
|------|--------|
| **DEFINE** | `brainstorming` (ambiguo), `spec-driven-development`, `idea-refine` |
| **PLAN** | `planning-and-task-breakdown`, `writing-plans` |
| **BUILD** | dominio específico + **`ponytail` (full)** + `incremental-implementation` |
| **BUILD (Rust)** | `source-driven-development` |
| **BUILD (Frontend)** | `frontend-ui-engineering` |
| **VERIFY** | `debugging-and-error-recovery`, `browser-testing-with-devtools` |
| **REVIEW** | `code-review-and-quality`, `doubt-driven-development`, `code-simplification` |
| **REVIEW (extra)** | `security-and-hardening`, `performance-optimization` (si aplica) |
| **SHIP** | `git-workflow-and-versioning`, `ci-cd-and-automation`, `documentation-and-adrs` |

**Siempre:** `skill progreso` al inicio y al completar.

---

## Fase 1: Análisis Profundo + Blast Radius

Usá `codegraph_explore` para mapear el impacto:

```
codegraph_explore "query sobre los archivos/símbolos a modificar"

Responder en la bitácora de la tarea:
- CALLERS: qué módulos llaman a estos archivos
- CALLEES: de qué dependen estos archivos
- IMPLICACIONES: ¿contratos rotos? ¿cambia API pública?
  ¿performance/memoria? ¿serialización? ¿migración necesaria?
- CONTRATO: "completado = [condición verificable]"
```

Registrar progreso:
```
opencode_loop_goal_progress summary:"Análisis completo — blast radius mapeado, contrato definido" next:"Implementar cambio atómico 1/N"
```

---

## Fase 2: Implementación (Loop Interno Plan→Act→Verify)

```
ITERAR hasta que todos los cambios atómicos estén listos:

  1. PLAN: decidir el cambio atómico (~100 líneas máx)
  2. ACT: editar código siguiendo skills cargadas
  3. VERIFY MECÁNICO (nunca auto-reporte):
     - `cargo check -p vantadb` (o crate específica)
     - `npx tsc --noEmit` (frontend)
     - `cargo nextest run <test> --build-jobs 2`
  4. SI PASA → siguiente cambio atómico
  5. SI FALLA → RETRY LADDER:
     a) Leer el error exacto
     b) Corregir con feedback del error
     c) 2 intentos consecutivos = MISMO error → ESCALAR:
        opencode_loop_goal_blocked reason:"[error]" needed:"[qué falta]"
```

### Escalera Ponytail (aplica siempre)

1. ¿Ya existe en el codebase? → reusar
2. ¿Stdlib lo hace? → stdlib
3. ¿Platform feature? → platform
4. ¿Dependency instalada? → usarla
5. ¿Una línea? → una línea
6. Recién acá: código mínimo

---

## Fase 3: Verificación Full

```
[ ] cargo build --workspace (o warm cache si Windows da page file error)
[ ] cargo nextest run --profile audit --workspace --build-jobs 2
[ ] npx tsc --noEmit (frontend)
[ ] cargo fmt --check
[ ] Verificación específica del contrato
```

Si algo falla → volver a Fase 2 con error como contexto.

Si pasa → registrar progreso:
```
opencode_loop_goal_progress summary:"Verificación full pasa" next:"Revisión post-implementación"
```

---

## Fase 4: Revisión Post-Implementación

Cargar skills de REVIEW:
```
skill code-review-and-quality
skill doubt-driven-development  (si stakes altos)
skill code-simplification
```

```
1. codegraph_explore para verificar impacto completo:
   - Módulos dependientes compilan y pasan tests?
   - Interfaz pública o contrato roto?
   - Edge cases no contemplados?

2. ¿Código más simple? ¿tests faltantes?

3. Checklist de calidad:
   - Manejo de errores
   - Edge cases (vacíos, nulos, límites)
   - Documentación actualizada (si cambió API)
```

Si hay issues → volver a Fase 2 con feedback.

Si está limpio → registrar:
```
opencode_loop_goal_progress summary:"Revisión pasa — código limpio" next:"Atrapar errores colaterales"
```

---

## Fase 5: Stagnation Detection + Errores Colaterales

### 5.1 Stagnation Detection (gate previo)
Antes de procesar errores colaterales, ejecutar middleware de estancamiento:
```
CHECK:
  - ¿3+ iteraciones con el mismo error?
  - ¿5+ iteraciones sin cambiar de paso?
  - ¿Mismos archivos tocados en últimas 3 iteraciones?
Si ALGUNA → opencode_loop_goal_blocked (no seguir iterando)
```
Referencia: VISION.md principio #8, Anthropic harness no-progress detection (2026).

### 5.3 Errores Colaterales

> **Clave del loop:** mientras revisás, si encontrás OTRO error en el camino,
> lo atrapás en la misma iteración.

```
MIENTRAS (haya errores encontrados durante la revisión):
  Para cada error colateral:
    1. Anotarlo
    2. Si es RÁPIDO (🟢 <30min, mismo archivo/conexión): arreglar y commitear junto
    3. Si es LENTO (🟡 >30min, módulo diferente): crear entrada en Backlog.md y seguir
    4. NO perder foco de la tarea principal
```

### 5.4 Budget Check (post-error)
```
Después de procesar errores colaterales (rápidos o diferidos):
1. Revisar budget restante del batch (si aplica)
2. Si MAX_ITER o tiempo estimado excedido → stop ordenado
3. Si ok → continuar a cierre
```

### Arreglar en mismo commit
- Bug de compilación/lint
- Test quebrado por el cambio
- Documentación inconsistente (nombres)
- Error evidente en código adyacente

### Diferir a Backlog.md
- Feature request no relacionada
- Refactor profundo de otro módulo
- Bug pre-existente en área no tocada

---

## Fase 6: Cierre

```
1. ÚLTIMA verificación: cargo build && cargo nextest run
2. git add -p (revisar cada cambio)
3. git commit -m "tipo(scope): descripción

   Blast radius: [módulos afectados]
   Skills: [lista de skills usadas]
   Contrato: [condición cumplida]
   Errores colaterales: [ninguno | lista con destino]"

4. opencode_loop_goal_complete
   summary:"[ID — Nombre] completo"
   evidence:"cargo nextest pasa (N/N), [verificación específica]"

5. Si hay más tareas en el plan:
   - Actualizar plan file (marcar tarea ✅)
   - Iniciar nuevo /loop con la siguiente tarea
6. Si no hay más tareas → FIN
```

---

## Resumen Visual del Loop

```
/loop "Ejecutar [ID] — [nombre]"
  │
  ├─ FASE 0: Cargar skills + leer VISION.md (anchor)
  ├─ FASE 1: codegraph_explore + blast radius + contrato
  │   └─ opencode_loop_goal_progress
  │
  ├─ FASE 2: Loop Interno (Plan→Act→Verify)
  │   ├─ └─ opencode_loop_goal_progress por cambio atómico
  │   ├─ RETRY LADDER si falla
  │   │   └─ opencode_loop_goal_blocked si escala
  │   └─ STAGNATION DETECTION (3 same-error = stop)
  │
  ├─ FASE 3: Evaluator-Optimizer (auto-crítica 3 ejes)
  │   └─ Volver a FASE 2 si inconsistencias
  │
  ├─ FASE 4: cargo build + nextest + tsc
  │
  ├─ FASE 5: Stagnation Detection gate → errores colaterales
  │   ├─ Rápido→arreglar / Lento→Backlog.md
  │   └─ Budget check post-error
  │
  └─ FASE 6: git commit + opencode_loop_goal_complete
       └→ SIGUIENTE TAREA (nuevo /loop)
```

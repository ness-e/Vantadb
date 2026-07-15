# Campaign Executor — Harness-Driven Pipeline

> Unifica backlog-executor (orquestación de campañas) y task-executor
> (ejecución profunda de tareas) en un solo skill.

## Arquitectura

```
HARNESS (PowerShell)                AGENTE (por turno)
┌────────────────────────┐         ┌──────────────────────────┐
│ while (tasks > 0)     │───────▶ │ 1. Leer plan file        │
│  1. next pending task │         │ 2. Leer task file (si)   │
│  2. inject prompt.md  │         │ 3. Una acción:           │
│  3. wait + validate   │         │    a. Discovery          │
│  4. detect stall      │◀────────│    b. Implementar step   │
│  5. repeat            │         │    c. Verify + commit    │
└────────────────────────┘         │ 4. Actualizar plan+task  │
                                   │ 5. Recitation           │
                                   │ 6. STOP                 │
                                   └──────────────────────────┘
```

## Componentes

| Componente | Ubicación | Propósito |
|------------|-----------|-----------|
| **plan.md** | `.opencode/prompts/plan.md` | Crear plan de campaña desde backlog |
| **task.md** | `.opencode/prompts/task.md` | Definir tarea individual a profundidad |
| **iter.md** | `.opencode/prompts/iter.md` | Prompt único del harness (1 iteración) |
| **campaign.md** | `.opencode/commands/campaign.md` | Entry point: backlog / task ID / run |
| **harness-executor.ps1** | raíz del proyecto | Loop PowerShell (timeout, git check, sync) |
| **Plan file** | `docs/plans/<fecha>-<nombre>.md` | Orquestación: qué tasks, en qué estado |
| **Task file** | `skills/campaign-executor/tasks/<ID>.md` | Profundidad: steps atómicos, blast radius |
| **RULES.md** | `skills/campaign-executor/RULES.md` | North star + reglas invariantes del sistema |

## Estados de una tarea

```
⬜ PENDING → ⏳ IN PROGRESS → ✅ COMPLETED
                              ❌ FAILED
```

## Ciclo completo

### Fase 0: Crear plan

1. Usuario invoca `/campaign docs/Backlog.md`
2. `campaign.md` carga `prompts/plan.md`
3. El agente aplica triage gate a cada tarea
4. Crea `docs/plans/<fecha>-<nombre>.md` con todas las ✅ DO
5. Muestra comando para arrancar el harness

### Fase 1: Discovery (por tarea, una vez)

1. Harness encuentra tarea ⬜ PENDING, inyecta `prompts/iter.md`
2. Agente detecta que el task file no existe
3. Auto-detecta tipo de tarea (Rust / Frontend / Python / ...)
4. `codegraph_explore` para blast radius
5. Web research si hay ambigüedad
6. Descompone en steps atómicos
7. Crea `tasks/<ID>.md` con steps, contrato, herramientas
8. Actualiza plan file: Estado → ⏳ IN PROGRESS
9. Recitation → STOP

### Fase 2: Ejecución (un step por iteración)

```
State machine por step:
  PLAN → ACT → VERIFY
          ↑      │
          └──────┘ (falló → PLAN con feedback)
                 │
                 ↓ (3 veces mismo error)
               STALL → ❌ FAILED

  VERIFY pasa ↓
         COLLATERAL → RESEARCH → ACT (si ambigüedad)
                    → EVALUATE (sin errores)
                         ↓ falla → ACT
                         ↓ pasa  → REVIEW
                                    ↓ issues → VERIFY
                                    ↓ pasa  → ACCEPT → CLOSE
```

1. Harness re-inyecta `prompts/iter.md`
2. Agente lee el próximo step del task file
3. PLAN → ACT → VERIFY (con Agente de Diagnóstico si falla)
4. Retry ladder: 1 retry → 2 fresh context → 3 different strategy → 4 escalate
5. Si pasa → errores colaterales → evaluator-optimizer → self-harness gate
6. Actualiza task file (step ✅) y plan file (iteración)
7. Recitation → STOP

### Fase 3: Cierre (verificación completa)

1. Todos los steps ✅
2. Verificación full del contrato (build + test + fmt + clippy + extra)
3. Pivotaje cognitivo (auto-revisión)
4. Evaluator-optimizer (3 ejes: correctitud, simplicidad, consistencia)
5. Self-Harness gate (propose → evaluate → accept)
6. Pre-commit gate (Definition of Done + checklists por tipo)
7. Commit + skill progreso
8. Plan file: Estado → ✅ COMPLETED
9. Recitation → STOP

## Formato plan file

```markdown
# Plan de Ejecución: [Nombre]

> **Inicio:** YYYY-MM-DD
> **Estado:** ⏳ EN PROGRESO
> **Fuente:** docs/Backlog.md

## Resumen
| DO | DEFER | SKIP | BLOQUEADO |
|----|-------|------|-----------|
| N  | N     | N    | N         |

### Task 1: ID — Descripción
- **Archivos clave:** `path`
- **Gate Justificación:** ...
- **Contrato:** "cargo nextest run pasa"
- **Task file:** `tasks/ID.md`
- **Estado:** ⬜ PENDING | ⏳ IN PROGRESS | ✅ COMPLETED | ❌ FAILED
- **last-synced:** YYYY-MM-DDTHH:MM
```

## Formato task file

```markdown
# TASK-ID: Descripción

## Metadata
- **Plan file:** [ruta]
- **Creado:** YYYY-MM-DDTHH:MM
- **last-synced:** YYYY-MM-DDTHH:MM
- **Estado:** ⬜ PENDING

## Blast Radius
Callers | Callees | Implicaciones

## Contrato
"comando verificable"

## Herramientas
- cargo-mcp, rust-analyzer-mcp, codegraph

## Steps
### Step 1: [Nombre]
- **Archivos:** `path`
- **Acción:** ...
- **Verify:** `cargo check -p vantadb`
- **Estado:** ⬜ PENDING

## Dependencias
- Task N-1: ID

## Notas

## Context Save Point
- **Fecha:** ISO
- **Branch:** nombre
- **CI pendiente:** sí/no
- **Decisiones:** X sobre Y porque [razón breve]
- **Problemas conocidos:** [ninguno | lista]
- **Próxima tarea:** TASK-N+1
```

## Compaction (mantenimiento periódico)

Cada 5 tareas completadas (o al alcanzar ~50 iteraciones en el plan file),
compactá el plan file:
1. Resumir iteraciones viejas en una tabla consolidada
2. Mantener solo la recitation actual + errores activos
3. Archivar decisiones pasadas en el task file correspondiente
4. Verificar que last-synced esté al día en todos los archivos
5. Anotar "Compaction N/5: OK" al inicio del plan file

## Probes de integridad (antes de cada tarea)

El pipeline verifica antes de arrancar cualquier tarea:
- Plan file existe y tiene al menos una task
- Recitation block (si existe) es parseable
- Última tarea procesada ≠ misma tarea dos veces sin progreso
- Plan file no tiene PID de harness activo de otra sesión
- Git status está limpio (o los cambios son del pipeline actual)
- `campaign_stalled_tasks` (MCP) no reporta bloqueos previos sin resolver

Si alguna probe falla → pausar y preguntar al usuario.

## Recitation block

```
=== RECITATION ===
Objetivo activo: TASK-N — ID
Estado: plan / act / verify / stall / research / collateral / evaluate / review / accept / completed / failed
Última acción: qué se acaba de hacer
Resultado: ✅ / ❌
State: ESTADO (desde: ESTADO_ANTERIOR)
Próxima acción: paso concreto (archivo + comando)
Contrato: "condición verificable"
Próxima tarea si completa: TASK-N+1 — ID
last-synced: YYYY-MM-DDTHH:MM
=== END RECITATION ===
```

## Escalation ladder

| Escalón | Acción |
|---------|--------|
| 1 | Retry con feedback del error procesado (Agente de Diagnóstico) |
| 2 | Contexto fresco: resumir lo aprendido (~200 tokens) + error |
| 3 | Estrategia materialmente distinta |
| 4 | Escalar a humano: documentar intentos, commit WIP, ❌ FAILED |

## Budget management

| Control | Default | Hard Limit | Comportamiento |
|---------|---------|------------|----------------|
| Iteraciones por tarea | 5 | 10 | Al alcanzar → ❌ FAILED |
| Sub-agentes totales (FAIL_MODE parallel) | 20 | 40 | HARD STOP + reporte parcial |
| Consecutive fails | 3 | 5 | FAIL_MODE pasa a "stop" forzosamente |
| NO_PROGRESS_LIMIT (stagnation) | 3 | 5 | `campaign_stalled_tasks` + pausa |
| Tool calls por sub-agente | 8 | 15 | Timeout + kill |
| Contexto inicial | < 20% (~40k tokens) | — | Si excede → usar sub-agentes |

## Ejecución paralela

Cuando el harness recibe `-Parallel`, identifica tareas independientes
(sin dependencias entre sí, sin archivos compartidos) y las ejecuta en
waves concurrentes:

```
Wave 0: tareas sin dependencias → N sub-procesos paralelos
Wave 1: tareas que dependen de Wave 0
Wave 2: tareas que dependen de Wave 1
...
```

MAX_CONCURRENT = min(4, tareas_en_wave). Cada tarea en paralelo usa su
propia invocación de `opencode run`.

## Contrato: vago vs verificable

| ❌ Vago | ✅ Verificable |
|---------|----------------|
| "Arreglar el bug de memoria" | "tests/test_memory.rs pasa, cargo machete 0 warnings, cargo nextest run pasa" |
| "Mejorar la web" | "npx tsc --noEmit 0 errors, npm run lint 0 errors" |
| "Refactorizar módulo" | "cargo check --workspace, clippy sin warnings nuevos, tests existentes pasan" |
| "Funciona bien" | "cargo build && cargo nextest run pasa, y comportamiento específico funciona" |

## Apéndice A: Comandos rápidos VantaDB

| Comando | Propósito |
|---------|-----------|
| `cargo check -p vantadb` | Build rápido (solo crate core) |
| `cargo nextest run --profile audit --workspace --build-jobs 2` | Tests completos |
| `cargo fmt --check` | Formato |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | Lints |
| `just verify` | Pre-flight completo |
| `codegraph_explore "query"` | Blast radius |
| `skill progreso` | Migrar backlog → progreso |

## Apéndice B: opencode-loop plugin

Si instalaste el plugin `opencode-loop` como alternativa al harness:

```bash
oc plugin install opencode-loop
```

Uso con Goal Mode (el plugin monitorea idle events y re-inyecta):

```
/loop-goal "Cargá campaign-executor. Ejecutá el plan file docs/plans/campaign.md
una tarea a la vez. Después de cada tarea, actualizá el plan file y escribí
la recitation. No avances sin haber completado verificación, commit, progreso."
```

## Apéndice C: Innovaciones incorporadas (2026)

| Innovación | Fuente | Dónde |
|------------|--------|-------|
| **VISION.md (north star)** | Steinberger | RULES.md |
| **Stagnation detection** | Anthropic: no-progress detection | iter.md, harness |
| **Budget ceilings** | explainx.ai | SKILL.md, harness |
| **Evaluator-optimizer** | Lilian Weng: agent self-review | iter.md MODO CIERRE |
| **State machine guardrails** | Statewright (415⭐) | iter.md State Machine C0 |
| **Self-Harness propose-evaluate-accept** | Self-Harness (Anthropic) | iter.md Self-Harness Gate |
| **Parallel orchestrator workers** | Anthropic: building effective agents | pipeline-run.md waves |
| **Auto-type discovery** | TaskWeaver (9k⭐) | iter.md MODO DISCOVERY |
| **Ponytail (shortest path)** | awesome-harness-engineering | RULES.md, iter.md |
| **Zero-code planning** | task-executor hybrid-prompt | iter.md MODO DISCOVERY step 4 |
| **Revisión cada N tareas** | backlog-executor loop-prompt | pipeline-run.md step 5.f |
| **Context Save Point** | backlog-executor §7 | task file format, iter.md Cierre |
| **FAIL_MODE triple (stop/skip/parallel)** | task-executor batch-prompt | pipeline-run.md |
| **Parallel DAG + waves** | task-executor batch-prompt §1.5 | pipeline-run.md step 6 |
| **Probes de integridad** | backlog-executor §8 | SKILL.md, pipeline-run.md step 3 |
| **Compaction periódica** | backlog-executor §7 | SKILL.md |
| **Prompt Templates** | backlog-executor §11 | commands/campaign.md Apéndice

### Referencias locales clonadas

```
.agents/references/
  awesome-harness-engineering/   ← catálogo patrones (walkinglabs, 3.6k⭐)
  statewright/                   ← state machine guardrails en Rust (415⭐)
  deepclaude/                    ← loop engine interchangeable (1k⭐)
  darwin-godel-machine/          ← harness evolution research (~500⭐)
```

## Apéndice D: Bibliografía de Loop Engineering

| Fuente | Concepto clave |
|--------|---------------|
| Steve Kinney, "Anatomy of an Agent Loop" (2026) | Todo framework converge en el mismo `while`. La diferencia está en el harness. |
| Addy Osmani, "Loop Engineering" (2026) | Verificación mecánica, stall detection, plan/act/verify. |
| Boris Cherny, Claude Code Lead (2026) | "I don't prompt anymore. I have loops running that prompt Claude." |
| Anthropic, "Harness Design for Long-Running Apps" (2025) | Context resets, plan/execute/review, durable artifacts. |
| Anthropic, "Building Effective AI Agents" (2024) | Workflows vs agents. "Start with a single loop." |
| Manus, "Recitation Pattern" (2025) | todo.md rewrite for goal preservation at end of context. |
| redreamality, "Agent Harness Pattern" (2026) | 5 layers: context, streaming, recovery, termination, state. |
| niv0, "Claude Code from Scratch" (2026) | 23-phase reproduction of Claude Code harness. |
| d3vr, "OCLoop" (2026) | OpenCode loop harness with dashboard, plan-based iteration. |
| ByBrawe, "opencode-loop" (2026) | `/loop` and `/loop-goal` plugin for OpenCode TUI. |
| Loop Engineering Guide (2026) | Circuit breaker: iterations, stagnation, no-progress, tokens. |
| StackOne, "Agent Suicide by Context" (2026) | 67.6% tokens son tool outputs. Sub-agentes como defensa. |
| Inngest Blog, "Agent Loop Architecture" (2026) | Loop + skill + orchestrator. Durable execution. |
| Oracle Developers (2026) | 3 niveles: Level 1 (tools), Level 2 (memory), Level 3 (harness). |

## Apéndice E: Troubleshooting

| Síntoma | Causa | Solución |
|---------|-------|----------|
| El agente hace 2+ tareas en un turno | Ignoró "una iteración" | Usar el harness |
| Harness no detecta progreso | Recitation faltante | Verificar que el agente escribió RECITATION |
| Plan file corrupto | Regex no parsea | Revisar encoding de emojis |
| last-synced desfasado | Task file editado sin plan file | El harness re-sincroniza automáticamente |
| opencode run colgado | Tool output muy grande | El timeout del harness aborta |
| Misma tarea reprocesada infinitamente | Stall detection mal configurado | Verificar $StallThreshold >= 2 |

## Diagrama de flujo completo

```
/campaign docs/Backlog.md
  │
  ├─ plan.md: triage gate → docs/plans/<fecha>.md
  │
  ├─ harness-executor.ps1
  │   │
  │   ├─ FASE 1: DISCOVERY (por tarea, 1 vez)
  │   │   ├─ auto-detect tipo (Rust/Frontend/Python/...)
  │   │   ├─ codegraph_explore → blast radius
  │   │   ├─ web research si ambigüedad
  │   │   ├─ crear task file con steps atómicos
  │   │   ├─ plan file → ⏳ IN PROGRESS
  │   │   └─ RECITATION → STOP
  │   │
  │   ├─ FASE 2: EJECUCIÓN (1 step por iteración)
  │   │   ├─ State Machine: PLAN → ACT → VERIFY
  │   │   ├─ Retry ladder (4 escalones)
  │   │   ├─ Agente de Diagnóstico en verify falla
  │   │   ├─ Stagnation Detection (3 same-error = stop)
  │   │   ├─ Errores colaterales (rápido→fix, lento→Backlog)
  │   │   ├─ Evaluator-Optimizer (3 ejes)
  │   │   ├─ Self-Harness Gate (propose→evaluate→accept)
  │   │   ├─ Pre-commit Gate
  │   │   ├─ git commit
  │   │   ├─ skill progreso
  │   │   └─ RECITATION → STOP
  │   │
  │   ├─ FASE 3: CIERRE (último step)
  │   │   ├─ Verificación full (build+test+fmt+clippy+extra)
  │   │   ├─ Plan file → ✅ COMPLETED
  │   │   └─ RECITATION → STOP
  │   │
  │   └─ (repite hasta que todas las tareas estén ✅ o ❌)
  │
  └─ Resumen final: N/M completadas
```

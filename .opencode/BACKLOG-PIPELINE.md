# Campaign Pipeline

Pipeline completo para ejecutar campañas de tareas desde backlog con
harness externo + agente por turno.

## Arquitectura

```
HARNESS (PowerShell)               AGENTE (OpenCode por turno)
┌────────────────────────┐        ┌──────────────────────────────┐
│ while (tasks > 0)     │        │ iter.md (una iteración)      │
│  1. git check          │───────▶│  1. Leer plan file            │
│  2. find pending task  │        │  2. Leer task file (si)      │
│  3. inject iter.md     │        │  3. Discovery / Implement /  │
│  4. timeout + wait     │        │     Verify / Close            │
│  5. validate recitation│◀───────│  4. Update plan + task files  │
│  6. detect stall       │        │  5. Recitation block          │
│  7. repeat             │        │  6. STOP                      │
└────────────────────────┘        └──────────────────────────────┘
```

## Componentes

| Componente | Ubicación | Propósito |
|------------|-----------|-----------|
| **plan.md** | `prompts/plan.md` | Crear plan desde backlog (triage gate) |
| **task.md** | `prompts/task.md` | Definir tarea individual (blast radius + steps) |
| **iter.md** | `prompts/iter.md` | Una iteración del harness (discovery/implement/close) |
| **campaign.md** | `commands/campaign.md` | Entry point: backlog / task ID / run |
| **harness-executor.ps1** | raíz | Loop PowerShell con timeout, git check, sync, SingleTask |
| **Plan file** | `docs/plans/<fecha>-<nombre>.md` | Orquestación: tasks, estados, recitation |
| **Task file** | `skills/campaign-executor/tasks/<ID>.md` | Profundidad: steps atómicos, blast radius |
| **SKILL.md** | `skills/campaign-executor/SKILL.md` | Referencia completa del skill |
| **RULES.md** | `skills/campaign-executor/RULES.md` | Reglas invariantes |

## Estados

```
⬜ PENDING → ⏳ IN PROGRESS → ✅ COMPLETED
                              ❌ FAILED
```

## Ciclo de vida

### Fase 0: /campaign backlog.md
1. Triage gate → ✅ DO / 🟡 DEFER / ❌ SKIP / 🔴 BLOQUEADO
2. Crea `docs/plans/<fecha>-<nombre>.md`
3. Muestra comando: `.\harness-executor.ps1 -PlanFile ...`

### Fase 1: Discovery (primer turno de cada tarea)
1. Harness inyecta `iter.md` con `{{PLAN_FILE}}`
2. Agente no encuentra task file → `codegraph_explore` → blast radius
3. Crea `tasks/<ID>.md` con steps atómicos + contrato
4. Plan file → ⏳ IN PROGRESS

### Fase 2: Implementación (un step por turno)
1. Agente lee próximo step del task file
2. PLAN → ACT (edit) → VERIFY (cargo check / nextest / tsc)
3. Task file: step ✅. Plan file: iteración agregada.
4. Si falla 2 veces igual → ❌ FAILED

### Fase 3: Cierre (último turno de cada tarea)
1. Todos los steps ✅
2. Verificación completa: build + test + fmt + clippy + extras
3. Pre-commit gate + commit
4. `skill progreso` (Trigger 1)
5. Plan file → ✅ COMPLETED

## Recitation

Después de cada acción, el agente escribe:

```
=== RECITATION ===
Objetivo activo: TASK-N — ID
Estado: implementing
Última acción: edit en src/engine.rs
Resultado: ✅ cargo check pasa
Próxima acción: cargo nextest run test_engine_persistence
Contrato: "cargo nextest run pasa"
Próxima tarea si completa: TASK-N+1 — ID
last-synced: 2026-07-14T16:00
=== END RECITATION ===
```

## Retry ladder

| Escalón | Acción |
|---------|--------|
| 1 | Retry con feedback del error procesado (Agente de Diagnóstico) |
| 2 | Contexto fresco: resumir lo aprendido (~200 tokens) + error |
| 3 | Estrategia materialmente distinta |
| 4 | Escalar a humano: documentar intentos, commit WIP, ❌ FAILED |

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
  │   │   └─ plan file → ⏳ IN PROGRESS
  │   │
  │   ├─ FASE 2: EJECUCIÓN (1 step por iteración)
  │   │   ├─ State Machine: PLAN → ACT → VERIFY
  │   │   ├─ Retry ladder (4 escalones)
  │   │   ├─ Agente de Diagnóstico en verify falla
  │   │   ├─ Stagnation Detection (3 same-error = stop)
  │   │   ├─ Errores colaterales (rápido→fix, lento→Backlog)
  │   │   ├─ Evaluator-Optimizer (3 ejes)
  │   │   ├─ Self-Harness Gate (propose→evaluate→accept)
  │   │   ├─ Pre-commit Gate + git commit + skill progreso
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

## Referencias

### Innovaciones incorporadas (2026)

| Innovación | Fuente | Dónde |
|------------|--------|-------|
| VISION.md (north star) | Steinberger | RULES.md |
| Stagnation detection | Anthropic: no-progress detection | iter.md, harness |
| Budget ceilings | explainx.ai | SKILL.md, harness |
| Evaluator-optimizer | Lilian Weng: agent self-review | iter.md MODO CIERRE |
| State machine guardrails | Statewright (415⭐) | iter.md State Machine C0 |
| Self-Harness propose-evaluate-accept | Self-Harness (Anthropic) | iter.md Self-Harness Gate |
| Parallel orchestrator workers | Anthropic: building effective agents | harness -Parallel |
| Auto-type discovery | TaskWeaver (9k⭐) | iter.md MODO DISCOVERY |
| Ponytail (shortest path) | awesome-harness-engineering | RULES.md, iter.md |

### Bibliografía — Loop Engineering

| Fuente | Concepto clave |
|--------|---------------|
| Steve Kinney, "Anatomy of an Agent Loop" (2026) | Todo framework converge en el mismo `while` |
| Addy Osmani, "Loop Engineering" (2026) | Verificación mecánica, stall detection |
| Boris Cherny, Claude Code Lead (2026) | Loops que invocan Claude, no prompting directo |
| Anthropic, "Harness Design for Long-Running Apps" (2025) | Context resets, plan/execute/review |
| Anthropic, "Building Effective AI Agents" (2024) | Workflows vs agents |
| Manus, "Recitation Pattern" (2025) | Goal preservation al final de contexto |
| redreamality, "Agent Harness Pattern" (2026) | 5 layers: context, streaming, recovery |
| niv0, "Claude Code from Scratch" (2026) | 23-phase reproduction of Claude Code harness |
| d3vr, "OCLoop" (2026) | OpenCode loop harness |
| ByBrawe, "opencode-loop" (2026) | Plugin loop para OpenCode TUI |
| StackOne, "Agent Suicide by Context" (2026) | 67.6% tokens son tool outputs |
| Inngest Blog, "Agent Loop Architecture" (2026) | Loop + skill + orchestrator |

## Archivos clave

Viejos (legacy): `.opencode/task-system/legacy/` — backup completo de la versión anterior
Nuevos: `prompts/plan.md`, `prompts/task.md`, `prompts/iter.md`
Unified skill: `skills/campaign-executor/`
Entry: `commands/campaign.md`
Harness: `harness-executor.ps1`

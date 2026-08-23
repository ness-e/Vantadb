---
name: campaign-executor
description: >
  Pipeline-driven system that unifies backlog-executor (campaign
  orchestration) and task-executor (deep task execution). Use when running
  /pipeline plan|task|run, defining task files with atomic steps, or driving
  tight iteration loops through PLAN/ACT/VERIFY states.
compatibility: opencode
---

# Campaign Executor — Pipeline-Driven Execution

> Unifica backlog-executor (orquestación de campañas) y task-executor
> (ejecución profunda de tareas) en un solo skill.

## Arquitectura

```
LOOP (vía /loop-goal o /pipeline run)   AGENTE (por turno)
┌────────────────────────┐         ┌──────────────────────────┐
│ for each pending task  │───────▶ │ 1. Leer plan file        │
│  1. next pending task  │         │ 2. Leer task file (si)   │
│  2. inyecta prompt     │         │ 3. Una acción:           │
│  3. wait + validate    │         │    a. Discovery          │
│  4. detect stall       │◀────────│    b. Implementar step   │
│  5. repeat             │         │    c. Verify + commit    │
└────────────────────────┘         │ 4. Actualizar plan+task  │
                                   │ 5. Recitation           │
                                   │ 6. STOP                 │
                                   └──────────────────────────┘
```

## Componentes

| Componente | Ubicación | Propósito |
|------------|-----------|-----------|
| **plan.md** | `prompts/plan.md` → `.opencode/task-system/prompts/plan.md` | Crear plan desde backlog |
| **task.md** | `prompts/task.md` → `.opencode/task-system/prompts/task.md` | Definir tarea individual |
| **iter-loop-tools.md** | `prompts/iter-loop-tools.md` → `.opencode/task-system/prompts/iter-loop-tools.md` | Una iteración del loop (1 paso) |
| **pipeline.md** | `.opencode/commands/pipeline.md` | Entry point: backlog / task ID / run |
| **Plan file** | `docs/plans/<fecha>-<nombre>.md` | Orquestación: qué tasks, en qué estado |
| **Task file** | `tasks/<ID>.md` (resuelve a `.opencode/skills/campaign-executor/tasks/<ID>.md`) | Profundidad: steps atómicos, blast radius |
| **RULES.md** | `skills/campaign-executor/RULES.md` | North star + reglas invariantes del sistema |

## Estados de una tarea

```
⬜ PENDING → ⏳ IN PROGRESS → ✅ COMPLETED
                              ❌ FAILED
```

## Ciclo completo

### Fase 0: Crear plan

1. Usuario invoca `/pipeline plan docs/Backlog.md`
2. `pipeline.md` carga `.opencode/task-system/prompts/plan.md`
3. El agente aplica triage gate a cada tarea
4. Crea `docs/plans/<fecha>-<nombre>.md` con todas las ✅ DO
5. Muestra comando para arrancar la ejecución (`/pipeline run` o `/loop-goal`)

### Fase 1: Discovery (por tarea, una vez)

1. Loop (`/loop-goal` → `iter-loop-tools.md`, o un sub-agente via `/pipeline run`) detecta tarea ⬜ PENDING
2. Agente detecta que el task file no existe
3. Auto-detecta tipo de tarea (Rust / Frontend / Python / ...)
4. `codegraph_explore` para blast radius
5. Web research si hay ambigüedad
6. Descompone en steps atómicos
7. Crea `tasks/<ID>.md` (`.opencode/skills/campaign-executor/tasks/<ID>.md`) con steps, contrato, herramientas
8. Actualiza plan file: Estado → ⏳ IN PROGRESS
9. Recitation → STOP

### Fase 2: Ejecución (un step por iteración)

```
States válidos (C0 — Statewright pattern, iter-loop-tools.md canonical):

  PLAN     → ACT
  ACT      → VERIFY
  VERIFY   → PLAN      (falló → reintentar)
  VERIFY   → STALL     (3 same-error → bloqueo)
  VERIFY   → COLLATERAL (pasó → colaterales)
  COLLATERAL → RESEARCH (ambigüedad → investigar)
  RESEARCH → ACT       (investigado → implementar)
  COLLATERAL → EVALUATE (sin errores → evaluar)
  EVALUATE → REVIEW    (auto-evaluación pasa → revisión)
  EVALUATE → ACT       (auto-evaluación falla → re-implementar)
  REVIEW   → VERIFY    (review encuentra issues → re-verificar)
  REVIEW   → ACCEPT    (review pasa → aceptar)
  ACCEPT   → CLOSE     (aceptado → cerrar/commit)

  STALL → ❌ FAILED (agotado)
```

1. Loop (`/loop-goal` u orquestador de `/pipeline run`) re-inyecta el prompt de iteración para el próximo step
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
Contrato: comando de verificación exacto + resultado (p.ej. `cargo nextest run --profile audit --workspace --build-jobs 2` ✅)
Invariantes: qué NO se puede romper al continuar (dominio/seguridad — handoff transferible)
Comandos de verificación: comando exacto + resultado esperado/obtenido
Deuda: lo que queda incompleto / pendiente para la próxima iteración ("ninguna" si no aplica)
Próxima tarea si completa: TASK-N+1 — ID
last-synced: YYYY-MM-DDTHH:MM
=== END RECITATION ===
```

> La recitation es un handoff transferible (eng-03-project.md:198): quien continúe
> debe poder seguir SIN preguntar al anterior — invariantes de dominio, comandos
> de verificación y deuda pendiente son obligatorios (gap-01 §3.3-18).

## Escalation ladder

Fuente canónica: `prompts/subagent-recovery.md` (SARL) + MoM ladder en `prompts/iter-loop-tools.md`. Resumen: RESUME misma sesión → RETRY fresco → STRATEGY distinta → Gate V (`question-gates.md`) → ESCALATE. Umbral único: 2 fallas verify mismo-error. Presupuestos: ver § Budget management.

## Sub-Agent Recovery Protocol (SARL)

**Canónico: `prompts/subagent-recovery.md`** — no duplicar acá. Reglas clave:
1. Clasificación: ✅ COMPLETO / 🟡 INCOMPLETO / ❌ FALLIDO / ⚠️ SIN-FORMATO (bloque `RESULTADO`, ver pipeline-full.md).
2. INCOMPLETE nunca se trata como FAILED (casi siempre se arregla con RESUME).
3. Los steps ✅ y el Context Save Point son estado durable — jamás se rehacen.
4. Entre intentos corre `campaign_verify_cmd` del contrato (sin verify no cuenta).
5. Cada recovery consume budget; 3 resultados no-DONE seguidos → pausar y preguntar al usuario.

## Budget management

**Fuente única de números: `BUDGET_LIMITS` en `.opencode/task-system/mcp/campaign-server.mjs`** — los prompts referencian, no duplican. Si esta tabla diverge del server, manda el server; corregir acá.

| Control | Límite (BUDGET_LIMITS) | Comportamiento |
|---------|------------------------|----------------|
| `maxIterations` | 10 | Al alcanzar → ❌ FAILED |
| `maxSubAgents` | 40 | HARD STOP + reporte parcial |
| `maxConsecutiveFails` | 5 | FAIL_MODE pasa a "stop" |
| `maxToolCalls` | 15 | `campaign_verify_cmd` rechaza |
| `maxDurationMinutes` | 120 | Budget expired → ❌ |
| NO_PROGRESS_LIMIT (stagnation, prompt-level) | 3 | `campaign_stalled_tasks` + pausa |
| Contexto inicial | < 20% (~40k tokens) | Si excede → usar sub-agentes |

Umbral transversal: **2 fallas de verify con el mismo error (archivo+línea+mensaje) → ❌ FAILED** — mismo número en MoM ladder, stagnation detection y SARL.

## Ejecución paralela

Cuando `/pipeline run` usa `FAIL_MODE=parallel`, identifica tareas independientes
(sin dependencias entre sí, sin archivos compartidos) y las ejecuta en
waves concurrentes:

```
Wave 0: tareas sin dependencias → N sub-procesos paralelos
Wave 1: tareas que dependen de Wave 0
Wave 2: tareas que dependen de Wave 1
...
```

MAX_CONCURRENT = min(3, tareas_en_wave). Cada tarea en paralelo usa su
propia invocación de `opencode run`.

## Contrato: vago vs verificable

| ❌ Vago | ✅ Verificable |
|---------|----------------|
| "Arreglar el bug de memoria" | "tests/test_memory.rs pasa, cargo machete 0 warnings, cargo nextest run pasa" |
| "Mejorar la web" | "npx tsc --noEmit 0 errors, npm run lint 0 errors" |
| "Refactorizar módulo" | "cargo check --workspace, clippy sin warnings nuevos, tests existentes pasan" |
| "Funciona bien" | "cargo build && cargo nextest run pasa, y comportamiento específico funciona" |

## System Integration

campaign-executor es el núcleo del sistema de tareas. Se relaciona con:

| Componente | Relación |
|------------|----------|
| `AGENTS.md` | Path resolution: `tasks/<ID>.md` → `.opencode/skills/campaign-executor/tasks/<ID>.md` |
| `pipeline.md` (command) | Entry point: `/pipeline plan\|task\|run` → invoca campaign-executor |
| `plan.md` (prompt) | Crea plan file desde Backlog, delega a campaign-executor |
| `iter-loop-tools.md` (prompt) | State machine ejecución — per-state tool enforcement vía MCP |
| `progreso` | Post-commit: migra tarea completada de Backlog a progreso |
| `unified-review --mode certify --profile vantadb` | Verify pre-push: certificación completa (replaces legacy vantadb-certify) |
| `ponytail` | Siempre activo: escalera YAGNI en cada step |
| `RULES.md` | North star invariante — no se edita durante ejecución |

**Integración con pipeline commands:**

```
/pipeline plan docs/Backlog.md   → plan.md → docs/plans/<file>.md
/pipeline task DRV-NN            → task.md (define task) → pipeline-full.md (ejecuta) → vanta-*
/pipeline run                    → pipeline-run.md (orquestador) → pipeline-full.md por sub-agente
                                   → subagent-recovery.md (SARL) para resultados no-DONE
/pipeline ejecución              → iter-loop-tools.md (loop, 1 iteración)
```

**Prompt canónico de ejecución:** `pipeline-full.md` es la forma ÚNICA de ejecutar una tarea
(DISCOVERY → EJECUCIÓN → CIERRE). `/pipeline task`, `/pipeline run` y vanta-lead delegan a él.
Todos los sub-agentes devuelven el bloque `RESULTADO` (ver § Resultado de pipeline-full.md).

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

Si instalaste el plugin `opencode-loop` como mecanismo de loop:

```bash
oc plugin install opencode-loop
```

Uso con Goal Mode (el plugin monitorea idle events y re-inyecta):

```
/loop-goal "Cargá campaign-executor. Ejecutá el plan file docs/plans/<nombre>.md
una tarea a la vez. Después de cada tarea, actualizá el plan file y escribí
la recitation. No avances sin haber completado verificación, commit, progreso."
```

## Apéndice C: Innovaciones incorporadas (2026)

| Innovación | Fuente | Dónde |
|------------|--------|-------|
| **VISION.md (north star)** | Steinberger | RULES.md |
| **Stagnation detection** | Anthropic: no-progress detection | iter-loop-tools.md |
| **Budget ceilings** | explainx.ai | SKILL.md |
| **Evaluator-optimizer** | Lilian Weng: agent self-review | iter-loop-tools.md MODO CIERRE |
| **State machine guardrails** | Statewright (415⭐) | iter-loop-tools.md State Machine C0 |
| **Self-Harness propose-evaluate-accept** | Self-Harness (Anthropic) | iter-loop-tools.md Self-Harness Gate |
| **Parallel orchestrator workers** | Anthropic: building effective agents | pipeline-run.md waves |
| **Auto-type discovery** | TaskWeaver (9k⭐) | iter-loop-tools.md MODO DISCOVERY |
| **Ponytail (shortest path)** | awesome-harness-engineering | RULES.md, iter-loop-tools.md |
| **Zero-code planning** | task-executor hybrid-prompt | iter-loop-tools.md MODO DISCOVERY step 4 |
| **Revisión cada N tareas** | backlog-executor loop-prompt | pipeline-run.md step 5.f |
| **Context Save Point** | backlog-executor §7 | task file format, iter-loop-tools.md Cierre |
| **FAIL_MODE triple (stop/skip/parallel)** | task-executor batch-prompt | pipeline-run.md |
| **Parallel DAG + waves** | task-executor batch-prompt §1.5 | pipeline-run.md step 6 |
| **Probes de integridad** | backlog-executor §8 | SKILL.md, pipeline-run.md step 3 |
| **Compaction periódica** | backlog-executor §7 | SKILL.md |
| **Prompt Templates** | backlog-executor §11 | commands/pipeline.md Apéndice

### Referencias locales clonadas

```
.opencode/references/
  awesome-harness-engineering/   ← catálogo patrones (walkinglabs, 3.6k⭐)
  statewright/                   ← eliminado 2026-08-23 (clon completo; patrón C0 ya extraído en config/state-tools.mjs; recuperable de GitHub)
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
| El agente hace 2+ tareas en un turno | Ignoró "una iteración" | Usar `/loop-goal` con `iter-loop-tools.md` |
| Loop no detecta progreso | Recitation faltante | Verificar que el agente escribió RECITATION |
| Plan file corrupto | Regex no parsea | Revisar encoding de emojis |
| Estado de tarea no detectado | Task file usa `- **Estado:** ⬜ PENDING` (markdown bold) | El parser ahora acepta `Estado:\**\s*[⬜⏳✅❌]` — tolera `**` y espacios alrededor del emoji |
| last-synced desfasado | Task file editado sin plan file | El pipeline re-sincroniza automáticamente |
| opencode run colgado | Tool output muy grande | El timeout del limit aborta |
| Misma tarea reprocesada infinitamente | Stall detection mal configurado | Verificar NO_PROGRESS_LIMIT en iter-loop-tools.md |

## Diagrama de flujo completo

```
/pipeline plan docs/Backlog.md
  │
  ├─ plan.md: triage gate → docs/plans/<fecha>.md
  │
  ├─ /pipeline run → pipeline-run.md (orquestador)
  │   │
  │   ├─ por tarea: routing por `Ruta` → sub-agent vanta-* → ejecuta pipeline-full.md
  │   │   ├─ DISCOVERY: task file (si no existe) → EJECUCIÓN → CIERRE
  │   │   ├─ resultado no-DONE → subagent-recovery.md (RESUME → RETRY → STRATEGY → ESCALATE)
  │   │   └─ de una tarea por iteración
  │   │
  ├─ /loop-goal → iter-loop-tools.md (una iteración, paso a paso, MCP)
  │   │
  │   ├─ State Machine: PLAN → ACT → VERIFY
  │   ├─ Retry ladder (4 escalones)
  │   ├─ Agente de Diagnóstico en verify falla
  │   ├─ Stagnation Detection (3 same-error = stop)
  │   ├─ Errores colaterales (rápido→fix, lento→Backlog)
  │   ├─ Evaluator-Optimizer (3 ejes)
  │   ├─ Self-Harness Gate (propose→evaluate→accept)
  │   ├─ Pre-commit Gate
  │   ├─ git commit
  │   ├─ skill progreso
  │   └─ RECITATION → STOP
  │   │
  │   └─ (repite hasta que todas las tareas estén ✅ o ❌)
  │
  └─ Resumen final: N/M completadas
```

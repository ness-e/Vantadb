# VantaDB — Manual de Operación del Sistema

> **Propósito:** Documentar las relaciones, flujos de trabajo, reglas y gobierno
> de todos los componentes del sistema de agentes: commands, skills, prompts,
> agents, task system, MCP servers, y su integración.

---

## Tabla de Contenidos

1. [Arquitectura General](#1-arquitectura-general)
2. [Path Resolution — Cómo se Resuelven las Rutas](#2-path-resolution)
3. [Commands — Entry Points del Usuario](#3-commands)
4. [Task System — campaign-executor](#4-task-system)
5. [C0 State Machine — El Corazón de la Ejecución](#5-c0-state-machine)
6. [Skills Engineering — Lifecycle Completo](#6-skills-engineering)
7. [Skills VantaDB — Integración Vertical](#7-skills-vantadb)
8. [Agents (vanta-*) — Roles Especializados](#8-agents)
9. [MCP Servers — Herramientas del Sistema](#9-mcp-servers)
10. [Flujos de Integración](#10-flujos-de-integración)
11. [Buenas Prácticas](#11-buenas-prácticas)
12. [Reglas y Prohibiciones](#12-reglas-y-prohibiciones)
13. [Problemas Conocidos y Troubleshooting](#13-troubleshooting)
14. [Glosario](#14-glosario)

---

## 1. Arquitectura General

```
USUARIO
  │
  ├─ /pipeline ...        → pipeline.md      → task-system/prompts/  → campaign-executor
  ├─ /audit ...           → audit.md          → skills/unified-review  (replaces vantadb-audit + vantadb-certify + vantadb-full-review)
  ├─ /build ...           → build.md          → dev-tools/scripts/
  ├─ /ship                → ship.md           → campaign-executor + certify
  ├─ /rollback            → rollback.md       → git revert + docs
  ├─ /status              → status.md         → git + plan files + progreso
  ├─ /backlog             → backlog.md        → docs/Backlog.md + priorización
  ├─ /spec                → spec.md           → spec-driven-development
  ├─ /webperf             → webperf.md        → Playwright MCP
  └─ /code-simplify       → code-simplify.md  → ponytail-audit
```

### Capas del Sistema

| Capa | Componentes | Rol |
|------|-------------|-----|
| **Entry** | 10 commands en `.opencode/commands/` | Detectan el intento del usuario, resuelven rutas, orquestan |
| **Pipeline** | `task-system/prompts/` (7 prompts) | Instrucciones detalladas para el agente por fase |
| **Ejecución** | `campaign-executor` (SKILL + RULES + prompts) | Loop (orquestador `/pipeline run` o iteración `/loop-goal`), state machine, recitation |
| **Skills** | 25 skills engineering + 7 skills VantaDB | Workflows especializados obligatorios |
| **Agents** | 8 vanta-* agents | Roles con perspectiva y herramientas restringidas |
| **MCP** | CodeGraph, Playwright, cargo-mcp, rust-analyzer-mcp | Tools de infraestructura |
| **Dev Tools** | Justfile, cargo-*, dev-tools/scripts/ | Automatización local |

### Relaciones entre capas

```
COMMAND → resuelve paths → carga PROMPTS → invoca SKILLS → escribe PLAN/TASK files
                                                              │
AGENTS (vanta-*) ← task tool ────────┘                       │
                                                              ↓
                                                    MCP SERVERS (codegraph, cargo, etc.)
                                                              │
                                                              ↓
                                                    DEV TOOLS (just, cargo, pwsh)
```

Los commands son el **entry point**. Los prompts son el **cerebro** (qué hacer paso a paso).
Las skills son el **manual** (cómo hacerlo bien). Los agents son **especialistas** (hacen una cosa).
Los plan/task files son la **memoria persistente** entre iteraciones.

---

## 2. Path Resolution

Todas las rutas relativas en comandos, prompts y skills se resuelven así:

| Referencia en el archivo | Resuelve a |
|---|---|
| `prompts/X.md` | `.opencode/task-system/prompts/X.md` |
| `skills/X` | Buscar ↓ en orden: `.opencode/skills/X/` → `.agents/skills/X/` → `~/.agents/skills/X/` (usar la primera que exista; preferir la copia del proyecto → `.opencode/` sobre `.agents/` sobre global) |
| `tasks/<ID>.md` | `.opencode/skills/campaign-executor/tasks/<ID>.md` (si no existe: `.opencode/skills/campaign-executor/tasks/complete/<ID>.md` → `.opencode/skills/campaign-executor/tasks/closed/<ID>.md`) |
| `docs/plans/X.md` | `docs/plans/X.md` (ruta directa) |

**Nota:** aunque la regla de resolución para `tasks/<ID>.md` arranca en la raíz (`tasks/`), las tareas **completadas** viven en `tasks/complete/` y las **cerradas sin resolver** en `tasks/closed/`. Un `grep` o `Read` sobre la raíz sola se pierde esas. Buscar en los tres niveles cuando el ID no aparezca en la raíz.

**Regla:** Siempre usar la forma corta (`tasks/P1-5.md` en vez de la ruta absoluta).
Nunca referenciar `.tasks/` (no existe — error legacy corregido).

---

## 3. Commands

### 3.1 Pipeline — `/pipeline`

**Archivo:** `.opencode/commands/pipeline.md`

| Modo | Uso | Qué hace |
|------|-----|----------|
| `plan` | `/pipeline plan docs/Backlog.md` | Triage gate, crea `docs/plans/<fecha>.md` |
| `task` | `/pipeline task DRV-NN` | Investiga, crea task file con steps |
| `run` | `/pipeline run -PlanFile ...` | Ejecuta backlog completo con sub-agentes (profundidad unificada) |
| `interactive` | `/pipeline` sin args | Menú interactivo |

**Flujo `plan`:**
1. Lee Backlog.md → aplica triage gate (DO/DEFER/SKIP/BLOQUEADO)
2. Crea plan file con tasks priorizadas
3. Muestra comando para arrancar la ejecución (`/pipeline run`)

**Flujo `task`:**
1. `codegraph_explore` para blast radius del cambio
2. Auto-detecta tipo (Rust / Frontend / Python / ...)
3. Crea task file con steps atómicos + contrato verificable

**Flujo `run`:**
1. Carga `pipeline-run.md` (orquestador) — rutea cada tarea por `Ruta` a un sub-agente vanta-*
2. Por cada tarea: sub-agente ejecuta `pipeline-full.md` completo (DISCOVERY → EJECUCIÓN → CIERRE)
3. Resultado no-DONE → SARL (`subagent-recovery.md`: RESUME → RETRY → STRATEGY → ESCALATE)
4. Al completar: commit + `skill progreso`

### 3.2 Audit — `/audit`

**Archivo:** `.opencode/commands/audit.md`

| Modo | Alcance |
|------|---------|
| `quick` | CLI checks solo (fmt, clippy, test core) |
| `certify` | Quick + security + performance + certify gate |
| `review` | Code review + deep module review |
| `full` | Todo: CLI + security + perf + review + deep + ISO + certify |

**Flujo:** Phase 0 (pre-check) → Phase 1 (CLI) → Phases 2-8 (skills) → Report en `docs/audit-reports/`

Cada ejecución de `/audit` crea un plan file (`docs/plans/plan-audit-*.md`) con task_id y resultados por fase.

### 3.3 Otros Commands

| Command | Archivo | Propósito |
|---------|---------|-----------|
| `/build` | `build.md` | Compila y verifica builds (Rust, web, Python) |
| `/ship` | `ship.md` | Fan-out GO/NO-GO con certify pre-push |
| `/rollback` | `rollback.md` | Revierte un ship fallido |
| `/status` | `status.md` | Dashboard del sistema (git, plan files, progreso) |
| `/backlog` | `backlog.md` | Revisar backlog, listar tareas activas, recomendar prioridad |
| `/spec` | `spec.md` | Spec-Driven Development — escribir spec antes de código |
| `/webperf` | `webperf.md` | Web performance audit con Playwright |
| `/code-simplify` | `code-simplify.md` | Simplifica código (ponytail-audit) |

---

## 4. Task System

### 4.1 Componentes del Pipeline

| Componente | Ruta real | Propósito |
|------------|-----------|-----------|
| **plan.md** | `.opencode/task-system/prompts/plan.md` | Crear plan desde backlog |
| **task.md** | `.opencode/task-system/prompts/task.md` | Definir tarea individual |
| **iter-loop-tools.md** | `.opencode/task-system/prompts/iter-loop-tools.md` | Una iteración del loop |
| **pipeline.md** | `.opencode/commands/pipeline.md` | Entry point |
| **pipeline-run.md** | `.opencode/task-system/prompts/pipeline-run.md` | Orquestador de backlog (sub-agentes) |
| **pipeline-full.md** | `.opencode/task-system/prompts/pipeline-full.md` | Prompt canónico de ejecución de tarea |
| **SKILL.md** | `.opencode/skills/campaign-executor/SKILL.md` | Referencia completa |
| **RULES.md** | `.opencode/skills/campaign-executor/RULES.md` | Reglas invariantes |
| **Plan file** | `docs/plans/<fecha>-<nombre>.md` | Orquestación de tasks |
| **Task file** | `tasks/<ID>.md` | Steps atómicos de una tarea |

### 4.2 Ciclo de Vida de una Tarea

```
/pipeline plan docs/Backlog.md
  │
  ├─ plan.md: triage gate → docs/plans/<fecha>.md
  │
  ├─ FASE 1: DISCOVERY (primer turno de cada tarea)
  │   ├─ auto-detect tipo (Rust / Frontend / Python / ...)
  │   ├─ codegraph_explore → blast radius
  │   ├─ web research si ambigüedad
  │   ├─ crear task file con steps atómicos + contrato
  │   └─ plan file → ⏳ IN PROGRESS
  │
  ├─ FASE 2: EJECUCIÓN (1 step por iteración del loop)
  │   ├─ State Machine: PLAN → ACT → VERIFY
  │   ├─ Retry ladder (4 escalones)
  │   ├─ Stagnation Detection (3 same-error = stop)
  │   ├─ Errores colaterales: rápido (<30min) → fix, lento → Backlog
  │   ├─ Evaluator-Optimizer (3 ejes: correctitud, simplicidad, consistencia)
  │   ├─ Self-Harness Gate (propose → evaluate → accept)
  │   ├─ Pre-commit Gate
  │   ├─ git commit
  │   ├─ skill progreso (Trigger 1)
  │   └─ RECITATION → STOP
  │
  ├─ FASE 3: CIERRE (cuando todos los steps están ✅)
  │   ├─ Verificación full (build + test + fmt + clippy + extra)
  │   ├─ Plan file → ✅ COMPLETED
  │   └─ RECITATION → STOP
  │
  └─ (repite hasta que todas las tareas estén ✅ o ❌)
```

### 4.3 Estados de una Tarea

```
⬜ PENDING → ⏳ IN PROGRESS → ✅ COMPLETED
                              ❌ FAILED
```

### 4.4 Formato de Plan File

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

### 4.5 Formato de Task File

```markdown
# TASK-ID: Descripción

## Metadata
- **Plan file:** [ruta]
- **Creado:** YYYY-MM-DDTHH:MM
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

## Context Save Point
- **Fecha:** ISO
- **Branch:** nombre
- **Decisiones:** X sobre Y porque [razón breve]
```

### 4.6 Recitation Block

La recitation es el handoff entre iteraciones del loop. Sin ella, la próxima iteración arranca perdida.

```
=== RECITATION ===
Objetivo activo: TASK-N — ID
Estado: plan / act / verify / stall / research / collateral / evaluate / review / accept / completed / failed
Última acción: edit en src/engine.rs
Resultado: ✅ / ❌
State: ESTADO (desde: ESTADO_ANTERIOR)
Próxima acción: paso concreto (archivo + comando)
Contrato: "condición verificable"
Próxima tarea si completa: TASK-N+1 — ID
last-synced: YYYY-MM-DDTHH:MM
=== END RECITATION ===
```

### 4.7 Retry Ladder

| Escalón | Acción |
|---------|--------|
| 1 | Retry con feedback del error procesado |
| 2 | Contexto fresco: resumir lo aprendido (~200 tokens) |
| 3 | Estrategia materialmente distinta |
| 4 | Escalar a humano: documentar intentos, commit WIP, ❌ FAILED |

### 4.8 Budget Management

| Control | Default | Hard Limit |
|---------|---------|------------|
| Iteraciones por tarea | 5 | 10 |
| Sub-agentes totales | 20 | 40 |
| Consecutive fails | 3 | 5 |
| Tool calls por tarea | 8 | 15 |
| Duración por tarea | 60min | 120min |
| Stagnation consecutiva | 3 | 5 |

---

## 5. C0 State Machine

La state machine es el **corazón de la ejecución**. Gobierna cada iteración del agente.

```
States válidos (Statewright pattern, iter-loop-tools.md canonical):

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

**Reglas de la state machine:**
- Solo un estado activo por iteración
- Cada transición requiere una acción verificable
- STALL es terminal: no se sale sin intervención humana
- El estado se persiste en la recitation (no en contexto)

---

## 6. Skills Engineering

### 6.1 Lifecycle Mapping

Las 25 skills de ingeniería se asignan automáticamente según la fase del trabajo:

| Fase | Skill | Disparador |
|------|-------|-----------|
| **DEFINE** | `spec-driven-development` | Nueva feature, API, cambio significativo |
| **DEFINE** | `interview-me` | Requisitos ambiguos |
| **DEFINE** | `idea-refine` | Concepto vago → propuesta concreta |
| **PLAN** | `planning-and-task-breakdown` | Spec listo → tareas pequeñas |
| **BUILD** | `incremental-implementation` | Implementar en slices verticales |
| **BUILD** | `test-driven-development` | Lógica nueva, bugs |
| **BUILD** | `context-engineering` | Sesión nueva, tarea compleja |
| **BUILD** | `source-driven-development` | Decisiones de framework/library |
| **BUILD** | `doubt-driven-development` | Stakes altos (producción, seguridad) |
| **BUILD** | `frontend-ui-engineering` | UI nueva en web/ |
| **BUILD** | `api-and-interface-design` | APIs, boundaries de módulos |
| **VERIFY** | `systematic-debugging` | Tests fallan, builds rotos |
| **VERIFY** | `browser-testing-with-devtools` | Depurar algo en navegador |
| **REVIEW** | `code-review-and-quality` | Antes de mergear |
| **REVIEW** | `code-simplification` | Código más complejo de lo necesario |
| **REVIEW** | `security-and-hardening` | Input de usuario, auth, datos |
| **REVIEW** | `performance-optimization` | Performance o regresiones |
| **SHIP** | `git-workflow-and-versioning` | Commits atómicos |
| **SHIP** | `ci-cd-and-automation` | CI/CD pipelines |
| **SHIP** | `shipping-and-launch` | Antes de deploy |
| **SHIP** | `documentation-and-adrs` | Decisiones arquitectónicas |
| **SHIP** | `deprecation-and-migration` | Remover sistemas viejos |
| **SHIP** | `observability-and-instrumentation` | Telemetría |
| **META** | `using-agent-skills` | Cómo usar este pack |

### 6.2 Carga de Skills

Siempre usar `skill <nombre>`:

```
skill code-review-and-quality
skill security-and-hardening
skill systematic-debugging
```

**Prohibido:** Saltarse la carga de una skill que aplica. Si hay duda, cargarla igual.

**Skills que NO son skills (son MCP tools):**
- `codegraph_explore` — es MCP tool de CodeGraph, no una skill
- `metasearchmcp_search_web` — es MCP tool de MetaSearchMCP
- `argus_extract_content` — es MCP tool de Argus

---

## 7. Skills VantaDB

Skills específicas del proyecto VantaDB. Cada una tiene un rol en el pipeline.

### 7.1 `unified-review` — Review, Audit & Certification Unificado (replaces legacy skills)

**Rol:** Único skill que reemplaza `vantadb-full-review`, `vantadb-certify` y `vantadb-audit`.
Orquesta sub-agentes en paralelo, tiene sistema de perfiles YAML, y 4 modos de operación.

**Ubicación:** `.opencode/skills/unified-review/SKILL.md` (1198 líneas)

**Modos:**
| Modo | Comando | Equivalente legacy |
|------|---------|--------------------|
| **quick** | `skill unified-review --mode quick` | — (nuevo) |
| **certify** | `skill unified-review --mode certify --profile vantadb` | `vantadb-certify` |
| **review** | `skill unified-review --profile vantadb` (default) | `vantadb-full-review` |
| **full** | `skill unified-review --mode full --profile vantadb` | `vantadb-full-review` + extra |
| **alias** | `/audit` | `vantadb-audit` (backwards compat) |

**Fases (perfil VantaDB):**
| Fase | Sub-agente | Contenido |
|------|-----------|-----------|
| L0 | — (directo) | Codegraph — impacto de cambios |
| L1 | vanta-worker | Rust: fmt + check + clippy + semver-checks + audit + deny + nextest + machete |
| L2 | vanta-worker | Bindings: Python + WASM + TypeScript SDK |
| L3 | vanta-worker | Web: Next.js build + lint + tsc |
| L4 | vanta-lead | CI/CD parity + dependencias |
| L5 | vanta-docs | Documentación coverage |
| L6 | vanta-arch | Arquitectura + codegraph layering |
| L7 | vanta-audit | Seguridad (OWASP ASVS) |
| L8 | vanta-tuner | Performance benchmarks |
| L9 | vanta-audit | Code review multi-skill |

**Arquitectura:** Fan-out de sub-agentes paralelos con contratos JSON. El orquestador consolida findings, scores, y recomendaciones. Máximo 2 rounds de profundidad. Budget: 10 sub-agent calls en modo full.

**Perfiles:**
- `profiles/default.yml` — genérico para cualquier proyecto Rust/TS/Python
- `profiles/vantadb.yml` — hereda default + Rust workspace, bindings, web frontend, scoring

**Reportes:** `docs/reviews/review-<mode>-<timestamp>.md`
**Pre-push barrier:** template PowerShell en `templates/pre-push.ps1` (SIPP). No instalado — verificación manual con `dev-tools/verify.ps1` (Regla 1).

**Verificación semver:** Incluye `cargo semver-checks check-release --workspace` en L1 como gate pre-publish obligatorio.

### 7.4 `review-deep` — Revisión por Módulo

**Rol:** Loop que itera módulo por módulo, investiga cada hallazgo en internet, compara con competidores, evalúa prioridad.

**Ubicación:** `.opencode/skills/review-deep/SKILL.md` (474 líneas) + `loop-prompt.md` (98 líneas)

**Diferencia con full-review:** No es one-shot. Es un loop que corre tantas iteraciones como módulos tenga el proyecto.

**Arquitectura:**
```
/loop-goal --prompt-file .opencode/skills/review-deep/loop-prompt.md "MODULE=..."
  │
  ├─ F0: Cargar skills según tipo de módulo
  ├─ F1-F3: Análisis estático (codegraph, rust-analyzer, cargo)
  ├─ F4-F5: Web research (metasearchmcp, argus)
  ├─ F6: Triage → Backlog.md
  ├─ F6b: Scorecard (PowerShell JSON)
  └─ F7: Reporte + Yield
```

**Tool lock-in (best-effort):**
| Fase | Tools ideales |
|------|---------------|
| F1-F3 (análisis) | codegraph, Read, Grep, cargo-mcp |
| F4-F5 (research) | metasearchmcp, argus, Read |
| F6 (triage) | Edit, Write, Read (Backlog.md) |
| F7 (reporte) | Write, Read |

### 7.5 `progreso` — Migración de Tareas

**Rol:** Mueve tareas completadas de Backlog.md a progreso/README.md. Mantiene la documentación sincronizada. Las filas completadas se **eliminan** de Backlog.md y quedan en `docs/progreso/README.md` — no acumular filas tachadas. Los items removidos del backlog se archivan a `docs/progreso/BACKLOG_HISTORY.md` (no se borran en silencio). Los planes completados se mueven a `docs/plans/archive/`.

**Ubicación:** `.opencode/skills/progreso/SKILL.md` (157 líneas)

**Triggers:**
| Trigger | Cuándo | Qué hace |
|---------|--------|----------|
| 1 | Tarea ✅ | Elimina la fila de Backlog + migra a progreso; plan completado → `docs/plans/archive/`, actualiza docs |
| 2 | Nueva tarea | Verifica que la anterior esté migrada |
| 3 | Mensual | Mantenimiento: icebox, dedup, cross-check, archivar removidos |

**Commit policy:**
- **Standalone** (sin campaign-executor): no commit — esperar instrucción
- **Desde campaign-executor**: el executor maneja commits automáticos
- Registrar decisión: `campaign_memory_write(file="decisions", ...)`

**Integración:** Todas las skills y commands cargan `progreso` al inicio y al completar tareas.

### 7.6 `campaign-executor` — Núcleo del Task System

**Rol:** Orquesta la ejecución de campañas completas desde backlog. Es el cerebro del pipeline.

**Ubicación:** `.opencode/skills/campaign-executor/SKILL.md` (420 líneas) + `RULES.md` (413 líneas)

**Relaciones con otros componentes:**
| Componente | Relación |
|------------|----------|
| `AGENTS.md` | Path resolution: `tasks/<ID>.md` → `.opencode/skills/campaign-executor/tasks/<ID>.md` |
| `pipeline.md` | Entry point: `/pipeline plan\|task\|run` |
| `plan.md` (prompt) | Crea plan file desde Backlog |
| `iter-loop-tools.md` (prompt) | State machine ejecución |
| `progreso` | Post-commit: migra tarea completada |
| `unified-review --mode certify --profile vantadb` | Verify pre-push (replaces vantadb-certify) |
| `ponytail` | Siempre activo: escalera YAGNI |
| `RULES.md` | North star invariante |

**Probes de integridad** (antes de cada tarea):
- Plan file existe y tiene al menos una task
- Recitation block es parseable
- No es la misma tarea sin progreso
- Git status está limpio
- No hay stalls previos sin resolver

---

## 8. Agents

### 8.1 Roles y Restricciones

| Agent | Rol | task tool | Invoica a |
|-------|-----|-----------|-----------|
| `vanta-arch` | Systems architect | ✅ permitido | Especialistas |
| `vanta-worker` | Implementador general | ✅ permitido | Especialistas |
| `vanta-engine` | Vector search / HNSW | ✅ permitido | Especialistas |
| `vanta-lead` | Coordinador (manual) | ✅ permitido | Cualquiera |
| `vanta-audit` | Security/correctness | ❌ denegado | Nadie (leaf) |
| `vanta-chaos` | Chaos engineering | ❌ denegado | Nadie (leaf) |
| `vanta-tuner` | Performance optimization | ❌ denegado | Nadie (leaf) |
| `vanta-docs` | Technical writer | ❌ denegado | Nadie (leaf) |

### 8.2 Patrón de Uso

```
Orquestadores (arch, worker, engine, lead)
  │
  ├─ task tool → vanta-audit (security review)
  ├─ task tool → vanta-tuner (performance)
  ├─ task tool → vanta-docs (documentation)
  └─ task tool → vanta-chaos (fuzzing)

Especialistas (audit, chaos, tuner, docs)
  └─ NO pueden invocar a nadie
     Son leaf nodes del árbol de invocación
```

### 8.3 Cuándo Usar Cada Agent

| Situación | Agent |
|-----------|-------|
| Diseñar una nueva feature del core | `vanta-arch` |
| Implementar bindings (PyO3, WASM) | `vanta-worker` |
| Optimizar HNSW o distancia | `vanta-engine` |
| Revisar seguridad de PR | `vanta-audit` |
| Hacer fuzzing o chaos testing | `vanta-chaos` |
| Profiling y optimización | `vanta-tuner` |
| Escribir docs de API | `vanta-docs` |
| Coordinar campaña multi-tarea | `vanta-lead` (manual) |

---

## 9. MCP Servers

### 9.1 Activos

| MCP | Comando | Propósito |
|-----|---------|-----------|
| **CodeGraph** | `codegraph serve --mcp` | Grafo de conocimiento del código (7.3K símbolos) |
| **Pencil** | `mcp-server-windows-x64.exe` | Editor de archivos `.pen` (diseño UI) |
| **Playwright** | `@playwright/mcp` | Automatización de navegador |
| **cargo-mcp** | `cargo-mcp serve` | Comandos Cargo (check, clippy, test, build, fmt, add) |
| **rust-analyzer-mcp** | `rust-analyzer-mcp` | LSP completo (goto def, hover, references, diagnostics) |
| ~~Recraft~~ | ~~eliminado~~ | Sin API key |
| ~~rust-mcp-server~~ | ~~deshabilitado~~ | Bug MCP handshake, redundante |

### 9.2 Guía de Uso

| Situación | Qué usar |
|-----------|----------|
| Preguntas de código | **CodeGraph** → `codegraph_explore` (siempre primero) |
| Rust build/test/clippy | **cargo-mcp** → `cargo_check`, `cargo_clippy`, `cargo_test` |
| Navegación Rust | **rust-analyzer-mcp** → `symbols`, `definition`, `hover`, `diagnostics` |
| Web scraping/testing | **Playwright** → `navigate`, `click`, `screenshot`, `snapshot` |
| Diseño UI visual | **Pencil** → archivos `.pen` |
| Buscar en internet | **MetaSearchMCP** → `metasearchmcp_search_web` |
| Extraer contenido web | **Argus** → `argus_extract_content` |

---

## 10. Flujos de Integración

### 10.1 Desarrollo Diario

```
1. skill progreso                    → leer backlog, check WIP
2. skill writing-plans               → si la tarea tiene múltiples pasos
3. skill systematic-debugging         → si es un bug

4. git status                        → ¿hay cambios sin commit?
5. git log --oneline -5              → ¿qué se hizo en la última sesión?

6. Implementar con skills según tipo
7. just verify                       → fmt + clippy + test + deny
8. skill progreso                    → migrar tarea completada
```

### 10.2 Feature Completa (con pipeline)

```
/pipeline plan docs/Backlog.md       → crea plan file
/pipeline task FEAT-01               → crea task file con steps
/pipeline run -PlanFile ...          → orquestador (sub-agentes, profundidad unificada)
    ├─ por tarea: routing por Ruta → sub-agente vanta-* → pipeline-full.md
    ├─ discovery → implementación → verify → close + commit + progreso
    └─ ... hasta completar todas las tasks

skill unified-review --mode certify --profile vantadb   → pre-push gate (replaces vantadb-certify)
git push                              → CI
```

### 10.3 Auditoría

```
/audit quick                          → fmt + clippy + test core
/audit certify                        → audit completo + security + certify gate
/audit full                           → todo: deep review + ISO + certify

skill review-deep                     → loop por módulo (si se necesita profundo)
skill unified-review --profile vantadb             → one-shot report completo (replaces vantadb-full-review)
```

### 10.4 Pre-push / Ship

```
skill unified-review --mode certify --profile vantadb   → certificación completa (replaces vantadb-certify)
    ├─ L0: codegraph impact
    ├─ L1-L3: checks mecánicos (Rust, bindings, web)
    ├─ L4-L6: CI/CD parity + docs + arquitectura
    ├─ L7-L9: security audit + performance + code review
    └─ L10: findings consolidation

/ship                                 → fan-out GO/NO-GO
    ├─ Verifica certify pass
    ├─ Confirma rama destino
    └─ Muestra diff final
```

### 10.5 Corrección de Bugs

```
skill systematic-debugging            → root-cause analysis
skill test-driven-development         → red-green-refactor
skill code-review-and-quality         → review del fix
skill ponytail-review                  → over-engineering check
just verify                            → fmt + clippy + test + deny
skill progreso                         → migrar a progreso
```

### 10.6 Integración: Agent → Skill → Command

```
USUARIO: /pipeline task P1-5

COMMAND pipeline.md
  → Lee prompt task.md
  → Carga campaign-executor (skill)
  → Ejecuta codegraph_explore para blast radius
  → Crea tasks/P1-5.md con steps / delega ejecución a pipeline-full.md
  → Muestra comando para ejecutar (pipeline-full.md via /pipeline task o /build)

ORQUESTADOR (/pipeline run) ejecuta:
  → Rutea por Ruta → sub-agente vanta-* → inyecta pipeline-full.md
  → AGENTE lee task file
  → AGENTE carga skills según tipo (Rust → source-driven-development)
  → AGENTE implementa step
  → AGENTE verifica (cargo check, nextest)
  → AGENTE actualiza plan file
  → AGENTE escribe recitation
  → resultado no-DONE → SARL (subagent-recovery.md)

AL COMPLETAR:
  → AGENTE hace commit
  → skill progreso (Trigger 1)
  → siguiente tarea
```

---

## 11. Buenas Prácticas

### 11.1 Generales

1. **CodeGraph primero** — antes de grep/Read para preguntas estructurales. Resuelve en ms lo que grep busca en minutos.
2. **Cargar skills antes de actuar** — si una skill aplica, debe cargarse con `skill <nombre>`. No implementar sin spec, no mergear sin review.
3. **Recitation siempre** — después de cada acción, escribir el bloque RECITATION. Sin ella la próxima iteración arranca perdida.
4. **Un paso por turno** — OpenCode opera por turnos. Cada turno ejecuta UNA acción atómica. `/pipeline run` itera por vos con sub-agentes.
5. **Contratos verificables** — cada tarea tiene una condición booleana. "cargo nextest run pasa" no "funciona bien".
6. **Sync bidireccional** — plan file y task file se referencian mutuamente. Ambos tienen `last-synced`.
7. **Ponytalla escalera antes de escribir código** — ¿ya existe? ¿stdlib? ¿platform? ¿dependency? ¿una línea? Recién ahí: código mínimo.
8. **~100 líneas por commit** — si un cambio es más grande, partilo en más steps.

### 11.2 Para Commands

1. Cada command resuelve rutas según la tabla de Path Resolution
2. Los prompts se cargan con `Read` y se ejecutan secuencialmente
3. Al finalizar: escribir recitation y detenerse
4. No continuar a la siguiente tarea sin que el usuario lo pida

### 11.3 Para Skills VantaDB

1. `progreso` se carga al inicio de sesión y al completar cada tarea
2. `ponytail (full)` está siempre activo
3. `campaign-executor` se carga en modo task/run
4. Las skills de audit/review crean plan files automáticamente
5. `unified-review --mode certify --profile vantadb` es el pre-push gate definitivo (replaces vantadb-certify)

### 11.4 Para el Task System

1. Un plan file = una campaña (conjunto de tareas)
2. Un task file = una tarea con steps atómicos
3. El contrato es la condición de éxito — si no se cumple, la tarea no está completa
4. Stagnation detection: 3 mismo error = stop
5. Errores colaterales: rápido (<30min) → fixear, lento → Backlog
6. Nunca cambiar scope durante ejecución

### 11.5 Para Agents

1. Orquestadores pueden invocar especialistas vía `task` tool
2. Especialistas son leaf nodes — no invocan a nadie
3. Cada agente tiene herramientas restringidas según su rol
4. No usar agents para tareas que una skill resuelve

---

## 12. Reglas y Prohibiciones

### 12.1 Prohibiciones Absolutas

| # | Prohibición | Razón |
|---|-------------|-------|
| 1 | `continue-on-error: true` en GitHub Actions | Silencia fallos que nadie monitorea |
| 2 | Mergear a main sin `just verify` | El CI gate corre igual, más barato local |
| 3 | Ignorar un test flaky sin Issue | El Issue con tag `flaky` es el mínimo |
| 4 | Eliminar archivos sin grep de referencias | Regla 0 de AGENTS.md: medir impacto antes |
| 5 | Usar `docs/bitacora.md` | Archivo eliminado, reemplazado por plan files |
| 6 | Referenciar `.tasks/` como ruta | No existe — usar `tasks/<ID>.md` |
| 7 | Saltarse la carga de una skill que aplica | Las skills son obligatorias, no opcionales |
| 8 | Hacer 2+ tareas en un turno | El loop itera una por una |
| 9 | Auto-reportar "anda" sin verificación mecánica | `cargo check`/`nextest`/`tsc` son los únicos válidos |
| 10 | Introducir más deuda técnica de la que se elimina por PR | Saldo neto debe ser cero o negativo |

### 12.2 Reglas de la State Machine

- Un estado activo por iteración
- STALL no se resuelve solo — requiere intervención humana
- VERIFY siempre requiere un comando real, no auto-reporte
- El estado se persiste en recitation, nunca en contexto

### 12.3 Reglas del Loop

- Cada invocación ejecuta EXACTAMENTE UNA iteración
- El plan file y task file son la única fuente de verdad
- Sin recitation, la próxima iteración arranca perdida
- 3 iteraciones sin progreso = stop

### 12.4 Reglas de Skills

- `skill <nombre>` es el único método de carga válido
- No listar `codegraph_explore` como skill — es MCP tool
- No usar `<!-- ponytail:` — ponytail no parsea HTML comments
- No usar heredoc bash `cat > file << 'EOF'` en PowerShell — usar `ConvertTo-Json | Out-File`
- Tool names correctos: `metasearchmcp_search_web`, `argus_extract_content`, `codegraph_explore`

### 12.5 Reglas de Documentación

| Disparador | Acción |
|---|---|
| Nueva `pub fn`, endpoint HTTP, binding PyO3/WASM | Actualizar el `.md` en `docs/api/` en el mismo PR |
| Nueva documentación (guías/API/arquitectura ad-hoc) | NO en `docs/archive/`, `docs/research/`. Reportes del pipeline (`/audit`/`/review`) → `docs/audit-reports/`, `docs/reviews/` + registro en `docs/reports/INDEX.md` |
| Documentación técnica en español | Redirigir a inglés. Español solo para backlog/planning |
| Auditoría completada | Reporte en `docs/audit-reports/` |
| Decisión arquitectónica con tradeoff | ADR en `docs/architecture/adr/` o `campaign_memory_write` |

---

## 13. Troubleshooting

| Síntoma | Causa | Solución |
|---------|-------|----------|
| El agente hace 2+ tareas en un turno | Ignoró "una iteración" | Usar `/loop-goal` con `iter-loop-tools.md` |
| Loop no detecta progreso | Recitation faltante | Verificar bloque RECITATION al final del plan file |
| Plan file corrupto | Regex no parsea emojis | Revisar encoding |
| `last-synced` desfasado | Task file editado sin plan file | El pipeline re-sincroniza automáticamente |
| Misma tarea reprocesada | Stall detection mal configurado | Verificar `NO_PROGRESS_LIMIT` en iter-loop-tools.md |
| Skill no encontrada | Ruta incorrecta | Verificar que existe en `.opencode/skills/<name>/SKILL.md` |
| `codegraph` no responde | Proyecto no indexado | Ejecutar `codegraph init` (solo si el usuario lo pide) |
| Shell syntax error | Bash heredoc en PowerShell | Usar PowerShell nativo (`ConvertTo-Json`, `Out-File`) |
| Verificación pre-push requerida | Hooks no instalados (sistema PowerShell) | Correr `dev-tools/verify.ps1` manualmente antes de push (Regla 1); template SIPP en `templates/pre-push.ps1` |
| `bitacora.md` no encontrado | Archivo eliminado | Referenciar `docs/Backlog.md` o plan files |

---

## 14. Glosario

| Término | Definición |
|---------|-----------|
| **Command** | Entry point del usuario (`.opencode/commands/*.md`). Detecta intento, resuelve rutas, orquesta |
| **Skill** | Workflow especializado (`.opencode/skills/<name>/SKILL.md`). Pasos + criterios de salida |
| **Prompt** | Instrucción detallada para el agente (`.opencode/task-system/prompts/*.md`) |
| **Agent** | Rol con perspectiva y herramientas restringidas (`.opencode/agents/*.md`) |
| **Plan file** | Archivo de orquestación (`docs/plans/<fecha>.md`). Tasks, estados, recitation |
| **Task file** | Profundidad de una tarea (`tasks/<ID>.md`). Steps atómicos, blast radius |
| **Recitation** | Bloque de handoff entre iteraciones. Persiste estado y objetivo |
| **Loop / Orquestador** | Accionado por `/pipeline run` (sub-agentes) o `/loop-goal` (una iteración). Timeout, git check, sync. Reemplaza al ciclo PowerShell |
| **C0 State Machine** | 10 estados de ejecución (PLAN→ACT→VERIFY→...→CLOSE/FAILED) |
| **Path Resolution** | Tabla que resuelve rutas relativas a absolutas (AGENTS.md) |
| **Stall** | 3 iteraciones sin progreso (mismo error, mismo archivo) → FAILED |
| **Contrato** | Condición booleana verificable que define el éxito de una tarea |
| **Blast Radius** | Impacto de un cambio: callers, callees, implicaciones |
| **Ponytail** | Escalera de minimalismo: ya existe > stdlib > platform > dependency > 1 línea > mínimo |
| **MCP** | Model Context Protocol — protocolo de herramientas para LLMs |

---

## Apéndice A: Árbol de Decisión Rápido

```
¿Querés ejecutar tareas desde backlog?
  ├─ Sí → /pipeline plan docs/Backlog.md
  │       (crea plan file, luego /pipeline run)
  │
  └─ No → ¿Querés auditar el proyecto?
       ├─ Sí → /audit quick | certify | review | full
       │
       └─ No → ¿Querés hacer un cambio rápido?
            ├─ código → implementar con skills + just verify
            ├─ bug → skill systematic-debugging → TDD → fix
            └─ doc → skill documentation-and-adrs

Antes de push: skill unified-review --mode certify --profile vantadb
Después de completar: skill progreso
```

## Apéndice B: Archivos del Sistema

```
.opencode/
  AGENTS.md                          ← Configuración global, path resolution, reglas
  VANTADB-OPERATING-MANUAL.md        ← Este archivo
  commands/
    pipeline.md                      ← /pipeline (plan / task / run)
    audit.md                         ← /audit (quick / certify / review / full)
    build.md                         ← /build
    ship.md                          ← /ship
    rollback.md                      ← /rollback
    status.md                        ← /status
    backlog.md                       ← /backlog
    spec.md                          ← /spec
    webperf.md                       ← /webperf
    code-simplify.md                 ← /code-simplify
  task-system/
    config/                          ← Configuración del sistema
    enforcement/                     ← Reglas de enforcement C0
    mcp/                             ← MCP server del task system
    memory/                          ← Memoria persistente del agente
    prompts/
      plan.md                        ← Crear plan desde backlog
      task.md                        ← Definir tarea individual
      iter-loop-tools.md             ← Una iteración del loop (paso a paso)
      pipeline-full.md               ← Prompt canónico de ejecución de tarea
      pipeline-run.md                ← Orquestador de backlog (sub-agentes)
      subagent-recovery.md           ← SARL (recovery de sub-agentes)
      research-agent.md              ← Research agent prompt
      audit-full.md                  ← Audit full prompt
    sandbox/                         ← Sandbox de ejecución
    self-modification/               ← Auto-modificación del sistema
    traces/                          ← Trazas de ejecución
    validation/                      ← Validación de salidas
    workflows/                       ← Workflow definitions (bug-fix, feature-add, etc.)
  skills/
    campaign-executor/               ← Núcleo del task system
      SKILL.md (420L)                ← Referencia completa
      RULES.md (413L)                ← Reglas invariantes
      tasks/                         ← Task files (DRV-*, P0-*, P1-*, etc.)
    progreso/                        ← Migración de tareas
    unified-review/                  ← Review, audit & certification unificado (reemplaza a los 3 legacy) 
    review-deep/                     ← Revisión por módulo
    (19 skills engineering más)
  agents/
    vanta-arch.md                    ← Systems architect
    vanta-worker.md                  ← Implementador
    vanta-engine.md                  ← Vector search
    vanta-audit.md                   ← Security (leaf)
    vanta-chaos.md                   ← Fuzzing (leaf)
    vanta-tuner.md                   ← Performance (leaf)
    vanta-docs.md                    ← Docs (leaf)
    vanta-lead.md                    ← Coordinador (manual)
  references/
    definition-of-done.md            ← DoD checklist
    security-checklist.md            ← Security patterns
    performance-checklist.md         ← Performance patterns
    testing-patterns.md              ← Test patterns
    accessibility-checklist.md       ← Accessibility patterns
    observability-checklist.md       ← Observability patterns
    orchestration-patterns.md        ← Orchestration patterns
    awesome-harness-engineering/     ← Repositorio clonado (3,648 ★)
    statewright/                     ← State machine patterns (417 ★)
    deepclaude/                      ← Cost-saving proxy (Claude Code → DeepSeek/OpenRouter, 2,212 ★)
    darwin-godel-machine/            ← Harness evolution

raíz/
  .opencode/task-system/prompts/pipeline-run.md  ← Orquestador de backlog
  docs/
    Backlog.md                       ← Active tasks
    progreso/README.md               ← Completed tasks
    plans/                           ← Plan files
    audit-reports/                   ← Audit reports
    architecture/adr/                ← ADRs
  dev-tools/
    scripts/                         ← PowerShell scripts
    verify.ps1                       ← Pre-flight completa
    verify_changed.ps1               ← Quick verify
```

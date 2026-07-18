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
  ├─ /audit ...           → audit.md          → skills/vantadb-audit  → skills/vantadb-certify
  ├─ /build ...           → build.md          → dev-tools/scripts/
  ├─ /ship                → ship.md           → campaign-executor + certify
  ├─ /rollback            → rollback.md       → git revert + docs
  ├─ /status              → status.md         → git + plan files + progreso
  ├─ /spec                → spec.md           → spec-driven-development
  ├─ /webperf             → webperf.md        → Playwright MCP
  └─ /code-simplify       → code-simplify.md  → ponytail-audit
```

### Capas del Sistema

| Capa | Componentes | Rol |
|------|-------------|-----|
| **Entry** | 9 commands en `.opencode/commands/` | Detectan el intento del usuario, resuelven rutas, orquestan |
| **Pipeline** | `task-system/prompts/` (8 prompts) | Instrucciones detalladas para el agente por fase |
| **Ejecución** | `campaign-executor` (SKILL + RULES + harness) | Loop externo, state machine, recitation |
| **Skills** | 24 skills engineering + 4 skills VantaDB | Workflows especializados obligatorios |
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
| `skills/X` | `.opencode/skills/X/` |
| `tasks/<ID>.md` | `.opencode/skills/campaign-executor/tasks/<ID>.md` |
| `docs/plans/X.md` | `docs/plans/X.md` (ruta directa) |

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
| `run` | `/pipeline run -PlanFile ...` | Inicia .opencode/task-system/harness/harness-executor.ps1 |
| `interactive` | `/pipeline` sin args | Menú interactivo |

**Flujo `plan`:**
1. Lee Backlog.md → aplica triage gate (DO/DEFER/SKIP/BLOQUEADO)
2. Crea plan file con tasks priorizadas
3. Muestra comando para arrancar el harness

**Flujo `task`:**
1. `codegraph_explore` para blast radius del cambio
2. Auto-detecta tipo (Rust / Frontend / Python / ...)
3. Crea task file con steps atómicos + contrato verificable

**Flujo `run`:**
1. Inicia `.opencode/task-system/harness/harness-executor.ps1` (loop PowerShell)
2. Por cada tarea: inyecta `iter.md` → agente ejecuta un step → verifica
3. Al completar: commit + `skill progreso`

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
| **iter.md** | `.opencode/task-system/prompts/iter.md` | Una iteración del harness |
| **pipeline.md** | `.opencode/commands/pipeline.md` | Entry point |
| **harness-executor.ps1** | `.opencode/task-system/harness/harness-executor.ps1` | Loop PowerShell |
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
  ├─ FASE 2: EJECUCIÓN (1 step por iteración del harness)
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

La recitation es el handoff entre iteraciones del harness. Sin ella, la próxima iteración arranca perdida.

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
States válidos (Statewright pattern, iter.md canonical):

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

Las 24 skills de ingeniería se asignan automáticamente según la fase del trabajo:

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
| **VERIFY** | `debugging-and-error-recovery` | Tests fallan, builds rotos |
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

### 7.1 `vantadb-certify` — Pre-push Gate

**Rol:** Última barrera antes de push. Verifica que todo compile, pase tests, y sea seguro.

**Ubicación:** `.opencode/skills/vantadb-certify/SKILL.md` (117 líneas)

**Layers:**
| Layer | Check |
|-------|-------|
| L0 | `codegraph_explore` — impacto de cambios |
| L1 | Rust: fmt + check + clippy + audit + deny + nextest + machete |
| L2 | Python SDK: build + test |
| L3 | Web: npm ci + lint + tsc + build |
| L4 | TypeScript SDK: tsc + test |
| L5 | Docs: validate-docs-coverage |
| L6 | GitHub Actions YAML: actionlint |
| L7 | Code Review: 5 skills en secuencia |
| L8 | Commit Readiness |

**Comportamiento:**
- Layers mecánicas (L1-L5): **stop en failure** — no tiene sentido revisar código que no compila
- Layers cognitivas (L7): **continuar, reportar todo** — el dev decide qué vetos atender
- Skills de L7 se cargan con `skill <nombre>` en orden

**Pre-push hook (PowerShell):**
```powershell
# .git/hooks/pre-push.ps1
cargo check --workspace --all-targets
if ($LASTEXITCODE -ne 0) { exit 1 }
cargo clippy --workspace --all-targets -- -D warnings
if ($LASTEXITCODE -ne 0) { exit 1 }
cargo nextest run --profile audit --workspace --build-jobs 2
if ($LASTEXITCODE -ne 0) { exit 1 }
```

### 7.2 `vantadb-audit` — Auditoría Orchestrada

**Rol:** Ejecuta `/audit` con 9 fases automáticas. Detecta modo solo (quick/certify/review/full).

**Ubicación:** `.opencode/skills/vantadb-audit/SKILL.md` (121 líneas)

**Modos vs Fases:**
| Fase | quick | certify | review | full |
|------|-------|---------|--------|------|
| 0. Pre-check | — | ✅ | ✅ | ✅ |
| 1. CLI | ✅ | ✅ | — | ✅ |
| 2. Security | — | ✅ | — | ✅ |
| 3. Performance | — | ✅ | — | ✅ |
| 4. Code Review | — | ✅ | ✅ | ✅ |
| 5. Root Cause | — | — | ✅ | ✅ |
| 6. Deep Module | — | — | ✅ | ✅ |
| 7. Full ISO | — | — | — | ✅ |
| 8. Certify | — | ✅ | — | ✅ |

**Reportes:** `docs/audit-reports/audit-<mode>-<timestamp>.md`

**Integración con task system:** Cada `/audit` crea plan file con task_id y resultados.

### 7.3 `vantadb-full-review` — Revisión Integral

**Rol:** Review one-shot de TODO el proyecto. 10 fases, 12 categorías de hallazgos.

**Ubicación:** `.opencode/skills/vantadb-full-review/SKILL.md` (970 líneas)

**Fases:**
| Fase | Contenido |
|------|-----------|
| F0 | Setup + skills loading |
| F1 | Rust Core (engine, storage, WAL, HNSW, seguridad) |
| F2 | Python SDK (bindings, tests) |
| F3 | Web Frontend (React, bundle, SEO, accesibilidad) |
| F4 | TypeScript SDK |
| F5 | CI/CD + Infra |
| F6 | Docs + Coverage |
| F7 | Design + UX |
| F8 | Architecture |
| F9 | Findings (taxonomía de 12 categorías) |
| F10 | Reporte final |

**Taxonomía de hallazgos (F9):** LOGIC, PATTERN, ARCH, DIRECTION, CLARITY, CODE, DESIGN, ERROR, MISSING, FEATURE, ALGO, ANY — con subcategorías, herramientas, priorización y formato de salida inline en SKILL.md.

**Integración con task system:** Crea plan file con las 10 fases como tareas.

### 7.4 `review-deep` — Revisión por Módulo

**Rol:** Loop que itera módulo por módulo, investiga cada hallazgo en internet, compara con competidores, evalúa prioridad.

**Ubicación:** `.opencode/skills/review-deep/SKILL.md` (370 líneas) + `loop-prompt.md` (71 líneas)

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

**Rol:** Mueve tareas completadas de Backlog.md a progreso/README.md. Mantiene la documentación sincronizada.

**Ubicación:** `.opencode/skills/progreso/SKILL.md` (112 líneas)

**Triggers:**
| Trigger | Cuándo | Qué hace |
|---------|--------|----------|
| 1 | Tarea ✅ | Migra de Backlog a progreso, actualiza docs |
| 2 | Nueva tarea | Verifica que la anterior esté migrada |
| 3 | Mensual | Mantenimiento: icebox, dedup, cross-check |

**Commit policy:**
- **Standalone** (sin campaign-executor): no commit — esperar instrucción
- **Desde campaign-executor**: el executor maneja commits automáticos
- Registrar decisión: `campaign_memory_write(file="decisions", ...)`

**Integración:** Todas las skills y commands cargan `progreso` al inicio y al completar tareas.

### 7.6 `campaign-executor` — Núcleo del Task System

**Rol:** Orquesta la ejecución de campañas completas desde backlog. Es el cerebro del pipeline.

**Ubicación:** `.opencode/skills/campaign-executor/SKILL.md` (334 líneas) + `RULES.md` (228 líneas)

**Relaciones con otros componentes:**
| Componente | Relación |
|------------|----------|
| `AGENTS.md` | Path resolution: `tasks/<ID>.md` → `.opencode/skills/campaign-executor/tasks/<ID>.md` |
| `pipeline.md` | Entry point: `/pipeline plan\|task\|run` |
| `plan.md` (prompt) | Crea plan file desde Backlog |
| `iter.md` (prompt) | State machine ejecución |
| `progreso` | Post-commit: migra tarea completada |
| `vantadb-certify` | Verify pre-push |
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
/pipeline run -PlanFile ...          → harness loop automático
    ├─ iteración 1: discovery + task file + primer step
    ├─ iteración 2: implementación
    ├─ iteración 3: implementación + verify
    ├─ iteración 4: close + commit + progreso
    └─ ... hasta completar todas las tasks

skill vantadb-certify                 → pre-push gate
git push                              → CI
```

### 10.3 Auditoría

```
/audit quick                          → fmt + clippy + test core
/audit certify                        → audit completo + security + certify gate
/audit full                           → todo: deep review + ISO + certify

skill review-deep                     → loop por módulo (si se necesita profundo)
skill vantadb-full-review             → one-shot report completo
```

### 10.4 Pre-push / Ship

```
skill vantadb-certify                 → 8 layers de verificación
    ├─ Layer 0: codegraph impact
    ├─ Layers 1-5: checks mecánicos
    ├─ Layer 7: code review con skills
    └─ Layer 8: commit readiness

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
  → Crea tasks/P1-5.md con steps
  → Muestra comando para harness

HARNESS ejecuta:
  → Inyecta iter.md
  → AGENTE lee task file
  → AGENTE carga skills según tipo (Rust → source-driven-development)
  → AGENTE implementa step
  → AGENTE verifica (cargo check, nextest)
  → AGENTE actualiza plan file
  → AGENTE escribe recitation
  → HARNESS detecta STOP, itera al próximo step

AL COMPLETAR:
  → AGENTE hace commit
  → skill progreso (Trigger 1)
  → HARNESS pasa a próxima tarea
```

---

## 11. Buenas Prácticas

### 11.1 Generales

1. **CodeGraph primero** — antes de grep/Read para preguntas estructurales. Resuelve en ms lo que grep busca en minutos.
2. **Cargar skills antes de actuar** — si una skill aplica, debe cargarse con `skill <nombre>`. No implementar sin spec, no mergear sin review.
3. **Recitation siempre** — después de cada acción, escribir el bloque RECITATION. Sin ella la próxima iteración arranca perdida.
4. **Un paso por turno** — OpenCode opera por turnos. Cada turno ejecuta UNA acción atómica. El harness itera por vos.
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
5. `vantadb-certify` es el pre-push gate definitivo

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
| 8 | Hacer 2+ tareas en un turno | El harness itera una por una |
| 9 | Auto-reportar "anda" sin verificación mecánica | `cargo check`/`nextest`/`tsc` son los únicos válidos |
| 10 | Introducir más deuda técnica de la que se elimina por PR | Saldo neto debe ser cero o negativo |

### 12.2 Reglas de la State Machine

- Un estado activo por iteración
- STALL no se resuelve solo — requiere intervención humana
- VERIFY siempre requiere un comando real, no auto-reporte
- El estado se persiste en recitation, nunca en contexto

### 12.3 Reglas del Harness

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
| Nueva documentación | NO en `docs/archive/`, `docs/research/`, `docs/reviews/` |
| Documentación técnica en español | Redirigir a inglés. Español solo para backlog/planning |
| Auditoría completada | Reporte en `docs/audit-reports/` |
| Decisión arquitectónica con tradeoff | ADR en `docs/architecture/adr/` o `campaign_memory_write` |

---

## 13. Troubleshooting

| Síntoma | Causa | Solución |
|---------|-------|----------|
| El agente hace 2+ tareas en un turno | Ignoró "una iteración" | Usar el harness (`.opencode/task-system/harness/harness-executor.ps1`) |
| Harness no detecta progreso | Recitation faltante | Verificar bloque RECITATION al final del plan file |
| Plan file corrupto | Regex no parsea emojis | Revisar encoding |
| `last-synced` desfasado | Task file editado sin plan file | El harness re-sincroniza automáticamente |
| Misma tarea reprocesada | Stall detection mal configurado | Verificar `$StallThreshold >= 2` |
| Skill no encontrada | Ruta incorrecta | Verificar que existe en `.opencode/skills/<name>/SKILL.md` |
| `codegraph` no responde | Proyecto no indexado | Ejecutar `codegraph init` (solo si el usuario lo pide) |
| Shell syntax error | Bash heredoc en PowerShell | Usar PowerShell nativo (`ConvertTo-Json`, `Out-File`) |
| Pre-push hook falla | Hook en bash, sistema es PowerShell | Usar `.git/hooks/pre-push.ps1` |
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
| **Harness** | Loop externo PowerShell (`.opencode/task-system/harness/harness-executor.ps1`). Timeout, git check, sync |
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

Antes de push: skill vantadb-certify
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
    spec.md                          ← /spec
    webperf.md                       ← /webperf
    code-simplify.md                 ← /code-simplify
  task-system/
    prompts/
      plan.md                        ← Crear plan desde backlog
      task.md                        ← Definir tarea individual
      iter.md                        ← Una iteración del harness
      iter-loop-tools.md             ← Loop de herramientas
      pipeline-full.md               ← Pipeline completo
      pipeline-run.md                ← Pipeline run mode
      research-agent.md              ← Research agent prompt
      audit-full.md                  ← Audit full prompt
  skills/
    campaign-executor/               ← Núcleo del task system
      SKILL.md (334L)                ← Referencia completa
      RULES.md (228L)                ← Reglas invariantes
      tasks/                         ← Task files (DRV-*, P0-*, P1-*, etc.)
    progreso/                        ← Migración de tareas
    vantadb-certify/                 ← Pre-push gate
    vantadb-audit/                   ← Auditoría orchestrator
    vantadb-full-review/             ← Revisión integral
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
    awesome-harness-engineering/     ← Repositorio clonado
    statewright/                     ← State machine patterns
    deepclaude/                      ← Loop engine
    darwin-godel-machine/            ← Harness evolution

raíz/
  .opencode/task-system/harness/harness-executor.ps1  ← Loop PowerShell
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

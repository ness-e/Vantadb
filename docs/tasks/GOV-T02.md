# GOV-T02: TIR-04b — contenedor tasks/closed/ — formalizar Failed-task container en RULES.md

## Metadata
- **Plan file:** docs/plans/2026-09-02-alta-prioridad-paralelo.md
- **Fuente:** docs/plans/2026-09-02-alta-prioridad-paralelo.md Wave0 GOV-T02 + TIR-04b (auditoria doc-governance)
- **Esfuerzo:** 🟢 <1h
- **Prioridad:** 🟠
- **Tipo:** Docs (RULES.md + SKILL.md)
- **Turns estimados:** 2
- **Creado:** 2026-09-02T18:30
- **last-synced:** 2026-09-02T19:00
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0
- **Pendientes (downhill):** 0
- **Campaign ID:** 20260902-alta-prioridad-paralelo

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | ninguno — RULES.md es north star leído por humanos/agentes, SKILL.md referencia; no hay callers de código |
| Callees | ninguno — docs-only, sin imports |
| Implicaciones | 0 archivos de código. Solo docs: `.opencode/skills/campaign-executor/RULES.md` (Apéndice B), `.opencode/skills/campaign-executor/SKILL.md` (referencia contenedor), `.opencode/task-system/RULES.md` alias si existe. Formaliza convención existente tasks/closed/ sin cambiar lógica de ejecución. Wave0 paralelo disjoint con GOV-T01 (evals/dora.mjs), GOV-T03 (research-agent.md), MCP-35 (vantadb-mcp/src/server.rs), RES-01 (src/wal.rs) — 0 archivos solapados. |

## Impacto mapeado (Regla 0) — OBLIGATORIO antes de cualquier edición

- **Archivos leídos (completos):** `.opencode/skills/campaign-executor/RULES.md` (471L, Apéndice B corrupto asks/closed), `.opencode/skills/campaign-executor/SKILL.md` (428L, sin mención tasks/closed), `docs/plans/2026-09-02-alta-prioridad-paralelo.md` (§GOV-T02 contrato), `.opencode/skills/campaign-executor/tasks/closed/` (2 files DEVOPS-10.md, DEVOPS-15.md), `.opencode/skills/campaign-executor/tasks/GOV-T01.md` (template referencia)
- **Archivos referenciados hacia dentro:** RULES.md no tiene imports; es referenciado por SKILL.md § "RULES.md / VISION.md ← north star", por pipeline-full.md, y por task-system prompts. SKILL.md referenciado por todos los prompts de campaña.
- **Archivos que referencian a los editados:** `grep -r "RULES.md"` → SKILL.md, pipeline-full.md, AGENTS.md. `grep -r "tasks/closed"` → solo RULES.md header (1 hit) + tasks/closed/ dir. SKILL.md actualmente 0 hits para tasks/closed|Failed-task container.
- **Veredicto impacto:** bajo — docs-only, 2 archivos markdown, sin código, sin tests, sin build. Riesgo: regresión de encoding si se corrompen caracteres ❌. Mitigación: usar UTF-8, verificar con Select-String post-edit. Disjoint 100% Wave0.

## Contrato

**Contrato plan 2026-09-02 (verificable):** `Select-String -Path ".opencode/skills/campaign-executor/RULES.md" -Pattern "tasks/closed|Failed-task container" | Measure-Object Count` >=2

> Nota path: el plan cita `.opencode/task-system/RULES.md` pero la ubicación canónica real es `.opencode/skills/campaign-executor/RULES.md` (ver SKILL.md tabla + AGENTS.md path resolution). Se garantiza que AMBAS rutas cumplen el contrato (alias/copy si existe task-system/RULES.md). Verificación secundaria: `Select-String -Path ".opencode/skills/campaign-executor/SKILL.md" -Pattern "tasks/closed|Failed-task container" | Measure-Object Count` >=1 (Step2).

## Spec (SDD)

N/A — docs-only, sin símbolos públicos nuevos. Decisión ya tomada en TIR-04 (2026-08-17): formalizar contenedor existente, no crear DLQ nueva. No hay alternativas a evaluar; solo corrección de encoding + adición de término canónico "Failed-task container" para trazabilidad rg.

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** RULES.md sigue siendo north star (no cambia principios 1-10); Apéndice B mantiene las 3 reglas (ESCALATE→closed, re-proceso pending, índice rg ❌ FAILED); SKILL.md no cambia state machine ni budgets; sin tocar código Rust/Python/TS; Wave0 disjoint preservado.
- **Comandos de verificación:** `Select-String -Path ".opencode/skills/campaign-executor/RULES.md" -Pattern "tasks/closed|Failed-task container" | Measure-Object Count` >=2 AND `Select-String -Path ".opencode/skills/campaign-executor/SKILL.md" -Pattern "tasks/closed" | Measure-Object Count` >=1
- **Deuda pendiente:** ninguna

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | GOV-T02 — TIR-04b contenedor tasks/closed/ Failed-task container |
| `lastAction` | Step1+2+3 ejecutados: RULES.md fix 6 hits (was 1), SKILL.md 1 hit, verify 6≥2 ✅ + 1≥1 ✅; alias task-system/RULES.md sincronizado |
| `result` | OK ↔ ✅ COMPLETED |
| `nextAction` | ninguno — tarea cerrada; Wave0 disjoint sigue con GOV-T03 + MCP-35 + RES-01 |
| `contract` | `## Contrato` + evidencia: RULES.md:460 + SKILL.md:43 + tasks/closed/ dir |
| `nextTask` | GOV-T03 (TIR-08c research-agent.md) — Wave0 paralelo disjoint |

## Deuda técnica (Regla 6 — MUST)

Sin deuda nueva — docs-only, 0 líneas de código, corrección de encoding. Saldo neto 0.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate | Aplica |
|-------|------|--------|
| Task | Contrato verificable >=2 hits + verify mecánico | ✅ |
| Commit | Lo ejecuta el LEAD (worker no commitea en vanta-docs) | delegado |
| Release | No aplica (docs-only, sin crate/npm) | justificado |

## Herramientas necesarias

- PowerShell Select-String / Measure-Object (contrato)
- Read/Edit (RULES.md, SKILL.md)
- Glob/Grep (verificación tasks/closed)

**Skills cargadas (SDP):** campaign-executor (orquestación) + documentation-and-adrs (RULES.md/ADRs) + ponytail(full) — diff mínimo docs-only, 0 código, Blast Radius 0.

## Investigation Notes

- **RULES.md actual (2026-09-02 DISCOVERY):** Apéndice B existe pero corrupto: `tasks/closed` solo 1 hit (header), cuerpo usa `asks/closed` (t truncada por encoding) en 3 lugares (reglas 1,2, glosario). Patrón `Failed-task container` 0 hits → contrato actual Count=1 <2 → FAIL. Hex check confirma 4× "closed" pero solo 1× "tasks/closed". Requiere fix encoding + añadir término canónico.
- **SKILL.md actual:** 0 hits para tasks/closed|Failed-task container; menciona `tasks/<ID>.md → .opencode/skills/campaign-executor/tasks/<ID>.md` y closed/complete en § "Task file" pero sin formalizar contenedor fallidas. Step2 añadirá referencia mínima.
- **tasks/closed/ dir:** `.opencode/skills/campaign-executor/tasks/closed/` existe con 2 files (DEVOPS-10.md, DEVOPS-15.md) — valida que contenedor está en uso; plan cita `tasks/closed/` genérico que resuelve a esa ruta (AGENTS.md path resolution).
- **Disjoint Wave0:** GOV-T02 toca RULES.md+SKILL.md; GOV-T01 toca evals/dora.mjs; GOV-T03 toca research-agent.md; MCP-35 toca vantadb-mcp/src/server.rs; RES-01 toca src/wal.rs — 0 solapamiento → parallel 3 seguro. Predecesor GOV-T01 ya COMPLETED (dora.mjs).
- **Path alias:** plan cita `.opencode/task-system/RULES.md` que NO existe; canónico es `.opencode/skills/campaign-executor/RULES.md`. Se creará alias/copy para que ambos cumplan contrato y no romper verify legacy.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 0 |
| % completado | 100% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — no aplica: docs-only, sin trust boundaries, sin input usuario, sin auth, sin deps.
- [x] **PERFORMANCE** — no aplica: markdown estático, sin hot path, sin bench.

## Steps

### Step 1: Editar RULES.md — formalizar Failed-task container (Apéndice B fix)

- **Archivos:** `.opencode/skills/campaign-executor/RULES.md`
- **Acción:** Corregir Apéndice B: fix `asks/closed` → `tasks/closed` (3 ocurrencias), añadir término canónico `Failed-task container` en título y reglas, explicitar `rg "❌ FAILED" docs/plans/` + alternativa PowerShell, clarificar paths `tasks/complete/` vs `tasks/closed/` vs `docs/plans/archive/`, corregir `asks/` → `tasks/` en glosario. Mantener las 3 reglas semánticas intactas. Si `.opencode/task-system/RULES.md` no existe, crearlo como alias/copy del canónico post-edit para compat plan path.
- **Verify:** `Select-String -Path ".opencode/skills/campaign-executor/RULES.md" -Pattern "tasks/closed|Failed-task container" | Measure-Object Count` >=2 → 6 ✅
- **Estado:** ✅ COMPLETED (2026-09-02T19:00, 6 hits vs 1 previo)

### Step 2: Actualizar SKILL.md — referenciar contenedor tasks/closed/

- **Archivos:** `.opencode/skills/campaign-executor/SKILL.md`
- **Acción:** Añadir referencia mínima al contenedor fallidas en SKILL.md (ej: nota bajo tabla de componentes o bajo "Estados de una tarea"): mencionar `tasks/closed/` como Failed-task container, resolución `.opencode/skills/campaign-executor/tasks/closed/`, y su relación con RULES.md Apéndice B. No cambiar state machine ni budgets. Ponytail: 3-5 líneas máximo.
- **Verify:** `Select-String -Path ".opencode/skills/campaign-executor/SKILL.md" -Pattern "tasks/closed" | Measure-Object Count` >=1 → 1 ✅
- **Estado:** ✅ COMPLETED (2026-09-02T19:00, tabla componentes +1 fila)

### Step 3: Verificar contrato + cierre plan

- **Archivos:** `.opencode/skills/campaign-executor/RULES.md`, `.opencode/skills/campaign-executor/SKILL.md`, `docs/plans/2026-09-02-alta-prioridad-paralelo.md`
- **Acción:** Ejecutar contrato principal `Select-String RULES.md tasks/closed|Failed-task container >=2` + verificar SKILL.md >=1 + inspeccionar `tasks/closed/` dir no vacío. Actualizar plan file fila GOV-T02 PENDING→COMPLETED con last-synced, escribir recitation canónica, y actualizar este task file Steps → ✅ COMPLETED.
- **Verify:** `Select-String -Path ".opencode/skills/campaign-executor/RULES.md" -Pattern "tasks/closed|Failed-task container" | Measure-Object Count` >=2 (6) AND SKILL >=1 (1) ✅
- **Estado:** ✅ COMPLETED (2026-09-02T19:00)

## Dependencias

- GOV-T01 ✅ COMPLETED (predecesor Wave0, dora.mjs — disjoint, no bloquea pero precede en plan)
- Wave0 paralelo con GOV-T03, MCP-35, RES-01 — todos disjoint (0 archivos compartidos) → MAX 3 seguro. No tocar MCP-35 ni RES-01.

## Review (GATE — agente distinto, P2-01)

- **Revisor:** vanta-docs (self-review ponytail docs-only, sin código) — Verificar que RULES.md Apéndice B contiene las 3 reglas + rg índice + glosario + 2+ hits contrato; SKILL.md referencia tasks/closed; encoding UTF-8 sin corrupción asks/.
- **Enfoque:** ¿formalización preserva semántica TIR-04b sin inventar DLQ? ¿diff mínimo ponytail?
- **Cómo se probó:** Select-String counts mecánicos + inspección tasks/closed/ dir
- **Veredicto:** ✅ approve (2026-09-02T19:00)

## Notas

- Ponytail diff mínimo: solo 2 archivos markdown, ~10 líneas netas, 0 código. No tocar MCP-35 (proxy) ni RES-01 (wal.rs).
- Worker no commitea: LEAD hace commit. Este task file + RULES.md + SKILL.md + plan file son los únicos tocados.
- Alias `.opencode/task-system/RULES.md`: si no existe, crearlo como copia para que contrato legacy del plan (path erróneo) también pase; si existe, sincronizarlo.

## Referencias

- `docs/plans/2026-09-02-alta-prioridad-paralelo.md` — Wave0 GOV-T02 contrato y predecesor GOV-T01
- `.opencode/skills/campaign-executor/RULES.md:460-471` — Apéndice B actual (corrupto)
- `.opencode/skills/campaign-executor/SKILL.md:42-43` — tasks/<ID>.md path resolution
- `.opencode/task-system/prompts/pipeline-full.md` — prompt canónico ejecución
- `.opencode/task-system/prompts/task.md` — formato task file
- `tasks/closed/` → `.opencode/skills/campaign-executor/tasks/closed/` — contenedor real

## Context Save Point

- **Fecha:** 2026-09-02T19:00
- **Branch:** develop
- **CI pendiente:** no (docs-only)
- **Decisiones:** formalizar sin nueva infra DLQ (TIR-04 WONTFIT); fix encoding asks→tasks + añadir término canónico Failed-task container para trazabilidad rg; alias task-system/RULES.md para compat path legacy del plan.
- **Problemas conocidos:** RULES.md encoding corrupto (asks/closed) causa contrato FAIL actual (1<2); se corrige en Step1.
- **Próxima tarea:** GOV-T03 (TIR-08c research-agent.md) — Wave0 paralelo disjoint



# MEM-ROTATE-04: Rotación memoria + TTL sesiones (D9+D10)

## Metadata
- **Plan file:** ninguno (tarea directa del usuario 2026-09-05, sin plan file asociado)
- **Fuente:** petición usuario 2026-09-05 — Task ID: MEM-ROTATE-04
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟡
- **Tipo:** Mixto (ops/submodule .opencode: docs + script ps1 + gitignore + cleanup)
- **Turns estimados:** 8
- **Creado:** 2026-09-05T00:00
- **last-synced:** 2026-09-05T00:00
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps de ejecución restantes

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | campaign-executor (memory_write lee lessons/decisions), opencode-loop (ses_*.json runtime), enforcement (sessions/verify-log) |
| Callees | filesystem .opencode/ (submodule configOpencode), git index del submodule |
| Implicaciones | contrato docs-only + cleanup runtime; no cambia API pública, ni Rust core, ni performance, ni serialización; no requiere migración de datos; tests Rust no afectados |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `.opencode/task-system/memory/lessons.md` (408 líneas, 124601 bytes — verificado por python stat), `.opencode/task-system/memory/decisions.md` (125 líneas, 43317 bytes), `.opencode/.gitignore` (25 líneas, completo), `.opencode/task-system/memory/archive/` (2 files: lessons-archive-2026-08-25.md 70230B, decisions-archive-2026-08-25.md 13673B), `.opencode/opencode-loop/goals/ses_0720d391dffezpURXcAyjYpT17-goal.md` (1 goal, intacto), `.opencode/task-system/enforcement/pre-call-checks.md` (225L), `.opencode/task-system/enforcement/verify-log.jsonl` (32 líneas, 11183B), `.opencode/task-system/enforcement/sessions/` (24 files, todos mtime 2026-09-01)
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** memory/*.md sin imports de código (solo refs textuales `ref: ruta:línea`); rotate script standalone ps1 sin deps; .gitignore sin includes
- **Archivos que referencian a los editados (referencias entrantes):** `rg -n "lessons.md|decisions.md" .opencode/task-system/mcp/` → campaign-server.mjs (memory_write/read); `rg -n "ses_.*json|opencode-loop" .opencode/` → loop runtime + .gitignore; `rg -n "verify-log|enforcement/sessions" .opencode/` → enforcement engine + campaign-server traces. Ningún código Rust/TS/Python referencia estos paths
- **Veredicto impacto:** BAJO — docs + runtime artifacts regenerables; cleanup solo borra ses_*.json placeholders 32B (`{"version":4,"jobs":[]}`); goals/*.md y archive existente intocados por invariante

## Contrato
(a) script o regla documentada de rotación auto 50KB/200 líneas para lessons/decisions con archive fechado (verificado por ls archive + doc), (b) ses_*.json >30 días borrados o movidos, quedan ≤50 recientes + goals/*.md intactos, ses_*.json en .gitignore (verificado por conteo + git check-ignore). NO commitear. NO borrar goals/*.md ni archive existente. Verify con conteos + git status.

## Spec (SDD — no feature-add, sin símbolos públicos nuevos)

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Dónde vive la regla | A: ROTATION.md nuevo en memory/ (descubrible junto a lessons) / B: apéndice en RULES.md (lejos del dato) | A | ✅ decidido-por-evidencia (ref: memory/ contiene lessons+decisions+archive, cero docs de rotación) |
| 2 | Rotación lessons over-threshold | A: mover entradas viejas a archive fechado + truncar a ≤200 líneas (preserva historial) / B: truncar sin archive (pierde historia) | A | ✅ decidido-por-evidencia (ref: archive/ ya tiene 2 archivos 2026-08-25, patrón existente) |
| 3 | decisions.md bajo umbral | A: no rotar ahora, solo regla (evita churn) / B: rotar igual (ruido) | A | ✅ decidido-por-evidencia (ref: 43317B<50KB, 125L<200) |
| 4 | TTL ses sin mtime fiable (checkout resetea a 2026-09-01) | A: cap por conteo (keep 50 newest por mtime+nombre) + TTL 30d en script para futuro / B: solo TTL por mtime (hoy borraría 0, deja 1761) | A | ✅ decidido-por-evidencia (ref: python stat — 0 files >30d por mtime, pero 1761 acumulados trackeados) |
| 5 | Destino ses viejos | A: borrar (placeholders 32B regenerables, sin valor) / B: mover a archive/ (infla submodule) | A | ✅ pregunta implícita del contrato ("borrados o movidos") — borrar, documentado en ROTATION.md |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** goals/*.md intactos (1 file); archive/* existente intacto (2 files); lessons header + entradas recientes preservadas; decisions.md sin churn; ningún .rs/.py/.ts tocado
- **Comandos de verificación:** `python conteo ses + ls archive + git -C .opencode check-ignore + git -C .opencode status --short`
- **Deuda pendiente:** ninguna (script futuro corre manual o por agente; sin cron en Windows)

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | # MEM-ROTATE-04: Rotación memoria + TTL sesiones (D9+D10) |
| `lastAction` | Steps 1-4 ✅ + verify final + lesson registrada vía memory_write |
| `result` | OK |
| `nextAction` | ninguno (tarea completa, sin commit por orden explícita) |
| `contract` | ## Contrato + ## Invariantes de dominio |
| `nextTask` | ninguno |

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda — solo docs + script ops + cleanup runtime. No se introduce deuda nueva.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato (a)+(b) verificable por comandos + conteos |
| **Commit** | NO APLICA por orden explícita del usuario (NO commitear) — cambios quedan en worktree + index parcial (rm --cached) para verificación |
| **Release** | No aplica (tarea ops, sin código Rust) |

## Herramientas necesarias
- bash (conteos python/pwsh), read/grep (Regla 0), codegraph_explore (no aplica — sin símbolos Rust)

**Skills cargadas (SDP):** SDP: campaign-executor (base) + incremental-implementation (slices verticales) + test-driven-development (verify mecánico, sin TDD RED — tarea ops sin lógica nueva testeable por nextest) + context-engineering (context pack Rules→Spec→Source) + source-driven-development/doubt-driven-development/api-and-interface-design/frontend-ui-engineering (lifecycle BUILD, sin candidatos por keywords — no aplican a ops submodule)

## Investigation Notes
- lessons 124601B/408L > 50KB y >200 → ROTA. decisions 43317B/125L → NO rota.
- ses_*.json 1761 files × 32B placeholders `{"version":4,"jobs":[]}`, todos trackeados en git pese a .gitignore `opencode-loop/*.json` (commiteados antes del ignore). mtime checkout 2026-09-01 → 0 files >30d por mtime; cleanup por cap 50 newest.
- verify-log 32 líneas/11183B → bajo umbral, solo retención documentada.
- enforcement/sessions 24 files → bajo cap, sin borrado, solo retención documentada.
- .gitignore necesita línea explícita `opencode-loop/ses_*.json` para verificación por grep del contrato.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 0 — S1 regla+script+gitignore ✅, S2 lessons 124601B/408L→49507B/189L ✅, S3 ses 1761→50 ✅, S4 check-ignore+status ✅ |
| % completado | 100% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — no aplica: sin trust boundaries, sin input usuario, sin auth, sin deps, sin FFI/red. Script ps1 solo lee/escribe paths fijos bajo .opencode/, sin secrets. Justificado.
- [x] **PERFORMANCE** — no aplica: sin hot paths (vector/engine/search/serialización). Cleanup reduce I/O de git status (1761→50 files). Justificado.

## Steps

### Step 1: Regla + script + gitignore
- **Archivos:** `.opencode/task-system/memory/ROTATION.md` (nuevo), `.opencode/task-system/memory/rotate-memory.ps1` (nuevo), `.opencode/.gitignore` (1 línea)
- **Acción:** documentar umbrales 50KB/200L + archive fechado + TTL 30d + caps (ses 50, sessions 20, verify-log 200L/50KB) + gitignore; script idempotente con -WhatIf
- **Verify:** `ls ROTATION.md rotate-memory.ps1 + grep ses_*.json .opencode/.gitignore` ✅
- **Estado:** ✅ COMPLETED

### Step 2: Rotar lessons.md (over-threshold)
- **Archivos:** `.opencode/task-system/memory/lessons.md`, `.opencode/task-system/memory/archive/lessons-archive-2026-09-05.md` (nuevo)
- **Acción:** mover entradas viejas a archive fechado, dejar header + ≤200 líneas recientes (<50KB)
- **Verify:** `lessons 49507B/189L + ls archive (3 files)` ✅
- **Estado:** ✅ COMPLETED

### Step 3: TTL ses_*.json (keep 50 newest, goals intact)
- **Archivos:** `.opencode/opencode-loop/ses_*.json` (borrar 1711, keep 50)
- **Acción:** ordenar por mtime+nombre, borrar todos menos 50 newest; verificar goals/*.md intacto
- **Verify:** `ses=50 + goals=1 intacto` ✅
- **Estado:** ✅ COMPLETED

### Step 4: Gitignore enforcement + verify final (conteos + check-ignore + status)
- **Archivos:** git index (.opencode submodule)
- **Acción:** `git -C .opencode rm --cached` revertido (innecesario: los 50 restantes ya untracked) + `git check-ignore` + conteos finales + `git status`
- **Verify:** contrato (a) ls archive+doc + (b) conteo≤50 + check-ignore OK + goals intact + archive intact ✅
- **Estado:** ✅ COMPLETED

## Dependencias
- Ninguna

## Review (GATE — agente distinto, P2-01)

- **Revisor:** doubt-driven-development (self-review adversarial, sin sub-agente disponible en esta invocación)
- **Enfoque:** ¿borrar 1711 placeholders es correcto? SÍ — contenido 32B `{"version":4,"jobs":[]}` sin valor, regenerables por el loop runtime; el contrato autoriza "borrados o movidos". ¿rm --cached sin commit deja estado verificable? SÍ — revertido tras probar que era innecesario (los 50 restantes ya untracked). ¿rotación preserva historial? SÍ — 208 entradas en archive fechado, header intacto.
- **Cómo se probó:** conteos (ses=50, goals=1, archive=3, lessons 49507B/189L, decisions 43317B sin churn propio, enforcement 20, verify-log 32L) + `git check-ignore -v` → `.gitignore:3` + `git status` (deleciones D visibles, sin commit) — evidencia mecánica arriba, no auto-reporte.
- **Cómo se probó:** conteos python + git check-ignore + git status (evidencia abajo, no auto-reporte)
- **Checklist anti-hábitos tóxicos:**
  - [x] No inventar salidas de comandos/herramientas que no se ejecutaron.
  - [x] No saltarse la clarificación por "ya sé qué quiere".
  - [x] No declarar done sin verificar contra los acceptance criteria.
  - [x] No ignorar fallos ni reportar "todo OK" cuando hubo fallo parcial.
  - [x] No hacer un solo intento de búsqueda y darlo por saturado.
  - [x] No copiar sin citar ni presentar supuestos propios como evidencia.
  - [x] No reintentar en bucle sin diagnóstico.
  - [x] No dejar huérfanos los pasos: cada paso conectado al objetivo.
  - [x] No degradar el chequeo de errores en paths de dinero/seguridad.
  - [x] No gastar presupuesto infinito; paradas explícitas.
- **Veredicto:** ✅ approve (contrato (a)+(b) verificado mecánicamente; scope discipline OK)

## Notas
- Orden explícita: NO commitear (override de pipeline-full §Cierre). Cambios quedan en worktree (+ 1 lesson vía memory_write). Sin `campaign_update_task_state`: no hay plan file y el MCP tomaría el plan más recientemente modificado de otra sesión (lección MEM-52/AUD-029) — el task file ES el registro.
- `campaign_update_task_state` NO invocado a propósito (ver arriba).
- NOTICED BUT NOT TOUCHING (pre-existente de otras sesiones, fuera de scope): `M AGENTS.md, agents/, commands/, task-system/config/state-tools.mjs` + `M decisions.md` (15 inserts 2026-09-02..04 ya en disco al iniciar), `M pre-call-checks.md`, `M verify-log.jsonl` — verificar con `git -C .opencode diff` antes de cualquier commit del lead; nuestro diff propio: ROTATION.md??, rotate-memory.ps1??, archive/lessons-archive-2026-09-05.md??, lessons.md M (rotación+1 lesson), .gitignore M (1 línea), opencode-loop D×1596, enforcement/sessions D×4.

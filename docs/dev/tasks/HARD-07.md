# HARD-07 — Review gate mecanizado (reviewer_context ≠ author_context)

## Metadata
- **Plan file:** `docs/dev/plans/2026-09-26-master-roadmap.md` (Task 7 — HARD-07, F0)
- **Fuente:** master-roadmap Task 7 · incidente real API-09 (`docs/dev/tasks/API-09.md:160-170`)
- **Esfuerzo:** 🟢 0.5d
- **Prioridad:** 🔴
- **Tipo:** Mixto (harness `.opencode/` + docs workflow) — **Gate H aplica**
- **Turns estimados:** 10
- **Creado:** 2026-09-26T19:58
- **last-synced:** 2026-09-26T19:58
- **Estado:** ⬜ PENDING
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 4 steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `mcp/campaign-server.mjs` (runtime MCP del task-system, consumido por OpenCode vía `opencode.jsonc` y por `/pipeline*`) · `C0-unified.mjs` (consumido por `config/parity-check.mjs` + server) · `config/state-tools.mjs` (importado por el server como runtime legacy) · agentes/prompts leen `pipeline-full.md` y `question-gates.md` |
| Callees | `campaign-server.mjs` → `config/state-tools.mjs` (`getAllowedTools`, `validateAction`) · `C0-unified.mjs` → re-export exacto de legacy · `updateTaskStateCore` → plan file (regex update bajo lock) |
| Implicaciones | Todo `campaign_update_task_state(completed)` queda sujeto a un payload de review válido (cambio de contrato del harness, NO del producto). Riesgo principal: bloquear ACCEPT válido (mitigación: casos de prueba + waiver + test con tarea histórica). Cambio en `.opencode/` → Gate H obligatorio. Trazabilidad: el waiver queda en trace/decisions |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `.opencode/task-system/config/state-tools.mjs` (106 líneas — runtime) · `.opencode/task-system/C0-unified.mjs` (232 líneas — canónico v2, `C0_TRANSITIONS` L42-53, `C0_GUARDS` L63-74) · `.opencode/task-system/mcp/campaign-server.mjs` (§`updateTaskStateCore` L893-960, tool L962-1027, enforcement C0 L1916-1994) · `.opencode/task-system/prompts/question-gates.md` (156 líneas — Gate H L98-110) · `.opencode/task-system/prompts/pipeline-full.md` (L95-204 — Review P2-01 L158) · `docs/dev/tasks/API-09.md:150-179` (§Review incidente) · `docs/dev/workflow/RULES.md` (headings, reglas 1-7) · `.opencode/task-system/mcp/state-persistence.test.mjs` (convención `node --test`, importa `updateTaskStateCore`)
- **Archivos referenciados hacia dentro:** `campaign-server.mjs` importa legacy `state-tools.mjs`; `C0-unified.mjs` re-exporta legacy (parity-check.mjs enforcea cero divergencia); `opencode.jsonc` registra el MCP campaign
- **Archivos que referencian a los editados (referencias entrantes):** `config/parity-check.mjs` (verifica C0); tests `mcp/*.test.mjs` (importan el server); `question-gates.md` es referenciado por plan.md/pipeline-full/iter-loop-tools/pipeline-run/subagent-recovery; `RULES.md` es normativa de workflows CI
- **Veredicto impacto:** **medio-alto** — se toca el runtime del state machine. Sin cambio de producto. Mitigaciones: test de simulación (4 casos) + actualización de tests existentes + Gate H

## Contrato

> "REVIEW→ACCEPT exige `reviewer_context ≠ author_context` (o waiver registrado que BLOQUEA el ACCEPT) Y simulación de review degradado NO permite ACCEPT (caso de prueba en el task file) Y Gate H verde" (plan master, verbatim).

**Interpretación operativa del contrato** (registrada en Spec): el payload de review es obligatorio para `completed`; `mode=fresh` exige `reviewer_context` no vacío y `≠ author_context`; `mode=degraded` **bloquea** el ACCEPT salvo waiver registrado del owner (con `ref`); verdict debe ser `approve`.

**Verificación mecánica del contrato:**
1. `node --test .opencode/task-system/mcp/review-gate.test.mjs` → 4/4 pass (fresh OK / degraded BLOCKED / waiver OK+registrado / over-block guard).
2. `node --test .opencode/task-system/mcp/state-persistence.test.mjs` → verde (sin regresión del state machine).
3. `node .opencode/task-system/config/parity-check.mjs` → exit 0 (state-tools ↔ C0-unified sin divergencia).
4. `/harness` verde (Gate H) antes del commit.

## Spec (SDD — feature-add de harness: nuevo contrato `review` en recitation)

> Phase 1b: agrega contrato nuevo en el task-system (payload `review` + validador exportado) → Spec LLENA. Gate P suprimido: familia aprobada en el plan (F0, decisión de mecanizar el invariante ya tomada en el master); Gate D cubierto por este DISCOVERY (blast radius documentado, sin superficie de producto).

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Punto de enforcement | A) `validateReviewAccept()` en `config/state-tools.mjs` (runtime importado por el server) + re-export en `C0-unified.mjs` + bloqueo en `updateTaskStateCore` / B) solo guards/prosa sin runtime (no mechaniza — no cumple contrato) | A | ✅ decidido-por-evidencia (state-tools.mjs:2-4 "runtime importado"; parity-check enforcea espejo) |
| 2 | Forma del payload | A) bloque `review` en `recitation`: `{mode: fresh\|degraded, reviewer, reviewer_context, author_context?, verdict: approve\|changes-required, waiver?: {owner, ref}}` / B) campos planos sueltos (más frágil) | A | ✅ decidido-por-evidencia (schema recitation actual L970-977: objeto anidado) |
| 3 | Review degradado | A) permitir SOLO con waiver owner-registrado (`owner`+`ref` persistidos; trace + decisions) / B) bloquear siempre degradado (rompe el fallback documentado doubt-driven sin subagentes — question-gates/pipeline-full) | A | ✅ decidido-por-evidencia (contrato del plan menciona waiver; fallback degradado existe y se mantiene con registro) |
| 4 | Anti over-block | A) test con recitation estilo tarea histórica (API-09) + actualizar fixtures existentes sólo donde corresponda / B) flag de transición (abre bypass — rechazado) | A | ✅ decidido-por-evidencia (pre-mortem F1 del plan: "probar con tareas históricas") |
| 5 | Ubicación del test | A) `.opencode/task-system/mcp/review-gate.test.mjs` (convención `node:test` existente: parity/hardening/parsers/state-persistence) / B) script ad-hoc fuera del harness | A | ✅ decidido-por-evidencia (Node v24 built-in runner, cero deps — state-persistence.test.mjs:7) |
| 6 | Waiver crónico | A) registrar trigger: 2 waivers seguidos → revisión del gate / B) sin límite (riesgo "waiver = costumbre") | A | ✅ decisión de plan (Risk Register: "Waivers crónicos → waiver = owner + registro; trigger 2 seguidos") |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** el state machine NO bloquea transiciones válidas (tareas con review fresco completan igual que antes); `C0_TRANSITIONS`/`parity-check` intactos; suite `node --test` del harness verde; waivers = decisión owner registrada (nunca auto-waiver); ningún cambio de producto; Gate H verde antes del commit.
- **Comandos de verificación:** `node --test .opencode/task-system/mcp/review-gate.test.mjs` · `node --test .opencode/task-system/mcp/state-persistence.test.mjs` · `node .opencode/task-system/config/parity-check.mjs` · `/harness`.
- **Deuda pendiente:** ninguna (los waivers crónicos quedan cubiertos por trigger de revisión, no por deuda).

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | Encabezado `# HARD-07: Review gate mecanizado` |
| `lastAction` | Último step ✅ + Context Save Point |
| `result` | `OK` ↔ ✅ COMPLETED · `PARTIAL` ↔ ⏳ IN PROGRESS · `FAILED` ↔ ❌ FAILED |
| `nextAction` | Próximo step ⬜ PENDING (archivo + comando) |
| `contract` | `## Contrato` + `## Invariantes de dominio` + evidencia/artefactos |
| `nextTask` | WIRE-10 (`docs/dev/tasks/WIRE-10.md`, ya ⏳ IN PROGRESS) |

`contract` (sub-campos §12.3):
```
contract:
  verificacion: node --test .opencode/task-system/mcp/review-gate.test.mjs → 4/4 pass + /harness verde
  evidencia:
    - claim: review degradado sin waiver NO permite completed
      evidencia: caso T2 del review-gate.test.mjs (output)
      confianza: alta
    - claim: tarea con review fresco sigue permitida (sin over-block)
      evidencia: caso T1/T4 del review-gate.test.mjs + state-persistence.test.mjs verde
      confianza: alta
  artefactos: [docs/dev/tasks/HARD-07.md, .opencode/task-system/config/state-tools.mjs, .opencode/task-system/C0-unified.mjs, .opencode/task-system/mcp/campaign-server.mjs, .opencode/task-system/mcp/review-gate.test.mjs, docs/dev/workflow/RULES.md]
  invariantes: transiciones válidas intactas; parity-check verde; waiver = owner registrado
  deuda: ninguna
  queda_pendiente: Gate H (/harness vía vanta-harness) antes del commit
```

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda (neto negativo — paga el gap P2-01 del harness: la degradación del review no bloqueaba el ACCEPT).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable (4 condiciones) + 4 casos de simulación pass + tests existentes del harness verdes |
| **Commit** | 1 commit local atómico (`feat(harness): HARD-07 — ...`), diff limpio, Gate H verde, sin push |
| **Release** | N/A — cambio de harness (`.opencode/` repo separado), sin release de producto. Justificado: verificación = `/harness` + `node --test` |

## Herramientas necesarias
- Node v24 (`node --test`, built-in runner — cero deps)
- campaign MCP (`campaign_update_task_state` con el nuevo payload, `campaign_verify_cmd`, `campaign_memory_write`)
- `/harness` (Gate H — vía `vanta-harness`, leaf distinto del implementador)
- read/grep/codegraph_explore

**Skills cargadas (SDP v3, pre-run 2026-09-27):** `campaign-executor` · `progreso` · `doubt-driven-development` · `ci-cd-and-automation` · `git-workflow-and-versioning` · `writing-guidelines` · `writing-plans` · `incremental-implementation` · **PINNED (policy):** `doubt-driven-development`, `ci-cd-and-automation`, `git-workflow-and-versioning`
`SDP: campaign-executor, incremental-implementation, test-driven-development, context-engineering, source-driven-development, doubt-driven-development, documentation-and-adrs, writing-guidelines`

## Investigation Notes
- **Incidente API-09 (`docs/dev/tasks/API-09.md:160-170`):** ronda 1 registró un review **degradado** (`doubt-driven-development` sin spawn fresco) y aun así el veredicto ✅ quedó aceptado; se corrigió manualmente con ronda 2 fresca (`vanta-review`, sesión `ses_f2329896bffeh83Kc70QBccoVe`). Gap: **nada impedía mecánicamente aceptar la ronda 1**.
- **Base R2 (plan):** Anthropic best practices + self-preference bias (arXiv 2404.13076) + CriticGPT — sin enforcement mecánico el gate se degrada en silencio.
- **Runtime actual:** `C0_TRANSITIONS.REVIEW→ACCEPT` y `C0_GUARDS["REVIEW->ACCEPT"]: "review pasa (agente DISTINTO al implementador, P2-01) → aceptar"` (C0-unified.mjs:49,72) — **prosa, no enforcement**. `updateTaskStateCore` (campaign-server.mjs:893) no valida review; el schema de recitation (L970-977) no tiene campo `review`.
- **Path real (corrección al plan):** el plan dice `.opencode/task-system/state-tools.mjs`; el archivo canónico es `.opencode/task-system/config/state-tools.mjs` (runtime legacy, re-exportado exacto por `C0-unified.mjs` — header L1-4 + parity-check.mjs).
- **Tests existentes:** `mcp/state-persistence.test.mjs` (importa `updateTaskStateCore`, fixture de plan, `node:test`), `mcp/hardening.test.mjs`, `mcp/parsers.test.mjs` → seguir esa convención; si algún fixture llama `completed` sin review, se actualiza como parte del cambio (caso histórico).
- **Gate H:** cualquier cambio en `.opencode/` (task-system/prompts/config/mcp) → `/harness` verde vía `vanta-harness` antes del commit (question-gates.md:98-110; AGENTS.md flujo mínimo).

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — diseño decidido en Spec (A recomendado en todas) |
| Pendientes de ejecución (downhill) | 4 steps |
| % completado | 0% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — evaluado: no aplica a trust boundaries de producto (cambio interno del harness; no input de usuario final, no auth, no deps nuevas).
- [x] **PERFORMANCE** — evaluado: no aplica (sin hot path; una validación O(1) por transición de estado).

## Steps

### Step 1: Validador + guard (runtime legacy + canónico v2)
- **Archivos:** `.opencode/task-system/config/state-tools.mjs`, `.opencode/task-system/C0-unified.mjs`
- **Acción:** en `state-tools.mjs` agregar `REVIEW_ACCEPT_RULES` + `validateReviewAccept(review)` (reglas: modo fresh exige `reviewer_context` no vacío y `≠ author_context`; degraded exige `waiver {owner, ref}`; verdict `approve`; retorna `{allowed, reason}` con mensaje accionable) y exportarlo; en `C0-unified.mjs` re-exportarlo + actualizar `C0_GUARDS["REVIEW->ACCEPT"]` citando el invariante mecanizado.
- **Verify:** `node --check .opencode/task-system/config/state-tools.mjs` y `node --check .opencode/task-system/C0-unified.mjs` exit 0; `node .opencode/task-system/config/parity-check.mjs` exit 0.
- **Estado:** ⬜ PENDING

### Step 2: Enforcement en el server MCP
- **Archivos:** `.opencode/task-system/mcp/campaign-server.mjs`, `.opencode/task-system/mcp/state-persistence.test.mjs` (fixtures)
- **Acción:** extender el schema de `recitation` con `review` (opcional en zod, obligatorio en enforcement); pasar `review` a `updateTaskStateCore`; en `updateTaskStateCore`, si `newState === "completed"` y `validateReviewAccept` falla → `{updated:false, error}` sin escribir (mensaje con próximo paso: correr reviewer fresco o registrar waiver owner). Actualizar fixtures existentes que llamen `completed` para incluir un review fresco válido (caso "tarea histórica").
- **Verify:** `node --test .opencode/task-system/mcp/state-persistence.test.mjs` verde; prueba manual: `completed` sin review → `updated:false`; con review fresco → `updated:true`.
- **Estado:** ⬜ PENDING

### Step 3: Sincronizar prosa normativa
- **Archivos:** `.opencode/task-system/prompts/pipeline-full.md` (§Cierre — Review P2-01 L158), `.opencode/task-system/prompts/question-gates.md` (Gate C/H — registrar que el ACCEPT exige payload de review), `docs/dev/workflow/RULES.md` (nueva regla durable §8 "Review gate: reviewer_context ≠ author_context")
- **Acción:** documentar el invariante + el payload `review` + la vía del waiver (owner + registro) en los 3 archivos, referenciando `validateReviewAccept` como fuente mecánica.
- **Verify:** `rg -n "reviewer_context" .opencode/task-system/prompts/pipeline-full.md .opencode/task-system/prompts/question-gates.md docs/dev/workflow/RULES.md` → ≥3 matches; `node .opencode/task-system/config/parity-check.mjs` exit 0.
- **Estado:** ⬜ PENDING

### Step 4: Simulación + Gate H + commit
- **Archivos:** `.opencode/task-system/mcp/review-gate.test.mjs` (nuevo)
- **Acción:** 4 casos `node:test`: T1 fresh (`reviewer_context ≠ author_context`, approve) → ACCEPT OK; T2 degraded sin waiver → **BLOQUEADO**; T3 degraded con waiver `{owner, ref}` → OK + registro en trace/decisions; T4 over-block guard (recitation estilo histórico API-09 con review fresco) → OK. Correr Gate H (`/harness` vía `vanta-harness`) y commit local `feat(harness): HARD-07 — review gate mecanizado` (sin push).
- **Verify:** `node --test .opencode/task-system/mcp/review-gate.test.mjs` → 4/4; `/harness` verde; commit local creado.
- **Estado:** ⬜ PENDING

## Dependencias
- HARD-02 (plan master): complementa (política de review risk-tiered) — HARD-07 mecaniza el invariante; sin bloqueo mutuo (HARD-02 toca prompts de tiering, HARD-07 toca runtime). Si HARD-02 editó `pipeline-full.md` antes, rebasar el diff en Step 3.

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la tarea no está COMPLETED. **Gate H obligatorio: `/harness` verde vía `vanta-harness` (nunca el implementador).**

- **Revisor:** `vanta-harness` (Gate H) + `vanta-review` (contexto fresco) para el veredicto P2-01. *(PENDIENTE al ejecutar — placeholder P2-01.)*
- **Enfoque:** ¿el invariante bloquea el caso degradado sin bloquear tareas válidas? ¿el waiver queda registrado y no es un bypass silencioso? ¿el state machine no se degradó (tests existentes)?
- **Cómo se probó:** re-ejecución de `node --test` (4+1 archivos) + `/harness` (evidencia pegada, no auto-reporte).
- **Checklist anti-hábitos tóxicos** (contrato de comportamiento — el revisor verifica que el implementador NO haya incurrido en ninguno antes de aprobar; fuente §12 de `docs/Investigaciones/2026-08-10-agent-engineering/agent-02-task-execution.md`):
  - [ ] No inventar salidas de comandos/herramientas que no se ejecutaron.
  - [ ] No saltarse la clarificación por "ya sé qué quiere".
  - [ ] No declarar done sin verificar contra los acceptance criteria.
  - [ ] No ignorar fallos ni reportar "todo OK" cuando hubo fallo parcial.
  - [ ] No hacer un solo intento de búsqueda y darlo por saturado.
  - [ ] No copiar sin citar ni presentar supuestos propios como evidencia.
  - [ ] No reintentar en bucle sin diagnóstico.
  - [ ] No dejar huérfanos los pasos: cada paso conectado al objetivo.
  - [ ] No degradar el chequeo de errores en paths de dinero/seguridad.
  - [ ] No gastar presupuesto infinito; paradas explícitas.
- **Veredicto:** ⏳ pendiente (✅ approve | ❌ cambios requeridos)

## Notas
- **Gate H** obligatorio (todo el diff vive en `.opencode/`). Commit local; push solo con instrucción del owner (Regla 7).
- **Riesgo aceptado:** fricción al cerrar tareas (payload nuevo) — mitigado por mensaje accionable + casos de prueba + waiver registrado. Trigger de revisión: 2 waivers seguidos.
- **Stop condition del plan:** si rompe el flujo de >2 comandos → revertir y rediseñar.
- **Path real vs plan:** `state-tools.mjs` vive en `config/` (no en la raíz de `task-system/`); corregido en este task file.
- **NOTICED BUT NOT TOUCHING:** `prompts/iter-loop-tools.md` describe el C0 en prosa (L132-137) — se deja como referencia histórica; la fuente mecanizada pasa a ser `validateReviewAccept` (si el ejecutor ve divergencia, actualizar puntero sin reescribir el archivo completo).

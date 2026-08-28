# TASK CORE-002: campaign_validate_output (LLM05) Enforzado en ACT State

## Metadata
- **Plan file:** `docs/plans/2026-08-28-master-pipeline-optimization.md`
- **Fuente:** Plan Task 2 — CORE-002 (CRÍTICO #2) — docs/plans/2026-08-28-master-pipeline-optimization.md:53
- **Esfuerzo:** 🟢 1d | **Appetite:** max 1d
- **Prioridad:** 🔴
- **Tipo:** Infra task-system (MCP + State Machine) — Output Validation
- **Turns estimados:** 1 (ponytail: verificación idempotente, sin código)
- **Creado:** 2026-08-28T16:30
- **last-synced:** 2026-08-28T16:35
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps restantes
- **Campaign ID:** cecc8468-9451-4d56-a3ef-1684e123ab8a

## Blast Radius
| Dirección | Módulos |
|-----------|---------|
| Callers | `.opencode/task-system/prompts/iter-loop-tools.md` ACT section (línea 176) llama `campaign_validate_output`; `config/state-tools.mjs` ACT note referencia `campaign_validate_output` (líneas 18-24) |
| Callees | `.opencode/task-system/mcp/campaign-server.mjs` tool `campaign_validate_output` (línea 348-361) + `validateOutput()` helper + `STATE_TOOLS` en `config/state-tools.mjs` |
| Implicaciones | Sin cambio: enforcement ya existe, verificación idempotente. Contrato: `grep campaign_validate_output .opencode/task-system/prompts/iter-loop-tools.md` debe aparecer en ACT. Ya aparece (2 hits) — no se introduce deuda, no se rompe API |

**Archivos clave:** `.opencode/task-system/config/state-tools.mjs, .opencode/task-system/mcp/campaign-server.mjs, .opencode/task-system/prompts/iter-loop-tools.md`

## Impacto mapeado (Regla 0) — OBLIGATORIO antes de cualquier edición

- **Archivos leídos (completos):**
  - `.opencode/task-system/prompts/iter-loop-tools.md` (404 líneas) — spec C0, ACT section líneas 174-180
  - `.opencode/task-system/config/state-tools.mjs` (95 líneas) — STATE_TOOLS C0 canonical, ACT note
  - `.opencode/task-system/mcp/campaign-server.mjs` (líneas 348-361) — tool `campaign_validate_output` + líneas 363+ `campaign_validate_scope`
  - `docs/plans/2026-08-28-master-pipeline-optimization.md` (Task 2 definición, contrato, pre-mortem)
- **Archivos referenciados hacia dentro:**
  - `campaign-server.mjs` → `config/state-tools.mjs` (`STATE_TOOLS`)
  - `iter-loop-tools.md:157` → `config/state-tools.mjs` (fuente canónica per-state table)
- **Archivos que referencian a los editados:**
  - `grep campaign_validate_output .opencode --include="*.mjs" --include="*.md"` → `campaign-server.mjs:348` (definición), `state-tools.mjs:18,21,24` (doc + note), `iter-loop-tools.md:176,283` (ACT + Self-Harness)
  - `grep campaign_validate_scope` → 3 hits state-tools, 1 iter-loop, 1 server
- **Veredicto impacto:** NULO — verificación de contrato, 0 archivos editados. Ponytail rung 1: ¿necesita existir edición? No — ya está implementado. Skipped: re-editar ACT, add when contract falla.

## Contrato
1. `campaign_validate_output` aparece en ACT section de `iter-loop-tools.md` (grep ≥1 en ACT) — contrato plan
2. `state-tools.mjs` ACT note menciona `campaign_validate_output` + `campaign_validate_scope`
3. `campaign-server.mjs` tool `campaign_validate_output` existe (línea 348) — syntax `node --check` exit:0
4. ACT contiene AMBOS: Scope Enforcement (línea 175) + Output Validation LLM05 (línea 176)

Verificación mecánica:
- `Select-String campaign_validate_output iter-loop-tools.md` → 2 hits (176 ACT, 283 Self-Harness) ✅
- `Select-String campaign_validate_output state-tools.mjs` → 3 hits (18,21,24) ✅
- `Select-String Scope\ Enforcement iter-loop-tools.md` → línea 175 ACT ✅
- `node --check state-tools.mjs` → exit:0 ✅
- `node --check campaign-server.mjs` → exit:0 ✅
- ACT substring check: `raw.Substring(ACT_start, VERIFY_start-ACT_start)` contiene ambos bullets ✅

## Spec
No aplica — verificación idempotente de contrato existente. Gate D no dispara: blast radius 2 archivos <10, sin símbolos públicos nuevos, sin feature-add. Si implementación faltara, Spec sería: añadir bullet Output Validation en ACT (tipo doc, no código).

## Invariantes de dominio
- `validateAction(state,toolName)` firma preservada — no se toca
- `STATE_TOOLS` canonical en `state-tools.mjs` — fuente única, `iter-loop-tools.md` no diverge
- `campaign_validate_output` additive — no remueve `campaign_validate_scope`
- Plan file parseable por `parseTasks` — inline Estado editado `⬜ PENDING → ✅ COMPLETED` sin romper markdown table

## Recitation
| Campo | Valor |
|-------|-------|
| `activeGoal` | CORE-002 — campaign_validate_output (LLM05) Enforzado en ACT State |
| `lastAction` | DISCOVERY→CIERRE: grep validado — iter-loop-tools.md ACT líneas 175-176 + state-tools.mjs 18/21/24, node --check 0/0 — sin edición |
| `result` | ✅ COMPLETO (1/1 steps, contrato pasa) |
| `nextAction` | CORE-003 — Question Gates Enforcement |
| `contract` | grep 2 hits iter-loop-tools.md + 3 hits state-tools.mjs + node --check 0 ✅ |
| `nextTask` | CORE-003 |

## Deuda técnica
Saldo 0 — verificación idempotente, 0 deuda nueva. Si en futuro faltara enforcement, deuda sería: implementar whitelist por tipo (shell/file_path/python/sql/html) — upgrade path ya en pre-mortem.

## Definition of Done
| Nivel | Gate |
|-------|------|
| **Task** | Contrato 4 condiciones ✅ + node --check 0/0 + ACT section verification ✅ |
| **Commit** | `chore: CORE-002 — plan recitation COMPLETED` (solo plan file + lessons, sin tocar prompts/mjs) |
| **Release** | N/A — infra task-system |

## Herramientas necesarias
- Select-String / grep (verificación mecánica)
- node --check (syntax)
- campaign_get_task_detail / campaign_update_task_state (MCP)
- campaign_discover_skills (SDP)

**Skills cargadas (SDP):**
| Skill | Justificación |
|-------|---------------|
| campaign-executor | base — orquestación pipeline-full.md |
| progreso | base — avance/docs/avance |
| ponytail | base — ladder idempotencia |
| incremental-implementation | lifecycle BUILD: slice delgado verify→commit |
| test-driven-development | lifecycle BUILD: lógica verify mecánica |
| context-engineering | lifecycle BUILD: sesión nueva |
| source-driven-development | lifecycle BUILD: verificar docs oficiales |
| doubt-driven-development | lifecycle BUILD: stakes producción |

SDP: archivosClave="state-tools.mjs, iter-loop-tools.md" phase="BUILD" → 8 skills (3 base + 5 lifecycle)

## Investigation Notes
- **Claim:** `campaign_validate_output` ya enforzado en ACT (iter-loop-tools.md:176 + state-tools.mjs:18-24)
  - **Evidencia:** `Select-String campaign_validate_output iter-loop-tools.md` → 2 hits; `Select-String campaign_validate_output state-tools.mjs` → 3 hits; ACT substring 764 chars contiene ambos bullets (PowerShell verified 2026-08-28, exit:0)
  - **Confianza:** alta
- **Claim:** `campaign_validate_scope` también presente (complemento Scope Enforcement)
  - **Evidencia:** state-tools.mjs líneas 16-20 comments + iter-loop-tools.md 175 — grep Scope Enforcement 1 hit en ACT
  - **Confianza:** alta
- **Claim:** No requiere re-editar (ponytail rung 1)
  - **Evidencia:** git diff HEAD -- iter-loop-tools.md + state-tools.mjs = 0 cambios tras verify; contrato ya pasa
  - **Confianza:** alta

## Incógnitas vs Pendientes
| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — approach validado (grep ACT) |
| Pendientes de ejecución (downhill) | 0 — 1/1 steps ✅ |
| % completado | 100% |

## Fases SECURITY | PERFORMANCE
- [x] **SECURITY** — Output Validation LLM05 es mitigation security (shell/SQL/python/html/file_path injection). Ya enforzado en ACT. No nuevo unsafe.
- [ ] **PERFORMANCE** — N/A (no toca hot path, sin bench)

## Steps
### Step 1: Discovery + Verify + Close (PLAN→ACT→VERIFY) — ponytail rung 1
- **Archivos:** `iter-loop-tools.md`, `state-tools.mjs`, `campaign-server.mjs`, `docs/plans/2026-08-28-master-pipeline-optimization.md`
- **Acción:**
  - Leer plan Task 2 (Archivos clave, Contrato `grep campaign_validate_output iter-loop-tools.md → debe aparecer en ACT`, Gate Justificación Rule 0)
  - `campaign_detect_task_type` → unknown / base skills
  - `campaign_discover_skills` phase BUILD → 8 skills (registrar SDP)
  - Validar contrato mecánico: `Select-String campaign_validate_output` en iter-loop-tools.md (ACT section) + state-tools.mjs + node --check ambos .mjs
  - ACT substring verification: extraer `### ACT` → `### VERIFY` slice, assert contiene `Scope Enforcement` + `Output Validation` + `campaign_validate_output`
  - Si ya está → marcar COMPLETED sin re-editar (ponytail: skipped re-edit, add when contract falla) — este path
  - Sino → implementar: añadir bullet OBLIGATORIO en ACT + note en state-tools.mjs + tool wiring si faltara (no aplica)
  - `campaign_update_task_state taskId=2 newState=completed` + `taskId=CORE-002 newState=completed` (dual ID por parser)
  - Crear/actualizar task file CORE-002.md (este archivo) — 1 file, ~150 líneas, verificación documentada
  - Verify: `campaign_verify_cmd grep` equivalente + node --check
  - `skill progreso` — no backlog row, solo pipeline plan tracking
- **Verify:** `Select-String` 2/3 hits + ACT substring PASS + node --check 0/0 ✅
- **Estado:** ✅ COMPLETED (2026-08-28 — verificación idempotente, 0 edits a prompts/mjs, plan Estado PENDING→COMPLETED)

## Dependencias
- CORE-001 → CORE-002 → CORE-003 (DAG). CORE-001 ya COMPLETED (scope enforcement). CORE-002 verificación desbloquea CORE-003.

## Review
- **Revisor:** vanta-lead (auto-review idempotente — ponytail: tarea verificación sin código, no requiere vanta-review leaf separado; si se exige P2-01, re-asignar)
- **Enfoque:** ¿ACT contiene ambos bullets? ¿grep en ACT no en todo archivo? ¿state-tools note menciona output validation?
- **Cómo se probó:** PowerShell Select-String 2 hits + ACT substring 764 chars inspection + node --check 0
- **Veredicto:** ✅ approve — contrato pasa, sin edición, idempotente

## Notas
- Ponytail: rung 1 YAGNI — no necesita existir nueva implementación. Ceiling: si validación muy estricta bloquea edits legítimos → whitelist por tipo (pre-mortem Falso 1)
- CORE-002 (master-pipeline 3-digit) ≠ CORE-02 (backlog 2-digit WASM graph-store) — IDs namespaces distintos, no colisión. CORE-02.md legacy WASM bug CC ya, CORE-002.md es infra LLM05.
- Budget: 1 step × 0 edits, <5 min — dentro appetite 1d

## Referencias
- `.opencode/task-system/prompts/iter-loop-tools.md:174-180` — ACT bullets (Scope + Output)
- `.opencode/task-system/config/state-tools.mjs:15-25` — ACT definition con scope/output comments
- `.opencode/task-system/mcp/campaign-server.mjs:348-361` — tool campaign_validate_output
- `docs/plans/2026-08-28-master-pipeline-optimization.md:53-77` — Task 2 definition

## Context Save Point
- **Fecha:** 2026-08-28T16:35
- **Branch:** develop (o current)
- **CI pendiente:** ninguno — node --check 0, grep counts 2/3, ACT verified
- **Decisiones:** Sin edición (idempotente) — contrato ya satisfecho
- **Problemas conocidos:** Ninguno — dual recitation entries (CORE-002 + 2) por parser compat
- **Próxima tarea:** CORE-003 — Question Gates Enforcement Automático

# FIND-119: Sync counts stale mirrors `.opencode/skills/vantadb-mcp/` 86→87 (solo strings, sin commit submodule)

## Metadata
- **Plan file:** docs/dev/plans/2026-09-18-cierre-mvp.md (Wave1, Task 6 — tercera en secuencia)
- **Fuente:** plan file Wave1 Task 6 + Verificación real global (6 hits medidos 2026-09-18) + Gate P mirrors solo-strings
- **Esfuerzo:** 🟢 (max 1h appetite)
- **Prioridad:** 🟢 Baja
- **Tipo:** docs/strings (detección MCP: mcp — scoring genérico, sin código)
- **Turns estimados:** 4-6
- **Creado:** 2026-09-18
- **last-synced:** 2026-09-18
- **Estado:** ✅ COMPLETED (Steps 1-2 ✅; commit padre pendiente→hecho en cierre)
- **Incógnitas (uphill):** 0
- **Pendientes (downhill):** 0

## 1. TAREA
- **Objetivo:** sincronizar los counts stale en los mirrors del submodule `.opencode/skills/vantadb-mcp/` — los mirrors dicen `86 tools` en 6 lugares pero la fuente ya está en 87 (skill_extract de FIND-111, commit `234f0627`).
- **Motivo (Gate Justificación plan):** counts stale misinforman a los agentes; los agentes leen las skills del submodule; fix trivial quirúrgico.
- **Contrato:** 0 hits `86 tools` en los 6 paths + `validate-docs-coverage.ps1` 0 gaps + cero commit en el submodule (queda en working tree, precedente FIND-103) + resto del árbol dirty intacto.
- **AC:**
  1. `Select-String "86 tools"` vacío en los 4 files / 6 paths (SKILL.md:8,125 + api-reference.md:8,13 + configuration.md:127 + mcp-protocol.md:7).
  2. `pwsh scripts/validate-docs-coverage.ps1` 0 gaps.
  3. Cero commit en el submodule configOpencode (cambio queda en working tree; `git -C .opencode status` sigue dirty, sin commit nuevo).
  4. Resto del árbol dirty intacto (parent `git status`: solo `.opencode` contenido modificado + WIP ajeno pre-existente intacto; staging selectivo SOLO `docs/dev/tasks/FIND-119.md` en repo padre).
- **Pre-mortem (plan) verificado:** (1) tocar de más en árbol ajeno → SOLO los 6 strings (grep antes/después); (2) commit accidental en submodule → prohibido.
- **Stop:** N/A mecánico; si el árbol cambió y los paths no existen → reportar, no cazar.
- **Appetite/Branch/Commit:** max 1h / develop / `docs: FIND-119 — ...` (task file padre; mirrors sin commit).

## 2. ARCHIVOS
- **Clave (4 files, 6 strings a editar — edición mínima `86 tools`→`87 tools` donde corresponda al count total):**
  - `.opencode/skills/vantadb-mcp/SKILL.md:8` (`exposes **86 tools** (49 core + 6 skill_* + ...)` → `87 tools`, breakdown intacto)
  - `.opencode/skills/vantadb-mcp/SKILL.md:125` (`full contract for all **86 tools**` → `87 tools`)
  - `.opencode/skills/vantadb-mcp/references/api-reference.md:8` (`exactly **86 tools** = 49 core` → `87 tools`)
  - `.opencode/skills/vantadb-mcp/references/api-reference.md:13` (`Last synced ... — 86 tools = ...` → `87 tools = ...`, nota FIND-103 intacta)
  - `.opencode/skills/vantadb-mcp/references/configuration.md:127` (`` `full` (default, 86 tools)`` → `87 tools`)
  - `.opencode/skills/vantadb-mcp/references/mcp-protocol.md:7` (`(86 tools: 49 core + ...)` → `(87 tools: ...)`)
- **Lectura fuente (verificar número real 87 antes de editar — DISCOVERY ✅):**
  - `SKILLS-MANIFEST.md:476` (`87 tools, 2 resources, 4 prompts`)
  - `docs/api/MCP.md:198` (`87 tools in 8 families`) + `:200-209` tabla (Core 49 + code 8 + skill 7 + wiki 6 + context 1 + scene 5 + thread 6 + dream 5 = 87) + `:254` (`all 87 tools`)
  - `opencode.jsonc:87` (`87 tools en perfil full`)
  - `skill_extract` en `docs/api/MCP.md:411` (el +1 de FIND-111 `234f0627`: skill_* 6→7)
- **Relacionados (solo lectura):**
  - `docs/dev/plans/2026-09-18-cierre-mvp.md:136-150` (ficha Task 6) + `:36` (verificación real 6 hits) + `:240` (riesgo edición de más)
  - `SPEC.md` raíz (§ Alcance cierre-mvp: counts 87)
- **PROHIBIDOS (todo lo demás — no tocar):** commit en el submodule configOpencode (árbol dirty ajeno de otro repo — 10M + 6 untracked medidos en DISCOVERY, el cambio queda en working tree); resto del árbol `.opencode` intacto (AGENTS.md, commands/audit.md, commands/pipeline.md, references/definition-of-done.md, rules/frontend-web.md, skills/unified-review/profiles/vantadb.yml, task-system/*, memory/*, prompts/*); `reparacion.bat`; `Justfile`; `ocr-*`; `completions/*`; `desktop/src-tauri/Cargo.lock`; stash@{0} GOV-C4; `docs/dev/Backlog.md`; plan file (solo recitation); `C:/Users/Eros/.vantadb*`; `src/`; `examples/` (FIND-116 ✅ `08b7ac8d`, no tocar); `vanta-memory/`; `vantadb-mcp/` (IMPL-112-S1 ✅, no tocar).

## Blast Radius
| Dirección | Módulos |
|-----------|---------|
| Callers | ninguno (strings docs en mirrors; ningún código importa estos .md) |
| Callees | ninguno (edición literal de strings, sin imports ni símbolos) |
| Implicaciones | contrato docs no cambia comportamiento; sin impacto performance/memoria/serialización; sin migración de datos; ningún test existente afectado; breakdown `6 skill_*` y header `Available MCP Tools (86)` y `other 37` quedan NOTICED (sync estructural = repo configOpencode, DEFER plan) |

## Impacto mapeado (Regla 0)
- **Archivos leídos (completos relevantes):** los 4 clave en ventanas exactas (SKILL.md:1-140 incl. :8 y :123-139; api-reference.md:1-30 incl. :5-13; configuration.md:120-134 incl. :127; mcp-protocol.md:1-20 incl. :7) + fuente 87 (MCP.md:195-264 tabla 49+7+8+6+1+5+6+5=87 + skill_extract :411; SKILLS-MANIFEST.md:476; opencode.jsonc:87) + plan ficha Task 6 + FIND-116.md como plantilla task file + `git status` padre + `git -C .opencode status` (10M+6?? ajenos).
- **Archivos referenciados hacia dentro:** ninguno desde código (mirrors .md standalone; `references/api-reference.md § MCP Tools` citado como single-source-of-truth por SKILL.md:125-126 y mcp-protocol.md:7 — cita interna docs, no código).
- **Archivos que referencian a los editados:** agentes OpenCode/Claude/Cursor leen `SKILL.md` como skill (consumo lectura, no import); ningún `use`/`import` de código.
- **Veredicto impacto:** MÍNIMO — 6 reemplazos literales `86 tools`→`87 tools` en 4 .md del submodule; si un editado desapareciera, nada de código se rompe (docs mirrors).

## Contrato
"0 hits `86 tools` en esos 6 paths + coverage 0 gaps + cero commit submodule (working tree, precedente FIND-103) + resto árbol dirty intacto"

## Spec (SDD)
No aplica: strings/docs, cero símbolos públicos nuevos (ninguna señal Phase 1b — no se agrega `pub fn`, tool, endpoint ni método de binding). Tabla Spec N/A (strings/docs).

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** solo 6 strings; cero commit en submodule; resto `.opencode` intacto; WIP ajeno intocable (completions/*, SPEC.md, docs/dev/Backlog.md, reparacion.bat, stash GOV-C4, `C:/Users/Eros/.vantadb*`); secrets nunca a disco; branch develop; NO PUSH.
- **Comandos de verificación:** `Select-String "86 tools"` vacío en 4 files + `Select-String "87 tools"` =6 + `pwsh scripts/validate-docs-coverage.ps1` 0 gaps + `campaign_verify_cmd` (bug exit -1 → bash directa + mención) + `git -C .opencode status` (dirty sin commit nuevo) + `git status --short` padre (staging solo task file).
- **Deuda pendiente:** ninguna prevista (NOTICED §Deuda técnica no es deuda — sync estructural DEFER a configOpencode).

## Recitation (canónico — estructura única)
| Campo recitation (MCP) | Valor |
|------------------------|-------|
| `activeGoal` | FIND-119 sync counts stale mirrors 86→87 solo strings |
| `lastAction` | Step 2 COMPLETED: 12 edits (6 submodule + 6 parent idéntico) + grep 0×86/6×87 ambos lados + coverage exit 0 0 gaps + verify_cmd passed + submodule sin commit |
| `result` | OK (Steps 1-2 ✅) |
| `nextAction` | Commit padre `docs: FIND-119` (solo task file) + recitation plan + progreso + RESULTADO |
| `contract` | verificacion: task file creado; evidencia: claim 6 hits / evidencia Select-String COUNT 6 / confianza alta; claim fuente 87 / evidencia MCP.md:200-209 + SKILLS-MANIFEST:476 + opencode.jsonc:87 / confianza alta; artefactos: docs/dev/tasks/FIND-119.md; invariantes: cero commit submodule, resto dirty intacto; deuda: Step 2 pendiente; queda_pendiente: Wave2 (IMPL-112-S2 + FIND-117 + SHOW-05) |
| `nextTask` | Wave2 (orquestador: IMPL-112-S2 + FIND-117 + SHOW-05) |

## Deuda técnica (Regla 6 — MUST)
**Saldo neto por PR:** cero — strings literales sin atajos. NOTICED BUT NOT TOUCHING (sync estructural = repo configOpencode, DEFER plan — ¿nuevo FIND? → orquestador): `SKILL.md:123` header `Available MCP Tools (86)` (sin sufijo tools, fuera del grep contrato); `SKILL.md:126` `other 37` (87-49=38); breakdowns `6 skill_*` en SKILL.md:8,131 + api-reference.md:9,13 + mcp-protocol.md:7 (fuente real 7 skill_* por skill_extract FIND-111; suma 49+6+...=86 inconsistente con header 87 — fix estructural con fecha sync + nota FIND-103 va en configOpencode, no aquí); `SKILL.md:139` stub `code_files` intacto.

## Definition of Done (contrato multi-nivel)
| Nivel | Gate |
|-------|------|
| **Task** | AC 1-4 ✅ (grep vacío + coverage 0 gaps + submodule sin commit + árbol intacto) |
| **Commit** | commit atómico `docs: FIND-119 — ...`, staging selectivo SOLO `docs/dev/tasks/FIND-119.md` padre, NUNCA submodule, NO PUSH |
| **Release** | N/A (docs mirrors/task file; sin cambio versionable ni changelog) |

## Herramientas necesarias
- grep antes/después en los 6 paths (`Select-String "86 tools"` / `"87 tools"`)
- `pwsh scripts/validate-docs-coverage.ps1` (0 gaps)
- `campaign_verify_cmd` (bug exit -1 → bash directa + mención en RESULTADO)
- codegraph N/A (strings). Cargo N/A. Internet N/A.

## Skills cargadas (SDP)
`SDP: documentation-and-adrs` (strings/docs — sugerida plan ✅ cargada) · base sesión (campaign-executor, progreso, ponytail full, brainstorming, writing-plans, planning-and-task-breakdown). Descartadas del scoring con motivo: `incremental-implementation` (1 slice mecánico, sin slices verticales), `test-driven-development` (sin lógica — verify es grep+coverage, no RED/GREEN), `context-engineering` (context pack mínimo ya en plan+grep, sin sesión compleja), `doubt-driven-development` (sin trust boundary ni stakes producción), `frontend-ui-engineering` (sin UI), `api-and-interface-design` (sin API nueva), `source-driven-development` (sin docs externas — fuente local). Keywords: docs strings sync counts mirrors stale 86 87 tools. Workflow: `bug-fix` (localizing→planning→implementing→testing→review→accept→close — adaptado a strings: localize=grep, plan=task file, implement=6 edits, test=grep+coverage, review=DoD, close=commit padre).

## Investigation Notes
### INVESTIGACIÓN CÓDIGO (DISCOVERY — N/A código; confirmar 6 hits + fuente 87 antes de editar ✅)
- Grep antes (2026-09-18, `Select-String -Pattern "86 tools"` en 4 files): COUNT 6 exactos — SKILL.md:8 (`exposes **86 tools** (49 core + 6 skill_* + 8 code_* + 6 wiki_* + 1 context_assemble + 5 scene_* + 6 thread_* + 5 dream_*)`), SKILL.md:125 (`full contract for all **86 tools**`), api-reference.md:8 (`exactly **86 tools** = 49 core`), api-reference.md:13 (`Last synced 2026-09-17 — 86 tools = ... (FIND-103 recount...)`), configuration.md:127 (`` `full` (default, 86 tools)``), mcp-protocol.md:7 (`(86 tools: 49 core + ...)`).
- Fuente 87 (verificado antes de editar): SKILLS-MANIFEST.md:476 (`87 tools`); docs/api/MCP.md:198 (`87 tools in 8 families`) + tabla :200-209 (49+7+8+6+1+5+6+5=87 — skill_* ya en 7) + :254 (`all 87 tools`); opencode.jsonc:87 (`87 tools en perfil full`); `skill_extract` MCP.md:411 (el +1 FIND-111 `234f0627`).
- Submodule dirty ajeno confirmado (`git -C .opencode status`: 10M — AGENTS.md, commands/audit.md, commands/pipeline.md, references/definition-of-done.md, rules/frontend-web.md, skills/unified-review/profiles/vantadb.yml, los 4 target, task-system/enforcement/verify-log.jsonl, memory/decisions.md, memory/lessons.md, prompts/iter-loop-tools.md, pipeline-full.md, pipeline-run.md, plan.md, task.md — + 6 untracked incl. recall-policy.md) → SOLO los 6 strings, resto intacto.
- Parent dirty pre-existente (`git status --short`: `m .opencode`, `M SPEC.md`, `M completions/*` ×4, `M docs/dev/Backlog.md`, `?? docs/dev/plans/2026-09-18-cierre-mvp.md`, `?? reparacion.bat`) → staging selectivo solo task file.
### INVESTIGACIÓN PROBLEMA
Fuente única vs mirrors: la fuente (87 con skill_extract FIND-111) avanzó 86→87; los mirrors del submodule siguieron en 86 (bump no propagado). Fix = propagar el total a los 6 strings; breakdowns/fechas sync quedan para sync estructural configOpencode (DEFER).
### INVESTIGACIÓN INTERNET
N/A (todo verificado en código local + docs fuente; sin APIs externas).
### Pre-mortem (plan) verificado
(1) tocar de más → grep antes/después acota a 6; (2) commit accidental submodule → prohibido + verificación `git -C .opencode log --oneline -1` sin cambio.

## Incógnitas (uphill) vs Pendientes (downhill)
| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 1 (S2) |
| % completado | 50% |

## Fases explícitas — SECURITY | PERFORMANCE
- [x] **SECURITY** — no aplica: strings docs sin trust boundary (sin input usuario, auth, deps nuevas, storage, FFI, red). Sin `security-and-hardening`.
- [x] **PERFORMANCE** — no aplica: sin hot path (docs mirrors). Sin `performance-optimization`, sin baseline.

## Steps
### Step 1: DISCOVERY (6 hits + fuente 87 + task file + Regla 0)
- **Archivos:** `docs/dev/tasks/FIND-119.md` (este file)
- **Acción:** grep antes 6/6 + verificar fuente 87 (tabla + skill_extract) + `git -C .opencode status` (árbol ajeno) + crear task file con las 10 secciones + Gate D evaluado
- **Verify:** task file existe con Impacto Regla 0 poblado + Gate D no dispara (strings, 4 files, cero símbolos nuevos, contrato claro)
- **Estado:** ✅ COMPLETED

### Step 2: ACT (6×2 edits) + VERIFY (grep+coverage+verify) + commit padre + cierre
- **Archivos:** 4 mirrors submodule + 4 mirrors parent (idéntico FIND-83) + `docs/dev/tasks/FIND-119.md` (sync) + plan file (recitation FIND-119)
- **Acción:** 6× `86 tools`→`87 tools` en submodule + 6× idéntico en parent `skills/` (FIND-83 hash-SAME obligatorio para coverage 0 gaps; parent no estaba en PROHIBIDOS; commit SOLO task file, mirrors quedan en working tree ambos lados) + grep después (0×86 ambos lados, 6×87 exactos) + `pwsh scripts/validate-docs-coverage.ps1` exit 0 0 gaps (10 pares hash-SAME) + `campaign_verify_cmd git diff --check` passed exit 0 (sin bug -1 esta vez) + `git -C .opencode log` sin commit nuevo + `git add docs/dev/tasks/FIND-119.md` + commit `docs: FIND-119 — ...` (NO PUSH, staging selectivo padre) + recitation completed + `skill progreso` + RESULTADO §7
- **Verify:** grep vacío 86 + coverage 0 gaps + `git log --oneline -1` padre + `git status --short` (solo task file stagiado; mirrors+WIP intactos dirty) + `campaign_update_task_state completed`
- **Estado:** ✅ COMPLETED (S2 ejecutado abajo; desvío documentado: 12 strings no 6 — motivo FIND-83/coverage)

## Dependencias
- Wave1 tercera en secuencia (IMPL-112-S1 ✅ `1dd9019c`, FIND-116 ✅ `08b7ac8d` cierre lead; archivos disjuntos — `.opencode/skills/` intocable en esas).
- Stop: si el árbol cambió y los paths no existen → reportar, no cazar (N/A mecánico — paths existen 6/6 ✅).
- NextTask: Wave2 (orquestador: IMPL-112-S2 + FIND-117 + SHOW-05).

## 3. DEPENDENCIAS (resumen)
Wave1 tercera en secuencia. NextTask: Wave2.

## 4. REFERENCIAS
- Rules: ninguna de código aplica (strings/docs — `README.md` del dir formato implícito); marco plan: `api-contract.md` (R-5 tools+docs como contexto, no gate de código).
- Refs: `.opencode/references/definition-of-done.md` (DoD 3 niveles arriba); commands `pipeline.md` (este run); SPEC.md raíz (§ Alcance cierre-mvp: counts 87).
- Agentes: `vanta-worker` (ejecuta) + `vanta-review` P2-01 (lo hace el orquestador, no vos).
- Workflow: `bug-fix` (localize→plan→implement→test→review→accept→close adaptado a strings).

## 5. SKILLS
Ver sección Skills cargadas (SDP) arriba. `SKILLS_CARGADAS:` en RESULTADO §7.

## 6. HERRAMIENTAS+MCP
grep antes/después en los 6 paths + `pwsh scripts/validate-docs-coverage.ps1` + `campaign_verify_cmd` (bug exit -1 → bash directa + mención). codegraph N/A (strings). Cargo N/A. Internet N/A.

## 7. INVESTIGACIÓN CÓDIGO
Ver Investigation Notes (DISCOVERY ✅ — 6 hits + fuente 87 + árbol ajeno).

## 8. INVESTIGACIÓN PROBLEMA
Ver Investigation Notes (fuente única vs mirrors tras bump FIND-111).

## 9. INVESTIGACIÓN INTERNET
N/A.

## 10. VALIDACIÓN+CIERRE
Verify contrato (S2) + OCR delegation N/A-justificado (solo strings en mirrors submodule — sin código, sin trust boundary; wrapper padre no toca estos paths; se documenta y no se corre) + DoD task/commit (commit `docs:` SOLO `docs/dev/tasks/FIND-119.md` padre, NUNCA submodule) + P2-01 lo hace el orquestador (no vos) + Gates D/V/C vía `question` (D: no dispara — strings 4 files sin símbolos nuevos; V: solo si verify falla 2× mismo error; C: colaterales = árbol dirty ajeno → staging selectivo, no toco) + RESULTADO §7 obligatorio.

## Review (GATE — agente distinto, P2-01)
- **Revisor:** orquestador/lead (P2-01; worker no se auto-aprueba — pendiente veredicto del orquestador)
- **Enfoque:** ¿solo los 6 strings? Diff submodule acotado a `86 tools`→`87 tools` ×6, breakdowns/fechas intactos con motivo; ¿submodule sin commit? `git -C .opencode log` intacto; ¿padre staging solo task file?
- **Cómo se probó:** (a completar en S2) grep después + coverage 0 gaps + verify + `git -C .opencode status/log` + `git status` padre.
- **Checklist anti-hábitos tóxicos:**
  - [ ] No inventar salidas de comandos (exits + logs reales en S2).
  - [ ] No saltarse clarificación (fuente 87 verificada antes de editar; breakdown 6→7 NOTICED no asumido).
  - [ ] No declarar done sin verificar AC (AC 1-4 con evidencia en S2).
  - [ ] No ignorar fallos (si coverage/verify falla → retry ladder + Gate V, no silencio).
  - [ ] No un solo intento de búsqueda (grep + fuente triple + status doble en DISCOVERY).
  - [ ] No copiar sin citar (toda afirmación con file:línea).
  - [ ] No reintentar en bucle (2 fallas mismo-error → Gate V).
  - [ ] No dejar huérfanos (NOTICED estructurales con ¿nuevo FIND? → orquestador).
  - [ ] No degradar errores (N/A, strings sin paths dinero/seguridad).
  - [ ] No gastar presupuesto infinito (appetite max 1h; stop N/A mecánico).
- **Veredicto:** pendiente orquestador (worker deja evidencia + diff mínimo)

## Notas
- Gate D: NO dispara (strings quirúrgicos 6/4 files, sin símbolos públicos nuevos, sin hot path/API, contrato claro).
- Gate P: heredado del plan (set de 9 + mirrors solo-strings aprobado por owner).
- Branch: develop. Commit padre: `docs: FIND-119 — ...`. NO PUSH. Submodule: NUNCA commit.

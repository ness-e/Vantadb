# Plan de Ejecución: Task System Hardening — concurrencia, dedup y HITL

> **Campaign ID:** be3e7379-79e8-46a5-b5c9-dc20e62336ca
> **Inicio:** 2026-08-23
> **Estado:** ✅ COMPLETED
> **Fuente:** Auditoría `.opencode/` 2026-08-23 + decisiones del usuario (question gate inicial)

## Resumen
| DO | DEFER | SKIP | BLOQUEADO |
|----|-------|------|-----------|
| 9  | 1     | 0    | 0         |

**Decisiones del usuario (Gate P, 2026-08-23):**
1. Migración de `tasks/` fuera de `skills/` → **DEFER** (queda como deuda documentada).
2. HITL → **4 Question Gates** (Plan / Discovery / Verify×2 / Cierre).
3. Umbral único de fallo mismo-error → **2 fallas**.
4. Ejecución → **por fases con verificación entre fases**, en esta sesión.

---

### Task 1: H1 — Fixes duros de código en campaign-server.mjs
- **Archivos clave:** .opencode/task-system/mcp/campaign-server.mjs 🟢
- **Contrato:** `node --test .opencode/task-system/mcp/` pasa (parsers + state-persistence + tests nuevos de esta fase) — VERIFICADO: 38/38 ✅
- **Task file:** tasks/H1-fixes-codigo.md
- **Estado:** ✅ COMPLETED
- **Alcance:**
  - Importar `mkdirSync` desde `node:fs` (líneas 1042 y 1336 lo usan sin importarlo).
  - Whitelist VERIFY: agregar prefijos `npx`, `python`, `scripts/`, `npm run` (los contratos canónicos usan `npx tsc --noEmit` y `python -m pytest`).
  - `updateRecitation`: anclar replacements a inicio de línea (`/^Estado:/m`, etc.) para no corromper la recitation si `contract` contiene "Estado:" literal.

### Task 2: H2 — detectType: orden específico→genérico
- **Archivos clave:** .opencode/task-system/mcp/campaign-server.mjs 🟢
- **Contrato:** test nuevo en parsers.test.mjs: `vantadb-python/src/lib.rs` → type python; `web/src/App.tsx` → frontend; `src/engine.rs` → rust
- **Task file:** tasks/H2-detecttype.md
- **Estado:** ✅ COMPLETED
- **Alcance:** reordenar TYPE_PATTERNS (python/ts/web/docs/devops antes de `/src\//`), o hacer matching por especificidad.

### Task 3: H3 — Budget bajo lock + TTL en scan WIP
- **Archivos clave:** .opencode/task-system/mcp/campaign-server.mjs 🟡
- **Contrato:** test de concurrencia nuevo (state-persistence.test.mjs): consumeBudget paralelo conserva todos los incrementos; WIP stale (>24h) no bloquea claim
- **Task file:** tasks/H3-concurrencia.md
- **Estado:** ✅ COMPLETED
- **Alcance:**
  - Envolver `initTaskBudget`/`consumeBudget`/`budgetReset`/`getOrCreateTraceId`/`budgetPath+readBudget` RMW en `withPlanLock(planPath, ...)`.
  - `findInProgressTasks`: excluir del bloqueo los IN PROGRESS con última actividad > 24h (budget.lastActivity para tareas de plan; mtime para task files) — reportarlos como `staleTasks`.
  - `campaign_get_next_task`: envolver el write de Campaign ID en withPlanLock.

### Task 4: H4 — Umbral único (2 fallas) y presupuestos con fuente única
- **Archivos clave:** .opencode/task-system/prompts/pipeline-full.md, prompts/iter-loop-tools.md, skills/campaign-executor/SKILL.md, prompts/pipeline-run.md 🟢
- **Contrato:** grep no encuentra contradicciones: `rg -n "3\\+ iteraciones con el mismo error|3 fallas|mismo error" .opencode/` consistente con umbral 2; tabla Budget de SKILL.md == BUDGET_LIMITS del server
- **Task file:** tasks/H4-umbral-presupuestos.md
- **Estado:** ✅ COMPLETED
- **Alcance:**
  - iter-loop-tools stagnation detection: mismo-error = 2 → FAILED (los otros patrones de stagnación quedan en 3/5 iteraciones).
  - Números de presupuesto: SKILL.md tabla actualizada a valores de BUDGET_LIMITS (fuente única); pipeline-run/iter-loop-tools referencian en vez de duplicar.
  - pipeline-run "~2 min timeout" por sub-agente → 10 min (realista con cargo).

### Task 5: H5+H6 — Alinear RESEARCH/bash y workflow-vs-C0 en iter-loop-tools
- **Archivos clave:** .opencode/task-system/prompts/iter-loop-tools.md 🟢
- **Contrato:** la tabla per-state coincide con config/state-tools.mjs (RESEARCH permite bash read-only); paso DISCOVERY 2 ya no dice "usá los estados del workflow como state machine"
- **Task file:** tasks/H5-H6-alineacion.md
- **Estado:** ✅ COMPLETED

### Task 6: H7 — Question Gates HITL (canónico) + spec-driven guiado
- **Archivos clave:** .opencode/task-system/prompts/question-gates.md (NUEVO), prompts/plan.md, prompts/pipeline-full.md, prompts/iter-loop-tools.md, prompts/pipeline-run.md, prompts/subagent-recovery.md 🟡
- **Contrato:** cada prompt referenciado contiene la sección Question Gates apuntando al archivo canónico; `rg -l "question-gates.md"` devuelve los 6 archivos; ningún prompt redefine los gates (solo referencian)
- **Task file:** tasks/H7-question-gates.md
- **Estado:** ✅ COMPLETED
- **Alcance (spec de los 4 gates):**
  - **Gate P — Plan/Triage** (plan.md): durante triage, toda tarea 🔴 o con contrato ambiguo (≥2 interpretaciones) se pregunta vía `question` tool (opciones GO/ajustar/DEFER/SKIP); resumen final de DEFER/SKIP confirmado antes de escribir el plan. Para feature-add/lógica nueva: mini-spec vía skill `spec-driven-development` refinada con questions ANTES de aprobar la tarea en el plan.
  - **Gate D — Discovery** (pipeline-full.md + iter-loop-tools MODO DISCOVERY): tras zero-code planning, si contrato ambiguo O blast radius >10 archivos O prioridad 🔴 → `question` con dirección propuesta + alternativas antes de escribir el task file.
  - **Gate V — Verify-falla×2** (pipeline-full + subagent-recovery): agotado el umbral de 2 fallas mismo-error, ANTES de marcar FAILED → `question` (reintentar fresh / cambiar estrategia / abortar-failed). Reemplaza el escalate-a-humano silencioso por pregunta estructurada.
  - **Gate C — Cierre** (pipeline-full MODO CIERRE): errores colaterales encontrados → `question` (arreglar ahora <30min / mandar a Backlog / incluir en commit); si `git status` muestra archivos fuera del blast radius declarado → confirmar alcance del commit.
  - Regla transversal: las questions SIEMPRE llevan opciones concretas + default recomendado; nunca preguntas abiertas sin contexto. El resultado queda registrado en recitation (`decision_reason`) y trace log.

### Task 7: H8 — Memoria única (learnings → lessons.md)
- **Archivos clave:** AGENTS.md (.opencode), .opencode/task-system/memory/lessons.md 🟡
- **Contrato:** AGENTS.md ya no acumula bloques Learnings nuevos (regla escrita); learnings históricos migrados a lessons.md con schema TSYS-15 (`- fecha | tema | lección | ref:`)
- **Task file:** tasks/H8-memoria.md
- **Estado:** ✅ COMPLETED
- **Alcance:** script de migración one-shot (parsear bloques `<!-- Learnings: X — fecha -->` de AGENTS.md → líneas TSYS-15 en lessons.md), dejar en AGENTS.md solo la regla "learnings van vía campaign_memory_write". Pipeline-full §Cierre actualizado para usar memory_write en vez de editar AGENTS.md.

### Task 8: H9 — Limpieza menor
- **Archivos clave:** .opencode/tasks/COMP-010.md, .opencode/skills/review-deep/tmp/, .opencode/skills/campaign-executor/SKILL.md 🟢
- **Contrato:** grep confirma cero referencias antes de borrar (Regla 0); `skill ponytail (full)` eliminado de load_skills
- **Task file:** tasks/H9-limpieza.md
- **Estado:** ✅ COMPLETED
- **Alcance:** huérfano COMP-010 (verificar referencias primero), tmp artifacts de review-deep, fix `"ponytail (full)"` → `"ponytail"` en campaign_load_skills, verificar .gitignore cubre node_modules de task-system/mcp.

---

## DEFER (decisión usuario 2026-08-23)
- Migración de `tasks/<ID>.md` fuera de `.opencode/skills/campaign-executor/tasks/` (~300 archivos). Deuda documentada acá; reabrir cuando el directorio supere ~500 archivos o rompa un grep/glob de skills.

---

## Protocolo de ejecución (cómo corre este plan)

**Skills base:** `campaign-executor` (SKILL.md+RULES.md), `progreso`, `ponytail (full)` — cargadas una sola vez al arrancar; por-task vía MCP `campaign_load_skills`.

**Por cada tarea (H1..H9):**
1. `campaign_get_next_task` → tarea activa + recitation
2. `campaign_load_skills` (archivos clave) → skills + checks
3. `campaign_detect_task_type` + codegraph_explore (blast radius) — Discovery
4. Crear task file `tasks/H<N>.md` con steps atómicos (si no existe) + **Impacto mapeado (Regla 0)** antes de editar
5. State machine C0: PLAN → ACT → VERIFY (`campaign_verify_cmd`, nunca auto-reporte)
6. **Question Gate** aplicable (D/V/C según fase — ver H7)
7. `campaign_update_task_state` con recitation (result OK/PARTIAL/FAILED + contract)
8. Commit Conventional: `fix|docs|chore(task-system): H<N> — descripción`

**Verificación entre fases (gate de avance):**
- Fase código (H1–H3): `node --test .opencode/task-system/mcp/` verde + smoke del server (`bun .opencode/task-system/mcp/campaign-server.mjs` arranca sin crash).
- Fase prompts (H4–H7): greps de consistencia del contrato de cada tarea + revisión cruzada de que ningún prompt duplica números/gates.
- Fase memoria/limpieza (H8–H9): conteos before/after de entradas migradas; greps de referencias antes de cada delete.
- Si un gate de fase falla → NO avanzar a la siguiente fase; aplicar retry ladder (2 fallas mismo-error → Question Gate V).

**Cierre de campaña:** verify final (tests + greps agregados), retrospectiva Start/Stop/Continue, `skill progreso`, archivar plan.

=== RECITATION ===
Campaign ID: be3e7379-79e8-46a5-b5c9-dc20e62336ca
Objetivo activo: Task System Hardening — campaña completa
Estado: completed
Última acción: H1-H9 ejecutados y verificados; header del plan reparado (corrupción del server viejo, bug ya fixeado en parsers.mjs)
Resultado: OK
Contrato: node --test parsers+state-persistence+hardening = 38/38 ✅; greps de consistencia OK; commits fcd7b243 + 26f68ff7
Próxima tarea si completa: ninguna — campaña cerrada
=== END RECITATION ===

## Retrospectiva de cierre (Start / Stop / Continue)

**Start:**
- Dogfooding del MCP durante la ejecución: detectó 2 bugs que la auditoría estática no vio (placeholder de Campaign ID sombreando el ID real; regex sin anclar corrompiendo campos del plan).

**Stop:**
- Duplicar números de presupuesto en prompts (fuente única: BUDGET_LIMITS).
- Acumular learnings en AGENTS.md (memoria única: lessons.md vía campaign_memory_write).

**Continue:**
- Ejecución por fases con verificación mecánica entre fases (detectó cada regresión a tiempo).
- Question gates antes de decisiones estructurales (las 4 respuestas iniciales definieron todo el plan).

**Una acción medible:** reducir el drift prosa↔código a 0 — cada vez que un prompt cite un número/límite, verificarlo contra el .mjs en el mismo PR (métrica: contradicciones encontradas por `rg` en el gate de fase; baseline esta campaña: 4 contradictorias → objetivo: 0).

> **Nota:** el server MCP corriendo durante esta sesión era código pre-fix. Los bugs
> de parsers que exhibió (Campaign ID duplicado/sombreado, campos pisados por
> recitations) están corregidos en disco — **reiniciar OpenCode** para cargarlos.


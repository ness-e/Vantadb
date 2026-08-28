# Plan de Ejecución: Master Pipeline Optimization — VantaDB

> **Inicio:** 2026-08-28
> **Estado:** ⏳ EN PROGRESO
> **Fuente:** Análisis completo del sistema de ejecución de tareas y planes

## Resumen

| Resultado | Count |
|-----------|-------|
| ✅ DO | 20 |
| 🟡 DEFER | 0 |
| ❌ SKIP | 0 |
| 🔴 BLOQUEADO | 0 |

Status: ⬆️ uphill = 0 · ⬇️ downhill = 20

## Tasks

### Task 1: CORE-001 — Scope Enforcement en ACT State (CRÍTICO #1)

- **Appetite:** max 2d
- **Esfuerzo:** 🟡 1-2d
- **Prioridad:** 🔴
- **Archivos clave:** `config/state-tools.mjs`, `.opencode/task-system/mcp/campaign-server.mjs`, `.opencode/task-system/prompts/iter-loop-tools.md`
- **Verificación real:** `codegraph_explore` → `validateAction` en `state-tools.mjs:73-84` recibe solo `(state, toolName)` sin scope; `campaign_enforce_state` no valida blast radius
- **Gate Justificación:** Gap crítico de seguridad — agente puede editar fuera del blast radius declarado (Regla 0)
- **Contrato:** `campaign_verify_cmd command="node -e \"require('.opencode/task-system/mcp/campaign-server.mjs')\""` + test manual: crear task con blast radius acotado, intentar editar archivo fuera → debe fallar en ACT
- **Pre-mortem:**
  - Fallo 1: `validateAction` no tiene acceso al task file para leer blast radius → necesito nuevo tool MCP `campaign_validate_scope` que reciba taskId + filePath
  - Falso 2: Cambio en `state-tools.mjs` rompe transiciones existentes → test exhaustivo de C0 states
- **Stop conditions:** Appetite excedido >2d → re-triaje; validateAction roto → rollback inmediato
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🔴 | `validateAction` breaking changes | Test matrix completa 10 states × tools | 2 iteraciones sin green |
  | 🟡×🟠 | Blast radius parsing inconsistente | Formato estándar en task file | Gate C pre-commit |
- **Cynefin:** 🟨 Complicado — requiere experto en state machine + MCP tools
- **Top 3 riesgos:**
  1. `validateAction` signature change rompe `campaign_enforce_state` calls
  2. Blast radius en task file no parseable → false positives/negatives
  3. Performance: validación por cada tool call en ACT
- **Uphill/Downhill:** ⬆️ 3 · ⬇️ 5
- **DoD multi-nivel:** Task: verify mecánico + test manual; Commit: conventional + verify full; Release: changelog + docs
- **Estado inicial:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/CORE-001.md`

---

### Task 2: CORE-002 — campaign_validate_output (LLM05) Enforzado en ACT (CRÍTICO #2)

- **Appetite:** max 1d
- **Esfuerzo:** 🟢 1d
- **Prioridad:** 🔴
- **Archivos clave:** `config/state-tools.mjs`, `.opencode/task-system/mcp/campaign-server.mjs`, `.opencode/task-system/prompts/iter-loop-tools.md`
- **Verificación real:** `campaign_validate_output` tool existe (línea 320-333) pero NO se llama en ACT state; `state-tools.mjs` ACT permite `edit`, `write`, `bash` sin validación
- **Gate Justificación:** Rule 0 (AGENTS.md) exige Output Validation antes de write/edit/bash que genera shell/SQL/Python/HTML/file_path
- **Contrato:** `campaign_verify_cmd command="grep -r 'campaign_validate_output' .opencode/task-system/prompts/iter-loop-tools.md"` → debe aparecer en ACT section
- **Pre-mortem:**
  - Falso 1: Validación muy estricta bloquea edits legítimos → whitelist por tipo de contenido
  - Falso 2: `bash` commands heterogéneos → clasificar solo los que generan archivos
- **Stop conditions:** Tests rotos en ACT → ajustar whitelist; >1d → simplificar a solo `write` + `edit`
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟢×🔴 | False positives bloquean desarrollo | Whitelist `edit` con oldString/newString | Gate C |
  | 🟡×🟠 | `bash` classification incompleta | Solo validar `write` + `edit` output | 1 iteración |
- **Cynefin:** 🟦 Obvio — hook directo en state machine
- **Top 3 riesgos:**
  1. `bash` commands que escriben archivos pasan sin validar
  2. `edit` tool output no validado (oldString/newString son seguros por definición)
  3. Performance overhead mínimo (validación sincrónica)
- **Uphill/Downhill:** ⬆️ 1 · ⬇️ 3
- **Estado inicial:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/CORE-002.md`

---

### Task 3: CORE-003 — Question Gates Enforcement Automático (CRÍTICO #3)

- **Appetite:** max 1d
- **Esfuerzo:** 🟢 1d
- **Prioridad:** 🔴
- **Archivos clave:** `.opencode/task-system/prompts/pipeline-run.md`, `.opencode/task-system/prompts/subagent-recovery.md`, `.opencode/task-system/prompts/question-gates.md`
- **Verificación real:** `pipeline-run.md` paso 6.h clasifica resultado pero **no valida** que sub-agentes sin `question` devuelvan `BLOQUEO:` cuando gate D/V/C dispara
- **Gate Justificación:** Flujo HITL roto — sub-agente puede saltar gate silenciosamente
- **Contrato:** `campaign_verify_cmd command="grep -A5 'BLOQUEO:' .opencode/skills/campaign-executor/tasks/*.md"` → todo task con gate disparado debe tener BLOQUEO
- **Pre-mortem:**
  - Falso 1: Sub-agente olvida BLOQUEO → orquestador no pregunta → gate saltado
  - Falso 2: Orquestador no reanuda vía SARL RESUME correctamente
- **Stop conditions:** Gate saltado en test manual → fix inmediato
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🔴 | Gate D/V/C saltado silenciosamente | Validación en pipeline-run paso 6.h | Cada task completion |
  | 🟢×🟠 | SARL RESUME con feedback malformado | Test de reanudación con task_id | 1er retry |
- **Cynefin:** 🟦 Obvio — validación + logging obligatorio
- **Top 3 riesgos:**
  1. Sub-agente no devuelve `BLOQUEO:` aunque gate dispare
  2. Orquestador no hace `question` al recibir `BLOQUEO:`
  3. `task_id` no propagado para RESUME
- **Uphill/Downhill:** ⬆️ 2 · ⬇️ 4
- **Estado inicial:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/CORE-003.md`

---

### Task 4: CORE-004 — Task File Template Completo (CRÍTICO #4)

- **Appetite:** max 0.5d
- **Esfuerzo:** 🟢 0.5d
- **Prioridad:** 🔴
- **Archivos clave:** `.opencode/skills/campaign-executor/templates/task-definition.md`
- **Verificación real:** Template actual tiene 38 líneas vs especificación en `task.md` que requiere 20+ secciones obligatorias
- **Gate Justificación:** Task files generados incompletos → faltan campos críticos (Regla 0, Spec, Invariantes, Deuda, Review, etc.)
- **Contrato:** `campaign_verify_cmd command="diff -u <(grep -c '^##' .opencode/skills/campaign-executor/templates/task-definition.md) <(echo 20)"` → ≥20 secciones `##`
- **Pre-mortem:**
  - Falso 1: Template muy verboso → agentes lo ignoran → mantener conciso pero completo
  - Falso 2: Campos opcionales vs obligatorios no distinguidos → marcar con `(obligatorio)`
- **Stop conditions:** Template >200 líneas → recortar a esenciales
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟢×🟠 | Template incompleto → task files malformados | Checklist en pipeline-full Discovery | Gate D |
- **Cynefin:** 🟦 Obvio — actualización de plantilla
- **Top 3 riesgos:**
  1. Falta `Impacto mapeado (Regla 0)` → ediciones sin verificación de impacto
  2. Falta `Review (gate P2-01)` → tasks marcadas COMPLETED sin review
  3. Falta `Invariantes de dominio` → handoff roto entre iteraciones
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 2
- **Estado inicial:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/CORE-004.md`

---

### Task 5: CORE-005 — SDP Unificado: campaign_discover_skills MCP Tool (CRÍTICO #5)

- **Appetite:** max 2d
- **Esfuerzo:** 🟡 1-2d
- **Prioridad:** 🔴
- **Archivos clave:** `.opencode/task-system/mcp/campaign-server.mjs`, `.opencode/references/skills-engineering.md`, `.opencode/task-system/prompts/pipeline-full.md`, `.opencode/task-system/prompts/iter-loop-tools.md`, `.opencode/task-system/prompts/task.md`, `.opencode/task-system/prompts/plan.md`
- **Verificación real:** SDP duplicado en 4 prompts, `plan.md` y `task.md` no usan `campaign_load_skills`, lifecycle mapping manual, grep SKILLS-MANIFEST.md manual
- **Gate Justificación:** Inconsistencia en skill loading → skills incorrectas/faltantes → verificación débil
- **Contrato:** Nuevo tool `campaign_discover_skills(keywords, phase)` devuelve `{ skills, justificaciones, lifecycle_phase }`; `campaign_load_skills` actualizado para usarlo; todos prompts invocan MCP
- **Pre-mortem:**
  - Falso 1: Tool muy complejo → empezar simple: keywords + phase → skills
  - Falso 2: Cache invalidation → TTL por sesión
  - Falso 3: Prompts existentes rompen → migración gradual con fallback
- **Stop conditions:** >2d → dividir: tool primero, migración prompts después
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🔴 | Breaking change en prompts | Fallback a carga manual si tool falla | Cada prompt load |
  | 🟡×🟠 | Lifecycle mapping incompleto | Mantener tabla en skills-engineering.md como fallback | SDP step 2 |
- **Cynefin:** 🟨 Complicado — MCP tool + migración 6 prompts
- **Top 3 riesgos:**
  1. `campaign_load_skills` signature change rompe `pipeline-run.md` paso 6.c
  2. `plan.md` y `task.md` no migran → inconsistencia persiste
  3. Cache SDP introduce staleness → TTL 1 sesión
- **Uphill/Downhill:** ⬆️ 4 · ⬇️ 8
- **Estado inicial:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/CORE-005.md`

---

### Task 6: HIGH-006 — detect_changes en plan.md Paso 0 (ALTO #6)

- **Appetite:** max 0.5d
- **Esfuerzo:** 🟢 0.5d
- **Prioridad:** 🟠
- **Archivos clave:** `.opencode/task-system/prompts/plan.md`
- **Verificación real:** Paso 0 usa solo `codegraph_explore`; `detect_changes` da blast radius transitivo + risk classification + módulos impactados
- **Gate Justificación:** Verificación real más completa para triage gate
- **Contrato:** `campaign_verify_cmd command="grep -n 'detect_changes' .opencode/task-system/prompts/plan.md"` → línea en Paso 0
- **Pre-mortem:** Ninguno — adición simple
- **Stop conditions:** Ninguno
- **Risk Register:** (ninguno — cambio aditivo)
- **Cynefin:** 🟦 Obvio
- **Top 3 riesgos:** (ninguno)
- **Uphill/Downhill:** ⬆️ 0 · ⬇️ 1
- **Estado inicial:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/HIGH-006.md`

---

### Task 7: HIGH-007 — Re-validar Skills tras Discovery (ALTO #7)

- **Appetite:** max 0.5d
- **Esfuerzo:** 🟢 0.5d
- **Prioridad:** 🟠
- **Archivos clave:** `.opencode/task-system/prompts/pipeline-full.md`
- **Verificación real:** Paso 0 llama `campaign_load_skills`; Discovery puede cambiar tipo (fix→feature-add) pero skills no se recargan
- **Gate Justificación:** Skills desfasadas → verificación incorrecta
- **Contrato:** `campaign_verify_cmd command="grep -A3 'Re-validar skills' .opencode/task-system/prompts/pipeline-full.md"` → bloque en Discovery
- **Pre-mortem:** Recarga innecesaria si tipo no cambia → condicionar a cambio de tipo
- **Stop conditions:** Ninguno
- **Cynefin:** 🟦 Obvio
- **Estado inicial:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/HIGH-007.md`

---

### Task 8: HIGH-008 — Autonomous Flag en Plan File (ALTO #8)

- **Appetite:** max 0.5d
- **Esfuerzo:** 🟢 0.5d
- **Prioridad:** 🟠
- **Archivos clave:** `.opencode/task-system/prompts/plan.md`, `.opencode/task-system/prompts/question-gates.md`
- **Verificación real:** `question-gates.md` §Anti-abuso referencia `> **Autonomous:** true` pero plan template no tiene campo
- **Gate Justificación:** Modo autónomo no usable sin campo en plan
- **Contrato:** `campaign_verify_cmd command="grep -n 'Autonomous:' .opencode/task-system/prompts/plan.md"` → campo en template
- **Pre-mortem:** Default `false` para backward compat
- **Stop conditions:** Ninguno
- **Cynefin:** 🟦 Obvio
- **Estado inicial:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/HIGH-008.md`

---

### Task 9: HIGH-009 — Session Cleanup al Cerrar Campaña (ALTO #9)

- **Appetite:** max 0.5d
- **Esfuerzo:** 🟢 0.5d
- **Prioridad:** 🟠
- **Archivos clave:** `.opencode/task-system/prompts/pipeline-run.md`
- **Verificación real:** `campaign_session_track action="create/update"` usado pero no `delete` al finalizar
- **Gate Justificación:** Stale sessions contaminan siguientes campañas
- **Contrato:** `campaign_verify_cmd command="grep -n 'session_track.*delete' .opencode/task-system/prompts/pipeline-run.md"` → en paso 8
- **Pre-mortem:** Session ID tracking para cleanup correcto
- **Stop conditions:** Ninguno
- **Cynefin:** 🟦 Obvio
- **Estado inicial:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/HIGH-009.md`

---

### Task 10: HIGH-010 — Context Save Point Reconstruction Tool (ALTO #10)

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟠
- **Archivos clave:** `.opencode/task-system/mcp/campaign-server.mjs`
- **Verificación real:** SARL §3.6 dice "mandar a RESUME a reconstruir con git diff/git log + task file" pero no hay tool MCP
- **Gate Justificación:** RESUME falla si sub-agente no escribió Context Save Point
- **Contrato:** Nuevo tool `campaign_reconstruct_context(taskId, planFile)` → devuelve `{ lastAction, nextStep, modifiedFiles, decisions }`
- **Pre-mortem:** Reconstrucción imperfecta → marcar confianza `media` en recitation
- **Stop conditions:** >1d → simplificar a solo `git diff --name-only` + task file steps
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🟠 | Reconstrucción incompleta | Confianza `media` + requerir verificación manual | RESUME attempt |
- **Cynefin:** 🟨 Complicado — parsing git diff + task file
- **Top 3 riesgos:**
  1. `git diff` ruidoso → archivos no relacionados
  2. Task file steps no reflejan estado real
  3. Decisiones no persistidas en task file
- **Uphill/Downhill:** ⬆️ 2 · ⬇️ 3
- **Estado inicial:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/HIGH-010.md`

---

### Task 11: HIGH-011 — Fork/Join en EJECUCIÓN (ALTO #11)

- **Appetite:** max 2d
- **Esfuerzo:** 🟡 1-2d
- **Prioridad:** 🟠
- **Archivos clave:** `.opencode/task-system/prompts/pipeline-full.md`
- **Verificación real:** Fork/Join solo en CIERRE (fmt/clippy/nextest); EJECUCIÓN secuencial
- **Gate Justificación:** Steps independientes (ej: editar archivo A + archivo B sin dependencias) podrían paralelizarse
- **Contrato:** En EJECUCIÓN, detectar steps ⬜ PENDING sin dependencias mutuas → spawn sub-agentes paralelos (max 2)
- **Pre-mortem:**
  - Falso 1: Dependencias implícitas no declaradas → race conditions
  - Falso 2: Context merging complejo → solo para steps verdaderamente independientes
- **Stop conditions:** >2d → posponer a Fase 2
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🔴 | Race condition en edits paralelos | Solo steps con archivos disjuntos + sin dependencias | Gate D zero-code planning |
  | 🟢×🟠 | Context merge loss | Merge solo recitation + git commits | SARL trace |
- **Cynefin:** 🟨 Complicado — análisis de dependencias entre steps
- **Top 3 riesgos:**
  1. Steps declarados independientes pero comparten estado global
  2. Sub-agente parallel consume budget 2x
  3. SARL recovery más complejo con múltiples sub-agentes
- **Uphill/Downhill:** ⬆️ 3 · ⬇️ 5
- **Estado inicial:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/HIGH-011.md`

---

### Task 12: HIGH-012 — Auto-transition State Machine (ALTO #12)

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟠
- **Archivos clave:** `.opencode/task-system/mcp/campaign-server.mjs`, `.opencode/task-system/prompts/iter-loop-tools.md`
- **Verificación real:** Agente debe llamar `campaign_update_task_state` manualmente; no hay auto-transition en verify pass/fail
- **Gate Justificación:** Error humano olvida actualizar estado → recitation desactualizada
- **Contrato:** `campaign_verify_cmd` en MCP server actualiza estado automáticamente: pass → `completed`, fail → `in-progress` (retry)
- **Pre-mortem:** Auto-transition puede ser prematuro → solo si step tiene `verify` command definido
- **Stop conditions:** >1d → solo documentar obligatoriedad manual
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟢×🟠 | Auto-transition prematuro | Requerir `verify` command en step | Step definition |
- **Cynefin:** 🟨 Complicado — MCP server state management
- **Top 3 riesgos:**
  1. Verify command no definido → estado incorrecto
  2. MoM ladder interfere con auto-transition
  3. SARL RESUME espera estado específico
- **Uphill/Downhill:** ⬆️ 2 · ⬇️ 3
- **Estado inicial:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/HIGH-012.md`

---

### Task 13: HIGH-013 — classifyBashWrite Implementado (ALTO #13)

- **Appetite:** max 0.5d
- **Esfuerzo:** 🟢 0.5d
- **Prioridad:** 🟠
- **Archivos clave:** `.opencode/task-system/mcp/campaign-server.mjs`
- **Verificación real:** `state-tools.mjs` RESEARCH state dice `classifyBashWrite bloquea writes` pero no implementado
- **Gate Justificación:** RESEARCH state permite writes vía bash sin control
- **Contrato:** `campaign_verify_cmd command="grep -n 'classifyBashWrite' .opencode/task-system/mcp/campaign-server.mjs"` → función implementada + hook en `validateAction` para bash
- **Pre-mortem:** Clasificación heurística → false positives → solo patrones obvios (`>`, `tee`, `write`, `echo` con redirección)
- **Stop conditions:** >0.5d → solo documentar y dejar para v2
- **Cynefin:** 🟦 Obvio — pattern matching en bash command
- **Estado inicial:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/HIGH-013.md`

---

### Task 14: HIGH-014 — Index Health Check al Inicio (ALTO #14)

- **Appetite:** max 0.5d
- **Esfuerzo:** 🟢 0.5d
- **Prioridad:** 🟠
- **Archivos clave:** `.opencode/skills/campaign-executor/SKILL.md`, `.opencode/task-system/prompts/pipeline-run.md`
- **Verificación real:** Skill `codebase-memory` dice "incluye health check + re-index auto si >7d o falla" pero no se invoca al inicio de campaña
- **Gate Justificación:** Índice stale → blast radius incorrecto → verificación falsa
- **Contrato:** `campaign_verify_cmd command="grep -n 'index_status' .opencode/task-system/prompts/pipeline-run.md"` → en probes de integridad (paso 4)
- **Pre-mortem:** Re-index lento → async con timeout
- **Stop conditions:** Re-index >5min → skip con warning
- **Cynefin:** 🟦 Obvio
- **Estado inicial:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/HIGH-014.md`

---

### Task 15: MED-015 — Skill Discovery MCP Tool (MEDIO #15)

- **Appetite:** max 2d
- **Esfuerzo:** 🟡 1-2d
- **Prioridad:** 🟡
- **Archivos clave:** `.opencode/task-system/mcp/campaign-server.mjs`, `.opencode/references/skills-engineering.md`
- **Verificación real:** SDP usa lifecycle mapping manual + grep SKILLS-MANIFEST.md manual; podría ser tool
- **Gate Justificación:** Automatizar discovery reduce errores y tiempo
- **Contrato:** Nuevo tool `campaign_discover_skills(keywords, phase)` → usa lifecycle mapping table + grep SKILLS-MANIFEST.md → devuelve skills con justificaciones
- **Pre-mortem:** Tool duplica lógica de `campaign_load_skills` → unificar en uno solo
- **Stop conditions:** >2d → posponer, usar prompt compartido como fallback
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟡×🟠 | Duplicación con load_skills | Merge en `campaign_load_skills` v2 | Design review |
- **Cynefin:** 🟨 Complicado — automatización de discovery
- **Top 3 riesgos:**
  1. Keywords ambiguas → skills irrelevantes
  2. Lifecycle mapping desactualizado vs skills-engineering.md
  3. Cache SDP staleness
- **Uphill/Downhill:** ⬆️ 2 · ⬇️ 4
- **Estado inicial:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/MED-015.md`

---

### Task 16: MED-016 — Plan File Autonomous Field (MEDIO #16)

- **Appetite:** max 0.5d
- **Esfuerzo:** 🟢 0.5d
- **Prioridad:** 🟡
- **Archivos clave:** `.opencode/task-system/prompts/plan.md`
- **Verificación real:** Campo `autonomous` referenciado en question-gates pero no en plan template
- **Gate Justificación:** Completar implementación del flag
- **Contrato:** `campaign_verify_cmd command="grep -n 'autonomous' .opencode/task-system/prompts/plan.md"` → campo en template + parsing
- **Pre-mortem:** Default `false`; parsing en `campaign_get_next_task`
- **Stop conditions:** Ninguno
- **Cynefin:** 🟦 Obvio
- **Estado inicial:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/MED-016.md`

---

### Task 17: MED-017 — Investigation Notes Formato Estándar (MEDIO #17)

- **Appetite:** max 0.5d
- **Esfuerzo:** 🟢 0.5d
- **Prioridad:** 🟡
- **Archivos clave:** `.opencode/task-system/prompts/task.md`, `.opencode/skills/campaign-executor/templates/task-definition.md`
- **Verificación real:** Investigation Notes sin estructura; debería usar formato `contract.evidencia` (claim, evidencia, confianza)
- **Gate Justificación:** Consistencia para recitation y review
- **Contrato:** `campaign_verify_cmd command="grep -A10 'Investigation Notes' .opencode/skills/campaign-executor/templates/task-definition.md"` → estructura estándar
- **Pre-mortem:** Formato muy rígido → permitir flexibilidad con campos obligatorios mínimos
- **Stop conditions:** Ninguno
- **Cynefin:** 🟦 Obvio
- **Estado inicial:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/MED-017.md`

---

### Task 18: MED-018 — SDP Cache (MEDIO #18)

- **Appetite:** max 0.5d
- **Esfuerzo:** 🟢 0.5d
- **Prioridad:** 🟡
- **Archivos clave:** `.opencode/task-system/mcp/campaign-server.mjs`
- **Verificación real:** Cada `campaign_load_skills` + SDP discovery recalcula; memoization por sesión reduce I/O
- **Gate Justificación:** Performance — skill loading repetido en sub-agentes
- **Contrato:** In-memory cache `Map<cacheKey, {skills, timestamp}>` con TTL 1 sesión (campaignId)
- **Pre-mortem:** Cache invalidation si skills cambian → TTL corto + campaignId key
- **Stop conditions:** Ninguno
- **Cynefin:** 🟦 Obvio
- **Estado inicial:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/MED-018.md`

---

### Task 19: MED-019 — SARL Trace Visualization (MEDIO #19)

- **Appetite:** max 3d
- **Esfuerzo:** 🟡 2-3d
- **Prioridad:** 🟡
- **Archivos clave:** `.opencode/task-system/enforcement/session-tracking.ps1`, nuevo web UI
- **Verificación real:** SARL trace se registra en JSONL (`traces/<campaign-id>.jsonl`) pero no hay visualización
- **Gate Justificación:** Observabilidad de recovery ladder para debugging
- **Contrato:** Dashboard simple (HTML + JS) que lee `traces/*.jsonl` y muestra timeline por taskId con rungs/outcomes
- **Pre-mortem:** Scope creep → MVP: solo timeline + filtros; sin edición
- **Stop conditions:** >3d → posponer, logs JSONL son suficientes por ahora
- **Risk Register:**
  | Prob×Impacto | Riesgo | Respuesta | Trigger / Due |
  |--------------|--------|-----------|---------------|
  | 🟢×🟢 | Low ROI vs effort | Posponer a v2 | Decision review |
- **Cynefin:** 🟨 Complicado — frontend + parsing JSONL
- **Top 3 riesgos:**
  1. Effort alto para valor marginal (logs son queryables)
  2. Maintenance burden de dashboard separado
  3. Security: exposición de traces
- **Uphill/Downhill:** ⬆️ 1 · ⬇️ 2
- **Estado inicial:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/MED-019.md`

---

### Task 20: MED-020 — MoM Ladder Model Validation (MEDIO #20)

- **Appetite:** max 1d
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟡
- **Archivos clave:** `.opencode/task-system/mcp/campaign-server.mjs` (tool `campaign_mom_escalate`)
- **Verificación real:** `campaign_mom_escalate` escala modelo pero no valida permisos/skills del nuevo modelo
- **Gate Justificación:** Modelo escalado sin permisos → sub-agente falla silenciosamente
- **Contrato:** `campaign_mom_escalate` extendido: recibe `subagent_type` → valida que nuevo modelo tiene permissions para ese agente
- **Pre-mortem:** Model traits config en `config/model-traits.mjs` → usar esa fuente
- **Stop conditions:** >1d → solo logging warning si validación falla
- **Cynefin:** 🟨 Complicado — integración con model traits
- **Estado inicial:** ⬜ PENDING
- **Task file:** `.opencode/skills/campaign-executor/tasks/MED-020.md`

---

## Dependencias entre Tasks

```mermaid
CORE-001 → CORE-002 → CORE-003
    ↓
CORE-005 (SDP tool) → HIGH-007, HIGH-010, HIGH-012, MED-015, MED-018
    ↓
HIGH-006, HIGH-008, HIGH-009, HIGH-013, HIGH-014, MED-016, MED-017
    ↓
HIGH-011, MED-019, MED-020
```

**Orden de ejecución secuencial estricta:** CORE-001 → CORE-002 → CORE-003 → CORE-004 → CORE-005 → HIGH-006 → HIGH-007 → HIGH-008 → HIGH-009 → HIGH-010 → HIGH-011 → HIGH-012 → HIGH-013 → HIGH-014 → MED-015 → MED-016 → MED-017 → MED-018 → MED-019 → MED-020

---

## Al Finalizar

Testing final: `/pipeline task <ID-real-del-backlog>` → verificar:
- [ ] Scope enforcement bloquea edits fuera de blast radius
- [ ] `validate_output` corre en ACT
- [ ] Question Gates: sub-agente devuelve BLOQUEO → orquestador pregunta → RESUME funciona
- [ ] Task file generado con template completo
- [ ] SDP unificado vía MCP tool
- [ ] `detect_changes` en plan Paso 0
- [ ] Skills re-validadas tras Discovery
- [ ] Autonomous flag en plan
- [ ] Session cleanup al cerrar
- [ ] Context reconstruction tool funciona
- [ ] Fork/Join en EJECUCIÓN (opcional)
- [ ] Auto-transition en verify
- [ ] classifyBashWrite en RESEARCH
- [ ] Index health check al inicio
- [ ] SDP cache activo
- [ ] SARL visualization (opcional)
- [ ] MoM model validation

Retrospectiva obligatoria (Start/Stop/Continue + 1 acción medible) → `skill progreso` → archivar plan.
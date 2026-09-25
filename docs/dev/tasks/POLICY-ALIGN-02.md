# POLICY-ALIGN-02: Alineación políticas pipeline (D2+D5+D3+D4+D12)

## Metadata
- **Plan file:** (ejecución directa — sin plan file; invocación `/pipeline task` con contrato inline)
- **Creado:** 2026-09-05
- **last-synced:** 2026-09-05
- **Estado:** ✅ COMPLETED
- **Tipo:** docs/policy-alignment (multi-archivo, ~100 líneas, 0 código Rust)
- **SDP:** campaign-executor, progreso, ponytail, incremental-implementation, test-driven-development, context-engineering, source-driven-development, doubt-driven-development + api-and-interface-design, documentation-and-adrs (arch-mandato)
- **Ruta:** vanta-arch

## Blast Radius
| Dirección | Hallazgo |
|---|---|
| Archivos clave (7) | `.opencode/AGENTS.md`, `.opencode/commands/pipeline.md`, `.opencode/task-system/RULES.md`, `.opencode/skills/campaign-executor/SKILL.md`, `.opencode/task-system/prompts/iter-loop-tools.md`, `.opencode/task-system/prompts/pipeline-run.md`, `.opencode/task-system/prompts/question-gates.md` |
| Referencia | `campaign-server.mjs:216-222` (BUDGET_LIMITS 10/15/40/5/120 — fuente única, solo lectura) |
| Callers | `pipeline.md` → `pipeline-run.md` → `pipeline-full.md` → `iter-loop-tools.md` + `question-gates.md`; `AGENTS.md` → path resolution + anti-proliferación; `RULES.md`/`SKILL.md` = north star + budgets |
| Callees | Ningún código Rust; solo prosa normativa. Sin hot path, sin WAL, sin FFI |
| Implicaciones | Contratos de proceso (no API pública). Hyrum: orden iteración/docs no garantizado; `VantaError` no tocado. Backward compat: aditivo (defaults aclarados, umbrales unificados a Gate V menos punitivo que FAILED directo) |
| Riesgo | bajo |

## Contrato
`rg` muestra 0 contradicciones: (a) AGENTS.md sin "único esquema es FIND-*" (multi-prefijo oficial), (b) pipeline.md FAIL_MODE default parallel alineado a pipeline-run.md:23, (c) iter-loop-tools.md sin "máx 5 iteraciones" (pointer BUDGET_LIMITS 10/15/40/5/120), (d) stagnation único 2-fallas→Gate V en RULES/SKILL/iter-loop/pipeline-full, (e) non-stop defaults documentados (D crítico pregunta, V/C auto+log). Verify: `rg` + `git -C .opencode diff --stat` (~100 líneas, 7 archivos). NO commit.

## Spec
`sin decisiones técnicas` + lista de archivos tocados (tarea 100% docs/markdown — excepción canónica question-gates.md §"Contenido válido de ## Spec"). Decisiones de política vienen cerradas en el contrato (D2+D5+D3+D4+D12); no hay alternativas abiertas que requieran `question`.

## Herramientas
- read, edit, bash (rg, git diff --stat), campaign_discover_skills, campaign_detect_task_type

## Impacto mapeado (Regla 0)
- **Archivos leídos completos:** AGENTS.md:68-71, pipeline.md:172-179, pipeline-run.md:22-27+225-243, iter-loop-tools.md:193-211+230-236+396-404, RULES.md:48-58+186-190+252-259, SKILL.md:238-252+410-418, question-gates.md:98-134, campaign-server.mjs:216-222, findings.md (solo lectura, fuera de scope)
- **Referencias hacia dentro (qué importa cada archivo):** AGENTS.md anti-proliferación ← findings.md ESQUEMA ÚNICO (hallazgos, no todos los tickets); pipeline.md FAIL_MODE ← pipeline-run.md:23 (canónico parallel); iter-loop-tools Budget/Stagnation ← BUDGET_LIMITS server + Gate V; RULES/SKILL tablas ← mismo server + Gate V
- **Referencias entrantes (quién depende):** `/pipeline plan|task|run`, `/loop-goal`, SARL, question-gates (4 gates), plan files con IDs multi-prefijo (FIND-/GOV-/STABLE-/BND-/FUT-/MEM-/MCP-/POLICY- en uso real)
- **Veredicto:** edits prosa-only, disjuntos por archivo (sin solapamiento), reversibles (`git -C .opencode checkout -- <file>`), sin ciclos de módulos (docs, no Rust), sin feature gates. Proceder por steps D2→D5→D3→D4→D12.

## Steps
### Step 1: D2 — AGENTS.md multi-prefijo oficial
- **Archivos:** `.opencode/AGENTS.md:68-71`
- **Acción:** derogar "único esquema FIND-*" → multi-prefijo oficial por dominio (FIND-* hallazgos; POLICY-/GOV-/STABLE-/BND-/FUT-/MEM-/MCP-/* según taxonomía Backlog/plan)
- **Verify:** `rg "único esquema es" .opencode/AGENTS.md` → 0 hits
- **Estado:** ⬜ PENDING

### Step 2: D5 — pipeline.md default parallel
- **Archivos:** `.opencode/commands/pipeline.md:175-179`
- **Acción:** fijar default parallel alineado a pipeline-run.md:23 (+ variante stop explícita)
- **Verify:** `rg "FAIL_MODE" .opencode/commands/pipeline.md` muestra default parallel
- **Estado:** ⬜ PENDING

### Step 3: D3 — iter-loop-tools pointer BUDGET_LIMITS
- **Archivos:** `.opencode/task-system/prompts/iter-loop-tools.md:236,402`
- **Acción:** borrar "máx 5 iteraciones" → pointer `BUDGET_LIMITS (campaign-server.mjs:216-222: 10/15/40/5/120)`
- **Verify:** `rg "5 iteraciones" .opencode/task-system/prompts/iter-loop-tools.md` → 0 hits
- **Estado:** ⬜ PENDING

### Step 4: D4 — stagnation único 2-fallas Gate V
- **Archivos:** `RULES.md:52-57,186-190,259`, `SKILL.md:249,252,417`, `iter-loop-tools.md:204-211,400-401`, `pipeline-full.md:276`
- **Acción:** unificar todo a "2 fallas mismo-error → Gate V (question-gates.md); sin respuesta → STOP"
- **Verify:** `rg "3 vueltas|3 mismo error|5 sin cambiar|5\+ iteraciones|3 intentos sin" .opencode/` → 0 hits en los 4 archivos
- **Estado:** ⬜ PENDING

### Step 5: D12 — non-stop defaults en pipeline-run + question-gates
- **Archivos:** `.opencode/task-system/prompts/pipeline-run.md`, `.opencode/task-system/prompts/question-gates.md`
- **Acción:** documentar run usa defaults seguros + log; solo D crítico pregunta; V pregunta, C auto+log
- **Verify:** `rg "Non-stop|non-stop" .opencode/task-system/prompts/pipeline-run.md .opencode/task-system/prompts/question-gates.md` → ≥2 hits
- **Estado:** ⬜ PENDING

### Step 6: Verify contrato + diff --stat (sin commit)
- **Acción:** `rg` 5 checks (a-e) + `git -C .opencode diff --stat`
- **Verify:** 0 contradicciones, ~100 líneas, 7 archivos
- **Estado:** ⬜ PENDING

## Notas
- findings.md queda fuera de scope (documenta FIND-* para hallazgos; no contradice multi-prefijo de tickets generales tras D2).
- pipeline-run.md umbrales "3 consecutivas" a nivel campaña (no verify mismo-error) se conservan — no son stagnation verify-level.

## Context Save Point
- **Fecha:** 2026-09-05
- **Branch:** develop (.opencode submodule detached/branch propio)
- **CI pendiente:** no (docs-only)
- **Decisiones:** unificar a Gate V (menos punitivo que FAILED directo); BUDGET_LIMITS como fuente única; multi-prefijo refleja uso real (plan 2026-09-04)
- **Próxima tarea:** ninguna (ejecución directa)

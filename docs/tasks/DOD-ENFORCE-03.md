# DOD-ENFORCE-03: Unificar DoD + deprecar skills v1 + 3 checks SPEC en runtime

## Metadata

- **Plan file:** docs/plans/2026-09-04-durability-release-readiness.md (tarea ad-hoc del orquestador, sin fila en plan)
- **Fuente:** orquestador (D7+D13+D14)
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟡
- **Tipo:** Mixto (Docs + JS task-system)
- **Turns estimados:** 10
- **Creado:** 2026-09-05
- **last-synced:** 2026-09-05
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0
- **Pendientes (downhill):** 0 steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | campaign-server.mjs importa state-tools.mjs (validateAction, getAllowedTools); tests importan detectType/consumeBudget/findInProgressTasks/findPlanFile + parsers.mjs |
| Callees | state-tools.mjs no depende de nadie; campaign-server.mjs depende de state-tools, parsers, tracer, model-traits |
| Implicaciones | validateAction(state,tool) firma intacta + 3er param opcional → sin break; STATE_TOOLS suma campos opcionales en ACT → parity-check solo chequea keys (safe); deprecación v1 solo añade flag en respuesta (aditiva); DoD.md solo docs |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** .opencode/references/definition-of-done.md (108L) ✅ · .opencode/task-system/RULES.md (471L) ✅ · .opencode/agents/vanta-lead.md (546L) ✅ · .opencode/task-system/config/state-tools.mjs (95L) ✅ · .opencode/task-system/enforcement/pre-call-checks.md (221L) ✅ · .opencode/skills/progreso/SKILL.md (218L) ✅ · .opencode/task-system/config/parity-check.mjs (39L) ✅ · tests hardening/parsers/state-persistence ✅ · campaign-server.mjs: mapa estructural completo vía grep (exports/tools/funciones, L26-2085) + lectura íntegra de los bloques a editar (L1187-1470 tools skills, L1846-1970 enforce_state, L764-783 validate_action); resto del archivo (plan/budget/recitation/tools 1-15) no se toca
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** campaign-server.mjs → state-tools.mjs, parsers.mjs, tracer.mjs, model-traits.mjs, @modelcontextprotocol/sdk, zod; state-tools.mjs → sin dependencias
- **Archivos que referencian a los editados (referencias entrantes):** state-tools.mjs ← campaign-server.mjs:11, parity-check.mjs; campaign-server.mjs ← hardening.test.mjs, state-persistence.test.mjs (funciones no tocadas); definition-of-done.md ← 10 agent files §6/§7 + skills + prompts (solo lectura, no se les cambia nada); vanta-lead.md:443 ← pipeline.md (routing canónico vive en lead, pipeline.md lo cita)
- **Veredicto impacto:** bajo — cambios aditivos (param opcional, campos opcionales, flag deprecated, secciones docs). RULES.md NO se edita (su header lo prohíbe: north star inmutable). Task files históricos bajo tasks/ NO se tocan (registro, no spec viva).

## Contrato

`node --test .opencode/task-system/mcp/` 42/42 verde + `rg` de verificación por slice (detallado abajo) + DoD unificado contiene cada ítem RULES v1-v4 y progreso (verificado por lista).

## Spec (decisiones)

| Decisión | Opción elegida | Por qué |
|----------|---------------|---------|
| Límite EditGuard en ACT | 100 líneas | RULES.md invariante #4: ~100 líneas por commit; el check lo hace exigible |
| Límite file-scope en ACT | 5 archivos | slice atómico típico ≤5; re-edits exentos; transición resetea |
| Acumulador file-scope | Map en memoria keyed por sessionId + fallback toolArgs.filesEdited | sdpCache precedent en mismo archivo; sin I/O en hot path; reset al cambiar de estado |
| allowed_commands | generalizar a config-driven (ya existe en VERIFY) | sin duplicar listas; validateAction acepta toolArgs opcional |
| RULES.md | no editar | su propio header lo declara inmutable; DoD lo absorbe por copia + cita |
| Docs load_skills | solo spec viva (lead/research/pipeline/skills-engineering/task-system) | tasks/ son historia, no spec |

## Steps

- [x] S0 — Discovery + task file + baseline node --test 42/42
- [x] S1 — vanta-lead.md:443 ya en `campaign_discover_skills_v2`, cero `campaign_load_skills` (verificado por rg, idempotente — cambio pre-existente en worktree, contrato b ✅)
- [x] S2 — spec viva a v2: pipeline-full.md (4 hits) + iter-loop-tools.md (3) + plan.md:40 + task.md:82 → `campaign_discover_skills_v2`; server ya expone `deprecated:true` en load_skills/discover v1 (D7)
- [x] S3 — 3 checks SPEC → runtime: STATE_TOOLS.ACT `max_edit_lines:100` + `max_files_per_state:5`, `validateAction(state,tool,toolArgs?)`, C0_CHECK_CONFIG ACT espejo, `estimateEditLines`/`checkEditGuard`/`checkFileScope` (Map por sessionId + fallback filesEdited, re-edits exentos) cableados como bloques en `campaign_enforce_state`; header pre-call-checks.md a 6 runtime (D14, contrato c)
- [x] S4 — definition-of-done.md +35L: Capa determinista (0-5 + Rust safety) + Thresholds v1-v4 (ratchet) + Progreso Trigger 1.A-G, todo con cita a fuente (D13, contrato a; RULES.md intacto)
- [x] S5 — Verify final: node --test 42/42 + parity-check 10/10 + rg listas + prueba funcional 3 bloques (NO commit por instrucción del orquestador)

## SDP

SDP: test-driven-development (RED/GREEN + stack node --test) + incremental-implementation (slices S1-S5, ~100L, compilable) + context-engineering (Rules→Spec→Source→Error) + base-only (campaign-executor/progreso/ponytail auto) — discovery MCP v2 no disponible en runtime (regex), SDP manual justificado

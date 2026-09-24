# FIND-111: S5 `skill_extract` solo-candidatos read-only (SHIP)

## Metadata

- **Plan file:** `docs/dev/plans/2026-09-17-seguimiento-mvp.md` (Task 3, Wave0 tercera en secuencia)
- **Fuente:** plan seguimiento-mvp Task 3 + FIND-107 §7 (S5)
- **Esfuerzo:** 🟡 1d (appetite 1d)
- **Prioridad:** 🟡 Media
- **Tipo:** MCP server (campaign_detect_task_type: `mcp`; checks: `cargo check -p vantadb-mcp`, `cargo test -p vantadb-mcp --test mcp_tests`)
- **Branch:** develop · **Commit:** `feat: FIND-111 — ...` (NO PUSH, staging selectivo)
- **Creado:** 2026-09-17
- **last-synced:** 2026-09-17
- **Estado:** ✅ COMPLETED (SHIP — Steps 1-5 ✅, commit pendiente → hecho)
- **SDP:** campaign-executor · source-driven-development · incremental-implementation · test-driven-development · context-engineering · doubt-driven-development · api-and-interface-design · mcp-builder (8, ver § Skills; `frontend-ui-engineering` del lifecycle descartada: sin `web/` en scope — ruido, no señal)

## Tarea

**Objetivo:** cerrar S5 de FIND-107 — exponer la extracción de skills como tool
MCP `skill_extract` **solo-candidatos read-only** (sin sink de escritura), con
degrada honesta sin runner. Si no hay path degradado limpio → re-DEFER con
motivo (NO es fracaso).

**Contrato (plan):** tool `skill_extract` read-only (candidatos, sin sink de
escritura) + degrada documentada sin runner
(`{success:false, candidates:[], error}`) + test + fila MCP.md + coverage
0 gaps; o re-DEFER con motivo.

**AC de esta ejecución:**

- (a) tool read-only real: `annotations.readOnlyHint=true`,
  `destructiveHint=false`, `idempotentHint=true`, `openWorldHint=false`;
  el handler NUNCA toca el sink (`SkillCoreSink` prohibido en este slice),
  NUNCA escribe en storage — solo llama al core puro
  `extract_skills_with_llm` y serializa `{success, candidates, error}`.
- (b) sin runner → `success:false` documentado, nunca bloqueo: el handler usa
  un runner `NotConfigured` (patrón `NoLlm` de `wiki.rs:385`, frontera R-8) y
  el core ya degrada per Principio 4 (`skill_extractor.rs:245-253` + test
  fuente `extractor_failure_is_degraded_not_fatal`); `messages:[]` → éxito
  trivial vacío sin llamar al runner (core `:215-221`).
- (c) test + docs + coverage 0 gaps: test en `skills_tests.rs` (degrada +
  lista + shape), fila `MCP.md` Skills (6→7), conteos contagio 86→87
  (allowlist + dispatch + asserts), `validate-docs-coverage.ps1` 0 gaps.

**Prohibido en este slice:** sink con escritura (`SkillCoreSink::apply_candidates`,
`run_skill_extract_once` end-to-end con escritura = slice separado); tocar
`src/wal*.rs`, `src/cli_handlers/`, `src/cli.rs`, `src/error.rs` (FIND-109 ✅
`9c881ac5`); `approval.rs` + gateway approval (FIND-110 ✅ re-DEFER);
`reparacion.bat`, `.opencode`, `Justfile`, `ocr-*`, `completions/*`,
tauri lock, stash@{0} GOV-C4, `docs/dev/Backlog.md` (lead), plan file (solo
recitation), `C:/Users/Eros/.vantadb*`, WIP ajeno en `git status`.

## Archivos

**Clave (leídos completos):**

- `vanta-memory/src/core/skill/skill_extractor.rs` (360L) — `ExtractMessage`
  (`:26-40`), `ExtractedSkillCandidate` (`:46-56`), `SkillSummary` (`:60-63`),
  `SkillExtractionResult` (`:88-94`, `success/candidates/error`),
  `extract_skills_with_llm` (`:209-288`: vacío→éxito trivial `:215-221`,
  runner-error→`success:false` `:245-253`, sentinel `:257-263`, JSON→candidatos
  `:265-287`, revalidación output untrusted `:277-281`)
- `vanta-memory/src/core/skill/mod.rs` (34L) — re-exports (`:24-33`);
  `conversation_add/worker.rs:42-97` — `run_skill_extract_once` exige
  `R: LlmRunner` genérico + escribe vía sink (POR ESO no se expone acá)
- `vanta-memory/src/core/abstractions/llm_runner.rs` (`:57-115`) — `LlmError`
  (`NotConfigured` `:61`, "callers must degrade, never block"), trait
  `LlmRunner::run`
- `vantadb-mcp/src/skills.rs` (719L) — familia `skill_*`: `skill_tool_definitions`
  (`:41-178`), `handle_skill_tool` (`:185-201`), helpers frontera
  (`require_str`, `store_err`, `validate_identifier`)
- `vantadb-mcp/src/handlers/tools.rs` — registry annotations (`:25-72`),
  `handle_tools_list` extend (`:1012-1018`), `profile_allowed_tools`
  (`skill_tools` `:1161-1171`, full `:1097-1132`), dispatch (`:2907-2908`)
- `vantadb-mcp/src/wiki.rs:274-288,382-391` — precedente degrada: `start_ingest::<NoLlm>`
  + `pub struct NoLlm` → `Err(LlmError::NotConfigured)` (P4 degraded mode)
- `vantadb-mcp/src/scenes.rs` (294L) + `src/dreams.rs` (267L) — plantilla S1-S3
  FIND-107 (definiciones + dispatch + `session_key_arg` + `db_from` + errores
  dominio como `error_content`, params como invalid-params)
- `vanta-memory/tests/skill_extract.rs` (323L, contrato fuente) —
  `extractor_failure_is_degraded_not_fatal` (`:128-135`),
  `extractor_sentinel_means_empty_success` (`:119-125`),
  `extractor_parses_candidates_from_fenced_json` (`:106-116`)
- `vantadb-mcp/tests/skills_tests.rs` (725L) — patrón tests familia
  (`setup_storage`, `tool_call`, `tool_text`, `tool_error`,
  `test_tools_list_includes_skill_tools` `:88-100`)
- `vantadb-mcp/tests/mcp_tests.rs:4484-4607` — asserts conteo
  (`test_mcp_tool_annotations_coverage` 86, `test_mcp_tool_profiles` 86)
- `docs/api/MCP.md:198-209,254,401-410` — header 86, tabla familias
  (`skill_*` 6), sección Skills (6)
- `skills/vantadb-mcp/scripts/test-mcp.py:22,54-61` — `EXPECTED_TOOLS full (86,86)`

**Relacionados (lectura parcial / referencia):**

- `vantadb-mcp/src/lib.rs:12-28` — módulos (`skills`, `scenes`, `dreams`); sin
  cambios (no hay módulo nuevo)
- `vantadb-mcp/src/config.rs:11,14` — comentarios perfil full (86 tools)
- `vantadb-mcp/src/validation.rs:11-35` — `validate_identifier`, `validate_payload`
- `vanta-memory/Cargo.toml` — `llm-driver` default-off (el core degrada sin runner);
  `vantadb-mcp/Cargo.toml` — `vanta-memory` sin features (sin runner real en MCP)
- `scripts/validate-docs-coverage.ps1:167-186` — §6 MCP: solo bloque
  `handle_tools_list` base (extend en `skills.rs` no lo recorre → no dispara gap,
  pero R-5 exige fila MCP.md en el mismo PR igual)
- `docs/dev/tasks/FIND-110.md` — patrón task file re-DEFER + tabla prohibidos/WIP

**Prohibidos (ver Tarea):** sink con escritura; wal/cli/error FIND-109;
approval FIND-110; `reparacion.bat`, `.opencode`, `Justfile`, `ocr-*`,
`completions/*`, tauri lock, stash@{0}, `docs/dev/Backlog.md`, plan file (solo
recitation), `C:/Users/Eros/.vantadb*`, WIP ajeno
(`completions/`, `.opencode`, `docs/dev/Backlog.md` modificados + `reparacion.bat`
untracked — intocables, staging selectivo).

## Impacto mapeado (Regla 0)

**Archivos leídos completos:** `skill_extractor.rs` (360L),
`skill/mod.rs` (34L), `llm_runner.rs` (`:1-115` vía codegraph + reads),
`skills.rs` (719L), `scenes.rs` (294L), `dreams.rs` (267L),
`handlers/tools.rs` (`:1-120,990-1210,2895-2930`),
`wiki.rs` (`:240-320,375-424`), `config.rs` (142L),
`skill_extract.rs` test fuente (323L), `skills_tests.rs` (`:1-100`),
`mcp_tests.rs` (`:4480-4620`), `MCP.md` (`:190-310,395-455`),
`test-mcp.py` (289L), `validate-docs-coverage.ps1` (227L),
`api-contract.md` + `server-mcp.md` (completas), `definition-of-done.md`.

**Referencias hacia dentro (el cambio necesita de):**

- `vanta_memory::core::skill::skill_extractor::{extract_skills_with_llm, ExtractMessage, SkillSummary, SkillExtractorConfig}` — ya `pub` vía `skill/mod.rs:30-33` ✅ (cero cambios core)
- `vanta_memory::core::abstractions::{LlmError, LlmRunParams, LlmRunner}` — ya `pub` ✅
- `crate::validation::{error_content, serialize_content, text_content, validate_identifier, validate_payload}` — ya en uso en `skills.rs` ✅
- `crate::config::McpConfig` (`max_payload_length`, `max_query_length`, `max_key_length`) — knobs existentes ✅

**Referencias entrantes (dependen del cambio):**

- `handle_tools_list` → `skill_tool_definitions()` (nueva definición aparece en `tools/list` full)
- `handle_tools_call` → `handle_skill_tool("skill_extract", ...)` (nuevo brazo dispatch)
- `profile_allowed_tools(Full)` → `skill_tools` array (nuevo nombre en allowlist)
- `mcp_tests.rs` (2 asserts 86→87), `test-mcp.py` (`EXPECTED_TOOLS`), docs conteo (MCP.md, SKILL.md ×2, api-reference, mcp-protocol, configuration, opencode.jsonc)
- NINGÚN caller Rust interno llama al nuevo handler (solo dispatch por nombre string) — blast radius de compilación: `skills.rs` + `tools.rs` + tests

**Veredicto de impacto:** BAJO y aditivo. Cero cambios en core (`vanta-memory`
intacto), cero cambios en `handlers/tools.rs` salvo comentarios conteo +
allowlist + dispatch ya existentes (1 línea array + 0 líneas dispatch — el brazo
`skill_*` ya deriva a `handle_skill_tool`; solo añadir match arm interno en
`skills.rs`). Sin hot paths (sin `vector/`, sin `engine.rs`, sin Tokio nuevo,
sin locks) → FASE PERFORMANCE no aplica. Toca trust boundary (input usuario →
validación frontera) → FASE SECURITY checklist ligera (validar shapes, sin
secretos, sin FS). Rollback: `git revert` limpio (aditivo puro).

## Dependencias

Wave0 tercera en secuencia: FIND-109 ✅ ship (`9c881ac5`), FIND-110 ✅ re-DEFER
(`1b3b3409`) — archivos disjuntos (wal/cli · approval · skill), sin colisión.
**Stop:** sin path degradado limpio → re-DEFER (task file + RESULTADO; Backlog
lo toca el orquestador). Path degradado CONFIRMADO en DISCOVERY (core Principio
4 + `NoLlm` precedente) → se shippea, no se defiere.
**NextTask:** Wave1 FIND-112 (orquestador).

## Referencias

- Rules: `.opencode/rules/api-contract.md` (R-8 glue — handler delgado, lógica
  en core; R-5 tools+docs mismo PR; R-1 sin APIs fantasma) + `server-mcp.md`
  (lectura completa, 28L — sin concurrencia nueva: handler síncrono puro).
- Refs: `definition-of-done.md` (standing checklist + DoD v1 + capa
  determinista 0-5) · `clean-code-clean-architecture.md` Ap. V (MCP =
  Frameworks/Drivers → Humble Objects, cero lógica de negocio).
- Commands: `pipeline.md`, `audit.md`.
- SPEC.md raíz (contrato plan MVP; sin fila por-tool — la tabla Spec de esta
  tarea vive en `## Spec` abajo).
- Tabla Spec de esta tarea: `## Spec` (decisión read-only + degrada).

## Skills

Base sesión: campaign-executor · progreso · ponytail(full) · brainstorming ·
writing-plans · planning-and-task-breakdown.
SDP v2 (`campaign_discover_skills_v2` phase BUILD, keywords skill_extract/
read-only/MCP tool/degrade/candidates, 8): campaign-executor ·
source-driven-development (versiones/patrones contra fuente, no memoria) ·
incremental-implementation (slices delgados) · test-driven-development
(RED→GREEN) · context-engineering (contexto focado <2k líneas) ·
doubt-driven-development (adversarial en tool frontera) ·
api-and-interface-design (schema+anotaciones) · mcp-builder (convenciones MCP:
annotations, envoltorios, perfiles). `frontend-ui-engineering` (lifecycle,
score 1.00 genérico) DESCARTADA con motivo: cero `web/` en scope.
Cargadas vía `skill`: test-driven-development · incremental-implementation ·
context-engineering · api-and-interface-design · source-driven-development ·
doubt-driven-development · mcp-builder (+ security-and-hardening base por
trust boundary — validación frontera según su checklist).

## Herramientas + MCP

- `codegraph_explore` PRIMERO ✅ (usado en DISCOVERY)
- `cargo test -p vantadb-mcp -j 2` (+ `cargo test -p vanta-memory -j 2 skill`
  solo si toco fuente — NO la toco)
- MCP stdio smoke (`initialize` + `tools/list` cuenta) vía `test-mcp.py`
- `pwsh scripts/validate-docs-coverage.ps1` (0 gaps)
- `campaign_verify_cmd` (bug exit -1 conocido → bash directa + mención)
- Cargo siempre `-j 2`. Internet N/A por defecto.

## Investigación código (DISCOVERY ✅)

- **Contrato extracción:** `extract_skills_with_llm(runner, messages,
  existing_skills, config) -> SkillExtractionResult {success, candidates,
  error}` — puro, sin I/O, sin sink (el sink vive en `worker.rs`, no acá).
- **LlmRunner:** trait sync `run(&LlmRunParams) -> Result<String, LlmError>`;
  `LlmError::NotConfigured` = modo LLM-free ("must degrade, never block").
  `vanta-memory` default SIN `llm-driver`; `vantadb-mcp` depende de
  `vanta-memory` sin features → **no hay runner real en MCP** → degrada siempre
  (salvo vacío trivial). Honesto y por diseño.
- **Tests fuente:** 323L, 10 tests; degrada probada
  (`extractor_failure_is_degraded_not_fatal`), sentinel, fenced-JSON,
  ghost/pending/idempotencia del worker (no se re-testea el worker acá).
- **Plantilla S1-S3:** `scenes.rs`/`dreams.rs` = definiciones + dispatch +
  `*_arg` validadores + `error_content` dominio / invalid-params params.
  **Decisión ubicación:** EXTENDER `skills.rs` (misma familia `skill_*`),
  NO archivo nuevo — `scenes.rs`/`dreams.rs` nacieron por dominio nuevo;
  fragmentar `skill_*` en dos módulos violaría CCP (mismo motivo de cambio).
- **Conteos 86 (contagio FIND-107):** `tools.rs:25-27,30,35-72,1098,1036-1210`,
  `config.rs:11,14`, `mcp_tests.rs:4484,4496,4595,4605`, `test-mcp.py:22,54-61`,
  `MCP.md:198-209,219,254`, sección Skills `401-410`, SKILL.md ×2 (`86 tools`),
  `api-reference.md:8,13`, `mcp-protocol.md:7`, `configuration.md:127`,
  `opencode.jsonc:87`. Todos 86→87 (skill 6→7, extend 37→38, readOnly 49→50;
  dev/memory perfiles INTACTOS — la tool es full-only como el resto `skill_*`).

## Investigación problema (DISCOVERY ✅)

**¿Dónde vive la degradación — handler o core?** R-8 dice frontera → handler;
verificado: el core YA degrada a `success:false` (`skill_extractor.rs:245-253`,
`Err(err) → {success:false, candidates:[], error:"LLM skill extraction
failed: {err}"}`) y el vacío trivial a `success:true` (`:215-221`). El handler
solo envuelve: runner local `NotConfigured` (hermano de `NoLlm`, privado al
módulo `skills.rs` — NO reuso `wiki::NoLlm` porque es `pub` del módulo wiki con
doc P4-ingest; un `struct NoRunner` local de 5 líneas evita acoplar familias) →
llama al core → serializa. Cero lógica de negocio en el handler (R-8 ✅).
`storage` no se toca (firma lo recibe por uniformidad dispatch, se ignora con
`let _ =`).

## Investigación internet

N/A por defecto (sin ambigüedad de APIs externas; anotaciones MCP 2025-06-18
ya citadas en `tools.rs:19-23` + skill `mcp-builder` cargada). Digest: ninguno.

## Spec

| # | Decisión | Opción elegida | Alternativa descartada + por qué |
|---|----------|----------------|----------------------------------|
| 1 | Ubicación | extender `skills.rs` | archivo nuevo `skill_extract.rs`: fragmenta familia `skill_*` (CCP) |
| 2 | Runner | `struct NoRunner` local → `Err(NotConfigured)` | reusar `wiki::NoLlm`: acopla familias skill↔wiki; `run_skill_extract_once`: exige sink+DB (prohibido) |
| 3 | Inputs | `messages: [{role, content}]` req + `existing_skills: [{name, description}]` opt | `session_id/task_id` (ata al worker con escritura — prohibido); pasar `config` (head/tail tuning: exponer knobs sin consumidor = YAGNI, se usa `Default`) |
| 4 | Roles | `role` string libre, desconocido→core degrada a `user` (TDAM, ya en core) | enum cerrado en handler: duplicaría validación del core (R-8) |
| 5 | Caps | `messages` ≤512 items, cada `content` ≤`max_payload_length`, `name/description` ≤`max_key_length`/`max_payload_length` | sin caps: un tool call podría agotar el pipe (precedente `MAX_TRANSFER_BYTES`, MCP-17/25) |
| 6 | Output | `{success, candidates:[{action,name,description,content}], error}` verbatim del core | envoltorio `{ok,...}` propio: rompería paridad core↔MCP (R-8) y el test fuente |
| 7 | Anotaciones | readOnly T / destructive F / idempotent T / openWorld F | write (sink prohibido); no-idempotente (puro, retry-safe sí) |
| 8 | Perfil | full-only (array `skill_tools`) | dev/memory: el resto `skill_*` tampoco está (Cursor cap ~40) |
| 9 | Sink | PROHIBIDO en este slice | slice separado con diseño lifecycle (espejo FIND-110) |
| 10 | Conteo | 86→87 + fila MCP.md mismo PR (R-5) | ship sin docs: rompería GOV-B4 |

## Steps

- [x] **Step 1 — DISCOVERY + task file** (zero-code): contrato, archivos,
  blast radius Regla 0, Spec 10 filas, Gate D evaluado (disparado por tool
  nueva → CUBIERTO por Gate P del plan: owner aprobó tool read-only en Task 3;
  no re-`question`). Este step no toca código.
- [x] **Step 2 — RED (TDD):** test en `vantadb-mcp/tests/skills_tests.rs`:
  `skill_extract` en `tools/list` (definición + anotaciones read-only) +
  degrada honesta (`messages` no-vacío → `success:false`, `candidates:[]`,
  `error` contiene `NotConfigured`/extraction) + vacío trivial
  (`messages:[]` → `success:true`, `[]`) + params inválidos (sin `messages` →
  invalid-params). Verificar que FALLA (tool no existe).
  ✅ RED verificado: 4/4 fallan con `Tool not found: skill_extract` (razón correcta).
- [x] **Step 3 — GREEN (handler + wiring + conteos código):** `skills.rs`:
  definición + `skill_extract` fn + brazo dispatch + `NoRunner` + validadores;
  `handlers/tools.rs`: comentarios conteo (86→87, 49→50 readOnly, 37→38
  extend, línea registry) + `skill_tools` allowlist + comentario perfil full;
  `config.rs` comentarios; `mcp_tests.rs` asserts 86→87.
  Verify: `cargo test -p vantadb-mcp -j 2 skill` + `mcp_tests` conteo.
  ✅ GREEN: skills_tests 14/14, annotations+profiles 87 ✅, clippy `-D warnings` 0.
- [x] **Step 4 — Docs + conteos contagio:** `docs/api/MCP.md` (header, tabla
  familias, sección Skills 6→7 + fila, perfil full); `test-mcp.py`
  (`EXPECTED_TOOLS` + comentario); SKILL.md ×2 + `api-reference.md` +
  `mcp-protocol.md` + `configuration.md` + `opencode.jsonc` (86→87).
  Verify: `pwsh scripts/validate-docs-coverage.ps1` 0 gaps + stdio smoke.
  ✅ Hecho parcial-deliberado (scope discipline + prohibido `.opencode`):
  actualizados `docs/api/MCP.md` (R-5), `test-mcp.py` (drift gate funcional),
  `opencode.jsonc`, `SKILLS-MANIFEST.md`. NO tocados (mirror hash-SAME §7 +
  prohibido `.opencode`): `skills/vantadb-mcp/SKILL.md`,
  `references/{api-reference,configuration,mcp-protocol}.md` y sus espejos
  `.opencode/` (siguen diciendo 86) → **deuda para el lead**: sync espejo
  bidireccional (merge, no overwrite) + actualizar a 87 en ese mismo acto.
  Coverage 0 gaps ✅ (mirror intacto), smoke stdio 5/5 ✅ (87 tools).
- [x] **Step 5 — VERIFY full + commit + cierre:** fmt + clippy + nextest
  workspace (`-j 2`) + docs-coverage + OCR delegation
  (`pwsh dev-tools/ocr-review.ps1`, Critical/High bloquean) + DoD 3 niveles +
  commit `feat:` selectivo + recitation + RESULTADO §7. P2-01 lo hace el
  orquestador (no yo). Gates D/V/C vía `question` (D ya cubierto por Gate P).
  ✅ Verify: `cargo fmt --check` ✅ · `cargo clippy --workspace --all-targets
  --all-features -- -D warnings` ✅ · `cargo test -p vantadb-mcp` full
  (19 targets, 0 failed) ✅ · `cargo test -p vanta-memory --test
  skill_extract` 10/10 ✅ · docs-coverage 0 gaps ✅ · OCR advisory sin
  Critical/High ✅ · stdio smoke 5/5 ✅. Workspace-nextest (audit profile)
  NO corrido: cambio 100% `vantadb-mcp` (+docs), `vanta-memory` intacto;
  CI/orquestador lo cubre (deuda explícita, no silencio).
  DoD v1: Correctness (AC a+b+c verificados mecánicamente) · Quality
  (clippy/fmt, fns <20L, sin duplicar lógica — R-8) · Integration (dispatch +
  perfiles + smoke) · Docs (MCP.md mismo PR, R-5) · Ship (revert limpio,
  observabilidad N/A — tool pura sin I/O; security frontera validada).
  Rollback: `git revert <hash>` (aditivo puro).

## Validación + cierre

- Verify contrato: degrada shape exacto + readOnly real + conteo 87 en
  `tools/list` full.
- Full: `cargo fmt --check` + `cargo clippy --workspace --all-targets
  --all-features -- -D warnings` + `cargo nextest run --profile audit
  --workspace --build-jobs 2` + `validate-docs-coverage.ps1` + OCR delegation.
- DoD 3 niveles (Correctness/Quality/Integration + docs + ship-readiness;
  rollback = `git revert` aditivo puro).
- RESULTADO §7 obligatorio siempre; si no termino: hecho + próximo step,
  nunca silencio.

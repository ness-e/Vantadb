# FIND-103 — Skills correctas + motor de recall continuo

> **Plan:** `docs/plans/2026-09-17-mvp-memoria-agentes.md` (Wave1 sola, appetite 2d, branch `develop`, commit `docs:`/`fix: FIND-103 — ...`)
> **Ruta:** vanta-worker · **Estado:** ⬜ PENDING → IN PROGRESS (DISCOVERY completo 2026-09-17)
> **Campaign:** b2ece025-e9d3-4f8b-835d-1d0143a86b66 · **NextTask:** FIND-104 + FIND-106 (las ejecuta el orquestador, Wave2)
> **SDP:** `campaign_discover_skills_v2` phase=BUILD (8 keywords) → base + lifecycle; `SKILLS_CARGADAS:` en RESULTADO §7.

## 1. TAREA

**Objetivo:** revisión profunda y corrección total de `skills/vantadb-mcp/` + diseño del motor de recall continuo (recall mid-conversación + traductor temporal + umbrales + curaduría). La skill es lo que lee el agente.

**Contrato exacto (plan):** claims verificados mecánicamente + `instructions` presente en `initialize` + diseño temporal (filtros + traductor determinista con tests) + coverage 0 gaps (`validate-docs-coverage.ps1` limpio).

**Acceptance criteria:**
- (a) todo claim de la skill verificado mecánicamente contra código (counts 86, `~`, env vars, prompts, handshake).
- (b) `instructions` presente en `initialize` (código + test + doc).
- (c) filtros temporales + traductor determinista con suite de tests (`temporal.rs` + `temporal_tests.rs`).
- (d) coverage 0 gaps (`pwsh scripts/validate-docs-coverage.ps1` exit 0, skills mirror hash-SAME).

**Alcance exacto (no crear):** las plantillas por cliente que ejecutan esta política viven en FIND-106 — NO crear `assets/*` de hooks acá.

## 2. ARCHIVOS

**Clave (con :línea):**
- `skills/vantadb-mcp/SKILL.md:8` (79→86), `:116-131` (tabla familias: scenes 3→5, falta dreams 5), `:433-438` (prompts genéricos), `:30/35/49` (paths `~`), `:568` (§ "MCP Tools (45)" stale)
- `skills/vantadb-mcp/references/api-reference.md:5-13` (79→86, scenes 3→5, sin dreams), `:116-128` (scenes 3, sin write/edit)
- `skills/vantadb-mcp/references/mcp-protocol.md:7` (60 tools stale)
- `skills/vantadb-mcp/references/configuration.md` (sin `VANTADB_MCP_PROFILE`, `VANTADB_MCP_BYTE_BUDGET`, `VANTADB_EMBEDDING_PROVIDER`, `VANTADB_LOCAL_MODEL`, `VANTADB_OPENAI_*`, `ORT_DYLIB_PATH`)
- `skills/vantadb-mcp/scripts/test-mcp.py:56-63` (solo handshake 4 checks; profiles full=86 ya ok)
- `vantadb-mcp/src/handlers/initialize.rs:29-40` (sin `instructions`)
- `vantadb-mcp/src/handlers/prompts.rs:1-101` (4 prompts genéricos, sin workflow recall-first; count 4 SE MANTIENE)
- `vantadb-mcp/src/handlers/tools.rs:26` (readOnly 49 — vigente, no tocar), `:1010-1214` (extend 37: scenes 5 + dreams 5 ya), `:1499-1561` (memory_list filters→core filter_ops), `:2773-2839` (graph_traverse time_range SOLO aristas), `:3209-3226` (search filters SOLO igualdad)
- `vantadb-mcp/src/lib.rs:12-27` (registry módulos; añadir `temporal`), `:35-36` (re-export initialize)
- `vantadb-mcp/src/config.rs:8/73/81/122-141` (env vars MCP reales), `src/config.rs:203-209/586-595/936-957` (env vars LLM/embedding reales)
- `docs/api/MCP.md:198` (79→86, 7 familias), `:212-220` (profiles), `:253` (79), `:428-438` (scenes 3→5), `:440-450` (dreams 5 — de FIND-107, solo lectura), `:498` (last sync)
- `SKILLS-MANIFEST.md:476` ("15 tools" stale), `opencode.jsonc:87` ("79 tools" stale)

**Relacionados (callers/callees):**
- `vantadb-mcp/tests/mcp_tests.rs:23-68` (initialize tests — extender), `:205-290` (prompts tests — actualizar), `:4463-4620` (86 asserts + profiles — solo lectura), `tests/test_embed_texts.rs`, `scene/dream/thread_tests.rs` (regresión)
- `scripts/validate-docs-coverage.ps1:167-219` (§6 tools base 49 + §7 mirror 10 pares)
- `vanta-proxy/src/capture.rs:15` (`TURNS_NAMESPACE="proxy-turns"`), `vanta-memory/src/core/hooks/auto_capture.rs:84-138` (hook L0), `src/sdk/serialization/mod.rs:551-574` (`matches_advanced_filters` SOLO metadata — timestamps NO filtrables por `memory_list`)
- `docs/research/archive/COGNEE_EVALUATION.md:390` (inyecta `additionalContext` en SessionStart — evidencia in-repo del patrón), `docs/tasks/complete/ECO-001.md:10-17` (OpenCode `session.created` vs `SessionStart` — input FIND-106)
- Notion `Problema` (dims 2 episódica / 4 procedimental / 5 temporal bitemporal) + `Propuesta` (§2.1 dim 5 [PARCIAL] campos base reales, time-travel [PROPUESTA]; §2.4 approve/reject) — filtro VantaDB aplicado; `Nuevas features`/`Plan de accion` no mapean (índices, sin hijas de esta tarea).

**Prohibidos (NO tocar):** `reparacion.bat`, `.opencode` (submodule — EXCEPCIÓN FIND-83: mirror idéntico de skills editadas, commit SOLO paths parent; precedente lessons 2026-09-17), `Justfile`, `ocr-delegate.yml`, `ocr-review.ps1`, `completions/*`, `desktop/src-tauri/Cargo.lock`, stash@{0} GOV-C4, `docs/Backlog.md` (FIND-109 lead), `docs/plans/2026-09-17-mvp-memoria-agentes.md` (recitations lead), `src/llm.rs` (FIND-100 ✅), `examples/` (SHOW-04 ✅), `SPEC.md` (`~79` — dueño plan), `server.json` (sin cambio de superficie), `vantadb-mcp/src/handlers/tools.rs` + `config.rs` + `scenes.rs` + `dreams.rs` + `mcp_tests.rs` + `docs/api/MCP.md` §§ dreams + `test-mcp.py` §§ profiles (FIND-107 commits 472526dc/325b1237/74c225a0 — diff+merge POR SECCIÓN, jamás overwrite; mis edits son secciones disjuntas).

## 3. DEPENDENCIAS

Wave1 sola (Wave0 ✅ FIND-100 22d5a142 / SHOW-04 3bb5210d / FIND-107 472526dc+325b1237+74c225a0+41af793d). Sin bloqueantes (FIND-107 fijó superficie en 86 en paralelo). NextTask: FIND-104 + FIND-106 Wave2 (orquestador; FIND-106 implementa mi política por cliente; FIND-104 consume mi regla `~`/env).

## 4. REFERENCIAS + SPEC

**Rules (leídas completas antes de codificar):** `.opencode/rules/server-mcp.md` (R-1 sync doc↔código — core de esta tarea) + `api-contract.md` (R-1 claims→símbolos reales, R-5 paridad mismo-PR) + `README.md` del dir (formato).
**Refs:** `skills-engineering.md` (SDP) · `definition-of-done.md` (DoD 3 niveles + ratchet v1) · `task-system.md` · `architecture.md` · `research-modules.md` · `clean-code-clean-architecture.md` Ap. V (código nuevo: temporal/instructions/prompts) · `ocr-review.md` (gates VERIFY) · `dev-tools.md` + `test-suite.md` + `testing-patterns.md`. **Commands:** `pipeline.md` · `research.md` (no deriva; R3 reusado) · `audit.md`.

## Spec (feature-add con símbolos públicos nuevos — Gate D registrado, ver §8)

| # | Decisión | Opciones | Evidencia / Default |
|---|----------|----------|---------------------|
| 1 | Dónde vive el traductor temporal | (a) `vantadb-mcp/src/temporal.rs` pub + re-export **(Recomendado)** / (b) core `src/` / (c) solo docs sin código | (a): glue de interfaz, no lógica de búsqueda (R-8: no toca ranking/fusión); core intacto; `memory_list` no filtra timestamps (`mod.rs:551-574` metadata-only) → el traductor es helper de política, no cambio de motor |
| 2 | Exponer traductor como tool MCP | NO — 86 se mantiene | Nueva tool → sweep counts + profiles + FIND-106 re-trabajo; la política (FIND-106) lo invoca como procedimiento determinista documentado + helper testeado |
| 3 | `instructions` en `initialize` | (a) string recall-first estático **(Recomendado)** / (b) por perfil | (a): MCP 2025-06-18 `InitializeResult.instructions?` (verificar vía webfetch spec); estático = predecible, testeable |
| 4 | Prompts: ¿4 o más? | 4 SE MANTIENEN, contenido recall-first | `test-mcp.py:62` + `mcp_tests` gatean count=4; cambiar count rompe gates |
| 5 | Path ejecutable del rango [from,to] v1 | (a) `memory_list` paginado + filtro cliente por `created_at_ms` **(Recomendado)** / (b) IQL WHERE `__vanta_*` / (c) `search_memory` filters | (a): único verificado (`mod.rs:551-574` excluye timestamps del server-side; `get` devuelve `created_at_ms`); (b) SIN VERIFICAR → no documentar como working; (c) imposible (`tools.rs:3209-3226` igualdad-only) |
| 6 | Fallback traductor ambiguo | rango amplio (últimos 30d) + aviso explícito, nunca silencio | Pre-mortem plan (2): casos raros → suite + fallback avisado |

## 5. SKILLS (SDP Paso 0b)

`campaign_discover_skills_v2` BUILD → base (`source-driven-development`, `security-and-hardening`, `campaign-executor`, `progreso`) + lifecycle (`incremental-implementation`, `test-driven-development`, `context-engineering`, `doubt-driven-development`, `frontend-ui-engineering`, `api-and-interface-design`) + keywords (`documentation-and-adrs`, `writing-guidelines`, `api-and-interface-design`, `spec-driven-development`). **Cargadas (8):** `incremental-implementation` (slices verticales) · `test-driven-development` (RED→GREEN traductor/instructions/prompts) · `context-engineering` (context packs por slice) · `doubt-driven-development` (claims falsos 79/60/45/15) · `api-and-interface-design` (instructions/prompts como contrato) · `source-driven-development` (MCP spec `instructions` vía webfetch) · `documentation-and-adrs` (contenido skill) · `spec-driven-development` (diseño recall §4). N/A: `frontend-ui-engineering` (sin `web/`), `writing-guidelines`, `security-and-hardening` (sin trust boundary nuevo — initialize/prompts son solo-lectura; secret scan en S7).

## 6. HERRAMIENTAS

codegraph_explore PRIMERO (hecho en DISCOVERY) · `codebase-memory-mcp` (N/A pesado: blast radius ya mapeado vía codegraph+grep; `detect_changes` en S7 si hace falta) · Notion fetch Problema/Propuesta ✅ (hecho) · `pwsh scripts/validate-docs-coverage.ps1 -ReportOnly` por slice · `cargo test -p vantadb-mcp -j 2 --test <focado>` + `cargo clippy -p vantadb-mcp --all-targets -- -D warnings` + `cargo fmt --check` · `python -m py_compile` (test-mcp.py) + corrida viva vs `target/debug/vantadb-server.exe` (existe, 2026-09-17 12:05) · `campaign_verify_cmd` (bug exit -1 conocido → bash directa + mención) · web SOLO `instructions` spec (gap real vs R3) · OCR delegation en S7 (`pwsh dev-tools/ocr-review.ps1`; Critical/High=bloquea, Medium→FIND-*). Cargo siempre `-j 2`.

## 7. IMPACTO MAPEADO (Regla 0)

**Archivos leídos completos:** SKILL.md 641L · configuration.md 190L · mcp-protocol.md 329L · api-reference.md (1-200) · test-mcp.py 253L · initialize.rs 41L · prompts.rs 101L · threads.rs 282L · config.rs 142L · lib.rs 64L · scenes.rs 1-60 · dreams.rs 1-60 · tools.rs §§ 77-140/1010-1214/1499-1561/2740-2860/3140-3260 · MCP.md 521L · capture.rs 1-60 · server.rs 1-80 · serialization/mod.rs 425-574 · server-mcp.md + api-contract.md · definition-of-done.md · plan file · Backlog FIND-103/104/106 · FIND-107.md 1-60 · SPEC.md:11 · SKILLS-MANIFEST:476 · opencode.jsonc:87 · Notion Problema/Propuesta.
**Referencias hacia dentro:** tools.rs→7 módulos extend; server.rs→handlers; test-mcp.py→binario; skill docs→código; recall-policy→memory_recall/context_assemble/threads/proxy-turns/aprobaciones (FIND-107 S4 DEFER).
**Referencias entrantes:** mcp_tests.rs (86/profile/initialize/prompts asserts) · test_embed_texts/scene/dream/thread tests · validate-docs-coverage §6/§7 · opencode.jsonc · FIND-104/106 (consumen política) · desktop child_process (spawn, no firma).
**Veredicto:** ADITIVO y reversible. Docs: counts/`~`/env/prompts-sección/recall-policy (secciones propias). Código: `initialize.rs` (+1 campo), `prompts.rs` (textos), NUEVO `temporal.rs` + `temporal_tests.rs` + 2 líneas `lib.rs`. Sin cambio de superficie tools (86 intacto), sin tocar core, sin schemas. Riesgo overwrite FIND-107 → ediciones en secciones disjuntas + `git diff --stat` antes del commit.

## 8. STEPS ATÓMICOS (~100 líneas, PLAN→ACT→VERIFY c/u)

- [x] **S1 counts 79→86 (docs-only)** ✅ VERIFY: grep 0 stale + coverage 0 gaps + mirrors SAME.
- [x] **S2 `~` + env vars (docs-only)** ✅ DONE (0 stale `~` JSON + 8 env mentions + mirrors SAME + coverage 0 gaps). configuration.md §§ MCP (`VANTADB_MCP_PROFILE`, `VANTADB_MCP_BYTE_BUDGET`) + embeddings (`VANTADB_EMBEDDING_PROVIDER`, `VANTADB_LOCAL_MODEL`, `VANTADB_OPENAI_API_KEY/MODEL`, `ORT_DYLIB_PATH`) + nota `~` no expande (clientes spawn directo; `Path::new` sin tilde — `server.rs`/`cli_handlers/server.rs`) · SKILL.md:30/35/49 + MCP.md:44-79 ejemplos a path absoluto + cross-link. Verify: grep env × código.
- [x] **S3 `instructions` (código+test+doc)** initialize.rs `instructions` recall-first (webfetch spec primero) · mcp_tests assert `instructions` non-empty + menciona recall · SKILL/MCP nota. Verify: `cargo test -p vantadb-mcp -j 2 --test mcp_tests initialize`.
- [x] **S4 prompts recall-first (código+tests+doc)** prompts.rs 4 textos (search→recall-then-search+temporal; analyze→incluye curaduría/inbox; summarize→cita supersession/TTL; query_builder→ejemplo temporal+IQL honesto) · mcp_tests prompts update · SKILL.md:433-438. Count 4 intacto. Verify: tests prompts + test-mcp.py 4/4.
- [x] **S5 traductor temporal (código+tests)** NUEVO `temporal.rs` `parse_temporal_expression(expr, now_ms)` ES+EN (hoy/ayer/anteayer/esta semana/semana pasada/este mes/últimos N días/horas/"ayer a las 2pm"/"yesterday at 2pm"/"hace N días"/weekday pasado/lunes…; fallback `None`→política 30d) + `temporal_tests.rs` (≥15 casos, bordes: pm/am, lunes=weekday, mes-año, futuro→None) + `lib.rs` mod+re-export. Verify: test focado + clippy + fmt.
- [x] **S6 política recall + test-mcp.py** NUEVO `references/recall-policy.md` (hooks SessionStart/message/PreCompact/Stop · auto-recall por mensaje `memory_recall` scope/top-k · umbrales: top-k default 5, cuándo NADA se inyecta (0 hits o score<umbral → no inyectar, no rellenar) · `additionalContext` (COGNEE:390) · auto-capture al cerrar (threads/`thread_send`) · curaduría proxy-turns→hilos vía bandeja aprobación (FIND-107 S4 DEFER: `capture_list_pending/approve/reject` pendientes — política lista, tools pendientes) · presupuesto tokens) + SKILL wiring · test-mcp.py paso 5 recall (`memory_recall` en DB temp → `recalled`+`prepend_context`) + mirrors. Verify: py_compile + corrida viva 5/5 + coverage 0 gaps.
- [x] **S7 verify full + commit:** fmt+clippy+nextest focado (`-p vantadb-mcp -j 2`) + coverage + OCR delegation + `git diff --stat` (solo paths propios) + commit conventional + `campaign_memory_write` lessons.

## Context Save Point

DISCOVERY ✅ (código↔skill verificados mecánicamente; Spec §4 con 6 decisiones; Notion Problema/Propuesta leídas; R3 reusado sin web nueva salvo spec `instructions`). Pendiente: S1→S7. Deuda: Notion hijas (8 dims/6 áreas/10 ámbitos) no leídas — no mapean a esta tarea (solo dims 2/4/5 citadas vía Problema); ai_search sin plan (irrelevante). Gates: P:no (wave sola asignada) · D:disparado-sin-question-tool (símbolos nuevos exigidos por contrato §4; sigo plan aprobado) · V:no (0 fallas) · C:no (sin colaterales; WIP ajeno intacto verificado `git status`).

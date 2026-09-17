# FIND-107 — Exponer lo que falta de `vanta-memory` en el MCP

> **Plan:** `docs/plans/2026-09-17-mvp-memoria-agentes.md` (Wave0, appetite 3d, branch `develop`, commit `feat: FIND-107 — ...`)
> **Ruta:** vanta-worker · **Estado:** ⏳ IN PROGRESS (Slice 1 en curso, resto ⬜)
> **Campaign:** b2ece025-e9d3-4f8b-835d-1d0143a86b66 · **NextTask:** FIND-103 (la ejecuta el orquestador)

## 1. TAREA

**Objetivo:** exponer la superficie útil de `vanta-memory` que hoy no tiene mostrador MCP (mostrador incompleto). Sub-módulos del gateway → handlers:

| # | Sub-módulo | Fuente (`vanta-memory`) | Destino (`vantadb-mcp`) | Tools nuevas |
|---|-----------|------------------------|------------------------|--------------|
| S1 | escenas-escritura | `src/core/scene/scene_tools.rs:118-167` (`write_scene_tool`, `edit_scene_tool`, `execute_scene_tool`) | `src/scenes.rs` + `src/handlers/tools.rs` | `scene_write`, `scene_edit` |
| S2 | sueños-lectura | `src/core/dream/mod.rs:536-602` (`list_dream_runs`, `load_dream_run`, `discard_dream_run`) | nuevo `src/dreams.rs` | `dream_list`, `dream_load`, `dream_discard` |
| S3 | sueños-escritura | `src/core/dream/mod.rs:614-709` (`consolidate_session` LLM-free, `promote_dream_run` stub-count) | `src/dreams.rs` | `dream_consolidate`, `dream_promote` (preview count, NO muta L1) |
| S4 | bandeja aprobación | `src/gateway/approval_handlers.rs:90-118` (`capture_list_pending`, `capture_approve`, `capture_reject`) + `src/core/record/approval.rs:64-158` (`CaptureApprovalQueue` in-memory) | nuevo `src/approvals.rs` | `capture_list_pending`, `capture_approve`, `capture_reject` |
| S5 | extracción habilidades | `src/core/skill/mod.rs:24-34` (`run_skill_extract_once`, `extract_skills_with_llm`) | nuevo `src/skill_extract.rs` o ext. `src/skills.rs` | `skill_extract` (1 tool, degrada sin runner) |
| S6a | ingesta real | `src/ingest/` (`worker.rs`, `mod.rs`) + `src/wiki.rs:399-467` (`start_ingest::<NoLlm>` hoy fijo) | `src/wiki.rs` | `wiki_ingest` con runner real (hoy `NoLlm` = sources skipped) |
| S6b | programador | `src/services/pipeline_worker.rs:1-120` (`PipelineWorker::run_once`, `MemoryTaskHandler`, `TaskKind::Dream`) | nuevo `src/scheduler.rs` | `scheduler_status`, `scheduler_run_once` |

**Contrato exacto (plan):** `tools/list` incluye nuevas tools + tests por tool verdes + docs + coverage 0 gaps.
**Acceptance criteria:**
- (a) `tools/list` con nuevas tools (cada slice registra sus definitions + dispatch + annotations MCP-38).
- (b) tests por tool verdes (`vantadb-mcp/tests/<slice>_tests.rs`, red→green, `cargo test -p vantadb-mcp -j 2`).
- (c) docs actualizadas en el mismo PR (`docs/api/MCP.md` + `server.json` si aplica; regla `api-contract.md` R-5).
- (d) coverage 0 gaps (`pwsh scripts/validate-docs-coverage.ps1` limpio para las tools nuevas).
- (e) sin regresiones (suite `mcp_tests` + `scene_tests` verdes), clippy `-D warnings` 0, `cargo fmt --check` limpio.
- **DoD 3 niveles:** (1) contrato del slice ✅; (2) standing DoD (`definition-of-done.md`: correctness+quality+integration+docs+ship-readiness); (3) ratchet VantaDB (capa determinista 0-5 + pre-commit 7 ítems, DoD v1).
- **Stop:** 1 sub-módulo trancado → ship resto + DEFER-ratificado puntual (ej. ingesta sin runner → DEFER). **Esta invocación:** ship S1; S4/S5/S6a candidatos a DEFER-ratificado (ver §8).

## 2. ARCHIVOS

**Clave (con :línea):**
- `vanta-memory/src/core/dream/mod.rs:1-90` (doc + invariantes no-mutación L1 + `DreamConfig`), `:484-622` (store layer + `promote_dream_run` stub), `:629-709` (`consolidate_session`)
- `vanta-memory/src/gateway/mod.rs:1-19` (re-exports knowledge + approval)
- `vanta-memory/src/gateway/knowledge_handlers.rs:34-124` (`KnowledgeError`, `scene_read/list/query`)
- `vanta-memory/src/gateway/approval_handlers.rs:1-118` (queue handlers puros)
- `vanta-memory/src/core/scene/scene_tools.rs:104-200` (`read/write/edit/execute_scene_tool` + validación frontera)
- `vanta-memory/src/core/skill/mod.rs:20-34` + `skill_extractor.rs:1-120` (contrato extracción)
- `vanta-memory/src/ingest/mod.rs:1-96` + `worker.rs` (pipeline ingesta serial)
- `vanta-memory/src/services/pipeline_worker.rs:1-120` (worker + `TaskKind::Dream`)
- `vanta-memory/src/core/record/approval.rs:1-80` (queue in-memory, scope sin persistencia)
- `vantadb-mcp/src/handlers/tools.rs:25-70` (registry annotations MCP-38), `:70-...` (`handle_tools_list`), `:2882-2891` (dispatch scenes/skills/wiki)
- `vantadb-mcp/src/scenes.rs:1-185` (**plantilla** a seguir para S1: definitions + dispatch + validación frontera + `error_content` vs invalid-params)
- `vantadb-mcp/src/lib.rs:12-63` (registro módulos + re-exports)
- `vantadb-mcp/src/wiki.rs:134-166` (definitions ingest), `:363-467` (facade `start_ingest::<NoLlm>` — punto S6a)

**Relacionados:**
- `vantadb-mcp/tests/scene_tests.rs:1-257` (plantilla tests: tools/list + roundtrip + error contract)
- `vantadb-mcp/tests/mcp_tests.rs` (suite regresión, 181KB — no romper)
- `vanta-memory/tests/dreaming.rs`, `scene_tools.rs`, `capture_approval.rs`, `skill_extract.rs`, `ingest.rs` (contratos fuente, fixtures reutilizables)
- `docs/api/MCP.md:428-436` (sección Scenes a extender con `scene_write/edit`; resto secciones por slice)
- `scripts/validate-docs-coverage.ps1` (gate coverage), `server.json` (descriptor registry)
- `vantadb-mcp/src/validation.rs` (`validate_identifier`, `validate_payload`, `error_content`, `text_content`), `src/error.rs` (`McpError::invalid_params`)

**Prohibidos (NO tocar):**
- `reparacion.bat` (WIP ajeno), `.opencode` (submodule), `Justfile`, `ocr-delegate.yml`, `ocr-review.ps1`, `completions/*`, `desktop/src-tauri/Cargo.lock`, `stash@{0}` GOV-C4
- `src/llm.rs` (FIND-100 en paralelo), `examples/` (SHOW-04 en paralelo)
- `vantadb/src/wal.rs`, `vantadb/src/vector/`, `vantadb/src/storage/` (propiedad Arch/Engine — fuera de dominio worker-bindings salvo lectura)

## 3. DEPENDENCIAS

- **Wave0 sin dependencias.** Paralela con FIND-100 (`src/llm.rs`) + SHOW-04 (`examples/`) — archivos disjuntos, verificado.
- **Stop:** 1 sub-módulo trancado → ship resto + DEFER-ratificado puntual (ej. ingesta sin runner → DEFER; aprobación sin lifecycle persistente → DEFER; extracción sin runner → degrada o DEFER).
- **NextTask:** FIND-103 (la ejecuta el orquestador).
- **Orden interno (riesgo × valor):** S1 (escenas-escritura, puro + plantilla exacta) → S2 (sueños-lectura, puro Embedded) → S3 (sueños-escritura, diseño params idle) → S4 (aprobación, lifecycle queue) → S5 (skills, runner) → S6a/S6b (ingesta real + programador, diseño pesado).

## 4. REFERENCIAS

**Rules (lectura completa antes de codificar — hecho 2026-09-17):**
- `.opencode/rules/core-engine.md` — R-3 (sin `unwrap` en prod, `?` + `Result`), R-4 (`unsafe` no aplica aquí — no se introduce), R-1 (nada experimental sin gate; lo nuevo es estable).
- `.opencode/rules/api-contract.md` — R-1 (todo claim apunta a símbolo real), R-3 (no exponer lo que el core rechaza por defecto), R-5 (**tools + docs en el mismo PR**), R-8 (**lógica en core, bindings glue+memoria**: handlers validan frontera + mapean errores, no reimplementan dedup/fusión).
- `.opencode/rules/server-mcp.md` — R-1 (serverInfo/tools sync doc↔código), R-2 (handlers sync vía `spawn_blocking` — los nuevos siguen el mismo patrón que `scenes.rs`, sin bloquear el loop).
- **Refs:** `architecture.md` (workspace: `vanta-memory` fuente, `vantadb-mcp` mostrador), `definition-of-done.md` (standing + DoD VantaDB + shippable a-e), `clean-code-clean-architecture.md` Ap. V (frontera: entidades `vanta-memory::core`, casos de uso gateway, adaptadores `vantadb-mcp/src/*.rs` como Humble Objects + DTOs = tipos gateway), `understand-anything.md` (CodeGraph primero para símbolos).
- **Commands:** `pipeline.md` (ejecución), `research.md` (solo si ambigüedad — N/A por defecto), `audit.md` (verify L9/post-tarea).
- **SPEC.md raíz** como contexto (F5 = este task; éxito medible §Success Criteria 3-4).

**Tabla Spec (una fila por tool/símbolo público nuevo — decisión o por-evidencia):**

| Tool nuevo | Decisión / evidencia | Input schema | Output | Errores | Annotations |
|-----------|---------------------|--------------|--------|---------|-------------|
| `scene_write` | **Por-evidencia:** `write_scene_tool` existe (`scene_tools.rs:118-129`), puro sobre `&Embedded`, validación frontera ya definida; MCP solo la envuelve (R-8). | `session_key*`, `scene_name*`, `summary*`, `content*` | `{scene:{...}}` (SceneBlock) | params ausentes → `-32602`; `Invalid` → `error_content`; `Scene` → `error_content` | readOnly false, destructive false, idempotent false (upsert no idempotente en heat), openWorld false |
| `scene_edit` | **Por-evidencia:** `edit_scene_tool` existe (`:136-167`); `NotFound` cuando falta; requiere ≥1 campo. | `session_key*`, `scene_name*`, `summary?`, `content?` (≥1) | `{scene:{...}}` | sin campos → `error_content(Invalid)`; `NotFound` → `error_content` (indistinguible por diseño) | readOnly false, destructive false, idempotent false, openWorld false |
| `dream_list` | **Decisión:** metadata-only (`DreamRunMeta`), orden estable por `run_id` (ya garantizado `:566`). | `session_key*` | `{runs:[{run_id,started_at_ms,ended_at_ms,inputs_scanned,runner_label}]}` | session vacía → `Invalid`→content | readOnly true, idempotent true |
| `dream_load` | **Decisión:** payload completo para inspección/replay. | `session_key*`, `run_id*` | `{run:{...}}` o `not found` content | corrupto → `error_content(Read)` | readOnly true, idempotent true |
| `dream_discard` | **Decisión:** delete real de `dream/<s>/<run_id>` (`:593-602`); L1 intacto. Destructive **scoped** (solo ns dream). | `session_key*`, `run_id*` | `{discarded:true}` | store fail → content | readOnly false, destructive true(scoped), idempotent true |
| `dream_consolidate` | **Decisión:** envuelve `consolidate_session` **LLM-free** (`dreaming_runner: None`); exige `now_ms`, `last_active_at_ms`, `idle_threshold_ms?`. No-idle → `error_content(Runner)` (no protocolo). | `session_key*`, `now_ms*`, `last_active_at_ms*`, `idle_threshold_ms?`, `run_id_salt?` | `{run:{...}}` | not-idle → content (no -32602) | readOnly false, destructive false, idempotent false |
| `dream_promote` | **Decisión:** expone el stub **como preview** (`promote_dream_run` devuelve count, NO muta — `:604-622`); nombre documenta `preview_count`. Alternativa B (no exponer hasta MEM-65) → se elige exponer-preview porque el plan pide "sueños" visibles y el stub ya es el contrato público. | `session_key*`, `run_id*` | `{preview_count:N, mutated:false}` | run ausente → content | readOnly true (no muta), idempotent true |
| `capture_list_pending` | **Decisión pendiente (Gate diseño):** queue es **in-memory por proceso** (`approval.rs:9`); MCP stdio es stateless por spawn → la bandeja solo tiene sentido con lifecycle documentado (mismo proceso) o persistencia. Opción A ship-scope-proceso + docs; B persistir (fuera appetite); C DEFER. **Recomendación: A si hay test que lo prueba en-proceso, si no C.** | — | `{pending:[...]}` | — | readOnly true |
| `capture_approve/reject` | Idem S4. `approve` persiste vía `apply_dedup_batch` (mutación L1 real — destructive false pero write). | `id*` | `{written:[...]}` / `{rejected:bool}` | id vacío → invalid-params; `NotFound` → content | write, idempotent: approve false / reject true |
| `skill_extract` | **Decisión pendiente:** necesita `LlmRunner`; sin runner → degrada a `{success:false, candidates:[], error}` (Principio 4, `skill_extractor`). Exponer con degradación documentada o DEFER. | `messages[]`, `config?` | `{success,candidates[],error?}` | LLM fail → content con success false (no protocolo) | readOnly false→true? (no escribe sin sink; solo candidatos) → readOnly true |
| `wiki_ingest[real]` | **Pre-mortem plan:** sin runner real → **DEFER-ratificado puntual**. Hoy `NoLlm` = sources skipped (degradado P4). Runner real = nueva dependencia/config → Ask first (SPEC Boundaries). | — | — | — | — |
| `scheduler_*` | **Decisión pendiente:** `run_once` necesita `LocalStateBackend` + dir estado + locks TTL; exponer requiere diseño de lifecycle del scheduler en MCP (¿quién posee el backend?). Riesgo scope → slice último o DEFER. | `...` | `{processed,failed,skipped_locked}` | — | write, non-idempotent |

## 5. SKILLS

**SDP (Paso 0b, `campaign_discover_skills_v2` phase BUILD, keywords `mcp/tools-list/dream/approval/skill-extraction/ingestion/scheduler/scenes`):**
- Devueltas (8, con justificación): campaign-executor(base MCP) · source-driven-development(base) · incremental-implementation(lifecycle BUILD slices) · test-driven-development(lógica nueva RED→GREEN) · context-engineering(sesión compleja) · doubt-driven-development(stakes producción) · frontend-ui-engineering(**ruido** — no hay `web/`, se descarta) · api-and-interface-design(schemas/tools).
- Cargadas efectivas (8 justificadas, ≤8): `api-and-interface-design` (schemas/tools S1) · `test-driven-development` (RED→GREEN por tool) · `systematic-debugging` (si verify falla, no reintentar a ciegas) · `codebase-memory` (blast radius + architecture, ya ejecutado §7) · `doubt-driven-development` (claim "no muta L1" + destructive-scoped) · `documentation-and-adrs` (MCP.md + ADR si decisión S4/S6) · `mcp-builder` (guía tools MCP: schemas/annotations/errores) · `incremental-implementation` (slices S1→S6, ship parcial) + base auto (campaign-executor/progreso/ponytail-full/context-engineering/source-driven).
- `SDP: api-and-interface-design, test-driven-development, systematic-debugging, codebase-memory, doubt-driven-development, documentation-and-adrs, mcp-builder, incremental-implementation (descartada frontend-ui-engineering por no aplicar; base campaign-executor/progreso/ponytail-full/context-engineering/source-driven activa)`

## 6. HERRAMIENTAS+MCP

- `codegraph_explore` primero (hecho: gateway→handlers; plantilla `scenes.rs` identificada).
- `codebase-memory-mcp`: `check_index_coverage` (hecho: 3 paths `no_recorded_issue`, best-effort — se leyó fuente igual), `get_architecture` (hecho: overview/clusters/hotspots/boundaries; hotspot `DreamConfig::clone` fan-in 880 — no tocar), `detect_changes` (hecho: solo WIP ajeno + plan/backlog, `impacted_total: 0` — blast radius limpio).
- `cargo test -p vantadb-mcp -j 2` (siempre `-j 2` — rustc crash/OOM).
- MCP stdio smoke (`initialize` + `tools/list` cuenta; `First test` de MCP.md).
- `notion` fetch (hecho Paso 0c: `Problema` completa 2026-09-07; filtro VantaDB: ciclo de vida + 6 áreas + 8 dimensiones aplican; resto holones N/A).
- `agent-search`/`metasearchmcp` SOLO si ambigüedad de diseño (patrones Mem0/Letta) — **N/A por defecto** (fuente local suficiente; dream.rs ya cita `letta.com/blog/sleep-time-compute`).
- `campaign_verify_cmd` (bug exit -1 → bash directa; documentar en RESULTADO si ocurre).
- Prohibido: tocar archivos §2-prohibidos; secrets nunca a disco.

## 7. INVESTIGACIÓN CÓDIGO (blast radius, DISCOVERY)

- **Orden por sub-módulo; riesgo scope ×6 → dividir al primer síntoma (pre-mortem #1).**
- **S1 escenas-escritura (riesgo 🟢):** puras reutilizables `write_scene_tool`/`edit_scene_tool`/`execute_scene_tool` (`scene_tools.rs:104-200`) + validación frontera (`:207-249`). Handler plantilla `vantadb-mcp/src/scenes.rs:25-185` (definitions `:25-85`, dispatch `:87-100`, `session_key_arg` `:102-110`, `db_from` `:112-114`, error contract: params→`-32602`, dominio→`error_content`). Blast radius: `scenes.rs` + `handlers/tools.rs` (registro + dispatch) + `docs/api/MCP.md` + tests nuevos. Callers actuales: `scene_read/list/query` (gateway) — escribir no los afecta (upsert separado, heat bump). **Veredicto: impacto quirúrgico, reversible (aditivo).**
- **S2/S3 sueños (riesgo 🟡):** store `dream/<s>/<run_id>` aislado; invariante "L1 jamás se muta" (`dream/mod.rs:13-27`, test `tests/dreaming.rs` byte-identical). `promote` es stub-count (documentar `mutated:false`). Hotspot `DreamConfig::clone` fan-in 880 — **no modificar `DreamConfig`**, solo construirlo.
- **S4 aprobación (riesgo 🟡🔴):** gateway puro, pero `CaptureApprovalQueue` es **in-memory por proceso** (`approval.rs:9,64-72`); MCP stdio spawnea proceso por cliente → bandeja vacía entre procesos salvo que el mismo servidor atienda captura+decisión. Sin lifecycle diseñado, exponer es mostrador vacío. **Veredicto: Gate diseño — ship solo con scope-proceso documentado + test en-proceso, si no DEFER.**
- **S5 skills (riesgo 🟡):** `run_skill_extract_once` + `extract_skills_with_llm` necesitan `LlmRunner`; sin runner degradan a success:false (nunca bloquean). Exponer como solo-candidatos (read-only) es seguro; sink con escritura = slice separado.
- **S6a ingesta real (riesgo 🔴):** `start_ingest::<NoLlm>` fijo en `wiki.rs:274`; runner real = transporte LLM + config + secrets → SPEC "Ask first: nueva dependencia" + pre-mortem #2 → **DEFER-ratificado puntual por defecto**.
- **S6b programador (riesgo 🟠):** `PipelineWorker::run_once` + `LocalStateBackend` + locks TTL + `TaskKind::Dream` (ya cableado MEM-65). Exponer `run_once` exige dueño del backend en el proceso MCP (¿dir estado? ¿quién lo reclama?). Diseño no trivial → último slice o DEFER.
- **Transversal:** `handle_tools_list` + annotations MCP-38 (cada tool declara 4 hints; actualizar comentario conteo 79→N). `validate-docs-coverage.ps1` debe seguir 0 gaps (cada tool nueva con fila en MCP.md). Regla 8 api-contract: handlers = glue (validar frontera, mapear errores), cero lógica de negocio duplicada.

## 8. INVESTIGACIÓN PROBLEMA (alcance exacto por sub-módulo — uphill del plan)

- Uphill real = "alcance exacto exposición" (plan). Resuelto por-módulo arriba: S1/S2/S3 alcance cerrado (funciones puras + store aislado); S4 alcance condicionado a lifecycle; S5 condicionado a runner; S6a/S6b condicionados a diseño + dependencias.
- **Si ingesta no tiene runner real → DEFER-ratificado puntual en vez de forzar** (pre-mortem #2). Aplica igual a S5 sin runner (degrada, no fuerza) y S4 sin lifecycle (DEFER, no mostrador vacío).
- Notion `Problema` (filtro VantaDB): exponer sueños/aprobación/skills ataca directamente autoenvenenamiento (procedencia + curaduría), duplicación/ruido (dedup + aprobación), deriva por resúmenes (consolidación idle revisable/descartable). Escenas-escritura cierra el loop L2 que hoy solo se lee.

## 9. INVESTIGACIÓN INTERNET

- **N/A por defecto** (plan: solo si ambigüedad de diseño). Sin ambigüedad para S1 (plantilla local completa). Si S4/S5/S6 la requieren (patrones Mem0/Letta approvals/schedulers), digest ≤500 palabras + URLs verificadas; sin red → `[cita NO VERIFICADA]` + deuda TSYS-13. Esta invocación: no se usó web (cero citas).

## 10. VALIDACIÓN+CIERRE

- Verify por slice: `cargo test -p vantadb-mcp -j 2 --test <slice>` + `cargo test -p vanta-memory -j 2 <mod>` (si toca fuente — S1 no toca fuente) + MCP stdio smoke + `validate-docs-coverage.ps1`.
- Verify full cierre: `cargo fmt --check` → `cargo clippy --workspace --all-targets --all-features -- -D warnings` → `cargo nextest run --profile audit --workspace --build-jobs 2` → `scripts/validate-docs-coverage.ps1` → OCR delegation (`pwsh dev-tools/ocr-review.ps1`; Critical/High=bloquea).
- DoD 3 niveles (§1) + P2-01 lo hace el orquestador (no auto-auditarse) + Gates D/V/C vía `question`.
- Cierre con commit conventional (**NO PUSH** — solo `vanta-lead` pushea): `git add <solo tocados> && git commit -m "feat: FIND-107 — ..."`. RESULTADO §7 obligatorio. Si no termina: hecho + próximo step.

---

## Impacto mapeado (Regla 0) — S1 escenas-escritura

- **Archivos leídos completos:** `vanta-memory/src/core/scene/scene_tools.rs` (249L: write/edit/execute + validación), `vantadb-mcp/src/scenes.rs` (185L: plantilla definitions/dispatch), `vantadb-mcp/src/handlers/tools.rs` (registro + dispatch, líneas §2), `vantadb-mcp/tests/scene_tests.rs` (257L: patrón tests), `docs/api/MCP.md:428-436` (sección Scenes), `vanta-memory/src/gateway/*`, `vanta-memory/src/core/dream/mod.rs` (alcance resto).
- **Referencias hacia dentro (lo que S1 usa):** `vanta_memory::core::scene::scene_index::{upsert_scene,get_scene}` ← `write_scene_tool`/`edit_scene_tool`; `vantadb::Embedded::from_engine` (`db_from`); `crate::validation::{validate_identifier, error_content, text_content, serialize_content}`; `McpError::invalid_params`.
- **Referencias entrantes (quién usa lo que S1 toca):** `handle_tools_list` ← tests `mcp_tests.rs` (cuenta total tools — **actualizar conteo si aserta 79**), `handle_tools_call` dispatch ← `server.rs` stdio; `docs/api/MCP.md` ← `validate-docs-coverage.ps1`; nada en `src/llm.rs` ni `examples/` (disjuntos Wave0 ✅).
- **Veredicto:** impacto **quirúrgico-aditivo** (2 tools + 2 arms dispatch + docs + tests). Reversible (`git revert` limpio). Sin hot paths (escenas no están en `vector/`/`engine.rs` search loop). Sin `unwrap` nuevo (propagar con `?`/`map_err`). Sin `unsafe`. Sin dependencias nuevas.

## Steps atómicos (~100 líneas c/u, verify mecánico)

- [x] **Step 0 — DISCOVERY + task file** (este archivo; Spec §4 + blast radius §7 + Regla 0 arriba). Verify: existe + plan recitation sync. ✅
- [x] **Step 1 — S1 `scene_write` + `scene_edit`**: definitions + dispatch en `scenes.rs`, registro en `tools.rs` (lista + match + conteo comentario), tests `scene_write_edit_tests.rs` (RED→GREEN: list/write/edit/notfound/validation), docs MCP.md (2 filas + conteo), coverage 0 gaps. Verify: `cargo test -p vantadb-mcp -j 2 --test scene_write_edit` + `cargo test -p vantadb-mcp -j 2 --test scene_tests` + smoke tools/list.
  - **Hecho 2026-09-17:** RED (`scene_write missing from tools/list`) → GREEN (5/5 `scene_write_edit_tests` ✅, 7/7 `scene_tests` ✅, `test_mcp_tool_annotations_coverage` + `test_mcp_tool_profiles` ✅, suite `vantadb-mcp` completa 0 failed ✅, `cargo fmt --check -p vantadb-mcp` ✅, `clippy -D warnings` ✅, `validate-docs-coverage.ps1` 0 gaps ✅, OCR delegation sin bloqueos). Full 79→81 (Full-only; dev/memory intactos).
  - **Commit S1 (2026-09-18, hook verde tras FIND-100 `22d5a142`):** re-verificado 5/5 + 7/7 en esta sesión; commit selectivo 8 paths con `feat: FIND-107 S1 — scene_write/scene_edit MCP tools (81 total)`. ✅
- [ ] **Step 2 — S2 sueños-lectura** (`dream_list/load/discard`): nuevo `src/dreams.rs` + registro + tests + docs.
- [ ] **Step 3 — S3 sueños-escritura** (`dream_consolidate` LLM-free + `dream_promote` preview): + tests (not-idle, roundtrip, L1-intacto) + docs (advertir `mutated:false`).
- [ ] **Step 4 — S4 aprobación** (Gate diseño lifecycle; si no cierra → DEFER-ratificado + fila Backlog).
- [ ] **Step 5 — S5 skill_extract** (solo-candidatos + degrada sin runner; si runner exige dependencia → DEFER).
- [ ] **Step 6 — S6a/S6b ingesta-real + programador** (por defecto DEFER-ratificado puntual salvo runner real disponible).
- [ ] **Step 7 — Cierre FIND-107**: verify full + OCR + commit `feat:` + recitation + progreso + RESULTADO.

## Context Save Point (actualizado 2026-09-17, post-S1)

- **Dónde quedó:** Step 0 ✅ + Step 1 ✅ (S1 commiteado, ver COMMIT abajo). Steps 2-7 ⬜.
- **Próximo:** Step 2 (S2 sueños-lectura) — nuevo `vantadb-mcp/src/dreams.rs` (`dream_list/load/discard`) + registro + tests + docs, mismo patrón S1.
- **Comandos:** `cargo test -p vantadb-mcp -j 2 --test scene_write_edit` · `cargo test -p vantadb-mcp -j 2 --test scene_tests` · `cargo fmt --check` · `cargo clippy -p vantadb-mcp --all-targets -- -D warnings`
- **Deuda:** S4/S5/S6a/S6b pendientes de Gate diseño (posible DEFER-ratificado). P2-01 (review por agente distinto) pendiente al orquestador.

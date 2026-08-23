# Campaña P27 — Vanta Memory Engine (F1-F4)

> **Migrado desde** `docs/progreso/README.md` (split GOV-D2, 2026-08-22). Contenido histórico sin cambios salvo dedup indicado.

### Campaña P27 Vanta Memory Engine completada 2026-08-20 - 24/24 tareas F1-F4 (plan `2026-08-18-vanta-memory.md`)

Cierre de campaña: **MEM-01..21 (+08a/08b, 34, 35) completadas** entre 2026-08-18 y 2026-08-20. Crate nuevo `vanta-memory/` (LLM-driven, publish=false) + integración core/server/MCP. Suite final: **361/361 tests** (`cargo nextest run -p vanta-memory`), fmt/clippy `-D warnings` limpios.

- **F1 (core):** MEM-01 SearchProfileConfig por request + cláusula IQL PROFILE (`6a50b8ee`); MEM-02 passthrough MCP (`32b09daf`); MEM-34 telemetría por capa L1/L2/L3/recall/offload (`84f28a18`).
- **F2/F3 (core+server+MCP):** MEM-03 entidades entity_* CRUD partición InternalMetadata (`23719e23`); MEM-04 permission-checker allow-only (`9717bf03`); MEM-05 auth 3 capas + audit JSONL (`01a5de66`); MEM-06 skills multi-versión optimistic lock (`92cf709f`); MEM-07 tools MCP skill_* (`4763bf44`); MEM-35 data plane REST `/conversation/add` + `/skill/listing` (`9693d0ff`).
- **F4 (crate vanta-memory, 14 commits `76a73969`→`31e676b1`):** MEM-08a/b scaffolding + contratos L1/trait LlmRunner sync (D1); MEM-09 L0 capture idempotente cursor `l0_cursor/<session>`; MEM-10 L1 extractor split+1 call LLM JSON con parse reparado (repair_priority_scalars + sanitize_json_for_parse tras gate P2-01 vanta-audit); MEM-11 dedup 2 fases store/update/merge/skip fallback store-all; MEM-12 contrato META {created,updated,summary,heat} + SceneNodeStore en core; MEM-13 tools sandboxed read/write/edit; MEM-14 strategy UPDATE>MERGE>CREATE + soft-delete flag `deleted`; MEM-15 persona first/incremental + triggers P1-P4 + escape_xml_tags; MEM-16 orquestación timers+locks estado local (trait Clock inyectable SystemClock/FakeClock, sin Redis, checkpoint paga deuda MEM-15); MEM-17 skill extract transcript marcadores anti role-capture + sink idempotente doble cursor+content-hash; MEM-18 recall prepend/append 3 modos (keyword/embedding/hybrid degradan a keyword LLM-free) + profile_sync; MEM-19 sanitize consolidado (sanitize_text 10 reglas sin regex) + truncación code-point; MEM-20 cursor offload `lastOffloadedToolCallId` por sesión; MEM-21 gateway knowledge_handlers scene_read/list/query.
- **Deuda documentada para F5-F7:** embeddings reales para recall hybrid (MEM-37), wiring SkillStore directo (MEM-35 ya cubre REST), docs/ADR del crate (MEM-38), capas refs/MMD/registry TDAM sin caller.

**Retrospectiva (D2):** Start: verify mecánico del lead tras cada sub-agente (atrapó corrupción del plan file en 11/12 tareas). Stop: confiar en el bloque RESULTADO sin verificar worktree. Continue: SARL RESUME misma sesión — recuperó 5 tareas detenidas en silencio sin perder trabajo. Acción medida: tasa de primer-intento de sub-agentes vanta-worker 6/12 (50%) → investigar prompt/cutoff de contexto antes de la próxima campaña (baseline: >90% objetivo RULES.md).

**Plan archivado:** `docs/plans/archive/2026-08-18-vanta-memory.md` — 24/24 completadas.
### MEM-21: F4 Tools MCP scene_read/list/query (gateway knowledge handlers)
- **Fuente:** Plan `docs/plans/2026-08-18-vanta-memory.md` (Task 24)
- **Fecha:** 2026-08-20
- **Objetivo:** Capa de entrada tipada (request/response serde) para las tools MCP `scene_read` / `scene_list` / `scene_query` sobre el store de escenas (MEM-12/MEM-15); un server MCP la expondrá después.
- **Resultado:** ✅ `vanta-memory/src/gateway/knowledge_handlers.rs` (nuevo): 3 handlers puros + `KnowledgeError` (`#[non_exhaustive]`) + tipos wire snake_case. Soft-delete respetado (read→NotFound, list/query excluidos, MEM-14); query LLM-free vía `overlap_score`/`significant_terms` reutilizados de l1_reader; validación boundary (`validate_scene_name`, session/keyword no vacíos). Wiring en `gateway/mod.rs`; `read_blocks` → pub(crate) (1 token). 10 tests D19 nuevos; suite vanta-memory 361 ✅; fmt+clippy -D warnings ✅. Commit pendiente del lead.
- **Ids:** `MEM-21`

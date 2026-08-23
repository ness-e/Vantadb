# Campaña P29 — Vanta Context Engine (F5)

> **Migrado desde** `docs/progreso/README.md` (split GOV-D2, 2026-08-22). Contenido histórico sin cambios salvo dedup indicado.

### Campaña P29 Vanta Context Engine completada 2026-08-21 - 9/9 tareas F5 (plan `2026-08-21-vanta-context-engine.md`)

Cierre de campaña: **F5 Context Engine completo** — compresión LLM-free + MMD + recall híbrido + GC, suite final **430/430 tests** (`cargo nextest run -p vanta-memory`), fmt/clippy `-D warnings` limpios, docs coverage 0 gaps.

- **Wave 0:** MEM-23 token estimator chars/3 + emergency truncate pair-safe (`8de35359`); MEM-40 recall_scope híbrido session|agent|team default agent + primer test search_multi (`89777704`); MEM-41 generation-log provenance best-effort genlog/<session> cap 100 (`1f89c0b6`); MEM-39 seed CLI vanta-seed idempotente por content-hash (`d3eba4fc`).
- **Wave 1:** MEM-22 assemble + cascada mild (MIN=10/INITIAL=7/FLOOR=1) + aggressive one-shot fingerprint idempotente + emergency prefix-aware; cursor MEM-20 vía protected_prefix (`4d1363ec`).
- **Wave 2:** MEM-42 reclaimer GC retention_days post-cursor estricto (`214a7820`); MEM-24 MMD persistente TaskMemory META dedup fingerprint + injector pair-safe (`ddc5671f`, D23 cerrada por lead tras 2 sub-agentes vacíos — STRATEGY SARL); MEM-37 assemble_with_recall budget coordinator único + e2e compress→recall (`ae7fe30b`).
- **Gate:** MEM-38 ADR-029 borrador técnico (revisión del autor humano pendiente — Regla 5) + superficies F5 en EMBEDDED_SDK.md, docs coverage 0 gaps (`badb5b9c`).

**Retrospectiva (D2):** Start: prompts con decisiones cerradas (MEM-24 con D23 abierta falló 2×; con decisión fijada completó 1ª vez). Stop: asumir que el sub-agente cerró el verify — 1 test e2e llegó roto. Continue: RESUME con feedback exacto del fallo mecánico. Acción medida: primer-intento de sub-agentes 3/9 (33%) vs objetivo >90% — investigar cutoff de contexto de vanta-worker antes de F6.

**Plan archivado:** `docs/plans/archive/2026-08-21-vanta-context-engine.md` — 9/9 completadas.
### MEM-41: F5 Generation-log provenance L1/L2/L3 consultable
- **Fuente:** Plan `docs/plans/2026-08-21-vanta-context-engine.md` (Task 3)
- **Fecha:** 2026-08-21
- **Objetivo:** Log consultable de provenance de generaciones L1/L2/L3 (layer/status/anchor/session/ts), best-effort (nunca bloquea el pipeline — Principio 4), con cap por sesión.
- **Resultado:** ✅ `vanta-memory/src/core/memory_generation_log/{mod,store}.rs` (nuevos): entry `{layer, status, anchor_id?, session_key, ts_ms, error?}`; ns `genlog/<session>` sanitizado; key `{ts:013}_{seq}`; `record_best_effort` traga errores con tracing::warn; cap 100 keep-recent; `query_session(db, session, layer?)` ordenado por ts. Hooks aditivos: l1_writer (succeeded, anchor=record.id), scene_extractor/persona_generator (wrappers inner+log, solo generaciones reales o fallos), pipeline_worker run_l1 (failed). 11 tests D19 nuevos (7 unit + 4 integración); suite vanta-memory 389 ✅; check/fmt/clippy -p vanta-memory exit 0. Commit pendiente del lead.
- **Ids:** `MEM-41`

### MEM-39: F5 Seed/import CLI (skills/persona iniciales)
- **Fuente:** Plan `docs/plans/2026-08-21-vanta-context-engine.md` (Task 4)
- **Fecha:** 2026-08-21
- **Objetivo:** Comando de import/seed inicial para vanta-memory: importa un JSON de seed (skills + persona) a namespaces sanitizados, idempotente por content-hash.
- **Resultado:** ? `vanta-memory/src/seed/{mod,input}.rs` (nuevos): schema propio mínimo JSON-only (desviación documentada del schema TDAM sessions/messages, acoplado a OpenClaw); skills → `skills_extract/<scope>` con payload StoredSkill parity MEM-06; persona → `persona/<session>`/`persona.md` como PersonaRecord legible por get_persona. Idempotencia content-hash (replay → todo unchanged); counts created/updated/unchanged; errores tipados `SeedError`. CLI: bin target `vanta-seed` (`cargo run -p vanta-memory --bin vanta-seed -- <seed.json> [--db <path>]`) — glue en src/cli.rs imposible por ciclo de dependencias vanta-memory→vantadb (documentado en task file); feature passthrough `fjall` para persistencia. 6 tests D19 nuevos (4 integración con archivo temporal + 2 unit parser); suite vanta-memory 395 ?; check/fmt/clippy -p vanta-memory exit 0. Commit pendiente del lead.
- **Ids:** `MEM-39`

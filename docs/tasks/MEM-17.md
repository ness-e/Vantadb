# MEM-17: F4 Skill extract transcript + sink idempotente

## Metadata
- **Plan file:** `docs/plans/2026-08-18-vanta-memory.md` (Task 20)
- **Fuente:** plan file Task 20 (MEM-17)
- **Esfuerzo:** 🔴
- **Prioridad:** 🔴
- **Tipo:** Rust (crate `vanta-memory`)
- **Creado:** 2026-08-20
- **Estado:** ✅ COMPLETED (verify 4/4 gates exit 0)

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `pipeline-full.md` (247), plan file Task 20 + Principios, task file `MEM-16.md` (93, plantilla), TDAM `MC/core/skill/skill-extractor.ts` (587 completo: formatTranscript marcadores `<<past-{role}>>` + `<<end-of-transcript>>`, truncateHeadTail, sanitizeGeneratedQuery, prefix skills blocks full/relevant/recent), TDAM `conversation-add/skill-core-sink.ts` (93 completo — sink NO re-crea, idempotente), `trigger-service.ts` (224 completo — orden archive→task), `prepare-archive.ts` (91 completo), `message-compressor.ts` (97 completo), `oversize-strategy.ts` (138 completo), `extract-worker.ts` (L1-200: contrato SkillCandidatesSink "must be idempotent", ghost tasks), `prompts/skill-review-prompt.ts` (198 completo, ya en inglés), `skill-listing-prompt.ts` (41 completo); crate: `scene_extractor.rs` (572, estilo referencia), `l0_recorder.rs` (sanitize_component/sanitize_key/now_ms/cursor pattern), `l1_writer.rs` (put_record namespace pattern), `llm_runner.rs` (LlmRunner/complete_json/extract_json), `core/mod.rs`, `prompts/mod.rs`, `Cargo.toml` (sin deps nuevas)
- **Referencias hacia dentro:** nuevos módulos consumen `core::abstractions::{LlmRunner, LlmRunParams}`, `core::conversation::{now_ms, sanitize_component, sanitize_key}` (pub(crate)), SDK `VantaEmbedded::put/get/delete`; prompts consumen tipos propios del módulo skill
- **Referencias entrantes:** ninguna hoy — MEM-18/19/35 consumirán; wirings aditivos en `core/mod.rs` (solo `pub mod`)
- **Veredicto impacto:** bajo — archivos 100% nuevos bajo `core/skill/` + 1 wiring aditivo; cero archivos del core `vantadb` tocados; API pública existente intacta

## Contrato

"`cargo check -p vanta-memory` pasa; tests dedicados de skill extract (D19) pasan (`cargo nextest run -p vanta-memory`); `cargo fmt --check` pasa; `cargo clippy -p vanta-memory --all-targets --no-deps -- -D warnings` pasa."

## Simplificaciones documentadas (ponytail — TDAM conversation-add = ~140KB TS)

| TDAM | Port Rust | Por qué |
|---|---|---|
| `skill-extractor.ts` tool-calling review agent (16 iteraciones, SkillToolsV2) | Extracción pure-text JSON: prompt review (taxonomía v2) + `complete_json::<Vec<ExtractedSkillCandidate>>`; "Nothing to save." → vacío | El trait `LlmRunner` del crate es sync sin tool-loop (decisión MEM-08a); mismo patrón que L1/L2/L3 |
| `agent-task-queue.ts` (31KB Redis BRPOP/SADD) + `worker-pool.ts` (16KB) + `wire.ts` + `add-handler.ts` HTTP | Sin puerto: cola/dispatch ya existe en MEM-16 (`PipelineWorker` + `LocalStateBackend`); el worker skill es un handler consumible por ese dispatch | Single-process Rust; Redis prohibido (Principio 7); duplicar dispatcher = slop |
| `buffer-storage.ts` (12KB COS JSONL) | Records VantaDB: archive ns + tasks ns (Principio 2) con helpers de namespace en `archive.rs` | Persistencia = VantaDB siempre |
| `trigger-service.ts` mutex Redis + `_tasks.json` CAS | Orden preservado (archive PRIMERO, luego task entry); atomicidad por read-modify-write single-record | El orden archive→task es la lección del incidente 2026-07-20; locks distribuidos no aplican single-process |
| Sink TDAM (asset-register no-op sin metadata service) | Sink ESCRIBE skills en namespace `skills_extract` con idempotencia doble: content-hash upsert (paridad `SkillStore.create` MEM-06) + cursor per-task (`{task_id}__applied`) | En vanta-memory solo hay `VantaEmbedded` (SkillStore del core necesita `&StorageEngine`, no expuesto); la integración real con MEM-06 ocurre en wiring MEM-35/07 a nivel de datos (mismo contrato name/description/content/content_hash) |

## Invariantes de dominio (handoff - MUST)

1. Transcript con marcadores no-naturales `<<past-user>>`/`<<past-assistant>>`/etc. + ancla `<<end-of-transcript>>` (anti role-capture, TDAM f546ab8c).
2. Truncado head-tail determinista (default 8000/32000 chars) con placeholder `[truncated N chars]`.
3. Taxonomía review v2: SOP / Background / Preference — captura default ("when in doubt, capture"), nunca secrets.
4. Sink IDEMPOTENTE: re-procesar la misma task/conversación NO duplica skills (cursor per-task + content-hash upsert).
5. Orden trigger: archive primero, task entry después (worker jamás ve task sin archive → ghost check).
6. LLM opcional (Principio 4): fallo de runner o JSON inválido → candidatos vacíos + success=false, NUNCA escritura parcial ni pérdida del archive.
7. Sanitización namespace `[A-Za-z0-9._/-]` ≤128, keys ≤512 sin NUL (reuso sanitize_component/sanitize_key).
8. Sin unwrap/expect en producción; errores tipados #[non_exhaustive]; sin deps nuevas.
9. Firma genérica `<R: LlmRunner>` (trait NO dyn-compatible).

## Steps

### Step 1 — Discovery + task file
- [x] Leer TDAM (skill-extractor, conversation-add/*, prompts completos) + APIs del crate
- [x] Crear task file (este) con Impacto mapeado Regla 0
- **Gate:** ✅ registro antes de tocar código

### Step 2 — core/skill/prompts/{skill_review_prompt,skill_listing_prompt}.rs + mod
- [x] `skill_review_prompt.rs`: system prompt review agent v2 (taxonomía 3-kind, role isolation, output contract JSON adaptado)
- [x] `skill_listing_prompt.rs`: header/footer/guidance constants
- [x] Wiring `core/skill/mod.rs` + `core/mod.rs`
- **Gate:** `cargo check -p vanta-memory` ✅

### Step 3 — core/skill/skill_extractor.rs
- [x] ExtractMessage, format_transcript (marcadores), truncate_head_tail, sanitize_generated_query
- [x] render_skills_block (prefijo full/recent), extract_skills_with_llm<R>
- [x] Tests unitarios inline D19
- **Gate:** `cargo check -p vanta-memory` ✅

### Step 4 — core/skill/conversation_add/: compressor + oversize + archive
- [x] `compressor.rs`: compress_message(s) tool>2KB head/tail+placeholder
- [x] `oversize.rs`: apply_oversize_strategy chunkMax/headKeep/tailKeep
- [x] `archive.rs`: prepare_archive_payload + ArchiveStore (write/read archive + task entries sobre VantaDB, orden archive→task)
- **Gate:** `cargo check -p vanta-memory` ✅

### Step 5 — conversation_add/: sink + worker (+ trigger consolidado en archive.rs)
- [x] `sink.rs`: SkillCoreSink.apply_candidates — cursor `{task_id}__applied` + content-hash upsert (IDEMPOTENTE)
- [x] `worker.rs`: run_once — read task → ghost check (archive ausente → drop) → extract → sink → mark done
- **Gate:** `cargo check -p vanta-memory` ✅

### Step 6 — Tests D19 integración + verify completo + cierre
- [x] `tests/skill_extract.rs`: transcript markers/truncado, extractor con fake runner, compressor/oversize, trigger orden archive→task, worker e2e, SINK IDEMPOTENTE (re-run misma task = 0 duplicados)
- [x] Verify: cargo check + nextest + fmt --check + clippy -D warnings exit 0
- [x] CIERRE: campaign_update_task_state taskId=20 completed + recitation; bloque RESULTADO §7
- **Gate:** verify todo exit 0

## Deuda técnica (Regla 6)

Sin deuda nueva neta. Desviaciones documentadas arriba (cola/pool/wire HTTP no portados — MEM-16 ya provee dispatch; sink escribe a namespace propio pendiente de wiring directo contra SkillStore del core cuando MEM-35 lo exponga).

## Recitation (canónico)

- **activeGoal:** MEM-17: F4 Skill extract transcript + sink idempotente
- **lastAction:** Implementado el pipeline skill-extract completo: `prompts/skill_review_prompt.rs` (review agent v2 — taxonomía SOP/Background/Preference, role isolation con marcadores `<<past-*>>`, output contract JSON adaptado al runner pure-text) + `skill_listing_prompt.rs` (header/footer/guidance); `skill_extractor.rs` (format_transcript anti role-capture + ancla end-of-transcript, truncate_head_tail 8000/32000, sanitize_generated_query FTS5-safe ≤120 chars, render_skills_block prefijo full/recent consolidado, extract_skills_with_llm<R> con sentinel "Nothing to save." y parse tolerante json_utils, retención de candidatos válidos only); `conversation_add/compressor.rs` (tool_call/tool_result >2KB → head/tail char-boundary-safe), `oversize.rs` (chunkMax 80KB/headKeep/tailKeep 20KB, ≥1 mensaje por lado, passthrough cuando head+tail cubren todo — techo documentado TDAM-parity), `archive.rs` (ArchiveStore sobre VantaDB ns skill_archive|skill_tasks/{session}; prepare_archive_payload; trigger_archive con ORDEN archive→task preservado — lección incidente TDAM 2026-07-20; task_id determinista skill-extract-task-{ms}, sin dep uuid), `sink.rs` (SkillCoreSink IDEMPOTENTE doble capa: cursor `{task_id}__applied` patrón MEM-09 + content-hash upsert paridad SkillStore.create MEM-06; StoredSkill name/description/content/content_hash/updated_at_ms en ns skills_extract/{scope}), `worker.rs` (run_skill_extract_once: ghost check archive ausente→dropped, done→AlreadyDone, fallo LLM→task queda pending retryable Principio 4). 28 tests D19 nuevos (13 unit inline + 15 integration tests/skill_extract.rs incl. test de idempotencia del sink: re-run misma task = AlreadyDone + store intacto).
- **result:** OK — 4/4 gates exit 0: cargo check ✅, nextest 268/268 (240 previos + 28 nuevos) ✅, fmt --check ✅, clippy -p vanta-memory --all-targets --no-deps -D warnings ✅ (7 warnings unsafe pre-existentes del core vantadb, fuera de scope)
- **nextAction:** Ninguna para MEM-17. Siguiente tarea del plan: Task 21 (MEM-18 — recall prepend/append + 3 modos); el lead commitea (`feat:` MEM-17)
- **contract:** cargo check -p vanta-memory exit 0; cargo nextest run -p vanta-memory 268/268 passed; cargo fmt --check exit 0; cargo clippy -p vanta-memory --all-targets --no-deps -- -D warnings exit 0
- **invariantes:** marcadores transcript no-naturales + ancla end-of-transcript (anti role-capture); sink idempotente (cursor per-task + content-hash upsert) — re-procesar la misma conversación NUNCA duplica skills; orden trigger archive→task; LLM opcional (fallo→candidatos vacíos/task pending, nunca escritura parcial ni pérdida del archive); sanitización namespace [A-Za-z0-9._/-] ≤128 / keys ≤512 sin NUL; sin unwrap/expect producción; sin deps nuevas; firma genérica <R: LlmRunner>
- **deuda:** desviaciones ponytail documentadas (cola Redis/pool/wire HTTP no portados — MEM-16 PipelineWorker provee dispatch; sink escribe a namespace propio con paridad lógica MEM-06 — wiring directo contra SkillStore del core cuando MEM-35 lo exponga; oversize passthrough cuando head+tail cubren todo, techo heredado de TDAM)
- **queda_pendiente:** commit por el lead
- **nextTask:** Task 21 (MEM-18 — F4 Recall prepend/append + 3 modos)

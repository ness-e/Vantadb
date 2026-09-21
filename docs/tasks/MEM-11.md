# MEM-11: F4 L1 dedup 2 fases (store/update/merge/skip)

## Metadata
- **Plan file:** `docs/plans/2026-08-18-vanta-memory.md` (Task 14, líneas 224-229)
- **Fuente:** plan file Task 14 (MEM-11)
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🔴
- **Tipo:** Rust
- **Turns estimados:** 15-30
- **Creado:** 2026-08-20T23:30
- **last-synced:** 2026-08-20T23:30
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 (6/6 steps) — 2026-08-20T23:55

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `vanta-memory` (MEM-16/17 orquestación consumirá `run_l1_dedup`/`batch_dedup`; MEM-10 ya devuelve memories normalizadas) |
| Callees | `core::abstractions::{LlmRunner, LlmRunParams, ExtractedMemory, MemoryRecord, DedupDecision, DedupAction, MemoryType}`, `core::prompts::{PromptMode, l1_dedup}`, `offload::local_llm::parsers::{json_utils::extract_json, l1_parser::normalize_type}`, SDK `vantadb::sdk::{VantaEmbedded, VantaMemoryInput, VantaMemoryListOptions, VantaMemoryListPage, VantaMemoryMetadata, VantaMemoryRecord, VantaValue}` |
| Implicaciones | `core/record/mod.rs` gana `l1_reader`/`l1_writer`/`l1_dedup`; `core/prompts/mod.rs` gana `l1_dedup`; ningún contrato existente cambia; dedup es LLM-optional (Principio 4 — runner falla → store-all, nunca pierde datos); persistencia vía SDK VantaDB (Principio 2 — namespace `l1/<session>`, key = record id); tests existentes no se ven afectados |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `vanta-memory/src/core/abstractions/types.rs` (370: MemoryRecord/DedupDecision/DedupAction/ExtractedMemory/MemoryType), `vanta-memory/src/core/abstractions/llm_runner.rs` (248: LlmRunner/LlmRunParams/LlmError), `vanta-memory/src/core/record/l1_extractor.rs` (245: extract_l1_memories/L1ExtractorConfig), `vanta-memory/src/core/record/mod.rs` (6), `vanta-memory/src/core/prompts/mod.rs` (11), `vanta-memory/src/core/prompts/l1_extraction.rs` (253: PromptMode/format_extraction_prompt/epoch_ms_to_rfc3339), `vanta-memory/src/core/conversation/l0_recorder.rs` (318: patrón sanitize namespace/key + put/get/list + now_ms), `vanta-memory/src/lib.rs` (45), `vanta-memory/Cargo.toml` (33), `vanta-memory/tests/l0_capture.rs` (168: patrón tests D19 con VantaEmbedded InMemory), `vanta-memory/tests/l1_extractor.rs` (257: CapturingRunner fake), `vanta-memory/src/offload/local_llm/parsers/json_utils.rs` (271: extract_json), `vanta-memory/src/offload/local_llm/parsers/l1_parser.rs` (309: normalize_type), `vanta-memory/src/offload/mod.rs` (9), `vanta-memory/src/offload/local_llm/mod.rs` (4), `vanta-memory/src/offload/local_llm/parsers/mod.rs` (8), `src/sdk/api.rs` (put:217, get:428, delete:503, list:601), `src/sdk/types.rs` (VantaMemoryInput/Record/ListOptions/ListPage), TDAM `MC/core/record/l1-dedup.ts` (408), `MC/core/record/l1-reader.ts` (247), `MC/core/record/l1-writer.ts` (365), `MC/core/prompts/l1-dedup.ts` (236), task file MEM-10.md (158)
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** `vanta-memory` → `vantadb` (SDK put/get/delete/list para persistencia L1); nuevos módulos `core::record::{l1_reader,l1_writer,l1_dedup}`, `core::prompts::l1_dedup`; dependencias existentes serde/serde_json/thiserror/tracing (sin deps nuevas)
- **Archivos que referencian a los editados (referencias entrantes):** ninguno — todos módulos nuevos; solo se editan `core/record/mod.rs` y `core/prompts/mod.rs` (agregan `pub mod`)
- **Veredicto impacto:** bajo — solo se agregan módulos nuevos y 2 líneas de `pub mod` en módulos existentes; cero archivos existentes modificados en su lógica

## Contrato
"`cargo check -p vanta-memory` pasa, `cargo nextest run -p vanta-memory` pasa (incluye tests dedicados de dedup), `cargo fmt --check` pasa, `cargo clippy -p vanta-memory --all-targets --no-deps -- -D warnings` pasa, y el comportamiento específico es: (1) dedup 2 fases — fase 1 candidate recall sin LLM (keyword overlap heurístico sobre records L1 persistidos de la sesión), fase 2 batch LLM judgment en UNA call (`task_id: "l1-conflict-detection"`) con fallback store-all; (2) las 4 acciones se aplican correctamente — store (put nuevo), update/merge (delete targets + put merged con version bump y merged_* fields), skip (no-op); (3) runner falla o parse falla → todas las memories van a store (Principio 4, nunca pierde datos); (4) el parse tolerante usa `json_utils::extract_json` + `l1_parser::normalize_type` (deuda MEM-10 resuelta — no usa `complete_json` estricto); (5) prompts reescritos en inglés (Principio 7)"

## Invariantes de dominio (handoff - MUST)

- **Invariantes a preservar:** (1) LLM opcional — si `LlmRunner::run` falla, dedup degrada a store-all sin propagar error ni perder memories (Principio 4); (2) prompts **reescritos en inglés** (Principio 7 — NO traducir el chino de Kenty); (3) una sola call LLM (task_id `l1-conflict-detection`), JSON array de decisiones; (4) persistencia SOLO vía SDK VantaDB (Principio 2) — namespace `l1/<session>`, key = record id sanitizado, payload = MemoryRecord serializado; update/merge borran targets con `delete`; (5) sin `unwrap()`/`expect()` en código nuevo; (6) tipos normalizados con `l1_parser::normalize_type` (merged_type inválido → None, no rompe la decisión); (7) sin deps nuevas; (8) sanitización namespace `[A-Za-z0-9._/-]` ≤128 bytes, keys ≤512 sin NUL (patrón l0_recorder)
- **Comandos de verificación:** `cargo check -p vanta-memory`, `cargo nextest run -p vanta-memory`, `cargo fmt --check`, `cargo clippy -p vanta-memory --all-targets --no-deps -- -D warnings` — todos exit 0
- **Deuda pendiente:** candidate recall es keyword-overlap heurístico (sin vector search — `vanta-memory` no tiene embeddings LLM-free); MEM-19 consolidará sanitize utils; el orquestador MEM-16 integrará `run_l1_dedup` con `extract_l1_memories`

## Recitation (canónico - estructura única)

- **activeGoal:** ✅ COMPLETED — MEM-11: F4 L1 dedup 2 fases (store/update/merge/skip)
- **lastAction:** Implementación completa: 4 módulos nuevos (prompts/l1_dedup.rs, record/l1_reader.rs, record/l1_writer.rs, record/l1_dedup.rs) + reexports en mod.rs + tests D19 (tests/l1_dedup.rs, 12 tests) + helpers sanitize/now_ms pub(crate) en l0_recorder
- **result:** ✅ — verify mecánico: cargo check ✅, nextest 101/101 ✅, fmt --check ✅, clippy -D warnings ✅ (0 warnings del crate)
- **nextAction:** nil (tarea cerrada; orquestador MEM-16 integra `run_l1_dedup` con `extract_l1_memories`)
- **contract:** ✅ cumplido — dedup 2 fases con fallback store-all (Principio 4), 4 acciones aplicadas, parse tolerante con extract_json+normalize_type (deuda MEM-10 resuelta), prompts en inglés (Principio 7)
- **nextTask:** Task 15 (MEM-12 — F4 Contrato META + nodo escena)

## Deuda técnica (Regla 6 - MUST)

**Saldo neto de deuda por PR:** Sin deuda

> No se introduce deuda nueva: 4 archivos nuevos + 2 líneas de `pub mod` en módulos existentes. Sin dependencias nuevas. El recall heurístico keyword-overlap es deliberadamente naive (sin embeddings) con techo conocido documentado — misma degradación que TDAM cuando no hay vector/FTS disponible (l1-dedup.ts:89-97: skip dedup → store-all).

## Definition of Done (contrato multi-nivel - P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato arriba: cargo check + nextest (tests dedup D19) + fmt + clippy con `-D warnings`, todos exit 0 |
| **Commit** | Commit atómico (~100 líneas/slice), conventional commit (`feat:` MEM-11), verificación mecánica (no auto-reporte) — commit lo hace vanta-lead, no este worker |
| **Release** | No aplica esta iteración (tarea de feature intermedia, sin release) — justificado: `vanta-memory` es `publish = false`, aún en construcción MEM-09..18 |

**Gate:** se marca COMPLETED solo si pasa el nivel Task (los niveles Commit/Release quedan en manos de vanta-lead).

## Herramientas necesarias
- cargo (check, nextest, fmt, clippy)
- codegraph_explore (blast radius ya ejecutado vía lectura directa)
- TDAM clone `C:\Users\Eros\AppData\Local\Temp\opencode\tdam` @ `97f9465`

## Investigation Notes
- **Diseño del port (TDAM l1-dedup.ts/l1-reader.ts/l1-writer.ts → Rust, no copia literal):**
  1. **2 fases:** fase 1 = candidate recall SIN LLM (leer records de la sesión vía `list`, keyword overlap heurístico → top-k candidatos por memory); fase 2 = batch LLM judgment en UNA call (`task_id: "l1-conflict-detection"`) que recibe pool unificado de candidatos + memories nuevas y devuelve JSON array de `DedupDecision`.
  2. **Fallbacks (Principio 4):** si no hay candidatos → todas `store`; si `runner.run` falla → todas `store`; si el parse tolerante falla → todas `store`. Nunca propaga error, nunca pierde memories.
  3. **Writer:** `write_memory` — skip → `Ok(None)`; store → `put` nuevo (id = `decision.record_id` o generado `m_{now}_{counter}`); update/merge → `delete` targets + `put` merged con `version = max(targets)+1` y `merged_*` fields (content/type/priority/timestamps).
  4. **Parse deuda MEM-10:** usar `json_utils::extract_json::<Vec<Value>>` + walk tolerante con `normalize_type` para `merged_type` — NO `complete_json` estricto.
  5. **Prompts:** reescritos en inglés (Principio 7) — `get_conflict_detection_system_prompt(mode)` + `format_batch_conflict_prompt(matches)` con pool unificado de candidatos (dedup por record_id).
  6. **Tipos:** `CandidateMatch { record_id, memory, candidates }` en prompts/l1_dedup.rs (como TDAM lo define en prompts); `PendingMemory { record_id, memory }` para el pipeline.
- **Decisiones de diseño:** funciones libres con `&VantaEmbedded` (componibles, sin ownership problem como L0Recorder); `L1Error` enum en `core/record/l1_writer.rs` (Vanta/Serde) reexportado desde `core/record/mod.rs`; ids determinísticos `m_{now_ms}_{idx}` sin dep crypto.

## Steps

### Step 1 — Discovery + task file
- [x] Leer plan Task 14, MEM-10.md, TDAM l1-dedup/reader/writer/prompt, SDK api/types, módulos vanta-memory
- [x] Verificar task file MEM-11.md no existe → crear (este archivo, Impacto mapeado Regla 0)
- **Gate:** ✅ registro en task file antes de tocar código

### Step 2 — `core/prompts/l1_dedup.rs` (prompts inglés + CandidateMatch)
- [ ] Crear `vanta-memory/src/core/prompts/l1_dedup.rs`: `CandidateMatch { record_id: String, memory: ExtractedMemory, candidates: Vec<MemoryRecord> }`, `get_conflict_detection_system_prompt(mode: PromptMode) -> String`, `format_batch_conflict_prompt(matches: &[CandidateMatch]) -> String`
- [ ] Update `vanta-memory/src/core/prompts/mod.rs`: `pub mod l1_dedup;` + reexport `CandidateMatch`, `get_conflict_detection_system_prompt`, `format_batch_conflict_prompt`
- **Gate:** `cargo check -p vanta-memory` ✅

### Step 3 — `core/record/l1_reader.rs` (lectura sesión + candidate recall)
- [ ] Crear `vanta-memory/src/core/record/l1_reader.rs`: `L1Reader` (wrapper `&VantaEmbedded` + session), `read_records() -> Result<Vec<MemoryRecord>, L1Error>` (list namespace `l1/<session>`, parse payload), `recall_candidates(content, top_k) -> Result<Vec<MemoryRecord>, L1Error>` (keyword overlap, LLM-free)
- [ ] Helper sanitize namespace/key (patrón l0_recorder; `[A-Za-z0-9._/-]` ≤128 / ≤512 sin NUL) — local al módulo (MEM-19 consolidará)
- **Gate:** `cargo check -p vanta-memory` ✅

### Step 4 — `core/record/l1_writer.rs` (aplicación de decisiones)
- [ ] Crear `vanta-memory/src/core/record/l1_writer.rs`: `L1Error` enum, `write_memory(db, session, memory, decision, now) -> Result<Option<MemoryRecord>, L1Error>`, `generate_memory_id(now, idx)`, `apply_dedup_batch(...)` conveniencia (itera decisiones, devuelve records escritos)
- [ ] skip → None; store → put; update/merge → delete targets + put merged (version bump, merged_* fields, timestamps union)
- **Gate:** `cargo check -p vanta-memory` ✅

### Step 5 — `core/record/l1_dedup.rs` (pipeline 2 fases)
- [ ] Crear `vanta-memory/src/core/record/l1_dedup.rs`: `PendingMemory`, `prepare_pending(memories) -> Vec<PendingMemory>`, `batch_dedup<R: LlmRunner>(...) -> Vec<DedupDecision>` (fase 1 recall → fase 2 una call LLM → parse tolerante), `parse_batch_result(raw) -> Vec<DedupDecision>` (extract_json + normalize_type), `run_l1_dedup<R>(...) -> Result<Vec<MemoryRecord>, L1Error>` pipeline unificado (reader → recall → batch_dedup → write)
- [ ] Update `vanta-memory/src/core/record/mod.rs`: `pub mod l1_reader; pub mod l1_writer; pub mod l1_dedup;` + reexports
- **Gate:** `cargo check -p vanta-memory` ✅

### Step 6 — Tests D19 + verify completo
- [ ] Crear `vanta-memory/tests/l1_dedup.rs`: (a) recall heurístico encuentra candidatos y top-k; (b) batch_dedup sin candidatos → store-all; (c) runner falla → store-all; (d) parse tolerante (JSON roto → store-all, campos faltantes → defaults, merged_type inválido → None); (e) e2e write_memory store/update/merge/skip con DB in-memory real; (f) prompt contiene pool unificado; (g) apply_dedup_batch version bump + timestamps union
- [ ] Verify: `cargo check -p vanta-memory`, `cargo nextest run -p vanta-memory`, `cargo fmt --check`, `cargo clippy -p vanta-memory --all-targets --no-deps -- -D warnings`
- [ ] CIERRE: `campaign_update_task_state` taskId=14 completed + recitation canónica; respuesta final con bloque RESULTADO §7 + resumen ≤200 tokens
- **Gate:** todos exit 0
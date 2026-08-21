# AUD-E2E: test de flujo completo L0→L1→L2→L3→recall

## Metadata
- **Fuente:** auditoría post-campaña P27 — hueco de testing (módulos con tests aislados, sin encadenamiento e2e)
- **Esfuerzo:** 🟢
- **Tipo:** Rust (crate `vanta-memory`, solo tests)
- **Creado:** 2026-08-21
- **Estado:** ✅ COMPLETED (sin commit — instrucción explícita de NO commitear)

## Impacto mapeado (Regla 0)

- **Archivos leídos:** `tests/l1_extractor.rs` (257), `tests/l0_capture.rs` (168), `tests/l1_dedup.rs` (422), `tests/recall.rs` (344), `tests/pipeline_manager.rs` (449), `src/services/pipeline_worker.rs` (`MemoryTaskHandler::run_l1/run_l2/run_l3`), `src/core/record/l1_extractor.rs` (task_id `l1-extraction`), `src/core/record/l1_dedup.rs` (`batch_dedup`, `decision_from_value`), `src/core/record/l1_writer.rs` (`apply_dedup_batch`), `src/core/persona/persona_trigger.rs` + `utils/checkpoint.rs` (triggers), `src/core/scene/scene_index.rs` + `scene_extractor.rs`
- **Referencias hacia dentro:** el test consume la superficie pública existente (`L0Recorder`, `MemoryTaskHandler`, `perform_auto_recall`, `read_session_records`, `list_scenes`, `get_persona`)
- **Referencias entrantes:** ninguna — archivo nuevo de tests, cero cambios a src/
- **Veredicto impacto:** nulo en producción; 1 archivo nuevo `vanta-memory/tests/e2e_flow.rs`

## Contrato

"`cargo nextest run -p vanta-memory` (361 previos + 3 nuevos) pasa; `cargo fmt --check` pasa; `cargo clippy -p vanta-memory --all-targets --no-deps -- -D warnings` pasa."

## Steps

### Step 1 — Discovery + patrón
- [x] Leer los 5 suites de tests existentes para copiar setup (DB in-memory, runners fake scripted)
- [x] Mapear el flujo real vía codegraph: `run_l1` → `extract_l1_segments` → `run_l1_dedup`; `run_l2` → `extract_scenes_with_llm`; `run_l3` → trigger P2 cold-start → `generate_persona`

### Step 2 — tests/e2e_flow.rs (3 tests)
- [x] **Happy path:** L0 `record_turn` → handler L1/L2/L3 (runner scripted por `task_id`) → 1 memory + 1 scene + persona → `perform_auto_recall` inyecta prepend (memories) + append (persona)
- [x] **Degradación Principio 4:** runner falla en `l1-extraction` → handle devuelve Err sin panic, L0 intacto (2 mensajes), cero escrituras parciales, recall devuelve `Ok(None)`, y un pase sano posterior recupera el flujo completo
- [x] **Idempotencia:** replay del mismo turno → cursor L0 lo rechaza; segundo pase L1/L2 → dedup `skip` (echo del record_id real, patrón MergeEcho de l1_dedup.rs) y escena UPDATE (heat 1→2), nunca duplicados
- **Gate:** ✅ 3/3 pasan

### Step 3 — Verify + cierre
- [x] `cargo nextest run -p vanta-memory` — 364/364 ✅
- [x] `cargo fmt --check` ✅ · `cargo clippy -p vanta-memory --all-targets --no-deps -- -D warnings` ✅
- [x] Task file (este). NO se llama campaign_update_task_state (tarea fuera del plan MCP)

## Bugs encontrados durante implementación
- Ninguno de integración: las firmas y el estado se propagan correctamente entre capas. Únicos ajustes fueron del propio test (tipo de retorno `Option<RecallResult>` y formato clippy `doc_lazy_continuation`). El pipeline encadena limpio.

## Deuda técnica (Regla 6)

Sin deuda nueva (solo tests). Cobertura e2e no incluye fallo de LLM en L2/L3 individualmente (la degradación por etapa ya está cubierta en los unit tests de cada módulo).

## Recitation (canónico)

- **activeGoal:** AUD-E2E — demostrar el flujo killer end-to-end con LLM mockeado
- **lastAction:** creado `vanta-memory/tests/e2e_flow.rs` (3 tests); verify completo exit 0
- **result:** ✅
- **nextAction:** ninguna — commit pendiente del lead si aplica
- **contract:** nextest -p vanta-memory 364/364 ✅; fmt --check ✅; clippy -D warnings ✅
- **invariantes:** sin deps nuevas; determinista (sin sleeps, sin red); NO se tocó el core `vantadb` ni src/ de vanta-memory
- **deuda:** ninguna relevante
- **queda_pendiente:** commit por el lead (instrucción: NO commitear en esta tarea)

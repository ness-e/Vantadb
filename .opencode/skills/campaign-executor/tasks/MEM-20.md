# MEM-20: F4 Cursor persistente por sesión

## Metadata
- **Plan file:** `docs/plans/2026-08-18-vanta-memory.md` (Task 23)
- **Fuente:** plan file Task 23 (MEM-20)
- **Esfuerzo:** 🟡
- **Prioridad:** 🔴
- **Tipo:** Rust (crate `vanta-memory`)
- **Creado:** 2026-08-20
- **Estado:** ✅ COMPLETED (pendiente commit del lead)

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `pipeline-full.md` (247), plan file Task 23, task files `MEM-19.md` (89, plantilla) + `MEM-09.md` (patrón cursor L0); TDAM `MemoryCore/src/offload/state-manager.ts` (460 completo), `storage.ts` (664 completo), `hooks/after-tool-call.ts` (594 completo); crate: `offload/mod.rs` (9), `offload/types.rs` (144 — `OffloadEntry`/`ToolPair`/`PluginState` YA definidos por MEM-08b con `last_offloaded_tool_call_id`), `core/conversation/l0_recorder.rs` (318 — patrón hermano de cursor persistente: namespace separado, key `__cursor`, JSON payload, fallback), `utils/sanitize.rs` (1-80 — re-exports pub(crate) de `sanitize_component`/`sanitize_key`/`now_ms`), `lib.rs` (45), `Cargo.toml` (33 — sin deps nuevas disponibles más allá de serde/serde_json/thiserror/tracing/vantadb/tempfile-dev); patrón de tests con DB: `core/scene/scene_index.rs:220-227` (`VantaEmbedded::open_with_config(InMemory)`)
- **Referencias hacia dentro:** los 3 módulos nuevos consumen `offload::types::{OffloadEntry, ToolPair, PluginState}` (ya públicos), `utils::sanitize::{sanitize_component, sanitize_key, now_ms}` (re-exports pub(crate)), `vantadb::sdk::{VantaEmbedded, VantaMemoryInput, VantaMemoryListOptions, VantaMemoryListPage, VantaMemoryMetadata, VantaValue}`, `vantadb::error::VantaError`
- **Referencias entrantes:** ninguna hoy — módulos nuevos; únicas ediciones a archivos existentes: `offload/mod.rs` (agregar `pub mod state_manager; pub mod storage; pub mod hooks;`) + nuevo `offload/hooks/mod.rs`. NO se toca `offload/local_llm/` ni su wiring
- **Veredicto impacto:** bajo — 4 archivos 100% nuevos (`state_manager.rs`, `storage.rs`, `hooks/mod.rs`, `hooks/after_tool_call.rs`) + wiring aditivo en `offload/mod.rs`; cero callers rotos, cero archivos del core `vantadb` tocados

## Contrato

"`cargo check -p vanta-memory` pasa; tests dedicados de cursor (D19) pasan (`cargo nextest run -p vanta-memory`), incluido test de idempotencia del cursor (re-procesar el mismo tool call no duplica); `cargo fmt --check` pasa; `cargo clippy -p vanta-memory --all-targets --no-deps -- -D warnings` pasa."

## Diseño (puente TDAM → Rust, decisiones)

| Pieza TDAM | Acción MEM-20 |
|---|---|
| `state-manager.ts` (estado en memoria + state.json) | `state_manager.rs`: `OffloadStateManager` sobre `VantaEmbedded` — `PluginState` persistido como JSON en `offload_state/<session>` key `__state` (análogo al cursor L0 de MEM-09); accesores de cursor `last_offloaded_tool_call_id` con load-on-open + fallback default |
| `storage.ts` (JSONL + refs + mmds + registry) | `storage.rs`: SOLO el núcleo — entradas `OffloadEntry` en `offload/<session>` keyed por `tool_call_id` sanitizado; append con dedup por existencia previa (get-before-put); lectura paginada. Refs/MMDs/registry/JSONL-defense NO portados (VantaDB store reemplaza archivos; sanitizers ya consolidados) |
| `after-tool-call.ts` (buffer + thresholds + L3) | `after_tool_call.rs`: LLM-free — umbral de tamaño del resultado (bytes serializados); si supera → construye `OffloadEntry` (summary placeholder determinístico truncado; el summary real lo genera L1 después), persiste, avanza cursor. Idempotencia doble: cursor + dedup por key |

## Invariantes de dominio (handoff - MUST)

1. Sin deps nuevas; sin unwrap/expect en código de producción; errores tipados `#[non_exhaustive]`.
2. Cursor persistente por sesión en namespace separado (`offload_state/<session>`), nunca mezclado con las entradas (`offload/<session>`) — patrón MEM-09.
3. Idempotencia D19: re-procesar el mismo `tool_call_id` no duplica entradas ni retrocede el cursor.
4. Sanitización: session vía `sanitize_component(128, false)`, keys vía `sanitize_key` (≤512, sin NUL ni `/`).
5. LLM-free (Principio 4): el hook nunca bloquea por LLM; summary es placeholder hasta que L1 corra.
6. NO tocar `offload/local_llm/` ni el core `vantadb`.

## Steps

### Step 1 — Discovery + task file
- [x] Leer TDAM (state-manager/storage/after-tool-call completos) + APIs del crate
- [x] Crear task file (este) con Impacto mapeado Regla 0
- **Gate:** ✅ registro antes de tocar código

### Step 2 — offload/state_manager.rs + wiring
- [x] `OffloadStateManager`: load/save `PluginState` (key `__state`, ns `offload_state/<session>`), accesores de cursor
- [x] Wiring aditivo en `offload/mod.rs` (+ re-export `OffloadError`, derive `Default` en `PluginState`)
- **Gate:** ✅ `cargo check -p vanta-memory` exit 0

### Step 3 — offload/storage.rs
- [x] `OffloadStorage`: `append_entry` (dedup get-before-put), `read_entries` paginado, `has_entry`
- **Gate:** ✅ `cargo check -p vanta-memory` exit 0

### Step 4 — offload/hooks/after_tool_call.rs
- [x] Hook LLM-free: umbral de tamaño → `OffloadEntry` → storage → cursor; skip reasons tipados (`SkipReason::{AlreadyProcessed, BelowThreshold}`)
- **Gate:** ✅ `cargo check -p vanta-memory` exit 0

### Step 5 — Tests D19 + verify completo + cierre
- [x] Tests: persistencia de cursor across reopen, umbral, idempotencia (re-procesar no duplica), sanitización de keys, corrupt-payload fallback, aislamiento de sesiones — 14 tests D19 nuevos, suite 350/350
- [x] Verify: cargo check + nextest + fmt --check + clippy -D warnings — todos exit 0
- [x] CIERRE: campaign_update_task_state taskId=23 completed + recitation; bloque RESULTADO §7
- **Gate:** ✅ verify todo exit 0

## Bugs encontrados durante implementación
- Helper de test `hook()` devolvía managers por valor mientras el hook los tomaba prestados (E0515/E0382) → helper devuelve `(state, storage)` y el hook se construye inline.
- Clippy `field_reassign_with_default` en test de preservación de campos → struct-update syntax.

## Deuda técnica (Regla 6)

Sin deuda nueva neta. Desviaciones documentadas: `summary` de la entrada es placeholder determinístico truncado (200 chars) hasta que L1 lo genere; `result_ref` es una referencia lógica `offload/<session>/<tool_call_id>` (el payload completo vive en el propio registro VantaDB); las capas refs/MMD/registry/JSONL-defense de TDAM storage.ts NO se portan (VantaDB store las reemplaza).

## Recitation (canónico)

- **activeGoal:** MEM-20: F4 Cursor persistente por sesión
- **lastAction:** state_manager + storage + after_tool_call implementados, 14 tests D19 nuevos, verify 4/4 exit 0; task cerrada
- **result:** ✅
- **nextAction:** ninguna — tarea completada; commit pendiente del lead
- **contract:** cargo check -p vanta-memory ✅; cargo nextest run -p vanta-memory ✅ (350/350); cargo fmt --check ✅; cargo clippy -p vanta-memory --all-targets --no-deps -- -D warnings ✅
- **invariantes:** cursor en namespace separado (`offload_state/<session>`) nunca mezclado con entradas (`offload/<session>`); idempotencia doble (cursor + dedup por key); sin deps nuevas; sin unwrap/expect en producción; LLM-free (Principio 4)
- **deuda:** summary placeholder hasta L1; capas refs/MMD/registry de TDAM no portadas (sin caller)
- **queda_pendiente:** commit por el lead
- **nextTask:** Task 24 (MEM-21, siguiente del plan)

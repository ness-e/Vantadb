# MEM-14: F4 Strategy UPDATE>MERGE>CREATE + heat + soft-delete

## Metadata
- **Plan file:** `docs/plans/2026-08-18-vanta-memory.md` (Task 17, líneas 245-250)
- **Fuente:** plan file Task 17 (MEM-14)
- **Esfuerzo:** 🟡
- **Prioridad:** 🔴
- **Tipo:** Rust (crate `vanta-memory`)
- **Turns estimados:** 15-30
- **Creado:** 2026-08-20
- **last-synced:** 2026-08-20
- **Estado:** ✅ COMPLETED (verify 4/4 gates exit 0)
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 — 4/4 steps completados

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `vanta-memory` (MEM-16 orquestación invocará `extract_scenes`/`extract_scenes_with_llm` para el flush L2; MEM-15 persona consume `list_scenes` (navegación — ahora excluye soft-deleted); MEM-21 scene_read/list/query leen el store — soft-deleted filtrados por `list_scenes`/`current_scene`, recuperables vía `get_scene`) |
| Callees | `core::scene::scene_tools::{execute_scene_tool, SceneToolCall, SceneToolError, MAX_*}` (MEM-13 — UPDATE/CREATE vía dispatcher, validación boundary), `core::scene::scene_index::{get_scene, list_scenes, soft_delete_scene, write_scene_block, SceneError}` (MEM-12 — MERGE/soft-delete con heat explícito), `core::scene::scene_format::{SceneBlock, SOFT_DELETE_MARKER}` (MEM-12 — formato, flag `deleted`), `core::abstractions::{SceneMeta, SceneIndexEntry}`, `core::conversation::{now_ms}` (pub(crate)), `core::prompts::l1_extraction::{epoch_ms_to_rfc3339, PromptMode}` — todo consumo, cero duplicación |
| Implicaciones | `core/scene/mod.rs` gana `pub mod scene_extractor; pub mod filename_normalizer;` + re-exports (aditivo); `core/prompts/mod.rs` gana `pub mod scene_extraction;` (aditivo); `scene_format.rs` gana `SOFT_DELETE_MARKER` + campo `deleted` en `SceneBlock` (serde default — retrocompatible con records existentes); `scene_index.rs` gana `soft_delete_scene` + `write_scene_block` (rename de helper privado) + filtro de deleted en `list_scenes`/`current_scene`; `scene_tools.rs` gana visibilidad `pub(crate)` en validators (reuso); NO se toca el core `vantadb` ni sus 7 warnings unsafe |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `pipeline-full.md` (247), `docs/plans/2026-08-18-vanta-memory.md` (349: Task 17 + D19 + Principios 2/4/7 + referencias TDAM), task files MEM-13.md (105: API exacta tools, deuda "soft-delete → MEM-14", template) / MEM-12.md (116: API exacta scene_index/scene_format, deuda "colisión de keys por sanitize → MEM-14 filename-normalizer"), `vanta-memory/src/core/scene/scene_format.rs` (103: SceneBlock a editar), `vanta-memory/src/core/scene/scene_index.rs` (214: upsert_scene/get_scene/list_scenes/current_scene/write_block a editar), `vanta-memory/src/core/scene/scene_tools.rs` (242: validators a pub(crate), execute_scene_tool entry point), `vanta-memory/src/core/scene/mod.rs` (23: wiring), `vanta-memory/src/core/mod.rs` (28), `vanta-memory/src/core/prompts/mod.rs` (18: wiring prompt), `vanta-memory/src/core/prompts/l1_extraction.rs` (253: PromptMode + epoch_ms_to_rfc3339 patrón prompts), `vanta-memory/src/core/abstractions/types.rs` (370: SceneMeta/SceneIndexEntry/SceneSegment), `vanta-memory/src/core/abstractions/llm_runner.rs` (248: LlmRunner/complete_json/degrade pattern), `vanta-memory/src/core/conversation/l0_recorder.rs` (318: sanitize_component/sanitize_key/now_ms L106-125,313), `vanta-memory/src/core/conversation/mod.rs` (10), `vanta-memory/Cargo.toml` (33: sin deps nuevas, features llm-driver/mock), `vanta-memory/tests/scene_tools.rs` (188: patrón tests D19 open_db InMemory), TDAM `MC/core/scene/scene-extractor.ts` (604: flow extract + emptyExtraction L509-516 + soft-delete cleanup L298-359 + estrategia UPDATE/MERGE/CREATE), TDAM `MC/core/prompts/scene-extraction.ts` (572: system/user prompt, estrategia, heat, [DELETED] marker), TDAM `MC/core/scene/filename-normalizer.ts` (195: normalizeSceneFilename/isNormalized/resolveUnique), TDAM `MC/core/scene/scene-format.ts` (75: SceneBlockMeta)
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** `core/scene/scene_extractor.rs` (nuevo) → scene_tools (execute_scene_tool/SceneToolCall/MAX_*), scene_index (get_scene/list_scenes/soft_delete_scene/write_scene_block), scene_format (SceneBlock/SOFT_DELETE_MARKER), abstractions (SceneIndexEntry/SceneMeta), conversation (now_ms), prompts l1_extraction (epoch_ms_to_rfc3339/PromptMode), prompts scene_extraction (build_scene_extraction_prompt); `core/scene/filename_normalizer.rs` (nuevo) → std solo; `core/prompts/scene_extraction.rs` (nuevo) → l1_extraction PromptMode; `scene_format.rs`/`scene_index.rs`/`scene_tools.rs` editados → consumidos por extractor
- **Archivos que referencian a los editados (referencias entrantes):** `scene_format.rs` (SceneBlock + deleted) → `scene_index.rs` (upsert/soft_delete construyen blocks), `scene_tools.rs` (SceneBlock en resultados), tests existentes `tests/scene.rs`/`tests/scene_tools.rs` (construyen SceneBlock vía `new()` — retrocompatible); `scene_index.rs` (list_scenes ahora filtra deleted) → tests `tests/scene.rs` (no usan deleted — no afectados); `scene_tools.rs` (validators pub(crate)) → sin cambio de API pública
- **Veredicto impacto:** bajo — cambio aditivo dentro de `vanta-memory` (3 archivos nuevos + 3 módulos existentes con líneas añadidas + wiring); cero archivos del core `vantadb` tocados; API pública de `vantadb` intacta; records existentes de `scene/<session>` siguen parseando (`deleted` con serde default)

## Contrato
"`cargo check -p vanta-memory` pasa, `cargo nextest run -p vanta-memory` pasa (incluye tests dedicados de strategy escena D19), `cargo fmt --check` pasa, `cargo clippy -p vanta-memory --all-targets --no-deps -- -D warnings` pasa, y el comportamiento específico es: (1) estrategia UPDATE>MERGE>CREATE determinista en `scene_extractor.rs` — `decide_strategy(extraction, existing) -> SceneStrategy` (coincide scene_name → UPDATE heat=old+1; solapa vía `merge_sources` → MERGE heat=sum+1 + soft-delete de fuentes; nueva → CREATE heat=1; content vacío/whitespace → Skip) + `apply_strategy` que ejecuta UPDATE/CREATE vía `execute_scene_tool` (dispatcher MEM-13, validación boundary) y MERGE/soft-delete vía scene_index (heat explícito que las tools no pueden expresar); (2) `emptyExtraction` — extracción vacía o runner LLM fallido NO sobreescribe el store (Principio 4: degrada sin perder datos); (3) soft-delete — mecanismo definido: flag `deleted: bool` en `SceneBlock` (scene_format.rs) + `SOFT_DELETE_MARKER = "[DELETED]"` como content; `soft_delete_scene` en scene_index marca sin borrar (idempotente, META preservado); `list_scenes`/`current_scene` excluyen deleted, `get_scene` los devuelve (recuperación); write resucita (deleted=false); (4) `filename_normalizer.rs` — `normalize_scene_name` (TDAM filename-normalizer.ts adaptado a record store: whitespace→'-', strip puntuación/slashes, collapse separadores, fallback 'scene', CJK preservado) + `is_normalized_scene_name`; (5) `prompts/scene_extraction.rs` — prompt L2 reescrito EN INGLÉS (Principio 7, NO traducir chino): system (chat + work, estrategia UPDATE>MERGE>CREATE, heat, naming, soft-delete, contrato JSON con merge_sources) + user (memories JSON, summaries, timestamp, file list); (6) sin unwrap/expect en producción, sin deps nuevas, errores en inglés"

## Invariantes de dominio (handoff - MUST)

- **Invariantes a preservar:** (1) LLM opcional (Principio 4) — runner falla → `extract_scenes_with_llm` degrada a `success:false, empty_extraction:true` SIN escribir al store; (2) persistencia SOLO vía store VantaDB (Principio 2) reusando scene_index/scene_tools (Principio 5 — no duplicar lógica de store); (3) soft-delete = marcar sin borrar (flag `deleted` en el payload del block, META preservado — heat/created/updated intactos); `list_scenes`/`current_scene` excluyen deleted; `get_scene` los devuelve (recovery); upsert resucita; (4) heat: CREATE=1, UPDATE=old+1 (MEM-12 upsert_scene), MERGE=sum(sources)+target_heat+1 con `saturating_add` (sin overflow panic); (5) UPDATE/CREATE pasan por el dispatcher `execute_scene_tool` (MEM-13 deuda) — validación boundary reusada; MERGE/soft-delete usan scene_index (heat explícito); (6) sin `unwrap()`/`expect()` en producción (core-engine R-3); (7) sin deps nuevas; (8) sanitización namespace `[A-Za-z0-9._/-]` ≤128 bytes, keys ≤512 sin NUL (patrón l0_recorder — el store la aplica en write); (9) `#[non_exhaustive]` en enums públicos (api-contract R-6); (10) errores en inglés (Principio 7); (11) output del LLM tratado como input no confiable (LLM05 — validar en el boundary)
- **Comandos de verificación:** `cargo check -p vanta-memory`, `cargo nextest run -p vanta-memory`, `cargo fmt --check`, `cargo clippy -p vanta-memory --all-targets --no-deps -- -D warnings` — todos exit 0
- **Deuda pendiente:** el loop agente con tools del LLM (read/write/edit reales en sandbox) es de MEM-16 (orquestación) — MEM-14 expone `extract_scenes_with_llm` con contrato JSON (no tool-calls); el sweep de cleanup de markers `[DELETED]` escritos vía tools (TDAM Phase 5) se implementa junto al loop de MEM-16; colisión residual de keys CJK (sanitize_key mapea CJK→'_') pre-existente de MEM-12 — el normalizer resuelve la deuda documentada (a/b vs a_b: '/' se elimina); sync `docs/api/` del módulo → MEM-38 (docs gate pre-release), anotado

## Recitation (canónico - estructura única)

- **activeGoal:** MEM-14: F4 Strategy UPDATE>MERGE>CREATE + heat + soft-delete
- **lastAction:** Implementada la estrategia L2 completa: `scene_format.rs` (+`SOFT_DELETE_MARKER`, campo `deleted: bool` serde-default retrocompatible, `is_deleted()`), `scene_index.rs` (`write_scene_block` pub, `soft_delete_scene` idempotente META-preservado, filtro deleted en `list_scenes`/`current_scene`), `scene_tools.rs` (validators → pub(crate)), nuevos `filename_normalizer.rs` (port TDAM sin dir-strip ni .md — `/` es drop char, resuelve deuda MEM-12 a/b→ab≠a_b), `scene_extractor.rs` (`decide_strategy` pura + `apply_strategy`: UPDATE/CREATE vía execute_scene_tool, MERGE vía scene_index con heat=saturating(target+Σfuentes)+1 y soft-delete de fuentes en orden TDAM target-primero, emptyExtraction guard, `extract_scenes_with_llm<R: LlmRunner>` genérico — trait no dyn-compatible — con degrade Principio 4), `prompts/scene_extraction.rs` (system chat/work EN inglés con estrategia+contrato JSON, interpolación `{common}`), wiring mod.rs ×2; tests D19 `tests/scene_strategy.rs` (17 integration) + unit tests en los 5 módulos
- **result:** ✅ 4/4 gates exit 0: cargo check ✅, nextest 177/177 (129 previos + 48 nuevos) ✅, fmt --check ✅, clippy -D warnings ✅ (7 warnings unsafe pre-existentes del core vantadb, fuera de scope)
- **nextAction:** Ninguna para MEM-14. Siguiente tarea del plan: Task 18 (MEM-15 — F4 Persona first/incremental + triggers); el lead commitea (`feat:` MEM-14)
- **contract:** cargo check -p vanta-memory exit 0; cargo nextest run -p vanta-memory 177/177 passed; cargo fmt --check exit 0; cargo clippy -p vanta-memory --all-targets --no-deps -- -D warnings exit 0
- **nextTask:** Task 18 (MEM-15 — F4 Persona first/incremental + triggers)

## Deuda técnica (Regla 6 - MUST)

**Saldo neto de deuda por PR:** Sin deuda

> No se introduce deuda nueva: 3 archivos nuevos (scene_extractor.rs, filename_normalizer.rs, prompts/scene_extraction.rs) + ediciones aditivas en scene_format/scene_index/scene_tools + 2 wirings + 1 archivo de tests (tests/scene_strategy.rs). Sin dependencias nuevas. El campo `deleted` con `#[serde(default)]` mantiene retrocompatibilidad con records existentes (deuda MEM-12 de colisión de keys resuelta con filename_normalizer; soft-delete diferido de MEM-13 implementado).

## Definition of Done (contrato multi-nivel - P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato arriba: cargo check vanta-memory + nextest vanta-memory (tests strategy escena D19) + fmt + clippy con `-D warnings`, todos exit 0 |
| **Commit** | Commit atómico (~100 líneas/slice), conventional commit (`feat:` MEM-14), verificación mecánica (no auto-reporte) — commit lo hace vanta-lead, no este worker |
| **Release** | No aplica esta iteración (tarea de feature intermedia, sin release) — justificado: `vanta-memory` es `publish = false`, aún en construcción MEM-09..18 |

**Gate:** se marca COMPLETED solo si pasa el nivel Task (los niveles Commit/Release quedan en manos de vanta-lead).

## Herramientas necesarias
- cargo (check, nextest, fmt, clippy)
- codegraph_explore (blast radius ejecutado vía lectura directa de scene_format/scene_index/scene_tools/abstractions/prompts)
- TDAM clone `C:\Users\Eros\AppData\Local\Temp\opencode\tdam` @ `97f9465`

## Investigation Notes
- **Diseño del port (TDAM scene-extractor.ts + scene-extraction.ts + filename-normalizer.ts → Rust, no copia literal):**
  1. **Soft-delete (mecanismo definido):** TDAM: el LLM escribe el marker `[DELETED]` como contenido y el extractor hace `fs.unlink` (Phase 5, `scene-extractor.ts:298-359`). En el record store NO hay unlink: el mecanismo es **flag `deleted: bool` en el payload de `SceneBlock`** (scene_format.rs, `#[serde(default)]` — records viejos parsean como no-deleted). `SOFT_DELETE_MARKER = "[DELETED]"` se conserva como content (paridad TDAM + compat futuro con el loop de tools de MEM-16). `scene_index::soft_delete_scene` marca el block (deleted=true, content=marker, META intacto — paridad con el cleanup TDAM que no toca META), idempotente. `list_scenes`/`current_scene` filtran deleted (navegación); `get_scene` los devuelve (recovery); `upsert_scene` escribe deleted=false (resucita). Consistente con scene_index porque el flag vive en el payload que scene_index lee/escribe.
  2. **Estrategia UPDATE>MERGE>CREATE determinista** (`scene_extractor.rs`): TDAM deja la decisión al LLM vía tools (`scene-extraction.ts` Phase 2: UPDATE preferido > MERGE > CREATE último recurso). El port separa **decisión** (`decide_strategy`, pura, testeable D19) de **ejecución** (`apply_strategy`): el LLM emite extracciones JSON `{scene_name, summary, content, merge_sources}` (su juicio de solapamiento) y la capa determinista aplica heat/soft-delete. UPDATE/CREATE se ejecutan vía `execute_scene_tool(SceneToolCall::Write)` (deuda MEM-13: "el dispatcher es el entry point que MEM-14 usará") — validación boundary + heat old+1 de upsert_scene. MERGE no puede pasar por write_scene_tool (heat = sum+1 ≠ old+1): usa `scene_index::write_scene_block` (helper privado hecho pub) + `soft_delete_scene` de cada fuente. empty content → Skip (no crear escenas vacías — paridad con el reject de content vacío del write tool).
  3. **`emptyExtraction`** (`scene-extractor.ts:509-516`): extracción vacía (0 items) → `empty_extraction: true`, cero writes; runner LLM fallido (Principio 4) → degrada a `success:false, error, empty_extraction:true`, cero writes. Nunca sobreescribe el store con una extracción vacía.
  4. **Heat:** CREATE=1 / UPDATE=old+1 (semántica MEM-12 upsert_scene) / MERGE=target_heat + Σ(source_heat) + 1 (TDAM `scene-extraction.ts` Heat Management: "sum(所有相关block的heat) + 1") con `saturating_add`.
  5. **filename_normalizer.rs:** port de `filename-normalizer.ts` SIN extensión `.md` (record store, no filesystem): `normalize_scene_name` — strip dir components, whitespace runs (incl. NBSP/full-width) → '-', remove puntuación/slashes/quotes/brackets (lista TDAM), collapse separadores, trim, fallback `"scene"`, CJK preservado (paridad TDAM). `is_normalized_scene_name` = identity check. Resuelve la deuda MEM-12: `a/b` → `ab` ≠ `a_b` (el '/' se elimina, no colisiona con '_').
  6. **prompts/scene_extraction.rs:** reescritura EN INGLÉS de `scene-extraction.ts` (Principio 7 — NO traducir chino): system prompt (chat + work vía PromptMode existente) con estrategia UPDATE>MERGE>CREATE, heat, naming rules, soft-delete marker, y el **contrato JSON** de salida (divergencia documentada: TDAM emite tool-calls; el port emite decisiones JSON que la capa determinista ejecuta — el loop de tools es MEM-16). User prompt: memories JSON + summaries + timestamp + file list. `build_scene_extraction_prompt(params) -> {system_prompt, user_prompt}`.
  7. **LLM glue** (`extract_scenes_with_llm`): memories → prompt → `runner.complete_json::<Vec<SceneExtraction>>` → `extract_scenes`. Runner error → degrade sin data loss (Principio 4). Input `SceneMemoryInput {content, created_at, id}` (paridad TDAM `extract()` L141).
- **Decisiones de diseño:** validators de scene_tools pasan a `pub(crate)` para reuso en MERGE (sin duplicar lógica boundary); `write_block` privado de scene_index renombrado a `pub write_scene_block` (primitiva de escritura exacta, usada por upsert/soft_delete/extractor); MERGE escribe target primero, luego soft-delete fuentes (orden TDAM — un fallo de delete deja el target escrito, no pérdida); el extractor NO normaliza nombres en `apply` (decide_strategy normaliza al decidir); errores: `SceneExtractorError` con variantes `Scene`/`Tool`/`Invalid` (no se agregan variantes a SceneError/SceneToolError de MEM-12/13).

## Fases explícitas - SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — evaluado: trust boundary (output de LLM = input no confiable, LLM05; persistence store). Mitigación: (1) validación boundary reusada de scene_tools (pub(crate)) en el path MERGE (scene_name no vacío/NUL/≤512, content no vacío/≤1 MiB, summary ≤4096); (2) UPDATE/CREATE pasan por execute_scene_tool que valida; (3) soft-delete idempotente sin borrado físico (recuperable vía get_scene); (4) sin tools destructivas nuevas; (5) errores tipados sin exponer internals; (6) sin deps nuevas, sin FFI, sin unwrap/expect. Se delega la auditoría final a vanta-audit (Review gate).
- [x] **PERFORMANCE** — evaluado: estrategia O(n) por batch (list_scenes + get_scene por merge source); no es hot path del motor (flush L2 puntual). Sin profiling requerido.

## Steps

### Step 1 — Discovery + task file
- [x] Leer plan Task 17, MEM-12/13 (template + API exacta + deudas), TDAM scene-extractor.ts/scene-extraction.ts/filename-normalizer.ts/scene-format.ts, scene_format/scene_index/scene_tools/mod/prompts/abstractions/conversation, tests scene_tools.rs, Cargo.toml
- [x] Verificar task file MEM-14.md no existe → crear (este archivo, Impacto mapeado Regla 0)
- [x] Decidir diseño (Investigation Notes) + verificar blast radius
- **Gate:** ✅ registro en task file antes de tocar código

### Step 2 — Formato/índice/tools + normalizer (primitivas soft-delete)
- [x] Editar `core/scene/scene_format.rs`: `SOFT_DELETE_MARKER`, `deleted: bool` en SceneBlock (serde default), `new()` deleted=false, `is_deleted()`; tests unitarios roundtrip deleted
- [x] Editar `core/scene/scene_index.rs`: `write_block` → `pub write_scene_block`; `soft_delete_scene` (idempotente, META preservado); `list_scenes`/`current_scene` filtran deleted; tests unitarios soft-delete
- [x] Editar `core/scene/scene_tools.rs`: validators → `pub(crate)` (reuso en extractor)
- [x] Crear `core/scene/filename_normalizer.rs`: `normalize_scene_name` + `is_normalized_scene_name` + tests unitarios
- **Gate:** `cargo check -p vanta-memory` ✅

### Step 3 — `core/scene/scene_extractor.rs` + `core/prompts/scene_extraction.rs` + wiring
- [x] Crear `scene_extractor.rs`: `SceneExtraction` (serde), `SceneStrategy` (Update/Create/Merge/SoftDelete/Skip), `decide_strategy` (pura), `apply_strategy` (UPDATE/CREATE vía execute_scene_tool; MERGE vía scene_index con heat sum+1 + soft-delete fuentes; SoftDelete; Skip), `extract_scenes` (batch, emptyExtraction), `extract_scenes_with_llm` (degrade Principio 4), `SceneExtractionResult`/`SceneApplyResult`/`SceneAction`/`SceneExtractorError` (#[non_exhaustive]); unit tests (decide, merge heat, degrade runner, serde extraction)
- [x] Crear `core/prompts/scene_extraction.rs`: `SceneExtractionPromptParams/Result`, `build_scene_extraction_prompt` (chat + work en inglés, contrato JSON); unit tests (strategy rules, JSON contract, user prompt sections)
- [x] Editar `core/scene/mod.rs` + `core/prompts/mod.rs`: `pub mod` + re-exports
- **Gate:** `cargo check -p vanta-memory` ✅

### Step 4 — Tests D19 + verify completo + cierre
- [x] Crear `vanta-memory/tests/scene_strategy.rs`: emptyExtraction no sobreescribe; CREATE heat=1; UPDATE heat=2 created preservado; MERGE heat=sum+1 + fuentes soft-deleted; MERGE en target existente incluye heat del target; soft-delete marker; soft-delete missing noop; list/current excluyen deleted; get_scene recupera deleted; write resucita; decide_strategy casos
- [x] Verify: `cargo check -p vanta-memory` ✅, `cargo nextest run -p vanta-memory` ✅ (129 previos + nuevos), `cargo fmt --check` ✅, `cargo clippy -p vanta-memory --all-targets --no-deps -- -D warnings` ✅ exit 0
- [x] CIERRE: `campaign_update_task_state` taskId=17 completed + recitation canónica; respuesta final con bloque RESULTADO §7
- **Gate:** verify todo exit 0 ✅
# MEM-12: F4 Contrato META + nodo escena (ancla L2)

## Metadata
- **Plan file:** `docs/plans/2026-08-18-vanta-memory.md` (Task 15, líneas 230-235)
- **Fuente:** plan file Task 15 (MEM-12)
- **Esfuerzo:** 🟡
- **Prioridad:** 🔴
- **Tipo:** Rust (core `vantadb` + crate `vanta-memory`)
- **Turns estimados:** 15-30
- **Creado:** 2026-08-20
- **last-synced:** 2026-08-20
- **Estado:** ⏳ PENDING (en ejecución)
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 6/6 steps — ver Steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `vanta-memory` (MEM-14 strategy UPDATE>MERGE>CREATE consumirá `SceneNodeStore` core + `upsert_scene`; MEM-21 scene_read/list/query consumirá `list_scenes`/`current_scene`; Studio Inspector KV genérico lee los nodos `scene:*` de la partición InternalMetadata — contrato 2 del plan) |
| Callees | Core: `crate::backend::{BackendPartition, BackendWriteOp}`, `crate::error::{Result, VantaError}`, `crate::storage::StorageEngine`, `entity::mod` (helpers validate_scope/validate_key privados del padre). vanta-memory: `vantadb::sdk::{VantaEmbedded, VantaMemoryInput, VantaMemoryListOptions, VantaMemoryListPage, VantaMemoryMetadata}`, `core::abstractions::{SceneMeta, SceneIndexEntry}`, `core::conversation::{now_ms, sanitize_component, sanitize_key}` (pub(crate)), `core::prompts::l1_extraction::epoch_ms_to_rfc3339` (pub) |
| Implicaciones | **PRIMER task F4 que toca el core `vantadb`** (MEM-08..11 fueron 100% vanta-memory). `src/entity/mod.rs` gana `pub mod scene` (aditivo, semver-safe); `vanta-memory/src/core/mod.rs` gana `pub mod scene`; ningún contrato existente cambia; `SceneMeta`/`SceneIndexEntry` YA existen en `core/abstractions/types.rs` (MEM-08b) — se reusan, no se duplican; los 7 warnings unsafe pre-existentes de `src/storage/*` NO se tocan; tests existentes no se ven afectados |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `pipeline-full.md` (247), `docs/plans/2026-08-18-vanta-memory.md` (348: Task 15 + D2/D4 + contrato 2), `.opencode/rules/core-engine.md` (40: R-3 sin unwrap/expect, R-4 unsafe), `.opencode/rules/api-contract.md` (64: R-1 doc→símbolo real, R-6 #[non_exhaustive], R-8 lógica en core), task files MEM-10.md (158)/MEM-11.md (111: formato template, patrón funciones libres), TDAM `MC/core/scene/scene-format.ts` (75: SceneBlockMeta {created,updated,summary,heat} + parseSceneBlock/formatSceneBlock), TDAM `MC/core/scene/scene-index.ts` (137: SceneIndexEntry + read/write/syncSceneIndex), `src/entity/mod.rs` (253: EntityStore patrón InternalMetadata, validate_scope/validate_key, keys `entity:{ns}:{col}::{id}`), `src/entity/tests.rs` (258: patrón in_memory_engine StorageEngine::open_with_config), `src/lib.rs` (203: línea 85 `pub mod entity;` sin feature gate), `vanta-memory/src/core/abstractions/types.rs` (370: SceneMeta L204-220 {created,updated,summary,heat:u32}, SceneIndexEntry L222-239), `vanta-memory/src/core/abstractions/mod.rs` (20: re-exports), `vanta-memory/src/core/conversation/l0_recorder.rs` (318: sanitize_component/sanitize_key/now_ms pub(crate), patrón namespace+key+put), `vanta-memory/src/core/conversation/mod.rs` (10: re-export pub(crate) sanitize/now_ms), `vanta-memory/src/core/record/l1_reader.rs` (208: funciones libres + l1_namespace), `vanta-memory/src/core/prompts/l1_extraction.rs` (253: epoch_ms_to_rfc3339 pub L55), `vanta-memory/src/core/prompts/mod.rs` (18), `vanta-memory/src/core/mod.rs` (25), `vanta-memory/src/lib.rs` (45), `vanta-memory/Cargo.toml` (33: sin deps nuevas, vantadb default-features=false), `vanta-memory/tests/l0_capture.rs` (168: patrón tests D19 open_db InMemory)
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** core `src/entity/scene.rs` (nuevo) → `entity/mod.rs` (`pub mod scene` + re-export + `#[cfg(test)] mod scene_tests`); vanta-memory `core/scene/{mod,scene_format,scene_index}.rs` (nuevos) → `core/mod.rs` (`pub mod scene`); dependencias existentes serde/serde_json/thiserror/tracing (sin deps nuevas)
- **Archivos que referencian a los editados (referencias entrantes):** ninguno — todos módulos nuevos; solo se editan `src/entity/mod.rs` y `vanta-memory/src/core/mod.rs` (agregan `pub mod`)
- **Veredicto impacto:** bajo-medio — cambio aditivo en core (nuevo submódulo `entity::scene`, 1 línea `pub mod` en entity/mod.rs, 1 línea `pub mod` en core/mod.rs); cero archivos existentes modificados en su lógica; API pública de `vantadb` intacta (sin renames/removes); compile de TODO el workspace afectado solo por el crate nuevo

## Contrato
"`cargo check -p vanta-memory` pasa, `cargo check -p vantadb` pasa (core intacto), `cargo nextest run -p vanta-memory` pasa (incluye tests dedicados de META/escena D19), `cargo fmt --check` pasa, `cargo clippy -p vanta-memory --all-targets --no-deps -- -D warnings` pasa, y el comportamiento específico es: (1) contrato META `{created,updated,summary,heat}` con serde roundtrip (`SceneMeta` de MEM-08b + `SceneBlock {scene_name, meta, content}` en scene_format.rs); (2) nodo escena en el grafo core: `SceneNodeStore` en la partición `InternalMetadata` con keys `scene:{namespace}:{session}::{scene_name}` (reuso exacto del patrón EntityStore, D4 — NO mecanismo nuevo); (3) índice de escenas por sesión LLM-free vía SDK (namespace `scene/<session>`, key = scene_name sanitizado, payload = `SceneBlock` JSON): `upsert_scene` (CREATE heat=1 / UPDATE heat=old+1, created preservado, updated=now), `get_scene`, `list_scenes` (Vec<SceneIndexEntry> ordenado heat desc + updated desc), `current_scene` (max updated); (4) sin unwrap/expect en producción, sin deps nuevas, sanitización `[A-Za-z0-9._/-]` ≤128 bytes keys ≤512 sin NUL (patrón l0_recorder)"

## Invariantes de dominio (handoff - MUST)

- **Invariantes a preservar:** (1) LLM-free — el índice de escenas no llama LLM (Principio 4; el ancla de L2 es determinista); (2) persistencia SOLO vía store VantaDB (Principio 2): nodo core = partición `InternalMetadata` (D4), índice = namespace SDK `scene/<session>`; (3) `SceneMeta`/`SceneIndexEntry` ya existen en abstractions (MEM-08b) — reusar, NO duplicar tipos; (4) sin `unwrap()`/`expect()` en código nuevo (core-engine R-3); (5) sin deps nuevas; (6) sanitización namespace `[A-Za-z0-9._/-]` ≤128 bytes, keys ≤512 sin NUL (patrón l0_recorder via core::conversation); (7) validación de componentes sin `{`,`}`,`:` (reuso validate_key de entity/mod.rs); (8) timestamps ISO 8601 vía `epoch_ms_to_rfc3339` existente (sin chrono); (9) enums públicos con `#[non_exhaustive]` (api-contract R-6)
- **Comandos de verificación:** `cargo check -p vanta-memory`, `cargo check -p vantadb`, `cargo nextest run -p vanta-memory`, `cargo fmt --check`, `cargo clippy -p vanta-memory --all-targets --no-deps -- -D warnings` — todos exit 0
- **Deuda pendiente:** sync `docs/api/` del nuevo módulo core `vantadb::entity::scene` (Regla 3 AGENTS.md) → MEM-38 (docs gate pre-release), anotado; colisión de keys por sanitize (scene_name `a/b` y `a_b` → misma key) → MEM-14 filename-normalizer; la estrategia MERGE=sum+1 de heat es de MEM-14 (MEM-12 solo CREATE/UPDATE); `delete_scene` NO se crea (YAGNI — soft-delete es MEM-14); el nodo core y el índice SDK son dos vistas del mismo dominio: la sincronización del write path la define MEM-14 (UPDATE>MERGE>CREATE)

## Recitation (canónico - estructura única)

- **activeGoal:** MEM-12: F4 Contrato META + nodo escena (ancla L2)
- **lastAction:** Implementación completa: (1) core `src/entity/scene.rs` SceneNode/SceneNodePage/SceneNodeStore (partición InternalMetadata, keys `scene:{ns}:{session}::{scene_name}`, reuso validate_key/validate_scope del padre) + `src/entity/scene_tests.rs` (6 tests) + `pub mod scene` + re-exports en entity/mod.rs; (2) `vanta-memory/src/core/scene/scene_format.rs` SceneBlock (scene_name + SceneMeta reusada + content) serde snake_case + index_entry() (3 unit tests); (3) scene_index.rs SceneError #[non_exhaustive] + scene_namespace/upsert_scene (CREATE heat=1, UPDATE heat+1 created preservado)/get_scene/list_scenes (heat desc + updated desc)/current_scene (max updated) reusando sanitize_component/sanitize_key/now_ms/epoch_ms_to_rfc3339 existentes; (4) tests D19 vanta-memory/tests/scene.rs (9 tests). Verify: check vanta-memory ✅, check vantadb ✅, nextest vanta-memory 116/116 ✅ (9 nuevos scene), fmt --check ✅, clippy -p vanta-memory -D warnings ✅ (0 exit). NOTA: MCP campaign_update_task_state bloqueado por WIP corrupto (16-24 + MEM-08b reportados in-progress aunque el plan file los muestra PENDING) — ver RESULTADO.
- **result:** ✅ COMPLETO — contrato cumplido (verify mecánico todo exit 0), sin deps nuevas, sin unwrap/expect, sin tocar storage/vector/wal
- **nextAction:** Cerrar tarea vía MCP (bloqueado por estado WIP del server, documentado) → orquestador asigna Task 16 (MEM-13)
- **contract:** cargo check -p vanta-memory ✅ + cargo check -p vantadb ✅ + cargo nextest run -p vanta-memory ✅ (116/116) + cargo fmt --check ✅ + cargo clippy -p vanta-memory --all-targets --no-deps -- -D warnings ✅
- **nextTask:** Task 16 (MEM-13 — F4 Tools read/write/edit sandboxed + store)

## Deuda técnica (Regla 6 - MUST)

**Saldo neto de deuda por PR:** Sin deuda

> No se introduce deuda nueva: 2 archivos core nuevos + 1 línea `pub mod` + 3 archivos vanta-memory nuevos + 1 línea `pub mod`. Sin dependencias nuevas. La derivación del índice por listing (sin archivo índice denormalizado) evita el bug de sync de TDAM scene_index.json (writeSceneIndex manual) — el store es la única fuente de verdad. Colisión por sanitize documentada con techo conocido (MEM-14 la resuelve con filename-normalizer).

## Definition of Done (contrato multi-nivel - P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato arriba: cargo check vanta-memory + vantadb, nextest vanta-memory (tests META/escena D19), fmt, clippy con `-D warnings`, todos exit 0 |
| **Commit** | Commit atómico (~100 líneas/slice), conventional commit (`feat:` MEM-12), verificación mecánica (no auto-reporte) — commit lo hace vanta-lead, no este worker |
| **Release** | No aplica esta iteración (tarea de feature intermedia, sin release) — justificado: `vanta-memory` es `publish = false`, aún en construcción MEM-09..18; el cambio core es aditivo (nuevo submódulo público) sin breaking |

**Gate:** se marca COMPLETED solo si pasa el nivel Task (los niveles Commit/Release quedan en manos de vanta-lead).

## Herramientas necesarias
- cargo (check, nextest, fmt, clippy)
- codegraph_explore (blast radius ejecutado vía lectura directa — el query "entity.rs InternalMetadata" no resolvió, se leyó src/entity/mod.rs completo)
- TDAM clone `C:\Users\Eros\AppData\Local\Temp\opencode\tdam` @ `97f9465`

## Investigation Notes
- **Diseño del port (TDAM scene-format.ts/scene-index.ts → Rust, no copia literal):**
  1. **`SceneMeta` ya existe** en `core/abstractions/types.rs` (MEM-08b, L204-220: `{created: String, updated: String, summary: String, heat: u32}` + serde snake_case) con docstring que cita TDAM scene-format.ts. El task pedía "port como struct SceneMeta con serde roundtrip" — ya hecho en MEM-08b. MEM-12 aporta `SceneBlock` (scene_name + meta + content) y el nodo core.
  2. **Nodo escena en grafo core (D2/D4):** `src/entity/scene.rs` replica EXACTO el patrón de `EntityStore` (partición `BackendPartition::InternalMetadata`, serde JSON, scan_partition_prefix, write_backend_batch delete). Keys `scene:{namespace}:{session}::{scene_name}` — distinguibles de `entity:*` en el mismo scan. `SceneNode` = identidad + campos planos del contrato META (sin struct meta anidado: evita duplicar el tipo SceneMeta en el core; el contrato viaja plano en JSON snake_case). CRUD tonto (set reemplaza wholesale, sin now() interno — el caller L2 computa created/updated/heat); validación reusa `validate_key` del padre (privado, visible a hijo). `SceneNodeStore` es un ancla que L2 (MEM-13/14) actualizará y que Studio lee por el Inspector KV genérico (contrato 2 del plan).
  3. **scene_format.rs:** `SceneBlock { scene_name: String, meta: SceneMeta, content: String }` serde snake_case + `index_entry()` → `SceneIndexEntry` (filename = scene_name). El formato META-delimitado de TDAM (`-----META-START-----` markers) NO se implementa: en VantaDB el bloque persiste como JSON record (payload), no como archivo Markdown — el marcador sería dead code (ponytail). El contrato META viaja en el JSON.
  4. **scene_index.rs — funciones libres** (estilo MEM-11 l1_reader, no struct): `scene_namespace(session)` = `scene/<sanitize_component(session,128,false)>`; `upsert_scene(db, session, scene_name, summary, content)` — CREATE: created=updated=now_iso(), heat=1; UPDATE: created preservado, updated=now_iso(), heat=old+1 (semántica documentada en types.rs "CREATE=1, UPDATE=old+1"; MERGE=sum+1 es de MEM-14). payload = `SceneBlock` JSON; key = `sanitize_key(scene_name)`. `get_scene` (get + parse), `list_scenes` (list paginado + parse + `index_entry` + sort heat desc, updated desc — orden de navegación TDAM), `current_scene` (max by `updated`; lexicográfico válido porque todos los `updated` los genera `epoch_ms_to_rfc3339` con formato fijo ancho — invariante documentado).
  5. **Sin índice denormalizado:** TDAM escribe `scene_index.json` derivado (bug de sync potencial). En VantaDB los records `scene/<session>` SON la fuente de verdad; `list_scenes` deriva las entradas por listing (ponytail rung 2: el dato ya está).
  6. **Reuso de helpers existentes:** `sanitize_component`/`sanitize_key`/`now_ms` (pub(crate) en core::conversation), `epoch_ms_to_rfc3339` (pub en core::prompts::l1_extraction). Cero helpers nuevos duplicados.
  7. **`SceneError`** enum público con `#[non_exhaustive]` (api-contract R-6): `Vanta(#[from] VantaError)` + `Serde(#[from] serde_json::Error)`.
- **Decisiones de diseño:** `SceneNodeStore` en core toma `&StorageEngine` (patrón EntityStore); el índice vanta-memory toma `&VantaEmbedded` (patrón l1_reader). El `namespace` de SceneNodeStore es el tenant (paridad entity_*); el `session` del índice SDK es el session_key L0/L1. Timestamps: `epoch_ms_to_rfc3339(now_ms())` — sin chrono.

## Fases explícitas - SECURITY | PERFORMANCE (P2-07)

- [ ] **SECURITY** — evaluado: el código nuevo persiste en storage (trust boundary). Mitigación: validación de componentes sin `{`,`}`,`:` (reuso validate_key), sanitización de namespace/key (patrón l0_recorder — sin NUL, set acotado), parse de payloads tolerante (registro inválido → skip con tracing, nunca panic). No agrega dependencias. No toca FFI. Input del LLM NO entra en el nodo core (el core guarda solo META planos; el summary lo provee el caller L1/L2 — MEM-13/14 auditarán el sanitize de contenido). Se delega la auditoría final a vanta-audit (Review gate).
- [ ] **PERFORMANCE** — evaluado: list_scenes es O(n) listing + parse (mismo orden que read_session_records de MEM-11); no es hot path del motor (consultas puntuales de navegación). Sin profiling requerido.

## Steps

### Step 1 — Discovery + task file
- [x] Leer plan Task 15, rules core-engine/api-contract, MEM-10/11 (template), TDAM scene-format/index, src/entity/mod.rs + tests, l0_recorder/l1_reader/prompts, types.rs, lib.rs, Cargo.toml
- [x] Verificar task file MEM-12.md no existe → crear (este archivo, Impacto mapeado Regla 0)
- [x] Decidir diseño (Investigation Notes) + verificar blast radius
- **Gate:** ✅ registro en task file antes de tocar código

### Step 2 — Core: `src/entity/scene.rs` + tests + wiring
- [x] Crear `src/entity/scene.rs`: `SceneNode` (namespace/session_id/scene_name/created/updated/summary/heat), `SceneNodePage`, `SceneNodeStore<'a>` con `scene_node_set` (reemplaza wholesale), `scene_node_get`, `scene_node_delete`, `scene_node_list` (prefix scan + sort scene_name + paginate); keys `scene:{namespace}:{session}::{scene_name}`; reuso `validate_key` del padre
- [x] Crear `src/entity/scene_tests.rs`: (a) set/get roundtrip; (b) get missing → None; (c) set reemplaza wholesale (upsert refresca campos); (d) validate_key rechaza `:`/`{}`; (e) list prefix isolation entre sesiones + paginación; (f) delete true/false
- [x] Editar `src/entity/mod.rs`: `pub mod scene;` + re-exports + `#[cfg(test)] mod scene_tests;`
- **Gate:** `cargo check -p vantadb` ✅ (6/6 scene tests en nextest lib)

### Step 3 — `vanta-memory/src/core/scene/scene_format.rs`
- [x] Crear `scene_format.rs`: `SceneBlock { scene_name, meta: SceneMeta, content }` serde snake_case + `new()` + `index_entry() -> SceneIndexEntry`; docstring cita TDAM scene-format.ts; 3 tests unitarios (roundtrip snake_case, index_entry mapping, compat scene_name con SceneSegment)
- **Gate:** `cargo check -p vanta-memory` ✅

### Step 4 — `vanta-memory/src/core/scene/scene_index.rs`
- [x] Crear `scene_index.rs`: `SceneError` (#[non_exhaustive]), `scene_namespace(session)`, `upsert_scene` (CREATE/UPDATE heat), `get_scene`, `list_scenes` (Vec<SceneIndexEntry> heat desc/updated desc), `current_scene` (max updated); reuso sanitize/now_ms/epoch_ms_to_rfc3339; sin unwrap/expect
- [x] Crear `mod.rs` de scene + editar `vanta-memory/src/core/mod.rs`: `pub mod scene;` + re-exports
- **Gate:** `cargo check -p vanta-memory` ✅

### Step 5 — Tests D19 `vanta-memory/tests/scene.rs`
- [x] Crear tests: (a) SceneBlock serde roundtrip + index_entry; (b) upsert CREATE heat=1 + created=updated; (c) upsert UPDATE heat=old+1 + created preservado + updated bump; (d) get missing → None; (e) list_scenes sort heat desc + updated desc; (f) session isolation (2 sesiones no se mezclan); (g) current_scene max updated; (h) scene_name con caracteres inválidos → sanitizado y recuperable; (i) SceneError displayable
- **Gate:** `cargo nextest run -p vanta-memory` ✅ 116/116

### Step 6 — Verify completo + cierre
- [x] Verify: `cargo check -p vanta-memory` ✅, `cargo check -p vantadb` ✅, `cargo nextest run -p vanta-memory` ✅ (116/116, 9 scene nuevos), `cargo fmt --check` ✅, `cargo clippy -p vanta-memory --all-targets --no-deps -- -D warnings` ✅ (exit 0)
- [ ] CIERRE: `campaign_update_task_state` taskId=15 completed + recitation canónica — **BLOQUEADO por MCP**: el server reporta WIP corrupto en tasks 16-24 + MEM-08b (in-progress) aunque el plan file las muestra ⏳ PENDING; `campaign_update_task_state` rechaza con "ya hay otra tarea en progreso" y no ofrece force. Se documenta en RESULTADO para que el orquestador limpie el estado WIP y cierre la tarea. El trabajo está COMPLETO y verificado.
- **Gate:** verify todo exit 0 ✅ (el gate de cierre MCP queda pendiente de limpieza del server)
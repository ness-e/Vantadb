# MEM-13: F4 Tools read/write/edit sandboxed + store

## Metadata
- **Plan file:** `docs/plans/2026-08-18-vanta-memory.md` (Task 16, líneas 238-243)
- **Fuente:** plan file Task 16 (MEM-13)
- **Esfuerzo:** 🟢
- **Prioridad:** 🔴
- **Tipo:** Rust (crate `vanta-memory`)
- **Turns estimados:** 10-20
- **Creado:** 2026-08-20
- **last-synced:** 2026-08-20
- **Estado:** ✅ COMPLETO (ejecutado y verificado) — cierre MCP pendiente de limpieza WIP corrupto (ver Recitation)
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0/4 steps — ver Steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `vanta-memory` (MEM-14 strategy UPDATE>MERGE>CREATE consumirá `execute_scene_tool`/`SceneToolCall` para el sandbox del LLM L2; MEM-16 orquestación invoca el tool layer; MEM-21 scene_read/list/query sobre el mismo store) |
| Callees | `core::scene::scene_index::{upsert_scene, get_scene, SceneError}` (MEM-12), `core::scene::scene_format::SceneBlock` (MEM-12), `core::abstractions::{SceneMeta}` (MEM-08b) — todo consumo, cero duplicación |
| Implicaciones | `core/scene/mod.rs` gana `pub mod scene_tools;` + re-exports (aditivo); ningún contrato existente cambia; `SceneError` sigue siendo la fuente de errores de storage (wrapped en `SceneToolError`); tests existentes no se ven afectados; NO se toca el core `vantadb` ni sus 7 warnings unsafe |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `pipeline-full.md` (247), `docs/plans/2026-08-18-vanta-memory.md` (349: Task 16 + D19 + Principios 2/4/7 + referencias TDAM), task files MEM-12.md (116: API exacta scene_index/scene_format, template) / MEM-11.md (111: template), `vanta-memory/src/core/scene/scene_index.rs` (214: upsert_scene/get_scene/SceneError/scene_namespace — CONSUMO), `vanta-memory/src/core/scene/scene_format.rs` (103: SceneBlock), `vanta-memory/src/core/scene/mod.rs` (15: a editar, 1 línea pub mod), `vanta-memory/src/core/mod.rs` (28: wiring scene), `vanta-memory/tests/scene.rs` (158: patrón tests D19 open_db InMemory), `vanta-memory/src/core/conversation/l0_recorder.rs` (318, L1-120: sanitize_component/sanitize_key, patrones sanitización), `vanta-memory/src/core/abstractions/types.rs` (vía codegraph: SceneMeta L209-220, SceneIndexEntry L226-239), TDAM `MC/core/scene/scene-extractor.ts` (604 — leídos L1-60 sandbox, L240-309 Phase 4/5, L520-604: sandbox `workspaceDir=scene_blocks/`, sin exec tool, write tool rechaza content vacío/whitespace-only L300-304, system files invisibles), TDAM `MC/core/scene/scene-index.ts` (grep: readSceneIndex/writeSceneIndex — sandbox file invisible)
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** `vanta-memory/src/core/scene/scene_tools.rs` (nuevo) → `scene_index::{upsert_scene, get_scene, SceneError}`, `scene_format::SceneBlock`, `abstractions::SceneMeta`; `core/scene/mod.rs` (`pub mod scene_tools` + re-exports); dependencias existentes serde/serde_json/thiserror (sin deps nuevas)
- **Archivos que referencian a los editados (referencias entrantes):** ninguno — módulo nuevo; solo se edita `core/scene/mod.rs` (agrega `pub mod` + re-exports, 4 líneas)
- **Veredicto impacto:** bajo — cambio aditivo dentro de `vanta-memory` (1 archivo nuevo + 1 módulo existente con líneas añadidas); cero archivos existentes modificados en su lógica; API pública de `vantadb` intacta; compile del workspace afectado solo por el crate

## Contrato
"`cargo check -p vanta-memory` pasa, `cargo nextest run -p vanta-memory` pasa (incluye tests dedicados de tools escena D19), `cargo fmt --check` pasa, `cargo clippy -p vanta-memory --all-targets --no-deps -- -D warnings` pasa, y el comportamiento específico es: (1) tools sandboxed read/write/edit sobre el store de escenas por sesión — `SceneToolCall` (serde, wire LLM: `{"tool": "read|write|edit", ...}`) + dispatcher `execute_scene_tool(db, session, call) -> SceneToolResult`; (2) sandboxing = confinamiento por sesión (ninguna tool acepta namespace — todas derivan `scene/<session>` vía scene_index), sin tool destructiva (delete NO existe — soft-delete es MEM-14), validación de inputs en el boundary (scene_name no vacío/sin NUL/≤512 bytes, content no vacío ni whitespace-only/≤1 MiB, summary ≤4096 bytes) con `SceneToolError::{Invalid, NotFound, Scene}` tipados; (3) `write_scene_tool` = CREATE/UPDATE del store (reusa upsert_scene), `edit_scene_tool` = patch de campos (fetch + merge + upsert; NotFound si no existe; Invalid si ningún campo), `read_scene_tool` = get_scene; (4) sin unwrap/expect en producción, sin deps nuevas, prompts/errores en inglés (Principio 7)"

## Invariantes de dominio (handoff - MUST)

- **Invariantes a preservar:** (1) LLM-free — las tools NO llaman LLM (Principio 4; el tool layer es la capa de herramientas, el orquestador es MEM-16); (2) persistencia SOLO vía store VantaDB (Principio 2) reusando scene_index (Principio 5 — no duplicar lógica de store); (3) sandbox: ninguna tool expone namespace arbitrario (confinamiento por sesión), ninguna operación fuera de `scene/<session>`, sin delete (MEM-14); (4) sin `unwrap()`/`expect()` en código nuevo (core-engine R-3); (5) sin deps nuevas; (6) sanitización namespace `[A-Za-z0-9._/-]` ≤128 bytes, keys ≤512 sin NUL (patrón l0_recorder — validación en el boundary, sanitización en el store); (7) `#[non_exhaustive]` en enums públicos (api-contract R-6); (8) errores en inglés (Principio 7); (9) output del LLM tratado como input no confiable (LLM05 — validar en el boundary, security-and-hardening)
- **Comandos de verificación:** `cargo check -p vanta-memory`, `cargo nextest run -p vanta-memory`, `cargo fmt --check`, `cargo clippy -p vanta-memory --all-targets --no-deps -- -D warnings` — todos exit 0
- **Deuda pendiente:** el dispatcher `execute_scene_tool` es el entry point que MEM-14 usará para la estrategia UPDATE>MERGE>CREATE (heat/soft-delete/emptyExtraction de MEM-14, NO de MEM-13); tamaño de caps (1 MiB content / 4096 summary) son decisiones de boundary documentadas — revisar si MEM-19 sanitize define caps transversales; sync `docs/api/` del módulo → MEM-38 (docs gate pre-release), anotado

## Recitation (canónico - estructura única)

- **activeGoal:** MEM-13: F4 Tools read/write/edit sandboxed + store
- **lastAction:** Creé `vanta-memory/src/core/scene/scene_tools.rs` (SceneToolCall/SceneToolResult serde tagged, SceneToolError #[non_exhaustive], caps 512/4096/1MiB, validators boundary, read/write/edit/execute_scene_tool sobre scene_index) + `pub mod scene_tools` y re-exports en `core/scene/mod.rs` + tests D19 `vanta-memory/tests/scene_tools.rs` (13 tests); cargo fmt aplicado; fix clippy needless_as_bytes
- **result:** ✅ COMPLETO — contrato verificado mecánicamente: `cargo check -p vanta-memory` ✅, `cargo nextest run -p vanta-memory` ✅ 129/129 (13 nuevos scene_tools), `cargo fmt --check` ✅, `cargo clippy -p vanta-memory --all-targets --no-deps -- -D warnings` ✅ exit 0
- **nextAction:** Cierre MCP `campaign_update_task_state` taskId=16 completed — BLOQUEADO por WIP corrupto del server (17-24, MEM-08b, MEM-12, MEM-13 marcadas in-progress; error "ya hay otra tarea en progreso"). El plan file sigue PENDING (correcto); la limpieza del estado MCP la hace vanta-lead. Código listo para commit (vanta-lead ejecuta, conventional `feat:` MEM-13)
- **contract:** cumplido — contrato textual del task file, todos los gates exit 0 (ver Contrato arriba)
- **nextTask:** Task 17 (MEM-14 — F4 Strategy UPDATE>MERGE>CREATE + heat + soft-delete)

## Deuda técnica (Regla 6 - MUST)

**Saldo neto de deuda por PR:** Sin deuda

> No se introduce deuda nueva: 1 archivo nuevo (scene_tools.rs) + 4 líneas en scene/mod.rs + 1 archivo de tests (tests/scene_tools.rs). Sin dependencias nuevas. Las tools son thin wrappers con validación sobre scene_index (MEM-12) — cero lógica de store duplicada.

## Definition of Done (contrato multi-nivel - P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato arriba: cargo check vanta-memory + nextest vanta-memory (tests tools escena D19) + fmt + clippy con `-D warnings`, todos exit 0 |
| **Commit** | Commit atómico (~100 líneas/slice), conventional commit (`feat:` MEM-13), verificación mecánica (no auto-reporte) — commit lo hace vanta-lead, no este worker |
| **Release** | No aplica esta iteración (tarea de feature intermedia, sin release) — justificado: `vanta-memory` es `publish = false`, aún en construcción MEM-09..18 |

**Gate:** se marca COMPLETED solo si pasa el nivel Task (los niveles Commit/Release quedan en manos de vanta-lead).

## Herramientas necesarias
- cargo (check, nextest, fmt, clippy)
- codegraph_explore (blast radius ejecutado: scene_tools/upsert_scene/get_scene/SceneBlock/SceneMeta + wiring core/mod.rs)
- TDAM clone `C:\Users\Eros\AppData\Local\Temp\opencode\tdam` @ `97f9465`

## Investigation Notes
- **Diseño del port (TDAM scene-extractor.ts → Rust, no copia literal):**
  1. **Sandbox TDAM:** `scene-extractor.ts:7-9` — "The LLM is sandboxed — workspaceDir is set to scene_blocks/ so it can ONLY operate on .md scene files. System files (checkpoint, scene_index, persona.md) are physically invisible to the LLM". `:300` — "The LLM has no `exec` tool and cannot run shell commands". `:302` — write tool rechaza content vacío/whitespace-only (así el LLM no "borra" escribiendo vacío). En VantaDB: el sandbox se expresa como (a) confinamiento por sesión — las tools NUNCA aceptan un namespace; todas derivan `scene/<session>` vía scene_index (el namespace es invisible al LLM, como los system files); (b) sin tool destructiva — no hay delete (soft-delete es MEM-14); (c) validación de inputs en el boundary — output del LLM es input no confiable (LLM05); (d) sin analogo de exec — las tools son operaciones puras del store.
  2. **Tipos wire:** `SceneToolCall` enum serde internally-tagged por `tool` (`{"tool":"read","scene_name":"..."}`, `{"tool":"write",...}`, `{"tool":"edit","summary":"...","content":"..."}` con `#[serde(default)]` en Options) — el LLM produce tool calls como JSON, MEM-14 las parsea y despacha. `SceneToolResult` serde tagged por `result` (Read{scene:Option}/Write{scene}/Edit{scene}) — MEM-14 serializa el resultado de vuelta al LLM.
  3. **Funciones libres** (estilo scene_index, no struct): `read_scene_tool` (get_scene), `write_scene_tool` (upsert_scene = CREATE/UPDATE con heat), `edit_scene_tool` (fetch + merge de campos + upsert; NotFound si no existe; Invalid si ningún campo a patchar) + dispatcher `execute_scene_tool`.
  4. **`SceneToolError`** enum público `#[non_exhaustive]` (api-contract R-6): `Scene(#[from] SceneError)` + `Invalid(String)` (boundary validation) + `NotFound(String)` (edit target). NO se agregan variantes a `SceneError` (MEM-12 lo define; wrapper en la tool layer).
  5. **Caps de boundary documentados:** `MAX_SCENE_NAME_BYTES = 512` (límite keys), `MAX_SUMMARY_BYTES = 4096`, `MAX_CONTENT_BYTES = 1 MiB`. Reject (no truncate) — el LLM debe conocer el límite. Reject content vacío/whitespace-only (paridad con write tool TDAM).
  6. **Sin estrategia:** MEM-13 es SOLO la capa de herramientas — no decide UPDATE>MERGE>CREATE ni emptyExtraction (MEM-14). No implementa "agent loop" (MEM-16).
- **Decisiones de diseño:** validación en el boundary (reject NUL/empty/oversize) vs sanitización en el store (sanitize_key reemplaza chars inválidos — preservado de MEM-12, test `scene_name_with_invalid_chars_is_sanitized_but_retrievable`); error precedence en edit: validar campos antes del fetch (NotFound solo para requests válidos).

## Fases explícitas - SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — evaluado: trust boundary (output de LLM = input no confiable, LLM05; persistence store). Mitigación: (1) sandbox por sesión — ninguna tool acepta namespace (confinamiento `scene/<session>`); (2) validación en el boundary — scene_name no vacío/sin NUL/≤512 bytes, content no vacío ni whitespace-only/≤1 MiB, summary ≤4096; (3) sin tool destructiva (delete NO existe en la tool layer); (4) errores tipados sin exponer internals (SceneToolError mapea SceneError de storage); (5) sin deps nuevas, sin FFI, sin unwrap/expect. Se delega la auditoría final a vanta-audit (Review gate).
- [x] **PERFORMANCE** — evaluado: tools son operaciones O(1) del store (get/put) salvo read (no list); no es hot path del motor. Sin profiling requerido.

## Steps

### Step 1 — Discovery + task file
- [x] Leer plan Task 16, MEM-12/11 (template + API exacta), TDAM scene-extractor.ts (sandbox), scene_index/format/mod/core-mod, tests scene.rs, l0_recorder
- [x] Verificar task file MEM-13.md no existe → crear (este archivo, Impacto mapeado Regla 0)
- [x] Decidir diseño (Investigation Notes) + verificar blast radius (codegraph)
- **Gate:** ✅ registro en task file antes de tocar código

### Step 2 — `core/scene/scene_tools.rs` (tools sandboxed + types wire)
- [x] Crear `scene_tools.rs`: `SceneToolError` (#[non_exhaustive]: Scene/Invalid/NotFound), `SceneToolCall` (serde tag=tool: Read/Write/Edit), `SceneToolResult` (serde tag=result), caps MAX_SCENE_NAME_BYTES/MAX_SUMMARY_BYTES/MAX_CONTENT_BYTES, helpers validate_scene_name/validate_content/validate_text, `read_scene_tool`/`write_scene_tool`/`edit_scene_tool`/`execute_scene_tool`; reuso scene_index (upsert_scene/get_scene); sin unwrap/expect
- [x] Editar `core/scene/mod.rs`: `pub mod scene_tools;` + re-exports (read/write/edit/execute, SceneToolCall, SceneToolError, SceneToolResult)
- **Gate:** `cargo check -p vanta-memory` ✅

### Step 3 — Tests D19 `vanta-memory/tests/scene_tools.rs`
- [x] Crear tests: (a) read missing → None; (b) write CREATE heat=1 + retrievable vía read; (c) write UPDATE heat=2 + created preservado + full replace; (d) edit content-only (summary preservado, heat bump); (e) edit summary-only (content preservado); (f) edit missing → NotFound; (g) edit sin campos → Invalid; (h) scene_name vacío → Invalid; (i) NUL en scene_name → Invalid; (j) content oversized → Invalid; (k) content vacío/whitespace → Invalid; (l) wire roundtrip: serde SceneToolCall JSON → execute → serialize SceneToolResult; (m) aislamiento de sesión vía tools
- **Gate:** `cargo nextest run -p vanta-memory` ✅ 129/129 (13 nuevos scene_tools)

### Step 4 — Verify completo + cierre
- [x] Verify: `cargo check -p vanta-memory` ✅, `cargo nextest run -p vanta-memory` ✅ 129/129, `cargo fmt --check` ✅ (fmt aplicado, fix clippy needless_as_bytes), `cargo clippy -p vanta-memory --all-targets --no-deps -- -D warnings` ✅ exit 0
- [x] CIERRE: `campaign_update_task_state` taskId=16 completed INTENTADO → bloqueado por WIP corrupto del server (17-24/MEM-08b/MEM-12/MEM-13 en progreso según MCP, plan file las muestra PENDING); cierre MCP queda para vanta-lead tras limpiar estado; respuesta final con bloque RESULTADO §7
- **Gate:** verify todo exit 0 ✅
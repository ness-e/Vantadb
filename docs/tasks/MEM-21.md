# MEM-21: F4 Tools MCP scene_read/list/query

## Metadata
- **Plan file:** `docs/plans/2026-08-18-vanta-memory.md` (Task 24)
- **Fuente:** plan file Task 24 (MEM-21)
- **Esfuerzo:** 🟡
- **Prioridad:** 🔴
- **Tipo:** Rust (crate `vanta-memory`)
- **Creado:** 2026-08-20
- **Estado:** ✅ COMPLETED (pendiente commit del lead)

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `pipeline-full.md` (247), plan file Task 24, task files `MEM-20.md` (84, plantilla); TDAM `MemoryCore/src/gateway/knowledge-handlers.ts` (155 completo — patrón request→validate→store.method→envelope); crate: `gateway/mod.rs` (3 — stub "Filled in by MEM-21"), `lib.rs` (45 — `pub mod gateway` YA existe), `core/scene/scene_index.rs` (340 completo — `get_scene`/`list_scenes`/`upsert_scene`/`soft_delete_scene`/`read_blocks` privado + tests con `VantaEmbedded::open_with_config(InMemory)`), `core/scene/scene_tools.rs` (249 completo — `SceneToolError`, `validate_scene_name`/`validate_text` pub(crate), límites MAX_*), `core/scene/scene_format.rs` (150 — `SceneBlock.deleted` + `is_deleted()`), `core/record/l1_reader.rs` (208 — `significant_terms`/`overlap_score` pub(crate)), `utils/sanitize.rs` (774), `core/hooks/auto_recall.rs` (patrón de consumo de list_scenes + significant_terms)
- **Referencias hacia dentro:** el módulo nuevo consume `scene_index::{get_scene, list_scenes, read_blocks (a exponer pub(crate)), SceneError}`, `scene_tools::{validate_scene_name, SceneToolError}`, `l1_reader::{significant_terms, overlap_score}`, `abstractions::{SceneIndexEntry, SceneMeta}`
- **Referencias entrantes:** ninguna hoy — módulo nuevo; únicas ediciones a archivos existentes: `gateway/mod.rs` (agregar `pub mod knowledge_handlers;`) + `scene_index.rs` (visibilidad de `read_blocks` → `pub(crate)`, un token). NO se toca el core `vantadb`
- **Veredicto impacto:** bajo — 1 archivo nuevo (`knowledge_handlers.rs`) + wiring aditivo en `gateway/mod.rs` + cambio de visibilidad en `scene_index.rs`; cero callers rotos

## Contrato

"`cargo check -p vanta-memory` pasa; tests dedicados de scene tools (D19) pasan (`cargo nextest run -p vanta-memory`); `cargo fmt --check` pasa; `cargo clippy -p vanta-memory --all-targets --no-deps -- -D warnings` pasa."

## Diseño (puente TDAM → Rust, decisiones)

| Pieza TDAM | Acción MEM-21 |
|---|---|
| `knowledge-handlers.ts` (Zod safeParse → store → envelope) | `knowledge_handlers.rs`: handlers puros tipados serde (request/response structs) sobre `scene_index`; sin servidor HTTP/MCP — la capa de entrada que un server MCP expondrá después. Errores tipados `KnowledgeError` (`#[non_exhaustive]`) reemplazan los envelopes |
| `/v3/knowledge/get` (404 si missing) | `scene_read(db, SceneReadRequest)` → `SceneReadResponse { scene }`; missing o soft-deleted → `KnowledgeError::NotFound` (paridad con `list_scenes` que excluye deleted) |
| `/v3/knowledge/list` | `scene_list(db, SceneListRequest)` → `SceneListResponse { scenes }` = paridad directa con `list_scenes` (heat desc, deleted filtrados) |
| query por keyword (sin endpoint TDAM directo; reuse recall heuristics) | `scene_query(db, SceneQueryRequest { keyword, top_k })` → hits rankeados por overlap de `significant_terms(keyword)` contra content+summary del bloque; deleted excluidos; top_k default 5 |

## Invariantes de dominio (handoff - MUST)

1. Sin deps nuevas; sin unwrap/expect en código de producción; errores tipados `#[non_exhaustive]`.
2. Soft-delete respetado en las 3 tools: read → NotFound, list/query → excluidos (MEM-14).
3. Validación en boundary: session_key no vacío, scene_name vía `validate_scene_name` (≤512, sin NUL), keyword no vacío tras trim.
4. LLM-free (Principio 4): query es keyword-overlap (`significant_terms`), sin embeddings ni LLM.
5. NO tocar el core `vantadb`; lógica vive en `scene_index`/`l1_reader` — handlers son thin wrappers.

## Steps

### Step 1 — Discovery + task file
- [x] Leer TDAM knowledge-handlers + APIs del crate (codegraph_explore)
- [x] Crear task file (este) con Impacto mapeado Regla 0
- **Gate:** ✅ registro antes de tocar código

### Step 2 — gateway/knowledge_handlers.rs + wiring
- [x] Tipos request/response serde + `KnowledgeError`
- [x] Handlers `scene_read` / `scene_list` / `scene_query` (thin wrappers sobre scene_index)
- [x] Wiring aditivo: `gateway/mod.rs` + visibilidad `read_blocks` → pub(crate)
- **Gate:** ✅ `cargo check -p vanta-memory` exit 0

### Step 3 — Tests D19 + verify completo + cierre
- [x] Tests: read ok/not-found/deleted, list paridad heat desc + deleted excluido, query ranking + top_k + keyword vacío rechazado, sanitización scene_name — 10 tests D19 nuevos
- [x] Verify: cargo check + nextest + fmt --check + clippy -D warnings — todos exit 0
- [x] CIERRE: campaign_update_task_state taskId=24 completed + recitation; bloque RESULTADO §7
- **Gate:** ✅ verify todo exit 0

## Bugs encontrados durante implementación
- Tests fallaban por asunción de heats distintos: el seed dejaba todas las escenas con heat=1, así que el tie-break heat-desc no distinguía → seed con doble upsert de "deploy" (heat 2) para orden determinístico.
- Clippy: import `significant_terms` sin uso en no-test + helper `query_terms` muerto → eliminados (ponytail).

## Deuda técnica (Regla 6)

Sin deuda nueva neta. Query es overlap-count naive (mismo techo documentado que `recall_candidates` en l1_reader — upgrade path: vector index cuando el core lo exponga a esta crate). `read_blocks` expuesto como pub(crate) (1 token de visibilidad) para que query lea contenido sin duplicar la paginación.

## Recitation (canónico)

- **activeGoal:** MEM-21 (Task 24): F4 Tools MCP scene_read/list/query — última de F4
- **lastAction:** knowledge_handlers.rs implementado (3 handlers puros tipados serde + KnowledgeError non_exhaustive), wiring en gateway/mod.rs, read_blocks pub(crate); 10 tests D19 nuevos; verify fmt+clippy+nextest 3/3 exit 0; tarea cerrada
- **result:** ✅
- **nextAction:** ninguna — tarea completada; commit pendiente del lead
- **contract:** cargo check -p vanta-memory ✅; cargo nextest run -p vanta-memory ✅ (361 tests, 10 nuevos D19); cargo fmt --check ✅; cargo clippy -p vanta-memory --all-targets --no-deps -- -D warnings ✅
- **invariantes:** soft-delete respetado en las 3 tools (read→NotFound, list/query→excluidos); validación boundary (session/scene_name/keyword); LLM-free (Principio 4); sin deps nuevas; sin unwrap/expect en producción; NO se tocó el core vantadb
- **deuda:** query overlap naive (techo documentado, upgrade vector index)
- **queda_pendiente:** commit por el lead; F4 completa (MEM-08a..21)

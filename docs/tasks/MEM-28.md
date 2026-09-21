# MEM-28 — Wiki store + state machine pending→ready (core, LLM-free)

**Plan:** `docs/plans/2026-08-21-vanta-proxy-knowledge.md` · **Task 2** · **Ruta:** vanta-worker

## Contrato
`cargo check -p vantadb` pasa; tests D19:
- (a) create → pending
- (b) ingest en pending/processing → 409-equivalente (`VantaError::ExecutionConflict`)
- (c) transición completa pending→processing→ready con run_id
- (d) fallo → failed con sync_error truncado 500
- (e) dedup path canónico type+title (`wiki/{dir}/{slug}.md`)
- (f) locked:true en páginas gestionadas + cascade delete

## Impacto mapeado (Regla 0)
**Archivos leídos completos:** `src/entity/mod.rs` (EntityStore completo), `src/entity/scene.rs` (SceneNodeStore completo — patrón InternalMetadata MEM-12), `src/entity/tests.rs` + `src/entity/scene_tests.rs` (fixture in_memory_engine), `src/error.rs` VantaError enum (variantes ExecutionConflict/InvalidInput/ValidationError), `.opencode/rules/core-engine.md`, `.opencode/rules/api-contract.md`, `docs/plans/2026-08-21-vanta-proxy-knowledge.md` Task 2.

**Referencias hacia dentro (nuevo módulo):**
- `crate::backend::{BackendPartition, BackendWriteOp}` — put_to_partition/get_from_partition/scan_partition_prefix/write_backend_batch
- `crate::storage::StorageEngine`
- `crate::error::{ChainedError, Result, VentaError→VantaError}`
- `crate::entity::generate_id` — run_id generation (sin deps nuevas; TDAM randomUUID paridad aproximada documentada)

**Referencias entrantes:** NINGUNA hoy (`src/wiki/` NO existe; consumidores futuros: MEM-30 ingest en vanta-memory vía SDK, MEM-33 wiki_* MCP tools). Wiring único: `src/lib.rs` agrega `pub mod wiki;`.

**Veredicto de impacto:** BAJO — módulo 100% aditivo (nuevos archivos + 1 línea en lib.rs). No toca wal/vector/storage/entity existentes. API pública solo crece (semver-safe). Riesgo principal: clippy -D warnings sobre código nuevo.

## Steps
- ✅ Step 1: `src/wiki/state.rs` (enum WikiState #[non_exhaustive]) + `src/wiki/store.rs` parte 1 (Wiki record, create/get, keys, validación sanitización ≤512/NUL) + wiring lib.rs + cargo check
- ✅ Step 2: transiciones de estado (request_ingest/begin_processing/complete/fail) con CAS por estado + truncado sync_error 500
- ✅ Step 3: páginas gestionadas (canonical_path dedup, put_page locked:true, list/delete_page, cascade delete_wiki)
- ✅ Step 4: tests D19 (a)-(f) + verify mecánico completo

## Context Save Point (cierre)
- Verify mecánico ✅: `cargo check -p vantadb` exit 0 · `cargo nextest run -p vantadb wiki::` 11/11 PASS · `cargo fmt --check` exit 0 · `cargo clippy -p vantadb --all-targets --no-deps -- -D warnings` exit 0 · `cargo check -p vanta-memory` exit 0 · `cargo check -p vantadb-mcp` exit 0
- Commit: NO hecho (instrucción explícita del orquestador — Regla 2 de la invocación). Archivos listos para commit del lead.
- API expuesta: `vantadb::wiki::{WikiState, Wiki, WikiPage, WikiStore, canonical_path}` — WikiStore::{create,get,delete,request_ingest,begin_processing,complete,fail,put_page,get_page,list_pages,delete_page}. MEM-30 (vanta-memory) consume vía SDK; MEM-33 tools wiki_* sobre esto.
- Nota clippy: los "7 warnings unsafe" de src/storage/* aparecen solo en builds con features no-default (vanta-memory); pre-existentes, intocados según instrucción.

## Decisiones de diseño
- Persistencia: partición `InternalMetadata` — patrón exacto SceneNodeStore (D4). Keys: `wiki:{ns}::{slug}` (record) / `wiki:{ns}::{slug}:page:{path}` (páginas).
- 409-equivalente = `VantaError::ExecutionConflict` (no-retriable, semántica conflicto de estado).
- run_id = `crate::entity::generate_id("wikirun")` — sin dep uuid nueva; unicidad ts4+rand6 base36 suficiente para runs single-process (paridad randomUUID aproximada, documentada).
- CAS de estado: cada transición read-validate-write contra estado fuente esperado + `version` bump. Techo conocido (ponytail): no hay txn cross-key en el engine; callers single-process (MEM-30 worker único). Upgrade path: txn a nivel engine si aparece multi-writer.
- STRUCTURAL_FILES protegidos (:69-75 TDAM) → fuera de scope acá, es del ingest MEM-30.

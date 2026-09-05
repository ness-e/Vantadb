# MEM-06: F3 Esquema skills multi-versión en core

## Metadata
- **Plan file:** `docs/plans/2026-08-18-vanta-memory.md`
- **Creado:** 2026-08-20T16:30
- **last-synced:** 2026-08-20T16:30
- **Estado:** ✅ COMPLETED (commit 92cf709f, verify 14/14 skills tests ✅ 2026-08-20)

## Blast Radius

**Callers (aguas arriba — dependen de esto):**
- MEM-07 (F3, ⬜ PENDING): MCP tools `skill_*` sobre este store — usa `SkillRecord`, `SkillCreateInput/UpdateInput/PatchInput`, optimistic lock `expected_version`, owner check → 404.
- MEM-35 (F3, ⬜ PENDING): `GET /skill/listing` — usa `SkillStore::list` / `SkillListPage` para listar heads.
- `src/lib.rs` re-export público → semver surface nueva.

**Callees (aguas abajo — de lo que depende):**
- `src/entity/mod.rs` (MEM-03, commit `23719e23`): `EntityStore` — `entity_set/get/delete/list`, `generate_id("skl")`, `Entity { namespace, collection, entity_id, fields: HashMap<String, FieldValue> }`, keys `entity:{ns}:{col}::{id}` en partición `InternalMetadata`. **Reuso como base de persistencia.**
- `src/storage/engine/partition.rs`: `write_backend_batch(Vec<BackendWriteOp>)` (pub(crate), atómico) + `scan_partition_prefix` + `get_from_partition` + `put_to_partition` — para atomicidad multi-key (versión + índice).
- `src/backend.rs`: `BackendWriteOp::Put/Delete`, `BackendPartition::InternalMetadata`.
- `src/error.rs`: `VantaError` — usar `ExecutionConflict { resource, detail }` (optimistic lock), `NotFound { kind, id }`, `ValidationError { field, reason }`, `SerializationError`.
- `src/node.rs`: `FieldValue` (String/Int/Bool) para fields de versión.
- `src/sdk/types.rs`: tipos públicos SDK (patrón `SearchProfileConfig`, `VantaMemoryFilter`).
- `web_time` (ya dep del core, usado en entity): `SystemTime` para timestamps.

**Implicaciones:**
- NO tocar `src/wal.rs`, `src/vector/`, `src/storage/` (dominio Arch/Engine).
- NO storage nuevo: skills = Entities en `InternalMetadata` (Regla 6, D4).
- NO deps nuevas: content_hash = FNV-1a 64 hex (estable, no-criptográfico, idempotencia-only) — MD5 de TDAM requiere dep, FNV-1a es 8 líneas.
- NO vec0 / HNSW en este task: el contrato es CRUD multi-versión + optimistic lock + TTL + idempotencia; búsqueda semántica NO está en el contrato (viene con MEM-07 listing por nombre/owner, no por embedding).
- WASM-compatible: sin std::time (usar web_time), sin deps de red.
- `#[non_exhaustive]` no aplica a structs de datos (api-contract R-6 aplica a enums públicos en crecimiento; `SkillStatus` no se crea — sin status en v1).

## Contrato
`cargo check -p vantadb` pasa; tests dedicados de skills multi-versión (D19):
- `cargo nextest run -p vantadb -- skills` ✅ (CRUD, versionado, optimistic lock, TTL keep-recent=3, idempotencia, índice único is_head)

## Herramientas
- bash: `cargo check -p vantadb`, `cargo nextest run -p vantadb -- skills`, `cargo fmt --check`, `cargo clippy -p vantadb -- -D warnings`
- codegraph (intel), skill TDD para steps de lógica

## Impacto mapeado (Regla 0) — GATE ANTES DE EDITAR

**Archivos leídos completos (2026-08-20):**
- `src/entity/mod.rs` — leído completo vía codegraph (250 líneas): EntityStore CRUD, validate_key (rechaza `{`,`}`,`:` en namespace/collection/entity_id), `generate_id`, entity_key/collection_prefix. **El key es `entity:{ns}:{col}::{id}`; entity_id NO puede contener `:` → usar `~v{N}` como separador de versión.**
- `src/entity/tests.rs` — leído completo (258 líneas): patrón de tests in-memory (`StorageEngine::open_with_config(":memory:", Some(config))`).
- `src/lib.rs` — leído completo (197 líneas): `pub mod entity;` existe; re-export `pub use sdk::{...}` — ahí se agregan tipos de skill; falta `pub mod skills;`.
- `src/sdk/types.rs` — leído parcial (150/1723 líneas): estructura de tipos serde, `VantaMemoryFilter`, etc. Los tipos de skill van al final del archivo (patrón: `#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]`).
- `src/error.rs` — leído parcial (40-259): `VantaError::ExecutionConflict { resource, detail }` (línea 212), `NotFound { kind, id }` (176), `ValidationError { field, reason }` (185), `SerializationError` (126), `InvalidInput` (253). Usar estos, NO crear enum nuevo.
- `src/storage/engine/partition.rs` — leído completo vía codegraph (63 líneas): `write_backend_batch` pub(crate) atómico, `scan_partition_prefix` pub(crate), `get_from_partition` pub(crate), `put_to_partition` pub.
- `src/backend.rs` — leído parcial (27-216): `BackendWriteOp::Put/Delete`, `BackendPartition::InternalMetadata`, `StorageBackend` trait.
- TDAM clon `MemoryCore/src/core/skill/skill-store.ts` (733 líneas, leído completo) + `skill-versioning.ts` (435 líneas, leído completo) + `skill-config.ts` (289, parcial): modelo referencia.

**Referencias hacia dentro (grep):** ningún archivo externo referencia `src/skills.rs` aún (módulo nuevo); MEM-07/MEM-35 lo referenciarán en tasks futuras (no existen aún).

**Referencias salientes del cambio:**
- `src/sdk/types.rs` ← agregar tipos: `SkillRecord`, `SkillCreateInput`, `SkillUpdateInput`, `SkillPatchInput`, `SkillListOptions`, `SkillListPage`, `SkillWriteResult` (todos serde + Clone + PartialEq). Nada existente se modifica (append al final).
- `src/lib.rs` ← agregar `pub mod skills;` (junto a `pub mod entity;`) + extender `pub use sdk::{...}` con tipos de skill. Nada se rompe (append).
- `src/skills.rs` (nuevo) ← usa `EntityStore` (pub), `StorageEngine::write_backend_batch`/`scan_partition_prefix`/`get_from_partition` (pub(crate) — accesible dentro del crate), `VantaError`, `FieldValue`, `generate_id`, `web_time`. Sin cambios a esos archivos.

**Veredicto de impacto:** ✅ bajo — 2 archivos modificados por append (types.rs, lib.rs) + 1 nuevo (skills.rs) + 1 nuevo tests (skills/tests.rs). Ningún archivo existente cambia de comportamiento. Sin deps nuevas. WASM-safe.

## Steps

### Step 1: Tipos SDK de skill en `src/sdk/types.rs`
- **Archivos:** `src/sdk/types.rs`, `src/lib.rs`
- **Acción:** Agregar al final de `src/sdk/types.rs` los tipos públicos serde:
  - `SkillRecord { skill_id, version: u64, is_head: bool, owner_agent: String, name, description, content, content_hash: String, metadata: BTreeMap<String,String>, created_at: u64, updated_at: u64, expires_at: Option<u64> }`
  - `SkillCreateInput { name, description, content, owner_agent, metadata (default {}), ttl_secs: Option<u64> }`
  - `SkillUpdateInput { description, content, metadata: Option<BTreeMap<String,String>> }` (None = conservar)
  - `SkillPatchInput { description: Option<String>, content: Option<String>, metadata: Option<BTreeMap<String,String>> }`
  - `SkillListOptions { owner_agent: Option<String>, name_prefix: Option<String>, limit (default 50), offset }`
  - `SkillListPage { items: Vec<SkillRecord>, total: usize }`
  - `SkillWriteResult { record: SkillRecord, idempotent: bool }` (idempotent=true cuando content_hash no cambió → no-op)
  - Extender `pub use sdk::{...}` en `src/lib.rs` con los nuevos tipos.
- **Verify:** `cargo check -p vantadb` — ✅ 2026-08-20
- **Estado:** ✅ COMPLETED (commit 92cf709f, verify 14/14 skills tests ✅ 2026-08-20)

### Step 2: SkillStore base — reads + helpers en `src/skills.rs`
- **Estado:** ✅ COMPLETED (commit 92cf709f, verify 14/14 skills tests ✅ 2026-08-20)
- **Archivos:** `src/skills.rs` (nuevo), `src/lib.rs`
- **Acción:** Crear `src/skills.rs`:
  - `pub struct SkillStore<'a> { engine: &'a StorageEngine }` + `new()`.
  - Constantes: `SKILL_NS = "skills"`, colecciones `skill` (versiones) y `skill_head` (índice único parcial), `KEEP_RECENT = 3`.
  - Keys: versión = `entity:{{skills}}:{{skill}}::{skill_id}~v{N}`; head = `entity:{{skills}}:{{skill_head}}::{owner}#{name}` (validar owner/name sin `#~{}:`).
  - `fnv1a_64(content) -> String` (hex, estable, no-criptográfico).
  - `from_entity(Entity) -> Result<SkillRecord>` y `to_entity(record)` (FieldValue mapping).
  - Reads: `get_version(skill_id, version)`, `get_head(skill_id)` (scan versión prefix, is_head=true), `list_versions(skill_id, limit, offset)` (DESC), `list(opts)` (scan `skill_head`, resolve head por skill_id). Errores con `NotFound`.
  - Registrar `pub mod skills;` en `src/lib.rs`.
- **Verify:** `cargo check -p vantadb` — ✅ 2026-08-20 (src/skills.rs completo)
- **Estado:** ✅ COMPLETED (commit 92cf709f, verify 14/14 skills tests ✅ 2026-08-20)

### Step 3: Writes — create/update/patch/delete con optimistic lock + idempotencia + índice único
- **Archivos:** `src/skills.rs`
- **Acción:** Métodos writes (todos batch atómico vía `write_backend_batch`):
  - `create(input)`: valida name/owner; check índice `skill_head::{owner}#{name}` → si existe y content_hash igual → devolver head existente `idempotent=true`; si existe con hash distinto → `ExecutionConflict` ("name already exists"); si no existe → v1 `is_head=true`, put versión + put índice en batch.
  - `update(skill_id, expected_version, input)`: head = get_head → si no → `NotFound`; si head.version != expected_version → `ExecutionConflict` ("expected X, head is Y"); si content_hash(input.content) == head.content_hash → no-op `idempotent=true`; si no → batch [marcar viejo is_head=false, put vN+1, put índice actualizado].
  - `patch(skill_id, expected_version, input)`: igual que update pero merge parcial sobre head (Option fields).
  - `delete(skill_id, expected_version)`: head check + version check; batch [delete todas las versiones (scan prefix), delete índice]; retorna `bool` existed.
  - Helper `write_head_batch(...)` para construir ops.
  - **Concurrencia (Regla 8):** `write_backend_batch` es atómico por backend → sin lock manual intra-proceso; el optimistic lock `expected_version` es el mecanismo de serialización para writes concurrentes (chequeo + batch atómico). Documentar invariante.
- **Verify:** `cargo check -p vantadb` — ✅ 2026-08-20 (create/update/patch/delete batch atómico)
- **Estado:** ✅ COMPLETED (commit 92cf709f, verify 14/14 skills tests ✅ 2026-08-20)

### Step 4: TTL keep-recent=3
- **Archivos:** `src/skills.rs`
- **Acción:** `cleanup_expired_versions(skill_id, now: u64) -> Result<usize>`: lista versiones del skill; borra versiones no-head con `expires_at < now` EXCEPTO los 3 no-head más recientes (KEEP_RECENT=3, por version DESC) — port fiel de TDAM `SkillVersioning.cleanupExpiredVersionsForSkill` (líneas 388-430). `expires_at` se setea en create/update si `ttl_secs > 0`. Borrado vía batch.
- **Verify:** `cargo check -p vantadb` — ✅ 2026-08-20 (cleanup_expired_versions + KEEP_RECENT)
- **Estado:** ✅ COMPLETED (commit 92cf709f, verify 14/14 skills tests ✅ 2026-08-20)

### Step 5: Tests dedicados D19 — `src/skills/tests.rs`
- **Estado:** ✅ COMPLETED (commit 92cf709f, verify 14/14 skills tests ✅ 2026-08-20)
- **Archivos:** `src/skills/tests.rs` (nuevo), `src/skills.rs` (registrar `#[cfg(test)] mod tests;`)
- **Acción:** Tests con engine in-memory (patrón `src/entity/tests.rs`):
  1. CRUD: create→get_head→update→patch→delete roundtrip.
  2. Versionado: update genera v2 con v1 is_head=false; `list_versions` DESC; name/owner inmutables entre versiones.
  3. Optimistic lock: update con expected_version viejo → `ExecutionConflict`; con correcto → OK.
  4. TTL keep-recent=3: 6 versiones con expires_at expirado → cleanup borra 3 (mantiene head + 3 recientes); versión con expires_at futuro sobrevive.
  5. Idempotencia: create repetido con mismo content_hash → misma skill, `idempotent=true`, sin v2; update con mismo content → no-op.
  6. Índice único: create v1 owner A name X; create otro con owner A name X → `ExecutionConflict`; owner B name X → OK; delete → recreate same name → OK.
  7. Expiración: `expires_at` presente con ttl_secs, ausente sin.
- **Verify:** `cargo nextest run -p vantadb -- skills` — ✅ 14/14 (2026-08-20)
- **Estado:** ✅ COMPLETED (commit 92cf709f, verify 14/14 skills tests ✅ 2026-08-20)

### Step 6: Verify full + docs + commit
- **Acción:** (ejecutado)
  - Verify full: `cargo fmt --check` ✅ + `cargo clippy -p vantadb -- -D warnings` ✅ + `cargo nextest run -p vantadb -- skills` ✅ 14/14
  - Docs: `docs/api/EMBEDDED_SDK.md` — sección Skills API (Regla 3)
  - Commit conventional con task ID: `feat(core): esquema skills multi-versión con optimistic lock (MEM-06)`
- **Verify:** contrato completo ✅ (cargo check + tests skills 14/14 + fmt + clippy)
- **Estado:** ✅ COMPLETED (commit 92cf709f, verify 14/14 skills tests ✅ 2026-08-20)
- **Archivos:** `src/skills.rs`, `src/sdk/types.rs`, `src/lib.rs`, `docs/api/EMBEDDED_SDK.md`
- **Acción:**
  - Verify full: `cargo fmt --check` + `cargo clippy -p vantadb -- -D warnings` + `cargo nextest run --profile audit --workspace --build-jobs 2` + `scripts/validate-docs-coverage.ps1`.
  - Docs: actualizar `docs/api/EMBEDDED_SDK.md` con la API de skills (Regla 3: doc en mismo PR para struct pub nuevo).
  - Commit conventional con task ID: `feat(core): esquema skills multi-versión con optimistic lock (MEM-06)`.
- **Verify:** contrato completo ✅
- **Estado:** ✅ COMPLETED (commit 92cf709f, verify 14/14 skills tests ✅ 2026-08-20)

## Dependencias
- MEM-03 (EntityStore) ✅ commit `23719e23`
- MEM-04 (checker) ✅ — no usado directo en core, MEM-07 hace owner check
- MEM-05 (auth) ✅ — no usado en core
- TDAM fuente: `MemoryCore/src/core/skill/skill-store.ts` @ `97f9465` (referencia algoritmo)

## Notas
- TDAM usa SQLite + FTS5 + vec0 — **NO portar** (SYNTHESIS §2.4). VantaDB: EntityStore + InternalMetadata; sin FTS/vec para skills en v1 (list por nombre/owner, no búsqueda).
- `write_backend_batch` es `pub(crate)` → accesible desde `src/skills.rs` (mismo crate). Es la vía para atomicidad multi-key sin tocar storage/.
- `scan_partition_prefix` es `pub(crate)` — OK dentro del crate.
- Content_hash FNV-1a: decisión explícita (sin dep nueva, estable entre runs — DefaultHasher de std NO es estable entre versiones de Rust, por eso FNV manual).
- MEM-07 consumirá: `SkillRecord` para view, `expected_version` para writes, `owner` para filtros → el core expone todo.
- Invariante de seguridad: `owner#name` y `{skill_id}~v{N}` como entity_id — validar que owner/name/skill_id no contengan `#`,`~`,`{`,`}`,`:` (validate_key del EntityStore ya rechaza `{}:` pero `#~` son nuestros).

## Context Save Point
- **Fecha:** 2026-08-20T16:30
- **Branch:** develop
- **CI pendiente:** no
- **Decisiones:**
  - Content hash FNV-1a 64 hex sobre MD5 de TDAM → sin dep nueva, WASM-safe, estable entre runs (DefaultHasher no es estable).
  - Sin `SkillStatus`/archived en v1 → no está en el contrato; delete es borrado físico (patrón TDAM deleteAllVersions).
  - Sin búsqueda semántica (HNSW/text_index) en este task → no está en el contrato; listing es por (owner, name_prefix).
  - Índice único parcial = colección `skill_head` con key `{owner}#{name}` → unicidad por construcción de clave + batch atómico (check + put).
  - Optimistic lock vía `VantaError::ExecutionConflict` (ya existe, línea 212) — sin enum de error nuevo.
- **Problemas conocidos:** `write_backend_batch` no expone una variante "check-and-put" atómica nativa — el check del índice único y el lock de versión se hacen con read-then-batch; correcto para single-writer embedded (Regla 8: documentar que el lock es el expected_version + batch atómico, sin locks extra).
- **Próxima tarea:** MEM-07
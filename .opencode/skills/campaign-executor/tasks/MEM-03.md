# MEM-03: Entidades entity_* + CRUD en core (F2)

## Metadata
- **Plan file:** `docs/plans/2026-08-18-vanta-memory.md`
- **Creado:** 2026-08-20T14:30
- **last-synced:** 2026-08-20T14:30
- **Estado:** ✅ COMPLETED (commit `23719e23`, verify `cargo check -p vantadb` ✅ + `cargo nextest run -p vantadb -- entity` 14/14 ✅ 2026-08-20)

## Blast Radius
- **Callers (futuros):** MEM-04 (permission-checker lee teams/users/assets), MEM-05 (auth: user-key → userId), MEM-06 (skills), MEM-35 (data plane), Studio (Inspector KV genérico, contrato 2 del plan).
- **Callees:** `StorageEngine::{put_to_partition, get_from_partition, write_backend_batch, scan_partition_prefix}`, `BackendPartition::InternalMetadata`, `BackendWriteOp::Delete`, `UnifiedNode`/`FieldValue` (solo como tipo de campos), `VantaError::serialization/InvalidInput`, serde_json, rand, web_time.
- **Implicaciones:** módulo público nuevo → `cargo check -p vantadb` debe pasar sin features nuevas; `src/lib.rs` exporta `pub mod entity;`. NO toca `src/storage/` (dominio Arch) ni `src/wal.rs`/`src/vector/`. NO rompe WASM (serde_json/rand/web_time ya usados por `src/agentic/thread.rs`). Sin deps nuevas.

## Impacto mapeado (Regla 0)
> **GATE ANTES DE CUALQUIER EDICIÓN (MUST — AGENTS.md Regla 0):** todo archivo que se vaya a modificar/eliminar se lee completo y se mapea su impacto ANTES del primer step de edición.

- **Archivos leídos (completos):** `src/agentic/thread.rs` (patrón D4 completo, 327L), `src/lib.rs` (195L), `src/engine.rs` (160L — InMemoryEngine NO envuelve StorageEngine), `src/storage/engine/partition.rs` (API put/get/scan), `src/storage/engine/get.rs` + `insert.rs` (firmas), `src/node/unified.rs` (UnifiedNode id u128, set/get_field), `src/node/field.rs` (FieldValue, derive Serialize/Deserialize), `src/error.rs` (variantes + constructores), `src/metadata.rs` (es versioning — NO choca), `src/backend.rs` (BackendWriteOp::Delete, BackendPartition), `.opencode/rules/api-contract.md` (R-1..R-8), fuentes TDAM `metadata/types.ts` + `store/interface.ts` + `constants.ts` + `utils/id-generator.ts` (referencia de modelo, NO copiar).
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** `src/entity/mod.rs` (nuevo) importa `crate::backend::{BackendPartition, BackendWriteOp}`, `crate::error::{ChainedError, Result, VantaError}`, `crate::node::FieldValue`, `crate::storage::StorageEngine`, serde, serde_json, rand, web_time, std::collections::HashMap — todas ya deps del crate (`src/agentic/thread.rs` usa el mismo set).
- **Archivos que referencian a los editados (referencias entrantes):** `src/lib.rs` gana `pub mod entity;` (única referencia entrante hoy). Grep previo: NO existe `entity_*` en el repo (confirmado en SYNTHESIS §2: "hoy el repo no tiene tablas entity_*").
- **Veredicto impacto:** bajo — módulo nuevo aditivo; `lib.rs` solo agrega una línea `pub mod entity;`; nada existente se modifica ni elimina. Sin cambio en API existente → sin bump semver ni sync de bindings (bindings se tocan en MEM-36).

## Contrato
`cargo check -p vantadb` pasa y `cargo nextest run -p vantadb -- entity` pasa (tests dedicados CRUD entity, D19)

## Herramientas
- Terminal (cargo check/nextest/fmt/clippy), codegraph, campaign_verify_cmd

## Steps
### Step 1: Implementar `src/entity/mod.rs` — modelo Entity + EntityStore CRUD (D4)
- **Archivos:** `src/entity/mod.rs` (nuevo), `src/entity/` (dir)
- **Acción:**
  - Doc comment de módulo (estilo repo: thread.rs).
  - `pub struct Entity { namespace, collection, entity_id, fields: HashMap<String, FieldValue>, created_at: u64, updated_at: u64 }` derive `Debug, Clone, Serialize, Deserialize, PartialEq`.
  - `pub struct EntityPage { items: Vec<Entity>, total: usize }` derive `Debug, Clone, PartialEq`.
  - `pub struct EntityStore<'a> { engine: &'a StorageEngine }` + `pub fn new(engine: &'a StorageEngine) -> Self`.
  - Key de partición: `entity:{ns}:{col}::{id}` via `format!("entity:{{{}}}:{{{}}}::{{{}}}", ...)`; list prefix `format!("entity:{{{}}}:{{{}}}::", ...)`.
  - Métodos públicos (nombres del contrato): `entity_set` (upsert: preserva created_at, reemplaza fields, setea updated_at=now), `entity_get` → `Result<Option<Entity>>`, `entity_delete` → `Result<bool>` (usar `write_backend_batch(vec![BackendWriteOp::Delete { partition: InternalMetadata, key }])` — NO tocar src/storage/), `entity_list(ns, col, limit, offset)` → `EntityPage` vía `scan_partition_prefix(InternalMetadata, prefix)` + sort por key + skip/take + total.
  - `pub fn generate_id(prefix: &str) -> String` — port de `id-generator.ts` (4 chars Base36 timestamp + 6 chars Base36 random, formato `{prefix}-{ts}{rand}`); lo necesita MEM-05 (user ids). rand + web_time ya deps.
  - Validación input: namespace/collection/entity_id vacíos → `VantaError::InvalidInput` (variante existente). Errores serde → `VantaError::serialization(ChainedError::with_source(...))` (patrón thread.rs).
  - **NO** usar `unwrap()`/`expect()` (Regla 1); `now_secs()` helper con `web_time` (patrón thread.rs).
- **Verify:** `cargo check -p vantadb`

### Step 2: Tests dedicados CRUD entity en `src/entity/tests.rs` (D19)
- **Archivos:** `src/entity/tests.rs` (nuevo)
- **Acción:**
  - Helper local `in_memory_engine()` → `StorageEngine::open_with_config(":memory:", config)` con `VantaConfig { backend_kind: BackendKind::InMemory, read_only: false, ..default() }` (patrón `src/storage/engine/tests/mod.rs:22-30`).
  - Tests AAA (patrón `references/testing-patterns.md`): set/get roundtrip (fields String/Int/Bool/Float), get missing → None, set upsert preserva created_at y actualiza updated_at, delete → true + get → None, delete missing → false, list paginación (limit/offset/total), aislamiento por namespace (mismo col+id en ns distintos no colisionan), aislamiento por collection, id vacío → InvalidInput, generate_id formato/unicidad básica.
- **Verify:** `cargo nextest run -p vantadb -- entity`

### Step 3: Exportar módulo en `src/lib.rs`
- **Archivos:** `src/lib.rs`
- **Acción:** agregar `pub mod entity;` con doc comment de una línea en la zona alfabética de módulos (tras `engine`, antes de `error`). Re-exportar `Entity, EntityStore, EntityPage` en el bloque `pub use` si aporta ergonomía (estilo: `pub use node::{...}`) — decidir por simplicidad: solo `pub mod entity;` (consumidores usan `vantadb::entity::EntityStore`).
- **Verify:** `cargo check -p vantadb` && `cargo clippy -p vantadb -- -D warnings`

### Step 4: Cierre — verify full + commit
- **Archivos:** (ninguno nuevo; verify)
- **Acción:** `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo nextest run --profile audit --workspace --build-jobs 2`, `scripts/validate-docs-coverage.ps1`. Actualizar plan file (MEM-03 ✅ + recitation) y task file (Estado ✅ COMPLETED + commit hash). Commit conventional: `feat(core): entidades entity_* + CRUD con partición InternalMetadata (MEM-03)` — preparado para que vanta-lead lo ejecute (worker no commitea).
- **Verify:** `campaign_verify_cmd command="cargo nextest run --profile audit --workspace --build-jobs 2"`

## Dependencias
- Task 1-3 (MEM-01/02/34): ✅ COMPLETED — no tocar paths F1 (`SearchProfileConfig`, IQL PROFILE, MCP passthrough, telemetría).
- Task 5 (MEM-04): consume este modelo (checker).
- Task 6 (MEM-05): consume `entity_get` user + `generate_id`.

## Notas
- **Interpretación D4:** "nodos en partición InternalMetadata (patrón thread.rs)" = registros JSON de entidad en `BackendPartition::InternalMetadata` con key prefijada y listado por prefijo — el patrón de thread.rs de datos JSON en partición + índice, sin inventar storage. NO se usan UnifiedNode en Default partition (id u128 no puede representar ids string `usr-xxx`; get/list requerirían hash + hop extra). Decisión documentada para MEM-04/05/review.
- `scan_partition_prefix` es `pub(crate)` y `#[allow(dead_code)]` — al usarlo desde entity.rs el allow queda sin efecto (inofensivo, no lo quito: storage/ es de Arch).
- TDAM es referencia de modelo de datos (types.ts: User/Team/Agent/Task/Asset + UserKey + ACL); MEM-03 implementa el CRUD genérico por collection (user/team/agent/task/asset). El modelado tipado (campos específicos) lo consumen MEM-04/05 leyendo `fields` del Entity genérico.
- `FieldValue::Float(NaN)` falla serialización serde_json → error `serialization` propagado (aceptable; mismo límite que JSON estándar).
- Archivos clave del plan dicen `src/entity.rs` + `src/entity/tests.rs` → layout directorio `src/entity/mod.rs` + `src/entity/tests.rs` (idiomático, satisface ambos literalmente).
- Budget: 4 steps, verify mecánico por step.

## Context Save Point
- **Fecha:** 2026-08-20T14:30
- **Branch:** develop (worktree limpio salvo `docs/plans/2026-08-18-vanta-memory.md` + `tasks/MEM-34.md` modificados por cierre F1 — NO incluir en commit MEM-03 salvo el plan file que se actualiza en cierre)
- **CI pendiente:** no
- **Decisiones:** D4 → registros JSON en partición InternalMetadata (key prefijada + scan por prefijo); layout directorio `src/entity/`; métodos nombrados `entity_set/get/delete/list` (contrato); `generate_id` portado (necesario en MEM-05); sin deps nuevas.
- **Problemas conocidos:** `InMemoryEngine` (engine.rs) NO envuelve StorageEngine — tests usan `StorageEngine::open_with_config(":memory:", BackendKind::InMemory)`.
- **Próxima tarea:** MEM-04 (permission-checker allow-only).
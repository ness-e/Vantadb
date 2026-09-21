# MEM-04: Permission-checker allow-only (F2, eslabón permission de la cadena D7)

## Metadata
- **Plan file:** `docs/plans/2026-08-18-vanta-memory.md`
- **Creado:** 2026-08-20T13:30
- **last-synced:** 2026-08-20T14:10
- **Estado:** ✅ COMPLETED

## Decisión MEM-04 (coexistencia con src/rbac.rs)
- **`src/rbac.rs` NO es dead code** (el SYNTHESIS decía "confirmado", pero el grep real lo desmiente):
  - `src/cli_server.rs:527` → `auth.rbac.has_permission(role, &permission)` (middleware auth HTTP, feature `server`)
  - `src/cli_server.rs:198-205` → `Rbac::new()` + `add_role("admin"/"reader"/"writer")`
  - `src/config.rs:114` → `RbacConfig` (token_role_map) — config pública existente
- **Por qué coexisten:** `rbac.rs` es auth de capa transporte (token → role → read/write global) del servidor HTTP embebido; el checker nuevo es autorización de recursos (assets) por entidades (`entity_*`) con ACL allow-only — eslabón permission de la cadena D7. Capas distintas, ambos necesarios (MEM-05 usa el checker entity-ACL para auth 3 capas L3 user-key; cli_server mantiene su RBAC token→role).
- **Acción:** NO borrar `src/rbac.rs` ni tocar `cli_server.rs`/`config.rs`. El checker vive en `src/entity/checker.rs` (co-locado con EntityStore que lee).
- **Resolución contradicción SYNTHESIS (96 vs ~40 líneas):** la fuente de verdad es el clon TDAM @ `97f9465` → `MemoryCore/src/metadata/service/permission-checker.ts` = **172 líneas reales** (ni 96 ni ~40 — ambos números del reporte eran incorrectos; el plan file línea 71 ya lo corrige a "172 líneas reales"). Port del algoritmo, no copia literal (principios de adaptación plan).

## Blast Radius
- **Callers (futuros):** MEM-05 (auth 3 capas: L3 user-key → userId/isSystemAdmin, y `can_access_asset` para endpoints), MEM-07 (skill-permission owner check), MEM-35 (data plane), Studio (contrato 2, si algún día renderiza assets).
- **Callees:** `EntityStore::{entity_get}` (read de collections user/team/team_member/asset/acl), `FieldValue::as_str`, `VantaError::InvalidInput`, std collections. Sin storage/ directo (va vía EntityStore, dominio MEM-03).
- **Implicaciones:** módulo nuevo aditivo → `cargo check -p vantadb` debe pasar sin features nuevas; WASM-compatible (mismo set de deps que entity: serde/serde_json); **sin deps nuevas** (restricción plan). NO toca `src/rbac.rs` (coexistencia) ni `src/storage/`/`src/wal.rs`/`src/vector/` (dominios Arch/Engine).

## Impacto mapeado (Regla 0)
> **GATE ANTES DE CUALQUIER EDICIÓN (MUST — AGENTS.md Regla 0):** todo archivo que se vaya a modificar/eliminar se lee completo y se mapea su impacto ANTES del primer step de edición.

- **Archivos leídos (completos):** `src/entity/mod.rs` (249L — Entity/EntityStore/validate_key/entity_key), `src/entity/tests.rs` (258L — patrón AAA, in_memory_engine), `src/rbac.rs` (164L — dead-code claim refutado, ver Decisión MEM-04), `src/lib.rs` (197L — módulos + re-exports), `src/node/field.rs` (FieldValue as_str/as_bool/as_int, derive Serialize/Deserialize), `src/cli_server.rs` (parcial 500-559 + grep — usa rbac, NO se toca), `src/config.rs` (parcial grep RbacConfig — NO se toca), fuentes TDAM `metadata/service/permission-checker.ts` (172L — algoritmo de referencia), `metadata/types.ts` (modelo Asset/TeamMember/AclEntity), `.opencode/rules/api-contract.md` (R-1..R-8).
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** `src/entity/checker.rs` (nuevo) importa `crate::entity::{Entity, EntityStore}`, `crate::node::FieldValue`, `crate::error::{Result, VantaError}`, `std::collections::HashMap` — todas ya deps del crate; `src/entity/mod.rs` gana `pub mod checker;`.
- **Archivos que referencian a los editados (referencias entrantes):** `src/entity/mod.rs` (submódulo nuevo `pub mod checker;`) — única edición a archivo existente; `src/entity/checker.rs` nuevo no tiene referencias entrantes hoy.
- **Veredicto impacto:** bajo — módulo nuevo aditivo en `src/entity/`; `entity/mod.rs` solo agrega `pub mod checker;`; nada existente se modifica ni elimina; `rbac.rs` intocado (coexistencia). Sin cambio en API existente → sin bump semver ni sync de bindings (bindings se tocan en MEM-36).

## Contrato
`cargo check -p vantadb` pasa y `cargo nextest run -p vantadb -- entity` pasa (tests dedicados del checker, D19)

## Herramientas
- Terminal (cargo check/nextest/fmt/clippy), codegraph, campaign_verify_cmd

## Steps
### Step 1: Implementar `src/entity/checker.rs` — PermissionChecker allow-only (port cadena TDAM)
- **Archivos:** `src/entity/checker.rs` (nuevo)
- **Acción:**
  - Doc comment de módulo: eslabón permission de la cadena D7; allow-only (si no hay regla que permita explícitamente → denegar); lee teams/users/assets vía `EntityStore` (MEM-03).
  - Tipos públicos `#[non_exhaustive]` (R-6): `Action { Read, Write, Assign, Share, Use }`, `Visibility { Private, Team, Restricted, Agent, Task }`, `TeamRole { Admin, Member, Reviewer }`.
  - `pub struct PermDecision { pub allowed: bool, pub reason: String }` (port de `PermCheckResult`).
  - `pub struct PermissionChecker<'a> { store: &'a EntityStore<'a> }` + `pub fn new(store: &'a EntityStore<'a>) -> Self`.
  - Métodos públicos (nombres del contrato): `is_admin(ns, user_id, team_id) -> Result<bool>`, `is_member(ns, user_id, team_id) -> Result<bool>`, `can_read(ns, user_id, asset_id) -> Result<PermDecision>`, `can_write(ns, user_id, asset_id) -> Result<PermDecision>`, `can_access_asset(ns, user_id, asset_id, action, agent_id: Option<&str>) -> Result<PermDecision>`.
  - Convención de collections/keys (documentar en doc comment): user=`user`/user_id; team=`team`/team_id (fields: owner_user_id, status); membership=`team_member`/`{team_id}.{user_id}` (fields: role, status); asset=`asset`/asset_id (fields: team_id, owner_user_id, visibility, status); acl=`acl`/`{asset_id}.{subject_type}.{subject_id}` (fields: permission, effect). Separador `.` seguro porque `generate_id` produce [a-z0-9-].
  - Cadena `can_access_asset` (port fiel del orden TDAM, allow-only):
    1. asset ausente o `status == "archived"` → DENY `asset_not_available`
    2. `owner_user_id == user_id` → ALLOW `owner`
    3. membership ausente o `status != "active"` → DENY `not_team_member`
    4. visibility: `private` → DENY `visibility_restricted` (solo owner); `restricted` + role != admin → solo ACL explícita (si no → DENY), admin cae a defaults; `task` + action != read + role != admin → DENY; `team`/`agent` → sigue; default/desconocido → DENY `visibility_restricted`
    5. role defaults (código, sin tabla): admin → [Read, Write, Assign, Share]; member → [Read]. Si action en defaults → ALLOW `role_default:<role>`
    6. ACL explícita (subjects user / team_role / agent): entity_get directo por key determinista, `permission == action && effect == "allow"` → ALLOW `acl`
    7. DENY `no_permission`
  - Helpers privados: `get_membership`, `get_asset`, `acl_match` (3 entity_get por subject candidate), `role_default_covers`.
  - Sin `unwrap()`/`expect()` (Regla 1); errores de store se propagan (`?`); strings de reason en minúscula (paridad TDAM).
- **Verify:** `cargo check -p vantadb`

### Step 2: Tests dedicados del checker en `src/entity/checker_tests.rs` (D19)
- **Archivos:** `src/entity/checker_tests.rs` (nuevo)
- **Acción:**
  - Helper local `in_memory_engine()` + helpers de setup: `seed_user`, `seed_team`, `seed_member`, `seed_asset`, `seed_acl` (patrón AAA de `entity/tests.rs`).
  - Tests: owner_allowed_read_write, non_member_denied, member_read_only (read OK, write DENY), admin_all_defaults (read/write/assign/share), private_denies_non_owner_even_admin, restricted_acl_allows_user, restricted_no_acl_denies, restricted_admin_falls_to_defaults, task_visibility_read_only_for_member, task_admin_write_ok, archived_asset_denied, acl_team_role_subject, acl_agent_subject (con agent_id), is_admin_true_false, is_member_true_false, unknown_visibility_denies, missing_asset_denies.
  - Verificar allow-only: caso "no hay regla" → DENY con reason.
- **Verify:** `cargo nextest run -p vantadb -- entity`

### Step 3: Registrar submódulo en `src/entity/mod.rs` + docs
- **Archivos:** `src/entity/mod.rs`
- **Acción:** agregar `pub mod checker;` con doc comment de una línea (tras los types, antes de `mod tests`). NO re-exportar en `lib.rs` (consumidores usan `vantadb::entity::checker::PermissionChecker` — patrón MEM-03: solo `pub mod entity;`).
- **Verify:** `cargo check -p vantadb` && `cargo clippy -p vantadb -- -D warnings`

### Step 4: Cierre — verify full + commit
- **Archivos:** (ninguno nuevo; verify)
- **Acción:** `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo nextest run --profile audit --workspace --build-jobs 2`, `scripts/validate-docs-coverage.ps1`. Actualizar plan file (MEM-04 ✅ + recitation) y task file (Estado ✅ COMPLETED + commit hash). Commit conventional: `feat(core): permission-checker allow-only sobre entity_* (MEM-04)` — preparado para que vanta-lead lo ejecute (worker no commitea).
- **Verify:** `campaign_verify_cmd command="cargo nextest run --profile audit --workspace --build-jobs 2"`
- **Resultado 2026-08-20T14:10:**
  - `cargo nextest run -p vantadb -- entity` → ✅ 37/37 passed (24 checker + 13 entity) — 1 fix de clippy aplicado (`needless_borrow` checker.rs:153: `&team_id` → `team_id`, `team_id` ya es `&str` de `Option<&str>::unwrap_or_default`; semánticamente idéntico, re-verificado 37/37 ✅)
  - `cargo fmt --check` → ✅ (1 archivo formateado: `checker_tests.rs` — 2 diffs de rustfmt, aplicado con `cargo fmt -- src/entity/checker_tests.rs`)
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings` → ✅ (vantadb + vantadb-mcp + vantadb-wasm + vantadb-server + vantadb_py, 0 warnings)
  - Regla 1: 0 `unwrap()`/`expect()` en `checker.rs` (core) — los 35 matches del grep están solo en `checker_tests.rs` (tests, patrón AAA de MEM-03) ✅
  - Commit: 9717bf03

## Dependencias
- Task 4 (MEM-03): ✅ COMPLETED — EntityStore/Entity consumidos (no tocar `src/entity/mod.rs` CRUD, solo agregar submódulo).
- Task 6 (MEM-05): consume `PermissionChecker` para auth 3 capas.
- F1 (MEM-01/02/34): ✅ COMPLETED — no tocar paths F1.

## Notas
- **FASE SECURITY (checker = trust boundary, autorización):** checklist security-and-hardening — allow-only deny-first ✅, input validado en entity_get (validate_key ya valida) ✅, sin logs de datos sensibles ✅, sin secretos ✅, cargo audit NO necesario (sin deps nuevas) ✅. Threat model: Elevation of privilege es el riesgo principal → la cadena siempre termina en DENY cuando no hay regla explícita; private es estricto (ni admin de team accede a asset privado ajeno).
- **FASE PERFORMANCE:** NO aplica (checker no es hot path; entity_get es 1 lookup por partición).
- **Regla 8 (concurrencia):** NO aplica — checker es read-only sobre EntityStore, sin locks propios ni dashmap/parking_lot/Tokio.
- **Decisión de diseño:** ACL por key determinista `{asset_id}.{subject_type}.{subject_id}` + 3 entity_get (user/team_role/agent) evita scan O(n) de la collection acl — mismo patrón D4 de keys prefijadas; sin deps nuevas; sin lógica en bindings (R-8: lógica vive en core).
- Budget: 4 steps, verify mecánico por step.

## Context Save Point
- **Fecha:** 2026-08-20T14:10
- **Branch:** develop (worktree: 2 archivos modificados pre-existentes — `tasks/MEM-34.md`, `completions/*` — NO tocar; staging de MEM-04 incluye SOLO: `src/entity/checker.rs`, `src/entity/checker_tests.rs`, `src/entity/mod.rs`, task file, plan file)
- **CI pendiente:** no — verify local completo (fmt/clippy/tests 37/37) antes del commit
- **Decisiones:** coexistencia con `rbac.rs` (NO dead code — cli_server.rs lo usa, ver Decisión MEM-04); checker en `src/entity/checker.rs` (co-locado con EntityStore); ACL determinista por key; allow-only deny-first; sin deps nuevas; `#[non_exhaustive]` en enums públicos (R-6)
- **Problemas conocidos:** ninguno — 1 fix de clippy + 1 fmt aplicados en cierre (ver Step 4 Resultado)
- **Próxima tarea:** MEM-05 (auth 3 capas en server + audit log).
# MEM-05: F2 Auth 3 capas en server (L1/L2/L3) + audit auth events

## Metadata
- **Plan file:** `docs/plans/2026-08-18-vanta-memory.md`
- **Creado:** 2026-08-20T14:30
- **last-synced:** 2026-08-20T15:15
- **Estado:** ✅ COMPLETED (commit `CAMBIO_POR_VANTA_LEAD`, verify `cargo check -p vantadb-server` ✅ + tests auth 16/16 ✅)

## Blast Radius

**Arquitectura (verificada con codegraph + Read):**
- `vantadb-server/` es un **wrapper thin**: `src/middleware.rs` (1 línea) y `src/server.rs` (4 líneas) solo re-exportan `vantadb::cli_server::{auth_middleware, AuthState, app, run, ...}`. El server HTTP real vive en `src/cli_server.rs` (core, feature `server`, axum). Contrato `cargo check -p vantadb-server` compila el core con `features=["cli","server"]`.
- Auth actual (L1) en `auth_middleware` (`src/cli_server.rs:455`): `/health` público; sin `api_key` → dev mode allow-all; Bearer vs `api_key` con `ct_eq` (subtle::ConstantTimeEq, **ya timing-safe**); RBAC coarse via `token_role_map` + `auth.rbac.has_permission`; `AuthRateLimiter` 5 fails/60s por IP.
- `AuthState` (`cli_server.rs:384`) NO tiene acceso a storage ni al audit logger — se crea en `app_with_cors` desde `ServerState` (que SÍ tiene `storage: Arc<StorageEngine>` y `db: VantaEmbedded`).
- `VantaEmbedded.audit: Option<Arc<AuditLogger>>` es **campo privado** en `src/sdk/builder.rs`; hay `pub(crate) fn audit()` pero no accessor para obtener el Arc.
- `AuditEvent` (`src/audit.rs`): struct `{timestamp, op, namespace, key, outcome, reason}` + helper `memory()` (MEM-34). Endpoint `GET /api/v2/audit` ya existe (`cli_server.rs:227`) y lee el JSONL. **NO crear `vantadb-server/src/audit.rs`** — extender `AuditEvent::auth()`.
- MEM-03 (`EntityStore`, `src/entity/mod.rs`): `entity_get/set/delete/list` en partición InternalMetadata; collections user/team/team_member/asset/acl; user = `id: user_id`, fields `status`/`user_type` (checker.rs:15), `user_key` (TDAM getUserByKey).
- MEM-04 (`PermissionChecker`, `src/entity/checker.rs`): `is_admin(ns,user_id,team_id)`, `can_access_asset(ns,user_id,asset_id,action,agent_id)`. Disponible para downstream de L3 vía identidad resuelta.
- TDAM ref (`clon @97f9465`): `router/auth.ts` — L1 `Authorization: Bearer <KERNEL_AUTH_TOKEN>` (gateway); L2 `x-tdai-service-id` = instancia, **Bearer + service-id = credencial admin** (`metadata-service.ts:1909`); L3 `x-tdai-user-key` → `verifyAuth(userKey)` → `{userId, isSystemAdmin}` (`user_type === "system_admin"`).

**Callers:** `auth_middleware` — 1 caller (`app_with_cors`); `AuthState` — 1 caller (`app_with_cors`) + re-export `vantadb-server/src/{middleware,server}.rs`. Tests de `mod tests` en `cli_server.rs` construyen `ServerState` (no `AuthState` directo) → cambiar firma de `AuthState::new` solo afecta `app_with_cors`.

**Implicaciones:** cambios aditivos en core (sin feature-gate WASM: audit.rs y builder.rs compilan en WASM, sin deps nuevas — subtle ya en árbol). `app`/`app_with_cors`/`auth_middleware` mantienen firma → zero break en `vantadb-server`. Coexistencia con `rbac.rs` (NO borrar — transporte existente).

## Impacto mapeado (Regla 0)

| Archivo | Leído completo | Ref hacia dentro | Ref entrantes | Veredicto |
|---|---|---|---|---|
| `src/cli_server.rs` (3808L) | ✅ secciones 1-70, 100-189, 413-742, 1160-1249, 1400-1659, 2400-2529 | `AuthState`, `auth_middleware`, `Rbac`, `Permission`, `AuditEvent`, `ct_eq` | `vantadb-server/src/{middleware,server}.rs` re-exportan; `app_with_cors` usa `AuthState::new` | **EDITAR** (auth 3 capas; aditivo) |
| `src/audit.rs` (162L) | ✅ | `AuditEvent`, `AuditLogger`, `memory()` | `sdk/builder.rs`, `cli_server.rs`, tests | **EDITAR** (helper `auth()`) |
| `src/sdk/builder.rs` (378L, leído 1-140) | ✅ (1-140; resto no relevante) | `VantaEmbedded.audit` privado | `cli_server.rs:115` (`state.db`), bindings | **EDITAR** (accessor `pub(crate) audit_logger()`) |
| `src/entity/mod.rs` / `checker.rs` | ✅ (codegraph verbatim) | `EntityStore`, `PermissionChecker` | MEM-04 tests | **NO TOCAR** (solo consumir) |
| `vantadb-server/src/{middleware,server,lib,main}.rs` | ✅ | re-exports | binario | **EDITAR solo si hace falta** re-exportar `AuthIdentity` |

## Contrato
- `cargo check -p vantadb-server` pasa
- Tests dedicados de auth 3 capas (D19) pasan: `cargo nextest run -p vantadb -- auth`

## Herramientas
- codegraph, cargo/nextest (terminal), campaign_verify_cmd (si el runner falla 0.3s → bash real), skill security-and-hardening (trust boundary), source-driven-development (axum 0.8)

## Steps
### Step 1: `AuditEvent::auth()` helper + tests en `src/audit.rs`
- **Archivos:** `src/audit.rs`
- **Acción:** añadir `pub fn auth(kind: &str, namespace: &str, key: &str, outcome: &str, reason: Option<String>) -> Self` → op `auth_{l1|l2|l3}` (kind desconocido cae a `auth_{raw}`, error-silent como `memory()`). 2 tests: op names + JSONL roundtrip.
- **Verify:** `cargo test -p vantadb audit::` (o nextest `audit`)
- **Estado:** ⬜ PENDING

### Step 2: Accessor `pub(crate) fn audit_logger()` en `VantaEmbedded`
- **Archivos:** `src/sdk/builder.rs`
- **Acción:** `pub(crate) fn audit_logger(&self) -> Option<Arc<crate::audit::AuditLogger>> { self.audit.clone() }` — permite a `app_with_cors` pasar el logger al `AuthState`.
- **Verify:** `cargo check -p vantadb`
- **Estado:** ⬜ PENDING

### Step 3: Auth 3 capas en `src/cli_server.rs` (core)
- **Archivos:** `src/cli_server.rs`
- **Acción:** (a) `AuthState` gana `storage: Option<Arc<StorageEngine>>` + `audit: Option<Arc<AuditLogger>>`; `AuthState::new` extendido; (b) `AuthIdentity` enum (`Transport`/`Service{service_id}`/`User{user_id,is_system_admin}`) insertado en request extensions; (c) `resolve_identity(req, auth)` pub(crate): L3 `x-vanta-user-key` (precedencia) → `resolve_user_key()` scan `entity_list(ns="default", "user")` match `fields.user_key` con `ct_eq` → `(user_id, is_system_admin)`; L2 `x-vanta-service-id` no vacío → Service; else Transport. Resolución falla → 401 fail-closed; (d) `auth_middleware` reescrito: L1 igual (ct_eq), audit `auth_l1` err (invalid/missing/rate_limited) + `auth_l2`/`auth_l3` ok/err, RBAC coarse SOLO para Transport, identity en extensions. Constantes `AUTH_ENTITY_NS="default"`, headers `x-vanta-user-key`/`x-vanta-service-id` documentados.
- **Verify:** `cargo check -p vantadb --features server` + `cargo check -p vantadb-server`
- **Estado:** ⬜ PENDING

### Step 4: Tests dedicados auth 3 capas (D19)
- **Archivos:** `src/cli_server_auth_tests.rs` (nuevo, `#[cfg(test)] mod auth_tests;` en cli_server.rs) + `src/cli_server.rs`
- **Acción:** L1 (missing/wrong/valid → 401/401/200), L2 (service-id con Bearer → 200, identity unit), L3 (user_key resuelve user → 200; invalid → 401; precedencia sobre service-id; system_admin detectado), audit events (auth_l1 err y auth_l3 ok en JSONL con tempfile). Unit tests de `resolve_identity`/`resolve_user_key` + integración HTTP con listener (patrón existente).
- **Verify:** `cargo nextest run -p vantadb -- auth`
- **Estado:** ⬜ PENDING

### Step 5: Re-exports + verify full + commit
- **Archivos:** `vantadb-server/src/middleware.rs`, `vantadb-server/src/server.rs` (si `AuthIdentity` debe re-exportarse)
- **Acción:** re-export `AuthIdentity` si aplica; verify full (fmt/clippy/nextest contrato) y commit `feat(server): auth 3 capas L1/L2/L3 + audit auth events (MEM-05)`.
- **Verify:** fmt --check, clippy -D warnings, nextest contrato, `cargo check -p vantadb-server`
- **Estado:** ⬜ PENDING

## Dependencias
- MEM-03 ✅ (`EntityStore`, commit `23719e23`)
- MEM-04 ✅ (`PermissionChecker`, commit `9717bf03`)
- MEM-34 ✅ (`AuditEvent::memory` + `/api/v2/audit`, commit `84f28a18`)

## Notas
- **Descubrimiento clave:** `auth_middleware` ya usa `ct_eq` (L1 timing-safe existe) — el trabajo real es L2/L3 + audit + identidad.
- L2 en TDAM = `x-tdai-service-id` (instancia `n`); **Bearer + service-id = admin-level** — no es secreto separado, es claim sobre L1 válido. Sin multi-tenancy → validation = presente + no vacío.
- Audit: solo FAILURES (auth_l1 err) + L2/L3 outcomes (identidad establece WHO) — evitar flooding de `auth_l1 ok` por request.
- L3 scan O(users) sobre `entity_list` — `ponytail:` comment con upgrade (índice user_key→user_id cuando crezca).
- FASE SECURITY (obligatoria): timing-safe en L1 y L3, deny-by-default, fail-closed, rate-limit existente, sin secrets en logs/audit, sin deps nuevas (subtle ya en árbol), CORS/body-limit intactos.
- No tocar `src/rbac.rs` ni sus callers (`cli_server.rs:198-205,527`, `config.rs:114`).

## Context Save Point
- **Fecha:** 2026-08-20T14:30
- **Branch:** develop
- **CI pendiente:** sí (verify full antes de commit)
- **Decisiones:** Server objetivo = `src/cli_server.rs` (vantadb-server es wrapper — verificado). L3 gana sobre L2 si ambos headers presentes (determinístico). RBAC coarse solo para identidad Transport (L2/L3 authz downstream vía PermissionChecker). Audit: solo failures L1 + outcomes L2/L3.
- **Problemas conocidos:** ninguno
- **Próxima tarea:** Checkpoint F1+F2 (revisión humana)
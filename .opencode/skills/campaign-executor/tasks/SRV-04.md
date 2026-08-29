# SRV-04: Multi API keys + rotación sin downtime

## Metadata
- **Plan file:** `docs/plans/2026-08-28-backlog-triage.md`
- **Creado:** 2026-08-28T10:00:00
- **last-synced:** 2026-08-29T14:30:00
- **Estado:** ⏳ IN PROGRESS

## Impacto mapeado (Regla 0)
- **Archivos leídos completos:**
  - `src/cli_server.rs:137-161` (ServerState fields, todos `pub`)
  - `src/cli_server.rs:445-486` (AuthState struct + `AuthState::new`)
  - `src/cli_server.rs:732-832` (auth_middleware — acepta ambas keys con `ct_eq`)
  - `src/config.rs:285-307` (VantaConfig.api_key, alt_api_key, require_auth)
  - `src/config.rs:675-691` (env loading VANTADB_API_KEY/VANTADB_ALT_API_KEY)
  - `src/rbac.rs:30-80` (Rbac struct, métodos `pub`, **struct es `pub(crate)` → NO accesible desde integration tests**)
  - `src/cli_server_auth_tests.rs:1-100` (patrón de test AAA, helper `auth_state`, `server_state`, `spawn`, `http_get`)
  - `Cargo.toml:62-67,99-100,170-184` (tokio/axum son opcionales vía `feature = "server"`)
- **Referencias hacia dentro del cambio:**
  - `tests/server_auth_rotation.rs` (integration) → usa `VantaConfig`, `RbacConfig`, `ServerState`, `app()`, `BackendKind::InMemory`, `StorageEngine`, `VantaEmbedded`, `CircuitBreaker`, `ConnectionPool` — todos `pub`.
  - `src/cli_server_auth_tests.rs` (unit, `#[path]` include) → `super::*` con acceso a `Rbac` interno, `AuthState::new`, `token_role_map`.
- **Referencias entrantes:** 65 callers de `ServerState` en `src/cli_server.rs`; 3 callers de `AuthState`; 5 callers de `RbacConfig`. Tests: `cli_server_auth_tests.rs`, `vantadb-server/tests/`.
- **Veredicto:** bajo riesgo. **No tocamos `config.rs` ni `cli_server.rs`** (la lógica de auth + alt key ya está implementada, validada y auditada). Solo añadimos tests y docs. Stop condition del prompt: ">1d → docs-only" → NO escalamos a `Vec<String>` (scope-creep, ya hay audit del path actual).

## Blast Radius
**Callers:** `cli_server.rs` (auth middleware, lines 796-809), `config.rs` (VantaConfig, lines 292-300, 683-687, 960-972)
**Callees:** None new (uses existing `Arc<str>`, `constant_time_eq`)
**Implicaciones:**
- Config: `VANTADB_API_KEY`, `VANTADB_ALT_API_KEY` env vars ya implementados
- Auth middleware ya acepta ambas keys (líneas 796-809 en cli_server.rs)
- Falta: test de rotación, docs SECURITY.md, validación RBAC para alt key

## Contrato
```
cargo test -p vantadb --test server_auth_rotation 2>&1 | Select-String "rotat.*ok|2 passed" | Measure-Object | Select-Object Count >= 1
```
Test con old+new activas simultáneamente.

**Nota mecánica:** `tests/server_auth_rotation.rs` requiere `--features "server cli"` para compilar
(`cli_server` solo se compila con esos features). El comando del contrato en su forma literal
retorna Count=0 porque el crate `vantadb` no compila los integration tests de `cli_server` con
default features. Con `--features "server cli"`:
```
$ cargo test -p vantadb --test server_auth_rotation --features "server cli" 2>&1 | Select-String "rotat.*ok|2 passed" | Measure-Object | Select-Object Count
3   (Count=3: 2 "rotat...ok" + 1 "2 passed")
```

## Herramientas
- cargo-mcp, rust-analyzer-mcp, codegraph
- webfetch para validar patrón Qdrant alt_api_key

## Steps

### Step 1: Crear test de rotación integration (`tests/server_auth_rotation.rs`)
- **Archivos:** `tests/server_auth_rotation.rs` (nuevo) — 79 líneas, 2 tests
- **Acción:** 2 tests integration con `tokio::test` (AAA):
  1. `rotation_old_and_new_active_simultaneously` — server con `api_key="old"` + `alt_api_key="new"`, ambos Bearer pasan a `/api/v2/health`
  2. `rotation_promote_alt_to_primary_revokes_old` — server con `api_key="new"`, `alt_api_key=None`; `old` falla con 401, `new` pasa
- **Verify:** `cargo test -p vantadb --test server_auth_rotation --features "server cli"` → 2/2 OK ✅
- **Estado:** ✅ COMPLETED

### Step 2: Documentar env vars y patrón rotación en SECURITY.md
- **Archivos:** `docs/operations/SECURITY.md` (+68 líneas)
- **Acción:** (a) corregir drift en audit log docs (código real usa `key: "N/A"`, no hash); (b) agregar ejemplo programático de `token_role_map` con notas sobre cómo se aplica a ambas keys; (c) actualizar `last_reviewed: 2026-08-29`; (d) nota sobre el env var loader pendiente en FIND-49
- **Verify:** `Select-String -Path "docs/operations/SECURITY.md" -Pattern "VANTADB_ALT_API_KEY|rotation" | Measure-Object | Select-Object Count` = 13 (>= 2 ✅)
- **Estado:** ✅ COMPLETED

### Step 3: Validar RBAC token_role_map funciona con alt_api_key (unit test)
- **Archivos:** `src/cli_server_auth_tests.rs` (+141 líneas: helper `server_state_with_alt_rbac` + 3 unit tests)
- **Acción:** 3 unit tests `#[tokio::test]`:
  1. `auth_l1_alt_key_with_rbac_token_role_mapping_passes` — alt Bearer + role "reader" (pre-registered) → 200. Prueba que alt_api_key está wired y token_role_map se consulta.
  2. `auth_l1_alt_key_unknown_role_in_map_denied` — alt Bearer + role inexistente en map → 403 en `/api/v2/records` (record endpoint toma el RBAC path).
  3. `auth_l1_alt_key_unknown_role_falls_through_to_transport` — alt Bearer sin entry en map → bare transport → 200.
- **Verify:** `cargo test -p vantadb --features "server cli" --lib "auth_l1_alt"` → 3/3 OK ✅ (y los otros 22 tests `auth_*` siguen pasando: 25/25 ✅)
- **Estado:** ✅ COMPLETED

### Step 4: Verify full (fmt/clippy/nextest/docs) + commit
- **Archivos:** todos los tocados
- **Verify:** `cargo fmt --check` ✅ | `cargo clippy -p vantadb --features "server cli" --tests` ✅ (0 errors) | `cargo test -p vantadb --features "server cli" --test server_auth_rotation` 2/2 ✅ | `cargo test -p vantadb --features "server cli" --lib "auth_"` 25/25 ✅ | docs `Select-String` count=13 ✅
- **Estado:** ✅ COMPLETED (4 archivos staged, commit pendiente — bloqueado por rol `vanta-worker` no autoriza git commit; vanta-lead ejecuta)

## Dependencias
- GOV-TK3 ✅ COMPLETED (prereq del plan)

## Notas
- Patrones: Qdrant v1.17 `alt_api_key` (confirmado en config.rs comentarios)
- `token_role_map` ya existe en RbacConfig — verificar que aplica a ambas keys
- Gate security: auth middleware toca trust boundary → security-and-hardening checklist
- **Decisión de scope:** stop condition del prompt ">1d → docs-only" → NO escalamos a `Vec<String>`. Validamos el path actual (`api_key` + `alt_api_key`) y lo documentamos.

## Context Save Point
- **Fecha:** 2026-08-29
- **Branch:** develop
- **CI pendiente:** sí
- **Decisiones:** 
  - alt_api_key ya implementado en config + auth middleware (validado por codegraph blast radius)
  - Solo faltan: test dedicado integration (`tests/server_auth_rotation.rs`), unit test RBAC en `cli_server_auth_tests.rs`, docs `SECURITY.md`
  - NO se refactoriza a `Vec<String>` — stop condition explícito del prompt + refactor viejo a medias en develop roto
  - RBAC se valida con unit test (no integration) porque `Rbac` es `pub(crate)`
  - Refactor `Vec<String>` queda como deuda en `tests/api/server_auth_rotation.rs` (untracked, preexistente) — registrar como FIND-49
- **Problemas conocidos:** Ninguno en mi scope. Pre-existen: rama `develop` tiene cambios M (no míos) en `completions/`, `docs/avance/`, `docs/plans/`, `.opencode/task-system/memory/lessons.md` que NO toco.
- **Próxima tarea:** SRV-02 (tracing-id)

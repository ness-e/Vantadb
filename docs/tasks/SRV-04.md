# Task: SRV-04 — Multi API keys + rotación sin downtime

**Campaign ID:** b28f-20260828-backlog-triage
**Plan file:** docs/plans/2026-08-28-backlog-triage.md
**State:** ✅ COMPLETED
**Completed:** 2026-09-01

---

## Archivos clave

- `src/server/state.rs` — `ServerState` con `api_key` + `alt_api_key` + `token_role_map`
- `src/server/routing.rs` — `auth_middleware` validando ambas keys (líneas 445-458)
- `src/config.rs` — `VantaConfig` con `api_key` + `alt_api_key` desde env vars
- `.opencode/rules/server-mcp.md` — Reglas del server
- `tests/server_auth_rotation.rs` — Tests de rotación (2 tests passing)

---

## Descubrimiento (SDP)

**Skills cargadas:**
- `security-and-hardening` — validación de trust boundaries, constant-time comparison
- `test-driven-development` — tests existentes verifican el contrato
- `codebase-memory` — blast radius analysis
- `api-and-interface-design` — contrato de API estable

---

## Impacto mapeado (Regla 0)

### Archivos leídos completos:
- `src/server/state.rs:104-128` — `ServerState` struct con `api_key`, `alt_api_key`
- `src/server/state.rs:183-224` — `AuthState` con `api_key`, `alt_api_key`, `token_role_map`
- `src/server/routing.rs:137-145` — `AuthState::new` construyendo estado de auth
- `src/server/routing.rs:445-458` — Validación de ambas keys en `auth_middleware`
- `src/config.rs:319-327` — `api_key` + `alt_api_key` en `VantaConfig`
- `src/config.rs:710-713` — Carga desde `VANTADB_API_KEY` + `VANTADB_ALT_API_KEY`

### Referencias hacia dentro (lo que usan estos archivos):
- `AuthState` usa `RbacConfig`, `Rbac`, `AuthRateLimiter`, `StorageEngine`, `AuditLogger`
- `ServerState` usa `StorageEngine`, `VantaEmbedded`, `CircuitBreaker`, `ConnectionPool`

### Referencias entrantes (quién los usa):
- `tests/server_auth_rotation.rs` — Tests de integración
- `src/cli_server.rs` — Shim de compatibilidad (re-exporta `crate::server::*`)
- `vantadb-server/` — Binary del servidor HTTP
- `vanta-memory/` — Tests de conversation hook

### Veredicto de impacto:
**Bajo** — La funcionalidad ya estaba implementada en el código. Los tests validan el contrato. Solo se requiere documentación y cierre de la tarea.

---

## Implementación

### Estado actual (verificado):
1. ✅ `ServerState` tiene `api_key: Option<Arc<str>>` + `alt_api_key: Option<Arc<str>>` (state.rs:117-119)
2. ✅ `AuthState` tiene `api_key` + `alt_api_key` + `token_role_map` (state.rs:187-191)
3. ✅ `AuthState::new` construye ambos desde config (state.rs:213-215)
4. ✅ `auth_middleware` acepta **ambas** keys simultáneamente (routing.rs:446-458):
   ```rust
   let authorized = match token {
       Some(token) => {
           let token_bytes = token.as_bytes();
           let primary_ok = expected_key.as_bytes().ct_eq(token_bytes).into();
           let alt_ok = auth
               .alt_api_key
               .as_ref()
               .map(|alt| alt.as_bytes().ct_eq(token_bytes).into())
               .unwrap_or(false);
           primary_ok || alt_ok
       }
       None => false,
   };
   ```
5. ✅ Constant-time comparison con `subtle::ConstantTimeEq` previene timing attacks
6. ✅ Config carga desde `VANTADB_API_KEY` + `VANTADB_ALT_API_KEY` (config.rs:710-713)
7. ✅ Tests pasan: `rotation_old_and_new_active_simultaneously` + `rotation_promote_alt_to_primary_revokes_old`

### RBAC mapping verificado:
- `token_role_map` se consulta con el **valor literal del token** presentado por el cliente (routing.rs:511)
- Tanto primary como alt key pasan por el mismo mapa — no hay escalación de privilegios automática
- Para roles distintos por key, se configuran ambas entradas en el mapa

### Documentación existente (SECURITY.md):
- Sección "API Key Rotation (Zero-Downtime)" ya documenta el flujo completo
- Variables de entorno: `VANTADB_API_KEY`, `VANTADB_ALT_API_KEY`, `VANTADB_REQUIRE_AUTH`
- Workflow de rotación con ejemplo bash
- Notas de seguridad: constant-time comparison, `alt_api_key` requiere `api_key`, RBAC aplica a ambas

---

## Verificación

```bash
# Tests de rotación (contrato: 2 passed)
cargo test -p vantadb --test server_auth_rotation --features server

# Check general
cargo check -p vantadb --features server
```

**Resultado:**
```
running 2 tests
test rotation_old_and_new_active_simultaneously ... ok
test rotation_promote_alt_to_primary_revokes_old ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.50s
```

---

## Gates evaluados

| Gate | Estado | Evidencia |
|------|--------|-----------|
| **Gate Regla 0** | ✅ | Impacto mapeado arriba antes de cualquier edición (ninguna edición necesaria) |
| **Security** | ✅ | `security-and-hardening` checklist: constant-time comparison, no secret logging, fail-closed auth |
| **Tests** | ✅ | 2 tests passing, cubren rotación old+new simultáneo + promote/revoke |
| **Docs** | ✅ | SECURITY.md ya documenta `VANTADB_API_KEY` + `VANTADB_ALT_API_KEY` + workflow |
| **Build** | ✅ | `cargo check -p vantadb --features server` — solo warnings pre-existentes |

---

## Commits

**Commit:** `feat(SRV-04): add multi API key rotation without downtime`
- No code changes needed — implementation already complete
- Task file created for traceability
- Tests verified passing

---

## Invariantes (qué NO se puede romper)

1. **Constant-time comparison** — `subtle::ConstantTimeEq` para ambas keys (routing.rs:449,453)
2. **Fail-closed auth** — Sin `api_key` configurado → dev mode (warning logged), con `REQUIRE_AUTH=true` → refuse to start
3. **RBAC por token literal** — `token_role_map` mapea el Bearer exacto presentado, no el "slot" (primary/alt)
4. **Rate limiting** — 5 intentos / 60s por IP antes de 429 (AuthRateLimiter)
5. **Audit logging** — `auth_l1` events registrados con outcome/reason, **nunca** el Bearer raw

---

## Deuda pendiente

- **N keys > 2**: Config actual solo soporta 2 keys fijas (`api_key` + `alt_api_key`). Extensión a N keys requeriría `Vec<Arc<str>>` + ventana de rotación configurable (issue backlog: SRV-06)
- **Env var para token_role_map**: `VANTADB_TOKEN_ROLE_<KEY>=<role>` propuesto en FIND-49 (Backlog)

---

## Próximos pasos

Ninguno — tarea completa. El orquestador puede continuar con la siguiente tarea del plan.
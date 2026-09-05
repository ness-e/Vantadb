# AUDREP-13: Dev mode bypassa toda autenticación silenciosamente

## Metadata
- **Plan file:** docs/plans/2026-08-05-backlog-validation-actions.md (Phase 13)
- **Fuente:** docs/Backlog.md línea 472
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟠
- **Tipo:** Rust (server/auth)
- **Estado:** ⬜ PENDING

## Blast Radius
| Dirección | Módulos |
|-----------|---------|
| Callers | HTTP auth middleware (`cli_server.rs` router) |
| Callees | `next.run(req)`, `client_ip`, `rate_limiter` |
| Implicaciones | Sin `VANTADB_API_KEY`, todas las requests pasan sin auth NI logging (`let Some(expected_key) = &auth.api_key else { return next.run(req).await }`). Fix: loggear warning (rate-limited para no spamear) por request no autenticada en dev mode. NO cambiar el comportamiento de allow-all (dev mode es intencional) — solo visibilidad. |

## Contrato
"`cargo check -p vantadb --features server` pasa; `cargo clippy -p vantadb -- -D warnings` pasa; cada request en dev mode (sin API key) emite un log de warning identificable; comportamiento allow-all preservado; no hay warning por request en modo autenticado."

## Herramientas
- cargo-mcp (check, clippy, fmt), rust-analyzer-mcp

## Steps
### Step 1: Investigar logging disponible
- **Archivos:** `src/cli_server.rs:279-282` + cómo se loguea en el resto del middleware (tracing macros, rate limit de logs)
- **Acción:** leer el middleware completo (función, middleware de rate limit, `auth` struct) y decidir el patrón: warning por request con `tracing::warn!` (¿ya hay una vez-per-window? si no, considerar `tracing::warn!` simple o rate-limited con `tracing::rate_limited` si la versión lo soporta — validar API).
- **Verify:** comprensión; sin cambios aún.
- **Estado:** ⬜ PENDING

### Step 2: Aplicar fix
- **Archivos:** `src/cli_server.rs`
- **Acción:** en el brazo `else { return next.run(req).await }`, loggear warning (request path/method) antes de `next.run`. Mínimo posible; no tocar lógica de auth.
- **Verify:** `cargo check -p vantadb --features server`
- **Estado:** ⬜ PENDING

### Step 3: Verificación
- **Archivos:** tests existentes de cli_server si aplica
- **Acción:** test (si el harness lo permite) o verificación manual con `vantadb-server` sin API key: request responde 200 y emite el warning. Si no hay test framework para el log, verificar con build + inspección.
- **Verify:** `cargo fmt --check` + `cargo clippy -p vantadb -- -D warnings` + tests server relevantes
- **Estado:** ⬜ PENDING

## Dependencias
- Ninguna.

## Notas
- Backlog: "Recomendación: loggear warning por request no autenticada en dev mode".
- Commit selectivo: SOLO `src/cli_server.rs` + tests tocados.

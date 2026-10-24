--
status: completed
title: "AUDREP-13: Dev mode bypassa toda autenticaciÃ³n silenciosamente"
kind: task
description: cargo check -p vantadb --features server pasa; cargo clippy -p vantadb -- -D warnings pasa; cada request en dev mode (sin API key) emite un log de warning identificable; comportamiento allow-all preservado; no hay warning por request en...
---

# AUDREP-13: Dev mode bypassa toda autenticaciÃ³n silenciosamente

## Metadata
- **Plan file:** docs/dev/plans/2026-08-05-backlog-validation-actions.md (Phase 13)
- **Fuente:** docs/dev/Backlog.md lÃ­nea 472
- **Esfuerzo:** ðŸŸ¢ 1h
- **Prioridad:** ðŸŸ 
- **Tipo:** Rust (server/auth)
- **Estado:** â¬œ PENDING

## Blast Radius
| DirecciÃ³n | MÃ³dulos |
|-----------|---------|
| Callers | HTTP auth middleware (`cli_server.rs` router) |
| Callees | `next.run(req)`, `client_ip`, `rate_limiter` |
| Implicaciones | Sin `VANTADB_API_KEY`, todas las requests pasan sin auth NI logging (`let Some(expected_key) = &auth.api_key else { return next.run(req).await }`). Fix: loggear warning (rate-limited para no spamear) por request no autenticada en dev mode. NO cambiar el comportamiento de allow-all (dev mode es intencional) â€” solo visibilidad. |

## Contrato
"`cargo check -p vantadb --features server` pasa; `cargo clippy -p vantadb -- -D warnings` pasa; cada request en dev mode (sin API key) emite un log de warning identificable; comportamiento allow-all preservado; no hay warning por request en modo autenticado."

## Herramientas
- cargo-mcp (check, clippy, fmt), rust-analyzer-mcp

## Steps
### Step 1: Investigar logging disponible
- **Archivos:** `src/cli_server.rs:279-282` + cÃ³mo se loguea en el resto del middleware (tracing macros, rate limit de logs)
- **AcciÃ³n:** leer el middleware completo (funciÃ³n, middleware de rate limit, `auth` struct) y decidir el patrÃ³n: warning por request con `tracing::warn!` (Â¿ya hay una vez-per-window? si no, considerar `tracing::warn!` simple o rate-limited con `tracing::rate_limited` si la versiÃ³n lo soporta â€” validar API).
- **Verify:** comprensiÃ³n; sin cambios aÃºn.
- **Estado:** â¬œ PENDING

### Step 2: Aplicar fix
- **Archivos:** `src/cli_server.rs`
- **AcciÃ³n:** en el brazo `else { return next.run(req).await }`, loggear warning (request path/method) antes de `next.run`. MÃ­nimo posible; no tocar lÃ³gica de auth.
- **Verify:** `cargo check -p vantadb --features server`
- **Estado:** â¬œ PENDING

### Step 3: VerificaciÃ³n
- **Archivos:** tests existentes de cli_server si aplica
- **AcciÃ³n:** test (si el harness lo permite) o verificaciÃ³n manual con `vantadb-server` sin API key: request responde 200 y emite el warning. Si no hay test framework para el log, verificar con build + inspecciÃ³n.
- **Verify:** `cargo fmt --check` + `cargo clippy -p vantadb -- -D warnings` + tests server relevantes
- **Estado:** â¬œ PENDING

## Dependencias
- Ninguna.

## Notas
- Backlog: "RecomendaciÃ³n: loggear warning por request no autenticada en dev mode".
- Commit selectivo: SOLO `src/cli_server.rs` + tests tocados.

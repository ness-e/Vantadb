# AUDREP-12: Limitar tamaño de body en endpoint /api/v2/query

## Metadata
- **Plan file:** docs/plans/2026-08-05-backlog-validation-actions.md (plan completado; tarea tomada directo de Backlog)
- **Fuente:** docs/Backlog.md:471
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟠 ALTO
- **Tipo:** Rust (server)
- **Turns estimados:** 5-8
- **Creado:** 2026-08-06
- **last-synced:** 2026-08-06
- **Estado:** ✅ COMPLETED

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `run()` → `app()` (cli_server.rs:781); tests unitarios in-file |
| Callees | axum 0.8 (`Router`, `DefaultBodyLimit` — feature `server`) |
| Implicaciones | Contrato HTTP: `/api/v2/query` y `/metrics` rechazan body > 1MB con 413. No cambia API pública del SDK. No rompe tests existentes (solo agrega capa). |

## Contrato
"cargo check -p vantadb --features server pasa; test nuevo `body_limit_rejects_oversized` pasa; cargo nextest run --profile audit -p vantadb pasa"

## Herramientas necesarias
- cargo-mcp (check, nextest)
- rust-analyzer-mcp (diagnostics)

## Investigation Notes
- Router actual en `src/cli_server.rs:149-159`: `Router::new().merge(public).merge(protected).layer(...)`. Sin `DefaultBodyLimit` → DoS por body gigante en POST `/api/v2/query`.
- Fix: `.layer(axum::extract::DefaultBodyLimit::max(1_000_000))` en el router raíz (cli_server.rs:149). Axum 0.8 ya lo incluye (sin dependencia nueva).
- tower-http solo tiene feature `trace` — no necesario para esto.

## Steps

### Step 1: Agregar DefaultBodyLimit al router
- **Archivos:** `src/cli_server.rs` (~149-159)
- **Acción:** agregar `.layer(DefaultBodyLimit::max(1_000_000))` al `Router::new()` raíz.
- **Verify:** `cargo check -p vantadb --features server`
- **Estado:** ✅ COMPLETED (fix + test; check/test/fmt/clippy verdes)

### Step 2: Test de rechazo de body > 1MB
- **Archivos:** `src/cli_server.rs` (mod tests, ~969)
- **Acción:** test que construye el router y verifica que un POST a `/api/v2/query` con body > 1MB devuelve 413.
- **Verify:** `cargo nextest run --profile audit -p vantadb --test cli_server` (o test in-file)
- **Estado:** ✅ COMPLETED

### Step 3: Verify completo
- **Archivos:** —
- **Acción:** `cargo check --workspace` + `cargo fmt --check` + `cargo clippy -p vantadb --features server -- -D warnings`
- **Verify:** todos pasan
- **Estado:** ✅ COMPLETED

## Dependencias
- Ninguna

## Notas
- No tocar `vantadb-server/` (tiene router propio, fuera de scope de este hallazgo).
- 1MB recomendado por el hallazgo original del backlog.

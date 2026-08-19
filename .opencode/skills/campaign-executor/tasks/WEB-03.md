# WEB-03 — Servir estáticos `/dashboard` + SPA fallback + flag CLI

> Plan: `docs/plans/2026-08-18-vanta-studio-fase3.md` · Wave 1 · Estado: ⏳ IN PROGRESS (2026-08-19)
> Contexto: WEB-01 (c81bc23a) / WEB-02 (c856b3bd) ya agregaron los REST v2 en `src/cli_server.rs`. Esta tarea agrega el server de estáticos sin tocar endpoints existentes.

## Impacto mapeado (Regla 0)

**Archivos leídos completos:**
- `src/cli_server.rs` (2976L) — `app()`/`app_with_cors()` (L127-234), `ServerState` (L106-125), `run()` (L1339-1393), `serve_http_or_tls` (L1482+), módulo tests (L1992+, 5 constructores de `ServerState` en L2052/2141/2264/2373/2635)
- `src/cli_handlers/server.rs` (352L) — `cmd_server` (L173) → `cmd_server_http` (L215) → `cli_server::run(config)` (L242)
- `src/cli.rs` (405L) — comando `Server` (L296-316): `http/mcp/port/host/require_auth`
- `src/bin/vanta-cli.rs` (220L) — dispatch `Commands::Server` (L195-210) → `cmd_server`
- `src/config.rs` (1610L) — `VantaConfig` struct (L221+), `Default` (L491-799, `from_env()` = `Self::default()` L804), builders con `..Default::default()`
- `Cargo.toml` — `tower-http = { version = "0.6.11", features = ["trace", "cors"], optional = true }` (L63); feature `server` incluye `dep:tower-http` (L128-134)

**Referencias entrantes (quién depende de lo que cambio):**
- `app()` / `app_with_cors()` — tests de `cli_server.rs` (L2052-2068, L2156, L2170) + `run()` (L1383). NO cambio sus firmas → no se rompen.
- `cmd_server(...)` — único caller `vanta-cli.rs` L201. Cambio firma → actualizo caller.
- `VantaConfig` — muchos constructores usan `..Default::default()` (tests, SDK). Agregar campo con default en `impl Default` → no se rompen.
- `Commands::Server` — clap derive; agregar campo → único consumer `vanta-cli.rs`.

**Referencias salientes (de lo que depende):**
- `tower-http` (ya dep opt-in con feature `server`) — agrego feature `fs` para `ServeDir`/`ServeFile` (docs.rs/tower-http/0.6.11 `ServeDir`: disponible solo con feature `fs`; `fallback()` llamado solo si no hay archivo en el path; `append_index_html_on_directories` default true).
- `axum::service_fn` (re-export de `tower::service_fn`, core de axum 0.8) — fallback SPA por extensión.

**Veredicto de impacto:** cambio aditivo. No toco rutas existentes; monto `/dashboard` en `run()` (fuera del middleware auth → público por D12). Tests existentes de `app()` no se ven afectados porque `mount_dashboard` se aplica solo en `run()`.

## Fases / Steps

### FASE 1 — DISCOVERY
- [x] Leer plan file + pipeline-full.md + task context
- [x] Verificar `tower-http` en Cargo.toml (existe, falta feature `fs`)
- [x] Mapear `app()`/`app_with_cors()`/`run()`/`ServerState`/tests
- [x] Mapear flujo CLI: `Commands::Server` → `cmd_server` → `cmd_server_http` → `run(config)`
- [x] Verificar API oficial tower-http 0.6.11 `ServeDir` (feature `fs`, `fallback()`, `append_index_html_on_directories`)
- [x] Decidir approach: `mount_dashboard(router, dir)` en `run()` — no rompe firmas públicas ni tests

### FASE 2 — IMPLEMENTACIÓN
- [x] `Cargo.toml`: feature `"fs"` en tower-http
- [x] `Cargo.toml`: dep `tower = "0.5"` opt-in (fallback SPA usa `tower::service_fn` — `axum::service_fn` NO existe en axum 0.8.9)
- [x] `src/config.rs`: campo `dashboard_dir: Option<PathBuf>` + default desde `VANTADB_DASHBOARD_DIR`
- [x] `src/cli.rs`: flag `--dashboard-dir` en `Commands::Server`
- [x] `src/bin/vanta-cli.rs`: pasar `dashboard_dir` a `cmd_server`
- [x] `src/cli_handlers/server.rs`: `cmd_server` + `cmd_server_http` aceptan y propagan `dashboard_dir`
- [x] `src/cli_server.rs`: `mount_dashboard()` (ServeDir + SPA fallback por extensión + hint 404) + llamada en `run()`
- [x] Tests: dashboard sirve estáticos + SPA fallback + 404 hint (en `mod tests` de cli_server.rs)

### FASE 3 — VERIFICACIÓN
- [x] `cargo fmt` / `cargo fmt --check` — limpio
- [x] `cargo check --features server` — PASS (1 error intermedio: `axum::service_fn` no existe → fix `tower::service_fn`)
- [x] `cargo test -p vantadb --features server --lib -- cli_server` — 24/24 PASS (2 nuevos: `dashboard_serves_static_files_and_spa_fallback`, `dashboard_disabled_returns_404_hint`)
- [x] Smoke: dir de prueba → `vanta-cli --db <temp> server --port 18080 --dashboard-dir <dir>` → 5 casos OK: `/dashboard/` 200 index, `/dashboard/alguna-ruta-spa` 200 index, `/dashboard/assets/x.js` 200 asset, `/dashboard` 200 index, `/dashboard/assets/no-existe.js` 404 "Not found" sin index
- [x] Limpiar artifacts de prueba (db temp, dirs, logs, proceso)

### FASE 4 — CIERRE
- [x] Dejar archivos formateados + tests verdes
- [x] NO commit (lead commitea tras verify)
- [x] RESULTADO block

## Context Save Point (si se interrumpe)
- Files a tocar: `Cargo.toml`, `src/config.rs`, `src/cli.rs`, `src/bin/vanta-cli.rs`, `src/cli_handlers/server.rs`, `src/cli_server.rs`
- Approach: `mount_dashboard(router, Option<&Path>)` llamada en `run()` después de `app_with_cors`; fallback SPA = `ServeDir::fallback(axum::service_fn(...))` que sirve index.html solo para rutas sin extensión; sin dir → 404 con hint.
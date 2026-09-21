# TECH-06 — CORS como feature request (reformular)

- **Estado:** ✅ CERRADO (2026-08-05) — sin consumidor browser real
- **Archivos clave:** `src/cli_server.rs`, `src/config.rs`, `docs/api/HTTP_API.md:148-150`
- **Contrato plan:** CORS configurable; default sin headers nuevos; test e2e con Origin header.

## Decisión

**Cerrar documentando la decisión** (opción b del plan). No se tocó código.

## Evidencia

1. **No existe desktop/webview**: `Test-Path src-tauri/desktop/webview` = `False` — no hay código Tauri/wry/WebView en el repo.
2. **web/ no fetchea el HTTP server**: los únicos `fetch()` son `web/public/vanta-wasm/vantadb_wasm.js:1409` y `web/src/components/vanta/code-playground.tsx:120` → cargan el **binario WASM** (VantaDB corre in-browser vía WASM, no contra el HTTP API). `docs-view.tsx:399` es string hardcodeado de ejemplo.
3. **Todos los consumidores del HTTP API son nativos/server-side**: reqwest en `vantadb-server/tests/{benchmarks,e2e,server}.rs`, `src/llm.rs`, `src/wal_shipping.rs`.
4. **El propio backlog lo confirma**: "el plan desktop lo evita (reqwest desde Rust)" (`docs/Backlog.md:101`, `docs/Investigaciones/DESKTOP-01b...`).
5. `tower-http` (0.6.11, features `trace`) ya es dep opcional tras feature `server` — el módulo `cors` está disponible si algún día se necesita.

## Acción tomada

- `docs/api/HTTP_API.md` — sección CORS actualizada: decisión documentada + cuándo activar (reverse proxy recomendado; o middleware tower-http `cors` con `cors_origins` en `VantaConfig`, default OFF, nunca `AllowOrigin::any()`).

## Verificación

- `git diff docs/api/HTTP_API.md` — solo cambio de docs.
- Sin cambios de código → no requiere `cargo check`.
- Commit: `docs(TECH-06): cerrar CORS — sin consumidor browser real`

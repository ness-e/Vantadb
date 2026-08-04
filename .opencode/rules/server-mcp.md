# Server & MCP — Reglas

> **Scope:** `vantadb-server/` (`src/server.rs`, `middleware.rs`, `main.rs`, `lib.rs`), `vantadb-mcp/` (`src/lib.rs` — MCP stdio)
> **No tocar aquí:** bindings Python (`python-bindings.md`), JS/WASM (`js-ecosystem.md`), API pública del core (`api-contract.md`)
> **Status:** 🟢 Vigente
> **Fuentes:** INV-011, INV-003

## Reglas

### R-1: serverInfo/versiones coherentes entre doc y código

- **Must:** mantener `serverInfo.name`/`version` de `vantadb-mcp` (`src/metadata.rs`) y los recursos/tools documentados (MCP.md) sincronizados con la implementación real.
- **Must not:** documentar tools que no existen (`query` vs `query_lisp`) ni versiones de protocolo imaginarias.
- **Por qué:** la auditoría INV-011 detectó drift entre MCP.md y el código (tool `query` inexistente, `schema://` fantasma, versión 0.1.5 vs protocolo real 2024-11-05).

### R-2: Concurrencia de handlers vía semáforo + `spawn_blocking`

- **Must:** limitar concurrencia con `tokio::sync::Semaphore` (`acquire_owned()` en `vantadb-mcp`) y ejecutar el trabajo del motor en `tokio::task::spawn_blocking`.
- **Must not:** bloquear el event loop del servidor con trabajo síncrono del motor ni inventar límites con mutex síncronos.
- **Por qué:** ver `concurrency-async.md` R-3 y R-7 (INV-003) — los handlers MCP son el mismo patrón que el HTTP server.

<!-- Referencias cruzadas: → ver concurrency-async.md, api-contract.md, release-ci.md -->

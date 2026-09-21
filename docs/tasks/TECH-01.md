# TECH-01 — Fix `--db` en MCP server (P0)

**Estado:** ✅ COMPLETADO (2026-08-05)
**Commit:** `9d085d001dc07d00f4bae27c3f5355c372a6ec06` (branch `develop`)
**Archivo:** `src/cli_handlers/server.rs` (+3 líneas, 1 archivo)

## Bug

El hijo `vantadb-server --mcp` resuelve storage vía `VantaConfig::from_env()` → `VANTADB_STORAGE_PATH` (`src/config.rs:408`, fallback `"vantadb_data"`), pero el padre setea solo `VANTA_DB` (`src/cli_handlers/server.rs:244`) → la DB caía en `vantadb_data` del CWD en vez del path pedido con `--db`.

## Fix (1-línea + comentario ADR-012)

```rust
cmd.env("VANTA_DB", db_path);
// ADR-012: el child resuelve storage via VantaConfig::from_env()
// -> VANTADB_STORAGE_PATH (config.rs). VANTA_DB es solo flag CLI.
cmd.env("VANTADB_STORAGE_PATH", db_path);
```

AÑADIR, no reemplazar — ver `docs/architecture/adr/012_env_var_naming.md`.

## Verificación

- `cargo check -p vantadb` ✅
- `cargo check -p vantadb --features server,cli` ✅
- `cargo build -p vantadb --features server,cli --bin vanta-cli` ✅ (5m01s)
- **E2E** `vanta-cli server --mcp --db $env:TEMP\tech01-e2e` ✅:
  - Lock `.vanta.lock` + persistencia (WALs, `vector_index.bin`, keyspaces) creados en `%TEMP%\tech01-e2e`
  - `vantadb_data` NO creado en CWD
  - MCP stdio server shut down limpio (el harness cierra stdin; storage flush OK)

## Contexto

- Child `vantadb-server.exe` ya existía en `target\debug` — no requirió rebuild (el fix está en el padre).
- Sin test unitario de `cmd_server_mcp` (spawna proceso) — verificación por e2e manual en Windows.
- Limpieza e2e pendiente opcional: `%TEMP%\tech01-e2e`, `%TEMP%\tech01-{out,err}.log`.

# TECH-03 — Corregir 3 stale-docs reales (de 4)

**Plan:** `docs/plans/2026-08-05-backlog-validation-actions.md` — Task 20
**Estado:** ✅ COMPLETED 2026-08-05

## Contexto

4 stale-docs detectados en investigación DESKTOP-01b (§6.3-6.6). 3 reales + 1 nomenclatura (NO tocar: `python_sdk` feature SÍ existe en Cargo.toml:104 + src/python.rs:1).

## Fixes aplicados

### 1. `docs/api/HTTP_API.md:124-125` — claim "Full MCP + HTTP" falso
- **Antes:** `# Full MCP + HTTP` / `vanta-cli server --http --mcp --port 8080 --db ./vanta_data`
- **Después:** comentario refleja exclusividad: `mcp_mode = mcp && !http` (src/cli_handlers/server.rs:182) — pasar `--http --mcp` juntos arranca solo MCP; HTTP no se sirve. Ejemplo corregido a `--mcp` solo.

### 2. `vantadb-python/README.md:33,48,59` — métodos inexistentes
- **L33:** `put_memory(...)` → `put(...)` (firma idéntica en vantadb-python/src/lib.rs:676: `namespace, key, payload, metadata, vector`).
- **L48:** `search_hybrid(..., query_text=...)` → `search_memory(..., text_query=...)` (lib.rs:899; param real `text_query`).
- **L59:** `memory_stats()` → `operational_metrics()` (lib.rs:1210); claves de dict corregidas a las reales: `hnsw_logical_bytes` y `process_rss_bytes` (convert.rs:596-599) en vez de `logical_bytes`/`physical_rss`.

### 3. `docs/api/MCP.md:56` — tool `query` vs `query_iql`
- **Sin cambios:** YA corregido por Task 13/AUD-004 en commit `f097bac7` (diff: `- **\`query\`**` → `+ **\`query_iql\`**`). Código real: vantadb-mcp/src/lib.rs:898 (`"name": "query_iql"`) + :1147 (handler).

## NO tocado
- Item (3) del plan: `python_sdk` feature (drift de nomenclatura, no falsedad — feature existe).

## Verificación (contrato)
- `rg put_memory|search_hybrid|memory_stats` en `vantadb-python/README.md` = 0 hits. Restantes fuera de scope (Backlog.md, plan, backups, Investigaciones) o `get_memory_stats` del engine Rust (método distinto, MEMORY_TELEMETRY.md).
- Métodos reales confirmados: `fn put` lib.rs:676, `fn search_memory` :899, `fn operational_metrics` :1210, `fn get_memory` :731, `fn put_batch` :312.
- `mcp_mode` confirmado: server.rs:182 `let mcp_mode = mcp && !http;`
- `query_iql` confirmado: lib.rs:898 + :1147 + MCP.md:56.
- `pwsh scripts/validate-docs-coverage.ps1` → **EXIT=0, 0 gaps** (no rompido; Task 19 lo dejó verde).

## Commit
`docs(TECH-03): corregir 3 stale-docs (MCP excluyente, API python real, query_iql)`

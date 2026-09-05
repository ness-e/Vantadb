# TASK-ID: ERR-MCP-01 — From<VantaError> for McpError + isError con code/retriable (CRITICAL, Wave 2)

## Metadata
- **Plan file:** `docs/plans/2026-09-02-error-observability-excellence.md` (Task 5, Wave 2 paralelo)
- **Creado:** 2026-09-02 (vanta-worker)
- **last-synced:** 2026-09-02
- **Estado:** ✅ COMPLETED (2026-09-02 — commits `feat(mcp): From<VantaError> for McpError + isError con code/retriable (ERR-MCP-01)`; 177/177 crate, workspace clippy -D warnings 0, contrato 6/6)
- **Esfuerzo:** 🟡 1d (appetite max 1d) · **Prioridad:** 🔴 CRITICAL (LLM no podía decidir retry)
- **Dependencias:** Wave 1 `e1fe7ec2` (`VantaError::code()` → 10 `VANTADB_*`)

## Spec (decisiones con evidencia — fuente canónica: tabla §6.2 de `docs/api/ERROR_HANDLING.md`)

### Mapeo implementado (`impl From<VantaError> for McpError`, por `e.code()` — nunca por variantes)

| JSON-RPC | `code()` fuente | Notas |
|---|---|---|
| `-32001` | `VANTADB_BUSY` | retriable por variante vive en `data.retriable` (Busy ✅ / NotInitialized ❌) |
| `-32002` | `VANTADB_CORRUPT` | cubre Restore/Backup además de la fila §6.2 |
| `-32003` | — reservado | core pliega ExecutionConflict/NodeIdCollision/CycleDetected en VALIDATION_ERROR → -32009; documentado en MCP.md |
| `-32004` | `VANTADB_NOT_FOUND` | |
| `-32005` | (auth) | server.rs writer-proxy Bearer: corregido -32001→-32005 (chocaba con busy); sin `data.code` (no es salida de `code()`) |
| `-32006` | (rate limit) | no emitido aún |
| `-32007` | `VANTADB_RESOURCE_LIMIT` | |
| `-32008` | `VANTADB_TIMEOUT` | |
| `-32009` | `VANTADB_VALIDATION_ERROR` + `VANTADB_INVALID_ARGUMENT` | |
| `-32603` | IO_ERROR / WASM_ERROR / CLOSED | códigos sin fila §6.2 → fallback internal (conservador) |

### Envelope (decisión Gate P — DISCOVERY previo, sin bloqueo)
- `McpError` gana `data: Value` (Null → omitido). Factories std 5 intactas. `to_json()` → `{code, message, data{code,retriable,hint?}}` = §6.3 literal.
- Canal JSON-RPC (resources/validación/code.rs): `error.data` estructurado — clientes leen `code`/`retriable`.
- Canal isError (tools/skills/wiki): shape documentado en `skills/vantadb-mcp/SKILL.md` (`isError:true` + `content[0].text` string) INTACTO — el text ahora serializa el mismo objeto error como JSON string. `raise RuntimeError(text)` de los consumidores sigue funcionando; el LLM parsea y branch-ea.
- 6 validadores de `validation.rs` → factory nueva `McpError::validation()` = -32602 + `data.code=VANTADB_VALIDATION_ERROR` (códigos sin cambio, por contrato).

### Sweep de sitios
- tools.rs: 38 sitios `"Xxx Error: {e}"` → `error_content_vanta(e)`; 2 multi-línea contextuales (snapshot_restore append hint de operador; bulk_import_file prefija path) → `McpError::from(e)` + message enriquecido; 5 revertidos por tipo no-VantaError: `RecallError` (1690), String×3 (collection stats 2007, collection delete 2104, export streamed 2392), serde_json::Error×2 (2497, 2507) — fuera del `From`, conservan su string.
- skills.rs: `store_err` único choke point. wiki.rs: `domain_err` único choke point. code.rs: graphrag_search + graph_bfs + fetch_record(get_node). resources.rs: get + list → `McpError::from(e).into_err()`.

## Tests
- **Nuevos (9, `mcp_tests.rs` `err_mcp_01_*`):** -32004 NodeNotFound/NotFound, -32001 Busy retriable=true, -32008 Timeout retriable=true, -32009 dim+conflict, -32007, -32002, fallback -32603 (Generic→WASM_ERROR con data.code), factories std sin data.
- **Adaptados (3):** `skills_tests` ×2 `contains("Skill Error")` → `contains("VANTADB_VALIDATION_ERROR") && contains("-32009")` (prefijo eliminado por contrato); `mcp_tests::bulk_import_stream..._missing_file` → aserta `VANTADB_IO_ERROR` + `cannot import from` (prefijo "Bulk Import File Error" eliminado). Total suite crate: 177/177 (91 mcp_tests = 82 + 9).

## Contrato (verificación mecánica, 6/6 ✅)
1. `rg -rn '"Put Error' vantadb-mcp/src` → 0 (exit 1)
2. `rg -c "impl From<VantaError> for McpError" vantadb-mcp/src/error.rs` → 1
3. `cargo check -p vantadb-mcp --all-targets` → 0 errores
4. `cargo test -p vantadb-mcp --no-fail-fast` → 177 passed, 0 failed
5. `cargo clippy --workspace --all-targets --all-features -- -D warnings` → 0
6. `rg -c 'code' docs/api/MCP.md` → 24→31 (tabla -320xx actualizada a implementación real)

## NOTICED BUT NOT TOUCHING (fuera de scope / otros agentes)
- `threads.rs`/`scenes.rs`/`context.rs` usan `error_content(e.to_string())` con VantaError — mismos candidatos a `error_content_vanta` en un follow-up (no listados en la tarea).
- `-32006` rate-limit nunca emitido; fila reservada en MCP.md.
- `docs/CHANGELOG.md` en working tree con ediciones del agente ERR-TS-01 ( Wave 2) — no se toca para no mezclar stages.

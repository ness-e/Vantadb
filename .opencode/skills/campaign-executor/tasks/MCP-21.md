# Task MCP-21 — GDS vía MCP: graph_page_rank + graph_degree_centrality

**Fuente de verdad:** `docs/Backlog.md` → fase **P25 - Exposición MCP/HTTP** → fila `MCP-21` (leer ANTES de ejecutar: problema, acciones numeradas, archivos exactos).
**Prioridad:** 🔴 · **Esfuerzo:** 🟠

## Impacto mapeado (Regla 0)
- **Archivos leídos completos:** `vantadb-mcp/src/handlers/tools.rs` (1362L pre-edit), `vantadb-mcp/src/validation.rs` (`parse_node_id`, `error_content`, `serialize_content`), `src/sdk/gds.rs` (`graph_page_rank`:15, `graph_degree_centrality`:32 + `src/gds.rs` page_rank con dangling redistribution), `src/sdk/graph.rs` completo, `vantadb-mcp/tests/mcp_tests.rs` (patrones setup_storage/handle_tools_call/MCP-27 fixture IQL), filas MCP-21/22 del backlog.
- **Referencias entrantes:** `handle_tools_list`/`handle_tools_call` (dispatch del server); `test_mcp_tools_list` usa asserts `contains` → aditivo seguro.
- **Referencias salientes:** wrappers SDK ya públicos en `VantaEmbedded` (`graph_page_rank`, `graph_degree_centrality`); `vantadb::graph::TraversalDirection` pub.
- **Veredicto:** cambios aditivos-only; sin alterar API pública del core (wrappers only). ✔

## Fase 1 — DISCOVERY
- [x] Leer la fila `MCP-21` completa en `docs/Backlog.md`.
- [x] `codegraph_explore` de los símbolos/archivos listados en la fila (blast radius).
- [x] Confirmar que ningún cambio requiere alterar API pública del SDK (wrappers only).

## Fase 2 — EJECUCIÓN
- [x] Implementar las acciones (1..n) definidas en la fila, en orden.
- [x] Tool(s) nueva(s) en `vantadb-mcp/src/handlers/tools.rs` + schema JSON-RPC. (`graph_page_rank` + `graph_degree_centrality`; helper `parse_node_ids`; ids u128 como decimal strings)
- [x] Tests round-trip en `vantadb-mcp/tests/mcp_tests.rs` según criterios de la fila. (fixture A→B→C vía query_iql INSERT+RELATE; page_rank suma ≈1.0 y hoja C dominante; centrality (in,out) exactos)

## Fase 3 — VERIFICACIÓN (contrato mecánico)
- [x] `cargo check -p vantadb-mcp`
- [x] `cargo clippy -p vantadb-mcp --all-targets -- -D warnings`
- [x] `cargo test -p vantadb-mcp` (57 passed en mcp_tests, 0 failed en toda la crate)

## Fase 4 — CIERRE
- [x] Actualizar `skills/vantadb-mcp/SKILL.md` + `.opencode/skills/vantadb-mcp/SKILL.md` (hash SAME: `F22171C0…`) + `references/api-reference.md` + `references/mcp-protocol.md` + `docs/api/MCP.md` (conteo 50 tools = 30 core, verificado contra source — learning MCP-18/19).
- [x] Marcar la fila como ✅ en `docs/Backlog.md` con nota breve del cambio.

## RESULTADO
✅ COMPLETO — 2 tools GDS expuestas vía MCP; contrato mecánico pasa. Commit lo hace el lead.

RESULTADO: ✅ COMPLETO
TASK_ID: MCP-21
STEPS_OK: 12/12
PROXIMO_STEP: ninguno
COMMIT_HASH: ninguno (lead comitea)
ARCHIVOS: vantadb-mcp/src/handlers/tools.rs, vantadb-mcp/src/lib.rs, vantadb-mcp/tests/mcp_tests.rs, skills/vantadb-mcp/SKILL.md (+copia .opencode), references/api-reference.md (+copia), references/mcp-protocol.md (+copia), docs/api/MCP.md, docs/Backlog.md
VERIFY_CONTRATO: pasa
BLOQUEO: ninguno

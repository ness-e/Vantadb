# Task MCP-27 — BUG integración: query_iql devuelve 0 filas sobre datos de memory_put

**Fuente de verdad:** `docs/Backlog.md` → fase **P25 - Exposición MCP/HTTP** → fila `MCP-27` (leer ANTES de ejecutar: problema, acciones numeradas, archivos exactos).
**Prioridad:** 🔴 · **Esfuerzo:** 🟠

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `vantadb-mcp/src/handlers/tools.rs` (tools list + dispatch), `src/physical_plan/scan.rs`, `src/executor.rs` (execute_statement/execute_plan), `src/sdk/serialization/mod.rs` (FIELD_* reservados, memory_node_id), `src/backend.rs` (BackendPartition), `vantadb-mcp/tests/mcp_tests.rs` (patrones setup/assert).
- **Referencias entrantes:** `handle_tools_call` ← server.rs/lib.rs; descripción de tool consumida por clientes MCP.
- **Veredicto:** cambio acotado a `vantadb-mcp` (descripción + test). NO se toca core: mapear namespaces→tablas IQL exigiría setear `type=namespace` en cada put de memoria, rompiendo visibilidad de records existentes y fallando para namespaces con caracteres no-identificador (`a/b`) → camino 2 elegido.

## Root cause (verificado con probe test)

Los memory records SÍ viven en la partición Default (node id determinista xxHash3-128), pero sus campos relacionales son reservados (`__vanta_namespace`, `__vanta_key`, ...) y no tienen campo `type`. El scan físico IQL (`PhysicalScan`) filtra `type == <entity>` salvo `*` — y `SELECT * FROM *` ni siquiera parsea. Por eso toda tabla nombrada devuelve `[]` sin error.

## Fase 1 — DISCOVERY
- [x] Leer la fila `MCP-27` completa en `docs/Backlog.md`.
- [x] `codegraph_explore` de los símbolos/archivos listados en la fila (blast radius).
- [x] Confirmar que ningún cambio requiere alterar API pública del SDK (wrappers only).

## Fase 2 — EJECUCIÓN
- [x] Implementar las acciones (1..n) definidas en la fila, en orden. → Camino 2: semántica documentada; sin tool insert_node nueva (query_iql ya soporta `INSERT NODE#id TYPE T {...}`).
- [x] Tool(s) nueva(s) en `vantadb-mcp/src/handlers/tools.rs` + schema JSON-RPC. → N/A: solo descripción actualizada de `query_iql`.
- [x] Tests round-trip en `vantadb-mcp/tests/mcp_tests.rs` según criterios de la fila. → `test_query_iql_memory_records_are_not_tables_but_graph_nodes_round_trip`.

## Fase 3 — VERIFICACIÓN (contrato mecánico)
- [x] `cargo check -p vantadb-mcp`
- [x] `cargo clippy -p vantadb-mcp --all-targets -- -D warnings`
- [x] `cargo test -p vantadb-mcp` (75 tests OK, 0 failed)

## Fase 4 — CIERRE
- [x] Actualizar `skills/vantadb-mcp/SKILL.md` + `.opencode/skills/vantadb-mcp/SKILL.md` (hash SAME ✅ MD5 8519EAF9...).
- [x] Marcar la fila como ✅ en `docs/Backlog.md` con nota breve del cambio.

## RESULTADO
✅ COMPLETO — commit pendiente (lo hace el lead tras verificación).

## Context Save Point
- Sin trabajo parcial: tarea cerrada en una iteración.
- Deuda documentada en la fila del backlog: qué falta para el camino 1 futuro (decisión core + migración + validación de identificadores).

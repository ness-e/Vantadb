# Task MCP-22 — Traversal de grafos: bfs/dfs/filtered/topo/is_dag/accumulators

**Fuente de verdad:** `docs/Backlog.md` → fase **P25 - Exposición MCP/HTTP** → fila `MCP-22` (leer ANTES de ejecutar: problema, acciones numeradas, archivos exactos).
**Prioridad:** 🟠 · **Esfuerzo:** 🟠

## Impacto mapeado (Regla 0)
- Mismo discovery que MCP-21 (mismo dominio, misma sesión): `tools.rs` completo, `src/sdk/graph.rs` completo (`graph_bfs`/`graph_dfs` :50-77, `_filtered` :89-125, topo :130, is_dag :139, accumulators :12-43), `vantadb::graph::TraversalDirection` pub con variantes Forward/Reverse/Both.
- **Veredicto:** aditivo-only, wrappers only. ✔

## Decisión de diseño — accumulators
**FUERA de alcance.** Los accumulators (`GraphAccumulator`, `src/sdk/graph.rs:12-43`) son primitivas de paralelismo in-process para algoritmos internos (PageRank/centrality paralelos): no guardan estado del engine y no sobreviven entre llamadas. Una tool lifecycle vía MCP requeriría estado de sesión server-side (handle registry) para valor cero del agente. Anotado en la fila del backlog y en SKILL.md.

## Fase 1 — DISCOVERY
- [x] Leer la fila `MCP-22` completa en `docs/Backlog.md`.
- [x] `codegraph_explore` de los símbolos/archivos listados en la fila (blast radius).
- [x] Confirmar que ningún cambio requiere alterar API pública del SDK (wrappers only).

## Fase 2 — EJECUCIÓN
- [x] Implementar las acciones (1..n) definidas en la fila, en orden.
- [x] Tool(s) nueva(s) en `vantadb-mcp/src/handlers/tools.rs` + schema JSON-RPC. (`graph_traverse` start/mode/max_depth/direction/filter → rutea a bfs/dfs plain o filtered; `graph_topological_sort`; `graph_is_dag`; helper compartido `parse_direction`)
- [x] Tests round-trip en `vantadb-mcp/tests/mcp_tests.rs` según criterios de la fila. (bfs orden exacto [A,B,C]; dfs cubre cadena; mode inválido → isError MEM-32; topo-sort orden válido; is_dag true → false al cerrar ciclo C→A)

## Fase 3 — VERIFICACIÓN (contrato mecánico)
- [x] `cargo check -p vantadb-mcp`
- [x] `cargo clippy -p vantadb-mcp --all-targets -- -D warnings`
- [x] `cargo test -p vantadb-mcp`

## Fase 4 — CIERRE
- [x] Actualizar `skills/vantadb-mcp/SKILL.md` + `.opencode/skills/vantadb-mcp/SKILL.md` (hash SAME) — incluye la decisión de accumulators documentada.
- [x] Marcar la fila como ✅ en `docs/Backlog.md` con nota breve del cambio (incluye justificación accumulators).

## RESULTADO
✅ COMPLETO — 3 tools de traversal expuestas; accumulators fuera por diseño justificado.

RESULTADO: ✅ COMPLETO
TASK_ID: MCP-22
STEPS_OK: 12/12
PROXIMO_STEP: ninguno
COMMIT_HASH: ninguno (lead comitea)
ARCHIVOS: vantadb-mcp/src/handlers/tools.rs, vantadb-mcp/tests/mcp_tests.rs, skills/vantadb-mcp/SKILL.md (+copia .opencode), references/api-reference.md (+copia), docs/Backlog.md
VERIFY_CONTRATO: pasa
BLOQUEO: ninguno

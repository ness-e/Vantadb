# MEM-32 - MCP tools code_* query-only (8 tools sobre graphrag propio)

Plan: `docs/plans/2026-08-21-vanta-proxy-knowledge.md` Task 1 · Ruta: vanta-worker
Contrato: `cargo check -p vantadb-mcp` pasa; tests D19: 8 tools `code_search/explore/callers/callees/impact/node/status/files` responden sobre un grafo seedeado; respetan dirección de aristas; read-only (sin mutación); tool desconocida → error claro
Stop condition: si impact requiere análisis que graphrag no soporta → stub "not supported" documentado (no inventar semántica). **D28: graphrag PROPIO — NO `@colbymchenry/codegraph`.**

## Impacto mapeado (Regla 0)

**Archivos leídos completos:**
- `vantadb-mcp/src/handlers/tools.rs` (892L) — patrón de wiring: `handle_tools_list` extiende array con `crate::skills::skill_tool_definitions()`; `handle_tools_call` delega `skill_*` → `crate::skills::handle_skill_tool(name, args, storage, config)`; fallback `_ => McpError::method_not_found`
- `vantadb-mcp/src/skills.rs` (1-150) — patrón MEM-07: definiciones JSON + dispatcher único con `(name, args, storage, config)`
- `vantadb-mcp/src/lib.rs` (37L) — declaraciones mod + re-exports
- `vantadb-mcp/Cargo.toml` — deps: vantadb(path), tokio, serde, serde_json, tracing; dev: tempfile. Sin deps nuevas necesarias.
- `vantadb-mcp/tests/mcp_tests.rs` (1-120) — setup: `StorageEngine::open(tempdir)` + `Executor::new(&storage)` + `McpConfig::default()`
- `src/graph.rs` — `GraphTraverser`, `TraversalDirection{Forward,Reverse,Both}` (`follows`: Forward=!reverse, Reverse=reverse)
- `src/sdk/graph.rs` (234L) — SDK público: `graph_bfs/graph_dfs(roots, max_depth, direction)`, `graph_topological_sort`, `graph_is_dag` sobre `VantaEmbedded`
- `src/sdk/builder.rs` — `VantaEmbedded::graphrag_search(namespace, query, query_vector) -> Result<GraphRagResult>` (pipeline propio seed→expand→retrieve→context)
- `src/graphrag/pipeline.rs` (112L) — GraphRagResult{nodes{id,content,score,hop_distance}, edges{source,target,label}, context_text, stats} SIN serde derive → serializar manual
- `src/sdk/api.rs` add_edge (1175-1214) — crea forward edge en source + reverse edge en target ⇒ callers = reverse traversal O(degree)
- `src/sdk/serialization/graph_types.rs` — VantaNodeRecord/VantaEdgeRecord: record NO expone flag reverse ⇒ callers/callees NO pueden derivarse del record; usar `graph_bfs` con dirección

**Referencias entrantes:** `handlers/tools.rs` es el único punto de dispatch MCP; server.rs llama handle_tools_call. Ningún caller rompe (cambios aditivos).

**Referencias salientes (del módulo nuevo):** `vantadb::sdk::{VantaEmbedded}` vía storage Arc<StorageEngine>; `vantadb::graph::TraversalDirection`; helpers `crate::validation::{text_content,error_content,serialize_content,parse_node_id}`, `crate::error::McpError`.

**Veredicto:** impacto BAJO. 1 archivo nuevo (`vantadb-mcp/src/code.rs`) + wiring aditivo en 2 existentes (`handlers/tools.rs` match arm + list extend, `lib.rs` mod decl). Sin cambios de firmas públicas existentes. Blast radius = crate `vantadb-mcp` solamente.

## Mapping tool↔primitiva (pre-mortem 1)

| Tool | Primitiva local | Dirección aristas |
|---|---|---|
| `code_search` | `embedded.graphrag_search(namespace, query, None)` — pipeline propio | n/a (seed→expand) |
| `code_explore` | `get_node(id)` + vecinos separados outgoing/incoming (bfs depth1 Forward/Reverse) | explícita |
| `code_callers` | `graph_bfs([id], 1, Reverse)` − root = edges entrantes (reverse edges que add_edge crea en target) | Reverse |
| `code_callees` | `graph_bfs([id], 1, Forward)` − root = edges salientes | Forward |
| `code_impact` | `graph_bfs([id], max_depth, direction)` default Forward, param `direction` opcional — subgrafo alcanzable dirigido | param |
| `code_node` | `embedded.get_node(id)` → VantaNodeRecord serializado | n/a |
| `code_status` | `embedded.operational_metrics()` serializado + `{node_count}` de memory stats — datos reales del engine, sin semántica inventada | n/a |
| `code_files` | **STUB "not supported"** — graphrag propio no tiene concepto files-por-nodo (D28: no portar semántica de codegraph TDAM); stop condition aplica | n/a |

Errores: node inexistente → error_content("Node not found"); grafo sin datos → resultados vacíos válidos (no error, no panic); tool desconocida → method_not_found (wiring existente).

## Steps

### Step 1 - code.rs: 8 tools + wiring + definiciones ✅ DONE
- `vantadb-mcp/src/code.rs` creado (`code_tool_definitions()` + `handle_code_tool()`); `mod code;` en lib.rs; wiring aditivo en tools.rs (match arm + list extend)
- Verify: `cargo check -p vantadb-mcp` → Finished dev profile exit 0

### Step 2 - Tests D19 (grafo seedeado) ✅ DONE
- `vantadb-mcp/tests/code_tests.rs`: 9 tests sobre diamante seedeado (root→left/right→sink): dirección respetada (callees Forward / callers Reverse / root sin callers), impact param direction + error claro dirección inválida, node/explore/status/files-stub, read-only estructural, unknown tool → method_not_found
- Fixes durante verify: (1) errores de dominio como `Ok(error_content)` no `Err` de protocolo (patrón memory_get "Record not found"); (2) seed requiere `ensure_indexes_current()` antes del BM25 del graphrag (patrón MCP-01/AUD-044); (3) read-only verificado sobre estructura del grafo — `hits`/`last_accessed` suben con CUALQUIER lectura (AccessTracker core, telemetría ≠ mutación)
- Verify: `cargo nextest run -p vantadb-mcp --test code_tests` → 9/9 PASS

### Step 3 - Verify mecánico completo ✅ DONE
- `cargo fmt --check` → exit 0
- `cargo clippy -p vantadb-mcp --all-targets --no-deps -- -D warnings` → exit 0
- `cargo check -p vantadb-mcp` → exit 0
- `cargo nextest run -p vantadb-mcp` → 22/22 PASS (9 code_* nuevos + 13 pre-existentes)

## Recitation final

- **Resultado:** OK — contrato D19 cumplido completo
- **Mapping tool↔primitiva:** ver tabla arriba (7 implementadas + code_files stub documentado)
- **Deuda:** ninguna nueva; sin deps nuevas; blast radius = solo crate vantadb-mcp
- **Próxima tarea:** Task 2 (MEM-28 wiki store core)


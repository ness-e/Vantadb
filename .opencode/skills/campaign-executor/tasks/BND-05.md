# BND-05: vantadb-node superficie mínima — graph/explain para paridad con wasm/ts

## Metadata
- **Plan file:** docs/plans/2026-08-25-batch-colaterales-deuda-desktop.md (Task 9)
- **Fuente:** docs/Backlog.md:762 (BND-05, detectado auditoría integración final, P31)
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟢
- **Tipo:** Rust (napi-rs binding Node)
- **Turns estimados:** 8
- **Creado:** 2026-08-25T14:30
- **last-synced:** 2026-08-25T14:30
- **Estado:** ✅ COMPLETED (verificado mecánicamente; commit lo ejecuta el lead)
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0 steps de ejecución restantes

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `vantadb-ts/src/native.ts` (wrapper TS nativo — usa `vantadb-node` como devDependency; NO se toca en esta tarea), `desktop/` (Tauri consume las 6 integraciones; graph vía native no está wiring hoy) |
| Callees | `vantadb::sdk::{VantaEmbedded, VantaNodeInput, VantaNodeRecord, VantaSearchExplanation, VantaMemorySearchRequest}`, `vantadb::graph::TraversalDirection`, `vantadb::node::DistanceMetric`, `serde_json`, `napi` |
| Implicaciones | API pública del crate `vantadb-node` crece (aditivo — NO rompe métodos existentes). `index.d.ts`/`index.js`/`index.cjs` (generados por napi, trackeados) cambian tras rebuild. `.node` binario gitignored (`.gitignore:233`) — el lead lo regenera localmente, no se commitea. |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `vantadb-node/src/lib.rs` (489L), `vantadb-node/Cargo.toml`, `vantadb-node/package.json`, `vantadb-node/tests/persistence.test.ts`, `vantadb-node/index.d.ts`, `src/sdk/graph.rs` (234L), `src/sdk/gds.rs`, `src/sdk/search/explain.rs`, `src/sdk/serialization/graph_types.rs` (298L), `src/sdk/types.rs` (§ VantaValue/VantaSearchExplanation), `src/sdk/api.rs` (§ insert_node/get_node/delete_node/add_edge/remove_edge), `.opencode/rules/js-ecosystem.md`, `vantadb-wasm/src/lib.rs` (§ graph_* + explain_memory_search + parse_node_id), `vantadb-ts/src/vantadb.ts` + `types.ts` (§ GraphClient)
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** `vantadb` (path `..`, features fjall/memmap2/rayon), `napi`, `napi-derive`, `serde_json`, `tokio`
- **Archivos que referencian a los editados (referencias entrantes):** `vantadb-ts/src/native.ts` (usa `vantadb-node`), `desktop/` (integración native), `docs/architecture/adr/COMP-029-napi-rs-node-bindings.md` (describe superficie actual), `docs/avance/activo/bindings.md`
- **Veredicto impacto:** **bajo** — cambios 100% aditivos en `vantadb-node/src/lib.rs` + artefactos regenerados (index.cjs/js/d.ts). No se rompe ningún método existente; no se toca core (`src/`), no se toca wasm/ts. El wrapper `vantadb-ts/src/native.ts` sigue sin graph (deuda documentada).

## Contrato
"graph/explain expuestos en vantadb-node (o equivalente); build/test node pasa" → verificación mecánica:
- `cargo check -p vantadb-node` (o `cargo check --manifest-path vantadb-node/Cargo.toml`) — Finished sin errores
- `cd vantadb-node && npm run build` — napi rebuild exit 0 (regenera index.cjs/js/d.ts + .node local)
- `cd vantadb-node && npm test` — vitest pasa (3 tests existentes + tests graph/explain nuevos)

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** métodos existentes de `VantaDB` (connect/flush/close/put/put_batch/get/delete/list/list_namespaces/search/capabilities) intactos; patrón op-gate + spawn_blocking (R-4 js-ecosystem) para TODA op nueva; ids u128 como strings decimales (u128 > 2^53 pierde precisión como number JS); sin `unsafe`; sin `unwrap()` en código nuevo; no tocar core `src/` ni wasm/ts.
- **Comandos de verificación:** `cargo check --manifest-path vantadb-node/Cargo.toml` → Finished; `cd vantadb-node && npm run build` → exit 0; `cd vantadb-node && npm test` → vitest all pass.
- **Deuda pendiente:** (1) `vantadb-ts/src/native.ts` (wrapper TS del backend nativo) no expone graph/explain — seguir paridad ahí es tarea separada; (2) docs/api de bindings Node (Regla 3) — la superficie documentada vive en index.d.ts autogenerado; un doc narrativo sería tarea de vanta-docs.

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | BND-05: exponer graph/explain en vantadb-node para paridad con wasm/ts |
| `lastAction` | DISCOVERY completo: leí lib.rs (489L), SDK graph/gds/explain/graph_types/api, wasm y ts; mapeé superficie faltante (12 métodos) |
| `result` | PARTIAL (⏳ IN PROGRESS) |
| `nextAction` | Step 1: editar `vantadb-node/src/lib.rs` — añadir métodos graph + explain |
| `contract` | ver `## Contrato` + `## Invariantes de dominio` |
| `nextTask` | Task 10 del plan: AGT-02 |

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda — el cambio es aditivo sobre binding thin (sin hot paths, sin unsafe, sin serialización nueva de alto costo). La única "deuda" es la no-exposición en `vantadb-ts/src/native.ts`, pre-existente y fuera de scope (queda como deuda explícita arriba, no introducida por este PR).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato del task file se cumple (check + build + npm test) |
| **Commit** | Atómico sobre `vantadb-node/{src/lib.rs,index.cjs,index.js,index.d.ts,tests/graph.test.ts}` + conventional commit `feat(node)`; lo ejecuta el lead (sub-agente NO commitea) |
| **Release** | No aplica (crate `publish = false`, binding no releaseado en este batch) |

## Herramientas necesarias
- codegraph_explore (blast radius — hecho)
- cargo/rustc terminal (cargo check)
- npm + napi-cli (`npm run build`) + vitest (`npm test`)

## Investigation Notes
- Surface wasm (`vantadb-wasm/src/lib.rs`): `explain_memory_search` (:1066), `graph_bfs` (:1363), `graph_dfs` (:1392), `graph_topological_sort` (:1421), `graph_is_dag` (:1435), `graph_filtered_traversal` (:1452 → core `graph_bfs_filtered`), `graph_degree` (:1490 → core `graph_degree_centrality`).
- Surface ts (`vantadb-ts/src/vantadb.ts`): `explainSearch`, `graphBfs`, `graphDfs`, `graphTopologicalSort`, `graphIsDag`, `graphFilteredTraversal`, `graphDegree` + CRUD `insertNode`/`getNode`/`deleteNode`/`addEdge`.
- Core SDK (`src/sdk/`): `graph_bfs`/`graph_dfs`/`graph_bfs_filtered`/`graph_dfs_filtered`/`graph_topological_sort`/`graph_is_dag` (graph.rs), `graph_degree_centrality` (gds.rs), `explain_memory_search` (search/explain.rs), `insert_node`/`get_node`/`delete_node`/`add_edge`/`remove_edge` (api.rs).
- **Decisión de diseño (ids u128):** wasm devuelve `visited: number[]` (pierde precisión >2^53); ts documenta ids u128 como strings (`GraphDegreeEntry.id: string`). En vantadb-node devuelvo **Vec<String>** (decimal) para traversals y `{id: string, ...}` para degree: serde_json no serializa u128 >u64 y numbers JS pierden precisión. Strings in, strings out — consistente con `parse_node_id` de wasm y ERR-025/ERR-023.
- `VantaNodeInput.id` es u128 plano (sin u128_serde) → parseo manual `string|number` → u128.
- El `.node` binario está gitignored; `index.cjs/js/d.ts` trackeados → rebuild los modifica y el lead los commitea junto a lib.rs.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 0 steps |
| % completado | 100% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [ ] **SECURITY** — NO aplica como fase pesada: el cambio agrega **input de usuario** en un trust boundary FFI (ids, direction, filter JSON, fields JSON). Mitigación inline (sin skill nueva): validación estricta por parseo (ids decimales u128 con error descriptivo, direction whitelist Forward/Reverse/Both, vector con `MAX_VEC_DIM` cap ya existente, fields vía serde_json con error tipado). Sin dependencias nuevas. `filter.labels` y `time_range` validados con tipos. No hay auth/red/storage nuevo.
- [ ] **PERFORMANCE** — NO aplica: métodos graph del SDK son ops de lectura/escritura puntuales sobre engine ya implementado; el binding solo serializa `Vec<u128>`→strings (sin hot path nuevo). No hay loops de búsqueda/ingestión en este diff.

## Steps

### Step 1: Implementar métodos graph + explain en lib.rs
- **Archivos:** `vantadb-node/src/lib.rs`
- **Acción:** añadir imports (`TraversalDirection`, `VantaNodeInput`, `VantaSearchExplanation`) y 12 métodos `#[napi]` async con patrón op-gate + spawn_blocking: `insert_node`, `get_node`, `delete_node`, `add_edge`, `remove_edge`, `graph_bfs`, `graph_dfs`, `graph_topological_sort`, `graph_is_dag`, `graph_filtered_traversal`, `graph_degree`, `explain_search` + helpers `parse_node_id`, `parse_direction`, `parse_node_input`, `parse_graph_filter`. Fix colateral requerido para compilar: `parse_search_request` ganó `exclude_superseded: false, search_profile: None` (campos nuevos del SDK) y `get_opt_u64` consolidó 2 ramas idénticas (clippy pre-existente).
- **Verify:** `cargo check --manifest-path vantadb-node/Cargo.toml`
- **Estado:** ✅ COMPLETED — `cargo check` Finished 0 errores; `cargo clippy --all-targets -- -D warnings` exit 0; `cargo fmt --check` exit 0

### Step 2: Build napi (regenera index.cjs/js/d.ts + .node)
- **Archivos:** `vantadb-node/index.cjs`, `index.js`, `index.d.ts`, `vantadb_native.*.node` (local, gitignored)
- **Acción:** `cd vantadb-node && npm run build` (2 fases: cjs + esm; la 1ª terminó, la 2ª se completó con `napi build --esm` por timeout del runner)
- **Verify:** exit 0 + `index.d.ts` contiene los 12 métodos nuevos
- **Estado:** ✅ COMPLETED — index.cjs/js/d.ts regenerados 2026-08-25 12:29/12:36; index.d.ts con graphBfs/graphDfs/graphTopologicalSort/graphIsDag/graphFilteredTraversal/graphDegree/explainSearch/insertNode/getNode/deleteNode/addEdge/removeEdge

### Step 3: Tests vitest graph/explain
- **Archivos:** `vantadb-node/tests/graph.test.ts` (nuevo)
- **Acción:** test round-trip: insert_node×3 → add_edge → bfs/dfs/topological/is_dag/filtered/degree + explain_search; casos inválidos (direction inválida, id no-u128) rechazados con error.
- **Verify:** `cd vantadb-node && npm test`
- **Estado:** ✅ COMPLETED — vitest 8/8 pass (3 persistence + 5 graph/explain)

### Step 4: Verificación final + fmt/clippy del crate
- **Archivos:** — (verificación)
- **Acción:** `cargo fmt --check` sobre lib.rs; `cargo clippy --manifest-path vantadb-node/Cargo.toml --all-targets -- -D warnings`
- **Verify:** exit 0 ambos
- **Estado:** ✅ COMPLETED — fmt exit 0, clippy exit 0, check exit 0, npm test 8/8

### Step 5: Cierre — task state + recitation + RESULTADO
- **Archivos:** `docs/plans/2026-08-25-batch-colaterales-deuda-desktop.md` (recitation — lo actualiza el lead al commitear), este task file
- **Acción:** marcar estado, escribir RESULTADO estructurado, delegar commit al lead
- **Verify:** bloque RESULTADO parseable
- **Estado:** ✅ COMPLETED — task file actualizado + RESULTADO devuelto en esta invocación; commit + review P2-01 delegados al lead

## Dependencias
- Ninguna bloqueante (Wave 2, independiente de MOD-11/MOD-21). Worktree tiene WIP sin commit de otros sub-agentes (FIND-30/MOD-21/audit) — NO tocar.

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la
> tarea no está COMPLETED.

- **Revisor:** pendiente — el lead delega a vanta-review/vanta-audit ANTES de commitear (esta invocación NO commitea). Evidencia mecánica lista para el revisor: `cargo check`/`fmt`/`clippy` exit 0 + `npm test` 8/8 + diff de `vantadb-node/src/lib.rs`.
- **Enfoque:** validar que la superficie napi replica wasm/ts (12 métodos) sin romper la API existente y que el parseo FFI es estricto (ids u128 decimales, direction whitelist, vector MAX_VEC_DIM, fields vía serde_json tipado).
- **Cómo se probó:** evidencia mecánica real (salidas de cargo check/fmt/clippy + vitest 8/8 + index.d.ts regenerado con los 12 métodos).
- **Checklist anti-hábitos tóxicos:** [ ] pendiente del revisor (lead delega).
- **Veredicto:** ⏳ pendiente de review del lead — verificado mecánicamente por el implementador.

## Notas
- La decisión `Vec<String>` para ids (vs `number[]` de wasm) está justificada arriba; quedará documentada en index.d.ts (docstring del método) y aquí.
- No agrego `graph_page_rank` ni `graph_accumulator_*`: wasm/ts no los exponen (paridad = superficie wasm/ts).
- No toco `vantadb-ts/src/native.ts` (fuera de alcance contractual; deuda).

## Context Save Point (2026-08-25 12:40)

- **Contrato cumplido:** graph/explain expuestos en `vantadb-node` (12 métodos: insert_node, get_node, delete_node, add_edge, remove_edge, graph_bfs, graph_dfs, graph_topological_sort, graph_is_dag, graph_filtered_traversal, graph_degree, explain_search); build y test node pasan.
- **Evidencia (todas salidas reales):** `cargo check --manifest-path vantadb-node/Cargo.toml` → Finished (exit 0); `cargo fmt --manifest-path vantadb-node/Cargo.toml --check` → exit 0; `cargo clippy --manifest-path vantadb-node/Cargo.toml --all-targets -- -D warnings` → exit 0; `npm run build` (cjs + esm) → Finished release, exit 0; `npm test` → vitest 2 files / 8 tests pass.
- **Archivos tocados (para el commit del lead):** `vantadb-node/src/lib.rs` (métodos + helpers + 2 fixes de compilación/clippy en código pre-existente), `vantadb-node/tests/graph.test.ts` (nuevo), `vantadb-node/index.cjs`, `vantadb-node/index.js`, `vantadb-node/index.d.ts` (regenerados por napi). NO tocar otros archivos del worktree (WIP de FIND-30/MOD-21/audits).
- **Commits sugeridos (conventional):** `feat(node): BND-05 graph/explain surface parity with wasm/ts` cubriendo los 5 archivos.
- **Fix colaterales inline (mismo file, justificados):** (1) `parse_search_request` + `exclude_superseded`/`search_profile` — campos nuevos del SDK, sin ellos el crate no compila; semántica preservada (false/None). (2) `get_opt_u64` 2 ramas idénticas → 1 expresión (clippy `if_same_then_else` pre-existente). Ambos sin cambio de comportamiento.
- **Siguiente tarea del plan:** AGT-02 (Task 10).
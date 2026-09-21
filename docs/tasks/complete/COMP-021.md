# COMP-021: Temporal edges (timestamp-aware relationships)

## Metadata
- **Plan file:** — (backlog directo)
- **Fuente:** docs/Backlog.md:280
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟠
- **Tipo:** Rust core + Bindings (Mixto)
- **Turns estimados:** 15-30
- **Creado:** 2026-08-02
- **Estado:** ✅ COMPLETADO
- **Routing:** vanta-worker

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `src/sdk/api.rs` (`add_edge`, `remove_edge`), `src/executor.rs`, `vantadb-python/src/lib.rs:1300` (`add_edge`), `vantadb-wasm/src/lib.rs:897` (`add_edge`), `vantadb_py/__init__.py:225` (AsyncVantaDB), `src/graph.rs` (GraphTraverser: bfs/dfs, filtered variants), `src/gds.rs` (GraphDataScience usa GraphTraverser) |
| Callees | `src/node.rs` (`Edge` struct, `UnifiedNode::edges: Vec<Edge>`, `label_index`), `crate::node::Edge` en tests (`tests/graphrag_test.rs`, `examples/rust/graphrag.rs`) |
| Implicaciones | Campo NUEVO en struct serializado (`Edge` va dentro de `UnifiedNode.edges`, persistido vía bincode en fjall/rocksdb). **Backward-compat obligatoria:** `#[serde(default)]` (mismo patrón ya usado en `reverse: bool` y `label_index`) — datasets existentes deserializan con `created_at_ms = 0`. No rompe contratos públicos existentes si el nuevo param es `Option` con default. `add_edge` en Python tiene `#[pyo3(signature = (source_id, target_id, label, weight=None))]` — agregar `created_at_ms=None` es aditivo. WASM `add_edge(source_id: u64, target_id: u64, label, weight)` → agregar param opcional en TS wrapper. No requiere migración de datos ni bump de schema (serde default). Afecta `memory_size()` (Edge crece 8B). |

## Contrato
"`cargo nextest run --profile audit -p vantadb --build-jobs 2` pasa, incluyendo: (1) test de backward-compat que deserializa un `UnifiedNode` serializado con Edge sin `created_at_ms` → `created_at_ms == 0`; (2) test de traversal temporal: edges con timestamp dentro de la ventana `[from_ms, to_ms]` se siguen, fuera de la ventana NO se siguen; (3) `add_edge` con `created_at_ms: Option<u64>` explícito persiste el valor dado y `None` setea `now()`. El campo `created_at_ms` es legible en el struct público `Edge`."

## Herramientas necesarias
- cargo-mcp (check, nextest, fmt, clippy)
- codegraph_explore (blast radius) — ya ejecutado
- rust-analyzer-mcp (diagnostics)

## Investigation Notes
- `Edge` (src/node.rs:648-659): `#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)] pub struct Edge { target: u128, label_id: u32, weight: f32, #[serde(default)] reverse: bool }`.
- Constructores existentes: `Edge::new` (weight 1.0), `Edge::with_weight`, `Edge::reverse` — los 3 deben setear `created_at_ms` (default `now()`).
- `UnifiedNode::add_edge`/`add_weighted_edge` (src/node.rs:1081-1092): push a `edges` + `label_index.entry(label_id).or_default().push(target)` + flag HAS_EDGES. `rebuild_label_index` itera `edge.label_id`/`edge.target` — no se rompe con campo nuevo.
- SDK `VantaEmbedded::add_edge` (src/sdk/api.rs:956-988): crea Edge forward (`reverse: false`) en source + Edge reverse (`reverse: true`) en target, ambos con el mismo `weight`. El timestamp debe propagarse a AMBOS (forward y reverse comparten creación lógica).
- Traversal: `GraphTraverser::bfs_traverse_filtered` (src/graph.rs:103) usa `direction.follows(edge)` + `label_index`. Para filtro temporal: agregar rango `(from_ms: Option<u64>, to_ms: Option<u64>)` a `bfs_traverse_filtered`/`dfs_traverse_filtered` (o variante nueva) que filtre `edge.created_at_ms` antes de seguir. `discover_edges_filtered` (src/graph.rs:378) es el punto de cache — filtrar ahí.
- Python binding `add_edge` (vantadb-python/src/lib.rs:1300): `#[pyo3(signature = (source_id, target_id, label, weight=None))]` → agregar `created_at_ms=None` (aditivo, no rompe). `vantadb_py/__init__.py:225` AsyncVantaDB wrapper.
- WASM `add_edge` (vantadb-wasm/src/lib.rs:897) + TS wrapper `vantadb-ts/src/vantadb.ts` (~775 `addEdge`) — agregar param opcional.
- ⚠️ **Disco C: casi lleno (~45 MB libres; `target/` = 101.84 GB).** `cargo check -p vantadb` es el mínimo footprint. Si falla por falta de espacio, correr `cargo clean` primero (libera ~100 GB). NO correr pytest de integraciones Python sin liberar disco.

## Steps

### Step 1: Agregar `created_at_ms` al struct `Edge` + constructores
- **Archivos:** `src/node.rs`
- **Acción:** Agregar `pub created_at_ms: u64` con `#[serde(default)]` al struct `Edge` (línea ~648). Actualizar `Edge::new`, `Edge::with_weight`, `Edge::reverse` para setear `created_at_ms: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64` (mismo patrón que `UnifiedNode::new`, src/node.rs:1062). Agregar helper `Edge::with_timestamp(target, label_id, created_at_ms)` o param en los constructores existentes (elegir la firma más coherente con el codebase).
- **Verify:** `cargo check -p vantadb`
- **Estado:** ✅ COMPLETADO

### Step 2: Propagar timestamp en SDK `add_edge` (forward + reverse)
- **Archivos:** `src/sdk/api.rs`
- **Acción:** `add_edge(source_id, target_id, label, weight: Option<f32>, created_at_ms: Option<u64>)` — si `None`, usar `now()`. Setear el mismo `created_at_ms` en el Edge forward (línea ~970) y el Edge reverse (línea ~981). `remove_edge` no cambia (no toca el struct completo).
- **Verify:** `cargo check -p vantadb`
- **Estado:** ✅ COMPLETADO

### Step 3: Filtro temporal en GraphTraverser
- **Archivos:** `src/graph.rs`
- **Acción:** Agregar rango temporal a `bfs_traverse_filtered`/`dfs_traverse_filtered` (o variante `*_temporal` si los callers existentes no deben cambiar): `time_range: Option<(u64, u64)>` (from_ms, to_ms). Filtrar `edge.created_at_ms >= from && edge.created_at_ms <= to` (bound inclusive) al seguir edges, en ambos paths: `label_index` (encontrar Edge para verificar) y fallback scan. Aplicar en `discover_edges_filtered` para DFS. Callers existentes pasan `None` → comportamiento idéntico.
- **Verify:** `cargo check -p vantadb`
- **Estado:** ✅ COMPLETADO

### Step 4: Tests Rust (backward-compat + temporal)
- **Archivos:** `src/node.rs` (mod tests), `tests/graphrag_test.rs` o nuevo `tests/core/temporal_edges.rs`
- **Acción:** (1) Serializar `UnifiedNode` con Edge viejo-shape (sin created_at_ms) y deserializar → `created_at_ms == 0` (bincode backward-compat). (2) `add_edge` con timestamp explícito → persiste en ambos nodos (forward + reverse). (3) Traversal con ventana temporal: edge dentro → seguido; fuera → no seguido; `None` → todo. (4) `add_edge` sin timestamp → `created_at_ms > 0`.
- **Verify:** `cargo nextest run --profile audit -p vantadb --build-jobs 2`
- **Estado:** ✅ COMPLETADO

### Step 5: Bindings Python (add_edge + filtro temporal)
- **Archivos:** `vantadb-python/src/lib.rs`, `vantadb_py/__init__.py`
- **Acción:** `add_edge(..., created_at_ms=None)` en lib.rs:1300 (aditivo). AsyncVantaDB wrapper `vantadb_py/__init__.py:225` + método de traversal filtrado por tiempo si existe equivalente (`graph_bfs_filtered`/`graph_dfs_filtered` ya existen — agregar param de rango temporal si el core lo expone).
- **Verify:** `cargo check -p vantadb-python` (si disco permite) o al menos `cargo check -p vantadb`
- **Estado:** ✅ COMPLETADO

### Step 6: Bindings WASM + TS
- **Archivos:** `vantadb-wasm/src/lib.rs`, `vantadb-ts/src/vantadb.ts`
- **Acción:** `add_edge` WASM (lib.rs:897) acepta `created_at_ms: Option<u64>` (wasm_bindgen: `Option<u64>`). TS wrapper `addEdge` agrega param opcional `created_at_ms?: number`.
- **Verify:** `cargo check -p vantadb-wasm` (si disco permite) + `cd vantadb-ts && npx tsc --noEmit` (si hay node)
- **Estado:** ✅ COMPLETADO

## Dependencias
- Ninguna. (COMP-021 no depende de DRV-121/122 — eso es COMP-028.)

## Notas
- NO tocar WAL/schema de almacenamiento — `#[serde(default)]` cubre backward-compat sin migración.
- `memory_size()` (src/node.rs:1139) suma `size_of::<Edge>()` — Edge crece de 20B a 28B (u64 align), el cálculo se ajusta solo.
- El timestamp ES Unix epoch en milisegundos (u64), consistente con `last_accessed`, `created_at_ms` de `VantaMemoryRecord`, y `BinaryHeader.timestamp`.
- NO modificar `GraphRagEdge` (src/graphrag/pipeline.rs:19) — es otra estructura para el pipeline GraphRAG, fuera de alcance.
- Regla 3 (docs sync): si se expone param nuevo en API pública, actualizar `docs/api/PYTHON_SDK.md` en el mismo PR.
- ⚠️ Disco: si `cargo check` falla por ENOSPC → `cargo clean` primero.

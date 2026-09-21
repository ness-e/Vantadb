# COMP-022: Graph Data Science library (PageRank, centrality)

**Estado:** ✅ COMPLETED
**Esfuerzo:** 🟡 2-3 sem
**Dependencias:** COMP-017 ✅ (GraphAccumulator)
**Bloquea:** Nothing
**Resultado:** `GraphDataScience` en `src/gds.rs` con `page_rank()` + `degree_centrality()`, SDK `graph_page_rank`/`graph_degree_centrality` + Python bindings. 7 tests.

---

## Objetivo

Implementar algoritmos clásicos de Graph Data Science sobre `GraphAccumulator` + `GraphTraverser`:
1. **PageRank** — versión paralelizada clásica
2. **Degree Centrality** — in/out degree como métrica base
3. Opcional (si sobra espacio): **Betweenness Centrality** aproximada (muestreo)

## Contexto

- `src/accumulator.rs` — `GraphAccumulator` con `add()` lock-free (CAS loop sobre `AtomicU64`/`f64`), `get()`, `snapshot()`, `clear()`
- `src/graph.rs` — `GraphTraverser::traverse_with_accumulator(roots, max_depth, acc, apply_fn)` — BFS + llama `apply_fn(node_id, edges, acc)` por nodo y auto-acumula
- `src/sdk/graph.rs` — `graph_create_accumulator()`, `graph_accumulator_add/get/snapshot`
- `src/node.rs` — `Edge { target: u128, label_id: u32, weight: f32 }`
- `dashmap` + `rayon` (feature opcional) ya disponibles
- No hay nada de GDS aún

## Diseño

### 1. `GraphDataScience` struct (nuevo `src/gds.rs`)

```rust
pub struct GraphDataScience<'a> {
    storage: &'a StorageEngine,
}
```

Métodos:

#### PageRank
```rust
pub fn page_rank(
    &self,
    roots: &[u128],
    max_iterations: usize,
    damping: f64,
    tolerance: f64,
) -> Result<HashMap<u128, f64>>
```

Algoritmo:
1. Descubrir todos los nodos alcanzables desde `roots` (usar `discover_edges`)
2. Inicializar `rank = 1.0/N` por nodo en un `GraphAccumulator`
3. Para cada iteración:
   a. Por cada nodo, distribuir `rank / out_degree` a sus vecinos
   b. Aplicar damping: `new_rank = (1-d)/N + d * sum(contributions)`
   c. Calcular delta total `sum(|new_rank - old_rank|)`
   d. Si delta < tolerance, convergió
4. Devolver snapshot

Ponytail: usar `traverse_with_accumulator` para la fase de distribución si es posible. Si no, bucle manual sobre nodos descubiertos.

#### Degree Centrality
```rust
pub fn degree_centrality(&self, roots: &[u128]) -> Result<HashMap<u128, (usize, usize)>>
```
Devuelve `(in_degree, out_degree)` por nodo. Simple: contar edges entrantes/salientes.

#### (Opcional) Betweenness Centrality aproximada
```rust
pub fn betweenness_centrality(&self, roots: &[u128], sample_size: usize) -> Result<HashMap<u128, f64>>
```
Brandes algorithm + muestreo de nodos fuente para grafos grandes.

### 2. SDK API (`src/sdk/gds.rs` — nuevo)

```rust
impl VantaEmbedded {
    pub fn graph_page_rank(&self, roots: &[u128], max_iter: usize, damping: f64, tolerance: f64) -> Result<HashMap<u128, f64>>
    pub fn graph_degree_centrality(&self, roots: &[u128]) -> Result<HashMap<u128, (usize, usize)>>
    pub fn graph_betweenness_centrality(&self, roots: &[u128], sample: usize) -> Result<HashMap<u128, f64>>
}
```

### 3. Python bindings (`vantadb-python/src/lib.rs`)

Agregar métodos con `#[pymethods]`:
```rust
fn graph_page_rank(&self, py: Python, roots: Vec<u128>, max_iterations: usize, damping: f64, tolerance: f64) -> PyResult<HashMap<u128, f64>>
fn graph_degree_centrality(&self, py: Python, roots: Vec<u128>) -> PyResult<HashMap<u128, (usize, usize)>>
```

### 4. WASM bindings (`vantadb-wasm/src/lib.rs`)

Agregar métodos expuestos via `#[wasm_bindgen]`:
```rust
pub fn graph_page_rank(&self, roots: Vec<u64>, max_iterations: usize, damping: f64, tolerance: f64) -> Result<JsValue, JsValue>
```

### 5. Exposición en `src/lib.rs`

Agregar `pub mod gds;`

### 6. Tests

- `test_page_rank_simple` — 3-node chain (0→1→2), PageRank converge, ranks > 0 y suman ≈ 1
- `test_page_rank_diamond` — 4-node diamond (0→{1,2}→3), verificar simetría
- `test_page_rank_damping` — damping=1.0 (sin teleport) vs damping=0.85
- `test_page_rank_convergence` — tolerance alta → converge rápido, tolerance baja → más iteraciones
- `test_degree_centrality_basic` — verificar in/out degree counts
- `test_degree_centrality_disconnected` — múltiples roots, verificar grados

## Archivos a modificar/crear

| Archivo | Acción |
|---------|--------|
| `src/gds.rs` | **CREAR** — `GraphDataScience` con PageRank, degree centrality |
| `src/sdk/gds.rs` | **CREAR** — SDK wrapper para GDS |
| `src/lib.rs` | **MODIFICAR** — `pub mod gds;` |
| `vantadb-python/src/lib.rs` | **MODIFICAR** — Python bindings |
| `vantadb-wasm/src/lib.rs` | **MODIFICAR** — WASM bindings (opcional en esta fase) |
| `docs/Backlog.md` | **MODIFICAR** — marcar COMP-022 completada |

## Criterio de aceptación

- `cargo check -p vantadb` pasa
- `cargo test -p vantadb gds` pasa (todos los tests de GDS)
- `cargo test -p vantadb --test graph` pasa (integración no rota)
- `cargo clippy -p vantadb -- -D warnings` pasa
- PageRank converge en < 100 iteraciones para grafos < 1000 nodos
- Degree centrality devuelve resultados correctos
- API SDK expuesta y testeable

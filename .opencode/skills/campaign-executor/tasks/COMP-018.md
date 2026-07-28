# COMP-018: Double-linked Relationship Chains

**Prioridad:** 🟡 Media-Alta | **Esfuerzo:** ~1-2 sem | **Fuente:** `docs/Backlog.md` Phase 10

## Objetivo

Implementar relaciones bidireccionales (doble enlace) en el graph engine. Actualmente las aristas son single-directed: Node A → Node B. Esto significa que para navegar en reversa ("quién apunta a mí") hay que escanear todos los nodos, que es O(n).

## Estado Actual

- `src/node.rs:478` — `Edge { target: u128, label_id: u32, weight: f32 }`
- `src/node.rs:895` — `UnifiedNode::add_edge()` — solo agrega a `self.edges`
- `src/edge_index.rs` — `EdgeIndex { edges: DashSet<(u128, u128)> }` — forward index
- `src/sdk/api.rs:763` — `add_edge(source, target, label, weight)` — inserta en nodo source solamente
- `src/graph.rs` — `GraphTraverser` — BFS/DFS solo sigue `node.edges` (forward)
- `src/storage/engine/ops.rs` — `remove_edge` / `delete` — solo remueve forward
- `COMP-006` (Edge Label Interning) — ✅ ya completado

## Implementation

### 1. `Edge` struct — agregar dirección

En `src/node.rs`, agregar campo `reverse: bool` a `Edge`:

```rust
pub struct Edge {
    pub target: u128,
    pub label_id: u32,
    pub weight: f32,
    pub reverse: bool,  // NEW: true = this is the reverse half of a forward edge
}
```

El flag `reverse` permite filtrar reverse edges en traversals que solo quieran forward.

### 2. `add_edge` — auto-crear reverse edge

En `src/node.rs`, modificar `add_edge()` y `add_weighted_edge()` para solo agregar forward (no cambiar aún).
En `src/sdk/api.rs:763` `add_edge()`, después de pushear el edge forward al source node, también obtener y modificar el target node para agregar el reverse edge:

```rust
// Después de insertar forward edge en source node:
let mut target_node = engine.get(target_id)?
    .ok_or(VantaError::NodeNotFound(target_id))?;
target_node.edges.push(crate::node::Edge {
    target: source_id,
    label_id,
    weight: weight.unwrap_or(1.0),
    reverse: true,
});
engine.insert(&target_node)?;
```

Considerar Edge Label Interning ya existe (COMP-006 ✅). Reusar `engine.intern_label()`.

### 3. `remove_edge` — también remover reverse

En el SDK, agregar método `remove_edge(source, target, label)` que:
1. Obtiene source node, remueve el edge forward
2. Obtiene target node, remueve el edge reverse
3. Actualiza EdgeIndex
4. Re-inserta ambos nodos

### 4. `EdgeIndex` — indexar reverse edges

En `src/edge_index.rs`, `insert()` debe también indexar `(to, from)` o agregar un `reverse_edges: DashSet<(u128, u128)>`.
`remove_all_for_node()` ya limpia ambos lados (chequea `*f != node_id && *t != node_id`).

### 5. `GraphTraverser` — soportar dirección reverse

En `src/graph.rs`, agregar `Direction` enum:

```rust
pub enum TraversalDirection {
    Forward,   // follow source → target edges
    Reverse,   // follow target → source edges (reverse edges)
    Both,      // follow both directions
}
```

Agregar parámetro `direction: TraversalDirection` a los métodos de `GraphTraverser`:
- `bfs_traverse()` / `dfs_traverse()` — parámetro nuevo
- `bfs_traverse_filtered()` / `dfs_traverse_filtered()` — parámetro nuevo

Durante la traversa, si `direction == Reverse`, seguir edges con `reverse: true`.
Si `direction == Both`, seguir todos los edges.

### 6. SDK methods — exponer dirección

En `src/sdk/graph.rs`, actualizar `graph_bfs`, `graph_dfs`, `graph_bfs_filtered`, `graph_dfs_filtered` para aceptar un parámetro de dirección opcional.

### 7. WASM + Python bindings

Actualizar:
- `vantadb-wasm/src/lib.rs` — métodos graph_bfs/dfs con parámetro direction
- `vantadb-python/src/lib.rs` — métodos graph_bfs/dfs con parámetro direction

El default debe ser `Forward` (backward compatible).

### 8. Tests

Agregar tests para:
- Crear edge A→B, verificar que B tiene reverse edge apuntando a A
- BFS/DFS en dirección reverse desde B encuentra A
- BFS/DFS en dirección Both desde B encuentra ambos
- Remove edge A→B también remueve reverse B→A
- Cascade delete de nodo A remueve reverse edges en todos los targets
- Backward compatibility: tests existentes sin direction parameter deben seguir funcionando

## Archivos a modificar

1. `src/node.rs` — Edge struct + UnifiedNode add_edge
2. `src/edge_index.rs` — posible reverse index
3. `src/sdk/api.rs` — add_edge + remove_edge bidireccional
4. `src/graph.rs` — Direction enum + direction param en BFS/DFS
5. `src/sdk/graph.rs` — direction param en SDK methods
6. `src/storage/engine/ops.rs` — remove_edge con reverse cleanup
7. `vantadb-wasm/src/lib.rs` — WASM bindings
8. `vantadb-python/src/lib.rs` — Python bindings
9. Tests existentes de graph traversal

## Criterios de Aceptación

- [ ] `cargo check -p vantadb` pasa
- [ ] `cargo test -p vantadb` pasa (tests existentes + nuevos)
- [ ] `cargo test -p vantadb_py` pasa (si hay cambios en bindings)
- [ ] Forward traversal (sin direction param) funciona igual que antes
- [ ] Reverse traversal descubre nodos que apuntan al root
- [ ] Both traversal descubre ambos lados
- [ ] Delete cascade limpia reverse edges

## Referencias

- `src/node.rs` — Edge struct (línea 478), add_edge (línea 895)
- `src/edge_index.rs` — EdgeIndex (7 líneas)
- `src/sdk/api.rs:763` — SDK add_edge
- `src/graph.rs` — GraphTraverser BFS/DFS
- `src/sdk/graph.rs` — SDK graph_bfs/graph_dfs
- `COMP-006` — Edge Label Interning (✅ ya completado, reusa `LabelIntern` / `intern_label`)

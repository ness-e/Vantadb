# Plan de Implementación: Propuesta 4 — Lock-Free Neighbor Lists con Flatten

## Overview
Extraer `neighbors: Vec<NeighborVec>` de `HnswNode` a una estructura separada `HnswNeighborIndex` con per-neighbor-list `RwLock`. Esto elimina la contención de DashMap shard locks durante `connect_layer_neighbors()` y `shrink_neighbors()`, permitiendo ~1.5-2× más throughput en rebuild paralelo.

## Arquitectura de la Nueva API

```rust
// src/index/neighbor_index.rs
pub(crate) struct HnswNeighborIndex {
    lists: parking_lot::Mutex<Vec<parking_lot::RwLock<NeighborVec>>>,
    id_to_meta: DashMap<u128, (usize, usize)>,
}
```

- `lists`: Vec plano de RwLock<NeighborVec>, cada entrada = (node_id, layer) pair
- `id_to_meta`: mapea node_id → (start_index, num_layers) en lists[]
- HnswNode pierde el campo `neighbors`
- `CPIndex` gana `neighbor_index: HnswNeighborIndex`

### API methods
- `allocate(id, num_layers)` — reserva espacio contiguo, inserts id_to_meta
- `get_neighbors(id, layer) -> Option<NeighborVec>` — read lock + clone
- `add_neighbor(id, layer, neighbor) -> bool` — write lock + push (si no duplicado)
- `set_neighbors(id, layer, neighbors)` — write lock + replace
- `num_layers(id) -> Option<usize>`
- `remove_node(id)` — borra entrada de id_to_meta
- `replace_node(id, new_id, new_num_layers)` — para reindex (swap id manteniendo data)
- `collect_all() -> Vec<(u128, Vec<NeighborVec>)>` — para serialización

## Tareas

### Task 4a [vanta-worker]: Crear HnswNeighborIndex + modificar HnswNode/CPIndex
**Archivos:** `src/index/neighbor_index.rs` (nuevo), `src/index/graph.rs`, `src/index/mod.rs`
- Crear struct + impl en nuevo archivo
- Quitar `neighbors` de HnswNode
- Agregar `neighbor_index` a CPIndex + init
- Actualizar `insert_hnsw_with_level()` para usar `allocate()`
- Actualizar `estimate_memory_bytes()` 
- Agregar `pub mod neighbor_index` a mod.rs + re-export

### Task 4b [vanta-worker]: Migrar connect_layer_neighbors + shrink_neighbors + validate_index + reindex_nodes
**Archivos:** `src/index/graph.rs`
- `connect_layer_neighbors()` → usa `neighbor_index.add_neighbor()` + `set_neighbors()` en shrink
- `shrink_neighbors()` → lee vectores de nodes, escribe vía `neighbor_index.set_neighbors()`
- `serialization_order()` → usa `neighbor_index.num_layers()` + `get_neighbors()`
- `repair_orphan_links()` → usa `neighbor_index` para leer/escribir
- `validate_index()` (stats.rs) → usa `neighbor_index`

### Task 4c [vanta-worker]: Migrar search_layer()
**Archivos:** `src/index/search.rs`
- Reemplazar `node.neighbors[layer]` con `self.neighbor_index.get_neighbors(id, layer)`
- ACORN expansion igual

### Task 4d [vanta-worker]: Migrar serialize.rs
**Archivos:** `src/index/serialize.rs`
- Serialize: usa `neighbor_index.collect_all()` en vez de `node.neighbors`
- Deserialize: popula `neighbor_index` en vez de `node.neighbors`
- Actualizar tests helpers (quitar `neighbors` de HnswNode construction)

### Task 4e [vanta-worker]: Migrar archivos menores (stats, core, cache_warmer, archive)
**Archivos:** `src/index/stats.rs`, `src/index/core.rs`, `src/cache_warmer.rs`, `src/storage/archive.rs`
- Cada `.neighbors` access → `neighbor_index.get_neighbors()` / `neighbor_index.num_layers()`

### Task 4f [vanta-worker]: Fix test helpers en serialize.rs, ivf.rs, search.rs, graph.rs
**Archivos:** `src/index/serialize.rs` (tests), `src/index/ivf.rs` (tests), `src/index/search.rs` (tests), `src/index/graph.rs` (tests)
- Todas las construcciones de `HnswNode { ..., neighbors: ..., }` pierden el campo neighbors
- Tests que usan `node.neighbors` directamente → usan `neighbor_index` API
- ✅ COMPLETADO — ivf.rs (3 construcciones), graph.rs (3 tests con node.neighbors[0].push), serialize.rs/search.rs (fixtures sin neighbors) arreglados manualmente

### Task 4g [vanta-audit]: Code review ✅ COMPLETADO
- ✅ Revisar race conditions en locking de neighbor_index → 2 HIGH findings (lock scoping), fix aplicado
- ✅ Verificar que no hay deadlocks → 0 deadlocks garantizados, lock ordering consistente
- ✅ Confirmar compatibilidad de serialización → 100% backward compatible con v7
- **Veredicto:** APROBADO con hallazgos. 0 unsafe, 0 UB, formato binario sin cambios.

### Task 4h [vanta-chaos]: Verification ✅ COMPLETADO
- ✅ `cargo check -p vantadb` — 0 errors
- ✅ `cargo test -p vantadb --lib -- index` — 101 tests, 0 failures
- ✅ Benchmark rebuild time vs baseline — **2.47× improvement** (1.88s → 760ms para 2000 vectores)

## Dependencias (Todas ✅ Completadas)
```
Task 4a ──► Task 4b, 4c, 4d, 4e (paralelo) ──► Task 4f ──► Task 4g ──► Task 4h
  ✅           ✅  ✅  ✅  ✅                     ✅          ✅          ✅
```

## Formato de Serialización
Se mantiene el formato actual (VECTOR_INDEX_VERSION = 8). En serialize, se itera collect_all() y se escribe inline como hoy. En deserialize, se lee inline y se popula neighbor_index. Sin cambios de formato — solo cambia dónde se almacena en memoria.

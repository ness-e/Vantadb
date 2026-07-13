# Fragmentar `index/core.rs` Implementation Plan

> **Finding 2.2 del FULL_CODEBASE_AUDIT:** Crear `src/index/graph.rs`, `search.rs`, `serialize.rs` (`validate.rs` no aplica — `validate_node` queda en graph.rs, `validate_index` ya está en stats.rs)

**Goal:** Dividir `src/index/core.rs` (2187 líneas) en 3 submódulos + tests separados.

**Architecture:** `core.rs` se convierte en un archivo puro de tests; cada área funcional (graph, search, serialize) en su propio archivo. Todos exportados via `mod.rs`.

**Tech Stack:** Rust, DashMap, serde, parking_lot, memmap2

---

### Task 1: Crear `src/index/graph.rs`

**Files:**
- Create: `src/index/graph.rs`
- Modify: `src/index/core.rs` (remover contenido extraído)

**Content:** Extraer de `core.rs`:
- Líneas 1-32: imports, NeighborVec, constantes (ENTRY_POINT_NONE, MAX_VEC_F32_LEN, FLAG_TOMBSTONE)
- Líneas 34-128: prefetch_mmap_vector, release_mmap_vector
- Líneas 130-160: set_prefetch_mode, should_prefetch, PREFETCH_MODE
- Líneas 162-190: HnswNode struct
- Líneas 192-263: IndexBackend enum + impl
- Líneas 265-294: HnswConfig struct + impl
- Líneas 296-344: NodeSim, NodeSimMin wrappers
- Líneas 346-433: CPIndex struct + constructors (new, new_with_config, with_backend, estimate_memory_bytes, random_layer)
- Líneas 435-461: Entry point accessors (get_entry_point, find_new_entry_point, set_entry_point, update_metadata)
- Líneas 463-508: fast_similarity
- Líneas 873-906: validate_node
- Líneas 908-922: add
- Líneas 924-942: compute_cached_norms
- Líneas 944-1079: insert_hnsw
- Líneas 1081-1134: shrink_neighbors
- Líneas 1222-1258: serialization_order
- Líneas 1886-1891: Default impl

### Task 2: Crear `src/index/search.rs`

**Files:**
- Create: `src/index/search.rs`
- Modify: `src/index/core.rs` (remover contenido extraído)

**Content:** Extraer de `core.rs`:
- Líneas 510-750: search_layer
- Líneas 752-871: select_neighbors
- Líneas 1136-1220: search_nearest

### Task 3: Crear `src/index/serialize.rs`

**Files:**
- Create: `src/index/serialize.rs`
- Modify: `src/index/core.rs` (remover contenido extraído)

**Content:** Extraer de `core.rs`:
- Líneas 1260-1436: serialize_to_bytes, serialize_to_writer
- Líneas 1438-1740: deserialize_from_bytes + inline helpers (take_bytes, read_le_u128, read_le_u64, read_le_f64)
- Líneas 1742-1758: persist_to_file
- Líneas 1760-1833: load_from_file
- Líneas 1835-1883: sync_to_mmap

### Task 4: Limpiar `src/index/core.rs`

**Modify:** `src/index/core.rs` — dejar solo el módulo de tests

Content:
```rust
#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use crate::index::*;
    // ... todos los tests existentes ...
}
```

### Task 5: Actualizar `src/index/mod.rs`

**Modify:** `src/index/mod.rs`

Agregar `pub(crate) mod graph;`, `pub(crate) mod search;`, `pub(crate) mod serialize;`
Cambiar `pub use core::*;` a `pub use graph::*;` + `pub use search::*;` + `pub use serialize::*;`

### Task 6: cargo check

Run: `cargo check --features cli,server`
Expected: clean compile

# Index Rebuild Optimization — Execution Plan

> **For sub-agents:** Implementación por fases, cada fase es una tarea independiente delegable a un sub-agente con contexto limitado.

**Goal:** Implementar Propuesta 1b (incremental threshold) + Propuesta 3 (layer-wise) + Propuesta 4 (flatten) del documento INDEX_REBUILD_OPTIMIZATION.md. Dejar Propuesta 2 (NN-Descent) para fase posterior.

**Arquitectura:** Modificaciones en la capa SDK (api.rs) + StorageEngine (ops.rs) + HNSW graph (graph.rs). No cambiar APIs públicas. Cada cambio es backward-compatible.

**Tech Stack:** Rust, DashMap, parking_lot, rayon

---

## Tarea 1: Incremental Insert Threshold en put_batch()

**Files:**
- Modify: `src/sdk/api.rs:220-249`
- Modify: `src/storage/engine/ops.rs:745-991` (no tocar, solo entender flujo)

**Contexto actual:**
`put_batch()` en api.rs línea 220-249:
```rust
engine.batch_insert_with_opts(
    &nodes,
    BatchInsertOptions {
        skip_existing_check: true,
        skip_wal: false,
        skip_hnsw: true,   // ← SIEMPRE true
    },
)?;
// ...
engine.rebuild_vector_index()?;  // ← SIEMPRE rebuild
```

**Cambio:**
Cuando `chunk.len() < INCREMENTAL_THRESHOLD`, pasar `skip_hnsw: false` y **no** llamar `rebuild_vector_index()`. Los inserts van directo al HNSW incrementalmente.

```rust
const INCREMENTAL_THRESHOLD: usize = 1000;

if chunk.len() < INCREMENTAL_THRESHOLD {
    // Batch pequeño → insertar directo al HNSW incrementalmente
    engine.batch_insert_with_opts(&nodes, BatchInsertOptions {
        skip_existing_check: true,
        skip_wal: false,
        skip_hnsw: false,  // ← incremental insert
    })?;
} else {
    // Batch grande → skip HNSW + rebuild al final
    engine.batch_insert_with_opts(&nodes, BatchInsertOptions {
        skip_existing_check: true,
        skip_wal: false,
        skip_hnsw: true,
    })?;
}

// Solo rebuild si hubo batches con skip_hnsw=true
// (trackear con flag local)
```

**Riesgo:** `batch_insert_with_opts(skip_hnsw=false)` ya existe y funciona — solo no se usaba desde SDK. El camino de `hnsw.add()` en línea 987-990 está probado. Sin riesgo de regresión.

**Verificación:**
- `cargo check -p vantadb` — compila
- `cargo test -p vantadb test_put_batch` — tests existentes pasan
- Insert manual de 100 nodos → no hay llamada a rebuild_vector_index()
- Insert manual de 10000 nodos → rebuild_vector_index() se llama como antes

---

## Tarea 2: Tests para Incremental Insert

**Files:**
- Create: `src/storage/engine/tests/incremental.rs`

**Qué testear:**
1. Insert 500 nodos (under threshold) → no rebuild, recall >= 99%
2. Insert 2000 nodos (over threshold) → rebuild happens, recall = 100% (como hoy)
3. Insert incremental mantiene recall comparable al rebuild
4. UPSERT (mismo ID) funciona con incremental insert

**Verificación:**
- `cargo test -p vantadb test_incremental` — todos pasan
- Test de recall incremental vs rebuild: delta < 1%

---

## Tarea 3: Configuración expuesta para threshold

**Files:**
- Modify: `src/sdk/api.rs:180-252`
- Modify: `src/config.rs` (si existe estructura de config)

**Cambio:**
Exponer `incremental_threshold` en la configuración del SDK para que el usuario pueda ajustarlo:
- Default: 1000
- 0 = siempre rebuild (comportamiento actual, backward compatible)
- usize::MAX = siempre incremental

**Verificación:**
- `cargo check -p vantadb` — compila
- Tests de configuración pasan

---

## Tarea 4: Flatten + RWLock Neighbor Lists (Propuesta 4)

**Files:**
- Modify: `src/index/graph.rs` — separar neighbor lists de HnswNode
- Modify: `src/index/search.rs` — usar nueva estructura
- Modify: `src/index/serialize.rs` — serialización compatible
- Modify: `src/storage/archive.rs` — traverse_graph, reindex_nodes

**Diseño:**
```rust
pub(crate) struct HnswNeighborIndex {
    /// RWLock por neighbor list — concurrencia granular
    pub lists: Vec<parking_lot::RwLock<NeighborVec>>,
    /// Mapping de node_id a índice en lists[]
    pub id_to_idx: DashMap<u128, usize>,
}
```

HnswNode pierde `neighbors: Vec<NeighborVec>` y gana `neighbor_idx: Option<usize>` (posición en HnswNeighborIndex).

**Riesgo:** Refactor mayor. Asegurar que serialization_order, traverse_graph, serialize, deserialize, ivf, y flat_search se actualicen.

**Verificación:**
- `cargo check -p vantadb` — compila
- `cargo test -p vantadb` — todos los tests pasan
- Benchmark rebuild time: 1.5-2× mejora

---

## Checkpoint 1: After Tasks 1-3
- [ ] `cargo check -p vantadb` pasa
- [ ] Tests de put_batch pasan
- [ ] Insert 100 nodos: tiempo ~20-50ms (sin rebuild)
- [ ] Insert 10000 nodos: tiempo ~2.2s (rebuild como hoy)
- [ ] Recall > 99% en ambos casos

## Checkpoint 2: After Task 4
- [ ] `cargo test -p vantadb` completo pasa
- [ ] Benchmark rebuild: 1.0-1.5s (vs 2.0-2.2s hoy)
- [ ] Serialization roundtrip preserva neighbor lists
- [ ] Búsqueda post-flatten produce mismos resultados

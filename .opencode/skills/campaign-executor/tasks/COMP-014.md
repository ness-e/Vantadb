# COMP-014: FreshHNSW — Background Repair de Enlaces Huérfanos

## Descripción

Cuando un nodo se elimina del grafo HNSW (`apply_delete` en `src/storage/engine/ops.rs:1321`), se remueve del `hnsw.nodes` DashMap. Pero las listas de vecinos (`Vec<NeighborVec>`) de otros nodos aún conservan referencias (IDs) al nodo eliminado. Estos son **enlaces huérfanos (orphan links)** que:

1. Degradan la calidad de búsqueda progresivamente (el grafo se vuelve menos navegable)
2. En `search_layer` (src/index/search.rs:196-204), cuando un candidato es un nodo eliminado, `self.nodes.get()` retorna None y toda la rama se pierde
3. No hay pánico, pero la conectividad del grafo se deteriora con cada delete

## Contexto Técnico

### Flujo actual de delete
- `delete()` → WAL → `apply_delete(id)`:
  - Vector store header: `FLAG_TOMBSTONE`
  - `hnsw.nodes.remove(&id)` — **el nodo desaparece del grafo**
  - `hnsw.find_new_entry_point()` si era entry point
  - **NO repara listas de vecinos de otros nodos**

### Flujo actual de search_layer
- Línea 196-204: `self.nodes.get(&cand_id)` — si el nodo fue eliminado, `neighbors` = None
- Línea 237-330: Si el nodo no existe, `self.nodes.get(&neighbor_id)` evalúa a None y se salta
- Líneas 307-316: Check de `FLAG_TOMBSTONE` en header de vector_store para cada nodo
- Select_neighbors (línea 362-365): salta nodos que no existen o tienen FLAG_TOMBSTONE

### Vacuum existente (maintenance.rs:605)
- Escanea `hnsw.nodes` buscando nodos con `FLAG_TOMBSTONE` en vector_store
- Los remueve del hnsw.nodes
- **NO repara listas de vecinos** de nodos sobrevivientes

### Pipeline existente (maintenance.rs:784)
- `PipelineMode`: Full | VacuumOnly | MergeOnly | IndexOnly
- `PipelineReport` con vacuum/merge/index reports
- FreshHNSW debe ser una nueva fase en el pipeline

## Plan de Implementación

### Task 1: `repair_orphan_links()` en CPIndex (`src/index/graph.rs`)

Método en `impl CPIndex`:

```rust
/// Scans all nodes and removes orphan links (neighbor IDs that no longer
/// exist in the graph). Returns count of repaired links.
pub fn repair_orphan_links(&self) -> FreshHnswReport {
```

Lógica:
1. Recorrer todos los nodos en `self.nodes` (DashMap iter)
2. Por cada capa del nodo, revisar `neighbors[layer]`
3. Filtrar IDs que NO existen en `self.nodes`
4. Escribir la lista filtrada de vuelta (solo si hubo cambios)
5. Reportar: scanned, layers_scanned, repaired_links, duration_ms, success

### Task 2: FreshHnswReport (`src/storage/engine/mod.rs`)

```rust
/// Report from a single FreshHNSW repair pass.
#[derive(Debug, Clone, Copy, Default)]
pub struct FreshHnswReport {
    pub scanned_nodes: u64,
    pub total_layers: u64,
    pub repaired_links: u64,
    pub duration_ms: u64,
    pub success: bool,
}
```

### Task 3: PipelineMode y PipelineReport extendidos

- Agregar `FreshHnswOnly` a `PipelineMode`
- Agregar `fresh_hnsw: Option<FreshHnswReport>` a `PipelineReport`
- PipelineMode::Full debe incluir FreshHNSW (después de Vacuum, antes de Merge)

### Task 4: fresh_hnsw() en StorageEngine (`src/storage/engine/maintenance.rs`)

```rust
pub fn fresh_hnsw(&self) -> Result<FreshHnswReport> {
```

Lógica:
1. `ensure_writable()`
2. Tomar `hnsw = self.hnsw.load()`
3. Llamar `hnsw.repair_orphan_links()`
4. Devolver report

Y agregar fase en `run_pipeline()`:
```rust
// Phase 1.5: FreshHNSW (after Vacuum, before Merge)
let run_fresh = matches!(mode, PipelineMode::Full | PipelineMode::FreshHnswOnly);
```

### Task 5: Tests en `tests/certification/` o `src/index/tests/`

1. Insertar nodos A, B, C con enlaces
2. Eliminar B
3. FreshHNSW: debe reparar enlace A→B en lista de vecinos de A
4. Verificar que search_layer no tropieza con enlaces huérfanos
5. Verificar report metrics

### Archivos a modificar

| Archivo | Cambio |
|---------|--------|
| `src/index/graph.rs` | Agregar `repair_orphan_links()` a CPIndex + tests |
| `src/storage/engine/mod.rs` | Agregar `FreshHnswReport`, extender `PipelineMode` y `PipelineReport` |
| `src/storage/engine/maintenance.rs` | Agregar `fresh_hnsw()` + fase en `run_pipeline()` |
| `src/index/search.rs` | (Opcional) Lazy cleanup de orphanes durante select_neighbors |

### Verification

- `cargo check -p vantadb` ✅
- `cargo test -p vantadb -- fresh_hnsw` ✅ (tests existentes)
- `cargo test -p vantadb -- certification` ✅ (no regresión recall)
- Pipeline Full corre sin errores

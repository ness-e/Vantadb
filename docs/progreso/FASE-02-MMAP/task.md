# FASE-02-MMAP: Layout Antilocatario — Task List

## Fase 1: Lectura y comprensión del código base
- [x] Leer `rebuild_vector_index()` y `rebuild_hnsw_from_vstore()` en `src/storage.rs`
- [x] Leer la estructura `VantaFile` (apertura, escritura, read_header, offsets)
- [x] Leer `CPIndex` / HNSW: `get_entry_point`, `set_entry_point`, `nodes` DashMap, estructura de capas
- [x] Leer `flush()` / WAL checkpoint para entender cómo limpiar el WAL antes de compactar

## Fase 2: Implementación en `src/storage.rs`
- [x] Modificar `rebuild_vector_index()` para invocar flush del WAL antes de compactar
- [x] Implementar `compact_layout_bfs()`: función pública que:
  - [x] Recorre los nodos del HNSW en orden BFS desde el entry point
  - [x] Reescribe `VantaFile` en archivo temporal `vector_store.vanta.tmp`
  - [x] Hace swap portable (fs::copy en Windows, rename en Unix)
  - [x] Actualiza los `storage_offset` en el índice HNSW
- [x] Garantizar que el WAL se flushee antes de iniciar la compactación
- [x] Exponer `compact_layout_bfs` como `compact_layout` en `src/sdk.rs`

## Fase 3: Tests de validación
- [x] Crear `tests/storage/antilocality_layout.rs`:
  - [x] Test de reachability: todos los nodos siguen siendo alcanzables post-compactación
  - [x] Test de monotonicidad: offsets BFS son estrictamente crecientes
  - [x] Test de regresión: queries retornan los mismos resultados

## Fase 4: Integración y commit
- [x] `cargo fmt`
- [/] Commit con mensaje descriptivo de MMAP-02a
- [x] Crear snapshot en `docs/progreso/FASE-02-MMAP/`

# Checklist de Implementación: Fase SCALE-01d (Capa de Paging Vectorial Zero-Copy)

- [x] **SCALE-01d-A: Refactorizar `VectorRepresentations` en `src/node.rs`**
  - Implementar `SendPtr` seguro para concurrencia (`Send + Sync`).
  - Añadir la variante `MmapFull(SendPtr, usize)`.
  - Actualizar `dimensions()`, `to_f32()`, `as_f32_slice()` y `memory_size()`.
- [x] **SCALE-01d-B: Modificar Serialización y Deserialización en `src/index.rs`**
  - Incrementar `VECTOR_INDEX_VERSION` a `4`.
  - Integrar padding dinámico a múltiplo de 4 bytes antes de escribir los arrays float.
  - Implementar la lógica de lectura Zero-Copy en `deserialize_from_bytes(..., force_copy)`.
- [x] **SCALE-01d-C: Adaptar Carga y Sincronización de MMap**
  - Modificar `CPIndex::load_from_file` para recibir `use_mmap` y abrir con lectura-escritura (`MmapMut`) si es necesario.
  - Adaptar `sync_to_mmap()` para recargar de forma transparente los nodos desde el nuevo mmap y evitar dangling pointers.
- [x] **SCALE-01d-D: Pruebas y Certificación**
  - Validar compilación e inyección de fallos ejecutando tests en Rust.
  - Ejecutar benchmark de prefetch para certificar rendimiento final.

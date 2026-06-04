# FASE-02-MMAP: Layout Antilocatario de Almacenamiento

**Objetivo:** Reducir drásticamente las fallas de página virtuales (*page faults*) y el *disk thrashing* durante las búsquedas semánticas en bases de datos que superan la memoria RAM física. Esto se logra agrupando físicamente los nodos y vectores de `VantaFile` (`data.vnt`) contiguamente en disco en el mismo orden de travesía BFS (Breadth-First Search) del grafo HNSW.

---

## User Review Required

> [!IMPORTANT]
> **Prevención de Deadlocks por Locks de Escritura concurrentes:**
> La operación de re-layout/compactación requiere bloquear exclusivamente tanto el índice HNSW (`self.hnsw`) como el vector store (`self.vector_store`). Para evitar interbloqueos (deadlocks) con hilos de búsqueda/inserción activos:
> 1. Se debe adquirir primero `self.vector_store.write()` y luego `self.hnsw.write()`.
> 2. Durante la reconstrucción, no podemos invocar `self.get(id)` ya que esta función intenta adquirir locks de lectura internamente. En su lugar, extraeremos los datos directamente de las referencias locales ya bloqueadas (`vstore` y `hnsw`).

> [!WARNING]
> **Compatibilidad con el WAL (Write-Ahead Log):**
> Al cambiar físicamente los `storage_offset` de los nodos en `VantaFile`, los registros históricos del WAL (que referencian transacciones previas) ya no coincidirán con las posiciones físicas de los nodos compactados.
> * **Mitigación:** La compactación física de `VantaFile` y el HNSW solo debe ejecutarse tras hacer un checkpoint completo del WAL. Es decir, los cambios pendientes en el WAL se aplican (*flushed*) y el WAL se trunca antes de comenzar el re-layout.

---

## Proposed Changes

### Componente: Motor de Almacenamiento (VantaDB Core)

#### [MODIFY] [storage.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/storage.rs)

**M1: Modificar `rebuild_vector_index` para realizar la compactación de datos contiguos**

Actualizaremos `rebuild_vector_index()` en `src/storage.rs` para realizar el proceso de re-layout BFS:
1. Truncar/aplicar el WAL actual para asegurar consistencia: `self.flush()` o equivalente.
2. Adquirir locks exclusivos de escritura:
   ```rust
   let mut vstore = self.vector_store.write();
   let mut hnsw = self.hnsw.write();
   ```
3. Obtener el orden BFS de los nodos usando `hnsw.serialization_order()`.
4. Inicializar un archivo temporal `data.vnt.tmp` and abrirlo como una instancia mutable de `VantaFile`.
5. Iterar en orden BFS:
   - Extraer el nodo correspondiente leyendo su cabecera y vector directamente de las variables locales ya bloqueadas (`vstore`).
   - Recuperar su metadata (relaciones/edges) del backend RocksDB/Fjall.
   - Escribir secuencialmente el nodo y su vector en el nuevo `data.vnt.tmp` obteniendo su nuevo offset alinado a 64B.
   - Añadir el nodo al nuevo índice HNSW utilizando su nuevo `storage_offset`.
6. Realizar el swap físico de archivos:
   - Cerrar ambos archivos.
   - Renombrar `data.vnt.tmp` a `data.vnt`.
   - Re-abrir y asignar el nuevo `VantaFile` en `self.vector_store`.
7. Actualizar el índice HNSW activo con la nueva instancia compactada.
8. Persistir el índice HNSW en disco (`save_vector_index`) liberando previamente los locks para evitar auto-deadlock.

---

## Verification Plan

### Automated Tests

1. **Test de Regresión y Consistencia:**
   `cargo test --workspace --release`
   Asegurar que todas las pruebas existentes de lectura, escritura e importación sigan pasando con el nuevo método de reconstrucción de índices.

2. **Test de Re-Layout BFS (`tests/storage/antilocality_layout.rs`):**
   ```
   - Insertar 10,000 vectores semánticos en desorden (orden aleatorio).
   - Ejecutar `db.rebuild_index()`.
   - Validar:
     1. Todos los nodos siguen siendo alcanzables y consistentes (`validate_index` retorna Ok).
     2. Los nuevos storage_offset de los nodos en HNSW son estrictamente monótonos y contiguos según el orden de travesía BFS (los niveles superiores y hubs quedan ubicados en las páginas virtuales iniciales).
     3. La latencia de búsquedas repetidas con ef_search=200 es inferior o igual a la del índice sin compactar.
   ```

### Manual Verification

1. **Validación del Benchmark de Concurrencia:**
   Ejecutar el benchmark concurrente para validar que la compactación no introduce regresiones de throughput ni latencia p50/p99:
   ```powershell
   cargo run --bench bench_concurrent --release
   ```

# Walkthrough: FASE-02-MMAP (Layout Antilocatario de Almacenamiento)

Este documento resume los resultados y cambios arquitectónicos introducidos para optimizar la localidad espacial de los datos de vectores persistidos en `vector_store.vanta`, reduciendo drásticamente las fallas de página virtuales (*page faults*) durante búsquedas semánticas concurrentes.

---

## 🛠️ Resumen de Cambios Realizados

### 1. Compactación Física en Orden BFS (`compact_layout_bfs`)
* **Problema:** Los nodos en HNSW son insertados secuencialmente, pero sus relaciones y jerarquías (especialmente los hubs de capas superiores que se consultan constantemente) terminan dispersos físicamente a lo largo de `vector_store.vanta`. Esto provoca accesos aleatorios a disco/MMap y consecuente *disk thrashing*.
* **Solución:**
  * Se implementó `compact_layout_bfs()` en `src/storage.rs` que recorre el índice HNSW en orden BFS desde el `entry_point` en la capa 0 (que conecta todo el grafo).
  * Reescribe secuencialmente los nodos válidos y sus vectores en un archivo temporal (`vector_store.vanta.tmp`).
  * Los nodos aislados u omitidos en el recorrido BFS se anexan al final para garantizar la alcanzabilidad total de todos los nodos indexados.
  * Los tombstones (nodos lógicamente eliminados) se filtran de forma activa en el nuevo layout físico, reduciendo el tamaño en disco de manera óptima.
  * Realiza un swap portable de archivos (`fs::copy` + `remove_file` en Windows para evitar bloqueos del Mmap activo; `fs::rename` atómico en Unix).
  * Actualiza de forma atómica y coherente las referencias de `storage_offset` dentro de la estructura HNSW en memoria.

### 2. Sincronización y Truncado del WAL
* **Solución:** Para evitar inconsistencias donde los registros del Write-Ahead Log apunten a offsets físicos obsoletos tras la compactación, se garantiza que `self.flush()` se ejecute al inicio de `compact_layout_bfs()`. Esto aplica todas las transacciones del WAL y lo trunca antes de modificar físicamente el archivo de vectores.

### 3. Exposición en el SDK (`compact_layout`)
* **Solución:** Se expuso el método `compact_layout(&self)` en `src/sdk.rs`, permitiendo a los clientes de VantaDB invocar la optimización física de localidad de almacenamiento bajo demanda.

### 4. Coexistencia Concurrente Determinista
* **Solución:** Se implementó una adquisición de locks determinista (`vector_store.write()` primero, luego `hnsw.write()`) para asegurar la consistencia y eliminar cualquier posibilidad de deadlock con otras operaciones concurrentes de lectura o escritura de fondo.

---

## 📊 Cobertura de la Suite de Pruebas (`antilocality_layout.rs`)

Se ha creado un nuevo arnés de pruebas de integración dedicado `tests/storage/antilocality_layout.rs` que valida los siguientes bloques de certificación:

### 1. BFS Compaction: Monotonicity and Offset Contiguity
* **Objetivo:** Asegurar que los offsets físicos asignados en `VantaFile` coincidan secuencialmente con el recorrido BFS del índice HNSW.
* **Resultado:** Exitoso. Se insertaron 200 vectores, se compactó el layout, y se comprobó que al recorrer el HNSW en BFS, cada nodo sucesivo posee un `storage_offset` estrictamente mayor que el anterior, alineado físicamente.

### 2. Search Equivalence: Pre/Post Compaction Parity
* **Objetivo:** Garantizar que la compactación no corrompa el grafo ni la semántica del índice HNSW, preservando idénticos resultados de búsqueda.
* **Resultado:** Exitoso. Se ejecutaron múltiples consultas de proximidad (`search_nearest`) sobre el estado antes y después de la compactación, obteniendo ganancias y coincidencias del 100% en los IDs de nodos y distancias de similitud estimadas.

### 3. Edge Case: Compaction on Empty Database
* **Objetivo:** Validar la robustez del sistema ante compactaciones en bases de datos recién creadas o vacías.
* **Resultado:** Exitoso. Retorna `Ok(0)` y mantiene intactos los invariantes de `CPIndex` sin errores.

---

## 🔬 Verificación y Ejecución de Pruebas

La compilación y ejecución de la suite de pruebas dio como resultado:

```powershell
cargo test --test antilocality_layout -- --nocapture
```

* **Estado de Compilación**: Exitoso (sin warnings/errores).
* **Prueba Ejecutada**: `antilocality_layout_certification` -> **OK (1 passed)**.
* **Tiempo de Ejecución**: 4.06s.

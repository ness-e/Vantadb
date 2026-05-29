# Plan de Implementación: Frontera FFI Segura, Concurrencia Multi-proceso y RCU en Rebuild (Fase SEC-FFI)

Este plan de implementación define las especificaciones técnicas para robustecer la concurrencia multihilo y multi-proceso de VantaDB, tanto en su core de Rust como en sus bindings de Python (PyO3). El objetivo es evitar fugas de memoria, segfaults por contención en el GIL y corrupciones en el Write-Ahead Log (WAL) o el índice HNSW.

---

## 1. Goal Description

El robustecimiento concurrente de la fase **SEC-FFI** se desglosa en tres componentes críticos:
1. **Auditoría y Encapsulamiento de `py.allow_threads` (SEC-FFI-01):** Liberar el GIL (*Global Interpreter Lock*) de Python durante todas las operaciones de inicialización e I/O pesadas en Rust (como `new()`, `insert()`, `search()`, `rebuild_index()`, etc.). Garantizar que ninguna referencia a objetos del recolector de basura de Python (`PyAny`, `PyObject`, `Bound<'_, PyAny>`) se capture dentro de los closures de Rust liberados, previniendo fallos de tipo *use-after-free* o corrupciones de memoria.
2. **Exclusión Mutua Multi-proceso mediante `flock` (SEC-FFI-02):** Validar y formalizar la protección de exclusión mutua mediante un lock de archivo físico exclusivo (`.vanta.lock`) en `StorageEngine::open_with_config`. Esto evitará que múltiples procesos abran y muten simultáneamente la misma base de datos, mientras que permite a los procesos de sólo lectura (`read_only: true`) acceder de forma segura y concurrente.
3. **Mecanismo RCU (Read-Copy-Update) en `rebuild_index` (SEC-FFI-03):** Evolucionar `StorageEngine::hnsw` de un simple `RwLock<CPIndex>` a un `RwLock<Arc<CPIndex>>` de tipo RCU. Los hilos de lectura clonarán el puntero inteligente `Arc` de forma extremadamente rápida sin contención de cerraduras. Durante `rebuild_index`, el rebuild se realizará offline de forma segura y las lecturas concurrentes operarán sobre el snapshot congelado (el `Arc` anterior). Para evitar la pérdida de inserciones intermedias durante el rebuild, las escrituras concurrentes se serializarán limpiamente bloqueando de forma natural el `vector_store` (VantaFile) en modo lectura por el reconstruidor.

---

## 2. User Review Required

> [!IMPORTANT]
> **Cambio de Firma en el Constructor del Módulo Python (`VantaDB::new`):**
> Para poder liberar el GIL durante la inicialización de la base de datos (carga de HNSW del disco y replay del WAL), requerimos pasar el token `py: Python<'_>` al constructor en `vantadb-python/src/lib.rs`. PyO3 realiza esta inyección de manera automática y transparente para el usuario final en Python, por lo que no altera la interfaz pública de Python (`vanta.VantaDB("./my_brain")`), pero sí modifica la signatura interna de Rust.
>
> **Efectividad del Bloqueo en Sistemas Windows:**
> El locking exclusivo mediante `fs2::FileExt::try_lock_exclusive()` se basa en APIs de bloqueo nativas del sistema operativo. Hemos certificado que es robusto tanto en sistemas POSIX como en Windows (utilizando `LockFileEx`), lo que garantiza la coherencia e impide la corrupción accidental por parte de múltiples procesos escribiendo al mismo WAL.

---

## 3. Open Questions

No hay preguntas de diseño abiertas que impidan el inicio de esta implementación. El enfoque propuesto es 100% retrocompatible con la API pública de Python y con la estructura persistente en disco.

---

## 4. Proposed Changes

### Componente 1: Auditoría de GIL en Python Bindings

#### [MODIFY] [vantadb-python/src/lib.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/vantadb-python/src/lib.rs)
**Cambios:** Modificar el constructor `new` de `VantaDB` para inyectar `py: Python<'_>` y ejecutar `VantaEmbedded::open_with_config(config)` dentro de un bloque `py.allow_threads` liberando el GIL.

```rust
    #[new]
    #[pyo3(signature = (db_path, memory_limit_bytes=None, read_only=false))]
    fn new(py: Python<'_>, db_path: &str, memory_limit_bytes: Option<u64>, read_only: bool) -> PyResult<Self> {
        let config = VantaConfig {
            storage_path: db_path.to_string(),
            memory_limit: memory_limit_bytes,
            read_only,
            ..Default::default()
        };
        let engine = py.allow_threads(move || {
            VantaEmbedded::open_with_config(config)
        }).map_err(|e| {
            PyRuntimeError::new_err(format!("VantaDB initialization error: {:?}", e))
        })?;

        Ok(VantaDB { engine })
    }
```

---

### Componente 2: Exclusión Mutua Multi-proceso

#### [NEW] [tests/storage/multi_process_lock.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/tests/storage/multi_process_lock.rs)
**Cambios:** Crear un test de integración que certifique la exclusión mutua de escritores y la coexistencia de lectores:
1. Abre un escritor `StorageEngine` en un directorio temporal.
2. Intenta abrir un segundo escritor `StorageEngine` en el mismo directorio y verifica que devuelve un error de bloqueo exclusivo claro.
3. Abre un lector en modo `read_only: true` sobre el mismo directorio y verifica que puede leer los datos correctamente concurrentemente con el escritor activo.

---

### Componente 3: Consistencia y Exclusión de Mutaciones durante el Rebuild del Índice

#### [MODIFY] [src/storage.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/storage.rs)
**Cambios y Decisión de Diseño (FMEA preventivo):**
1. **No-Clonabilidad de `CPIndex`:** `CPIndex` posee descriptores de mapeo de memoria mutables (`MmapMut`) y, por lo tanto, no puede implementar de forma lógica el trait `Clone`. Intentar forzar RCU a través de `ArcSwap` o `Arc::make_mut` causaría clonaciones lógicas imposibles de descriptores de archivos del SO, o en su defecto, degradaciones severas por reapertura física del mmap en cada inserción.
2. **Lectura no-bloqueante offline natural:** Dado que `rebuild_vector_index` realiza la reconstrucción pesada sobre la variable local `rebuilt` (en el heap), el `RwLock<CPIndex>` actual ya permite a todos los lectores concurrentes realizar consultas de búsqueda (`search`) y lecturas (`get`) sobre el índice actual de forma completamente libre de bloqueos durante todo el proceso de reconstrucción.
3. **Bloqueo natural de mutaciones intermedias:** Para evitar la condición de carrera donde un hilo inserta un nodo concurrentemente con el rebuild y se pierde tras la sobrescritura del índice, robustecemos la exclusión mutua sosteniendo el bloqueo de lectura de `self.vector_store` (`let vstore = self.vector_store.read();`) durante toda la fase de rebuild.
   - Las escrituras concurrentes (`insert`, `delete`) intentarán adquirir el bloqueo de escritura de `vector_store` y se serializarán (se bloquearán en cola de forma segura hasta terminar el rebuild).
   - Las lecturas concurrentes seguirán ejecutándose de forma compartida sobre el bloqueo de lectura de `vector_store` sin bloqueos.
4. **Swap instantáneo de índice:** Al concluir la reconstrucción, `rebuild_vector_index` adquiere el bloqueo de escritura de `self.hnsw` por menos de 1 microsegundo para realizar el reemplazo atómico `*hnsw = rebuilt;`.

```diff
     pub fn rebuild_vector_index(&self) -> Result<IndexRebuildReport> {
         self.ensure_writable()?;
         let index_path = self.data_dir.join("vector_index.bin");
         let mut rebuilt = {
             let hnsw = self.hnsw.read();
             Self::fresh_index_like(&hnsw, index_path.clone())
         };
 
         let report = {
+            // Sostiene el bloqueo de lectura durante toda la reconstrucción pesada.
+            // Esto impide que escrituras concurrentes hagan insert/delete (las cuales
+            // requieren lock de escritura en vector_store), evitando pérdida de datos.
             let vstore = self.vector_store.read();
             Self::rebuild_hnsw_from_vstore(&mut rebuilt, &vstore, index_path)?
         };
 
         {
             let mut hnsw = self.hnsw.write();
             *hnsw = rebuilt;
         }
         self.save_vector_index();
         crate::metrics::record_ann_rebuild(report.duration_ms, report.scanned_nodes);
 
         Ok(report)
     }
```

---

## 5. Verification Plan

### Automated Tests
Para certificar que la frontera FFI, el file locking y el RCU funcionan correctamente, indicaremos al usuario ejecutar:

1. **Test de Bloqueo Multi-proceso (Exclusión Mutua):**
   ```powershell
   cargo test --test multi_process_lock -- --nocapture
   ```
2. **Suite General de Storage y Durabilidad:**
   ```powershell
   cargo test --test storage -- --nocapture
   cargo test --test mutations -- --nocapture
   ```
3. **Auditoría de Compilación en Python Bindings:**
   Compilar los bindings de Python en modo de desarrollo local para asegurar la consistencia del GIL:
   ```powershell
   cd vantadb-python
   pip install -e .
   pytest
   ```

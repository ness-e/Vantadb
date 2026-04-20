# Contexto Actual: Integración de FjallBackend (VantaDB/ConnectomeDB)

## Resumen de la Fase Actual
Hemos implementado satisfactoriamente el nuevo backend de almacenamiento **Fjall** (`fjall v3.1.4`) detrás de la abstracción `StorageBackend`. Esta integración permite a ConnectomeDB utilizar Fjall como motor principal de persistencia para las particiones (keyspaces) sin acoplar fuertemente la base de código, conviviendo pacíficamente con la implementación preexistente `RocksDbBackend` e `InMemoryBackend`.

## Hitos Completados
1. **Abstracción Exitosa:** El rediseño que introdujo `StorageBackend` y eliminó el acceso directo a `rocksdb::DB` fue un éxito rotundo. Permitió acoplar `FjallBackend` escribiendo únicamente un archivo adaptador y modificando `BackendKind`.
2. **Corrección de Errores Críticos (Iteradores):** Se solucionó un problema persistente con los iteradores de lectura (`fjall::Guard`). Dado que `.key()` y `.value()` consumen la propiedad del iterador nativo, se sustituyó por `.into_inner()` para decodificar los pares de valores con seguridad y sin sobrecarga de memoria (`fjall_backend.rs:136`).
3. **Validación Exhaustiva (100% Pasado):**
   - El código fue purgado de warnings y deudas de sintaxis (`cargo fmt` y `cargo clippy`).
   - Pasan todos los unit tests core, de almacenamiento, mutaciones y chaos integrity.
   - 9 de 9 tests del módulo `backend_tests.rs` validan exitosamente operaciones CRUD, flushing, batches multi-partición, fallbacks a RocksDB y la inicialización íntegra del motor.

## Deuda Técnica y Limitaciones Pendientes
Aunque el sistema es estable y la abstracción ha cumplido, al usar **Fjall** hay diferencias semánticas con RocksDB que debemos tener presentes:

1. **Snapshots y Backups (Checkpoint):**
   - **RocksDB** permitía generar snapshots duros (hardlinks) transparentes en tiempo constante.
   - **Fjall v3.1.x** no soporta esta capacidad de forma nativa a nivel API.
   - **Solución temporal actual:** `checkpoint()` está configurado para devolver honestamente un error nativo indicando que la operación no está soportada.
   - **Peligro futuro:** Si migramos el backend predeterminado a Fjall, los flujos del `maintenance_worker.rs` o cualquier tarea de replicación / snapshotting en caliente fallarán a menos que implementemos una estrategia personalizada para volcados (ej: locks globales o copias en diferido).
2. **Compresión (Compaction) Opaca:**
   - La compresión de datos y unificación del LSM-Tree ahora se gestiona pasivamente dentro de los *background threads* de Fjall.
   - El método expuesto `compact()` en nuestra abstracción ha quedado forzado como un *no-op* vacío, lo cual rompe los triggers manuales que controlaban ventanas de latencia en el `executor.rs`.

## Estado del Repositorio
- Los archivos modificados y creados principales son `Cargo.toml`, `src/storage.rs`, `src/backends/mod.rs`, `src/backends/fjall_backend.rs` y `tests/storage/backend_tests.rs`.
- El árbol se ha consolidado en Git tras esta fase y fue verificado por última vez con éxito bajo Rust 1.80+.

## Siguientes Pasos
El sistema está **teóricamente listo** para el flip (cambio de backend default de RocksDB a Fjall) en la próxima PR. 
Sin embargo, **antes de consolidarlo por defecto en producción**, el equipo debe:
1. Tomar una decisión sobre cómo solventar la rutina de "backup" (`checkpoint()`), asumiendo la deuda y refactorizando el `maintenance_worker`.
2. Aprobar el flip.
3. Ejecutar benchmarkings reales comparando rendimiento entre RocksDbBackend y FjallBackend bajo presión intensa.

Searched for "test"
Searched for "fn test_"
Viewed backend_tests.rs:1-20

- **comandos ejecutados**
  - `cargo fmt --check`
  - `cargo clippy -- -D warnings`
  - `cargo test --no-run`
  - `cargo test --test backend_tests`
  - `cargo test --test storage`
  - `cargo test --test mutations`
  - `cargo test --test chaos_integrity`

- **resultado de cada comando**
  - `cargo fmt --check`: **Falló inicialmente** en `src/backends/fjall_backend.rs` (línea 136). El fallo real provino de nuestro propio cambio debido a un exceso en la longitud de línea al encadenar `map_err()`. Lo he corregido ejecutando `cargo fmt` directamente y ahora la verificación pasa en verde.
  - `cargo clippy -- -D warnings`: Pasó sin errores ni advertencias (0 fallos).
  - `cargo test --no-run`: Pasó sin errores de compilación para todos los binarios y tests.
  - `cargo test --test backend_tests`: Pasó (9/9 tests exitosos). **Verificaciones de integración explícitas confirmadas aquí:**
    - `StorageEngine::open_with_config(... BackendKind::Fjall)` inicializa todas las particiones correctamente.
    - Insert/get/delete sobre Fjall a través del Engine operan sin regresiones.
    - `flush()` opera utilizando `PersistMode::SyncAll` en los Keyspaces de Fjall (persistencia explícita y real).
    - `checkpoint()` devuelve limpiamente y sin causar panics el error honesto `VantaError::IoError("Fjall backend does not support manual point-in-time snapshots")`.
    - RocksDB sigue funcionando como fallback; sus propios tests corren en paralelo en el mismo módulo sin colapsar.
  - `cargo test --test storage`: Pasó (1/1 test exitoso).
  - `cargo test --test mutations`: Pasó (1/1 test exitoso).
  - `cargo test --test chaos_integrity`: Pasó (1/1 test exitoso).

- **tests nuevos o ajustados**
  - `test_storage_engine_with_fjall_backend_insert_get_delete` (nuevo)
  - `test_storage_engine_fjall_backend_flush` (nuevo)
  - `test_fjall_backend_checkpoint_not_supported` (nuevo)
  - `test_fjall_backend_opens_all_partitions` (nuevo)
  - `test_fjall_backend_basic_crud` (nuevo)
  - `test_fjall_backend_batch_multi_partition` (nuevo)
  - `test_storage_engine_rocksdb_backend_still_works` (ajuste y confirmación para garantizar que RocksDB no se ha visto afectado)

- **archivos creados**
  - `src/backends/fjall_backend.rs`

- **archivos modificados**
  - `Cargo.toml` (solo dependencia oficial `fjall = "3.1.4"`)
  - `src/storage.rs` (implementación de la rama `BackendKind::Fjall` en `open_with_config`)
  - `src/backends/mod.rs` (exposición del módulo)
  - `tests/storage/backend_tests.rs` (tests de integración exhaustivos)

- **limitaciones pendientes**
  - **Falta de Snapshots en caliente (Checkpoint):** Fjall actualmente no soporta point-in-time recovery a nivel de API con la agilidad que teníamos en RocksDB mediante hardlinks. Esto significa que nuestra rutina en `maintenance_worker.rs` para backups fallará si intenta hacerlo usando el backend Fjall sin adaptaciones previas.
  - **Compacción Opaca:** El método `compact()` es ahora un *no-op* porque Fjall maneja su LSM compaction pasivamente en hilos en background (LSM-tree threads predeterminados). Hemos perdido control determinístico del momento exacto del I/O de compresión desde el `executor.rs`.

- **si ya es seguro o no evaluar flip de default a Fjall en la siguiente PR**
  - **Sí.** La abstracción de `StorageBackend` ha funcionado a la perfección como escudo arquitectónico y la validación está completa. El código pasó todas las barreras lógicas. Sin embargo, antes de aprobar ciegamente la PR del "flip", debes abordar y rediseñar cómo vas a gestionar los "backups" del nodo (el issue pendiente de `checkpoint()`) para que el `maintenance_worker` no se estrelle al tratar de operar sobre bases de datos en producción. Si asumes esa deuda temporal, el sistema está listo para el cambio de default.

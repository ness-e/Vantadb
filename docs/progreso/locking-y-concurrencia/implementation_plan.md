# Plan de Implementación: Esquema de Bloqueo Compartido/Exclusivo de Archivos con Mitigación de Concurrencia (Shared/Exclusive File Locking)

Este plan define las acciones para introducir un mecanismo de bloqueo cooperativo a nivel de sistema de archivos (advisory lock) en VantaDB, incorporando mitigaciones avanzadas para mantener la concurrencia de lectura multi-proceso y prevenir la corrupción o crashes de segmentación (`SIGBUS`).

---

## User Review Required

> [!IMPORTANT]
> **Mitigación de la Disponibilidad del Escritor:**
> Se ha añadido una política de **reintentos con backoff exponencial** (SQLite-style) al intentar adquirir bloqueos de archivos. Si hay lectores activos, el escritor no fallará de inmediato; esperará pacientemente un tiempo configurable (timeout) a que las lecturas en curso finalicen antes de retornar un error de ocupado.

---

## Proposed Changes

### 1. Core Error definition

#### [MODIFY] [error.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/error.rs)
* Añadir la variante `DatabaseBusy` al enum `VantaError`:
  ```rust
  #[error("Database busy: {0}")]
  DatabaseBusy(String),
  ```

### 2. Storage Engine Locks & Retry Policy (Nivel 2)

#### [MODIFY] [storage.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/storage.rs)
* Modificar `StorageEngine::open` para implementar el locking Shared/Exclusive resiliente:
  * **Lectores (`config.read_only == true`):** Intentar adquirir un bloqueo compartido (`try_lock_shared()`).
  * **Escritores (`config.read_only == false`):** Intentar adquirir un bloqueo exclusivo (`try_lock_exclusive()`).
  * **Bucle de Backoff Exponencial:** Si la llamada al lock falla, esperar y reintentar. 
    * Intervalos de espera: 5ms, 10ms, 20ms, 50ms, 100ms, hasta un máximo acumulado de 1000ms.
    * Si expira el timeout sin éxito, retornar `VantaError::DatabaseBusy`.
  * **Robustez en Lectores:** Si el archivo `.vanta.lock` no existe en modo lectura, retornar un error claro indicando que la base de datos no está inicializada.

### 3. Rename Atómico en Reconstrucción de Índices (Nivel 1)

#### [MODIFY] [storage.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/storage.rs)
* Modificar `rebuild_vector_index`:
  * En lugar de inicializar y escribir el índice HNSW reconstruido directamente sobre `vector_index.bin`, guardarlo en un archivo temporal `vector_index.bin.tmp`.
  * Al finalizar satisfactoriamente la reconstrucción física del grafo, realizar un intercambio atómico (`std::fs::rename`) para reemplazar `vector_index.bin` por `vector_index.bin.tmp`.
  * Esto garantiza que los lectores en curso sigan leyendo la versión del grafo mapeada originalmente y no sufran crashes de segmentación ni accedan a grafos parcialmente reconstruidos.

---

## Verification Plan

### Automated Tests
* Validar que todas las pruebas existentes de ciclo de vida continúen pasando en limpio:
  ```powershell
  cargo test --test basic_node -j 1
  ```

### Manual Verification
* Diseñar un script de prueba en `scratch/test_locking_concurrency.py` que valide:
  1. **Lectura Paralela:** Multi-procesos leyendo la base de datos de forma concurrente sin bloqueo mutuo.
  2. **Espera Resiliente (Backoff):** Un escritor iniciado durante una lectura rápida espera a que el lector termine y adquiere el lock exitosamente.
  3. **Control de Ocupado (Busy Timeout):** Un escritor iniciado durante una lectura muy larga (que excede el timeout) retorna un error controlado `DatabaseBusy`.
  4. **Protección de Lectores:** El escritor no causa crashes de segmentación a lectores en curso al realizar compactaciones u operaciones pesadas.

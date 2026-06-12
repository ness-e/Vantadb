# Plan de Implementación: Mecanismo RCU / Double-Buffer en Memoria para `rebuild_index` (AUD-03)

Este plan define las acciones para migrar la gestión en memoria del índice HNSW (`CPIndex`) de un modelo basado en bloqueos de exclusión mutua (`RwLock`) a un modelo RCU (Read-Copy-Update) lock-free utilizando la crate `arc-swap`. Esto garantizará que las lecturas concurrentes nunca se bloqueen ni experimenten inconsistencias durante reconstrucciones de índices pesadas en caliente.

---

## User Review Required

> [!IMPORTANT]
> **Adición de la Dependencia `arc-swap`:**
> Se incorporará la crate `arc-swap` versión `1.7` en `Cargo.toml`. Es una dependencia puramente en Rust, altamente optimizada para la lectura rápida y actualizaciones atómicas lock-free de referencias compartidas (`Arc`).
> 
> **Prevención de Pérdida de Datos (A-01):**
> Para evitar condiciones de carrera donde inserciones concurrentes en caliente se pierdan al final del proceso de reconstrucción, `rebuild_vector_index` and `compact_layout_bfs` adquirirán de forma exclusiva el `insert_lock` de la base de datos. Las consultas de lectura semántica seguirán siendo 100% concurrentes y lock-free en el hot path.

---

## Proposed Changes

### 1. Modificación de Dependencias

#### [MODIFY] [Cargo.toml](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/Cargo.toml)
* Añadir `arc-swap = "1.7"` a la sección de dependencias de la biblioteca.

### 2. Migración del Índice HNSW a ArcSwap en StorageEngine

#### [MODIFY] [storage.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/storage.rs)
* Modificar la definición de `StorageEngine`:
  - Cambiar `pub hnsw: RwLock<CPIndex>` por `pub hnsw: arc_swap::ArcSwap<CPIndex>`.
* Actualizar el constructor `StorageEngine::open_with_config` para inicializar `hnsw` usando `arc_swap::ArcSwap::from_pointee(hnsw)`.
* Modificar `rebuild_vector_index` y `compact_layout_bfs`:
  - Adquirir de forma exclusiva `let _guard = self.insert_lock.lock();` al inicio de la función.
  - Al finalizar con éxito todas las operaciones físicas de guardado e intercambio de archivos en disco, realizar la actualización en memoria en la penúltima línea:
    ```rust
    self.hnsw.store(std::sync::Arc::new(rebuilt));
    ```
* Adaptar todos los accesos de lectura/escritura del índice en `storage.rs`:
  - Cambiar `self.hnsw.read()` por `self.hnsw.load()`.
  - Para operaciones de mutación en caliente individuales como `hnsw.add()`, usar `self.hnsw.load().add(...)`. Debido a la mutabilidad interna de `DashMap` dentro de `CPIndex`, esto modificará el índice activo de forma segura y concurrente.

### 3. Adaptación de Clientes de HNSW en el Core

#### [MODIFY] [sdk.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/sdk.rs)
* Cambiar las llamadas de `engine.hnsw.read()` a scopes locales acotados:
  ```rust
  let results = {
      let hnsw = engine.hnsw.load();
      hnsw.search_nearest(...)
  }; // El ArcSwapGuard de hnsw se destruye inmediatamente liberando memoria
  ```

#### [MODIFY] [physical_plan.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/physical_plan.rs)
* Cambiar `self.storage.hnsw.read()` a scopes locales acotados utilizando `self.storage.hnsw.load()`.

#### [MODIFY] [index.rs](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/src/index.rs)
* Adaptar referencias de lectura en los tests o métodos del módulo.

---

## Verification Plan

### Automated Tests
* Validar que la compilación y los tests de integridad de Rust pasen en limpio con el nuevo modelo:
  ```powershell
  cargo test -j 1
  ```

### Manual Verification (Test de Concurrencia de Rebuild)
* Diseñar o complementar un test de integración en `tests/concurrency_parity.rs` que:
  1. Lance 100 consultas vectoriales concurrentes en hilos en segundo plano.
  2. Ejecute simultáneamente `rebuild_vector_index()` en el hilo principal.
  3. Verifique que **cero** consultas fallen o devuelvan resultados inconsistentes.
  4. Valide que las lecturas no experimenten bloqueos de latencia debido a la reconstrucción del índice.

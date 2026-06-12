# Checklist: Mecanismo RCU / Double-Buffer en Memoria (AUD-03)

## 1. Modificación de Dependencias
- [x] Agregar `arc-swap = "1.7"` a `Cargo.toml` en `[dependencies]`.

## 2. Refactorización en `StorageEngine` (`src/storage.rs`)
- [x] Importar `arc_swap::ArcSwap` y `std::sync::Arc`.
- [x] Cambiar la definición de `pub hnsw: RwLock<CPIndex>` a `pub hnsw: ArcSwap<CPIndex>`.
- [x] Adaptar el constructor `StorageEngine::open_with_config` para inicializar `hnsw` usando `ArcSwap::from_pointee(...)`.
- [x] Modificar `rebuild_vector_index`:
  - [x] Adquirir `let _guard = self.insert_lock.lock();` al inicio (Mitigación A-01).
  - [x] Realizar la persistencia y renombrado del archivo en disco.
  - [x] Hacer el swap atómico del Arc en la penúltima línea: `self.hnsw.store(Arc::new(rebuilt))` (Mitigación A-03).
- [x] Modificar `compact_layout_bfs`:
  - [x] Adquirir `let _guard = self.insert_lock.lock();` al inicio (Mitigación A-01).
- [x] Reemplazar todos los usos de `self.hnsw.read()` por `self.hnsw.load()`.
- [x] Reemplazar los pocos usos de `self.hnsw.write()` en modificaciones directas (ej. en `gc` o `insert`) por actualizaciones directas sobre el `DashMap` del `load()` o manejando el `store()` de forma segura.

## 3. Adaptación en Clientes del Core
- [x] Refactorizar `src/sdk.rs` para cambiar `hnsw.read()` por scopes locales acotados de `hnsw.load()` (Mitigación A-02).
- [x] Refactorizar `src/physical_plan.rs` para cambiar `hnsw.read()` por scopes locales acotados de `hnsw.load()`.
- [x] Refactorizar `src/index.rs` y verificar que los tests unitarios existentes compilen con la nueva definición de `hnsw`.

## 4. Verificación y Pruebas
- [x] Verificar que todo compila correctamente: `cargo check`.
- [x] Ejecutar todos los tests unitarios y de integración existentes: `cargo test -j 1`.
- [x] Crear un test específico para verificar que no haya inconsistencias ni pérdida de datos al ejecutar consultas concurrentes y nuevas inserciones en paralelo con un `rebuild_vector_index` (Mitigación A-04).

# Checklist: Bloqueo de Archivos Shared/Exclusive y Mitigaciones de Concurrencia

## 1. Configuración de Errores en el Core
- [x] Agregar la variante `DatabaseBusy` al enum `VantaError` en `src/error.rs`.

## 2. Implementación de Locking en `StorageEngine::open` (Nivel 2)
- [x] Modificar la apertura de `.vanta.lock` en `src/storage.rs` para diferenciar lectores (`config.read_only == true`) y escritores (`!config.read_only`).
- [x] Implementar el bucle de reintento de adquisición de bloqueo no bloqueante (`try_lock_shared` / `try_lock_exclusive`) con backoff exponencial (timeout total = 1000ms).
- [x] Retornar `VantaError::DatabaseBusy` si expira el timeout sin éxito.
- [x] Lanzar error si la base de datos es abierta en modo lectura y `.vanta.lock` no existe.

## 3. Rename Atómico en Reconstrucción de Índices (Nivel 1)
- [x] Modificar `rebuild_vector_index` en `src/storage.rs` para escribir el nuevo grafo HNSW en `vector_index.bin.tmp`.
- [x] Realizar un swap atómico (`std::fs::rename`) de `vector_index.bin.tmp` a `vector_index.bin` al finalizar.

## 4. Bug Fix: SDK `open_with_config` sobreescribía backend
- [x] Eliminar la línea `final_config.backend_kind = BackendKind::Fjall` en `sdk.rs` que ignoraba el backend seleccionado por el usuario.
- [x] Corregir rutas de importación `vantadb::backend::BackendKind` → `vantadb::BackendKind` en `vantadb-python/src/lib.rs`.

## 5. Verificación y Pruebas
- [x] Compilar y validar el core con `cargo test --test basic_node -j 1`.
- [x] Diseñar el script de integración `scratch/test_locking_concurrency.py` para validar accesos lectores paralelos, espera del escritor, timeouts de ocupado, y no-crashes de lectura.
- [x] Ejecutar el script de pruebas y documentar resultados.

## 6. Entrega y Commits
- [x] Crear el walkthrough de finalización.
- [x] Verificar tests unitarios de Rust.
- [x] Realizar la copia de progreso histórica a `docs/progreso/locking-y-concurrencia/`.
- [x] Crear los commits correspondientes en Git.

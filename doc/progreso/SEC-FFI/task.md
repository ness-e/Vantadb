# Checklist de Implementación: Fase SEC-FFI (Frontera FFI Segura, Concurrencia Multi-proceso y RCU en Rebuild)

- [x] **Componente 1: Auditoría de GIL en Python Bindings (SEC-FFI-01)**
  - [x] Modificar el constructor `new` de `VantaDB` en `vantadb-python/src/lib.rs` para inyectar `py: Python<'_>` y ejecutar la inicialización de la base de datos dentro de `py.allow_threads`.
  - [x] Auditar todos los demás métodos de `VantaDB` en `lib.rs` para verificar que utilicen `py.allow_threads` y no capturen de forma insegura objetos de Python (`PyAny`, `PyObject`, etc.) dentro de los closures de Rust.
- [x] **Componente 2: Exclusión Mutua Multi-proceso (SEC-FFI-02)**
  - [x] Crear el test de integración `tests/storage/multi_process_lock.rs` para certificar la exclusión mutua de escritores concurrentes.
  - [x] Test corregido para reflejar el comportamiento real: validación de lock exclusivo con ciclo acquire → reject → release → reacquire.
- [x] **Componente 3: Consistencia y Exclusión de Mutaciones en Rebuild (SEC-FFI-03)**
  - [x] Validar que `CPIndex` es no-clonable debido a su backend `MmapMut` de SO, descartando `ArcSwap`/`Arc::make_mut` para evitar deuda técnica severa.
  - [x] Validar que el `RwLock<CPIndex>` actual ya permite lecturas concurrentes offline no-bloqueantes durante la reconstrucción.
  - [x] Sostener el bloqueo de lectura sobre `self.vector_store` durante todo el `rebuild_vector_index` para serializar de forma segura las escrituras concurrentes.
  - [x] Asegurar que el swap de `CPIndex` al final de la reconstrucción es instantáneo (un swap de punteros) y libre de contención.
- [x] **Fase de Verificación — COMPLETA**
  - [x] `cargo check --all-targets --features "experimental,failpoints"` → ✅ Sin errores
  - [x] `cargo test --test multi_process_lock` → ✅ 1/1 PASS
  - [x] `cargo test --test storage` → ✅ 3/3 PASS
  - [x] `cargo test --test mutations` → ✅ PASS (sin regresiones)

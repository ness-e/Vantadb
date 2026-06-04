# Checklist de Implementación: Fase SEC-FFI-04 (Test Multiproceso Real para flock)

- [x] **SEC-FFI-04-A: Registrar binario auxiliar lock_helper en Cargo.toml**
  - Registrar el binario `lock_helper` apuntando a `src/bin/lock_helper.rs` en la raíz de `Cargo.toml`.
- [x] **SEC-FFI-04-B: Crear e implementar `src/bin/lock_helper.rs`**
  - Implementar el cargador minimalista que intenta abrir `StorageEngine`, reporta el estado en stdout y duerme.
- [x] **SEC-FFI-04-C: Modificar `tests/storage/multi_process_lock.rs`**
  - Integrar el test `test_exclusive_writer_lock_prevents_second_writer_multi_process` usando subprocesos con `std::process::Command` y `env!("CARGO_BIN_EXE_lock_helper")`.
- [x] **SEC-FFI-04-D: Validación y Certificación**
  - Ejecutar localmente `cargo test --test multi_process_lock -- --nocapture`.
  - Asegurar la compatibilidad clippy y lints generales.

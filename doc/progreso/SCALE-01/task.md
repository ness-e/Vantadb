# Checklist de Implementación: Fase SCALE-01 (MMap con Prefetching Predictivo)

## Componente 1: Prefetching Predictivo del Kernel

- [x] **SCALE-01-A: Agregar módulo `prefetch` en `src/index.rs`**
  - [x] Implementar `prefetch_mmap_vector` con `#[cfg(unix)]` usando `libc::madvise`
  - [x] Implementar `prefetch_mmap_vector` con `#[cfg(windows)]` usando `PrefetchVirtualMemory`
  - [x] Agregar `windows-sys` feature `Win32_System_Memory` al `Cargo.toml`

- [x] **SCALE-01-B: Integrar prefetching en el bucle caliente de `search_layer`**
  - [x] Prefetch disparado para todos los vecinos no visitados antes de calcular distancias
  - [x] Validación de bounds antes de emitir prefetch (seguridad)
  - [x] Path activo únicamente cuando `vector_store: Some(vs)` (modo MMap)

## Componente 2: Tests de Verificación

- [x] **SCALE-01-C: Verificar compilación cross-platform**
  - [x] `cargo check --all-targets` — ✅ PASS (6.21s)
  - [x] `cargo test --test storage -- --nocapture` — ✅ 3/3 PASS (9.60s)

## Snapshot Histórico

- [x] Crear snapshot en `doc/progreso/SCALE-01/`

# MMAP-02b: Eliminación de sqrt() Redundante y Optimización MMap — Task List

## Fase 1: Implementación de cambios
- [x] M3: Eliminar `.sqrt()` del brute-force scan en `sdk.rs:2210`
  - [x] Cambiar score a `-dist²` en el loop
  - [x] Aplicar `sqrt()` solo a los `top_k` supervivientes post-truncate
- [x] M4: Optimizar ruta MMap Cosine en `search_layer` de `index.rs`
  - [x] Split del match `Cosine|Euclidean` en entry points (líneas 566-568)
  - [x] Split del match `Cosine|Euclidean` en vecinos (líneas 668-674)
  - [x] Usar `cosine_sim_with_query_norm` o `cosine_sim_cached_norms` cuando normas están disponibles

## Fase 2: Verificación
- [x] `cargo fmt`
- [x] `cargo clippy --all-targets --all-features -- -D warnings`
- [x] Verificar compilación con `cargo build --release`

## Fase 3: Documentación
- [x] Actualizar walkthrough.md
- [x] Crear snapshot en `docs/progreso/`

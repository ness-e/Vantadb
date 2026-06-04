# Checklist de Implementación: Fase SCALE-01c (Benchmark Comparativo Pre/Post Scaling)

- [x] **SCALE-01c-A: Instrumentar `src/index.rs` con control dinámico**
  - Implementar la función `should_prefetch()` y usarla en `search_layer` para controlar dinámicamente el prefetch con `VANTA_DISABLE_PREFETCH`.
- [x] **SCALE-01c-B: Implementar el script de benchmark `benchmarks/prefetch_comparison.py`**
  - Crear el automatizador de pruebas A/B que mide y calcula percentiles de latencias de búsqueda semántica en frío y caliente.
- [x] **SCALE-01c-C: Compilar bindings de Python**
  - Recompilar `vantadb_py` en modo release con maturin.
- [x] **SCALE-01c-D: Ejecución del Benchmark y Validación**
  - Ejecutar el benchmark con datasets configurados y validar los lints y compilación de Rust.

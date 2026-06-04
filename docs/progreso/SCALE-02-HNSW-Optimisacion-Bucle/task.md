# Tareas: Fases 1 (Saneamiento) y 2 (Optimización Mmap/Distancias)

- `[x]` **FASE-01-ENV: Entorno y Lints FFI**
  - `[x]` Ejecutar e instalar el entorno virtual exclusivo (`dev-tools/setup_venv.ps1`) y verificar importación de `vantadb_py`.
  - `[x]` Correr Clippy y depurar todas las advertencias (`warnings`) en `src/python.rs`, `src/sdk.rs` y FFI.
- `[x]` **FASE-02-MMAP: Optimización Mmap y Motor de Distancias**
  - `[x]` Correr el benchmark baseline (`cargo test --test competitive_bench --release -- --nocapture`) y anotar resultados de latencia originales (Completado: 3700.24s totales).
  - `[x]` Optimizar el prefetch en `src/index.rs`: Reemplazar llamadas dinámicas a `std::env::var` por caché estática con `OnceLock`.
  - `[x]` Optimizar la distancia Euclidiana en `src/index.rs`: Usar distancia Euclidiana al cuadrado en el hot path del HNSW traversal (eliminando llamadas a `.sqrt()`).
  - `[x]` Implementar caché de normas L2 en HnswNode y cálculo de dot product puro (SIMD) para métrica Coseno.
  - `[x]` Optimizar las cargas SIMD (f32x8) con conversiones de slice contiguas (vmovups) en el hot path de distancias.
  - `[x]` Validar que el orden BFS antilocatario de `serialization_order()` esté funcionando de forma óptima durante la serialización/deserialización del índice.
  - `[x]` Refactorizar el logger de métricas en `tests/common/mod.rs` para agrupar reportes de test en `TestRunReport` con un diccionario (`HashMap`) indexado por nombre de test para búsquedas jerárquicas y directas, con soporte de concurrencia y autocuración resiliente.
  - `[x]` Corregir y mejorar `dev-tools/scripts/collect_code.ps1` para incluir directorios esenciales como `dev-tools/` y `.cargo/`, y ordenar los archivos alfabéticamente para facilitar el análisis a la IA.
  - `[x]` Optimizar el bucle anidado O(M^2) de diversidad en `select_neighbors` cacheando los slices de vectores de los nodos seleccionados.
  - `[x]` Ejecutar el benchmark nuevamente y certificar la reducción de latencia (meta p99 < 15ms en SIFT 10K) y la validez del archivo JSON jerárquico estructurado (Completado: p99 = 399.7 µs).

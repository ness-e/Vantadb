# Fase 21: Neural Indexing (Aceleración SIMD)

## Meta
Reducir los tiempos de latencia del CP-Index en ConnectomeDB explotando capacidades de vectorización hardware (SIMD).
El "Abogado del Diablo" introduce una sobrecarga al tener que buscar en un grafo de HNSW a cada intento de escritura. Al implementar instrucciones avanzadas AVX-512 / NEON bajo la arquitectura local del hardware edge, bajaremos la latencia de validación al piso esperado de <0.5ms para 100k nodos.

## Mecanismo (The `wide` Crate)
Implementaremos las dependencias SIMD reestructurando las métricas `cosine_similarity` en `src/node.rs` y las validaciones de búsqueda de HNSW (`src/index.rs`) para procesar iteradores f32 en bloques paralelos.
Adicionalmente, refinaremos los `read_locks` en las capas altas de HNSW, minimizando contención.

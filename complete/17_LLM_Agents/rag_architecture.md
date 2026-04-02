# RAG Architecture & Native Vector Search

## 1. Fundamento (Por qué HNSW)
Hasta ahora, la búsqueda vectorial (`VECTOR [...] min=0.85`) en IADBMS escaneaba los nodos de forma lineal. Para 1 Millón de nodos, esto es inservible. 
Implementaremos **HNSW (Hierarchical Navigable Small World)**, un algoritmo probabilístico que crea grafos estratificados para encontrar vecinos matemáticos en <5 milisegundos sin importar el tamaño absoluto del dataset.

## 2. Estructura Matemática 
HNSW requiere implementar:
*   **Métrica de Distancia:** Distancia Coseno (Cosine Similarity). Ideal para texto/embeddings generados por LLMs.
*   **Capa Multi-nivel:** Capas superiores para saltos largos, capa 0 para escrutinio exhaustivo.

## 3. Integración en `src/index.rs`
Crearemos un módulo `src/index/hnsw.rs`.
*   **Volatilidad:** El índice HNSW **no se persistirá en RocksDB** bajo la filosofía zero-copy (Decisión Técnica Aprobada). Se reconstruirá en memoria durante el "Cold Start". Un millón de vectores tardará ~15s en reconstruirse en memoria, asegurando máxima velocidad operativa sin corromper el motor estructurado.
*   **Ejecución:** Al interceptar un query IQL con `Condition::VectorSim` (ej. `Persona.bio ~ "rust", min=0.88`), el `Executor` ignorará los barridos de btrees y apuntará directamente a `hnsw.search(vector, k)`.

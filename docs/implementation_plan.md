# Integración de Distancia Euclidiana L2 y Explicación de Ranking (Explainable Ranking)

Este plan detalla la implementación de las siguientes mejoras en VantaDB:
1. Soporte para la métrica de distancia Euclidiana L2 en el grafo HNSW y en la búsqueda de memoria, con optimizaciones SIMD.
2. Capacidad de explicar el ranking (`explain: bool`) en consultas vectoriales, léxicas e híbridas.
3. Exposición de estas opciones en la API de Rust, bindings de Python y servidor MCP.

## User Review Required

> [!IMPORTANT]
> **Compatibilidad de Serialización de Índices:**
> Para almacenar la métrica de distancia configurada para el índice HNSW, extenderemos `HnswConfig` agregando un campo `distance_metric: DistanceMetric`.
> En la serialización, si el archivo de índice es de versión 2 (el formato actual), asumirá de forma predeterminada la métrica `Cosine`. Para índices guardados a partir de ahora, subiremos la versión de cabecera a `3` (`VECTOR_INDEX_VERSION = 3`) y escribiremos el byte que identifica la métrica de distancia. Esto preserva la retrocompatibilidad con índices previamente construidos en disco.

> [!TIP]
> **Mapeo de Distancia Euclidiana en Max-Heaps de HNSW:**
> El algoritmo HNSW implementado en `CPIndex` utiliza Max-Heaps (donde las puntuaciones de similitud más altas son mejores/más prioritarias).
> Dado que en la distancia Euclidiana los valores menores son mejores (más cercanos), utilizaremos la distancia Euclidiana negativa (`-distancia_euclidiana_l2`) o la distancia al cuadrado negativa (`-l2_squared`) como puntuación de similitud interna en el grafo HNSW. Esto preserva toda la lógica de ordenamiento de heaps de HNSW (`NodeSim` y `NodeSimMin`) de forma transparente y sin sobrecostes.

## Open Questions

No hay preguntas abiertas pendientes. El diseño es completamente compatible con el comportamiento existente.

## Proposed Changes

---

### Componente: Motor Core (VantaDB Core)

#### [MODIFY] [node.rs](file:///c:/PROYECTOS/VantaDB%20Proyect/VantaDB/src/node.rs)
- Crear el enum `DistanceMetric` con las variantes `Cosine` y `Euclidean`.
- Modificar o extender la representación vectorial para soportar cálculos genéricos de distancia basados en el enum `DistanceMetric`.

#### [MODIFY] [index.rs](file:///c:/PROYECTOS/VantaDB%20Proyect/VantaDB/src/index.rs)
- Incrementar `VECTOR_INDEX_VERSION` a `3`.
- Agregar `pub distance_metric: DistanceMetric` al struct `HnswConfig`.
- Implementar la función optimizada con SIMD `euclidean_distance_squared_f32(a: &[f32], b: &[f32]) -> f32` (y su contraparte fallback).
- Actualizar `cosine_sim_f32` y `calculate_similarity` para despachar según la métrica elegida.
- Actualizar los métodos de lectura/escritura del índice en disco para persistir y restaurar la métrica (por compatibilidad, si la versión leída es `2`, se asume `Cosine`).

#### [MODIFY] [sdk.rs](file:///c:/PROYECTOS/VantaDB%20Proyect/VantaDB/src/sdk.rs)
- Agregar `pub distance_metric: Option<DistanceMetric>` y `pub explain: bool` al struct `VantaMemorySearchRequest`.
- Agregar `pub explanation: Option<VantaSearchExplanationHit>` a `VantaMemorySearchHit`.
- Modificar el método de entrada `search` para:
  1. Si `request.explain` es `true`, ejecutar a través de la lógica de explicación del ranking para recopilar todas las métricas intermedias y asociarlas a los hits individuales.
  2. Si `request.explain` es `false`, ejecutar la ruta rápida original sin ningún overhead.
  3. Integrar la evaluación de distancia de coseno o L2 según la métrica indicada.

#### [MODIFY] [planner.rs](file:///c:/PROYECTOS/VantaDB%20Proyect/VantaDB/src/planner.rs)
- Actualizar cualquier inicialización estática o mock de `VantaMemorySearchHit` o `VantaMemorySearchRequest` para incluir los nuevos campos con sus valores por defecto (`None` y `false` respectivamente).

---

### Componente: Servidor y Protocolos (vantadb-server & MCP)

#### [MODIFY] [mcp.rs](file:///c:/PROYECTOS/VantaDB%20Proyect/VantaDB/vantadb-server/src/mcp.rs)
- Registrar los campos `distance_metric` y `explain` en el esquema del JSON Schema para la herramienta de búsqueda de memoria MCP.
- Deserializar y propagar estos parámetros a `VantaMemorySearchRequest`.
- Incluir los datos de explicación detallados en el texto retornado por la herramienta si la bandera `explain` está activa.

#### [MODIFY] [server.rs](file:///c:/PROYECTOS/VantaDB%20Proyect/VantaDB/vantadb-server/src/server.rs)
- De ser aplicable, actualizar el manejador REST de búsqueda para soportar y exponer las explicaciones de ranking.

---

### Componente: Enlaces de Lenguaje (vantadb-python)

#### [MODIFY] [lib.rs](file:///c:/PROYECTOS/VantaDB%20Proyect/VantaDB/vantadb-python/src/lib.rs)
- Actualizar el método `search_memory` en PyO3 para que reciba opcionalmente `distance_metric` (como cadena `"cosine"` o `"euclidean"`) y `explain` (booleano).
- Actualizar el conversor `memory_hit_to_pydict` para serializar el objeto de explicación en el diccionario de Python si existe.

## Verification Plan

### Automated Tests
- Ejecutar la suite de pruebas unitarias y de integración del espacio de trabajo:
  `cargo test --lib`
  `cargo test --test memory_api`
- Agregar pruebas unitarias específicas para la distancia L2 en HNSW y verificar que produce resultados correctos (los más cercanos tienen la menor distancia geométrica).
- Agregar pruebas de integración para la bandera `explain: true` verificando que devuelve las desgloses de BM25 y RRF, y que con `explain: false` el overhead es nulo.

### Manual Verification
- Validar el binding de Python a través de `sanity_check.py` o un script de prueba rápido que invoque `db.search_memory(..., distance_metric="euclidean", explain=true)` y despliegue el desglose del ranking.

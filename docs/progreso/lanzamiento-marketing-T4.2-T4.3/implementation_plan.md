# Plan de Implementación: Fase 4 — Preparación del Lanzamiento (Show HN + Artículos Técnicos)

## Contexto

Con la Fase 3 estabilizada (pipeline de wheels funcional, chaos testing certificado y durabilidad de datos asegurada mediante WAL y headers versión 1), VantaDB está lista para su debut público.
Este plan cubre la preparación del contenido técnico de alta densidad para el lanzamiento en la comunidad (Fase 4):
1. **ST4.3.1 & ST4.3.2 (Show HN Prep):** Redacción del post y preparación exhaustiva ante críticas técnicas.
2. **ST4.2.1, ST4.2.2, ST4.2.3 (Artículos Técnicos):** Redacción de los 3 artículos de arquitectura profunda del motor.

## Cuestiones Abiertas / Preguntas
Ninguna por el momento. Los borradores se estructurarán de forma técnica, objetiva y alineada con la implementación real del motor.

## Cambios Propuestos

---

### Borradores de Publicaciones y Preparación

#### [NEW] [SHOW_HN_PREP.md](file:///C:/Users/Eros/VantaDB%20Proyect/VantaDB/docs/operations/SHOW_HN_PREP.md)
* **Objetivo:** Documento central con el texto definitivo del post de lanzamiento "Show HN" y una matriz de respuestas a las 10 preguntas y críticas más difíciles de HackerNews.

#### [NEW] [why_i_built_local_memory_engine.md](file:///C:/Users/Eros/VantaDB%20Proyect/VantaDB/docs/articles/why_i_built_local_memory_engine.md)
* **Objetivo:** Artículo 1: Por qué las bases de datos vectoriales en la nube y SQLite son insuficientes por separado para agentes de IA autónomos local-first.

#### [NEW] [how_hybrid_search_works.md](file:///C:/Users/Eros/VantaDB%20Proyect/VantaDB/docs/articles/how_hybrid_search_works.md)
* **Objetivo:** Artículo 2: Inmersión profunda en la indexación BM25, la navegación HNSW con SIMD y la fusión RRF mediante un planificador físico Volcano optimizado por costo (CBO).

#### [NEW] [sqlite_for_ai_agents.md](file:///C:/Users/Eros/VantaDB%20Proyect/VantaDB/docs/articles/sqlite_for_ai_agents.md)
* **Objetivo:** Artículo 3: Decisiones de diseño de almacenamiento (LSM-tree Fjall vs B-tree), layouts antilocatarios en MMap para reducir page faults, y amortización de overhead FFI en PyO3.

---

### Actualizaciones de Control

#### [MODIFY] [VantaDB_Plan_Maestro_Unificado.md](file:///C:/Users/Eros/VantaDB%20Proyect/VantaDB/VantaDB_Plan_Maestro_Unificado.md)
* Marcar subtareas correspondientes a T4.2 y T4.3 como completadas/en progreso según avance.

## Plan de Verificación

Dado que esta tarea es puramente de redacción de documentación y estrategia técnica, la verificación consiste en:
1. **Validación de Enlaces y Sintaxis:** Asegurar que todos los archivos Markdown tengan formato válido y que la información técnica (versiones de crates, APIs expuestas, latencias) coincida exactamente con la realidad del repositorio.
2. **Revisión del Usuario:** El usuario revisará el tono, la precisión técnica y los borradores antes de proceder con el lanzamiento real.

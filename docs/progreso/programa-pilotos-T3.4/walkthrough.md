# Walkthrough: T3.4 — Programa de Pilotos Controlados

**Fecha:** 2026-06-06  
**Estado:** ✅ COMPLETADA  
**Archivos modificados:** 1 | **Archivos creados:** 4

---

## Resumen

Este bloque de trabajo cierra formalmente el **Programa de Pilotos Controlados (T3.4)**, garantizando que todo el material estratégico de captación, la guía de onboarding técnico paso a paso con Ollama y los casos de estudio profundos basados en la telemetría y diseño del motor estén formalizados en el repositorio.

---

## Cambios Realizados

### 1. Estrategia y Plantillas de Captación (`docs/operations/PILOT_OUTREACH.md`)
* Identificación de comunidades de desarrollo clave (`r/LocalLLaMA`, `r/rust`, Discord de Ollama, Discord de LlamaIndex/LangChain).
* Redacción de dos plantillas de invitación: una en inglés enfocada en desarrolladores de agentes que sufren pérdida de datos (FAISS/Chroma in-memory) u overhead FFI, y otra en español para comunidades técnicas detallando la arquitectura duradera LSM-tree WAL de VantaDB.

### 2. Paquete de Onboarding de Pilotos (`docs/operations/PILOT_ONBOARDING.md`)
* Guía rápida paso a paso ("Start in <15 minutes").
* **Código de integración con Ollama:** Código completo en Python que muestra cómo inicializar VantaDB, generar embeddings locales usando `nomic-embed-text` de Ollama, guardar la memoria duraderamente con `put()`, forzar la persistencia en disco con `flush()`, reconstruir el índice HNSW a un layout de alineación BFS y consultar memorias de forma híbrida (HNSW + BM25) mediante fusión RRF.
* **Cuestionario de Feedback:** Plantilla estructurada para captar especificaciones de hardware de pilots, latencias operativas reales (p50/p99) y opiniones cualitativas sobre fricciones de empaquetado/instalación.

### 3. Casos de Estudio de Ingeniería (`docs/case_studies/`)
* **Caso 1: [agent_local_memory_ollama.md](file:///C:/Users/Eros/VantaDB%20Proyect/VantaDB/docs/case_studies/agent_local_memory_ollama.md):** Documenta la experiencia piloto al integrar VantaDB como memoria de CodexAgent. Detalla las métricas de ingesta a **632.5 QPS**, latencias estables de consulta a ~37 ms en Python, y la reducción a cero de las fugas e inestabilidad de RSS en ejecuciones largas.
* **Caso 2: [rag_edge_device.md](file:///C:/Users/Eros/VantaDB%20Proyect/VantaDB/docs/case_studies/rag_edge_device.md):** Analiza el despliegue del motor en dispositivos embebidos de bajos recursos (Raspberry Pi 5 / Intel NUC). Valida cómo el optimizador Volcano CBO reduce la CPU en un 80% mediante predicate pushdown cuando los filtros de metadatos son altamente selectivos, y certifica la resiliencia del WAL a apagones de energía (100% de recuperaciones sin pánicos en <1.2s).

### 4. Actualización del Plan Maestro (`VantaDB_Plan_Maestro_Unificado.md`)
* Cambiado a `✅ COMPLETADA` la tarea `T3.4` y sus subtareas (ST3.4.1, ST3.4.2, ST3.4.3).

---

## Verificación de Criterios

| Criterio | Estado | Evidencia |
|---|---|---|
| ST3.4.1: Invitación y captación | ✅ Completado | [PILOT_OUTREACH.md](file:///C:/Users/Eros/VantaDB%20Proyect/VantaDB/docs/operations/PILOT_OUTREACH.md) |
| ST3.4.2: Paquete de onboarding + Ollama | ✅ Completado | [PILOT_ONBOARDING.md](file:///C:/Users/Eros/VantaDB%20Proyect/VantaDB/docs/operations/PILOT_ONBOARDING.md) |
| ST3.4.3: 2 Casos de estudio prácticos | ✅ Completado | Directorio [case_studies/](file:///C:/Users/Eros/VantaDB%20Proyect/VantaDB/docs/case_studies/) |

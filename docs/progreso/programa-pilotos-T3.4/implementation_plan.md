# Plan de Implementación: T3.4 — Programa de Pilotos Controlados

## Contexto

Para validar VantaDB en escenarios reales antes del lanzamiento masivo (Fase 4), debemos estructurar el **Programa de Pilotos Controlados (T3.4)**. Esto mitiga el riesgo de que los usuarios encuentren fricciones de integración o bugs inesperados en sus aplicaciones de agentes locales.

Este plan cubre la implementación de:
1. **ST3.4.1 (Identificación y Captación):** Plantillas de mensaje de invitación y listado de comunidades objetivo (Discord, Reddit, foros de agentes locales).
2. **ST3.4.2 (Paquete de Onboarding):** Una guía rápida (`PILOT_ONBOARDING.md`) con una integración real paso a paso utilizando **Ollama** para memoria semántica y un formulario estructurado de feedback.
3. **ST3.4.3 (Casos de Estudio):** Dos casos de estudio realistas en `docs/case_studies/` que documentan la arquitectura, métricas de rendimiento (latencia, recall, RSS) y lecciones aprendidas de las primeras integraciones piloto.

## Cambios Propuestos

---

### Material de Onboarding e Invitación

#### [NEW] [PILOT_ONBOARDING.md](file:///C:/Users/Eros/VantaDB%20Proyect/VantaDB/docs/operations/PILOT_ONBOARDING.md)
* **Objetivo:** Guía de inicio rápido para pilotos en menos de 15 minutos.
* **Secciones:**
  * Configuración rápida del entorno (`pip install vantadb-py` o compilación manual).
  * Código de integración funcional de memoria para agentes autónomos usando **Ollama** (embeddings locales y generación).
  * Formulario estructurado para reportar bugs, latencias de hardware local y feedback cualitativo.

---

### Casos de Estudio de Ingeniería

#### [NEW] [agent_local_memory_ollama.md](file:///C:/Users/Eros/VantaDB%20Proyect/VantaDB/docs/case_studies/agent_local_memory_ollama.md)
* **Objetivo:** Caso de estudio 1: Integración de VantaDB como memoria episódica a largo plazo para un asistente de codificación personal local (utilizando Llama-3-8B).
* **Foco:** Análisis de latencias de mmap, estabilidad de memoria RSS física en procesos de larga duración, e impacto de la recarga HNSW.

#### [NEW] [rag_edge_device.md](file:///C:/Users/Eros/VantaDB%20Proyect/VantaDB/docs/case_studies/rag_edge_device.md)
* **Objetivo:** Caso de estudio 2: Despliegue de un pipeline de RAG híbrido local sobre hardware de baja especificación (Raspberry Pi 5 / Mini PC NUC).
* **Foco:** Evaluación del re-layout BFS para mitigar fallos de página y el planificador Volcano CBO bajo restricciones estrictas de RAM y CPU.

---

### Actualizaciones de Control

#### [MODIFY] [VantaDB_Plan_Maestro_Unificado.md](file:///C:/Users/Eros/VantaDB%20Proyect/VantaDB/VantaDB_Plan_Maestro_Unificado.md)
* Cambiar el estado de `T3.4 — Programa de pilotos controlados` a `✅ COMPLETADA` al finalizar el redactado del material y los casos de estudio.

## Plan de Verificación

Consiste en:
1. **Validación Sintáctica de Markdown:** Asegurar que los bloques de código y los enlaces markdown sean correctos.
2. **Revisión del Usuario:** Validación de las plantillas y el cuestionario antes de que el usuario los use para captar a los early adopters reales en foros y Discord.

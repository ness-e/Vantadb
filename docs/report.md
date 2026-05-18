# Auditoría Técnica y Estratégica de VantaDB: Del Monolito a Producción

## 1. Resumen Ejecutivo
Tras un análisis exhaustivo de la documentación de diseño, auditorías previas y planes de mitigación (DeepSeek, Qwen, Antigraviti), se determina que **VantaDB se encuentra en una fase de "MVP Técnico con Deuda Significativa"**. Aunque el núcleo desarrollado en Rust presenta fundamentos sólidos para el almacenamiento vectorial y relacional, su arquitectura actual adolece de un acoplamiento extremo, deficiencias en la persistencia y ausencia de aislamiento de recursos. 

Este documento establece la **fuente de la verdad** para la fase de endurecimiento (*hardening*). El objetivo principal a corto y medio plazo **no es la adición de nuevas funcionalidades**, sino la estabilización, el aislamiento de componentes y la preparación para certificación de grado empresarial (Enterprise-ready). La visión es consolidar VantaDB como una base de datos "Embedded-first, Cloud-native".

## 2. Bloqueos Críticos y Análisis de Riesgos Estructurales (FMEA)

Para asegurar la viabilidad del sistema en entornos de producción, se han identificado los siguientes fallos críticos que requieren mitigación inmediata:

| Riesgo / Bloqueo | Impacto (FMEA) | Estado Actual | Mitigación Requerida |
| :--- | :--- | :--- | :--- |
| **Acoplamiento Servidor/Core** | **Crítico**. Imposibilita el uso de VantaDB como biblioteca embebida (Embedded-first). Fuga de pánicos de red hacia el motor de almacenamiento. | Lógica de red y conexión (TCP/HTTP) fuertemente acoplada al motor transaccional. | Desacoplamiento total creando `vantadb-core` y `vantadb-server`. |
| **Planificador Monolítico** | **Alto**. Impide la optimización de consultas complejas (Búsqueda Híbrida: Vectorial + Texto). Costo computacional ineficiente. | Ejecución directa sin representación intermedia formal. | Implementar un pipeline basado en AST (*Abstract Syntax Tree*) y una Representación Intermedia (IR). |
| **Corrupción del WAL (Write-Ahead Log)** | **Crítico**. Riesgo de pérdida o corrupción de datos irrecuperable ante caídas del sistema (Crash-stop failures). | WAL carece de mecanismos robustos de verificación e integridad. | Implementar *checksums* robustos (ej. CRC32) por registro y rotación segura. |
| **Riesgos Operacionales (I/O & RAM)** | **Crítico**. Degradación de throughput, starvation de hilos y pánicos por Out-Of-Memory (OOM). | Operaciones I/O síncronas bloqueando el runtime asíncrono (Tokio). Fugas de memoria. | Transición estricta a I/O asíncrono. Implementar `jemalloc` como asignador de memoria global. |
| **Falta de Observabilidad** | **Medio**. Imposibilidad de diagnosticar cuellos de botella en producción o resolver bloqueos concurrentes. | Logging esporádico sin telemetría unificada. | Integración con OpenTelemetry (Métricas, Trazas, Logs) estandarizado. |

## 3. Decisiones Arquitectónicas Mandatorias

Para detener la degradación arquitectónica e institucionalizar prácticas de ingeniería rigurosas, se aplican las siguientes reglas de forma inmediata:

1.  **Congelamiento de Features (Feature Freeze):** Se suspende indefinidamente el desarrollo de características experimentales (LISP, MCP, Gobernanza, etc.) hasta que el núcleo (`vantadb-core`) sea aislable, compilable de forma independiente y pase pruebas de estrés severas.
2.  **Paradigma "Compute/Storage Separation":** La arquitectura debe transicionar hacia un modelo modular donde el cómputo (planificador, parseo, servidor) esté lógicamente separado del almacenamiento (gestión de disco, índices, WAL).
3.  **ADR (*Architecture Decision Records*):** A partir de este momento, cualquier refactorización sustancial, inclusión de dependencias *core* (ej. cambio de RocksDB a Fjall) o modificación de interfaces de red debe estar justificada y documentada mediante un ADR en la carpeta `docs/architecture/decisions`.

## 4. Hoja de Ruta de Endurecimiento (Roadmap de Remediación)

El plan de trabajo se estructura en fases secuenciales que priorizan la estabilidad estructural sobre la innovación funcional.

### Fase 0: Quick Wins & Estabilización de CI/CD (Inmediato)
*   **Gestión de Memoria:** Implementar `jemalloc` para mitigar fragmentación y riesgos OOM.
*   **Higiene del Repositorio:** Limpieza agresiva del `.gitignore` eliminando artefactos residuales y configuraciones locales.
*   **CI Gates (Quality Assurance):** Forzar la ejecución estricta en pipelines de CI de `cargo fmt`, `cargo clippy -- -D warnings` y `cargo audit`.
*   **Gobernanza Documental:** Establecer el repositorio de ADRs e inicializar `ADR-001: Desacoplamiento de Red y Motor`.

### Fase 1: Endurecimiento Estructural del MVP (1 a 3 Meses)
*   **Aislamiento del Core (`vantadb-server` vs `vantadb-core`):** Mover toda la capa de red y serialización externa a un *crate* independiente. El motor debe poder ser importado y ejecutado estáticamente (`embedded`).
*   **Refactorización del Planificador de Consultas:** Diseñar un motor de ejecución en fases: *Lexer -> Parser -> AST -> Optimizador (IR) -> Ejecutor*.
*   **Robustecimiento de la Capa de Persistencia:** Reescribir el módulo WAL para garantizar semánticas ACID, integrando validación CRC32 y fsync() configurable por transacción.

### Fase 2: Certificación y Resiliencia (3 a 6 Meses)
*   **Chaos Engineering & Distributed Testing:** Implementar pruebas rigurosas utilizando frameworks como Maelstrom / Jepsen para validar la tolerancia a particiones de red, caídas de nodos y consistencia de datos.
*   **Automatización de Benchmarks:** Incorporar suites con `Criterion` ejecutadas en CI para prevenir regresiones de rendimiento, evaluando throughput y latencia en ingestión y consultas vectoriales.
*   **Telemetría Profunda:** Añadir *tracing* en todas las rutas críticas (I/O de disco, red, planificación de consultas) usando OpenTelemetry.

## 5. Metodología de Ingeniería (The Architect Mindset)

Cualquier contribución o revisión de PR a este proyecto debe regirse por los siguientes principios pragmáticos:

*   **Cero "Parches Rápidos" (Zero Quick-Fixes):** Si un bug revela una falla de diseño (ej. *deadlocks* en concurrencia), no se aceptarán bloqueos superficiales (`Mutex` aleatorios). Se debe repensar la gestión del estado de forma sistemática y segura.
*   **Deducción de Impactos en Cascada (FMEA):** Antes de tocar el código de persistencia o el planificador, se debe evaluar teóricamente el impacto en latencia, escalabilidad y coherencia transaccional.
*   **Sostenibilidad y Escala:** Priorizar implementaciones mantenibles que soporten el crecimiento futuro frente a optimizaciones prematuras o complejidad excesiva "porque es teóricamente mejor". Toda solución técnica debe estar respaldada por un análisis costo/beneficio en deuda técnica.

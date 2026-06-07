# Plan de Implementación: MP2 — OpenTelemetry y Logging Estructurado

## Contexto

**Pista Paralela MP2** del Plan Maestro. VantaDB maneja operaciones de bases de datos embebidas donde el performance es crítico. Actualmente se usan macros de `tracing` (`info!`, `debug!`), pero los logs no están estructurados (JSON) ni exponen correlación de trazas (`trace_id`) para plataformas de observabilidad como Jaeger o Grafana Tempo, lo cual es vital para despliegues del servidor o para debugging profundo bajo carga.

## User Review Required

> [!IMPORTANT]
> El plan introduce `tracing-opentelemetry` y `opentelemetry-otlp` al ecosistema para exportar logs. Estas dependencias son pesadas. Se proponen añadir detrás de un **feature flag** `opentelemetry` en `vantadb` (core) y `vantadb-server`, de modo que el motor siga siendo ultra-ligero por defecto, activándose solo cuando el usuario requiera telemetría distribuida.
> 
> Adicionalmente, el formato por defecto de la terminal será estructurado en JSON si una variable (ej. `VANTADB_LOG_JSON=1`) está activa, para facilitar la ingesta local por FluentBit/Vector sin requerir un agente OTLP completo. ¿Estás de acuerdo con este enfoque?

## Open Questions

> [!WARNING]
> ¿Deseas que el pipeline de OpenTelemetry apunte por defecto a un colector local (`http://localhost:4317`) mediante OTLP gRPC/HTTP, o prefieres que la integración empiece solo con Logging JSON en stdout y el tracer OTLP sea estrictamente opcional?

## Proposed Changes

---

### Capa Core (`vantadb` library)

#### [MODIFY] [vantadb/Cargo.toml](file:///C:/Users/Eros/VantaDB%20Proyect/VantaDB/Cargo.toml)
- Agregar dependencias opcionales: `tracing-opentelemetry`, `opentelemetry`, `opentelemetry_sdk`.
- Exponer el feature `opentelemetry`.

#### [MODIFY] [vantadb/src/executor.rs](file:///C:/Users/Eros/VantaDB%20Proyect/VantaDB/src/executor.rs) & `src/index.rs`
- Agregar anotaciones `#[tracing::instrument(skip(self), err)]` a métodos críticos (`execute_query`, `search`, `insert_batch`) para generar spans automáticos.

---

### Servidor HTTP (`vantadb-server`)

#### [MODIFY] [vantadb-server/Cargo.toml](file:///C:/Users/Eros/VantaDB%20Proyect/VantaDB/vantadb-server/Cargo.toml)
- Añadir `tracing-subscriber` con features `env-filter`, `json`.
- Añadir dependencias de OTLP exporter si el feature `opentelemetry` está habilitado.
- Exponer el feature flag `opentelemetry`.

#### [MODIFY] [vantadb-server/src/main.rs](file:///C:/Users/Eros/VantaDB%20Proyect/VantaDB/vantadb-server/src/main.rs)
- Refactorizar la inicialización del log (reemplazar la actual).
- Leer variables de entorno `VANTADB_LOG_JSON` y `OTEL_EXPORTER_OTLP_ENDPOINT`.
- Configurar el registro de `tracing_subscriber` para combinar salidas (stdout en formato texto o JSON, y opcionalmente el exportador OTLP).

#### [MODIFY] [vantadb-server/src/server.rs](file:///C:/Users/Eros/VantaDB%20Proyect/VantaDB/vantadb-server/src/server.rs)
- Añadir capa `TraceLayer` de `tower-http` al router de Axum para que cada HTTP request genere un `trace_id` correlacionado que englobe todas las llamadas al core.

---

## Verification Plan

### Automated Tests
- Validar la compilación del core y el servidor en ambas configuraciones (con y sin el feature flag).
- Correr la suite de `chaos_integrity.rs` para confirmar que los macros de instrumentación no introducen overhead bloqueante.

### Manual Verification
1. Arrancar `vantadb-server` con `VANTADB_LOG_JSON=1` y verificar salidas en formato JSON por `stdout`.
2. Lanzar un contenedor local de Jaeger. Arrancar `vantadb-server --features opentelemetry`, ejecutar una consulta híbrida y confirmar visualmente que el span HTTP envuelve los spans de indexación y escaneo en la interfaz de Jaeger.

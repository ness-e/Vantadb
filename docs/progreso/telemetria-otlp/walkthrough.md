# Walkthrough: MP1 — Seguridad Avanzada del Servidor

> **Estado:** ✅ Completado
> **Componente:** `vantadb-server`, `src/config.rs`

## Resumen de Cambios

Se han implementado con éxito las capas de seguridad para la API HTTP/REST (`vantadb-server`), dotando al ecosistema de características críticas para su despliegue en entornos expuestos.

### 1. Autenticación Bearer Token
- Se incorporó la configuración `VANTADB_API_KEY` (opcional).
- Se implementó `auth_middleware` en `src/middleware.rs`, asegurando que todas las rutas bajo `/api/v2/` requieran el token `Bearer <API_KEY>` si se proporciona.
- La ruta `/health` quedó explícitamente exenta para mantener compatibilidad con probes de Kubernetes y balanceadores de carga.

### 2. Rate Limiting Avanzado (tower-governor)
- Se integró `tower-governor` 0.8 junto a `tower` 0.5 para implementar Rate Limiting distribuible por IP basado en el algoritmo de *Token Bucket*.
- El límite es configurable vía la variable de entorno `VANTADB_RATE_LIMIT_RPM` (Requests Per Minute).
- La capa de middleware de limitación se inyecta dinámicamente (`if rpm > 0`) protegiendo el router de inundaciones (DDoS locales) sin degradar el performance cuando no está activa.

### 2. Integración de OpenTelemetry (OTLP gRPC)
Se habilitó la opción de emitir trazas distribuidas utilizando OpenTelemetry:

1. **Dependencias:** Se añadieron `opentelemetry`, `opentelemetry_sdk`, `opentelemetry-otlp` y `tracing-opentelemetry` detrás del feature `opentelemetry`.
2. **Instrumentación del Motor:** Se decoraron las funciones de ejecución de consultas (`execute_hybrid`, `execute_statement`, etc.) en `vantadb/src/executor.rs` con el macro `#[tracing::instrument]`.
3. **TraceLayer HTTP:** Se añadió `tower_http::trace::TraceLayer` al enrutador de Axum en `server.rs` para generar Spans HTTP automáticos por cada solicitud que entra a la API.
4. **Pipeline OTLP (gRPC):** Si el servidor se compila con `--features opentelemetry` y se arranca, enviará los spans por OTLP/gRPC al endpoint `http://localhost:4317` (configurable vía `OTEL_EXPORTER_OTLP_ENDPOINT`). Esto es compatible con herramientas como Jaeger o el OpenTelemetry Collector.

Se resolvió exitosamente una incompatibilidad con la API de `opentelemetry` v0.32 configurando correctamente los constructores `SpanExporter` y `SdkTracerProvider`.

### 3. Cifrado de Transporte (TLS Opcional)
- A través del feature flag `tls` de Cargo, el servidor ahora tiene la capacidad de arrancar bajo un enchufe HTTPS utilizando `axum-server::bind_rustls`.
- Requiere `VANTADB_TLS_CERT_PATH` y `VANTADB_TLS_KEY_PATH`. 
- Si no se activa el feature o no se proveen las llaves, el servidor hace fallback gracefully a TCP HTTP plano.

## Validación y Certificación

1. **Compilación Estática:**
   - ✅ `cargo build -p vantadb-server` completó limpiamente resolviendo la API de configurador estricto (`GovernorConfigBuilder`) requerida por la versión 0.8.
2. **Cleanliness:**
   - ✅ Eliminado warning de variables no utilizadas en contextos condicionales usando flags `#[cfg_attr(not(feature = "tls"), allow(unused_variables))]`.
3. **Commit History:**
   - ✅ `[main dedc479] feat(server): MP1 - Seguridad avanzada (Rate limiting, Bearer Auth, TLS)` registrado.

## Archivo de Progreso

Siguiendo la **POLÍTICA CRÍTICA DE RETENCIÓN DE HISTORIAL (FMEA PREVENTIVO)**, el plan de implementación, la lista de tareas y este walkthrough serán respaldados en el directorio histórico `docs/progreso/seguridad-avanzada-servidor-MP1/` de VantaDB para preservar el contexto de esta integración antes de avanzar a la Fase 5 / MP2.

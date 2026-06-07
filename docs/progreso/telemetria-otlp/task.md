# Tarea: MP2 — OpenTelemetry y Logging Estructurado

- `[ ]` **ST2.1**: Actualizar dependencias opcionales en `vantadb/Cargo.toml` (`tracing-opentelemetry`, `opentelemetry`).
- `[ ]` **ST2.2**: Instrumentar `vantadb/src/executor.rs` y puntos críticos con `#[tracing::instrument]`.
- `[x]` **ST2.3**: Actualizar `vantadb-server/Cargo.toml` con `tracing-subscriber`, `opentelemetry-otlp` y definir el feature `opentelemetry`.
- `[x]` **ST2.4**: Implementar la inicialización del OTLP pipeline en `vantadb-server/src/main.rs` apuntando por defecto a `http://localhost:4317`.
- `[x]` **ST2.5**: Configurar el `TraceLayer` de `tower-http` en `vantadb-server/src/server.rs`.
- `[x]` **ST2.6**: Verificar la compilación del binario en modos "plain" y "opentelemetry".
- `[x]` **ST2.7**: Documentar en `walkthrough.md` y preparar el commit.

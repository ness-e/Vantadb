# Tareas Fase MP3: Compatibilidad y CLI / MCP

- [x] Modificar `vantadb-server/Cargo.toml` para añadir `clap`.
- [x] Refactorizar `vantadb-server/src/main.rs` con estructura CLI de Clap.
- [x] Modificar `init_telemetry` en `vantadb-server/src/main.rs` para rutear a `stderr` si `is_mcp` es activo.
- [x] Actualizar `.env.example` con las variables de Rate Limiting, Auth, JSON logging, y OTLP.
- [x] Validar compilación usando `--features opentelemetry`.
- [x] Actualizar `walkthrough.md`.

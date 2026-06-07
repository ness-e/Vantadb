# Tarea: MP1 — Seguridad Avanzada del Servidor

- `[x]` Tarea Principal: Implementar capas de seguridad avanzadas en `vantadb-server`. (`src/config.rs`)
- `[x]` ST1.2: Implementar el middleware de Auth Bearer (`auth_middleware`) en `src/middleware.rs`. (auth + rate limit)
- `[x]` ST1.3: Actualizar `src/server.rs` para inyectar dinámicamente Auth y `GovernorLayer` basados en la configuración.
- `[x]` ST1.5: Resolver problemas de compilación (compatibilidad entre `tower 0.5`, `axum` y `tower-governor 0.8`).
- `[x]` Eliminar `vantadb-server/vanta_certification.json`
- `[x]` Verificar compilación `cargo build -p vantadb-server`
- `[x]` Verificar compilación con TLS `cargo build -p vantadb-server --features tls`
- `[x]` Commit y snapshot histórico

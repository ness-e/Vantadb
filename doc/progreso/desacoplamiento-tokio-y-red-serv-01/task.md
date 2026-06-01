# Tareas: Fase SERV-01 (Desacoplamiento de Tokio y Red del Core)

- `[x]` Modificaciones en la configuración de dependencias del Core (`vantadb`)
    - `[x]` Remover `tokio` de `[dependencies]` en `Cargo.toml` raíz (relegándolo a `[dev-dependencies]`).
    - `[x]` Renombrar la feature `llm` a `remote-inference` en `[features]` en `Cargo.toml`.
- `[x]` Modificaciones de código en el Core (`vantadb`)
    - `[x]` Actualizar importaciones y condicionales en `src/lib.rs` (cambiar `#[cfg(feature = "llm")]` a `#[cfg(feature = "remote-inference")]`).
    - `[x]` Actualizar todos los condicionales condicionados a `llm` en `src/executor.rs` por `remote-inference`.
- `[x]` Modificaciones en subcrates locales del Workspace
    - `[x]` Modificar `Cargo.toml` de `packages/experimental-governance` para mapear `remote-inference` a `vantadb/remote-inference`.
    - `[x]` Actualizar directivas en `packages/experimental-governance/src/maintenance_worker.rs` de `llm` a `remote-inference`.
- `[x]` Verificación y Validación de la Arquitectura
    - `[x]` Ejecutar auditoría estática de dependencias mediante `cargo tree` para asegurar que `tokio` y `reqwest` no se compilen en modo síncrono.
    - `[x]` Comprobar compilación general del workspace.
    - `[ ]` Solicitar ejecución de la suite de tests al usuario para verificar la estabilidad concurrente.

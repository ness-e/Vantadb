# Tareas: Fase CLI-01 (Modernización de la Interfaz de Línea de Comandos)

- `[x]` Integrar dependencias en `Cargo.toml`
    - `[x]` Añadir `clap` y `clap_complete` como dependencias opcionales.
    - `[x]` Habilitar las nuevas dependencias en la feature `cli`.
- `[x]` Diseñar y reescribir `src/bin/vanta-cli.rs`
    - `[x]` Definir la estructura `Cli` y el enum de subcomandos `Commands` usando `clap::Parser`.
    - `[x]` Agregar variable de entorno `VANTA_DB` como fallback del argumento global `--db`.
    - `[x]` Implementar el generador de completions en `build.rs` (automático al compilar con la feature `cli`).
- `[x]` Refinar la experiencia visual (UX Premium)
    - `[x]` Integrar spinners de progreso con `indicatif` en `rebuild-index` e `import`.
    - `[x]` Formatear listados y resultados de `query` en tablas limpias y estéticas con `console`.
    - `[x]` Diseñar el dashboard detallado del subcomando `status`.
- `[/]` Pruebas y Validación Manual
    - `[ ]` Comprobar que compila correctamente con `cargo check --workspace --all-targets --all-features`.
    - `[ ]` Validar funcionamiento de subcomandos y reportar.

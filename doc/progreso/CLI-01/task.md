# Tareas: Fase CLI-01 (Modernización de la Interfaz de Línea de Comandos)

## Estado General: ✅ COMPLETADO

### Integración de Dependencias
- `[x]` Integrar dependencias en `Cargo.toml`
  - `[x]` Añadir `clap` y `clap_complete` como dependencias opcionales.
  - `[x]` Habilitar las nuevas dependencias en la feature `cli`.

### Diseño y Reescritura de la CLI
- `[x]` Diseñar y reescribir `src/bin/vanta-cli.rs`
  - `[x]` Definir la estructura `Cli` y el enum de subcomandos `Commands` usando `clap::Parser`.
  - `[x]` Agregar variable de entorno `VANTA_DB` como fallback del argumento global `--db`.

### Autocompletado
- `[x]` Implementar el generador de completions en `build.rs` (automático al compilar con la feature `cli`).
- `[x]` Implementar subcomando `completions` con soporte multi-shell (bash, zsh, fish, powershell).

### Experiencia Visual (UX Premium)
- `[x]` Refinar la experiencia visual (UX Premium)
  - `[x]` Integrar spinners de progreso con `indicatif` en `rebuild-index` e `import`.
  - `[x]` Formatear listados y resultados de `query` en tablas limpias y estéticas con `console`.
  - `[x]` Diseñar el dashboard detallado del subcomando `status` (con detección de hardware, AVX2, RAM, Cores).

### Pruebas y Validación Manual
- `[x]` Comprobar que compila correctamente con `cargo check --workspace --all-targets --all-features`.
- `[x]` Validar funcionamiento de subcomandos y reportar.
  - `[x]` Menú de ayuda autogenerado (`--help`) funcional.
  - `[x]` Dashboard de estado (`status`) muestra métricas del sistema correctamente.
  - `[x]` Autocompletado para PowerShell funcional (acepta `powershell` y `power-shell`).

## Notas de Implementación
- El comando `status` requiere que la base de datos exista en la ruta especificada. Si no existe, retorna un error controlado del `StorageEngine`, lo cual es el comportamiento esperado.
- Se corrigió el enum `Shell` para aceptar `powershell` (sin guion) como valor principal, mejorando la UX frente al `power-shell` por defecto de `clap`.

# `chaos-45.yml` — Chaos: Failpoint Injection & Resilience Tests

## ¿Qué hace?

Ejecuta los tests de caos de VantaDB con inyección de failpoints para verificar resiliencia ante fallos inyectados (errores de I/O, panics en puntos críticos, cortes) durante operaciones normales.

## ¿Cómo lo hace?

Un solo job `chaos`:

1. Setup de Rust con `rust-setup` (nextest, system deps, 4GB swap)
2. Ejecuta `cargo nextest run --profile chaos --features failpoints -p vantadb`

Usa el **perfil `chaos`** de nextest (`.config/nextest.toml`), que incluye los tests marcados como de caos/resiliencia.

## ¿Qué tests usa?

Los tests del crate `vantadb` compilados con la feature `failpoints` y el perfil `chaos` de nextest.

## ¿Qué verifica?

- Que la base sobreviva a fallos inyectados en puntos críticos
- Que no haya panics no controlados ante errores de I/O o corrupción parcial
- Resiliencia de WAL, storage e índices ante fallos a mitad de operación

## Funcionalidad final

Validación continua de que los mecanismos de failpoint y recuperación de VantaDB funcionan, complementando los tests pesados de certificación.

## ¿Cuándo se ejecuta?

- **Push a `main`** (paths: `src/**`, `tests/**`, `Cargo.toml`, `Cargo.lock`, `.config/nextest.toml`, el propio workflow)
- **Pull request a `main`** (mismos paths)
- **Workflow dispatch** manual (sobre cualquier rama)

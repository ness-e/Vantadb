# FIND-147 — warnings `unused_imports` en `src/sdk/search/debug_ops.rs` (perfil release/bench)

> **Plan:** (standalone, fix directo) · **Estado:** ⬜ PENDIENTE
> **Origen:** warnings reportados por owner al compilar `benches/canonical_p99.rs` 2026-09-22

## Objetivo

Eliminar los 5 warnings `unused_imports` que `vantadb` (lib) emite en perfil bench/release.

## Causa raíz (verificada en código, no opinión)

Todo el contenido de `src/sdk/search/debug_ops.rs` (397 líneas) vive bajo
`#[cfg(debug_assertions)]`, pero 5 `use` están SIN gatear (:2, :5, :6, :7, :10).
Con `debug_assertions` off (perfil `bench`/`release`), nada los usa → warnings:

- :2 `serialization::{validate_metadata, validate_namespace}`
- :5 `super::super::types::*`
- :6 `super::debug`
- :7 `crate::backend::BackendPartition`
- :10 `crate::error::{Error, Result}`

(Precedente en el mismo archivo: :3-4 y :8-9 YA están gateados con `#[cfg(debug_assertions)]`.)

## Contrato

- `cargo clippy -p vantadb --all-targets -- -D warnings` → exit 0.
- `cargo check -p vantadb --benches` → 0 warnings en `debug_ops.rs`.
- Cero cambios de comportamiento (solo `#[cfg]` en imports;release ya compila un `impl` vacío).

## Acceptance criteria

- [ ] Los 5 `use` llevan `#[cfg(debug_assertions)]` (espejo de :3-4/:8-9)
- [ ] `cargo check -p vantadb` (dev) sigue verde — los imports siguen vivos con debug_assertions
- [ ] `cargo check -p vantadb --benches` sin warnings del archivo
- [ ] `cargo clippy -p vantadb --all-targets -- -D warnings` verde

## Archivos

- Clave: `src/sdk/search/debug_ops.rs:1-10`
- Prohibidos: todo lo demás (fix de 5 líneas; si aparece algo más, STOP + reportar)

## Pasos atómicos

- [ ] S1: gatear los 5 imports + `cargo check -p vantadb --benches` (0 warnings archivo)
- [ ] S2: `cargo check -p vantadb` + `cargo clippy -p vantadb --all-targets -- -D warnings` + commit `fix: FIND-147 — ...` (NO PUSH)

## Context Save Point

Creado 2026-09-22 desde reporte owner. Rama `develop`. Sin WIP en el archivo.

---
title: "Workflow local de desarrollo"
kind: concept
status: active
description: "Herramientas del flujo diario: Justfile, verify scripts, perfiles de build y comandos de documentación."
---

# Workflow local de desarrollo

## Build y perfiles

`[HECHO]` [`Cargo.toml`](../../../Cargo.toml) define perfiles `release` (lto thin), `ci`, `bench` (debug=1) y `dev` (opt-level=1, debug=0); `rust-version` declarado en `[workspace.package]` y toolchain fijado por [`rust-toolchain.toml`](../../../rust-toolchain.toml). `[HECHO]` Existe `Justfile` en raíz con recetas de build/verify (leer antes de invocar).

## Verificación del día a día

`[HECHO]` Los scripts documentados son `dev-tools/verify.ps1` (pre-flight) y `dev-tools/verify_changed.ps1` (verificación rápida de cambios); el gate de floor es `dev-tools/floor-guard.ps1`. Comandos cargo habituales documentados en AGENTS: `cargo check -p vantadb`, `cargo clippy -p vantadb`, `cargo nextest run --profile audit -p vantadb`. `[INFERENCIA]` Los tests/lints los ejecuta el usuario; esta wiki no afirma su resultado.

## Documentación

`[HECHO]` Flujo docs: editar `.md` con frontmatter (`title`+`kind` requeridos; `description` recomendable — sin ella el índice deriva la primera línea de prosa), luego `node scripts/docs/check-docs.mjs`, `check-links.mjs`, y `gen-index.mjs --write` para regenerar `docs/index.md`/`llms.txt`/índices de sección (nunca a mano). `deriveKind` decide `kind` por ruta (`scripts/docs/lib.mjs`).

## Fuentes

`Cargo.toml` · `Justfile` · `dev-tools/` · `scripts/docs/` · [verification](verification.md) · [contributor/change-checklist](../contributor/change-checklist.md)

---
title: Declarar publish = false en vantadb-wasm/Cargo.toml
kind: task
status: completed
description: "vantadb-wasm se distribuye exclusivamente por wasm-pack/npm, nunca por crates.io"
id: FIND-473
type: chore
plan: docs/dev/plans/2026-11-07-consolidacion-tecnica-pre-v0100.md
---

# FIND-473 — `publish = false` en `vantadb-wasm/Cargo.toml`

## Objetivo
`vantadb-wasm` se distribuye exclusivamente por `wasm-pack`/npm, nunca por crates.io.
Sin `publish = false` en `[package]`, un `cargo publish --workspace` o un cambio en
`release-plz.toml` podría intentar subirlo. Defensa en profundidad para v0.10.0.

## Causa raíz
Las líneas 1-11 del `[package]` de `vantadb-wasm/Cargo.toml` declaran `name`,
`version.workspace`, `edition.workspace`, `rust-version.workspace`, `description`,
`repository`, `license`, `keywords`. `package.publish` lee `None` (tomllib) →
cargo permite publicar a crates.io. `release-plz.toml` ya lo excluye con
`release = false`, pero eso solo cubre release-plz, no un `cargo publish` manual
ni un `--workspace`.

## Impacto mapeado (Regla 0)
- Archivos leídos completos: `vantadb-wasm/Cargo.toml`, `release-plz.toml`,
  `tests/version_coherence.rs`
- Referencias hacia dentro: cargo publish / release-plz leen `[package].publish`;
  wasm-pack lee `Cargo.toml` para generar `pkg/package.json`
- Referencias entrantes: release-plz (`[[package]] name = "vantadb-wasm"`,
  `release = false`); workflow `release-npm-wasm.yml` (fuera de scope, FIND-550)
- Veredicto: metadato de manifiesto. Blast radius = empaquetado únicamente.
  No altera `crate-type = ["cdylib", "lib"]` ni `[package.metadata.wasm-pack.profile.release]`.

## Investigación internet (fuentes)
- Cargo book — `publish` field: "To prevent a package from being published to a
  registry (like crates.io) by mistake... you can disable publishing:
  `publish = false`. The publish value may also be `false` to restrict all
  publishing, which is the same as an empty list."
  https://doc.rust-lang.org/cargo/reference/manifest.html /
  https://doc.rust-lang.org/cargo/reference/registries.html
- `cargo publish` checks `package.publish` ANTES de empaquetar:
  https://doc.rust-lang.org/cargo/commands/cargo-publish.html
- wasm-pack `src/manifest/mod.rs::write_package_json` mapea name, version,
  authors→collaborators, repository, keywords, files, license/README; NO lee
  `publish` (es restricción de registro cargo, no metadata npm):
  https://github.com/rustwasm/wasm-pack/blob/master/src/manifest/mod.rs

## Diferencia `release = false` (release-plz) vs `publish = false` (cargo)
- `release = false` en `release-plz.toml`: le dice a release-plz que ignore el
  crate (sin update/changelog/publish). Solo gobierna la herramienta release-plz.
- `publish = false` en `Cargo.toml`: le dice a cargo (cualquier `cargo publish`,
  con o sin `--workspace`) que rechace la publicación a TODOS los registros.
  Defensa en profundidad: el crate no puede subirse aunque release-plz cambie.

## Estado verificado (baseline)
- `rg "^publish\s*=\s*false" vantadb-wasm/Cargo.toml` → exit 1 (no match),
  el `&&` del contrato corta → contrato FAIL esperado ✔

## Steps
- [x] 1. Añadir `publish = false` dentro de `[package]` en `vantadb-wasm/Cargo.toml`
- [x] 2. Contrato POST: `rg` casa + `cargo check -p vantadb-wasm --locked` verde
- [x] 3. DoD: `cargo nextest run --profile audit -p vantadb --test version_coherence --locked --build-jobs 2`
- [x] 4. Review P2-01 (tier fast — path no matchea globs adversariales)
- [x] 5. Commit local `build(wasm): set publish = false in vantadb-wasm Cargo.toml` (FIND-473) — sin push

## Contrato (verificación mecánica)
rg "^publish\s*=\s*false" vantadb-wasm/Cargo.toml && cargo check -p vantadb-wasm --locked

## Riesgos (pre-mortem)
- Insertar `publish` fuera de `[package]` (p.ej. en `[lib]` o metadata wasm-pack)
  no tendría efecto → va junto a `license`, antes del comentario de `keywords`.
- `version_coherence.rs` parsea líneas `version*`: `publish` no empieza por
  `version`, sin colisión (verificado líneas 18-37 del test).
- Concurrencia: FIND-467 edita workflows de release y FIND-550 edita
  `release-npm-wasm.yml`/`vantadb-ts` EN PARALELO — solo se toca
  `vantadb-wasm/Cargo.toml` (único archivo de este task).

## SDP
SKILLS: shipping-and-launch, git-workflow-and-versioning (MCP campaign_* no
disponible en este entorno — SDP manual por keywords del contrato)

## Review (P2-01)
- Tier: fast (`vantadb-wasm/Cargo.toml` no matchea globs adversariales §163 pipeline-full)
- Veredicto: approve (vanta-review, contexto fresco 8a0adc97).
  Red-team ejecutado: verifico gen-surface-manifest.mjs:199 (regex ^publish) —
  sin colision con el comentario (ancla ^) y sin deriva de SURFACE_MANIFEST
  (precedencia release-plz :289-294). 10 crates del workspace ya usan publish=false.

## Context Save Point
Trabajo parcial = 1 línea en `vantadb-wasm/Cargo.toml` + este task file.
Si se interrumpe: re-ejecutar contrato, retomar desde el primer step ⬜.

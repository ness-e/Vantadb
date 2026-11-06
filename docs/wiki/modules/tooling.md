---
title: "Módulo: tooling (scripts, dev-tools, CI, docs toolchain)"
kind: concept
status: active
description: "Herramientas del repo: scripts de docs, dev-tools, workflows CI/CD y gates que mantienen índices y superficies coherentes."
---

# Módulo: tooling (scripts, dev-tools, CI, docs toolchain)

Responsabilidad: la maquinaria que mantiene el repo verificable — generadores, gates y pipelines.

## Toolchain de docs

`[HECHO]` [`scripts/docs/`](../../../scripts/docs/) contiene los scripts `.mjs` de la toolchain: `gen-index.mjs` (genera `docs/index.md`, `docs/<section>/index.md`, `docs/dev/architecture/adr/README.md`, `llms.txt`), `check-docs.mjs` (frontmatter+kind), `check-links.mjs`, `gen-surface-manifest.mjs`, `check-doc-examples.mjs`, `check-api-docs.mjs`, `check-npm-versions.mjs`, `stamp-frontmatter.mjs`, `repair-links.mjs`, `wikilinks-to-md.mjs`, `check-secrets.mjs`, `check-structure.mjs` y `lib.mjs` (deriveKind, parser frontmatter). Reglas: `kind` derivado de ruta, índices nunca editados a mano; sin `description` el índice **deriva** la primera línea de prosa (`firstProseLine` en `lib.mjs`) — declarar `description` sigue siendo lo recomendable porque la derivada puede salir peor que la ausencia.

## dev-tools y scripts

`[HECHO]` `dev-tools/` incluye `parity-compare.mjs` (comparador de paridad DIST-17), `package-stats.mjs` y `verify.ps1`/`verify_changed.ps1`/`floor-guard.ps1` (referenciados por la documentación de workflow). `[HECHO]` `Justfile` en raíz con recetas de build/verify; `scripts/` incluye instaladores `install.sh`/`install.ps1` referenciados por el quickstart.

## CI

`[HECHO]` Workflows en `.github/workflows/`: `ci-rust`, `ci-gate`, `ci-fleet`, `docs-gates`, `certification`, `nightly`, `platform-ops`, `metrics-collector`, `demo-acceptance`, `publish-mcp-registry` y los de release (`release`, `release-plz` en `release-plz.toml`, `release-wheels`, `release-npm-wasm`, `release-npm-node`, `release-binaries`, `release-adapters`, `release-providers`, `release-verify`). `[INFERENCIA]` Triggers/permisos/pins exactos: por revisar en cada archivo antes de citarlos.

## Fuentes

`scripts/docs/` · `dev-tools/` · `.github/workflows/` · `Justfile` · `release-plz.toml` · [CI_POLICY](../../dev/operations/CI_POLICY.md)

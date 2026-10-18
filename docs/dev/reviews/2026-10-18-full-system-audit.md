---
title: "Auditoría Integral del Sistema VantaDB v0.9.0 — Release Automation, CI Hardening y Cierre de Hallazgos"
kind: review
status: active
description: "Documentación de hallazgos finales FIND-379 y FIND-380 para el tren de release 0.9.0, complementando la auditoría multi-agente FIND-345..378"
tags: [vantadb, audit, release, ci, security, github-actions]
---

# Auditoría Integral del Sistema VantaDB v0.9.0 — Release Automation, CI Hardening y Cierre de Hallazgos

> **Fecha:** 2026-10-18  
> **Modo:** `full` (Remediación y verificación de cierre para tren de release `0.9.0`)  
> **Estado:** RESUELTO (2026-10-18)  
> **Trazabilidad:** Complementa la auditoría multi-agente [`docs/dev/reviews/audit-full-20261017-multiagent.md`](audit-full-20261017-multiagent.md) (`FIND-345..378`). Registra la resolución integral de `FIND-379` y `FIND-380`.

---

## 1. Resumen Ejecutivo

En el marco del plan de ejecución `docs/dev/plans/2026-10-18-release-0.9.0-automation-and-dx.md`, tras la remediación de los 34 hallazgos iniciales (`FIND-345..378`) y la consolidación de workflows (`CI-08`), se identificaron y resolvieron de forma exhaustiva los dos hallazgos sistémicos finales previos al merge de `develop` hacia `main`:

1. **`FIND-379`**: Automatización de sincronización multi-ecosistema en Release PR y desacoplamiento de canales de publicación.
2. **`FIND-380`**: Hardening de seguridad y cumplimiento de `RULES.md` en los 20 workflows activos de GitHub Actions.

---

## 2. Hallazgos Finales Resueltos

### `FIND-379` — Automatización de sincronización multi-ecosistema en Release PR y desacoplamiento de canales de publicación
- **Estado:** `RESUELTO (2026-10-18)`
- **Ubicación:** `scripts/docs/sync-workspace-version.mjs` · `.github/workflows/release.yml` · `.github/workflows/release-npm-node.yml` · `.github/workflows/release-wheels.yml` · `feature-freez/desktop/src-tauri/Cargo.toml` · `tests/version_coherence.rs` · `integrations/*/pyproject.toml` · `docs/dev/workflow/PUBLISH.md`
- **Diagnóstico / FMEA:**
  - `release-plz` calcula correctamente el incremento semántico de versión (`0.9.0`) analizando Conventional Commits, pero por diseño únicamente actualiza los manifiestos Cargo raíz (`Cargo.toml`, `Cargo.lock`) y `docs/CHANGELOG.md`.
  - Los contratos y manifiestos de otros ecosistemas acoplados a la versión de VantaDB (`vantadb-ts/package.json`, `server.json`, `docs/api/openapi.yaml`, `docs/api/MCP.md`, `integrations/*/pyproject.toml`, `docs/api/SURFACE_MANIFEST.md`) quedaban desfasados o requerían sincronización manual propensa a error humano.
  - El workflow `release-npm-node.yml` se disparaba ante cualquier tag `v*.*.*` del core, arriesgando publicación accidental a npm en paralelo al release de crates.io, y `release-wheels.yml` carecía de `skip-existing: true` ante reintentos.
- **Remediación Implementada:**
  - Se implementó `scripts/docs/sync-workspace-version.mjs` con soporte para `--check` y `--self-test`, integrándolo automáticamente en el paso `release-plz-pr` de `.github/workflows/release.yml` para comitear los manifiestos sincronizados directamente en la rama del PR de release.
  - Se desacopló `release-npm-node.yml` protegiéndolo con un input booleano explícito `publish` y canal independiente.
  - Se configuró `skip-existing: true` en `release-wheels.yml`.
  - Se marcó `publish = false` en `feature-freez/desktop/src-tauri/Cargo.toml` para evitar intentos espurios de publicación en crates.io.
  - Se extendió la suite de pruebas `tests/version_coherence.rs` para garantizar la coherencia en todas las superficies integradas.

---

### `FIND-380` — Hardening de seguridad y cumplimiento de `RULES.md` en los 20 workflows de GitHub Actions
- **Estado:** `RESUELTO (2026-10-18)`
- **Ubicación:** `.github/workflows/electron-node.yml` · `.github/workflows/ci-fleet.yml` · `.github/workflows/ci-rust.yml` · `.github/workflows/certification.yml` · `.github/workflows/perf-bench.yml` · `docs/dev/workflow/README.md`
- **Diagnóstico / FMEA:**
  - `electron-node.yml` carecía del bloque explícito `permissions: contents: read` a nivel top-level y ejecutaba triggers de `pull_request` en ramas arbitrarias en lugar de limitarse a `branches: [main]` (violación de Reglas 4 y 7 de `docs/dev/workflow/RULES.md`).
  - `ci-fleet.yml` usaba `jetli/wasm-pack-action` sin versión fija.
  - `ci-rust.yml` y `certification.yml` invocaban `cargo install wasm-pack` y `cargo install critcmp` sin flags `--version ... --locked`, introduciendo no-determinismo ante upstream releases.
  - En `certification.yml` y `perf-bench.yml`, existían interpolaciones directas de expresiones `${{ inputs.* }}` en scripts inline de shell `run:`, abriendo vectores potenciales de inyección de comandos si los valores contienen caracteres de control.
  - El índice normativo en `docs/dev/workflow/README.md` no reflejaba la consolidación a los 20 workflows activos.
- **Remediación Implementada:**
  - Se aplicaron permisos de mínimo privilegio `permissions: contents: read` a nivel global en `electron-node.yml` y se restringieron los disparadores `pull_request` a `main`.
  - Se fijó `version: 'v0.13.1'` en `jetli/wasm-pack-action` y se incorporó Python `3.14` a la matriz (`DX-09`).
  - Se añadieron pines estrictos `--version ... --locked` a todas las instalaciones ad-hoc con `cargo install`.
  - Se eliminó la interpolación directa en bloques `run:`, pasando los inputs a través de variables de entorno seguras (`env:`).
  - Se actualizó el catálogo en `docs/dev/workflow/README.md` reflejando de forma canónica los 20 workflows de GitHub Actions activos.

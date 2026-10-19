---
title: "FIND-380 — Hardening de seguridad y cumplimiento de RULES.md en workflows CI"
kind: task
description: "Remediación de WF-01..05 en los 20 workflows de GitHub Actions y catálogo README"
status: completed
---

# Task `FIND-380` (+ `DX-09`): Hardening de Seguridad y Reglas CI en `.github/workflows/`

## Scope (Allowed Files)
- `.github/workflows/electron-node.yml`
- `.github/workflows/ci-fleet.yml`
- `.github/workflows/ci-rust.yml`
- `.github/workflows/certification.yml`
- `.github/workflows/perf-bench.yml`
- `docs/dev/workflow/README.md`

## Acceptance Criteria
1. `electron-node.yml` declares top-level `permissions: contents: read` and restricts `pull_request` to `branches: [main]`.
2. `ci-fleet.yml` pins `jetli/wasm-pack-action` `version: 'v0.13.1'` (replacing `latest`) and adds `'3.14'` to the Python version matrix (`DX-09`).
3. `ci-rust.yml` pins `cargo install wasm-pack --version 0.13.1 --locked` and `certification.yml` pins `cargo install critcmp --version 0.1.8 --locked`.
4. `certification.yml` and `perf-bench.yml` move `${{ inputs.* }}` / `${{ github.event.inputs.* }}` into `env:` variables instead of interpolating them directly inside `run:` shell scripts.
5. `docs/dev/workflow/README.md` lists `electron-node.yml` and `publish-mcp-registry.yml` so all 20 workflows are documented.

## Verification Commands
- `node scripts/docs/check-docs.mjs`
- `node scripts/docs/check-links.mjs`

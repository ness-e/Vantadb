---
title: "FIND-379 — Automatizar sincronización multi-ecosistema en Release PR"
kind: task
description: "Sincronización automática de manifiestos en Release PR y desacoplamiento de canales de publicación"
status: completed
---

# Task `FIND-379`: Automatización Integral de Versionado y Publicación Multi-Ecosistema

## Scope (Allowed Files)
- `scripts/docs/sync-workspace-version.mjs` (new)
- `.github/workflows/release.yml`
- `.github/workflows/release-npm-node.yml`
- `.github/workflows/release-wheels.yml`
- `feature-freez/desktop/src-tauri/Cargo.toml`
- `tests/version_coherence.rs`
- `integrations/crewai/pyproject.toml`
- `integrations/dspy/pyproject.toml`
- `integrations/haystack/pyproject.toml`
- `integrations/langchain/pyproject.toml`
- `integrations/letta/pyproject.toml`
- `integrations/llamaindex/pyproject.toml`
- `integrations/mem0/pyproject.toml`
- `integrations/ollama/pyproject.toml`
- `integrations/openai/pyproject.toml`
- `docs/dev/workflow/PUBLISH.md`

## Acceptance Criteria
1. `scripts/docs/sync-workspace-version.mjs` reads `[workspace.package] version` from root `Cargo.toml` and supports `--check`, `--write`, and `--self-test` flags, keeping `vantadb-ts/package.json`, `server.json`, `docs/api/openapi.yaml`, `docs/api/MCP.md`, `integrations/*/pyproject.toml` (`<0.{minor+1}.0`), and `docs/api/SURFACE_MANIFEST.md` synchronized.
2. `.github/workflows/release.yml` sequences `release-plz-pr` after `release-plz-release` (`if: needs.release-plz-release.outputs.releases_created != 'true'`), sets `timeout-minutes: 15`, runs `node scripts/docs/sync-workspace-version.mjs --write` on the open `release-plz-*` branch, and commits/pushes changes if needed.
3. `.github/workflows/release.yml` removes `gh workflow run release-npm-node.yml --ref "$TAG"` from `fanout`.
4. `.github/workflows/release-npm-node.yml` adds a `publish` boolean input on `workflow_dispatch` (default `'false'`) and guards the `publish` job so it only runs on `refs/tags/node-v*` or when `inputs.publish == 'true'`.
5. `.github/workflows/release-wheels.yml` adds `skip-existing: true` to `pypa/gh-action-pypi-publish`.
6. `feature-freez/desktop/src-tauri/Cargo.toml` includes `publish = false`.

## Verification Commands
- `node scripts/docs/sync-workspace-version.mjs --self-test`
- `node scripts/docs/sync-workspace-version.mjs --check`
- `cargo nextest run --profile audit -p vantadb --test version_coherence`

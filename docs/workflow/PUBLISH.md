---
title: "Workflows — Publish flow per registry"
type: workflow-index
status: active
tags: [vantadb, ci, workflows, publish, release]
last_reviewed: 2026-09-22
aliases: []
related: ["docs/workflow/README.md", "docs/workflow/TRIGGERS.md", "docs/workflow/RUNBOOK.md"]
---

# Workflows — Publish flow per registry

Publishing is tokenless (OIDC Trusted Publishing, no PATs). Post-FIND-140
state. Rollback policy per plan: `git revert` per commit + re-run.

## Chain overview

```text
merge develop -> main
  -> release.yml (release-plz: bump, changelog, git tag, crates.io)
    -> tag v*.*.*      -> wheels-60 (PyPI) + npm-61 (npm wasm+TS) + sbom-64 (SBOM)
    -> tag node-v*.*.* -> npm-node (npm Node binding)
    -> tag adapters-v* -> adapters-62 (PyPI adapters)
    -> GitHub Release  -> binaries-63 (binaries + docker image assets)
```

## crates.io — `release.yml`

- Trigger: `push: branches: [main]` only.
- `release-plz-release` job: version bump, changelog, tag push, `cargo publish`
  via OIDC (`id-token: write`). Timeout 30 min, no cancel (`cancel-in-progress: false`).
- `release-plz-pr` job: opens the Release PR on subsequent pushes.

## PyPI wheels — `release-wheels.yml`

- Tags `v*.*.*` publish to prod PyPI (`environment: pypi`, OIDC + provenance
  attestation); `workflow_dispatch` with `publish_testpypi=true` publishes to
  TestPyPI (`environment: testpypi`).
- PRs to `main` touching `src/`, `vantadb-python/`, `Cargo.*` run build +
  smoke only (no publish).
- Tag namespaces: strict `v*.*.*` (three numeric parts).

## npm WASM + TS — `release-npm-61.yml`

- Tags `v*.*.*` publish both packages (`environment: npm`, OIDC):
  `publish-wasm` builds `vantadb-wasm` with wasm-pack, then `publish-ts`
  rewrites the exact wasm version into `vantadb-ts` and publishes `vantadb`.
- PRs touching `vantadb-wasm/**` or `vantadb-ts/**` run CI only.
- `workflow_dispatch` selects `wasm` / `ts` / `both` + `dry_run`.

## npm Node — `release-npm-node.yml`

- Tags `node-v*.*.*` publish the Node binding (`environment: npm`, OIDC).
- PRs touching `vantadb-node/**` run CI only.
- Own namespace so core `v*` releases never publish the Node package by accident.

## Adapters — `release-adapters.yml`

- Tags `adapters-v*.*.*` publish 9 adapters
  (langchain, llamaindex, mem0, crewai, dspy, haystack, letta, openai, ollama)
  to prod PyPI after the test matrix passes.
- `workflow_dispatch` with `publish_testpypi=true` goes to TestPyPI.
- Own namespace so adapter-only releases never trigger core wheels/npm.

## Binaries — `release-binaries.yml`

- Trigger: `release: types: [published]` + manual dispatch only (FIND-140:
  the old `push.tags: [v*]` built 5 targets + docker just to discard).
- Builds `vanta-cli` + `vantadb-server` for 5 targets with the custom
  allocator (Windows mimalloc, Linux/macOS jemalloc), uploads
  `tar.gz`/`zip` + sha256 to the GitHub Release, plus a build-no-push
  docker image smoke test.

## SBOM — `release-sbom.yml`

- Tags `v*` (broad: matches any `v` tag, including `v*.*.*`) generate
  CycloneDX SBOMs (Rust + web + Python) as workflow artifacts.
- Read-only permissions; never blocks publish.

## Tag namespace table

| Namespace | Owner workflow(s) | Publishes to |
|-----------|-------------------|--------------|
| `v*.*.*` | wheels-60, npm-61 | PyPI (`vantadb-py`), npm (`vantadb-wasm`, `vantadb`) |
| `node-v*.*.*` | npm-node | npm (Node binding) |
| `adapters-v*.*.*` | adapters-62 | PyPI (9 adapters) |
| `v*` (broad) | sbom-64 | Artifacts only (no registry) |
| GitHub Release | binaries-63 | Release assets (binaries, docker tarball) |

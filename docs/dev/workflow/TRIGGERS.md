---
title: Workflows — Trigger matrix
kind: runbook
status: active
description: "Source of truth is each file's on: block (read 2026-10-04, post FIND-228 + DIST-05 + DIST-17)"
tags: [vantadb, ci, workflows, triggers]
---

# Workflows — Trigger matrix

Source of truth is each file's `on:` block (read 2026-10-04, post
FIND-228 + DIST-05). Reuses and updates the FIND-128 matrix
(`docs/dev/tasks/FIND-128.md` §Notas), which counted 28 pre-FIND-137.

Post-FIND-228 rule (RULES.md §1): no CI/demo/gate workflow lists `develop`
under `push.branches`; pre-merge validation for `develop` rides
`pull_request` (kept on the six dual-validation workflows). Documented
exception: `perf-bench.yml` (see Notes).

Legend: Y = yes, — = no. `paths` means a path filter applies (see file).

## CI / gates / desktop

| Workflow | push | pull_request | schedule | dispatch | Other |
|----------|------|--------------|----------|----------|-------|
| `ci-rust.yml` | Y (`main`) | Y (`main`) | — | Y | — |
| `ci-rustdoc.yml` | Y (`main`) | Y (`main`,`develop`) | — | Y | — |
| `ci-bindings-parity.yml` | Y (`main`, paths bindings) | Y (`main`,`develop`, paths bindings) | — | Y | — |
| `ci-examples.yml` | Y (`main`) | Y (`main`) | — | Y | — |
| `chaos.yml` | Y (`main`) | Y (`main`) | — | Y | — |
| `gate-docs.yml` | Y (`main`) | Y (`main`,`develop`) | — | Y | — |
| `gate-docs-links.yml` | Y (`main`) | Y (`main`,`develop`) | Y (Mon 07:23) | Y | — |
| `gate-docs-secrets.yml` | Y (`main`) | Y (`main`,`develop`) | — | Y | — |
| `gate-api-docs.yml` | Y (`main`) | Y (`main`,`develop`) | — | Y | — |
| `gate-doc-examples.yml` | Y (`main`) | Y (`main`,`develop`) | — | Y | — |
| `providers-ci.yml` | Y (`main`) | Y (`main`) | — | Y | — |
| `desktop.yml` | Y (`main`) | Y (`main`) | — | Y | — |
| `perf-bench.yml` | Y (`main`,`develop`) | — | — | Y | — |
| `sec-codeql.yml` | Y (`main`) | Y (`main`) | Y (Sun) | Y | — |
| `ci-gate.yml` | — | — | — | — | `workflow_call` |

## Demos (acceptance — F5/F6)

| Workflow | push | pull_request | schedule | dispatch | Other |
|----------|------|--------------|----------|----------|-------|
| `ci-ai-ides-demo.yml` | Y (`main`) | Y (`main`) | — | Y | — |
| `ci-frameworks-demo.yml` | Y (`main`) | Y (`main`) | — | Y | — |
| `icp02-privacy-demo.yml` | Y (`main`) | Y (`main`) | — | Y | — |
| `injection-governance-demo.yml` | Y (`main`) | Y (`main`) | — | Y | — |
| `wal-verify-demo.yml` | Y (`main`) | Y (`main`) | — | Y | — |

## Heavy / fuzz / informational / bots

| Workflow | push | pull_request | schedule | dispatch | Other |
|----------|------|--------------|----------|----------|-------|
| `fuzz.yml` | — | Y (paths src/fuzz) | Y (Mon 06:00) | Y | — |
| `heavy-bench-nightly.yml` | — | — (C-06: fuera del Fast Gate) | Y (daily 02:00) | Y | — |
| `heavy-certification.yml` | — | — | Y (Sun 03:00) | Y | — |
| `nightly.yml` | — | — | Y (daily 05:00) | Y | — |
| `adapters-compat.yml` | — | — | Y (Sun 03:00) | Y | — |
| `arch-metrics-informational.yml` | — | Y (paths src/Cargo) | — | Y | — |
| `bench-canonical-p99-informational.yml` | — | Y (paths index/storage) | — | Y | — |
| `ocr-delegate.yml` | — | Y (all) | — | Y | — |
| `ocr-nightly.yml` | — | — | Y (daily 04:00) | Y | — |
| `opencode.yml` | — | — | — | — | `issue_comment`, `review_comment` |

## Release (tag namespaces)

| Workflow | push tags | pull_request | dispatch | Other |
|----------|-----------|--------------|----------|-------|
| `release.yml` | — (branches `main` only) | — | — | — |
| `release-wheels.yml` | `v*.*.*` | Y (paths src/python) | Y | — |
| `release-npm-61.yml` | `v*.*.*` | Y (paths wasm/ts) | Y | — |
| `release-npm-node.yml` | `node-v*.*.*` | Y (paths node) | Y | — |
| `release-adapters.yml` | `adapters-v*.*.*` | — | Y | — |
| `release-providers.yml` | `providers-v*.*.*` | Y (paths providers) | Y | — |
| `release-binaries.yml` | — | — | Y | `release` published |
| `release-sbom.yml` | `v*` | — | Y | — |
| `release-verify.yml` | — | — | Y | schedule Mon 08:00 |

## Notes

- `release.yml` is the only workflow on bare `push: branches: [main]`
  (FIND-140: release-plz entry point). All other `push` entries carry
  `branches` + `paths` (exception: `sec-codeql.yml` scopes by `branches`
  only); since FIND-228 every CI/demo/gate push scopes to `[main]` —
  pre-merge validation lives in `pull_request`.
- Tags never evaluate `branches`/`paths` (FIND-140): tag workflows keep a
  tags-only `push:` block; branch CI for the same area goes through `pull_request`.
- Dual `pull_request` → `develop` validation is intentional (FIND-128
  proposal item 3; gate-docs gap closed by FIND-139) and kept on exactly
  six workflows: `ci-rustdoc`, `gate-docs`, `gate-docs-links`,
  `gate-docs-secrets`, `gate-api-docs`, `gate-doc-examples`.
- FIND-228 (2026-10-02): 15 workflows dropped `develop` from
  `push.branches` (compliance with RULES.md §1; duplicate pair measured on
  PR #233: ~80 checks vs ~45 expected). Exception: `perf-bench.yml` keeps
  `push: [main, develop]` — dependency of the FIND-232 post-push
  verification (separate decision).
- `ci-web.yml` row removed 2026-10-02: the file was deleted in `82317140`
  (W-05 — `web/` extracted to `ness-e/Vantadb-web`).
- `lurkr-informational.yml` is not yet in the matrix (out of FIND-228 scope).
- Schedule slots (FIND-141 + HARD-02): bench daily 02:00, cert Sun 03:00,
  ocr-nightly daily 04:00, nightly subset daily 05:00, fuzz Mon 06:00,
  docs-links weekly Mon 07:23, release-verify weekly Mon 08:00 — no overlaps.

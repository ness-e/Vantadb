---
title: "Workflows — Trigger matrix"
type: workflow-index
status: active
tags: [vantadb, ci, workflows, triggers]
last_reviewed: 2026-09-22
aliases: []
related: ["docs/workflow/README.md", "docs/workflow/FAQ.md"]
---

# Workflows — Trigger matrix

Source of truth is each file's `on:` block (read 2026-09-22, post
FIND-134/139/140/141/146). Reuses and updates the FIND-128 matrix
(`docs/dev/tasks/FIND-128.md` §Notas), which counted 28 pre-FIND-137.

Legend: Y = yes, — = no. `paths` means a path filter applies (see file).

## CI / gates / desktop

| Workflow | push | pull_request | schedule | dispatch | Other |
|----------|------|--------------|----------|----------|-------|
| `ci-rust.yml` | Y (`main`) | Y (`main`) | — | Y | — |
| `ci-rustdoc.yml` | Y (`main`,`develop`) | Y (`main`,`develop`) | — | Y | — |
| `ci-web.yml` | Y (`main`,`develop`) | Y (`main`) | — | Y | — |
| `ci-examples.yml` | Y (`main`,`develop`) | Y (`main`) | — | Y | — |
| `chaos.yml` | Y (`main`,`develop`) | Y (`main`) | — | Y | — |
| `gate-docs.yml` | Y (`main`,`develop`) | Y (`main`,`develop`) | — | Y | — |
| `sec-codeql.yml` | Y (`main`) | Y (`main`) | Y (Sun) | Y | — |
| `providers-ci.yml` | Y (`main`,`develop`) | Y (`main`) | — | Y | — |
| `desktop.yml` | Y (`main`,`develop`) | Y (`main`) | — | Y | — |
| `perf-bench.yml` | Y (`main`,`develop`) | — | — | Y | — |
| `ci-gate.yml` | — | — | — | — | `workflow_call` |

## Heavy / fuzz / informational / bots

| Workflow | push | pull_request | schedule | dispatch | Other |
|----------|------|--------------|----------|----------|-------|
| `fuzz.yml` | — | Y (paths src/fuzz) | Y (Mon 06:00) | Y | — |
| `heavy-bench-nightly.yml` | — | — (C-06: fuera del Fast Gate) | Y (daily 02:00) | Y | — |
| `heavy-certification.yml` | — | — | Y (Sun 03:00) | Y | — |
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
| `release-binaries.yml` | — | — | Y | `release` published |
| `release-sbom.yml` | `v*` | — | Y | — |

## Notes

- `release.yml` is the only workflow on bare `push: branches: [main]`
  (FIND-140: release-plz entry point). All other `push` entries carry
  `branches` + `paths`.
- Tags never evaluate `branches`/`paths` (FIND-140): tag workflows keep a
  tags-only `push:` block; branch CI for the same area goes through `pull_request`.
- `ci-rustdoc.yml` is the only CI workflow with PR → `develop`
  (kept intentionally, FIND-128 proposal item 3).
- Schedule slots (FIND-141): bench daily 02:00, cert Sun 03:00,
  ocr-nightly daily 04:00, fuzz Mon 06:00 — no overlaps.

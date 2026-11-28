---
title: Workflows — Trigger matrix
kind: runbook
status: active
description: "Source of truth is each workflow file's on: block"
tags: [vantadb, ci, workflows, triggers]
---

# Workflows — Trigger matrix

Source of truth is each file's `on:` block, read 2026-11-16.
The owner integrates by direct push to `develop`; matching CI workflows include
that branch under `push` (RULES.md §1).

Legend: Y = yes, — = no. `paths` means a path filter applies (see file).

## CI / gates / desktop

| Workflow | push | pull_request | schedule | dispatch | Other |
|----------|------|--------------|----------|----------|-------|
| `ci-rust.yml` | Y (`main`, `develop`, paths) | Y (`main`, `develop`, paths) | — | Y | — |
| `main-ancestry.yml` | — | Y (`main`, unfiltered) | — | — | `release-plz-*` exempt (FIND-633) |
| `docs-gates.yml` | Y (`main`, `develop`, paths) | Y (`main`, `develop`, paths) | Y (Mon 07:23) | Y | — |
| `ci-fleet.yml` | Y (`main`, `develop`, paths) | Y (`main`, `develop`, paths) | Y (Sun 03:00) | Y | 5 jobs (CI-08 Step 4) |
| `ci-electron.yml` | Y (`main`, `develop`, paths `vantadb-node/**`) | Y (`main`, `develop`, paths `vantadb-node/**`) | — | Y | Electron real ×3 OSes + `asarUnpack` (DIST-21) |
| `platform-ops.yml` | Y (`main`, `develop`, paths) | Y (`main`, `develop`, paths) | Y (Sun 00:00) | Y | CodeQL |
| `ci-gate.yml` | — | — | — | — | `workflow_call` |

## Demos (acceptance — F5/F6)

| Workflow | push | pull_request | schedule | dispatch | Other |
|----------|------|--------------|----------|----------|-------|
| `demo-acceptance.yml` | Y (`main`, `develop`, union paths) | Y (`main`, `develop`, union paths) | — | Y | 5 jobs (CI-08 Step 2) |

## Heavy / fuzz / informational / bots

| Workflow | push | pull_request | schedule | dispatch | Other |
|----------|------|--------------|----------|----------|-------|
| `certification.yml` | Y (`main`, chaos paths) | Y (paths, no `branches` — fuzz verbatim) | Y (daily 02:00, Sun 03:00, Mon 06:00) | Y | 21 jobs, ci-gate dedup ×3→1 (CI-08 Step 4) |
| `nightly.yml` | — | Y (paths info-only; heavy jobs skip PR) | Y (daily 05:00) | Y | 5 info jobs (CI-08 Step 3) |
| `metrics-collector.yml` | — | — | Y (Mon 06:00) | Y | — |

## Release (tag namespaces)

| Workflow | push tags | pull_request | dispatch | Other |
|----------|-----------|--------------|----------|-------|
| `release.yml` | — (branches `main` only) | — | — | — |
| `release-wheels.yml` | — (fanout `release.yml` despacha `--ref v*.*.*`; FIND-467) | Y (paths src/python) | Y | — |
| `release-npm-wasm.yml` | `v*.*.*` | Y (paths wasm/ts) | Y | — |
| `release-npm-node.yml` | — | Y (paths node) | Y | — |
| `release-adapters.yml` | — | — | Y | — |
| `release-providers.yml` | — | Y (`main`, paths providers) | Y | — |
| `release-binaries.yml` | — | — | Y | `release` published |
| `release-verify.yml` | — | — | Y | — |
| `publish-mcp-registry.yml` | — | — | Y (dispatch only, environment `mcp-registry`) | — |

## Notes

- `release.yml` uses bare `push: branches: [main]` for release-plz; branch
  CI push triggers use `branches` and `paths`. `certification.yml` and
  `nightly.yml` have `pull_request` paths without a branch filter.
- The `release-npm-wasm.yml` push trigger is tag-only; it does not validate
  a branch push.
- FIND-228's 2026-10-02 `push: [main]` policy is historical; see the
  2026-11-16 owner amendment in RULES.md §1. `perf-bench.yml`,
  `electron-node.yml` and `release-sbom.yml` are no longer present as
  workflow files. `ci-electron.yml` (DIST-21, 2026-11-28) is a **new** file,
  not a restoration: it loads the `vantadb-node` binding inside a real Electron
  process on 3 OSes and checks `asarUnpack`.
- `ci-web.yml` row removed 2026-10-02: the file was deleted in `82317140`
  (W-05 — `web/` extracted to `ness-e/Vantadb-web`).
- `lurkr-informational.yml` absorbed into `nightly.yml` (CI-08 Step 3,
  2026-10-15 — job `lurkr`); file deleted, no matrix row needed.
- CI-08 Steps 2-4 (2026-10-15) consolidated demos, informational jobs,
  documentation gates, fleet, certification and platform workflows. The
  matrix above reflects the files and triggers present now, not that snapshot.
- Schedule slots come from the `schedule` rows above; jobs may run only on a
  subset of their workflow's events.
- `ci-rust.yml` job `sanitizer-tsan` (FIND-737, 2026-11-28) runs on
  **`workflow_dispatch` only** — never on `push`/`pull_request`. The lane is
  heavy (it compiles an instrumented `std` with `-Zbuild-std`), so dispatching
  the workflow runs the whole gate plus that leg; there is no `schedule:` slot
  yet (procedure and debt: `docs/dev/operations/CI_POLICY.md` § "How the lane
  comes back").

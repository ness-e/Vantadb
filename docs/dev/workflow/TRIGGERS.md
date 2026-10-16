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
| `docs-gates.yml` | Y (`main`) | Y (`main`,`develop`) | Y (Mon 07:23) | Y | 18 jobs (CI-08 Step 4 + actionlint) |
| `ci-fleet.yml` | Y (`main`) | Y (`main`,`develop`) | Y (Sun 03:00) | Y | 5 jobs (CI-08 Step 4) |
| `perf-bench.yml` | Y (`main`,`develop`) | — | — | Y | — |
| `platform-ops.yml` | Y (`main`, desktop paths) | Y (`main`, desktop paths) | Y (Sun 00:00, daily 04:00) | Y | `issue_comment`, `review_comment`; 8 jobs (CI-08 Step 4) |
| `ci-gate.yml` | — | — | — | — | `workflow_call` |

## Demos (acceptance — F5/F6)

| Workflow | push | pull_request | schedule | dispatch | Other |
|----------|------|--------------|----------|----------|-------|
| `demo-acceptance.yml` | Y (`main`, union paths) | Y (`main`, union paths) | — | Y | 5 jobs (CI-08 Step 2) |

## Heavy / fuzz / informational / bots

| Workflow | push | pull_request | schedule | dispatch | Other |
|----------|------|--------------|----------|----------|-------|
| `certification.yml` | Y (`main`, chaos paths) | Y (paths, no `branches` — fuzz verbatim) | Y (daily 02:00, Sun 03:00, Mon 06:00) | Y | 21 jobs, ci-gate dedup ×3→1 (CI-08 Step 4) |
| `nightly.yml` | — | Y (paths info-only; heavy jobs skip PR) | Y (daily 05:00) | Y | 5 info jobs (CI-08 Step 3) |

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
  `branches` + `paths`; since FIND-228 every CI/demo/gate push scopes to
  `[main]` — pre-merge validation lives in `pull_request`. Post CI-08
  Step 4 hay una excepción adicional: `certification.yml` `pull_request`
  sin `branches` (preserva `fuzz.yml` verbatim — RULES §7 follow-up,
  igual que `nightly.yml` en Step 3).
- Tags never evaluate `branches`/`paths` (FIND-140): tag workflows keep a
  tags-only `push:` block; branch CI for the same area goes through `pull_request`.
- Dual `pull_request` → `develop` validation is intentional (FIND-128
  proposal item 3; gate-docs gap closed by FIND-139) and, post CI-08 Step 4,
  rides `docs-gates.yml` + `ci-fleet.yml` (union trigger conserva ambas ramas;
  los 6 workflows origen con dual validation viven ahi como jobs).
- FIND-228 (2026-10-02): 15 workflows dropped `develop` from
  `push.branches` (compliance with RULES.md §1; duplicate pair measured on
  PR #233: ~80 checks vs ~45 expected). Exception: `perf-bench.yml` keeps
  `push: [main, develop]` — dependency of the FIND-232 post-push
  verification (separate decision).
- `ci-web.yml` row removed 2026-10-02: the file was deleted in `82317140`
  (W-05 — `web/` extracted to `ness-e/Vantadb-web`).
- `lurkr-informational.yml` absorbed into `nightly.yml` (CI-08 Step 3,
  2026-10-15 — job `lurkr`); file deleted, no matrix row needed.
- CI-08 Steps 2-4 (2026-10-15): 5 demos → `demo-acceptance.yml`;
  4 informational → jobs in `nightly.yml` (40 → 32); resto 32 → 18 via
  `docs-gates.yml` (6→1), `ci-fleet.yml` (4→1), `certification.yml` (4→1,
  `ci-gate` ×3 dedup), `platform-ops.yml` (4→1). Estructura final:
  CI (`ci-rust`, `ci-gate`, `docs-gates`, `ci-fleet`, `demo-acceptance`,
  `nightly`, `perf-bench`) / Certificación (`certification`) /
  Release (9 `release-*` intactos, filenames/prohibit§BLOQUEO SRV-10).
  `release*.yml`, tag-namespaces y publish paths: intactos (verificado).
- Schedule slots (FIND-141 + HARD-02): bench daily 02:00, cert Sun 03:00,
  ocr-nightly daily 04:00, nightly subset daily 05:00, fuzz Mon 06:00,
  docs-links weekly Mon 07:23, release-verify weekly Mon 08:00 — no overlaps.

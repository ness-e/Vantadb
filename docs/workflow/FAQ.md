---
title: "Workflows — FAQ"
type: workflow-index
status: active
tags: [vantadb, ci, workflows, faq]
last_reviewed: 2026-09-22
aliases: []
related: ["docs/workflow/README.md", "docs/workflow/TRIGGERS.md", "docs/workflow/RUNBOOK.md"]
---

# Workflows — FAQ

## Why do I see two runs for the same commit (push + PR)?

By design of GitHub, not a bug (evidence: FIND-128 — same SHA `cb954abc`
produced a `push` run and a `pull_request` run 6 s apart).

- A `push` to `develop` with an open PR `develop → main` fires both events:
  `push` (ref `refs/heads/develop`) and `pull_request` (ref `refs/pull/N/merge`).
- `concurrency.group` includes `${{ github.ref }}`, so the two refs never
  cancel each other — you see a green/red pair for the same code.
- Mitigation (FIND-134): `ci-rust.yml` dropped `develop` from
  `push.branches`; PR validation covers pre-merge, `push: [main]` covers
  post-merge. Other dual-trigger workflows (`chaos`, `ci-examples`,
  `desktop`, `gate-docs`, …) still double-fire when paths match.

## What does `cancel-in-progress` mean here?

- `true` (CI/PR workflows): a new run on the same ref cancels the old one.
  Superseded PR pushes never waste runners.
- `false` (release/schedule/codeql): runs never cancel each other.
  Tag publishes (`cancel-in-progress: ${{ !startsWith(github.ref, 'refs/tags/') }}`)
  cancel branch runs but never cancel another tag run — two releases can
  overlap safely.
- The rustdoc "flake" (FIND-138) was this mechanism working: 5 runs of the
  same concurrency group cancelled within seconds, leaving 0 s jobs with
  empty logs. Not an infra failure.

## Skipped vs required checks — which is which?

- **Required** (branch protection on `main`/`develop`): the `ci-rust.yml`
  jobs (fmt, clippy, nextest matrix, ADR gate), `gate-docs.yml`,
  `providers-ci.yml`, CodeQL `Analyze`. A red required check blocks merge.
- **Fail-closed gate** (FIND-139): `ci-gate.yml` treats a missing or pending
  required check as failure (`*) FAILED=1`). Heavy nightly runs abort
  instead of silently passing over unknown CI state.
- **Skipped / informational** (never required): workflows whose every
  job/step has `continue-on-error: true` — `arch-metrics-informational.yml`,
  `bench-canonical-p99-informational.yml`, `ocr-delegate.yml`. They report
  `success` with failed steps inside; branch protection must not list them.
- **`needs: ci-gate` skips**: when the gate fails, downstream heavy jobs show
  `skipped`, not `failure`. Skipped-by-gate is a deliberate save, not a gap.

## Where do the durable rules live?

Short version here; normative rules go to `docs/workflow/RULES.md`
(FIND-144, after this task): triggers, timeouts, SHA pins, permissions,
publish policy, and good/bad examples per high-severity finding.
`docs/operations/CI_POLICY.md` refresh (26 → 27, real triggers) ships there too.

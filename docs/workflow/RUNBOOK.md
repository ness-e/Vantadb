---
title: "Workflows — Runbook"
type: workflow-index
status: active
tags: [vantadb, ci, workflows, runbook]
last_reviewed: 2026-09-22
aliases: []
related: ["docs/workflow/README.md", "docs/workflow/PUBLISH.md", "docs/workflow/FAQ.md"]
---

# Workflows — Runbook

Commands use GitHub CLI (`gh`). All publish environments are approval-gated;
releases wait for a reviewer — that is by design, not stuck.

## Re-run failed jobs only

```bash
gh run view <run-id> --failed
gh run rerun <run-id> --failed
```

- Never `gh run rerun` without `--failed` on release workflows: it rebuilds
  every matrix target (5 binaries, 9 adapters) and burns runners.
- Publishing rule (plan owner decision): extra care — rollback is
  `git revert` of the offending commit + re-run of the workflow, never a
  force-push of a tag.

## Approve a waiting environment (pypi / testpypi / npm)

Release jobs pause on `environment: pypi|testpypi|npm` until approved.
Two ways:

1. Web: run page → "Review deployments" → approve.
2. API (pending deployments):

```bash
gh api repos/{owner}/{repo}/actions/runs/<run-id>/pending_deployments \
  -f environment_ids[]=<env-id> -f state=approved -f comment="approved"
```

To find `<env-id>`, list environments first:

```bash
gh api repos/{owner}/{repo}/environments --jq ".environments[].name"
```

## `[no-adr]` marker

- The ADR gate (`ci-rust.yml` job `adr-gate`) fails PRs that change the
  public API surface without a new `docs/architecture/adr/ADR-*.md`.
- If the PR intentionally needs no ADR, put `[no-adr]` in the PR body:
  the gate downgrades to a warning instead of failing.
- Abuse (`[no-adr]` on real API changes) is caught in review, not by CI.

## `ci-gate` (heavy jobs skipped?)

- `heavy-certification.yml`, `heavy-bench-nightly.yml`, `fuzz.yml`
  call the reusable `ci-gate.yml` with `event_name`.
- On `schedule`, the gate checks the 11 required checks on `main` and
  fails the run if any is red (fail-closed, FIND-139).
- On `workflow_dispatch`, the gate always passes — manual runs force execution.

## First checks when CI looks wrong

1. `gh run list --branch <branch> --limit 5` — is it the run you think it is
   (push vs `pull_request` duplicates, see [FAQ.md](./FAQ.md))?
2. `gh run view <run-id> --job <job-id> --log --failed` — failing step log.
3. `gh api repos/{owner}/{repo}/commits/<sha>/check-runs` — what `ci-gate` saw.

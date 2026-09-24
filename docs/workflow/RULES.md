---
title: "Workflows — Durable rules"
type: workflow-index
status: active
tags: [vantadb, ci, workflows, rules, policy]
last_reviewed: 2026-09-22
aliases: []
related: ["docs/workflow/README.md", "docs/workflow/TRIGGERS.md", "docs/workflow/PUBLISH.md", "docs/workflow/RUNBOOK.md", "docs/workflow/FAQ.md"]
---

# Workflows — Durable rules

> **Scope:** `.github/workflows/` (27 files, names as on disk 2026-09-22) + `docs/workflow/*` + `docs/user/operations/CI_POLICY.md`
> **No tocar aquí:** engine code, bindings, release versioning (see release-ci rules); procedure lives in `RUNBOOK.md`, trigger matrix in `TRIGGERS.md`, publish chain in `PUBLISH.md`
> **Status:** 🟢 Vigente
> **Fuentes:** FIND-134/135/136/139/140/141/146 + renames FIND-142 (commit `97a3a03c`); matrix `TRIGGERS.md` (2026-09-22)

One verifiable rule per high-severity audit finding. Each rule states **Must / Must not / Why**, a good/bad example, and a mechanical check. If the check fails, the PR fails.

## Exception — filenames pinned by external trusted publishers (non-negotiable)

`release.yml`, `release-npm-61.yml`, `release-npm-node.yml` are **never renamed**. crates.io and npm Trusted Publishing bind OIDC to the filename; renaming breaks publish. Decreed by lead, recorded in FIND-142 (`97a3a03c`). All other workflows dropped their numeric suffix in that same commit.

## Rules

### 1 — Triggers: one push, one run; drop `develop` from `push` except where it belongs

- **Must:** scope every `push`/`pull_request` with `branches` + `paths` so pre-merge validation runs on PRs and post-merge runs on `main`.
- **Must not:** list `develop` under `push.branches` on CI workflows — an open PR `develop → main` fires `push` + `pull_request` for the same SHA (duplicate pair, see `FAQ.md`).
- **Por qué:** duplicates waste runners and hide the real signal; PR covers pre-merge, `push: [main]` covers post-merge.

```yaml
# BAD: duplicate pair on every push to develop with an open PR
on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

# GOOD (ci-rust.yml pattern): PR validates, main seals
on:
  push:
    branches: [main]
    paths: ["src/**", "Cargo.*"]
  pull_request:
    branches: [main]
    paths: ["src/**", "Cargo.*"]
```

- **Verify:** open a PR `develop → main`, push once → exactly 1 run per workflow when paths match (`gh run list --commit <sha>`).

### 2 — Timeouts: every job has `timeout-minutes`

- **Must:** set `timeout-minutes` on every job, calibrated to real duration + margin (FIND-135: values = measured wall time + headroom).
- **Must not:** leave a job without a timeout, including informational/bot jobs.
- **Por qué:** a hung runner burns minutes silently; the timeout is the kill-switch.

```yaml
# BAD
jobs:
  build:
    runs-on: ubuntu-latest
    steps: [...]

# GOOD
jobs:
  build:
    runs-on: ubuntu-latest
    timeout-minutes: 15
    steps: [...]
```

- **Verify:** `Select-String -Path .github/workflows/*.yml -Pattern "timeout-minutes"` covers every `jobs:` entry; no file without a hit except `ci-gate.yml` (reusable, inherits caller timeout).

### 3 — Pins: SHA, zero floating tags

- **Must:** pin every third-party `uses:` to a full commit SHA with a `# vX.Y.Z` comment (e.g. `actions/checkout@3d3c42e5... # v7.0.1`).
- **Must not:** use floating tags (`@v4`, `@main`, `@latest`) or `npm install -g <pkg>@latest` — pin the version (`1.12.9` pattern).
- **Por qué:** floating tags move under you; SHA is the only immutable reference.

```yaml
# BAD
- uses: actions/checkout@v4
- run: npm install -g ocr-review@latest

# GOOD
- uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
- run: npm install -g ocr-review@1.12.9
```

- **Verify:** `Select-String -Pattern "uses:.*@(v\\d|main|master|latest)"` returns 0 hits.

### 4 — Permissions: least privilege, OIDC before tokens

- **Must:** declare minimal `permissions:` per workflow/job (`contents: read` by default; `id-token: write` only where Trusted Publishing needs it; `contents: write` only on `release.yml` release-plz job).
- **Must not:** use broad `permissions: write-all`, long-lived PATs, or `secrets.*` for publishing — publish is 100% tokenless OIDC (`gh secret list` empty at repo level is correct).
- **Por qué:** least privilege bounds blast radius; OIDC removes secret rotation and leak surface.

```yaml
# BAD
permissions: write-all

# GOOD (publish job)
permissions:
  contents: write
  id-token: write # Required for trusted publishing (OIDC)
```

- **Verify:** every workflow has a `permissions:` block; `Select-String -Pattern "secrets\\."` on publish workflows returns 0 hits for publish steps.

### 5 — Publish: only where it must, gated, namespaced tags

- **Must:** publish only on its lane — `release.yml` on `push: branches: [main]`; wheels/npm/adapters/sbom on their tag namespaces (`v*.*.*`, `node-v*.*.*`, `adapters-v*.*.*`, `v*`); binaries on `release: [published]`; prod PyPI/npm behind `environment: pypi|npm` + OIDC + `version-exists`/`skip-existing` gate so re-runs never double-publish.
- **Must not:** mix branch CI and tag publish in one `push:` block (`tags` never evaluate `branches`/`paths` — branch CI for the same area goes through `pull_request`).
- **Por qué:** a mis-scoped trigger publishes from a branch or rebuilds artifacts only to discard them.

```yaml
# BAD: branch push can reach publish steps
on:
  push:
    branches: [main]
    tags: ["v*.*.*"]

# GOOD (release-npm-61.yml pattern): tags publish, PR validates
on:
  push:
    tags: ["v*.*.*"]
  pull_request:
    paths: ["vantadb-wasm/**", "vantadb-ts/**"]
```

- **Verify:** `TRIGGERS.md` release table matches each file's `on:` block; tag namespaces in `PUBLISH.md` (`v*.*.*` → wheels/npm, `node-v*.*.*` → npm-node, `adapters-v*.*.*` → adapters, `v*` → sbom, `release.published` → binaries).

### 6 — No silent failures: `continue-on-error` always carries a `CATEGORY`

- **Must:** tag every `continue-on-error: true` with an inline `# CATEGORY: EXPERIMENTAL | BEST-EFFORT | NON-CRITICAL | INFORMATIONAL` justification; informational workflows (`arch-metrics-informational.yml`, `bench-canonical-p99-informational.yml`, `ocr-delegate.yml`) keep it on every step and stay out of branch protection.
- **Must not:** add un-tagged `continue-on-error`, or `|| echo` / `|| true` swallowing real failures outside informational lanes.
- **Por qué:** untagged green hides red; the category tells the reader whether the failure is signal or noise.

```yaml
# BAD
- run: cargo fuzz --run 60
  continue-on-error: true

# GOOD
- run: cargo fuzz --run 60
  continue-on-error: true # CATEGORY: BEST-EFFORT - nightly corpus, never blocks merge
```

- **Verify:** every `continue-on-error: true` line has a `CATEGORY:` comment on the same or adjacent line; `actionlint` 0.

### 7 — Branch scoping: `pull_request` declares `branches` in a two-branch repo

- **Must:** declare `branches: [main]` (or `[main, develop]` where dual validation is intentional: `ci-rustdoc.yml`, `gate-docs.yml`) on every `pull_request:` trigger; PR-only informational workflows (`arch-metrics-informational.yml`, `bench-canonical-p99-informational.yml`, `ocr-delegate.yml`) and path-scoped lanes (`fuzz.yml`, `heavy-bench-nightly.yml`) document why they omit it.
- **Must not:** leave `pull_request:` bare without a reason — in a `main`/`develop` repo an unscoped PR trigger fires on branches that never merge.
- **Por qué:** explicit branches make the merge contract reviewable; bare triggers are implicit allow-lists.

```yaml
# BAD (in a main/develop repo, no intent recorded)
on:
  pull_request:
    paths: ["src/**"]

# GOOD
on:
  pull_request:
    branches: [main]
    paths: ["src/**"]
```

- **Verify:** `TRIGGERS.md` matrix `pull_request` column matches each file's `on:` block; intentionally unscoped rows carry a note (informational / path-scoped).

## See also

- [README.md](./README.md) — inventory (27 active)
- [TRIGGERS.md](./TRIGGERS.md) — full trigger matrix (source of truth is each file's `on:` block)
- [PUBLISH.md](./PUBLISH.md) — publish flow per registry + tag namespaces
- [RUNBOOK.md](./RUNBOOK.md) — re-run and approvals
- [FAQ.md](./FAQ.md) — duplicates, cancel-in-progress, skipped vs required
- `docs/user/operations/CI_POLICY.md` — Fast Gate vs Heavy split, coverage policy

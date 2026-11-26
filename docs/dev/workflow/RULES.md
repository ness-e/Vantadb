---
title: Workflows — Durable rules
kind: runbook
status: active
description: "One verifiable rule per high-severity audit finding. Each rule states Must / Must not / Why, a good/bad example, and a mechanical check. If the check fails, the PR fails"
tags: [vantadb, ci, workflows, rules, policy]
---

# Workflows — Durable rules

> **Scope:** `.github/workflows/` (27 files, names as on disk 2026-09-22) + `docs/dev/workflow/*` + `docs/dev/operations/CI_POLICY.md`
> **No tocar aquí:** engine code, bindings, release versioning (see release-ci rules); procedure lives in `RUNBOOK.md`, trigger matrix in `TRIGGERS.md`, publish chain in `PUBLISH.md`
> **Status:** 🟢 Vigente
> **Fuentes:** FIND-134/135/136/139/140/141/146 + renames FIND-142 (commit `97a3a03c`); matrix `TRIGGERS.md` (2026-10-02, post FIND-228); rule 1 completion: FIND-228 (2026-10-02); rule 8: HARD-03 (owner decision 2026-09-26 — push deferred to plan close, bundles as loss mitigation); rule 9: HARD-07 (review gate mechanized — `reviewer_context ≠ author_context`, 2026-09-27); rule 10: FIND-760 (pre-push judges the range, 2026-11-26); rule 10 advisory: FIND-754 (2026-11-26); rule 11: FIND-753 (private index + content verification, 2026-11-26)

One verifiable rule per high-severity audit finding. Each rule states **Must / Must not / Why**, a good/bad example, and a mechanical check. If the check fails, the PR fails.

## Exception — filenames pinned by external trusted publishers (non-negotiable)

`release.yml`, `release-npm-61.yml`, `release-npm-node.yml` are **never renamed**. crates.io and npm Trusted Publishing bind OIDC to the filename; renaming breaks publish. Decreed by lead, recorded in FIND-142 (`97a3a03c`). All other workflows dropped their numeric suffix in that same commit.

## Rules

### 1 — Triggers: validate integrated code on `develop` and `main`

- **Must:** scope CI `push`/`pull_request` with appropriate `branches` and `paths`; validate directly integrated `develop` code on push and post-merge `main` code on push.
- **Must not:** rely on `pull_request` alone to validate direct pushes to `develop`, or remove a `push` branch without checking the integration flow.
- **Por qué:** a direct push does not itself create a PR targeting `develop`; path filters still determine which checks run.

```yaml
# BAD: no CI signal for direct pushes to develop
on:
  push:
    branches: [main]
  pull_request:
    branches: [main, develop]

# GOOD (ci-rust.yml pattern): validate direct pushes and PRs
on:
  push:
    branches: [main, develop]
    paths: ["src/**", "Cargo.*"]
  pull_request:
    branches: [main, develop]
    paths: ["src/**", "Cargo.*"]
```

- **Verify:** inspect each workflow's `on:` and path filters; after a direct push to `develop`, check matching workflow runs for the SHA (`gh run list --commit <sha>`).

> **Amendment 2026-11-16 (owner):** el owner integra directamente en `develop` por push (bypass de admin admitido por el remoto), sin que un `pull_request` dirigido a `develop` dé señal de esa integración. Los CI que validan código integrado en `develop` listan esa rama en `push.branches` como excepción intencional: `ci-rust.yml`, `demo-acceptance.yml`, `docs-gates.yml`, `platform-ops.yml` y `ci-fleet.yml`. `perf-bench.yml` ya no existe en `.github/workflows/`. El defecto TS de `211d9d6bd` (FFI-02) permaneció 9 días en `origin/develop` sin CI de bindings. Con un PR `develop → main` abierto, un push puede producir un par push+PR si coinciden los filtros; la `concurrency` por `github.ref` cancela runs obsoletos de la misma ref, pero no deduplica necesariamente el push de rama y el PR, que tienen refs distintas.
>
> **Amendment 2026-10-02 (FIND-228), histórico:** los 15 workflows CI/demo/gate pasaron entonces a `push: [main]`, con `perf-bench.yml` como excepción. Esta enmienda ya no describe el estado real ni la política vigente.

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

### 8 — Local continuity: verified `git bundle` backups; releases ride fixed trains

- **Must:** keep an off-repo, verified backup current while work accumulates as local-only commits (push happens only on explicit owner instruction — `.opencode/AGENTS.md` Regla 7 §Política de git): run `pwsh scripts/git-backup.ps1` (default `$HOME\VantaDB-Backups`; `-Dest` for an external or synced private folder, `-Keep` for retention, default 7). The script is fail-closed — it only reports success after `git bundle verify` exits 0, and never overwrites an existing bundle.
- **Must:** treat releases as pre-committed trains, not ad-hoc bumps: **0.8.0 after F3** (schema cut, via release-plz after SCH-08) and **1.0** only when the HARD-01 exit criteria are met (`docs/api/COMPATIBILITY.md`, `docs/api/VERSIONING.md`). Version, tag and changelog stay 100% release-plz-owned.
- **Must not:** keep the only backup on the repo's own volume (single point of failure — the script warns), commit bundles to the repo, or restate the push/commit policy in this rule or `CONTRIBUTING.md` — the normative text lives in `.opencode/AGENTS.md` Regla 7; both cross-reference it.
- **Por qué:** with push deferred to the end of the plan, the working tree runs days ahead of any remote (31 local commits measured 2026-09-27) — R2 (GitLab 2017): backups that fail silently destroy weeks of work. A bundle is self-contained, offline-verifiable and restorable (`git clone <bundle>`); trains keep 0.8.0/1.0 from drifting into ad-hoc releases.

```powershell
# BAD: live copy of .git, same disk, never verified, no restore path
Copy-Item -Recurse . ..\VantaDB-copy

# GOOD: timestamped bundle, fail-closed verify, off-volume destination
pwsh scripts/git-backup.ps1 -Dest D:\VantaDB-Backups   # exit 0 only after `git bundle verify` exit 0
git clone D:\VantaDB-Backups\vantadb-20260927-0033.bundle C:\tmp\restore-test   # restorable
```

- **Verify:** `pwsh scripts/git-backup.ps1 -Dest <off-repo-folder>` exits 0 and `git bundle verify <bundle>` exits 0; `Select-String -Path docs/dev/workflow/RULES.md -Pattern 'git-backup.ps1'` finds rule 8; train milestones match `docs/dev/plans/2026-09-26-master-roadmap.md` §Gates por fase.

### 9 — Review gate: ACCEPT requires `reviewer_context ≠ author_context` (mechanized)

- **Must:** close a task (`campaign_update_task_state(completed)`) with a valid `review` payload in the recitation (HARD-07): `mode:'fresh'` with `reviewer_context ≠ author_context` (reviewer distinct from the implementer, P2-01) and `verdict:'approve'`; or `mode:'degraded'` **only** with an owner `waiver: {owner, ref}` — the waiver is registered in the trace (`review.waiver`) and `.opencode/task-system/memory/decisions.md`, never a silent bypass.
- **Must not:** let a degraded review without waiver reach ACCEPT (the write is blocked: `updated:false` + `reviewBlocked:true` with an actionable message), or pass `reviewer_context` equal to `author_context` (degradation in disguise).
- **Por qué:** API-09 incident — a degraded round (same context) was accepted while fresh subagent capacity existed and had to be fixed by hand; without mechanical enforcement the gate degrades silently (self-preference bias, arXiv 2404.13076). Source of truth: `validateReviewAccept` (`.opencode/task-system/config/state-tools.mjs`).

```jsonc
// BAD (degraded, no waiver — blocked by validateReviewAccept)
review: { mode: "degraded", verdict: "approve" }

// GOOD (fresh — reviewer in a distinct context)
review: { mode: "fresh", reviewer: "vanta-review", reviewer_context: "ses_abc", author_context: "ses_xyz", verdict: "approve" }

// GOOD (degraded — owner waiver registered)
review: { mode: "degraded", verdict: "approve", waiver: { owner: "Eros", ref: "2026-09-27 owner decision" } }
```

- **Verify:** `node --test .opencode/task-system/mcp/review-gate.test.mjs` → 5/5 (T1–T4 canónicos + T5 regresión del guard); waiver visible in `traces/<campaignId>.jsonl` (`review.waiver`) + `.opencode/task-system/memory/decisions.md`.
- **Rollout:** el enforcement vive en `campaign-server.mjs`; the running MCP process keeps the previous code — the gate activates on process restart.

### 10 — The pre-push gate judges the range that travels, not the working tree

- **Must:** scope a pre-push gate to the files the push carries (`git diff --name-only @{u}..HEAD`). `fmt` runs per range file (`rustfmt --check --edition 2021 --config skip_children=true <file>`); a gate that cannot be file-scoped (`check`, `clippy`, `audit`, `deny`, `nextest`) runs only when the range touches Rust, and keeps measuring the whole crate because a crate compiles as a unit and a test has no owning file.
- **Must not:** run a whole-tree formatter/linter as a **pre-push** gate (it blocks every lane on the slowest lane's WIP), or quietly drop a gate to make a push pass. Use the granular `SKIP=<gate>` and let the hook **name** the skipped gate — `SKIP` is ignored when `CI` is set.
- **Advisory (never a verdict):** a check whose subject is a **locally installed artifact** runs in pre-push as advisory — `scripts/check-cli-fresh.ps1 -Advisory` (`FIND-754`) prints nothing when the binary is absent or already matches `HEAD`, names both commits when it is stale, and prints the reason when it cannot read it; it never changes the exit code, and it is not called at all when `CI` is set (a runner has no `~/.cargo/bin`). Blocking there would stop a push for a binary the pusher may never have installed.
- **Por qué:** FIND-760, measured. The push of `FIND-152` (2 commits, both fmt-clean: `cargo fmt --check -p vantadb-wasm` → 0) was blocked by unformatted WIP **from another lane** (`vanta-memory/src/core/conversation/consolidation.rs`, `vantadb-mcp/src/memory_scheduler.rs`), because the hook ran `verify.ps1` on the working tree. The hook had no `SKIP`, so the only exits were `--no-verify` (kills every gate) or waiting — and 4 live lanes make every push hostage to the slowest one. Tree-wide `fmt` stays covered by CI (`ci-rust.yml:123`).

```sh
# BAD: measures the tree — another lane's WIP decides whether you can push
cargo fmt --all -- --check

# GOOD (.githooks/pre-push): measures what travels — the hook feeds the shas git
# gives it on stdin, which is right for new branches and multi-ref pushes too.
printf '%s\n' "$local_sha" | while IFS= read -r s; do
    git diff --name-only "$base..$s" | grep '\.rs$' | while IFS= read -r f; do
        rustfmt --check --edition 2021 --config skip_children=true "$f" || exit 1
    done
done
```

- **Verify:** `bash -n .githooks/pre-push` → 0; two-sided proof — (a) an unformatted `.rs` **outside** the range in the working tree → hook exits 0; (b) an unformatted `.rs` **inside** the range → hook exits ≠ 0 and names the file. `SKIP=<id>[,<id>…] git push` widens: a **single** id and a **comma list** must both name the skipped gate (a comma list that silently matches nothing is the same bug as no waiver at all — measured and fixed 2026-11-26).
- **Rollout:** `FIND-760` (2026-11-26).

### 11 — Commits go through a private index and are proven by content

- **Must:** a commit made while other lanes work on the same checkout uses a **private index** (exact command below) — never the shared `.git/index` — and **verifies the result by content** (`git show HEAD:<file>`), not by exit code.
- **Must not:** trust `git commit`'s exit code as evidence, or use `git commit -a` / a bare `git commit` with other lanes' work staged. Also: do **not** use the private index for `git add -p` interactive staging, for a merge commit (`MERGE_HEAD` must be in the index), or when the commit must carry what is already staged deliberately in the shared index — use `git commit -- <paths>` there. `--amend` is out of scope.

```powershell
# GOOD: private index, named paths only, then a content check, then sync the index
$idx = "$env:TEMP\opencode\idx-$(Get-Random)"       # your file, no globbing of others'
$env:GIT_INDEX_FILE = $idx
git read-tree HEAD
git add -- docs/dev/Backlog.md .githooks/pre-push
git commit -m "fix(ci): …"
Remove-Item Env:GIT_INDEX_FILE
Remove-Item $idx -ErrorAction SilentlyContinue      # the temp index FILE, not just the env var
git show HEAD:docs/dev/Backlog.md | rg 'FIND-760'   # expect 0 hits, not just exit 0
git add -- docs/dev/Backlog.md .githooks/pre-push   # REQUIRED: sync the shared index
# or, one step (does all of the above): pwsh dev-tools/git-commit-isolated.ps1 -Message "…" -Paths … -ExpectAbsent docs/dev/Backlog.md::FIND-760

# GOOD (shared file that another lane is ALSO editing): take the content from HEAD
# plus your edit — never the possibly-dirty working copy (that is how a stale copy
# gets baked in).
git show HEAD:docs/dev/Backlog.md > $env:TEMP\b.md   # edit $env:TEMP\b.md, then:
$blob = (git hash-object -w $env:TEMP\b.md).Trim()
$env:GIT_INDEX_FILE = "$env:TEMP\opencode\idx-$(Get-Random)"; git read-tree HEAD
git update-index --cacheinfo "100644,$blob,docs/dev/Backlog.md"
git commit -m "fix(ci): …"

# BAD: shared index — whatever another lane staged rides along, and a concurrent
# commit can bake a stale copy of a shared file and revert your edit WITH EXIT 0
git commit -a -m "fix(ci): …"
```

- **Por qué:** FIND-753, measured three times in one day. (1) `Unable to create .git/index.lock` on 8 consecutive attempts with 4 lanes; (2) a concurrent commit baked a **stale** `docs/dev/Backlog.md` and a row another lane had just deleted **reappeared in HEAD** — detected by re-reading the fragment from HEAD (exit code was 0); (3) the index held another lane's pre-staged `tests/cli_tests.rs` (−92 lines), so a `git commit -a` would have deleted committed work. **The trap is the exit code:** a reverting commit succeeds.
- **Por qué el `git add` final no es cosmético (medido al cerrar esta misma regla, `cff8fa626`):** con `GIT_INDEX_FILE`, `.git/index` **no se actualiza**; justo después del commit, `git diff --cached` mostró el commit entero como staged-para-revertirse. Cualquier `git commit` sin pathspec de otro carril lo habría revertido. Es el mismo fallo que esta regla previene, causado por la herramienta que lo previene.
- **Un blob derivado de `HEAD` es TOCTOU — verificá el DELTA, no solo tus fragmentos (`cff8fa626`):** ese commit construyó `Backlog.md` como `HEAD` + su edición, pero otro carril commiteó entre construir y commitear: el resultado horneó la copia anterior y **borró 9 líneas ajenas** (`FIND-761`, recién creada) además de las propias. La verificación por fragmento **pasó** porque solo miraba las marcas propias. Los dos chequeos que lo cazan:
  - `git rev-parse <commit>^` == el `HEAD` con el que construiste el blob (si no, otro carril commiteó en medio: rehacé el blob desde el nuevo `HEAD`);
  - `git diff <base>..HEAD -- <fichero>` contiene **solo tus hunks** y **0 adiciones** cuando tu cambio es puramente de borrado. `git diff --numstat` lo dice en una línea (`0  13  docs/dev/Backlog.md`).
  - Reparación medida en `30fdc9c42` (9 líneas restauradas, 0 adiciones). Y ojo con el inverso: `git hash-object <worktree>` == `git rev-parse HEAD:<path>` solo prueba que el worktree coincide con HEAD, **no** que tu commit no haya revertido a otro.
- **Verify:** the content check returns 0 hits for a fragment you removed and ≥1 for one you added; a deliberately wrong `-ExpectAbsent` makes `dev-tools/git-commit-isolated.ps1` exit 1; and `git diff --cached -- <your paths>` is **empty** right after the commit (if it is not, the shared index is stale — run the `git add` above).
- **Techo declarado:** the private index shrinks the race window to `add`+`commit` (milliseconds) and **detects** a reversal; it does not **prevent** one. Prevention needs a wave-wide lock, which does not exist. `FIND-753` (2026-11-26).

## See also

- [README.md](./README.md) — inventory (27 active)
- [TRIGGERS.md](./TRIGGERS.md) — full trigger matrix (source of truth is each file's `on:` block)
- [PUBLISH.md](./PUBLISH.md) — publish flow per registry + tag namespaces
- [RUNBOOK.md](./RUNBOOK.md) — re-run and approvals
- [FAQ.md](./FAQ.md) — duplicates, cancel-in-progress, skipped vs required
- `docs/dev/operations/CI_POLICY.md` — Fast Gate vs Heavy split, coverage policy

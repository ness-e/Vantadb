---
title: F0.3 — Publication hygiene for v0.10.0
kind: task
status: active
description: Close four publication gaps that make the public repo and the npm package look incomplete or expose personal data
tags: [release, npm, security, docs]
---

# F0.3 — Publication hygiene for v0.10.0

Plan: [`docs/dev/plans/2026-11-23-release-v0.10.0-gates.md`](../plans/2026-11-23-release-v0.10.0-gates.md) §4, Fase 0.
Wave 0.2. No dependencies. Owner role: `vanta-docs`.

## Problem

The repository is public since `b0d740e32` (when `docs/` became tracked). Four
gaps travel to the reader or to the npm registry:

| # | Gap | Surface |
|---|-----|---------|
| a | `vantadb-node/package.json` has no `homepage`, `bugs`, `keywords` | npm registry page |
| b | Personal email `developererosnessy@gmail.com` in the security contact | public repo + published docs |
| c | MSRV claim drift (`1.94.1` vs `rust-version = "1.95.0"`) | skill file shipped in the repo |
| d | `vantadb.dev` declared as `homepage` in `vantadb-ts` but the zone does not resolve | npm registry page |

## Impacto mapeado (Regla 0)

Files read in full before editing, references inward, references outward.

### (a) `vantadb-node/package.json`

- Read in full: 67 lines (name, version, description, repository with
  `directory: vantadb-node`, type/main/module/types/exports/files/scripts/
  devDependencies/napi/engines/os/cpu/publishConfig/license).
- Read in full: `vantadb-ts/package.json` (84 lines) — the sibling that already
  carries `homepage`, `bugs`, `keywords`. Criterion copied from it, not invented.
- Inward references: `vantadb-ts/package.json` depends on `vantadb-node` via
  `file:../vantadb-node`; `.github/workflows/release-npm-node.yml` publishes it;
  `vantadb-node/README.md` exists (so `files` already ships a README).
- Outward references: `repository.url` → `git+https://github.com/ness-e/Vantadb.git`.
- Veredicto: **additive only**. Three new keys, no existing key touched, no
  build/test/`napi` change. Nothing reads these keys at build time (`napi` CLI
  reads `napi`/`files`/`main`); they are registry metadata only.

### (b) Security contact

- Read in full: `.github/SECURITY.md` (28 lines).
- Read context of `docs/user/operations/SECURITY.md` around the report section.
- Read context of `docs/user/operations/DISASTER_RECOVERY_RUNBOOK.md` §7.
- Inward references: `.github/SECURITY.md` is the GitHub-recognised security
  policy surface; it links to `docs/user/operations/SECURITY.md`.
- Outward references: both SECURITY.md files point at
  `https://github.com/ness-e/Vantadb/security/advisories`.
- Veredicto: **replace the email channel with the channel already declared
  first** (GitHub Security Advisories). No new domain invented: `security@`
  does not exist (`docs/dev/strategy/VantaDB-Informe-Analisis-Completo.md:496`
  records `security@/enterprise@ ❌ dominio muerto`).

### (c) MSRV

- Read `Cargo.toml:822` → `rust-version = "1.95.0"` (source of truth).
- `rg '1\.80|1\.94\.1' --glob '*.md' --glob '!docs/dev/**'` → 4 hits, of which
  2 are numeric false positives in `docs/user/operations/BENCHMARKS.md`
  (`179.810 ms`, `$451.80`), 1 is the historical `CHANGELOG.md:722` entry
  (release-plz-owned, historical record), and 1 is a live claim:
  `skills/vantadb/SKILL.md:782` `Rust Minimum: 1.94.1`.
- Veredicto: fix the live claim only. No `1.80` claim exists anywhere in the
  repo — that one lives in Notion (out of scope, see §Incógnitas).

### (d) `vantadb.dev`

- Measured 2026-11-24: `curl -I https://vantadb.dev` → `curl: (6) Could not
  resolve host: vantadb.dev`; `Resolve-DnsName vantadb.dev` → no records.
- Veredicto: **not usable** as a homepage. Use the GitHub repository URL that
  npm's own `homepage` example documents.

## SDP

`campaign_discover_skills_v2` phase=BUILD, archivosClave
`vantadb-node/package.json,.github/SECURITY.md,docs/user/operations/DISASTER_RECOVERY_RUNBOOK.md,docs/user/operations/SECURITY.md`,
contractKeywords `["npm metadata","security contact","MSRV","homepage"]`.

SDP: `documentation-skill` (mandatory, docs touched), `documentation-and-adrs`,
`writing-guidelines`, `source-driven-development` (npm spec cited),
`git-workflow-and-versioning` (pinned, release metadata), `ci-cd-and-automation`
(pinned), `incremental-implementation`.

## Steps

| # | Step | Verify |
|---|------|--------|
| 1 | Add `homepage`, `bugs`, `keywords` to `vantadb-node/package.json` | `node -e` contract (3 keys present) |
| 2 | Remove the personal email from `.github/SECURITY.md` | `rg developererosnessy .github/SECURITY.md` = 0 |
| 3 | Remove it from `docs/user/operations/SECURITY.md` | `rg` = 0 |
| 4 | Replace it in `docs/user/operations/DISASTER_RECOVERY_RUNBOOK.md` | `rg` = 0 |
| 5 | Fix `skills/vantadb/SKILL.md` MSRV claim → 1.95.0 | `rg 'Rust Minimum'` = `1.95.0` |
| 6 | Docs gates + `npm pack --dry-run` | all 4 gates green, fields in tarball metadata |

## Incógnitas

1. **`vantadb.dev` does not resolve** (measured 2026-11-24, `curl (6) Could not
   resolve host`). `vantadb-ts/package.json:46` still declares it as `homepage`.
   Not fixed here: it is a product/GTM decision (**gate humano `GTM-05`**;
   related backlog item `PUB-01` in `docs/dev/reviews/PLAN-ACCION-UNIFICADO.md:36`).
   To close it: register the domain and point it at the docs, or change
   `vantadb-ts/package.json` `homepage` to `https://github.com/ness-e/Vantadb#readme`.
   Unblock command: `curl -sS -I https://vantadb.dev` returning HTTP 200.
   **Resolved by the orchestrator (commit `29cbcbcf1`)**: `vantadb-ts/package.json:46`
   now declares `homepage: "https://github.com/ness-e/Vantadb#readme"` — the same
   criterion applied here. Only the domain itself and the `security@` alias
   remain open (see item 2).
2. **No `security@` alias exists.** The email channel was dropped in favour of
   the GitHub Security Advisories route already declared in both `SECURITY.md`
   files. Creating `security@vantadb.dev` is **pending on the owner** and blocked
   on the same dead zone as (1). Unblock: a working alias, then re-add the
   `**Email:**` bullet.
3. **The `Rust 1.80+` claim lives in Notion**, not in the repo. `rg '1\.80'`
   over all tracked `.md` outside `docs/dev/` returns zero real hits. The Notion
   page to correct is the one referenced by the plan (`Plan de accion` /
   `Nuevas features`); no repo file cites it, so there is no repo path to fix.
   This is the owner's task per the plan.

## Deuda

**Rule learned (skills mirror, `FIND-83`):** every edit to
`skills/vantadb/SKILL.md` must be copied verbatim to
`.opencode/skills/vantadb/SKILL.md` **in the same commit**, and
`scripts/validate-docs-coverage.ps1` must run before closing. The `1.94.1` →
`1.95.0` fix left the mirror hash divergent and the gate at **1 gap**
("skills mirror 1/10 drift"); the orchestrator resynced the mirror and the gate
is back to **0 gaps**. Same rule applies to any other mirrored skill file.

`check-msrv-claim.mjs` (proposed, not implemented — out of scope): a docs gate
that reads `rust-version` from `Cargo.toml` and fails when any tracked `.md`
outside `docs/dev/` and `CHANGELOG.md` states a different Rust minimum. Cost:
~40 lines + one line in `docs-gates.yml` (~30 min). It would have caught
`skills/vantadb/SKILL.md:782`, which survived because nothing lints a version
claim. Belongs in a `FIND-*` row, not in this task.

## Verification

| Command | Result |
|---------|--------|
| `node -e "…for(const k of ['homepage','bugs','keywords'])…"` | ✅ `ok` |
| tarball check: `npm pack --pack-destination <tmp>` → `tar -xOf … package/package.json` | ✅ `TARBALL OK: https://github.com/ness-e/Vantadb#readme \| https://github.com/ness-e/Vantadb/issues \| 13 keywords` |
| `npm pkg get homepage bugs keywords` | ✅ the three values |
| `rg -n 'developererosnessy@' .github/SECURITY.md docs/user/operations/SECURITY.md docs/user/operations/DISASTER_RECOVERY_RUNBOOK.md` | ✅ 0 hits (exit 1) |
| `rg -n '1\.80\|1\.94\.1' --glob '*.md' --glob '!docs/dev/**'` | ✅ only `CHANGELOG.md:722` (historical, release-plz-owned) and 2 numeric false positives in `BENCHMARKS.md` (`191.805`, `$451.80`) |
| `node scripts/docs/check-links.mjs` | ✅ 0 broken |
| `node scripts/docs/check-docs.mjs` | ✅ all clear |
| `node scripts/docs/check-secrets.mjs` | ✅ 0 errors |
| `node scripts/docs/gen-index.mjs --check` | ✅ 0 stale |
| `pwsh dev-tools/verify_changed.ps1` | ✅ ALL 4 PASS |
| `curl -I https://vantadb.dev` | ❌ does not resolve (see §Incógnitas 1) |
| `curl -L https://github.com/ness-e/Vantadb#readme` / `/issues` / `/security/advisories` | ✅ 200 / 200 / 200 |
| `pwsh dev-tools/ocr-review.ps1` | ✅ no Critical/High; the two JSON files match their rule groups (no new dependency, no `latest`/`*`, no duplicate declaration) |

`docs/index.md` and `llms.txt` were regenerated (`gen-index.mjs --write`) because
this task file enters the index. **The regenerated index also carries the entry
for `docs/dev/tasks/FIND-701.md`**, written concurrently by the parallel F0.1
session; that file exists on disk and both commits must land together for the
index link to stay valid.

## Review (P2-01)

**Tier:** Fast — paths touched are `vantadb-node/package.json`,
`.github/SECURITY.md`, `docs/**`, `skills/**`. No path matches the adversarial
glob list. Diff is 3 metadata keys + 3 contact edits + 1 version string + 1
generated index.

**Mode: DEGRADED** — this session is a sub-agent and cannot spawn a distinct
reviewer (`subagent: deny` on `vanta-docs`). Recorded as degraded, not as a
fresh-context review. Escalated to the orchestrator: run `vanta-review` over
`docs(core)` commit before the release PR if a fresh pair of eyes is required.

Adversarial passes actually performed:

1. **"An invented `homepage` that does not resolve."** `https://github.com/ness-e/Vantadb#readme`
   → HTTP 200 measured, and it is the exact shape npm documents as the
   `homepage` example (`https://github.com/npm/example#readme`). Rejected
   `https://vantadb.dev` because `curl (6) Could not resolve host`.
2. **"Deleting the email breaks the disclosure flow."** Both SECURITY.md files
   keep a working channel (GitHub Security Advisories, HTTP 200, private by
   design). The email was the *second* channel, not the only one.
3. **"Wrong repo casing in the URL."** `Vantadb` is what `git remote -v` says;
   the `VantaDB` spelling in `.github/SECURITY.md` redirects. Not changed
   (out of scope, both 200).
4. **"MSRV claim is still wrong."** `rust-version = "1.95.0"` (Cargo.toml:822)
   and `rust-toolchain.toml channel = "1.95.0"` agree. The claim now matches
   both. Searched for a hidden third source of the number — none.
5. **"`keywords` is filler that pollutes npm search."** 13 terms, all either
   already used by `vantadb-ts` for the same product or genuinely describing
   this artefact (`nodejs`, `napi`, `native-addon`).

**Verdict: approve** (degraded review).

## Context Save Point

Complete. All four gaps addressed; two of them (security alias, `vantadb.dev`)
are blocked on decisions outside this role and are recorded in §Incógnitas.
`docs/pipeline-state.json` changed by another session — not staged.

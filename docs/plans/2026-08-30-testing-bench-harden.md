# Implementation Plan: Testing & Benchmarking Hardening (audit 2026-08-30)

**Date:** 2026-08-30
**Phase:** P39 (new)
**Origin:** Multi-agent audit (5 sub-agents) on 2026-08-30 — full digest in session `ses_fabf69692ffeP5c7mycKcsGSV0` + summary in `docs/research/audit-testing-benchmarks-2026-08-30.md`.
**Owner:** vanta-lead (release/CI orchestrator)
**Scope:** 23 actions derived from the audit; ALTA (5) + MED (10) + BAJA (8). All uncontested except strategy decisions resolved with user on 2026-08-30.

---

## Overview

VantaDB has a mature testing ecosystem (2034 tests, 19 criterion benches, 4 libFuzzer targets, Miri+ASan+TSan, llvm-cov at 81.40% root). The audit uncovered **gaps that break the "replicable y funcional siempre que se pruebe" promise**: certification tests silently skip when datasets are missing (Recall@10 can degrade undetected), the criterion regression baseline is empty, the ci-gate doesn't cut PRs over red main, and 6 workflows don't listen to the `develop` branch (Dependabot target mismatch). This plan hardens the verification pipeline across **CI/CD, datasets, benchmarks, tests, and scripts**.

---

## Architecture Decisions (resolved with owner 2026-08-30)

| # | Decision | Rationale |
|---|----------|-----------|
| **D1** | **Conservative dataset/benchmark strategy.** No migration to VIBE, no PQ/i8 HNSW, no head-to-head vs sqlite-vss/duckdb-vss, no `divan` alongside criterion. | ann-benchmarks is still functional; comparisons aren't apples-to-apples; criterion covers the use case. Defer the 4 open-questions to a future audit. |
| **D2** | **Implement all 23 fixes** in this plan (ALTA + MED + BAJA). | User explicit: "TODO (23 fixes)". Sprint scope = all 3 tiers. |
| **D3** | **`ci-gate.yml` universal gate** — remove `if: inputs.event_name == 'schedule'`. | Aligns the implementation with the documented contract (gate cuts all runs over red main). |
| **D4** | **TS SDK broken tests deferred** — create a separate `ISSUE-TS-001` ticket, do NOT block this plan. | Belongs to bindings layer (vanta-worker territory). Pre-existing issue, not a regression introduced by this plan. |
| **D5** | **No new dependencies without explicit need**: only add `insta` (1.48.0), `cargo-mutants` (27.1.0), `dhat` (0.3.3, --features gated). | Each must justify its weight. Existing proptest + criterion + Miri + ASan/TSan already cover most gaps. |
| **D6** | **Branch strategy**: PRs target `develop` (Dependabot config), 6 workflows add `develop` to push triggers. `main` stays gated for release-plz. | Aligns with `.github/dependabot.yml` `target-branch: develop`; fixes the Dependabot/CI mismatch. |
| **D7** | **Reuse `dev-tools/gate-common.ps1`** as the canonical source of features for any new verify script. | Avoid drift; single source of truth already established. |

---

## Task List

### Phase 1 — ALTA: Core Correctness (5 tasks, ~2-3 days)

> These break the "replicable y funcional" promise. Ship first, gate the rest on a green PR for this phase.

- [ ] **TASK-01 — `verify_datasets.{sh,ps1}` + CI gate** (gate-ci)
  - Detect which `tests/certification/*` would skip without downloaded datasets
  - Exit 1 when ≥1 expected dataset is missing
  - Hook into `heavy-certification-50.yml` as a pre-test step
  - Files: `scripts/verify_datasets.sh`, `scripts/verify_datasets.ps1`, `.github/workflows/heavy-certification-50.yml`
  - Verify: `bash scripts/verify_datasets.sh` fails when `data/` is empty; passes when downloads complete

- [ ] **TASK-02 — Initialize `benchmarks/criterion_baseline.json`** (bench-baseline)
  - Run `cargo bench --workspace` once locally (or via CI commit)
  - Pipe through `scripts/bench_regression.py update-baseline benchmark_report_criterion.json`
  - Commit the populated baseline + `target/criterion/` output is gitignored (already is)
  - Files: `benchmarks/criterion_baseline.json` (content change), `scripts/bench_regression.py` (verify)
  - Verify: `heavy-bench-nightly-51.yml` now has a non-empty baseline to compare against

- [ ] **TASK-03 — Fix `ci-gate.yml` gate effectiveness** (ci-gate-fix)
  - Remove `if: inputs.event_name == 'schedule'` line
  - Universal gate: cuts all workflow_call invokers (fuzz, heavy-cert, heavy-bench) on red main
  - Files: `.github/workflows/ci-gate.yml` (1-line change)
  - Verify: Trigger a heavy-cert run on a known-red main; gate should fail

- [ ] **TASK-04 — Add `develop` to 6 workflows + `release.yml`** (branch-coverage)
  - Add `develop` to `push.branches` in: `ci-rust-10.yml`, `ci-web-11.yml`, `gate-docs-21.yml`, `ci-examples-12.yml`, `chaos-45.yml`, `perf-bench-40.yml`
  - Same for `release.yml` so release-plz opens PRs on `develop` too
  - Files: 7 workflow files (1-line config each)
  - Verify: A Dependabot PR on `develop` triggers each workflow's CI

- [ ] **TASK-05 — `.gitignore` benchmark artifacts** (gitignore-cleanup)
  - Add `benchmarks/data_comp_bench/`, `benchmarks/data_bench_db/` to `.gitignore`
  - `git rm --cached` to untrack existing tracked copies
  - Files: `.gitignore`, `git rm --cached benchmarks/data_comp_bench/...` `benchmarks/data_bench_db/...`
  - Verify: `git check-ignore benchmarks/data_comp_bench/` returns exit 0

### Checkpoint: Phase 1
- [ ] `just verify` passes locally
- [ ] Heavy-cert workflow fails-fast when datasets missing
- [ ] Bench nightly produces regression diff vs populated baseline
- [ ] No `# ponies` introduced without docs

---

### Phase 2 — MED: Quality & Coverage (10 tasks, ~1-2 weeks)

> Build incrementally. Each task independent. Can parallelize across multiple sub-agents.

- [ ] **TASK-06 — Add `insta` snapshots** (snapshot-tests)
  - Add `insta = "1.48"` to `[dev-dependencies]` in workspace root `Cargo.toml`
  - Migrate 3 parser tests + 2 query-result tests to `insta::assert_snapshot!`
  - Add `cargo-insta` to pre-commit (or as optional CI step)
  - Files: `Cargo.toml`, `tests/logic/parser.rs`, `tests/.../query_result*.rs` (5 files)
  - Verify: `cargo test --features test-bench-datasets` runs snapshots; first run generates `.snap.new` for review

- [ ] **TASK-07 — Configure `cargo-mutants` weekly job** (mutation-testing)
  - Add new job to `heavy-certification-50.yml`: `mutation-test` running `cargo mutants --check -p vantadb --timeout 120`
  - Gated on `workflow_dispatch` + `schedule weekly`
  - Surface "mutation score" in `benchmarks/mutation_score.json` artifact
  - Files: `.github/workflows/heavy-certification-50.yml`
  - Verify: Job completes with a non-zero mutation score; report artifact exists

- [ ] **TASK-08 — `benches/wal_throughput.rs`** (bench-wal)
  - Use `benches/common::synthetic_vectors`
  - Sweep: WAL on/off, fsync on/off, batch sizes [1, 100, 1k, 10k]
  - Output: ops/sec + p50/p95/p99 latencies via criterion
  - Files: `benches/wal_throughput.rs`, `Cargo.toml` (add `[[bench]]` block)
  - Verify: `cargo bench --bench wal_throughput` runs; results in `target/criterion/wal_throughput/`

- [ ] **TASK-09 — `benches/crash_recovery.rs`** (bench-recovery)
  - Open + WAL replay cronometrado; sweep corpus size [100, 10k, 100k]
  - Files: `benches/crash_recovery.rs`, `Cargo.toml`
  - Verify: `cargo bench --bench crash_recovery` runs

- [ ] **TASK-10 — Convert `bench_concurrent.rs` to `criterion_main!`** (bench-concurrent-fix)
  - Replace `fn main()` + custom `Instant::now()` with `criterion_group!` + `criterion_main!`
  - Files: `benches/bench_concurrent.rs`
  - Verify: `estimates.json` is now generated; `scripts/bench_regression.py` parses it

- [ ] **TASK-11 — Extend `heavy-bench-nightly-51.yml` to 8 benches** (bench-nightly-coverage)
  - Add `canonical_p99`, `memory_budget`, `incremental_bench`, `ivf_bench` to the nightly matrix
  - Keep `light-benchmarks` and `high-density` jobs
  - Files: `.github/workflows/heavy-bench-nightly-51.yml`
  - Verify: Job runs nightly and picks up the 8 benches

- [ ] **TASK-12 — `data/README.md` + `datasets/README.md`** (data-docs)
  - Create both files with a table: dataset name | source URL | license | size | SHA256 | download command
  - Cross-link from `scripts/download_*.sh` and `dev-tools/scripts/download_*.py`
  - Files: `data/README.md`, `datasets/README.md`
  - Verify: `ls data/README.md datasets/README.md` succeeds; both render correctly

- [ ] **TASK-13 — SHA-pin remaining workflows** (sha-pin-fix)
  - Pin `actions/checkout`, `actions/setup-node`, `tauri-action` in `desktop.yml` (3 refs)
  - Pin `actions/checkout@v6` + `opencode` action in `opencode.yml` (2 refs)
  - Files: `.github/workflows/desktop.yml`, `.github/workflows/opencode.yml`
  - Verify: `grep -E '@(v[0-9]+|latest)' .github/workflows/*.yml` returns only documented exceptions

- [ ] **TASK-14 — Fix `cliff.toml` `conventional_commits = true`** (cliff-fix)
  - Change line 15 from `false` → `true` to match the `commit_parsers` config
  - Files: `cliff.toml`
  - Verify: `git cliff --unreleased --dry-run` produces correctly-grouped output for a test commit

- [x] **TASK-15 — Consolidate `scripts/audit-tokens.{sh,ps1}`** (audit-tokens-consolidate) ✅
  - Decision: delete `.sh` (keep `.ps1`) — executed 2026-08-30
  - No `Justfile`/`dev-tools`/`workflows` references found (clean)
  - Files: bash variant (deleted), PowerShell variant (preserved)
  - Verify: grep for the bash filename → 0 matches ✅

### Checkpoint: Phase 2
- [ ] Snapshot tests catch at least 1 regression in a synthetic test
- [ ] Mutation score ≥70% on `vantadb` root crate
- [ ] WAL + recovery benches produce data in `target/criterion/`
- [ ] Nightly bench run covers 8+ benches

---

### Phase 3 — BAJA: Optional Hardening (8 tasks, ~3-4 days)

> Nice-to-have. Lower risk, can ship out-of-order.

- [ ] **TASK-16 — Evaluate `divan` alongside criterion** (divan-eval)
  - Add `divan = "0.1"` as optional `[dev-dependencies]`
  - Port 1 micro-bench (e.g. `tokenizer_bench`) to divan to compare DX
  - Decide keep/remove based on iteration speed + `AllocProfiler` utility
  - Files: `Cargo.toml`, `benches/divan_tokenizer.rs` (or similar)

- [ ] **TASK-17 — Evaluate `loom` for new concurrency primitives** (loom-eval)
  - Skip if no new concurrency primitives planned this sprint
  - Document decision in `docs/research/concurrency-testing-2026-08-30.md`

- [ ] **TASK-18 — Evaluate `dhat` for heap-usage testing** (dhat-eval)
  - Add `dhat = "0.3"` behind `--features dhat-heap`
  - Add 2 heap-usage asserts to existing tests (e.g. `tests/memory/alloc.rs`)
  - Files: `Cargo.toml`, `tests/memory/...` (new or extended)

- [ ] **TASK-19 — Markdownlint pre-commit hook** (pre-commit-md)
  - Add `markdownlint-cli2` hook mirroring `gate-docs-21.yml`
  - Files: `.pre-commit-config.yaml`
  - Verify: `pre-commit run --all-files` lints `docs/**/*.md`

- [ ] **TASK-20 — `ci-examples-12.yml` Windows+macOS matrix** (ci-examples-matrix)
  - Add `windows-latest` and `macos-latest` to the examples matrix
  - Files: `.github/workflows/ci-examples-12.yml`
  - Verify: PR triggers 3 OS jobs (ubuntu, windows, macos)

- [ ] **TASK-21 — Document `CoverageThreshold=60` review cadence** (coverage-policy)
  - Add entry to `CI_POLICY.md` documenting review schedule (e.g. quarterly)
  - Update `verify.ps1` `coverageThreshold=60` to source from policy
  - Files: `CI_POLICY.md`, `dev-tools/verify.ps1` (or `gate-common.ps1`)

- [ ] **TASK-22 — `release-binaries-63.yml` add `push tags v*`** (release-binaries-tags)
  - Add `push.tags: ['v*']` trigger alongside `release published`
  - Files: `.github/workflows/release-binaries-63.yml`

- [ ] **TASK-23 — Unify `cargo fmt --all` scope in Justfile + verify.ps1** (fmt-scope)
  - Decide target scope (Justfile `--all`? verify.ps1 drop `--all`?) and apply consistently
  - Document in `dev-tools/gate-common.ps1` as canonical
  - Files: `Justfile`, `dev-tools/verify.ps1`, `dev-tools/audit-all.ps1`

### Checkpoint: Phase 3
- [ ] All 23 tasks closed
- [ ] `docs/avance/activo/testing-bench-harden.md` updated with final state
- [ ] Plan file archived to `docs/plans/archive/2026-08-30-testing-bench-harden.md`

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| `insta` snapshots generate noise on first run | Med | Use `INSTA_UPDATE=no` for CI; commit `.snap` files manually after review |
| `cargo-mutants` is slow (>2h on full workspace) | High | Scope to root crate `-p vantadb` only initially; expand incrementally |
| `ci-gate` change cuts CI more aggressively | Med | Communicate in PR; revert if too noisy after 1 week |
| WAL bench writes to actual disk; CI disk fill | Med | Use `tempfile::TempDir` for benches; clean up in `Drop` |
| TS SDK broken tests not addressed | Low | Separate ticket `ISSUE-TS-001`; out of scope for this plan |
| `verify_datasets.sh` may false-positive on optional datasets | Med | Whitelist known-optional tests; only count `tests/certification/*` strictly |

---

## Open Questions / Out of Scope

- **ann-benchmarks → VIBE migration**: deferred to future audit (D1)
- **PQ/i8 in HNSW**: deferred (D1)
- **Head-to-head benchmarks vs sqlite-vss/duckdb-vss**: deferred (D1)
- **TS SDK fix (80/219 broken tests)**: separate `ISSUE-TS-001` (D4)
- **vector_index.bin tracking history verification**: requires `git log --follow`; done in TASK-05 as part of gitignore cleanup

---

## Files Likely Touched (consolidated)

```
.github/workflows/{ci-rust-10,ci-web-11,gate-docs-21,ci-examples-12,chaos-45,perf-bench-40,release,desktop,opencode,release-binaries-63,heavy-certification-50,heavy-bench-nightly-51}.yml
Cargo.toml
cliff.toml
.gitignore
Justfile
.pre-commit-config.yaml
dev-tools/{verify.ps1, gate-common.ps1, audit-all.ps1}
scripts/{verify_datasets.sh, verify_datasets.ps1, audit-tokens.ps1}
benches/{wal_throughput.rs, crash_recovery.rs, bench_concurrent.rs}
tests/{logic/parser.rs, .../query_result*.rs}
data/README.md (new)
datasets/README.md (new)
CI_POLICY.md
```

---

## Verification Commands

```bash
# Phase 1 verification (run after TASK-01..05 complete)
just verify                                       # fast gate
bash scripts/verify_datasets.sh                   # must pass if datasets present
cargo bench --workspace 2>&1 | tail -20            # populate baseline (TASK-02)

# Phase 2 verification (incremental per task)
cargo test --workspace --features test-bench-datasets
cargo mutants --check -p vantadb --timeout 120
cargo bench --bench wal_throughput --bench crash_recovery --bench bench_concurrent

# Phase 3 verification
pre-commit run --all-files
git cliff --unreleased --dry-run

# Full validation
just ci                                            # full CI gate
```

---

## Definition of Done (per `.opencode/references/definition-of-done.md`)

Each TASK closes when:
- [ ] Files changed match the task description
- [ ] Acceptance criteria met (see per-task verify)
- [ ] No new `#[ignore]` without Issue `flaky` documented
- [ ] `cargo check --workspace --tests --benches` passes
- [ ] Commit uses Conventional Commits (`fix:`, `feat:`, `chore:`, `ci:`)
- [ ] PR links this plan file

---

## Handoff

- **Plan file:** `docs/plans/2026-08-30-testing-bench-harden.md` (this file)
- **Backlog phase:** New **P39** with 23 tasks (`TBH-01..23`)
- **Plan file archived** after completion to `docs/plans/archive/2026-08-30-testing-bench-harden.md`
- **Progress doc:** `docs/avance/activo/testing-bench-harden.md` (created on first task)
- **Related research:** `docs/research/audit-testing-benchmarks-2026-08-30.md` (to be written after TASK-12 completes the data docs)

=== RECITATION TBH-03 ===
Campaign ID: 6ace02cd-7e74-4b23-9d55-bb32222eee13
Objetivo activo: TBH-03: 1-line removal of `if: schedule` from ci-gate.yml
Estado: completed
Última acción: Commit 72b98dc6 on develop. Pre-commit actionlint passed. 1 deletion in ci-gate.yml. Task file created.
Resultado: ✅ COMPLETO
Próxima acción: Hand off. Orchestrator should pick up TBH-04 (add `develop` to 6 workflows + release.yml) next.
Contrato: All 6 acceptance criteria met: line 24 removed, YAML parses, actionlint ok, 3 invocadores untouched, no ADR, conventional commit ci(TBH-03):
Próxima tarea si completa: TBH-04
=== END RECITATION ===

=== RECITATION TBH-04 ===
Campaign ID: 99ec94fc-370d-456a-8776-180b67de9bc1
Objetivo activo: TBH-04: add develop to 6 workflows + release.yml to align with dependabot target-branch (D6)
Estado: completed
Última acción: git commit abcce28f - 8 files changed (7 workflows + task file), pre-commit actionlint passed
Resultado: ✅
Próxima acción: TBH-10 (convert bench_concurrent) or another independent task — next wave task, handoff to orquestador
Contrato: grep -L develop on 7 files = empty; python yaml.safe_load utf-8 on all 7 = OK; actionlint via pre-commit hook = ok; pull_request.branches untouched; out-of-scope workflows (release-npm-*, release-wheels-*, sec-codeql-30, ci-rustdoc, desktop) unchanged
Próxima tarea si completa: TBH-10 (or next independent parallel task per orquestador assignment)
=== END RECITATION ===

=== RECITATION TBH-13 ===
Campaign ID: 6ace02cd-7e74-4b23-9d55-bb32222eee13
Objetivo activo: TBH-13: SHA-pin remaining 5 third-party action refs in desktop.yml + opencode.yml (D5)
Estado: completed
Última acción: git commit 42141706 on develop. 6 line changes (5 audit-named + 1 bonus actions/upload-artifact for empty-grep contract), pre-commit actionlint ok, conventional commit ci(TBH-13):.
Resultado: ✅ COMPLETO
Próxima acción: handoff to orquestador. Próxima tarea candidata: TBH-10 / TBH-12 / TBH-14 (any wave-2 phase-2 task).
Contrato: grep -E "@(v[0-9]+|@latest)" desktop.yml opencode.yml → empty (Select-String exit 1 = no match); python yaml.safe_load → OK; actionlint → 0 errors; 5 SHA-pins per acceptance criteria + 1 bonus upload-artifact; SHAs verified via gh api git/commits/<sha> (tauri-action@v1 dereferenced from annotated tag 944946e3... → commit 1deb371b...); no other workflow touched.
Próxima tarea si completa: TBH-10 / TBH-12 / TBH-14 (next wave parallel task per orquestador)
=== END RECITATION ===

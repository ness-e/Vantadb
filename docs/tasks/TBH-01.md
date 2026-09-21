# TBH-01 — `verify_datasets.{sh,ps1}` + CI gate

**Plan:** `docs/plans/2026-08-30-testing-bench-harden.md` (Phase 1 — ALTA)
**Owner:** vanta-worker (this task) → handoff to vanta-lead for TBH-02
**Created:** 2026-08-30

## Spec

The 5 canonical datasets the heavy-cert workflow ultimately depends on
(inferred from `tests/certification/*` and `tests/benchmark_datasets.rs`):

| Dataset | Test files that depend on it | Path(s) verified |
|---|---|---|
| **SIFT-1M** | `tests/certification/sift_validation.rs`, `tests/certification/competitive_bench.rs` | `datasets/sift/sift_base.fvecs`, `datasets/sift/sift_query.fvecs`, `datasets/sift/sift_groundtruth.ivecs` |
| **GloVe-100** | `tests/certification/glove_dimension_scale.rs`, `tests/benchmark_datasets.rs` | `data/benchmark/glove.6B.100d.txt` |
| **GloVe-300** | `tests/certification/glove_dimension_scale.rs` | `data/benchmark/glove.6B.300d.txt` |
| **SIFT-128 euclidean subset** | `benches/param_sweep.rs` (criterion bench) | `data/benchmark/sift-128/train.f32`, `data/benchmark/sift-128/test.f32`, `data/benchmark/sift-128/test_neighbors.u64`, `data/benchmark/sift-128/meta.json` |
| **GloVe-100 angular subset** | `benches/param_sweep.rs` (criterion bench) | `data/benchmark/glove-100-angular/{train.f32, test.f32, test_neighbors.u64, meta.json}` |

Exit codes:
- `0` when every expected file exists.
- `1` when ≥1 file is missing (table printed + non-zero exit).

Output: human-readable markdown-style table (dataset | source | status | action).
`--json` flag for machine consumption (CI annotation). Default: human.

Skip-pattern: tests with `#[ignore]` that are NOT dataset-related are kept
whitelisted. Verification uses `test -e` / `Test-Path` (no extra deps).

## Impacto mapeado (Regla 0)

### Archivos leídos completos

- `scripts/download_benchmark_datasets.sh` — only downloads GloVe-100 (line 14-17); no GloVe-300
- `scripts/download_benchmark_datasets.ps1` — mirror of `.sh`
- `scripts/download_ground_truth.py` — produces SIFT-128/GloVe-100 HDF5 subsets
- `dev-tools/scripts/download_sift.py` — produces SIFT-1M .fvecs
- `tests/certification/sift_validation.rs` — explicit skip pattern (lines 14-18)
- `tests/certification/glove_dimension_scale.rs` — explicit skip pattern (lines 173-181)
- `tests/certification/hnsw_validation.rs` — **NO dataset skip** (synthetic seeded)
- `tests/certification/hybrid_retrieval_quality.rs` — **NO dataset skip** (tempdir)
- `tests/certification/hybrid_ranking_metrics.rs` — **NO dataset skip** (tempdir)
- `tests/certification/competitive_bench.rs` — explicit skip pattern (lines 170-174)
- `tests/benchmark_datasets.rs` — explicit skip pattern (lines 15-19), NOT in `certification/`
- `.github/workflows/heavy-certification-50.yml` — single step downloads GloVe-100 in `other-heavy` job (line 234-235); no pre-test gate
- `dev-tools/gate-common.ps1` — confirms canonical features config (no drift risk)

### Refs salientes (¿qué archivos va a tocar este script?)

- `scripts/verify_datasets.sh` (new, ~50 lines) — bash, set -euo pipefail
- `scripts/verify_datasets.ps1` (new, ~60 lines) — pwsh, $ErrorActionPreference = "Stop"
- `.github/workflows/heavy-certification-50.yml` — add single pre-test step in `other-heavy` job
- `.opencode/skills/campaign-executor/tasks/TBH-01.md` (this file)

### Refs entrantes (¿quién invoca lo que vamos a tocar?)

- `heavy-certification-50.yml` is the only workflow that depends on certification
  datasets. It's invoked via `workflow_dispatch` (manual) + `schedule cron: 0 3 * * 0`
  (weekly Sun). Verified via grep: only the 3 workflows reference
  `download_benchmark_datasets.sh` (`ci-rust-10.yml`, `heavy-certification-50.yml`,
  `release-binaries-63.yml`). The latter two do not need the gate (release-binaries
  builds binaries; ci-rust-10 has the same `glove-100d` cache that the `other-heavy`
  job has today — the gate would also belong there per plan, but the contract says
  insert in `heavy-certification-50.yml` first).
- `scripts/verify_datasets.{sh,ps1}` will be the gate. Invoked by
  `heavy-certification-50.yml` `other-heavy` job via a new pre-test step
  (`run: bash scripts/verify_datasets.sh`). No other callers expected.

### Veredicto (impact classification)

- **Scope:** narrow (2 new files + 1 line in workflow + 1 task file)
- **Risk:** low (script is read-only — `test -e` / `Test-Path`)
- **Contract verifiability:** mechanical (`bash scripts/verify_datasets.sh` exit 0/1)
- **Public API change:** no
- **Dependency change:** no (zero `Cargo.toml` changes per contract §8)
- **Hot path:** no (CI-only, ~1s runtime)
- **Breaking:** no (only adds a gate; tests still skip gracefully inside)
- **Side effects:** `heavy-certification-50.yml` `other-heavy` job will fail fast
  on missing datasets instead of silently passing — this is the ENTIRE point
  of the task (audit's gap #1).

**Decision:** proceed without Gate D (no `pub fn` exposed, no new spec needed —
purely operational gate with mechanical contract from `AGENTS.md` Regla 0).

## Steps atómicos

- [x] **STEP-1: Read all key files** (DONE in discovery above)
- [x] **STEP-2: Populate Impacto mapeado** (DONE above)
- [ ] **STEP-3: Write `scripts/verify_datasets.sh`**
- [ ] **STEP-4: Write `scripts/verify_datasets.ps1`**
- [ ] **STEP-5: Manual test with data present** (currently `glove.6B.100d.txt` + `glove.6B.300d.txt` + `sift-128/` subset present; SIFT-1M full missing; glove-100-angular subset missing)
- [ ] **STEP-6: Manual test with data absent** (move a file aside, run script, expect exit 1)
- [ ] **STEP-7: Add pre-test step to `heavy-certification-50.yml`**
- [ ] **STEP-8: Verify full: `cargo fmt --check` + `cargo check --workspace --benches` + YAML syntax**
- [ ] **STEP-9: Commit + memoria + handoff**

## SDP

`SKILLS_CARGADAS: campaign-executor (base) + ponytail (base) + ci-cd-and-automation (lifecycle BUILD) + incremental-implementation (lifecycle BUILD) + doubt-driven-development (lifecycle BUILD) + test-driven-development (lifecycle BUILD)`

Notes:
- `doubt-driven-development` no aplica en este caso (no es security-sensitive;
  el script es read-only y trivially auditable).
- `test-driven-development` no aplica en el sentido canónico: el script no es
  lógica nueva, es un gate de CI; el "test" es el manual STEP-5/6 + la
  invocación en el workflow (STEP-7).
- `incremental-implementation` aplica: STEP-3 (sh) → STEP-4 (ps1) →
  STEP-5/6 (test) → STEP-7 (wire) → STEP-8 (verify) → STEP-9 (commit).

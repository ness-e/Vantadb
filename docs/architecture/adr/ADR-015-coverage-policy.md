---
title: "ADR-015: Coverage policy gate — root crate vs workspace aggregate and binding expectations"
type: adr
status: accepted
tags: [vantadb, architecture, adr, ci, coverage, quality-gates]
created: 2026-08-09
last_reviewed: 2026-08-09
---

# ADR-015: Coverage policy gate — root crate vs workspace and bindings

## Context

CI enforces a code-coverage gate in `.github/workflows/ci-rust-10.yml`
(`coverage` job). The mechanics today (verified 2026-08-09, COV-004):

- **Measurement:** `cargo llvm-cov nextest --profile audit --workspace
  --features "cli,arrow,tls,opentelemetry"` with `--exclude vantadb-wasm
  --exclude vantadb-server --exclude vantadb-mcp`. The workspace is `[
  ".", "vantadb-python", "vantadb-server", "vantadb-mcp", "vantadb-wasm" ]`
  (`Cargo.toml [workspace].members`), so the effective coverage surface is
  **root `vantadb` + the `vantadb-python` pyo3 binding crate**.
- **Exclusions:** `--ignore-filename-regex '(tests/|benches/|packages/experimental|crash_injection)'`
  for the lcov artifact; the gate re-runs the report with
  `'(tests/|benches/|packages/experimental)'`.
- **Gate:** the enforce step is named "Enforce coverage threshold (>=70%)"
  but the embedded Python check reads `ok = pct >= 80.0`. The real enforced
  threshold is **workspace-wide line coverage ≥ 80%**; the step name is stale.
- **Bindings:** `vantadb-wasm`, `vantadb-server`, `vantadb-mcp` are excluded
  from the coverage run entirely (experimental crates; see
  `docs/operations/CI_POLICY.md`). Python bindings are independently gated by
  their own suite: wrapper coverage ≥ 85% on `vantadb_py/__init__.py`
  (contract in plan 2026-08-09 Task 36; measured 96% at last run).

Measured baselines (COV-004 campaign, 2026-08-09):

- **Root crate (`vantadb`): 81.40%** line coverage — above the 80% gate.
- **Workspace aggregate (root + `vantadb-python` Rust crate): 72.76%** —
  below the 80% gate. The gap is dilution by the binding crate's Rust glue,
  whose behavior is exercised primarily through the Python test suite.

The plan contract for COV-004 asks the ADR to express root (81.40%) vs
workspace (72.76%) and the binding expectation. Note that the plan's framing
("umbral root 81.40% vs workspace 72.76%") describes baselines, not
thresholds; the only hard threshold in CI is the workspace-wide 80% gate.

## Decision

1. **The gate stays workspace-wide ≥ 80% line coverage** as implemented in
   `ci-rust-10.yml`. It is the single enforced numeric threshold and must not
   be lowered to accommodate the aggregate baseline.
2. **Root `vantadb` is the primary quality signal.** Its baseline (81.40%)
   exceeds the gate; PRs must keep root coverage ≥ 80%. Root-only progress is
   reported separately because it reflects engine/SDK code quality without
   binding-crate dilution.
3. **The workspace aggregate (72.76%) is measured and reported, not treated
   as a per-crate target.** The gap is `vantadb-python`'s Rust surface, which
   is far more cheaply covered by Python-side tests than by Rust unit tests.
   Raising the Rust-side binding number is deliberately optional
   (`ponytail:`-style: enriched Python tests cover the same lines at lower
   cost).
4. **Binding expectations:**
   - **Python:** gated by pytest + coverage on `vantadb_py/__init__.py` ≥ 85%
     (its own contract, not by llvm-cov). The Rust side of the binding is
     included in the workspace aggregate but is not individually gated.
   - **WASM / MCP / server:** experimental (Tier 3). Excluded from llvm-cov;
     no coverage gate. WASM browser tests are best-effort
     (`continue-on-error: true`). Do not add coverage gates for these until
     they leave experimental status.
   - **Node bindings (`vantadb-node`)**: separate workspace, outside the
     coverage run; no coverage gate today.
5. **Tooling debt recorded:** the gate step name "(>=70%)" is stale — actual
   check is `pct >= 80.0`. The gate name should be renamed when the workflow
   is next edited; the check itself already implements this decision.

## Consequences

- Pros:
  - One enforced, unambiguous threshold (workspace ≥ 80%) with root ≥ 80% as
    the interpretable primary signal.
  - No new CI cost: the gate already exists; this ADR documents why the
    aggregate sits below 80% instead of being "fixed" by gaming the gate.
  - Binding coverage cost is allocated where it is cheapest (Python suite for
    python, nothing for experimental crates until they graduate).
  - Future agents understand the 81.40% / 72.76% asymmetry without
    re-deriving the workspace member list.
- Cons:
  - If `vantadb-python`'s Rust side continues to sit at low coverage, any run
    where its lines dominate can fail the aggregate gate even when root is
    healthy — mitigation: bindings must keep their Rust surface thin and lean
    on Python tests.
  - The stale "(>=70%)" step name can mislead readers until renamed.
  - Experimental crates (wasm/mcp/server) have no coverage signal at all
    until they graduate; acknowledged cost of the experimental policy.

## Related

- `.github/workflows/ci-rust-10.yml` (`coverage` job, lines 252–322)
- `docs/operations/CI_POLICY.md` (experimental crate policy)
- `Cargo.toml` (`[workspace].members`, `default-members`)
- Plan 2026-08-09 Task 36 (COV-001, python wrapper coverage ≥ 85%)
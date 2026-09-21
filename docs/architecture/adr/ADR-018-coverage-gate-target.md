---
title: "ADR-018: Coverage gate target — root crate (vantadb), not workspace aggregate"
type: adr
status: accepted
tags: [vantadb, architecture, adr, ci, coverage, quality-gates]
created: 2026-08-12
last_reviewed: 2026-08-12
owner: vanta-arch
---

# ADR-018: Coverage gate target — root crate (vantadb), not workspace aggregate

## Context

The CI coverage gate lives in `.github/workflows/ci-rust-10.yml` (`coverage`
job, lines 315–386). ADR-015 recorded the policy but left it internally
inconsistent: its prose says "the gate stays workspace-wide ≥ 80%", while the
in-tree CI comment (line 363) reads "root-gate 80% en CI (bindings/wasm/server/mcp
miden en runners nativos)". The two cannot both be true, and this ADR resolves
the ambiguity.

Measured baselines (COV campaign, 2026-08-12):

- **Root crate `vantadb`: 81.40%** line coverage — above 80%.
- **Workspace aggregate (root + `vantadb-python` Rust glue): 72.76%** — below 80%.

So gating on the workspace aggregate fails the build on every run, while gating
on the root crate passes. The `coverage` job currently enforces `pct >= 80.0`
from `cargo llvm-cov report --json` after a `--workspace` run with
`vantadb-wasm/-server/-mcp` excluded; the in-tree comment documents the *intent*
as a root gate. The decision is to make that intent explicit and authoritative.

This task is DOC-ONLY: the CI YAML is not modified here. Implementing the gate
(scoping the measurement to the root crate) is a separate task.

## Decision

1. **Gate target = the root crate `vantadb`** at **≥ 80% line coverage**
   (`cargo llvm-cov` scoped to `-p vantadb`, or the `vantadb` package's percent
   read from the JSON). Matches the proven-passing baseline (81.40%) and is the
   meaningful quality signal — `vantadb` is the core library.
2. **The workspace aggregate (72.76%) is NOT the gate target.** It is diluted by
   binding crates whose behavior is validated more cheaply by their own native
   suites (pytest for Python, c8 for TS). Gating on it forces low-value Rust
   unit tests for binding glue and breaks CI on every run. The aggregate is
   still measured and reported for visibility.
3. **This supersedes ADR-015 §Decision #1** ("workspace-wide ≥ 80%"): the
   authoritative gate target is the root crate.
4. **Binding policy (how bindings enter the gate):**
   - **Python (`vantadb-python`):** gated by pytest coverage on
     `vantadb_py/__init__.py` ≥ 85% in its own runner — not by llvm-cov. The
     Rust side of the binding is in the aggregate for visibility only, never
     individually gated (it is exercised by the Python suite).
   - **TypeScript (`vantadb-ts`):** measured via `c8` in `test-runner.mjs`
     (COV-002 resolved the `vitest`↔`vite-plugin-wasm` blocker). Add as an
     independent CI gate on the TS SDK (recommend start ≥ 70%, ratchet up). Not
     measured by cargo-llvm-cov (separate JS runtime).
   - **CLI binary tests (`cli_tests`, COV-003):** INCLUDE in the root coverage
     measurement. Remove any exclusion of the CLI binary / `cli_tests` from the
     coverage denominator so the ~2,500 ln of `src/cli_handlers/*` +
     `src/bin/vanta-cli.rs` count toward the root-crate gate. These are Rust in
     the root crate and must raise, not be carved out of, the root number.
   - **server / mcp / wasm:** experimental (Tier 3). Excluded from llvm-cov; no
     coverage gate until they graduate.
5. **Tooling debt (record, fix when the workflow is next edited):** the gate
   step is named "(>=70%)" but checks `pct >= 80.0`; rename it to
   "(>=80%, root crate)" and lock the enforcement to the `vantadb` package only.

## Consequences

- Pros:
  - One unambiguous, passing gate (root `vantadb` ≥ 80%) instead of a workspace
    number that fails by design.
  - Binding coverage cost allocated where cheapest (Python suite, TS c8) — no
    gaming the Rust gate to cover JS/Py glue.
  - CLI investment (COV-003) visibly lifts the root number rather than being
    excluded from it.
  - Resolves the ADR-015 vs CI-comment contradiction for future agents.
- Cons:
  - The workspace aggregate (72.76%) has no enforced floor; a binding crate's
    Rust surface could regress without failing CI — mitigated by per-binding
    native gates (Python ≥85%, TS c8).
  - Experimental crates (server/mcp/wasm) still have no coverage signal until
    they graduate (accepted cost of the experimental policy).

## Recommended CI action (implementation — out of scope for COV-004)

1. Scope the coverage measure to the root crate:
   `cargo llvm-cov nextest --profile audit -p vantadb --features "cli,arrow,tls,opentelemetry" --lcov --output-path lcov.info --ignore-filename-regex '(tests/|benches/|packages/experimental|crash_injection)'`.
2. Enforce on the `vantadb` package percent:
   `cargo llvm-cov report --json -p vantadb ...` → `data[0].totals.lines.percent >= 80.0`.
3. Ensure `cli_tests` is NOT excluded from the root measurement.
4. Rename the gate step to "(>=80%, root crate)".

## Related

- `.github/workflows/ci-rust-10.yml` (`coverage` job, lines 315–386)
- `ADR-015-coverage-policy.md` (superseded §Decision #1; binding expectations preserved)
- `docs/operations/CI_POLICY.md` (experimental crate policy)
- Plan `docs/plans/2026-08-12-cov-coverage.md` (COV-002 TS c8, COV-003 cli_tests)

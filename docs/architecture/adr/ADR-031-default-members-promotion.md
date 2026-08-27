---
title: "ADR-031: Promotion to default-members — 10-check DoD, cost table and reversible gate"
type: adr
status: proposed
tags: [vantadb, architecture, adr, ci, default-members, promotion, fast-gate]
created: 2026-08-27
last_reviewed: 2026-08-27
owner: TBD (pending Owner answer on Fast Gate threshold)
---

# ADR-031: Promotion to `default-members` — 10-check DoD, cost table and reversible gate

## Context

`Cargo.toml:636-642` declares a **workspace circuit breaker**:

```toml
[workspace]
members = [".", "vantadb-python", "vantadb-server", "vantadb-mcp", "vantadb-wasm", "vanta-memory", "vanta-proxy"]
default-members = [".", "vantadb-python"]
# CATEGORY: EXPERIMENTAL — vantadb-server, vantadb-mcp, vantadb-wasm were removed from
# default-members. A failure in an experimental crate must not block core CI.
# See docs/operations/CI_POLICY.md for the experimental crate policy.
```

`server / mcp / wasm / memory / proxy` are full workspace members but **outside** `default-members`
on purpose (`CATEGORY: EXPERIMENTAL`). `cargo check / clippy / nextest / deny / coverage` without
`--workspace` skip them. `ts`/`node` are npm packages and can never enter `default-members`, but
need an equivalent npm Fast Gate.

Phase P47 (`docs/Backlog.md:721-736`) was opened 2026-08-27 to answer the missing question:
**what must pass before we can call a crate "100% stable and promotable to default"** and to keep the
promotion itself a **1-line reversible** change. Until P47 there was no written DoD — the `experimental-check`
job (`ci-rust-10.yml:experimental-check`) was non-blocking and the promotion path was implicit.

This ADR is **STABLE-00** (P47 foundation): it writes the 10-check DoD, the per-crate cost table, and
the Owner gate on the Fast Gate `<5 min` threshold **before** any `Cargo.toml` edit. The actual
promotion is deferred to **STABLE-09** after STABLE-01..08 validate every crate in 3 clean runs.

## Decision

### 1 — Promotion DoD: 10 checks, 3 consecutive clean runs

Every Rust crate or npm package to be promoted must pass **all** gates below in **3 consecutive**
clean runs (`cargo clean` / `npm ci`) with **zero flaky**, on a clean runner, without `#[ignore]` /
`#[ignored]` or `continue-on-error`. A single failure resets the count.

| # | Gate | Exact command / criterion | Pass |
|---|------|-----------------------------|------|
| 1 | **check + fmt + clippy** | `cargo check -p <crate> --all-targets --all-features` **and** `cargo fmt --check` **and** `cargo clippy -p <crate> --all-targets --all-features -- -D warnings` | `0 warnings`, `0 errors` |
| 2 | **tests** | `cargo nextest run -p <crate> --profile audit -j 2` (or `npm test` → `vitest run` for `ts`/`node`) | `0 failed`, `0 ignored flaky`, no `#[ignore]` / `#[ignored]` in crate, no skipped flaky in report |
| 3 | **deny** | `cargo deny check` (licenses MIT/Apache-2.0 only, advisories, bans per `deny.toml`) | `0` advisories / bans / license violations |
| 4 | **docs coverage** | `pwsh scripts/validate-docs-coverage.ps1` (SDK/config/error/CLI/python/MCP parity vs `docs/api/*`) | `0 gaps` for the crate's public surface |
| 5 | **workflow gate** | CI workflow for the crate: has `paths:` filter, has **no** `continue-on-error: true` (except `CATEGORY:`-tagged non-blocking jobs per `CI_POLICY.md`), is **blocking** on PRs, has a concrete `timeout-minutes` and a **measured** wall time `<5 min` (Fast Gate) **or** is documented as Heavy with justification (see #9) | No `continue-on-error` on the crate's blocking job; measured timeout <5 min or Heavy label justified in `CI_POLICY.md` |
| 6 | **package metadata** | `cargo package -p <crate> --dry-run` (`publish = false` is allowed — gate checks metadata, not publishability) | exit `0`, no missing `description`/`license`/`readme` |
| 7 | **wasm toolchain** | For `vantadb-wasm` only: `rustup target add wasm32-unknown-unknown` present in CI image **and** `wasm-pack build --target bundler` **and** `cargo test --target wasm32-unknown-unknown` (or `wasm-pack test --node` / `--chrome --headless` where applicable) | `wasm-pack build` succeeds, `wasm32` target installed, no missing `wasm-bindgen` |
| 8 | **node matrix** | For `vantadb-node` only: `napi build --platform` 7-target matrix via `.github/workflows/release-npm-node.yml` (x86_64-pc-windows-msvc + x86_64-unknown-linux-gnu + x86_64-unknown-linux-musl + aarch64-unknown-linux-gnu + aarch64-unknown-linux-musl + x86_64-apple-darwin + aarch64-apple-darwin) with artifact upload, `npm pack` includes `*.node` | 7 artifacts present, `npm pack` tarball contains `.node` binary |
| 9 | **Fast Gate wall time** | `just verify` / `dev-tools/verify.ps1` (and `dev-tools/verify_changed.ps1`) with `default-members` **expanded** to `[ ".", "vantadb-python", "vanta-memory", "vanta-proxy", "vantadb-server", "vantadb-mcp", "vantadb-wasm" ]` on a cold cache, branch `test/default-all`, measured on `ubuntu-latest` | Wall time `<5 min` → stays in Fast Gate; else re-label as **Heavy** and add justification in `docs/operations/CI_POLICY.md` (requires Owner approval — see §Question to Owner) |
| 10 | **ADR + reversibility** | This ADR (or a per-crate addendum) records the promotion, the measured CI time and the toolchain delta, and states the 1-line rollback (`default-members` revert) | ADR merged, promotion is `git revert` of 1 line in `Cargo.toml:636`, `publish = false` unchanged, `cargo publish` unaffected |

> **Definition of Done for default:** a crate/package is promotable **only if** gates 1-10 pass together
> in 3 consecutive runs. STABLE-01..07 validate gates 1-8 per crate; STABLE-08 validates gate 9;
> STABLE-09 performs the atomic promotion (gate 10).

### 2 — Per-crate cost table (baseline 2026-08-27, dev machine, warm `cargo` cache)

Measured with `cargo check --all-targets` per crate (no `--workspace`; sequential). Wall times vary
by machine and cold vs warm cache — the table below is the **order of magnitude** to size the Fast Gate
budget. STABLE-01..08 must re-measure on `ubuntu-latest` with `cargo clean` and record the CI numbers
here before promotion.

| Crate / package | `cargo check` | `clippy -- -D warnings` | `nextest -p <crate> --profile audit -j 2` / `vitest run` | Toolchain extra | `Cargo.lock` delta | Note |
|-----------------|---------------|--------------------------|-----------------------------------------------------------|-----------------|--------------------|------|
| `vantadb` (root, current default) | ~18-22s local (gate `cli,fjall,memmap2,fs2,roaring`) | ~25s (core only) | audit profile ~90-120s (suite core, excludes `RESOURCE-GUARD` 3 tests) | none | 0 | Baseline Fast Gate; does not include experimental crates |
| `vantadb-python` | ~12-15s (PyO3 glue) | ~18s | `cargo test -p vantadb-python` does not run as Rust suite; Python wrapper measured via `pytest` (~15s, 96% coverage) | `maturin` for wheels, `python` 3.11+ | 0 | Already in `default-members`; Python side gated by `pytest` ≥85% (ADR-015) |
| `vanta-memory` (`publish=false`) | ~36s (`cargo check -p vanta-memory --all-targets`) | ~40s | `nextest -p vanta-memory` ~20-40s (suite small, host-neutral) | none (depends on `vantadb` without `server`) | 0 (already in `[workspace].members`) | No `server` feature; expected Fast |
| `vanta-proxy` (`publish=false`, `axum+tokio+reqwest`) | ~32s (`cargo check -p vanta-proxy --all-targets`) | ~45s | `nextest -p vanta-proxy` + e2e with upstream mock (~30s) | none | 0 | Heaviest Rust compile in P47; if `check` >60s on CI → candidate **Heavy** |
| `vantadb-server` | ~21s (`cargo check -p vantadb-server --all-targets`) | ~25s (with `server` feature) | `cargo test -p vantadb-server` 42 tests ~15-25s | none | 0 | Already polished (SRV-01/02/06); expected Fast |
| `vantadb-mcp` | ~6.7s (`cargo check -p vantadb-mcp --all-targets`) | ~10s | `cargo nextest run -p vantadb-mcp --profile audit` 62 tests ~18s; `test-mcp.py` 37 checks ~5s | none | 0 | `handle_tools_list` is the `validate-docs-coverage` source |
| `vantadb-wasm` | ~3.6s (`cargo check -p vantadb-wasm --all-targets`) | ~8s (with `wasm` feature) | `cargo check -p vantadb-wasm` passes; `wasm-pack test --node` ~12s (requires wasm-pack) | `rustup target add wasm32-unknown-unknown` + `wasm-pack` + `binaryen` (`wasm-opt`) | 0 | Tier 3 experimental today (`continue-on-error: true` on `wasm-test` job) |
| `vantadb-ts` (npm, never in `default-members`) | `npm ci` ~18s + `tsc --noEmit` ~6s | `npx eslint .` ~3s | `npm run build && npx vitest run` 264 tests ~26s (measured in `release-npm-61.yml:tests`, timeout 10) | `node >=22.12`, `npm` | 0 (no Cargo.lock) | Validated by `release-npm-61.yml:tests` (Fast Gate <5 min already) |
| `vantadb-node` (npm, napi-rs, never in `default-members`) | `npm ci` ~18s | `npx tsc --noEmit` (if TS) | `npm test` 25 tests ~8s (single platform) | `@napi-rs/cli`, 7-target cross build | 0 (no Cargo.lock) | Full matrix via `release-npm-node.yml` is **Heavy** by construction (7 platforms, ~15-20 min cross); single-target `cargo check` style validation is Fast |

**Aggregates:**

- `cargo check --workspace --all-targets --all-features` (all 7 Rust crates) **is not** the gate — the gate
  is `cargo check -p <crate>` per crate and `verify.ps1` with expanded `default-members`. A cold
  `cargo check -p` for proxy+memory+server+mcp+wasm together adds ~80-100s compile time locally,
  dominated by `vanta-proxy` and `vanta-memory`.
- `Cargo.lock` delta is **0 KB** for all Rust promotions: every candidate is already in
  `[workspace].members`; `default-members` only changes the default `cargo` set, not the lockfile.
  The lockfile grows only if a new third-party dep is added — tracked in STABLE-01..07.
- `publish = false` is kept on `vanta-memory` / `vanta-proxy` — `cargo publish` is unaffected; the
  DoD uses `cargo package --dry-run` only to assert metadata completeness.

### 3 — Reversibility

- **Forward:** one line in `Cargo.toml:636`:
  `default-members = [".", "vantadb-python", "vanta-memory", "vanta-proxy", "vantadb-server", "vantadb-mcp", "vantadb-wasm"]`
  (order alphabetical after root). For `ts`/`node`, promotion means "gate is blocking in Fast Gate"
  (workflow already has `timeout-minutes` and `vitest` in `release-npm-61.yml`); no `Cargo.toml` edit.
- **Rollback:** `git revert <promotion-commit>` or manually revert that one line to
  `default-members = [".", "vantadb-python"]` and revert the `CI_POLICY.md` promotion note.
  Zero data migration, zero `cargo publish`, zero tag — `publish = false` crates never publish.
- **Sidecar docs:** `docs/operations/CI_POLICY.md` §default-members and
  `.opencode/rules/release-ci.md` (STABLE-09) are reverted alongside.

### 4 — Question to Owner (Gate — must be answered before STABLE-09)

> **Owner decision required (STABLE-00 Gate).** This ADR stays `proposed` until the Owner answers.
> STABLE-01..08 may validate individual crates, but STABLE-09 (atomic promotion) is **blocked**
> pending this answer.

**Context for the question:** gate #9 measures `just verify` / `dev-tools/verify.ps1` with all 7 Rust
crates in `default-members` on `ubuntu-latest` with a cold cache. If the wall time stays `<5 min`,
the Fast Gate constraint holds and the promotion can be pure Fast Gate. If it exceeds `<5 min`, the
`experimental-check` separation must be abandoned for those crates and they become part of the
blocking Fast Gate — either the gate is re-labeled **Heavy** (requires justification in
`CI_POLICY.md` and a higher `timeout-minutes` with explicit `CATEGORY:` rationale), or the
promotion is **scoped** (promote only the subset that keeps `<5 min`, defer the heavy crate).

**Question (choose one, record below):**

- **(A) `<5 min` is a hard Fast Gate invariant** — if STABLE-08 measures `≥5 min` with all 7 crates,
  **do not** promote the slow crate to `default-members`; instead document it as Heavy in
  `CI_POLICY.md`, keep it in `experimental-check` (non-default), and re-measure after optimization
  (e.g., feature trimming in `gate-common.ps1:Get-CoreFeatures`, sccache warm cache, or splitting
  `verify.ps1` steps). Heavy promotion requires a separate ADR amendment with `CATEGORY:`
  justification per `CI_POLICY.md` Regla 2.

- **(B) `<5 min` is a soft target** — if STABLE-08 measures `5-8 min`, promote anyway and **re-label**
  the Fast Gate as `~8 min` in `CI_POLICY.md` and `.opencode/AGENTS.md` CI table, update
  `dev-tools/verify.ps1` comments and `ci-rust-10.yml:coverage` `timeout-minutes`, and accept the
  velocity cost. Beyond `>8 min` still requires Heavy split (e.g., keep `vanta-proxy` out).

**Recorded answer:**

```
Owner: ____________________________
Date:  ____________________________
Choice: [ ] A  [ ] B  (with 8-min ceiling)  [ ] other: ______________
Rationale / condition: _______________________________________________
```

Until recorded, this ADR is `status: proposed` and STABLE-09 must not merge. The answer is
appended to this section with the Owner's name and date; the ADR status then moves to `accepted`
(if A or B is chosen) and STABLE-09 may proceed.

### 5 — Promotion procedure (deferred to STABLE-09)

1. STABLE-01..07: each crate passes gates 1-8 in 3 clean runs; wall times recorded in §2 table.
2. STABLE-08: measure gate #9 on branch `test/default-all` with expanded `default-members`;
   decide per Owner answer (A vs B) whether to promote all 7 or a subset.
3. STABLE-09: single atomic PR — `Cargo.toml:636` edit + `CI_POLICY.md` §default-members note +
   `.opencode/rules/release-ci.md` + `dev-tools/verify.ps1` comment about `wasm32` target;
   PR description contains the 1-line rollback command. No `git tag`, no `cargo publish`.

## Consequences

- **Pros**
  - A written, auditable DoD for `default-members` where none existed — future agents no longer
    infer the promotion rule from scattered `CATEGORY: EXPERIMENTAL` comments.
  - Per-crate cost is explicit before promotion (CI time + toolchain), so STABLE-08's `<5 min`
    verdict has a baseline to compare against.
  - Reversible by `git revert` — `publish = false` is preserved, so the promotion has zero
    release coupling.
  - `ts`/`node` are included with an equivalent npm gate (`vitest` + `npm pack` with `.node`),
    even though they can never be in `default-members`.

- **Cons**
  - The 10-check bar is high (3 clean runs, `deny` + `docs-coverage` + `wasm-pack` + matrix) — valid
    crates may sit in `experimental-check` longer while STABLE-01..07 work through their gates.
  - The Fast Gate may need to grow from `<5 min` to `~8 min` if `vanta-proxy` + `vanta-memory`
    together push the cold compile over the line — velocity cost that the Owner must accept (see
    §Question to Owner).
  - `cargo deny` and `validate-docs-coverage` are already blocking elsewhere, but re-asserting them
    per crate adds CI minutes — mitigated by per-crate `cargo check -p` (not `--workspace`).

## Alternatives Considered

- **Promote without written DoD (status quo):** rejected — the circuit breaker exists precisely
  because `server/mcp/wasm` once broke core CI implicitly; promoting without criteria would repeat
  the failure that motivated `CATEGORY: EXPERIMENTAL`.
- **Promote incrementally one crate per PR without ADR:** rejected — incremental PRs are fine
  *after* the DoD exists (STABLE-01..07 are per crate), but the DoD itself must be agreed once.
- **Keep everything experimental forever:** rejected — `vantadb-server` (42 tests, SRV-01/02/06),
  `vantadb-mcp` (62 nextest + 37 `test-mcp.py` checks), and `vanta-memory` are product-critical;
  leaving them non-default hides regressions that P47 is meant to surface.
- **Put `ts`/`node` into `default-members`:** impossible — `default-members` only lists Cargo members
  (`[workspace].members`); npm packages validate via `release-npm-61.yml` / `release-npm-node.yml`
  instead (gate #8 and the `ts` row in §2).

## References

- `Cargo.toml:620-642` (`[workspace].members` + `default-members` + `CATEGORY: EXPERIMENTAL`)
- `docs/operations/CI_POLICY.md` §Experimental Crate Circuit Breaker (lines 109-143; promotion rule
  at 134-135) + §1 Fast Gate (`<5 min`, deterministic, offline) + `.config/nextest.toml` audit profile
- `dev-tools/verify.ps1` (local Fast Gate: fmt → check → clippy → audit → deny → nextest → coverage → docs-coverage)
- `dev-tools/gate-common.ps1:Get-CoreFeatures` (`cli,fjall,memmap2,fs2,roaring`)
- `scripts/validate-docs-coverage.ps1` (6 checks: SDK/config/error/CLI/python/MCP)
- `.github/workflows/ci-rust-10.yml` (`fmt`, `clippy --workspace`, `experimental-check`, `coverage`, `wasm-test` with `continue-on-error: true`)
- `.github/workflows/release-npm-61.yml:42-82` (`tests` job, `vitest run` 264 tests, timeout 10, ~26s measured) and `release-npm-node.yml` (7-target matrix, `npm pack` with `*.node`)
- `docs/Backlog.md:721-736` (P47 10-check contract) + `docs/Backlog.md:738-749` (STABLE-00..09)
- `.opencode/references/definition-of-done.md` (VantaDB-specific DoR/DoD)
- `docs/_templates/adr.md` (template) and ADR-027/ADR-030 (reference style)

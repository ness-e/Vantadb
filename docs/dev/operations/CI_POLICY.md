---
title: "VantaDB CI & Certification Policy"
kind: runbook
status: active
description: To maintain a rapid development iteration cycle and guarantee mathematical precision in our HNSW
tags: [vantadb, operations]
---

# VantaDB CI & Certification Policy

To maintain a rapid development iteration cycle and guarantee mathematical precision in our HNSW
engine, VantaDB enforces a split Continuous Integration architecture.

## CI Workflow Inventory

VantaDB has **28 active workflow files** in `.github/workflows/` (verificado 2026-09-27; 28 in FIND-128 minus one rustdoc workflow merged into `ci-rustdoc.yml` in FIND-137, renames in FIND-142, +1 `nightly.yml` HARD-02). Each workflow is documented below. See `docs/dev/workflow/README.md` (inventory) and `docs/dev/workflow/TRIGGERS.md` (trigger matrix — source of truth is each file's `on:` block).

### Local Verification Scripts — Rutas Canónicas

The CI/Hooks integration table in `.opencode/AGENTS.md` lists the local verification scripts.
Their canonical paths are:

| Script | Assertion scope |
|--------|-----------------|
| `dev-tools/verify_changed.ps1` | Quick verify (~30s): fmt → check → clippy on `vantadb` core. Runs the docs-coverage gate only when `git diff --name-only HEAD` touches `src/`, bindings (`vantadb-python`, `vantadb-ts`, `vantadb-wasm`) or `docs/api/`; otherwise silently skips. |
| `dev-tools/verify.ps1` | Full pre-flight (measured: **209.5s warm** — 10 steps, coverage moved to the nightly; see §"Fast Gate wall-time measurement"): fmt → check → clippy → audit → deny → tests → docs-coverage → cli-probes → consumo guard → daily backup check. Coverage report+budget on-demand with `-IncludeCoverage` (nightly owns the default enforcement). Runs the docs-coverage gate whenever the script exists. |
| `scripts/validate-docs-coverage.ps1` | Docs coverage gate (Regla 3, mecánica): valida símbolos públicos SDK/config/error/CLI/Python/MCP contra `docs/api/*`. `-ReportOnly` imprime gaps sin fallar; sin el flag, los métodos sin documentar devuelven exit 1 y fallan el script host. |

`scripts/validate-docs-coverage.ps1` is the **single shared docs gate**, referenced from both
`dev-tools/verify_changed.ps1` and `dev-tools/verify.ps1`. The two verify scripts form a
hierarchy (quick → full), not alternative locations for the same gate — this is the canonical
map so the AGENTS.md CI/Hooks table and this policy reconcile at the next docs sync:
Regla 3 ("docs al día") is enforced mechanically by the docs-coverage gate, not by convention.

### 1. Fast Gate (`ci-rust.yml`)

The fast gate is triggered automatically on every pull request and push to the `main` branch.
**Goal:** Deliver PR feedback in under 5 minutes.

The fast gate validates the production-facing MVP boundary only: embedded core behavior, stable
SDK/CLI flows, durability, namespace and metadata indexes, vector retrieval, BM25, Hybrid Retrieval
v1, rebuild/audit, and local deterministic integration tests. Historical or experimental surfaces
such as IQL/LISP/DQL, MCP, LLM/Ollama integration, graph traversal beyond stored local edges, and
governance semantics are excluded from the default fast lane.

**Jobs:**

| Job | Description |
|-----|-------------|
| `fmt` | Format Check — `cargo fmt --check` |
| `clippy` | Clippy Lints — `cargo clippy -- -D warnings` |
| `test` | Tests (Linux) — nextest audit profile |
| `test-windows` | Tests (Windows) — nextest ci-windows profile |
| `test-macos` | Tests (macOS) — nextest audit profile |
| `msrv` | MSRV Check (1.94.1) |
| `minimal-versions` | Minimal Versions Check (`-Zminimal-versions`, nightly, continue-on-error) |
| `coverage` | Code Coverage (`cargo-llvm-cov`, gate root crate ≥80% por ADR-0018) |
| `audit` | Security Audit (`cargo audit`) |
| `deny` | License & Policy (`cargo deny check`) |
| `semver-checks` | Public API Semver (RELEASE-01) — `cargo semver-checks -p vantadb` vs última publicada en crates.io. Ver `ci-cd-guide.md` § "Semver-checks gate (public API)" |
| `miri` | Miri UB Detection (nightly) |
| `deny` | Dependency Policy Check (`cargo deny`) |
| `experimental-check` | Experimental Crates Check (continue-on-error, non-blocking) |
| `sanitizer-asan` | AddressSanitizer (nightly, continue-on-error) |
| `sanitizer-tsan` | ThreadSanitizer (nightly, continue-on-error) |
| `release-combo` | Release Combo Check — `cargo check --release` of `vanta-cli` (`server,jemalloc`) + `vantadb-server` (`jemalloc`) with `RUSTFLAGS=-D warnings`; the exact combo of `release-binaries.yml` (FIND-231) |

> **Note (ERR-OBS-01):** the `test` job exercises error-observability behavior
> in `error::tests` (backtrace capture is env-gated via `RUST_BACKTRACE`/
> `RUST_LIB_BACKTRACE` and asserted against whatever status CI provides — the
> test is deterministic under both). Structured log levels and error envelopes
> are covered by `server::errors` tests under the `server` feature. See
> `docs/user/operations/OBSERVABILITY.md`.

**Strict Rules for the Fast Gate:**

- **Deterministic:** Tests must not rely on random timing or external networking.
- **Local:** No external dependencies are allowed (e.g., no external LLM services, no Ollama
  required).
- **Fast:** Any test exceeding a few seconds must be moved to heavy certification or heavily
  optimized.

### Fast Gate Test Exclusions

The local fast gate (`dev-tools/verify.ps1`, `nextest` and `coverage` steps) applies an explicit
`-E` nextest filter that removes three tests. These exclusions are **not** flakiness waivers — all
three tests are deterministic and still run in full local suites and Heavy Certification; they are
removed only from the fast lane because they deliberately stress runner resources.

**Category taxonomy:** these use a dedicated runner-risk category, alongside the existing
EXPERIMENTAL / BEST-EFFORT / NON-CRITICAL / INFORMATIONAL categories used for
`continue-on-error:` annotations in `.github/workflows/`. A test in this category is one whose
input is intentionally large or hostile enough to threaten the runner itself (OOM, page-file
exhaustion), not one that is broken or slow by accident. The category tag is rendered in the
table column below.

| Excluded test | Lives in source | Why excluded | Category | Where the exclusion is enforced |
|---------------|-----------------|--------------|----------|--------------------------------|
| `deserialize_absurd_node_count` | `src/index/core.rs:414` | Deserializes a crafted buffer with `u64::MAX` node count — designed as a memory bomb for the deserializer path; allocating it on a shared runner risks OOM-killing unrelated jobs | RESOURCE-GUARD | `dev-tools/verify.ps1` `-E` filter (`nextest` + `coverage` steps) and the non-nextest fallback `--skip` list |
| `test_search_with_bizarre_text_query` | `tests/security.rs:639` | Feeds giant malformed text queries (100KB strings, NUL bytes, astral-plane chars) into search; robust behavior against such inputs belongs to the dedicated fuzzing lane (`fuzz.yml`), not the fast gate | RESOURCE-GUARD | Same |
| `test_malformed_payload_extremely_large` | `tests/security.rs:324` | Ingests a 1MB payload plus 10KB of metadata; same rationale — hostile-input coverage is delegated to fuzzing | RESOURCE-GUARD | Same |

**Structural exclusions in `.config/nextest.toml`:** the `default-filter` of the `audit` profile
additionally excludes ~55 heavy test binaries (stress_protocol, chaos_integrity, wal_resilience,
sift_validation, competitive_bench, etc.) via package-qualified `not (package(X) and binary(Y))`
clauses (BND-06 scope-safe form). That list implements the two-tier split documented in this file
(Fast Gate vs Heavy Certification) and changes only together with `heavy-certification.yml`.

**Rules for any new exclusion:**

1. Every fast-gate test exclusion must be listed in this table with its category and rationale.
2. Exclusions must be traceable: `dev-tools/verify.ps1` carries an inline comment pointing back to
   this policy (Regla 2 traceability).
3. **Who can add or revert an exclusion:** only the project lead (`vanta-lead`) after review;
   re-enabling a runner-risk-category test in the fast gate requires evidence that the input size was
   reduced below runner-risk thresholds (e.g. bounded allocations in the test itself).

#### Fast Gate wall-time measurement (HARD-02, 2026-09-27)

`pwsh dev-tools/verify.ps1` (local pre-push gate) measured on a Windows MSVC box
(32GB / 12 cores, `CARGO_TARGET_DIR=target/session-api01`) with parallel sessions
running on the host (load 67–100%):

| Run | Gate state | Wall time | Verdict |
|-----|------------|-----------|---------|
| #1 | 11 steps (coverage in-line), semi-cold (Cargo.toml touched by parallel work → rebuilds) | 1397s (23.3m) | not representative (rebuild + load) |
| #2 | 11 steps (coverage in-line), warm (steady state) | 509.4s (8.5m) | > 300s target → FIND-163 |
| #3 | **10 steps (coverage moved to nightly — owner decision (c))**, warm | **209.5s (3.5m)** | **✅ < 5 min target met** |

Command: stopwatch around `pwsh dev-tools/verify.ps1` (equivalent to
`Measure-Command`); logs: `target/session-api01/hard02-verify.log`,
`hard02-verify-warm.log`, `hard02-verify-r3.log`.

**Resolution (owner decision (c), 2026-09-27):** the coverage report+budget was the
dominant marginal cost of the local lane (the suite ran twice: plain `nextest` +
instrumented `coverage` re-run). It now lives in `nightly.yml` job
`coverage-budget` (`dev-tools/coverage-budget.ps1`); the local gate keeps
`-IncludeCoverage` for on-demand use. Measured result: **509.4s → 209.5s warm**
(<5 min target met; ADR-0031 §9 / STABLE-00 honored). FIND-163 is resolved by this
change; residual note: run #3 was taken under the same host load as #2.

### Experimental Crate Circuit Breaker

The workspace includes several **experimental crates** that are not part of the core MVP:

| Crate | Description | Status |
|-------|-------------|--------|
| `vantadb-server` | Local HTTP server + MCP stdio binary | Experimental |
| `vantadb-mcp` | Model Context Protocol interface (lib-only; served by `vanta-cli server --mcp`) | Experimental |
| `vantadb-wasm` | WASM bindings for JS/TS SDK | Experimental |
| `providers/openai` | OpenAI embedding adapter (NOT a workspace member — checked via `--manifest-path`) | Experimental |
| `providers/ollama` | Ollama embedding adapter (idem) | Experimental |
| `providers/litellm` | LiteLLM embedding adapter (idem) | Experimental |

**Providers release channel (PROV-12, 2026-10-04):** `providers/{openai,ollama,litellm}` ship
as PyPI wheels (`vantadb-openai` / `vantadb-ollama` / `vantadb-litellm`) via
`release-providers.yml` — maturin matrix 3 providers × 4 platforms (linux x86_64 + aarch64,
macOS, Windows), tag namespace `providers-v*.*.*`, TestPyPI dry-run via `workflow_dispatch`.
The circuit-breaker rules below are unchanged: the release workflow does not touch
`default-members`, workspace clippy or coverage.

**Circuit breaker rules:**

1. **Removed from `default-members`** in root `Cargo.toml` — `cargo check`, `cargo build`, and
   `cargo test` without `--workspace` skip them entirely.
2. **Excluded from `--workspace` clippy** — the `--exclude` flag is used for all three platforms
   (Linux, Windows, macOS) so lint failures in experimental code do not block CI.
3. **Excluded from `--workspace` coverage** — the `cargo llvm-cov nextest` step uses `--exclude` so
   compilation or test failures in experimental code do not block coverage reporting.
4. **Dedicated `experimental-check` job** — runs `cargo check` on all experimental crates with
   `continue-on-error: true`. This provides visibility into experimental crate health without
   blocking the fast lane.

**To promote an experimental crate to stable**, remove it from the exclusion list in
`ci-rust.yml` and re-add it to `default-members` in `Cargo.toml`. The full
promotion DoD (10 checks, per-crate cost table, wall-time budget and
reversibility) is defined in **[ADR-0031: Promotion to default-members](../../dev/architecture/adr/ADR-0031-default-members-promotion.md)** — no crate may be promoted without passing ADR-0031 in 3 consecutive clean runs; see ADR-0031 §Question to Owner for the Fast Gate `<5 min` vs Heavy threshold gate (STABLE-00).

#### Promotion to `default-members` — ADR-0031 DoD (STABLE-00, P47)

`default-members` today is `[ ".", "vantadb-python" ]` (`Cargo.toml:636`). Candidates
`vanta-memory`, `vanta-proxy`, `vantadb-server`, `vantadb-mcp`, `vantadb-wasm`
(Rust) and `vantadb-ts`/`vantadb-node` (npm, equivalent gate) must each pass
the **10-check DoD** before promotion; the checks and their exact commands are
the single source of truth in ADR-0031 (gates 1-10: `cargo check`+`fmt`+`clippy -D warnings`,
`cargo nextest --profile audit`, `cargo deny check`, `validate-docs-coverage`,
workflow `paths:` + no `continue-on-error` + `timeout-minutes <5 min` measured,
`cargo package --dry-run`, `wasm-pack`/`wasm32` for `wasm`, `napi` 7-target matrix +
`npm pack` for `node`, `verify.ps1` wall time `<5 min` with all crates included,
ADR with cost + rollback). **Rollback is 1 line:** `git revert` of `Cargo.toml:636`
(the `publish = false` crates never affect `cargo publish`).

Until ADR-0031 is `accepted` (Owner answers STABLE-00 question A vs B on `<5 min`
vs Heavy), promotion is **blocked** — STABLE-01..08 may validate per-crate gates
but STABLE-09 must not merge. Gate #9 is measured on branch `test/default-all`
with expanded `default-members` (`verify.ps1` / `just verify` cold cache).

**Review 2026-08-05 (TECH-08):** Decision: **keep `vantadb-server`, `vantadb-mcp`, `vantadb-wasm`
EXPERIMENTAL** — not promoted to `default-members`. Evidence: all three compile together
(`cargo check -p vantadb-server -p vantadb-mcp -p vantadb-wasm` → OK, 49s) and their test suites are
green, but the circuit-breaker policy is deliberate: a failure in an experimental crate must not block
core CI, and the planned desktop build (`DESKTOP-01b`) depends on being able to consume these crates
with an empty `[workspace]` decoupling. Re-evaluate after desktop ships.

#### STABLE-09 promotion 2026-09-09 — subset keeping Fast Gate <5min (Owner A)

ADR-0031 `accepted` (Owner chose A 2026-09-09: `<5 min` hard). Full-7 expansion
measured Heavy (STABLE-08: `just verify` cold 8.26m), so only the subset that
keeps `<5 min` is promoted; the rest stays experimental with Heavy justification:

```toml
default-members = [
    ".",
    "vantadb-python",
    "vanta-memory",
    "vantadb-server",
    "vantadb-mcp",
]
```

Excluded (stay in `experimental-check`, non-default): `vanta-proxy` — heaviest
Rust compile, Heavy wall time documented STABLE-02 + ADR-0031 §2 cost table;
`vantadb-wasm` — `wasm32`/`wasm-pack`/`binaryen` toolchain extra, Tier 3
(`wasm-test` BEST-EFFORT `continue-on-error`). `ts`/`node` never enter
`default-members` (npm packages; equivalent gate `release-npm-61.yml`).

Subset measurement 2026-09-09, Windows MSVC box (same host class as STABLE-08):
`cargo check` (default-members) cold 132s ✅; `cargo nextest run --profile audit`
(default-members, as `ci-rust.yml:test`) warm 296s / 2831 passed ✅ (<5min,
steady-state second run; first warm 368s with link churn); `cargo package -p
<memory,server,mcp> --list --allow-dirty` exit 0 ×3 ✅. Cold test-target compile
on Windows MSVC is Heavy (core alone ~785s first build) — pre-existing for the
current default too, not marginal to this promotion; CI `ubuntu-latest` + sccache
warm mitigates (~1.5–2× faster per STABLE-08). `members` unchanged, `Cargo.lock`
delta 0, `publish = false` intact → `cargo publish` unaffected.
**Rollback (1 line):** `git revert <promotion-commit>` (restores
`default-members = [".", "vantadb-python"]`).

#### STABLE-08 measurement 2026-08-27 — branch `test/default-all`, `default-members` expanded, wall time & Heavy verdict (P47 gate 9)

Branch `test/default-all` expands `Cargo.toml:636` to 7 members:

```toml
default-members = [
    ".",
    "vantadb-python",
    "vanta-memory",
    "vanta-proxy",
    "vantadb-server",
    "vantadb-mcp",
    "vantadb-wasm",
]
```

`members` unchanged (7 Rust crates already in `[workspace].members`; `Cargo.lock` delta 0). Simulation measured **locally** on Windows (no `ubuntu-latest` runner yet); wall times are `Measure-Command` per job with `cargo clean`/`npm ci` cold cache where noted. 3 cold runs: `cargo clean` + `just verify` / `verify_changed.ps1` — 0 failed, no flaky (`nextest --profile audit --workspace --build-jobs 2` + `cargo test -p vantadb-server/vanta-memory` already validated STABLE-01/03: 473/473, 42/42).

**Environment (measurement host):** `cargo 1.95.0`, `rustc 1.95.0`, `just 1.55.1`, `pwsh 7.6.5`, `node v24.16.0`, `npm 11.6.0`, `MSVC 14 (BuildTools 2022) + LLVM 19 (LIBCLANG_PATH=C:\Program Files\LLVM\bin)`, `Windows 11 (win32)`, `RAM 31.77 GB (34120724480) / 12 cores → Jobs=4 (gate-common.ps1 Get-AdaptiveJobs)`, `RUST_MIN_STACK 33554432 (verify.ps1) / 16777216 (verify_changed.ps1)`, `rust-toolchain.toml 1.94.1`, `sccache off (cold = cargo clean deletes target/)`. CI `ubuntu-latest` wall times will differ (~1.5-2× faster on Linux sccache warm, slower on cold due to no MSVC overhead) — numbers below are order-of-magnitude baseline for ADR-0031 §2 cost table.

**`just verify` (Justfile `verify: fmt clippy test deny` — uses `--workspace` directly, so `default-members` expansion does NOT change its `--workspace` check; measured with expanded `Cargo.toml` to confirm Heavy is clippy/nextest compile, not default-members filtering):**

| Job | Command (gate) | Warm (target present) | Cold (cargo clean) | Timeout (ci-rust.yml) | Verdict |
|-----|----------------|----------------------|--------------------|--------------------------|---------|
| `fmt` | `cargo fmt --check` | 2.54s | 2.11s (cold fmt unaffected) | 10m | Fast |
| `clippy` | `cargo clippy --workspace --all-targets --all-features -- -D warnings` | 10.73s (incremental) | **>600s timeout (10 min)** — cold full rebuild with `all-features` across 7 crates (tantivy + roaring + server/mcp/wasm) timed out at 600s; warm after cold is 10.7s. Estimated cold ~650-900s on Windows without sccache. | 15m | **Heavy** (cold >5 min, even warm clippy 10s <5 but full pipeline cold dominates) |
| `test` | `cargo nextest run --profile audit --workspace --build-jobs 2` | 234.92s (3.91m) | not re-measured cold separately (warm already 3.91m; cold + clippy compile share target, estimated 400-500s). Full `just verify` cold first run measured 495.5s (8.26m) total (fmt+clippy+test+deny) — see below. | 30m (test Linux) | Cold >5 min |
| `deny` | `cargo deny check` | 1.75s | 1.75s | 5m | Fast |
| `audit` | `cargo audit` (not in `just verify`, but in `verify.ps1`) | 4.91s | 4.91s | 5m | Fast |
| **Total `just verify`** | `fmt + clippy + test + deny` (sequential local) | **~249s (4.15m) incremental warm** (fmt 2.5 + clippy 10.7 + test 234.9 + deny 1.75) — after initial cold build; **first run after clean 495.5s (8.26m)** `Measure-Command { just verify }` (`C:\Users\Eros\AppData\Local\Temp\just-verify-warm.log`) | **Cold >5 min (8.26m first run, clippy cold >10m)** | Fast Gate <5 min invariant (ADR-0031 §9, `docs/dev/operations/CI_POLICY.md` §1) | **Heavy** (cold fails <5) |

**`dev-tools/verify_changed.ps1` (quick gate, `fmt → check -p vantadb → clippy -p vantadb`, `-j 2`, `Get-CoreFeatures cli,fjall,memmap2,fs2,roaring`):**

| Run | Cache | Wall time | Verdict |
|-----|-------|-----------|---------|
| 1 | Cold (`cargo clean`) | 115.14s (1.92m) | **<5 min Fast** |
| 2 | Warm (incremental) | 8.08s | Fast |
| 3 | Warm | 8.41s | Fast |
| Warm baseline (prior to clean) | Warm | 7.15s | Fast |

`verify_changed.ps1` stays <5 min even cold (115s) — it checks only `-p vantadb` (single crate, 5L job), not `--workspace`. It is **not** the Heavy gate; `just verify`/`verify.ps1` with `--workspace` is.

**`cargo check` baseline (default-members impact):**

| Command | Members | Cache | Wall time |
|---------|---------|-------|-----------|
| `cargo check` (no args → uses `default-members`) | `[ ".", "vantadb-python"]` (current) vs `[ ".", "vantadb-python", "vanta-memory", "vanta-proxy", "vantadb-server", "vantadb-mcp", "vantadb-wasm"]` (expanded) | Warm | 71.65s (1.19m) with expanded (vs ~5.3s for `cargo check -p vantadb` single crate) |
| `cargo check -p vantadb --no-default-features --features cli,fjall,memmap2,fs2,roaring` (gate-common core) | single | Warm 0.43s / Cold 53.39s | Fast |
| `cargo check --workspace --all-targets` | all 7 | Warm 3.52s (incremental) | Fast warm, cold dominated by clippy |

**`cargo nextest` expanded cost (all crates, `--workspace`):** `cargo nextest run --profile audit --workspace --build-jobs 2` warm 234.9s already includes `vanta-memory` 473 tests + `vantadb-server` 42 + `vantadb-mcp` 62 + `vantadb-wasm` + root suite. Cold would be + clippy compile time (>600s) sharing target, so total pipeline cold >800s if measured fully cold.

**`web` gate (`release-npm-61.yml:tests`, `npm ci + tsc --noEmit + vitest run`):**

| Job | Warm | Cold (`npm ci` fresh) |
|-----|------|-----------------------|
| `npm ci --prefix web` | 70.47s | 70.47s (cold includes download; sccache not applicable) |
| `npx tsc --noEmit` | 13.98s | 13.98s |
| `npx vitest run` (264 tests per `release-npm-61.yml` 27s) | 18.98s (measured) | 18.98s |
| **Total web `tests`** | **~103s (1.72m)** | ~103s | Fast <5 |

**3 corridas `cargo clean` + `npm ci` sin flaky (STABLE-01/03 gates 1-6 already 0 failed):** Runs 1-3 above (verify_changed cold/warm/warm + just verify warm 2.5s/10.7s/234.9s + web 103s) all 0 failed, 0 flaky (nextest 473/473 vanta-memory, 42/42 server, deny ok, fmt ok, clippy -D warnings 0). Heavy gate is **not** flaky — deterministic cold compile time, not test instability.

**Verdict gate 9 (ADR-0031 §9):** `just verify` / `cargo clippy --workspace --all-targets --all-features` + `nextest --workspace` **exceeds `<5 min` on cold cache** (495.5s first run, clippy cold >600s timeout) on Windows 32GB/12c host without sccache. Warm incremental (<5: 249s 4.15m) passes, but **cold fails** — per ADR-0031 §Question to Owner, this requires **Heavy label with justification** (or scoped promotion). Measurement on `ubuntu-latest` with sccache warm will be faster but cold without sccache will still be >5 (7 Rust crates + `all-features` + tantivy WASM). Until Owner answers STABLE-00 question **A (<5 hard — do not promote slow crate)** vs **B (<5 soft — re-label Fast Gate to ~8 min)**, promotion in STABLE-09 stays **blocked**; this crate set must stay `CATEGORY: EXPERIMENTAL` / `experimental-check` non-blocking. If Owner chooses A, promote only subset that keeps `<5` cold (e.g., `[ ".", "vantadb-python", "vantadb-server", "vantadb-mcp"]` without `vanta-memory`/`vanta-proxy`/`vantadb-wasm`) and re-measure; if B, update `CI_POLICY.md` Fast Gate invariant to `~8 min`, bump `ci-rust.yml:clippy`/`test` `timeout-minutes` and `dev-tools/verify.ps1` comments.

*STABLE-08 measurement recorded 2026-08-27, branch `test/default-all` (local simulation, not pushed). `Cargo.toml` revert before STABLE-09; `Cargo.lock` delta 0 (already members).*

### Experimental Suite

Experimental tests are retained for local/manual diagnostics but do not define the v0.1.x MVP. Run
them explicitly with:

```bash
cargo nextest run --profile experimental --workspace --features experimental
```

Failures in this suite should be triaged, but they do not block the Fast Gate unless the failure is
caused by a change to production-facing MVP behavior.

### 2. Heavy Certification (`heavy-certification.yml`)

The heavy certification suite validates the engine's capability to run under production stress,
ensuring recall guarantees and scaling limits. **Goal:** Validate engine stability, recall, and
scale capabilities without bottlenecking daily development.

**Jobs:**

| Job | Description |
|-----|-------------|
| `stress-protocol` | Dynamic scaling (10K, 50K, 100K vectors), persistence, latency, 0.95+ Recall@10 |
| `hnsw-validation` | HNSW validation |
| `hnsw-recall` | HNSW recall certification |
| `sift-validation` | SIFT-1M validation (manual opt-in) |
| `competitive-bench` | Competitive benchmarks vs FAISS/HNSWlib (manual opt-in) |
| `failpoint-tests` | `chaos_integrity`, `wal_resilience`, `crash_injection` — crash recovery with `--features failpoints` |
| `storage-persistence` | Storage persistence & recovery (backend, durability, GC, schema evolution, etc.) |
| `text-index` | Text index recovery (`text_index_recovery`) |
| `memory-concurrency` | Memory & concurrency heavy tests (concurrency, memory brutality, fuzz proptest) |
| `other-heavy` | Remaining heavy tests: CLI, benchmarks, hybrid ranking, MCP (`vantadb-mcp`), columnar, `concurrent_insert_preserves_hnsw_invariants` |

**Why are these tests separated?** Running `stress_protocol` can take close to 2 hours on hosted
runners and requires significant system resources (AVX2 plus heavy swap). It runs in its own
scheduled/manual job with a 150 minute step timeout so it can complete without blocking the other
certification checks. Running this on every PR would paralyze development velocity.

### 2b. Nightly Certification Subset (`nightly.yml`, HARD-02)

Added 2026-09-27 (HARD-02, owner decision 2026-09-26 (c)). The heavy
certification suite above remains the **weekly full** lane; `nightly.yml` runs
a **daily subset** of the deterministic core certification jobs so a regression
in crash recovery / durability / HNSW correctness / text-index recovery
surfaces in ≤24h instead of ≤7d:

| Job (nightly.yml) | Mirrors heavy-certification.yml | Timeout |
|---|---|---|
| `failpoint-tests` | same job (`chaos_integrity`, `wal_resilience`, `crash_injection`) | 30m |
| `storage-persistence` | same job (16 binaries, `--release --features cli`) | 90m |
| `hnsw-validation` | same job | 120m |
| `hnsw-recall` | same job (`hnsw_recall_certification`) | 40m |
| `text-index` | same job (`text_index_recovery`) | 60m |
| `coverage-budget` | NEW (owner decision (c) 2026-09-27) — `dev-tools/coverage-budget.ps1`: coverage report + per-directory budget (see §Coverage) | 60m |
| `release-dry-run` | NEW — `cargo publish -p vantadb --dry-run` (packaging + verify, **never uploads** — release-plz remains the only publisher) | 45m |

- **Why a separate file, not a daily `heavy-certification.yml`** (decision
  evidence): the full suite contains jobs that are deliberately weekly —
  `stress-protocol` (~2h), `memory-concurrency`, `mutation-test` (non-gating by
  design) — and running them daily multiplies flake exposure without adding
  per-day signal. The nightly subset covers the failure-critical lanes; the
  full suite stays weekly (see §"Running Heavy Certification Manually").
- **Reception point for slow gates:** gates measured >5min in the fast gate are
  moved here (HARD-01 stop condition; e.g. future `cargo semver-checks` /
  public-api snapshot steps land as jobs in `nightly.yml`).
- **Coverage (owner decision (c) 2026-09-27):** the per-directory coverage budget
  runs here (job `coverage-budget`); the local gate keeps it on-demand only
  (`verify.ps1 -IncludeCoverage`) — see §"Coverage — Report & Per-Directory Budget".
- **Schedule:** daily 05:00 UTC — deconflicted (FIND-141): bench nightly 02:00,
  full cert Sun 03:00, OCR nightly 04:00.
- **Notification:** on failure, `notify-failure` opens (or comments on) the
  auto-issue `[Nightly] Heavy certification subset failed` listing the failed
  jobs; `issues: write` is granted **only** on that job (RULES.md §4). Badge:
  README (`nightly.yml` workflow badge).
- **Command coupling:** the job commands are copies of their
  `heavy-certification.yml` counterparts — a test-binary list change edits both
  files together (same coupling rule as `.config/nextest.toml` structural
  exclusions).

### Coverage — Report & Per-Directory Budget (nightly) + CI canonical gate (HARD-02)

**Decisions (owner):** 2026-09-26 (a) — the local gate no longer blocks on a global
line-coverage threshold; coverage becomes a **report + per-directory budget** (ratchet).
2026-09-27 (c) — the report+budget **moves out of the local fast gate** into the
nightly lane (`nightly.yml` job `coverage-budget`, script
`dev-tools/coverage-budget.ps1`), restoring the local fast-gate `<5 min` target
(see §"Fast Gate wall-time measurement"). The CI canonical gate
(**ADR-0018: root crate ≥80%**, `ci-rust.yml` coverage job) is **unchanged** by both
decisions. Rationale (R1): a single global threshold is a lossy gate — a
per-directory ratchet keeps the signal localized and loud, and the coverage re-run
was the dominant cost of the local lane.

**Where it runs:**

| Lane | Command | Semantics |
|------|---------|-----------|
| Nightly (enforcement) | `pwsh dev-tools/coverage-budget.ps1` — `nightly.yml` job `coverage-budget` (ubuntu-latest, timeout 60m) | JSON report + budget; exit 1 only on explicit bucket violation; JSON uploaded as `coverage-report` artifact |
| Local on-demand | `pwsh dev-tools/coverage-budget.ps1` (or `pwsh dev-tools/verify.ps1 -IncludeCoverage`) | same script, same budgets |
| CI canonical (unchanged) | `ci-rust.yml` coverage job | root crate ≥80% (ADR-0018) |

**Command (exact — regenerates the report artifact):**

```bash
cargo llvm-cov nextest --profile audit -p vantadb \
  --no-default-features --features cli,fjall,memmap2,fs2,roaring \
  --build-jobs 1 \
  -E "not test(/deserialize_absurd_node_count/) and not test(/test_search_with_bizarre_text_query/) and not test(/test_malformed_payload_extremely_large/)" \
  --json --output-path <target-dir>/coverage-report.json
```

Without re-running tests, the JSON can be re-generated from collected profdata with
`cargo llvm-cov report --json --output-path <target-dir>/coverage-report.json`.
`dev-tools/coverage-budget.ps1` runs the full command, aggregates `data[0].files` by
directory (`src/<dir>`; `src/<file>.rs` → `src (root files)`) and compares each bucket
against the budget table. **Fail rule:** a budgeted directory below its budget fails
the script (explicit ratchet violation); non-budgeted directories are report-only.

**Per-directory budget** (baseline measured 2026-09-26, Windows MSVC box, full pass
2263/2263 with the command above; re-checked 2026-09-27 via `cargo llvm-cov report
--json` aggregation — stable; budget = baseline rounded − 1.0 pt; the mechanical copy
lives in `dev-tools/coverage-budget.ps1` `$CoverageBudget` and mirrors this table —
update both together. Platform note: the nightly runs on ubuntu-latest while the
baseline is Windows; expected delta <1pt (only 24 `cfg(windows/unix)` lines across
`src/`) — if the first ubuntu run legitimately differs, recalibrate the baseline from
the uploaded `coverage-report` artifact and record it here):

| Directory | Baseline (lines) | Budget | Command |
|-----------|------------------|--------|---------|
| `src (root files)` | 91.0% | 90.0% | command above (via `coverage-budget.ps1`) |
| `src/vector` | 95.7% | 94.7% | idem |
| `src/index` | 87.4% | 86.4% | idem |
| `src/storage` | 89.0% | 88.0% | idem |
| `src/sdk` | 90.1% | 89.1% | idem |
| `src/parser` | 97.2% | 96.2% | idem |

**History / supersession:** this replaces the P2-06 local wiring
(`--fail-under-lines 60` blocking in `verify.ps1`, removed 2026-09-27 by HARD-02),
re-anchors the TBH-21 review cadence (previously tied to the `60` literal), and
supersedes HARD-02's interim placement of the report+budget in `verify.ps1`
(owner decision (c) moved it to the nightly the same day). P2-06's original intent
(mechanical coverage enforcement) survives as the per-directory budget + the
untouched ADR-0018 CI gate.

**Review cadence (TBH-21, re-anchored):**

| Field | Value |
|-------|-------|
| **Budgets** | per-directory table above (baseline − 1.0 pt) |
| **Review cadence** | **Quarterly** (every 90 days) |
| **Last reviewed** | 2026-09-27 (HARD-02 — owner decisions (a)+(c)) |
| **Next review due** | 2026-12-26 (or earlier if a budget violation is observed) |
| **Owner** | `vanta-lead` (release/CI orchestrator) |
| **Source of truth** | This section + `dev-tools/coverage-budget.ps1` (`$CoverageBudget`) |

**What "review" means:** at each checkpoint, regenerate the report with the command
above (or download the nightly `coverage-report` artifact) and compare each budgeted
directory against its baseline. If a bucket climbed materially (e.g. ≥+5 pt), ratchet
baseline/budget upward in both places. Lowering a budget requires a documented
justification and the same owner sign-off as the original decision. If
`cargo-llvm-cov` is missing, the script prints a warning and exits 0 (missing tool
never blocks `just verify`).

**Test exclusion:** cargo-llvm-cov excludes `tests/` directories and `*_tests.rs`
files from the report by default; bench files outside `src/` are excluded from
budgets by the `src/*` bucket filter. The `-E` filter is the same RESOURCE-GUARD
exclusion set documented in §"Fast Gate Test Exclusions" (single source:
`dev-tools/gate-common.ps1` `Get-FastGateFilter`).

**Policy decision (COV-004, 2026-08-09):** the strategic coverage policy — root crate vs workspace
aggregate vs per-runner binding measurement — is decided in
[ADR-0015](../../dev/architecture/adr/ADR-0015-coverage-policy.md) (accepted, owner TBD). In force:

- Gate canónico (ADR-0018, supersede ADR-0015 §D1): root crate antadb ≥ 80% line (baseline
  81.40%). Workspace aggregate (root + antadb-python, 72.76% medida) se reporta solo para
  visibilidad. Nunca bajar el umbral de 80% para acomodar un baseline.
  to accommodate a baseline.
- Bindings are measured on their native runners: Python wrapper coverage ≥ 85% via pytest;
  WASM/MCP/server carry no coverage gate while experimental (Tier 3).
- No new `--fail-under` gate is added for the aggregate. Revisit on release or when a binding
  graduates from experimental.

### 3. Web CI (`ci-web.yml`)

Builds and lints the web frontend (`web/` directory — Next.js 16). Runs `npm ci`, `npm run lint`,
`npx tsc --noEmit`, and `npm run build` on push to `main`+`develop` and PR to `main` that touch `web/**`. No test infra
— the Next.js SPA is client-only (`"use client"` everywhere). Triggered by `workflow_dispatch` as
well.

### 4. Docs Gate (`gate-docs.yml`)

Lints Markdown files in `docs/**` with `markdownlint-cli2`. Triggered on push/PR to `main`+`develop`
touching `docs/**` (plus router/scripts paths).

### 5. Security Scan (`sec-codeql.yml`)

CodeQL analysis for Rust. Runs on push/PR to `main` and weekly. Triggers via `workflow_dispatch`.

### 6. Fuzzing (`fuzz.yml`)

LibFuzzer corpus + regression via `cargo fuzz`. Scheduled weekly (Monday 06:00 UTC), PRs touching
`src/**`/fuzz paths, or `workflow_dispatch`.

### 7. Performance Benchmarks (`perf-bench.yml`)

Python integration performance benchmarks. Triggered on push to `main`+`develop` touching core or
Python paths, or via `workflow_dispatch` with configurable vector/queries/dim inputs.

### 8. Nightly Benchmarks (`heavy-bench-nightly.yml`)

Nightly benchmark regression suite (daily CRON 02:00 plus `workflow_dispatch`, PRs touching bench paths). Runs light benchmarks
and heavy benchmarks across multiple package scopes.

### 9. Python Wheel Build & Publish (`release-wheels.yml`)

Builds the Python SDK on Linux (x86_64 and aarch64 cross via maturin-action's manylinux_2_28
cross container), macOS, and Windows with `maturin`, installs the generated wheel
by resolved path, and runs the Python SDK smoke suite (native targets only — the aarch64
cross wheel is validated by its platform tag and the PyPI install verify job). Manual TestPyPI upload is available only
through an explicit workflow input and the `TEST_PYPI_API_TOKEN` secret. Production PyPI
publication and signing remain deferred.

### 10. Release Workflows

| Workflow | File | Trigger |
|----------|------|---------|
| Release Automation | `release.yml` | Push to `main` — `release-plz` auto-version, changelog, tag, publish |
| Python Wheels | `release-wheels.yml` | Tag `v*.*.*`, `pull_request` to `main` (paths src/python), or `workflow_dispatch` |
| NPM Publish (WASM+TS) | `release-npm-61.yml` | Tag `v*.*.*`, `pull_request` (paths wasm/ts), or `workflow_dispatch` — includes Fast Gate job `tests` (`npm ci && npm run build && npx vitest run`, measured 27s <5min, no `continue-on-error`, PR+tags gate per TS-06) |
| NPM Publish (Node) | `release-npm-node.yml` | Tag `node-v*.*.*`, `pull_request` (paths node), or `workflow_dispatch` |
| PyPI Adapters | `release-adapters.yml` | Tag `adapters-v*.*.*` or `workflow_dispatch` (TestPyPI) |
| Binary Builds | `release-binaries.yml` | Release published or `workflow_dispatch` |
| SBOM Generation | `release-sbom.yml` | Tag `v*` or `workflow_dispatch` |

## External Dependencies (Ollama/LLMs)

VantaDB integrates with external LLMs for embeddings and semantic queries. However, **integration
tests requiring network access to LLMs (like Ollama) are strictly excluded from the Fast Gate.**
They are either marked with `#[ignore]` or gated behind environment variables (e.g.,
`VANTADB_RUN_LLM_TESTS=1`). This ensures the core engine can be built and tested completely offline.

## Running Heavy Certification Manually

The `heavy-certification.yml` workflow runs automatically via a CRON schedule (weekly on
Sundays at 03:00 UTC). The scheduled lane runs the local deterministic core certification jobs.
SIFT-1M validation and competitive benchmarks are manual opt-ins because they require external
datasets. You can also trigger it manually from the GitHub Actions UI:

1. Navigate to the **Actions** tab in the repository.
2. Select **HEAVY: Certification — All Tests** from the left sidebar.
3. Click **Run workflow**.
4. You can optionally check the boxes to include `SIFT-1M validation` or `Competitive benchmarks`.

## Docker Image Publishing — retired (2026-10-02)

Docker support was removed repo-wide by owner decision: the `docker-image` job was deleted from `release-binaries.yml`, the Dockerfiles/compose files were removed, and deployment docs no longer cover containers.


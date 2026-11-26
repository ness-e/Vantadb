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

VantaDB has **20 active workflow files** in `.github/workflows/`. Each workflow is documented below. See `docs/dev/workflow/README.md` (inventory) and `docs/dev/workflow/TRIGGERS.md` (trigger matrix - source of truth is each file's `on:` block; matrix covers active files).

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

#### TSan lane — retired 2026-11-25 (FIND-734; previously `sanitizer-tsan`, FIND-714)

**No ThreadSanitizer job runs in CI.** The job was removed from
`.github/workflows/ci-rust.yml`, and so was its `dev-tools/tsan-suppressions.txt`. It could not
produce a verdict about VantaDB, and the alternative — a chronic red behind `continue-on-error` —
is the case Regla 2 forbids. This section keeps the measurement, the reason, and the recipe to
bring the lane back.

**What was measured.** Run `45668284667`, job `139404898566` (partition 1/4, `b7a5d2eab`). For the
first time the lane ran its corpus instead of dying after four tests: `822 tests run (selected ==
executed)`, no abort, the FIND-714 assertion green — while reporting `race reports: 243` and
`suppression hits: 0`. All 243 belong to one class, and in every one of them **both disputed
accesses sit inside uninstrumented `std`/`core`/`alloc`**, which is why TSan cannot adjudicate any
of them. That is a statement about the instrument, not a clean bill of health for the engine:

| Reports | Signature | What it actually is |
|---|---|---|
| 176 | `Arc<Mutex<Option<test::run_test::{closure#0}>>>::drop` vs `compiler_move` (`library/test/src/lib.rs:687`) | libtest's own event plumbing, guarded by a `std::sync::Mutex` |
| 58 | `free`/`dealloc_nonnull` vs `Atomic<usize>::fetch_sub` on `Arc<flume::Hook<fjall::worker_pool::WorkerMessage, flume::signal::SyncSignal>>` | an `Arc` refcount during fjall/flume teardown — all of it inside `std` |
| 8 | tokio `blocking::pool`: 7 shutdown (`ThreadManagementState::begin_shutdown`), 1 spawn (`VecDeque::push_back_mut`) | a `std::sync::Mutex` doing its job, invisibly |
| 1 | `OnceLock<DistanceKernels>` init (`kernels.rs:275`) | the futex-based `Once` |

Note what that table does *not* say. 67 of the 243 reports do carry `vantadb::` frames — the drop
chain of `FjallBackend`/`StorageEngine`, `gc::spawn_memory_ttl_sweeper`, the test bodies — but in
every one the *conflicting pair* is `free`/`Atomics`/`MaybeUninit` in `std`. A real double-free or
use-after-free in `FjallInner`'s teardown would print the same shape, and with `std` uninstrumented
TSan cannot tell them apart. That is the reason to retire the lane, not evidence that the engine is
race-free.

**Root cause — the lane cannot see any synchronization.** TSan learns about synchronization from
two sources: libc interception (`pthread_mutex_lock`) and compile-time instrumentation of atomics.
VantaDB's synchronization arrives by neither route. Rust's `std::sync::Mutex`, `Once`/`OnceLock` and
`Arc` refcounts are futex- and atomic-based *inside `std`*; `std` is not instrumented here (no
`-Zbuild-std`, it ships precompiled); and a futex is not a `pthread_mutex`, so the libc interceptor
never fires. The log says it directly: the stack runs through
`<std::sys::sync::once::futex::Once>::call`. The Rust Unstable Book is explicit — *"Using it without
instrumenting all the program code can lead to false positive reports"*, and *"It is strongly
recommended to combine sanitizers with recompiled and instrumented standard library, for example
using cargo `-Zbuild-std` functionality"*
([`sanitizer`](https://doc.rust-lang.org/nightly/unstable-book/compiler-flags/sanitizer.html)).

**Why a suppression file could not fix it.** The mechanism itself works: `race:` is matched against
function *and* file names of the reporting stack and of the previous-access stack, the string being
a substring with `*` implied (see
[ThreadSanitizerSuppressions](https://github.com/google/sanitizers/wiki/ThreadSanitizerSuppressions)
and `IsSuppressed` in
[`tsan_suppressions.cpp`](https://github.com/llvm/llvm-project/blob/main/compiler-rt/lib/tsan/rtl/tsan_suppressions.cpp)).
Two things blocked it, and neither is the format.

*The list expires faster than a nightly cycle.* FIND-714's two entries were loaded by this very run
— `Using suppressions: …/tsan-suppressions.txt (2 entries)` — and scored `suppression hits: 0`
within ~24 h, because libtest replaced its event channel with a different internal type.
`suppression hits` exists to make that rot visible, and it did its job.

*Some signatures can only be named by naming engine types.* Per-family tokens do exist
(`test::run_test::{closure#0}` would cover 176), but warn #42 — the 42nd `WARNING` in that
partition's log — carries no harness and no dependency frames at all:

```
Read of size 8 at 0x5555577e3420 by thread T4:
  #0 vantadb::index::distance::kernels::f32_dot_and_norm_b_sq  src/index/distance/kernels.rs:275
  #1 vantadb::index::distance::metrics::f32_l2_norm            src/index/distance/metrics.rs:14
Previous write of size 8 at 0x5555577e3420 by thread T3:
  #0 <core::mem::maybe_uninit::MaybeUninit<DistanceKernels>>::write
  #1 <std::sync::once_lock::OnceLock<DistanceKernels>>::initialize   once_lock.rs:574
```

`select_kernels()` is `KERNELS.get_or_init(..)` — correct code; the acquire happens in the
uninstrumented `Once`. Silencing that one report means writing a suppression naming
`OnceLock<vantadb::index::distance::kernels::DistanceKernels>` — an engine type — or `MaybeUninit<`.
The remaining 67 are fjall's write queue and tokio's pool, i.e. the queue that carries our writes to
disk. A green reachable only by naming engine and storage types is the `race:std` failure mode
FIND-227 rejected, one level down — and it has to be re-earned every time those internals move,
which the 24-hour expiry already demonstrated.

**Bringing the lane back.** The prerequisite is an instrumented `std`, not a suppression:

```
export RUSTFLAGS="-Zsanitizer=thread -Cunsafe-allow-abi-mismatch=sanitizer"
cargo +nightly nextest run --target x86_64-unknown-linux-gnu -Zbuild-std <same partitions as before>
```

`rust-src` must be installed (`rustup component add rust-src --toolchain nightly`, seconds) and the
build gets slower — measured, not estimated: on a Windows dev box at `rustc 1.101.0-nightly
(c36f14571)`, cold cache and no sccache, compiling instrumented `std` for
`x86_64-unknown-linux-gnu` took **3.2 min** (164 MB of artifacts) before the run stopped at the link
step, which needs a Linux linker that host does not have. That is the *added* cost per leg over
today's precompiled `std`, it does not include the `-p vantadb` dependency graph, and partitions 2-4
should get it from sccache — so the 50-minute step budget is not the obstacle. Only then is
`race reports: 0` a pass condition worth trusting, because only then does TSan see
`std::sync::Mutex`. A returning lane ships with `-Zbuild-std` from day one, without
`continue-on-error`, with no suppression file, and as a scheduled heavy job — never as a red check
on every pull request. Revival tracked as `FIND-737`.

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
| `test_search_with_bizarre_text_query` | `tests/security.rs:639` | Feeds giant malformed text queries (100KB strings, NUL bytes, astral-plane chars) into search; robust behavior against such inputs belongs to the dedicated fuzzing lane (`certification.yml` fuzz jobs), not the fast gate | RESOURCE-GUARD | Same |
| `test_malformed_payload_extremely_large` | `tests/security.rs:324` | Ingests a 1MB payload plus 10KB of metadata; same rationale — hostile-input coverage is delegated to fuzzing | RESOURCE-GUARD | Same |

**Structural exclusions in `.config/nextest.toml`:** the `default-filter` of the `audit` profile
additionally excludes ~55 heavy test binaries (stress_protocol, chaos_integrity, wal_resilience,
sift_validation, competitive_bench, etc.) via package-qualified `not (package(X) and binary(Y))`
clauses (BND-06 scope-safe form). That list implements the two-tier split documented in this file
(Fast Gate vs Heavy Certification) and changes only together with `certification.yml`.

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

`default-members` today is `[ ".", "vantadb-python", "vanta-memory", "vantadb-server", "vantadb-mcp" ]` (`Cargo.toml`) — `vanta-proxy`/`vantadb-wasm` promoted to members but stay out of defaults (STABLE review below). Candidates
`vanta-memory`, `vanta-proxy`, `vantadb-server`, `vantadb-mcp`, `vantadb-wasm`
(Rust) and `vantadb-ts`/`vantadb-node` (npm, equivalent gate) must each pass
the **10-check DoD** before promotion; the checks and their exact commands are
the single source of truth in ADR-0031 (gates 1-10: `cargo check`+`fmt`+`clippy -D warnings`,
`cargo nextest --profile audit`, `cargo deny check`, `validate-docs-coverage`,
workflow `paths:` + no `continue-on-error` + `timeout-minutes <5 min` measured,
`cargo package --dry-run`, `wasm-pack`/`wasm32` for `wasm`, `napi` 2-target matrix (pre-pub recorte de 7) +
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
`default-members` (npm packages; equivalent gate `release-npm-wasm.yml`).

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

**`web` gate (`release-npm-wasm.yml:tests`, `npm ci + tsc --noEmit + vitest run`):**

| Job | Warm | Cold (`npm ci` fresh) |
|-----|------|-----------------------|
| `npm ci --prefix web` | 70.47s | 70.47s (cold includes download; sccache not applicable) |
| `npx tsc --noEmit` | 13.98s | 13.98s |
| `npx vitest run` (264 tests per `release-npm-wasm.yml` 27s) | 18.98s (measured) | 18.98s |
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

### 2. Heavy Certification (`certification.yml`)

The heavy certification suite validates the engine's capability to run under production stress,
ensuring recall guarantees and scaling limits. **Goal:** Validate engine stability, recall, and
scale capabilities without bottlenecking daily development.

> Renamed from `heavy-certification.yml`/`heavy-certification-50.yml`; the real file is
> `certification.yml` (weekly CRON, dispatch, and path-filtered PRs).

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
| `mutation-test` | **Gate on a pinned surface** (FIND-717) — `cargo mutants --file <pinned> -p vantadb` **runs `cargo test --lib` per mutant**; threshold is **0 survivors** (exit 2 = red), budget 90 min step / 105 min job. Measured on the pinned surface: 9 mutants, 7 caught, 2 missed (the two are named exclusions with a documented test to write). The crate-wide inventory (12,676 mutants) is published as context, never as coverage. See §"FIND-717 — the mutation gate, and what it does not cover" |
| `light-benchmarks` | Bench series, daily 02:00 cron (`hnsw_pure`, `hybrid_queries`, `canonical_p99`, `ingestion_concurrent`, …). Recovered by FIND-716 (job-level `concurrency` collision removed); cap 240 min. FIND-716 |
| `high-density` | `high_density` bench at the highest density profile, daily 02:00 cron. Recovered by FIND-716; cap 360 min |
| `analyze` | Merges the bench result dirs and publishes the comparison (`always()`, tolerates missing dirs) |
| `critcmp-regression` | Opt-in critcmp comparison against a previous `critcmp-baseline` artifact (`enable_critcmp` input) |

**Why are these tests separated?** Running `stress_protocol` can take close to 2 hours on hosted
runners and requires significant system resources (AVX2 plus heavy swap). It runs in its own
scheduled/manual job with a 150 minute step timeout so it can complete without blocking the other
certification checks. Running this on every PR would paralyze development velocity.

#### FIND-328 triage — the three red jobs (2026-11-24)

The backlog row named `heavy-certification.yml`, a file that no longer exists (the tier is
`certification.yml` since `a4c290914`, 2026-10-15). The triage command from the row returned
`HTTP 404: workflow heavy-certification.yml not found`, which is why a 9-day red stayed
invisible: nobody was looking at the file that actually runs. Use
`gh run list --workflow certification.yml` and `gh run view <id> --log-failed`.

Verdicts, evidenced on 25 consecutive scheduled runs (2026-11-06 → 2026-11-24, per-job conclusions
read from `gh api .../runs/<id>/jobs`):

| Job | Verdict | Evidence |
|-----|---------|----------|
| `memory_concurrency` | **Already green — no change.** Not retired: it is the concurrency gate Regla 8 requires. | `success` on all 25 scheduled runs since 2026-11-06 (e.g. run 45388973797, 11:57:56 → 12:06:32). Job body is identical to the pre-consolidation `heavy-certification.yml` (only `timeout-minutes` 120→180 differs), so the fix landed in code/tests, not CI (candidates in the original red window: `6b8002c1c`, `23c594f4f`, `8aad0cac1` on `tests/edge_cases.rs` — batch 1 of this job). Exact commit not isolable: the deleted workflow's runs are no longer enumerable. |
| `other_heavy` | **Fixed (CI).** TBH-01 gate scoped to the dataset this job consumes and moved after the download. | Failure was `=== verify_datasets: 5 missing / 5 expected ===` + exit 1 at 11:59:58, i.e. before the step that provides GloVe-100. The 5-dataset gate was unsatisfiable on a hosted runner (SIFT-1M / GloVe-300 / ground-truth subsets belong to the opt-in `sift-validation` / `competitive-bench` jobs). `scripts/verify_datasets.sh` keeps its exit 0/1 contract for local and opt-in use. |
| `mutation_test` | **Fixed (CI) + re-categorized `INFORMATIONAL`** *(superseded by FIND-717, which turned it into a gate on a pinned surface — see the row above and §"FIND-717")*. Invalid action pin replaced; the step now produces an inventory instead of a fake "score". | It died in "Set up job" every run: `Unable to resolve action actions/upload-artifact@4e7e4d1bb8e5ae8b5b8e6dad68a47e8afbd4d76a` — that SHA does not exist (v4.5.0 is `6f51ac03…`; the repo-wide valid pin is `cf430e03…` = v7.0.2). Second defect: `--check` only cargo-checks mutants ("don't actually run the tests") and `--output-json` is not a flag, so even with a valid pin the step would have exited non-zero (cargo-mutants maps a clap usage error to `ExitCode::Usage` = 1) and the trailing `|| true` would have turned the job **green with no artifact**. |

**Category rule for this job (`mutation-test`):** it is a **gate** since FIND-717, so it carries
no `continue-on-error` and no `|| true` (Regla 2); `if-no-files-found: error` keeps the "green but
did nothing" mode closed. What changed is what it *measures*: `--check` never ran a test (it only
cargo-checks each mutant), so the old job could not produce a mutation score even with a valid
action pin. Contract and numbers: §"FIND-717 — the mutation gate, and what it does not cover".

#### FIND-717 — the mutation gate, and what it does not cover (2026-11-26)

`mutation-test` now measures. Selection is a **pinned surface** (`env.MUTANTS_FILES`), the run
executes the tests **per mutant**, and the threshold is **0 survivors**.

Why a pinned surface and not "mutation coverage" — measured in this repo with `cargo-mutants`
27.1.0 (`--list` parses, it does not compile, so counting is cheap):

| Scope | Mutants | Command |
|---|---|---|
| Crate `-p vantadb` | **12,676** (231 files) | `cargo mutants --list -p vantadb` |
| Diff since `v0.9.0` | **3,741** (167 files, +37,520/−4,341) | `git diff v0.9.0..HEAD -- 'src/**'` → `--list --in-diff` |
| Pinned surface (2 files) | **9** listed → **7 tested** | `--list -f src/physical_plan/dedup.rs -f src/index/search/pool.rs --exclude-re <known survivors>` |

The cost per mutant is **not** the test suite (74 s for `cargo test --lib -p vantadb` warm): it is the
recompile of the mutated crate — 183–443 s per mutant measured on Windows 12c/32 GB, plus the baseline
build before the first mutant. A crate-wide score is weeks of CPU, and the diff since the last release is
still ~3,700 mutants, so the budget fixes the **number of mutants**:

| Knob | Value | Why |
|---|---|---|
| `MUTANTS_FILES` | `src/physical_plan/dedup.rs`, `src/index/search/pool.rs` | 9 mutants, high-value (query dedup, search pool), unit tests in the same modules |
| `MUTANTS_KNOWN_SURVIVORS` | 2 named mutants | measured survivors (real test gaps) excluded so "0 survivors" keeps meaning "no NEW survivor"; each one is deleted in the PR that adds its test — see FIND-717 §Deuda |
| `MUTANTS_MIN_SURFACE` / `MUTANTS_MAX_SURFACE` | `6` / `12` | two-sided guard: the floor stops the surface from being trimmed to a single file, the ceiling fails in seconds (with the reason) instead of timing out 90 min later; the margin above today's 7 absorbs a new branch in the pinned files |
| step timeout | 90 min | 12 × 4.4 min + baseline ≈ 57 min worst case measured, and the exact command on today's 7 mutants took **35 min** (exit 0) |
| job timeout | 105 min | step + checkout/install/inventory |
| `--timeout` | **not set** | the flag caps the **test** phase only, and a fixed 90 s cap classified a healthy mutant as `TIMEOUT` on a loaded machine (measured: the same mutant took 42–64 s in one run and died at 90 s in another). The default — 5× the baseline test — adapts to the runner |
| `--build-timeout` | 900 s | `--in-place` disables build timeouts entirely and `--cap-lints=true` can make rustc hang; 900 s is ~2× the slowest measured per-mutant build (443 s) |
| threshold | **0 survivors** | a survivor is an untested behaviour; no invented percentage. Measured: 9 mutants → 7 caught + 2 missed (exit 2); with the 2 named exclusions, the shipped command runs 7 mutants → 7 caught, **35 min, exit 0** |

**What it does not cover, said out loud:** 7 of 12,676 mutants under the threshold, and `cargo test --lib`
only. It is a starter surface with a hard threshold, not crate coverage — the step summary prints `surface`
and the excluded count next to `crate` so the green cannot be misread.

Growth paths, cheapest first: add a file after measuring it (`--list` count × 4.7 min); shard
(`--shard k/n` + matrix, [mutants.rs/shards](https://mutants.rs/shards.html)) to grow the surface
without growing wall-time; `--iterate` to skip mutants caught in a previous run (needs
`mutants.out` persisted between runs).

Traps recorded for whoever touches this next:

- **`--check` does not measure** ("Cargo check generated mutants, but don't run tests") — it is the
  original TBH-07 command, and it is still the reason the old job could never produce a score.
- **`--in-diff` needs a tree that matches the diff**: with uncommitted edits cargo-mutants aborts
  with exit **5**, "the new side of the `--in-diff` diff doesn't match the text in the tree"
  (reproduced against this repo's dirty `src/`).
- **`--jobs` is incompatible with `--in-place`**: parallelism comes from shards, one process each.
- **A fixed `--timeout` is a false-negative machine**: it bounds the test phase only, so the value
  has to scale with the machine that runs it. Leave it on the default multiplier.
- **A flaky test can fabricate a false `caught`** (measured: `dedup.rs:63` read as `caught` in one run and
  `MISSED` in the next, because an unrelated `..._probe_timeout` test fell over under CPU load). `--baseline=run`
  protects the baseline, not the individual mutants: the signal is only trustworthy when the suite is
  deterministic.

**Resolved (FIND-716, `9e4e04d70`):** `Light benchmarks` and
`High density` used to fail on **every** run with **zero steps** and `runner_name: none` — they were
failed **8 s after start**, not queued: four jobs in this file declared a job-level `concurrency:` block
reusing the workflow's own group, so each job waited on a slot its own run held. The four blocks were
removed (the workflow-level `cancel-in-progress: false` is the single authority); `failpoint-tests` and
`fuzz-pr` keep theirs because they use distinct groups. Cost ceiling when the lanes run again: 7 × (240 +
360) min + ~70 min/week ≈ 71 h of runner capacity — not billed on a public repo, but real in concurrency
and artifact retention. Original note kept below for context. Four jobs in this file declare a job-level `concurrency:` block
that **reuses the workflow-level group** `${{ github.workflow }}-${{ github.ref }}`
(`light-benchmarks` L416, `high-density` L489, `analyze` L524, `critcmp-regression` L697). GitHub
keeps at most one running job or workflow per concurrency group, so a job in that group waits for
a slot its own run holds until the run ends; `queue: single` then cancels the older pending entry,
which is why two of them die before step 1 and the other two only surface when their `if:` is true
(`analyze` on dispatch, `critcmp-regression` with its input enabled). The other two
`concurrency:` blocks in the file are fine (`failpoint-tests` uses `failpoints-${{ github.ref }}`,
`fuzz-pr` appends `${{ github.job }}`). The four colliding blocks are also redundant (the
workflow-level block already sets `cancel-in-progress: false`). Fix is deleting three lines in each
of the four — held for an owner decision because it re-enables a 2 h and a 6 h bench lane.

### 2a. Linux Snapshot Certification (`snapshot-certification.yml`)

The [isolated snapshot lane](../../../.github/workflows/snapshot-certification.yml)
explicitly runs the snapshot binary excluded by the fast nextest filter. It
triggers on snapshot-related push and pull-request changes to `main` and
`develop`; `workflow_dispatch` is available once registered on the default
branch. It does not dispatch the general heavy benchmark/fuzz workflow.

One native Linux job runs seven phases in series: core snapshot/pending-restore
units without and with encryption, complete selected snapshot certification
without and with encryption, the real RocksDB checkpoint round trip,
`vanta-memory` snapshot/rollback consumers, and MCP snapshot tools. The build
profile is Cargo `ci`, the runner is nextest `audit`, with two build jobs and
one test thread. The existing 180-second per-test limit, datasets and assertions
are unchanged. The 90-minute job budget also includes the cold native C++ build.

The artifact contains execution context, per-phase TXT logs, exit codes and
JUnit XML. An empty test selection fails; a failure stops subsequent phases.
An old nextest report is moved aside before each phase so cached XML cannot
stand in for an unexecuted run. Artifacts upload even on failure. In-process
snapshot coherence is enforced by the FIND-651 `insert_lock` barrier; quiesced
writers are required only for out-of-process / raw file mutation.

### 2b. Nightly Certification Subset (`nightly.yml`, HARD-02)

Added 2026-09-27 (HARD-02, owner decision 2026-09-26 (c)). The heavy
certification suite above remains the **weekly full** lane; `nightly.yml` runs
a **daily subset** of the deterministic core certification jobs so a regression
in crash recovery / durability / HNSW correctness / text-index recovery
surfaces in ≤24h instead of ≤7d:

| Job (nightly.yml) | Mirrors certification.yml | Timeout |
|---|---|---|
| `failpoint-tests` | same job (`chaos_integrity`, `wal_resilience`, `crash_injection`) | 30m |
| `storage-persistence` | same job (16 binaries, `--release --features cli`) | 90m |
| `hnsw-validation` | same job | 120m |
| `hnsw-recall` | same job (`hnsw_recall_certification`) | 40m |
| `text-index` | same job (`text_index_recovery`) | 60m |
| `memory-cycle-e2e` | NEW (MEM-73-NIGHTLY, 2026-11-25) — the composed L0→L3 cycle e2e (`vantadb-mcp/tests/l0_l3_cycle_e2e.rs`, `#[ignore]`) through `dev-tools/e2e-memory-cycle.ps1` | 30m |
| `coverage-budget` | NEW (owner decision (c) 2026-09-27) — `dev-tools/coverage-budget.ps1`: coverage report + per-directory budget (see §Coverage) | 60m |
| `release-dry-run` | NEW — `cargo publish -p vantadb --dry-run` (packaging + verify, **never uploads** — release-plz remains the only publisher) | 45m |

- **Why a separate file, not a daily `certification.yml`** (decision
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
  `certification.yml` counterparts - a test-binary list change edits both
  files together (same coupling rule as `.config/nextest.toml` structural
  exclusions).
- **Composed memory cycle (MEM-73-NIGHTLY, 2026-11-25):** job
  `memory-cycle-e2e` runs the `#[ignore]`d L0→L3 e2e
  (`vantadb-mcp/tests/l0_l3_cycle_e2e.rs`) — one real `vantadb-server --mcp`
  process, JSON-RPC over stdio, a scheduler tick, then L0→L1→L2→L3 and
  `memory_recall` without any `memory_put`. It closes the recommendation from
  the MEM-73 review: an e2e that only runs when somebody remembers it rots in
  silence.
  - **Not in push/PR** (class `FIND-727`/`FIND-739`): it spawns a child process
    and a scripted LLM endpoint on an ephemeral port, and this file's
    `pull_request:` trigger matches `src/**` — hence
    `if: github.event_name != 'pull_request'`.
  - **Single entry point:** the YAML calls
    `pwsh -NoProfile -File dev-tools/e2e-memory-cycle.ps1` (build → FIND-741 SHA
    gate → `nextest --run-ignored`). No command is inlined, so build/gate/test
    cannot drift between the manual lane and CI.
  - **No `continue-on-error`** (Regla 2): a red run here means the composed
    cycle is broken; the test gets fixed, the job is not muted.
  - **Linux only:** the e2e is OS-agnostic (spawns the binary, `git`, a
    loopback port); the Windows lane is the developer's local run of the same
    script, so a second runner adds spawn/socket flake surface without new
    signal.
  - **The model is scripted by design** (no ollama, no key in CI): the runner
    path (`VANTADB_INGEST_*` → `ConcreteRunner::Ollama` → HTTP → parse) is real;
    what is proven is the composition, not model quality.
  - **Budget:** 30 min. The e2e is measured (1.45 s local; MEM-73 recorded
    3.5-4.2 s wall). The build is not: 525 s is a local **warm** run of the exact
    job command (2026-11-25, Windows/MSVC) — a floor, not the CI figure. A cold
    checkout adds a dependency compile, so the CI number stays an estimate until
    the first run records it. The build half is larger than one binary because
    nextest builds all 27 `vantadb-mcp` test targets before filtering. 30 min is
    ~3.4x that floor and matches this file's ceiling for compile-heavy jobs
    (`failpoint-tests`: 30 min, with fewer target binaries than this one).
    Tighten after the first scheduled run.
  - **Who looks when it fails:** it is in `notify-failure`'s `needs`, so a red
    run opens (or comments on) the `[Nightly] Heavy certification subset
    failed` auto-issue listing this job.

### 2c. Excluded-binary coverage ledger (FIND-752, 2026-11-26)

The `default-filter` of `.config/nextest.toml` excludes ~55 heavy binaries by name. For six of
them the exclusion was *documented but never measured*: nobody had counted how many tests they
hold, or whether a healthy lane runs them at all. Counted on 2026-11-26 against `gh`, the answer
is **154 tests in the six binaries, of which 118 (77 %) have no lane that executes them**.

| Binary | Tests | Lane today | Last real result | Verdict |
|---|---:|---|---|---|
| `benchmark_internal` | 1 | `certification.yml` → job `other_heavy` (step "Run remaining heavy tests") | run `45598671538` (2026-11-25): job **failure at the TBH-01 dataset gate, before the test step** → 0 success / 36 failure in 30 d | **does not run** |
| `benchmark_datasets` | 1 | same job | same | **does not run** — and when run locally without GloVe-100 it returns early in green (`tests/benchmark_datasets.rs:18`) |
| `cli_tests` | 96 | same job | same | **does not run** — CI and the local nextest gate both exclude the binary |
| `concurrency_parity` | 5 | `certification.yml` → job `memory_concurrency` | run `45598671538`: **success** → 36 success / 0 failure / 3 skipped in 30 d | **runs** 5 of 5 — `test_triple_backend_parity_validation` is `#[cfg_attr(not(feature = "rocksdb"), ignore)]` and this lane passes `--features rocksdb`, so the ignore is not applied |
| `snapshot_certification` | 56 with `--features failpoints` | `snapshot-certification.yml` phases 03/04/05 | run `45802050950` (2026-11-26): **success** → 39 success / 6 failure / 3 cancelled in 30 d | **runs partially** — the phase selector `test(snapshot) or test(pending_restore) or test(schedule)` matches **42 of 56**; 14 are never selected |
| `security_audit` | 6 | **none** | never ran; `git log -S"security_audit" -- .github/workflows` → 0 commits | **does not run** — `verify.ps1:117` and `coverage-budget.ps1:79` both `--skip security_audit` (the `cargo test` fallback; the primary nextest path excludes it by `default-filter`) |

**How these numbers were obtained** (repeatable; the full evidence lives in
[`docs/dev/tasks/FIND-752.md`](../tasks/FIND-752.md)):

```bash
# inventory — the default filter is what hides them, hence --ignore-default-filter
cargo nextest list -p vantadb --ignore-default-filter --run-ignored all \
  -E 'binary(benchmark_internal) or binary(benchmark_datasets) or binary(cli_tests) \
      or binary(concurrency_parity) or binary(snapshot_certification) or binary(security_audit)'
# 154 tests (1 + 1 + 96 + 5 + 45 + 6)

# what the snapshot lane actually selects, with the features that lane builds
cargo nextest list -p vantadb --ignore-default-filter --run-ignored all \
  --features failpoints --test snapshot_certification \
  -E 'test(snapshot) or test(pending_restore) or test(schedule)'
# 42 of 56

# per-job conclusions for every scheduled run in the window
gh run list --workflow certification.yml --created ">=2026-10-27" --limit 200 \
  --json databaseId,event,conclusion
gh api repos/{owner}/{repo}/actions/runs/<id>/jobs --jq '.jobs[] | "\(.conclusion) \(.name)"'
```

**The three states this ledger separates.** From outside, "excluded from the fast filter" looks
the same whether the test (1) runs in another lane and passes, (2) runs and fails, or (3) does not
run anywhere. Only (3) is the hole. `other_heavy` is the reason the distinction matters: it has
not produced a single green run in 30 days, and it fails **before** the test step, so its three
binaries are not "failing tests" — they are tests nothing executes. The immediate cause is
`FIND-328`'s fix (download before the dataset gate) sitting in `develop` 552 commits ahead of
`main`, which is the tree the schedule actually runs.

**Rule for the next exclusion:** a binary added to `default-filter` must appear in this table with
the lane that runs it and the command that counts it. An exclusion without a named green lane is
the fast gate quietly shrinking.

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

### 3. Docs Gate (`docs-gates.yml`)

Gate de documentación (no existe `gate-docs.yml` ni `gate-docs-21.yml`): `markdownlint-cli2`
sobre `docs/**`, `check-links`, gate de frontmatter/orphans (`scripts/docs/check-docs.mjs`), el
gate de inventario de workflows (compara el número declarado aquí contra `.github/workflows/`) y
el guard de documentación de release. Corren en push/PR a `main`+`develop` que toquen docs,
scripts o workflows. **En `pull_request` ya NO hay filtro de `paths`** (`FIND-765`): un workflow
que no dispara no reporta nada, y sus tres jobs son *required* en `develop` — un PR que sólo
tocara ficheros fuera del filtro quedaba con esos contexts en `Expected` para siempre. El
filtro de `push` se conserva (21 patrones).

**`ci-rust.yml` quedó igual** (`FIND-770`): `paths` fuera de `on.pull_request`, `on.push` con sus
24 intactos, y el filtro **dentro** con un job `rust-scope` *fail-open* que alimenta la guarda de
`clippy`/`semver-checks`/`public-api-snapshot`; los **9** jobs con `needs: clippy` se omiten por
dependencia. Así los **14** contexts de `develop` se reportan siempre. Coste medido del arreglo:
**~570 s (6,3 %)** frente a los **9062 s / 69 min** que costaría quitar el filtro sin más. El precio,
declarado: en un PR sólo-docs, **6 de los 10** contexts de Rust quedan `skipped` **sin medir**.

### 4. Platform ops (`platform-ops.yml`)

Análisis CodeQL para Rust (push/PR a `main` + semanal), dependency review y los gates de
seguridad de plataforma (SBOM, OSV). Sustituye a los antiguos `sec-codeql.yml`/`sec-codeql-30.yml`.

### 5. Fuzzing (dentro de `certification.yml`)

Los 4 targets de `cargo fuzz` (`fuzz_parser`, `fuzz_wal`, `fuzz_node_deserialize`, `fuzz_archive`)
se ejecutan como jobs de `certification.yml` (semanal/dispatch). **No hay `fuzz.yml`**; el harness
vive en `fuzz/` y su lock sigue gitignored (FIND-404).

### 6. Benchmarks (dentro de `nightly.yml` y `certification.yml`)

**No hay `heavy-bench-nightly.yml`.** Los benches ligeros corren en `certification.yml`
(`light-benchmarks`) y la serie pesada + `analyze`/`bench_regression` en `nightly.yml`
(job `bench-informational`). El gate `perf-bench.yml` se retiró el 2026-10-29 (ver
`docs/user/operations/BENCHMARKS.md` §20).

#### FIND-317 — `bench-canonical-p99-informational.yml` retirado; presupuesto del sucesor

**Decisión: el archivo standalone queda retirado y no se resucita. La capacidad vive en
`nightly.yml` como dos jobs —`toolchain-check` en la lane de PR y `bench-informational`
en la lane programada— con presupuesto medido y `CATEGORY: INFORMATIONAL`.**

Por qué el archivo no puede volver: la fila original decía "30+ corridas canceladas, check
rojo permanente" y el comando de triage que traía **no podía encontrar nada**, porque el
workflow ya no existe. `gh run list --workflow bench-canonical-p99-informational.yml`
devuelve `HTTP 404: workflow … not found on the default branch`; el archivo se borró en
`2ba62d46f` (2026-10-15) al absorberlo CI-08/SRV-10 en `nightly.yml` (`nightly.yml:29-33`).
Es la misma clase de fallo que `FIND-328`: una fila que nombra un archivo inexistente deja
el rojo invisible. Un nombre de workflow que apunta al vacío no puede ser un check.

Evidencia de que el sucesor **sí** completa y **sí** publica (corridas 44228761011 y
44228771621, 2026-11-18, leídas con `gh run view <id> --json jobs`):

| Job | Duración medida | Conclusión |
|---|---|---|
| `Toolchain compiles bench (release, no run)` | 9m42s / 9m06s | success |
| `Run canonical_p99 vs baseline (informational)` | **159m39s / 145m02s** | success |

La hipótesis "hang vs lentitud extrema" queda resuelta del lado de **lentitud**, con el
número del propio runner: `insert_100k_1536d` tarda `698.16 s` por iteración y Criterion
necesita 10 muestras ⇒ `7030 s` estructurales (117 min), más 86 s de compilación y 40 s de
search. El step no se cuelga: recolecta sus muestras y termina.

**Presupuesto declarado** (Regla 9: medido, no estimado):

| Gate | Timeout | Máximo medido | Margen |
|---|---|---|---|
| `bench-informational` | 240 min | 159m39s | 1.5× |
| `toolchain-check` | 30 min | 9m42s | 3.0× |

Reglas que se derivan:

- **Must:** `bench-informational` corre en `schedule`/`workflow_dispatch`, nunca en la lane
  de PR (`if: github.event_name != 'pull_request'`). Un gate de 2h30 no es una señal de PR,
  y en PR publicaba `stable_count: 0` — sin baseline comparable, la maquinaria de regresión
  no podía disparar nunca.
- **Must:** el job lleva `concurrency` propio con `cancel-in-progress: false`
  (misma forma que `failpoint-tests`), para que un merge a `main` no mate un bench de 2h30
  a mitad de vuelo. **Must not:** reutilizar el grupo de concurrencia del workflow — esa
  colisión es la causa raíz de `FIND-716`.
- **Must:** `toolchain-check` (compile-gate `--no-run`, 9 min) se queda en la lane de PR:
  es el `consumo guard` de `BENCHMARKS.md` §11 y es barato.
- **Must not:** subir el `timeout-minutes: 240` sin una medición nueva que lo justifique.
  El margen actual es 1.5×; se subió 90 → 240 en 2026-10-07 (`93b201a76`) y esa fue la
  última vez que se tocó sin medición.
- **Must not:** recrear `bench-canonical-p99-informational.yml`. Si hace falta un check
  nuevo de bench, se añade como job de `nightly.yml` o de `certification.yml`, que es donde
  el inventario y esta política lo esperan.
- **Categoría:** `INFORMATIONAL` (Regla 2 de `AGENTS.md`). Todos los `continue-on-error`
  del job llevan su `# CATEGORY: INFORMATIONAL`; esta decisión **no añade ninguna exención
  nueva**.

**Dos deudas vivas que esta decisión no paga** (fuera del alcance de `FIND-317`):

1. `docs/user/operations/BENCHMARKS.md:454` todavía cita el archivo retirado como el lugar
   donde se evalúa el `>10%`; el destino real es el job `bench-informational`.
2. `main` sigue sirviendo el bench `1536d` (`const DIM: usize = 1536;`) mientras `develop`
   ya tiene GloVe-100d (`2b821184a`). Mientras no se sincronicen, la serie programada mide
   el corpus que `BENCHMARKS.md` §11 declara superseded y el comparador seguirá reportando
   `stable_count: 0` (sin baseline ⇒ nunca hay regresión que reportar). Owner: `vanta-tuner`.


### 7. Release verification (`release-verify.yml`)

Verificación post-release de assets y registries; renderiza su lista esperada desde
`release-manifest.json` (nunca hardcodea).

### 8. Demo acceptance (`demo-acceptance.yml`)

Smoke E2E del flujo demo/showcase (dispatch y paths de `examples/`, `vantadb-server/`).

### 9. MCP registry (`publish-mcp-registry.yml`)

Publica `server.json` en el registro MCP (dispatch). Requiere `MCP_REGISTRY_TOKEN`.

### 10. CI Fleet (`ci-fleet.yml`) y `ci-gate.yml`

`ci-fleet.yml` corre la matriz de jobs complementarios; `ci-gate.yml` es el agregador que exige
todos los checks requeridos (su lista `REQUIRED` es la fuente de verdad de los contexts del
ruleset de `main`).

### 11. Python Wheel Build & Publish (`release-wheels.yml`)

Builds the Python SDK on Linux (x86_64 and aarch64 cross via maturin-action's manylinux_2_28
cross container), macOS, and Windows with `maturin`, installs the generated wheel
by resolved path, and runs the Python SDK smoke suite (native targets only — the aarch64
cross wheel is validated by its platform tag and the PyPI install verify job). Manual TestPyPI upload is available only
through an explicit workflow input and the `TEST_PYPI_API_TOKEN` secret. Production PyPI
publication and signing remain deferred.

### 12. Weekly Adoption Metrics Collector (`metrics-collector.yml`)

Runs on a weekly schedule (`0 6 * * 1`) and via `workflow_dispatch` to collect ecosystem adoption telemetry (stars, forks, PyPI/npm download stats) via `scripts/collect_adoption_metrics.py` and commit the snapshot to `docs/dev/metrics/adoption.json` with `[skip ci]`.

### 13. Main Ancestry Gate (`main-ancestry.yml`)

Dedicated unfiltered workflow (`on.pull_request.branches: [main]`, no `paths:`/`paths-ignore:`) that checks out `github.event.pull_request.head.sha` with `fetch-depth: 0` and verifies `git merge-base --is-ancestor origin/main HEAD` on every PR targeting `main` (exempting `release-plz-*` branches), preventing `develop → main` merges from reverting hotfixes that only exist on `main` (FIND-455, FIND-633).

### 9. Release Workflows

| Workflow | File | Trigger |
|----------|------|---------|
| Release Automation | `release.yml` | Push to `main` — `release-plz` auto-version, changelog, tag, publish |
| Python Wheels | `release-wheels.yml` | `workflow_dispatch` via fanout de `release.yml` sobre tag `v*.*.*` (FIND-467, `push.tags` eliminado por doble disparo), `pull_request` to `main` (paths src/python), o `workflow_dispatch` manual |
| NPM Publish (WASM+TS) | `release-npm-wasm.yml` | Tag `v*.*.*`, `pull_request` (paths wasm/ts), or `workflow_dispatch` — includes Fast Gate job `tests` (`npm ci && npm run build && npx vitest run`, measured 27s <5min, no `continue-on-error`, PR+tags gate per TS-06) |
| NPM Publish (Node) | `release-npm-node.yml` | Tag `node-v*.*.*`, `pull_request` (paths node), or `workflow_dispatch` |
| PyPI Adapters | `release-adapters.yml` | Tag `adapters-v*.*.*` or `workflow_dispatch` (TestPyPI) |
| Binary Builds | `release-binaries.yml` | Release published or `workflow_dispatch` |
| SBOM Generation | step en `release-binaries.yml` (ex `release-sbom.yml`, fusionado pre-pub) | Release con upload |

## External Dependencies (Ollama/LLMs)

VantaDB integrates with external LLMs for embeddings and semantic queries. However, **integration
tests requiring network access to LLMs (like Ollama) are strictly excluded from the Fast Gate.**
They are either marked with `#[ignore]` or gated behind environment variables (e.g.,
`VANTADB_RUN_LLM_TESTS=1`). This ensures the core engine can be built and tested completely offline.

## Running Heavy Certification Manually

The `certification.yml` workflow runs automatically via a CRON schedule (weekly on
Sundays at 03:00 UTC). The scheduled lane runs the local deterministic core certification jobs.
SIFT-1M validation and competitive benchmarks are manual opt-ins because they require external
datasets. You can also trigger it manually from the GitHub Actions UI:

1. Navigate to the **Actions** tab in the repository.
2. Select **HEAVY: Certification — All Tests** from the left sidebar.
3. Click **Run workflow**.
4. You can optionally check the boxes to include `SIFT-1M validation` or `Competitive benchmarks`.

## Docker Image Publishing — retired (2026-10-02)

Docker support was removed repo-wide by owner decision: the `docker-image` job was deleted from `release-binaries.yml`, the Dockerfiles/compose files were removed, and deployment docs no longer cover containers.


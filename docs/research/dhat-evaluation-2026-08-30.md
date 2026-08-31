# Heap-Usage Testing — `dhat` 0.3.3 Evaluation (2026-08-30)

> **Document type:** Research note / mini-ADR (not a full ADR — no formal template needed because
> the decision is "no action").
> **Status:** Accepted (decision: do NOT introduce `dhat`).
> **Author:** vanta-worker (TBH-18, Wave 3 of testing-bench-harden plan).
> **Date:** 2026-08-30.

---

## TL;DR

**`dhat` 0.3.3 is NOT introduced into the VantaDB workspace.** The workspace has no
documented allocation regressions (no `P2-*` alloc debt, no `alloc_regression` /
`heap_regression` / `memory_leak` markers in code or docs), and `dhat` itself is
explicitly marked **experimental** by its author with the note that "maintenance is
not a high priority". `dhat` will be reconsidered only when concrete alloc-regression
evidence emerges.

---

## 1. Context

The 2026-08-30 multi-agent testing audit (Wave 2 of `2026-08-30-testing-bench-harden`)
included a soft recommendation to evaluate `nnethercote/dhat-rs` 0.3.3 as a heap-usage
testing tool — feature-gated dependency that wraps the global allocator and lets tests
assert on alloc counts/bytes (e.g. *"this function must do ≤10 allocations"*).
Detecting alloc regressions is something the existing functional memory suite cannot do
(`tests/memory_telemetry.rs` reads `jemalloc-ctl` gauges and `MemoryBreakdown` fields,
not per-operation alloc counts).

The owner decision (D5, plan doc): *"justificar cada dep antes de añadir"* — meaning no
new dependency enters the workspace without clear, demonstrated need.

This evaluation answers: **does VantaDB currently need `dhat`?**

---

## 2. What the workspace uses today

Confirmed by `git grep "dhat" Cargo.toml` → **0 hits** and `git grep -liE "dhat" -- '*.rs' '*.toml'` → **0 hits**.
The only occurrences of "dhat" in the tree are:

- `.opencode/agents/vanta-tuner.md:41` — agent roster mention ("`dhat-rs`" listed alongside `allocative`, `heapsize`).
- `docs/benchmarks/docs/BENCHMARK_OPTIMIZATION_2026.md:567,585-586,593,894,920` — future-option reference in the optimization playbook (NOT a current dependency).

No test currently uses `#[global_allocator] = dhat::Alloc`. The closest related capability
is `benches/memory_budget.rs` (criterion-based memory-budget benchmark — measures whole-program
RSS via sysinfo, not per-op alloc counts).

### Production allocators (`Cargo.toml` features + bins)

| Feature | Allocator | Gated to | Purpose | Source |
|---|---|---|---|---|
| `custom-allocator` | `mimalloc` | Windows | Default for distributed `vanta-cli`/`vantadb-server` binaries | `src/bin/vanta-cli.rs:12-21`, `vantadb-server/src/main.rs:6-18` |
| `jemalloc` | `tikv-jemallocator` + `tikv-jemalloc-ctl` | Linux / macOS | Default for distributed binaries | Same as above |

Rule source: `.opencode/rules/release-ci.md` regla 1 + INV-004.

### Existing memory test surface

| File | What it tests |
|---|---|
| `tests/memory_telemetry.rs` | `MemoryBreakdown` shape + correctness of per-component byte accounting |
| `tests/memory_api.rs` | `operational_metrics()` exposes jemalloc/RSS/HNSW/mmap fields with sane magnitudes |
| `tests/memory_export_import.rs` | Memory state survives export/import round-trip |
| `tests/memory_brutality.rs` | Adversarial memory inputs (large batches, concurrent inserts) — non-regression guard |
| `benches/memory_budget.rs` | Criterion bench for memory-budget regression detection |

These cover *observed memory state*, not *per-call-site alloc counts*. `dhat` would fill
that gap — but only if we have a specific alloc-budget invariant to enforce.

---

## 3. Is there an actual alloc-regression problem?

`git grep -liE "alloc_regression|heap_regression|memory_leak" -- '*.rs' '*.md'` → **0 hits**.

Cross-checked the P2 debt register in `.opencode/AGENTS.md` (Regla 6, tabla resumida):

| ID | Topic | Status |
|---|---|---|
| P2-1 through P2-3 | PyO3 / LRU / match exhaustiveness | ✅ RESUELTO |
| P2-5 | `put_batch` dual API | 🟢 ABI debt, no alloc issue |
| P2-7 | Serialization zero-copy | ✅ RESUELTO |
| P2-8 | `collect_all_deduped()` O(n) | 🟡 memory pressure in WASM, not alloc-regression in core |

No outstanding `P2-*` alloc-regression item. No `FIND-*` finding tagged `heap` or
`alloc` in the audit deliverable. No issue tracker entry.

**The problem `dhat` solves does not currently exist in our workspace.**

---

## 4. What `dhat` would add (and what it would cost)

### Pro

- **Per-call-site alloc assertions** — `dhat::assert_eq!(stats.total_blocks, 3)` style. Catches
  regressions that whole-program RSS gauges miss (e.g. "this refactor now allocates 50 vectors
  per call instead of 5").
- **Zero runtime cost in production** — feature-gated, only active with `--features dhat-heap`.
- **Heap profiling on any platform** — unlike valgrind/DHAT (Linux-only), works on Windows + macOS.
- **Maintained** by the same author as `cargo` (`@nnethercote`); 999★ / 131 commits, last release
  29 Aug 2026.

### Con (the cost that justifies saying no, for now)

1. **Author-marked experimental.** Verbatim from `docs.rs/dhat/0.3.3` crate-level docs (consulted
   2026-08-30):

   > *"This crate is experimental. It relies on implementation techniques that are hard to keep
   > working for 100% of configurations. It may work fine for you, or it may crash, hang, or
   > otherwise do the wrong thing. Its maintenance is not a high priority of the author."*

   For a dep that would touch `#[global_allocator]` in the distributed binaries, this is a load-
   bearing piece of code we cannot afford to rely on a "not a high priority" maintenance signal.

2. **Conflicts with production allocator.** Rust allows only one `#[global_allocator]` per binary.
   VantaDB's release binaries use `mimalloc` (Windows) or `jemalloc` (Linux/macOS) — see
   `release-ci.md` regla 1. `dhat`'s `Alloc` would *replace* that, meaning heap-usage tests would
   run under a different allocator than the one users actually run. Numbers would not transfer.
   Workaround: write parallel assertions per allocator (one `Alloc` per `[features.dhat-heap-*
   allocator*]`). Doubles the test surface for marginal value.

3. **CI integration friction.** `dhat` requires (per its own docs):
   - **Release profile only** (debug builds allocate more and lose determinism).
   - **Per-test isolated processes** — `dhat` panics if more than one `Profiler` runs concurrently,
     so heap-usage tests must each live in their own `tests/*.rs` integration test file, OR
     `cargo test -- --test-threads=1` (defeats parallel test execution, breaks Fast Gate budget).

4. **Slow on Windows.** Author note: *"Even more so on Windows, because backtrace gathering can
   be drastically slower on Windows than on other platforms."* VantaDB CI runs on Windows runners;
   this would balloon test time.

5. **YAGNI.** No current evidence the workspace needs per-call-site alloc assertions.
   `tests/memory_brutality.rs` already stress-tests memory; `benches/memory_budget.rs` catches
   whole-program drift; `vanta-tuner` agent has `allocative`, `heapsize`, and direct
   `jemalloc-ctl` access for ad-hoc profiling. Adding `dhat` is the fourth tool for a problem
   we haven't seen.

---

## 5. Trigger conditions for re-evaluation

`dhat` (or an equivalent per-call-site allocator profiler — `allocative`, `heaptrack` on Linux,
valgrind/DHAT where applicable) becomes worth adding when **any** of the following appears:

- [ ] A `FIND-*` or `P2-*` finding tagged `alloc-regression` or `heap-budget` enters the backlog.
- [ ] `tests/memory_brutality.rs` or `benches/memory_budget.rs` starts flaking or drifting without
      CPU-side explanation (suggests an alloc-side cause the existing gauges can't localize).
- [ ] A specific refactor ships and post-hoc RSS growth is observed that the per-component
      `MemoryBreakdown` cannot attribute to a component.
- [ ] A production binary exhibits memory growth under a workload that `vanta-tuner`'s existing
      tools (`allocative`, `heapsize`, `jemalloc-ctl` epoch-based stats) cannot localize to a
      call site.
- [ ] Someone needs to enforce an invariant like *"HNSW bulk insert must allocate ≤N blocks per
      1k vectors"* (i.e. an alloc-budget contract becomes a public API guarantee).

When one of these fires, file a new TBH-XX task referencing this document and re-evaluate.
At that point the author-experimental warning should also be re-checked against the current
release — if `dhat` has moved out of "experimental" status, the calculus changes.

---

## 6. Decision

**No action.** Do not add `dhat` 0.3.3 (or any per-call-site allocator profiler) to `Cargo.toml`.

This decision follows:
- D5 principle (justificar antes de añadir).
- `ponytail` reflex: the smallest change that works is no change — we have no observed
  alloc-regression problem, so we add no tool.
- Regla 9 of `.opencode/AGENTS.md` (no optimices sin medir): we have no measurement need,
  so we add no measurement tool.

If trigger condition 5.1 fires, reopen with a new TBH-XX referencing this doc.

---

## 7. References

- Plan: `docs/plans/2026-08-30-testing-bench-harden.md` (TBH-18)
- Task file: `.opencode/skills/campaign-executor/tasks/TBH-18.md`
- Allocation rules: `.opencode/rules/release-ci.md` regla 1 (custom allocator per platform) +
  `.opencode/rules/memory-budget.md` (memory governance)
- Memory test surface: `tests/memory_telemetry.rs`, `tests/memory_api.rs`,
  `tests/memory_export_import.rs`, `tests/memory_brutality.rs`
- Memory benchmark: `benches/memory_budget.rs`
- Allocator wiring: `src/bin/vanta-cli.rs:12-21`, `vantadb-server/src/main.rs:6-18`
- INV-004 (decision: release binaries use mimalloc/jemalloc)
- 2026-08-30 multi-agent audit (gap analysis) — internal, Wave 2 deliverable
- `dhat` upstream: <https://github.com/nnethercote/dhat-rs>
- `dhat` 0.3.3 docs: <https://docs.rs/dhat/0.3.3/dhat/> (consulted 2026-08-30; crate remains
  explicitly experimental at this version)
- Companion evaluation doc for `loom`: `docs/research/concurrency-testing-2026-08-30.md`

---

*Review on next plan that touches memory accounting, or when trigger condition 5.1 fires,
whichever comes first.*
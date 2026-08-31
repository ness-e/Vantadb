# Benchmark Framework Evaluation — `divan` vs `criterion` (2026-08-30)

> **Document type:** Research note / mini-ADR (not a full ADR — no formal template needed because
> the decision is "no action").
> **Status:** Accepted (decision: do NOT introduce `divan`).
> **Author:** vanta-lead (TBH-16, Phase 3 BAJA of testing-bench-harden plan).
> **Date:** 2026-08-30.

---

## TL;DR

**`divan` 0.1.21 is NOT introduced into the VantaDB workspace.** `criterion` 0.8 already covers
the complete benchmark surface (22 benches, html_reports, async_tokio, nightly CI with regression
detection, baseline JSON). `divan` offers marginal ergonomic gains (cleaner `#[bench]` attribute
syntax, integrated `AllocProfiler`) at the cost of rewriting every existing bench plus the entire
nightly pipeline (`heavy-bench-nightly-51.yml`, `scripts/bench_regression.py`,
`benchmarks/criterion_baseline.json`). `divan` will be reconsidered only if criterion is abandoned
upstream or if VantaDB specifically needs `AllocProfiler` data on every bench run.

---

## 1. Context

The 2026-08-30 multi-agent testing audit (Phase 3 BAJA of `2026-08-30-testing-bench-harden`)
included a soft recommendation to evaluate `nikolausmayer/divan` (current 0.1.21) as a
modern Rust benchmarking framework, conditional on DX improvements over criterion. `divan`
is celebrated for:

- Attribute-based bench syntax (`#[divan::bench]`) — less boilerplate than `criterion_group!`/`criterion_main!`
- First-class `AllocProfiler` — per-bench allocation counts without external tooling
- Better output formatting and faster compile-times for small benches
- Strong defaults that work out of the box (no manual `criterion_group!` registration)

The owner decisions relevant here:

- **D1 (conservative dataset/benchmark strategy):** explicit in plan — "no `divan` alongside criterion".
- **D5 (justificar cada dep antes de añadir):** no new dependency enters the workspace without
  clear, demonstrated need.

This task formalizes the rationale so future audits don't re-litigate the same decision.

## 2. What the workspace uses today

`Cargo.toml` line 179:

```toml
criterion = { version = "0.8", features = ["html_reports", "async_tokio"] }
```

`git grep "divan" Cargo.toml` → **0 hits** (verified at task time).

22 bench files under `benches/` (excluding the `common/` helper module), all using the criterion
`criterion_group!` / `criterion_main!` idiom. Representative sample:

| File | What it benches |
|---|---|
| `benches/hnsw_pure.rs` | Pure HNSW build/search (the 1 micro-bench candidate the audit flagged) |
| `benches/hnsw_recall_ef.rs` | HNSW recall vs `ef_search` parameter sweep |
| `benches/hybrid_queries.rs` | BM25 + HNSW hybrid retrieval |
| `benches/tokenizer_bench.rs` | Tokenizer throughput (179 lines — natural candidate to port) |
| `benches/bench_concurrent.rs` | Concurrent insert/search under fixed wall-clock (custom main, no estimates.json) |
| `benches/wal_throughput.rs` | WAL throughput (Throughput::Elements) |
| `benches/crash_recovery.rs` | Crash recovery path latency |
| `benches/canonical_p99.rs` | p99 latency tracking |
| `benches/memory_budget.rs` | Memory budget assertion |
| (14 more) | Coverage of all hot paths |

The full nightly pipeline (`.github/workflows/heavy-bench-nightly-51.yml`) is wired around
criterion's `target/criterion/` output:

- `scripts/bench_regression.py --criterion-root target/criterion` (extracts mean_ms/median_ms/std_ms)
- `benchmarks/criterion_baseline.json` (regression baseline, auto-updated by nightly)
- 5 nightly bench groups: `hnsw_pure`, `hybrid_queries`, `stress_test`, `high_density`,
  `bench_concurrent` (the last intentionally absent from baseline — uses `harness=false` custom main).

This pipeline is **criterion-specific** — porting any bench to divan would mean either
duplicating the bench or rewriting the regression detection scripts to parse divan's output.

## 3. What `divan` would add (and what it would cost)

### Pro

1. **Cleaner attribute syntax.** `#[divan::bench]` replaces `criterion_group!` / `criterion_main!`
   boilerplate. ~5 fewer lines per bench file.
2. **`AllocProfiler` built in.** Per-bench `Vec`/`String` allocations counted automatically,
   without `dhat`/`heaptrack`. Useful for hot-path memory accounting.
3. **Faster small-bench compile time.** For benches that only have 1-2 functions, divan compiles
   ~30% faster than criterion (from divan upstream docs).
4. **Active development.** divan 0.1.x has frequent releases; criterion 0.8 has been stable
   since 2025 but is in maintenance mode (no major features planned).

### Con (the cost that justifies saying no, for now)

1. **Rewrite all 22 benches.** Every bench file currently uses `criterion::{criterion_group,
   criterion_main, Criterion, BenchmarkId, BatchSize, Throughput, …}`. Porting requires:
   - Replace `criterion_group!`/`criterion_main!` with `#[divan::bench_group]` / `#[divan::main]`
   - Convert `Criterion::bench_function("name", |b| b.iter(|| …))` to `#[divan::bench] fn …`
   - Convert `BatchSize::SmallInput / PerIteration` to divan's `bench(threads = N)` counters
   - Convert `Throughput::Elements(N)` to divan's `counter!` / `BytesCount` macros
   - Manually update imports across all files

   Estimate: ~3-5 days of mechanical port + risk of subtle measurement regressions (criterion
   samples `iterations × repeats` differently than divan — direct port can change numbers).

2. **Rewrite `heavy-bench-nightly-51.yml`.** The pipeline reads `target/criterion/<bench>/new/estimates.json`.
   divan writes to a different layout (`target/divan/…`). The merge step (lines 137-149),
   the regression extraction step (lines 149-258), and `bench_regression.py` itself would all
   need to be adapted.

3. **Rewrite `benchmarks/criterion_baseline.json`.** All 5 baseline entries (mean_ms/median_ms/
   std_ms from `hnsw_pure`, `hybrid_queries`, `stress_test`, `high_density`, plus the manually-
   tracked `bench_concurrent` exclusion) would be invalidated. First nightly after port would
   re-seed the baseline from fresh divan numbers — regressing CI confidence for ~1 week while
   the baseline stabilizes.

4. **Two frameworks to maintain.** Keeping criterion for some benches + divan for others
   doubles the surface area: two output directories, two sets of HTML reports, two dependency
   graphs, two mental models. Plan D1 explicitly forbids this: *"no `divan` alongside criterion"*.

5. **`AllocProfiler` is replaceable.** The actual gap (per-bench allocation counts) is served
   by `cargo bench --features dhat-heap` if needed (TBH-18 in the same plan evaluates this
   as a separate, opt-in feature gate). No need to switch the whole framework to get allocation
   data.

6. **YAGNI.** The audit found no concrete benchmark need that criterion fails to serve. The
   proposed benefit — DX improvement — is cosmetic; all criterion boilerplate is already
   wrapped in `benches/common/mod.rs` helpers where applicable.

## 4. Why `criterion` 0.8 is already sufficient

The criterion setup covers everything VantaDB actually needs:

| Requirement | criterion 0.8 coverage |
|---|---|
| Stable statistical estimates (mean / median / std / slope) | ✅ Native (`Bencher::iter`) |
| Throughput metrics (ops/sec) | ✅ `Throughput::Elements` / `Bytes` |
| Custom multi-thread / fixed-wall-clock scenarios | ✅ `Bencher::iter_custom(\|iters\| Duration)` (TBH-10 pattern) |
| Async benches (`async fn` + tokio runtime) | ✅ `async_tokio` feature enabled (line 179) |
| HTML reports | ✅ `html_reports` feature enabled (line 179) |
| Baseline regression detection in CI | ✅ `bench_regression.py` + nightly workflow |
| Compile-time stability | ✅ 0.8 is mature, no churn |
| Allocation profiling (when needed) | ⚠️ External: `dhat` feature gate (TBH-18) |

The only criterion gap — no built-in `AllocProfiler` — is filled by TBH-18 in the same plan
without requiring a framework switch.

## 5. Trigger conditions for re-evaluation

`divan` (or any criterion replacement) becomes worth considering when **any** of the following
appears:

- [ ] `criterion` upstream is abandoned or marked unmaintained for >6 months (currently
      actively maintained by bheisler; 0.8 stable since 2025).
- [ ] A new bench requires `AllocProfiler` data on **every** bench (not just opt-in via dhat).
      If this is the case, evaluate porting specific benches to divan — not the whole suite.
- [ ] criterion's compile-time becomes a CI bottleneck (>60s added to Fast Gate). Current
      bench-compile overhead is negligible (TBH-11 measured: nightly bench compile ≤90s for
      8+ benches, well within the 2hr Heavy Certification budget).
- [ ] A breaking change in Rust nightly makes criterion fail to build (we already pin to a
      stable Rust release; not currently a concern).
- [ ] The 22 existing benches are deleted or rewritten for unrelated reasons — port becomes
      free. Currently all 22 are load-bearing.

When one of these fires, file a new TBH-XX task referencing this document and re-evaluate.

## 6. Decision

**No action.** Do not add `divan` to `Cargo.toml`. Do not port any bench to divan.

This decision follows the D1 principle (conservative strategy, no framework duplication) and
the D5 principle (justify before adding), plus the [`ponytail`](../.opencode/skills/ponytail/SKILL.md)
reflex: the smallest change that works is no change.

## 7. References

- Plan: `docs/plans/2026-08-30-testing-bench-harden.md` (TBH-16)
- Task file: `.opencode/skills/campaign-executor/tasks/TBH-16.md`
- Sister decision: `docs/research/concurrency-testing-2026-08-30.md` (TBH-17, loom eval — same
  "no action" outcome for analogous reasons)
- criterion config: `Cargo.toml` line 179
- Bench files evaluated: `benches/*.rs` (22 files using `criterion_group!`/`criterion_main!`)
- Nightly pipeline: `.github/workflows/heavy-bench-nightly-51.yml` (criterion-specific output)
- Regression detector: `scripts/bench_regression.py`
- Baseline: `benchmarks/criterion_baseline.json`
- 2026-08-30 multi-agent audit (gap analysis) — internal, Wave 2 deliverable
- `divan` upstream: <https://github.com/nikolausmayer/divan>
- Criterion TBH-10 pattern (custom wall-clock): `benches/bench_concurrent.rs:1-30`,
  `.opencode/task-system/memory/lessons.md:334`

---

*Review on next plan that introduces a custom benchmark framework need, or in 6 months
(2027-02-28), whichever comes first.*
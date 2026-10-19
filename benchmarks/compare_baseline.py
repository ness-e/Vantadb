#!/usr/bin/env python3
"""Compare benchmark runs against the stored baseline (perf-bench gate).

Extracted from the inline heredoc in `.github/workflows/perf-bench.yml`
(FIND-154) so the comparison is unit-testable and its tolerances are explicit
instead of one global 15% constant.

Tolerance model (FIND-154, 2026-09-27; recalibrated by FIND-232, 2026-10-03
after measuring the instrument's cross-job noise floor):

- **Stable sections** (`insert`, `rebuild`, `query_vector`): warn past
  ``STABLE_WARN_PCT``; block past ``STABLE_BLOCK_PCT``.
- **Quarantined noisy sections** (`query_hybrid`, `query_text`): warn past
  ``NOISY_WARN_PCT``; block only past BOTH the ceiling ``NOISY_BLOCK_PCT``
  AND an absolute increase of ``NOISY_BLOCK_ABS_MS``.
- **Absolute-level metrics** (``ABS_BLOCK_LEVELS_MS``): block on the absolute
  level only; relative moves warn. Used where cross-job spread makes
  relative gating meaningless (``insert.p99_ms``).

Why the stable band moved from 15% to 200% — same method, same profile
(10000/1000), identical code, across GitHub-hosted jobs:

- 2026-09-25 pair (runs 36093538630 vs 36094025761, 8 min apart):
  insert.p99 −28.4%, query_hybrid.p50 −22.8%, throughput +9.4%.
- 2026-10-03 same-SHA pair (runs 37089421873 vs 37089581213, both bf1e7476,
  same profile, ~20 min apart): ingestion 37.8 s vs 47.8 s (throughput
  264.2 vs 209.3 rec/s), query_vector.p50 1.29 vs 2.46 ms, p95 1.53 vs
  3.16 ms, hybrid.p50 3.16 vs 6.37 ms — key metrics swing 1.3x–2.1x.
  Including a slow-runner job (run 37088714140) the range reaches 2.4x on
  query_vector.p95 (1.53 → 3.68 ms) and insert.p99 5.07 → 85.3 ms (16.8x,
  fsync jitter).

A 15% band sits far below that noise floor, so single-run-vs-baseline
comparisons at 15% were structurally flaky. The 13 red runs of
2026-09-25..10-03 were a profile mismatch on top of this: push runs
benchmarked 1000 records / 100 queries while the baseline (and the dispatch
defaults, benchmarks/README.md, BENCHMARKS.md) use 10000 / 1000. With
``queries=100`` the p99 *is* the max sample (``int(n * 0.99) = 99``), so
``query_text.p99`` (baseline 5.7 µs) blocked 13/13 runs at +547..+898% from
34-51 µs of scheduler jitter; the absolute floor now demotes those to
warnings.

The workflow pins every run to the 10000/1000 baseline profile and this
module refuses to compare when ``insert.total_records`` differs from the
baseline's. The bands are calibrated to the instrument — a single GitHub
runner, wall clock, no environment control: this gate detects collapses
(≥3x on stable metrics, ≥4x + 0.5 ms on µs-scale families, ≥100 ms
insert.p99), not fine-grained regressions. Fine-grained gating belongs to
``cargo bench --bench canonical_p99`` (Regla 9) in a controlled environment.
A false positive outside the band is a signal to re-baseline (or widen the
band with new evidence), not to add more tolerance blindly.

Same-job A/B mode (FIND-233, opt-in via ``--ab-runs``): the cross-VM spread
above (1.7-2.4x) makes fine regressions (<2-3x) invisible in the single-run-vs-
stored-baseline gate. When both sides are measured in the SAME job (wheel A =
the ``ab_ref`` ref built in-job, wheel B = HEAD), the machine factor cancels
and paired deltas can gate at ~50%. The workflow alternates A_i, B_i pairs
because every job shows a monotonic warm-up ramp (sub-runs of identical code:
adjacent-pair deltas −1.2%..−22% across runs 37089421873/37089581213/
37088714140); the median of the 3 paired ratios absorbs most of it. Bands are
PROVISIONAL (warn >15%, block >50%) until calibrated with a same-SHA dispatch
(``ab_ref=<HEAD sha>``): the deltas of identical code ARE the noise floor.
``insert.p99_ms`` is fsync-jitter dominated (16.8x cross-VM) — it warns but
never blocks in A/B mode.

Usage:
  python benchmarks/compare_baseline.py                    # compare (CI gate)
  python benchmarks/compare_baseline.py --self-test        # prove the bands
  python benchmarks/compare_baseline.py --update-baseline  # re-baseline
  python benchmarks/compare_baseline.py --ab-runs 1 2 3    # same-job A/B vs ab_results.<i>.json
"""

from __future__ import annotations

import argparse
import datetime
import json
import statistics
import sys

# ── Tolerance bands (single source of truth — keep in sync with the module
# docstring and .github/workflows/perf-bench.yml step comment) ──────────────
STABLE_WARN_PCT = 25.0  # stable families: visibility for sustained moves
STABLE_BLOCK_PCT = 200.0  # stable families: block ≥3x (calibrated, FIND-232)
NOISY_WARN_PCT = 15.0  # noisy families: noise edge — warning only
NOISY_BLOCK_PCT = 300.0  # noisy families: catastrophic ceiling — still blocks
NOISY_BLOCK_ABS_MS = 0.5  # FIND-232: µs-scale metrics also need an absolute floor
# FIND-232: metrics whose cross-job spread (16.8x measured for insert.p99)
# makes relative gating meaningless — they gate on an absolute level instead
# and only warn on relative moves.
ABS_BLOCK_LEVELS_MS = {"insert.p99_ms": 100.0}
QUARANTINED_SECTIONS = ("query_hybrid", "query_text")  # FIND-154 evidence

# ── Same-job A/B bands (FIND-233, --ab-runs mode) — PROVISIONAL ──────────────
# Calibrate with a same-SHA dispatch (ab_ref=<HEAD sha>): identical code on
# both sides makes the measured deltas the real noise floor. Start: warn >15%,
# block >50% (median of paired ratios). insert.p99 is fsync-jitter dominated —
# warn-only, never blocks.
AB_WARN_PCT = 15.0
AB_BLOCK_PCT = 50.0
AB_WARN_ONLY_KEYS = ("insert.p99_ms",)


def relative_delta_pct(key: str, current: float, base: float) -> float | None:
    """How much *worse* ``current`` is vs ``base`` in percent (positive = worse).

    Direction per key: ``throughput_records_per_sec`` is higher-is-better,
    ``*_ms`` are lower-is-better. Anything else has no declared direction and
    returns ``None`` (skipped, same as the original inline gate).
    """
    if key == "throughput_records_per_sec":
        return (base - current) / base * 100.0
    if key.endswith("_ms"):
        return (current - base) / base * 100.0
    return None


def median_report(runs: list[dict]) -> dict:
    """Median of each metric across runs, preserving the report shape."""
    report: dict = {}
    for section in runs[0].keys():
        report[section] = {}
        for key in runs[0][section]:
            values = [r[section][key] for r in runs if section in r and key in r[section]]
            report[section][key] = statistics.median(values)
    return report


def compare_report(median: dict, baseline: dict) -> tuple[list[str], list[str]]:
    """Return ``(blocking, warnings)`` regression messages.

    Blocking entries fail the gate; warnings are noise-band signals from the
    quarantined families that must not fail the run.
    """
    stored = baseline.get("benchmarks", {})
    blocking: list[str] = []
    warnings: list[str] = []

    # FIND-232: runs and baseline must share the same size profile. Anything
    # else is apples-to-oranges (p99 granularity, hybrid leg) and used to
    # block silently on artifacts of the mismatch. Fail-closed: a baseline
    # without total_records cannot prove the profile matches.
    stored_records = stored.get("insert", {}).get("total_records")
    current_records = median.get("insert", {}).get("total_records")
    if not stored_records:
        blocking.append(
            "baseline is missing insert.total_records — cannot verify the bench profile; "
            "re-baseline with the current workflow (FIND-232)"
        )
    elif current_records != stored_records:
        blocking.append(
            f"profile mismatch: insert.total_records {current_records} vs baseline "
            f"{stored_records} — bench runs must use the baseline profile (see perf-bench.yml; FIND-232)"
        )

    for section, metrics in median.items():
        if not isinstance(metrics, dict):
            continue
        quarantined = section in QUARANTINED_SECTIONS
        for key, current in metrics.items():
            base = stored.get(section, {}).get(key)
            if base is None or base <= 0 or not isinstance(current, (int, float)):
                continue
            delta = relative_delta_pct(key, current, base)
            if delta is None or delta <= 0:
                continue
            ref = f"{section}.{key}: {current:.2f} vs baseline {base:.2f} ({delta:+.1f}%)"
            abs_level = ABS_BLOCK_LEVELS_MS.get(f"{section}.{key}")
            if abs_level is not None:
                # FIND-232: relative gating is meaningless for these metrics;
                # only an absolute collapse blocks (relative moves only warn).
                if current > abs_level:
                    blocking.append(f"{ref} > {abs_level} ms absolute catastrophic level")
                elif delta > STABLE_WARN_PCT:
                    warnings.append(
                        f"{ref} — absolute-level metric, warn band > {STABLE_WARN_PCT}%"
                    )
            elif quarantined:
                if delta > NOISY_BLOCK_PCT and (current - base) > NOISY_BLOCK_ABS_MS:
                    blocking.append(
                        f"{ref} > {NOISY_BLOCK_PCT}% catastrophic ceiling "
                        f"(> {NOISY_BLOCK_ABS_MS} ms absolute)"
                    )
                elif delta > NOISY_WARN_PCT:
                    warnings.append(f"{ref} — quarantined family, noise band > {NOISY_WARN_PCT}%")
            elif delta > STABLE_BLOCK_PCT:
                blocking.append(f"{ref} > {STABLE_BLOCK_PCT}%")
            elif delta > STABLE_WARN_PCT:
                warnings.append(f"{ref} — stable family, warn band > {STABLE_WARN_PCT}%")
    return blocking, warnings


def compare_ab(runs_a: list[dict], runs_b: list[dict]) -> tuple[list[str], list[str], list[str]]:
    """Same-job A/B comparison (FIND-233): median of paired B-vs-A deltas.

    Pair ``i`` is ``(runs_a[i], runs_b[i])`` — the workflow alternates A_i, B_i
    so each pair shares its temporal position and the measured warm-up ramp
    cancels out of the ratio. Returns ``(blocking, warnings, report_lines)``;
    report_lines carries every metric delta so a same-SHA calibration dispatch
    can read the real noise floor from the logs.
    """
    blocking: list[str] = []
    warnings: list[str] = []
    report: list[str] = []
    median_a = median_report(runs_a)
    median_b = median_report(runs_b)

    if len(runs_a) != len(runs_b):
        blocking.append(
            f"A/B run count mismatch: {len(runs_b)} B runs vs {len(runs_a)} A runs — "
            "the comparison needs paired runs (FIND-233)"
        )
        return blocking, warnings, report

    a_records = median_a.get("insert", {}).get("total_records")
    b_records = median_b.get("insert", {}).get("total_records")
    if not a_records or not b_records:
        blocking.append(
            "A/B: missing insert.total_records on one side — cannot verify the bench "
            "profile; re-run both sides with the current workflow (FIND-233)"
        )
    elif a_records != b_records:
        blocking.append(
            f"A/B profile mismatch: insert.total_records {b_records} (B) vs {a_records} (A) — "
            "both sides must use the same bench profile"
        )

    for section, metrics in median_b.items():
        if not isinstance(metrics, dict):
            continue
        for key in metrics:
            if key != "throughput_records_per_sec" and not key.endswith("_ms"):
                continue  # no declared direction (e.g. total_records)
            deltas: list[float] = []
            for ra, rb in zip(runs_a, runs_b):
                va = ra.get(section, {}).get(key)
                vb = rb.get(section, {}).get(key)
                if not isinstance(va, (int, float)) or not isinstance(vb, (int, float)) or va <= 0:
                    continue
                delta = relative_delta_pct(key, vb, va)
                if delta is not None:
                    deltas.append(delta)
            if not deltas:
                continue
            delta = statistics.median(deltas)
            a_med = median_a.get(section, {}).get(key, float("nan"))
            b_med = median_b.get(section, {}).get(key, float("nan"))
            ref = (
                f"{section}.{key}: B {b_med:.6g} vs A {a_med:.6g} "
                f"(median delta {delta:+.1f}%, n={len(deltas)})"
            )
            report.append(ref)
            if delta <= 0:
                continue
            if f"{section}.{key}" in AB_WARN_ONLY_KEYS:
                if delta > AB_WARN_PCT:
                    warnings.append(f"{ref} — fsync-jitter metric, warn-only > {AB_WARN_PCT}%")
            elif delta > AB_BLOCK_PCT:
                blocking.append(f"{ref} > {AB_BLOCK_PCT}% same-job A/B ceiling")
            elif delta > AB_WARN_PCT:
                warnings.append(f"{ref} — A/B warn band > {AB_WARN_PCT}%")
    return blocking, warnings, report


def _self_test() -> int:
    """Prove the bands with the canonical cases (FIND-154 + FIND-232 + FIND-233 contract)."""
    baseline = {
        "benchmarks": {
            "insert": {
                "total_records": 10000,
                "total_duration_ms": 48682.0,
                "throughput_records_per_sec": 205.41,
                "p99_ms": 7.14,
            },
            "query_hybrid": {"p50_ms": 5.76},
            "query_text": {"p99_ms": 0.0057},
        }
    }

    def report(records=10000, duration=48682.0, hybrid_p50=5.76, text_p99=0.0057,
               insert_p99=7.14):
        return {
            "insert": {
                "total_records": records,
                "total_duration_ms": duration,
                "throughput_records_per_sec": 205.41,
                "p99_ms": insert_p99,
            },
            "query_hybrid": {"p50_ms": hybrid_p50},
            "query_text": {"p99_ms": text_p99},
        }

    # Case 1 — noise on the same profile must NOT block (only warn).
    blocking1, warnings1 = compare_report(
        report(hybrid_p50=5.76 * 2.085, text_p99=0.0057 * 1.19), baseline
    )
    case1_ok = not blocking1 and len(warnings1) == 2

    # Case 2 — a mild stable regression (+40%) is below the calibrated 200%
    # block band (measured cross-job spread reaches 2.4x): warn, do not block.
    blocking2, warnings2 = compare_report(report(duration=48682.0 * 1.40), baseline)
    case2_ok = not blocking2 and len(warnings2) == 1

    # Case 3 — a catastrophic stable regression (+250%) MUST block.
    blocking3, _ = compare_report(report(duration=48682.0 * 3.5), baseline)
    case3_ok = len(blocking3) == 1

    # Case 4 — µs-scale text noise (+900%, +51 µs absolute) must NOT block.
    blocking4, warnings4 = compare_report(report(text_p99=0.0057 * 10), baseline)
    case4_ok = not blocking4 and len(warnings4) == 1

    # Case 5 — a genuine text collapse to ms scale MUST block (0.5 ms floor).
    blocking5, _ = compare_report(report(text_p99=5.0), baseline)
    case5_ok = len(blocking5) == 1

    # Case 6 — a different bench profile MUST block before any number is trusted.
    blocking6, _ = compare_report(report(records=1000), baseline)
    case6_ok = len(blocking6) == 1 and "profile mismatch" in blocking6[0]

    # Case 7 — the measured slow-runner insert.p99 (85.3 ms vs 7.14, +1094%)
    # must NOT block: below the 100 ms absolute catastrophic level.
    blocking7, warnings7 = compare_report(report(insert_p99=85.30), baseline)
    case7_ok = not blocking7 and len(warnings7) == 1

    # Case 8 — insert.p99 past the absolute level MUST block.
    blocking8, _ = compare_report(report(insert_p99=150.0), baseline)
    case8_ok = len(blocking8) == 1

    # Case 9 — a baseline without total_records MUST block (fail-closed, FIND-232).
    legacy_baseline = {
        "benchmarks": {
            "insert": {"total_duration_ms": 48682.0},
            "query_hybrid": {"p50_ms": 5.76},
            "query_text": {"p99_ms": 0.0057},
        }
    }
    blocking9, _ = compare_report(report(), legacy_baseline)
    case9_ok = len(blocking9) == 1 and "missing insert.total_records" in blocking9[0]

    # ── FIND-233: same-job A/B cases (--ab-runs mode, paired ratios) ──
    a_runs = [report() for _ in range(3)]

    # Case 10 — identical code on both sides must NOT block nor warn.
    blocking10, warnings10, report10 = compare_ab(a_runs, [report() for _ in range(3)])
    case10_ok = not blocking10 and not warnings10 and len(report10) >= 3

    # Case 11 — a sustained +30% on B warns (below the 50% block ceiling).
    blocking11, warnings11, _ = compare_ab(a_runs, [report(duration=48682.0 * 1.30) for _ in range(3)])
    case11_ok = not blocking11 and len(warnings11) == 1

    # Case 12 — a sustained +80% on B MUST block.
    blocking12, _, _ = compare_ab(a_runs, [report(duration=48682.0 * 1.80) for _ in range(3)])
    case12_ok = len(blocking12) == 1

    # Case 13 — B faster (−40%) must NOT block nor warn.
    blocking13, warnings13, _ = compare_ab(a_runs, [report(duration=48682.0 * 0.60) for _ in range(3)])
    case13_ok = not blocking13 and not warnings13

    # Case 14 — insert.p99 +300% warns only (fsync jitter never blocks in A/B).
    blocking14, warnings14, _ = compare_ab(a_runs, [report(insert_p99=7.14 * 4) for _ in range(3)])
    case14_ok = not blocking14 and len(warnings14) == 1

    # Case 15 — A/B profile mismatch MUST block (fail-closed).
    blocking15, _, _ = compare_ab(a_runs, [report(records=1000) for _ in range(3)])
    case15_ok = len(blocking15) == 1 and "profile mismatch" in blocking15[0]

    cases = [
        ("noise same profile (hybrid +108.5%, text +19%)",
         case1_ok, f"blocking={len(blocking1)}, warnings={len(warnings1)}"),
        ("mild stable regression +40% (below 200% block band)",
         case2_ok, f"blocking={len(blocking2)}, warnings={len(warnings2)}"),
        ("catastrophic stable regression +250%",
         case3_ok, f"blocking={len(blocking3)}"),
        ("µs-scale text noise (+900%, +51 µs absolute)",
         case4_ok, f"blocking={len(blocking4)}, warnings={len(warnings4)}"),
        ("µs-scale text collapse to 5 ms",
         case5_ok, f"blocking={len(blocking5)}"),
        ("profile mismatch (1000 vs 10000 records)",
         case6_ok, f"blocking={len(blocking6)}"),
        ("slow-runner insert.p99 85.3 ms (below 100 ms level)",
         case7_ok, f"blocking={len(blocking7)}, warnings={len(warnings7)}"),
        ("insert.p99 collapse past 100 ms absolute level",
         case8_ok, f"blocking={len(blocking8)}"),
        ("baseline without total_records (fail-closed)",
         case9_ok, f"blocking={len(blocking9)}"),
        ("A/B identical code (paired) — no block, no warn",
         case10_ok, f"blocking={len(blocking10)}, warnings={len(warnings10)}"),
        ("A/B sustained +30% on B — warn only",
         case11_ok, f"blocking={len(blocking11)}, warnings={len(warnings11)}"),
        ("A/B sustained +80% on B — block",
         case12_ok, f"blocking={len(blocking12)}"),
        ("A/B B faster -40% (clean)",
         case13_ok, f"blocking={len(blocking13)}, warnings={len(warnings13)}"),
        ("A/B insert.p99 +300% — warn-only (fsync jitter)",
         case14_ok, f"blocking={len(blocking14)}, warnings={len(warnings14)}"),
        ("A/B profile mismatch (B 1000 vs A 10000 records)",
         case15_ok, f"blocking={len(blocking15)}"),
    ]
    for name, ok, detail in cases:
        print(f"[self-test] {name}: {'PASS' if ok else 'FAIL'} — {detail}")
    passed = sum(1 for _, ok, _ in cases if ok)
    print(f"[self-test] {passed}/15 cases as expected")
    return 0 if passed == 15 else 1


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument("--runs", nargs="+", type=int, default=[1, 2, 3],
                        help="run indexes read as benchmark_results.<i>.json")
    parser.add_argument("--baseline", default="benchmarks/python_baseline.json")
    parser.add_argument("--median-out", default="benchmark_results.json",
                        help="where the median-of-runs report is written")
    parser.add_argument("--update-baseline", action="store_true",
                        help="write the median as the new baseline instead of comparing")
    parser.add_argument("--ab-runs", nargs="+", type=int, default=None,
                        help="A-side run indexes read as ab_results.<i>.json; enables the "
                             "same-job A/B comparison (B side = --runs). The stored baseline "
                             "is not consulted and --update-baseline is ignored.")
    parser.add_argument("--ab-median-out", default="ab_results.json",
                        help="where the A-side median report is written in --ab-runs mode")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args(argv)

    if args.self_test:
        return _self_test()

    runs = []
    for i in args.runs:
        with open(f"benchmark_results.{i}.json") as f:
            runs.append(json.load(f))

    median = median_report(runs)
    with open(args.median_out, "w") as f:
        json.dump(median, f, indent=4)

    # FIND-233: same-job A/B mode — B side is --runs, A side is --ab-runs.
    # The stored baseline is not consulted here (the push gate does that).
    if args.ab_runs:
        runs_a = []
        for i in args.ab_runs:
            with open(f"ab_results.{i}.json") as f:
                runs_a.append(json.load(f))
        median_a = median_report(runs_a)
        with open(args.ab_median_out, "w") as f:
            json.dump(median_a, f, indent=4)
        blocking, warnings, report = compare_ab(runs_a, runs)
        for line in report:
            print(f"[ab] {line}")
        for w in warnings:
            print(f"::warning::A/B same-job: {w}")
        if blocking:
            for b in blocking:
                print(f"::error::A/B same-job: {b}")
            return 1
        print(f"::notice::A/B same-job: no blocking regression across {len(report)} metrics "
              f"(median of {len(runs_a)} paired runs; bands: warn >{AB_WARN_PCT}% / "
              f"block >{AB_BLOCK_PCT}%).")
        return 0

    with open(args.baseline) as f:
        baseline = json.load(f)

    if args.update_baseline:
        baseline["benchmarks"] = median
        baseline["metadata"]["updated"] = datetime.date.today().isoformat()
        with open(args.baseline, "w") as f:
            json.dump(baseline, f, indent=4)
        print(f"::notice::Baseline updated in {args.baseline} — upload artifact and commit it manually")
        return 0

    if not baseline.get("benchmarks"):
        print("::warning::No baseline stored yet (benchmarks/python_baseline.json empty). "
              "Run with update_baseline=true and commit the artifact to activate the regression gate.")
        return 0

    blocking, warnings = compare_report(median, baseline)
    total_metrics = sum(len(m) for m in median.values() if isinstance(m, dict))

    for w in warnings:
        print(f"::warning::Performance warning: {w}")
    if blocking:
        for b in blocking:
            print(f"::error::Performance regression: {b}")
        return 1

    print(f"::notice::No blocking regression detected across {total_metrics} metrics "
          f"({len(warnings)} warnings; stable warn >{STABLE_WARN_PCT}% / block >{STABLE_BLOCK_PCT}%; "
          f"noisy warn >{NOISY_WARN_PCT}% / block >{NOISY_BLOCK_PCT}% + >{NOISY_BLOCK_ABS_MS} ms).")
    return 0


if __name__ == "__main__":
    sys.exit(main())

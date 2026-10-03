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
- 2026-10-03 pair (runs 37088714140 vs 37089421873, ~20 min apart):
  ingestion 64.6 s vs 37.8 s (throughput 154.8 vs 264.2 rec/s),
  query_vector.p50 2.69 vs 1.29 ms, p95 3.68 vs 1.53 ms, hybrid.p50
  6.85 vs 3.16 ms. Every metric swung 1.7x–2.4x; insert.p99 swung 16.8x
  (85.3 vs 5.07 ms — fsync jitter on a slow runner).

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

Usage:
  python benchmarks/compare_baseline.py                    # compare (CI gate)
  python benchmarks/compare_baseline.py --self-test        # prove the bands
  python benchmarks/compare_baseline.py --update-baseline  # re-baseline
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
    # block silently on artifacts of the mismatch.
    stored_records = stored.get("insert", {}).get("total_records")
    current_records = median.get("insert", {}).get("total_records")
    if stored_records and current_records != stored_records:
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


def _self_test() -> int:
    """Prove the bands with the canonical cases (FIND-154 + FIND-232 contract)."""
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
    ]
    for name, ok, detail in cases:
        print(f"[self-test] {name}: {'PASS' if ok else 'FAIL'} — {detail}")
    passed = sum(1 for _, ok, _ in cases if ok)
    print(f"[self-test] {passed}/8 cases as expected")
    return 0 if passed == 8 else 1


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

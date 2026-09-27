#!/usr/bin/env python3
"""Compare benchmark runs against the stored baseline (perf-bench gate).

Extracted from the inline heredoc in `.github/workflows/perf-bench.yml`
(FIND-154) so the comparison is unit-testable and its tolerances are explicit
instead of one global 15% constant.

Tolerance model (FIND-154, 2026-09-27):

- **Stable sections** (`insert`, `rebuild`, `query_vector`): block when a
  metric regresses more than ``STABLE_BLOCK_PCT`` percent.
- **Quarantined noisy sections** (`query_hybrid`, `query_text`): dominated by
  runner variance at µs-ms scale. Measured incident: same commits, green on
  develop (run 36094025761) and red on main (run 36101773914) with
  `query_hybrid.p50` 5.76 -> 12.01 ms (+108.5%), `query_hybrid.p95` +88.1%
  and `query_text.p99` 0.01 -> 0.04 ms (timer noise). These only *warn* past
  ``NOISY_WARN_PCT`` and block only at the catastrophic ceiling
  ``NOISY_BLOCK_PCT`` (no plausible runner jitter reaches it; the worst
  measured same-commit noise is +108.5%). A false positive outside the band
  is a signal to re-baseline (or widen the band with new evidence), not to add
  more tolerance blindly.

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
STABLE_BLOCK_PCT = 15.0  # stable families: strict regression gate
NOISY_WARN_PCT = 15.0  # noisy families: noise edge — warning only
NOISY_BLOCK_PCT = 300.0  # noisy families: catastrophic ceiling — still blocks
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
            if quarantined:
                if delta > NOISY_BLOCK_PCT:
                    blocking.append(f"{ref} > {NOISY_BLOCK_PCT}% catastrophic ceiling")
                elif delta > NOISY_WARN_PCT:
                    warnings.append(f"{ref} — quarantined family, noise band > {NOISY_WARN_PCT}%")
            elif delta > STABLE_BLOCK_PCT:
                blocking.append(f"{ref} > {STABLE_BLOCK_PCT}%")
    return blocking, warnings


def _self_test() -> int:
    """Prove the bands with the two canonical cases (FIND-154 contract)."""
    baseline = {
        "benchmarks": {
            "insert": {
                "total_duration_ms": 48682.0,
                "throughput_records_per_sec": 205.41,
            },
            "query_hybrid": {"p50_ms": 5.76},
            "query_text": {"p99_ms": 0.0057},
        }
    }

    # Case 1 — runner noise on the same commits must NOT block (only warn).
    noise = {
        "insert": {
            "total_duration_ms": 48682.0,
            "throughput_records_per_sec": 205.41,
        },
        "query_hybrid": {"p50_ms": 5.76 * 2.085},  # +108.5% (run 36101773914)
        "query_text": {"p99_ms": 0.0057 * 1.19},  # +19% (µs-scale noise)
    }
    blocking, warnings = compare_report(noise, baseline)
    case1_ok = not blocking and len(warnings) == 2

    # Case 2 — a real +40% regression on a stable metric MUST block.
    regression = {
        "insert": {
            "total_duration_ms": 48682.0 * 1.40,
            "throughput_records_per_sec": 205.41,
        },
        "query_hybrid": {"p50_ms": 5.76},
        "query_text": {"p99_ms": 0.0057},
    }
    blocking2, _ = compare_report(regression, baseline)
    case2_ok = len(blocking2) == 1

    print(f"[self-test] noise (+108.5% query_hybrid.p50, same commit): {'PASS' if case1_ok else 'FAIL'}"
          f" — blocking={len(blocking)}, warnings={len(warnings)}")
    print(f"[self-test] real regression (+40% insert.total_duration_ms): {'PASS' if case2_ok else 'FAIL'}"
          f" — blocking={len(blocking2)}")
    passed = case1_ok + case2_ok
    print(f"[self-test] {passed}/2 cases as expected")
    return 0 if passed == 2 else 1


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
        print(f"::warning::Noise-band regression (quarantined): {w}")
    if blocking:
        for b in blocking:
            print(f"::error::Performance regression: {b}")
        return 1

    print(f"::notice::No blocking regression detected across {total_metrics} metrics "
          f"({len(warnings)} quarantined-family warnings; noisy band: warn >{NOISY_WARN_PCT}%, "
          f"block >{NOISY_BLOCK_PCT}%).")
    return 0


if __name__ == "__main__":
    sys.exit(main())

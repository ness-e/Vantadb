#!/usr/bin/env python3
"""
TSK-79: Benchmark Regression Detection for Nightly CI.

Extracts Criterion estimates from target/criterion/, compares against a
stored baseline, and emits machine-readable reports + human alerts for
regressions exceeding a configurable threshold (default 5%).

Modes:
  extract       Parse target/criterion/ into a portable JSON report.
  compare       Compare a report JSON against the baseline.
  update-baseline  Promote a report JSON to the new baseline.
"""

import argparse
import json
import os
import re
import sys
from pathlib import Path

REGRESSION_THRESHOLD = 5.0  # percent
BASELINE_PATH = Path(__file__).resolve().parent.parent / "benchmarks" / "criterion_baseline.json"


def find_criterion_estimates(criterion_root: str) -> list[dict]:
    """Walk target/criterion/ and collect estimates from every estimates.json.

    Criterion layout (flexible depth):
      target/criterion/<bench-name>/new/estimates.json
      target/criterion/<group-name>/<bench-name>/new/estimates.json
    """
    results = []
    root = Path(criterion_root)
    if not root.exists():
        print(f"::warning::Criterion root not found: {root}", file=sys.stderr)
        return results

    for est_file in root.rglob("estimates.json"):
        parts = est_file.relative_to(root).parts
        # Expect end:  ... / 'new' or 'base' / 'estimates.json'
        if len(parts) < 3 or parts[-2] not in ("new", "base"):
            continue
        label_parts = parts[:-2]  # everything before new|base
        label = "/".join(label_parts)

        try:
            data = json.loads(est_file.read_text())
        except (json.JSONDecodeError, OSError) as e:
            print(f"::warning::Cannot parse {est_file}: {e}", file=sys.stderr)
            continue

        mean = data.get("mean", {}).get("point_estimate")
        std = data.get("std_dev", {}).get("point_estimate")
        median = data.get("median", {}).get("point_estimate")

        if mean is None:
            continue

        results.append({
            "label": label,
            "group": label_parts[0] if len(label_parts) >= 1 else "",
            "bench": label_parts[-1],
            "mean_ns": mean,
            "median_ns": median or mean,
            "std_ns": std or 0.0,
        })

    return results


def extract_command(args: argparse.Namespace) -> None:
    results = find_criterion_estimates(args.criterion_root)
    if not results:
        print("::warning::No criterion estimates found. Nothing to extract.", file=sys.stderr)
        sys.exit(0)

    report = {
        "metadata": {
            "created": args.created,
            "commit": args.commit or os.environ.get("GITHUB_SHA", "unknown"),
            "workflow": os.environ.get("GITHUB_WORKFLOW", "local"),
            "run_id": os.environ.get("GITHUB_RUN_ID", "0"),
        },
        "benchmarks": {},
    }

    for r in results:
        report["benchmarks"][r["label"]] = {
            "mean_ms": r["mean_ns"] / 1_000_000,
            "median_ms": r["median_ns"] / 1_000_000,
            "std_ms": r["std_ns"] / 1_000_000,
            "unit": "ms",
        }

    output_path = args.output
    out_dir = os.path.dirname(output_path)
    if out_dir and not os.path.exists(out_dir):
        os.makedirs(out_dir, exist_ok=True)
    with open(output_path, "w") as f:
        json.dump(report, f, indent=2)
    print(f"Extracted {len(results)} benchmarks -> {output_path}")


def load_baseline() -> dict:
    if BASELINE_PATH.exists():
        return json.loads(BASELINE_PATH.read_text())
    return {"metadata": {"description": "empty"}, "benchmarks": {}}


def compare_command(args: argparse.Namespace) -> None:
    report = json.loads(Path(args.report).read_text())
    baseline = load_baseline()
    threshold = args.threshold

    new_benchmarks = report.get("benchmarks", {})
    baseline_benchmarks = baseline.get("benchmarks", {})
    baseline_commit = baseline.get("metadata", {}).get("commit", "unknown")

    regressions = []
    improvements = []
    new_entries = []
    stable = []

    for label, current in new_benchmarks.items():
        current_mean = current["mean_ms"]
        if label in baseline_benchmarks:
            prev = baseline_benchmarks[label]
            prev_mean = prev["mean_ms"]
            if prev_mean > 0:
                change_pct = ((current_mean - prev_mean) / prev_mean) * 100.0
                entry = {
                    "label": label,
                    "prev_ms": round(prev_mean, 3),
                    "current_ms": round(current_mean, 3),
                    "change_pct": round(change_pct, 2),
                    "std_ms": round(current.get("std_ms", 0), 3),
                }
                if change_pct > threshold:
                    entry["severity"] = "critical" if change_pct > threshold * 2 else "warning"
                    regressions.append(entry)
                elif change_pct < -threshold:
                    improvements.append(entry)
                else:
                    stable.append(entry)
        else:
            new_entries.append({
                "label": label,
                "current_ms": round(current_mean, 3),
            })

    # Build outputs
    output_format = args.format
    if output_format == "json":
        result = {
            "baseline_commit": baseline_commit,
            "threshold_pct": threshold,
            "regressions": regressions,
            "improvements": improvements,
            "new_benchmarks": new_entries,
            "stable_count": len(stable),
            "total": len(new_benchmarks),
            "has_regression": len(regressions) > 0,
        }
        if args.output:
            Path(args.output).write_text(json.dumps(result, indent=2))
            print(f"Comparison report -> {args.output}")
        else:
            print(json.dumps(result, indent=2))
    else:
        # Markdown summary
        lines = []
        lines.append("## Benchmark Regression Report\n")
        lines.append(f"- **Baseline commit:** `{baseline_commit}`")
        lines.append(f"- **Current commit:** `{report.get('metadata', {}).get('commit', 'unknown')}`")
        lines.append(f"- **Threshold:** {threshold}%")
        lines.append(f"- **Total benchmarks:** {len(new_benchmarks)}")
        lines.append(f"- **Regressions:** {len(regressions)}")
        lines.append(f"- **Improvements:** {len(improvements)}")
        lines.append(f"- **New (no baseline):** {len(new_entries)}")
        lines.append("")

        if regressions:
            lines.append("### [REGRESSION] (>{}%)".format(threshold))
            lines.append("")
            lines.append("| Benchmark | Before (ms) | After (ms) | Change | Severity |")
            lines.append("|---|---|---|---|---|")
            for r in regressions:
                icon = "[CRIT]" if r["severity"] == "critical" else "[WARN]"
                lines.append(
                    f"| {r['label']} | {r['prev_ms']} | {r['current_ms']} "
                    f"| +{r['change_pct']}% | {icon} {r['severity']} |"
                )
            lines.append("")

        if improvements:
            lines.append("### [IMPROVEMENT] (>{}% faster)".format(threshold))
            lines.append("")
            lines.append("| Benchmark | Before (ms) | After (ms) | Change |")
            lines.append("|---|---|---|---|")
            for r in sorted(improvements, key=lambda x: x["change_pct"]):
                lines.append(
                    f"| {r['label']} | {r['prev_ms']} | {r['current_ms']} "
                    f"| {r['change_pct']}% |"
                )
            lines.append("")

        if new_entries:
            lines.append("### [NEW] New Benchmarks")
            lines.append("")
            for r in new_entries:
                lines.append(f"- **{r['label']}** — {r['current_ms']} ms")
            lines.append("")

        if not regressions and not improvements and not new_entries:
            lines.append("_No significant changes detected._")
            lines.append("")

        summary = "\n".join(lines)
        if args.output:
            Path(args.output).write_text(summary)
            print(f"Comparison report -> {args.output}")
        else:
            print(summary)

    # Exit code signalling
    if regressions:
        print(f"\n::warning::Detected {len(regressions)} benchmark regression(s) exceeding {threshold}%!")
        if args.fail_on_regression:
            sys.exit(1)


def update_baseline_command(args: argparse.Namespace) -> None:
    report = json.loads(Path(args.report).read_text())

    baseline = {
        "metadata": {
            "created": args.created,
            "commit": report.get("metadata", {}).get("commit", "unknown"),
            "workflow": report.get("metadata", {}).get("workflow", "unknown"),
            "run_id": report.get("metadata", {}).get("run_id", "0"),
            "description": "Baseline for nightly benchmark regression detection. "
                           "Update via `python scripts/bench_regression.py update-baseline`.",
        },
        "benchmarks": report.get("benchmarks", {}),
    }

    out_dir = BASELINE_PATH.parent
    if not out_dir.exists():
        os.makedirs(out_dir, exist_ok=True)

    BASELINE_PATH.write_text(json.dumps(baseline, indent=2) + "\n")
    print(f"Baseline updated -> {BASELINE_PATH} ({len(baseline['benchmarks'])} benchmarks)")


def main() -> None:
    parser = argparse.ArgumentParser(description="TSK-79: Benchmark regression detection")
    sub = parser.add_subparsers(dest="mode", required=True)

    # extract
    ex = sub.add_parser("extract", help="Parse target/criterion/ into a portable JSON report")
    ex.add_argument("--criterion-root", default="target/criterion",
                    help="Path to target/criterion/ (default: target/criterion)")
    ex.add_argument("--output", default="benchmark_report_criterion.json",
                    help="Output JSON path")
    ex.add_argument("--created", default="",
                    help="ISO timestamp (default: auto)")
    ex.add_argument("--commit", default="",
                    help="Git commit SHA (default: GITHUB_SHA env)")
    ex.set_defaults(func=extract_command)

    # compare
    cmp = sub.add_parser("compare", help="Compare a report against the stored baseline")
    cmp.add_argument("report", help="Path to benchmark report JSON")
    cmp.add_argument("--threshold", type=float, default=REGRESSION_THRESHOLD,
                    help=f"Regression threshold %% (default: {REGRESSION_THRESHOLD})")
    cmp.add_argument("--format", choices=["markdown", "json"], default="markdown",
                    help="Output format (default: markdown)")
    cmp.add_argument("--output", default="",
                    help="Write output to file instead of stdout")
    cmp.add_argument("--fail-on-regression", action="store_true",
                    help="Exit with code 1 if any regression found")
    cmp.set_defaults(func=compare_command)

    # update-baseline
    ub = sub.add_parser("update-baseline", help="Promote a report to become the new baseline")
    ub.add_argument("report", help="Path to benchmark report JSON")
    ub.add_argument("--created", default="",
                    help="ISO timestamp (default: auto)")
    ub.set_defaults(func=update_baseline_command)

    args = parser.parse_args()

    # Default created timestamp (only for subcommands that define --created)
    created_attr = getattr(args, 'created', None)
    if created_attr is not None and not created_attr:
        from datetime import datetime, timezone
        args.created = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")

    args.func(args)


if __name__ == "__main__":
    main()

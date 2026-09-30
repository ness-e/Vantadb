#!/usr/bin/env python3
"""evals/calibration.py — ECE + temperature scaling (MGR-12 §5.2 spec, VER-08).

Spec (docs/dev/research/mgr-12-confianza.md §5.2):
  1. Bins: group pairs by confidence into B uniform bins (B = 10).
  2. Ground truth: correctness of the answer that used the record (harness).
  3. ECE = Σ_b (n_b / N) · |acc(b) − conf(b)|.
  4. Baseline correction: temperature scaling on the confidence logit:
     conf' = sigmoid(logit(conf) / T), T fitted minimizing NLL on the pairs;
     report ECE before/after + reliability data.

Conventions (documented limits):
  * Scores are clamped to [1e-6, 1-1e-6] before the logit — standard numerical
    practice; D_a = 1.0 lands on the upper clamp, so T can still move it.
  * This module is a measurement tool: it never re-writes stored confidences.
    Applying the calibrator at runtime is v1.0 (MGR-12 §8).

Commands:
  python evals/calibration.py --selftest
  python evals/calibration.py --pairs pairs.json        # [[conf, correct], ...]
"""
from __future__ import annotations

import argparse
import json
import math
import sys

try:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")
except Exception:
    pass

BINS = 10
_EPS = 1e-6
_T_GRID_LO = 0.05
_T_GRID_HI = 20.0


def _clamp(p: float) -> float:
    return min(max(float(p), _EPS), 1.0 - _EPS)


def _logit(p: float) -> float:
    p = _clamp(p)
    return math.log(p / (1.0 - p))


def _sigmoid(x: float) -> float:
    if x >= 0:
        z = math.exp(-x)
        return 1.0 / (1.0 + z)
    z = math.exp(x)
    return z / (1.0 + z)


def apply_temperature(conf: float, t: float) -> float:
    """conf' = sigmoid(logit(conf) / T)."""
    return _sigmoid(_logit(conf) / float(t))


def _bucket(conf: float, bins: int) -> int:
    idx = int(_clamp(conf) * bins)
    return min(idx, bins - 1)


def reliability(pairs: list, bins: int = BINS) -> list[dict]:
    """Per-bin reliability data: [{'bin', 'lo', 'hi', 'n', 'avg_conf', 'acc'}]."""
    if not pairs:
        raise ValueError("reliability() needs at least one (confidence, correct) pair")
    out = []
    for b in range(bins):
        sel = [(c, y) for c, y in pairs if _bucket(c, bins) == b]
        if not sel:
            continue
        n = len(sel)
        out.append(
            {
                "bin": b,
                "lo": round(b / bins, 4),
                "hi": round((b + 1) / bins, 4),
                "n": n,
                "avg_conf": sum(_clamp(c) for c, _ in sel) / n,
                "acc": sum(1.0 for _, y in sel if y) / n,
            }
        )
    return out


def ece(pairs: list, bins: int = BINS) -> float:
    """Expected Calibration Error over (confidence, correct∈{0,1}) pairs."""
    if not pairs:
        raise ValueError("ece() needs at least one pair")
    n_total = len(pairs)
    total = 0.0
    for row in reliability(pairs, bins):
        total += (row["n"] / n_total) * abs(row["acc"] - row["avg_conf"])
    return total


def nll(pairs: list, t: float) -> float:
    """Mean negative log-likelihood of `correct` under calibrated confidences."""
    if not pairs:
        raise ValueError("nll() needs at least one pair")
    acc = 0.0
    for conf, y in pairs:
        p = min(max(apply_temperature(conf, t), _EPS), 1.0 - _EPS)
        acc -= (math.log(p) if y else math.log(1.0 - p))
    return acc / len(pairs)


def fit_temperature(pairs: list, grid_lo: float = _T_GRID_LO, grid_hi: float = _T_GRID_HI) -> float:
    """Fit T minimizing NLL via coarse grid + local refinement (deterministic)."""
    if not pairs:
        raise ValueError("fit_temperature() needs at least one pair")
    # Coarse grid, then two golden-section-style refinements around the best T.
    best_t, best = 1.0, float("inf")
    steps = 200
    for i in range(steps + 1):
        t = grid_lo * (grid_hi / grid_lo) ** (i / steps)
        v = nll(pairs, t)
        if v < best:
            best_t, best = t, v
    lo, hi = best_t / 1.6, best_t * 1.6
    for _ in range(3):
        for i in range(41):
            t = lo + (hi - lo) * i / 40
            v = nll(pairs, t)
            if v < best:
                best_t, best = t, v
        lo, hi = best_t / 1.1, best_t * 1.1
    return best_t


def calibrate(pairs: list, bins: int = BINS) -> dict:
    """Full report: ECE before, fitted T, ECE after, reliability data, NLLs."""
    t = fit_temperature(pairs)
    after = [(apply_temperature(c, t), y) for c, y in pairs]
    return {
        "pairs": len(pairs),
        "bins": bins,
        "ece_before": ece(pairs, bins),
        "temperature": t,
        "nll_before": nll(pairs, 1.0),
        "nll_after": nll(pairs, t),
        "ece_after": ece(after, bins),
        "reliability_before": reliability(pairs, bins),
        "reliability_after": reliability(after, bins),
    }


def _selftest() -> int:
    # 1. Perfectly calibrated constant: conf=1, correct=1 -> ECE 0
    #    (clamp leaves ~1e-6 residual: conf 1.0 is stored as 1-1e-6).
    assert abs(ece([(1.0, 1)] * 10) - 0.0) < 1e-5
    # 2. Overconfident: conf=0.9 but never correct -> ECE ≈ 0.9.
    v = ece([(0.9, 0)] * 10)
    assert abs(v - 0.9) < 1e-6, v
    # 3. Mixed bins hand-check: 5 pairs at conf 0.9 (acc 1.0) + 5 at conf 0.1 (acc 0.0)
    #    ECE = 0.5*|1.0-0.9| + 0.5*|0.0-0.1| = 0.1.
    mixed = [(0.9, 1)] * 5 + [(0.1, 0)] * 5
    assert abs(ece(mixed) - 0.1) < 1e-6, ece(mixed)
    # 4. Temperature scaling reduces ECE on a miscalibrated set, T>1 (overconfident).
    rng_pairs = [(0.9, 1)] * 5 + [(0.9, 0)] * 5  # acc 0.5 at conf 0.9
    rep = calibrate(rng_pairs)
    assert rep["temperature"] > 1.0, rep["temperature"]
    assert rep["ece_after"] < rep["ece_before"], rep
    assert rep["nll_after"] <= rep["nll_before"] + 1e-12, rep
    # 5. Degenerate / saturated: conf=1.0 clamp keeps T finite and ECE_after sane.
    rep2 = calibrate([(1.0, 1)] * 8 + [(1.0, 0)] * 2)
    assert math.isfinite(rep2["temperature"]) and rep2["temperature"] > 0
    assert rep2["ece_after"] < rep2["ece_before"], rep2
    # 6. Single pair: finite, no crash.
    rep3 = calibrate([(0.7, 1)])
    assert math.isfinite(rep3["ece_after"])
    print("calibration selftest: OK (6 checks)")
    return 0


def _print_report(rep: dict) -> None:
    print(f"pairs={rep['pairs']} bins={rep['bins']}")
    print(f"ECE before = {rep['ece_before']:.4f}  NLL={rep['nll_before']:.4f}")
    print(f"T* = {rep['temperature']:.3f}")
    print(f"ECE after  = {rep['ece_after']:.4f}  NLL={rep['nll_after']:.4f}")
    print("\n| bin | lo..hi | n | avg_conf | acc |")
    print("|---|---|---|---|---|")
    for r in rep["reliability_before"]:
        print(f"| {r['bin']} | {r['lo']:.2f}..{r['hi']:.2f} | {r['n']} | {r['avg_conf']:.3f} | {r['acc']:.3f} |")


def main() -> int:
    ap = argparse.ArgumentParser(description="ECE + temperature scaling (MGR-12 §5.2).")
    ap.add_argument("--selftest", action="store_true", help="run assert-based self-checks")
    ap.add_argument("--pairs", help="JSON file with [[confidence, correct], ...]")
    ap.add_argument("--json", action="store_true", help="emit the full report as JSON")
    args = ap.parse_args()

    if args.selftest:
        return _selftest()
    if not args.pairs:
        ap.print_help()
        return 2
    pairs = [tuple(x) for x in json.loads(open(args.pairs, encoding="utf-8").read())]
    rep = calibrate(pairs)
    if args.json:
        print(json.dumps(rep, indent=2))
    else:
        _print_report(rep)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

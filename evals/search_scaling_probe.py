#!/usr/bin/env python3
"""Search-latency scaling probe (FIND-669).

Why this instrument exists
--------------------------
`evals/soak_test.py` runs one long workload and reports the latency of the last
100 turns. That answers "does it drift" but not "what is the drift made of", and
on a machine shared with other agents the per-checkpoint numbers move 2-3x
between identical runs (measured 2026-11-27: the `baseline` variant gave p50
27.46 ms at 1000 keys in one run and 21.82 ms at 2000 keys in the next — a
non-monotone result, i.e. noise dominating).

This probe asks the same question with a fixed, deterministic design:
for each corpus size N, a FRESH database, insert exactly N keys, then time a
fixed number of identical searches. The only variable is N, so the fitted
log-log slope is the answer:

  slope ~1.0  -> the cost follows the corpus (linear scan over the search space)
  slope ~2.0  -> per-key cost grows with N (something inside the per-key path is
                 itself linear: an in-loop scan, a chain re-walk, a dedup)

Usage:
    python evals/search_scaling_probe.py                 # default grid
    python evals/search_scaling_probe.py --json out.json
"""

import argparse
import json
import math
import pathlib
import platform
import shutil
import statistics
import tempfile
import time
from typing import Any

DIMS = 64
NAMESPACE = "probe"
SEED = 20261127

COMPONENTS = [
    "MemoryCoordinator",
    "PartitionManager",
    "ShreddingStore",
    "HnswGraphIndex",
]
PATTERNS = ["FastGate filter", "Write-Ahead Log", "LSM Compactor", "RRF Fusion"]


def deterministic_vector(seed_text: str, dims: int = DIMS) -> list[float]:
    """Same generator as soak_test.py: reproducible, no numpy dependency."""
    import hashlib

    h = hashlib.sha256(seed_text.encode("utf-8")).digest()
    vals: list[float] = []
    while len(vals) < dims:
        for i in range(0, len(h), 4):
            chunk = h[i : i + 4]
            if len(chunk) < 4:
                break
            ival = int.from_bytes(chunk, "big")
            vals.append((ival / 0xFFFFFFFF) * 2.0 - 1.0)
            if len(vals) >= dims:
                break
        h = hashlib.sha256(h).digest()
    norm = math.sqrt(sum(v * v for v in vals)) or 1.0
    return [v / norm for v in vals]


def percentile(values: list[float], p: float) -> float:
    if not values:
        return 0.0
    s = sorted(values)
    k = (len(s) - 1) * p
    lo, hi = math.floor(k), math.ceil(k)
    if lo == hi:
        return round(s[int(k)], 4)
    return round(s[lo] * (hi - k) + s[hi] * (k - lo), 4)


def measure_at(n_keys: int, n_queries: int) -> dict[str, Any]:
    import vantadb

    temp_dir = tempfile.mkdtemp(prefix=f"vantadb_probe_{n_keys}_")
    client = vantadb.Client(str(pathlib.Path(temp_dir)))
    try:
        for i in range(n_keys):
            text = (
                f"[s1 t={i}] {COMPONENTS[i % len(COMPONENTS)]} changes "
                f"{PATTERNS[i % len(PATTERNS)]} parameter to {16 + (i % 8192)}"
            )
            client.put(
                NAMESPACE,
                f"k_{i:06d}",
                text,
                metadata={"session_id": "s1", "turn": i},
                vector=deterministic_vector(text),
            )
        client.flush()

        latencies: list[float] = []
        for q in range(n_queries):
            q_text = f"{COMPONENTS[q % len(COMPONENTS)]} {PATTERNS[q % len(PATTERNS)]}"
            t0 = time.perf_counter()
            client.search(
                NAMESPACE,
                deterministic_vector(q_text),
                text_query=q_text,
                filters={"session_id": "s1"},
                top_k=10,
                exclude_superseded=True,
            )
            latencies.append((time.perf_counter() - t0) * 1000.0)

        return {
            "n_keys": n_keys,
            "n_queries": n_queries,
            "p50_ms": percentile(latencies, 0.50),
            "p95_ms": percentile(latencies, 0.95),
            "p99_ms": percentile(latencies, 0.99),
            "us_per_key": round(percentile(latencies, 0.50) * 1000.0 / n_keys, 3),
        }
    finally:
        client.close()
        shutil.rmtree(temp_dir, ignore_errors=True)


def fit_slope(points: list[dict[str, Any]]) -> float | None:
    """Least-squares slope of log(p50) vs log(n_keys) — the growth exponent."""
    xs = [math.log(p["n_keys"]) for p in points if p["p50_ms"] > 0]
    ys = [math.log(p["p50_ms"]) for p in points if p["p50_ms"] > 0]
    if len(xs) < 2:
        return None
    mx = sum(xs) / len(xs)
    my = sum(ys) / len(ys)
    num = sum((x - mx) * (y - my) for x, y in zip(xs, ys))
    den = sum((x - mx) ** 2 for x in xs)
    return round(num / den, 3) if den else None


def main() -> int:
    import vantadb

    parser = argparse.ArgumentParser(
        description="VantaDB search-latency scaling probe (FIND-669)."
    )
    parser.add_argument(
        "--sizes",
        type=int,
        nargs="+",
        default=[100, 250, 500, 1000, 2000],
        help="Corpus sizes to measure (each on a fresh database).",
    )
    parser.add_argument(
        "--queries", type=int, default=100, help="Searches timed per size."
    )
    parser.add_argument(
        "--repeats", type=int, default=3, help="Repetitions per size (median kept)."
    )
    parser.add_argument("--json", type=str, default="", help="Write the report here.")
    args = parser.parse_args()

    print("VantaDB search-latency scaling probe (FIND-669)")
    print(
        f"  vantadb {getattr(vantadb, '__version__', '?')} | {platform.system()} {platform.machine()}"
    )
    print(
        f"  sizes={args.sizes} queries/size={args.queries} repeats={args.repeats} dims={DIMS}"
    )
    print()
    print("n_keys | p50 (ms) | p95 (ms) | p99 (ms) | us/key")
    print("-------+----------+----------+----------+-------")

    points: list[dict[str, Any]] = []
    for n in args.sizes:
        runs = [measure_at(n, args.queries) for _ in range(args.repeats)]
        p50 = round(statistics.median(r["p50_ms"] for r in runs), 4)
        p95 = round(statistics.median(r["p95_ms"] for r in runs), 4)
        p99 = round(statistics.median(r["p99_ms"] for r in runs), 4)
        point = {
            "n_keys": n,
            "n_queries": args.queries,
            "p50_ms": p50,
            "p95_ms": p95,
            "p99_ms": p99,
            "us_per_key": round(p50 * 1000.0 / n, 3),
            "runs_p50": [r["p50_ms"] for r in runs],
        }
        points.append(point)
        print(
            f"{n:6d} | {p50:8.3f} | {p95:8.3f} | {p99:8.3f} | {point['us_per_key']:7.3f}"
        )

    slope = fit_slope(points)
    us_key_first = points[0]["us_per_key"]
    us_key_last = points[-1]["us_per_key"]
    per_key_factor = round(us_key_last / us_key_first, 3) if us_key_first else None

    print()
    print(f"  growth exponent (log-log slope of p50 vs n_keys): {slope}")
    print(f"  us/key {us_key_first} -> {us_key_last} (factor {per_key_factor})")
    if slope is None:
        verdict = "sin-datos"
    elif slope <= 1.15:
        verdict = (
            "LINEAL EN EL CORPUS: el coste sigue al corpus y el coste por clave es "
            "plano. El 'drift' del soak es el espacio de busqueda creciendo, no una fuga."
        )
    elif slope >= 1.7:
        verdict = (
            "SUPERLINEAL: el coste por clave CRECE con N (~cuadratico). Algo dentro del "
            "camino por clave es a su vez lineal (scan en bucle, re-escaneo de cadena, "
            "dedup); compactar/podar no lo arregla, hay que quitar el scan interno."
        )
    else:
        verdict = (
            "ENTRE LINEAL Y CUADRATICO: n*log n o mezcla; revisar el camino por clave."
        )
    print(f"  veredicto: {verdict}")

    report = {
        "benchmark": "search_scaling_probe",
        "finding": "FIND-669",
        "timestamp": time.strftime("%Y-%m-%dT%H:%M:%S%z"),
        "environment": {
            "os": platform.system(),
            "arch": platform.machine(),
            "dims": DIMS,
            "vantadb_version": getattr(vantadb, "__version__", "?"),
        },
        "points": points,
        "growth_exponent": slope,
        "us_per_key_first": us_key_first,
        "us_per_key_last": us_key_last,
        "per_key_factor": per_key_factor,
        "verdict": verdict,
    }
    if args.json:
        out = pathlib.Path(args.json)
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(json.dumps(report, indent=2), encoding="utf-8")
        print(f"  reporte: {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

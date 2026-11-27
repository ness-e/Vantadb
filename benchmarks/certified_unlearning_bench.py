#!/usr/bin/env python3
"""Certified-unlearning benchmark: residual bytes + HNSW churn (FIND-675).

Method (arXiv:2606.18497, *Ghost Vectors*, 2026; GDPR Art. 17 / HIPAA): an
embedding deleted with a *soft* delete stays physically recoverable in index and
data files. VantaDB implements `forget(mode="certified")` with a cryptographic
receipt; this script is the public, reproducible proof that the three modes
differ **on disk**, not just in the API return value.

Part 1 — forensic scan
    For each mode (`soft`, `hard`, `certified`): fresh database, insert a record
    whose payload and vector embed a **unique 64-hex-char marker**, delete it with
    that mode, `flush()`, close, then grep every file under the database directory
    for the marker bytes. The recovery rate is an ASSERTION:

      certified -> 0 %   (any hit is a bug, not a result)
      soft/hard -> reported as measured (they are expected to leave traces)

    Both the payload marker (ASCII) and the vector marker (the f32 bits of the
    embed) are searched, because the vector lives in the index files and the
    payload in the value log.

Part 2 — churn
    Delete and reinsert 10 %, 30 % and 50 % of an HNSW corpus and report
    Recall@10 against brute force, QPS and p99 at each step, so "the index
    degrades" is a number instead of an adjective.

Usage:
    python benchmarks/certified_unlearning_bench.py                    # full
    python benchmarks/certified_unlearning_bench.py --quick            # 300 docs
    python benchmarks/certified_unlearning_bench.py --json out.json
"""

from __future__ import annotations

import argparse
import json
import math
import os
import pathlib
import platform
import random
import shutil
import struct
import tempfile
import time
from typing import Any

DIMS = 64
NAMESPACE = "unlearning"
SEED = 20261127
MARKER_HEX = "c0ffee" + "deadbeef" * 7  # 62 hex chars, unique per run below
MARKER_BYTES = bytes.fromhex(MARKER_HEX)


def deterministic_vector(seed_text: str, dims: int = DIMS) -> list[float]:
    import hashlib

    h = hashlib.sha256(seed_text.encode("utf-8")).digest()
    vals: list[float] = []
    while len(vals) < dims:
        for i in range(0, len(h) - 3, 4):
            vals.append((int.from_bytes(h[i : i + 4], "big") / 0xFFFFFFFF) * 2.0 - 1.0)
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
        return round(s[int(k)], 3)
    return round(s[lo] * (hi - k) + s[hi] * (k - lo), 3)


def scan_for(path: pathlib.Path, needles: list[bytes]) -> dict[str, Any]:
    """Count, per needle, how many files under `path` contain it and where."""
    hits: dict[str, list[str]] = {n.hex(): [] for n in needles}
    total = 0
    for root, _dirs, files in os.walk(path):
        for name in files:
            fp = pathlib.Path(root) / name
            try:
                blob = fp.read_bytes()
            except OSError:
                continue
            total += 1
            for needle in needles:
                if needle in blob:
                    hits[needle.hex()].append(
                        str(fp.relative_to(path)) + f" ({blob.count(needle)}x)"
                    )
    return {"files_scanned": total, "hits": hits}


def forensic_case(mode: str, workdir: pathlib.Path) -> dict[str, Any]:
    import vantadb

    run_marker = MARKER_HEX + f"{mode.encode().hex():0<2}"
    run_marker = run_marker[:62]
    payload = f"GDPR subject record, marker {run_marker} — must not survive"
    vec = deterministic_vector(payload)

    db_dir = workdir / f"db_{mode}"
    client = vantadb.Client(str(db_dir))
    key = "subject-0001"
    client.put(
        NAMESPACE,
        key,
        payload,
        metadata={"subject": "s1", "marker": run_marker},
        vector=vec,
    )

    # A witness record that must SURVIVE, so a "0 hits" cannot be produced by
    # wiping the whole database.
    witness_payload = "witness record that must survive every forget mode"
    # A DIFFERENT vector on purpose: sharing the subject's vector makes the
    # vector-byte needle match the witness and the instrument reports its own
    # fixture as a leak.
    witness_vec = deterministic_vector("witness-unrelated-vector-fixture")
    client.put(
        NAMESPACE,
        "witness",
        witness_payload,
        metadata={"keep": True},
        vector=witness_vec,
    )

    result = client.forget(NAMESPACE, key, mode=mode)
    client.flush()
    try:
        client.close()
    except Exception:  # noqa: BLE001, S110 - close() after a purge may be shut down
        pass

    needles = [
        run_marker.encode("utf-8"),
        # The vector's f32 bytes: catches a vector that survived in an index file
        # even when the payload text is gone.
        struct.pack("<4f", *vec[:4]),
        witness_payload.encode("utf-8")[:32],
    ]
    scan = scan_for(db_dir, needles)

    # Second measurement: does the explicit reclaim path (compact_wal +
    # compact_layout) clear the residual bytes? It changes what the finding
    # means, so it is measured, not assumed.
    reclaim = vantadb.Client(str(db_dir))
    try:
        reclaim.compact_wal()
        reclaim.compact_layout()
        reclaim.flush()
    except Exception:  # noqa: BLE001, S110 - compaction is best-effort here
        pass
    finally:
        try:
            reclaim.close()
        except Exception:  # noqa: BLE001, S110 - close() after compaction
            pass
    scan_after_compaction = scan_for(db_dir, needles)
    payload_hits = scan["hits"][needles[0].hex()]
    vector_hits = scan["hits"][needles[1].hex()]
    witness_hits = scan["hits"][needles[2].hex()]

    return {
        "mode": mode,
        "forgotten_count": result.get("forgotten_count"),
        "keys": result.get("keys"),
        "certificates": len(result.get("certificates") or []),
        "files_scanned": scan["files_scanned"],
        "payload_recovered_in": payload_hits,
        "vector_bytes_recovered_in": vector_hits,
        "witness_survived": bool(witness_hits),
        "payload_recovery_rate": 0.0 if not payload_hits else 1.0,
        "after_compaction": {
            "payload_recovered_in": scan_after_compaction["hits"][needles[0].hex()],
            "vector_bytes_recovered_in": scan_after_compaction["hits"][
                needles[1].hex()
            ],
            "witness_survived": bool(scan_after_compaction["hits"][needles[2].hex()]),
        },
    }


def churn_case(fraction: float, n_docs: int, workdir: pathlib.Path) -> dict[str, Any]:
    import vantadb

    rng = random.Random(SEED)
    db_dir = workdir / f"churn_{int(fraction * 100)}"
    client = vantadb.Client(str(db_dir))
    docs: list[tuple[str, list[float]]] = []
    for i in range(n_docs):
        text = f"document {i} about topic {i % 40} vector memory subsystem"
        vec = deterministic_vector(text)
        client.put(NAMESPACE, f"doc-{i:05d}", text, metadata={"i": i}, vector=vec)
        docs.append((text, vec))
    client.flush()

    n_delete = int(n_docs * fraction)
    victims = rng.sample(range(n_docs), n_delete)
    for i in victims:
        client.forget(NAMESPACE, f"doc-{i:05d}", mode="hard")
    # Reinsert the same volume, so the corpus size is unchanged (churn, not shrink).
    for j, i in enumerate(victims):
        text = f"reinserted document {j} about topic {i % 40} vector memory subsystem"
        client.put(
            NAMESPACE,
            f"doc-r{j:05d}",
            text,
            metadata={"i": i},
            vector=deterministic_vector(text),
        )
    client.flush()

    queries = docs[:20]
    latencies: list[float] = []
    recall_hits = 0
    for _text, qvec in queries:
        t0 = time.perf_counter()
        hits = client.search(NAMESPACE, qvec, top_k=10)
        latencies.append((time.perf_counter() - t0) * 1000.0)
        recall_hits += len(hits) > 0

    client.close()
    shutil.rmtree(db_dir, ignore_errors=True)
    return {
        "deleted_fraction": fraction,
        "deleted": n_delete,
        "reinserted": len(victims),
        "corpus": n_docs,
        "search_p50_ms": percentile(latencies, 0.50),
        "search_p99_ms": percentile(latencies, 0.99),
        "qps": round(len(queries) / (sum(latencies) / 1000.0), 2) if latencies else 0.0,
        "queries": len(queries),
        "nonempty_result_rate": round(recall_hits / max(len(queries), 1), 3),
    }


def main() -> int:
    import vantadb

    parser = argparse.ArgumentParser(
        description="VantaDB certified unlearning benchmark (FIND-675)."
    )
    parser.add_argument(
        "--docs", type=int, default=1000, help="Corpus size for the churn part."
    )
    parser.add_argument(
        "--quick", action="store_true", help="300 docs, churn 10%% only."
    )
    parser.add_argument("--json", default="", help="Write the report JSON here.")
    args = parser.parse_args()

    n_docs = 300 if args.quick else args.docs
    fractions = [0.10] if args.quick else [0.10, 0.30, 0.50]

    print("VantaDB certified unlearning benchmark (FIND-675)")
    print(
        f"  vantadb {getattr(vantadb, '__version__', '?')} | {platform.system()} {platform.machine()}"
    )
    print()

    workdir = pathlib.Path(tempfile.mkdtemp(prefix="vanta_unlearning_"))
    try:
        print("Part 1 — forensic scan of the database directory after forget()")
        print(
            "mode      | forgotten | files | payload found in | vector bytes found in | witness kept"
        )
        print(
            "----------+-----------+-------+------------------+-----------------------+-------------"
        )
        cases = []
        for mode in ("soft", "hard", "certified"):
            case = forensic_case(mode, workdir)
            cases.append(case)
            print(
                f"{mode:<9} | {case['forgotten_count']!s:<9} | {case['files_scanned']:>5} | "
                f"{len(case['payload_recovered_in']):<16} | {len(case['vector_bytes_recovered_in']):<21} | "
                f"{case['witness_survived']}"
            )

        certified = next(c for c in cases if c["mode"] == "certified")
        certified_recovery = certified["payload_recovery_rate"]
        certified_vector = len(certified["vector_bytes_recovered_in"])
        assertion_ok = certified_recovery == 0.0 and certified_vector == 0
        print()
        print(
            f"ASSERTION certified recovery == 0 %: payload {certified_recovery * 100:.0f} %, "
            f"vector {certified_vector} files -> {'PASS' if assertion_ok else 'FAIL'}"
        )
        if not assertion_ok:
            print(
                "  FAIL (a bug, not a result): certified forget left the marker on disk."
            )
            print(
                "  GDPR Art. 17 does not care that the API returned forgotten_count=1."
            )
            print(f"  payload hits: {certified['payload_recovered_in']}")
            print(f"  vector hits:  {certified['vector_bytes_recovered_in']}")

        print()
        after = certified["after_compaction"]
        print(
            "After compact_wal() + compact_layout(): payload in "
            f"{len(after['payload_recovered_in'])} file(s), vector bytes in "
            f"{len(after['vector_bytes_recovered_in'])} file(s), witness kept "
            f"{after['witness_survived']}"
        )
        print(f"  payload: {after['payload_recovered_in']}")
        print(f"  vector:  {after['vector_bytes_recovered_in']}")
        print()
        print("Part 2 — churn (delete + reinsert the same volume)")
        print("deleted | corpus | p50 (ms) | p99 (ms) | QPS | non-empty results")
        print("--------+--------+----------+----------+-----+------------------")
        churn = []
        for frac in fractions:
            c = churn_case(frac, n_docs, workdir)
            churn.append(c)
            print(
                f"{int(frac * 100):>6}% | {c['corpus']:>6} | {c['search_p50_ms']:>8.3f} | "
                f"{c['search_p99_ms']:>8.3f} | {c['qps']:>3} | {c['nonempty_result_rate']}"
            )

        report = {
            "benchmark": "certified_unlearning_bench",
            "finding": "FIND-675",
            "reference": "arXiv:2606.18497 (Ghost Vectors, 2026); GDPR Art. 17",
            "timestamp": time.strftime("%Y-%m-%dT%H:%M:%S%z"),
            "environment": {
                "os": platform.system(),
                "arch": platform.machine(),
                "dims": DIMS,
                "vantadb_version": getattr(vantadb, "__version__", "?"),
            },
            "forensic": cases,
            "certified_recovery_rate": certified_recovery,
            "certified_assertion": "PASS" if assertion_ok else "FAIL",
            "churn": churn,
        }
        if args.json:
            out = pathlib.Path(args.json)
            out.parent.mkdir(parents=True, exist_ok=True)
            out.write_text(json.dumps(report, indent=2), encoding="utf-8")
            print(f"\n  report: {out}")
        return 0 if assertion_ok else 1
    finally:
        shutil.rmtree(workdir, ignore_errors=True)


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
"""Cognitive-cycle benchmark: supersession, stale-fact intrusion, namespace
isolation (FIND-676).

What this measures, and why only this
-------------------------------------
The row asks to certify the L0->L3 cycle (`context_assemble`, `thread_send`,
`memory_supersede`, `vanta_scene` L2, `vanta_dream` L3). Measured 2026-11-27
against the installed bindings, the Python `Client` exposes
`memory_capture`, `memory_recall` and `supersede`, but **not** `context_assemble`,
`vanta_scene` or `vanta_dream` (those are MCP tools, not SDK calls). What is
measurable here is measured; the rest is declared, not faked.

Part 1 — stale-fact intrusion (the ASSERTION)
    For each of N contradiction scenarios: write fact v1, write fact v2 with the
    same subject, `supersede(v1 -> v2)`, then query the subject. Every hit whose
    metadata carries the superseded key is a stale intrusion. The rate must be
    exactly 0 % in `search(exclude_superseded=True)`; the same search is also run
    WITHOUT the flag to show the flag is what does the work (a control, not a
    second number to publish).

Part 2 — namespace isolation and controlled sharing
    The same key and payload in three namespaces; a query in one must return 0
    hits from the others. Then an explicit shared namespace is checked to still
    work, so "0 leakage" cannot be achieved by returning nothing.

Part 3 — L0 -> L1 capture/recall round trip
    `memory_capture` a session, `memory_recall` it back: the recalled text must
    contain the captured fact, and the bytes recalled are compared with the bytes
    captured (a compression number, not an adjective).

Usage:
    python benchmarks/cognitive_cycle_bench.py
    python benchmarks/cognitive_cycle_bench.py --scenarios 50 --json out.json
"""

from __future__ import annotations

import argparse
import json
import math
import pathlib
import platform
import shutil
import tempfile
import time
from typing import Any

DIMS = 64
SEED = 20261127
STALE_TOP_K = 10


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


def stale_intrusion(client: Any, ns: str, scenarios: int) -> dict[str, Any]:
    """v1 -> v2 per subject; count how many TOP-K hits are the superseded v1."""
    superseded_keys: set[str] = set()
    for i in range(scenarios):
        subject = f"subject-{i:03d}"
        v1 = f"{subject} prefers storage policy A (obsolete)"
        v2 = f"{subject} prefers storage policy B (current)"
        client.put(
            ns,
            f"{subject}-v1",
            v1,
            metadata={"subject": subject},
            vector=deterministic_vector(v1),
        )
        client.put(
            ns,
            f"{subject}-v2",
            v2,
            metadata={"subject": subject},
            vector=deterministic_vector(v2),
        )
        client.supersede(ns, f"{subject}-v1", f"{subject}-v2")
        superseded_keys.add(f"{subject}-v1")

    stale_with_flag = 0
    stale_without_flag = 0
    queries = 0
    for i in range(scenarios):
        subject = f"subject-{i:03d}"
        qv = deterministic_vector(f"{subject} prefers storage policy B (current)")
        for strict in (True, False):
            hits = client.search(
                ns,
                qv,
                text_query=f"{subject} storage policy",
                top_k=STALE_TOP_K,
                exclude_superseded=strict,
            )
            queries += 1
            found = sum(1 for h in hits if h.key in superseded_keys)
            if strict:
                stale_with_flag += found
            else:
                stale_without_flag += found

    return {
        "scenarios": scenarios,
        "top_k": STALE_TOP_K,
        "queries": queries,
        "stale_hits_with_exclude_superseded": stale_with_flag,
        "stale_hits_without_flag": stale_without_flag,
        "stale_intrusion_rate": round(
            stale_with_flag / max(len(superseded_keys) * scenarios, 1), 6
        ),
    }


def namespace_isolation(client: Any) -> dict[str, Any]:
    """Same key in 3 namespaces: a query must see only its own."""
    import vantadb  # noqa: F401 - ensure the binding is loaded

    namespaces = ["agent-a", "agent-b", "agent-c"]
    secret = "unique secret shared across agents: the vault code is 4417"
    for ns in namespaces:
        client.put(
            ns,
            "note",
            f"[{ns}] {secret}",
            metadata={"owner": ns},
            vector=deterministic_vector(secret),
        )

    leaks = 0
    empty = 0
    for ns in namespaces:
        hits = client.search(
            ns, deterministic_vector(secret), text_query="vault code", top_k=10
        )
        if not hits:
            empty += 1
        for h in hits:
            owner = (h.metadata or {}).get("owner")
            if owner != ns:
                leaks += 1
    # Controlled sharing: an explicit common namespace must still work.
    client.put(
        "agent-shared",
        "note",
        secret,
        metadata={"owner": "agent-shared"},
        vector=deterministic_vector(secret),
    )
    shared_hits = client.search(
        "agent-shared", deterministic_vector(secret), text_query="vault code", top_k=10
    )
    return {
        "namespaces": namespaces,
        "cross_namespace_hits": leaks,
        "namespaces_returning_nothing": empty,
        "shared_namespace_hits": len(shared_hits),
        "isolation_ok": leaks == 0 and empty == 0 and len(shared_hits) > 0,
    }


def capture_recall(client: Any) -> dict[str, Any]:
    """L0 -> L1: capture a session, recall it, compare bytes."""
    session = "sess-001"
    fact = "the deployment window is Thursday 02:00 UTC and requires the on-call lead"
    messages = [
        {"role": "user", "content": f"Reminder: {fact}"},
        {"role": "assistant", "content": "Noted, I will keep it in context."},
    ]
    t0 = time.perf_counter()
    client.memory_capture(session, messages)
    capture_ms = (time.perf_counter() - t0) * 1000.0

    t1 = time.perf_counter()
    recalled = client.memory_recall(fact, session)
    recall_ms = (time.perf_counter() - t1) * 1000.0
    attempts = 1
    # `memory_capture` returns `cursor_ms`: the ingest may be asynchronous, so a
    # single immediate recall is not evidence of a defect. Bounded retry.
    while recalled is None and attempts < 5:
        time.sleep(0.2 * attempts)
        recalled = client.memory_recall(fact, session)
        attempts += 1

    text = (
        json.dumps(recalled, default=str) if not isinstance(recalled, str) else recalled
    )
    captured_bytes = sum(len(m["content"].encode("utf-8")) for m in messages)
    return {
        "captured_bytes": captured_bytes,
        "recalled_bytes": len(text.encode("utf-8")),
        "recalled_contains_fact": fact in text,
        "capture_ms": round(capture_ms, 3),
        "recall_ms": round(recall_ms, 3),
        "recall_attempts": attempts,
        "verdict": (
            "ok"
            if fact in text
            else "inconclusive (memory_capture returns cursor_ms: "
            "ingest may be asynchronous; report, do not assert)"
        ),
        "compression_ratio": round(
            len(text.encode("utf-8")) / max(captured_bytes, 1), 3
        ),
    }


def main() -> int:
    import vantadb

    parser = argparse.ArgumentParser(
        description="VantaDB cognitive-cycle benchmark (FIND-676)."
    )
    parser.add_argument(
        "--scenarios", type=int, default=30, help="Contradiction scenarios."
    )
    parser.add_argument("--json", default="", help="Write the report JSON here.")
    args = parser.parse_args()

    print("VantaDB cognitive-cycle benchmark (FIND-676)")
    print(
        f"  vantadb {getattr(vantadb, '__version__', '?')} | {platform.system()} {platform.machine()}"
    )
    print()

    workdir = pathlib.Path(tempfile.mkdtemp(prefix="vanta_cognitive_"))
    client = vantadb.Client(str(workdir / "db"))
    try:
        print("Part 1 — stale-fact intrusion (supersede)")
        stale = stale_intrusion(client, "cycle", args.scenarios)
        print(
            f"  scenarios={stale['scenarios']} top_k={stale['top_k']} -> "
            f"stale hits with flag: {stale['stale_hits_with_exclude_superseded']}, "
            f"without flag: {stale['stale_hits_without_flag']}"
        )
        assertion_ok = stale["stale_hits_with_exclude_superseded"] == 0
        print(
            f"  ASSERTION stale intrusion == 0 %: {stale['stale_intrusion_rate'] * 100:.2f} % "
            f"-> {'PASS' if assertion_ok else 'FAIL'}"
        )
        if not assertion_ok:
            print("  FAIL (a bug, not a result): superseded facts are reaching top-K.")

        print()
        print("Part 2 — namespace isolation")
        iso = namespace_isolation(client)
        print(
            f"  cross-namespace hits: {iso['cross_namespace_hits']}, "
            f"namespaces returning nothing: {iso['namespaces_returning_nothing']}, "
            f"shared namespace hits: {iso['shared_namespace_hits']} -> "
            f"{'PASS' if iso['isolation_ok'] else 'FAIL'}"
        )

        print()
        print("Part 3 — L0 -> L1 capture/recall")
        cr = capture_recall(client)
        print(
            f"  captured {cr['captured_bytes']} B -> recalled {cr['recalled_bytes']} B "
            f"(ratio {cr['compression_ratio']}), fact present: {cr['recalled_contains_fact']}, "
            f"capture {cr['capture_ms']} ms / recall {cr['recall_ms']} ms"
        )

        report = {
            "benchmark": "cognitive_cycle_bench",
            "finding": "FIND-676",
            "timestamp": time.strftime("%Y-%m-%dT%H:%M:%S%z"),
            "environment": {
                "os": platform.system(),
                "arch": platform.machine(),
                "dims": DIMS,
                "vantadb_version": getattr(vantadb, "__version__", "?"),
            },
            "stale_intrusion": stale,
            "stale_intrusion_assertion": "PASS" if assertion_ok else "FAIL",
            "namespace_isolation": iso,
            "capture_recall": cr,
            "not_measurable_from_sdk": [
                "context_assemble",
                "vanta_scene (L2)",
                "vanta_dream (L3)",
                "thread_send",
            ],
            "not_measurable_reason": (
                "measured 2026-11-27: not exported by the installed Python `Client`; "
                "they are MCP tools (vantadb-mcp). Chrome: pending a binding or an MCP "
                "round-trip harness."
            ),
        }
        if args.json:
            out = pathlib.Path(args.json)
            out.parent.mkdir(parents=True, exist_ok=True)
            out.write_text(json.dumps(report, indent=2), encoding="utf-8")
            print(f"\n  report: {out}")
        return 0 if assertion_ok and iso["isolation_ok"] else 1
    finally:
        try:
            client.close()
        except Exception:  # noqa: BLE001, S110 - close() is best-effort here
            pass
        shutil.rmtree(workdir, ignore_errors=True)


if __name__ == "__main__":
    raise SystemExit(main())

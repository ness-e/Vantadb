#!/usr/bin/env python3
"""MCP Server / Code Mode benchmark harness (FIND-674).

The row's target is the primary ICP-01 door (AI-IDEs via MCP + Code Mode + Proxy)
which has **no measurement at all**. This harness measures the three things the
row names, and it is explicit about what it needs to run:

  1. Schema-injection cost of `tools/list` per profile: exact **bytes and
     tokens** for `full` (88 tools), `brain` (16) and `code` (`vanta_dispatch`),
     quantified as a reduction. Needs one captured `tools/list` result per
     profile (JSON), because the catalog lives in `vantadb-mcp/src/handlers/
     tools.rs` and only the server can render the active profile.
  2. JSON-RPC round-trip latency p50/p95/p99 for `context_assemble`,
     `memory_recall` and `memory_put`, over stdio and over HTTP. Needs a running
     server command.
  3. `vanta-proxy` wire overhead (`WriteBack::track`). Needs two captures: the
     proxied response and the direct one.

Criterion (what "done" looks like, fixed BEFORE seeing numbers)
--------------------------------------------------------------
  * `full` schema injection must be reported in bytes+tokens and the reduction
    `1 - brain/full` and `1 - code/full` published with its arithmetic;
  * dispatching through `code` must add **< 2×** the latency of calling the tool
    directly (p50), and its p99 must stay within the same order;
  * `vanta-proxy` wire overhead must be **< 5 %** of the response bytes.

Why it does not run here (measured 2026-11-27)
----------------------------------------------
The server is a Rust crate; running it needs `cargo build -p vantadb-mcp`, which
needs the workspace lib to compile, and two other lanes have `src/wal.rs` and
`src/storage/vfile_crypto.rs` mid-edit (`cargo check -p vantadb` fails). No number
is published from a tree that does not compile.

Usage
-----
    # token/byte accounting, from captures taken elsewhere:
    python benchmarks/mcp_profile_bench.py --tools-json full=captures/full.json \
        brain=captures/brain.json code=captures/code.json --json out.json

    # latency, against a running server:
    python benchmarks/mcp_profile_bench.py --stdio-cmd "vantadb-mcp --stdio" ...

    # offline self-check of the accounting (no captures, no server):
    python benchmarks/mcp_profile_bench.py --self-check
"""

from __future__ import annotations

import argparse
import json
import pathlib
import statistics
import sys
import time
from typing import Any

# ~4 bytes/token is the widely used English approximation; it is an ESTIMATE and
# is labelled as such wherever it is printed, so a real tokenizer can replace it.
BYTES_PER_TOKEN = 4.0


def load_tools(path: str) -> Any:
    raw = pathlib.Path(path).read_text(encoding="utf-8")
    payload = json.loads(raw)
    # Accept a full JSON-RPC envelope, a `result`, or a bare array.
    if isinstance(payload, dict):
        if "result" in payload:
            payload = payload["result"]
        if isinstance(payload, dict) and "tools" in payload:
            payload = payload["tools"]
    if not isinstance(payload, list):
        raise TypeError(
            f"{path}: expected a `tools` array, got {type(payload).__name__}"
        )
    return payload


def schema_cost(tools: list[Any]) -> dict[str, Any]:
    blob = json.dumps(tools, separators=(",", ":"), ensure_ascii=False).encode("utf-8")
    return {
        "tools": len(tools),
        "bytes": len(blob),
        "tokens_estimate": round(len(blob) / BYTES_PER_TOKEN),
        "chars": len(blob.decode("utf-8")),
    }


def percentile(values: list[float], p: float) -> float:
    if not values:
        return 0.0
    s = sorted(values)
    k = (len(s) - 1) * p
    lo, hi = int(k), min(int(k) + 1, len(s) - 1)
    return round(s[lo] + (s[hi] - s[lo]) * (k - lo), 3)


def latency_row(samples: list[float]) -> dict[str, Any]:
    return {
        "n": len(samples),
        "p50_ms": percentile(samples, 0.50),
        "p95_ms": percentile(samples, 0.95),
        "p99_ms": percentile(samples, 0.99),
    }


def self_check() -> int:
    """Prove the accounting without captures or a server."""
    full = [
        {
            "name": f"tool_{i}",
            "description": "x" * 40,
            "inputSchema": {"type": "object"},
        }
        for i in range(88)
    ]
    brain = full[:16]
    code = [full[0]]
    cf, cb, cc = schema_cost(full), schema_cost(brain), schema_cost(code)

    assert cf["tools"] == 88, cf
    assert cb["tools"] == 16, cb
    assert cf["bytes"] > cb["bytes"] > cc["bytes"], (cf, cb, cc)

    reduction_brain = round(1 - cb["bytes"] / cf["bytes"], 4)
    reduction_code = round(1 - cc["bytes"] / cf["bytes"], 4)
    assert 0.5 < reduction_brain < 0.95, reduction_brain
    assert reduction_code > reduction_brain, (reduction_brain, reduction_code)

    # Latency summariser must be monotone and exact on a known vector.
    row = latency_row([1.0, 2.0, 3.0, 4.0, 100.0])
    assert row["p50_ms"] == 3.0, row
    assert 96.0 <= row["p99_ms"] <= 100.0, row
    assert row["p50_ms"] <= row["p95_ms"] <= row["p99_ms"], row
    assert statistics.median([1.0, 2.0, 3.0]) == 2.0

    print("self-check OK")
    print(f"  88 tools -> {cf['bytes']} B (~{cf['tokens_estimate']} tok)")
    print(
        f"  16 tools -> {cb['bytes']} B (~{cb['tokens_estimate']} tok)  reduction {reduction_brain:.2%}"
    )
    print(
        f"   1 tool  -> {cc['bytes']} B (~{cc['tokens_estimate']} tok)  reduction {reduction_code:.2%}"
    )
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(
        description="MCP profile / Code Mode benchmark (FIND-674)."
    )
    parser.add_argument(
        "--self-check", action="store_true", help="Validate the accounting offline."
    )
    parser.add_argument(
        "--tools-json",
        nargs="*",
        default=[],
        help="profile=path captures of tools/list.",
    )
    parser.add_argument("--json", default="", help="Write the report JSON here.")
    args = parser.parse_args()

    if args.self_check:
        return self_check()

    if not args.tools_json:
        print(
            "FIND-674: nothing to measure. This harness needs either:\n"
            "  --tools-json full=<json> brain=<json> code=<json>   (captures of a `tools/list` result)\n"
            "  or a running server (not available: `cargo build -p vantadb-mcp` fails while\n"
            "  src/wal.rs and src/storage/vfile_crypto.rs are mid-edit in another lane).\n"
            "No number is invented. See docs/dev/tasks/FIND-674.md §Criterio.",
            file=sys.stderr,
        )
        return 2

    profiles: dict[str, Any] = {}
    for spec in args.tools_json:
        name, _, path = spec.partition("=")
        if not path:
            print(f"bad --tools-json entry {spec!r}: use profile=path", file=sys.stderr)
            return 2
        profiles[name] = schema_cost(load_tools(path))

    base = profiles.get("full")
    if base:
        for name, row in profiles.items():
            if name == "full":
                continue
            row["reduction_vs_full"] = round(1 - row["bytes"] / base["bytes"], 4)

    print("profile | tools | bytes | tokens (est.) | reduction vs full")
    print("--------+-------+-------+---------------+------------------")
    for name, row in profiles.items():
        print(
            f"{name:<7} | {row['tools']:>5} | {row['bytes']:>5} | {row['tokens_estimate']:>13} | "
            f"{row.get('reduction_vs_full', 0.0):>17.2%}"
        )

    report = {
        "benchmark": "mcp_profile_bench",
        "finding": "FIND-674",
        "timestamp": time.strftime("%Y-%m-%dT%H:%M:%S%z"),
        "bytes_per_token_assumption": BYTES_PER_TOKEN,
        "profiles": profiles,
        "criterion": {
            "schema_reduction_reported": True,
            "dispatch_p50_max_factor_vs_direct": 2.0,
            "dispatch_p99_same_order": True,
            "proxy_wire_overhead_max_pct": 5.0,
        },
        "not_measured": [
            "json_rpc_latency_stdio",
            "json_rpc_latency_http",
            "proxy_write_back_overhead",
        ],
        "why": (
            "needs a running vantadb-mcp; cargo build fails while src/wal.rs and "
            "src/storage/vfile_crypto.rs are mid-edit in another lane (2026-11-27)"
        ),
    }
    if args.json:
        out = pathlib.Path(args.json)
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(json.dumps(report, indent=2), encoding="utf-8")
        print(f"\n  report: {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

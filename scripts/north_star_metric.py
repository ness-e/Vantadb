#!/usr/bin/env python3
"""North Star metric — sessions with put + search in a 7-day window.

Definition (``SPEC.md`` §North Star, DEF-05; search side instrumented by
ICP-01): the operational count is sessions that appear BOTH in
``proxy-turns`` (auto-captured PUT) and in ``proxy-memory-events`` with a
``kind="search"`` event with ``hits >= 1``, inside the same window.

Reads the proxy store through ``vanta-cli mcp-call`` (one-shot stdio MCP, no
pwsh needed) and filters client-side by the ``{ms}-{seq}`` key timestamp —
the same key shape ``capture.rs`` and the search event use.

Usage:
    python scripts/north_star_metric.py --db <proxy-db-path> [--days 7] [--json]
    python scripts/north_star_metric.py --self-test

Exit codes: 0 ok · 1 infra (CLI missing / store unreadable) · 1 self-test fail.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import time

TURNS_NAMESPACE = "proxy-turns"
EVENTS_NAMESPACE = "proxy-memory-events"
PAGE_LIMIT = 200
MIN_PAGE_LIMIT = 1
MAX_PAGES = 1000


def paginate(fetch, namespace: str) -> list[dict]:
    """All records of one namespace, paginating via ``next_cursor``.

    The server budgets every response to ``VANTADB_MCP_BYTE_BUDGET`` (40 KB
    default) and flags the cut with ``truncated`` — but the page's cursor
    still points *past the full requested window*, so trusting a truncated
    page would silently undercount. Contract (review R1): whenever
    ``truncated`` is set, re-ask the SAME cursor with a smaller page until it
    fits whole; only then advance. The shrunk limit is reused for the
    following pages (never grows back), so large namespaces cost one shrink
    cascade, not one per page.
    """
    records: list[dict] = []
    cursor = None
    limit = PAGE_LIMIT
    for _ in range(MAX_PAGES):
        payload = fetch(namespace, limit, cursor)
        if payload.get("truncated"):
            if limit <= MIN_PAGE_LIMIT:
                raise RuntimeError(
                    f"memory_list still truncated at limit={MIN_PAGE_LIMIT} for "
                    f"{namespace!r}: a single record exceeds the byte budget — "
                    "raise VANTADB_MCP_BYTE_BUDGET"
                )
            limit = max(limit // 2, MIN_PAGE_LIMIT)
            continue
        records.extend(payload.get("records", []))
        cursor = payload.get("next_cursor")
        if cursor is None:
            return records
    raise RuntimeError(f"pagination cap ({MAX_PAGES} pages) hit for {namespace!r}")


def _fetch_page(cli: str, db: str, namespace: str, limit: int, cursor) -> dict:
    args = {"namespace": namespace, "limit": limit}
    if cursor is not None:
        args["cursor"] = cursor
    proc = subprocess.run(
        [cli, "mcp-call", "--db", db, "--tool", "memory_list", "--args", json.dumps(args)],
        capture_output=True,
        text=True,
    )
    if proc.returncode != 0:
        raise RuntimeError(f"mcp-call failed ({proc.returncode}): {proc.stderr.strip()}")
    result = json.loads(proc.stdout)
    return json.loads(result["content"][0]["text"])


def mcp_call(cli: str, db: str, namespace: str) -> list[dict]:
    """All records of one namespace via the one-shot MCP caller."""
    return paginate(lambda ns, limit, cursor: _fetch_page(cli, db, ns, limit, cursor), namespace)


def key_ms(key: str) -> int | None:
    """``{ms}-{seq}`` → ms (same key shape as capture.rs / search events)."""
    head = key.split("-", 1)[0]
    return int(head) if head.isdigit() else None


def sessions_in_window(records: list[dict], cutoff_ms: int, kind: str) -> set[str]:
    """Sessions with an event of ``kind`` (``search`` requires ``hits >= 1``)
    whose key timestamp falls inside ``[cutoff_ms, now]``."""
    out: set[str] = set()
    for rec in records:
        ms = key_ms(str(rec.get("key", "")))
        if ms is None or ms < cutoff_ms:
            continue
        try:
            payload = json.loads(rec.get("payload") or "{}")
        except (json.JSONDecodeError, TypeError):
            continue
        if payload.get("kind", "turn") != kind:
            continue
        if kind == "search" and int(payload.get("hits") or 0) < 1:
            continue
        session = payload.get("session")
        if session:
            out.add(str(session))
    return out


def north_star(
    cli: str, db: str, days: int = 7
) -> tuple[list[str], set[str], set[str]]:
    cutoff_ms = int(time.time() * 1000) - days * 86_400_000
    turns = mcp_call(cli, db, TURNS_NAMESPACE)
    events = mcp_call(cli, db, EVENTS_NAMESPACE)
    # Turns carry no `kind` field (capture.rs payload) → default "turn".
    put_sessions = sessions_in_window(turns, cutoff_ms, "turn")
    search_sessions = sessions_in_window(events, cutoff_ms, "search")
    return sorted(put_sessions & search_sessions), put_sessions, search_sessions


def _self_test() -> int:
    """Runnable check of the pure windowing logic (no CLI, no store)."""
    now = int(time.time() * 1000)
    records = [
        {"key": f"{now - 1000}-0", "payload": json.dumps({"session": "s1"})},
        {"key": f"{now - 8 * 86_400_000}-1", "payload": json.dumps({"session": "old"})},
        {"key": "not-a-ms-key", "payload": json.dumps({"session": "junk"})},
        {"key": f"{now - 2000}-2", "payload": "not json"},
    ]
    events = [
        {"key": f"{now - 500}-0", "payload": json.dumps({"session": "s1", "kind": "search", "hits": 3})},
        {"key": f"{now - 400}-1", "payload": json.dumps({"session": "s2", "kind": "search", "hits": 0})},
        {"key": f"{now - 300}-2", "payload": json.dumps({"session": "s1", "kind": "search", "hits": 1})},
        {"key": f"{now - 9 * 86_400_000}-3", "payload": json.dumps({"session": "s1", "kind": "search", "hits": 5})},
    ]
    cutoff = now - 7 * 86_400_000
    put = sessions_in_window(records, cutoff, "turn")
    search = sessions_in_window(events, cutoff, "search")
    assert put == {"s1"}, f"put windowing broken: {put}"
    assert search == {"s1"}, f"search windowing broken (hits>=1 + 7d only): {search}"
    assert sorted(put & search) == ["s1"]
    assert key_ms("1700000000000-4") == 1700000000000
    assert key_ms("junk") is None

    # R1 (review): a byte-budget-truncated page must NOT undercount — the
    # client shrinks the page until it fits whole, then advances. The fake
    # mirrors the server: any window >20 records comes back truncated with
    # the cursor pointing past the FULL requested window (the trap).
    total = 61

    def fake_fetch(namespace, limit, cursor):
        start = cursor or 0
        window = list(range(start, min(start + limit, total)))
        emitted = [
            {"key": f"{now}-{i:03d}", "payload": json.dumps({"session": f"s{i % 3}"})}
            for i in window[:20]
        ] if len(window) > 20 else [
            {"key": f"{now}-{i:03d}", "payload": json.dumps({"session": f"s{i % 3}"})}
            for i in window
        ]
        next_cursor = None if start + limit >= total else start + limit
        return {
            "records": emitted,
            "next_cursor": next_cursor,
            "truncated": len(window) > 20,
        }

    got = paginate(fake_fetch, "fake")
    assert len(got) == total, f"truncated pages undercount: {len(got)} != {total}"
    print("north_star_metric self-test: OK")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--db", help="proxy store path ([auth] db_path)")
    parser.add_argument("--days", type=int, default=7, help="window in days (default 7)")
    parser.add_argument("--json", action="store_true", help="machine-readable output")
    parser.add_argument("--cli", default="vanta-cli", help="CLI binary (default vanta-cli)")
    parser.add_argument("--self-test", action="store_true", help="run the built-in check and exit")
    args = parser.parse_args()

    if args.self_test:
        return _self_test()
    if not args.db:
        parser.error("--db is required (or use --self-test)")

    try:
        both, put_sessions, search_sessions = north_star(args.cli, args.db, args.days)
    except (RuntimeError, OSError, json.JSONDecodeError, KeyError, IndexError) as exc:
        print(f"north-star: {exc}", file=sys.stderr)
        return 1

    if args.json:
        print(
            json.dumps(
                {
                    "window_days": args.days,
                    "north_star_sessions": len(both),
                    "sessions": both,
                    "put_sessions": len(put_sessions),
                    "search_sessions": len(search_sessions),
                },
                indent=2,
            )
        )
    else:
        print(f"North Star - sessions with put+search in the last {args.days}d: {len(both)}")
        print(f"  put sessions:    {len(put_sessions)}")
        print(f"  search sessions: {len(search_sessions)}")
        for session in both:
            print(f"  - {session}")
    return 0


if __name__ == "__main__":
    sys.exit(main())

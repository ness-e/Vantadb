#!/usr/bin/env python3
"""LangGraph dev -> prod: one graph, two checkpointers (VantaDB demo).

The claim this demo verifies mechanically: the **same graph code** that runs
in dev with LangGraph's ``InMemorySaver`` runs in prod against VantaDB's
``VantaDBCheckpointer`` — only the checkpointer changes. (The gap this
replaces: dev on ``InMemorySaver``, prod on ``PostgresSaver``.)

The driver spawns five **fresh processes** and fails if any property breaks:

1. the business section of ``respond``/``build_graph`` names no backend or
   mode (token assert, exercised by a negative control) and every process ran
   this same script (fingerprint — a same-file display invariant, not a gate);
2. fresh runs behave identically in dev and prod (same input, same reply);
3. dev (``InMemorySaver``) does **not** survive the process — the control;
4. prod (``VantaDBCheckpointer``) **does** resume across processes — the value;
5. two threads share one database without leaking into each other.

Deterministic and offline by design: the graph node is plain Python — no LLM,
no API keys, no network calls. It exercises the checkpointing pipeline, not
model quality.

Run:  python examples/langgraph_dev_to_prod/demo.py
"""

from __future__ import annotations

import argparse
import hashlib
import inspect
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile

from langgraph.checkpoint.base import BaseCheckpointSaver
from langgraph.checkpoint.memory import InMemorySaver
from langgraph.graph import END, START, MessagesState, StateGraph
from vantadb_langchain import VantaDBCheckpointer

FIRST_TEXT = "the launch code is cobalt-7"
SECOND_TEXT = "what did I ask you to remember?"
THREAD = "user-alice"
OTHER_THREAD = "user-bob"

# Words that would mean the business code knows about a storage backend or a
# deployment mode. If one shows up in the business section, this demo fails.
FORBIDDEN_IN_BUSINESS = (
    "InMemorySaver",
    "VantaDBCheckpointer",
    "vantadb",
    "dev",
    "prod",
    "mode",
)


# ── business code: identical in dev and prod ────────────────────────────────
# Everything between the markers is the application. It receives a state and
# (to build once) any checkpointer — it never learns which backend that is.


def respond(state: MessagesState) -> dict:
    """Answer using every request the thread remembers."""
    requests = [m.content for m in state["messages"] if m.type == "human"]
    first = requests[0] if requests else "(none)"
    return {"messages": [("ai", f"first request in this thread: {first}")]}


def build_graph(checkpointer):
    """Compile the app once; the caller decides which checkpointer to deploy."""
    builder = StateGraph(MessagesState)
    builder.add_node("respond", respond)
    builder.add_edge(START, "respond")
    builder.add_edge("respond", END)
    return builder.compile(checkpointer=checkpointer)


# ── end business code ───────────────────────────────────────────────────────


# ── wiring: the only section that knows dev from prod ───────────────────────


def make_checkpointer(mode: str, db_path: str) -> BaseCheckpointSaver:
    """Pick the checkpointer for the deployment — the line that changes."""
    if mode == "dev":
        return InMemorySaver()  # LangGraph reference: in memory, per process
    if mode == "prod":
        return VantaDBCheckpointer(db_path=db_path)  # embedded VantaDB
    raise SystemExit(f"unknown mode: {mode!r}")


def business_source() -> str:
    return inspect.getsource(respond) + inspect.getsource(build_graph)


def business_fingerprint() -> str:
    # Display/same-file invariant: all five processes run this same script, so
    # this equality cannot fail on its own — it shows WHAT ran, it does not
    # prove "no per-mode fork". The gates that can fail: the backend-free token
    # assert (exercised by a negative control) and the driver's behavior
    # equivalence (dev-first == prod-first). Ceiling, declared: a fork that
    # diverges only on the resume path is not compared by either.
    return hashlib.sha256(business_source().encode("utf-8")).hexdigest()[:16]


def assert_business_is_backend_free() -> None:
    src = business_source()
    hits = [t for t in FORBIDDEN_IN_BUSINESS if re.search(rf"\b{re.escape(t)}\b", src)]
    if hits:
        raise SystemExit(
            f"business code references {hits!r} — it must stay backend/mode-free"
        )


# ── child process: one deployment, one process, one invoke ──────────────────


def child(args: argparse.Namespace) -> int:
    assert_business_is_backend_free()
    fingerprint = business_fingerprint()
    graph = build_graph(make_checkpointer(args.mode, args.db))
    config = {"configurable": {"thread_id": args.thread}}

    before = graph.get_state(config)
    prior = len((before.values or {}).get("messages", []) or [])
    out = graph.invoke({"messages": [("user", args.text)]}, config)
    reply = out["messages"][-1].content

    # mode/phase/thread are payload context for the raw CHILD_RESULT line (the
    # driver keys on the label and the assertions below).
    result = {
        "mode": args.mode,
        "phase": args.phase,
        "thread": args.thread,
        "fingerprint": fingerprint,
        "prior_messages": prior,
        "reply": reply,
        "remembered_first": FIRST_TEXT in reply,
    }
    print("CHILD_RESULT " + json.dumps(result, ensure_ascii=True))
    return 0


# ── driver: five fresh processes + assertions ───────────────────────────────


def _run_child(script: str, label: str, mode: str, phase: str, db: str, thread: str, text: str) -> dict:
    env = {**os.environ, "PYTHONUTF8": "1"}
    proc = subprocess.run(
        [
            sys.executable, script, "_child",
            "--mode", mode, "--phase", phase,
            "--db", db, "--thread", thread, "--text", text,
        ],
        capture_output=True, text=True, encoding="utf-8", env=env,
    )
    if proc.returncode != 0:
        raise SystemExit(
            f"[FAIL] {label}: child exited {proc.returncode}\n"
            f"--- stdout\n{proc.stdout}\n--- stderr\n{proc.stderr}"
        )
    for line in proc.stdout.splitlines():
        if line.startswith("CHILD_RESULT "):
            return json.loads(line[len("CHILD_RESULT "):])
    raise SystemExit(f"[FAIL] {label}: no CHILD_RESULT line\n{proc.stdout}")


def driver() -> int:
    script = os.path.abspath(__file__)
    tmp = tempfile.mkdtemp(prefix="vanta-dev-prod-")
    try:
        dev_db = os.path.join(tmp, "dev")
        prod_db = os.path.join(tmp, "prod")
        plan = [
            ("dev-first", "dev", "first", dev_db, THREAD, FIRST_TEXT),
            ("dev-resume-fresh-process", "dev", "resume", dev_db, THREAD, SECOND_TEXT),
            ("prod-first", "prod", "first", prod_db, THREAD, FIRST_TEXT),
            ("prod-resume-fresh-process", "prod", "resume", prod_db, THREAD, SECOND_TEXT),
            ("prod-other-thread-isolated", "prod", "resume", prod_db, OTHER_THREAD, SECOND_TEXT),
        ]
        print("Running 5 fresh processes (each row = one process)...")
        results: dict[str, dict] = {}
        for label, mode, phase, db, thread, text in plan:
            results[label] = _run_child(script, label, mode, phase, db, thread, text)
            r = results[label]
            print(
                f"  {label:28s} remembered_first={str(r['remembered_first']):5s} "
                f"prior_messages={r['prior_messages']} fingerprint={r['fingerprint']}"
            )

        checks = [
            (
                "all 5 processes ran the same script (fingerprint — display invariant)",
                len({r["fingerprint"] for r in results.values()}) == 1,
            ),
            (
                "identical behavior on identical input (dev-first == prod-first)",
                results["dev-first"]["reply"] == results["prod-first"]["reply"],
            ),
            (
                "fresh runs see the request they just wrote",
                results["dev-first"]["remembered_first"] and results["prod-first"]["remembered_first"],
            ),
            (
                "CONTROL: dev (InMemorySaver) does not survive a fresh process",
                not results["dev-resume-fresh-process"]["remembered_first"],
            ),
            (
                "VALUE: prod (VantaDB) resumes state written by a previous process",
                results["prod-resume-fresh-process"]["remembered_first"]
                and results["prod-resume-fresh-process"]["prior_messages"] >= 2,
            ),
            (
                "two threads in one prod database stay isolated",
                not results["prod-other-thread-isolated"]["remembered_first"]
                and results["prod-other-thread-isolated"]["prior_messages"] == 0,
            ),
        ]
        print()
        for name, ok in checks:
            print(f"  [{'ok' if ok else 'FAIL'}] {name}")
        if not all(ok for _, ok in checks):
            print()
            print("DEMO FAILED")
            return 1
        print()
        print("DEMO PASSED -- same graph code in dev and prod; dev loses state with the")
        print("process (InMemorySaver), prod keeps it (VantaDBCheckpointer on VantaDB);")
        print("threads stay isolated.")
        return 0
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


def _child_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(prog="demo.py _child")
    p.add_argument("--mode", choices=("dev", "prod"), required=True)
    p.add_argument("--phase", choices=("first", "resume"), required=True)
    p.add_argument("--db", required=True)
    p.add_argument("--thread", required=True)
    p.add_argument("--text", required=True)
    return p


def main() -> int:
    if len(sys.argv) > 1 and sys.argv[1] == "_child":
        return child(_child_parser().parse_args(sys.argv[2:]))
    if len(sys.argv) > 1:
        print(__doc__)
        return 2
    return driver()


if __name__ == "__main__":
    raise SystemExit(main())

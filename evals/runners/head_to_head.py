#!/usr/bin/env python3
"""evals/runners/head_to_head.py — VER-09 head-to-head driver.

Runs external memory systems through the SAME VER-08 harness metrics by
delegating to `evals/memory_harness.run_harness` with a runner-specific store
(see `evals/runners/adapters.py`). Zero forks of the harness: this driver only
adds the multi-system layer + report plumbing.

Commands (Regla 11):
    python evals/runners/head_to_head.py --check                 # availability per system (no run)
    python evals/runners/head_to_head.py --self-test             # offline checks (fake SDKs, no network)
    python evals/runners/head_to_head.py --systems vantadb,mem0 --label h2h-5q
    python evals/runners/head_to_head.py --systems vantadb,mem0 \
        --data datasets/longmemeval/longmemeval_s_cleaned.json --label h2h-full500

Reports: `evals/runners/report_h2h_<label>_<system>.json` (gitignored — regenerate
with the commands above; BENCHMARKS.md §19 is the committed record of the runs).
"""
from __future__ import annotations

import argparse
import json
import os
import pathlib
import platform
import sys
import types

HERE = pathlib.Path(__file__).parent
EVALS = HERE.parent
sys.path.insert(0, str(EVALS))  # memory_harness lives in evals/
sys.path.insert(0, str(HERE))  # adapters lives here

from memory_harness import (  # noqa: E402
    _load_questions,
    _total_ram_gb,
    question_is_abs,
    run_harness,
    sha256_file,
)
import adapters  # noqa: E402
from adapters import (  # noqa: E402
    PINNED,
    SYSTEMS,
    RunnerSkipped,
    build_store,
    check_availability,
    chunk_text,
    utc_now,
)

DEFAULT_DATA = EVALS / "data" / "longmemeval_s_subset.json"


# ---------------------------------------------------------------------- helpers
def _hardware() -> dict:
    return {
        "os": platform.platform(),
        "cpu": platform.processor() or platform.machine(),
        "cpu_count": {"logical": os.cpu_count()},
        "ram_gb": _total_ram_gb(),
        "python": platform.python_version(),
    }


def _fmt(value) -> str:
    return f"{value:.4f}" if isinstance(value, (int, float)) else "n/a"


def _md_row(report: dict) -> str:
    m, s, r = report["meta"], report["search"], report["recall_at_k"]
    wq = report["write_quality"]
    cal = report.get("calibration")
    ece = "N/A (external)" if cal is None else f"{cal['ece_before']:.4f}→{cal['ece_after']:.4f}"
    return (
        f"| {m['system']} | {m['questions']} | {_fmt(r['all'])} | {_fmt(r['any'])} "
        f"| {s['p50_ms']} | {s['p99_ms']} | {report['ingest']['qps']} | {_fmt(wq['value'])} "
        f"| {report['token_economy']['tokens_p50']} | {ece} |"
    )


TABLE_HEADER = (
    "| system | n | recall_all@5 | recall_any@5 | q p50 (ms) | q p99 (ms) "
    "| ingest QPS | write-quality | tokens p50 | ECE |\n"
    "|---|---|---|---|---|---|---|---|---|---|"
)


def _calibration_note(report: dict) -> None:
    """External scores are not calibrated probabilities — ECE only for vantadb."""
    if report["meta"]["system"] != "vantadb" and report.get("calibration") is not None:
        report["calibration_note"] = (
            "ECE is reported for vantadb only; external scores are retrieval scores, "
            "not calibrated probabilities (protocol §Limits)."
        )
        report["calibration"] = None


def _stratified_sample(questions: list, per_type: int) -> list:
    """First `per_type` questions of each question_type, dataset order preserved.

    LongMemEval's file is roughly ordered in type blocks; a plain `--limit N`
    slice would cover one type only (accidental cherry-picking). Stratifying by
    type keeps the published comparison honest across capabilities.
    """
    seen: dict[str, int] = {}
    keep: set[int] = set()
    for index, question in enumerate(questions):
        qtype = question.get("question_type", "?")
        if seen.get(qtype, 0) < per_type:
            seen[qtype] = seen.get(qtype, 0) + 1
            keep.add(index)
    return [q for i, q in enumerate(questions) if i in keep]


# ------------------------------------------------------------------- run mode
def run_systems(args) -> int:
    data_path = pathlib.Path(args.data)
    questions = _load_questions(data_path)
    if args.only == "nonabs":
        questions = [q for q in questions if not question_is_abs(q)]
    elif args.only == "abs":
        questions = [q for q in questions if question_is_abs(q)]
    if args.sample_stratified:
        questions = _stratified_sample(questions, args.sample_stratified)
    elif args.limit:
        questions = questions[: args.limit]
    if not questions:
        print(f"no questions selected from {data_path}", file=sys.stderr)
        return 2
    data_sha = sha256_file(data_path)
    systems = [s.strip() for s in args.systems.split(",") if s.strip()]
    unknown = [s for s in systems if s not in SYSTEMS]
    if unknown:
        print(f"unknown systems: {', '.join(unknown)} (known: {', '.join(SYSTEMS)})", file=sys.stderr)
        return 2

    skipped: list[dict] = []
    rows: list[dict] = []
    for system in systems:
        try:
            store = build_store(system, mem0_mode=args.mem0_mode)
        except RunnerSkipped as exc:
            skipped.append({"system": system, "reason": exc.reason})
            print(f"SKIP  {exc}", file=sys.stderr)
            continue
        started = utc_now()
        try:
            report = run_harness(
                store,
                questions,
                top_k=args.top_k,
                namespace_prefix=args.namespace_prefix,
                measure_write_quality=getattr(store, "supports_write_quality", True),
            )
        finally:
            store.close()
        label = args.label or f"{data_path.stem}-{len(questions)}q"
        report["meta"].update(
            {
                "label": label,
                "system": system,
                "system_version": getattr(store, "version", "?"),
                "harness": "evals/memory_harness.py (VER-08 — reused, not forked)",
                "data_file": str(data_path),
                "data_sha256": data_sha,
                "started_at": started,
                "finished_at": utc_now(),
            }
        )
        report["hardware"] = _hardware()
        report["system_config"] = (
            store.config_summary() if hasattr(store, "config_summary") else {"system": system}
        )
        _calibration_note(report)
        out = HERE / f"report_h2h_{label}_{system}.json"
        out.write_text(json.dumps(report, indent=2), encoding="utf-8")
        print(_md_row(report))
        print(f"  JSON: {out}", file=sys.stderr)
        rows.append(report)

    if rows:
        print(f"\n### head-to-head — label `{args.label or 'auto'}` (source: evals/runners/head_to_head.py)")
        print(TABLE_HEADER)
        for report in rows:
            print(_md_row(report))
        print("\n> External-system configs are declared per system (see report JSON / runners README).")
    if skipped:
        print("\nSkipped (documented):", file=sys.stderr)
        for entry in skipped:
            print(f"  - {entry['system']}: {entry['reason']}", file=sys.stderr)
    return 0 if rows else 1


# ----------------------------------------------------------------- check mode
def cmd_check(args) -> int:
    print("| system | status | version | reason |")
    print("|---|---|---|---|")
    bad = 0
    for entry in check_availability(mem0_mode=args.mem0_mode):
        print(f"| {entry['system']} | {entry['status']} | {entry['version'] or '-'} | {entry['reason'] or '-'} |")
        if entry["status"] == "skip":
            bad += 1
    print(f"\npins: {json.dumps(PINNED, ensure_ascii=False)}")
    return 1 if (args.strict and bad) else 0


# ------------------------------------------------------------------ self-test
def _selftest() -> int:
    """Offline: fake SDK modules injected into sys.modules; no network, no vantadb."""
    import tempfile

    # -- fake mem0 SDK (mirrors the pinned 2.2.1 surface used by the adapter) --
    class _FakeMem0:
        def __init__(self, config):
            self.config = config
            self.records: dict[str, dict] = {}
            self.closed = False

        @classmethod
        def from_config(cls, config):
            _FakeMem0.last_config = config
            return cls(config)

        def add(self, messages, *, user_id=None, metadata=None, infer=True, **kwargs):
            assert not kwargs, kwargs
            assert infer is False, "adapter must call raw mode with infer=False"
            results = []
            for message in messages:
                rid = f"rec{len(self.records)}"
                self.records[rid] = {
                    "id": rid,
                    "memory": message["content"],
                    "metadata": dict(metadata or {}),
                    "user_id": user_id,
                    "score": None,
                }
                results.append(dict(self.records[rid]))
            return {"results": results}

        def search(self, query, *, top_k=20, filters=None, threshold=0.1, **kwargs):
            assert not kwargs, kwargs  # strict: unknown kwargs (e.g. 'limit') must fail
            self.last_search = {"query": query, "top_k": top_k, "filters": filters}
            uid = (filters or {}).get("user_id")
            items = []
            for record in self.records.values():
                if record["user_id"] != uid:
                    continue
                hit = dict(record)
                hit["score"] = 1.0 if query in record["memory"] else 0.1
                items.append(hit)
            items.sort(key=lambda x: -x["score"])
            return {"results": items[:top_k]}

        def get_all(self, *, filters=None, top_k=20, show_expired=False, **kwargs):
            assert not kwargs, kwargs
            self.last_get_all = {"filters": filters, "top_k": top_k}
            uid = (filters or {}).get("user_id")
            return {"results": [r for r in self.records.values() if r["user_id"] == uid][:top_k]}

        def close(self):
            self.closed = True

    fake_mem0 = types.ModuleType("mem0")
    fake_mem0.Memory = _FakeMem0
    sys.modules["mem0"] = fake_mem0

    # -- fake zep SDK ---------------------------------------------------------
    class _FakeEpisode:
        def __init__(self, content, metadata, score=0.42):
            self.uuid_ = "ep-uuid"  # zep_cloud Episode exposes `uuid_`, not `uuid`
            self.content = content
            self.metadata = metadata
            self.score = score

    class _FakeGraph:
        def __init__(self, outer):
            self.outer = outer

        def search(self, user_id=None, query=None, scope="episodes", limit=10):
            episodes = [
                _FakeEpisode(m.content, m.metadata, score=0.42)
                for chunk in self.outer.threads.get(user_id, [])
                for m in chunk["messages"]
            ]
            return types.SimpleNamespace(episodes=episodes[:limit])

    class _FakeThread:
        def __init__(self, outer):
            self.outer = outer

        def create(self, thread_id=None, user_id=None):
            self.outer.threads.setdefault(thread_id, [])
            return {"thread_id": thread_id}

        def add_messages(self, thread_id=None, messages=None):
            self.outer.threads.setdefault(thread_id, []).append({"messages": messages})
            return {"message_uuids": ["m1"]}

    class _FakeZep:
        def __init__(self, api_key=None):
            self.api_key = api_key
            self.threads: dict = {}
            self.thread = _FakeThread(self)
            self.graph = _FakeGraph(self)

    class _FakeMessage:
        def __init__(self, name=None, role=None, content=None, metadata=None):
            self.name = name
            self.role = role
            self.content = content
            self.metadata = metadata

    zep_client_mod = types.ModuleType("zep_cloud.client")
    zep_client_mod.Zep = _FakeZep
    zep_types_mod = types.ModuleType("zep_cloud.types")
    zep_types_mod.Message = _FakeMessage
    zep_pkg_mod = types.ModuleType("zep_cloud")
    sys.modules["zep_cloud"] = zep_pkg_mod
    sys.modules["zep_cloud.client"] = zep_client_mod
    sys.modules["zep_cloud.types"] = zep_types_mod

    # 1. chunk_text: lossless + respect limit
    text = "".join(f"line {i}\n" for i in range(600))
    chunks = chunk_text(text, 4000)
    assert all(len(c) <= 4000 for c in chunks), [len(c) for c in chunks]
    assert "".join(chunks) == text, "chunking must be lossless"
    assert chunk_text("short", 4000) == ["short"]

    # 2. mem0 raw store mapping (add/get/search through the fake SDK)
    with tempfile.TemporaryDirectory() as tmp:
        store = adapters.Mem0Store(mode="raw", data_dir=tmp)
        store.put("ns", "s1", "hello world evidence")
        rec = store.get("ns", "s1")
        assert rec is not None and rec["payload"] == "hello world evidence", rec
        assert store.memory.last_get_all["top_k"] >= 1000, store.memory.last_get_all
        hits = store.search("ns", "hello", 5)
        assert hits and hits[0]["key"] == "s1" and hits[0]["score"] == 1.0, hits
        assert store.memory.last_search["top_k"] == 5, store.memory.last_search
        assert store.get("ns", "missing") is None
        store.close()
        assert store.memory.closed is True, "close() must close the underlying client"
        # config checks: raw mode = fastembed + embedded qdrant + infer=False
        cfg = _FakeMem0.last_config
        assert cfg["embedder"]["provider"] == "fastembed"
        assert cfg["vector_store"]["config"]["embedding_model_dims"] == 384

    # 3. mem0 native without key → documented skip
    os.environ.pop("OPENAI_API_KEY", None)
    try:
        adapters.Mem0Store(mode="native")
        raise AssertionError("native without OPENAI_API_KEY must skip")
    except RunnerSkipped as exc:
        assert "OPENAI_API_KEY" in exc.reason

    # 4. mem0 missing SDK → documented skip (patch the import point)
    original_import = adapters._import
    adapters._import = lambda name: (_ for _ in ()).throw(ImportError(name))
    try:
        adapters.Mem0Store(mode="raw")
        raise AssertionError("missing mem0 SDK must skip")
    except RunnerSkipped as exc:
        assert "pip install" in exc.reason
    adapters._import = original_import

    # 5. zep: skip without key; mapping with key + fake SDK
    os.environ.pop("ZEP_API_KEY", None)
    try:
        adapters.ZepStore()
        raise AssertionError("zep without ZEP_API_KEY must skip")
    except RunnerSkipped as exc:
        assert "ZEP_API_KEY" in exc.reason
    os.environ["ZEP_API_KEY"] = "zep-test-key"
    try:
        zstore = adapters.ZepStore()
        long_text = "x" * 9000
        zstore.put("ns", "s9", long_text)
        sent = [m for chunk in zstore.client.threads["ns"] for m in chunk["messages"]]
        assert len(sent) == 3, len(sent)  # 9000 chars → 3 chunks ≤4000
        assert "".join(m.content for m in sent) == long_text, "zep chunking lossless"
        assert all(m.metadata["session_id"] == "s9" for m in sent)
        zhits = zstore.search("ns", "any query", 5)
        # list-valued metadata (projection) must resolve to its first element
        assert zhits and zhits[0]["key"] == "s9", zhits
        assert zstore.get("ns", "s9") is None  # write-quality declared N/A
        # uuid_ fallback: an episode without session metadata must fall back to
        # the REAL field name (zep_cloud Episode exposes `uuid_`, not `uuid`).
        zstore.client.threads["ns-nometa"] = [
            {"messages": [_FakeMessage(name="s", role="user", content="no meta", metadata={})]}
        ]
        zhits2 = zstore.search("ns-nometa", "q", 5)
        assert zhits2 and zhits2[0]["key"] == "ep-uuid", zhits2
        zstore.close()
    finally:
        os.environ.pop("ZEP_API_KEY", None)

    # 6. letta: architectural skip (documented)
    try:
        adapters.LettaStore()
        raise AssertionError("letta must skip")
    except RunnerSkipped as exc:
        assert "agent harness" in exc.reason

    # 7. availability table reflects the environment (fastembed presence is
    #    simulated so this offline selftest does not depend on the local env)
    original_has_module = adapters._has_module
    adapters._has_module = lambda name: True
    try:
        table = {e["system"]: e for e in check_availability(mem0_mode="raw")}
        assert table["mem0"]["status"] == "ok", table["mem0"]
        assert table["zep"]["status"] == "skip" and "ZEP_API_KEY" in table["zep"]["reason"]
        assert table["letta"]["status"] == "skip"
        adapters._has_module = lambda name: name != "fastembed"
        table2 = {e["system"]: e for e in check_availability(mem0_mode="raw")}
        assert table2["mem0"]["status"] == "skip" and "fastembed" in table2["mem0"]["reason"], table2["mem0"]
    finally:
        adapters._has_module = original_has_module

    # 8. end-to-end through the REAL harness runner with the fake mem0 store
    question = {
        "question_id": "q1",
        "question_type": "single-session-user",
        "question": "hello",
        "answer_session_ids": ["s1"],
        "haystack_session_ids": ["s1", "s2"],
        "haystack_sessions": [
            [{"role": "user", "content": "hello evidence"}],
            [{"role": "user", "content": "unrelated filler"}],
        ],
    }
    with tempfile.TemporaryDirectory() as tmp:
        store = adapters.Mem0Store(mode="raw", data_dir=tmp)
        rep = run_harness(store, [question], top_k=2, namespace_prefix="st")
        store.close()
        assert rep["recall_at_k"]["all"] == 1.0, rep["recall_at_k"]
        assert rep["recall_at_k"]["any"] == 1.0
        assert rep["write_quality"]["value"] == 1.0, rep["write_quality"]
        assert rep["isolation"]["violations"] == 0, rep["isolation"]
        assert rep["ingest"]["docs"] == 2

    # 9. unknown system raises
    try:
        build_store("nope")
        raise AssertionError("unknown system must raise")
    except ValueError:
        pass

    # 10. stratified sampling: first N per type, dataset order preserved
    sampled = _stratified_sample(
        [
            {"question_id": "a1", "question_type": "A"},
            {"question_id": "a2", "question_type": "A"},
            {"question_id": "b1", "question_type": "B"},
            {"question_id": "c1", "question_type": "C"},
        ],
        1,
    )
    assert [q["question_id"] for q in sampled] == ["a1", "b1", "c1"], sampled

    print("head_to_head selftest: OK (10 checks)")
    return 0


# ---------------------------------------------------------------------- main
def main() -> int:
    parser = argparse.ArgumentParser(description="VER-09 head-to-head driver (same harness, external runners).")
    parser.add_argument("--systems", default=",".join(SYSTEMS), help="comma list (default: all)")
    parser.add_argument("--mem0-mode", choices=["raw", "native"], default="raw")
    parser.add_argument("--data", default=str(DEFAULT_DATA))
    parser.add_argument("--limit", type=int, default=0, help="max questions (0 = all)")
    parser.add_argument(
        "--sample-stratified",
        type=int,
        default=0,
        metavar="N",
        help="keep the first N questions of EACH question_type (anti cherry-picking; "
        "LongMemEval is type-block ordered so a plain --limit slice covers one type only)",
    )
    parser.add_argument("--only", choices=["all", "nonabs", "abs"], default="all")
    parser.add_argument("--top-k", type=int, default=5)
    parser.add_argument("--namespace-prefix", default="h2h")
    parser.add_argument("--label", default="")
    parser.add_argument("--check", action="store_true", help="availability per system (no run)")
    parser.add_argument("--strict", action="store_true", help="--check exits 1 when any system skips")
    parser.add_argument("--self-test", action="store_true", help="offline checks (fake SDKs)")
    args = parser.parse_args()

    if args.self_test:
        return _selftest()
    if args.check:
        return cmd_check(args)
    return run_systems(args)


if __name__ == "__main__":
    raise SystemExit(main())

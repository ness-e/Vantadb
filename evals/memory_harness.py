#!/usr/bin/env python3
"""evals/memory_harness.py — VER-08 real-dataset memory harness (LongMemEval-S).

Runs the real LongMemEval-S schema (committed sub-sample in `evals/data/`, or
the full locally-downloaded split) against the real `vantadb` Python bindings
and reports: recall_all@k + recall_any@k (session-level), p50/p95/p99 query
latency, ingest QPS, write-quality, abstention proxy, token-economy, per-user
namespace isolation, and the MGR-12 §5.2 calibration pairs (ECE + temperature
scaling live in `evals/calibration.py`).

Protocol (pinned — see docs/user/operations/BENCHMARKS.md §19):
  * Judge: DETERMINISTIC (no LLM). Correctness proxy = session-level evidence
    retrieval, reported in BOTH forms:
      - `recall_all@k` = ALL `answer_session_ids` present in the top-k hits —
        the form LongMemEval's own retrieval eval reports (upstream-comparable).
      - `recall_any@k` = at least one `answer_session_ids` session in the top-k
        (hit-rate; secondary, kept for continuity).
    The QA-level LLM judge (GPT-4o, LongMemEval's `evaluate_qa.py`) is
    deliberately NOT used here (owner Q1: manual report only, never the gate).
  * Calibration pairs carry `correct` = recall_any-style question-level
    correctness (any-evidence) — declared, not recomputed per record.
  * Determinism: dataset fixed (sha256 recorded), top_k fixed, backend fixed.
  * Hardware: recorded in the report (Regla 11).

Known proxies (declared limits — never present these as LLM-judged numbers):
  * token-economy   = whitespace word count of retrieved payloads (no tokenizer).
  * abstention      = score-threshold separation on `_abs` questions; there is
    no generated answer, so this is a retrieval-level proxy.
  * write-quality   = write→read fidelity: after ingest, every record must be
    retrievable by exact key (`memory.get`) with a byte-identical payload.
  * per-user isolation = every hit must belong to the searched namespace and to
    the searched question's session set (per-question namespaces = per-user).

Commands (Regla 11):
  python evals/memory_harness.py --label subset-5q
  python evals/memory_harness.py --data datasets/longmemeval/longmemeval_s_cleaned.json --limit 100 --label local-100q
  python evals/memory_harness.py --self-test     # offline, no vantadb, no network

# ponytail: single-file harness, stdlib only; reuses evals/calibration.py for ECE.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import pathlib
import platform
import statistics
import sys
import time

try:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")
except Exception:
    pass

# `calibration.py` lives next to this file; `python evals/memory_harness.py`
# puts evals/ on sys.path[0], and pytest/import-from-root also works.
try:
    from calibration import calibrate as calibration_report
except ImportError:  # package-style import (evals.memory_harness)
    from evals.calibration import calibrate as calibration_report  # type: ignore

HERE = pathlib.Path(__file__).parent
DEFAULT_DATA = HERE / "data" / "longmemeval_s_subset.json"
DEFAULT_OUTPUT = HERE / "memory_harness_report.json"
TOP_K = 5
ABSTENTION_TAU_PCTL = 0.05  # abstention proxy: 5th percentile of evidence scores


# ---------------------------------------------------------------- pure helpers
def percentile(vals: list, p: float) -> float:
    """Nearest-rank percentile (standard nearest-rank convention)."""
    if not vals:
        return 0.0
    ordered = sorted(vals)
    n = len(ordered)
    idx = max(0, min(math.ceil(p * n) - 1, n - 1))
    return ordered[idx]


def question_is_abs(q: dict) -> bool:
    return str(q.get("question_id", "")).endswith("_abs")


def session_text(session: list) -> str:
    """Flatten a LongMemEval session (list of {role, content}) to plain text."""
    return "\n".join(f"{t.get('role', '?')}: {t.get('content', '')}" for t in session)


def sha256_file(path: pathlib.Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def _total_ram_gb() -> float | None:
    """Best-effort physical RAM (Regla 11 hardware record)."""
    try:
        if platform.system() == "Windows":
            import ctypes

            class _MemStatus(ctypes.Structure):
                _fields_ = [
                    ("dwLength", ctypes.c_ulong),
                    ("dwMemoryLoad", ctypes.c_ulong),
                    ("ullTotalPhys", ctypes.c_ulonglong),
                    ("ullAvailPhys", ctypes.c_ulonglong),
                    ("ullTotalPageFile", ctypes.c_ulonglong),
                    ("ullAvailPageFile", ctypes.c_ulonglong),
                    ("ullTotalVirtual", ctypes.c_ulonglong),
                    ("ullAvailVirtual", ctypes.c_ulonglong),
                    ("ullAvailExtendedVirtual", ctypes.c_ulonglong),
                ]

            m = _MemStatus()
            m.dwLength = ctypes.sizeof(_MemStatus)
            ctypes.windll.kernel32.GlobalMemoryStatusEx(ctypes.byref(m))
            return round(m.ullTotalPhys / 1024**3, 2)
        page = os.sysconf("SC_PAGE_SIZE")  # type: ignore[name-defined]  # POSIX
        pages = os.sysconf("SC_PHYS_PAGES")  # type: ignore[name-defined]
        return round(page * pages / 1024**3, 2)
    except Exception:
        return None


# ------------------------------------------------------------------ adapters
class VantaStore:
    """Thin adapter over the real `vantadb` Python bindings."""

    name = "vantadb"

    def __init__(self, backend: str = "memory") -> None:
        import vantadb  # type: ignore

        self.db = vantadb.Client(":memory:", backend=backend)
        self.version = getattr(vantadb, "__version__", "?")

    def put(self, ns: str, key: str, text: str) -> None:
        self.db.put(ns, key, text)

    def get(self, ns: str, key: str) -> dict | None:
        rec = self.db.memory.get(ns, key)
        if rec is None:
            return None
        return {"payload": rec.payload, "namespace": rec.namespace, "confidence": float(rec.confidence)}

    def search(self, ns: str, query: str, top_k: int) -> list[dict]:
        hits = self.db.search(ns, [], text_query=query, top_k=top_k)
        return [
            {
                "key": h.key,
                "score": float(h.score),
                "confidence": float(h.confidence),
                "namespace": h.namespace,
                "payload": h.payload,
            }
            for h in hits
        ]

    def close(self) -> None:
        try:
            self.db.close()
        except Exception:
            pass


# ------------------------------------------------------------------- runner
def run_harness(
    store,
    questions: list[dict],
    top_k: int = TOP_K,
    namespace_prefix: str = "eval",
    measure_write_quality: bool = True,
) -> dict:
    """Core run: ingest per-question namespaces, search, compute metrics."""
    ingest_latencies_ms: list[float] = []
    query_latencies_ms: list[float] = []
    wq_checked = wq_recovered = 0
    isolation_checks = isolation_violations = 0
    hits_by_q: list[dict] = []
    session_ids_by_q: dict[str, set] = {}

    t_ingest0 = time.perf_counter()
    n_docs = 0
    for q in questions:
        qid = q["question_id"]
        ns = f"{namespace_prefix}/{qid}"
        ids = list(q.get("haystack_session_ids", []))
        sessions = list(q.get("haystack_sessions", []))
        session_ids_by_q[qid] = set(ids)
        for sid, sess in zip(ids, sessions):
            s = time.perf_counter()
            store.put(ns, sid, f"Session {sid}:\n{session_text(sess)}")
            ingest_latencies_ms.append((time.perf_counter() - s) * 1000.0)
            n_docs += 1
    ingest_s = time.perf_counter() - t_ingest0

    # write-quality: write→read fidelity — exact-key get must return the
    # byte-identical payload that was written.
    if measure_write_quality:
        for q in questions:
            qid = q["question_id"]
            ns = f"{namespace_prefix}/{qid}"
            ids = list(q.get("haystack_session_ids", []))
            sessions = list(q.get("haystack_sessions", []))
            for sid, sess in zip(ids, sessions):
                rec = store.get(ns, sid)
                wq_checked += 1
                if rec is not None and rec["payload"] == f"Session {sid}:\n{session_text(sess)}":
                    wq_recovered += 1

    # queries + isolation + calibration pairs
    calibration_pairs: list[list] = []
    for q in questions:
        qid = q["question_id"]
        ns = f"{namespace_prefix}/{qid}"
        s = time.perf_counter()
        hits = store.search(ns, q["question"], top_k)
        query_latencies_ms.append((time.perf_counter() - s) * 1000.0)
        expected_keys = session_ids_by_q[qid]
        for h in hits:
            isolation_checks += 1
            if h["namespace"] != ns or h["key"] not in expected_keys:
                isolation_violations += 1
        gold = set(q.get("answer_session_ids", []))
        top_keys = {h["key"] for h in hits}
        correct_any = bool(gold & top_keys)
        correct_all = bool(gold) and gold.issubset(top_keys)
        hits_by_q.append(
            {
                "qid": qid,
                "abs": question_is_abs(q),
                "correct_any": correct_any,
                "correct_all": correct_all,
                "hits": hits,
            }
        )
        if not question_is_abs(q) and hits:
            # Calibration pair correctness = recall_any-style (any-evidence),
            # question-level; declared in the docstring and BENCHMARKS §19.
            for h in hits:
                calibration_pairs.append([h["confidence"], 1 if correct_any else 0])

    nonabs = [r for r in hits_by_q if not r["abs"]]
    absq = [r for r in hits_by_q if r["abs"]]
    recall_any = (sum(1 for r in nonabs if r["correct_any"]) / len(nonabs)) if nonabs else None
    recall_all = (sum(1 for r in nonabs if r["correct_all"]) / len(nonabs)) if nonabs else None
    by_type_any: dict[str, list] = {}
    by_type_all: dict[str, list] = {}
    for q, r in zip([q for q in questions if not question_is_abs(q)], nonabs):
        t = q.get("question_type", "?")
        by_type_any.setdefault(t, []).append(1 if r["correct_any"] else 0)
        by_type_all.setdefault(t, []).append(1 if r["correct_all"] else 0)
    recall_any_by_type = {t: sum(v) / len(v) for t, v in sorted(by_type_any.items())}
    recall_all_by_type = {t: sum(v) / len(v) for t, v in sorted(by_type_all.items())}

    # abstention proxy (retrieval-level, no LLM): tau from evidence-score tail.
    evidence_scores = [
        max((h["score"] for h in r["hits"] if h["key"] in set(q["answer_session_ids"])), default=None)
        for r, q in zip(nonabs, [q for q in questions if not question_is_abs(q)])
    ]
    positive = [s for s in evidence_scores if s is not None]
    tau = percentile(positive, ABSTENTION_TAU_PCTL) if positive else None
    abs_top1 = [r["hits"][0]["score"] if r["hits"] else None for r in absq]
    abstention = None
    if tau is not None and absq:
        abstained = sum(
            1 for s in abs_top1 if s is None or s < tau
        )
        false_alarm = sum(
            1 for r in nonabs if (not r["hits"]) or r["hits"][0]["score"] < tau
        )
        abstention = {
            "abs_questions": len(absq),
            "threshold_tau": tau,
            "correct": abstained,
            "value": abstained / len(absq),
            "false_alarm_nonabs": false_alarm,
            "false_alarm_rate": (false_alarm / len(nonabs)) if nonabs else None,
            "proxy_note": "retrieval-level proxy (no generated answer); QA-level abstention belongs to the LLM-judge report",
        }

    # token economy: whitespace words of retrieved payloads (proxy, no tokenizer).
    tokens = [sum(len(h["payload"].split()) for h in r["hits"]) for r in hits_by_q]
    token_pairs = [
        {"question_id": r["qid"], "correct_any": r["correct_any"], "tokens": t}
        for r, t in zip(hits_by_q, tokens)
    ]

    cal = None
    if len(calibration_pairs) >= 2:
        cal = calibration_report(calibration_pairs)

    return {
        "meta": {
            "dataset": "longmemeval-schema",
            "questions": len(questions),
            "nonabs": len(nonabs),
            "abs": len(absq),
            "top_k": top_k,
            "namespace_prefix": namespace_prefix,
        },
        "ingest": {
            "docs": n_docs,
            "seconds": round(ingest_s, 3),
            "qps": round(n_docs / ingest_s, 1) if ingest_s else None,
            "p50_ms": round(statistics.median(ingest_latencies_ms), 4) if ingest_latencies_ms else None,
            "p99_ms": round(percentile(ingest_latencies_ms, 0.99), 4) if ingest_latencies_ms else None,
        },
        "search": {
            "queries": len(query_latencies_ms),
            "p50_ms": round(statistics.median(query_latencies_ms), 4) if query_latencies_ms else None,
            "p95_ms": round(percentile(query_latencies_ms, 0.95), 4) if query_latencies_ms else None,
            "p99_ms": round(percentile(query_latencies_ms, 0.99), 4) if query_latencies_ms else None,
            "latencies_ms": [round(x, 4) for x in query_latencies_ms],
        },
        "recall_at_k": {
            "k": top_k,
            "all": recall_all,
            "any": recall_any,
            "all_by_type": recall_all_by_type,
            "any_by_type": recall_any_by_type,
        },
        "write_quality": {
            "checked": wq_checked,
            "recovered_top1": wq_recovered,
            "value": (wq_recovered / wq_checked) if wq_checked else None,
        },
        "abstention": abstention,
        "token_economy": {
            "tokens_p50": round(statistics.median(tokens), 1) if tokens else None,
            "tokens_p99": round(percentile(tokens, 0.99), 1) if tokens else None,
            "tokens_total": sum(tokens),
            "pairs": token_pairs,
            "proxy_note": "whitespace word count (no tokenizer)",
        },
        "isolation": {"checks": isolation_checks, "violations": isolation_violations},
        "calibration": cal,
        "calibration_pairs": calibration_pairs,
        "per_question": [
            {
                "question_id": r["qid"],
                "abs": r["abs"],
                "correct_any": r["correct_any"],
                "correct_all": r["correct_all"],
                "top_keys": [h["key"] for h in r["hits"]],
                "top_scores": [h["score"] for h in r["hits"]],
            }
            for r in hits_by_q
        ],
    }


# ------------------------------------------------------------------ self-test
class FakeStore:
    """Deterministic store for --self-test (no vantadb, no network)."""

    def __init__(self) -> None:
        self.docs: dict[tuple, str] = {}

    def put(self, ns: str, key: str, text: str) -> None:
        self.docs[(ns, key)] = text

    def get(self, ns: str, key: str) -> dict | None:
        text = self.docs.get((ns, key))
        return None if text is None else {"payload": text, "namespace": ns, "confidence": 1.0}

    def search(self, ns: str, query: str, top_k: int) -> list[dict]:
        out = []
        for (dns, key), text in self.docs.items():
            if dns != ns:
                continue
            # Realistic-enough fake: a hit needs the query in the payload text
            # or to be an exact key lookup — NOT the payload's own "Session
            # <key>:" prefix (which would match every doc against any query).
            if query in text or query == key:
                out.append(
                    {
                        "key": key,
                        "score": 1.0 + text.count(query) * 0.1,
                        "confidence": 1.0,
                        "namespace": dns,
                        "payload": text,
                    }
                )
        out.sort(key=lambda h: -h["score"])
        return out[:top_k]


def _selftest() -> int:
    questions = [
        {
            "question_id": "q1",
            "question_type": "single-session-user",
            "question": "gold-one",
            "answer_session_ids": ["s1"],
            "haystack_session_ids": ["s1", "s2"],
            "haystack_sessions": [
                [{"role": "user", "content": "gold-one evidence"}],
                [{"role": "user", "content": "distractor"}],
            ],
        },
        {
            "question_id": "q3",
            "question_type": "multi-session",
            "question": "gold-one",
            "answer_session_ids": ["s1b", "s2b"],
            "haystack_session_ids": ["s1b", "s2b"],
            "haystack_sessions": [
                [{"role": "user", "content": "gold-one evidence v2"}],
                [{"role": "user", "content": "unrelated filler"}],
            ],
        },
        {
            "question_id": "q2_abs",
            "question_type": "single-session-user",
            "question": "no-such-event",
            "answer_session_ids": [],
            "haystack_session_ids": ["s3"],
            "haystack_sessions": [[{"role": "user", "content": "filler"}]],
        },
    ]
    rep = run_harness(FakeStore(), questions, top_k=2, namespace_prefix="st")
    assert rep["ingest"]["docs"] == 5, rep["ingest"]
    # q1 retrieves its single evidence session; q3 retrieves only 1 of its 2
    # gold sessions -> any = 1.0 but all = 0.5 (the two forms must separate).
    assert rep["recall_at_k"]["any"] == 1.0, rep["recall_at_k"]
    assert rep["recall_at_k"]["all"] == 0.5, rep["recall_at_k"]
    q3row = next(p for p in rep["per_question"] if p["question_id"] == "q3")
    assert q3row["correct_any"] is True and q3row["correct_all"] is False, q3row
    assert rep["write_quality"]["value"] == 1.0, rep["write_quality"]
    assert rep["isolation"]["violations"] == 0, rep["isolation"]
    assert rep["token_economy"]["tokens_total"] > 0
    assert rep["calibration"] is not None and rep["calibration"]["pairs"] == 2, rep["calibration"]
    assert rep["abstention"] is not None, rep["abstention"]
    # percentile = nearest-rank (p*n integer -> element p*n, 1-based).
    assert percentile([1, 2, 3, 4], 0.5) == 2, percentile([1, 2, 3, 4], 0.5)
    assert len(rep["search"]["latencies_ms"]) == 3, rep["search"]
    # negative probe: evidence missing -> both recall forms are 0 + isolation clean
    bad = [dict(questions[0], answer_session_ids=["nope"])]
    rep2 = run_harness(FakeStore(), bad, top_k=2)
    assert rep2["recall_at_k"]["any"] == 0.0, rep2["recall_at_k"]
    assert rep2["recall_at_k"]["all"] == 0.0, rep2["recall_at_k"]
    print("memory_harness selftest: OK (13 checks)")
    return 0


# ---------------------------------------------------------------------- main
def _load_questions(path: pathlib.Path) -> list[dict]:
    data = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(data, list) or not data:
        raise SystemExit(f"unexpected dataset shape in {path} (expected a JSON array of questions)")
    return data


def _markdown(report: dict) -> str:
    m, s, r, wq = report["meta"], report["search"], report["recall_at_k"], report["write_quality"]
    cal = report.get("calibration")

    def fmt(v) -> str:
        return f"{v:.4f}" if v is not None else "n/a"

    lines = [
        "| label | questions | top_k | recall_all@k | recall_any@k | write-quality | q p50 (ms) | q p99 (ms) | ingest QPS | ECE before → after |",
        "|---|---|---|---|---|---|---|---|---|---|",
        "| {label} | {n} ({non} non-abs + {ab} abs) | {k} | {rall} | {rany} | {wq} | {p50} | {p99} | {qps} | {ece} |".format(
            label=report["meta"].get("label", "?"),
            n=m["questions"],
            non=m["nonabs"],
            ab=m["abs"],
            k=r["k"],
            rall=fmt(r["all"]),
            rany=fmt(r["any"]),
            wq=fmt(wq["value"]),
            p50=s["p50_ms"],
            p99=s["p99_ms"],
            qps=report["ingest"]["qps"],
            ece=(
                f"{cal['ece_before']:.4f} → {cal['ece_after']:.4f} (T={cal['temperature']:.2f})"
                if cal
                else "n/a"
            ),
        ),
    ]
    return "\n".join(lines)


def main() -> int:
    ap = argparse.ArgumentParser(description="VER-08 memory harness (LongMemEval schema) against vantadb.")
    ap.add_argument("--data", default=str(DEFAULT_DATA), help="LongMemEval-schema JSON (subset or full split)")
    ap.add_argument("--limit", type=int, default=0, help="max questions to run (0 = all)")
    ap.add_argument("--only", choices=["all", "nonabs", "abs"], default="all", help="question filter")
    ap.add_argument("--top-k", type=int, default=TOP_K)
    ap.add_argument("--namespace-prefix", default="eval")
    ap.add_argument("--backend", default="memory", help="vantadb backend (memory default)")
    ap.add_argument("--label", default="", help="label recorded in the report (e.g. subset-5q)")
    ap.add_argument("--output", default=str(DEFAULT_OUTPUT))
    ap.add_argument("--no-write-quality", action="store_true", help="skip the per-record self-recovery sweep")
    ap.add_argument("--self-test", action="store_true", help="offline self-checks (no vantadb)")
    args = ap.parse_args()

    if args.self_test:
        return _selftest()

    data_path = pathlib.Path(args.data)
    questions = _load_questions(data_path)
    if args.only == "nonabs":
        questions = [q for q in questions if not question_is_abs(q)]
    elif args.only == "abs":
        questions = [q for q in questions if question_is_abs(q)]
    if args.limit:
        questions = questions[: args.limit]
    if not questions:
        raise SystemExit(f"no questions selected from {data_path} (filter={args.only}, limit={args.limit})")

    store = VantaStore(backend=args.backend)
    started = time.strftime("%Y-%m-%dT%H:%M:%S%z")
    report = run_harness(
        store,
        questions,
        top_k=args.top_k,
        namespace_prefix=args.namespace_prefix,
        measure_write_quality=not args.no_write_quality,
    )
    store.close()

    report["meta"].update(
        {
            "label": args.label or f"{data_path.stem}-{len(questions)}q",
            "data_file": str(data_path),
            "data_sha256": sha256_file(data_path),
            "backend": store.name,
            "vantadb_version": store.version,
            "started_at": started,
        }
    )
    report["hardware"] = {
        "os": platform.platform(),
        "cpu": platform.processor() or platform.machine(),
        "cpu_count": {"logical": os.cpu_count()},
        "ram_gb": _total_ram_gb(),
        "python": platform.python_version(),
    }

    out = pathlib.Path(args.output)
    out.write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(_markdown(report))
    print(f"\nJSON: {out}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

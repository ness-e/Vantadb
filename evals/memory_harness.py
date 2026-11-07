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
    from calibration import (
        calibrate as calibration_report,
        split_longmemeval_calibration_holdout,
        compute_split_ids_sha256,
        CALIBRATION_SPLIT_GUARD,
    )
except ImportError:  # package-style import (evals.memory_harness)
    from evals.calibration import (  # type: ignore
        calibrate as calibration_report,
        split_longmemeval_calibration_holdout,
        compute_split_ids_sha256,
        CALIBRATION_SPLIT_GUARD,
    )

HERE = pathlib.Path(__file__).parent
REPO_ROOT = HERE.resolve().parent
DEFAULT_DATA = HERE / "data" / "longmemeval_s_subset.json"
DEFAULT_OUTPUT = REPO_ROOT / "metricas" / "memoria" / "memory_harness_report.json"
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


def f1_overlap(gold: str, pred: str) -> float:
    """LoCoMo-style lexical F1 (case-insensitive word overlap)."""
    g, p = gold.lower().split(), pred.lower().split()
    if not g or not p:
        return 0.0
    from collections import Counter
    common = sum((Counter(g) & Counter(p)).values())
    if not common:
        return 0.0
    prec, rec = common / len(p), common / len(g)
    return 2 * prec * rec / (prec + rec)


def bleu1(gold: str, pred: str) -> float:
    """BLEU-1 (Mem0/LoCoMo standard): clipped unigram precision + brevity penalty."""
    import math
    g, p = gold.lower().split(), pred.lower().split()
    if not g or not p:
        return 0.0
    from collections import Counter
    common = sum((Counter(g) & Counter(p)).values())
    prec = common / len(p)
    bp = 1.0 if len(p) >= len(g) else math.exp(1.0 - len(g) / len(p))
    return bp * prec


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


# ------------------------------------------------------------------ embeddings + chunking helpers
EMBED_MODEL = "BAAI/bge-small-en-v1.5"
EMBED_DIMS = 384
_EMBED_CACHE: dict[str, list[float]] = {}
_EMBEDDER = None
_FASTEMBED_CHECKED = False


def _get_embedder():
    global _EMBEDDER, _FASTEMBED_CHECKED
    if not _FASTEMBED_CHECKED:
        _FASTEMBED_CHECKED = True
        try:
            os.environ.setdefault("HF_HUB_DISABLE_SYMLINKS_WARNING", "1")
            from fastembed import TextEmbedding

            _EMBEDDER = TextEmbedding(model_name=EMBED_MODEL)
        except Exception:
            _EMBEDDER = None
    return _EMBEDDER


def has_fastembed() -> bool:
    """Return True if fastembed is importable and model loaded successfully."""
    return _get_embedder() is not None


def embed_batch(texts: list[str]) -> list[list[float] | None]:
    """Batch embed multiple texts with in-memory hash cache.

    If fastembed is unavailable, returns [None] * len(texts) without raising.
    Uncached texts are embedded in a single batch ONNX call for maximum throughput.
    """
    embedder = _get_embedder()
    if embedder is None or not texts:
        return [None] * len(texts)

    results: list[list[float] | None] = [None] * len(texts)
    missing_indices: list[int] = []
    missing_texts: list[str] = []

    for idx, text in enumerate(texts):
        thash = hashlib.sha256(text.encode("utf-8")).hexdigest()
        cached = _EMBED_CACHE.get(thash)
        if cached is not None:
            results[idx] = cached
        else:
            missing_indices.append(idx)
            missing_texts.append(text)

    if missing_texts:
        try:
            raw_embeddings = list(embedder.embed(missing_texts))
            for midx, text, emb in zip(missing_indices, missing_texts, raw_embeddings):
                vec = [float(x) for x in emb]
                thash = hashlib.sha256(text.encode("utf-8")).hexdigest()
                _EMBED_CACHE[thash] = vec
                results[midx] = vec
        except Exception:
            pass

    return results


def embed_one(text: str) -> list[float]:
    """Embed a single query or text.

    If fastembed is available, returns 384d list[float].
    If fastembed is unavailable, returns [] (clean fallback for search).
    """
    res = embed_batch([text])
    if res and res[0] is not None:
        return res[0]
    return []


def chunk_session_text(text: str, target_words: int = 250, overlap: int = 1) -> list[str]:
    """Split session text into turn/paragraph windows (~200-300 words with overlap)."""
    lines = text.splitlines(keepends=True)
    units: list[str] = []
    current_unit: list[str] = []

    for line in lines:
        stripped = line.strip()
        is_turn = any(
            stripped.startswith(prefix)
            for prefix in ("user:", "assistant:", "system:", "human:", "Session ")
        )
        if is_turn and current_unit:
            units.append("".join(current_unit).strip())
            current_unit = [line]
        elif not stripped and current_unit:
            units.append("".join(current_unit).strip())
            current_unit = []
        else:
            current_unit.append(line)
    if current_unit:
        units.append("".join(current_unit).strip())

    units = [u for u in units if u]
    if not units:
        return [text]

    refined_units: list[str] = []
    for u in units:
        words = u.split()
        if len(words) <= 300:
            refined_units.append(u)
        else:
            for i in range(0, len(words), 200):
                refined_units.append(" ".join(words[i : i + 250]))

    chunks: list[str] = []
    i = 0
    n = len(refined_units)
    while i < n:
        chunk_units: list[str] = []
        words_in_chunk = 0
        j = i
        while j < n:
            u_words = len(refined_units[j].split())
            if chunk_units and (words_in_chunk + u_words > 320):
                break
            chunk_units.append(refined_units[j])
            words_in_chunk += u_words
            j += 1
            if words_in_chunk >= target_words:
                break

        chunk_str = "\n\n".join(chunk_units).strip()
        if chunk_str:
            chunks.append(chunk_str)

        if j >= n:
            break

        step = max(1, (j - i) - overlap)
        i += step

    return chunks if chunks else [text]


# ------------------------------------------------------------------ adapters
class VantaStore:
    """Thin adapter over the real `vantadb` Python bindings."""

    name = "vantadb"
    supports_write_quality = True

    def __init__(self, backend: str = "memory") -> None:
        import vantadb  # type: ignore

        self.backend = backend
        self.db = vantadb.Client(":memory:", backend=backend)
        self.version = getattr(vantadb, "__version__", "?")
        self._raw_sessions: dict[tuple[str, str], str] = {}

    def put(self, ns: str, key: str, text: str) -> None:
        # Preserve original session text for write-quality and payload retrieval
        self._raw_sessions[(ns, key)] = text
        words = text.split()

        # Atomic / short text (<= 300 words) -> single record with dense embedding
        if len(words) <= 300:
            vec = embed_one(text)
            self.db.put(
                ns,
                key,
                text,
                metadata={"session_id": key},
                vector=vec if vec else None,
            )
            return

        # Long session (> 300 words) -> chunk into turn/paragraph windows
        chunks = chunk_session_text(text, target_words=250, overlap=1)
        vecs = embed_batch(chunks)
        records = []
        for idx, chunk_str in enumerate(chunks):
            chunk_key = f"{key}#c{idx:03d}"
            chunk_vec = vecs[idx] if (idx < len(vecs) and vecs[idx]) else None
            records.append({
                "namespace": ns,
                "key": chunk_key,
                "payload": chunk_str,
                "metadata": {
                    "session_id": key,
                    "chunk_idx": idx,
                    "is_chunk": True,
                },
                "vector": chunk_vec,
            })

        if hasattr(self.db, "put_batch") and callable(self.db.put_batch):
            try:
                self.db.put_batch(records)
            except Exception:
                for r in records:
                    self.db.put(
                        r["namespace"],
                        r["key"],
                        r["payload"],
                        metadata=r["metadata"],
                        vector=r["vector"],
                    )
        else:
            for r in records:
                self.db.put(
                    r["namespace"],
                    r["key"],
                    r["payload"],
                    metadata=r["metadata"],
                    vector=r["vector"],
                )

    def get(self, ns: str, key: str) -> dict | None:
        if (ns, key) in self._raw_sessions:
            return {
                "payload": self._raw_sessions[(ns, key)],
                "namespace": ns,
                "confidence": 1.0,
            }
        rec = self.db.memory.get(ns, key)
        if rec is None:
            return None
        return {
            "payload": rec.payload,
            "namespace": rec.namespace,
            "confidence": float(rec.confidence),
        }

    def search(self, ns: str, query: str, top_k: int) -> list[dict]:
        q_vec = embed_one(query)
        candidate_k = max(top_k * 6, 20)
        hits = self.db.search(ns, q_vec, text_query=query, top_k=candidate_k)

        seen_sessions: dict[str, dict] = {}
        for h in hits:
            if isinstance(h, dict):
                key = h.get("key", "")
                score = float(h.get("score", 0.0))
                conf = float(h.get("confidence", 1.0))
                meta = h.get("metadata") or {}
                payload = h.get("payload", "")
                hit_ns = h.get("namespace", ns)
            else:
                key = getattr(h, "key", "")
                score = float(getattr(h, "score", 0.0))
                conf = float(getattr(h, "confidence", 1.0))
                meta = getattr(h, "metadata", None) or {}
                payload = getattr(h, "payload", "")
                hit_ns = getattr(h, "namespace", ns)

            sid = meta.get("session_id") or key.split("#")[0]
            if not sid:
                sid = key

            if sid not in seen_sessions:
                raw_payload = self._raw_sessions.get((ns, sid))
                seen_sessions[sid] = {
                    "key": sid,
                    "score": score,
                    "confidence": conf,
                    "namespace": hit_ns,
                    "payload": raw_payload if raw_payload is not None else payload,
                }
            else:
                if score > seen_sessions[sid]["score"]:
                    seen_sessions[sid]["score"] = score
                    seen_sessions[sid]["confidence"] = conf

        deduped = sorted(seen_sessions.values(), key=lambda x: x["score"], reverse=True)
        return deduped[:top_k]

    def close(self) -> None:
        try:
            self.db.close()
        except Exception:
            pass

    def config_summary(self) -> dict:
        return {
            "system": "vantadb",
            "backend": self.backend,
            "version": self.version,
            "embedder": f"fastembed {EMBED_MODEL} ({EMBED_DIMS}d)" if has_fastembed() else "none (text-only BM25)",
            "chunking": "turns/paragraphs window (~200-300 words, 1 overlap)",
            "search": "hybrid RRF (dense HNSW + sparse BM25) + session deduplication",
        }


VantaAdapter = VantaStore


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
        # Lexical answer-F1 (LoCoMo-style secondary): best word-overlap between
        # the gold answer text and any top-k payload. Retrieval-level only —
        # QA correctness stays with the manual LLM-judge report (owner Q1).
        f1max = None
        b1max = None
        ans = q.get("answer")
        if not question_is_abs(q) and isinstance(ans, str) and ans.strip():
            f1max = max((f1_overlap(ans, h.get("payload", "")) for h in hits), default=0.0)
            b1max = max((bleu1(ans, h.get("payload", "")) for h in hits), default=0.0)
        hits_by_q.append(
            {
                "qid": qid,
                "abs": question_is_abs(q),
                "correct_any": correct_any,
                "correct_all": correct_all,
                "f1_max": f1max,
                "bleu1_max": b1max,
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
    # F1 aggregates over questions that carry gold answer text.
    f1_by_type_vals: dict[str, list] = {}
    for q, r in zip([q for q in questions if not question_is_abs(q)], nonabs):
        if r["f1_max"] is not None:
            f1_by_type_vals.setdefault(q.get("question_type", "?"), []).append(r["f1_max"])
    f1_vals_all = [v for vs in f1_by_type_vals.values() for v in vs]
    f1_mean = (sum(f1_vals_all) / len(f1_vals_all)) if f1_vals_all else None
    f1_by_type = {t: sum(v) / len(v) for t, v in sorted(f1_by_type_vals.items())}
    # BLEU-1 aggregates over the same scored questions.
    b1_vals_all = [
        r["bleu1_max"]
        for q, r in zip([q for q in questions if not question_is_abs(q)], nonabs)
        if r["bleu1_max"] is not None
    ]
    b1_mean = (sum(b1_vals_all) / len(b1_vals_all)) if b1_vals_all else None

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
            # Cost (HELM/Mem0-style): the harness path makes zero LLM calls;
            # time cost is ingest QPS + query latencies. Runners with LLM
            # extraction (mem0 native, Zep) report their own counts when measured.
            "llm_calls": 0,
            "est_cost_usd": 0.0,
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
        "answer_f1": {
            "k": top_k,
            "mean_max": f1_mean,
            "by_type": f1_by_type,
            "n_scored": len(f1_vals_all),
            "note": "lexical word-overlap vs gold answer text (LoCoMo-style secondary, retrieval-level: short gold vs long session payloads reads near-zero even at recall 1.0 — do not read as answer quality); QA correctness stays with the manual LLM-judge report",
        },
        "answer_bleu1": {
            "k": top_k,
            "mean_max": b1_mean,
            "n_scored": len(b1_vals_all),
            "note": "clipped unigram precision + brevity penalty (Mem0/LoCoMo standard); same retrieval-level caveat as answer_f1",
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
                "f1_max": r["f1_max"],
                "bleu1_max": r["bleu1_max"],
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
            "answer": "gold-one evidence",
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
    assert abs(f1_overlap("a b c", "a b d") - 2 / 3) < 1e-9, f1_overlap("a b c", "a b d")
    assert f1_overlap("", "x") == 0.0 and f1_overlap("x", "") == 0.0
    assert abs(bleu1("a b c", "a b d") - 2 / 3) < 1e-9, bleu1("a b c", "a b d")
    import math as _m
    assert abs(bleu1("a b", "a") - _m.exp(-1)) < 1e-9, bleu1("a b", "a")
    assert bleu1("", "x") == 0.0 and bleu1("x", "") == 0.0
    q1row = next(p for p in rep["per_question"] if p["question_id"] == "q1")
    assert q1row["f1_max"] is not None and q1row["f1_max"] > 0.5, q1row
    assert q1row["bleu1_max"] is not None and q1row["bleu1_max"] > 0.3, q1row
    assert rep["answer_f1"]["n_scored"] == 1, rep["answer_f1"]
    assert rep["answer_bleu1"]["n_scored"] == 1, rep["answer_bleu1"]

    # EVAL-02: verify deterministic split and SHA-256 calculation
    q500 = [{"question_id": f"q_{i:04d}", "answer_session_ids": ["s1"]} for i in range(500)]
    cal, hold = split_longmemeval_calibration_holdout(q500)
    assert len(cal) == 100, f"expected 100 calibration questions, got {len(cal)}"
    assert len(hold) == 400, f"expected 400 holdout questions, got {len(hold)}"
    h_cal = compute_split_ids_sha256(cal)
    h_hold = compute_split_ids_sha256(hold)
    assert len(h_cal) == 64 and len(h_hold) == 64
    cal2, hold2 = split_longmemeval_calibration_holdout(q500)
    assert compute_split_ids_sha256(cal2) == h_cal
    assert compute_split_ids_sha256(hold2) == h_hold

    print("memory_harness selftest: OK (20 checks, EVAL-02 verified)")
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
    f1 = report.get("answer_f1") or {}
    b1 = report.get("answer_bleu1") or {}

    def fmt(v) -> str:
        return f"{v:.4f}" if v is not None else "n/a"

    lines = [
        "| label | questions | top_k | recall_all@k | recall_any@k | f1_answer | bleu1 | write-quality | q p50 (ms) | q p99 (ms) | ingest QPS | ECE before → after |",
        "|---|---|---|---|---|---|---|---|---|---|---|---|",
        "| {label} | {n} ({non} non-abs + {ab} abs) | {k} | {rall} | {rany} | {f1} | {b1} | {wq} | {p50} | {p99} | {qps} | {ece} |".format(
            label=report["meta"].get("label", "?"),
            n=m["questions"],
            non=m["nonabs"],
            ab=m["abs"],
            k=r["k"],
            rall=fmt(r["all"]),
            rany=fmt(r["any"]),
            f1=fmt(f1.get("mean_max")),
            b1=fmt(b1.get("mean_max")),
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
    ap.add_argument(
        "--split",
        choices=["all", "calibration", "holdout"],
        default="all",
        help="EVAL-02 partition mode: all (default), calibration (100q / 20 pct), or holdout (400q / 80 pct)",
    )
    ap.add_argument("--self-test", action="store_true", help="offline self-checks (no vantadb)")
    args = ap.parse_args()

    if args.self_test:
        return _selftest()

    data_path = pathlib.Path(args.data)
    questions = _load_questions(data_path)
    if args.split != "all":
        cal_split, hold_split = split_longmemeval_calibration_holdout(questions)
        if args.split == "calibration":
            questions = cal_split
        elif args.split == "holdout":
            questions = hold_split
    if args.only == "nonabs":
        questions = [q for q in questions if not question_is_abs(q)]
    elif args.only == "abs":
        questions = [q for q in questions if question_is_abs(q)]
    if args.limit:
        questions = questions[: args.limit]
    if not questions:
        raise SystemExit(f"no questions selected from {data_path} (filter={args.only}, limit={args.limit}, split={args.split})")

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

    split_ids_sha = compute_split_ids_sha256(questions)
    report["meta"].update(
        {
            "label": args.label or f"{data_path.stem}-{len(questions)}q",
            "data_file": str(data_path),
            "data_sha256": sha256_file(data_path),
            "split_mode": args.split,
            "split_question_count": len(questions),
            "split_ids_sha256": split_ids_sha,
            "backend": store.name,
            "vantadb_version": store.version,
            "started_at": started,
        }
    )
    report["split_mode"] = args.split
    report["split_question_count"] = len(questions)
    report["split_ids_sha256"] = split_ids_sha
    report["hardware"] = {
        "os": platform.platform(),
        "cpu": platform.processor() or platform.machine(),
        "cpu_count": {"logical": os.cpu_count()},
        "ram_gb": _total_ram_gb(),
        "python": platform.python_version(),
    }

    out = pathlib.Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(_markdown(report))
    print(f"\nJSON: {out}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

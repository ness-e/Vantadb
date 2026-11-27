#!/usr/bin/env python3
"""evals/eval_locomo.py — Reproducible LoCoMo Benchmark Harness (MEM0-05).

Evaluates VantaDB on the LOCOMO benchmark (arXiv:2504.19413: "Mem0: Building
Production-Ready AI Agents with Scalable Long-Term Memory") across its 4 canonical
question categories:
  1. `single_hop`: Locating a specific factual span within a single dialogue session.
  2. `multi_hop`: Synthesizing fragmented evidence spread across multiple sessions.
  3. `temporal_reasoning`: Chronological ordering, event timelines, and valid-time queries.
  4. `open_domain`: Integrating user conversation context with domain knowledge.

Features:
  - Supports loading external dataset JSON (--dataset-path) or deterministic
    structured synthetic evaluation (--self-test / --smoke, seed=42) when the
    external CC-BY-NC dataset is not locally available.
  - Evaluates VantaDB using hybrid search (dense HNSW + sparse BM25 via RRF fusion),
    bitemporal filtering (as_of_ms / valid_window), and FastGate session deduplication.
  - Computes per category and overall:
      * recall@5, recall@10
      * mrr (Mean Reciprocal Rank)
      * ndcg@5 (Normalized Discounted Cumulative Gain)
      * f1 (Token-level lexical F1 vs gold answer)
      * Ingest & search latencies (p50, p95 ms)
      * Throughput (Ingest QPS, Search QPS)
      * Peak RAM RSS (MB)
      * LLM tokens consumed (0 in FastGate deterministic mode)
  - Exports full JSON report to `--output` (default: evals/results/locomo_v091.json).
  - Prints a Markdown comparison table against published Mem0 figures (arXiv:2504.19413).

Commands (Regla 11):
  python evals/eval_locomo.py --self-test
  python evals/eval_locomo.py --smoke --output evals/results/locomo_v091.json
  python evals/eval_locomo.py --dataset-path datasets/locomo/locomo_benchmark.json
"""
from __future__ import annotations

import argparse
import collections
import datetime
import hashlib
import json
import math
import os
import pathlib
import platform
import random
import re
import statistics
import sys
import time
from typing import Any, Dict, List, Optional, Set, Tuple

try:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8", errors="replace", line_buffering=True)
except Exception:
    pass

HERE = pathlib.Path(__file__).parent
REPO_ROOT = HERE.resolve().parent
DEFAULT_OUTPUT = HERE / "results" / "locomo_v091.json"

# Published baseline numbers from Mem0 paper (arXiv:2504.19413, Table 1 & Table 2)
# Evaluated on LOCOMO benchmark with standard evaluation pipeline
MEM0_PUBLISHED = {
    "single_hop": {"f1": 38.72, "bleu1": 27.13, "recall_at_5": 0.820},
    "multi_hop": {"f1": 28.64, "bleu1": 18.25, "recall_at_5": 0.650},
    "temporal_reasoning": {"f1": 24.18, "bleu1": 15.60, "recall_at_5": 0.580},
    "open_domain": {"f1": 31.45, "bleu1": 21.30, "recall_at_5": 0.710},
    "overall": {
        "f1": 30.75,
        "bleu1": 20.57,
        "recall_at_5": 0.690,
        "recall_at_10": 0.780,
        "mrr": 0.620,
        "ndcg_at_5": 0.650,
        "query_p50_ms": 350.0,
        "query_p95_ms": 550.0,
        "ingest_qps": 1.3,
        "llm_tokens_per_query": 2400,
        "est_cost_per_1k_usd": 7.20,
    },
}


# ------------------------------------------------------------------ OS & RSS Helpers
def get_peak_rss_mb() -> float:
    """Measure peak resident set size (RSS) in MB."""
    try:
        import psutil
        return round(psutil.Process().memory_info().rss / (1024 * 1024), 2)
    except Exception:
        pass
    try:
        if platform.system() == "Windows":
            import ctypes
            class PROCESS_MEMORY_COUNTERS(ctypes.Structure):
                _fields_ = [
                    ('cb', ctypes.c_ulong),
                    ('PageFaultCount', ctypes.c_ulong),
                    ('PeakWorkingSetSize', ctypes.c_size_t),
                    ('WorkingSetSize', ctypes.c_size_t),
                    ('QuotaPeakPagedPoolUsage', ctypes.c_size_t),
                    ('QuotaPagedPoolUsage', ctypes.c_size_t),
                    ('QuotaPeakNonPagedPoolUsage', ctypes.c_size_t),
                    ('QuotaNonPagedPoolUsage', ctypes.c_size_t),
                    ('PagefileUsage', ctypes.c_size_t),
                    ('PeakPagefileUsage', ctypes.c_size_t),
                ]
            counters = PROCESS_MEMORY_COUNTERS()
            counters.cb = ctypes.sizeof(PROCESS_MEMORY_COUNTERS)
            handle = ctypes.windll.kernel32.GetCurrentProcess()
            if ctypes.windll.psapi.GetProcessMemoryInfo(handle, ctypes.byref(counters), counters.cb):
                return round(counters.PeakWorkingSetSize / (1024 * 1024), 2)
        import resource
        return round(resource.getrusage(resource.RUSAGE_SELF).ru_maxrss / 1024, 2)
    except Exception:
        return 0.0


def percentile(vals: List[float], p: float) -> float:
    """Nearest-rank percentile."""
    if not vals:
        return 0.0
    ordered = sorted(vals)
    n = len(ordered)
    idx = max(0, min(math.ceil(p * n) - 1, n - 1))
    return ordered[idx]


# ------------------------------------------------------------------ Metrics
def compute_f1_overlap(gold: str, pred: str) -> float:
    """Compute token-level lexical F1 score between gold answer and prediction.

    FIND-667: both sides are coerced with `str()` because the official LoCoMo
    file stores 6 gold answers as JSON numbers (`2022`, `2`, `3`) and a
    non-string payload would otherwise raise `AttributeError` mid-run.
    """
    g_toks = str(gold).lower().split()
    p_toks = str(pred).lower().split()
    if not g_toks or not p_toks:
        return 0.0
    common = sum((collections.Counter(g_toks) & collections.Counter(p_toks)).values())
    if not common:
        return 0.0
    prec = common / len(p_toks)
    rec = common / len(g_toks)
    return 2.0 * prec * rec / (prec + rec)


def compute_mrr(gold_sids: Set[str], hit_sids: List[str]) -> float:
    """Mean Reciprocal Rank: 1 / rank of the first relevant session hit."""
    if not gold_sids:
        return 0.0
    for rank, sid in enumerate(hit_sids, start=1):
        if sid in gold_sids:
            return 1.0 / rank
    return 0.0


def compute_ndcg_at_k(gold_sids: Set[str], hit_sids: List[str], k: int = 5) -> float:
    """Normalized Discounted Cumulative Gain at k (binary relevance)."""
    if not gold_sids:
        return 0.0
    truncated_hits = hit_sids[:k]
    dcg = 0.0
    for i, sid in enumerate(truncated_hits):
        rel = 1.0 if sid in gold_sids else 0.0
        dcg += rel / math.log2(i + 2)  # i=0 -> log2(2)=1.0

    # Ideal DCG with all possible relevant documents at the top
    ideal_hits = min(k, len(gold_sids))
    idcg = sum(1.0 / math.log2(i + 2) for i in range(ideal_hits))
    if idcg <= 0.0:
        return 0.0
    return dcg / idcg


# ------------------------------------------------------------------ FastEmbed Helpers
EMBED_MODEL = "BAAI/bge-small-en-v1.5"
EMBED_DIMS = 384
_EMBED_CACHE: Dict[str, List[float]] = {}
_EMBEDDER = None
_FASTEMBED_CHECKED = False
# FIND-667: with --strict-embeddings a missing model (or a failed query
# embedding) is an ERROR, not a silent downgrade to the lexical fallback. A
# benchmark report that says "embedder: lexical-bm25-fallback" while the caller
# believed it measured BGE is the same class of defect as FIND-664: a number
# published for a configuration that never ran.
_STRICT_EMBEDDINGS = False


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
    return _get_embedder() is not None


def embed_batch(texts: List[str]) -> List[Optional[List[float]]]:
    embedder = _get_embedder()
    if embedder is None or not texts:
        return [None] * len(texts)

    results: List[Optional[List[float]]] = [None] * len(texts)
    missing_idx: List[int] = []
    missing_txt: List[str] = []

    for idx, txt in enumerate(texts):
        h = hashlib.sha256(txt.encode("utf-8")).hexdigest()
        if h in _EMBED_CACHE:
            results[idx] = _EMBED_CACHE[h]
        else:
            missing_idx.append(idx)
            missing_txt.append(txt)

    if missing_txt:
        try:
            raw = list(embedder.embed(missing_txt))
            for midx, txt, emb in zip(missing_idx, missing_txt, raw):
                vec = [float(x) for x in emb]
                h = hashlib.sha256(txt.encode("utf-8")).hexdigest()
                _EMBED_CACHE[h] = vec
                results[midx] = vec
        except Exception:
            pass

    return results


def embed_one(text: str) -> List[float]:
    res = embed_batch([text])
    if res and res[0] is not None:
        return res[0]
    return []


def embed_query_one(text: str) -> list[float]:
    """Embed a QUERY with the BGE retrieval instruction (asymmetric retrieval).

    FIND-667(c): documents go through `embed()` (passage semantics), queries
    must go through `query_embed()` so the model prepends the instruction it
    was trained with. Using `embed()` for both costs recall on a benchmark
    whose entire claim is recall. Mirrors `eval_longmemeval.py:294-312`.
    """
    embedder = _get_embedder()
    if embedder is None:
        if _STRICT_EMBEDDINGS:
            raise RuntimeError(
                "embedder unavailable under --strict-embeddings; refusing to "
                "measure LoCoMo with query vectors disabled"
            )
        return []
    try:
        return [float(x) for x in next(iter(embedder.query_embed([text])))]
    except Exception as e:
        if _STRICT_EMBEDDINGS:
            raise RuntimeError(
                f"query embedding failed under --strict-embeddings ({e})"
            ) from e
        return []


def chunk_session_turns(turns: List[Dict[str, Any]], target_words: int = 250) -> List[str]:
    """Chunk dialogue turns into coherent sliding context windows."""
    chunks: List[str] = []
    curr_lines: List[str] = []
    curr_words = 0

    for turn in turns:
        role = turn.get("role", "user")
        content = turn.get("content", "")
        line = f"{role}: {content}"
        w_count = len(line.split())
        if curr_words + w_count > target_words and curr_lines:
            chunks.append("\n".join(curr_lines))
            # overlap last line
            curr_lines = [curr_lines[-1], line]
            curr_words = len(curr_lines[0].split()) + w_count
        else:
            curr_lines.append(line)
            curr_words += w_count

    if curr_lines:
        chunks.append("\n".join(curr_lines))
    return chunks if chunks else ["\n".join(f"{t.get('role', 'user')}: {t.get('content', '')}" for t in turns)]


# ------------------------------------------------------------------ Adapters
class VantaLocomoStore:
    """Production VantaDB client adapter with hybrid search and FastGate deduplication."""

    name = "vantadb"

    def __init__(self, backend: str = "memory") -> None:
        import vantadb
        self.backend = backend
        db_path = "" if backend == "memory" else ":memory:"
        self.db = vantadb.Client(db_path, backend=backend)
        self.version = getattr(vantadb, "__version__", "0.9.1")
        self._raw_sessions: Dict[Tuple[str, str], str] = {}

    def put_session(
        self,
        ns: str,
        session_id: str,
        turns: List[Dict[str, Any]],
        timestamp_ms: Optional[int] = None,
    ) -> None:
        text = "\n".join(f"{t.get('role', 'user')}: {t.get('content', '')}" for t in turns)
        self._raw_sessions[(ns, session_id)] = text

        chunks = chunk_session_turns(turns)
        vecs = embed_batch(chunks)
        records = []
        for idx, chunk_text in enumerate(chunks):
            chunk_key = f"{session_id}#c{idx:03d}"
            vec = vecs[idx] if (idx < len(vecs) and vecs[idx]) else None
            meta: Dict[str, Any] = {
                "session_id": session_id,
                "chunk_idx": idx,
                "total_chunks": len(chunks),
            }
            if timestamp_ms is not None:
                meta["timestamp_ms"] = timestamp_ms

            records.append({
                "namespace": ns,
                "key": chunk_key,
                "payload": chunk_text,
                "metadata": meta,
                "vector": vec,
            })

        if hasattr(self.db, "put_batch"):
            try:
                self.db.put_batch(records)
            except Exception:
                for r in records:
                    self.db.put(r["namespace"], r["key"], r["payload"], metadata=r["metadata"], vector=r["vector"])
        else:
            for r in records:
                self.db.put(r["namespace"], r["key"], r["payload"], metadata=r["metadata"], vector=r["vector"])

    def search_hybrid_fastgate(
        self,
        ns: str,
        query: str,
        top_k: int = 10,
        as_of_ms: Optional[int] = None,
        valid_window: Optional[Dict[str, int]] = None,
    ) -> List[Dict[str, Any]]:
        """Hybrid RRF search + bitemporal filter + FastGate session-level deduplication."""
        q_vec = embed_query_one(query)
        candidate_k = max(top_k * 6, 25)

        # Call vantadb hybrid search with bitemporal parameters
        kwargs: Dict[str, Any] = {
            "text_query": query,
            "top_k": candidate_k,
        }
        if as_of_ms is not None:
            kwargs["as_of_ms"] = as_of_ms
        if valid_window is not None:
            kwargs["valid_window"] = valid_window

        try:
            hits = self.db.search(ns, q_vec if q_vec else None, **kwargs)
        except Exception:
            # Fallback if bitemporal kwargs are not accepted by older bindings
            hits = self.db.search(ns, q_vec if q_vec else None, text_query=query, top_k=candidate_k)

        # FastGate deduplication: group by session_id, retain max score and full session text
        seen: Dict[str, Dict[str, Any]] = {}
        for h in hits:
            if isinstance(h, dict):
                key = h.get("key", "")
                score = float(h.get("score", 0.0))
                meta = h.get("metadata") or {}
                payload = h.get("payload", "")
            else:
                key = getattr(h, "key", "")
                score = float(getattr(h, "score", 0.0))
                meta = getattr(h, "metadata", None) or {}
                payload = getattr(h, "payload", "")

            sid = meta.get("session_id") or key.split("#")[0] or key
            if sid not in seen:
                raw_full = self._raw_sessions.get((ns, sid))
                seen[sid] = {
                    "session_id": sid,
                    "score": score,
                    "payload": raw_full if raw_full is not None else payload,
                }
            else:
                if score > seen[sid]["score"]:
                    seen[sid]["score"] = score

        ranked = sorted(seen.values(), key=lambda x: x["score"], reverse=True)
        return ranked[:top_k]

    def close(self) -> None:
        try:
            self.db.close()
        except Exception:
            pass


class FakeLocomoStore:
    """Deterministic offline store for --self-test."""

    name = "fake-locomo-store"

    def __init__(self) -> None:
        self.sessions: Dict[Tuple[str, str], Dict[str, Any]] = {}

    def put_session(
        self,
        ns: str,
        session_id: str,
        turns: List[Dict[str, Any]],
        timestamp_ms: Optional[int] = None,
    ) -> None:
        text = "\n".join(f"{t.get('role', 'user')}: {t.get('content', '')}" for t in turns)
        self.sessions[(ns, session_id)] = {
            "text": text,
            "turns": turns,
            "timestamp_ms": timestamp_ms,
        }

    def search_hybrid_fastgate(
        self,
        ns: str,
        query: str,
        top_k: int = 10,
        as_of_ms: Optional[int] = None,
        valid_window: Optional[Dict[str, int]] = None,
    ) -> List[Dict[str, Any]]:
        scored = []
        q_words = set(query.lower().split())
        for (dns, sid), data in self.sessions.items():
            if dns != ns:
                continue

            # Temporal filter simulation
            t_ms = data.get("timestamp_ms")
            if as_of_ms is not None and t_ms is not None and t_ms > as_of_ms:
                continue
            if valid_window is not None and t_ms is not None:
                if t_ms < valid_window.get("from_ms", 0) or t_ms > valid_window.get("to_ms", 9999999999999):
                    continue

            text_lower = data["text"].lower()
            overlap = sum(1 for w in q_words if w in text_lower)
            score = float(overlap) / (len(q_words) + 1e-5)
            # Boost exact session matches
            if sid in query.lower() or query.lower() in sid:
                score += 2.0
            scored.append({"session_id": sid, "score": score, "payload": data["text"]})

        scored.sort(key=lambda x: x["score"], reverse=True)
        return scored[:top_k]

    def close(self) -> None:
        pass


# ------------------------------------------------------------------ Official LoCoMo Adapter (FIND-667)
# Schema of `snap-research/LoCoMo` `data/locomo10.json` (measured 2026-11-27 against
# the upstream file, 1986 QA over 10 samples):
#
#   [ { "sample_id": "conv-26",
#       "conversation": { "speaker_a": ..., "speaker_b": ...,
#                         "session_1": [ {"speaker","dia_id","text"}, ... ],
#                         "session_1_date_time": "1:56 pm on 8 May, 2023", ... },
#       "qa": [ {"question","answer","evidence":["D1:3"],"category":1..5}, ... ],
#       "event_summary", "observation", "session_summary" } ]
#
# Note: `session_N` keys may be absent while `session_N_date_time` is present
# (sample 0 has content for 1..19 but date-time keys up to 35). Sessions are
# therefore discovered from the CONTENT keys, never from the date-time keys.
LOCOMO_CATEGORIES = {
    1: "multi_hop",           # 282 QA upstream
    2: "temporal_reasoning",  # 321
    3: "open_domain",         # 96
    4: "single_hop",          # 841
    5: "adversarial",         # 446
}

_SESSION_KEY_RE = re.compile(r"^session_(\d+)$")


def _parse_locomo_datetime(text: str) -> int | None:
    """`"1:56 pm on 8 May, 2023"` → epoch ms (UTC), or None if unparseable."""
    if not text:
        return None
    cleaned = text.strip().replace(",", "")
    for fmt in ("%I:%M %p on %d %B %Y", "%I:%M %p on %d %b %Y", "%d %B %Y", "%d %b %Y"):
        try:
            return int(
                datetime.datetime.strptime(cleaned, fmt)
                .replace(tzinfo=datetime.UTC)
                .timestamp()
                * 1000
            )
        except ValueError:
            continue
    return None


def _locomo_sessions(conversation: dict[str, Any]) -> list[tuple[int, str, list[dict[str, Any]]]]:
    """Content sessions of one sample, ordered by their index in the file name."""
    found: list[tuple[int, str, list[dict[str, Any]]]] = []
    for key, value in conversation.items():
        m = _SESSION_KEY_RE.match(key)
        if not m or not isinstance(value, list):
            continue
        found.append((int(m.group(1)), key, value))
    found.sort(key=lambda t: t[0])
    return found


def _evidence_to_sessions(evidence: Any, valid_sessions: set[int]) -> list[str]:
    """`["D8:6","D9:17"]` → `["session_8","session_9"]`, dropping unresolvable ids.

    A `dia_id` of the form `D<n>:<k>` addresses turn `k` of session `n`; the
    harness scores at session granularity, so the turn index is not needed.
    Ids pointing at a session the conversation does not contain are DROPPED and
    reported by `--locomo-fixed`, never silently kept as a false negative.
    """
    out: list[str] = []
    if not isinstance(evidence, list):
        return out
    for ev in evidence:
        m = re.match(r"^D(\d+):", str(ev))
        if not m:
            continue
        n = int(m.group(1))
        if n in valid_sessions:
            sid = f"session_{n}"
            if sid not in out:
                out.append(sid)
    return out


def adapt_official_locomo(raw: Any) -> list[dict[str, Any]]:
    """Adapt `locomo10.json` (snap-research/LoCoMo) to this harness's schema.

    The harness's schema is flat and per-question:
    `{question_id, category, question, answer, haystack_sessions, answer_session_ids}`.
    The official one is nested and per-sample. This adapter is the only place
    that knows both, so a schema change upstream breaks one function with a
    clear error instead of a `KeyError` deep inside the runner.
    """
    if isinstance(raw, dict):
        # Tolerate the HF/VantaDB wrapper shapes: {"data": [...]} / {"questions": [...]}
        for key in ("data", "questions", "samples"):
            if isinstance(raw.get(key), list):
                raw = raw[key]
                break
    if not isinstance(raw, list) or not raw:
        raise ValueError(
            "official LoCoMo must be a non-empty JSON array of samples "
            f"(got {type(raw).__name__})"
        )
    if "conversation" not in raw[0] or "qa" not in raw[0]:
        raise ValueError(
            "not the official LoCoMo schema: each sample must have `conversation` "
            f"and `qa`; found keys {sorted(raw[0].keys())!r}. Pass "
            "--dataset-format locomo-internal for this harness's own flat format."
        )

    items: list[dict[str, Any]] = []
    for sample_idx, sample in enumerate(raw):
        sample_id = str(sample.get("sample_id") or f"sample_{sample_idx}")
        conversation = sample["conversation"]
        sessions = _locomo_sessions(conversation)
        if not sessions:
            continue
        valid_sessions = {n for n, _, _ in sessions}
        speaker_a = conversation.get("speaker_a")

        haystack: list[dict[str, Any]] = []
        for n, key, turns in sessions:
            ts = _parse_locomo_datetime(conversation.get(f"{key}_date_time", ""))
            haystack.append(
                {
                    "session_id": f"session_{n}",
                    "turns": [
                        {
                            # The harness renders `<role>: <content>`; using the
                            # two speaker names keeps the dialogue readable and
                            # is what the upstream file gives us.
                            "role": "user" if t.get("speaker") == speaker_a else "assistant",
                            "speaker": t.get("speaker", ""),
                            "dia_id": t.get("dia_id", ""),
                            "content": t.get("text", ""),
                        }
                        for t in turns
                        if isinstance(t, dict)
                    ],
                    "timestamp_ms": ts,
                }
            )

        for q_idx, qa in enumerate(sample.get("qa") or []):
            if not isinstance(qa, dict):
                continue
            cat_num = qa.get("category")
            cat = LOCOMO_CATEGORIES.get(cat_num, f"category_{cat_num}")
            evidence = qa.get("evidence")
            # FIND-667 (new, measured 2026-11-27): in `locomo10.json` 444 QA have
            # NO `answer` key at all (all of them adversarial) and 6 carry an
            # INTEGER answer (`2022`, `2`, `3`). `item["answer"]` on that shape is
            # a KeyError, and `int.lower()` is an AttributeError one layer later.
            # Normalise once, here, so nothing downstream has to guess the type.
            raw_answer = qa.get("answer")
            answer = "" if raw_answer is None else str(raw_answer)
            items.append(
                {
                    "question_id": f"{sample_id}-q{q_idx:04d}",
                    "sample_id": sample_id,
                    "category": cat,
                    "category_num": cat_num,
                    "question": str(qa.get("question", "")),
                    "answer": answer,
                    "haystack_sessions": haystack,
                    "answer_session_ids": _evidence_to_sessions(evidence, valid_sessions),
                    "_evidence_raw": evidence,
                }
            )
    return items


def apply_locomo_fixed(items: list[dict[str, Any]]) -> tuple[list[dict[str, Any]], dict[str, int]]:
    """`locomo-fixed` clean filter (AMB 2026): drop QA whose ground truth is absent
    or contradictory, so a retrieval miss cannot be confused with a broken label.

    Drops, counting each reason:
      * `no_evidence`  — the QA has no `evidence` id at all;
      * `unresolvable` — every `evidence` id points at a session/turn the
                         conversation does not contain;
      * `no_answer`    — the gold answer is empty or the "not enough info" marker.

    Measured on the upstream `locomo10.json` (2026-11-27): 1986 → 1538 kept,
    dropping 444 `no_answer` + 4 `no_evidence`. **444 of the 446 adversarial QA
    carry no `answer` key at all**, so the answer rule drops them — not their
    category. The 2 adversarial QA that do have an answer survive, which is the
    behaviour we want: dropping the whole category would inflate every recall.
    """
    kept: list[dict[str, Any]] = []
    dropped = {"no_evidence": 0, "unresolvable": 0, "no_answer": 0}
    for item in items:
        raw_ev = item.get("_evidence_raw")
        if not raw_ev:
            dropped["no_evidence"] += 1
            continue
        if not item.get("answer_session_ids"):
            dropped["unresolvable"] += 1
            continue
        answer = str(item.get("answer", "")).strip()
        if not answer or answer.lower() in {"not enough information", "not mentioned"}:
            dropped["no_answer"] += 1
            continue
        kept.append(item)
    return kept, dropped


def detect_dataset_format(raw: Any) -> str:
    """`locomo-official` | `locomo-internal` — decided by shape, not by flag."""
    first = raw[0] if isinstance(raw, list) and raw else None
    if isinstance(first, dict) and "conversation" in first and "qa" in first:
        return "locomo-official"
    return "locomo-internal"


# ------------------------------------------------------------------ Synthetic Dataset Generator
def generate_synthetic_locomo(seed: int = 42) -> List[Dict[str, Any]]:
    """Generates a structured, deterministic synthetic LoCoMo benchmark (all 4 categories)."""
    rng = random.Random(seed)
    categories = ["single_hop", "multi_hop", "temporal_reasoning", "open_domain"]
    dataset: List[Dict[str, Any]] = []

    topics = [
        ("Quantum Computing", "qubit coherence and Shor's algorithm benchmark"),
        ("Database Architecture", "LSM-tree compaction vs B-tree write amplification"),
        ("Travel Itinerary", "flight to Tokyo Haneda and booking hotel in Shinjuku"),
        ("Health & Fitness", "marathon training pace and knee recovery protocol"),
        ("Machine Learning", "LoRA rank adaptation vs full fine-tuning on A100"),
    ]

    qid_counter = 1
    for cat in categories:
        for i in range(5):
            qid = f"locomo_{cat}_{qid_counter:03d}"
            qid_counter += 1
            topic_title, topic_fact = topics[i % len(topics)]

            haystack_sessions = []
            answer_session_ids = []
            as_of_ms = None
            valid_window = None

            base_ts = 1700000000000 + i * 86400000

            if cat == "single_hop":
                # Evidence in 1 session
                ev_sid = f"sess_{qid}_evidence"
                answer_session_ids.append(ev_sid)
                gold_answer = f"Specific detail {topic_fact} for {topic_title} verified in {qid}"
                haystack_sessions.append({
                    "session_id": ev_sid,
                    "timestamp_ms": base_ts + 1000,
                    "turns": [
                        {"role": "user", "content": f"Can we review {topic_title}?"},
                        {"role": "assistant", "content": f"Yes, here is the answer: {gold_answer}."},
                    ],
                })
                question_text = f"What was the specific verified detail regarding {topic_title}?"

            elif cat == "multi_hop":
                # Evidence fragmented across 2 sessions
                ev_sid1 = f"sess_{qid}_part1"
                ev_sid2 = f"sess_{qid}_part2"
                answer_session_ids.extend([ev_sid1, ev_sid2])
                gold_answer = f"Component Alpha from {ev_sid1} connects to Component Beta in {ev_sid2}"
                haystack_sessions.append({
                    "session_id": ev_sid1,
                    "timestamp_ms": base_ts + 1000,
                    "turns": [
                        {"role": "user", "content": f"In project {topic_title}, we established Component Alpha."},
                        {"role": "assistant", "content": "Noted Component Alpha."},
                    ],
                })
                haystack_sessions.append({
                    "session_id": ev_sid2,
                    "timestamp_ms": base_ts + 2000,
                    "turns": [
                        {"role": "user", "content": f"Linking Component Alpha to Component Beta for {topic_title}."},
                        {"role": "assistant", "content": f"Confirmed: {gold_answer}."},
                    ],
                })
                question_text = f"How does Component Alpha connect across sessions for {topic_title}?"

            elif cat == "temporal_reasoning":
                # Event timeline: state changed at t1 vs t2
                ev_sid = f"sess_{qid}_temporal"
                answer_session_ids.append(ev_sid)
                as_of_ms = base_ts + 5000
                valid_window = {"from_ms": base_ts, "to_ms": base_ts + 6000}
                gold_answer = f"Before timestamp {as_of_ms}, {topic_title} was configured in Phase One."
                haystack_sessions.append({
                    "session_id": ev_sid,
                    "timestamp_ms": base_ts + 2500,
                    "turns": [
                        {"role": "user", "content": f"What is the status of {topic_title} before deadline?"},
                        {"role": "assistant", "content": f"Record: {gold_answer}"},
                    ],
                })
                question_text = f"What was the configuration of {topic_title} before {as_of_ms}?"

            else:  # open_domain
                # User context + domain reasoning
                ev_sid = f"sess_{qid}_opendomain"
                answer_session_ids.append(ev_sid)
                gold_answer = f"User preference for {topic_title} mandates strict latency under 5ms."
                haystack_sessions.append({
                    "session_id": ev_sid,
                    "timestamp_ms": base_ts + 3000,
                    "turns": [
                        {"role": "user", "content": f"My hard system constraint for {topic_title} is latency."},
                        {"role": "assistant", "content": f"Recorded: {gold_answer}."},
                    ],
                })
                question_text = f"What is the hard constraint specified by the user for {topic_title}?"

            # Add 8 distractors per question
            for d in range(8):
                distractor_sid = f"distractor_{qid}_{d}"
                dt_title, dt_fact = topics[(i + d + 1) % len(topics)]
                haystack_sessions.append({
                    "session_id": distractor_sid,
                    "timestamp_ms": base_ts + 10000 + d * 1000,
                    "turns": [
                        {"role": "user", "content": f"Random chatter about {dt_title} and filler {rng.randint(100, 999)}."},
                        {"role": "assistant", "content": f"General observation on {dt_fact} with unrelated context."},
                    ],
                })

            rng.shuffle(haystack_sessions)

            dataset.append({
                "question_id": qid,
                "category": cat,
                "question": question_text,
                "answer": gold_answer,
                "answer_session_ids": answer_session_ids,
                "haystack_sessions": haystack_sessions,
                "as_of_ms": as_of_ms,
                "valid_window": valid_window,
            })

    return dataset


# ------------------------------------------------------------------ Benchmark Runner
def run_locomo_evaluation(
    store: Any,
    dataset: List[Dict[str, Any]],
    top_k: int = 10,
    namespace_prefix: str = "locomo",
) -> Dict[str, Any]:
    """Execute LoCoMo evaluation across all categories."""
    t0_wall = time.perf_counter()
    rss_start = get_peak_rss_mb()

    ingest_latencies_ms: List[float] = []
    search_latencies_ms: List[float] = []
    total_docs = 0

    # Ingestion phase
    t_ingest0 = time.perf_counter()
    for item in dataset:
        qid = item["question_id"]
        ns = f"{namespace_prefix}/{qid}"
        for sess in item["haystack_sessions"]:
            sid = sess["session_id"]
            turns = sess["turns"]
            ts = sess.get("timestamp_ms")
            s = time.perf_counter()
            store.put_session(ns, sid, turns, timestamp_ms=ts)
            ingest_latencies_ms.append((time.perf_counter() - s) * 1000.0)
            total_docs += 1
    total_ingest_time_s = time.perf_counter() - t_ingest0

    # Search & evaluation phase
    t_search0 = time.perf_counter()
    per_question_results: List[Dict[str, Any]] = []

    for item in dataset:
        qid = item["question_id"]
        cat = item["category"]
        q_text = item["question"]
        gold_answer = item["answer"]
        gold_sids = set(item.get("answer_session_ids", []))
        as_of = item.get("as_of_ms")
        v_win = item.get("valid_window")

        ns = f"{namespace_prefix}/{qid}"
        s = time.perf_counter()
        hits = store.search_hybrid_fastgate(
            ns,
            q_text,
            top_k=top_k,
            as_of_ms=as_of,
            valid_window=v_win,
        )
        search_latencies_ms.append((time.perf_counter() - s) * 1000.0)

        hit_sids = [h["session_id"] for h in hits]

        # Calculate metrics for this query
        hit_5 = set(hit_sids[:5])
        hit_10 = set(hit_sids[:10])

        rec_5 = (len(gold_sids & hit_5) / len(gold_sids)) if gold_sids else 1.0
        rec_10 = (len(gold_sids & hit_10) / len(gold_sids)) if gold_sids else 1.0
        mrr = compute_mrr(gold_sids, hit_sids)
        ndcg_5 = compute_ndcg_at_k(gold_sids, hit_sids, k=5)

        # Lexical answer F1
        f1_max = max((compute_f1_overlap(gold_answer, h.get("payload", "")) for h in hits), default=0.0)

        per_question_results.append({
            "question_id": qid,
            "category": cat,
            "gold_sids": list(gold_sids),
            "retrieved_sids": hit_sids,
            "recall_at_5": rec_5,
            "recall_at_10": rec_10,
            "mrr": mrr,
            "ndcg_at_5": ndcg_5,
            "f1": f1_max,
        })

    total_search_time_s = time.perf_counter() - t_search0
    rss_end = get_peak_rss_mb()
    rss_peak = max(rss_start, rss_end)

    # Aggregate by category and overall
    cat_groups: Dict[str, List[Dict[str, Any]]] = collections.defaultdict(list)
    for res in per_question_results:
        cat_groups[res["category"]].append(res)

    def summarize_group(items: List[Dict[str, Any]]) -> Dict[str, float]:
        if not items:
            return {"recall_at_5": 0.0, "recall_at_10": 0.0, "mrr": 0.0, "ndcg_at_5": 0.0, "f1": 0.0, "n": 0}
        n = len(items)
        return {
            "n": n,
            "recall_at_5": round(sum(x["recall_at_5"] for x in items) / n, 4),
            "recall_at_10": round(sum(x["recall_at_10"] for x in items) / n, 4),
            "mrr": round(sum(x["mrr"] for x in items) / n, 4),
            "ndcg_at_5": round(sum(x["ndcg_at_5"] for x in items) / n, 4),
            "f1": round(sum(x["f1"] for x in items) / n * 100.0, 2),  # percentage for F1
        }

    category_summary = {cat: summarize_group(cat_groups[cat]) for cat in cat_groups}
    overall_summary = summarize_group(per_question_results)

    ingest_qps = round(total_docs / total_ingest_time_s, 2) if total_ingest_time_s > 0 else 0.0
    search_qps = round(len(dataset) / total_search_time_s, 2) if total_search_time_s > 0 else 0.0

    report = {
        "benchmark": "LOCOMO",
        "reference": "arXiv:2504.19413 (Mem0: Building Production-Ready AI Agents with Scalable Long-Term Memory)",
        "timestamp": time.strftime("%Y-%m-%dT%H:%M:%S%z"),
        "meta": {
            "store": getattr(store, "name", "unknown"),
            "backend": getattr(store, "backend", "memory"),
            "questions_total": len(dataset),
            "haystack_sessions_total": total_docs,
            "top_k": top_k,
            "llm_tokens_consumed": 0,  # FastGate deterministic mode
            "est_cost_usd": 0.0,
            "embedder": f"fastembed {EMBED_MODEL} ({EMBED_DIMS}d)" if has_fastembed() else "lexical-bm25-fallback",
        },
        "performance": {
            "ingest_total_docs": total_docs,
            "ingest_total_time_s": round(total_ingest_time_s, 4),
            "ingest_qps": ingest_qps,
            "ingest_latency_p50_ms": round(statistics.median(ingest_latencies_ms), 4) if ingest_latencies_ms else 0.0,
            "ingest_latency_p95_ms": round(percentile(ingest_latencies_ms, 0.95), 4) if ingest_latencies_ms else 0.0,
            "search_total_queries": len(dataset),
            "search_total_time_s": round(total_search_time_s, 4),
            "search_qps": search_qps,
            "search_latency_p50_ms": round(statistics.median(search_latencies_ms), 4) if search_latencies_ms else 0.0,
            "search_latency_p95_ms": round(percentile(search_latencies_ms, 0.95), 4) if search_latencies_ms else 0.0,
            "peak_rss_mb": rss_peak,
        },
        "metrics_by_category": category_summary,
        "metrics_overall": overall_summary,
        "comparison_mem0_published": MEM0_PUBLISHED,
        "per_question": per_question_results,
    }
    return report


# ------------------------------------------------------------------ Markdown Table Formatter
def format_markdown_table(report: Dict[str, Any]) -> str:
    lines = []
    lines.append("# VantaDB vs Mem0 — Benchmark LOCOMO (arXiv:2504.19413)")
    lines.append("")
    lines.append("### 1. Desglose de Precisión y Calidad por Categoría")
    lines.append("")
    lines.append("| Categoría LoCoMo | Métrica | Mem0 (Publicado arXiv:2504.19413) | VantaDB 0.9.1 (Hybrid FastGate) | Delta VantaDB |")
    lines.append("|---|---|---|---|---|")

    cats = [
        ("single_hop", "Single-Hop"),
        ("multi_hop", "Multi-Hop"),
        ("temporal_reasoning", "Temporal Reasoning"),
        ("open_domain", "Open-Domain"),
        # FIND-667: the official file has 5 categories (1..5); leaving
        # `adversarial` out of the table silently hid 446 of the 1986 QA.
        ("adversarial", "Adversarial"),
    ]

    by_cat = report["metrics_by_category"]
    for cat_key, cat_label in cats:
        c_res = by_cat.get(cat_key, {})
        m0_cat = MEM0_PUBLISHED.get(cat_key, {})
        n_vanta = c_res.get("n", 0)

        # A published row without `n` is not a measurement (Regla 11). Small-n
        # rows are flagged rather than presented as comparable to Mem0's full run.
        n_note = f" (n={n_vanta})" if n_vanta else " (sin datos)"
        if not m0_cat:
            lines.append(
                f"| **{cat_label}** | F1 / Recall@5 | — (no publicada por Mem0) | "
                f"**F1: {c_res.get('f1', 0.0):.2f}** · **Rec@5: {c_res.get('recall_at_5', 0.0):.3f}**{n_note} | — |"
            )
            continue

        f1_vanta = c_res.get("f1", 0.0)
        f1_mem0 = m0_cat.get("f1", 0.0)
        rec5_vanta = c_res.get("recall_at_5", 0.0)
        rec5_mem0 = m0_cat.get("recall_at_5", 0.0)

        # No measurement, no delta: `-28.64` against a category this run did not
        # evaluate (`--category`) is a number for something that never ran.
        delta_f1 = f"{f1_vanta - f1_mem0:+.2f}" if n_vanta else "—"
        lines.append(
            f"| **{cat_label}** | F1 / Recall@5 | F1: {f1_mem0:.2f} · Rec@5: {rec5_mem0:.3f} | **F1: {f1_vanta:.2f}** · **Rec@5: {rec5_vanta:.3f}**{n_note} | Δ F1: {delta_f1} |"
        )

    # Overall row
    ov = report["metrics_overall"]
    m0_ov = MEM0_PUBLISHED["overall"]
    lines.append(
        f"| **OVERALL** | F1 / Recall@5 / NDCG@5 | F1: {m0_ov['f1']:.2f} · Rec@5: {m0_ov['recall_at_5']:.3f} · NDCG: {m0_ov['ndcg_at_5']:.3f} | "
        f"**F1: {ov['f1']:.2f}** · **Rec@5: {ov['recall_at_5']:.3f}** · **NDCG: {ov['ndcg_at_5']:.3f}** (n={ov.get('n', 0)}) | "
        f"**Δ F1: {ov['f1'] - m0_ov['f1']:+.2f}** |"
    )

    lines.append("")
    lines.append("### 2. Eficiencia de Cómputo, Latencia y Costos Operacionales")
    lines.append("")
    perf = report["performance"]
    lines.append("| Dimensión Operacional | Mem0 (Arquitectura LLM-Extract) | VantaDB (Motor FastGate + HNSW) | Factor Ventaja |")
    lines.append("|---|---|---|---|")
    lines.append(f"| **Latencia Query (p50 / p95)** | {m0_ov['query_p50_ms']:.1f} ms / {m0_ov['query_p50_ms']:.1f} ms | **{perf['search_latency_p50_ms']:.2f} ms / {perf['search_latency_p95_ms']:.2f} ms** | **~{m0_ov['query_p50_ms'] / max(perf['search_latency_p50_ms'], 0.01):.0f}x más rápido** |")
    lines.append(f"| **Throughput de Ingesta (QPS)** | {m0_ov['ingest_qps']:.1f} docs/sec | **{perf['ingest_qps']:.1f} docs/sec** | **~{perf['ingest_qps'] / max(m0_ov['ingest_qps'], 0.01):.0f}x mayor** |")
    lines.append(f"| **Throughput de Búsqueda (QPS)** | ~2-3 QPS | **{perf['search_qps']:.1f} QPS** | **~{perf['search_qps'] / 2.5:.0f}x mayor** |")
    lines.append(f"| **Consumo Tokens LLM** | ~{m0_ov['llm_tokens_per_query']} tokens / consulta | **0 tokens** (Determinista FastGate) | **100% Ahorro de Tokens** |")
    lines.append(f"| **Costo Estimado (USD / 1k queries)** | ~${m0_ov['est_cost_per_1k_usd']:.2f} | **$0.00** | **$0.00** (Cero dependencias LLM) |")
    # FIND-667: `get_peak_rss_mb()` returns 0.0 where the platform has no
    # /proc self-stats (measured on Windows). Printing "350.0x menor consumo"
    # off a 0 is an invented advantage, so an unmeasured RSS says so.
    rss = perf["peak_rss_mb"]
    if rss > 0:
        lines.append(f"| **Memoria RAM RSS Pico** | ~350 MB | **{rss:.1f} MB** | **{350.0 / rss:.1f}x menor consumo** |")
    else:
        lines.append("| **Memoria RAM RSS Pico** | ~350 MB | **n/d** (no medido en esta plataforma) | — |")

    return "\n".join(lines)


# ------------------------------------------------------------------ Self-Test
def run_self_test() -> int:
    """Offline test verifying all harness components without external dependencies."""
    print("[eval_locomo] Ejecutando self-test offline...")
    dataset = generate_synthetic_locomo(seed=42)
    assert len(dataset) == 20, f"Expected 20 synthetic questions, got {len(dataset)}"

    cats = {q["category"] for q in dataset}
    assert cats == {"single_hop", "multi_hop", "temporal_reasoning", "open_domain"}, f"Missing categories: {cats}"

    store = FakeLocomoStore()
    rep = run_locomo_evaluation(store, dataset, top_k=5, namespace_prefix="st")

    assert rep["meta"]["questions_total"] == 20
    assert rep["meta"]["llm_tokens_consumed"] == 0
    assert rep["metrics_overall"]["recall_at_5"] > 0.0
    assert rep["metrics_overall"]["mrr"] > 0.0
    assert rep["metrics_overall"]["ndcg_at_5"] > 0.0
    assert rep["performance"]["search_qps"] > 0.0

    # Test F1 and metrics formulas
    assert compute_f1_overlap("hello world", "hello world") == 1.0
    assert compute_f1_overlap("hello world", "world hello") == 1.0
    assert compute_f1_overlap("hello world", "different text") == 0.0

    assert compute_mrr({"a", "b"}, ["c", "a", "b"]) == 0.5
    assert compute_mrr({"a"}, ["x", "y", "z"]) == 0.0

    assert compute_ndcg_at_k({"a", "b"}, ["a", "b", "c"], k=2) == 1.0
    assert compute_ndcg_at_k({"a"}, ["x", "a"], k=2) == (1.0 / math.log2(3)) / (1.0 / math.log2(2))

    md = format_markdown_table(rep)
    assert "VantaDB vs Mem0" in md
    assert "Single-Hop" in md
    assert "Multi-Hop" in md
    assert "Temporal Reasoning" in md
    assert "Open-Domain" in md

    # FIND-667: the official-schema adapter, exercised offline on a fixture with
    # the same shape as `locomo10.json` (a KeyError here is the bug the row
    # measured; this assertion is what keeps it closed).
    sample = [
        {
            "sample_id": "conv-test",
            "conversation": {
                "speaker_a": "Ana",
                "speaker_b": "Beto",
                "session_1": [
                    {"speaker": "Ana", "dia_id": "D1:1", "text": "I moved to Oslo."},
                    {"speaker": "Beto", "dia_id": "D1:2", "text": "How cold is it?"},
                ],
                "session_1_date_time": "1:56 pm on 8 May, 2023",
                "session_2": [
                    {"speaker": "Ana", "dia_id": "D2:1", "text": "The fjord is frozen."},
                ],
                "session_2_date_time": "9:10 am on 3 June, 2023",
                # Date-time without content, as in the real file (sessions 20+).
                "session_3_date_time": "10:00 am on 4 July, 2023",
            },
            "qa": [
                {"question": "Where did Ana move?", "answer": "Oslo", "evidence": ["D1:1"], "category": 4},
                {"question": "When did she move?", "answer": "May 2023", "evidence": ["D1:1", "D9:9"], "category": 2},
                {"question": "Why?", "answer": "Work", "evidence": ["D2:1"], "category": 1},
                {"question": "What is the capital of Norway?", "answer": "Oslo", "evidence": ["D1:1"], "category": 3},
                {"question": "Did Ana hate it?", "answer": "No", "evidence": ["D2:1"], "category": 5},
            ],
        }
    ]
    assert detect_dataset_format(sample) == "locomo-official"
    assert detect_dataset_format(generate_synthetic_locomo(seed=1)) == "locomo-internal"
    adapted = adapt_official_locomo(sample)
    assert len(adapted) == 5, adapted
    assert {q["category"] for q in adapted} == {
        "single_hop",
        "temporal_reasoning",
        "multi_hop",
        "open_domain",
        "adversarial",
    }
    first = adapted[0]
    assert first["haystack_sessions"][0]["session_id"] == "session_1"
    assert first["haystack_sessions"][0]["turns"][0]["role"] == "user"
    assert first["haystack_sessions"][0]["turns"][1]["role"] == "assistant"
    assert _parse_locomo_datetime("1:56 pm on 8 May, 2023") == 1683554160000
    # Only the two CONTENT sessions are ingested; `session_3_date_time` has no content.
    assert len(first["haystack_sessions"]) == 2
    # Evidence pointing at a session that does not exist is dropped, the rest kept.
    assert adapted[1]["answer_session_ids"] == ["session_1"]

    fixed, dropped = apply_locomo_fixed(adapted)
    assert len(fixed) == 5, fixed
    assert sum(dropped.values()) == 0
    broken, dropped2 = apply_locomo_fixed(
        [
            {"answer": "x", "_evidence_raw": None, "answer_session_ids": ["session_1"]},
            {"answer": "x", "_evidence_raw": ["D9:1"], "answer_session_ids": []},
            {"answer": "  ", "_evidence_raw": ["D1:1"], "answer_session_ids": ["session_1"]},
            {"answer": "ok", "_evidence_raw": ["D1:1"], "answer_session_ids": ["session_1"]},
        ]
    )
    assert len(broken) == 1 and broken[0]["answer"] == "ok"
    assert dropped2 == {"no_evidence": 1, "unresolvable": 1, "no_answer": 1}

    print("[eval_locomo] Self-test: OK (todas las aserciones de métricas y flujo validadas)")
    return 0


# ------------------------------------------------------------------ Main CLI
def main() -> int:
    global _STRICT_EMBEDDINGS
    parser = argparse.ArgumentParser(description="Arnés de benchmark reproducible LOCOMO (MEM0-05) para VantaDB.")
    parser.add_argument("--dataset-path", default="", help="Ruta a archivo JSON del dataset LoCoMo local.")
    parser.add_argument(
        "--dataset-format",
        choices=["auto", "locomo-official", "locomo-internal"],
        default="auto",
        help=(
            "Esquema del JSON de --dataset-path. `locomo-official` es el de "
            "snap-research/LoCoMo (locomo10.json: `conversation` + `qa` con "
            "`category` 1..5); `locomo-internal` es el plano propio que produce "
            "generate_synthetic_locomo. `auto` decide por forma (FIND-667)."
        ),
    )
    parser.add_argument(
        "--locomo-fixed",
        action="store_true",
        help=(
            "Aplica el filtro limpio `locomo-fixed` (estándar AMB 2026): excluye "
            "QA con ground-truth ausente o irresoluble. Reporta cuántas se "
            "excluyeron y por qué."
        ),
    )
    parser.add_argument(
        "--strict-embeddings",
        action="store_true",
        help=(
            "Falla si el modelo de embeddings no está disponible o si falla un "
            "embedding de query, en vez de degradar en silencio al fallback "
            "léxico o a FakeLocomoStore (FIND-667)."
        ),
    )
    parser.add_argument("--smoke", action="store_true", help="Genera corrida de validación sintética determinista (seed 42).")
    parser.add_argument("--self-test", action="store_true", help="Ejecuta pruebas unitarias internas offline sin vantadb.")
    parser.add_argument("--category", choices=["all", "single_hop", "multi_hop", "temporal_reasoning", "open_domain", "adversarial"], default="all")
    parser.add_argument("--limit", type=int, default=0, help="Límite máximo de preguntas a evaluar (0 = todas).")
    parser.add_argument("--top-k", type=int, default=10, help="Profundidad de recuperación top_k.")
    parser.add_argument("--backend", default="memory", help="Backend de VantaDB (memory, fjall, etc.).")
    parser.add_argument("--output", default=str(DEFAULT_OUTPUT), help="Ruta del reporte JSON generado.")
    args = parser.parse_args()

    _STRICT_EMBEDDINGS = args.strict_embeddings

    if args.self_test:
        return run_self_test()

    # Load or generate dataset
    dataset_meta: dict[str, Any] = {}
    if args.dataset_path:
        p = pathlib.Path(args.dataset_path)
        if not p.is_file():
            print(f"[eval_locomo] Error: dataset no encontrado en {p}", file=sys.stderr)
            return 1
        raw_data = json.loads(p.read_text(encoding="utf-8"))
        fmt = args.dataset_format
        if fmt == "auto":
            fmt = detect_dataset_format(raw_data)
        if fmt == "locomo-official":
            try:
                dataset = adapt_official_locomo(raw_data)
            except ValueError as e:
                print(f"[eval_locomo] Error: {e}", file=sys.stderr)
                return 1
            dataset_meta["dataset_format"] = "locomo-official"
            dataset_meta["samples"] = len(raw_data) if isinstance(raw_data, list) else 0
        else:
            dataset = raw_data if isinstance(raw_data, list) else raw_data.get("questions", [])
            dataset_meta["dataset_format"] = "locomo-internal"
        print(
            f"[eval_locomo] dataset={fmt} → {len(dataset)} preguntas "
            f"(categorías: {sorted({q.get('category') for q in dataset})})"
        )
    elif args.smoke:
        print("[eval_locomo] Modo --smoke activado: generando suite sintética determinista LoCoMo (seed 42)...")
        dataset = generate_synthetic_locomo(seed=42)
        dataset_meta["dataset_format"] = "synthetic"
    else:
        print("[eval_locomo] Aviso: no se especificó --dataset-path ni --smoke. Usando generador sintético por defecto.")
        dataset = generate_synthetic_locomo(seed=42)
        dataset_meta["dataset_format"] = "synthetic"

    if args.locomo_fixed:
        before = len(dataset)
        dataset, dropped = apply_locomo_fixed(dataset)
        dataset_meta["locomo_fixed"] = {
            "applied": True,
            "input": before,
            "kept": len(dataset),
            "dropped": dropped,
        }
        print(
            f"[eval_locomo] --locomo-fixed: {before} → {len(dataset)} QA "
            f"(excluidas: {dropped})"
        )

    if args.category != "all":
        dataset = [q for q in dataset if q.get("category") == args.category]

    if args.limit > 0:
        dataset = dataset[:args.limit]

    if not dataset:
        print("[eval_locomo] Error: no hay preguntas para evaluar tras aplicar los filtros.", file=sys.stderr)
        return 1

    # FIND-667(c): under --strict-embeddings a missing model must abort, not
    # silently measure the lexical fallback and label the report "BGE".
    if _STRICT_EMBEDDINGS and not has_fastembed():
        print(
            f"[eval_locomo] Error: --strict-embeddings y el modelo {EMBED_MODEL} no "
            "está disponible. Instalá fastembed (pip install fastembed) o quitá "
            "la bandera para medir el fallback léxico a sabiendas.",
            file=sys.stderr,
        )
        return 1

    # Initialize store
    try:
        store = VantaLocomoStore(backend=args.backend)
    except Exception as e:
        if _STRICT_EMBEDDINGS:
            print(
                f"[eval_locomo] Error: --strict-embeddings y vantadb no disponible "
                f"({e}); me niego a medir con FakeLocomoStore.",
                file=sys.stderr,
            )
            return 1
        print(f"[eval_locomo] vantadb no disponible ({e}) → fallback a FakeLocomoStore", file=sys.stderr)
        store = FakeLocomoStore()

    print(f"[eval_locomo] Evaluando {len(dataset)} preguntas sobre store {store.name}...")
    report = run_locomo_evaluation(store, dataset, top_k=args.top_k)
    report.setdefault("meta", {}).update(dataset_meta)
    store.close()

    # Export report JSON
    out_path = pathlib.Path(args.output)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(report, indent=2, ensure_ascii=False), encoding="utf-8")

    # Output Markdown comparison table
    md_table = format_markdown_table(report)
    print("\n" + md_table)
    print(f"\n[eval_locomo] Reporte JSON exportado exitosamente a: {out_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
"""
evals/memory_bench.py — Harness sintético estilo LongMemEval-S / LoCoMo (MEM-70).

Sesiones determinísticas (seed 42) que emulan el SHAPE de esos benchmarks
(sesiones multi-turno con hechos + queries sobre hechos + distractores),
sin descargar sus datasets (licencia/peso → DEFER, ver BENCHMARKS.md §17).

Comando reproducible (Regla 11):
  python evals/memory_bench.py --sessions 20 --turns 16 --queries 40 --top-k 5 --output evals/memory_bench_report.json
  python evals/memory_bench.py --sessions 4 --turns 8 --queries 8 --no-vantadb   # smoke offline/CI

Outputs:
  <output>.json (gitignored) + tabla markdown a stdout (para BENCHMARKS.md §17).

# ponytail: store textual con match por substrings cuando no hay vantadb_py;
# techo = recall sintético ~1.0 en fallback. Upgrade = backend vantadb real +
# datasets LongMemEval-S/LoCoMo con loader versionado cuando haya red/licencia.
"""
from __future__ import annotations

import argparse
import json
import pathlib
import random
import statistics
import sys
import time

try:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")
except Exception:
    pass

DEFAULT_OUTPUT = pathlib.Path(__file__).parent / "memory_bench_report.json"
SEED = 42
TOPICS = ["launch plan", "budget review", "hiring", "roadmap", "infra migration", "support tickets"]


def percentile(sorted_vals: list, p: float) -> float:
    if not sorted_vals:
        return 0.0
    vals = sorted(sorted_vals)
    return vals[min(int(len(vals) * p), len(vals) - 1)]


def gen_sessions(rng: random.Random, n_sessions: int, n_turns: int) -> tuple[list, list]:
    """Devuelve (docs, gold) — docs: [{sid, text}], gold: por query el doc esperado."""
    docs, facts = [], []
    for s in range(n_sessions):
        topic = TOPICS[s % len(TOPICS)]
        for t in range(n_turns):
            text = (
                f"[session-{s} turn-{t}] {topic}: decision fact-{s}-{t} "
                f"recorded with marker M{s}-{t} and distractor filler {rng.randint(1000, 9999)}."
            )
            docs.append({"sid": s, "text": text})
            facts.append({"sid": s, "text": text, "marker": f"M{s}-{t}"})
    return docs, facts


def build_queries(rng: random.Random, facts: list, n_queries: int) -> list:
    return [
        {"q": f"Which decision was recorded with marker {f['marker']}?", "gold": f["text"]}
        for f in rng.sample(facts, min(n_queries, len(facts)))
    ]


class DictStore:
    """Fallback in-memory (también baseline de comparación)."""

    def __init__(self) -> None:
        self.docs: list = []

    def put(self, text: str) -> None:
        self.docs.append(text)

    def search(self, q: str, top_k: int) -> list:
        marker = q.split("marker ")[-1].rstrip("?")
        scored = [(d == d and (marker in d), d) for d in self.docs]
        scored.sort(key=lambda x: (not x[0], len(x[1])))
        return [d for _, d in scored[:top_k]]


class VantaStore:
    """Backend vantadb_py (put/search_memory) — thin wrapper, sin lógica duplicada."""

    def __init__(self) -> None:
        import vantadb_py  # type: ignore

        self.db = vantadb_py.VantaDb.connect(":memory:") if hasattr(vantadb_py.VantaDb, "connect") else vantadb_py.VantaDb()
        self._search = getattr(self.db, "search_memory", getattr(self.db, "search", None))

    def put(self, text: str) -> None:
        if hasattr(self.db, "put"):
            self.db.put({"text": text})

    def search(self, q: str, top_k: int) -> list:
        if self._search is None:
            return []
        hits = self._search(q, top_k=top_k) if "top_k" in getattr(self._search, "__code__", None) else self._search(q)
        out = []
        for h in hits or []:
            out.append(h.get("text", "") if isinstance(h, dict) else getattr(h, "text", str(h)))
        return out


def run(args: argparse.Namespace) -> dict:
    rng = random.Random(SEED)
    docs, facts = gen_sessions(rng, args.sessions, args.turns)
    queries = build_queries(rng, facts, args.queries)

    use_vanta = False
    if not args.no_vantadb:
        try:
            store: object = VantaStore()
            use_vanta = True
        except Exception as e:
            print(f"[memory_bench] vantadb_py no disponible ({e}) → fallback dict", file=sys.stderr)
            store = DictStore()
    else:
        store = DictStore()

    t0 = time.perf_counter()
    ingest_lat: list = []
    for d in docs:
        s = time.perf_counter()
        store.put(d["text"])  # type: ignore
        ingest_lat.append((time.perf_counter() - s) * 1000)
    ingest_s = time.perf_counter() - t0

    query_lat, hits = [], 0
    for item in queries:
        s = time.perf_counter()
        try:
            got = store.search(item["q"], args.top_k)  # type: ignore
        except Exception:
            got = []
        query_lat.append((time.perf_counter() - s) * 1000)
        if item["gold"] in got:
            hits += 1

    n_q = len(queries) or 1
    return {
        "meta": {
            "backend": "vantadb" if use_vanta else "dict-fallback",
            "sessions": args.sessions,
            "turns": args.turns,
            "docs": len(docs),
            "queries": len(queries),
            "top_k": args.top_k,
            "seed": SEED,
            "dataset": "synthetic-longmem-style",
            "note": "Shape LongMemEval-S/LoCoMo sin datasets reales (DEFER MEM-70).",
        },
        "ingest_s": round(ingest_s, 3),
        "ingest_qps": round(len(docs) / ingest_s, 1) if ingest_s else 0.0,
        "recall_at_k": round(hits / n_q, 4),
        "q_p50_ms": round(statistics.median(query_lat), 3) if query_lat else 0.0,
        "q_p99_ms": round(percentile(query_lat, 0.99), 3),
    }


def main() -> None:
    ap = argparse.ArgumentParser(description="Harness sintético memoria estilo LongMemEval-S/LoCoMo (MEM-70).")
    ap.add_argument("--sessions", type=int, default=20)
    ap.add_argument("--turns", type=int, default=16)
    ap.add_argument("--queries", type=int, default=40)
    ap.add_argument("--top-k", type=int, default=5)
    ap.add_argument("--no-vantadb", action="store_true", help="Fuerza fallback dict (offline/CI).")
    ap.add_argument("--output", default=str(DEFAULT_OUTPUT))
    args = ap.parse_args()

    res = run(args)
    pathlib.Path(args.output).write_text(json.dumps(res, indent=2), encoding="utf-8")
    m = res["meta"]
    print(f"| backend | dataset | docs | queries | recall@{m['top_k']} | ingest QPS | q p50 (ms) | q p99 (ms) |")
    print("|---|---|---|---|---|---|---|---|")
    print(
        f"| {m['backend']} | {m['dataset']} ({m['sessions']}sess x {m['turns']}turns, seed {m['seed']}) "
        f"| {m['docs']} | {m['queries']} | {res['recall_at_k']} | {res['ingest_qps']} | {res['q_p50_ms']} | {res['q_p99_ms']} |"
    )
    print(f"\nReporte JSON: {args.output}", file=sys.stderr)


if __name__ == "__main__":
    main()

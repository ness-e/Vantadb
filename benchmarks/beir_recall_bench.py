#!/usr/bin/env python3
"""
benchmarks/beir_recall_bench.py — BEIR/MTEB recall@k vs sqlite-vec (BENCH-02).

Measures end-to-end retrieval quality on a REAL text-retrieval dataset:
recall@k against human relevance judgments (qrels) — MTEB/TREC semantics,
recall@k = |top-k ∩ relevant| / |relevant| — on BEIR SciFact (test split),
with the SAME embeddings for every engine (all-MiniLM-L6-v2, ONNX local,
mean-pool + L2 norm). nDCG@10 is also reported: it is MTEB's main retrieval
metric, which makes the result cross-checkable against the published MTEB
reference for this model on SciFact (nDCG@10 = 64.51, MTEB paper Table 11 —
for binary qrels, linear and exponential gain conventions coincide).

Not to be confused with benchmarks/competitive_bench.py: that harness measures
index fidelity + speed on vector datasets (its `recall_at_k` compares
approximate search against exact kNN). This harness measures retrieval quality
vs qrels and ALSO reports index fidelity vs exact kNN as a secondary number.

Reproduce (Regla 11):
  .venv/Scripts/python benchmarks/beir_recall_bench.py \
      --dataset scifact --split test --model all-MiniLM-L6-v2 \
      --k 10 --engines vanta,sqlite-vec --seed 42

Self-test (offline — no network, no models, no engine):
  .venv/Scripts/python benchmarks/beir_recall_bench.py --self-test

Dataset: mteb/scifact on HuggingFace (BEIR SciFact: corpus 5,183 / 300 test
queries / 339 qrels rows). Cached under benchmarks/datasets/scifact/ (gitignored).
The canonical UKP host (public.ukp.informatik.tu-darmstadt.de) was unreachable
from the dev network at implementation time (2026-10-06); the HF mirror carries
the identical files (ids verified: 300/300 query-ids, 283/283 corpus-ids match).

Methodology note: sqlite-vec (vec0) is an exact brute-force scan. VantaDB also
routes to its FLAT exact scan at this corpus size: `flat_threshold` defaults to
10,000 nodes (src/config.rs:347) and `use_flat_search()` selects the flat path
when `nodes.len() <= threshold` (src/index/search/neighbors.rs). So at 5,183
nodes both engines are exact and the comparison measures retrieval quality of
the declared embedding + ranking stack, not ANN fidelity. Measuring the HNSW
graph itself requires a corpus above `flat_threshold` or
`VANTADB_FLAT_THRESHOLD=0` (set to 0 to disable the flat path — see
src/config.rs:903). The JSON report records the effective search mode.
"""

from __future__ import annotations

import argparse
import gc
import json
import os
import platform
import random
import shutil
import sqlite3
import statistics
import sys
import tempfile
import time
import urllib.request

# ---------------------------------------------------------------------------
# 0. Dependencies (clear message instead of an import traceback)
# ---------------------------------------------------------------------------

MISSING_DEPS = []
for dep in ["numpy", "onnxruntime", "tokenizers", "psutil"]:
    try:
        __import__(dep)
    except ImportError:
        MISSING_DEPS.append(dep)

if MISSING_DEPS:
    print("ERROR: missing Python dependencies for this benchmark:")
    for dep in MISSING_DEPS:
        print(f"  - {dep}")
    print("\nInstall (benchmarks/requirements.txt):")
    print("  pip install numpy onnxruntime tokenizers psutil sqlite-vec")
    sys.exit(1)

import numpy as np  # noqa: E402
import psutil  # noqa: E402

try:
    import sqlite_vec  # noqa: E402
    HAS_SQLITE_VEC = True
except ImportError:
    HAS_SQLITE_VEC = False

# ---------------------------------------------------------------------------
# 1. Dataset registry (BEIR via HuggingFace mirror)
# ---------------------------------------------------------------------------

DATASETS = {
    "scifact": {
        "hf_base": "https://huggingface.co/datasets/mteb/scifact/resolve/main",
        "corpus": "corpus.jsonl",
        "queries": "queries.jsonl",
        "qrels": {"test": "qrels/test.jsonl"},
        "license": "cc-by-nc-4.0",
        "reference": "https://github.com/allenai/scifact",
    },
}

MODELS = {
    "all-MiniLM-L6-v2": {
        # rev = manifest.json rev for sentence-transformers/all-MiniLM-L6-v2
        "dir": os.path.join("embeddings", "models", "all-MiniLM-L6-v2"),
        "onnx": os.path.join("onnx", "model.onnx"),
        "tokenizer": "tokenizer.json",
        "rev": "1110a24",
        "dim": 384,
        "max_seq": 256,
    },
}


# ---------------------------------------------------------------------------
# 2. Download / cache helpers
# ---------------------------------------------------------------------------
def _download(url, path):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    print(f"Downloading {url}\n  -> {path}")

    def progress(block_num, block_size, total_size):
        if total_size > 0:
            pct = min(100.0, block_num * block_size * 100.0 / total_size)
            sys.stdout.write(f"\r  {pct:5.1f}%")
            sys.stdout.flush()

    urllib.request.urlretrieve(url, path, progress)
    print()


def ensure_dataset(dataset, split, dataset_dir):
    spec = DATASETS[dataset]
    if split not in spec["qrels"]:
        raise SystemExit(f"ERROR: split '{split}' not available for '{dataset}' (have: {list(spec['qrels'])})")
    base = os.path.join(dataset_dir, dataset)
    paths = {
        "corpus": os.path.join(base, spec["corpus"]),
        "queries": os.path.join(base, spec["queries"]),
        "qrels": os.path.join(base, os.path.basename(spec["qrels"][split])),
    }
    remote = {
        "corpus": f"{spec['hf_base']}/{spec['corpus']}",
        "queries": f"{spec['hf_base']}/{spec['queries']}",
        "qrels": f"{spec['hf_base']}/{spec['qrels'][split]}",
    }
    for key, path in paths.items():
        if not os.path.exists(path):
            _download(remote[key], path)
    return paths


def load_jsonl(path):
    rows = []
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            line = line.strip()
            if line:
                rows.append(json.loads(line))
    return rows


def load_beir(dataset, split, dataset_dir):
    """Returns corpus (list of dicts), queries (list), qrels {qid: set(cids)}, sha256 per file."""
    paths = ensure_dataset(dataset, split, dataset_dir)
    corpus = load_jsonl(paths["corpus"])
    queries = load_jsonl(paths["queries"])
    qrels_rows = load_jsonl(paths["qrels"])
    qrels = {}
    for row in qrels_rows:
        if float(row.get("score", 1)) > 0:
            qrels.setdefault(str(row["query-id"]), set()).add(str(row["corpus-id"]))
    # evaluation set = queries that have judgments (BEIR/MTEB convention)
    qrels = {qid: rel for qid, rel in qrels.items() if rel}
    queries_eval = [q for q in queries if str(q["_id"]) in qrels]
    digests = {key: _sha256(path) for key, path in paths.items()}
    return corpus, queries_eval, qrels, digests


def _sha256(path):
    import hashlib
    h = hashlib.sha256()
    with open(path, "rb") as fh:
        for chunk in iter(lambda: fh.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def sample_queries(queries, limit, seed):
    """Deterministic subsample (seed) — the declared seed has a real effect here."""
    if limit <= 0 or limit >= len(queries):
        return queries, 0
    rng = random.Random(seed)
    idx = sorted(rng.sample(range(len(queries)), limit))
    return [queries[i] for i in idx], limit


# ---------------------------------------------------------------------------
# 3. Embeddings — all-MiniLM-L6-v2 ONNX (no torch)
# ---------------------------------------------------------------------------

def load_embedder(model_id, root):
    spec = MODELS[model_id]
    model_dir = os.path.join(root, spec["dir"])
    onnx_path = os.path.join(model_dir, spec["onnx"])
    tok_path = os.path.join(model_dir, spec["tokenizer"])
    if not os.path.exists(onnx_path) or not os.path.exists(tok_path):
        raise SystemExit(
            f"ERROR: model artifacts not found under {model_dir}.\n"
            "Download them with: python embeddings/download.py (see embeddings/README.md)"
        )
    import onnxruntime as ort
    import tokenizers

    sess = ort.InferenceSession(onnx_path, providers=["CPUExecutionProvider"])
    tok = tokenizers.Tokenizer.from_file(tok_path)
    tok.enable_truncation(max_length=spec["max_seq"])
    tok.enable_padding()
    return sess, tok, spec


def embed_texts(sess, tok, texts, dim, batch_size=64):
    """Mean-pool + L2 normalize. Returns float32 (n, dim) with cosine geometry."""
    out = np.empty((len(texts), dim), dtype=np.float32)
    for i in range(0, len(texts), batch_size):
        batch = texts[i : i + batch_size]
        enc = tok.encode_batch(batch)
        ids = np.array([e.ids for e in enc], dtype=np.int64)
        mask = np.array([e.attention_mask for e in enc], dtype=np.int64)
        tt = np.array([e.type_ids for e in enc], dtype=np.int64)
        hidden = sess.run(None, {"input_ids": ids, "attention_mask": mask, "token_type_ids": tt})[0]
        m = mask[..., None].astype(np.float32)
        pooled = (hidden * m).sum(1) / np.clip(m.sum(1), 1e-9, None)
        norms = np.linalg.norm(pooled, axis=1, keepdims=True)
        out[i : i + len(batch)] = (pooled / np.clip(norms, 1e-12, None)).astype(np.float32)
        sys.stdout.write(f"\r  embedded {min(i + batch_size, len(texts))}/{len(texts)}")
        sys.stdout.flush()
    print()
    return out


def doc_text(doc):
    """BEIR convention: title + text."""
    title = (doc.get("title") or "").strip()
    text = (doc.get("text") or "").strip()
    return (title + " " + text).strip() if title else text


# ---------------------------------------------------------------------------
# 4. Retrieval engines — each returns (predictions, query_ms, ingest_s)
# ---------------------------------------------------------------------------

def exact_knn(corpus_vecs, query_vecs, top_k):
    """Exact cosine kNN via numpy (vectors are L2-normalized). The ceiling row.

    Per-query latency IS measured (brute-force reference); there is no index to
    build, so ingest_s is 0. k is clamped to the corpus size (argpartition would
    raise otherwise).
    """
    k_eff = min(top_k, len(corpus_vecs))
    preds, query_ms = [], []
    for q in query_vecs:
        t1 = time.perf_counter()
        sims = corpus_vecs @ q
        idx = np.argpartition(-sims, k_eff - 1)[:k_eff]
        preds.append(idx[np.argsort(-sims[idx])].tolist())
        query_ms.append((time.perf_counter() - t1) * 1000.0)
    return preds, query_ms, 0.0


def bench_vanta(corpus_vecs, query_vecs, top_k, workdir, batch_size=999):
    """VantaDB via Python SDK. Keys are doc-<corpus_index>."""
    import vantadb

    db_path = os.path.join(workdir, "vanta_db")
    if os.path.exists(db_path):
        shutil.rmtree(db_path, ignore_errors=True)
    namespace = "beir-bench"
    keys = [f"doc-{i}" for i in range(len(corpus_vecs))]
    payloads = [""] * len(corpus_vecs)
    metadatas = [{"idx": i} for i in range(len(corpus_vecs))]

    t0 = time.perf_counter()
    db = vantadb.Client(db_path)
    chunk = max(1, min(batch_size, 999))  # < 1000: InsertMode::Auto incremental threshold
    for i in range(0, len(corpus_vecs), chunk):
        db.put_batch_raw(
            vectors=corpus_vecs[i : i + chunk],
            keys=keys[i : i + chunk],
            payloads=payloads[i : i + chunk],
            metadatas=metadatas[i : i + chunk],
            namespaces=[namespace] * len(keys[i : i + chunk]),
        )
    db.flush()
    db.rebuild_index()
    ingest_s = time.perf_counter() - t0

    query_ms, preds = [], []
    for q in query_vecs:
        t1 = time.perf_counter()
        hits = db.search(namespace=namespace, query_vector=q.tolist(), top_k=top_k, distance_metric="cosine")
        query_ms.append((time.perf_counter() - t1) * 1000.0)
        row = []
        for hit in hits:
            try:
                key = hit.key if hasattr(hit, "key") else hit.get("key", "")
                row.append(int(str(key).split("-")[1]))
            except (ValueError, IndexError, AttributeError):
                continue
        preds.append(row)
    db.close()
    shutil.rmtree(db_path, ignore_errors=True)
    return preds, query_ms, ingest_s


def bench_sqlite_vec(corpus_vecs, query_vecs, top_k, workdir):
    """sqlite-vec vec0 (exact scan). rowid = corpus_index + 1."""
    if not HAS_SQLITE_VEC:
        raise SystemExit("ERROR: sqlite-vec not installed. pip install sqlite-vec")
    db_path = os.path.join(workdir, "sqlite_vec.db")
    if os.path.exists(db_path):
        os.remove(db_path)
    db = sqlite3.connect(db_path)
    db.enable_load_extension(True)
    sqlite_vec.load(db)
    db.enable_load_extension(False)

    dim = corpus_vecs.shape[1]
    t0 = time.perf_counter()
    db.execute(f"CREATE VIRTUAL TABLE vec_items USING vec0(embedding float[{dim}] distance_metric=cosine)")
    for i, vec in enumerate(corpus_vecs):
        db.execute("INSERT INTO vec_items(rowid, embedding) VALUES (?, ?)", (i + 1, vec))
    db.commit()
    ingest_s = time.perf_counter() - t0

    query_ms, preds = [], []
    for q in query_vecs:
        t1 = time.perf_counter()
        rows = db.execute(
            "SELECT rowid FROM vec_items WHERE embedding MATCH ? ORDER BY distance LIMIT ?",
            (q, top_k),
        ).fetchall()
        query_ms.append((time.perf_counter() - t1) * 1000.0)
        preds.append([int(r[0]) - 1 for r in rows])
    db.close()
    os.remove(db_path)
    return preds, query_ms, ingest_s


# ---------------------------------------------------------------------------
# 5. Metrics — recall@k vs qrels (MTEB/TREC) + index fidelity vs exact kNN
# ---------------------------------------------------------------------------

def recall_at_k(preds, relevant_per_query, k):
    """MTEB/TREC recall@k = |top-k ∩ relevant| / |relevant|, averaged over queries.

    `relevant_per_query` is a list of sets aligned with `preds`; a query with an
    empty relevant set is skipped (denominator 0 — BEIR has none, defensive only).
    """
    vals = []
    for pred, relevant in zip(preds, relevant_per_query):
        if not relevant:
            continue
        hit = len(set(pred[:k]) & relevant)
        vals.append(hit / len(relevant))
    return float(np.mean(vals)) if vals else 0.0


def index_recall_at_k(preds, exact_preds, k):
    """Fidelity of an approximate index vs exact kNN (same metric competitive_bench.py reports)."""
    vals = []
    for pred, exact in zip(preds, exact_preds):
        vals.append(len(set(pred[:k]) & set(exact[:k])) / k)
    return float(np.mean(vals)) if vals else 0.0


def ndcg_at_k(preds, relevant_per_query, k):
    """nDCG@k with binary gains (MTEB/trec_eval semantics for binary qrels).

    DCG@k = sum_{i=1..k} rel_i / log2(i+1); IDCG@k = same over the ideal ranking.
    For binary relevance, gain = rel ∈ {0,1}, so linear and exponential gain
    conventions coincide. MTEB's main metric for retrieval (and SciFact).
    """
    vals = []
    for pred, relevant in zip(preds, relevant_per_query):
        if not relevant:
            continue
        dcg = sum(1.0 / np.log2(i + 2) for i, doc in enumerate(pred[:k]) if doc in relevant)
        idcg = sum(1.0 / np.log2(i + 2) for i in range(min(len(relevant), k)))
        vals.append(dcg / idcg if idcg > 0 else 0.0)
    return float(np.mean(vals)) if vals else 0.0


def summarize(name, preds, query_ms, ingest_s, relevant, exact_preds, k_values, top_k):
    return {
        "engine": name,
        "ingest_s": round(ingest_s, 3),
        "query_p50_ms": round(statistics.median(query_ms), 3),
        "query_p99_ms": round(float(np.percentile(query_ms, 99)), 3),
        "recall_at_k": {str(k): round(recall_at_k(preds, relevant, k), 4) for k in k_values},
        "ndcg_at_10": round(ndcg_at_k(preds, relevant, 10), 4),
        "index_recall_at_k": {str(k): round(index_recall_at_k(preds, exact_preds, k), 4) for k in k_values if k <= top_k},
        "retrieved_per_query": round(float(np.mean([len(p) for p in preds])), 1),
    }


# ---------------------------------------------------------------------------
# 6. Self-test (offline — recall math + parser + sampler, no network/models)
# ---------------------------------------------------------------------------

def run_self_test():
    failures = []
    total = 0

    def check(label, cond, detail=""):
        nonlocal total
        total += 1
        status = "PASS" if cond else "FAIL"
        print(f"  [{status}] {label} {detail}")
        if not cond:
            failures.append(label)

    print("=" * 60)
    print("  beir_recall_bench self-test (offline)                   ")
    print("=" * 60)

    # 1. recall@k math — hand-computed expectations
    preds = [[1, 2, 3], [9, 8, 7]]
    relevant = [{1, 3}, {7}]
    check("recall@1 = (1/2 + 0/1)/2 = 0.25", abs(recall_at_k(preds, relevant, 1) - 0.25) < 1e-9)
    check("recall@2 = (1/2 + 0/1)/2 = 0.25", abs(recall_at_k(preds, relevant, 2) - 0.25) < 1e-9)
    check("recall@3 = (2/2 + 1/1)/2 = 1.0", abs(recall_at_k(preds, relevant, 3) - 1.0) < 1e-9)
    check("empty relevant skipped (no div0)", recall_at_k([[1]], [set()], 1) == 0.0)

    # 2. index fidelity + nDCG@10 (binary qrels — MTEB main metric)
    exact = [[1, 2, 3], [5, 6, 7]]
    check("index_recall@1 = 0.5", abs(index_recall_at_k(preds, exact, 1) - 0.5) < 1e-9)
    check("index_recall@2 = (2/2 + 0/2)/2 = 0.5", abs(index_recall_at_k(preds, exact, 2) - 0.5) < 1e-9)
    # nDCG@3 q1: DCG=1+0+1/2=1.5, IDCG=1+1/log2(3)=1.63093 -> 0.91972; q2: DCG=1/log2(4)=0.5, IDCG=1 -> 0.5
    check("ndcg@3 = (0.91972 + 0.5)/2 = 0.70986", abs(ndcg_at_k(preds, relevant, 3) - 0.70986) < 1e-4)
    check("ndcg@3 perfect ranking = 1.0", abs(ndcg_at_k([[1, 3], [7]], relevant, 3) - 1.0) < 1e-9)

    # 3. qrels parsing from JSONL fixtures (BEIR schema)
    tmp = tempfile.mkdtemp(prefix="beir_selftest_")
    try:
        corpus_p = os.path.join(tmp, "corpus.jsonl")
        queries_p = os.path.join(tmp, "queries.jsonl")
        qrels_p = os.path.join(tmp, "qrels.jsonl")
        with open(corpus_p, "w", encoding="utf-8") as fh:
            fh.write(json.dumps({"_id": "1", "title": "T", "text": "doc one"}) + "\n")
            fh.write(json.dumps({"_id": "2", "title": "", "text": "doc two"}) + "\n")
        with open(queries_p, "w", encoding="utf-8") as fh:
            fh.write(json.dumps({"_id": "10", "text": "claim a"}) + "\n")
            fh.write(json.dumps({"_id": "11", "text": "claim b (no qrels)"}) + "\n")
        with open(qrels_p, "w", encoding="utf-8") as fh:
            fh.write(json.dumps({"query-id": "10", "corpus-id": "1", "score": "1"}) + "\n")
            fh.write(json.dumps({"query-id": "10", "corpus-id": "2", "score": "0"}) + "\n")
        qrels_rows = load_jsonl(qrels_p)
        qrels = {}
        for row in qrels_rows:
            if float(row.get("score", 1)) > 0:
                qrels.setdefault(str(row["query-id"]), set()).add(str(row["corpus-id"]))
        check("qrels: score=0 excluded", qrels == {"10": {"1"}}, f"got={qrels}")
        check("doc_text: title+text", doc_text({"title": "T", "text": "doc one"}) == "T doc one")
        check("doc_text: text-only", doc_text({"title": "", "text": "doc two"}) == "doc two")

        # 4. deterministic sampling with the declared seed
        queries = [{"_id": str(i)} for i in range(20)]
        s1, n1 = sample_queries(queries, 5, seed=42)
        s2, n2 = sample_queries(queries, 5, seed=42)
        s3, _ = sample_queries(queries, 5, seed=43)
        check("sampler: seed 42 deterministic", [q["_id"] for q in s1] == [q["_id"] for q in s2])
        check("sampler: seed 43 differs", [q["_id"] for q in s1] != [q["_id"] for q in s3])
        check("sampler: limit respected", len(s1) == 5 and n1 == 5)
        check("sampler: limit>=total returns all", len(sample_queries(queries, 0, 42)[0]) == 20)
        check("sampler: negative limit returns all", len(sample_queries(queries, -5, 42)[0]) == 20)
    finally:
        shutil.rmtree(tmp, ignore_errors=True)

    print(f"\nSelf-test: {total - len(failures)}/{total} PASS")
    return 0 if not failures else 1


# ---------------------------------------------------------------------------
# 7. Main
# ---------------------------------------------------------------------------

def main():
    parser = argparse.ArgumentParser(description="BEIR/MTEB recall@k vs sqlite-vec (BENCH-02)")
    parser.add_argument("--dataset", type=str, default="scifact", help="BEIR dataset (available: scifact)")
    parser.add_argument("--split", type=str, default="test", help="Dataset split (scifact: test)")
    parser.add_argument("--model", type=str, default="all-MiniLM-L6-v2", help="Embedding model (local ONNX)")
    parser.add_argument("--k", type=int, default=10, help="Headline k for the summary table (recall@k always reports 1/10/100)")
    parser.add_argument("--top-k", type=int, default=100, help="Retrieval depth per query")
    parser.add_argument("--engines", type=str, default="vanta,sqlite-vec", help="Comma-separated: vanta,sqlite-vec")
    parser.add_argument("--limit-queries", type=int, default=0, help="Subsample N queries (0 = all; sampled with --seed)")
    parser.add_argument("--seed", type=int, default=42, help="Sampling seed (declared in the reproducible command)")
    parser.add_argument("--dataset-dir", type=str, default=os.path.join("benchmarks", "datasets"), help="Dataset cache dir")
    parser.add_argument("--json-output", type=str, default=os.path.join("benchmarks", "beir_recall_report.json"), help="JSON report path")
    parser.add_argument("--batch-size", type=int, default=64, help="Embedding batch size")
    parser.add_argument("--self-test", action="store_true", help="Offline self-test (recall math + parser + sampler) and exit")
    args = parser.parse_args()

    if args.self_test:
        sys.exit(run_self_test())

    if args.dataset not in DATASETS:
        raise SystemExit(f"ERROR: unknown dataset '{args.dataset}' (available: {list(DATASETS)})")
    if args.model not in MODELS:
        raise SystemExit(f"ERROR: unknown model '{args.model}' (available: {list(MODELS)})")

    requested = [e.strip().lower() for e in args.engines.split(",") if e.strip()]
    engines = []
    for name in requested:
        if name in ("vanta", "sqlite-vec"):
            if name == "sqlite-vec" and not HAS_SQLITE_VEC:
                print("[SKIP] sqlite-vec not installed (pip install sqlite-vec)")
                continue
            engines.append(name)
        else:
            print(f"[WARN] unknown engine '{name}' — skipping")

    print("=" * 60)
    print("   BEIR/MTEB recall@k benchmark (BENCH-02)                 ")
    print("=" * 60)
    print(f"Dataset  : {args.dataset} (split={args.split})")
    print(f"Model    : {args.model} (ONNX local)")
    print(f"Engines  : {', '.join(engines) or '(none)'}")
    print(f"Seed     : {args.seed} | top-k: {args.top_k} | limit-queries: {args.limit_queries or 'all'}")
    print("=" * 60)

    corpus, queries, qrels, digests = load_beir(args.dataset, args.split, args.dataset_dir)
    queries, _ = sample_queries(queries, args.limit_queries, args.seed)
    relevant = [qrels[str(q["_id"])] for q in queries]
    corpus_ids = [str(d["_id"]) for d in corpus]
    id_to_idx = {cid: i for i, cid in enumerate(corpus_ids)}
    # map qrels ids -> corpus indices (all present by construction; defensive filter)
    relevant_idx = [set(id_to_idx[c] for c in rel if c in id_to_idx) for rel in relevant]

    print(f"Corpus: {len(corpus)} docs | Eval queries: {len(queries)} | "
          f"Relevant per query: {np.mean([len(r) for r in relevant_idx]):.2f} avg")

    sess, tok, model_spec = load_embedder(args.model, root=".")
    print("Embedding corpus...")
    corpus_vecs = embed_texts(sess, tok, [doc_text(d) for d in corpus], model_spec["dim"], args.batch_size)
    print("Embedding queries...")
    query_vecs = embed_texts(sess, tok, [str(q["text"]) for q in queries], model_spec["dim"], args.batch_size)

    k_values = sorted({1, 10, 100, args.k})
    workdir = tempfile.mkdtemp(prefix="beir_bench_")
    results = []
    try:
        print("\nExact kNN (numpy) — ceiling row...")
        preds, qms, ing = exact_knn(corpus_vecs, query_vecs, args.top_k)
        exact_preds = preds
        results.append(summarize("exact-knn", preds, qms, ing, relevant_idx, exact_preds, k_values, args.top_k))

        for name in engines:
            if name == "vanta":
                print("\nVantaDB...")
                preds, qms, ing = bench_vanta(corpus_vecs, query_vecs, args.top_k, workdir)
            else:
                print("\nsqlite-vec...")
                preds, qms, ing = bench_sqlite_vec(corpus_vecs, query_vecs, args.top_k, workdir)
            results.append(summarize(name, preds, qms, ing, relevant_idx, exact_preds, k_values, args.top_k))
    finally:
        shutil.rmtree(workdir, ignore_errors=True)
        gc.collect()

    # --- summary table (markdown) ---
    kcols = [k for k in k_values if k <= args.top_k]
    header = "| Engine | " + " | ".join(f"recall@{k} (qrels)" for k in kcols) + \
             " | nDCG@10 | " + " | ".join(f"index-recall@{k}" for k in kcols) + \
             " | ingest (s) | q p50 (ms) | q p99 (ms) |"
    sep = "|" + "---|" * (1 + len(kcols) + 1 + len(kcols) + 3)
    print("\n" + header)
    print(sep)
    for r in results:
        rec = " | ".join(f"{r['recall_at_k'][str(k)]:.4f}" for k in kcols)
        idx = " | ".join(f"{r['index_recall_at_k'].get(str(k), float('nan')):.4f}" for k in kcols)
        print(f"| {r['engine']} | {rec} | {r['ndcg_at_10']:.4f} | {idx} | "
              f"{r['ingest_s']:.2f} | {r['query_p50_ms']:.3f} | {r['query_p99_ms']:.3f} |")

    # --- JSON report ---
    try:
        import importlib.metadata as md
        import vantadb
        vantadb_version = getattr(vantadb, "__version__", None) or md.version("vantadb-py")
        sqlite_vec_version = md.version("sqlite-vec")
    except Exception:
        vantadb_version = sqlite_vec_version = "unknown"
    flat_env = os.environ.get("VANTADB_FLAT_THRESHOLD")
    if flat_env is None:
        flat_threshold, flat_env_source = 10000, "default(src/config.rs:347)"
    else:
        try:
            flat_threshold, flat_env_source = int(flat_env), "env:VANTADB_FLAT_THRESHOLD"
        except ValueError:
            flat_threshold, flat_env_source = 10000, "default(invalid env ignored)"
    search_mode = "flat-exact" if len(corpus) <= flat_threshold else "hnsw"
    report = {
        "benchmark": "beir_recall_bench",
        "task": "BENCH-02",
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%S"),
        "dataset": {
            "name": f"mteb/{args.dataset}",
            "split": args.split,
            "reference": DATASETS[args.dataset]["reference"],
            "license": DATASETS[args.dataset]["license"],
            "corpus_size": len(corpus),
            "eval_queries": len(queries),
            "files_sha256": digests,
        },
        "model": {
            "id": args.model,
            "rev": model_spec["rev"],
            "dim": model_spec["dim"],
            "max_seq": model_spec["max_seq"],
            "pooling": "mean",
            "normalized": True,
        },
        "protocol": {"seed": args.seed, "top_k": args.top_k, "k_values": kcols,
                     "metric": "recall@k vs qrels (MTEB/TREC: |top-k ∩ relevant| / |relevant|)"},
        "environment": {
            "os": platform.platform(),
            "cpu": platform.processor(),
            "ram_gb": round(psutil.virtual_memory().total / 1024**3, 2),
            "python": platform.python_version(),
            "vantadb_py": vantadb_version,
            "sqlite_vec": sqlite_vec_version,
        },
        "engine_config": {
            # VantaDB routes to the flat exact scan when nodes <= flat_threshold
            # (src/index/search/neighbors.rs); default 10,000 (src/config.rs:347,1430),
            # VANTADB_FLAT_THRESHOLD=0 disables the flat path (always HNSW).
            "vanta_flat_threshold": flat_threshold,
            "vanta_flat_threshold_source": flat_env_source,
            "vanta_search_mode_at_corpus_size": search_mode,
        },
        "results": results,
    }
    out_path = args.json_output
    os.makedirs(os.path.dirname(out_path) or ".", exist_ok=True)
    with open(out_path, "w", encoding="utf-8") as fh:
        json.dump(report, fh, indent=2)
    print(f"\nJSON report written to {out_path}")
    return 0


if __name__ == "__main__":
    sys.exit(main())

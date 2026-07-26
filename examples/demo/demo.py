"""
VantaDB DEMO
============
End-to-end showcase of VantaDB's embedded persistent memory engine.

Demonstrates:
  1. Creating / opening a VantaDB database
  2. Inserting documents with text, metadata, and embedding vectors
  3. Dense vector (ANN) search
  4. Hybrid search (vector + BM25 text fusion)
  5. Persistence across close / reopen
  6. Operational metrics and hardware profile introspection

Run:
  python examples/demo/demo.py

Requires:  vantadb-py>=0.4  (install from TestPyPI or build from source)
Optional:  sentence-transformers (generates real embeddings — see README)
"""

from __future__ import annotations

import os
import shutil
import tempfile


# ── Optional embedding helper ──────────────────────────────────────────

def _dummy_embed(text: str, dim: int = 4) -> list[float]:
    """Cheap deterministic mock embedding — NOT semantically meaningful.

    Replace with a real model (sentence-transformers, etc.) for production use.
    """
    import hashlib
    h = hashlib.sha256(text.encode()).digest()
    scale = 1.0 / max(h)  # normalise to [0, 1]
    return [(b * scale) for b in h[:dim]]


def _try_load_embedder():
    """Return an embedder, or fall back to ``_dummy_embed``."""
    try:
        from sentence_transformers import SentenceTransformer
        model = SentenceTransformer("all-MiniLM-L6-v2", device="cpu")
        print("  [embed] all-MiniLM-L6-v2 loaded")
        return lambda texts: model.encode(texts, normalize_embeddings=True).tolist()
    except ImportError:
        print("  [embed] sentence-transformers not installed — using mock embeddings")
        return lambda texts: [_dummy_embed(t) for t in texts]


# ── Demo helpers ───────────────────────────────────────────────────────

def section(title: str) -> None:
    """Print a visual section header."""
    sep = "─" * 56
    print(f"\n{sep}")
    print(f"  {title}")
    print(f"{sep}")


def fmt_bytes(n: int) -> str:
    """Human-readable byte count."""
    for unit in ("B", "KB", "MB", "GB"):
        if n < 1024:
            return f"{n:.1f} {unit}"
        n /= 1024
    return f"{n:.1f} TB"


# ── Main ───────────────────────────────────────────────────────────────

def main():
    db_path = os.path.join(tempfile.gettempdir(), "vantadb_demo")
    # Clean any leftover from a previous run
    if os.path.exists(db_path):
        shutil.rmtree(db_path)

    print("=" * 56)
    print("   VantaDB  —  Embedded Vector-Graph Database")
    print("   Demo App")
    print("=" * 56)

    # ── 1. Create / open database ──────────────────────────────────────
    section("1. Create / open database")

    import vantadb_py

    db = vantadb_py.VantaDB(db_path, memory_limit_bytes=128 * 1024 * 1024)
    caps = db.capabilities()
    print(f"  Storage  : {db_path}")
    print(f"  Profile  : {caps.get('runtime_profile', '?')}")
    print(f"  Backend  : {caps.get('backend_kind', '?')}")
    print(f"  Vector   : {caps.get('vector_search', '?')}")
    print(f"  Persist  : {caps.get('persistence', '?')}")

    # ── 2. Insert documents ────────────────────────────────────────────
    section("2. Insert documents (text + metadata + vectors)")

    embed_fn = _try_load_embedder()

    documents = [
        (
            "alice",
            "Alice loves hiking in the Swiss Alps during summer.",
            {"person": "Alice", "activity": "hiking", "season": "summer"},
        ),
        (
            "bob",
            "Bob is a data engineer who builds Python pipelines for ETL.",
            {"person": "Bob", "profession": "engineer", "topic": "data"},
        ),
        (
            "carol",
            "Carol runs a bakery in Paris and specializes in sourdough bread.",
            {"person": "Carol", "profession": "baker", "city": "Paris"},
        ),
        (
            "dave",
            "Dave writes Rust crates for embedded systems and IoT devices.",
            {"person": "Dave", "profession": "developer", "language": "Rust"},
        ),
        (
            "eve",
            "Eve is a data scientist working on NLP and vector search at a startup.",
            {"person": "Eve", "profession": "data-scientist", "topic": "NLP"},
        ),
    ]

    texts = [d[1] for d in documents]
    vectors = embed_fn(texts)

    for (key, payload, meta), vec in zip(documents, vectors):
        record = db.put(
            namespace="demo",
            key=key,
            payload=payload,
            metadata=meta,
            vector=vec,
        )
        # record is a VantaMemoryRecord — access fields by attribute or key
        print(f"  ✓ {record.key:12s}  dim={len(vec)}  meta={dict(record.metadata)}")

    # ── 3. Dense vector (ANN) search ───────────────────────────────────
    section("3. Dense vector search (ANN)")

    query = "machine learning and natural language processing"
    query_vec = embed_fn([query])[0]
    hits = db.search_memory(
        namespace="demo",
        query_vector=query_vec,
        top_k=3,
    )

    print(f"  Query: \"{query}\"\n")
    for i, hit in enumerate(hits, 1):
        print(f"  #{i}  {hit.key:12s}  score={hit.score:.4f}   {hit.payload}")

    # ── 4. Hybrid search (vector + BM25 text) ──────────────────────────
    section("4. Hybrid search (vector + text)")

    query2 = "who writes code"
    query_vec2 = embed_fn([query2])[0]
    hits2 = db.search_memory(
        namespace="demo",
        query_vector=query_vec2,
        text_query="Rust engineer developer",
        top_k=3,
    )

    print(f"  Query vector : \"{query2}\"")
    print(f"  Query text   : \"Rust engineer developer\"\n")
    for i, hit in enumerate(hits2, 1):
        meta_str = dict(hit.metadata)
        print(f"  #{i}  {hit.key:12s}  score={hit.score:.4f}  meta={meta_str}")
        print(f"       {hit.payload}")

    # ── 5. Persistence ─────────────────────────────────────────────────
    section("5. Persistence (close + reopen)")

    key_count_before = len(db.list_memory(namespace="demo", limit=9999).records)
    print(f"  Records before close : {key_count_before}")

    db.flush()
    db.close()
    print("  Database closed.")

    # Reopen
    db2 = vantadb_py.VantaDB(db_path, memory_limit_bytes=128 * 1024 * 1024)
    key_count_after = len(db2.list_memory(namespace="demo", limit=9999).records)
    print(f"  Records after reopen : {key_count_after}")

    # Verify a specific record survived
    alice_record = db2.get_memory("demo", "alice")
    if alice_record:
        print(f"  Retrieved alice     : \"{alice_record.payload}\"")
    else:
        print("  ⚠  alice record lost — persistence issue!")

    # ── 6. Operational metrics ─────────────────────────────────────────
    section("6. Operational metrics")

    metrics = db2.operational_metrics()
    for key in sorted(metrics.keys()):
        val = metrics[key]
        if isinstance(val, (int, float)):
            if "byte" in key or "rss" in key or "virtual" in key:
                print(f"  {key:35s} {fmt_bytes(int(val))}")
            else:
                print(f"  {key:35s} {val:,}" if isinstance(val, int) else f"  {key:35s} {val}")
        else:
            print(f"  {key:35s} {val}")

    # ── 7. Hardware profile ────────────────────────────────────────────
    section("7. Hardware profile")

    profile = db2.hardware_profile()
    for key in sorted(profile.keys()):
        val = profile[key]
        if "byte" in key or "rss" in key or "virtual" in key or "cache_cap" in key:
            print(f"  {key:35s} {fmt_bytes(int(val))}")
        elif isinstance(val, (int, float)):
            print(f"  {key:35s} {val:,}" if isinstance(val, int) else f"  {key:35s} {val:.2f}")
        else:
            print(f"  {key:30s} {val}")

    # ── Cleanup ────────────────────────────────────────────────────────
    section("Cleanup")

    db2.close()
    if os.path.exists(db_path):
        shutil.rmtree(db_path)
    print(f"  Removed {db_path}")
    print("  Done — VantaDB demo completed successfully.")


if __name__ == "__main__":
    main()

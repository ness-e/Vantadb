"""
PERF-15: put_batch_raw with 2D NumPy array (zero-copy batch ingestion)
PERF-16: VantaSearchHit pyclass (typed search results, no per-hit PyDict)

Core tests for put_batch_raw and VantaSearchHit live in test_sdk.py
(TestMemoryAPI, TestVantaSearchHit). This file covers edge cases and
scenarios unique to PERF-15/PERF-16.
"""

import os
import shutil
import uuid
import numpy as np
import vantadb_py as vanta

TEST_DB = "./test_perf_15_16_db"


def _clean():
    import glob
    for path in glob.glob(f"{TEST_DB}_*"):
        if os.path.exists(path):
            shutil.rmtree(path, ignore_errors=True)


def _unique():
    return f"{TEST_DB}_{uuid.uuid4().hex[:8]}"


# ── PERF-15: put_batch_raw edge cases ────────────────────────────────────


def test_put_batch_raw_default_namespace():
    _clean()
    db = vanta.VantaDB(_unique(), memory_limit_bytes=128 * 1024 * 1024)
    vectors = np.array([[0.5, 0.5]], dtype=np.float32)
    records = db.put_batch_raw(vectors, ["k"])
    assert records[0]["namespace"] == "default"
    assert records[0]["key"] == "k"


def test_put_batch_raw_empty_payload():
    _clean()
    db = vanta.VantaDB(_unique(), memory_limit_bytes=128 * 1024 * 1024)
    vectors = np.array([[1.0, 0.0, 0.0]], dtype=np.float32)
    records = db.put_batch_raw(vectors, ["k"], namespaces=["ns"])
    assert records[0]["payload"] == ""


def test_put_batch_raw_ttl():
    _clean()
    db = vanta.VantaDB(_unique(), memory_limit_bytes=128 * 1024 * 1024)
    vectors = np.array([[1.0, 0.0, 0.0]], dtype=np.float32)
    records = db.put_batch_raw(vectors, ["k"], ttls=[86_400_000], namespaces=["ns"])
    assert records[0]["expires_at_ms"] is not None
    assert records[0]["expires_at_ms"] > 0


def test_put_batch_raw_persists():
    path = _unique()
    _clean()
    db = vanta.VantaDB(path, memory_limit_bytes=128 * 1024 * 1024)
    vectors = np.array([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]], dtype=np.float32)
    db.put_batch_raw(vectors, ["x", "y"], namespaces=["ns", "ns"])
    db.flush()
    db.close()

    db2 = vanta.VantaDB(path, memory_limit_bytes=128 * 1024 * 1024)
    assert db2.get_memory("ns", "x") is not None
    assert db2.get_memory("ns", "y")["key"] == "y"


# ── PERF-16: VantaSearchHit edge cases ───────────────────────────────────


def test_vanta_search_hit_properties():
    _clean()
    db = vanta.VantaDB(_unique(), memory_limit_bytes=128 * 1024 * 1024)
    db.put("ns", "props", "payload",
           metadata={"a": 1, "b": "two"},
           vector=np.array([1.0, 0.0, 0.0], dtype=np.float32))

    hits = db.search_memory("ns", np.array([1.0, 0.0, 0.0], dtype=np.float32), top_k=1)
    hit = hits[0]

    assert hit.namespace == "ns"
    assert hit.key == "props"
    assert hit.payload == "payload"
    assert isinstance(hit.score, float)
    assert hit.id is not None
    assert isinstance(hit.created_at_ms, int) and hit.created_at_ms > 0
    assert isinstance(hit.updated_at_ms, int) and hit.updated_at_ms > 0
    assert isinstance(hit.version, int) and hit.version >= 1
    assert hit.node_id is not None
    assert hit.vector is not None
    assert len(hit.vector) == 3
    assert isinstance(hit.metadata, dict)
    assert hit.metadata["a"] == 1
    assert hit.metadata["b"] == "two"


def test_vanta_search_hit_expires_at():
    _clean()
    db = vanta.VantaDB(_unique(), memory_limit_bytes=128 * 1024 * 1024)
    db.put("ns", "ttl-hit", "payload",
           vector=np.array([1.0, 0.0, 0.0], dtype=np.float32),
           ttl_ms=86_400_000)
    hits = db.search_memory("ns", np.array([1.0, 0.0, 0.0], dtype=np.float32), top_k=1)
    assert hits[0].expires_at_ms is not None
    assert hits[0].expires_at_ms > 0


def test_vanta_search_hit_no_ttl_returns_none():
    _clean()
    db = vanta.VantaDB(_unique(), memory_limit_bytes=128 * 1024 * 1024)
    db.put("ns", "no-ttl", "payload",
           vector=np.array([1.0, 0.0, 0.0], dtype=np.float32))
    hits = db.search_memory("ns", np.array([1.0, 0.0, 0.0], dtype=np.float32), top_k=1)
    assert hits[0].expires_at_ms is None


def test_vanta_search_hit_accessible_from_package():
    assert hasattr(vanta, "VantaSearchHit"), \
        "VantaSearchHit must be accessible from the package level"


def test_vanta_search_hit_with_text_query():
    _clean()
    db = vanta.VantaDB(_unique(), memory_limit_bytes=128 * 1024 * 1024)
    db.put("ns", "text-hit", "hello world",
           vector=np.array([1.0, 0.0, 0.0], dtype=np.float32))
    hits = db.search_memory("ns", [], text_query="hello", top_k=3)
    assert len(hits) >= 1
    assert isinstance(hits[0], vanta.VantaSearchHit)
    assert hits[0].key == "text-hit"

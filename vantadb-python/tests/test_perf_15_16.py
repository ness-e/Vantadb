"""
PERF-15: put_batch_raw with 2D NumPy array (zero-copy batch ingestion)
PERF-16: VantaSearchHit pyclass (typed search results, no per-hit PyDict)

Standalone tests — run with: pytest tests/test_perf_15_16.py -v
"""

import os
import shutil
import time
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


# ── PERF-15: put_batch_raw ──────────────────────────────────────────────


def test_put_batch_raw_basic():
    """Insert 3 vectors with keys/payloads/namespaces."""
    _clean()
    db = vanta.VantaDB(_unique(), memory_limit_bytes=128 * 1024 * 1024)
    vectors = np.array([
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
    ], dtype=np.float32)
    records = db.put_batch_raw(
        vectors,
        ["a", "b", "c"],
        payloads=["alpha", "beta", "gamma"],
        namespaces=["ns", "ns", "ns"],
    )
    assert len(records) == 3
    assert records[0]["key"] == "a"
    assert records[1]["payload"] == "beta"
    assert records[2]["namespace"] == "ns"

    fetched = db.get_memory("ns", "c")
    assert fetched is not None
    assert fetched["payload"] == "gamma"


def test_put_batch_raw_f64_downcast():
    """f64 array auto-downcasts to f32."""
    _clean()
    db = vanta.VantaDB(_unique(), memory_limit_bytes=128 * 1024 * 1024)
    vectors = np.array([[1.0, 0.0], [0.0, 1.0]], dtype=np.float64)
    records = db.put_batch_raw(vectors, ["x", "y"], namespaces=["ns", "ns"])
    assert len(records) == 2


def test_put_batch_raw_with_metadata():
    """Per-row metadata dicts."""
    _clean()
    db = vanta.VantaDB(_unique(), memory_limit_bytes=128 * 1024 * 1024)
    vectors = np.array([[0.1, 0.2], [0.3, 0.4]], dtype=np.float32)
    metadatas = [{"tag": "first"}, {"tag": "second"}]
    records = db.put_batch_raw(
        vectors, ["a", "b"], metadatas=metadatas, namespaces=["ns", "ns"]
    )
    assert records[0]["metadata"]["tag"] == "first"
    assert records[1]["metadata"]["tag"] == "second"


def test_put_batch_raw_shape_mismatch():
    """Reject mismatched vector/key lengths."""
    import pytest
    _clean()
    db = vanta.VantaDB(_unique(), memory_limit_bytes=128 * 1024 * 1024)
    vectors = np.array([[1.0, 0.0], [0.0, 1.0]], dtype=np.float32)
    with pytest.raises(ValueError, match="vectors.shape"):
        db.put_batch_raw(vectors, ["only_one_key"])


def test_put_batch_raw_default_namespace():
    """Default namespace is 'default' when omitted."""
    _clean()
    db = vanta.VantaDB(_unique(), memory_limit_bytes=128 * 1024 * 1024)
    vectors = np.array([[0.5, 0.5]], dtype=np.float32)
    records = db.put_batch_raw(vectors, ["k"])
    assert records[0]["namespace"] == "default"
    assert records[0]["key"] == "k"


def test_put_batch_raw_empty_payload():
    """Empty string payload when payloads omitted."""
    _clean()
    db = vanta.VantaDB(_unique(), memory_limit_bytes=128 * 1024 * 1024)
    vectors = np.array([[1.0, 0.0, 0.0]], dtype=np.float32)
    records = db.put_batch_raw(vectors, ["k"], namespaces=["ns"])
    assert records[0]["payload"] == ""


def test_put_batch_raw_ttl():
    """TTL per record."""
    _clean()
    db = vanta.VantaDB(_unique(), memory_limit_bytes=128 * 1024 * 1024)
    vectors = np.array([[1.0, 0.0, 0.0]], dtype=np.float32)
    records = db.put_batch_raw(vectors, ["k"], ttls=[86_400_000], namespaces=["ns"])
    assert records[0]["expires_at_ms"] is not None
    assert records[0]["expires_at_ms"] > 0


def test_put_batch_raw_persists():
    """Records survive flush/close/reopen."""
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


# ── PERF-16: VantaSearchHit ─────────────────────────────────────────────


def test_search_memory_returns_vanta_search_hit():
    """search_memory returns VantaSearchHit objects (not dicts)."""
    _clean()
    db = vanta.VantaDB(_unique(), memory_limit_bytes=128 * 1024 * 1024)

    db.put("ns", "hit-1", "first hit",
           metadata={"type": "test"},
           vector=np.array([1.0, 0.0, 0.0], dtype=np.float32))

    hits = db.search_memory("ns", np.array([1.0, 0.0, 0.0], dtype=np.float32), top_k=3)
    assert len(hits) >= 1

    hit = hits[0]
    assert isinstance(hit, vanta.VantaSearchHit)
    assert hit.key == "hit-1"
    assert hit.payload == "first hit"
    assert hit.namespace == "ns"
    assert isinstance(hit.score, float)
    assert hit.score >= 0.0
    assert hit.id is not None
    assert hit.vector is not None
    assert len(hit.vector) == 3
    assert hit.metadata is not None
    assert hit.metadata["type"] == "test"


def test_vanta_search_hit_repr():
    """VantaSearchHit repr includes key and score."""
    _clean()
    db = vanta.VantaDB(_unique(), memory_limit_bytes=128 * 1024 * 1024)
    db.put("ns", "repr-test", "payload",
           vector=np.array([1.0, 0.0, 0.0], dtype=np.float32))
    hits = db.search_memory("ns", np.array([1.0, 0.0, 0.0], dtype=np.float32), top_k=1)
    r = repr(hits[0])
    assert "VantaSearchHit(" in r
    assert "repr-test" in r


def test_vanta_search_hit_properties():
    """All VantaSearchHit properties are accessible."""
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
    """expires_at_ms reflects TTL."""
    _clean()
    db = vanta.VantaDB(_unique(), memory_limit_bytes=128 * 1024 * 1024)
    db.put("ns", "ttl-hit", "payload",
           vector=np.array([1.0, 0.0, 0.0], dtype=np.float32),
           ttl_ms=86_400_000)
    hits = db.search_memory("ns", np.array([1.0, 0.0, 0.0], dtype=np.float32), top_k=1)
    assert hits[0].expires_at_ms is not None
    assert hits[0].expires_at_ms > 0


def test_vanta_search_hit_no_ttl_returns_none():
    """expires_at_ms is None when no TTL set."""
    _clean()
    db = vanta.VantaDB(_unique(), memory_limit_bytes=128 * 1024 * 1024)
    db.put("ns", "no-ttl", "payload",
           vector=np.array([1.0, 0.0, 0.0], dtype=np.float32))
    hits = db.search_memory("ns", np.array([1.0, 0.0, 0.0], dtype=np.float32), top_k=1)
    assert hits[0].expires_at_ms is None


def test_vanta_search_hit_accessible_from_package():
    """VantaSearchHit is accessible as vanta.VantaSearchHit."""
    assert hasattr(vanta, "VantaSearchHit"), \
        "VantaSearchHit must be accessible from the package level"


def test_vanta_search_hit_with_text_query():
    """VantaSearchHit works with text-only queries."""
    _clean()
    db = vanta.VantaDB(_unique(), memory_limit_bytes=128 * 1024 * 1024)
    db.put("ns", "text-hit", "hello world",
           vector=np.array([1.0, 0.0, 0.0], dtype=np.float32))
    hits = db.search_memory("ns", [], text_query="hello", top_k=3)
    assert len(hits) >= 1
    assert isinstance(hits[0], vanta.VantaSearchHit)
    assert hits[0].key == "text-hit"


if __name__ == "__main__":
    import pytest
    pytest.main([__file__, "-v"])

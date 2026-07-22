"""Tests for the mem0 VectorStoreBase-compliant VantaDBVectorStore."""
import pytest
import tempfile
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from vantadb_mem0 import VantaDBVectorStore, OutputData

VEC_128 = [0.1] * 128
VEC_128_B = [0.2] * 128


@pytest.fixture
def store():
    path = os.path.join(tempfile.mkdtemp(), "test_mem0")
    s = VantaDBVectorStore(db_path=path, namespace="test_mem0")
    yield s


# ── VectorStoreBase: 11 abstract methods ──────────────────────


class TestVectorStoreBaseMethods:
    """Every abstract method from mem0.vector_stores.base.VectorStoreBase."""

    def test_1_create_col(self, store):
        """create_col is a no-op for schemaless VantaDB."""
        store.create_col("test", vector_size=128, distance="cosine")
        # must not raise

    def test_2_insert_and_search(self, store):
        store.create_col("test", 128)
        store.insert(
            [VEC_128],
            payloads=[{"data": "hello world"}],
            ids=["k1"],
        )
        results = store.search("hello world", VEC_128, top_k=5)
        assert len(results) == 1
        r = results[0]
        assert r.id == "k1"
        assert r.score is not None
        assert 0.0 <= r.score <= 1.0

    def test_3_get(self, store):
        store.insert([VEC_128], payloads=[{"data": "gettest"}], ids=["get1"])
        result = store.get("get1")
        assert result is not None
        assert result.id == "get1"

    def test_4_get_missing(self, store):
        assert store.get("nonexistent") is None

    def test_5_delete(self, store):
        store.insert([VEC_128], payloads=[{"data": "deltest"}], ids=["del1"])
        store.delete("del1")
        assert store.get("del1") is None

    def test_6_update(self, store):
        store.insert([VEC_128], payloads=[{"data": "old"}], ids=["upd1"])
        store.update("upd1", vector=VEC_128_B, payload={"data": "new"})
        result = store.get("upd1")
        assert result is not None
        # verify the updated payload was stored
        assert result.payload.get("data") == "new"

    def test_7_list(self, store):
        store.insert(
            [VEC_128, VEC_128_B],
            payloads=[{"data": "a"}, {"data": "b"}],
            ids=["l1", "l2"],
        )
        results = store.list(top_k=10)
        ids = {r.id for r in results}
        assert "l1" in ids
        assert "l2" in ids

    def test_8_list_cols(self, store):
        cols = store.list_cols()
        assert isinstance(cols, list)

    def test_9_col_info(self, store):
        info = store.col_info()
        assert info["name"] == "test_mem0"

    def test_10_reset(self, store):
        store.insert([VEC_128], payloads=[{"data": "x"}], ids=["r1"])
        store.reset()
        results = store.list(top_k=10)
        assert len(results) == 0

    def test_11_delete_col(self, store):
        store.create_col("delete_me", 128)
        store.delete_col()
        # must not raise


# ── Backward-compatible convenience API ───────────────────────


class TestBackwardCompat:
    """Original flat-class methods that should still work."""

    def test_add(self, store):
        key = store.add("hello world")
        assert key is not None
        assert isinstance(key, str)

    def test_add_with_user_id(self, store):
        key = store.add("user text", user_id="alice")
        assert key is not None

    def test_add_with_metadata(self, store):
        key = store.add("meta text", metadata={"type": "test"})
        assert key is not None

    # list with default params still works (no filters, default top_k)
    def test_list_default(self, store):
        store.add("doc a")
        store.add("doc b")
        results = store.list()
        assert len(results) >= 2

    # delete from add
    def test_delete_added(self, store):
        key = store.add("delete-me")
        store.delete(key)
        assert store.get(key) is None


# ── OutputData helper ─────────────────────────────────────────


class TestOutputData:
    def test_defaults(self):
        d = OutputData()
        assert d.id is None
        assert d.score is None
        assert d.payload == {}

    def test_fields(self):
        d = OutputData(id="abc", score=0.95, payload={"key": "val"})
        assert d.id == "abc"
        assert d.score == 0.95
        assert d.payload["key"] == "val"

    def test_repr(self):
        d = OutputData(id="x", score=0.5)
        assert "OutputData" in repr(d)

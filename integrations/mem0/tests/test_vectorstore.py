"""Tests for the mem0 VectorStoreBase-compliant VantaDBVectorStore."""
import pytest
pytest.importorskip("mem0.vector_stores", reason="mem0 SDK not installed; adapter suite skipped")
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from vantadb_mem0 import VantaDBVectorStore, OutputData

VEC_128 = [0.1] * 128
VEC_128_B = [0.2] * 128


@pytest.fixture
def store(tmp_path):
    path = str(tmp_path / "test_mem0")
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


# ── FIND-579: filters are honored (user_id isolation) ─────────


class TestSearchFilters:
    """mem0 scopes memories per user/agent via ``filters`` — dropping them
    leaks one user's memories into another user's context."""

    def _populate_two_users(self, store):
        store.insert(
            [VEC_128, VEC_128_B],
            payloads=[
                {"data": "alice memory", "user_id": "alice"},
                {"data": "bob memory", "user_id": "bob"},
            ],
            ids=["u-a", "u-b"],
        )

    def test_search_filters_flat_user_id(self, store):
        self._populate_two_users(store)
        results = store.search("memory", VEC_128, top_k=5, filters={"user_id": "alice"})
        ids = {r.id for r in results}
        assert "u-a" in ids
        assert "u-b" not in ids

    def test_search_filters_eq_operator(self, store):
        self._populate_two_users(store)
        results = store.search(
            "memory", VEC_128, top_k=5, filters={"user_id": {"$eq": "alice"}}
        )
        ids = {r.id for r in results}
        assert "u-a" in ids
        assert "u-b" not in ids

    def test_search_filters_no_match(self, store):
        self._populate_two_users(store)
        results = store.search(
            "memory", VEC_128, top_k=5, filters={"user_id": "carol"}
        )
        assert results == []

    def test_search_without_filters_returns_all(self, store):
        self._populate_two_users(store)
        results = store.search("memory", VEC_128, top_k=5)
        ids = {r.id for r in results}
        assert {"u-a", "u-b"} <= ids

    def test_search_residual_operator_postfilter(self, store):
        """Operators that cannot be pushed down are evaluated in Python."""
        self._populate_two_users(store)
        results = store.search(
            "memory", VEC_128, top_k=5, filters={"user_id": {"$neq": "alice"}}
        )
        ids = {r.id for r in results}
        assert "u-b" in ids
        assert "u-a" not in ids

    def test_search_unknown_operator_fails_closed(self, store):
        """An operator the adapter cannot evaluate must not broaden results."""
        self._populate_two_users(store)
        results = store.search(
            "memory", VEC_128, top_k=5, filters={"user_id": {"$contains": "ali"}}
        )
        assert results == []

    def test_list_filters_flat_user_id(self, store):
        self._populate_two_users(store)
        results = store.list(filters={"user_id": "bob"}, top_k=10)
        ids = {r.id for r in results}
        assert "u-b" in ids
        assert "u-a" not in ids


class TestFilterTranslationHelpers:
    """Pure-Python contract of the filter split — no engine needed."""

    def test_translate_flat_and_eq(self):
        from vantadb_mem0.vectorstore import _translate_filters

        native, residual = _translate_filters(
            {"user_id": "alice", "meta": {"$eq": 1, "$gte": 5}}
        )
        assert native == {"user_id": "alice", "meta": 1}
        assert residual == [("meta", "$gte", 5)]

    def test_translate_unknown_op_residual(self):
        from vantadb_mem0.vectorstore import _translate_filters

        native, residual = _translate_filters({"f": {"$regex": "x"}})
        assert native == {}
        assert residual == [("f", "$regex", "x")]

    def test_matches_scalar_ops(self):
        from vantadb_mem0.vectorstore import _record_matches

        payload = {"user_id": "alice", "priority": 3}
        assert _record_matches(payload, [("user_id", "$eq", "alice")])
        assert not _record_matches(payload, [("user_id", "$neq", "alice")])
        assert _record_matches(payload, [("priority", "$gte", 3)])
        assert not _record_matches(payload, [("priority", "$lt", 3)])
        assert _record_matches(payload, [("user_id", "$in", ["alice", "bob"])])
        assert not _record_matches(payload, [("user_id", "$nin", ["alice"])])
        # Missing field never satisfies a predicate (fail closed).
        assert not _record_matches(payload, [("missing", "$eq", "x")])
        assert not _record_matches(payload, [("missing", "$bogus", "x")])


class TestDeleteColGuard:
    """M-8: delete_col must not loop forever when deletes make no progress."""

    class _FakeMemory:
        def __init__(self, keys, delete_ok=False):
            self._keys = keys
            self._delete_ok = delete_ok

        def list(self, namespace, **kwargs):
            import types

            return types.SimpleNamespace(
                records=[types.SimpleNamespace(key=k) for k in self._keys]
            )

        def delete(self, namespace, key):
            if self._delete_ok:
                self._keys.remove(key)
            return self._delete_ok

    class _FakeDb:
        def __init__(self, memory):
            self.memory = memory

    def test_delete_col_raises_on_no_progress(self, store):
        """If memory.delete returns False the page repeats → explicit error."""
        store._db = self._FakeDb(self._FakeMemory(["k1", "k2"], delete_ok=False))
        with pytest.raises(RuntimeError, match="no progress"):
            store.delete_col()

    class _RotatingMemory:
        """list() returns a *different* page each call; delete() always fails.

        The identical-page check can never fire here — only the consecutive
        zero-deletion-passes guard can stop the loop.
        """

        def __init__(self):
            self._gen = 0

        def list(self, namespace, **kwargs):
            import types

            self._gen += 1
            return types.SimpleNamespace(
                records=[
                    types.SimpleNamespace(key=f"k{self._gen}-{i}") for i in range(3)
                ]
            )

        def delete(self, namespace, key):
            return False

    def test_delete_col_raises_on_zero_delete_passes(self, store):
        """Every delete() reporting False twice in a row → explicit error."""
        store._db = self._FakeDb(self._RotatingMemory())
        with pytest.raises(RuntimeError, match="not making progress"):
            store.delete_col()

    def test_delete_col_empties_normally(self, store):
        store.insert([VEC_128], payloads=[{"data": "x"}], ids=["d1"])
        store.delete_col()
        assert store.list(top_k=10) == []


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


# ── QW-5: semántica exacta de _normalize_score ──

def test_normalize_score_exact_semantics():
    """Pin de semántica: None→0; [0,1] pasa tal cual; fuera se trata como distancia d→clamp(1-d,0,1)."""
    from vantadb_mem0.vectorstore import _normalize_score
    assert _normalize_score(None) == 0.0
    assert _normalize_score(0.0) == 0.0
    assert _normalize_score(0.5) == 0.5      # pre-normalizado pasa sin cambios
    assert _normalize_score(0.3) == 0.3      # [0,1] pasa tal cual (rama passthrough)
    assert _normalize_score(1.0) == 1.0
    assert _normalize_score(1.2) == 0.0      # fuera de [0,1]: distancia d→clamp(1-d)
    assert _normalize_score(1.7) == 0.0      # distancia >1 satura a 0
    assert _normalize_score(2.0) == 0.0      # peor distancia coseno
    assert _normalize_score(-0.3) == 0.0     # negativo inválido → clamp, nunca >1

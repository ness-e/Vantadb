"""Tests for the VantaDB Ollama adapter."""

import pytest

pytest.importorskip("ollama")
pytest.importorskip("vantadb_ollama")
from vantadb_ollama import VantaDBOllama, __version__


class TestVantaDBOllama:
    def test_version(self):
        assert isinstance(__version__, str)
        assert len(__version__) > 0

    def test_init(self, tmp_path):
        store = VantaDBOllama(str(tmp_path))
        assert store is not None

    def test_init_custom_namespace(self, tmp_path):
        store = VantaDBOllama(str(tmp_path), namespace="custom_ns")
        assert store is not None

    def test_store_and_search(self, tmp_path):
        store = VantaDBOllama(str(tmp_path))
        embedding = [0.1] * 128
        rid = store.store("ollama test", embedding)
        assert ":" in rid
        results = store.search("ollama_store", embedding, top_k=5)
        assert len(results) > 0
        assert results[0]["text"] == "ollama test"

    def test_store_with_metadata(self, tmp_path):
        store = VantaDBOllama(str(tmp_path))
        rid = store.store("meta test", [0.2] * 128, {"key": "val"})
        assert ":" in rid

    def test_search_invalid_distance_metric_raises(self, tmp_path):
        store = VantaDBOllama(str(tmp_path))
        with pytest.raises(ValueError, match="distance_metric"):
            store.search("ollama_store", [0.1] * 128, distance_metric="manhattan")

    def test_store_unsupported_metadata_warns_and_keeps_supported(self, tmp_path):
        store = VantaDBOllama(str(tmp_path), namespace="ns_warn")
        with pytest.warns(UserWarning, match="unsupported value types"):
            rid = store.store(
                "warn-test", [0.8] * 128, {"ok": "yes", "bad": {"nested": True}}
            )
        ns, key = rid.split(":", 1)
        record = store.get(ns, key)
        assert record["metadata"]["ok"] == "yes"
        assert "bad" not in record["metadata"]

    def test_embed_mocked(self, tmp_path, monkeypatch):
        class _FakeClient:
            def __init__(self, **kwargs):
                pass

            def embed(self, *, model=None, input=None):
                return {"embeddings": [[0.1] * 4 for _ in input]}

        import ollama

        monkeypatch.setattr(ollama, "Client", _FakeClient)
        store = VantaDBOllama(str(tmp_path))
        out = store.embed(["a"])
        assert len(out) == 1
        assert len(out[0]) == 4


class TestEmbedBatchProv11:
    """PROV-11: embed_batch() chunking + async pattern (mocked, no network)."""

    def _store(self, tmp_path, monkeypatch, calls):
        class _FakeClient:
            def __init__(self, **kwargs):
                pass

            def embed(self, *, model=None, input=None):
                calls.append(list(input))
                return {"embeddings": [[float(t[1:])] * 4 for t in input]}

        import ollama

        monkeypatch.setattr(ollama, "Client", _FakeClient)
        from vantadb_ollama import VantaDBOllama as _Cls

        return _Cls(str(tmp_path))

    def test_batch_chunks_and_preserves_order(self, tmp_path, monkeypatch):
        calls = []
        store = self._store(tmp_path, monkeypatch, calls)
        texts = [f"t{i}" for i in range(250)]
        out = store.embed_batch(texts, batch_size=100)
        assert len(out) == 250
        assert [v[0] for v in out] == [float(i) for i in range(250)]
        assert [len(c) for c in calls] == [100, 100, 50]

    def test_batch_small_is_single_request(self, tmp_path, monkeypatch):
        calls = []
        store = self._store(tmp_path, monkeypatch, calls)
        out = store.embed_batch(["t1", "t2"])
        assert len(out) == 2
        assert len(calls) == 1

    def test_batch_empty_makes_no_request(self, tmp_path, monkeypatch):
        calls = []
        store = self._store(tmp_path, monkeypatch, calls)
        assert store.embed_batch([]) == []
        assert calls == []

    def test_batch_invalid_size_raises(self, tmp_path, monkeypatch):
        import pytest as _pt

        calls = []
        store = self._store(tmp_path, monkeypatch, calls)
        with _pt.raises(ValueError, match="batch_size"):
            store.embed_batch(["t1"], batch_size=0)
        assert calls == []

    def test_batch_async_via_to_thread(self, tmp_path, monkeypatch):
        import asyncio

        calls = []
        store = self._store(tmp_path, monkeypatch, calls)
        texts = [f"t{i}" for i in range(5)]
        out = asyncio.run(asyncio.to_thread(store.embed_batch, texts, 2))
        assert [v[0] for v in out] == [float(i) for i in range(5)]
        assert [len(c) for c in calls] == [2, 2, 1]


# ── Direct storage tests via vantadb_py ──────────────────────────────────

import os
import tempfile

pytest.importorskip("vantadb_py")
import vantadb_py as vanta


@pytest.fixture
def db():
    path = os.path.join(tempfile.mkdtemp(), "test_ollama")
    s = vanta.VantaDB(path)
    yield s


def test_get_record(db):
    record_id = db.put("test_ollama", "k1", "hello world", vector=[0.1] * 128)
    record = db.get_memory("test_ollama", "k1")
    assert record is not None
    assert record["key"] == "k1"
    assert record["payload"] == "hello world"
    assert record["namespace"] == "test_ollama"
    assert isinstance(record["created_at_ms"], int)
    assert isinstance(record["updated_at_ms"], int)


def test_delete_record(db):
    db.put("test_ollama", "k_del", "delete me", vector=[0.2] * 128)
    found = db.get_memory("test_ollama", "k_del")
    assert found is not None
    db.delete_memory("test_ollama", "k_del")
    gone = db.get_memory("test_ollama", "k_del")
    assert gone is None


def test_list_records(db):
    for i in range(3):
        db.put("test_ollama", f"lst_{i}", f"item {i}", vector=[0.3 + i * 0.01] * 128)
    page = db.list_memory("test_ollama", limit=10)
    assert len(page["records"]) >= 3


def test_list_namespaces(db):
    db.put("test_ollama", "ns_key", "ns payload", vector=[0.4] * 128)
    namespaces = db.list_namespaces()
    assert "test_ollama" in namespaces

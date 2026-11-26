"""Tests for the VantaDB LiteLLM adapter."""

import pytest

pytest.importorskip("litellm")
pytest.importorskip("vantadb_litellm")
from vantadb_litellm import VantaDBLiteLLM, __version__


class TestVantaDBLiteLLM:
    def test_version(self):
        assert isinstance(__version__, str)
        assert len(__version__) > 0

    def test_init(self, tmp_path):
        store = VantaDBLiteLLM(str(tmp_path))
        assert store is not None

    def test_init_with_api_key(self, tmp_path):
        store = VantaDBLiteLLM(str(tmp_path), api_key="test-key")
        assert store is not None

    def test_init_custom_namespace(self, tmp_path):
        store = VantaDBLiteLLM(str(tmp_path), namespace="custom_ns")
        assert store is not None

    def test_store_and_search(self, tmp_path):
        store = VantaDBLiteLLM(str(tmp_path))
        embedding = [0.1] * 128
        rid = store.store("litellm test", embedding)
        assert ":" in rid
        results = store.search("litellm_store", embedding, top_k=5)
        assert len(results) > 0
        assert results[0]["text"] == "litellm test"

    def test_get_record(self, tmp_path):
        store = VantaDBLiteLLM(str(tmp_path), namespace="ns_get")
        embedding = [0.2] * 128
        rid = store.store("get me", embedding)
        key = rid.split(":", 1)[1]
        record = store.get("ns_get", key)
        assert record is not None
        assert record["text"] == "get me"
        assert record["namespace"] == "ns_get"
        assert record["key"] == key

    def test_delete_record(self, tmp_path):
        store = VantaDBLiteLLM(str(tmp_path), namespace="ns_del")
        embedding = [0.3] * 128
        rid = store.store("delete me", embedding)
        key = rid.split(":", 1)[1]
        deleted = store.delete(key)
        assert deleted is True
        record = store.get("ns_del", key)
        assert record is None

    def test_delete_nonexistent(self, tmp_path):
        store = VantaDBLiteLLM(str(tmp_path))
        deleted = store.delete("nonexistent_key")
        assert deleted is False

    def test_list_records(self, tmp_path):
        store = VantaDBLiteLLM(str(tmp_path), namespace="ns_list")
        emb = [0.4] * 128
        store.store("one", emb)
        store.store("two", emb)
        store.store("three", emb)
        page = store.list("ns_list", limit=100)
        assert len(page["records"]) == 3
        texts = {r["text"] for r in page["records"]}
        assert texts == {"one", "two", "three"}

    def test_search_returns_score(self, tmp_path):
        store = VantaDBLiteLLM(str(tmp_path), namespace="ns_score")
        embedding = [0.5] * 128
        store.store("score check", embedding)
        results = store.search("ns_score", embedding, top_k=5)
        assert len(results) > 0
        for r in results:
            assert "score" in r
            assert isinstance(r["score"], float)

    def test_embed_mocked_forwards_timeout(self, tmp_path, monkeypatch):
        captured = {}

        def _fake_embedding(*, model=None, input=None, **kwargs):
            captured.update(kwargs)
            return {"data": [{"embedding": [0.1] * 4} for _ in input]}

        import litellm

        monkeypatch.setattr(litellm, "embedding", _fake_embedding)
        store = VantaDBLiteLLM(str(tmp_path), timeout=12.5)
        out = store.embed(["a"])
        assert len(out) == 1
        assert len(out[0]) == 4
        assert captured.get("timeout") == 12.5

    def test_embed_mocked_omits_timeout_when_unset(self, tmp_path, monkeypatch):
        captured = {}

        def _fake_embedding(*, model=None, input=None, **kwargs):
            captured.update(kwargs)
            return {"data": [{"embedding": [0.1] * 4} for _ in input]}

        import litellm

        monkeypatch.setattr(litellm, "embedding", _fake_embedding)
        store = VantaDBLiteLLM(str(tmp_path))
        store.embed(["a"])
        assert "timeout" not in captured


class TestEmbedBatchProv11:
    """PROV-11: embed_batch() chunking + async pattern (mocked, no network)."""

    def _store(self, tmp_path, monkeypatch, calls):
        def _fake_embedding(*, model=None, input=None, **kwargs):
            calls.append(list(input))
            return {"data": [{"embedding": [float(t[1:])] * 4} for t in input]}

        import litellm

        monkeypatch.setattr(litellm, "embedding", _fake_embedding)
        from vantadb_litellm import VantaDBLiteLLM as _Cls

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

    def test_search_invalid_distance_metric_raises(self, tmp_path):
        store = VantaDBLiteLLM(str(tmp_path), namespace="ns_metric")
        with pytest.raises(ValueError, match="distance_metric"):
            store.search("ns_metric", [0.1] * 128, distance_metric="manhattan")

    def test_store_unsupported_metadata_warns_and_keeps_supported(self, tmp_path):
        store = VantaDBLiteLLM(str(tmp_path), namespace="ns_meta")
        with pytest.warns(UserWarning, match="unsupported value types"):
            rid = store.store(
                "warn-test", [0.8] * 128, {"ok": "yes", "bad": {"nested": True}}
            )
        key = rid.split(":", 1)[1]
        record = store.get("ns_meta", key)
        assert record["metadata"]["ok"] == "yes"
        assert "bad" not in record["metadata"]

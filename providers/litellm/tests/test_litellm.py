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
        assert results[0]["payload"] == "litellm test"

    def test_get_record(self, tmp_path):
        store = VantaDBLiteLLM(str(tmp_path), namespace="ns_get")
        embedding = [0.2] * 128
        rid = store.store("get me", embedding)
        key = rid.split(":", 1)[1]
        record = store.get("ns_get", key)
        assert record is not None
        assert record["payload"] == "get me"
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
        texts = {r["payload"] for r in page["records"]}
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

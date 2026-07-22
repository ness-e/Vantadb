"""Tests for the VantaDB LiteLLM adapter."""

import pytest
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
        results = store.search(embedding, top_k=5)
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
        results = store.search(embedding, top_k=5)
        assert len(results) > 0
        for r in results:
            assert "score" in r
            assert isinstance(r["score"], float)

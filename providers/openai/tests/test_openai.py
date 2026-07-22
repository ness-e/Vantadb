"""Tests for the VantaDB OpenAI adapter."""

import pytest
from vantadb_openai import VantaDBOpenAI, __version__


class TestVantaDBOpenAI:
    def test_version(self):
        assert isinstance(__version__, str)
        assert len(__version__) > 0

    def test_init(self, tmp_path):
        store = VantaDBOpenAI(str(tmp_path), api_key="test-key")
        assert store is not None

    def test_init_custom_namespace(self, tmp_path):
        store = VantaDBOpenAI(str(tmp_path), api_key="test-key", namespace="custom_ns")
        assert store is not None

    def test_store_and_search(self, tmp_path):
        store = VantaDBOpenAI(str(tmp_path), api_key="test-key")
        embedding = [0.1] * 128
        rid = store.store("test text", embedding)
        assert ":" in rid
        results = store.search(embedding, top_k=5)
        assert len(results) > 0
        assert results[0]["text"] == "test text"

    def test_store_with_metadata(self, tmp_path):
        store = VantaDBOpenAI(str(tmp_path), api_key="test-key")
        rid = store.store("meta text", [0.2] * 128, {"lang": "en"})
        assert ":" in rid

    def test_get_record(self, tmp_path):
        store = VantaDBOpenAI(str(tmp_path), api_key="test-key")
        embedding = [0.3] * 128
        rid = store.store("get-test-text", embedding, {"key1": "val1", "num": 42})
        ns, key = rid.split(":", 1)

        record = store.get(ns, key)
        assert record is not None
        assert record["namespace"] == ns
        assert record["key"] == key
        assert record["text"] == "get-test-text"
        assert record["metadata"]["key1"] == "val1"
        assert record["metadata"]["num"] == 42
        assert record["created_at_ms"] > 0
        assert record["version"] >= 0

        # verify not found returns None
        missing = store.get(ns, "nonexistent_key")
        assert missing is None

    def test_delete_record(self, tmp_path):
        store = VantaDBOpenAI(str(tmp_path), api_key="test-key")
        embedding = [0.4] * 128
        rid = store.store("delete-test", embedding)
        ns, key = rid.split(":", 1)

        assert store.get(ns, key) is not None

        deleted = store.delete(key)
        assert deleted is True

        assert store.get(ns, key) is None

        # deleting again should return False
        deleted_again = store.delete(key)
        assert deleted_again is False

    def test_list_records(self, tmp_path):
        store = VantaDBOpenAI(str(tmp_path), api_key="test-key")
        emb = [0.5] * 128
        store.store("list-item-1", emb)
        store.store("list-item-2", emb)
        store.store("list-item-3", emb)

        ns = "openai_store"
        result = store.list(ns, limit=10)
        assert len(result["records"]) >= 3

        texts = {r["text"] for r in result["records"]}
        assert "list-item-1" in texts
        assert "list-item-2" in texts
        assert "list-item-3" in texts

    def test_list_namespaces(self, tmp_path):
        store = VantaDBOpenAI(str(tmp_path), api_key="test-key")
        emb = [0.6] * 128
        store.store("namespace-test", emb)

        namespaces = store.list_namespaces()
        assert "openai_store" in namespaces

    def test_search_with_metadata(self, tmp_path):
        store = VantaDBOpenAI(str(tmp_path), api_key="test-key")
        emb = [0.7] * 128
        store.store("search-meta-1", emb, {"lang": "en", "score": 95})
        store.store("search-meta-2", [0.71] * 128, {"lang": "es", "score": 80})

        ns = "openai_store"
        results = store.search(ns, emb, top_k=5)
        assert len(results) >= 2

        for r in results:
            assert "namespace" in r
            assert "key" in r
            assert "text" in r
            assert "metadata" in r
            assert "score" in r
            assert "created_at_ms" in r
            assert "version" in r
            assert isinstance(r["metadata"], dict)

        # find the exact match
        exact = [r for r in results if r["text"] == "search-meta-1"]
        assert len(exact) >= 1
        assert exact[0]["metadata"]["lang"] == "en"
        assert exact[0]["metadata"]["score"] == 95

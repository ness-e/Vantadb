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

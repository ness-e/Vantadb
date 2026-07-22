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
        assert results[0]["text"] == "litellm test"

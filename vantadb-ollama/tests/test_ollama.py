"""Tests for the VantaDB Ollama adapter."""

import pytest
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
        results = store.search(embedding, top_k=5)
        assert len(results) > 0
        assert results[0]["text"] == "ollama test"

    def test_store_with_metadata(self, tmp_path):
        store = VantaDBOllama(str(tmp_path))
        rid = store.store("meta test", [0.2] * 128, {"key": "val"})
        assert ":" in rid

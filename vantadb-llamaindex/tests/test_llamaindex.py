"""Tests for the VantaDB LlamaIndex adapter."""

import pytest
from vantadb_llamaindex import VantaDBVectorStore, __version__


class TestVantaDBVectorStore:
    def test_version(self):
        assert isinstance(__version__, str)
        assert len(__version__) > 0

    def test_init(self, tmp_path):
        store = VantaDBVectorStore(str(tmp_path))
        assert store is not None

    def test_add_and_query(self, tmp_path):
        store = VantaDBVectorStore(str(tmp_path))
        embedding = [0.1] * 128
        ids = store.add(["hello world"], [embedding])
        assert len(ids) == 1
        assert ":" in ids[0]
        results = store.query(embedding, top_k=5)
        assert len(results) > 0
        assert results[0]["text"] == "hello world"

    def test_add_with_metadata(self, tmp_path):
        store = VantaDBVectorStore(str(tmp_path))
        ids = store.add(["meta doc"], [[0.2] * 128], [{"source": "test"}])
        assert len(ids) == 1

    def test_delete(self, tmp_path):
        store = VantaDBVectorStore(str(tmp_path))
        ids = store.add(["to delete"], [[0.1] * 128])
        store.delete(ids)
        results = store.query([0.1] * 128, top_k=5)
        assert len(results) == 0

    def test_unique_keys_across_calls(self, tmp_path):
        store = VantaDBVectorStore(str(tmp_path))
        embedding = [0.1] * 128
        ids1 = store.add(["first"], [embedding])
        ids2 = store.add(["second"], [embedding])
        assert ids1[0] != ids2[0], "keys must be unique across calls"

"""Tests for the VantaDB LangChain adapter."""

import pytest
from vantadb_langchain import VantaDBVectorStore, __version__


class TestVantaDBVectorStore:
    def test_version(self):
        assert isinstance(__version__, str)
        assert len(__version__) > 0

    def test_init(self, tmp_path):
        store = VantaDBVectorStore(str(tmp_path))
        assert store is not None

    def test_add_and_search(self, tmp_path):
        store = VantaDBVectorStore(str(tmp_path))
        embedding = [0.1] * 128
        ids = store.add_texts(["hello world"], [embedding])
        assert len(ids) == 1
        assert ":" in ids[0]
        results = store.similarity_search_by_vector(embedding, k=5)
        assert len(results) > 0
        assert results[0]["text"] == "hello world"

    def test_add_with_metadata(self, tmp_path):
        store = VantaDBVectorStore(str(tmp_path))
        ids = store.add_texts(["meta doc"], [[0.2] * 128], [{"source": "test"}])
        assert len(ids) == 1

    def test_delete(self, tmp_path):
        store = VantaDBVectorStore(str(tmp_path))
        ids = store.add_texts(["to delete"], [[0.1] * 128])
        store.delete(ids)
        results = store.similarity_search_by_vector([0.1] * 128, k=5)
        assert len(results) == 0

    def test_unique_keys_across_calls(self, tmp_path):
        store = VantaDBVectorStore(str(tmp_path))
        embedding = [0.1] * 128
        ids1 = store.add_texts(["first"], [embedding])
        ids2 = store.add_texts(["second"], [embedding])
        assert ids1[0] != ids2[0], "keys must be unique across calls"

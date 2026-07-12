"""Tests for the VantaDB Haystack adapter."""

import pytest
from vantadb_haystack import VantaDBDocumentStore, __version__


class TestVantaDBDocumentStore:
    def test_version(self):
        assert isinstance(__version__, str)
        assert len(__version__) > 0

    def test_init(self, tmp_path):
        store = VantaDBDocumentStore(str(tmp_path))
        assert store is not None

    def test_init_custom_namespace(self, tmp_path):
        store = VantaDBDocumentStore(str(tmp_path), namespace="custom_docs")
        assert store is not None

    def test_write_and_filter(self, tmp_path):
        store = VantaDBDocumentStore(str(tmp_path))
        docs = [
            {"id": "1", "content": "doc 1", "embedding": [0.1] * 128},
            {"id": "2", "content": "doc 2", "embedding": [0.2] * 128},
        ]
        ids = store.write_documents(docs)
        assert len(ids) == 2
        results = store.filter_documents(top_k=10)
        assert len(results) == 2

    def test_write_with_metadata(self, tmp_path):
        store = VantaDBDocumentStore(str(tmp_path))
        docs = [{"id": "3", "content": "meta doc", "embedding": [0.3] * 128, "metadata": {"source": "test"}}]
        store.write_documents(docs)
        results = store.filter_documents(top_k=10)
        assert len(results) == 1

    def test_count_documents(self, tmp_path):
        store = VantaDBDocumentStore(str(tmp_path))
        docs = [{"id": str(i), "content": f"doc {i}"} for i in range(5)]
        store.write_documents(docs)
        assert store.count_documents() == 5

    def test_delete_documents(self, tmp_path):
        store = VantaDBDocumentStore(str(tmp_path))
        store.write_documents([{"id": "1", "content": "to delete"}])
        store.delete_documents(["1"])
        assert store.count_documents() == 0

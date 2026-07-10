"""Tests for VantaDB Ollama adapter."""
import pytest
import tempfile
import os
import sys
sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from vantadb_ollama import VantaDBOllama


class FakeOllama:
    @staticmethod
    def embeddings(**kwargs):
        return {"embedding": [0.1] * 4}


@pytest.fixture
def store(monkeypatch):
    path = os.path.join(tempfile.mkdtemp(), "test_ol")
    monkeypatch.setattr("vantadb_ollama.vectorstore.ollama", FakeOllama)
    s = VantaDBOllama(db_path=path, namespace="test_ol")
    yield s


def test_add_and_search(store):
    ids = store.add_texts(["hello world"])
    assert len(ids) == 1


def test_empty_search(store):
    results = store.similarity_search("nothing", k=5)
    assert len(results) == 0


def test_add_with_metadata(store):
    ids = store.add_texts(["data"], metadatas=[{"type": "test"}])
    assert len(ids) == 1


def test_delete(store):
    ids = store.add_texts(["delete me"])
    assert store.delete(ids) is True

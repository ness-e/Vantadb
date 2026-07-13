"""Tests for VantaDB OpenAI adapter."""
import pytest
import tempfile
import os
import sys
sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from vantadb_openai import VantaDBOpenAI


class FakeEmbeddings:
    def create(self, **kwargs):
        return type("R", (), {
            "data": [type("D", (), {"embedding": [0.1] * 4})() for _ in kwargs.get("input", [])]
        })()


class FakeOpenAI:
    def __init__(self):
        self.embeddings = FakeEmbeddings()


@pytest.fixture
def store():
    path = os.path.join(tempfile.mkdtemp(), "test_oa")
    s = VantaDBOpenAI(api_key="sk-test", db_path=path, namespace="test_oa", client=FakeOpenAI())
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

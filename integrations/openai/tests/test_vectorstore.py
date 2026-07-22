"""Tests for VantaDB OpenAI adapter."""
import pytest
import tempfile
import os
import sys
sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from vantadb_openai import VantaDBOpenAI


class FakeEmbeddings:
    def create(self, **kwargs):
        inputs = kwargs.get("input", [])
        if isinstance(inputs, str):
            inputs = [inputs]
        return type("R", (), {
            "data": [type("D", (), {"embedding": [0.1] * 4})() for _ in inputs]
        })()


class FakeOpenAI:
    def __init__(self):
        self.embeddings = FakeEmbeddings()


@pytest.fixture
def store():
    path = os.path.join(tempfile.mkdtemp(), "test_oa")
    s = VantaDBOpenAI(api_key="sk-test", db_path=path, namespace="test_oa", client=FakeOpenAI())
    yield s


def test_add_texts(store):
    ids = store.add_texts(["hello world", "test doc"])
    assert len(ids) == 2
    assert all(isinstance(i, str) for i in ids)


def test_similarity_search(store):
    store.add_texts(["hello world", "test doc"])
    results = store.similarity_search("hello", k=5)
    assert len(results) >= 1


def test_similarity_search_empty(store):
    results = store.similarity_search("nothing", k=5)
    assert len(results) == 0


def test_add_with_metadata(store):
    ids = store.add_texts(["data"], metadatas=[{"type": "test"}])
    assert len(ids) == 1


def test_delete(store):
    ids = store.add_texts(["delete me"])
    assert store.delete(ids) is True


def test_add_empty_texts(store):
    ids = store.add_texts([])
    assert ids == []


def test_add_none_metadata(store):
    ids = store.add_texts(["doc1"], metadatas=None)
    assert len(ids) == 1


def test_add_empty_string(store):
    """Empty strings should not crash."""
    ids = store.add_texts([""])
    assert len(ids) == 1


@pytest.mark.asyncio
async def test_aadd_and_asearch(store):
    ids = await store.aadd_texts(["async doc1", "async doc2"])
    assert len(ids) == 2
    results = await store.asimilarity_search("async", k=5)
    assert len(results) >= 1

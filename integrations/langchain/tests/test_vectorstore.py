"""Tests for VantaDB LangChain vector store adapter."""
import pytest
import tempfile
import os
import sys
sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from vantadb_langchain import VantaDBVectorStore
from langchain_core.documents import Document
import vantadb_py as vanta


class FakeEmbeddings:
    def embed_query(self, text: str):
        return [0.1] * 4
    def embed_documents(self, texts):
        return [[0.1] * 4 for _ in texts]


@pytest.fixture
def store():
    embeddings = FakeEmbeddings()
    path = os.path.join(tempfile.mkdtemp(), "test_lc")
    store = VantaDBVectorStore(embeddings, db_path=path, namespace="test_lc")
    yield store
    # Cleanup could be added


def test_add_and_search(store):
    docs = [Document(page_content="hello world", metadata={"type": "greeting"})]
    ids = store.add_documents(docs)
    assert len(ids) == 1
    
    results = store.similarity_search("hello", k=5)
    assert len(results) >= 1
    assert "hello" in results[0].page_content


def test_delete_by_filter(store):
    docs = [Document(page_content="one", metadata={"tag": "a"}),
            Document(page_content="two", metadata={"tag": "b"})]
    store.add_documents(docs)
    
    count = store.delete_by_filter("tag", "a")
    assert count >= 1
    
    remaining = store.similarity_search("one", k=5)
    assert len(remaining) == 1


def test_metadata_filter(store):
    docs = [Document(page_content="cat", metadata={"kind": "animal"}),
            Document(page_content="car", metadata={"kind": "vehicle"})]
    store.add_documents(docs)
    
    results = store.similarity_search("cat", k=5, filter_key="kind", filter_val="animal")
    assert len(results) >= 1
    assert results[0].page_content == "cat"


def test_empty_store(store):
    results = store.similarity_search("nothing", k=5)
    assert len(results) == 0


def test_get_by_ids(store):
    docs = [Document(page_content="test")]
    ids = store.add_documents(docs)
    found = store.get_by_ids(ids)
    assert len(found) == 1

"""Tests for VantaDB Haystack adapter."""
import pytest
import tempfile
import os
import sys
sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from vantadb_haystack import VantaDBDocumentStore


@pytest.fixture
def store():
    path = os.path.join(tempfile.mkdtemp(), "test_hs")
    s = VantaDBDocumentStore(db_path=path)
    yield s


def test_write_and_filter(store):
    docs = [
        {"id": "1", "content": "hello world", "meta": {"type": "greeting"}},
    ]
    store.write_documents(docs)
    results = store.filter_documents()
    assert len(results) >= 1


def test_filter_by_field(store):
    store.write_documents([
        {"id": "a", "content": "cat", "meta": {"kind": "animal"}},
        {"id": "b", "content": "car", "meta": {"kind": "vehicle"}},
    ])
    results = store.filter_documents(filters={"kind": "animal"})
    assert len(results) >= 1
    assert "cat" in results[0].content


def test_count(store):
    store.write_documents([
        {"id": "1", "content": "one", "meta": {}},
        {"id": "2", "content": "two", "meta": {}},
    ])
    assert store.count_documents() >= 2


def test_delete(store):
    store.write_documents([{"id": "x", "content": "delete me", "meta": {}}])
    assert store.count_documents() >= 1
    store.delete_documents(filters={"id": "x"})
    assert store.count_documents() == 0


def test_empty_store(store):
    assert store.count_documents() == 0
    assert store.filter_documents() == []

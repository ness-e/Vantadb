"""Tests for VantaDB Letta adapter."""
import pytest
import tempfile
import os
import sys
sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from vantadb_letta import VantaDBVectorStore


@pytest.fixture
def store():
    path = os.path.join(tempfile.mkdtemp(), "test_lt")
    s = VantaDBVectorStore(db_path=path, namespace="test_lt")
    yield s


def test_insert_and_search(store):
    store.insert("hello world", source="test")
    results = store.search("hello", k=5)
    assert len(results) >= 1
    assert "hello" in results[0]["text"]


def test_empty_search(store):
    results = store.search("nothing", k=5)
    assert len(results) == 0


def test_insert_with_metadata(store):
    store.insert("secret data", metadata={"type": "secret"})
    results = store.search("secret", k=5)
    assert len(results) >= 1


def test_delete(store):
    store.insert("delete me")
    results = store.search("delete", k=5)
    assert len(results) >= 1
    store.delete(results[0]["key"])
    results = store.search("delete", k=5)
    assert len(results) == 0


def test_list(store):
    store.insert("a")
    store.insert("b")
    items = store.list(limit=100)
    assert len(items) >= 2

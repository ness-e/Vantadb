"""Tests for VantaDB Mem0 adapter."""
import pytest
import tempfile
import os
import sys
sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from vantadb_mem0 import VantaDBVectorStore


@pytest.fixture
def store():
    path = os.path.join(tempfile.mkdtemp(), "test_mem0")
    s = VantaDBVectorStore(db_path=path, namespace="test_mem0")
    yield s


def test_add_and_search(store):
    store.add("hello world", user_id="alice")
    results = store.search("hello", k=5)
    assert len(results) >= 1


def test_empty_search(store):
    results = store.search("nothing", k=5)
    assert len(results) == 0


def test_add_with_metadata(store):
    store.add("secret", user_id="bob", metadata={"type": "secret"})
    results = store.search("secret", k=5)
    assert len(results) >= 1


def test_list_with_user(store):
    store.add("doc1", user_id="alice")
    store.add("doc2", user_id="bob")
    items = store.list("alice", limit=100)
    assert len(items) >= 1


def test_delete(store):
    store.add("delete_me", user_id="test")
    results = store.search("delete_me", k=5)
    assert len(results) >= 1
    store.delete(results[0]["key"])
    results = store.search("delete_me", k=5)
    assert len(results) == 0

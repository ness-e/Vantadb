"""Tests for VantaDB Letta adapter."""
import pytest
import tempfile
import os
import sys
sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from vantadb_letta import VantaDBVectorStore


# -- helpers ----------------------------------------------------------------

def _dummy_embed(text: str) -> list:
    """Deterministic dummy embedding — one float per char."""
    return [float(ord(c)) for c in text[:16]]


# -- fixtures ---------------------------------------------------------------


@pytest.fixture
def store():
    path = os.path.join(tempfile.mkdtemp(), "test_lt")
    s = VantaDBVectorStore(db_path=path, namespace="test_lt")
    yield s


@pytest.fixture
def store_with_embed():
    path = os.path.join(tempfile.mkdtemp(), "test_lt_emb")
    s = VantaDBVectorStore(db_path=path, namespace="test_lt_emb",
                            embedding=_dummy_embed)
    yield s


# -- core CRUD --------------------------------------------------------------


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


# -- insert / search with embedding ----------------------------------------


def test_insert_with_embedding(store_with_embed):
    key = store_with_embed.insert("hello embedded", source="emb")
    assert key is not None


def test_search_with_embedding(store_with_embed):
    store_with_embed.insert("cat")
    store_with_embed.insert("dog")
    results = store_with_embed.search("cat", k=5)
    # With embedding the search returns vector results with score
    assert len(results) >= 1
    assert "score" in results[0]


# -- serialisation roundtrip ------------------------------------------------


def test_to_dict(store):
    d = store.to_dict()
    assert d["db_path"] == store.path
    assert d["namespace"] == store.namespace


def test_to_dict_from_dict_roundtrip(store):
    d = store.to_dict()
    # Use a different db_path so from_dict doesn't contend for the lock
    d["db_path"] = os.path.join(tempfile.mkdtemp(), "test_lt_rt")
    s2 = VantaDBVectorStore.from_dict(d)
    assert s2.namespace == store.namespace
    # Verify the new store works
    s2.insert("roundtrip ok")
    results = s2.search("roundtrip", k=5)
    assert len(results) >= 1


# -- list with filters ------------------------------------------------------


def test_list_with_filters(store):
    store.insert("a", metadata={"kind": "x"})
    store.insert("b", metadata={"kind": "x"})
    store.insert("c", metadata={"kind": "y"})
    items = store.list(limit=100, filters={"kind": "x"})
    assert len(items) >= 2
    for item in items:
        assert item["metadata"].get("kind") == "x"


def test_list_with_filters_no_match(store):
    store.insert("x", metadata={"env": "prod"})
    items = store.list(limit=100, filters={"env": "staging"})
    assert len(items) == 0


# -- edge cases -------------------------------------------------------------


@pytest.mark.parametrize("bad_text", ["", "   ", None])
def test_insert_empty_text_raises(store, bad_text):
    with pytest.raises(ValueError, match="non-empty"):
        store.insert(bad_text)


@pytest.mark.parametrize("bad_k", [0, -1, -100])
def test_search_invalid_k_raises(store, bad_k):
    with pytest.raises(ValueError, match="k must be > 0"):
        store.search("something", k=bad_k)

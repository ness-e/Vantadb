"""Tests for the VantaDB Ollama adapter."""

import pytest
import tempfile
import os
from vantadb_ollama import VantaDBOllama, __version__


class TestVantaDBOllama:
    def test_version(self):
        assert isinstance(__version__, str)
        assert len(__version__) > 0

    def test_init(self, tmp_path):
        store = VantaDBOllama(str(tmp_path))
        assert store is not None

    def test_init_custom_namespace(self, tmp_path):
        store = VantaDBOllama(str(tmp_path), namespace="custom_ns")
        assert store is not None

    def test_store_and_search(self, tmp_path):
        store = VantaDBOllama(str(tmp_path))
        embedding = [0.1] * 128
        rid = store.store("ollama test", embedding)
        assert ":" in rid
        results = store.search(embedding, top_k=5)
        assert len(results) > 0
        assert results[0]["text"] == "ollama test"

    def test_store_with_metadata(self, tmp_path):
        store = VantaDBOllama(str(tmp_path))
        rid = store.store("meta test", [0.2] * 128, {"key": "val"})
        assert ":" in rid


# ── Direct storage tests via vantadb_py ──────────────────────────────────

import vantadb_py as vanta


@pytest.fixture
def db():
    path = os.path.join(tempfile.mkdtemp(), "test_ollama")
    s = vanta.VantaDB(path)
    s.create_namespace("test_ollama")
    yield s


def test_get_record(db):
    record_id = db.put("test_ollama", "k1", "hello world", [0.1] * 128)
    record = db.get("test_ollama", "k1")
    assert record is not None
    assert record["key"] == "k1"
    assert record["text"] == "hello world"
    assert record["namespace"] == "test_ollama"
    assert "created_at_ms" in record
    assert "updated_at_ms" in record


def test_delete_record(db):
    db.put("test_ollama", "k_del", "delete me", [0.2] * 128)
    found = db.get("test_ollama", "k_del")
    assert found is not None
    db.delete("test_ollama", "k_del")
    gone = db.get("test_ollama", "k_del")
    assert gone is None


def test_list_records(db):
    for i in range(3):
        db.put("test_ollama", f"lst_{i}", f"item {i}", [0.3 + i * 0.01] * 128)
    page = db.list("test_ollama", limit=10)
    assert len(page["records"]) >= 3


def test_list_namespaces(db):
    namespaces = db.list_namespaces()
    assert "test_ollama" in namespaces

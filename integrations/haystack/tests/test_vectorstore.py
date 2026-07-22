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


# ── write_documents con DuplicatePolicy ──

def test_write_documents_overwrite(store):
    """OVERWRITE reemplaza contenido y metadata del documento existente."""
    store.write_documents([{"id": "1", "content": "original", "meta": {"v": 1}}])
    count_before = store.count_documents()

    from haystack.document_stores.types import DuplicatePolicy
    store.write_documents(
        [{"id": "1", "content": "overwritten", "meta": {"v": 2}}],
        policy=DuplicatePolicy.OVERWRITE,
    )
    assert store.count_documents() == count_before  # no se duplica

    # Leer todos los docs y verificar el reemplazo
    all_docs = store.filter_documents()
    doc1 = next(d for d in all_docs if d.id == "1")
    assert doc1.content == "overwritten"
    assert doc1.meta["v"] == 2


# ── filter_documents con operadores compuestos ──

def test_filter_documents_and(store):
    """AND retorna documentos que cumplen todas las condiciones."""
    store.write_documents([
        {"id": "1", "content": "brown dog", "meta": {"type": "animal", "color": "brown"}},
        {"id": "2", "content": "white dog", "meta": {"type": "animal", "color": "white"}},
        {"id": "3", "content": "brown table", "meta": {"type": "furniture", "color": "brown"}},
    ])
    filters = {
        "operator": "AND",
        "conditions": [
            {"field": "meta.type", "operator": "==", "value": "animal"},
            {"field": "meta.color", "operator": "==", "value": "brown"},
        ],
    }
    results = store.filter_documents(filters=filters)
    assert len(results) >= 1
    assert results[0].id == "1"


def test_filter_documents_or(store):
    """OR retorna documentos que cumplen al menos una condición."""
    store.write_documents([
        {"id": "a", "content": "cat", "meta": {"kind": "animal"}},
        {"id": "b", "content": "car", "meta": {"kind": "vehicle"}},
        {"id": "c", "content": "tree", "meta": {"kind": "plant"}},
    ])
    filters = {
        "operator": "OR",
        "conditions": [
            {"field": "meta.kind", "operator": "==", "value": "animal"},
            {"field": "meta.kind", "operator": "==", "value": "vehicle"},
        ],
    }
    results = store.filter_documents(filters=filters)
    assert len(results) >= 2
    ids = {r.id for r in results}
    assert "a" in ids
    assert "b" in ids


def test_filter_documents_not(store):
    """NOT excluye documentos que cumplen la condición."""
    store.write_documents([
        {"id": "x", "content": "exclude me", "meta": {"keep": False}},
        {"id": "y", "content": "keep me", "meta": {"keep": True}},
        {"id": "z", "content": "keep me too", "meta": {"keep": True}},
    ])
    filters = {
        "operator": "NOT",
        "conditions": [
            {"field": "meta.keep", "operator": "==", "value": False},
        ],
    }
    results = store.filter_documents(filters=filters)
    assert all(r.meta.get("keep") is True for r in results)
    ids = {r.id for r in results}
    assert "x" not in ids


# ── count_documents ──

def test_count_documents_many():
    """count_documents retorna el total sin límite artificial para conjuntos grandes."""
    path = os.path.join(tempfile.mkdtemp(), "test_hs_cnt")
    s = VantaDBDocumentStore(db_path=path)
    expected = 50
    for i in range(expected):
        s.write_documents([{"id": str(i), "content": f"doc {i}", "meta": {}}])
    assert s.count_documents() == expected


# ── to_dict / from_dict roundtrip ──

def test_to_dict_from_dict():
    """to_dict/from_dict roundtrip preserva parámetros de inicialización."""
    path = os.path.join(tempfile.mkdtemp(), "test_hs_ser")
    store = VantaDBDocumentStore(
        db_path=path,
        namespace="ser_test",
        memory_limit_bytes=65536,
        backend="flat",
    )
    data = store.to_dict()
    assert data["type"] == "VantaDBDocumentStore"
    params = data["init_parameters"]
    assert params["db_path"] == path
    assert params["namespace"] == "ser_test"
    assert params["memory_limit_bytes"] == 65536
    assert params["backend"] == "flat"
    assert "read_only" not in params  # False se omite

    # from_dict con path diferente para evitar lock de LSM
    path2 = os.path.join(tempfile.mkdtemp(), "test_hs_ser_copy")
    data["init_parameters"]["db_path"] = path2
    store2 = VantaDBDocumentStore.from_dict(data)
    assert store2._db_path == path2
    assert store2.namespace == "ser_test"
    assert store2._memory_limit_bytes == 65536
    assert store2._backend == "flat"


def test_to_dict_from_dict_minimal():
    """to_dict/from_dict con solo db_path."""
    path = os.path.join(tempfile.mkdtemp(), "test_hs_ser2")
    store = VantaDBDocumentStore(db_path=path)
    data = store.to_dict()
    assert data["type"] == "VantaDBDocumentStore"
    assert data["init_parameters"]["db_path"] == path
    assert data["init_parameters"]["namespace"] == "haystack"

    path2 = os.path.join(tempfile.mkdtemp(), "test_hs_ser2_copy")
    data["init_parameters"]["db_path"] = path2
    store2 = VantaDBDocumentStore.from_dict(data)
    assert store2._db_path == path2
    assert store2.namespace == "haystack"


# ── search con embedding ──

def test_search_with_embedding():
    """search con embedding mockeado retorna documentos."""
    path = os.path.join(tempfile.mkdtemp(), "test_hs_srch")
    emb = lambda x: [0.1, 0.2, 0.3]
    store = VantaDBDocumentStore(db_path=path, embedding=emb)
    store.write_documents([
        {"id": "1", "content": "hello world", "meta": {}},
        {"id": "2", "content": "goodbye world", "meta": {}},
    ])
    results = store.search("hello", k=5)
    assert len(results) >= 1
    assert results[0].content is not None

"""Tests for VantaDB LlamaIndex vector store adapter."""
import pytest
pytest.importorskip("llama_index", reason="llama_index SDK not installed; adapter suite skipped")
import tempfile
import os
import sys
sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from vantadb_llamaindex import VantaDBVectorStore
from llama_index.core.schema import TextNode, NodeRelationship, RelatedNodeInfo
from llama_index.core.vector_stores.types import (
    VectorStoreQuery,
    VectorStoreQueryMode,
    MetadataFilters,
    MetadataFilter,
    FilterOperator,
)


@pytest.fixture
def store():
    path = os.path.join(tempfile.mkdtemp(), "test_li")
    store = VantaDBVectorStore(db_path=path, namespace="test_li")
    yield store


def _node(text: str, id: str, embedding: list = None) -> TextNode:
    node = TextNode(text=text, id_=id)
    node.embedding = embedding or [0.1, 0.2, 0.3, 0.4]
    return node


# ── Existing tests (preserved) ─────────────────────────────────

def test_add_and_query(store):
    nodes = [_node("hello world", "1")]
    ids = store.add(nodes)
    assert ids == ["1"]

    query = VectorStoreQuery(
        query_embedding=[0.1]*4,
        similarity_top_k=5,
    )
    result = store.query(query)
    assert len(result.nodes) >= 1
    assert "hello" in result.nodes[0].text


def test_delete_by_ref_doc(store):
    n1 = _node("one", "n1")
    n1.relationships[NodeRelationship.SOURCE] = RelatedNodeInfo(node_id="doc_a")
    n2 = _node("two", "n2")
    n2.relationships[NodeRelationship.SOURCE] = RelatedNodeInfo(node_id="doc_b")
    store.add([n1, n2])

    store.delete("doc_a")

    result = store.query(VectorStoreQuery(query_embedding=[0.1]*4, similarity_top_k=5))
    assert all(n.node_id != "n1" for n in result.nodes)
    assert any(n.node_id == "n2" for n in result.nodes)


def test_empty_store(store):
    query = VectorStoreQuery(query_embedding=[0.1]*4, similarity_top_k=5)
    result = store.query(query)
    assert len(result.nodes) == 0


def test_get_nodes(store):
    nodes = [_node("test", "get1")]
    store.add(nodes)
    found = store.get_nodes(node_ids=["get1"])
    assert len(found) == 1


def test_clear(store):
    nodes = [_node("a", "a1"),
             _node("b", "b1")]
    store.add(nodes)
    store.clear()
    result = store.query(VectorStoreQuery(query_embedding=[0.1]*4, similarity_top_k=5))
    assert len(result.nodes) == 0


# ── New tests: add ─────────────────────────────────────────────

def test_add_multiple_nodes(store):
    nodes = [
        _node("first", "id1", [0.1, 0.2, 0.3, 0.4]),
        _node("second", "id2", [0.5, 0.6, 0.7, 0.8]),
        _node("third", "id3", [0.9, 1.0, 1.1, 1.2]),
    ]
    ids = store.add(nodes)
    assert ids == ["id1", "id2", "id3"]

    result = store.query(VectorStoreQuery(query_embedding=[0.1]*4, similarity_top_k=5))
    assert len(result.nodes) == 3


def test_add_empty_list(store):
    ids = store.add([])
    assert ids == []


def test_add_node_without_embedding_raises(store):
    # Node without explicit embedding — add() calls get_embedding() which raises
    node = TextNode(text="no vector", id_="nov")
    with pytest.raises(ValueError, match="embedding not set"):
        store.add([node])


# ── New tests: delete ──────────────────────────────────────────

def test_delete_nonexistent_ref_doc(store):
    # Should not raise
    store.delete("nonexistent_doc")
    # No-op is fine


def test_delete_multiple_ref_docs(store):
    n1 = _node("a", "na1")
    n1.relationships[NodeRelationship.SOURCE] = RelatedNodeInfo(node_id="doc_x")
    n2 = _node("b", "na2")
    n2.relationships[NodeRelationship.SOURCE] = RelatedNodeInfo(node_id="doc_x")
    n3 = _node("c", "nb1")
    n3.relationships[NodeRelationship.SOURCE] = RelatedNodeInfo(node_id="doc_y")
    store.add([n1, n2, n3])

    store.delete("doc_x")

    result = store.query(VectorStoreQuery(query_embedding=[0.1]*4, similarity_top_k=5))
    ids = [n.node_id for n in result.nodes]
    assert "na1" not in ids
    assert "na2" not in ids
    assert "nb1" in ids


# ── New tests: query ───────────────────────────────────────────

def test_query_with_none_embedding(store):
    store.add([_node("content", "q1")])
    query = VectorStoreQuery(query_embedding=None, similarity_top_k=5)
    result = store.query(query)
    assert len(result.nodes) == 0
    assert len(result.similarities) == 0
    assert len(result.ids) == 0


def test_query_with_filters(store):
    n1 = _node("cat content", "f1", [0.1, 0.2, 0.3, 0.4])
    n2 = _node("dog content", "f2", [0.5, 0.6, 0.7, 0.8])
    # Store metadata via node metadata
    n1.metadata["kind"] = "animal"
    n2.metadata["kind"] = "animal"
    store.add([n1, n2])

    filters = MetadataFilters(
        filters=[MetadataFilter(key="kind", value="animal", operator=FilterOperator.EQ)]
    )
    query = VectorStoreQuery(
        query_embedding=[0.1]*4,
        similarity_top_k=5,
        filters=filters,
    )
    result = store.query(query)
    # Both have kind=animal, so both should match
    assert len(result.nodes) == 2


def test_query_with_hybrid_mode(store):
    n1 = _node("synthetic biology", "h1", [0.1, 0.2, 0.3, 0.4])
    n2 = _node("machine learning", "h2", [0.5, 0.6, 0.7, 0.8])
    store.add([n1, n2])

    query = VectorStoreQuery(
        query_embedding=[0.1]*4,
        similarity_top_k=5,
        query_str="biology",
        mode=VectorStoreQueryMode.HYBRID,
    )
    result = store.query(query)
    assert len(result.nodes) >= 1


# ── New tests: get_nodes ───────────────────────────────────────

def test_get_nodes_nonexistent_ids(store):
    found = store.get_nodes(node_ids=["no-such-node"])
    assert found == []


def test_get_nodes_empty_ids(store):
    found = store.get_nodes(node_ids=[])
    assert found == []


def test_get_nodes_multiple(store):
    nodes = [_node("x", "gx"), _node("y", "gy")]
    store.add(nodes)
    found = store.get_nodes(node_ids=["gx", "gy"])
    assert len(found) == 2


# ── New tests: delete_nodes ────────────────────────────────────

def test_delete_nodes_by_ids(store):
    nodes = [_node("a", "da1"), _node("b", "da2")]
    store.add(nodes)
    store.delete_nodes(node_ids=["da1"])
    found = store.get_nodes(node_ids=["da1", "da2"])
    assert len(found) == 1
    assert found[0].node_id == "da2"


def test_delete_nodes_nonexistent(store):
    # Should not raise
    store.delete_nodes(node_ids=["no-such"])


# ── New tests: edge cases ──────────────────────────────────────

def test_query_with_k_zero_defaults_to_top_k(store):
    # k=0 is falsy in Python — the adapter falls back to DEFAULT_TOP_K (4)
    store.add([_node("content", "e1")])
    query = VectorStoreQuery(query_embedding=[0.1]*4, similarity_top_k=0)
    result = store.query(query)
    # Should still return results because 0 -> DEFAULT_TOP_K
    assert len(result.nodes) >= 1


def test_client_and_namespace_properties(store):
    assert store.client is not None
    assert store.namespace == "test_li"


def test_add_roundtrip_preserves_text(store):
    original = "exact text preservation test"
    nodes = [_node(original, "rt1")]
    store.add(nodes)
    found = store.get_nodes(node_ids=["rt1"])
    assert len(found) == 1
    assert found[0].text == original


# ── QW-3: attrs privados + imports completos ──

def test_method_type_hints_resolve():
    """Las anotaciones de tipo de los métodos se resuelven bajo get_type_hints.

    Regresión: MetadataFilter se usaba en firmas sin estar importado.
    """
    import typing
    hints = typing.get_type_hints(VantaDBVectorStore._build_vanta_filters)
    assert "return" in hints


def test_private_attrs_declared_and_serialization_clean(store):
    """Attrs privados declarados como PrivateAttr no filtran en model_dump."""
    store.add([_node("serial doc", "s1")])
    data = store.model_dump()
    assert "_client" not in data
    assert "_namespace" not in data

"""Tests for VantaDB LlamaIndex vector store adapter."""
import pytest
import tempfile
import os
import sys
sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from vantadb_llamaindex import VantaDBVectorStore
from llama_index.core.schema import TextNode
from llama_index.core.vector_stores.types import VectorStoreQuery


@pytest.fixture
def store():
    path = os.path.join(tempfile.mkdtemp(), "test_li")
    store = VantaDBVectorStore(db_path=path, namespace="test_li")
    yield store


def _node(text: str, id: str) -> TextNode:
    node = TextNode(text=text, id_=id)
    node.embedding = [0.1, 0.2, 0.3, 0.4]
    return node


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
    from llama_index.core.schema import NodeRelationship, RelatedNodeInfo
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

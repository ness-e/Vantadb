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


def test_add_and_query(store):
    nodes = [TextNode(text="hello world", id_="1")]
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
    nodes = [TextNode(text="one", id_="n1"),
             TextNode(text="two", id_="n2")]
    store.add(nodes)
    
    store.delete("n1")
    
    result = store.query(VectorStoreQuery(query_embedding=[0.1]*4, similarity_top_k=5))
    assert all(n.node_id != "n1" for n in result.nodes)


def test_empty_store(store):
    query = VectorStoreQuery(query_embedding=[0.1]*4, similarity_top_k=5)
    result = store.query(query)
    assert len(result.nodes) == 0


def test_get_nodes(store):
    nodes = [TextNode(text="test", id_="get1")]
    store.add(nodes)
    found = store.get_nodes(node_ids=["get1"])
    assert len(found) == 1


def test_clear(store):
    nodes = [TextNode(text="a", id_="a1"),
             TextNode(text="b", id_="b1")]
    store.add(nodes)
    store.clear()
    result = store.query(VectorStoreQuery(query_embedding=[0.1]*4, similarity_top_k=5))
    assert len(result.nodes) == 0

"""Tests for VantaDB DSPy adapter."""
import pytest
import tempfile
import os
import sys
sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from vantadb_dspy import VantaDBRetriever


@pytest.fixture
def retriever():
    path = os.path.join(tempfile.mkdtemp(), "test_dspy")
    r = VantaDBRetriever(db_path=path, namespace="test_dspy")
    r._add("hello world", "greeting")
    r._add("goodbye world", "farewell")
    yield r


def test_forward(retriever):
    result = retriever("hello")
    assert len(result) >= 1


def test_empty(retriever):
    result = retriever("nothing")
    assert len(result) == 0


def test_k_param():
    path = os.path.join(tempfile.mkdtemp(), "test_dspy_k")
    r = VantaDBRetriever(db_path=path, namespace="td", k=3)
    for i in range(5):
        r._add(f"doc{i}", str(i))
    result = r("doc")
    assert len(result) <= 3

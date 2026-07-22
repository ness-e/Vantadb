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


# ── forward retorna dspy.Prediction con .passages ──

def test_forward_returns_prediction_with_passages(retriever):
    """forward retorna dspy.Prediction con .passages (o lista si dspy no está)."""
    result = retriever("hello")
    if hasattr(result, "passages"):
        # dspy.Prediction
        assert len(result.passages) >= 1
    else:
        # fallback list
        assert isinstance(result, list)
        assert len(result) >= 1


# ── dump_state ──

def test_dump_state():
    """dump_state serializa namespace, db_path, k, backend."""
    path = os.path.join(tempfile.mkdtemp(), "test_dspy_dump")
    r = VantaDBRetriever(
        db_path=path,
        namespace="test_dump",
        k=7,
        backend="flat",
    )
    state = r.dump_state()
    assert isinstance(state, dict)
    assert state["namespace"] == "test_dump"
    assert state["db_path"] == path
    assert state["k"] == 7
    assert state["backend"] == "flat"


def test_dump_state_defaults():
    """dump_state incluye valores por defecto cuando no se especifican."""
    path = os.path.join(tempfile.mkdtemp(), "test_dspy_dump2")
    r = VantaDBRetriever(db_path=path)
    state = r.dump_state()
    assert state["namespace"] == "dspy"
    assert state["k"] == 4
    assert state["backend"] is None


# ── _add con metadata ──

def test_add_with_metadata():
    """_add con metadata se ejecuta sin error y el texto es recuperable."""
    path = os.path.join(tempfile.mkdtemp(), "test_dspy_add")
    r = VantaDBRetriever(db_path=path, namespace="test_add")
    r._add("document with metadata", "doc1", {"source": "test", "rank": 1})
    result = r("document")
    assert len(result) >= 1


# ── k passthrough ──

def test_k_passthrough():
    """k pasado como kwarg en forward limita la cantidad de resultados."""
    path = os.path.join(tempfile.mkdtemp(), "test_dspy_kpt")
    r = VantaDBRetriever(
        db_path=path,
        namespace="tkpt",
        k=10,
        embedding=lambda x: [0.5, 0.5, 0.5],
    )
    for i in range(5):
        r._add(f"doc{i}", str(i), {})

    result = r("doc", k=2)
    passages = result.passages if hasattr(result, "passages") else result
    assert len(passages) <= 2

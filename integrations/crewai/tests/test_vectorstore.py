"""Tests for VantaDB CrewAI adapter."""
import pytest
import tempfile
import os
import sys
sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from vantadb_crewai import VantaDBTool


@pytest.fixture
def tool():
    path = os.path.join(tempfile.mkdtemp(), "test_ca")
    t = VantaDBTool(
        name="test_search",
        description="Test tool",
        db_path=path,
        namespace="test_ca",
    )
    t._put("hello world", {"source": "test"})
    yield t


def test_tool_run(tool):
    result = tool._run("hello")
    assert "hello" in result


def test_tool_empty(tool):
    result = tool._run("nothing")
    assert result is not None


def test_tool_categorize(tool):
    result = tool.categorize("hello")
    assert isinstance(result, str)


# ── _put edge cases ──

def test_put_empty_raises(tool):
    """_put con texto vacío debe levantar ValueError."""
    with pytest.raises(ValueError, match="Text cannot be empty"):
        tool._put("")
    with pytest.raises(ValueError, match="Text cannot be empty"):
        tool._put("   ")


def test_put_with_embedding():
    """_put con embedding mockeado se ejecuta sin error y el texto es recuperable."""
    path = os.path.join(tempfile.mkdtemp(), "test_ca_emb")
    t = VantaDBTool(
        name="emb_test",
        description="Emb test",
        db_path=path,
        namespace="test_ca_emb",
        embedding=lambda x: [0.1, 0.2, 0.3],
    )
    t._put("embedded text", {"source": "embed_test"})
    result = t._run("embedded")
    assert "embedded" in result


def test_put_with_metadata():
    """_put almacena metadata y el texto es recuperable."""
    path = os.path.join(tempfile.mkdtemp(), "test_ca_meta")
    t = VantaDBTool(db_path=path, namespace="test_ca_meta")
    t._put("metadata test", {"key": "value", "num": 42})
    result = t._run("metadata")
    assert result is not None
    assert "metadata" in result


# ── categorize ──

def test_categorize_question():
    """categorize retorna 'question' para preguntas."""
    path = os.path.join(tempfile.mkdtemp(), "test_cat_q")
    t = VantaDBTool(db_path=path, namespace="test_cat_q")
    assert t.categorize("What is VantaDB?") == "question"
    assert t.categorize("How does this work") == "question"
    assert t.categorize("When will it be ready?") == "question"
    assert t.categorize("short?") == "question"
    assert t.categorize("Can you help?") == "question"


def test_categorize_technical():
    """categorize retorna 'technical' para texto técnico."""
    path = os.path.join(tempfile.mkdtemp(), "test_cat_t")
    t = VantaDBTool(db_path=path, namespace="test_cat_t")
    assert t.categorize("I have a bug in my code") == "technical"
    assert t.categorize("This function has an error") == "technical"
    assert t.categorize("The API returned an exception") == "technical"


def test_categorize_greeting():
    """categorize retorna 'greeting' para saludos."""
    path = os.path.join(tempfile.mkdtemp(), "test_cat_g")
    t = VantaDBTool(db_path=path, namespace="test_cat_g")
    assert t.categorize("hello there") == "greeting"
    assert t.categorize("hi how are you") == "greeting"
    assert t.categorize("good morning") == "greeting"
    assert t.categorize("hey") == "greeting"


def test_categorize_informational():
    """categorize retorna 'informational' para afirmaciones."""
    path = os.path.join(tempfile.mkdtemp(), "test_cat_i")
    t = VantaDBTool(db_path=path, namespace="test_cat_i")
    assert t.categorize("The sky is blue") == "informational"
    assert t.categorize("VantaDB is a vector database") == "informational"
    assert t.categorize("Today is Wednesday") == "informational"


def test_categorize_empty():
    """categorize retorna 'empty' para input vacío."""
    path = os.path.join(tempfile.mkdtemp(), "test_cat_e")
    t = VantaDBTool(db_path=path, namespace="test_cat_e")
    assert t.categorize("") == "empty"
    assert t.categorize("   ") == "empty"
    assert t.categorize("\t") == "empty"
    assert t.categorize("\n") == "empty"


# ── _run edge cases ──

def test_run_empty_query(tool):
    """_run con query vacía retorna mensaje claro."""
    assert tool._run("") == "No query provided."
    assert tool._run("   ") == "No query provided."

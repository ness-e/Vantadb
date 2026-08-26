"""Tests for VantaDB CrewAI adapter."""
import pytest
pytest.importorskip("crewai.tools", reason="crewai SDK not installed; adapter suite skipped")
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


# ── to_dict / from_dict roundtrip (QW-1) ──

def test_from_dict_roundtrip_no_typeerror():
    """from_dict no debe pasar el string embedding_model como callable.

    Regresión: to_dict serializa el embedding como nombre de tipo; from_dict
    lo pasaba crudo como ``embedding`` y _run/_put lanzaban TypeError.
    """
    path = os.path.join(tempfile.mkdtemp(), "test_ca_fd")
    t = VantaDBTool(
        db_path=path, namespace="test_ca_fd", embedding=lambda x: [0.1, 0.2, 0.3]
    )
    t._put("roundtrip doc")
    d = t.to_dict()
    assert d["embedding_model"] == "function"
    d["db_path"] = path + "_rt"  # path distinto para evitar lock de LSM
    t2 = VantaDBTool.from_dict(d)
    assert isinstance(t2._run("hello"), str)


def test_list_cursor_string():
    """list(cursor=...) acepta el cursor serializado como string (str→int)."""
    path = os.path.join(tempfile.mkdtemp(), "test_ca_cur")
    t = VantaDBTool(db_path=path, namespace="test_ca_cur")
    for i in range(5):
        t._put(f"doc {i}")
    page1 = t.list(limit=2)
    cursor = page1.get("cursor")
    page2 = t.list(limit=100, cursor=str(cursor) if cursor is not None else "0")
    assert isinstance(page2["records"], list)


# ── _run edge cases ──

def test_run_empty_query(tool):
    """_run con query vacía retorna mensaje claro."""
    assert tool._run("") == "No query provided."
    assert tool._run("   ") == "No query provided."

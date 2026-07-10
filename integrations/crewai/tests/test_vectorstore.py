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

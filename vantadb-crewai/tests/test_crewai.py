"""Tests for the VantaDB CrewAI adapter."""

import pytest
from vantadb_crewai import CrewAIMemory, __version__


class TestCrewAIMemory:
    def test_version(self):
        assert isinstance(__version__, str)
        assert len(__version__) > 0

    def test_init(self, tmp_path):
        memory = CrewAIMemory(str(tmp_path))
        assert memory is not None

    def test_init_custom_namespace(self, tmp_path):
        memory = CrewAIMemory(str(tmp_path), namespace="custom_ns")
        assert memory is not None

    def test_save_and_search(self, tmp_path):
        memory = CrewAIMemory(str(tmp_path))
        embedding = [0.1] * 128
        rid = memory.save("crew context", {"key": "val"}, embedding)
        assert ":" in rid
        results = memory.search(embedding, top_k=5)
        assert len(results) > 0
        assert results[0]["context"] == "crew context"

    def test_clear(self, tmp_path):
        memory = CrewAIMemory(str(tmp_path))
        memory.save("ctx1", {"k": "v"}, [0.1] * 128)
        memory.save("ctx2", {"k": "v"}, [0.2] * 128)
        memory.clear()
        results = memory.search([0.1] * 128, top_k=10)
        assert len(results) == 0

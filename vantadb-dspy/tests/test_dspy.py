"""Tests for the VantaDB DSPy adapter."""

import pytest
from vantadb_dspy import VantaDBRM, __version__


class TestVantaDBRM:
    def test_version(self):
        assert isinstance(__version__, str)
        assert len(__version__) > 0

    def test_init_defaults(self, tmp_path):
        rm = VantaDBRM(str(tmp_path))
        assert rm is not None

    def test_init_custom_collection(self, tmp_path):
        rm = VantaDBRM(str(tmp_path), collection="test_collection")
        assert rm is not None

    def test_add_and_forward(self, tmp_path):
        rm = VantaDBRM(str(tmp_path))
        embedding = [0.1] * 128
        rid = rm.add_passage("test passage", embedding)
        assert ":" in rid
        results = rm.forward(embedding, k=5)
        assert len(results) > 0
        assert "passage" in results[0]
        assert results[0]["passage"] == "test passage"

    def test_add_with_metadata(self, tmp_path):
        rm = VantaDBRM(str(tmp_path))
        embedding = [0.2] * 128
        rid = rm.add_passage("passage with meta", embedding, {"source": "test"})
        assert ":" in rid

    def test_forward_empty(self, tmp_path):
        rm = VantaDBRM(str(tmp_path))
        results = rm.forward([0.5] * 128, k=5)
        assert len(results) == 0

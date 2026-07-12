"""Tests for the VantaDB Letta adapter."""

import pytest
from vantadb_letta import LettaStore, __version__


class TestLettaStore:
    def test_version(self):
        assert isinstance(__version__, str)
        assert len(__version__) > 0

    def test_init(self, tmp_path):
        store = LettaStore(str(tmp_path))
        assert store is not None

    def test_store_and_retrieve(self, tmp_path):
        store = LettaStore(str(tmp_path))
        embedding = [0.1] * 128
        mid = store.store_memory("user1", "agent1", "my memory content", embedding)
        assert ":" in mid
        results = store.retrieve_memory("user1", "agent1", embedding, top_k=5)
        assert len(results) > 0
        assert results[0]["content"] == "my memory content"

    def test_list_memories(self, tmp_path):
        store = LettaStore(str(tmp_path))
        store.store_memory("u1", "a1", "content1", [0.1] * 128)
        store.store_memory("u1", "a1", "content2", [0.2] * 128)
        memories = store.list_memories("u1", "a1")
        assert len(memories) == 2

    def test_delete_memory(self, tmp_path):
        store = LettaStore(str(tmp_path))
        mid = store.store_memory("u1", "a1", "to delete", [0.1] * 128)
        store.delete_memory(mid)
        memories = store.list_memories("u1", "a1")
        assert len(memories) == 0

    def test_delete_invalid_id(self, tmp_path):
        store = LettaStore(str(tmp_path))
        with pytest.raises(Exception):
            store.delete_memory("invalid_id")

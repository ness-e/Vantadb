"""Tests for the VantaDB Mem0 adapter."""

import pytest
from vantadb_mem0 import VantaDBStore, __version__


class TestVantaDBStore:
    def test_version(self):
        assert isinstance(__version__, str)
        assert len(__version__) > 0

    def test_init(self, tmp_path):
        store = VantaDBStore(str(tmp_path))
        assert store is not None

    def test_insert_and_search(self, tmp_path):
        store = VantaDBStore(str(tmp_path))
        vectors = [[0.1] * 128, [0.2] * 128]
        payloads = ["hello", "world"]
        ids = store.insert(vectors, payloads)
        assert len(ids) == 2
        results = store.search("hello", [vectors[0]], top_k=5)
        assert len(results) > 0

    def test_get(self, tmp_path):
        store = VantaDBStore(str(tmp_path))
        ids = store.insert([[0.1] * 128], ["test"])
        result = store.get(ids[0].split(":")[1])
        assert result is not None
        assert result["payload"] == "test"

    def test_delete(self, tmp_path):
        store = VantaDBStore(str(tmp_path))
        ids = store.insert([[0.1] * 128], ["to delete"])
        store.delete(ids[0].split(":")[1])
        assert store.get(ids[0].split(":")[1]) is None

    def test_list_cols(self, tmp_path):
        store = VantaDBStore(str(tmp_path))
        cols = store.list_cols()
        assert isinstance(cols, list)

    def test_col_info(self, tmp_path):
        store = VantaDBStore(str(tmp_path), collection_name="mycol")
        info = store.col_info()
        assert info["collection_name"] == "mycol"

    def test_list(self, tmp_path):
        store = VantaDBStore(str(tmp_path))
        store.insert([[0.1] * 128], ["a"])
        store.insert([[0.2] * 128], ["b"])
        items = store.list(top_k=10)
        assert len(items) == 2

    def test_update(self, tmp_path):
        store = VantaDBStore(str(tmp_path))
        ids = store.insert([[0.1] * 128], ["original"])
        store.update(ids[0].split(":")[1], payload="updated")
        result = store.get(ids[0].split(":")[1])
        assert result["payload"] == "updated"

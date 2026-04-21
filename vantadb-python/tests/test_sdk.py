"""
Vantadb Python SDK — Integration Test Suite
Tests the full CRUD + Search + IQL lifecycle via PyO3 in-process bindings.
"""

import os
import shutil
import pytest

# The module name matches [lib] name in Cargo.toml
import vantadb_py as vanta


TEST_DB_PATH = "./test_sdk_db"


@pytest.fixture(autouse=True)
def cleanup():
    """Clean up test database before and after each test."""
    if os.path.exists(TEST_DB_PATH):
        shutil.rmtree(TEST_DB_PATH)
    yield
    if os.path.exists(TEST_DB_PATH):
        shutil.rmtree(TEST_DB_PATH)


import uuid


def _unique_path():
    return f"{TEST_DB_PATH}_{uuid.uuid4().hex[:8]}"


class TestVantaDBLifecycle:
    """Core CRUD lifecycle tests."""

    def test_open_and_repr(self):
        """VantaDB instance should open and display hardware profile."""
        db = vanta.VantaDB(_unique_path(), memory_limit_bytes=128 * 1024 * 1024)
        r = repr(db)
        assert "VantaDB(" in r
        assert "profile=" in r

    def test_insert_and_get(self):
        """Insert a node and retrieve it by ID."""
        db = vanta.VantaDB(_unique_path(), memory_limit_bytes=128 * 1024 * 1024)
        db.insert(42, "Hello VantaDB", [0.1] * 384)

        node = db.get(42)
        assert node is not None
        assert node["id"] == 42
        assert node["fields"]["content"] == "Hello Vantadb"
        assert node["vector_dims"] == 384
        assert node["is_alive"] is True

    def test_insert_with_extra_fields(self):
        """Insert with additional relational fields from a Python dict."""
        db = vanta.VantaDB(_unique_path(), memory_limit_bytes=128 * 1024 * 1024)
        db.insert(
            1,
            "Test node",
            [0.5] * 128,
            fields={
                "category": "test",
                "score": 42,
                "active": True,
            },
        )

        node = db.get(1)
        assert node is not None
        assert node["fields"]["category"] == "test"
        assert node["fields"]["score"] == 42
        assert node["fields"]["active"] is True

    def test_get_nonexistent(self):
        """Getting a non-existent node returns None."""
        db = vanta.VantaDB(_unique_path(), memory_limit_bytes=128 * 1024 * 1024)
        assert db.get(999999) is None

    def test_delete_tombstone(self):
        """Deleting a node should make it unretrievable."""
        db = vanta.VantaDB(_unique_path(), memory_limit_bytes=128 * 1024 * 1024)
        db.insert(10, "To be deleted", [0.2] * 128)
        assert db.get(10) is not None

        db.delete(10, "test cleanup")
        assert db.get(10) is None

    def test_flush(self):
        """Flush should persist data to disk without errors."""
        db = vanta.VantaDB(_unique_path(), memory_limit_bytes=128 * 1024 * 1024)
        db.insert(1, "Persistent data", [0.3] * 128)
        db.flush()  # Should not raise


class TestVectorSearch:
    """K-NN vector search tests."""

    def test_search_returns_results(self):
        """Search should find inserted vectors."""
        db = vanta.VantaDB(_unique_path(), memory_limit_bytes=128 * 1024 * 1024)

        # Insert some vectors
        for i in range(10):
            vec = [float(i) * 0.1] * 384
            db.insert(i + 1, f"Node {i}", vec)

        # Search for the first one
        results = db.search([0.0] * 384, top_k=5)
        assert len(results) > 0
        # Results are (node_id, distance) tuples
        assert all(isinstance(r, tuple) and len(r) == 2 for r in results)


class TestHardwareIntrospection:
    """Hardware profile introspection tests."""

    def test_hardware_profile(self):
        """Hardware profile should return valid keys."""
        db = vanta.VantaDB(_unique_path(), memory_limit_bytes=128 * 1024 * 1024)
        hw = db.hardware_profile()

        assert "profile" in hw
        assert "instructions" in hw
        assert "logical_cores" in hw
        assert "total_memory" in hw
        assert "vitality_score" in hw
        assert hw["logical_cores"] > 0


class TestEdgeManagement:
    """Graph edge tests."""

    def test_add_edge(self):
        """Adding an edge between two nodes."""
        db = vanta.VantaDB(_unique_path(), memory_limit_bytes=128 * 1024 * 1024)
        db.insert(1, "Source", [])
        db.insert(2, "Target", [])

        db.add_edge(1, 2, "relates_to", weight=0.95)

        node = db.get(1)
        assert node is not None
        assert len(node["edges"]) > 0
        edge = node["edges"][0]
        assert edge[0] == 2  # target
        assert edge[1] == "relates_to"  # label


class TestMemoryBoundary:
    """Memory budget isolation tests."""

    def test_explicit_memory_limit(self):
        """DB should respect explicit memory limit via constructor."""
        # 64MB — this should activate resource governance
        db = vanta.VantaDB(_unique_path(), memory_limit_bytes=64 * 1024 * 1024)

        hw = db.hardware_profile()
        # The engine should have initialized without crashing
        assert hw is not None


if __name__ == "__main__":
    pytest.main([__file__, "-v"])

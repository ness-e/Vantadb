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
        assert node["fields"]["content"] == "Hello VantaDB"
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

    def test_close_and_reopen(self):
        """Close should flush the embedded handle and allow reopen."""
        path = _unique_path()
        db = vanta.VantaDB(path, memory_limit_bytes=128 * 1024 * 1024)
        db.insert(7, "Reopen me", [0.4] * 16)
        db.flush()
        db.close()

        reopened = vanta.VantaDB(path, memory_limit_bytes=128 * 1024 * 1024)
        node = reopened.get(7)
        assert node is not None
        assert node["fields"]["content"] == "Reopen me"


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


class TestPersistentMemoryApi:
    """Namespace-scoped persistent memory API tests."""

    def test_put_get_list_search_memory(self):
        """Memory records should be namespace-scoped and searchable."""
        db = vanta.VantaDB(_unique_path(), memory_limit_bytes=128 * 1024 * 1024)

        record = db.put(
            "agent/main",
            "task-1",
            "ship the memory API",
            metadata={"category": "task", "done": False},
            vector=[1.0, 0.0, 0.0],
        )

        assert record["namespace"] == "agent/main"
        assert record["key"] == "task-1"
        assert record["payload"] == "ship the memory API"
        assert record["version"] == 1
        assert record["metadata"]["category"] == "task"

        fetched = db.get_memory("agent/main", "task-1")
        assert fetched is not None
        assert fetched["node_id"] == record["node_id"]

        page = db.list_memory("agent/main", filters={"category": "task"})
        assert len(page["records"]) == 1
        assert page["records"][0]["key"] == "task-1"

        hits = db.search_memory(
            "agent/main",
            [1.0, 0.0, 0.0],
            filters={"category": "task"},
            top_k=3,
        )
        assert len(hits) == 1
        assert hits[0]["record"]["key"] == "task-1"

        text_hits = db.search_memory(
            "agent/main",
            [],
            text_query="memory API",
            top_k=3,
        )
        assert len(text_hits) == 1
        assert text_hits[0]["record"]["key"] == "task-1"

        with pytest.raises(RuntimeError):
            db.search_memory(
                "agent/main",
                [1.0, 0.0, 0.0],
                text_query="memory API",
                top_k=3,
            )

    def test_memory_close_and_reopen(self):
        """Memory records should survive flush/close/reopen."""
        path = _unique_path()
        db = vanta.VantaDB(path, memory_limit_bytes=128 * 1024 * 1024)
        db.put("agent/main", "persist", "persistent payload")
        db.flush()
        db.close()

        reopened = vanta.VantaDB(path, memory_limit_bytes=128 * 1024 * 1024)
        record = reopened.get_memory("agent/main", "persist")
        assert record is not None
        assert record["payload"] == "persistent payload"

    def test_delete_memory(self):
        """Deleting a memory record should make it unretrievable."""
        db = vanta.VantaDB(_unique_path(), memory_limit_bytes=128 * 1024 * 1024)
        db.put("agent/main", "delete-me", "temporary")
        assert db.delete_memory("agent/main", "delete-me") is True
        assert db.get_memory("agent/main", "delete-me") is None

    def test_rebuild_export_import_memory(self, tmp_path):
        """Python memory API should expose rebuild and JSONL movement."""
        source_path = str(tmp_path / "source")
        target_path = str(tmp_path / "target")
        export_path = str(tmp_path / "agent-main.jsonl")

        db = vanta.VantaDB(source_path, memory_limit_bytes=128 * 1024 * 1024)
        db.put(
            "agent/main",
            "export-me",
            "portable memory",
            metadata={"category": "note"},
            vector=[1.0, 0.0, 0.0],
        )
        db.flush()

        rebuild = db.rebuild_index()
        assert rebuild["success"] is True
        assert rebuild["scanned_nodes"] >= 1

        exported = db.export_namespace(export_path, "agent/main")
        assert exported["records_exported"] == 1
        assert os.path.exists(export_path)

        target = vanta.VantaDB(target_path, memory_limit_bytes=128 * 1024 * 1024)
        imported = target.import_file(export_path)
        assert imported["inserted"] == 1
        assert imported["errors"] == 0

        fetched = target.get_memory("agent/main", "export-me")
        assert fetched is not None
        assert fetched["payload"] == "portable memory"

        all_export_path = str(tmp_path / "all.jsonl")
        all_export = target.export_all(all_export_path)
        assert all_export["records_exported"] == 1

    def test_operational_metrics(self, tmp_path):
        """Operational metrics should be available through the Python SDK."""
        export_path = str(tmp_path / "metrics.jsonl")
        db = vanta.VantaDB(str(tmp_path / "metrics-db"), memory_limit_bytes=128 * 1024 * 1024)
        before = db.operational_metrics()

        db.put("agent/main", "metric", "payload", vector=[1.0, 0.0, 0.0])
        rebuild = db.rebuild_index()
        db.export_all(export_path)
        after = db.operational_metrics()

        assert "startup_ms" in after
        assert "wal_records_replayed" in after
        assert "derived_rebuild_ms" in after
        assert "text_index_rebuild_ms" in after
        assert "text_postings_written" in after
        assert "text_index_repairs" in after
        assert "text_lexical_queries" in after
        assert "text_lexical_query_ms" in after
        assert "text_candidates_scored" in after
        assert "text_consistency_audits" in after
        assert "text_consistency_audit_failures" in after
        assert "records_exported" in after
        assert after["ann_rebuild_scanned_nodes"] >= rebuild["scanned_nodes"]
        assert after["records_exported"] >= before["records_exported"] + 1


class TestHardwareIntrospection:
    """Stable capability surface tests."""

    def test_capabilities(self):
        """Capabilities should return the stable SDK-facing keys."""
        db = vanta.VantaDB(_unique_path(), memory_limit_bytes=128 * 1024 * 1024)
        caps = db.capabilities()

        assert "profile" in caps
        assert "read_only" in caps
        assert "persistence" in caps
        assert "vector_search" in caps
        assert "iql_queries" in caps
        assert caps["persistence"] is True

    def test_hardware_profile_alias(self):
        """hardware_profile remains as a backward-compatible alias."""
        db = vanta.VantaDB(_unique_path(), memory_limit_bytes=128 * 1024 * 1024)
        hw = db.hardware_profile()
        assert "profile" in hw
        assert "vector_search" in hw


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

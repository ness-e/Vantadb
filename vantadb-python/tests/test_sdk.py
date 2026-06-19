"""
VantaDB Python SDK integration tests.

These cover the source-installed PyO3 binding and namespace-scoped memory API.
IQL/LISP/DQL is not part of the v0.1.x MVP boundary.
"""

import os
import time
import shutil
import pytest

# The module name matches [lib] name in Cargo.toml
import vantadb_py as vanta


import glob

TEST_DB_PATH = "./test_sdk_db"


@pytest.fixture(autouse=True)
def cleanup():
    """Clean up test databases before and after each test."""
    def _clean():
        for path in glob.glob(f"{TEST_DB_PATH}_*"):
            if os.path.exists(path):
                shutil.rmtree(path, ignore_errors=True)
    _clean()
    yield
    _clean()


import uuid


def _unique_path():
    return f"{TEST_DB_PATH}_{uuid.uuid4().hex[:8]}"


def _wait_until(predicate, timeout=5.0, interval=0.05):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if predicate():
            return
        time.sleep(interval)
    raise TimeoutError(f"Timed out waiting for: {predicate.__doc__ or 'condition'}")


class TestVantaDBLifecycle:
    """Core CRUD lifecycle tests."""

    def test_open_and_repr(self):
        """VantaDB instance should open and display hardware profile."""
        db = vanta.VantaDB(_unique_path(), memory_limit_bytes=128 * 1024 * 1024)
        r = repr(db)
        assert "VantaDB(" in r, f"repr should contain 'VantaDB(', got: {r[:80]}"
        assert "profile=" in r, f"repr should contain 'profile=', got: {r[:80]}"

    def test_insert_and_get(self):
        """Insert a node and retrieve it by ID."""
        db = vanta.VantaDB(_unique_path(), memory_limit_bytes=128 * 1024 * 1024)
        db.insert(42, "Hello VantaDB", [0.1] * 384)

        node = db.get(42)
        assert node is not None, "get(42) should return a node after insert"
        assert node["id"] == 42, f"expected id 42, got {node['id']}"
        assert node["fields"]["content"] == "Hello VantaDB", f"expected content 'Hello VantaDB', got {node['fields']['content']}"
        assert node["vector_dims"] == 384, f"expected vector_dims 384, got {node['vector_dims']}"
        assert node["is_alive"] is True, f"expected is_alive True, got {node['is_alive']}"

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
        assert node is not None, "get(1) should return a node after insert with extra fields"
        assert node["fields"]["category"] == "test", f"expected category 'test', got {node['fields']['category']}"
        assert node["fields"]["score"] == 42, f"expected score 42, got {node['fields']['score']}"
        assert node["fields"]["active"] is True, f"expected active True, got {node['fields']['active']}"

    def test_get_nonexistent(self):
        """Getting a non-existent node returns None."""
        db = vanta.VantaDB(_unique_path(), memory_limit_bytes=128 * 1024 * 1024)
        assert db.get(999999) is None, "getting a non-existent node should return None"

    def test_delete_tombstone(self):
        """Deleting a node should make it unretrievable."""
        db = vanta.VantaDB(_unique_path(), memory_limit_bytes=128 * 1024 * 1024)
        db.insert(10, "To be deleted", [0.2] * 128)
        assert db.get(10) is not None, "node should exist before deletion"

        db.delete(10, "test cleanup")
        assert db.get(10) is None, "node should be None after deletion"

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
        assert node is not None, "node should survive reopen"
        assert node["fields"]["content"] == "Reopen me", f"expected content 'Reopen me', got {node['fields']['content']}"


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
        assert len(results) > 0, f"search should return at least one result, got {len(results)}"
        # Results are (node_id, distance) tuples
        assert all(isinstance(r, tuple) and len(r) == 2 for r in results), f"each result should be a 2-tuple, got {results[:3]}"

    @pytest.mark.skip(reason="search_batch not yet exposed in Python SDK")
    def test_search_batch(self):
        """Batch search should yield equivalent results to individual searches in parallel."""
        db = vanta.VantaDB(_unique_path(), memory_limit_bytes=128 * 1024 * 1024)

        # Insert some vectors
        for i in range(10):
            vec = [float(i) * 0.1] * 384
            db.insert(i + 1, f"Node {i}", vec)

        query_vectors = [
            [0.0] * 384,
            [0.5] * 384,
            [0.9] * 384,
        ]

        # Individual searches
        individual_results = []
        for q in query_vectors:
            individual_results.append(db.search(q, top_k=3))

        # Batch search
        batch_results = db.search_batch(query_vectors, top_k=3)

        assert len(batch_results) == len(query_vectors), f"expected {len(query_vectors)} batch results, got {len(batch_results)}"
        for i in range(len(query_vectors)):
            assert len(batch_results[i]) == len(individual_results[i]), f"result {i}: expected {len(individual_results[i])} results, got {len(batch_results[i])}"
            for j in range(len(batch_results[i])):
                assert batch_results[i][j][0] == individual_results[i][j][0], f"result {i},{j}: expected id {individual_results[i][j][0]}, got {batch_results[i][j][0]}"
                assert abs(batch_results[i][j][1] - individual_results[i][j][1]) < 1e-5, f"result {i},{j}: distances differ by {abs(batch_results[i][j][1] - individual_results[i][j][1])}"


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

        assert record["namespace"] == "agent/main", f"expected namespace 'agent/main', got {record['namespace']}"
        assert record["key"] == "task-1", f"expected key 'task-1', got {record['key']}"
        assert record["payload"] == "ship the memory API", f"expected payload 'ship the memory API', got {record['payload']}"
        assert record["version"] == 1, f"expected version 1, got {record['version']}"
        assert record["metadata"]["category"] == "task", f"expected category 'task', got {record['metadata']['category']}"

        fetched = db.get_memory("agent/main", "task-1")
        assert fetched is not None, "get_memory should return the stored record"
        assert fetched["node_id"] == record["node_id"], f"expected node_id {record['node_id']}, got {fetched['node_id']}"

        page = db.list_memory("agent/main", filters={"category": "task"})
        assert len(page["records"]) == 1, f"expected 1 record, got {len(page['records'])}"
        assert page["records"][0]["key"] == "task-1", f"expected key 'task-1', got {page['records'][0]['key']}"

        hits = db.search_memory(
            "agent/main",
            [1.0, 0.0, 0.0],
            filters={"category": "task"},
            top_k=3,
        )
        assert len(hits) == 1, f"expected 1 hit, got {len(hits)}"
        assert hits[0]["record"]["key"] == "task-1", f"expected key 'task-1', got {hits[0]['record']['key']}"

        text_hits = db.search_memory(
            "agent/main",
            [],
            text_query="memory API",
            top_k=3,
        )
        assert len(text_hits) == 1, f"text search expected 1 hit, got {len(text_hits)}"
        assert text_hits[0]["record"]["key"] == "task-1", f"expected key 'task-1', got {text_hits[0]['record']['key']}"

        hybrid_hits = db.search_memory(
            "agent/main",
            [1.0, 0.0, 0.0],
            text_query="memory API",
            top_k=3,
        )
        assert len(hybrid_hits) == 1, f"hybrid search expected 1 hit, got {len(hybrid_hits)}"
        assert hybrid_hits[0]["record"]["key"] == "task-1", f"expected key 'task-1', got {hybrid_hits[0]['record']['key']}"

        db.put("agent/main", "phrase-exact", "alpha beta gamma")
        db.put("agent/main", "phrase-separated", "alpha spacer beta")
        phrase_hits = db.search_memory(
            "agent/main",
            [],
            text_query='"alpha beta"',
            top_k=3,
        )
        assert [hit["record"]["key"] for hit in phrase_hits] == ["phrase-exact"], f"expected ['phrase-exact'], got {[hit['record']['key'] for hit in phrase_hits]}"

    def test_memory_close_and_reopen(self):
        """Memory records should survive flush/close/reopen."""
        path = _unique_path()
        db = vanta.VantaDB(path, memory_limit_bytes=128 * 1024 * 1024)
        db.put("agent/main", "persist", "persistent payload")
        db.flush()
        db.close()

        reopened = vanta.VantaDB(path, memory_limit_bytes=128 * 1024 * 1024)
        record = reopened.get_memory("agent/main", "persist")
        assert record is not None, "memory record should survive flush/close/reopen"
        assert record["payload"] == "persistent payload", f"expected 'persistent payload', got {record['payload']}"

    def test_delete_memory(self):
        """Deleting a memory record should make it unretrievable."""
        db = vanta.VantaDB(_unique_path(), memory_limit_bytes=128 * 1024 * 1024)
        db.put("agent/main", "delete-me", "temporary")
        assert db.delete_memory("agent/main", "delete-me") is True, "delete_memory should return True"
        assert db.get_memory("agent/main", "delete-me") is None, "deleted memory should not be retrievable"

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
        assert rebuild["success"] is True, f"rebuild_index should succeed, got {rebuild}"
        assert rebuild["scanned_nodes"] >= 1, f"expected scanned_nodes >= 1, got {rebuild['scanned_nodes']}"
        audit = db.audit_text_index("agent/main")
        assert audit["passed"] is True, f"audit should pass, got {audit}"
        assert audit["status"] == "ok", f"expected status 'ok', got {audit['status']}"
        assert audit["namespace_filter"] == "agent/main", f"expected namespace_filter 'agent/main', got {audit['namespace_filter']}"
        assert audit["expected_entries"] > 0, f"expected_entries should be > 0, got {audit['expected_entries']}"

        exported = db.export_namespace(export_path, "agent/main")
        assert exported["records_exported"] == 1, f"expected 1 record exported, got {exported}"
        assert os.path.exists(export_path), f"export file should exist at {export_path}"

        target = vanta.VantaDB(target_path, memory_limit_bytes=128 * 1024 * 1024)
        imported = target.import_file(export_path)
        assert imported["inserted"] == 1, f"expected 1 inserted, got {imported}"
        assert imported["errors"] == 0, f"expected 0 errors, got {imported['errors']}"

        fetched = target.get_memory("agent/main", "export-me")
        assert fetched is not None, "imported record should be retrievable"
        assert fetched["payload"] == "portable memory", f"expected 'portable memory', got {fetched['payload']}"

        all_export_path = str(tmp_path / "all.jsonl")
        all_export = target.export_all(all_export_path)
        assert all_export["records_exported"] == 1, f"expected 1, got {all_export['records_exported']}"

    def test_operational_metrics(self, tmp_path):
        """Operational metrics should be available through the Python SDK."""
        export_path = str(tmp_path / "metrics.jsonl")
        db = vanta.VantaDB(str(tmp_path / "metrics-db"), memory_limit_bytes=128 * 1024 * 1024)
        before = db.operational_metrics()

        db.put("agent/main", "metric", "payload", vector=[1.0, 0.0, 0.0])
        rebuild = db.rebuild_index()
        db.export_all(export_path)
        after = db.operational_metrics()

        expected_keys = ["startup_ms", "wal_records_replayed", "derived_rebuild_ms", "text_index_rebuild_ms",
                         "text_postings_written", "text_index_repairs", "text_lexical_queries", "text_lexical_query_ms",
                         "text_candidates_scored", "text_consistency_audits", "text_consistency_audit_failures",
                         "hybrid_query_ms", "hybrid_candidates_fused", "planner_hybrid_queries",
                         "planner_text_only_queries", "planner_vector_only_queries", "records_exported"]
        for key in expected_keys:
            assert key in after, f"operational_metrics should contain '{key}'"
        assert after["ann_rebuild_scanned_nodes"] >= rebuild["scanned_nodes"], f"ann_rebuild_scanned_nodes ({after['ann_rebuild_scanned_nodes']}) should be >= rebuild scanned ({rebuild['scanned_nodes']})"
        assert after["records_exported"] >= before["records_exported"] + 1, f"records_exported should have increased (before={before['records_exported']}, after={after['records_exported']})"


class TestHardwareIntrospection:
    """Stable capability surface tests."""

    def test_capabilities(self):
        """Capabilities should return the stable SDK-facing keys."""
        db = vanta.VantaDB(_unique_path(), memory_limit_bytes=128 * 1024 * 1024)
        caps = db.capabilities()

        assert "profile" in caps, f"capabilities should contain 'profile', got {list(caps.keys())}"
        assert "read_only" in caps, f"capabilities should contain 'read_only', got {list(caps.keys())}"
        assert "persistence" in caps, f"capabilities should contain 'persistence', got {list(caps.keys())}"
        assert "vector_search" in caps, f"capabilities should contain 'vector_search', got {list(caps.keys())}"
        assert "iql_queries" in caps, f"capabilities should contain 'iql_queries', got {list(caps.keys())}"
        assert caps["persistence"] is True, f"expected persistence=True, got {caps['persistence']}"

    @pytest.mark.skip(reason="process_rss_bytes field not yet available in hardware_profile")
    def test_hardware_profile_alias(self):
        """hardware_profile remains as a backward-compatible alias with memory telemetry."""
        db = vanta.VantaDB(_unique_path(), memory_limit_bytes=128 * 1024 * 1024)
        hw = db.hardware_profile()
        expected_hw_keys = ["profile", "vector_search", "process_rss_bytes", "hnsw_logical_bytes", "hnsw_nodes_count"]
        for key in expected_hw_keys:
            assert key in hw, f"hardware_profile should contain '{key}'"
        assert hw["process_rss_bytes"] > 0, f"expected process_rss_bytes > 0, got {hw['process_rss_bytes']}"


class TestEdgeManagement:
    """Graph edge tests."""

    def test_add_edge(self):
        """Adding an edge between two nodes."""
        db = vanta.VantaDB(_unique_path(), memory_limit_bytes=128 * 1024 * 1024)
        db.insert(1, "Source", [])
        db.insert(2, "Target", [])

        db.add_edge(1, 2, "relates_to", weight=0.95)

        node = db.get(1)
        assert node is not None, "node should exist after insert"
        assert len(node["edges"]) > 0, f"expected at least 1 edge, got {len(node['edges'])}"
        edge = node["edges"][0]
        assert edge[0] == 2, f"expected edge target 2, got {edge[0]}"
        assert edge[1] == "relates_to", f"expected edge label 'relates_to', got {edge[1]}"


class TestNumPyIntegration:
    """Zero-copy NumPy array support."""

    def test_insert_with_numpy_vector(self):
        """Insert with numpy array should work identically to list."""
        import numpy as np
        db = vanta.VantaDB(_unique_path(), memory_limit_bytes=128 * 1024 * 1024)
        vec = np.ones(384, dtype=np.float32)
        db.insert(1, "numpy test", vec)
        node = db.get(1)
        assert node is not None, "node should exist after insert with numpy vector"
        assert node["vector_dims"] == 384, f"expected vector_dims 384, got {node['vector_dims']}"

    def test_search_with_numpy_vector(self):
        """Search with numpy array should return results."""
        import numpy as np
        db = vanta.VantaDB(_unique_path(), memory_limit_bytes=128 * 1024 * 1024)
        for i in range(5):
            db.insert(i + 1, f"Node {i}", np.full(384, float(i) * 0.1, dtype=np.float32))
        results = db.search(np.zeros(384, dtype=np.float32), top_k=3)
        assert len(results) > 0, f"search with numpy vector expected results, got {len(results)}"
        assert all(isinstance(r, tuple) and len(r) == 2 for r in results), f"each result should be a 2-tuple, got {results[:3]}"

    def test_memory_put_with_numpy_vector(self):
        """Memory put with numpy array should work."""
        import numpy as np
        db = vanta.VantaDB(_unique_path(), memory_limit_bytes=128 * 1024 * 1024)
        record = db.put("ns", "k", "payload", vector=np.array([1.0, 0.0, 0.0], dtype=np.float32))
        assert record["namespace"] == "ns", f"expected namespace 'ns', got {record['namespace']}"
        assert record["key"] == "k", f"expected key 'k', got {record['key']}"

    def test_put_batch_parallel(self):
        """put_batch should insert multiple records in parallel and return them in order."""
        db = vanta.VantaDB(_unique_path(), memory_limit_bytes=128 * 1024 * 1024)
        entries = [
            ("ns1", "a", "alpha", None, None),
            ("ns1", "b", "beta", {"type": "greek"}, None),
            ("ns2", "c", "gamma", None, [1.0, 0.0, 0.0]),
            ("ns1", "d", "delta", {"rank": "4"}, [0.0, 1.0, 0.0]),
        ]
        records = db.put_batch(entries)
        assert len(records) == 4, f"expected 4 records, got {len(records)}"

        assert records[0]["namespace"] == "ns1", f"expected 'ns1', got {records[0]['namespace']}"
        assert records[0]["key"] == "a", f"expected 'a', got {records[0]['key']}"
        assert records[0]["payload"] == "alpha", f"expected 'alpha', got {records[0]['payload']}"
        assert records[0]["version"] == 1, f"expected version 1, got {records[0]['version']}"

        assert records[1]["metadata"]["type"] == "greek", f"expected 'greek', got {records[1]['metadata']['type']}"
        assert len(records[2]["vector"]) == 3, f"expected vector of length 3, got {len(records[2]['vector'])}"
        assert records[3]["metadata"]["rank"] == "4", f"expected '4', got {records[3]['metadata']['rank']}"

        # verify persisted
        fetched = db.get_memory("ns1", "d")
        assert fetched["payload"] == "delta", f"expected 'delta', got {fetched['payload']}"

    def test_put_batch_empty(self):
        """put_batch with empty list should return empty list."""
        db = vanta.VantaDB(_unique_path(), memory_limit_bytes=128 * 1024 * 1024)
        records = db.put_batch([])
        assert records == [], f"expected empty list, got {records}"

    def test_put_batch_numpy_vectors(self):
        """put_batch should accept numpy arrays as vectors."""
        import numpy as np
        db = vanta.VantaDB(_unique_path(), memory_limit_bytes=128 * 1024 * 1024)
        vec = np.array([1.0, 0.0, 0.0], dtype=np.float32)
        records = db.put_batch([
            ("ns", "x", "numpy entry", None, vec),
        ])
        assert len(records) == 1, f"expected 1 record, got {len(records)}"
        assert records[0]["key"] == "x", f"expected key 'x', got {records[0]['key']}"

    def test_memory_search_with_numpy_vector(self):
        """Memory search with numpy array should work."""
        import numpy as np
        db = vanta.VantaDB(_unique_path(), memory_limit_bytes=128 * 1024 * 1024)
        db.put("ns", "k1", "hello", vector=np.array([1.0, 0.0, 0.0], dtype=np.float32))
        hits = db.search_memory("ns", np.array([1.0, 0.0, 0.0], dtype=np.float32), top_k=3)
        assert len(hits) == 1, f"expected 1 hit, got {len(hits)}"
        assert hits[0]["record"]["key"] == "k1", f"expected 'k1', got {hits[0]['record']['key']}"

    def test_numpy_f64_auto_downcast(self):
        """f64 numpy arrays should auto-downcast to f32."""
        import numpy as np
        db = vanta.VantaDB(_unique_path(), memory_limit_bytes=128 * 1024 * 1024)
        vec_f64 = np.ones(128, dtype=np.float64)
        db.insert(1, "f64 test", vec_f64)
        node = db.get(1)
        assert node is not None, "node should exist after insert with f64 vector"
        assert node["vector_dims"] == 128, f"expected vector_dims 128, got {node['vector_dims']}"

    def test_list_fallback_still_works(self):
        """Regular Python lists should still work after buffer protocol changes."""
        db = vanta.VantaDB(_unique_path(), memory_limit_bytes=128 * 1024 * 1024)
        db.insert(1, "list test", [0.5] * 128)
        node = db.get(1)
        assert node is not None, "node should exist after insert with list vector"
        assert node["vector_dims"] == 128, f"expected vector_dims 128, got {node['vector_dims']}"


class TestMemoryBoundary:
    """Memory budget isolation tests."""

    def test_explicit_memory_limit(self):
        """DB should respect explicit memory limit via constructor."""
        # 64MB — this should activate resource governance
        db = vanta.VantaDB(_unique_path(), memory_limit_bytes=64 * 1024 * 1024)

        hw = db.hardware_profile()
        # The engine should have initialized without crashing
        assert hw is not None, "hardware_profile should not be None after init with memory limit"


class TestAsyncVantaDB:
    """Async wrapper for query methods."""

    def test_async_basic_crud(self):
        """AsyncVantaDB should support put/get_memory/search_memory."""
        import asyncio

        async def run():
            async with vanta.AsyncVantaDB(
                _unique_path(), memory_limit_bytes=128 * 1024 * 1024
            ) as db:
                await db.put("ns", "k", "hello", metadata={"tag": "test"})
                record = await db.get_memory("ns", "k")
                assert record is not None, "get_memory should return a record after put"
                assert record["payload"] == "hello", f"expected 'hello', got {record['payload']}"
                assert record["metadata"]["tag"] == "test", f"expected tag 'test', got {record['metadata']['tag']}"

                results = await db.search_memory("ns", [1.0, 0.0, 0.0], top_k=5)
                assert isinstance(results, list), f"search_memory should return a list, got {type(results)}"

        asyncio.run(run())

    def test_async_list_memory(self):
        """AsyncVantaDB.list_memory should work."""
        import asyncio

        async def run():
            async with vanta.AsyncVantaDB(
                _unique_path(), memory_limit_bytes=128 * 1024 * 1024
            ) as db:
                await db.put("ns", "a", "alpha")
                await db.put("ns", "b", "beta")
                page = await db.list_memory("ns")
                assert len(page["records"]) == 2, f"expected 2 records, got {len(page['records'])}"

        asyncio.run(run())

    def test_async_delete_and_flush(self):
        """AsyncVantaDB.delete_memory and flush should work."""
        import asyncio

        async def run():
            async with vanta.AsyncVantaDB(
                _unique_path(), memory_limit_bytes=128 * 1024 * 1024
            ) as db:
                await db.put("ns", "x", "to-delete")
                deleted = await db.delete_memory("ns", "x")
                assert deleted is True, "delete_memory should return True"
                await db.flush()

        asyncio.run(run())


class TestWALCompaction:
    """TSK-75: WAL compaction / rotate."""

    def test_compact_wal(self):
        """compact_wal should flush and rotate WAL without data loss."""
        path = _unique_path()
        db = vanta.VantaDB(path, memory_limit_bytes=128 * 1024 * 1024)
        db.put("ns", "a", "alpha")
        db.put("ns", "b", "beta")
        db.compact_wal()

        # data still readable after compaction
        assert db.get_memory("ns", "a")["payload"] == "alpha", "compact_wal should preserve 'alpha'"
        assert db.get_memory("ns", "b")["payload"] == "beta", "compact_wal should preserve 'beta'"
        db.close()

        # reopen — data from rotated WAL still intact
        db2 = vanta.VantaDB(path, memory_limit_bytes=128 * 1024 * 1024)
        assert db2.get_memory("ns", "a")["payload"] == "alpha", "data should survive reopen after wal compaction"
        db2.close()

    def test_compact_wal_read_only_raises(self):
        """compact_wal in read-only mode should raise."""
        path = _unique_path()
        db = vanta.VantaDB(path, memory_limit_bytes=128 * 1024 * 1024)
        db.close()
        ro = vanta.VantaDB(path, read_only=True, memory_limit_bytes=128 * 1024 * 1024)
        import pytest
        with pytest.raises(Exception):
            ro.compact_wal()
        ro.close()


class TestTTL:
    """TSK-76: Time-To-Live on memory records."""

    def test_put_with_ttl(self):
        """put() with ttl_ms should store expires_at_ms on the record."""
        db = vanta.VantaDB(_unique_path(), memory_limit_bytes=128 * 1024 * 1024)
        record = db.put("ns", "k", "hello", ttl_ms=86_400_000)  # 1 day
        assert record["key"] == "k", f"expected key 'k', got {record['key']}"
        assert record["expires_at_ms"] is not None, "expires_at_ms should be set when ttl_ms is provided"
        assert record["expires_at_ms"] > 0, f"expected expires_at_ms > 0, got {record['expires_at_ms']}"

    def test_put_without_ttl(self):
        """put() without ttl_ms should have expires_at_ms = None."""
        db = vanta.VantaDB(_unique_path(), memory_limit_bytes=128 * 1024 * 1024)
        record = db.put("ns", "k", "hello")
        assert record["expires_at_ms"] is None, "expires_at_ms should be None when ttl_ms is not provided"

    def test_lazy_eviction(self):
        """Records with past TTL should be invisible on read."""
        db = vanta.VantaDB(_unique_path(), memory_limit_bytes=128 * 1024 * 1024)
        # ttl_ms=1 means expires in 1ms — by the time we read, it's gone
        record = db.put("ns", "k", "gone", ttl_ms=1)
        _wait_until(lambda: db.get_memory("ns", "k") is None, timeout=10.0)
        assert db.get_memory("ns", "k") is None, "expired record should not be retrievable"
        # list should also exclude it
        page = db.list_memory("ns")
        assert len(page["records"]) == 0, f"expected 0 records in list, got {len(page['records'])}"

    def test_purge_expired(self):
        """purge_expired should physically remove expired records."""
        db = vanta.VantaDB(_unique_path(), memory_limit_bytes=128 * 1024 * 1024)
        db.put("ns", "keep", "alive")
        db.put("ns", "gone", "dead", ttl_ms=1)
        _wait_until(lambda: db.get_memory("ns", "gone") is None, timeout=10.0)
        purged = db.purge_expired()
        assert purged >= 1, f"expected at least 1 purge, got {purged}"
        # keep is still there
        assert db.get_memory("ns", "keep")["payload"] == "alive", "non-expired records should survive purge"
        # gone is gone
        assert db.get_memory("ns", "gone") is None, "expired records should be removed after purge"


if __name__ == "__main__":
    pytest.main([__file__, "-v"])

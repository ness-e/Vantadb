"""Load and stress tests for VantaDB Python SDK.

These tests verify the engine handles concurrent operations, large batches,
memory pressure, and sustained throughput without crashing or degrading.
"""

import os
import gc
import time
import uuid
import shutil
import threading
import glob

import pytest

import vantadb_py as vanta

TEST_DB_PATH = "./test_load_db"


@pytest.fixture(autouse=True)
def cleanup():
    def _clean():
        for path in glob.glob(f"{TEST_DB_PATH}_*"):
            if os.path.exists(path):
                shutil.rmtree(path, ignore_errors=True)
    _clean()
    yield
    _clean()


def _unique_path():
    return f"{TEST_DB_PATH}_{uuid.uuid4().hex[:8]}"


class TestConcurrentOperations:
    """Concurrent insert/search stress tests."""

    def test_concurrent_inserts_4_threads(self):
        """Insert 1000 vectors from 4 threads concurrently."""
        db = vanta.VantaDB(_unique_path(), memory_limit_bytes=256 * 1024 * 1024)

        errors = []
        lock = threading.Lock()

        def insert_range(start, count):
            try:
                for i in range(start, start + count):
                    db.insert(i, content=f"vector_{i}", vector=[float(i % 10)] * 128)
            except Exception as e:
                with lock:
                    errors.append(e)

        threads = []
        for t in range(4):
            thread = threading.Thread(target=insert_range, args=(t * 250, 250))
            threads.append(thread)
            thread.start()

        for t in threads:
            t.join()

        assert not errors, f"Thread errors: {errors}"
        results = db.search(vector=[0.0] * 128, top_k=10)
        assert len(results) > 0

    def test_concurrent_inserts_8_threads(self):
        """Insert 2000 vectors from 8 threads concurrently."""
        db = vanta.VantaDB(_unique_path(), memory_limit_bytes=256 * 1024 * 1024)

        errors = []
        lock = threading.Lock()

        def insert_range(start, count):
            try:
                for i in range(start, start + count):
                    db.insert(i, content=f"stress_{i}", vector=[float(i % 5)] * 64)
            except Exception as e:
                with lock:
                    errors.append(e)

        threads = []
        for t in range(8):
            thread = threading.Thread(target=insert_range, args=(t * 250, 250))
            threads.append(thread)
            thread.start()

        for t in threads:
            t.join()

        assert not errors, f"Thread errors: {errors}"
        results = db.search(vector=[0.0] * 64, top_k=10)
        assert len(results) > 0

    def test_concurrent_search_during_inserts(self):
        """Search while inserts are happening in background threads."""
        db = vanta.VantaDB(_unique_path(), memory_limit_bytes=256 * 1024 * 1024)

        for i in range(500):
            db.insert(i, content=f"base_{i}", vector=[float(i % 10)] * 64)

        errors = []
        lock = threading.Lock()
        stop = threading.Event()

        def inserter():
            try:
                i = 0
                while not stop.is_set():
                    db.insert(1000 + i, content=f"live_{i}", vector=[float(i % 10)] * 64)
                    i += 1
            except Exception as e:
                with lock:
                    errors.append(e)

        def searcher():
            try:
                while not stop.is_set():
                    db.search(vector=[0.5] * 64, top_k=5)
            except Exception as e:
                with lock:
                    errors.append(e)

        threads = [threading.Thread(target=inserter) for _ in range(2)]
        threads += [threading.Thread(target=searcher) for _ in range(2)]
        for t in threads:
            t.start()

        time.sleep(1.0)
        stop.set()
        for t in threads:
            t.join(timeout=3)

        assert not errors, f"Thread errors: {errors}"
        results = db.search(vector=[0.5] * 64, top_k=10)
        assert len(results) > 0


class TestLargeBatchOperations:
    """Large dataset insert and search stress tests."""

    def test_large_batch_insert_10k(self):
        """Insert 10000 vectors and verify search still works."""
        db = vanta.VantaDB(_unique_path(), memory_limit_bytes=512 * 1024 * 1024)

        for i in range(10000):
            db.insert(i, content=f"batch_{i}", vector=[float(i % 256) / 256.0] * 64)

        results = db.search(vector=[0.5] * 64, top_k=10)
        assert len(results) == 10

    def test_large_batch_insert_25k(self):
        """Insert 25000 vectors (larger scale)."""
        db = vanta.VantaDB(_unique_path(), memory_limit_bytes=1024 * 1024 * 1024)

        for i in range(25000):
            db.insert(i, content=f"batch_{i}", vector=[float(i % 256) / 256.0] * 32)

        results = db.search(vector=[0.5] * 32, top_k=10)
        assert len(results) == 10

    def test_repeated_insert_and_delete_cycle(self):
        """Insert then delete in a cycle to stress WAL."""
        db = vanta.VantaDB(_unique_path(), memory_limit_bytes=256 * 1024 * 1024)

        for cycle in range(5):
            for i in range(500):
                db.insert(cycle * 500 + i, content=f"cycle_{cycle}_{i}", vector=[float(i % 10)] * 64)
            for i in range(500):
                db.delete(cycle * 500 + i)
            gc.collect()

        results = db.search(vector=[0.5] * 64, top_k=5)
        assert len(results) == 0


class TestMemoryPressure:
    """Memory pressure and large vector stress tests."""

    def test_large_vectors_repeated(self):
        """Insert large vectors (512 dims) repeatedly to test memory."""
        db = vanta.VantaDB(_unique_path(), memory_limit_bytes=256 * 1024 * 1024)
        for i in range(1000):
            db.insert(i, content=f"large_{i}", vector=[float(i)] * 512)
        gc.collect()
        results = db.search(vector=[0.0] * 512, top_k=5)
        assert len(results) > 0

    def test_high_dimensional_vectors(self):
        """Insert vectors with 1536 dimensions (OpenAI ada-002 scale)."""
        db = vanta.VantaDB(_unique_path(), memory_limit_bytes=256 * 1024 * 1024)
        for i in range(200):
            db.insert(i, content=f"highdim_{i}", vector=[float(i % 100) / 100.0] * 1536)
        gc.collect()
        results = db.search(vector=[0.5] * 1536, top_k=5)
        assert len(results) > 0


class TestSustainedThroughput:
    """Throughput and endurance tests."""

    def test_sustained_inserts(self):
        """Sustained inserts over many iterations."""
        db = vanta.VantaDB(_unique_path(), memory_limit_bytes=256 * 1024 * 1024)

        start = time.time()
        count = 5000
        for i in range(count):
            db.insert(i, content=f"perf_{i}", vector=[float(i % 10)] * 64)

        elapsed = time.time() - start
        ops_per_sec = count / elapsed

        assert ops_per_sec > 100, f"Insert throughput too low: {ops_per_sec:.0f} ops/s"
        results = db.search(vector=[0.5] * 64, top_k=10)
        assert len(results) == 10

    def test_sustained_search(self):
        """Sustained search throughput."""
        db = vanta.VantaDB(_unique_path(), memory_limit_bytes=256 * 1024 * 1024)

        for i in range(1000):
            db.insert(i, content=f"search_{i}", vector=[float(i % 10)] * 64)

        start = time.time()
        iterations = 500
        for _ in range(iterations):
            db.search(vector=[0.5] * 64, top_k=10)

        elapsed = time.time() - start
        ops_per_sec = iterations / elapsed

        assert ops_per_sec > 200, f"Search throughput too low: {ops_per_sec:.0f} ops/s"

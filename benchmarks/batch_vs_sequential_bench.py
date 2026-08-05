#!/usr/bin/env python3
"""
VantaDB FFI Batch vs Sequential Performance Benchmark
Compares db.search_batch() with sequential db.search() to demonstrate FFI amortization and multi-core Rayon speedup.
"""

import time
import random
import os
import shutil
import math

try:
    import vantadb_py as vantadb
except ImportError:
    print("ERROR: 'vantadb_py' is not installed.")
    print("Install it from PyPI (standalone, no Rust build required):")
    print("  pip install vantadb-py")
    print("Full benchmark dependencies: pip install -r benchmarks/requirements.txt")
    print("Or for local development against the source tree:")
    print("  maturin develop --manifest-path vantadb-python/Cargo.toml --release")
    exit(1)

def generate_unit_vector(dim):
    vec = [random.uniform(-1.0, 1.0) for _ in range(dim)]
    norm = math.sqrt(sum(x * x for x in vec))
    if norm > 0:
        return [x / norm for x in vec]
    return vec

def run_bench(db_path="./benchmarks/batch_bench_db", num_vectors=5000, dim=128, batch_size=100, top_k=10):
    if os.path.exists(db_path):
        shutil.rmtree(db_path)

    print("Initializing Database...")
    db = vantadb.VantaDB(db_path)

    print(f"Generating and inserting {num_vectors} vectors...")
    for i in range(num_vectors):
        vec = generate_unit_vector(dim)
        db.insert(i + 1, f"Node {i}", vec)
    db.flush()

    print("Generating query batch...")
    queries = [generate_unit_vector(dim) for _ in range(batch_size)]

    print("\nRunning Warmup...")
    for q in queries[:10]:
        db.search(q, top_k=top_k)
    db.search_batch(queries[:10], top_k=top_k)

    print("\n--- Running Sequential Search ---")
    start_seq = time.perf_counter()
    seq_results = []
    for q in queries:
        seq_results.append(db.search(q, top_k=top_k))
    duration_seq = (time.perf_counter() - start_seq) * 1000.0  # ms
    avg_seq = duration_seq / batch_size

    print("\n--- Running Batch Search (Rayon + Eager GIL Release) ---")
    start_batch = time.perf_counter()
    batch_results = db.search_batch(queries, top_k=top_k)
    duration_batch = (time.perf_counter() - start_batch) * 1000.0  # ms
    avg_batch = duration_batch / batch_size

    # Validate output parity
    assert len(seq_results) == len(batch_results)
    for i in range(batch_size):
        assert len(seq_results[i]) == len(batch_results[i])
        if len(seq_results[i]) > 0:
            assert seq_results[i][0][0] == batch_results[i][0][0]

    speedup = duration_seq / duration_batch
    reduction = (1.0 - (duration_batch / duration_seq)) * 100.0

    print("\n==================================================")
    print("           Batch vs Sequential Results            ")
    print("==================================================")
    print(f"Batch Size (Queries): {batch_size}")
    print(f"Total Sequential Time: {duration_seq:.2f} ms (avg {avg_seq:.4f} ms/query)")
    print(f"Total Batch Time:      {duration_batch:.2f} ms (avg {avg_batch:.4f} ms/query)")
    print(f"Speedup Factor:        {speedup:.2f}x faster")
    print(f"Latency Reduction:     {reduction:.2f}%")
    print("==================================================")

    db.close()
    if os.path.exists(db_path):
        shutil.rmtree(db_path)


BATCH_REQUESTS_TARGET = 3.0
"""INV-008-B target: a batch of 10 full SearchRequest queries must run in
less than 3x the time of a single sequential search_memory query."""


def run_batch_requests_bench(
    db_path="./benchmarks/batch_requests_bench_db",
    num_records=2000,
    dim=128,
    batch_size=10,
    top_k=10,
):
    """Benchmark search_batch_requests (full SearchRequest: text + vector +
    filters) vs sequential search_memory, and verify the INV-008-B target:
    batch of 10 < 3x single-query time."""
    if os.path.exists(db_path):
        shutil.rmtree(db_path)

    print("\n=== INV-008-B: search_batch_requests (full SearchRequest) ===")
    print("Initializing Database...")
    db = vantadb.VantaDB(db_path)

    print(f"Inserting {num_records} memory records (vector + payload)...")
    for i in range(num_records):
        vec = generate_unit_vector(dim)
        db.put(
            "bench",
            f"rec-{i}",
            f"the quick brown fox jumps over the lazy dog {i}",
            metadata={"group": f"g{i % 5}", "size": i % 100},
            vector=vec,
        )
    db.flush()

    queries = [generate_unit_vector(dim) for _ in range(batch_size)]
    requests = [
        vantadb.SearchRequest(
            namespace="bench",
            query_vector=q,
            text_query="quick brown fox",
            filters={"group": f"g{i % 5}"},
            top_k=top_k,
        )
        for i, q in enumerate(queries)
    ]

    # Warmup (also exercises the dict path once)
    db.search_memory("bench", queries[0], text_query="quick brown fox", top_k=top_k)
    db.search_batch_requests(requests[: min(3, batch_size)], top_k=top_k)
    db.search_batch_requests([r.asdict() for r in requests[: min(3, batch_size)]], top_k=top_k)

    print(f"\n--- Sequential search_memory ({batch_size} queries) ---")
    start_seq = time.perf_counter()
    for i, q in enumerate(queries):
        db.search_memory(
            "bench",
            q,
            text_query="quick brown fox",
            filters={"group": f"g{i % 5}"},
            top_k=top_k,
        )
    duration_seq = (time.perf_counter() - start_seq) * 1000.0
    single_ms = duration_seq / batch_size

    print(f"--- Batch search_batch_requests ({batch_size} queries, Rayon + GIL release) ---")
    start_batch = time.perf_counter()
    batch_results = db.search_batch_requests(requests, top_k=top_k)
    duration_batch = (time.perf_counter() - start_batch) * 1000.0

    # Validate output parity with sequential results
    seq_results = []
    for i, q in enumerate(queries):
        seq_results.append(
            db.search_memory(
                "bench",
                q,
                text_query="quick brown fox",
                filters={"group": f"g{i % 5}"},
                top_k=top_k,
            )
        )
    assert len(batch_results) == len(seq_results)
    for i in range(batch_size):
        assert len(batch_results[i]) == len(seq_results[i])
        if len(batch_results[i]) > 0:
            assert batch_results[i][0].key == seq_results[i][0].key

    target_ok = duration_batch < BATCH_REQUESTS_TARGET * single_ms
    speedup = duration_seq / duration_batch

    print("\n==================================================")
    print("        Batch Requests vs Sequential Results     ")
    print("==================================================")
    print(f"Batch Size (Queries): {batch_size}")
    print(f"Sequential Total:     {duration_seq:.2f} ms (avg {single_ms:.4f} ms/query)")
    print(f"Batch Total:          {duration_batch:.2f} ms")
    print(f"Speedup vs Sequential:{speedup:.2f}x")
    print(f"Target (batch < {BATCH_REQUESTS_TARGET}x single): "
          f"{duration_batch:.2f} < {BATCH_REQUESTS_TARGET * single_ms:.2f} ms -> "
          f"{'PASS' if target_ok else 'FAIL'}")
    print("==================================================")

    db.close()
    if os.path.exists(db_path):
        shutil.rmtree(db_path)
    return target_ok


if __name__ == "__main__":
    run_bench()
    ok = run_batch_requests_bench()
    # Exit non-zero when the documented performance target is violated, so
    # CI and scheduled runs surface regressions.
    sys_exit = 0 if ok else 1
    if sys_exit:
        print(f"FAIL: batch of 10 exceeded {BATCH_REQUESTS_TARGET}x single-query time")
    raise SystemExit(sys_exit)

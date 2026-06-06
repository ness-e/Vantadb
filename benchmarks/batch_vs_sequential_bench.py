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
    print("ERROR: 'vantadb_py' is not installed. Run 'maturin develop' in 'vantadb-python' first.")
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

if __name__ == "__main__":
    run_bench()

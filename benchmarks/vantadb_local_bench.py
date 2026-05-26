#!/usr/bin/env python3
"""
VantaDB Local Performance Benchmark Suite (BENCH-01)
Measures persistent memory ingestion (PUT) and query latencies (BM25, HNSW, Hybrid Fusion).
Strictly zero-dependency, aligned with Rust's benchmark_internal.rs.
"""

import argparse
import json
import math
import os
import random
import shutil
import time

try:
    import vantadb_py as vantadb
except ImportError:
    print("ERROR: 'vantadb_py' Python package is not installed.")
    print("Please build and install it first using:")
    print("  maturin develop --manifest-path vantadb-python/Cargo.toml --release")
    exit(1)


def generate_unit_vector(dim):
    """Generates a random vector normalized to unit length (L2 norm = 1.0)."""
    vec = [random.uniform(-1.0, 1.0) for _ in range(dim)]
    norm = math.sqrt(sum(x * x for x in vec))
    if norm > 0:
        return [x / norm for x in vec]
    return vec


def calculate_percentiles(latencies):
    """Computes p50, p95, and p99 latencies in milliseconds."""
    if not latencies:
        return 0.0, 0.0, 0.0
    sorted_lats = sorted(latencies)
    n = len(sorted_lats)
    p50 = sorted_lats[int(n * 0.50)]
    p95 = sorted_lats[int(n * 0.95)]
    p99 = sorted_lats[int(n * 0.99)]
    return p50, p95, p99


def run_benchmark(db_path, num_vectors, dim, top_k, num_queries, output_file):
    print("==================================================")
    print("      VantaDB Local Benchmark Suite (BENCH-01)    ")
    print("==================================================")
    print(f"Dataset Size : {num_vectors} vectors")
    print(f"Dimension    : {dim}")
    print(f"Top-K        : {top_k}")
    print(f"Queries      : {num_queries}")
    print(f"Database Path: {db_path}")
    print("--------------------------------------------------")

    # Limpieza para asegurar mediciones limpias sin contaminación de locks o datos previos
    if os.path.exists(db_path):
        print(f"Cleaning existing database directory: {db_path}")
        shutil.rmtree(db_path)

    # 1. Inicialización
    print("Initializing VantaDB...")
    start_init = time.perf_counter()
    db = vantadb.VantaDB(db_path)
    init_duration = time.perf_counter() - start_init
    print(f"VantaDB initialized in {init_duration:.4f} seconds.")

    # Generación de dataset sintético en memoria
    print("Generating synthetic dataset in memory...")
    vectors = [generate_unit_vector(dim) for _ in range(num_vectors)]

    # 2. Ingestión (Fase PUT)
    print(f"\n[1/5] Ingesting {num_vectors} memory records via PUT...")
    start_ingest = time.perf_counter()
    
    namespace = "bench/main"
    for i, vec in enumerate(vectors):
        # API MVP: namespace, key, payload, metadata, vector
        db.put(
            namespace=namespace,
            key=f"doc-{i:05d}",
            payload=f"synthetic memory record with token_{i % 100} category_{i % 10} and keyword_{i % 500}",
            metadata={"category": "benchmark", "index": i},
            vector=vec
        )
        
        # Reportar progreso cada 10% para la ingesta (que tarda segundos/minutos)
        step = num_vectors // 10
        if step == 0:
            step = 1
        if (i + 1) % step == 0 or (i + 1) == num_vectors:
            elapsed = time.perf_counter() - start_ingest
            speed = (i + 1) / elapsed if elapsed > 0 else 0
            eta = (num_vectors - (i + 1)) / speed if speed > 0 else 0
            percent = ((i + 1) / num_vectors) * 100
            print(f"  [{percent:3.0f}%] Ingested {i + 1}/{num_vectors} | Elapsed: {elapsed:.1f}s | Speed: {speed:.1f} vec/s | ETA: {eta:.1f}s")

    print("Flushing WAL & transactions to disk...")
    db.flush()
    ingest_duration = time.perf_counter() - start_ingest
    ingest_throughput = num_vectors / ingest_duration
    print(f"Ingestion Completed in {ingest_duration:.4f}s ({ingest_throughput:.2f} records/sec)")

    # 3. Reconstrucción de Índices (Fase Rebuild)
    print(f"\n[2/5] Building hybrid indexes (ANN + Lexical BM25)...")
    start_rebuild = time.perf_counter()
    rebuild_report = db.rebuild_index()
    rebuild_duration = (time.perf_counter() - start_rebuild) * 1000.0  # ms
    
    # Validación explícita de rebuild_report
    if not rebuild_report.get('success', False):
        print(f"ERROR: Index rebuild failed: {rebuild_report}")
        exit(1)
        
    print(f"Index Rebuild Completed in {rebuild_duration:.2f} ms")

    # Generación de consultas vectoriales
    query_vectors = [generate_unit_vector(dim) for _ in range(num_queries)]

    # 4. Fase de Búsqueda Lexical (BM25)
    print(f"\n[3/5] Running {num_queries} Lexical BM25 queries (no I/O overhead)...")
    lexical_latencies = []
    for i in range(num_queries):
        text_q = f"token_{i % 100} keyword_{i % 500}"
        start_query = time.perf_counter()
        
        # Búsqueda lexical pura (vector vacío, pasamos text_query)
        db.search_memory(
            namespace=namespace,
            query_vector=[],
            text_query=text_q,
            top_k=top_k
        )
        
        duration = (time.perf_counter() - start_query) * 1000.0
        lexical_latencies.append(duration)

    # 5. Fase de Búsqueda Vectorial (HNSW)
    print(f"[4/5] Running {num_queries} Vector-only HNSW queries (no I/O overhead)...")
    vector_latencies = []
    for i, q_vec in enumerate(query_vectors):
        start_query = time.perf_counter()
        
        # Búsqueda vectorial pura (query_vector, sin text_query)
        db.search_memory(
            namespace=namespace,
            query_vector=q_vec,
            top_k=top_k
        )
        
        duration = (time.perf_counter() - start_query) * 1000.0
        vector_latencies.append(duration)

    # 6. Fase de Búsqueda Híbrida (Fusion RRF)
    print(f"[5/5] Running {num_queries} Hybrid Fusion (BM25 + HNSW) queries (no I/O overhead)...")
    hybrid_latencies = []
    for i, q_vec in enumerate(query_vectors):
        text_q = f"token_{i % 100} keyword_{i % 500}"
        start_query = time.perf_counter()
        
        # Búsqueda híbrida completa
        db.search_memory(
            namespace=namespace,
            query_vector=q_vec,
            text_query=text_q,
            top_k=top_k
        )
        
        duration = (time.perf_counter() - start_query) * 1000.0
        hybrid_latencies.append(duration)

    # Cerrar la base de datos de manera segura para liberar locks
    db.close()

    # Cálculo y procesamiento final de métricas
    l_p50, l_p95, l_p99 = calculate_percentiles(lexical_latencies)
    v_p50, v_p95, v_p99 = calculate_percentiles(vector_latencies)
    h_p50, h_p95, h_p99 = calculate_percentiles(hybrid_latencies)

    print("\n--------------------------------------------------")
    print("                Benchmark Results                 ")
    print("--------------------------------------------------")
    print(f"Ingestion Throughput : {ingest_throughput:.2f} rec/sec")
    print(f"Index Rebuild Time   : {rebuild_duration:.2f} ms")
    print(f"Latencies (ms):")
    print(f"  Lexical BM25  -> p50: {l_p50:.4f} ms | p95: {l_p95:.4f} ms | p99: {l_p99:.4f} ms")
    print(f"  Vector HNSW   -> p50: {v_p50:.4f} ms | p95: {v_p95:.4f} ms | p99: {v_p99:.4f} ms")
    print(f"  Hybrid Fusion -> p50: {h_p50:.4f} ms | p95: {h_p95:.4f} ms | p99: {h_p99:.4f} ms")
    print("==================================================")

    # 5. Exportar reporte con paridad absoluta frente a benchmark_internal.rs
    report = {
        "insert": {
            "total_records": num_vectors,
            "total_duration_ms": ingest_duration * 1000.0,
            "throughput_records_per_sec": ingest_throughput,
        },
        "rebuild": {
            "duration_ms": rebuild_duration,
        },
        "query_text": {
            "p50_ms": l_p50,
            "p95_ms": l_p95,
            "p99_ms": l_p99,
        },
        "query_vector": {
            "p50_ms": v_p50,
            "p95_ms": v_p95,
            "p99_ms": v_p99,
        },
        "query_hybrid": {
            "p50_ms": h_p50,
            "p95_ms": h_p95,
            "p99_ms": h_p99,
        }
    }

    if output_file:
        # Asegurar que el directorio de salida existe
        out_dir = os.path.dirname(output_file)
        if out_dir and not os.path.exists(out_dir):
            os.makedirs(out_dir, exist_ok=True)
            
        with open(output_file, "w") as f:
            json.dump(report, f, indent=4)
        print(f"Report exported successfully with paridad MVP to: {output_file}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="VantaDB Performance Benchmark (MVP Boundary)")
    parser.add_argument("--size", type=int, default=10000, help="Number of records to ingest")
    parser.add_argument("--dim", type=int, default=128, help="Dimension of vectors")
    parser.add_argument("--top-k", type=int, default=10, help="Number of neighbors to retrieve")
    parser.add_argument("--queries", type=int, default=1000, help="Number of queries to perform")
    parser.add_argument("--db-path", type=str, default="./benchmarks/data_bench_db", help="Database storage path")
    parser.add_argument("--output", type=str, default="benchmarks/vanta_benchmark_report.json", help="Output JSON path")

    args = parser.parse_args()
    run_benchmark(
        db_path=args.db_path,
        num_vectors=args.size,
        dim=args.dim,
        top_k=args.top_k,
        num_queries=args.queries,
        output_file=args.output,
    )

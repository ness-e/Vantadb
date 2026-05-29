#!/usr/bin/env python3
"""
VantaDB Prefetch Performance Comparison Suite (SCALE-01c)
Executes A/B testing on vector search latency with and without prefetching.
Updates docs/BENCHMARKS.md with certified results.
"""

import argparse
import gc
import json
import math
import os
import random
import shutil
import sys
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
    """Computes mean, p50, p95, and p99 latencies in milliseconds."""
    if not latencies:
        return 0.0, 0.0, 0.0, 0.0
    sorted_lats = sorted(latencies)
    n = len(sorted_lats)
    mean = sum(sorted_lats) / n
    p50 = sorted_lats[int(n * 0.50)]
    p95 = sorted_lats[int(n * 0.95)]
    p99 = sorted_lats[int(n * 0.99)]
    return mean, p50, p95, p99


def format_duration(seconds):
    """Formats seconds into a human-readable string."""
    if seconds < 60:
        return f"{seconds:.1f}s"
    minutes = int(seconds // 60)
    secs = seconds % 60
    return f"{minutes}m {secs:.0f}s"


def print_progress_bar(current, total, prefix="", elapsed=0.0, bar_len=30):
    """Prints an inline progress bar with speed and ETA."""
    pct = current / total if total > 0 else 1.0
    filled = int(bar_len * pct)
    bar = "█" * filled + "░" * (bar_len - filled)

    speed = current / elapsed if elapsed > 0 else 0.0
    eta = (total - current) / speed if speed > 0 else 0.0

    line = f"\r  {prefix} [{bar}] {pct*100:5.1f}% | {current}/{total} | {speed:.0f} ops/s | ETA: {format_duration(eta)}"
    sys.stdout.write(line)
    sys.stdout.flush()


def update_benchmarks_markdown(markdown_path, results_table):
    """Dynamically replaces prefetch benchmark section in docs/BENCHMARKS.md."""
    if not os.path.exists(markdown_path):
        print(f"Warning: {markdown_path} not found. Skipping auto-update.")
        return

    with open(markdown_path, "r", encoding="utf-8") as f:
        content = f.read()

    start_tag = "<!-- PREFETCH_BENCHMARK_START -->"
    end_tag = "<!-- PREFETCH_BENCHMARK_END -->"

    start_idx = content.find(start_tag)
    end_idx = content.find(end_tag)

    if start_idx == -1 or end_idx == -1:
        print("Warning: Benchmark boundary comments not found in markdown file.")
        return

    new_content = (
        content[:start_idx + len(start_tag)]
        + "\n"
        + results_table
        + "\n"
        + content[end_idx:]
    )

    with open(markdown_path, "w", encoding="utf-8") as f:
        f.write(new_content)
    print(f"  ✅ Updated benchmark metrics in: {markdown_path}")


def main():
    parser = argparse.ArgumentParser(description="VantaDB Prefetch A/B Benchmark")
    parser.add_argument("--size", type=int, default=30000, help="Number of vectors to ingest")
    parser.add_argument("--dim", type=int, default=128, help="Dimensionality of vectors")
    parser.add_argument("--queries", type=int, default=500, help="Number of queries to execute")
    parser.add_argument("--top-k", type=int, default=10, help="Top-K nearest neighbors")
    parser.add_argument("--db-path", type=str, default="data_prefetch_bench", help="Path to temp database")
    args = parser.parse_args()

    print("╔══════════════════════════════════════════════════╗")
    print("║   VantaDB Prefetch A/B Comparison (SCALE-01c)   ║")
    print("╠══════════════════════════════════════════════════╣")
    print(f"║  Dataset Size : {args.size:>10} vectors              ║")
    print(f"║  Dimension    : {args.dim:>10}                      ║")
    print(f"║  Queries      : {args.queries:>10}                      ║")
    print(f"║  Top-K        : {args.top_k:>10}                      ║")
    print(f"║  Database Path: {args.db_path:<33}║")
    print("╚══════════════════════════════════════════════════╝")

    total_start = time.perf_counter()

    # Limpieza inicial
    if os.path.exists(args.db_path):
        print(f"\n  🗑️  Cleaning database directory: {args.db_path}")
        shutil.rmtree(args.db_path)

    # Habilitamos el prefetch durante la ingesta
    if "VANTA_DISABLE_PREFETCH" in os.environ:
        del os.environ["VANTA_DISABLE_PREFETCH"]

    print("\n  ⚙️  Initializing VantaDB...")
    db = vantadb.VantaDB(args.db_path)

    # ── Generación de vectores ─────────────────────────────────────
    print(f"  🎲 Generating {args.size} synthetic vectors ({args.dim}d)...")
    gen_start = time.perf_counter()
    vectors = []
    for i in range(args.size):
        vectors.append(generate_unit_vector(args.dim))
        if (i + 1) % (args.size // 10 or 1) == 0:
            print_progress_bar(i + 1, args.size, prefix="Gen", elapsed=time.perf_counter() - gen_start)
    print_progress_bar(args.size, args.size, prefix="Gen", elapsed=time.perf_counter() - gen_start)
    gen_duration = time.perf_counter() - gen_start
    print(f"\n  ✅ Vector generation completed in {format_duration(gen_duration)}")

    print(f"  🎲 Generating {args.queries} query vectors ({args.dim}d)...")
    query_vectors = [generate_unit_vector(args.dim) for _ in range(args.queries)]

    # ── Ingesta ────────────────────────────────────────────────────
    print(f"\n  📥 Ingesting {args.size} vectors into VantaDB...")
    start_ingest = time.perf_counter()
    namespace = "bench/prefetch"
    report_step = max(args.size // 20, 1)  # Report every 5%
    for i, vec in enumerate(vectors):
        db.put(
            namespace=namespace,
            key=f"doc-{i:05d}",
            payload=f"synthetic benchmark document record_{i}",
            vector=vec
        )
        if (i + 1) % report_step == 0 or (i + 1) == args.size:
            print_progress_bar(i + 1, args.size, prefix="PUT", elapsed=time.perf_counter() - start_ingest)

    print()  # newline after progress bar
    print("  💾 Flushing to disk...")
    db.flush()
    ingest_duration = time.perf_counter() - start_ingest
    print(f"  ✅ Ingestion completed in {format_duration(ingest_duration)} ({args.size / ingest_duration:.0f} vec/s)")

    # Cerramos la DB para liberar todo el estado
    print("  🔒 Closing database to freeze physical layout...")
    db = None
    gc.collect()
    time.sleep(1)

    # ==================================================
    # TEST A: Sin Prefetch (VANTA_DISABLE_PREFETCH = 1)
    # ==================================================
    print("\n┌──────────────────────────────────────────────────┐")
    print("│  TEST A: Search WITHOUT Prefetching              │")
    print("└──────────────────────────────────────────────────┘")
    os.environ["VANTA_DISABLE_PREFETCH"] = "1"

    print("  ⚙️  Opening database (prefetch DISABLED)...")
    db_no_pf = vantadb.VantaDB(args.db_path)

    # Warmup
    print("  🔥 Warming up search cache (10 queries)...")
    for q in query_vectors[:10]:
        db_no_pf.search_memory(namespace=namespace, query_vector=q, top_k=args.top_k)

    # Medición
    print(f"  📊 Measuring {args.queries} queries...")
    search_start = time.perf_counter()
    latencies_no_pf = []
    search_report_step = max(args.queries // 10, 1)
    for i, q in enumerate(query_vectors):
        t_start = time.perf_counter()
        db_no_pf.search_memory(namespace=namespace, query_vector=q, top_k=args.top_k)
        latencies_no_pf.append((time.perf_counter() - t_start) * 1000.0)  # ms
        if (i + 1) % search_report_step == 0 or (i + 1) == args.queries:
            print_progress_bar(i + 1, args.queries, prefix="QRY", elapsed=time.perf_counter() - search_start)

    search_a_duration = time.perf_counter() - search_start
    print(f"\n  ✅ Test A completed in {format_duration(search_a_duration)}")

    db_no_pf = None
    gc.collect()
    time.sleep(1)

    # ==================================================
    # TEST B: Con Prefetch (Default)
    # ==================================================
    print("\n┌──────────────────────────────────────────────────┐")
    print("│  TEST B: Search WITH Prefetching                 │")
    print("└──────────────────────────────────────────────────┘")
    if "VANTA_DISABLE_PREFETCH" in os.environ:
        del os.environ["VANTA_DISABLE_PREFETCH"]

    print("  ⚙️  Opening database (prefetch ENABLED)...")
    db_pf = vantadb.VantaDB(args.db_path)

    # Warmup
    print("  🔥 Warming up search cache (10 queries)...")
    for q in query_vectors[:10]:
        db_pf.search_memory(namespace=namespace, query_vector=q, top_k=args.top_k)

    # Medición
    print(f"  📊 Measuring {args.queries} queries...")
    search_start = time.perf_counter()
    latencies_pf = []
    for i, q in enumerate(query_vectors):
        t_start = time.perf_counter()
        db_pf.search_memory(namespace=namespace, query_vector=q, top_k=args.top_k)
        latencies_pf.append((time.perf_counter() - t_start) * 1000.0)  # ms
        if (i + 1) % search_report_step == 0 or (i + 1) == args.queries:
            print_progress_bar(i + 1, args.queries, prefix="QRY", elapsed=time.perf_counter() - search_start)

    search_b_duration = time.perf_counter() - search_start
    print(f"\n  ✅ Test B completed in {format_duration(search_b_duration)}")

    db_pf = None
    gc.collect()
    time.sleep(1)

    # ==================================================
    # Análisis y Resultados
    # ==================================================
    mean_no_pf, p50_no_pf, p95_no_pf, p99_no_pf = calculate_percentiles(latencies_no_pf)
    mean_pf, p50_pf, p95_pf, p99_pf = calculate_percentiles(latencies_pf)

    # Porcentajes de reducción (ganancia de rendimiento)
    def pct_change(old, new):
        if old <= 0:
            return 0.0
        return ((old - new) / old) * 100.0

    red_mean = pct_change(mean_no_pf, mean_pf)
    red_p50 = pct_change(p50_no_pf, p50_pf)
    red_p95 = pct_change(p95_no_pf, p95_pf)
    red_p99 = pct_change(p99_no_pf, p99_pf)

    # QPS (Queries Per Second)
    qps_no_pf = 1000.0 / mean_no_pf if mean_no_pf > 0 else 0.0
    qps_pf = 1000.0 / mean_pf if mean_pf > 0 else 0.0
    gain_qps = ((qps_pf - qps_no_pf) / qps_no_pf) * 100.0 if qps_no_pf > 0 else 0.0

    # Construir tabla comparativa
    table = []
    table.append("| Métrica | Sin Prefetch (A) | Con Prefetch (B) | Mejora (%) |")
    table.append("| :--- | :--- | :--- | :--- |")
    table.append(f"| **Latencia Media** | {mean_no_pf:.3f} ms | {mean_pf:.3f} ms | **{red_mean:.1f}%** |")
    table.append(f"| **Latencia p50** | {p50_no_pf:.3f} ms | {p50_pf:.3f} ms | **{red_p50:.1f}%** |")
    table.append(f"| **Latencia p95** | {p95_no_pf:.3f} ms | {p95_pf:.3f} ms | **{red_p95:.1f}%** |")
    table.append(f"| **Latencia p99** | {p99_no_pf:.3f} ms | {p99_pf:.3f} ms | **{red_p99:.1f}%** |")
    table.append(f"| **Throughput (QPS)** | {qps_no_pf:.1f} qps | {qps_pf:.1f} qps | **+{gain_qps:.1f}%** |")

    results_table = "\n".join(table)

    total_duration = time.perf_counter() - total_start

    print("\n╔══════════════════════════════════════════════════╗")
    print("║              BENCHMARK RESULTS                   ║")
    print("╠══════════════════════════════════════════════════╣")
    print(results_table)
    print("╠══════════════════════════════════════════════════╣")
    print(f"║  Total runtime: {format_duration(total_duration):>33}║")
    print("╚══════════════════════════════════════════════════╝")

    # Actualizar docs/BENCHMARKS.md
    markdown_path = os.path.join("docs", "BENCHMARKS.md")
    update_benchmarks_markdown(markdown_path, results_table)

    # Limpieza final de directorio
    if os.path.exists(args.db_path):
        shutil.rmtree(args.db_path)
    print("  🗑️  Cleaned up temporary database.")
    print("  ✅ Benchmark suite completed successfully.")


if __name__ == "__main__":
    main()

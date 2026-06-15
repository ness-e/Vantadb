#!/usr/bin/env python3
"""
VantaDB Benchmark Markdown Auto-Updater
Parses the benchmark results JSON and updates the metrics table in docs/BENCHMARKS.md.
"""

import os
import json

RESULTS_FILE = "benchmark_results.json"
BENCHMARK_MD = "docs/operations/BENCHMARKS.md"

START_MARKER = "<!-- BENCHMARK_METRICS_START -->"
END_MARKER = "<!-- BENCHMARK_METRICS_END -->"

def main():
    results_path = RESULTS_FILE
    if not os.path.exists(results_path):
        fallback_path = "benchmarks/vanta_benchmark_report.json"
        if os.path.exists(fallback_path):
            results_path = fallback_path
        else:
            print(f"Error: Neither {RESULTS_FILE} nor {fallback_path} found. Exiting.")
            return

    if not os.path.exists(BENCHMARK_MD):
        print(f"Error: {BENCHMARK_MD} not found.")
        return

    # 1. Leer los resultados JSON reales
    with open(results_path, "r") as f:
        metrics = json.load(f)

    # Extraer valores de forma segura
    insert_block = metrics.get("insert", {})
    insert_tput = insert_block.get("throughput_records_per_sec", 0.0)
    insert_p50 = insert_block.get("p50_ms", 0.0)
    insert_p95 = insert_block.get("p95_ms", 0.0)
    insert_p99 = insert_block.get("p99_ms", 0.0)
    total_records = insert_block.get("total_records", 0)

    rebuild_block = metrics.get("rebuild", {})
    rebuild_ms = rebuild_block.get("duration_ms", 0.0)
    rebuild_tput = total_records / (rebuild_ms / 1000.0) if rebuild_ms > 0 else 0.0
    
    q_text = metrics.get("query_text", {})
    q_vector = metrics.get("query_vector", {})
    q_hybrid = metrics.get("query_hybrid", {})

    def format_tput(tput):
        return f"**{tput:,.0f} ops/sec**" if tput > 0 else "*N/D*"

    def format_qps(ms):
        if ms > 0:
            qps = 1000.0 / ms
            return f"**{qps:,.0f} qps**"
        return "*N/D*"

    def format_latency(ms):
        return f"**{ms:.3f} ms**" if ms > 0 else "*N/D*"

    # 2. Construir la nueva tabla Markdown
    table_lines = [
        "| Operación / Fase | Dataset / Configuración | Latencia p50 | Latencia p95 | Latencia p99 | Rendimiento (Throughput) |",
        "| :--- | :--- | :--- | :--- | :--- | :--- |",
        f"| **Ingesta (`PUT`)** | {total_records:,} registros, 128d | {format_latency(insert_p50)} | {format_latency(insert_p95)} | {format_latency(insert_p99)} | {format_tput(insert_tput)} |",
        f"| **Index Rebuild** | Reconstrucción híbrida (HNSW + BM25) | **{rebuild_ms / 1000.0:.2f}s** | *N/A (Lote único)* | *N/A (Lote único)* | {format_tput(rebuild_tput)} |",
        f"| **Búsqueda Lexical (BM25)** | {total_records:,} registros, `top_k=10` | {format_latency(q_text.get('p50_ms', 0))} | {format_latency(q_text.get('p95_ms', 0))} | {format_latency(q_text.get('p99_ms', 0))} | {format_qps(q_text.get('p50_ms', 0))} |",
        f"| **Búsqueda Vectorial (HNSW)** | {total_records:,} registros, `top_k=10`, 128d | {format_latency(q_vector.get('p50_ms', 0))} | {format_latency(q_vector.get('p95_ms', 0))} | {format_latency(q_vector.get('p99_ms', 0))} | {format_qps(q_vector.get('p50_ms', 0))} |",
        f"| **Búsqueda Híbrida (RRF)** | {total_records:,} registros, `top_k=10`, RRF Fusion | {format_latency(q_hybrid.get('p50_ms', 0))} | {format_latency(q_hybrid.get('p95_ms', 0))} | {format_latency(q_hybrid.get('p99_ms', 0))} | {format_qps(q_hybrid.get('p50_ms', 0))} |"
    ]
    
    new_table_content = "\n".join(table_lines)

    # 3. Leer y actualizar BENCHMARKS.md
    with open(BENCHMARK_MD, "r", encoding="utf-8") as f:
        content = f.read()

    start_idx = content.find(START_MARKER)
    end_idx = content.find(END_MARKER)

    if start_idx == -1 or end_idx == -1:
        print("Error: Markers not found in docs/BENCHMARKS.md")
        return

    updated_content = (
        content[:start_idx + len(START_MARKER)]
        + "\n"
        + new_table_content
        + "\n"
        + content[end_idx:]
    )

    with open(BENCHMARK_MD, "w", encoding="utf-8") as f:
        f.write(updated_content)

    print("Success: docs/BENCHMARKS.md has been dynamically updated with latest CI metrics!")

if __name__ == "__main__":
    main()

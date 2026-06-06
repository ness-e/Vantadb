#!/usr/bin/env python3
"""
VantaDB Competitive Benchmark Suite (T3.2)
Compares VantaDB, LanceDB, and ChromaDB on performance (Ingestion, Read Latency, QPS, Recall, RSS Memory).
Supports glove-100-angular, sift-128-euclidean, and synthetic datasets.
"""

import argparse
import gc
import json
import os
import shutil
import time
import urllib.request
import sys

# 1. Environment and dependency check
MISSING_DEPS = []
for dep in ["numpy", "h5py", "lancedb", "chromadb", "psutil", "tabulate"]:
    try:
        __import__(dep)
    except ImportError:
        MISSING_DEPS.append(dep)

if MISSING_DEPS:
    print("=" * 60)
    print("ERROR: Missing Python dependencies required for this benchmark:")
    for dep in MISSING_DEPS:
        print(f"  - {dep}")
    print("\nPlease install them using the following command:")
    print(f"  pip install {' '.join(MISSING_DEPS)}")
    print("=" * 60)
    sys.exit(1)

import numpy as np
import h5py
import lancedb
import chromadb
import psutil
from tabulate import tabulate

try:
    import vantadb_py as vantadb
except ImportError:
    print("ERROR: 'vantadb_py' is not installed.")
    print("Please compile and install it first in your current python environment.")
    sys.exit(1)


# 2. Memory tracking utility
PROCESS = psutil.Process()

def get_current_rss():
    """Returns the current RSS memory of the process in MB."""
    gc.collect()
    return PROCESS.memory_info().rss / (1024.0 * 1024.0)


# 3. Dataset Downloader and Reader
DATASET_URLS = {
    "glove-100-angular": "http://ann-benchmarks.com/glove-100-angular.hdf5",
    "sift-128-euclidean": "http://ann-benchmarks.com/sift-128-euclidean.hdf5",
}

def download_progress(block_num, block_size, total_size):
    read_so_far = block_num * block_size
    if total_size > 0:
        percent = min(100.0, read_so_far * 100.0 / total_size)
        sys.stdout.write(f"\rDownloading dataset... {percent:5.1f}% [{read_so_far / (1024*1024):.1f}MB / {total_size / (1024*1024):.1f}MB]")
        sys.stdout.flush()
    else:
        sys.stdout.write(f"\rDownloading dataset... {read_so_far / (1024*1024):.1f}MB")
        sys.stdout.flush()

def load_dataset(dataset_name, dataset_dir, max_size, max_queries):
    """Loads GloVe/SIFT from local/downloaded HDF5, or falls back to synthetic data."""
    if dataset_name not in DATASET_URLS:
        print(f"Dataset '{dataset_name}' not in standard ann-benchmarks list. Generating synthetic fallback...")
        return generate_synthetic_data(128, max_size, max_queries, metric="euclidean")

    os.makedirs(dataset_dir, exist_ok=True)
    filepath = os.path.join(dataset_dir, f"{dataset_name}.hdf5")

    if not os.path.exists(filepath):
        url = DATASET_URLS[dataset_name]
        print(f"Dataset '{dataset_name}' not found locally at {filepath}.")
        print(f"Initiating download from: {url}")
        try:
            urllib.request.urlretrieve(url, filepath, download_progress)
            print("\nDownload complete.")
        except Exception as e:
            print(f"\nERROR: Failed to download dataset: {e}")
            print("Falling back to synthetic dataset generation...")
            dim = 100 if "glove" in dataset_name else 128
            metric = "cosine" if "glove" in dataset_name else "euclidean"
            return generate_synthetic_data(dim, max_size, max_queries, metric)

    print(f"Loading dataset from HDF5 file: {filepath}...")
    try:
        with h5py.File(filepath, "r") as f:
            train_all = f["train"]
            test_all = f["test"]
            
            n_train = min(len(train_all), max_size)
            n_test = min(len(test_all), max_queries)
            
            train_vectors = np.array(train_all[:n_train], dtype=np.float32)
            test_vectors = np.array(test_all[:n_test], dtype=np.float32)
            
            metric = "cosine" if "glove" in dataset_name else "euclidean"
            
            # If we load a subset of vectors, we MUST compute exact ground truth because the HDF5 pre-computed
            # neighbors are based on the full 1M+ dataset, which might contain closest items that we didn't load.
            if n_train < len(train_all):
                print(f"Dataset subset loaded ({n_train}/{len(train_all)}). Computing local ground truth for queries...")
                neighbors = compute_ground_truth(train_vectors, test_vectors, metric)
            else:
                print("Using pre-computed ground truth from HDF5...")
                neighbors = np.array(f["neighbors"][:n_test], dtype=np.int32)
                
            return train_vectors, test_vectors, neighbors, metric
    except Exception as e:
        print(f"ERROR reading HDF5: {e}. Falling back to synthetic...")
        dim = 100 if "glove" in dataset_name else 128
        metric = "cosine" if "glove" in dataset_name else "euclidean"
        return generate_synthetic_data(dim, max_size, max_queries, metric)

def generate_synthetic_data(dim, size, queries, metric):
    """Generates normalized synthetic vectors and calculates exact ground truth."""
    print(f"Generating synthetic dataset ({size} vectors, {dim}d, metric={metric})...")
    # Ingest vectors
    train_vectors = np.random.uniform(-1.0, 1.0, (size, dim)).astype(np.float32)
    # Query vectors
    test_vectors = np.random.uniform(-1.0, 1.0, (queries, dim)).astype(np.float32)

    # Normalize to unit length for cosine/angular distance evaluation
    if metric == "cosine":
        train_norms = np.linalg.norm(train_vectors, axis=1, keepdims=True)
        train_vectors = np.divide(train_vectors, train_norms, out=train_vectors, where=train_norms > 0)
        
        test_norms = np.linalg.norm(test_vectors, axis=1, keepdims=True)
        test_vectors = np.divide(test_vectors, test_norms, out=test_vectors, where=test_norms > 0)

    neighbors = compute_ground_truth(train_vectors, test_vectors, metric)
    return train_vectors, test_vectors, neighbors, metric

def compute_ground_truth(train_vectors, test_vectors, metric, top_k=100):
    """Computes exact neighbors (indices) for test_vectors against train_vectors."""
    print(f"Computing exact neighbors via brute-force numpy (top_k={top_k})...")
    neighbors = []
    
    # Process queries in batches to avoid high memory usage for large sizes
    batch_size = 100
    for i in range(0, len(test_vectors), batch_size):
        q_batch = test_vectors[i : i + batch_size]
        
        if metric == "cosine":
            # Cosine similarity = dot product of normalized vectors
            # Distance = 1 - similarity. So we want to maximize dot product (minimize distance).
            dots = np.dot(q_batch, train_vectors.T) # shape: (batch_size, train_size)
            # Find indices of largest elements
            indices = np.argpartition(-dots, top_k, axis=1)[:, :top_k]
            # Sort partition
            for row_idx, row in enumerate(indices):
                sorted_idx = row[np.argsort(-dots[row_idx, row])]
                neighbors.append(sorted_idx)
        else:
            # Euclidean distance: ||a - b||^2 = ||a||^2 + ||b||^2 - 2<a, b>
            q_sq = np.sum(q_batch ** 2, axis=1, keepdims=True)
            t_sq = np.sum(train_vectors ** 2, axis=1, keepdims=True).T
            dists = q_sq + t_sq - 2 * np.dot(q_batch, train_vectors.T)
            indices = np.argpartition(dists, top_k, axis=1)[:, :top_k]
            for row_idx, row in enumerate(indices):
                sorted_idx = row[np.argsort(dists[row_idx, row])]
                neighbors.append(sorted_idx)
                
    return np.array(neighbors, dtype=np.int32)


# 4. Engine Benchmark Functions
def bench_vantadb(db_path, train_vectors, test_vectors, ground_truth, metric, top_k):
    print("\nBenchmarking VantaDB...")
    if os.path.exists(db_path):
        shutil.rmtree(db_path)

    rss_start = get_current_rss()
    
    # 1. Ingestion
    start_time = time.perf_counter()
    db = vantadb.VantaDB(db_path)
    
    # VantaDB configuration check: map metric
    # The default distance metric on instantiation maps to cosine. Let's pass the parameter if supported
    # or rely on standard config.
    namespace = "bench"
    for i, vec in enumerate(train_vectors):
        db.put(
            namespace=namespace,
            key=f"doc-{i}",
            payload=f"Payload metadata entry for vector number {i}",
            metadata={"index": i},
            vector=vec.tolist()
        )
    
    db.flush()
    ingest_time = time.perf_counter() - start_time
    rss_after_ingest = get_current_rss()

    # 2. Index Rebuild
    start_index = time.perf_counter()
    db.rebuild_index()
    index_time = time.perf_counter() - start_index
    rss_after_index = get_current_rss()

    # 3. Queries
    query_times = []
    predictions = []
    
    for q in test_vectors:
        t_start = time.perf_counter()
        results = db.search_memory(
            namespace=namespace,
            query_vector=q.tolist(),
            top_k=top_k,
            distance_metric=metric
        )
        duration = (time.perf_counter() - t_start) * 1000.0 # ms
        query_times.append(duration)
        
        # Parse result indices
        pred_ids = []
        for item in results:
            try:
                # Key is formatted as "doc-i"
                idx = int(item['record']['key'].split('-')[1])
                pred_ids.append(idx)
            except Exception:
                pass
        predictions.append(pred_ids)

    db.close()
    
    # Calculate Recall
    recalls = []
    for pred, gt in zip(predictions, ground_truth):
        gt_set = set(gt[:top_k])
        matches = len(set(pred).intersection(gt_set))
        recalls.append(matches / top_k)
        
    avg_recall = np.mean(recalls)
    p50 = np.percentile(query_times, 50)
    p95 = np.percentile(query_times, 95)
    p99 = np.percentile(query_times, 99)
    qps = len(test_vectors) / (sum(query_times) / 1000.0)

    # Clean storage
    shutil.rmtree(db_path, ignore_errors=True)

    return {
        "engine": "VantaDB",
        "ingest_throughput": len(train_vectors) / ingest_time,
        "index_time_ms": index_time * 1000.0,
        "query_p50_ms": p50,
        "query_p99_ms": p99,
        "qps": qps,
        "recall_at_k": avg_recall,
        "mem_peak_rss_mb": max(rss_after_ingest, rss_after_index),
        "mem_leak_rss_mb": rss_after_index - rss_start
    }


def bench_lancedb(db_path, train_vectors, test_vectors, ground_truth, metric, top_k):
    print("\nBenchmarking LanceDB...")
    if os.path.exists(db_path):
        shutil.rmtree(db_path)

    rss_start = get_current_rss()
    
    # 1. Ingestion
    start_time = time.perf_counter()
    db = lancedb.connect(db_path)
    
    # Prepare data for insertion (list of dicts)
    data = [{"vector": vec.tolist(), "id": int(i)} for i, vec in enumerate(train_vectors)]
    tbl = db.create_table("vectors", data=data, mode="overwrite")
    ingest_time = time.perf_counter() - start_time
    rss_after_ingest = get_current_rss()

    # 2. Index Creation (IVF-PQ/Vector index to ensure fair index-to-index search)
    # LanceDB defaults to brute-force unless index is explicitly created.
    # Recommended sizes for indexing is typically >256 vectors.
    start_index = time.perf_counter()
    if len(train_vectors) >= 512:
        # Create IVF-PQ index. Partitions and sub-vectors based on dataset scale.
        num_partitions = min(256, max(16, len(train_vectors) // 64))
        tbl.create_index(
            metric="cosine" if metric == "cosine" else "l2",
            num_partitions=num_partitions,
            num_sub_vectors=8
        )
    index_time = time.perf_counter() - start_index
    rss_after_index = get_current_rss()

    # 3. Queries
    query_times = []
    predictions = []
    
    for q in test_vectors:
        t_start = time.perf_counter()
        results = tbl.search(q.tolist()).metric("cosine" if metric == "cosine" else "l2").nprobes(32).limit(top_k).to_list()
        duration = (time.perf_counter() - t_start) * 1000.0 # ms
        query_times.append(duration)
        
        pred_ids = [int(item['id']) for item in results]
        predictions.append(pred_ids)

    # Close/garbage collect lance connections
    del tbl
    del db
    gc.collect()

    # Calculate Recall
    recalls = []
    for pred, gt in zip(predictions, ground_truth):
        gt_set = set(gt[:top_k])
        matches = len(set(pred).intersection(gt_set))
        recalls.append(matches / top_k)
        
    avg_recall = np.mean(recalls)
    p50 = np.percentile(query_times, 50)
    p95 = np.percentile(query_times, 95)
    p99 = np.percentile(query_times, 99)
    qps = len(test_vectors) / (sum(query_times) / 1000.0)

    # Clean storage
    shutil.rmtree(db_path, ignore_errors=True)

    return {
        "engine": "LanceDB",
        "ingest_throughput": len(train_vectors) / ingest_time,
        "index_time_ms": index_time * 1000.0,
        "query_p50_ms": p50,
        "query_p99_ms": p99,
        "qps": qps,
        "recall_at_k": avg_recall,
        "mem_peak_rss_mb": max(rss_after_ingest, rss_after_index),
        "mem_leak_rss_mb": rss_after_index - rss_start
    }


def bench_chromadb(db_path, train_vectors, test_vectors, ground_truth, metric, top_k):
    print("\nBenchmarking ChromaDB...")
    if os.path.exists(db_path):
        shutil.rmtree(db_path)

    rss_start = get_current_rss()
    
    # 1. Ingestion (Index built automatically as metadata gets added to HNSW)
    start_time = time.perf_counter()
    client = chromadb.PersistentClient(path=db_path)
    
    # Map space metric
    space = "cosine" if metric == "cosine" else "l2"
    collection = client.create_collection(
        name="vectors",
        metadata={"hnsw:space": space}
    )
    
    # Ingest in batches to prevent gRPC/memory limits in Chroma's wrapper
    batch_size = 1000
    for idx in range(0, len(train_vectors), batch_size):
        end_idx = min(idx + batch_size, len(train_vectors))
        ids = [str(i) for i in range(idx, end_idx)]
        vectors_list = train_vectors[idx:end_idx].tolist()
        documents = [f"Doc_{i}" for i in range(idx, end_idx)]
        collection.add(
            ids=ids,
            embeddings=vectors_list,
            documents=documents
        )
        
    ingest_time = time.perf_counter() - start_time
    rss_after_ingest = get_current_rss()
    
    # Chroma handles indexing during inserts (incremental HNSW), 
    # so we measure indexing time as 0 or part of ingestion.
    index_time = 0.0 
    rss_after_index = get_current_rss()

    # 3. Queries
    query_times = []
    predictions = []
    
    for q in test_vectors:
        t_start = time.perf_counter()
        results = collection.query(
            query_embeddings=[q.tolist()],
            n_results=top_k
        )
        duration = (time.perf_counter() - t_start) * 1000.0 # ms
        query_times.append(duration)
        
        # Parse IDs
        pred_ids = [int(x) for x in results['ids'][0]] if results['ids'] else []
        predictions.append(pred_ids)

    # Clean Chroma references
    del collection
    del client
    gc.collect()

    # Calculate Recall
    recalls = []
    for pred, gt in zip(predictions, ground_truth):
        gt_set = set(gt[:top_k])
        matches = len(set(pred).intersection(gt_set))
        recalls.append(matches / top_k)
        
    avg_recall = np.mean(recalls)
    p50 = np.percentile(query_times, 50)
    p95 = np.percentile(query_times, 95)
    p99 = np.percentile(query_times, 99)
    qps = len(test_vectors) / (sum(query_times) / 1000.0)

    # Clean storage
    shutil.rmtree(db_path, ignore_errors=True)

    return {
        "engine": "ChromaDB",
        "ingest_throughput": len(train_vectors) / ingest_time,
        "index_time_ms": index_time * 1000.0,
        "query_p50_ms": p50,
        "query_p99_ms": p99,
        "qps": qps,
        "recall_at_k": avg_recall,
        "mem_peak_rss_mb": max(rss_after_ingest, rss_after_index),
        "mem_leak_rss_mb": rss_after_index - rss_start
    }


# 5. Main Execution Loop
def main():
    parser = argparse.ArgumentParser(description="VantaDB Competitive Benchmark Suite")
    parser.add_argument("--dataset", type=str, default="synthetic", help="glove-100-angular, sift-128-euclidean, or synthetic")
    parser.add_argument("--size", type=int, default=10000, help="Number of database vectors to load/generate")
    parser.add_argument("--queries", type=int, default=100, help="Number of query vectors")
    parser.add_argument("--top-k", type=int, default=10, help="Top K neighbors to retrieve")
    parser.add_argument("--dataset-dir", type=str, default="./benchmarks/datasets", help="Path to HDF5 dataset folder")
    parser.add_argument("--db-dir", type=str, default="./benchmarks/competitive_data", help="Temporal folder for databases")
    parser.add_argument("--output", type=str, default="docs/BENCHMARKS.md", help="Path to docs/BENCHMARKS.md to append results")
    args = parser.parse_args()

    print("=" * 60)
    print("        VantaDB Competitive Benchmark Suite (T3.2)       ")
    print("=" * 60)
    print(f"Dataset      : {args.dataset}")
    print(f"Dataset Size : {args.size}")
    print(f"Queries      : {args.queries}")
    print(f"Top-K        : {args.top_k}")
    print("=" * 60)

    # Load vectors and ground truth
    train_vectors, test_vectors, ground_truth, metric = load_dataset(
        args.dataset, args.dataset_dir, args.size, args.queries
    )

    print(f"\nVectors shape: {train_vectors.shape}")
    print(f"Queries shape: {test_vectors.shape}")
    print(f"Metric used  : {metric}")

    os.makedirs(args.db_dir, exist_ok=True)

    results = []

    # Run benchmarks with garbage collection in between
    engines = [
        ("vanta", bench_vantadb, os.path.join(args.db_dir, "vanta_db")),
        ("lance", bench_lancedb, os.path.join(args.db_dir, "lance_db")),
        ("chroma", bench_chromadb, os.path.join(args.db_dir, "chroma_db")),
    ]

    for name, bench_fn, path in engines:
        gc.collect()
        try:
            res = bench_fn(path, train_vectors, test_vectors, ground_truth, metric, args.top_k)
            results.append(res)
        except Exception as e:
            print(f"ERROR: Failed to benchmark {name}: {e}")
            import traceback
            traceback.print_exc()

    # Clear temp database folder
    shutil.rmtree(args.db_dir, ignore_errors=True)

    # 6. Format and Print Results
    headers = ["Engine", "Ingest QPS", "Index Time (ms)", "Query QPS", "Latency p50 (ms)", "Latency p99 (ms)", "Recall@10", "Peak RSS (MB)", "Delta RSS (MB)"]
    rows = []
    for r in results:
        rows.append([
            r["engine"],
            f"{r['ingest_throughput']:.1f}",
            f"{r['index_time_ms']:.1f}" if r['index_time_ms'] > 0 else "N/A (Inc)",
            f"{r['qps']:.1f}",
            f"{r['query_p50_ms']:.3f}",
            f"{r['query_p99_ms']:.3f}",
            f"{r['recall_at_k'] * 100:.2f}%",
            f"{r['mem_peak_rss_mb']:.1f}",
            f"{r['mem_leak_rss_mb']:.1f}"
        ])

    table_md = tabulate(rows, headers=headers, tablefmt="github")
    
    print("\n" + "=" * 60)
    print("                      BENCHMARK REPORT                      ")
    print("=" * 60)
    print(table_md)
    print("=" * 60)

    # Write report back to docs/BENCHMARKS.md if specified
    if args.output and os.path.exists(args.output):
        try:
            with open(args.output, "r", encoding="utf-8") as f:
                content = f.read()

            title_marker = "## 🚀 7. Competitive Benchmark vs LanceDB & Chroma"
            new_section = f"""

{title_marker}
Este benchmark compara **VantaDB** directamente contra **LanceDB** y **ChromaDB** en ingesta, latencias, precisión (Recall) y huella de memoria en reposo.

* **Fecha de ejecución**: {time.strftime("%Y-%m-%d %H:%M:%S")}
* **Configuración del Dataset**:
  * **Nombre**: `{args.dataset}`
  * **Tamaño Ingestado**: {args.size} registros
  * **Dimensión de Vectores**: {train_vectors.shape[1]}
  * **Consultas Evaluadas**: {args.queries}
  * **Métrica**: `{metric}`
  * **Vecinos (Top-K)**: {args.top_k}

### Tabla Comparativa

{table_md}

*Nota: LanceDB e incremental-HNSW de ChromaDB usan sus wrappers de C/C++ nativos integrados en Python. VantaDB corre a través de sus bindings FFI de PyO3 (`vantadb_py`) consumiendo el core de Rust mapeado en memoria (`mmap`).*
"""

            # If section already exists, replace it, otherwise append.
            if title_marker in content:
                idx = content.find(title_marker)
                content = content[:idx] + new_section
            else:
                content += new_section

            with open(args.output, "w", encoding="utf-8") as f:
                f.write(content)
            print(f"\nSuccessfully updated benchmark results in: {args.output}")
        except Exception as e:
            print(f"Error updating file: {e}")


if __name__ == "__main__":
    main()

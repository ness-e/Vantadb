#!/usr/bin/env python3
"""
VantaDB Competitive Benchmark Suite (T3.2)
Compares VantaDB, LanceDB, and ChromaDB on performance (Ingestion, Read Latency, QPS, Recall, RSS Memory).
Supports glove-100-angular, sift-128-euclidean, and synthetic datasets.

=== Measurement-methodology notes (read before comparing numbers) ===

1. HIDDEN REBUILD — VantaDB's Ingest timer double-counts index building.
   `db.put_batch_raw` is called once with ALL vectors (default `--size 10000`).
   With InsertMode::Auto, the engine skips incremental HNSW insertion when the
   batch is >= 1000 nodes (src/storage/engine/ops.rs:957-964) and performs ONE
   full HNSW rebuild at the end of the batch — INSIDE the Ingest timer. The
   explicit `db.rebuild_index()` in the Index timer then rebuilds AGAIN.
   Net: 2 full index builds per run; the Ingest/Index deltas measure the same
   build regression twice.
   To isolate the hidden rebuild: run with `--batch-size 999`. Chunking the
   Python-side insert to < 1000 nodes per `put_batch_raw` call forces
   InsertMode::Incremental (no rebuild inside Ingest). The delta
   `--batch-size 0` (single call) minus `--batch-size 999` IS the hidden
   rebuild cost.

2. PRE-REGRESSION BASELINE IS NOT DIRECTLY COMPARABLE.
   The 3,157 QPS / 2,196 ms baseline (git diff 1235830e on this file:
   +152/-29 lines) measured different work: no cosine normalization
   (`np.linalg.norm`), no JIT ground-truth (compute_ground_truth), no warmup,
   and no median-of-3 iterations. Any comparison against those numbers is
   approximate only.
"""

import argparse
import gc
import json
import os
import platform
import shutil
import statistics
import tempfile
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

# Optional engines (PERF-03 competitive extension): no-docker embedded modes.
# Each guarded so a missing client never breaks the whole suite; missing engines
# are skipped at runtime and reported as "not measured" in the honest table.
try:
    from qdrant_client import QdrantClient
    from qdrant_client.models import Distance, VectorParams, PointStruct
    HAS_QDRANT = True
except ImportError:
    HAS_QDRANT = False

try:
    from pymilvus import MilvusClient
    from pymilvus.milvus_client import IndexParams
    HAS_MILVUS = True
except ImportError:
    HAS_MILVUS = False

try:
    import vantadb_py as vantadb
except ImportError:
    print("ERROR: 'vantadb_py' is not installed.")
    print("Install it from PyPI (standalone, no Rust build required):")
    print("  pip install vantadb-py")
    print("Full benchmark dependencies: pip install -r benchmarks/requirements.txt")
    print("Or for local development against the source tree:")
    print("  maturin develop --manifest-path vantadb-python/Cargo.toml --release")
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
            
            # Normalize vectors for cosine metric (ann-benchmarks HDF5 stores raw vectors)
            if metric == "cosine":
                train_norms = np.linalg.norm(train_vectors, axis=1, keepdims=True)
                train_vectors = np.divide(train_vectors, train_norms, out=train_vectors, where=train_norms > 0)
                test_norms = np.linalg.norm(test_vectors, axis=1, keepdims=True)
                test_vectors = np.divide(test_vectors, test_norms, out=test_vectors, where=test_norms > 0)
            
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
    np.random.seed(42)  # D2: fixed seed for reproducibility
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
def bench_vantadb(db_path, train_vectors, test_vectors, ground_truth, metric, top_k, batch_size=0):
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
    # PERF: batch insert via put_batch_raw with zero-copy numpy array (~50-300x vs per-vector put())
    # batch_size > 0 chunks the insert so each put_batch_raw call is < 1000 nodes, forcing
    # InsertMode::Incremental (no hidden HNSW rebuild inside this timer — see header comment).
    n = len(train_vectors)
    keys = [f"doc-{i}" for i in range(n)]
    payloads = [f"Payload metadata entry for vector number {i}" for i in range(n)]
    metadatas = [{"index": i} for i in range(n)]

    def _put(vectors_chunk, keys_chunk, payloads_chunk, metadatas_chunk):
        db.put_batch_raw(
            vectors=vectors_chunk,  # numpy float32 ndarray (zero-copy via PyBuffer)
            keys=keys_chunk,
            payloads=payloads_chunk,
            metadatas=metadatas_chunk,
            namespaces=[namespace] * len(keys_chunk),
        )

    if batch_size and batch_size > 0:
        for i in range(0, n, batch_size):
            end = min(i + batch_size, n)
            _put(
                train_vectors[i:end],
                keys[i:end],
                payloads[i:end],
                metadatas[i:end],
            )
    else:
        # Default (--batch-size 0): single call, full array — the engine performs ONE full
        # HNSW rebuild at the end of the batch, INSIDE this timer (see header comment).
        _put(train_vectors, keys, payloads, metadatas)
    db.flush()
    ingest_time = time.perf_counter() - start_time
    rss_after_ingest = get_current_rss()

    # 2. Index Rebuild
    start_index = time.perf_counter()
    db.rebuild_index()
    index_time = time.perf_counter() - start_index
    rss_after_index = get_current_rss()

    # 3. Warm-up: 10 queries (not measured) — D3
    warmup_count = min(10, len(test_vectors))
    for q in test_vectors[:warmup_count]:
        db.search_memory(
            namespace=namespace,
            query_vector=q.tolist(),
            top_k=top_k,
            distance_metric=metric
        )

    # 4. Queries
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
        
        # Parse result indices — VantaSearchHit has `.key` / `.id`
        pred_ids = []
        for item in results:
            try:
                key = item.key if hasattr(item, 'key') else item.get('key', '')
                idx = int(key.split('-')[1])
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
        dim = train_vectors.shape[1]
        num_sub_vectors = 8
        if dim % 8 != 0:
            for d in [4, 5, 10, 2, 20, 25, 50]:
                if dim % d == 0:
                    num_sub_vectors = d
                    break
        tbl.create_index(
            metric="cosine" if metric == "cosine" else "l2",
            num_partitions=num_partitions,
            num_sub_vectors=num_sub_vectors
        )
    index_time = time.perf_counter() - start_index
    rss_after_index = get_current_rss()

    # 3. Warm-up: 10 queries (not measured) — D3
    warmup_count = min(10, len(test_vectors))
    for q in test_vectors[:warmup_count]:
        tbl.search(q.tolist()).metric("cosine" if metric == "cosine" else "l2").nprobes(32).limit(top_k).to_list()

    # 4. Queries
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

    # 3. Warm-up: 10 queries (not measured) — D3
    warmup_count = min(10, len(test_vectors))
    for q in test_vectors[:warmup_count]:
        collection.query(
            query_embeddings=[q.tolist()],
            n_results=top_k
        )

    # 4. Queries
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


def bench_qdrant(db_path, train_vectors, test_vectors, ground_truth, metric, top_k):
    """Qdrant in embedded local mode (QdrantClient(path=...)) — NO docker, same HW.

    This is the honest replacement for the old "we don't benchmark Qdrant" gap:
    Qdrant's local/persistent mode runs in-process, so it is directly comparable
    to VantaDB/LanceDB/ChromaDB without a server or container.
    """
    if not HAS_QDRANT:
        print("\n[SKIP] Qdrant not measured: qdrant_client not installed.")
        return None
    print("\nBenchmarking Qdrant (embedded local mode, no docker)...")
    if os.path.exists(db_path):
        shutil.rmtree(db_path)

    rss_start = get_current_rss()
    dim = train_vectors.shape[1]
    distance = Distance.COSINE if metric == "cosine" else Distance.EUCLID

    client = QdrantClient(path=db_path)
    client.recreate_collection(
        collection_name="vectors",
        vectors_config=VectorParams(size=dim, distance=distance),
    )

    # 1. Ingestion (HNSW is built incrementally during upload — no separate rebuild timer)
    start_time = time.perf_counter()
    points = [
        PointStruct(id=int(i), vector=train_vectors[i].tolist(), payload={"index": i})
        for i in range(len(train_vectors))
    ]
    client.upload_points(collection_name="vectors", points=points, wait=True)
    ingest_time = time.perf_counter() - start_time
    rss_after_ingest = get_current_rss()

    # Qdrant indexes during upload; separate index time not measured (incremental == 0).
    index_time = 0.0
    rss_after_index = get_current_rss()

    # 3. Warm-up: 10 queries (not measured)
    warmup_count = min(10, len(test_vectors))
    for q in test_vectors[:warmup_count]:
        client.query_points("vectors", query=q.tolist(), limit=top_k)

    # 4. Queries
    query_times = []
    predictions = []
    for q in test_vectors:
        t_start = time.perf_counter()
        hits = client.query_points("vectors", query=q.tolist(), limit=top_k)
        duration = (time.perf_counter() - t_start) * 1000.0
        query_times.append(duration)
        predictions.append([int(p.id) for p in hits.points])

    client.close()
    del client
    gc.collect()

    recalls = []
    for pred, gt in zip(predictions, ground_truth):
        gt_set = set(int(x) for x in gt[:top_k])
        matches = len(set(pred).intersection(gt_set))
        recalls.append(matches / top_k)

    avg_recall = np.mean(recalls)
    p50 = np.percentile(query_times, 50)
    p95 = np.percentile(query_times, 95)
    p99 = np.percentile(query_times, 99)
    qps = len(test_vectors) / (sum(query_times) / 1000.0)

    shutil.rmtree(db_path, ignore_errors=True)

    return {
        "engine": "Qdrant",
        "ingest_throughput": len(train_vectors) / ingest_time,
        "index_time_ms": index_time * 1000.0,
        "query_p50_ms": p50,
        "query_p99_ms": p99,
        "qps": qps,
        "recall_at_k": avg_recall,
        "mem_peak_rss_mb": max(rss_after_ingest, rss_after_index),
        "mem_leak_rss_mb": rss_after_index - rss_start,
    }


def bench_milvus(db_path, train_vectors, test_vectors, ground_truth, metric, top_k):
    """Milvus-frugal via milvus-lite (embedded, no docker). Reproducible harness.

    Guardado por import: si `pymilvus`/`milvus-lite` no está instalado, la función
    devuelve None y el motor se marca "no medido" en la tabla honesta (PERF-03).
    """
    if not HAS_MILVUS:
        print("\n[SKIP] Milvus not measured: pymilvus (milvus-lite) not installed.")
        return None
    print("\nBenchmarking Milvus (milvus-lite embedded, no docker)...")
    # Unique temp dir per run: the embedded lite server keeps a Windows file lock
    # (WinError 32) that would block the next iteration if paths collided.
    work_dir = tempfile.mkdtemp(prefix="vanta_milvus_")
    db_uri = os.path.join(work_dir, "milvus.db")

    rss_start = get_current_rss()
    dim = train_vectors.shape[1]
    metric_type = "COSINE" if metric == "cosine" else "L2"

    client = MilvusClient(uri=db_uri)
    if client.has_collection("vectors"):
        client.drop_collection("vectors")
    client.create_collection(
        collection_name="vectors",
        dimension=dim,
        metric_type=metric_type,
    )

    # 1. Ingestion
    start_time = time.perf_counter()
    data = [
        {"id": int(i), "vector": train_vectors[i].tolist(), "index_meta": i}
        for i in range(len(train_vectors))
    ]
    client.insert(collection_name="vectors", data=data)
    client.flush(collection_name="vectors")
    ingest_time = time.perf_counter() - start_time
    rss_after_ingest = get_current_rss()

    # 2. Index creation (HNSW, comparable to the other engines).
    # pymilvus >=2.5 auto-creates a default index on create_collection and loads
    # the collection, so release + drop it first, then build an explicit HNSW
    # index (M=16, efConstruction=100).
    start_index = time.perf_counter()
    client.release_collection(collection_name="vectors")
    client.drop_index(collection_name="vectors", index_name="vector")
    index_params = IndexParams()
    index_params.add_index(
        field_name="vector",
        index_type="HNSW",
        metric_type=metric_type,
        params={"M": 16, "efConstruction": 100},
    )
    client.create_index(collection_name="vectors", index_params=index_params)
    index_time = time.perf_counter() - start_index
    rss_after_index = get_current_rss()

    # 3. Warm-up
    warmup_count = min(10, len(test_vectors))
    for q in test_vectors[:warmup_count]:
        client.search(collection_name="vectors", data=[q.tolist()], limit=top_k)

    # 4. Queries
    query_times = []
    predictions = []
    for q in test_vectors:
        t_start = time.perf_counter()
        res = client.search(collection_name="vectors", data=[q.tolist()], limit=top_k)
        duration = (time.perf_counter() - t_start) * 1000.0
        query_times.append(duration)
        pred_ids = [int(r["id"]) for r in res[0]]
        predictions.append(pred_ids)

    client.close()
    del client
    gc.collect()

    recalls = []
    for pred, gt in zip(predictions, ground_truth):
        gt_set = set(int(x) for x in gt[:top_k])
        matches = len(set(pred).intersection(gt_set))
        recalls.append(matches / top_k)

    avg_recall = np.mean(recalls)
    p50 = np.percentile(query_times, 50)
    p95 = np.percentile(query_times, 95)
    p99 = np.percentile(query_times, 99)
    qps = len(test_vectors) / (sum(query_times) / 1000.0)

    shutil.rmtree(work_dir, ignore_errors=True)

    return {
        "engine": "Milvus",
        "ingest_throughput": len(train_vectors) / ingest_time,
        "index_time_ms": index_time * 1000.0,
        "query_p50_ms": p50,
        "query_p99_ms": p99,
        "qps": qps,
        "recall_at_k": avg_recall,
        "mem_peak_rss_mb": max(rss_after_ingest, rss_after_index),
        "mem_leak_rss_mb": rss_after_index - rss_start,
    }


# 5. System Health Check (D1)
def health_check(skip_prompt=False):
    """Run system health checks before benchmark, optionally prompt user."""
    print("\n" + "-" * 50)
    print("System Health Check")
    print("-" * 50)
    ok = "  [OK]"
    warn = "  [!]"

    # Disk space
    usage = psutil.disk_usage(".")
    disk_free_pct = usage.free / usage.total * 100
    if disk_free_pct > 15:
        print(f"{ok} Disk space: {disk_free_pct:.1f}% free")
    else:
        print(f"{warn} Disk space: {disk_free_pct:.1f}% free (<15% WARNING)")

    # RAM
    ram = psutil.virtual_memory()
    ram_free_gb = ram.available / (1024**3)
    if ram_free_gb > 4:
        print(f"{ok} RAM free: {ram_free_gb:.1f} GB")
    else:
        print(f"{warn} RAM free: {ram_free_gb:.1f} GB (<4 GB WARNING)")

    # RAYON_NUM_THREADS
    rayon_threads = os.environ.get("RAYON_NUM_THREADS")
    if rayon_threads:
        print(f"{ok} RAYON_NUM_THREADS={rayon_threads}")
    else:
        print(f"{warn} RAYON_NUM_THREADS not set (may over-subscribe)")

    # CPU load (short sample)
    cpu_load = psutil.cpu_percent(interval=0.5)
    if cpu_load < 30:
        print(f"{ok} CPU load: {cpu_load:.1f}%")
    else:
        print(f"{warn} CPU load: {cpu_load:.1f}% (>30% WARNING — benchmarks will be contaminated)")

    # VS Code processes
    vscode_count = 0
    for proc in psutil.process_iter(["name"]):
        try:
            if proc.info["name"] and "code" in proc.info["name"].lower():
                vscode_count += 1
        except (psutil.NoSuchProcess, psutil.AccessDenied):
            pass
    if vscode_count > 3:
        print(f"{warn} VS Code processes: {vscode_count} (>3 WARNING)")
    else:
        print(f"{ok} VS Code processes: {vscode_count}")

    print("-" * 50)
    if skip_prompt or not sys.stdin.isatty():
        return
    response = input("Continue with benchmark? [Y/n]: ").strip().lower()
    if response and response not in ("y", "yes", ""):
        print("Benchmark cancelled by user.")
        sys.exit(0)


# 5.5 JSON contract output (INV-007-B)
def _pkg_version(name):
    """Best-effort installed package version; None when not importable."""
    try:
        from importlib.metadata import version
        return version(name)
    except Exception:
        return None


def detect_hardware():
    """Hardware metadata recorded with every run (published in the JSON)."""
    return {
        "os": platform.platform(),
        "cpu_count": os.cpu_count(),
        "cpu_model": platform.processor() or None,
        "python": sys.version.split()[0],
    }


def detect_versions():
    """Engine/library versions at run time (published in the JSON)."""
    return {
        "vantadb": _pkg_version("vantadb-py") or _pkg_version("vantadb_py"),
        "lancedb": _pkg_version("lancedb"),
        "chromadb": _pkg_version("chromadb"),
        "numpy": _pkg_version("numpy"),
    }


def write_json_report(json_path, args, results, n_dim, metric, hardware, versions):
    """
    Write the versioned JSON contract consumed by web/ (INV-007-B).
    Schema lives in web/src/lib/vanta-data.ts; the web imports this file directly.
    """
    doc = {
        "schema_version": 1,
        "generated_by": "benchmarks/competitive_bench.py",
        "generated_at": time.strftime("%Y-%m-%d %H:%M:%S"),
        "source": f"run real del harness ({args.dataset}) — valores generados por esta ejecucion",
        "hardware": hardware,
        "versions": versions,
        "dataset": {
            "name": args.dataset,
            "metric": metric,
            "vectors": args.size,
            "queries": args.queries,
            "top_k": args.top_k,
            "ingest_mode": (
                f"chunked (--batch-size {args.batch_size})"
                if args.batch_size and args.batch_size > 0
                else "single put_batch_raw (--batch-size 0, doble rebuild — no comparable)"
            ),
        },
        "methodology": {
            "iterations_per_engine": 3,
            "aggregation": "median",
            "ground_truth": "exact brute-force numpy over the loaded subset",
            "warmup_queries": 10,
        },
        "results": [],
    }
    for r in results:
        doc["results"].append({
            "engine": r["engine"],
            "ingest_qps": round(r["ingest_throughput"], 1),
            # index_time_ms <= 0 => incremental index (no separate rebuild measured)
            "index_time_ms": round(r["index_time_ms"], 1) if r["index_time_ms"] > 0 else None,
            "query_qps": round(r["qps"], 1),
            "query_p50_ms": round(r["query_p50_ms"], 3),
            "query_p99_ms": round(r["query_p99_ms"], 3),
            "recall_at_k": round(r["recall_at_k"], 4),
            "mem_peak_rss_mb": round(r["mem_peak_rss_mb"], 1),
            "mem_delta_rss_mb": round(r["mem_leak_rss_mb"], 1),
        })
    os.makedirs(os.path.dirname(json_path), exist_ok=True)
    with open(json_path, "w", encoding="utf-8") as f:
        json.dump(doc, f, indent=2, ensure_ascii=False)
    print(f"\nWrote versioned JSON contract: {json_path}")


# 6. Main Execution Loop
def main():
    parser = argparse.ArgumentParser(description="VantaDB Competitive Benchmark Suite")
    parser.add_argument("--dataset", type=str, default="synthetic", help="glove-100-angular, sift-128-euclidean, or synthetic")
    parser.add_argument("--size", type=int, default=10000, help="Number of database vectors to load/generate")
    parser.add_argument("--queries", type=int, default=100, help="Number of query vectors")
    parser.add_argument("--top-k", type=int, default=10, help="Top K neighbors to retrieve")
    parser.add_argument("--batch-size", type=int, default=999,
                        help="VantaDB ingest chunk size. 999 (default) = chunked incremental insert "
                             "(no hidden HNSW rebuild inside the Ingest timer — rebuild measured "
                             "only in Index timer). 0 = single put_batch_raw call (legacy; hidden "
                             "rebuild runs inside Ingest AND Index timers = double rebuild). "
                             "See header comment.")
    parser.add_argument("--dataset-dir", type=str, default="./datasets", help="Path to HDF5 dataset folder")
    parser.add_argument("--db-dir", type=str, default="./benchmarks/competitive_data", help="Temporal folder for databases")
    parser.add_argument("--output", type=str, default="docs/BENCHMARKS.md", help="Path to docs/BENCHMARKS.md to append results")
    parser.add_argument("--json-output", type=str, default="web/src/lib/data/competitive-benchmark.json",
                        help="Path to write the versioned JSON contract (INV-007-B). The web imports this file directly.")
    parser.add_argument("--yes", action="store_true", help="Skip health check prompt")
    parser.add_argument("--engines", type=str, default="vanta,lance,chroma,qdrant",
                        help="Comma-separated engines to benchmark on the SAME HW, no docker. "
                             "Available: vanta,lance,chroma,qdrant,milvus. Missing clients are skipped "
                             "and reported as 'not measured' in the honest table (PERF-03).")
    args = parser.parse_args()

    print("=" * 60)
    print("        VantaDB Competitive Benchmark Suite (T3.2)       ")
    print("=" * 60)
    print(f"Dataset      : {args.dataset}")
    print(f"Dataset Size : {args.size}")
    print(f"Queries      : {args.queries}")
    print(f"Top-K        : {args.top_k}")
    print("=" * 60)

    # D1: Health check before loading datasets
    health_check(skip_prompt=args.yes)

    # Load vectors and ground truth
    train_vectors, test_vectors, ground_truth, metric = load_dataset(
        args.dataset, args.dataset_dir, args.size, args.queries
    )

    print(f"\nVectors shape: {train_vectors.shape}")
    print(f"Queries shape: {test_vectors.shape}")
    print(f"Metric used  : {metric}")

    os.makedirs(args.db_dir, exist_ok=True)

    # D4: 3 iterations per engine, report median.
    # Engine registry (PERF-03): every engine runs on the SAME HW, no docker.
    # Embedded local modes (Qdrant path=, Milvus milvus-lite uri=) are directly
    # comparable to VantaDB/LanceDB/ChromaDB without a server or container.
    # Missing clients are skipped gracefully and reported as "not measured".
    engine_registry = {
        "vanta": (bench_vantadb, os.path.join(args.db_dir, "vanta_db"), {"batch_size": args.batch_size}),
        "lance": (bench_lancedb, os.path.join(args.db_dir, "lance_db"), {}),
        "chroma": (bench_chromadb, os.path.join(args.db_dir, "chroma_db"), {}),
        "qdrant": (bench_qdrant, os.path.join(args.db_dir, "qdrant_db"), {}) if HAS_QDRANT else None,
        "milvus": (bench_milvus, os.path.join(args.db_dir, "milvus_db.db"), {}) if HAS_MILVUS else None,
    }
    requested = [e.strip().lower() for e in args.engines.split(",") if e.strip()]
    engines = []
    for name in requested:
        if name not in engine_registry:
            print(f"[WARN] Unknown engine '{name}' — skipping.")
            continue
        entry = engine_registry[name]
        if entry is None:
            print(f"[SKIP] Engine '{name}' not measured: client library not installed.")
            continue
        engines.append((name, entry[0], entry[1], entry[2]))

    if not engines:
        print("ERROR: No benchmarkable engines available for the requested set.")
        sys.exit(1)

    all_engine_results = []  # (name, [run1_dict, run2_dict, run3_dict])

    for name, bench_fn, path, kwargs in engines:
        print(f"\n--- Running {name.capitalize()} (3 iterations) ---")
        runs = []
        for i in range(3):
            gc.collect()
            try:
                res = bench_fn(path, train_vectors, test_vectors, ground_truth, metric, args.top_k, **kwargs)
                runs.append(res)
            except Exception as e:
                print(f"  ERROR: Failed to benchmark {name} (run {i+1}): {e}")
                import traceback
                traceback.print_exc()
        all_engine_results.append((name, runs))

        if runs:
            qps_values = [r["qps"] for r in runs]
            qps_str = " | ".join(f"run {i+1}: {r['qps']:.1f} QPS" for i, r in enumerate(runs))
            median_qps = statistics.median(qps_values)
            print(f"  {name.capitalize()} {qps_str} | Median: {median_qps:.1f} QPS")

    # Build median results for the final table
    results = []
    for name, runs in all_engine_results:
        if not runs:
            continue
        median_res = {}
        for key in runs[0]:
            if key == "engine":
                median_res[key] = runs[0][key]
            else:
                median_res[key] = statistics.median(r[key] for r in runs)
        results.append(median_res)

    # Clear temp database folder
    shutil.rmtree(args.db_dir, ignore_errors=True)

    # Emit the versioned JSON contract (INV-007-B) with hardware/versions/date.
    write_json_report(
        args.json_output, args, results,
        train_vectors.shape[1], metric,
        detect_hardware(), detect_versions(),
    )

    # 7. Format and Print Results
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
    if args.batch_size and args.batch_size > 0:
        print(f"Ingest mode: chunked (--batch-size {args.batch_size}) — no hidden rebuild inside VantaDB Ingest timer.")
    else:
        print("Ingest mode: single put_batch_raw call (--batch-size 0) — VantaDB's full HNSW rebuild")
        print("  runs INSIDE the Ingest timer AND again in Index timer (double rebuild).")
        print("  Use default (--batch-size 999) for isolated measurements.")
    print("\nNote: pre-Jul-31-2026 numbers used --batch-size 0 (double rebuild) and are NOT directly")
    print("  comparable. See header comment for full methodology changelog.")

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

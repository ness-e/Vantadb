[2026-08-12T16:45:55Z WARN  lance::dataset::write::insert] No existing dataset at C:\Users\Eros\VantaDB Proyect\VantaDB\benchmarks\competitive_data\lance_db\vectors.lance, it will be created
[2026-08-12T16:45:57Z WARN  lance::dataset::write::insert] No existing dataset at C:\Users\Eros\VantaDB Proyect\VantaDB\benchmarks\competitive_data\lance_db\vectors.lance, it will be created
[2026-08-12T16:45:59Z WARN  lance::dataset::write::insert] No existing dataset at C:\Users\Eros\VantaDB Proyect\VantaDB\benchmarks\competitive_data\lance_db\vectors.lance, it will be created
Failed to send telemetry event ClientStartEvent: capture() takes 1 positional argument but 3 were given
Failed to send telemetry event ClientCreateCollectionEvent: capture() takes 1 positional argument but 3 were given
Failed to send telemetry event CollectionAddEvent: capture() takes 1 positional argument but 3 were given
Failed to send telemetry event CollectionQueryEvent: capture() takes 1 positional argument but 3 were given
Traceback (most recent call last):
  File "C:\Users\Eros\VantaDB Proyect\VantaDB\benchmarks\competitive_bench.py", line 952, in main
    res = bench_fn(path, train_vectors, test_vectors, ground_truth, metric, args.top_k, **kwargs)
          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  File "C:\Users\Eros\VantaDB Proyect\VantaDB\benchmarks\competitive_bench.py", line 450, in bench_chromadb
    shutil.rmtree(db_path)
  File "C:\Program Files\WindowsApps\PythonSoftwareFoundation.Python.3.11_3.11.2544.0_x64__qbz5n2kfra8p0\Lib\shutil.py", line 787, in rmtree
    return _rmtree_unsafe(path, onerror)
           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  File "C:\Program Files\WindowsApps\PythonSoftwareFoundation.Python.3.11_3.11.2544.0_x64__qbz5n2kfra8p0\Lib\shutil.py", line 629, in _rmtree_unsafe
    _rmtree_unsafe(fullname, onerror)
  File "C:\Program Files\WindowsApps\PythonSoftwareFoundation.Python.3.11_3.11.2544.0_x64__qbz5n2kfra8p0\Lib\shutil.py", line 634, in _rmtree_unsafe
    onerror(os.unlink, fullname, sys.exc_info())
  File "C:\Program Files\WindowsApps\PythonSoftwareFoundation.Python.3.11_3.11.2544.0_x64__qbz5n2kfra8p0\Lib\shutil.py", line 632, in _rmtree_unsafe
    os.unlink(fullname)
PermissionError: [WinError 32] El proceso no tiene acceso al archivo porque estß siendo utilizado por otro proceso: './benchmarks/competitive_data\\chroma_db\\8fc45657-6a8e-44cb-8903-41efdb4fca09\\data_level0.bin'
Traceback (most recent call last):
  File "C:\Users\Eros\VantaDB Proyect\VantaDB\benchmarks\competitive_bench.py", line 952, in main
    res = bench_fn(path, train_vectors, test_vectors, ground_truth, metric, args.top_k, **kwargs)
          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  File "C:\Users\Eros\VantaDB Proyect\VantaDB\benchmarks\competitive_bench.py", line 450, in bench_chromadb
    shutil.rmtree(db_path)
  File "C:\Program Files\WindowsApps\PythonSoftwareFoundation.Python.3.11_3.11.2544.0_x64__qbz5n2kfra8p0\Lib\shutil.py", line 787, in rmtree
    return _rmtree_unsafe(path, onerror)
           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  File "C:\Program Files\WindowsApps\PythonSoftwareFoundation.Python.3.11_3.11.2544.0_x64__qbz5n2kfra8p0\Lib\shutil.py", line 629, in _rmtree_unsafe
    _rmtree_unsafe(fullname, onerror)
  File "C:\Program Files\WindowsApps\PythonSoftwareFoundation.Python.3.11_3.11.2544.0_x64__qbz5n2kfra8p0\Lib\shutil.py", line 634, in _rmtree_unsafe
    onerror(os.unlink, fullname, sys.exc_info())
  File "C:\Program Files\WindowsApps\PythonSoftwareFoundation.Python.3.11_3.11.2544.0_x64__qbz5n2kfra8p0\Lib\shutil.py", line 632, in _rmtree_unsafe
    os.unlink(fullname)
PermissionError: [WinError 32] El proceso no tiene acceso al archivo porque estß siendo utilizado por otro proceso: './benchmarks/competitive_data\\chroma_db\\8fc45657-6a8e-44cb-8903-41efdb4fca09\\data_level0.bin'
C:\Users\Eros\VantaDB Proyect\VantaDB\benchmarks\competitive_bench.py:564: DeprecationWarning: `recreate_collection` method is deprecated and will be removed in the future. Use `collection_exists` to check collection existence and `create_collection` instead.
  client.recreate_collection(
============================================================
        VantaDB Competitive Benchmark Suite (T3.2)       
============================================================
Dataset      : synthetic
Dataset Size : 2000
Queries      : 50
Top-K        : 10
============================================================

--------------------------------------------------
System Health Check
--------------------------------------------------
  [!] Disk space: 5.6% free (<15% WARNING)
  [OK] RAM free: 11.6 GB
  [OK] RAYON_NUM_THREADS=4
  [!] CPU load: 65.8% (>30% WARNING ù benchmarks will be contaminated)
  [!] VS Code processes: 15 (>3 WARNING)
--------------------------------------------------
Dataset 'synthetic' not in standard ann-benchmarks list. Generating synthetic fallback...
Generating synthetic dataset (2000 vectors, 128d, metric=euclidean)...
Computing exact neighbors via brute-force numpy (top_k=100)...

Vectors shape: (2000, 128)
Queries shape: (50, 128)
Metric used  : euclidean

--- Running Vanta (3 iterations) ---

Benchmarking VantaDB...

Benchmarking VantaDB...

Benchmarking VantaDB...
  Vanta run 1: 447.4 QPS | run 2: 648.9 QPS | run 3: 635.6 QPS | Median: 635.6 QPS

--- Running Lance (3 iterations) ---

Benchmarking LanceDB...

Benchmarking LanceDB...

Benchmarking LanceDB...
  Lance run 1: 126.2 QPS | run 2: 118.0 QPS | run 3: 126.8 QPS | Median: 126.2 QPS

--- Running Chroma (3 iterations) ---

Benchmarking ChromaDB...

Benchmarking ChromaDB...
  ERROR: Failed to benchmark chroma (run 2): [WinError 32] El proceso no tiene acceso al archivo porque estß siendo utilizado por otro proceso: './benchmarks/competitive_data\\chroma_db\\8fc45657-6a8e-44cb-8903-41efdb4fca09\\data_level0.bin'

Benchmarking ChromaDB...
  ERROR: Failed to benchmark chroma (run 3): [WinError 32] El proceso no tiene acceso al archivo porque estß siendo utilizado por otro proceso: './benchmarks/competitive_data\\chroma_db\\8fc45657-6a8e-44cb-8903-41efdb4fca09\\data_level0.bin'
  Chroma run 1: 398.8 QPS | Median: 398.8 QPS

--- Running Qdrant (3 iterations) ---

Benchmarking Qdrant (embedded local mode, no docker)...

Benchmarking Qdrant (embedded local mode, no docker)...

Benchmarking Qdrant (embedded local mode, no docker)...
  Qdrant run 1: 490.9 QPS | run 2: 375.4 QPS | run 3: 519.2 QPS | Median: 490.9 QPS

Wrote versioned JSON contract: docs/benchmarks/competitive_sdk_bench.json

============================================================
                      BENCHMARK REPORT                      
============================================================
| Engine   |   Ingest QPS | Index Time (ms)   |   Query QPS |   Latency p50 (ms) |   Latency p99 (ms) | Recall@10   |   Peak RSS (MB) |   Delta RSS (MB) |
|----------|--------------|-------------------|-------------|--------------------|--------------------|-------------|-----------------|------------------|
| VantaDB  |        520.3 | 1695.7            |       635.6 |              1.51  |              2.655 | 59.20%      |           236.1 |             50.9 |
| LanceDB  |      50086.9 | 679.3             |       126.2 |              7.379 |             17.853 | 27.00%      |           233.8 |              3.1 |
| ChromaDB |       1511.7 | N/A (Inc)         |       398.8 |              2.242 |              5.769 | 97.60%      |           257.1 |             25.5 |
| Qdrant   |        129.5 | N/A (Inc)         |       490.9 |              1.855 |              4.377 | 100.00%     |           253.9 |             -0.6 |
============================================================
Ingest mode: chunked (--batch-size 999) ù no hidden rebuild inside VantaDB Ingest timer.

Note: pre-Jul-31-2026 numbers used --batch-size 0 (double rebuild) and are NOT directly
  comparable. See header comment for full methodology changelog.

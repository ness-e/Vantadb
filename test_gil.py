import time
import threading
import os
import shutil
import math
import sys

# Ensure vantadb_py is importable.
# We will add vantadb-python to the path if needed, or assume it's installed/built.
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "vantadb-python")))

try:
    import vantadb_py as vanta
except ImportError:
    # If not built yet, we can try to look in target/debug/build or similar, 
    # but let's assume vantadb_py can be imported if compiled.
    pass

DB_PATH = "./test_gil_db"

def cleanup():
    if os.path.exists(DB_PATH):
        try:
            shutil.rmtree(DB_PATH)
        except Exception:
            pass

def cpu_work(duration):
    """Perform CPU-bound work in Python."""
    start = time.time()
    count = 0
    while time.time() - start < duration:
        # Just simple math to consume CPU
        math.sin(count) * math.cos(count)
        count += 1
    return count

def run_gil_verification():
    cleanup()
    print("Initializing Database...")
    db = vanta.VantaDB(db_path=DB_PATH)
    
    # Ingest some records to make operations measurable
    print("Ingesting 1000 records...")
    for i in range(1000):
        db.put(
            "bench/gil",
            f"doc-{i}",
            f"payload data {i} " * 10,
            vector=[float(i % 10) / 10.0] * 128
        )
    db.flush()
    print("Ingestion complete.")

    # We will run a search query in a background thread.
    # The search query in Python is a linear scan over 1000 records of 128D, which takes some time.
    # We will do it 200 times in a loop to ensure it runs for about 1-2 seconds.
    db_running = True
    db_ops_count = 0
    
    def db_thread_worker():
        nonlocal db_ops_count, db_running
        # Release GIL when entering Rust
        # We will loop search queries
        while db_running:
            # This call goes into Rust and releases GIL via py.allow_threads()
            hits = db.search_memory("bench/gil", [0.5] * 128, top_k=10)
            db_ops_count += len(hits)

    # Start DB thread
    t = threading.Thread(target=db_thread_worker)
    
    # 1. Run CPU work alone first (Baseline)
    print("Running Python CPU work baseline (1.0 second)...")
    t0 = time.time()
    baseline_work = cpu_work(1.0)
    elapsed_baseline = time.time() - t0
    print(f"Baseline CPU work done: {baseline_work} iterations in {elapsed_baseline:.4f}s")
    
    # 2. Run CPU work concurrently with DB queries
    print("Starting background DB thread...")
    t.start()
    time.sleep(0.1) # Let the DB thread start query loop
    
    print("Running Python CPU work concurrently with DB thread (1.0 second)...")
    t1 = time.time()
    concurrent_work = cpu_work(1.0)
    elapsed_concurrent = time.time() - t1
    
    # Stop DB thread
    db_running = False
    t.join()
    db.close()
    cleanup()
    
    ratio = concurrent_work / baseline_work
    print(f"Concurrent CPU work done: {concurrent_work} iterations in {elapsed_concurrent:.4f}s")
    print(f"CPU work efficiency while DB is running: {ratio * 100:.2f}%")
    print(f"DB background thread completed {db_ops_count} search hits.")
    
    if ratio > 0.8:
        print("SUCCESS: GIL is released! CPU work efficiency is > 80% (concurrency is active).")
    else:
        print("FAILURE: GIL might be locked! CPU work efficiency is low.")

if __name__ == "__main__":
    if 'vanta' not in globals():
        print("VantaDB Python module not loaded. Please build the python library first.")
        sys.exit(1)
    run_gil_verification()

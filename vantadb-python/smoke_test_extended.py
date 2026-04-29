import vantadb_py as vanta
import os
import shutil

DB_PATH = "./test_vanta_db_extended"

def cleanup():
    if os.path.exists(DB_PATH):
        shutil.rmtree(DB_PATH)

def run_test():
    cleanup()
    print("--- SMOKE TEST: START ---")
    
    print(f"1. Opening database at {DB_PATH}")
    db = vanta.VantaDB(db_path=DB_PATH)
    
    print("2. Writing memory data")
    record = db.put(
        "smoke/main",
        "first",
        "verified",
        metadata={"category": "smoke", "version": 1},
        vector=[0.1, 0.2, 0.3],
    )
    print(f"   Put Result: {record}")
    
    print("3. Reading memory data")
    read_res = db.get_memory("smoke/main", "first")
    print(f"   Read Result: {read_res}")
    assert read_res is not None
    assert read_res["payload"] == "verified"

    print("4. Listing memory data")
    page = db.list_memory("smoke/main", filters={"category": "smoke"})
    print(f"   List Result: {page}")
    assert len(page["records"]) == 1

    print("5. Searching memory vectors")
    hits = db.search_memory("smoke/main", [0.1, 0.2, 0.3], top_k=1)
    print(f"   Search Result: {hits}")
    assert hits[0]["record"]["key"] == "first"

    print("6. Inspecting capabilities")
    caps = db.capabilities()
    print(f"   Capabilities: {caps}")
    assert caps["persistence"] is True

    print("7. Flushing and closing")
    db.flush()
    db.close()
    
    print("8. Reopening database")
    
    db2 = vanta.VantaDB(db_path=DB_PATH)
    print("9. Reading memory data again to verify persistence")
    read_res2 = db2.get_memory("smoke/main", "first")
    print(f"   Re-read Result: {read_res2}")
    assert read_res2 is not None
    assert read_res2["payload"] == "verified"
    db2.close()
    
    print("--- SMOKE TEST: PASSED ---")

if __name__ == "__main__":
    try:
        run_test()
    finally:
        cleanup()

"""
VantaDB - Persistent Memory for Local AI Agents
Demonstrates namespace-scoped storage, hybrid retrieval, and telemetry.
"""
import vantadb_py as vantadb
import os

DB_PATH = "./agent_memory_db"

def main():
    # Initialize embedded memory engine
    # Allocates a 256MB isolation limit for the embedded Rust engine database
    db = vantadb.VantaDB(DB_PATH, memory_limit_bytes=256_000_000)

    # Store agent memories with semantic vectors and structured metadata
    memories = [
        ("ctx-001", "User prefers concise technical answers.", {"type": "preference", "priority": 2}, [0.8, 0.1, 0.5]),
        ("ctx-002", "Project uses Rust for core and Python for SDK.", {"type": "architecture", "priority": 3}, [0.2, 0.9, 0.4]),
        ("ctx-003", "Latency must stay under 15ms for local RAG.", {"type": "constraint", "priority": 1}, [0.7, 0.3, 0.8]),
    ]

    print("📥 Storing agent memories into 'agent/session-1' namespace...")
    for key, payload, meta, vec in memories:
        db.put("agent/session-1", key, payload, metadata=meta, vector=vec)

    # Hybrid retrieval: semantic + lexical fusion
    query_vec = [0.75, 0.2, 0.6]
    print("\n🔍 Searching memories using Hybrid Search (Cosine Vector Similarity + BM25 Lexical text)...")
    hits = db.search_memory("agent/session-1", query_vector=query_vec, text_query="Rust Python SDK", top_k=3)
    
    print("\n🔍 Retrieved Context:")
    for hit in hits:
        record = hit['record']
        print(f"  [{record['key']}] score={hit['score']:.3f} | {record['payload']} (meta: {record['metadata']})")

    # Telemetry & safe shutdown
    metrics = db.operational_metrics()
    print(f"\n📊 Process RSS: {metrics['process_rss_bytes']/1024/1024:.2f} MB")
    print(f"📊 HNSW Nodes Count: {metrics['hnsw_nodes_count']}")
    print(f"📊 HNSW Logical Memory: {metrics['hnsw_logical_bytes']/1024/1024:.2f} MB")
    
    db.flush()
    db.close()

if __name__ == "__main__":
    main()
    # Cleanup demo data
    import shutil
    if os.path.exists(DB_PATH):
        shutil.rmtree(DB_PATH)
        print("\n🧹 Cleaned up temporary database directory.")

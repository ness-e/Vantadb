# VantaDB Pilot Onboarding Guide

Welcome to the **VantaDB Control Pilot Program**! This guide is designed to get you up and running with VantaDB in under 15 minutes. 

We will walk through installing the SDK, setting up a persistent memory system for a local AI agent using **Ollama**, and how to submit your telemetry and qualitative feedback to us.

---

## 🛠️ Step 1: Installation & Environment Setup

VantaDB is distributed as a pre-compiled Python wheel for **Linux**, **macOS** (both Apple Silicon and Intel), and **Windows**. You do *not* need Rust compiler toolchains installed on your local machine.

Inside your active virtual environment, install the SDK along with the Ollama client dependencies:

```bash
# 1. Install VantaDB Python SDK
pip install vantadb-py

# 2. Install dependencies for the local LLM integration
pip install ollama psutil
```

---

## 🐍 Step 2: Persistent Semantic Memory Integration with Ollama

This script demonstrates how to configure a durable memory loop for a local AI agent. It uses Ollama's `nomic-embed-text` model to generate embeddings and VantaDB to store and query them with full WAL durability.

### Prerequisites:
Make sure you have Ollama running locally, and you have pulled the required models:
```bash
ollama pull nomic-embed-text
ollama pull llama3
```

### Integration Script (`agent_memory_loop.py`):
Create a file named `agent_memory_loop.py` and run it:

```python
import os
import ollama
import vantadb_py

# 1. Initialize VantaDB
# This creates a local database directory. All mutations are durably written to a Write-Ahead Log (WAL).
DB_PATH = "./agent_durable_memory"
db = vantadb_py.VantaDB(DB_PATH, distance_metric="cosine")

NAMESPACE = "agent_memories"

def get_local_embedding(text: str) -> list[float]:
    """Generates a 768-dimensional dense vector using local Ollama model."""
    response = ollama.embeddings(model="nomic-embed-text", prompt=text)
    return response["embedding"]

def remember_interaction(key: str, topic: str, content: str):
    """Stores a conversational memory durably in the database."""
    print(f"\n[Writing Memory] Key: {key} | Topic: {topic}")
    vector = get_local_embedding(content)
    
    # Store key, payload, and vector. Under the hood, this writes to WAL & LSM engine.
    db.put(
        namespace=NAMESPACE,
        key=key,
        vector=vector,
        payload={
            "topic": topic,
            "text": content
        }
    )
    db.flush() # Force synchronization to physical disk

def query_agent_memory(query_text: str, top_k: int = 2):
    """Executes a hybrid search query (Vector similarity + BM25 Lexical Keyword matching)"""
    print(f"\n[Querying Memory] Query: '{query_text}'")
    query_vector = get_local_embedding(query_text)
    
    # VantaDB runs both searches natively and fuses rankings using Reciprocal Rank Fusion (RRF)
    results = db.search_memory(
        namespace=NAMESPACE,
        query_vector=query_vector,
        text_query=query_text, # BM25 keyword query
        top_k=top_k
    )
    
    return results

# =====================================================================
# Execution Demo
# =====================================================================
if __name__ == "__main__":
    # Ingest some developer context memories
    remember_interaction(
        key="mem_01",
        topic="Project Architecture",
        content="VantaDB uses memory-mapped (MMap) page layout files compacted via topological BFS traversal to minimize random disk reads during HNSW queries."
    )
    remember_interaction(
        key="mem_02",
        topic="Python Integration",
        content="The Python wrapper of VantaDB is written in PyO3 and releases the GIL (using allow_threads) during queries to prevent blocking concurrent python tasks."
    )
    remember_interaction(
        key="mem_03",
        topic="General SQLite",
        content="SQLite is standard for embedded tables, but lacks natively integrated cost-based hybrid planner optimizations for combined HNSW + BM25 searches."
    )

    # Force rebuild the indexes. This structures HNSW links on disk in a BFS layout page alignment.
    print("\n[Compacting Index] Rebuilding topological BFS HNSW layout...")
    db.rebuild_index()

    # Search: Look for something mentioning "PyO3" and "GIL"
    search_results = query_agent_memory("PyO3 GIL release", top_k=2)

    for i, res in enumerate(search_results):
        print(f"Rank {i+1} | Score: {res.score:.4f} | Key: {res.key}")
        print(f"  Topic: {res.payload['topic']}")
        print(f"  Content: {res.payload['text']}\n")

    # Close the database connection safely on exit
    db.close()
```

---

## 📋 Step 3: Pilot Feedback & Telemetry Questionnaire

After integrating VantaDB into your local agent workflow, please fill out this questionnaire and paste it back into your feedback issue or email it to us.

### 1. Developer Environment Specs
* **Operating System & Version:** (e.g., Windows 11 Home, macOS Sequoia M2, Ubuntu 24.04 LTS)
* **CPU Cores & Threads:** (e.g., 8-core CPU Intel i7)
* **System RAM:** (e.g., 16 GB DDR5)
* **Storage Type:** (e.g., NVMe SSD, SATA SSD, HDD)
* **Python Version:** (e.g., Python 3.11.4)

### 2. Operational Metrics & Latency
* **Ingestion Latency (Average per `put`):** (e.g., ~10 ms, ~100 ms)
* **Index Rebuild Time (for your memory scale):** (e.g., 5s for 1K records)
* **Search Latency (p50 & p95):** (e.g., p50: 8ms, p95: 15ms)
* **Peak Memory Overhead (RSS process increase):** (e.g., RAM increased by 80MB)

### 3. Qualitative Assessment
* **Installation Friction:** Did `pip install vantadb-py` work on the first try? Did you encounter compiler warnings?
* **Retrieval Quality:** Did the hybrid search (RRF) match your semantic intent accurately? Did exact keyword matches (BM25) return correctly?
* **Onboarding Speed:** Did this onboarding process take less than 15 minutes?
* **Doubt / Bug Reports:** (Please list any unexpected pánicos, file locks, or behavior).

---

Thank you for participating in the VantaDB pilot program. Your feedback helps us harden the "SQLite for AI Memory" for everyone.

---
title: Beta Pilot Program and Onboarding Guide
type: operations
status: active
tags: [vantadb, operations, pilot]
last_reviewed: 2026-07-01
aliases: []
---

# Beta Pilot Program and Onboarding Guide

This document details the outreach strategy, rapid onboarding process, and technical feedback form for the **VantaDB Pilot Program**.

---

## 🎯 1. Outreach Strategy and Target Communities

We are looking for 3 to 5 developers building **local-first AI agents** who experience memory durability issues (data loss with in-memory FAISS or Chroma) or compilation friction with C++ extensions.

| Channel | Community | Recruitment Purpose |
|---|---|---|
| **Reddit** | `r/LocalLLaMA` | Developers building local RAG systems and agents with Ollama. |
| **Reddit** | `r/rust` | Systems engineers interested in database performance and PyO3 bindings. |
| **Discord** | Ollama Server (`#projects`) | AI builders running local models on consumer hardware. |
| **Discord** | LlamaIndex / LangChain | Developers integrating local vector stores. |

---

## 🛠️ 2. Onboarding Guide and Quick Setup (Ollama)

This guide lets you integrate VantaDB as the semantic memory engine for an AI agent in under 15 minutes.

### Prerequisites
Make sure **Ollama** is running locally and download the required models:
```bash
ollama pull nomic-embed-text
ollama pull llama3
```

### Install Dependencies
```bash
pip install vantadb-py ollama psutil
```

### Integration Script (`agent_memory_loop.py`):
```python
import os
import ollama
import vantadb_py

# 1. Initialize local database
DB_PATH = "./agent_durable_memory"
db = vantadb_py.VantaDB(DB_PATH, distance_metric="cosine")
NAMESPACE = "agent_memories"

def get_local_embedding(text: str) -> list[float]:
    """Generates a 768-dimensional vector using the Ollama model."""
    response = ollama.embeddings(model="nomic-embed-text", prompt=text)
    return response["embedding"]

def remember_interaction(key: str, topic: str, content: str):
    """Persistently stores a conversational interaction."""
    print(f"\n[Writing to WAL] Key: {key} | Topic: {topic}")
    vector = get_local_embedding(content)
    
    db.put(
        namespace=NAMESPACE,
        key=key,
        vector=vector,
        payload={
            "topic": topic,
            "text": content
        }
    )
    db.flush() # Force physical persistence to disk (fsync)

def query_agent_memory(query_text: str, top_k: int = 2):
    """Executes a native hybrid search (Vector HNSW + Lexical BM25) with RRF fusion."""
    print(f"\n[Hybrid Search] Query: '{query_text}'")
    query_vector = get_local_embedding(query_text)
    
    results = db.search_memory(
        namespace=NAMESPACE,
        query_vector=query_vector,
        text_query=query_text,
        top_k=top_k
    )
    return results

if __name__ == "__main__":
    remember_interaction(
        key="mem_01",
        topic="Engine Architecture",
        content="VantaDB uses memory-mapped (MMap) page layout files compacted sequentially in BFS order to reduce page faults."
    )
    remember_interaction(
        key="mem_02",
        topic="Python GIL",
        content="VantaDB's Python wrapper (PyO3) releases the GIL using allow_threads during searches for true thread concurrency."
    )

    print("\n[Compaction] Rebuilding vector index with BFS layout...")
    db.rebuild_index()

    # Search using keywords and semantic similarity simultaneously
    search_results = query_agent_memory("PyO3 release GIL", top_k=2)

    for i, res in enumerate(search_results):
        print(f"Rank {i+1} | Score: {res.score:.4f} | Key: {res.key}")
        print(f"  Topic: {res.payload['topic']}")
        print(f"  Content: {res.payload['text']}\n")

    db.close()
```

---

## 📋 3. Pilot Feedback Questionnaire

Once integrated, please share this completed questionnaire:

1. **Development Environment:**
   - Operating System (e.g., Windows 11, macOS M2, Ubuntu):
   - CPU (e.g., 8-core Intel i7):
   - Storage type (e.g., NVMe SSD, SATA SSD):
2. **Performance Metrics:**
   - Average ingestion latency per `put` (ms):
   - Index rebuild time (`rebuild_index`):
   - Search latency (p50 and p95):
3. **Qualitative Questions:**
   - Did the Python wheel install on first try without compiler warnings?
   - Did the hybrid search with RRF cover your semantic and lexical search intent?
   - Did you encounter any bugs, file locking issues, or unusual memory consumption?

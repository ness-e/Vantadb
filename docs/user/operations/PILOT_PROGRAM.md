---
title: Beta Pilot Program and Onboarding Guide
type: operations
status: active
tags: [vantadb, operations, pilot, early-adopters, program]
last_reviewed: 2026-07-26
aliases: []
---

# Beta Pilot Program and Onboarding Guide

> **Version:** 2.0 — Formal Pilot Program
> **Status:** Active — Recruiting round 1 (3–5 participants)
> **Duration:** 4–8 weeks per cohort

This document defines the **VantaDB Formal Pilot Program** — the profile, commitments, timeline, and success criteria for early adopters, plus the technical onboarding guide and feedback loop.

---

## 📋 0. Program Overview

The VantaDB Pilot Program connects early adopter teams building **local-first AI agents** with direct access to the engineering team in exchange for structured feedback, benchmarks, and use-case validation.

| Attribute | Detail |
|---|---|
| Cohort size | 3–5 participants |
| Duration | 4–8 weeks (negotiable) |
| Commitment | ~2–4 hrs/week (feedback + calls) |
| Perks | Direct Slack/Discord channel with eng team, priority bug fixes, free tier upgrade |
| Goal | Validate memory durability, hybrid search, and PyO3 ergonomics |

---

## 🎯 1. Early Adopter Profile

### Ideal Candidate

We are looking for teams building **local-first AI agents** who experience one or more of:

- **Memory durability issues** — data loss with in-memory FAISS or Chroma after process restart
- **Compilation friction** — C++ extension build failures on Windows or ARM macOS
- **Latency constraints** — need sub-50ms hybrid search on consumer hardware
- **Local-first requirements** — cannot ship user data to cloud vector databases

### Must-Have

- Active project using embeddings for semantic search or RAG
- Willingness to share anonymized benchmark results
- Available for one 30-min kickoff call + one 30-min midpoint call

### Nice-to-Have

- Previously tried Chroma, FAISS, LanceDB, or Qdrant
- Running on Windows or ARM macOS (helps us validate cross-platform wheels)
- Building multi-user or multi-namespace agent memory

### Exclusion Criteria

- Production workloads requiring SLA guarantees (VantaDB is pre-1.0)
- Proprietary data that cannot be described in aggregate feedback

---

## 🤝 2. Commitments

### VantaDB Commits To

| Area | Commitment |
|---|---|
| **Support** | Direct Slack/Discord channel with 2–4 hr response target (business hours) |
| **Bug fixes** | Critical/blocking issues triaged within 48 hours |
| **Feature prioritization** | Pilot-reported pain points get bumped to P1 |
| **Transparency** | Known issues, roadmap changes, and breaking changes communicated proactively |
| **Recognition** | Early adopter listed on project README and website (opt-in) |
| **License** | Extended evaluation period — no license enforcement during pilot |

### Pilot Participant Commits To

| Area | Commitment |
|---|---|
| **Integration** | Integrate VantaDB into one real or representative project |
| **Feedback** | Weekly feedback form submission (≤ 15 min) |
| **Benchmarks** | Run provided benchmark suite at weeks 1, 4, and 8 |
| **Calls** | Attend kickoff call (30 min) + midpoint call (30 min) |
| **Confidentiality** | Keep pre-release features and performance data under NDA until public |
| **Exit report** | 1-page summary at end of pilot: what worked, what didn't, what's missing |

---

## 📅 3. Duration & Timeline

Pilot runs in fixed cohorts. Each cohort follows the same 8-week cadence.

| Week | Milestone | Deliverables |
|---|---|---|
| **Pre** | Screening & agreement signing | Signed pilot agreement, signed NDA |
| **Week 1** | Kickoff + onboarding | 30-min kickoff call, onboarding checklist complete, `hello_vantadb` running |
| **Week 2** | Integration deep-dive | Core integration functional, first feedback form |
| **Week 3** | Benchmark baseline | Run `vantadb-bench` suite, share results |
| **Week 4** | Midpoint review | 30-min call, feedback form, adjust priorities |
| **Week 5–7** | Iteration | Weekly feedback, ad-hoc support, feature validation |
| **Week 8** | Final benchmark + exit | Benchmark suite run #2, exit report, close-out call |

---

## 📊 4. KPIs & Success Metrics

### Quantitative KPIs

| Metric | Target | How Measured |
|---|---|---|
| **Retention** | ≥ 80% participants active through week 8 | Weekly check-in response rate |
| **NPS (Net Promoter Score)** | ≥ 30 at exit | Exit survey: "How likely to recommend VantaDB?" (0–10) |
| **Install success rate** | 100% on supported platforms | Wheel installs without compiler invocation |
| **Benchmark improvement** | ≥ 15% latency reduction between week 3 and week 8 | `vantadb-bench` suite runs |
| **Feedback submission rate** | ≥ 85% weekly forms submitted | Form tracking |

### Qualitative KPIs

| Signal | Assessment |
|---|---|
| **Feature requests** | Categorized by theme — track volume and urgency |
| **Pain points** | Count of P0/P1 bugs reported vs resolved per week |
| **Use-case fit** | Subject match between pilot profile and actual usage |
| **Exit sentiment** | Thematic analysis of exit report |

### Definition of Pilot Success

The pilot is considered successful when:

1. ≥ 80% of participants complete the 8-week program
2. Average NPS ≥ 30
3. No unresolved P0 bugs at exit
4. At least 3 participants express intent to continue using VantaDB post-pilot
5. At least 2 verifiable case studies or testimonials collected

---

## 📣 5. Outreach Strategy and Target Communities

We are looking for 3 to 5 developers building **local-first AI agents** who experience memory durability issues (data loss with in-memory FAISS or Chroma) or compilation friction with C++ extensions.

| Channel | Community | Recruitment Purpose |
|---|---|---|
| **Reddit** | `r/LocalLLaMA` | Developers building local RAG systems and agents with Ollama. |
| **Reddit** | `r/rust` | Systems engineers interested in database performance and PyO3 bindings. |
| **Discord** | Ollama Server (`#projects`) | AI builders running local models on consumer hardware. |
| **Discord** | LlamaIndex / LangChain | Developers integrating local vector stores. |
| **Twitter/X** | #localai #vectordb #rustlang | Organic reach + share pilot landing page |
| **Hacker News** | Show HN | Launch post + pilot call-to-action |

---

## 🛠️ 6. Onboarding Guide and Quick Setup (Ollama)

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

### Integration Script (`agent_memory_loop.py`)

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

## 📋 7. Pilot Feedback Questionnaire

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

---

## 📎 8. Related Documents

| Document | Description |
|---|---|
| `pilot-agreement-template.md` | Formal agreement + NDA template for pilot participants |
| `pilot-feedback-template.md` | Structured weekly feedback form |
| `pilot-onboarding-checklist.md` | Step-by-step onboarding checklist for early adopters |
| `../QUICKSTART.md` | Public quickstart guide |

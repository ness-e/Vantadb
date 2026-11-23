---
title: "VantaDB OEM & Enterprise One-Pager"
kind: one-pager
status: active
last_reviewed: "2026-11-07"
tags: [vantadb, oem, enterprise, gtm, compliance, gdpr, architecture]
---

# VantaDB: Embedded AI Agent Memory Engine
### High-Performance Local-First Persistent Memory & Hybrid Retrieval for Platform Builders

> **Target Audience:** Engineering Leads & Product Architects at Agent Platforms (Dify, LangGraph, OpenWebUI, CrewAI, AutoGen) and Regulated Enterprise Deployments (Fintech, Healthtech, LegalTech in EU & LATAM).

---

## 1. Executive Summary & Core Value Proposition

Modern multi-agent systems and LLM workflows face critical bottlenecks: Docker infrastructure bloat, escalating vector database cloud costs, high latency on memory read/write cycles, and acute compliance liability under GDPR Article 17 and the EU AI Act.

**VantaDB** solves this by providing a **zero-dependency, single-file embedded persistent memory engine** written in Rust. It combines BM25 lexical search, HNSW dense vector indexing, and Reciprocal Rank Fusion (RRF) directly inside your host process or sidecar, eliminating external microservice hops while ensuring ironclad auditability.

```
┌────────────────────────────────────────────────────────────────────────┐
│                          YOUR AGENT PLATFORM                           │
│              (Python / TypeScript / Go / Native Host Process)          │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │ Direct FFI / IPC (< 0.1 ms)
┌───────────────────────────────────▼────────────────────────────────────┐
│                       VantaDB EMBEDDED ENGINE                          │
│ ┌────────────────────────┐ ┌──────────────────────┐ ┌────────────────┐ │
│ │  FastGate Token Filter │ │   Hybrid Retrieval   │ │ Cryptographic  │ │
│ │   (85% Token Savings)  │ │ (BM25 + HNSW + RRF)  │ │   Audit Log    │ │
│ └────────────────────────┘ └──────────────────────┘ └────────────────┘ │
│ ┌────────────────────────────────────────────────────────────────────┐ │
│ │ StorageEngine (Append-Only WAL + AES-256-GCM + Merkle History)     │ │
│ └────────────────────────────────────────────────────────────────────┘ │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │ Single Data File
                                    ▼
                          agent_memory.vantadb
```

---

## 2. Technical Differentiation (Verified Benchmarks)

| Metric / Dimension | Cloud Vector DBs (Pinecone, Qdrant Cloud) | Self-Hosted Heavy (Postgres + pgvector / Redis) | **VantaDB Embedded Engine** |
| :--- | :--- | :--- | :--- |
| **Operational Architecture** | Remote SaaS (Network Round-Trip) | Multi-Container Docker / Helm Stack | **Zero External Calls (Single Binary / Library)** |
| **Memory Footprint (Idle)** | N/A (External billing) | 800 MB – 2.5 GB RAM per instance | **< 25 MB RAM idle** |
| **Write Latency (p50)** | 45.0 ms – 120.0 ms (WAN latency) | 12.0 ms – 25.0 ms | **0.50 ms – 2.10 ms (In-Process / Local NVMe)** |
| **Hybrid Search (p50)** | 65.0 ms – 180.0 ms | 18.0 ms – 45.0 ms | **2.70 ms – 8.50 ms (Dense + Sparse RRF)** |
| **Write Token Overhead** | 100% LLM extraction cost | 100% LLM extraction cost | **85% LLM Token Reduction (`FastGate` Rust pipeline)** |
| **GDPR "Right to be Forgotten"** | Soft delete / asynchronous tombstone | Manual SQL cascades (residues in logs) | **Cryptographic `PurgeCertificate` (Physical Entry Purge)** |
| **Audit & Integrity** | Proprietary telemetry | Custom triggers / audit tables | **Merkle-chained `audit.jsonl` + SHA-256 tamper evidence** |
| **Core Licensing** | Proprietary SaaS / Vendor lock-in | SSPL / AGPL / BSL (Copyleft risk) | **100% Apache-2.0 Permissive (Zero contamination)** |

---

## 3. Pillar Capabilities for Enterprise & OEM Platforms

### A. 85% LLM Token Reduction via `FastGate`
Conventional agent architectures invoke expensive LLMs (GPT-4o, Claude 3.5 Sonnet) on every conversation turn to extract and categorize memory candidates, generating massive API bills. VantaDB’s `FastGate` heuristic filter runs in Rust in **< 0.1 ms**, deterministically rejecting ephemeral chit-chat, boilerplate, and low-salience noise before the embedding or LLM pipeline is touched.

### B. Built-In Regulatory Compliance (GDPR Art. 17 & EU AI Act)
Regulators require physical proof when personal data is deleted.
- **Physical Purge Verification:** `client.forget(namespace, key, mode="certified")` scrubs memory across all internal surfaces (document store, vector index, BM25 text posting, JSON shredding, version trees) and issues a cryptographically signed `PurgeCertificate`.
- **Merkle-Chained Audit Trail:** Every write, update, and deletion is recorded in an append-only `audit.jsonl` secured by SHA-256 rolling block hashes for non-repudiation.
- **Data Sovereignty:** Memory files remain strictly local or on private customer infrastructure; zero telemetry or telemetry calls phone home.

### C. Multi-Session Isolation & Zero Context Leakage
Engineered for multi-tenant and multi-session agent runtimes. Rigorously validated under 1,000-turn continuous soak testing (`PRD-04`) with active preference supersession (`c.supersede`), partition scoping, and bitemporal valid-time queries (`context_leakage_rate == 0.00%`).

### D. Native MCP Integration
Ships with native Model Context Protocol (MCP) server daemon (`vantadb-server`), enabling instant plug-and-play memory capabilities in Cursor, Claude Desktop, Windsurf, and custom agentic frameworks with zero configuration.

---

## 4. OEM Partnership & Commercial Support

VantaDB Core is 100% open source under the **Apache-2.0** license. For platform teams integrating VantaDB into white-labeled products, multi-tenant agent platforms, or mission-critical enterprise environments, we offer dedicated engineering partnerships.

### The `/enterprise` Partnership Package
**Commercial Support, Custom Integration & On-Prem Hardening**
- **Starting at $750 / month** (scaling to **~$1,000 / month** based on platform scope, custom drivers, and SLA level).

#### Included in Commercial Tier:
1. **Architectural Co-Design & Tuning:** Direct Slack/Discord channel with VantaDB core engineers for custom driver development, memory pooling optimization, and memory compaction tuning.
2. **Custom Embeddings & Pre-Warmers:** Hardware-accelerated embedding integration (ONNX, DirectML, Apple Silicon Accelerate, TensorRT) tailored to your deployment matrix.
3. **Enterprise SLA & Urgent Hotfixes:** 24-hour response SLA on core bugs and guaranteed backports.
4. **Compliance Assurance Package:** Verification harness artifacts, SOC2/ISO-27001 readiness review documentation, and custom tamper-evident audit hooks.

---

## 5. Direct B2B Procurement & Payment Rails

To eliminate international banking friction, currency conversion fees, and bureaucratic delays, VantaDB provides verified, streamlined payment channels:

### A. B2B & OEM Commercial Contracts (Invoicing & Crypto Rails)
For corporate contracts, retainers, and OEM licensing agreements:
- **Primary B2B Rail:** **Binance Pay / USDT (Tether)**
  - Supported Networks: **USDT-TRC20**, **USDT-ERC20**, **USDT-BEP20**
  - Instant settlement, zero international wire hold times, and formal PDF corporate invoices generated with tax registration compliance.

### B. Developer Sponsorship & Recurring Support
For engineering teams, startups, and open-source advocates sponsoring active development:
- **Ko-fi Tier:** Recurring and milestone support via [Ko-fi VantaDB](https://ko-fi.com/vantadb).
- **Patreon Tier:** Tiered monthly dev-support via [Patreon VantaDB](https://patreon.com/vantadb).

*(Note: We do not process direct Stripe, GitHub Sponsors, or Polar transactions. All enterprise agreements are executed via Binance Pay USDT or direct corporate bank arrangements).*

---

## 6. Integration in 3 Steps

### 1. Install Library or Binary
```bash
# Python Engine
pip install vantadb-py

# Node.js / TypeScript SDK
npm install vantadb

# Standalone CLI / MCP Daemon
# Windows:
winget install ness-e.VantaDB
# macOS / Linux:
brew install ness-e/tap/vantadb
```

### 2. Initialize In-Process Memory Client
```python
import vantadb

# Single-file persistent embedded database
client = vantadb.Client("./data/agent_memory.vantadb")

# Ingest episodic memory with deterministic FastGate filtering
record = client.put(
    namespace="agent_production",
    key="decision_0942",
    payload="User requested dark mode and strict data residency in Frankfurt.",
    metadata={"session_id": "sess_812", "topic": "preferences"},
    vector=embedding_vector_384d,
)
```

### 3. Query with Hybrid Search & Compliance
```python
# Hybrid Search (BM25 + Dense HNSW + RRF) with Session Scoping
hits = client.search(
    namespace="agent_production",
    query_vector=query_vector_384d,
    text_query="data residency Frankfurt",
    filters={"session_id": "sess_812"},
    top_k=5,
    exclude_superseded=True,
)

# Certified GDPR Article 17 Purge
cert = client.forget("agent_production", "decision_0942", mode="certified")
print(f"Purge Hash: {cert['certificates'][0]}")
```

---

## 7. Contact & OEM Evaluation Access

- **Technical Inquiries & Pilot Evaluations:** `engineering@vantadb.dev` / `oem@vantadb.dev`
- **GitHub Repository:** [https://github.com/ness-e/Vantadb](https://github.com/ness-e/Vantadb)
- **Documentation & Architecture Specs:** [https://vantadb.dev](https://vantadb.dev)
- **Community & Engineering Office Hours:** [Discord Community](https://discord.gg/g8nqB3NtXt)

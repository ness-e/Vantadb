# Pilot Outreach Strategy & Templates

This document details the target communities and communication templates used to recruit the 3–5 early adopters/pilot users for the **VantaDB Control Pilot Program (T3.4)**.

---

## 🎯 Target Communities

We will target communities where developers are actively building **local-first AI agents** and frequently encounter issues with memory persistence, FFI compilation, or database overhead.

| Platform | Community / Channel | Reason for Targeting |
|---|---|---|
| **Reddit** | `r/LocalLLaMA` | The primary hub for edge LLMs, Ollama users, and local-first AI development. |
| **Reddit** | `r/rust` | Capturing systems engineers interested in database performance and PyO3 bindings. |
| **Discord** | Ollama Server (`#projects` / `#dev`) | High concentration of builders deploying models on consumer hardware. |
| **Discord** | LlamaIndex / LangChain Servers | Developers trying to configure local vector stores and encountering FFI issues. |
| **HackerNews** | Show HN (launch day) | Technical audience looking for lightweight embedded Rust infrastructure. |

---

## ✉️ Outreach Templates

### Template 1: For Local LLM & AI Developer Forums (English)

**Subject:** Looking for pilot testers: VantaDB – Embedded, persistent memory & hybrid search in Rust (GIL-free Python SDK)

Hi everyone,

I’m looking for 3–5 early adopters who are actively building **local-first AI agents** (using Ollama, LlamaIndex, LangChain, etc.) to try out a lightweight, embedded memory engine we’ve been building called **VantaDB**.

If you've built agents that run locally, you've probably faced the trade-offs:
1. Spawning a cloud vector database container (adds network latency and violates offline-first privacy).
2. Using FAISS or Chroma in-memory (lacks persistent WAL durability and causes data loss on crash).
3. Compiling C++ SQLite vector extensions (heavy FFI boundary costs and difficult cross-compilation toolchains).

We built VantaDB to solve this as a pure-Rust embedded storage engine (using an LSM-tree WAL backend) mapped to HNSW and BM25 indexes. 

#### What we are validating in this pilot:
* **Embedded Stability:** Memory footprint (RSS) stability on edge devices (NUCs, Raspberry Pi 5, standard laptops) under high concurrency.
* **Onboarding Friction:** Can you get it integrated into your Ollama or LlamaIndex agent workflow in under 15 minutes?
* **Performance Parity:** How does our Volcano-style query planner and RRF hybrid retrieval scale on your local datasets?

If you are interested in testing it and giving us raw feedback, check out our [Pilot Onboarding Guide](file:///C:/Users/Eros/VantaDB%20Proyect/VantaDB/docs/operations/PILOT_ONBOARDING.md) and reply here or DM me. I'd love to help you wire it up!

---

### Template 2: For Systems & Rust Developer Groups (Spanish)

**Título:** Programa de Pilotos: VantaDB – Motor embebido de memoria persistente para agentes de IA (Rust + PyO3)

Hola a todos,

Estamos buscando de 3 a 5 desarrolladores que estén construyendo **agentes de IA locales** (usando Ollama, Llama.cpp, etc.) para probar el comportamiento de **VantaDB** como su base de datos embebida de persistencia.

VantaDB es un motor escrito 100% en Rust diseñado específicamente para memoria híbrida local. Implementa:
* Almacenamiento síncrono duradero mediante un motor LSM-tree (Fjall) con WAL y CRC32C.
* Travesía HNSW sobre archivos mapeados en memoria (`memmap2`) optimizada mediante un layout de compactación BFS topológica para reducir page faults.
* Búsqueda híbrida (BM25 FTS + HNSW) unificada mediante un optimizador por costo (CBO) y un motor Volcano.
* Wrappers en Python (PyO3) seguros con paralelismo multinúcleo con `Rayon` liberando el GIL.

Buscamos pilotos para validar la robustez ante fallos de disco simulados, fugas de memoria RSS en ejecuciones largas y la facilidad de distribución de las ruedas compiladas de Python en sus plataformas de desarrollo.

Si te interesa probarlo y darnos feedback técnico directo sobre la arquitectura, puedes arrancar con nuestra [Guía de Onboarding para Pilotos](file:///C:/Users/Eros/VantaDB%20Proyect/VantaDB/docs/operations/PILOT_ONBOARDING.md). ¡Estaremos encantados de ayudarte con la integración!

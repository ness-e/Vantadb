---
title: RAG — Retrieval-Augmented Generation
kind: glossary
status: stable
description: "##Definition"
aliases: [Retrieval-Augmented Generation]
tags: [concept, producto, rag, ia, retrieval]
links: "[[README.md]]"
---

# RAG — Retrieval-Augmented Generation

##Definition

**RAG** ​​(Retrieval-Augmented Generation) is an architectural pattern that combines an **information retrieval system** (retrieval) with a **generative language model** (LLM) to produce responses informed by domain-specific data, reducing hallucinations and improving factual accuracy.

## How It Works

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│   Consulta   │────▶│  Retrieval   │────▶│  Contexto    │
│   del Usuario│     │  (Search)    │     │  Recuperado  │
└──────────────┘     └──────────────┘     └──────┬───────┘
                                                  │
                                                  ▼
                                           ┌──────────────┐
                                           │     LLM      │
                                           │  (Generate)  │
                                           └──────┬───────┘
                                                  │
                                                  ▼
                                           ┌──────────────┐
                                           │  Respuesta   │
                                           │  Fundamentada│
                                           └──────────────┘
```

### RAG Pipeline Phases

1. **Indexing (Offline):** Documents are divided into chunks, vector embeddings are generated and stored in a vector database.
2. **Retrieval (Online):** Given a query, the most relevant chunks are searched using vector similarity, lexical-search or both.
3. **Generation (Online):** The recovered context is injected into the LLM prompt along with the original question.

##Why it Matters in VantaDB

VantaDB is designed as **the persistence and retrieval layer for RAG pipelines**:

- **Persistent memory** for agents that need to remember context between sessions
- **hybrid-search** ([hnsw](./hnsw.md) + [bm25](./bm25.md) + [rrf](./rrf.md)) to retrieve both semantics and exact keywords
- **[graph](./graph.md) of knowledge** for multi-hop traversal (GraphRAG), reducing tokens in the prompt between 40-60%
- **[transactional](./transactional.md)**: ensures that documents, embeddings and relationships are updated atomically

## Problems that RAG Solves

| Problema del LLM Puro | Solución con RAG |
|----------------------|-----------------|
| Alucinaciones | Fundamenta respuestas en datos reales |
| Conocimiento desactualizado | Recupera información actualizada del índice |
| Ventana de contexto limitada | Solo inyecta los chunks más relevantes |
| Sin acceso a datos privados | Permite consultar bases de conocimiento internas |

## RAG variants

### Naïve RAG
- Vector search → Top-K → Inject at prompt
- **Limitation:** Does not capture relationships between concepts

###Advanced RAG
- Query rewriting + reranking + hybrid search
- **VantaDB implements:** [rrf](./rrf.md) for ranking fusion

### GraphRAG
- Build a knowledge graph from documents
- Recovers contextual subgraphs instead of isolated chunks
- **Advantage:** Reduces prompt tokens by 40-60% vs Naïve RAG
- **VantaDB:** Supports native GraphRAG with edge traversal

## Key Metrics in RAG

| Métrica | Descripción | Objetivo |
|---------|-------------|----------|
| **Recall@K** | % de documentos relevantes recuperados en top-K | ≥ 0.95 |
| **Latencia p50** | Tiempo de recuperación | < 20ms |
| **Token Reduction** | Reducción vs inyectar todo el corpus | 40-60% |
| **Faithfulness** | Respuesta del LLM se alinea con contexto recuperado | Alta |

## Related Tools

| Herramienta | Rol | Relación con VantaDB |
|-------------|-----|---------------------|
| **LangChain** | Orquestación de pipelines RAG | Integración vía adapter (FEAT-01) |
| **LlamaIndex** | Framework de indexación y retrieval | Integración vía adapter (FEAT-01) |
| **Ollama** | Inferencia LLM local | Sidecar opcional, no acoplado |
| **OpenAI** | LLM API | Consumidor del contexto recuperado |

## See Also

- [vectors](./vectors.md) — Representations that feed the retrieval
- [hnsw](./hnsw.md) — Vector index for ANN search
- [bm25](./bm25.md) — Lexical index for keyword search
- [rrf](./rrf.md) — Hybrid Ranking Merger
- [graph](./graph.md) — For GraphRAG
- [transactional](./transactional.md) — Document-embedding consistency guarantee

---

*Fundamental concept for VantaDB's primary use case.*


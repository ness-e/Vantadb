---
type: glosario-entry
status: stable
tags: [concepto, producto, rag, ia, retrieval]
last_refined: 2026-06
links: "[Glosario](../Glosario.md)"
aliases: [Retrieval-Augmented Generation, Generación Aumentada por Recuperación]
description: "Patrón arquitectónico que combina un sistema de recuperación de información con un modelo de lenguaje generativo (LLM) para producir respuestas fundamentadas en datos específicos del dominio"
---

# RAG — Retrieval-Augmented Generation

## Definición

**RAG** (Retrieval-Augmented Generation) es un patrón arquitectónico que combina un **sistema de recuperación de información** (retrieval) con un **modelo de lenguaje generativo** (LLM) para producir respuestas fundamentadas en datos específicos del dominio, reduciendo alucinaciones y mejorando la precisión factual.

## Cómo Funciona

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

### Fases del Pipeline RAG

1. **Indexación (Offline):** Los documentos se dividen en chunks, se generan embeddings vectoriales y se almacenan en una base de datos vectorial.
2. **Recuperación (Online):** Dada una consulta, se buscan los chunks más relevantes usando similitud vectorial, búsqueda léxica o ambos.
3. **Generación (Online):** El contexto recuperado se inyecta en el prompt del LLM junto con la pregunta original.

## Por Qué Importa en VantaDB

VantaDB está diseñado como **la capa de persistencia y retrieval para pipelines RAG**:

- **Memoria persistente** para agentes que necesitan recordar contexto entre sesiones
- **Búsqueda híbrida** ([HNSW](HNSW.md) + [BM25](BM25.md) + [RRF](RRF.md)) para recuperar tanto por semántica como por keywords exactos
- **[Grafo](Grafo.md) de conocimiento** para traversal multi-hop (GraphRAG), reduciendo tokens en el prompt entre 40-60%
- **[Transaccional](Transaccional.md)**: garantiza que documentos, embeddings y relaciones se actualicen atómicamente

## Problemas que RAG Resuelve

| Problema del LLM Puro | Solución con RAG |
|----------------------|-----------------|
| Alucinaciones | Fundamenta respuestas en datos reales |
| Conocimiento desactualizado | Recupera información actualizada del índice |
| Ventana de contexto limitada | Solo inyecta los chunks más relevantes |
| Sin acceso a datos privados | Permite consultar bases de conocimiento internas |

## Variantes de RAG

### Naïve RAG
- Vector search → Top-K → Injectar en prompt
- **Limitación:** No captura relaciones entre conceptos

### Advanced RAG
- Query rewriting + reranking + hybrid search
- **VantaDB implementa:** [RRF](RRF.md) para fusión de rankings

### GraphRAG
- Construye un grafo de conocimiento a partir de documentos
- Recupera subgrafos contextuales en lugar de chunks aislados
- **Ventaja:** Reduce tokens del prompt en 40-60% vs Naïve RAG
- **VantaDB:** Soporta GraphRAG nativo con traversal de aristas

## Métricas Clave en RAG

| Métrica | Descripción | Objetivo |
|---------|-------------|----------|
| **Recall@K** | % de documentos relevantes recuperados en top-K | ≥ 0.95 |
| **Latencia p50** | Tiempo de recuperación | < 20ms |
| **Token Reduction** | Reducción vs inyectar todo el corpus | 40-60% |
| **Faithfulness** | Respuesta del LLM se alinea con contexto recuperado | Alta |

## Herramientas Relacionadas

| Herramienta | Rol | Relación con VantaDB |
|-------------|-----|---------------------|
| **LangChain** | Orquestación de pipelines RAG | Integración vía adapter (FEAT-01) |
| **LlamaIndex** | Framework de indexación y retrieval | Integración vía adapter (FEAT-01) |
| **Ollama** | Inferencia LLM local | Sidecar opcional, no acoplado |
| **OpenAI** | LLM API | Consumidor del contexto recuperado |

## Véase También

- [Vectores](Vectores.md) — Representaciones que alimentan el retrieval
- [HNSW](HNSW.md) — Índice vectorial para búsqueda ANN
- [BM25](BM25.md) — Índice léxico para keyword search
- [RRF](RRF.md) — Fusión de rankings híbridos
- [Grafo](Grafo.md) — Para GraphRAG
- [Transaccional](Transaccional.md) — Garantía de consistencia documento-embedding

---

*Concepto fundamental para el caso de uso primario de VantaDB.*


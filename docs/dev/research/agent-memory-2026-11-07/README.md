---
title: "README — Investigación comparativa agent-memory (2026-11-07)"
kind: index
status: proposed
description: "Indice navegable de la evaluacion estatica de 37 repositorios de memoria/contexto para agentes mas OpenViking"
tags: [research, memory, index]
---

# README — Investigación comparativa agent-memory (2026-11-07)

> **Qué es:** evaluación **estática** de repositorios públicos de memoria, contexto, registro y almacenamiento para agentes, orientada a ideas adaptables a VantaDB. Sin ejecución de código, tests, benchmarks ni servicios — todo es lectura con fecha de corte UTC.
> **No es:** un censo exhaustivo de GitHub, ni un ranking de calidad. Las estrellas fueron un **filtro de selección solicitado** (`>=30000`), no evidencia de calidad, licencia OSI ni seguridad.

## Documentos

| Documento | Contenido |
|---|---|
| [CONSOLIDATED.md](CONSOLIDATED.md) | Síntesis de diez puntos: recomendaciones, candidatos unificados, prerequisitos, rechazados, advertencias, benchmarks, brechas, deuda propia, UI |
| [METHOD.md](METHOD.md) | Protocolo de evaluación: contrato de evidencia, selección, preparación reproducible, referencia VantaDB |
| [BENCHMARK-NOTES.md](BENCHMARK-NOTES.md) | Notas metodológicas de benchmarks (definiciones de métricas, claims vs ejecución) |
| [github-candidates.json](github-candidates.json) | Snapshot UTC 2026-11-07 de los 50 candidatos (stars, licencia detectada) — valores congelados |
| [clone-manifest.json](clone-manifest.json) | Manifiesto de clones: commit SHA, shallow, rama, omisiones por repo |
| [../../Backlog-OpenViking-VantaDB.md](../../Backlog-OpenViking-VantaDB.md) | Informe central de OpenViking (`b8e8895`): los diez puntos en profundidad, fichas F1–F10, seguridad, licencias |

## Inventario (37 elegidos + OpenViking ya estudiado)

Datos `stars`/`licencia`/`SHA` **copiados** de los JSON congelados (snapshot 2026-11-07 UTC; `NOASSERTION` = licencia no determinada por la API, no ausencia ni permiso). Familia = clasificación funcional propia de esta evaluación, no del repo. OpenViking enlaza al informe central; su SHA es el snapshot inspeccionado `b8e8895`.

| Repositorio | Familia | Ficha | Stars | SHA | Licencia |
|---|---|---|---:|---|---|
| agno-agi/agno | framework agentes | [agno.md](agno.md) | 43089 | `f45ccee83d` | Apache-2.0 |
| colbymchenry/codegraph | code-memory / KG | [codegraph.md](codegraph.md) | 76956 | `f72c9d2efb` | MIT |
| crewAIInc/crewAI | framework agentes | [crewai.md](crewai.md) | 60593 | `73f48320db` | MIT |
| DeusData/codebase-memory-mcp | code-memory / KG | [codebase-memory-mcp.md](codebase-memory-mcp.md) | 48254 | `612eee2518` | MIT |
| Egonex-AI/Understand-Anything | code-memory / KG | [understand-anything.md](understand-anything.md) | 91147 | `a6ea402476` | MIT |
| facebookresearch/faiss | vector index | [faiss.md](faiss.md) | 41261 | `277b3c76b7` | MIT |
| FlowiseAI/Flowise | plataforma visual | [flowise.md](flowise.md) | 55562 | `9291856d1e` | NOASSERTION |
| getzep/graphiti | memoria de agente | [graphiti.md](graphiti.md) | 32614 | `747b480ec6` | Apache-2.0 |
| Graphify-Labs/graphify | code-memory / KG | [graphify.md](graphify.md) | 143653 | `16218bfc80` | Apache-2.0 |
| HKUDS/LightRAG | RAG / grafo | [light-rag.md](light-rag.md) | 40591 | `aaebfe1bb8` | MIT |
| infiniflow/ragflow | RAG / plataforma | [ragflow.md](ragflow.md) | 93705 | `3c6d2255af` | Apache-2.0 |
| khoj-ai/khoj | app / asistente | [khoj.md](khoj.md) | 38033 | `ae229ca894` | AGPL-3.0 |
| langchain-ai/langchain | framework agentes | [langchain.md](langchain.md) | 149142 | `d9caf08357` | MIT |
| langchain-ai/langgraph | framework agentes | [langgraph.md](langgraph.md) | 44598 | `e5e37b5d70` | MIT |
| langflow-ai/langflow | plataforma visual | [langflow.md](langflow.md) | 156397 | `60999cc9a7` | MIT |
| langgenius/dify | plataforma LLM | [dify.md](dify.md) | 160588 | `65b4e17d6e` | NOASSERTION |
| meilisearch/meilisearch | search engine | [meilisearch.md](meilisearch.md) | 59806 | `c18c81973a` | NOASSERTION |
| mem0ai/mem0 | memoria de agente | [mem0.md](mem0.md) | 68542 | `f1791daa86` | Apache-2.0 |
| MemPalace/mempalace | memoria de agente | [mempalace.md](mempalace.md) | 60446 | `f8e3536218` | MIT |
| microsoft/autogen | framework agentes | [autogen.md](autogen.md) | 61738 | `d07c667a32` | CC-BY-4.0 |
| microsoft/graphrag | RAG / grafo | [graphrag.md](graphrag.md) | 36585 | `34b2b67ae4` | MIT |
| milvus-io/milvus | vector DB | [milvus.md](milvus.md) | 46612 | `eba8be50f8` | Apache-2.0 |
| Mintplex-Labs/anything-llm | app / plataforma | [anything-llm.md](anything-llm.md) | 68021 | `308b27662b` | MIT |
| openai/openai-agents-python | framework agentes | [openai-agents-python.md](openai-agents-python.md) | 30539 | `27c75c5f9c` | MIT |
| pathwaycom/serviette | RAG | [serviette.md](serviette.md) | 58725 | `84e92a3372` | MIT |
| pingcap/tidb | DB / vector | [tidb.md](tidb.md) | 40715 | `41efd866c3` | Apache-2.0 |
| qdrant/qdrant | vector DB | [qdrant.md](qdrant.md) | 35458 | `222b2e44ab` | Apache-2.0 |
| redis/redis | DB / vector | [redis.md](redis.md) | 77268 | `5b69ba2ce9` | NOASSERTION |
| rohitg00/agentmemory | memoria de agente | [agentmemory.md](agentmemory.md) | 30183 | `4f758eff36` | Apache-2.0 |
| run-llama/llama_index | framework RAG | [llama-index.md](llama-index.md) | 52837 | `444fdf0684` | MIT |
| supermemoryai/supermemory | memoria de agente | [supermemory.md](supermemory.md) | 31559 | `16ed414e0e` | MIT |
| surrealdb/surrealdb | DB multi-modelo | [surrealdb.md](surrealdb.md) | 33289 | `9577ac5f0c` | NOASSERTION |
| TencentCloud/TencentDB-Agent-Memory | memoria de agente | [tencentdb-agent-memory.md](tencentdb-agent-memory.md) | 30589 | `2002196f2d` | NOASSERTION |
| The-Vibe-Company/quivr | app RAG | [quivr.md](quivr.md) | 39633 | `bb1b19ecf4` | MIT |
| topoteretes/cognee | memoria de agente | [cognee.md](cognee.md) | 33118 | `8853378eae` | Apache-2.0 |
| VectifyAI/PageIndex | RAG / índice | [pageindex.md](pageindex.md) | 40160 | `2ac8694bf8` | MIT |
| vectorize-io/hindsight | memoria de agente | [hindsight.md](hindsight.md) | 59509 | `eb963cb5a9` | MIT |
| volcengine/OpenViking | context DB agentes | [Backlog-OpenViking-VantaDB.md](../../Backlog-OpenViking-VantaDB.md) | 41718 | `b8e88959f3` | AGPL-3.0 |

## Excluidos por umbral de estrellas (12, conservados en el inventario)

| Repositorio | Stars | Licencia |
|---|---:|---|
| basicmachines-co/basic-memory | 4516 | AGPL-3.0 |
| chroma-core/chroma | 29651 | Apache-2.0 |
| HKUDS/RAG-Anything | 24452 | MIT |
| lancedb/lancedb | 11855 | Apache-2.0 |
| letta-ai/letta | 25373 | Apache-2.0 |
| MemoriLabs/Memori | 17452 | NOASSERTION |
| MemTensor/MemOS | 12307 | Apache-2.0 |
| microsoft/semantic-kernel | 28742 | MIT |
| NevaMind-AI/memU | 14607 | NOASSERTION |
| pgvector/pgvector | 23608 | NOASSERTION |
| plastic-labs/honcho | 7994 | AGPL-3.0 |
| weaviate/weaviate | 16941 | NOASSERTION |

## Notas de integridad

- OpenViking (`volcengine/OpenViking`, 41718 stars, AGPL-3.0) ya estaba clonado y estudiado: figura en el inventario con su ficha central fuera de este directorio.
- `FlowiseAI/Flowise` está **archivado** upstream (estado de mantenimiento declarado en su ficha).
- Licencias `NOASSERTION` / source-available se registran por proyecto; **no** todos son OSI-validated.
- Las 37 fichas están completas (22 iniciales + 15 de la segunda tanda). `.gitignore` cubre `docs/dev/research/` y se **preserva** para el resto; los 43 archivos de esta investigación fueron incorporados selectivamente con force-add (revisión del propietario).

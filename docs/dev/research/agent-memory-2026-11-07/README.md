---
title: "README — Investigación comparativa agent-memory (2026-11-07)"
kind: index
status: proposed
description: "Indice de 42 informes ampliados y revisados, con limites documentales y OpenViking como referencia"
tags: [research, memory, index]
---

# README — Investigación comparativa agent-memory (2026-11-07)

> **Qué es:** evaluación **estática** de repositorios públicos de memoria, contexto, registro y almacenamiento para agentes, orientada a ideas adaptables a VantaDB. Sin ejecución de código, tests, benchmarks ni servicios — todo es lectura con fecha de corte UTC.
> **No es:** un censo exhaustivo de GitHub, ni un ranking de calidad. Las estrellas fueron un **filtro de selección solicitado** (`>=30000`), no evidencia de calidad, licencia OSI ni seguridad.
> **Estado de ampliación (2026-11-07):** las 37 fichas iniciales fueron dictaminadas **insuficientes en profundidad** por el propietario (tener diez encabezados no replica el estudio OpenViking). Se rehicieron las 37 y se agregaron 5 por excepción expresa → **42 informes con autoría y revisión independiente** según [EXPANSION-CONTRACT.md](EXPANSION-CONTRACT.md): el lead acepta documentalmente **41 con límites** y **1 limitado** (Meilisearch, E3 incidencias no acreditado por prohibición de forge del usuario — no equivale a cumplimiento de diez ejes). Estado y cierre en [EXPANSION-STATUS.md](EXPANSION-STATUS.md), cobertura por proyecto en [REVIEW-COVERAGE.md](REVIEW-COVERAGE.md) y referencia [VANTA-REFERENCE.md](VANTA-REFERENCE.md). Composición: 37 elegidos por umbral (`>=30000`) + **5 excepciones del propietario** + OpenViking como referencia ya estudiada. Aceptación documental ≠ features listas como spec ni implementación aprobada.

## Documentos

- [Revisión independiente de la síntesis](reviews/consolidation-closure.md).
- [Registro de avance de esta investigación](../../avance/investigaciones.md#agent-memory-2026-11-07-evaluación-comparativa-de-memoriacontexto-para-agentes).

| Documento | Contenido |
|---|---|
| [CONSOLIDATED.md](CONSOLIDATED.md) | Síntesis de diez puntos: recomendaciones, candidatos unificados, prerequisitos, rechazados, advertencias, benchmarks, brechas, deuda propia, UI |
| [METHOD.md](METHOD.md) | Protocolo de evaluación: contrato de evidencia, selección, preparación reproducible, referencia VantaDB |
| [EXPANSION-CONTRACT.md](EXPANSION-CONTRACT.md) | Contrato de profundidad, evidencia y revisión de los 42 informes ampliados |
| [EXPANSION-STATUS.md](EXPANSION-STATUS.md) | Estado operativo y asignaciones durables de la reanudación |
| [REVIEW-COVERAGE.md](REVIEW-COVERAGE.md) | Cobertura por proyecto: autoría, revisión asignada, correcciones y aceptación |
| [VANTA-REFERENCE.md](VANTA-REFERENCE.md) | Referencia local de VantaDB para la comparación |
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

## Adiciones por excepción del propietario (5)

Cinco proyectos **por debajo del umbral de 30k** añadidos expresamente — no los presenta como superados del filtro. Datos `stars`/`SHA`/`licencia` **copiados** de [github-additions.json](github-additions.json) y [clone-additions-manifest.json](clone-additions-manifest.json) (captura nueva congelada 2026-11-07 UTC; respuestas RAW en `evidence/additions/*.json`). Las cinco fichas existen y están revisadas; su aceptación con límites se registra en [EXPANSION-STATUS.md](EXPANSION-STATUS.md) y [REVIEW-COVERAGE.md](REVIEW-COVERAGE.md).

| Repositorio | Familia | Ficha | Stars | SHA | Licencia |
|---|---|---|---:|---|---|
| weaviate/weaviate | vector DB | [weaviate.md](weaviate.md) | 16942 | `85bd085e6f` | NOASSERTION |
| pgvector/pgvector | vector index | [pgvector.md](pgvector.md) | 23612 | `724588a550` | NOASSERTION |
| letta-ai/letta | memoria de agente | [letta.md](letta.md) | 25377 | `8816d3a851` | Apache-2.0 |
| lancedb/lancedb | vector DB | [lancedb.md](lancedb.md) | 11857 | `bff93ff314` | Apache-2.0 |
| chroma-core/chroma | vector DB | [chroma.md](chroma.md) | 29650 | `1a3ac2c7c6` | Apache-2.0 |

## Excluidos por umbral de estrellas (12, lista histórica conservada)

> Cinco de estos repos (weaviate, pgvector, letta, lancedb, chroma) fueron **agregados expresamente por el propietario** como excepciones posteriores — esta lista histórica NO se reduce retroactivamente y sus estrellas no se re-evalúan.

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

- **Excepción Meilisearch:** el usuario prohíbe acceso a forge (issues/PR/discussions); E3 incidencias queda **no acreditado** — sin lectura de cuerpos ni fallbacks, y no se equipara a cumplimiento de los diez ejes.
- **Cierre selectivo:** la investigación es estática y selectiva — no certifica exhaustividad, inexistencia global, comportamiento en runtime, autorización, recuperación ante crash ni rendimiento.
- **Validación documental offline final:** auditorías automatizadas acotadas sin hallazgos; enlaces internos, frontmatter, índices generados, cobertura documental, whitespace, Markdown y JSON conformes en sus alcances. Los checks globales `check-links`, `check-docs` y `gen-index --check` finalizan con exit 0. Permanecen advertencias no bloqueantes de nombres duplicados ajenos al estudio. No se ejecutaron tests, aplicaciones, builds ni benchmarks.
- **Versionado:** entrega incorporada mediante commits locales selectivos en `develop`, sin push. `MASTER_PROMPT_EJECUCION_EXTERNA.md` permanece intacto, sin versionar y fuera de índices mediante exclusión local `.git/info/exclude`; `.gitignore` no se modifica. Evidencia documental: logs `Temp/final-*.log` y `vantadb-final-validation.md`.

- OpenViking (`volcengine/OpenViking`, 41718 stars, AGPL-3.0) ya estaba clonado y estudiado: figura en el inventario con su ficha central fuera de este directorio.
- `FlowiseAI/Flowise` está **archivado** upstream (estado de mantenimiento declarado en su ficha).
- Licencias `NOASSERTION` / source-available se registran por proyecto; **no** todos son OSI-validated. `NOASSERTION` no prueba ausencia de licencia ni permiso de reutilización.
- Los **42 informes** (37 de la selección original + 5 excepciones) tienen autoría y revisión independiente: **41 aceptados con límites + 1 limitado** (Meilisearch, E3 incidencias no acreditado). Aceptación documental no equivale a cumplimiento total de ejes ni a spec de implementación. El mapeo durable está en [EXPANSION-STATUS.md](EXPANSION-STATUS.md) y [REVIEW-COVERAGE.md](REVIEW-COVERAGE.md). Los `NOASSERTION` del snapshot se conservan RAW: lecturas locales posteriores acreditaron `LICENSE` **PostgreSQL License** en pgvector y **BSD-3-Clause fuera de `wl/` + Weaviate License dentro de `wl/`** en Weaviate — los JSON congelados no se reescriben.
- `.gitignore` (línea 268) ignora `docs/dev/research/` y se **preserva** para el resto del árbol; los archivos de esta entrega se incorporan selectivamente con `force-add`, sin modificar reglas globales ni incluir archivos ajenos.

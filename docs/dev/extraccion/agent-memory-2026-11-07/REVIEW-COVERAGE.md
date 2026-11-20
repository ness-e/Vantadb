---
title: "Cobertura de revision de los 42 informes ampliados"
kind: research
status: active
description: "Registro por proyecto de autoria, revision independiente, correcciones y aceptacion documental de la ampliacion"
tags: [research, memory, engineering]
---

# Cobertura de revision de los 42 informes ampliados

## Alcance y significado del estado

Este registro no acredita ejecucion de proyectos ni resultados de benchmarks.
Se refiere al cumplimiento documental del
[contrato de ampliacion](EXPANSION-CONTRACT.md), sobre los clones fijados y la
[referencia VantaDB](VANTA-REFERENCE.md). OpenViking es la referencia separada,
no un elemento adicional de los 42.

**Estado vigente — INPUT FROZEN LEAD (Tanda 8):** existen autoría y
revisión independiente para **los 42** proyectos. El lead acepta
documentalmente **41 con límites** y **1 limitado**: Meilisearch entregado
con restricción expresa — **E3 incidencias NO acreditado**, no se equipara
a cumplimiento total de los diez ejes. Los recibos, review y closure se
enlazan por proyecto abajo. Los estados anteriores (12/30) son históricos;
las closures aplicadas son `corrections-memory-closure.md` (11),
`corrections-engines-closure.md` (8) y `corrections-platforms-closure.md`
(11). No hay implementación, benchmarks ni tests ejecutados, ni candidata
aprobada.

Las asignaciones de recuperación (19 autorías/23 revisiones), IDs no
recuperables y tandas anteriores son antecedentes, no estado actual:
ver [histórico](EXPANSION-STATUS.md#histórico--reanudacion-tras-la-caida-no-estado-vigente).
La cohorte 6 mantiene máximo **tres** grupos/agentes simultáneos y no
delegación anidada: A memoria/grafos (11), B motores/RAG (8), C plataformas/
frameworks (11). A no certifica el avance de B/C.

Estados:

- **Autoria:** el informe se esta completando; no es una entrega aceptada.
- **Revision:** existe entrega de autoria y se contrasta independientemente.
- **Revision pendiente:** autoría entregada; todavía no se acredita revisión
  independiente de esta entrega ni aceptación.
- **Correcciones:** hay defectos o lagunas pendientes de resolver.
- **Correcciones — corregido, pendiente de closure:** autor aplicó el rework y
  dejó diff/recibo; falta reconfirmación independiente. No es aceptación.
- **Aceptado con limites:** revisado y corregido dentro del alcance estatico;
  las superficies no inspeccionadas y validaciones dinamicas siguen declaradas.

## Evidencia necesaria para revisar cada proyecto

La revision debe leer el informe completo y contrastar fuentes, no contar
encabezados, caracteres o enlaces como sustituto de cobertura.

Debe dejar constancia separada del contexto arquitectonico y de los diez ejes:
recomendaciones; fichas Rust; incidencias upstream; prerrequisitos VantaDB;
rechazos; advertencias; benchmarks; matriz de capacidades; negativos propios;
y escritorio/UI. Por eje se informa **sustentado**, **parcial**,
**no acreditado** o **no aplicable con justificacion**, sin puntuacion agregada.

Se comprueban especialmente las afirmaciones que pueden inducir tareas
incorrectas: tipos propuestos tratados como existentes, capacidades VantaDB
ya implementadas, defectos historicos tratados como actuales, archivo/simbolo
que no sostiene la afirmacion, estado de issue no confirmado, y resultados
publicados tratados como medicion propia. El apartado de benchmarks describe
las fuentes y aplica el metodo ya fijado; esta revision no modifica el harness,
sus prompts, configuraciones, metricas ni conjuntos de evaluacion.

Cada hallazgo debe identificar informe y seccion, evidencia discrepante,
consecuencia y correccion verificable. La ausencia de acceso a una fuente es
una laguna que debe quedar visible, no una prueba de ausencia de capacidad.
La aceptacion final corresponde al responsable principal, despues de revisar
las correcciones y sus evidencias.

## Registro por proyecto

| Proyecto / informe | Estado | Revision y correcciones |
|---|---|---|
| [mem0](mem0.md) | Aceptado con limites | [closure memoria](reviews/corrections-memory-closure.md); [memory: Mem0](reviews/memory.md#mem0), M1/M2/M3; recibo Temp `vantadb-fix-memory.md`  |
| [cognee](cognee.md) | Aceptado con limites | [closure memoria](reviews/corrections-memory-closure.md); [memory-code-remaining: Cognee](reviews/memory-code-remaining.md#cognee), CO-1/CO-2; recibo Temp A  |
| [hindsight](hindsight.md) | Aceptado con limites | [closure memoria](reviews/corrections-memory-closure.md); [memory-additions: Hindsight](reviews/memory-additions.md#hindsight), H-1/H-2; recibo Temp A  |
| [graphiti](graphiti.md) | Aceptado con limites | [closure memoria](reviews/corrections-memory-closure.md); [memory-code-remaining: Graphiti](reviews/memory-code-remaining.md#graphiti), GI-1/GI-2; recibo Temp A  |
| [mempalace](mempalace.md) | Aceptado con limites | [closure memoria](reviews/corrections-memory-closure.md); [memory-code-remaining: MemPalace](reviews/memory-code-remaining.md#mempalace), MP-1; recibo Temp A  |
| [tencentdb-agent-memory](tencentdb-agent-memory.md) | Aceptado con limites | [closure memoria](reviews/corrections-memory-closure.md); [memory-additions: TencentDB](reviews/memory-additions.md#tencentdb-agent-memory), T-1/T-2; recibo Temp A  |
| [supermemory](supermemory.md) — enmienda residual del lead (dirty/pin) incorporada | Aceptado con limites | Lead; [memory-additions: Supermemory](reviews/memory-additions.md#supermemory); fuera del rework A |
| [agentmemory](agentmemory.md) | Aceptado con limites | [closure memoria](reviews/corrections-memory-closure.md); [memory-additions: Agentmemory](reviews/memory-additions.md#agentmemory), AM-1; recibo Temp A  |
| [letta](letta.md) | Aceptado con limites | [closure memoria](reviews/corrections-memory-closure.md); [memory-additions: Letta](reviews/memory-additions.md#letta), L-1/L-2; recibo Temp A  |
| [codebase-memory-mcp](codebase-memory-mcp.md) — enmienda residual del lead (UI); E integración code-index caller/ACK/recovery **sigue no acreditada**; no crear scheduler | Aceptado con limites | [closure memoria](reviews/corrections-memory-closure.md); [code-memory](reviews/code-memory.md#codebase-memory-mcp), CM-1/CM-2; recibo Temp A  |
| [codegraph](codegraph.md) | Aceptado con limites | Lead; [memory-code-remaining: CodeGraph](reviews/memory-code-remaining.md#codegraph); fuera del rework A |
| [graphify](graphify.md) | Aceptado con limites | [closure memoria](reviews/corrections-memory-closure.md); [memory-code-remaining: Graphify](reviews/memory-code-remaining.md#graphify), GF-1; recibo Temp A  |
| [understand-anything](understand-anything.md) | Aceptado con limites | [closure memoria](reviews/corrections-memory-closure.md); [memory-code-remaining: UA](reviews/memory-code-remaining.md#understand-anything), UA-1; no convertir veredicto no bloqueante en aceptación  |
| [qdrant](qdrant.md) | Aceptado con limites | [closure engines](reviews/corrections-engines-closure.md); [engines: Qdrant](reviews/engines.md#qdrant); grupo B, sin cierre acreditado aquí  |
| [milvus](milvus.md) | Aceptado con limites | Lead; [closure Chroma/Milvus](reviews/chroma-milvus-closure.md) |
| [weaviate](weaviate.md) | Aceptado con limites | Lead cerro W1/W2 y redacciones; [review W/P](reviews/weaviate-pgvector.md) |
| [faiss](faiss.md) | Aceptado con limites | Lead; [engines: Faiss](reviews/engines.md#faiss) |
| [meilisearch](meilisearch.md) | Entregado con restriccion expresa — **E3 incidencias NO acreditado** | [closure plataformas](reviews/corrections-platforms-closure.md); [platforms-retrieval-remaining: Meilisearch](reviews/platforms-retrieval-remaining.md#meilisearch); usuario prohíbe forge: sin lectura de cuerpos ni fallbacks — no equivale a cumplimiento de diez ejes |
| [pgvector](pgvector.md) | Aceptado con limites | Lead cerro P1 y redacciones; [review W/P](reviews/weaviate-pgvector.md) |
| [redis](redis.md) | Aceptado con limites | [closure engines](reviews/corrections-engines-closure.md); [engines-rag-remaining: Redis](reviews/engines-rag-remaining.md#redis); grupo B  |
| [tidb](tidb.md) | Aceptado con limites | [closure engines](reviews/corrections-engines-closure.md); [engines-rag-remaining: TiDB](reviews/engines-rag-remaining.md#tidb); grupo B  |
| [surrealdb](surrealdb.md) | Aceptado con limites | [closure engines](reviews/corrections-engines-closure.md); [engines-rag-remaining: SurrealDB](reviews/engines-rag-remaining.md#surrealdb); grupo B  |
| [lancedb](lancedb.md) — enmienda residual del lead (§0.7) incorporada | Aceptado con limites | Lead; [rag-ingestion: LanceDB](reviews/rag-ingestion.md#lancedb) |
| [chroma](chroma.md) | Aceptado con limites | Lead; [closure Chroma/Milvus](reviews/chroma-milvus-closure.md) |
| [llama-index](llama-index.md) | Aceptado con limites | [closure engines](reviews/corrections-engines-closure.md); [engines-rag-remaining: LlamaIndex](reviews/engines-rag-remaining.md#llamaindex); grupo B  |
| [graphrag](graphrag.md) | Aceptado con limites | [closure engines](reviews/corrections-engines-closure.md); [engines-rag-remaining: GraphRAG](reviews/engines-rag-remaining.md#graphrag); grupo B  |
| [light-rag](light-rag.md) | Aceptado con limites | [closure plataformas](reviews/corrections-platforms-closure.md); [platforms-retrieval-remaining: LightRAG](reviews/platforms-retrieval-remaining.md#lightrag); grupo C  |
| [pageindex](pageindex.md) | Aceptado con limites | [closure plataformas](reviews/corrections-platforms-closure.md); [platforms-retrieval-remaining: PageIndex](reviews/platforms-retrieval-remaining.md#pageindex); grupo C  |
| [serviette](serviette.md) | Aceptado con limites | [closure plataformas](reviews/corrections-platforms-closure.md); [platforms-retrieval-remaining: Serviette](reviews/platforms-retrieval-remaining.md#serviette); grupo C  |
| [ragflow](ragflow.md) | Aceptado con limites | [closure engines](reviews/corrections-engines-closure.md); [engines-rag-remaining: RAGFlow](reviews/engines-rag-remaining.md#ragflow); grupo B  |
| [quivr](quivr.md) — hydrate principal→policy→resolver **sigue no acreditado** | Aceptado con limites | [closure engines](reviews/corrections-engines-closure.md); [engines-rag-remaining: Quivr](reviews/engines-rag-remaining.md#quivr); grupo B  |
| [anything-llm](anything-llm.md) | Aceptado con limites | Lead; [closure Anything/Khoj](reviews/anything-khoj-closure.md) |
| [khoj](khoj.md) | Aceptado con limites | Lead; [closure Anything/Khoj](reviews/anything-khoj-closure.md) |
| [dify](dify.md) | Aceptado con limites | [closure plataformas](reviews/corrections-platforms-closure.md); [platforms-frameworks: Dify](reviews/platforms-frameworks.md#dify); grupo C  |
| [langflow](langflow.md) | Aceptado con limites | [closure plataformas](reviews/corrections-platforms-closure.md); [platforms-frameworks: Langflow](reviews/platforms-frameworks.md#langflow); grupo C  |
| [flowise](flowise.md) | Aceptado con limites | [closure plataformas](reviews/corrections-platforms-closure.md); [frameworks-additions: Flowise](reviews/frameworks-additions.md#flowise); grupo C  |
| [langchain](langchain.md) | Aceptado con limites | [closure plataformas](reviews/corrections-platforms-closure.md); [platforms-retrieval-remaining: LangChain](reviews/platforms-retrieval-remaining.md#langchain); grupo C  |
| [langgraph](langgraph.md) | Aceptado con limites | [closure plataformas](reviews/corrections-platforms-closure.md); [platforms-retrieval-remaining: LangGraph](reviews/platforms-retrieval-remaining.md#langgraph); grupo C  |
| [agno](agno.md) | Aceptado con limites | [closure plataformas](reviews/corrections-platforms-closure.md); [frameworks-additions: Agno](reviews/frameworks-additions.md#agno); grupo C  |
| [crewai](crewai.md) | Aceptado con limites | Lead; [platforms-retrieval-remaining: CrewAI](reviews/platforms-retrieval-remaining.md#crewai) |
| [autogen](autogen.md) | Aceptado con limites | [closure plataformas](reviews/corrections-platforms-closure.md); [frameworks-additions: AutoGen](reviews/frameworks-additions.md#autogen); grupo C  |
| [openai-agents-python](openai-agents-python.md) | Aceptado con limites | Lead; [frameworks-additions: OpenAI Agents Python](reviews/frameworks-additions.md#openai-agents-python) |

La tabla refleja la autoridad congelada del lead (Tanda 8): **42
informes revisados**, **41 aceptaciones con límites**, **1 limitado**
(Meilisearch, E3 incidencias no acreditado por prohibición de forge del
usuario — sin lectura de cuerpos ni fallbacks). Las enmiendas residuales
del lead (CBM UI, LanceDB §0.7, Supermemory dirty/pin) quedan registradas
como corrección del propio lead, no como falta de revisión independiente.
Aceptación documental de informe **no** es spec lista: la integración E de
codebase-memory-mcp (caller code-index/ACK/recovery) sigue **no
acreditada** — no crear scheduler por desconocimiento; en Quivr
principal→policy→resolver hydrate sigue **no acreditado**; los jobs de
Studio no comparten pin con esta referencia.

Validación documental offline final y límites en
[README](README.md#notas-de-integridad): auditorías acotadas sin hallazgos y
checks documentales conformes. Entrega incorporada mediante commits locales
selectivos en `develop`, sin push ni ejecución de tests, apps o benchmarks.

Los índices derivados `docs/index.md` y `llms.txt` se regeneraron con `gen-index --write` (0 referencias a MASTER); metadata congelada, contratos y backlog intactos.

## Limites del cierre

La aprobacion de un informe no autoriza implementar sus propuestas ni
convertirlas automaticamente en tareas activas. No certifica seguridad,
recuperacion ante crash, rendimiento, calidad de respuesta o comportamiento
visual. Esas comprobaciones se documentan para ejecucion manual posterior.
Los candidatos comunes se consolidan sin borrar las diferencias entre
proyectos, y ningun cambio en este registro implica autorizacion de push.

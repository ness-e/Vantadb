---
title: "Estado de la ampliacion profunda de los informes"
kind: research
status: active
description: "Registro operativo de autoria, evidencia y revision de los 42 estudios solicitados"
tags: [research, memory, engineering]
---

# Estado de la ampliacion profunda de los informes

## Estado vigente — consolidación (Tanda 8, INPUT FROZEN del lead)

Este bloque es el estado vigente; la cohorte 6 (12/30) y las tandas
anteriores quedan reclasificadas como **históricas** debajo — no prevalecen.
Transcripción de autoridad del lead, sin re-derivar veredictos ni declarar
cumplimiento exhaustivo. Los IDs históricos no acreditan runtime vigente.

- **42 informes** con autoría y revisión independiente: el lead acepta
  documentalmente **41 con límites** y **1 limitado** — Meilisearch, E3
  incidencias **no acreditado** por prohibición de forge del usuario (sin
  lectura de cuerpos ni fallbacks; no equivale a cumplimiento de diez ejes).
- Closures aplicadas: `reviews/corrections-memory-closure.md` (11),
  `corrections-engines-closure.md` (8), `corrections-platforms-closure.md`
  (11) más las reviews/closure previas; enlazadas por proyecto en
  [REVIEW-COVERAGE](REVIEW-COVERAGE.md).
- Enmiendas residuales del lead registradas (CBM UI, LanceDB §0.7,
  Supermemory dirty/pin) — corrección del lead, no falta de revisión.
- Aceptación documental **no** es spec lista ni implementación aprobada:
  integración E de CBM (caller/ACK/recovery) y hydrate Quivr siguen no
  acreditados; jobs de Studio fuera de este pin.
- **Validación documental:** auditorías acotadas sin hallazgos y checks
  documentales conformes; resultados y límites en
  [README](README.md#notas-de-integridad). Entrega incorporada mediante commits
  locales selectivos en `develop`, sin push. No se ejecutaron tests, apps,
  builds ni benchmarks.

## Histórico — cohorte 6 comunicada (antecedente T6; NO estado vigente)

> El bloque siguiente era el "Estado vigente" en su momento; queda
> **reclasificado como histórico** tras la consolidación 41+1 de arriba.
> No prevalece ni describe el estado actual.


En su momento esta tabla prevalecía sobre las asignaciones anteriores;
tras la consolidación 41+1 de arriba es **antecedente** (transcripción de la
autoridad del lead de entonces, no recálculo). Los IDs no acreditan runtime.

**INPUT FROZEN LEAD:** existen recibos independientes para **los 42**
proyectos; recibo no es aceptación. [REVIEW-COVERAGE](REVIEW-COVERAGE.md)
enlaza el archivo de revisión correcto por proyecto. Se transcribe esta
autoridad, no se rederivan los veredictos ni se declara cumplimiento exhaustivo.

| Grupo de corrección — cohorte 6 | Archivos exclusivos | Estado documental |
|---|---|---|
| A: memoria/grafos (11) | mem0, cognee, graphiti, mempalace, hindsight, agentmemory, tencentdb-agent-memory, letta, codebase-memory-mcp, graphify, understand-anything | **Corregido, pendiente de closure independiente**; diff y recibo Temp `vantadb-fix-memory.{diff,md}`. No aceptados |
| B: motores/RAG (8) | qdrant, redis, tidb, surrealdb, llama-index, graphrag, ragflow, quivr | Correcciones; no se declara cierre ni se atribuyen fixes sin recibo del grupo |
| C: plataformas/frameworks (11) | dify, langflow, langchain, langgraph, agno, autogen, flowise, light-rag, pageindex, serviette, meilisearch | Correcciones; no se declara cierre ni se atribuyen fixes sin recibo del grupo |

- **12 aceptados con limites por el lead:** chroma, milvus, weaviate,
  pgvector, anything-llm, khoj, supermemory, openai-agents-python, crewai,
  codegraph, faiss y lancedb. Sus recibos están enlazados por proyecto en
  REVIEW-COVERAGE; no se tocan codegraph/supermemory en esta corrección.
- **Otros 30 en Correcciones**, incluidos los once de A corregidos y
  pendientes de reconfirmación. No se proclaman 30 fixes ni se autoacepta
  ninguno. La aceptación futura corresponde al lead tras revisión independiente.
- **Regla vigente: máximo tres grupos/agentes simultáneos**, sin delegación
  anidada. Esta tabla registra la distribución autorizada, no una comprobación
  de runtime a partir de IDs. Las tandas/IDs inferiores son antecedentes.
- **Meilisearch:** restricción expresa del usuario; no acceso a issues ni
  otras funciones forge. Recibo existente no subsana esa limitación ni
  permite declarar cuerpos leídos/cumplimiento falso del eje incidencias.
- Sin cambios de código, metadata congelada, README/consolidado, contratos,
  backlog ni harness; sin apps/tests/builds/benchmarks/install/servicios/Git writes.
  Las asignaciones históricas se conservan como trazabilidad.

## Alcance y regla de cierre

La entrega previa de 37 fichas no satisface la profundidad reclamada.
Se rehacen esas 37 y se agregan cinco por peticion expresa: **42 informes
ampliados**, ademas del informe OpenViking que sirve de referencia.

Un archivo preexistente, diez encabezados o mayor longitud no equivalen a
cumplimiento. Fases: autoria, revision independiente, rework y aceptado.
Solo el responsable principal acepta la evidencia y el cierre. Las lagunas
de acceso a fuentes o issues se registran expresamente.

Contrato: [EXPANSION-CONTRACT](EXPANSION-CONTRACT.md).
Referencia local: [VANTA-REFERENCE](VANTA-REFERENCE.md).
Captura nueva: `github-additions.json`, `clone-additions-manifest.json` y
`evidence/additions/*.json`. La seleccion original se conserva sin cambios.

## Histórico — reanudacion tras la caida (no estado vigente)

Se comprobo en disco la presencia de los **42 informes**. Los cinco anadidos
y las fichas que antes faltaban ya existen; presencia no equivale a profundidad
suficiente ni a aceptacion. Los doce recibos de la ultima tanda consultados
devolvieron `No subagent found`. No se les atribuye ejecucion vigente, cierre
ni cancelacion. Se preservan las ampliaciones guardadas.

La nueva distribucion separa los informes todavia en autoria de los que se
someten a revision independiente. No hay autores y revisores simultaneos sobre
un mismo informe. Una asignacion acredita el encargo, no su resultado.

| Trabajo | Informes exclusivos | Recibo de esta reanudacion | Estado al asignar |
|---|---|---|---|
| Autoria: extensiones vectoriales | weaviate, pgvector | `sidekick` | Completar borradores |
| Autoria: motores vectoriales | chroma, milvus | `b29fc73f` | Completar borradores |
| Autoria: aplicaciones | anything-llm, khoj | `d9f41b4f` | Completar borradores restituidos |
| Autoria: memoria A | hindsight, agentmemory | `6921931f` | Completar lagunas |
| Autoria: memoria B | letta, supermemory, tencentdb-agent-memory | `d56db3ce` | Completar lagunas |
| Autoria: frameworks | agno, autogen, openai-agents-python | `5d8598d2` | Completar borradores |
| Autoria: recuperacion | light-rag, pageindex, serviette | `ede15882` | Completar lagunas |
| Autoria: plataforma e indice | flowise, meilisearch | `ee3a8e46` | Completar lagunas |
| Revision: memoria | mem0, cognee, graphiti, mempalace | `950e20ae` | Revision asignada, no aceptacion |
| Revision: codigo | codebase-memory-mcp, codegraph, graphify, understand-anything | `74943137` | Revision asignada, no aceptacion |
| Revision: motores | qdrant, faiss, redis, tidb, surrealdb | `12eb38fd` | Revision asignada, no aceptacion |
| Revision: ingesta y RAG | lancedb, llama-index, graphrag, ragflow, quivr | `944792da` | Revision asignada, no aceptacion |
| Revision: plataformas y frameworks | dify, langflow, langchain, langgraph, crewai | `85c693cf` | Revision asignada, no aceptacion |

Los autores guardan un recibo por proyecto en
`C:\Users\Eros\AppData\Local\Temp\vantadb-author-<proyecto>.md` despues de
terminar cada informe. Los revisores guardan progresivamente sus hallazgos y
el ledger de ejes en `reviews/`, dentro de este directorio. Un recibo ausente
no se sustituye por una suposicion de finalizacion.

El registro por proyecto sigue siendo [REVIEW-COVERAGE](REVIEW-COVERAGE.md).
La revision de esta reanudacion no esta concluida por el hecho de estar
asignada. No se ejecutan aplicaciones, tests, builds, benchmarks ni servicios.
El archivo ajeno al inventario `docs/dev/MASTER_PROMPT_EJECUCION_EXTERNA.md`
permanece intacto y no se incorpora automaticamente a la entrega.

## Distribucion historica de autoria

Cada grupo tiene archivos exclusivos y prohibicion de tocar codigo, metadata,
metodo, indices o informes ajenos. Los IDs de ejecucion son recibos operativos,
no identificadores del backlog.

| Grupo | Proyectos / archivos principales | Autoria | Fase | Revision independiente |
|---|---|---|---|---|
| Memoria A | mem0, cognee, hindsight | `4b0bfde6` (previo `5fa5786a`, histórico no recuperable) | autoria reanudada | pendiente |
| Memoria B | graphiti, mempalace, tencentdb-agent-memory | `7c5dc0ba` (previo `2c7d12b1`, histórico no recuperable) | autoria reanudada | pendiente |
| Memoria C | supermemory, agentmemory, letta | `14928d73` (previo `6ae67b55`, histórico no recuperable) | autoria reanudada | pendiente |
| Code-memory | codebase-memory-mcp, codegraph, graphify, understand-anything | `9f38abb5` (previo `ecd54e28`, histórico no recuperable) | autoria reanudada | pendiente |
| Vector DB | qdrant, milvus, weaviate | `1b11b4ee` (previo `fd549851`, histórico no recuperable) | autoria reanudada | pendiente |
| Indices | faiss, meilisearch, pgvector | `3280b817` (previo `2bbd5bcd`, histórico no recuperable) | autoria reanudada | pendiente |
| Persistencia | redis, tidb, surrealdb | `021d668d` (previo `21fc839b`, histórico no recuperable) | autoria reanudada | pendiente |
| Ingesta | lancedb, chroma, llama-index | `059f8914` (previo `db8d2cb8`, histórico no recuperable) | autoria reanudada | pendiente |
| RAG | graphrag, light-rag, pageindex, serviette | `c0f945e4` (previo `99a5fa43`, histórico no recuperable) | autoria reanudada | pendiente |
| Productos | ragflow, quivr, anything-llm, khoj | `f527b84a` (previo `7668009f`, histórico no recuperable) | autoria reanudada | pendiente |
| Plataformas | dify, langflow, flowise | `c85743f7` (previo `984705e8`, histórico no recuperable) | autoria reanudada | pendiente |
| Frameworks A | langchain, langgraph, agno | `e27ec2da` (previo `832fc225`, histórico no recuperable) | autoria reanudada | pendiente |
| Frameworks B | crewai, autogen, openai-agents-python | `a7fb3cac` (previo `7149da5e`, histórico no recuperable) | autoria reanudada | pendiente |

**Nota de estado:** los IDs previos devolvieron `No subagent found` al consultarlos; se conservan como históricos/no recuperables — no se afirma cancelación ni ejecución actual de esos recibos. Los archivos parcialmente escritos se preservan; al inicio de la reanudación faltaban `anything-llm.md`, `khoj.md`, `langchain.md`, `quivr.md` y `ragflow.md`, además de cuatro fichas nuevas; los autores asignados restituirán desde evidencia previa/HEAD sin reset. No se da ningún informe por perdido definitivamente ni por terminado.

## Preparacion ya entregada

Los cinco clones nuevos se prepararon bajo `C:\Users\Eros\` en carpetas
`weaviate`, `pgvector`, `letta`, `lancedb` y `chroma`. Las respuestas originales
de GitHub se preservan byte a byte en `evidence/additions/`.
Las diferencias de estrellas frente a la captura original son temporales,
no cambios retroactivos del filtro anterior.

Los 37 clones existentes no se actualizan silenciosamente. Los autores
trabajan sobre los commits originales registrados, con lectura publica
complementaria de documentacion e incidencias.

## Restricciones

No se ejecutan tests, aplicaciones, builds, benchmarks, instalaciones o
servicios. Los comandos futuros solo se documentan para ejecucion manual.
Los checks locales de documentacion se aplican a lotes revisados.
No hay autorizacion de push ni de implementacion de las propuestas.

## Histórico — regla máximo 3 y Tanda 1

El propietario limita a **tres subagentes simultáneos** por tandas sucesivas. Tanda 1 (asignada, **no completada**): `weaviate.md`+`pgvector.md` con un autor; `chroma.md`+`milvus.md` y `anything-llm.md`+`khoj.md` con dos autores distintos. Los IDs de ejecución anteriores y los de esta tanda son recibos operativos — **no demuestran runtime vigente** (varios devuelven `No subagent found`). Las tablas históricas se conservan; nada se declara aceptado ni en revisión independiente.

- **tanda1 rework:** `weaviate.md`+`pgvector.md` reescritos a profundidad de contrato tras veredicto (ver `vantadb-author-{pgvector,weaviate}.md` en Temp). Nada aceptado; fase de revisión independiente pendiente.

## Histórico — estado tras correcciones acotadas (tanda1 rework)

- `weaviate.md` y `pgvector.md`: mi rework fue **no aceptado** tras revision; ambos **reasignados** a los otros dos agentes de la tanda (autoria en curso, no completados). Nada aceptado.
- `chroma.md`, `milvus.md`, `anything-llm.md`, `khoj.md`: correcciones mecanicas acotadas aplicadas segun `reviews/chroma-milvus.md` (CH-1, CH-2, MV-1, MV-2) y `reviews/anything-khoj.md` (minor Anything + 3 hallazgos Khoj). **Pendientes de reconfirmacion independiente**, no aceptados.
- Regla operativa vigente: maximo 3 agentes simultaneos; Tanda 1 registrada arriba.

## Histórico — Tanda 3 (antecedente; máximo 3 conservado)

- Este autor: correcciones finales W1/W2 en `weaviate.md` y P1 en `pgvector.md` segun `reviews/weaviate-pgvector.md` + actualizacion de estado — aplicadas, **pendientes de veredicto del lead** (no autoaceptadas).
- Otros dos autores (asignados, no verificables por ID): grupo **memoria** (`hindsight.md`, `agentmemory.md`, `supermemory.md`, `tencentdb-agent-memory.md`) y grupo **frameworks** (`agno.md`, `autogen.md`, `openai-agents-python.md`, `flowise.md`). No se afirma ejecucion desde IDs.
- Cuatro informes previos (`chroma`, `milvus`, `anything-llm`, `khoj`): **aceptados con limites documentales** por el lead tras closure; residuales corregidos por el propio lead — no revertir.
- `letta.md`: autoria entregada, **sin** revision independiente.

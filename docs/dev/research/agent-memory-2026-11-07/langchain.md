---
title: "LangChain: adaptadores, contexto y memoria en transicion"
kind: research
status: proposed
description: "Middleware de resumen, edicion de herramientas y aprobacion frente a los bindings y context engine existentes de VantaDB"
tags: [research, memory, engineering]
---

# LangChain: adaptadores, contexto y memoria en transicion

## Indice

- [1. Recomendacion y arquitectura](#1-recomendacion-y-arquitectura)
- [2. Flujo y fichas candidatas](#2-flujo-y-fichas-candidatas)
- [3. Riesgos e issues](#3-riesgos-e-issues)
- [4. Prerrequisitos](#4-prerrequisitos)
- [5. No trasladar](#5-no-trasladar)
- [6. Evidencia y cobertura](#6-evidencia-y-cobertura)
- [7. Benchmarks existentes](#7-benchmarks-existentes)
- [8. Matriz frente a VantaDB](#8-matriz-frente-a-vantadb)
- [9. Debilidades sugeridas](#9-debilidades-sugeridas)
- [10. Interfaz](#10-interfaz)

## 1. Recomendacion y arquitectura

**Propuesta:** evolucionar los bindings existentes, y estudiar invariantes de
compactacion de tool calls. No construir LangChain dentro de Rust.
La comparacion de persistencia/replay esta en [LangGraph](langgraph.md).

- Repo `langchain-ai/langchain`, `C:\Users\Eros\langchain`.
- HEAD local `d9caf083577c43a91c5bf6bd1f7dfc289581184d`.
- **149 142 estrellas**, MIT, no archivado segun
  [metadata congelada](github-candidates.json).
- Se espero checkout completo: HEAD resoluble y status tracked limpio antes
  de leer sus fuentes; los estados intermedios de clonacion no son evidencia.
- Stack: monorepo Python, LangChain Core (mensajes, documentos, retrievers,
  vector stores, runnables), paquetes de proveedores, `langchain_v1` para
  agentes/middleware y `langchain_classic` con contratos historicos.

`create_agent` acepta checkpointer y store de LangGraph y devuelve grafo
compilado. No atribuir al vector store las garantias del runtime.
`BaseChatMessageHistory`/`RunnableWithMessageHistory` pertenecen a otra ruta:
historial de entrada/salida por sesion, no memoria semantica ni checkpoint.
En este HEAD hay deprecaciones explicitas; no extrapolar versiones antiguas.

## 2. Flujo y fichas candidatas

### Flujo identificado

`mensajes de estado -> middleware before_model -> modelo -> AIMessage/tool
calls -> middleware HITL -> ejecucion de tools -> siguiente turno`.
`SummarizationMiddleware.before_model` cuenta tokens, determina corte seguro,
resume prefijo y devuelve `RemoveMessage(REMOVE_ALL_MESSAGES)` mas resumen y
sufijo preservado [C1]. El corte evita separar AI/Tool pairs.
`ClearToolUsesEdit.apply` busca tool calls por ID, sustituye resultados viejos
por placeholder y marca metadata; opcionalmente limpia tambien inputs [C2].
Eso es edicion del contexto, no borrado acreditado de eventos originales.

En [C3], el middleware HITL recoge acciones, llama a `interrupt`, valida
cardinalidad y reconstruye llamadas revisadas. La aprobacion no ejecuta una
transaccion exactamente una vez con el sistema externo.
En [C5], el historial se antepone y, al terminar, se agregan entradas/salidas,
quitando el prefijo historico por longitud para no repetirlo.

### Ficha A: compactacion por grupos causales de herramientas

- **Objetivo:** reforzar invariantes del context engine existente sin crear
  otro contador/presupuesto independiente.
- **Unidad minima:** `SummarizationMiddleware.before_model`,
  485-523 y `_find_token_based_cutoff`, 758-805 [C1];
  `ClearToolUsesEdit.apply`, 79-153 [C2].
- **Seam Vanta:** `vanta-memory/src/context_engine/engine.rs`,
  `assemble/assemble_with_recall` (93 y 252); spill, report_store y budget
  compartido existentes, no nuevo framework de middleware.
- **Contrato Rust propuesto:** `ContextGroup { message_ids, call_ids,
  dependencies, retention }`; `CompactionDecision { kept, replaced,
  evidence_refs, estimated_tokens }` dentro del reporte actual.
- **Pasos/dependencias:** mapear tool-call/result IDs existentes; agrupar
  unidades causales; decidir spill/resumen; contar nuevamente el resultado
  incluyendo recall/persona/escena; conservar originales referenciables.
- **FMEA:** resultado huerfano, resumen pierde aprobacion/rechazo, placeholder
  rompe proveedor, contador aproximado subestima y reinyeccion excede budget.
- **Aceptacion:** salida no contiene resultados sin llamada compatible;
  cada sustitucion tiene referencia al original y motivo; recomputar contexto
  no muta eventos crudos; el budget efectivo sigue siendo el compartido.
- **No objetivos:** adoptar prompts de resumen ni asegurar calidad sin medir.

### Ficha B: conformidad de vector store y significado del score

- **Objetivo:** que aplicaciones usen correctamente el adapter ya existente.
- **Unidad minima:** `VectorStore.search`, 293-325 y
  `_similarity_search_with_relevance_scores`, 450-504 [C6].
- **Seam:** `integrations/langchain/vantadb_langchain/vectorstore.py`,
  `VantaDBVectorStore`, busqueda con score en 254-305 y normalizacion
  566 en adelante; LlamaIndex tambien tiene binding propio.
- **Contrato Rust propuesto:** resultado con `score_kind` explicito
  (distancia/similitud/fusion), `metric` y raw score; el adapter convierte
  a relevancia solo cuando esa conversion tiene contrato definido.
- **Pasos/dependencias:** inventariar metrica/configuracion soportada;
  contrastar filtros, MMR, orden y conversion sync/async; documentar
  opciones rechazadas. Reutilizar BM25/HNSW/RRF, no copiarlos.
- **FMEA:** invertir orden de distancia, tratar RRF como probabilidad,
  doble normalizacion y filtrar despues de limitar. Son casos a validar,
  no defectos actuales probados de Vanta.
- **Aceptacion:** vectores conocidos producen orden/score coherentes con
  metrica declarada; filtros no cruzan namespace; versiones soportadas del
  adapter preservan IDs/documentos; tipo de score no se oculta.
- **No objetivos:** cambiar thresholds de benchmarks ni nuevo retriever Rust.

### Ficha C: evidencia de aprobacion distinta del resultado

- **Objetivo:** preservar decision humana y ediciones cuando se compacta.
- **Unidad minima:** `HumanInTheLoopMiddleware.after_model`, 759-850 [C3].
- **Seam:** provenance y task checkpoints existentes; binding Python para
  LangChain, Studio existente para mostrar decisiones si se aprueba el trabajo.
- **Contrato Rust propuesto:** `ToolDecision { call_id, proposed_digest,
  reviewed_digest, actor, revision, decision }`, enlazado al evento y al recibo
  del ejecutor. Es dato de dominio, no autorizacion delegada al LLM.
- **Pasos/dependencias:** capturar propuesta y argumentos revisados; validar
  actor fuera del modelo; persistir decision antes del efecto; usar clave
  idempotente/reconciliacion del ejecutor; compactar solo una proyeccion.
- **FMEA:** decision aplicada a llamada nueva, revision antigua, replay de
  efecto y resumen que presenta una accion rechazada como realizada.
- **Aceptacion:** rechazo/edicion sobreviven al resume y a compactacion;
  replay no cambia identidad; auditoria distingue aprobado de ejecutado.
- **No objetivos:** nueva capa general de orquestacion o exactly-once remoto.

## 3. Riesgos e issues

| Evidencia | Hecho o limite |
|---|---|
| Codigo [C7] | `BaseChatMemory` avisa que no soporta native tool calling y puede fallar silenciosamente; no usar para codigo nuevo |
| Codigo [C4] | `BaseChatMessageHistory` esta deprecado en el HEAD observado |
| Codigo [C5] | Ruta historica usa longitud del historial para quitar prefijo; concurrencia necesita contrato externo |
| Codigo [C2] | Edicion elimina artifact del mensaje proyectado y marca cleared |
| Hipotesis | Si se desecha el original, la compactacion destruye evidencia; no demostrado aqui para todos los backends |

No se consultaron issues upstream. No se presenta una deprecacion como
vulnerabilidad ni una hipotesis de concurrencia como reproduccion.
Mantener separadas calidad del resumen, correccion del grafo y persistencia.

## 4. Prerrequisitos

Revalidar worktree y versiones: Vanta ya tiene vector stores LangChain y
LlamaIndex, checkpointer/store LangGraph, context assembly, shared budget,
dedup opcional LLM, sesiones, provenance, scheduler y checkpoints.
Antes de añadir contratos, verificar serializacion de mensajes y call IDs,
mapping de namespace/tenant y limites de payload. La inferencia y la
orquestacion deben permanecer en adapters opcionales; motor sin dependencia
obligatoria de Python/LLM.

## 5. No trasladar

No recomendar memorias classic como solucion nueva ignorando sus advertencias.
No importar la cadena de callbacks/proveedores completa. No guardar solo
texto aplanado cuando hacen falta IDs y resultados de tools. No usar
similarity score como veracidad o confidence calibrada.
Revisar MIT, notices y licencias de cada paquete/proveedor antes de derivar
codigo; cambiar lenguaje no elimina obligaciones.

## 6. Evidencia y cobertura

Lectura estatica regida por [METHOD](METHOD.md), sin ejecutar ningun archivo.
Ledger de fuentes examinadas por rangos:

- [C1: summarization.py, 485-525 y 746-820](https://github.com/langchain-ai/langchain/blob/d9caf083577c43a91c5bf6bd1f7dfc289581184d/libs/langchain_v1/langchain/agents/middleware/summarization.py#L485-L525).
- [C2: context_editing.py, 79-155](https://github.com/langchain-ai/langchain/blob/d9caf083577c43a91c5bf6bd1f7dfc289581184d/libs/langchain_v1/langchain/agents/middleware/context_editing.py#L79-L155).
- [C3: human_in_the_loop.py, 759-850](https://github.com/langchain-ai/langchain/blob/d9caf083577c43a91c5bf6bd1f7dfc289581184d/libs/langchain_v1/langchain/agents/middleware/human_in_the_loop.py#L759-L850).
- [C4: chat_history.py, 23-60 y 182-209](https://github.com/langchain-ai/langchain/blob/d9caf083577c43a91c5bf6bd1f7dfc289581184d/libs/core/langchain_core/chat_history.py#L23-L60).
- [C5: runnables/history.py, 522-565](https://github.com/langchain-ai/langchain/blob/d9caf083577c43a91c5bf6bd1f7dfc289581184d/libs/core/langchain_core/runnables/history.py#L522-L565).
- [C6: vectorstores/base.py, 293-325 y 450-505](https://github.com/langchain-ai/langchain/blob/d9caf083577c43a91c5bf6bd1f7dfc289581184d/libs/core/langchain_core/vectorstores/base.py#L450-L505).
- [C7: classic chat_memory.py, 17-44](https://github.com/langchain-ai/langchain/blob/d9caf083577c43a91c5bf6bd1f7dfc289581184d/libs/langchain/langchain_classic/memory/chat_memory.py#L17-L44).
- [C8: benchmark de conversion, 1-69](https://github.com/langchain-ai/langchain/blob/d9caf083577c43a91c5bf6bd1f7dfc289581184d/libs/core/tests/benchmarks/test_tool_schema_conversion.py#L1-L69).
- [C9: create_agent, firmas y puntos de conexion](https://github.com/langchain-ai/langchain/blob/d9caf083577c43a91c5bf6bd1f7dfc289581184d/libs/langchain_v1/langchain/agents/factory.py#L824-L909).

**No examinado:** todos los proveedores y vector stores comunitarios,
prompts completos del resumidor, middlewares restantes, LangSmith, red ni
suite integral. No equivale a auditoria de toda la cadena de dependencias.
**Pendiente manual:** grupos causales reales, conformidad del adapter y
concurrencia de sesiones; solo despues de permiso del propietario.

## 7. Benchmarks existentes

[C8] es un microbenchmark `pytest-benchmark` de conversion de schemas de tools.
Construye 20 `StructuredTool` sinteticos con campos Pydantic; compara warm
(schema memo poblado) y cold (incluye creacion). No usa dataset de memoria
ni evalua respuesta/retrieval. Metricas de tiempo las produce el fixture;
no hay resultado local ni score de calidad acreditado.
Dependencias: pytest/pytest-benchmark/Pydantic/Core. No llama un LLM en el
codigo revisado; consume CPU/RAM y puede generar artefactos del runner.
No se ejecuto. No usar velocidad de schema como calidad de memoria Vanta.
Para comparaciones futuras rige unicamente
[METHOD](METHOD.md#protocolo-de-comparacion-de-benchmarks);
no se añaden prompts, grader, harness, thresholds ni pipeline.

## 8. Matriz frente a VantaDB

| Capacidad | Vanta | Accion razonable |
|---|---|---|
| Bindings LangChain/LlamaIndex | Existe | Conformidad por version, no recreacion |
| Vector/BM25/RRF | Existe | Aclarar score/filtros en adapter |
| Context assembly/compaction/shared budget | Existe | Verificar atomicidad causal de tool groups |
| Sesiones/provenance/dedup | Existe | Mantener origen frente a proyecciones |
| Scheduler/checkpoints | Existe | No sustituir por historial de chat |
| Edicion/aprobacion por tool call | No acreditado en este estudio | Contrato externo con evidencia |
| Framework de middleware en Rust | No aplicable | Evitar duplicar runtime |

## 9. Debilidades sugeridas

Documentar score y version soportada, inspeccion de llamadas compactadas y
linaje de decisiones ofrece mas valor que añadir otra memoria de chat.
No se probo que Vanta pierda esos datos: se deben revalidar rutas actuales
antes de convertir preguntas en tareas, especialmente con cambios concurrentes.
La falta de un benchmark de calidad en [C8] tampoco niega las mediciones Vanta
ya documentadas en el paso 23.

## 10. Interfaz

Las rutas inspeccionadas implementan SDK/middleware, no una UI de escritorio.
**UI local completa y escritorio nativo: no localizados** en este alcance.
LangSmith/otras superficies alojadas no se presumen OSS por mencionarse en
un SDK. Ningun servidor arrancado. Vanta Studio ya existe en
`develop-desktop`; un inspector de proyeccion versus evento podria extenderlo.

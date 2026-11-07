---
title: "LangChain: adaptadores, contexto y memoria en transicion"
kind: research
status: proposed
description: "Contratos de contexto causal, historial, scores y herramientas sobre los adapters y presupuestos existentes de VantaDB"
tags: [research, memory, engineering]
---

# LangChain: adaptadores, contexto y memoria en transicion

## Indice

- [0. Identidad, mapa y fronteras](#0-identidad-mapa-y-fronteras)
- [1. Recomendacion priorizada](#1-recomendacion-priorizada)
- [2. Flujos y fichas Rust](#2-flujos-y-fichas-rust)
- [3. Incidencias y seguridad](#3-incidencias-y-seguridad)
- [4. Contratos previos en VantaDB](#4-contratos-previos-en-vantadb)
- [5. Alternativas y no trasladar](#5-alternativas-y-no-trasladar)
- [6. Metodo y limites](#6-metodo-y-limites)
- [7. Benchmarks leidos](#7-benchmarks-leidos)
- [8. Matriz de capacidades](#8-matriz-de-capacidades)
- [9. Hallazgos locales discriminados](#9-hallazgos-locales-discriminados)
- [10. UI y delta Studio](#10-ui-y-delta-studio)
- [11. Fuentes, cobertura y division futura](#11-fuentes-cobertura-y-division-futura)

## 0. Identidad, mapa y fronteras

Investigacion estatica del 2026-11-07. El archivo estaba ausente del worktree:
se recupero su evidencia mediante `git show HEAD:docs/dev/research/agent-memory-2026-11-07/langchain.md`,
sin checkout/reset. Se conservan las tres oportunidades previas —compactacion
causal, scores y aprobacion— y se desarrollan historial y limites/retry.
No se ejecutaron aplicaciones, tests, benchmarks ni servicios.

| Identidad | Evidencia congelada |
|---|---|
| Repositorio/clon | `langchain-ai/langchain`, `C:\Users\Eros\langchain` |
| SHA completo | `d9caf083577c43a91c5bf6bd1f7dfc289581184d`, HEAD releido sin fetch |
| Captura | 2026-11-07T06:56:40.3815325Z; 149 142 estrellas, no archivado |
| Licencia | MIT del proyecto; proveedores/dependencias requieren revision propia |
| Omisiones del clon | Shallow, sin historia anterior/tags/otras ramas, submodulos/LFS no descargados |
| Referencia Vanta | `97c9623c8ef5a70fdb51aff215bfbb3b05b15612`; no confundir con HEAD futuro |

Metadata y omisiones: [manifest](clone-manifest.json). Las estrellas no son
calidad ni certificacion. No se actualizo el clon.

### Mapa y stack

| Ruta | Responsabilidad e interfaces |
|---|---|
| `libs/core/langchain_core/messages/` | `HumanMessage`, `AIMessage`, `ToolMessage`, IDs, tool calls, metadata/artifacts |
| `libs/core/langchain_core/runnables/history.py` | Composicion invoke/ainvoke e historial externo por configuracion [C5] |
| `libs/core/langchain_core/vectorstores/base.py` | Documentos, search, scores, MMR y retriever; storage delegado [C6] |
| `libs/langchain_v1/langchain/agents/factory.py` | `create_agent` produce `CompiledStateGraph`, recibe store/checkpointer [C9] |
| `libs/langchain_v1/langchain/agents/middleware/` | Hooks antes/despues del modelo y wrappers de tools; no motor de almacenamiento |
| `libs/langchain/langchain_classic/memory/` | Contratos legacy: `BaseChatMemory` advierte sobre native tool calling [C7] |
| `libs/partners/`, `libs/text-splitters/`, `libs/standard-tests/` | Integraciones, chunking y conformidad; no auditados integralmente |

Paquetes Python con `uv`, Pydantic y versiones independientes. Los pyproject
leidos de Core y v1 declaran Python `>=3.10.0,<4.0.0`; v1 requiere
`langchain-core>=1.7.5,<2.0.0` y `langgraph>=1.2.12,<1.3.0` [C12].
Una dependencia por rango no demuestra la version instalada del usuario.
Embedding/LLM, red, telemetria y persistencia dependen de los componentes
configurados: una primitiva Core no requiere por si sola un SaaS.

### Modelos y ownership

- Un `Document` es contenido + metadata/ID para retrieval. No es estado de
  ejecucion, una prueba de verdad ni un hecho extraido con provenance.
- `AgentState.messages` es entrada/estado del grafo. `RemoveMessage` es update
  al reducer, no una instruccion de borrado legal sobre todas las copias.
- Historial de chat es una secuencia por sesion; `BaseChatMessageHistory`
  esta deprecado en este SHA [C4]. No equivale a checkpoint/replay.
- Checkpointer y `BaseStore` son interfaces de **LangGraph** recibidas por
  `create_agent`: la durabilidad pertenece al runtime/saver, no al vector store.
- `runtime`/contexto de invocacion, argumentos de tools y memoria persistente
  tienen owners distintos. Un `user_id` que el modelo produce no autentica.

## 1. Recomendacion priorizada

1. **Conformidad antes de funcionalidad nueva:** mantener
   `VantaDBVectorStore`, `VantaDBCheckpointer` y `VantaDBStore`; explicitar
   scores/filtros/capacidades. No construir LangChain en Rust.
2. **Contexto causal y recuperable:** extender el coordinador de presupuesto
   existente con grupos tool-call/result y sustituciones trazables. Resumir
   vacio no debe equivaler a compactacion exitosa.
3. **Decisiones y efectos:** conservar propuesta, edicion/rechazo y recibo;
   una aprobacion humana no prueba ejecucion exactamente una vez.
4. **Historial y costes:** append con revision y limite de invocaciones en
   la frontera de agente/adapters, no otro presupuesto de tokens en storage.

Las fichas son propuestas evaluables, sin IDs de backlog reservados. Pueden
convertirse en tareas despues de aprobacion y revalidacion del snapshot.

## 2. Flujos y fichas Rust

### Flujos observados: exito, actualizacion, borrado y fallo

**Invocacion moderna:** `create_agent(model, tools, middleware, checkpointer,
store)` → grafo compilado [C9] → mensajes → hooks `before_model` → modelo →
AI/tool calls → `after_model`/HITL → wrappers de tools → siguiente turno.
El framework decide el orden; el adapter Vanta no ejecuta nodos ni reducers.
Guardar un vector no hace persistente este ciclo.

**Compactacion:** [C1] asegura IDs, cuenta tokens, calcula corte y particiona.
El corte token-based usa busqueda binaria; corrige la frontera buscando la
llamada AI correspondiente a ToolMessages, o avanza cuando no la encuentra.
Solo despues de generar resumen devuelve `RemoveMessage(REMOVE_ALL_MESSAGES)`
+ resumen + sufijo. Excepciones del resumidor se propagan; hay retry especifico
de overflow. El retorno vacio sigue pasando por `.strip()` [C13].
El update no acredita que el contexto final completo respete presupuesto.

**Edicion de tools:** [C2] cuenta tokens, selecciona ToolMessages antiguos,
excluye herramientas configuradas, encuentra AI/call por ID y sustituye
contenido por placeholder. Borra `artifact` de esa copia y marca metadata
`cleared`; opcionalmente limpia inputs. No borra la fuente externa.
Si no encuentra llamada compatible, omite esa sustitucion.

**Historial:** [C5] copia historial de entrada, lo antepone si corresponde;
al salir carga mensajes permitidos de inputs/outputs, resta el prefijo por
longitud del historial **releido** y hace `add_messages`. Hay rutas async.
No se observa CAS entre lectura y append; dos runs necesitan serializacion
externa. Un fallo antes del hook de salida no acredita append ni rollback.

**Retrieval:** el vector store recibe documento/embedding en la implementacion;
`similarity_search_with_score` devuelve score nativo. La API de relevancia
aplica la funcion elegida y luego threshold [C6]. Vanta genera embedding de
query, usa metrica cosine, filtros `filter_key/filter_val`, y permite
`text_query` para hibrido [V1]. MMR no es aprobacion ni dedup de hechos.
Get/delete de documentos son operaciones del adapter; no limpian por si
solas historial, checkpointer, logs del proveedor o indices ajenos.

**Mutación documental, frontera distinta del retrieval (LC-1):** Core
[`VectorStore.add_documents:234-291`](https://github.com/langchain-ai/langchain/blob/d9caf083577c43a91c5bf6bd1f7dfc289581184d/libs/core/langchain_core/vectorstores/base.py#L234-L291)
declara add/update, prioriza `kwargs.ids` sobre `Document.id` y delega en
`add_texts`; eso no garantiza atomicidad del conjunto. En Vanta al pin,
[`_build_key:84-85`](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/integrations/langchain/vantadb_langchain/vectorstore.py#L84-L85)
genera UUID5 de **texto+posición** para `add_texts` sin IDs. Editar texto o
reordenar puede producir otra clave, no reemplazo de la anterior.
[`add_texts:414-448`](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/integrations/langchain/vantadb_langchain/vectorstore.py#L414-L448)
valida longitudes de listas no vacías antes del embedding; después indexa
`embeddings[i]` y llama puts **individuales** con texto/metadata/vector.
No precomprueba aquí cardinalidad completa de embeddings ni crea transacción
de grupo. ID explícito estable dirige el nuevo contenido a la misma clave,
sujeto a la política del put; metadata no se fusiona en este adapter.
[`add_documents:450-475`](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/integrations/langchain/vantadb_langchain/vectorstore.py#L450-L475)
es override propio: usa `Document.id` o UUID4 por faltante y pasa `ids=ids`;
no heredar sin comprobar la precedencia de Core para `kwargs.ids` (puede
colisionar con ese argumento explícito). No es la identidad de add_texts.
[`delete:477-503`](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/integrations/langchain/vantadb_langchain/vectorstore.py#L477-L503)
itera keys y devuelve True al terminar, también para lista vacía/None.
Excepción tras un put/delete deja posible prefijo aplicado sin lista parcial
retornada; retry no es undo, ni cancelación del caller revierte lo aplicado.
Son hechos/deducciones del adapter individual, no prueba de pérdida o falta
de transacciones en todo el motor.

**HITL/replay/error:** [C3] construye peticiones solo para calls configuradas,
llama `interrupt`, valida numero de decisiones y reconstruye orden/IDs;
limpia ediciones viejas cuando no hay nuevas acciones. `per_call` usa otra
ruta en `wrap_tool_call`, no esta rama batch. Reanudar pertenece a LangGraph
y puede volver a ejecutar el nodo. [C11] propaga `GraphBubbleUp` sin retry;
otros errores retryables vuelven al handler con backoff. Cancelar no revierte
un efecto remoto ya aceptado.

### Ficha A — grupos causales y commit de compactacion

- **Problema:** un agente con varias tools paralelas no debe recordar un
  resultado sin su llamada, ni perder una instruccion por resumen fallido.
- **Fuente minima:** `SummarizationMiddleware.before_model`, 485-523;
  `_find_token_based_cutoff`, 758-805; corte seguro y `_create_summary`,
  900-986 [C1][C13]. SHA y ruta en el ledger.
- **Delta/seam:** `context_engine/engine.rs::assemble_inner`,
  `assemble_with_recall`, `SpillSink`, `IntegratedContext` ya existen [V2].
  Añadir invariantes de grupo y recibo de sustitucion, no otro compactador.
- **Contrato Rust propuesto:** `ContextGroup { message_ids, call_ids,
  dependencies }`; una decision ligada a revision de entrada produce
  `kept/replaced/evidence_refs/estimated_tokens` en el reporte actual.
  Errores diferenciados: fuente cambiada, grupo roto, resumen vacio,
  presupuesto excedido. El prefijo protegido puede impedir caber [V2].
- **Pasos/commit:** resolver IDs → formar grupos → planificar spill/resumen →
  generar fuera del motor → validar no vacio y revision → persistir evidencia
  requerida → publicar proyeccion y reporte → recontar union con recall.
  Si spill obligatorio falla, no publicar sustitucion destructiva.
- **Concurrencia/cancelacion:** rechazo de plan sobre revision nueva; cancelar
  antes de publicar conserva fuente. Despues del commit se recupera la misma
  proyeccion, no se inventa rollback de la llamada LLM.
- **Deps/alternativas:** usa presupuesto compartido/estimador y provenance;
  se relaciona con B y C sin fusionar texto proyectado y autorizacion.
  Alternativa conservadora: mantener grupo completo o spill sin resumen.
  Migracion aditiva del reporte; desactivar plan nuevo deja originales legibles.
- **FMEA:** corte huerfano → rechazo por IDs; resumen blanco → error visible;
  summary omite rechazo → conservar decision estructurada; estimador subcuenta
  → reportar exceso, no garantizar tokens exactos.
- **Aceptacion manual futura:** tools multiples/desordenadas, resumen vacio,
  fallo del proveedor y cambio concurrente dejan fuente accesible; contexto
  final conserva pares y decision; diagnostico usa IDs/motivos, no payload.
  **No objetivos:** prompt nuevo, calidad garantizada, nuevo framework.

### Ficha B — edicion selectiva sin confundir retencion y olvido

- **Problema:** un resultado grande ocupa contexto aunque el agente solo
  necesita saber que se obtuvo; quitarlo no debe ocultar evidencia ni PII.
- **Fuente:** `ClearToolUsesEdit.apply`, 79-155 [C2]. Se separa de A porque
  no genera hechos/resumen: reemplaza payloads y conserva enlace causal.
- **Delta/seam:** `SpillSink::spill` y reporte de `assemble_with_recall` [V2];
  Vanta ya compacta. Contrato propuesto `ToolProjection { call_id,
  source_revision, retained_ref, fields_removed, reason }`, con error
  `SourceUnavailable` cuando se requiere rehidratacion garantizada.
- **Pasos/commit:** clasificar resultado y politica de retencion → guardar
  original autorizado → construir placeholder compatible → publicar copia
  vinculada al reporte. Delete real es flujo aparte sobre todas las copias
  bajo politica; nunca declarar borrado por haber puesto `artifact=None`.
- **Concurrencia/cancelacion:** mismo ID/revision vuelve idempotente la
  proyeccion; no sobrescribir fuente editada; cancelar antes del commit no
  marca cleared. Despues se conserva recibo; rehidratacion revalida ACL.
- **Deps/alternativas:** A valida grupos, C conserva decision; allowlist de
  tools/retencion pertenece al host. Alternativa: truncar vista sin persistir
  placeholder, o no editar tools criticas. Version aditiva; rollback de vista
  desde fuente, nunca restaurar PII eliminada por politica posterior.
- **FMEA:** log/artifact conserva secreto → inventario de copias y redaccion;
  placeholder invalido → validar formato proveedor; referencia borrada →
  presentar indisponible, no evidencia inexistente como presente.
- **Aceptacion futura:** excluded tool intacta, edicion repetida estable,
  contenido retirado del contexto pero accesible solo con permiso; borrado
  posterior impide rehidratacion. Observabilidad por campos/IDs, no secretos.
  **No objetivos:** sistema universal de retention ni borrado remoto.

### Ficha C — decision humana distinta del recibo de herramienta

- **Problema/fuente:** edicion o rechazo no deben transformarse en «ejecutado»
  durante compactacion; `HumanInTheLoopMiddleware.after_model`, 759-850 [C3].
- **Delta/seam:** checkpoints de tareas [V3] y provenance existentes.
  `CaptureApprovalQueue` [V4] aprueba **memorias extraidas**, no tools;
  reutilizar patron de decision sin presentar ambos dominios como equivalentes.
- **Contrato propuesto:** `ToolDecision { call_id, proposed_digest,
  reviewed_digest, actor, revision, decision }` + recibo del ejecutor.
  Rechazar actor no autorizado, decision vieja y cardinalidad incorrecta.
- **Pasos/commit:** registrar propuesta → autenticar aprobador fuera del LLM →
  persistir decision → invocar con idempotency key → persistir recibo →
  proyectar contexto. Remoto acepto y respuesta perdida = desconocido/reconciliar.
- **Concurrencia/cancelacion:** decision terminal condicionada a revision;
  solo un claim de ejecucion. Cancelar impide nuevos dispatches, no deshace
  efecto; estado aplicado exige recibo. Migracion aditiva y executor opt-in.
- **Deps/alternativas:** misma identidad que replay de LangGraph; A/B retienen
  decision. Alternativa es usar middleware upstream y almacenar solo evidencia
  en Vanta; preferible a reimplementar HITL completo en Rust.
- **FMEA:** aprobacion sobre args nuevos → digest/revision; replay duplica envio
  → dedup remoto o reconciliacion; rechazo resumido como exito → estado tipado.
- **Aceptacion futura:** editar/rechazar y reanudar con varias calls conserva
  orden e identidad; recibo faltante no aparece como exito; actor/decision
  auditables sin guardar argumentos sensibles por defecto.
  **No objetivos:** exactly-once remoto, autorizar por texto o nuevo scheduler.

### Ficha D — append de historial ligado a la revision leida

- **Problema:** dos invocaciones de una sesion pueden intercalar historial;
  quitar prefijo por longitud actual no prueba que sea el prefijo usado.
- **Fuente:** `RunnableWithMessageHistory._enter_history/_exit_history`,
  522-580 [C5]; no se recomienda adoptar esa API deprecada como nueva base.
- **Delta/seam:** sesiones/provenance L1 y `l1_writer::plan_write`,
  `persist_planned`, `apply_dedup_batch` [V5]. No crear otro historial ni
  reusar checkpoint global como log de conversacion.
- **Contrato propuesto:** `append_turn(session, expected_revision, run_id,
  input_ids, output_ids) -> receipt` con `Conflict/AlreadyApplied/Unauthorized`.
  El caller registra IDs del prefijo leido; el storage no deduce identidad
  restando longitud ni compara solo texto.
- **Pasos/commit:** resolver sesion autorizada → capturar revision → inferir
  fuera del lock → append condicionado del turno → emitir recibo. Si el seam
  disponible no ofrece CAS, serializar por sesion en host y declarar limite
  entre procesos; no afirmar atomicidad por el nombre batch.
- **Concurrencia/cancelacion:** mismo run devuelve mismo recibo; conflicto no
  fusiona automaticamente respuestas producidas sobre contexto distinto.
  Cancelacion antes de append deja intento separado; posterior conserva
  turno confirmado. Migracion de eventos viejos no inventa run_id historico.
- **Deps/alternativas:** contrato de sesion y ownership; no depende de A/B
  para funcionar. Alternativa minima es un solo escritor por sesion en adapter.
- **FMEA:** retry duplica turno → run_id; fallo mitad de input/output → commit
  de unidad o recovery declarado; contaminacion entre sesiones → clave scoped.
- **Aceptacion futura:** dos runs sobre misma revision no pierden inputs;
  retry y cancelacion distinguen intento/confirmacion; logs por revision/IDs.
  **No objetivos:** replay de nodos ni resucitar memorias classic.

### Ficha E — contabilizacion de intentos y retry en la frontera correcta

- **Problema:** numero de tools propuesto, despachado y realmente ejecutado
  difiere; retry puede multiplicar coste o efectos sin aumentar turnos.
- **Fuente:** `ToolCallLimitMiddleware.after_model`, 327-388 [C10], distingue
  contador por thread y por run; `ToolRetryMiddleware.wrap_tool_call`,
  295-353 y 387-413 [C11], no reintenta `GraphBubbleUp`.
- **Delta/seam:** `TaskCheckpointManager::begin/advance/fail` [V3] puede
  guardar progreso del trabajo; el host/adapter controla dispatch/retry.
  No se altera el presupuesto de contexto de `assemble_with_recall`.
- **Contrato Rust propuesto:** recibo `InvocationUsage { operation_id,
  attempt_id, proposed, dispatched, outcome, usage_ref }` y clasificacion
  `Retryable/Terminal/Interrupted/Unknown`. Politica de limites sigue en host;
  no se proponen numeros ni thresholds.
- **Pasos/commit:** reservar intento segun politica vigente → registrar
  despacho → ejecutar → registrar resultado/coste conocido → decidir retry.
  Persistir separadamente control-flow interrupt, no convertirlo en tool error.
  Revision del recibo evita dos dispatchers del mismo intento; timeout despues
  de aceptar remoto exige reconciliar antes de repetir accion no idempotente.
- **Cancelacion/migracion:** backoff debe poder detener nuevos intentos;
  wrapper sync usa `time.sleep`, async usa `asyncio.sleep`: no asumir la misma
  responsividad. Datos historicos sin intentos quedan desconocidos. Opt-in
  desactivable sin borrar recibos; no migrar contador token a contador tool.
- **Deps/alternativas:** C comparte operation_id/recibo, no su decision.
  Alternativa preferida: middleware upstream + trazas de uso Vanta;
  implementar retry generico en storage queda rechazado.
- **FMEA:** retry de interrupt → control flow separado; doble gasto por carrera
  → claim/revision; presupuesto reiniciado al resume → distinguir run/thread;
  excepcion sensible en mensaje → codigos/motivos redactados.
- **Aceptacion futura:** intento bloqueado no aparece ejecutado; replay no
  duplica recibo; cancelacion en backoff no despacha otra tool; fallo remoto
  ambiguo no es exito. **No objetivos:** cuotas globales nuevas ni runtime Rust.

### Ficha F — score tipado y conformidad del vector store existente

Conserva y precisa la ficha previa de scores, no propone un indice nuevo.

- **Problema/fuente:** un consumidor no debe interpretar distancia, similitud
  y score fusionado como probabilidad. `VectorStore` relevancia [C6] convierte
  score nativo, advierte fuera de `[0,1]` y permite threshold posterior.
- **Delta/seam:** `VantaDBVectorStore.similarity_search_with_score`,
  254-305, `_select_relevance_score_fn`, 566-576 [V1]. Ya usa cosine y clamp
  de similitud: **no** reabrir el falso positivo de distancia invertida.
  Hibrido con `text_query` requiere caracterizar el score realmente devuelto.
- **Contrato propuesto:** resultado Rust con `score_kind/metric/raw_score`
  y conversion de relevancia explicita en adapter; `UnsupportedScoreMapping`
  si no existe transformacion semantica defendible. Identidad documental y
  revision del indice separadas del score.
- **Pasos/commit:** inventariar scores por ruta → mapping por version/metrica →
  preservar IDs/metadata → verificar filtros antes de top-k donde corresponda →
  exponer semantica. Incorporar el recorrido de mutación LC-1 anterior:
  distinguir IDs explícitos, UUID5 texto/posición y UUID4 de add_documents;
  declarar reemplazo, resultado parcial y retry por operación individual.
  No reindexar por añadir metadata del score; cambio de embedding si lo exige.
- **Concurrencia/cancelacion:** resultado indica revision observada o su
  ausencia; cancelacion de embedding no produce resultado «vacio exitoso».
  Delete concurrente no promete snapshot sin soporte probado.
- **Deps/alternativas:** version Core/adapter; BM25/HNSW/RRF existentes.
  Alternativa: devolver solo score raw y rechazar relevance donde no se pueda;
  no clamping silencioso de cualquier fusion como solucion universal.
- **FMEA:** doble normalizacion → verificar extremos; RRF como confianza →
  tipo explicito; filtrado tardio pierde candidatos → prueba discriminante;
  ID derivado cambia al editar → versión duplicada → identidad estable del
  caller; embedding corto/fallo después del primer put → prefijo aplicado →
  no responder “nada escrito”, conservar observación por operación en host.
- **Aceptacion futura:** vectores conocidos y ruta hibrida mantienen orden
  declarado; filtros/IDs sobreviven sync/async; unsupported es error visible,
  sin cambiar thresholds del benchmark. Casos futuros: add/update del mismo
  ID con texto/metadata/vector nuevos; sin ID editado/reordenado; kwargs.ids
  frente a Document.id; cardinalidad incorrecta antes/después de embedding;
  fallo/cancel/retry después del primer put y delete parcial. Confirmar
  contrato de la versión Core soportada, no asumir conformidad con la actual.
  **No objetivos:** ranking, índice, WAL o transacción paralelos.

### Contraste temporal acotado: buffer y persistencia no son equivalentes

LC-2: se leyó
[`TimeWeightedVectorStoreRetriever:64-198`](https://github.com/langchain-ai/langchain/blob/d9caf083577c43a91c5bf6bd1f7dfc289581184d/libs/langchain/langchain_classic/retrievers/time_weighted_retriever.py#L64-L198),
no toda la memoria classic. Combina recencia, relevancia y claves extra;
`buffer_idx` resuelve el documento contra `memory_stream`, no contra una
identidad durable universal. `_get_rescored_docs:115-132` toca
`last_accessed_at` **sólo en buffer**; el TODO no persiste ese touch.
`add_documents/aadd_documents:164-198` copian documentos, asignan timestamps/
índice y **extienden memory_stream antes** de llamar al vector store.
Si éste falla, buffer y store divergen; reinicio no reconstruye por sí solo
stream, orden o último acceso. No afirmar rollback o persistencia del touch.

**Decisión:** rechazar portar ese buffer, pesos o índice posicional. Vanta ya
tiene scoring opt-in sobre recall:
[`CompositeScoring/composite_score/composite_rank:65-144`](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/record/scoring.rs#L65-L144),
con recencia basada en updated_at, no necesariamente last_accessed_at.
Único delta a estudiar: **capa experimental propuesta** de identidad/touch
para quien necesite esa semántica, sobre scoring/adapters existentes; no
repetir retrieval ni crear índice. Host sería dueño de `(scope,id,revision)`,
revalidaría permiso al touch y declararía si es efímero o persistido, con
error explícito y sin cambiar fuente/updated_at sólo por leer.
FMEA: touch en buffer aparenta durability → mostrar frontera; store falla
tras extender buffer → no afirmar publicación; índice posicional viejo →
rechazar referencia no resoluble. Aceptación futura: fallo de persistencia,
reinicio, ID retirado/ajeno y touch repetido conservan identidad/permiso y
distinguen estado efímero/durable. No se adoptan scoring/configs/umbrales
ni se implementa esa capa.

## 3. Incidencias y seguridad

Fuentes publicas leidas el **2026-11-07**, no reproducciones propias:

| Issue, estado observado | Reporte y revision | Contraste de codigo y aplicabilidad |
|---|---|---|
| [#41323: empty summary](https://github.com/langchain-ai/langchain/issues/41323), **open**, creado 2026-10-20 | `langchain 1.4.6`, Core 1.6.12; cita master `d670a3ac7c`; modelo devuelve texto vacio y se retira historia | En el SHA congelado `_create_summary/_acreate_summary` devuelven `.text.strip()` sin guard no-vacio, antes de `RemoveMessage` [C13]. Mecanismo aun visible estaticamente; validar A, no atribuir defecto a Vanta |
| [#38867: exception used as summary](https://github.com/langchain-ai/langchain/issues/38867), **closed** 2026-08-07; creado 2026-07-16 | Reporta convertir error de proveedor en texto y eliminar historia; enlace relacionado #39268 | SHA leido documenta/propaga excepcion y tiene manejo especifico de overflow [C13]. No usar issue cerrado como prueba de mismo bug actual; no se audito diff del PR |

En [comentarios #41323](https://github.com/langchain-ai/langchain/issues/41323#issuecomment-6285552179)
se propone guard compartido sync/async/custom el 2026-10-20, no una correccion
mergeada demostrada. La API publica devolvio estado/body; no inferir CVE.
Busqueda DuckDuckGo y fallback Bing/metasearch para SummarizationMiddleware
no dieron resultados; se leyeron despues endpoints GitHub oficiales.

Frontera de confianza: mensajes recuperados son contenido no confiable, no
instrucciones de autorizacion; IDs de tools/documentos no son credenciales.
Persistir argumentos/resultados en historial, artifact, provider tracing o
proyecciones amplia copias de PII. Clear del contexto no demuestra borrado.
Un filtro de retrieval no sustituye ACL de get/update/delete.
`BaseChatMemory` advierte fallos silenciosos con native tools [C7]:
deprecacion/advertencia no es por si misma vulnerabilidad.

## 4. Contratos previos en VantaDB

Prerrequisitos **por candidato**, no bloqueo universal:

| Candidato | Capacidad ya existente | Decision necesaria antes de implementar |
|---|---|---|
| A/B | Context engine, spill, reporte, recall/budget [V2] | Formato de call IDs, fuente durable exigida y politica si prefijo excede |
| C | Task checkpoints [V3], cola capture approval [V4] | Identidad/actor y recibo de tool; no confundir captura con ejecucion |
| D | Plan/persistencia y dedup L1 [V5] | Revision/claim y contrato de append por sesion |
| E | Scheduler/checkpoints y presupuesto contextual | Owner del dispatch, cancelacion y unidad de coste; no cuotas de storage |
| F | Adapter/normalizacion cosine [V1] | Semantica de score hibrido y versiones oficialmente soportadas |

Provenance, sesiones, supersession, dedup opcional LLM, personas/escenas,
BM25/HNSW/RRF y gobernanza de inyeccion ya estan inventariados en
[VANTA-REFERENCE](VANTA-REFERENCE.md). Su existencia no prueba integracion
completa en todos los transports. LLM/embedding siguen opcionales fuera del
motor embebido; no imponer Python ni proveedor remoto a las primitivas Rust.

## 5. Alternativas y no trasladar

Extender adapter es preferible a copiar jerarquia Runnable/proveedores.
Retener todo evita error de resumen, pero consume contexto/coste; spill
con referencia reduce contexto, pero exige disponibilidad y permisos al
rehidratar. Resumen es derivado falible, no evidencia primaria.
No importar el buffer temporal classic; el contraste acotado LC-2 de §2
justifica rechazo y delimita sólo una capa experimental propuesta de touch.
No reemplazar checkpoints
por chat history ni usar threshold de similitud como verdad.
MIT requiere notices y revision de dependencias/licencias por componente.
SaaS de observabilidad tiene condiciones separadas: no queda cubierto por MIT.

## 6. Metodo y limites

[METHOD](METHOD.md) y [BENCHMARK-NOTES](BENCHMARK-NOTES.md) gobiernan la
comparacion; [EXPANSION-CONTRACT](EXPANSION-CONTRACT.md) gobierna amplitud.
Hecho estatico, reporte de issue, documentacion y propuesta se distinguen.
No se afirma lectura exhaustiva de proveedores/middleware ni resultado de test.
No se revalidan todos los cambios concurrentes: las conclusiones locales
fijan snapshot97c9623 completo, no certifican el worktree actual.

## 7. Benchmarks leidos

Runner [C8]: `libs/core/tests/benchmarks/test_tool_schema_conversion.py`,
tests `test_convert_tools_warm/cold` con fixture **pytest-benchmark**.
No CLI de memoria, loader LoCoMo, retrieval, answerer ni juez en este archivo.

| Aspecto | Comportamiento leido |
|---|---|
| Dataset/unidad | 20 `StructuredTool` sinteticas; 8 campos opcionales + `target` requerido, Pydantic |
| Warm | Crea tools y puebla schema memo antes del bloque medido; mide conversion de las 20 reutilizadas |
| Cold | Crea las 20 y convierte dentro del bloque; incluye subset-model creation |
| Metrica/denominador | Tiempo por invocacion del bloque de 20, no por pregunta ni por memoria; rondas/warmup efectivos los fija pytest-benchmark, no este archivo |
| Datos/ground truth | Schemas generados, sin respuestas anotadas, splits ni jueces; no semilla necesaria para ese generador determinista |
| Errores/vacios | No captura excepciones en el benchmark; fallo del bloque no es respuesta incorrecta ni abstencion |
| Recursos/costes | CPU/RAM Python, fixture puede producir artefactos; funcion tool devuelve `"ok"`, no se invoca LLM/DB en estos bloques |
| Excluye | Embeddings, red, extraccion, persistencia, recuperacion, tokens del prompt y calidad de memoria |

Core declara `pytest-benchmark` en pyproject [C12]; no se instalaron paquetes.
No se traslada una cifra promocional ni se midio nada.
**Adaptacion:** para F usar las interfaces ya existentes Document/IDs/raw
score del adapter; este runner solo caracteriza conversion Python y no admite
inferir mejora del Rust. Para A/B corresponde la pista contexto del METHOD;
para D, consistencia del motor/sesion. Si se comparan preguntas de memoria,
usar los runners/protocolo existentes de Vanta con dataset real y aislamiento,
sin cambiar prompts, scoring, umbrales, sampling o denominadores. No se crea
harness ni se prescribe ejecutar comandos en esta tarea.

## 8. Matriz de capacidades

| Capacidad | Vanta frente al contrato | Evidencia/decision |
|---|---|---|
| LangChain vector store | **Existe** | [V1]; conformidad versionada, no adapter nuevo |
| Add/update/delete por ID | **Existe; conformidad parcial** | LC-1: puts/deletes individuales, IDs con defaults distintos y fallo parcial sin transacción de grupo |
| Scoring temporal | **Existe opt-in** | `CompositeScoring` en §2; touch/stream durable tipo classic no acreditado ni adoptado |
| Cosine raw → relevancia | **Existe** | Clamp documentado [V1]; descartar inversion ya corregida |
| Score fusionado tipado end-to-end | **No acreditado** | F; caracterizar camino hibrido antes de llamarlo bug |
| Compactacion, spill y presupuesto compartido | **Existe** | [V2], con excepcion prefijo protegido |
| Grupos causales y recibo de sustitucion completo | **Parcial** | Reporte existe; invariantes A/B no acreditadas integralmente |
| Sesiones/provenance/dedup/supersession | **Existe** | [V5] y referencia; no son replay del runtime |
| Checkpointer/store LangGraph | **Existe** | Interfaces locales; conformidad se estudia separadamente |
| Aprobacion de capturas de memoria | **Existe** | [V4], cola in-memory opt-in |
| Decision durable + recibo de tool externa | **No acreditado** | C; no negar la cola de capturas existente |
| Runtime middleware Python dentro de Rust | **N/A** | Framework se queda en host |
| Escritorio nativo LangChain en estas fuentes | **No acreditado** | No deducirlo de LangSmith Studio |

## 9. Hallazgos locales discriminados

**Hechos:** [V1] ya fija normalizacion cosine: la recomendacion previa de
scores debe enfocarse en tipos y ruta hibrida, no «arreglar» distancia.
[V2] preserva prefijo aunque exceda budget; no afirmar garantia absoluta.
[V4] demuestra aprobacion de captura existente: negar todo HITL seria falso.
**Hipotesis:** grupos rotos, append concurrente y score hibrido mal interpretado
necesitan casos discriminantes. No se demostraron como defectos actuales.
**Pendiente:** permisos end-to-end, durabilidad de spill, version instalada,
cancelacion/retry de cada transport. Ninguna medicion invalida Paso 23.

## 10. UI y delta Studio

Codigo local revisado es SDK/middleware. No se localizo implementacion de
escritorio nativo en esas rutas. [Documentacion oficial LangSmith Studio](https://docs.langchain.com/langsmith/studio),
leida 2026-11-07, describe UI web Graph/Chat sobre Agent Server API, threads,
time travel, long-term memory, trazas y deployment cloud. Tambien admite
backend local: **backend local no equivale a frontend OSS en este monorepo**.
No se inspecciono su frontend ni se autentico en SaaS.

Vanta Studio ya existe en `develop-desktop`. Delta candidato, no implementado:
inspector de grupo causal → proyeccion compactada → fuente autorizada;
decision propuesta/editada/rechazada separada del recibo y tipo de score
visible en retrieval. UI solo consume contratos A/B/C/F y respeta ACL;
no debe hacer dispatch al abrir una traza ni restaurar secretos borrados.

## 11. Fuentes, cobertura y division futura

Rangos efectivamente examinados y evidencia recuperada del antecedente:

- [C1][C13]: resumen, corte seguro y errores; [C2]: edicion tools.
- [C3]: HITL batch; [C5]: entrada/salida history; [C6]: relevancia.
- [C4][C7]: advertencias preservadas del documento en HEAD, no auditoria legacy.
- [C8][C12]: runner y dependencias; [C9]: firmas factory.
- [C10][C11]: limites e implementacion retry; [V1]–[V5]: seams locales.
- Externas leidas: cuerpos API de issues #41323/#38867, comentarios #41323,
  pagina oficial Studio. PR #39268 citado por reportes, diff no inspeccionado.

**Lagunas:** resto de proveedores, model_call_limit/PII y otros middleware,
  fabrica completa, sockets/cancelacion de cada proveedor, suite de conformidad,
transacciones backend y frontend SaaS. No se aprueba seguridad global.
Cobertura de los diez ejes: recomendaciones/fichas 1–2; issues 3; prerequisitos
4; alternativas 5; metodo 6; benchmark 7; matriz 8; negativos 9; UI 10.
Validaciones funcionales de cada ficha quedan manuales, sin ejecutar aqui.
Revision adversarial independiente pendiente del responsable.
LC-1/LC-2 propagados a flujo/F, FMEA, aceptación y matriz: la conformidad
de mutación y touch permanece futura; no hay nuevo bug global ni aprobación.

**Division futura (>300 lineas):** si se convierten en specs, separar contexto
causal (A/B), contratos de invocacion (C/D/E) y conformidad retrieval (F),
con ledger comun de evidencia; no crear ahora archivos ni indices adicionales.

[C1]: https://github.com/langchain-ai/langchain/blob/d9caf083577c43a91c5bf6bd1f7dfc289581184d/libs/langchain_v1/langchain/agents/middleware/summarization.py#L485-L525
[C2]: https://github.com/langchain-ai/langchain/blob/d9caf083577c43a91c5bf6bd1f7dfc289581184d/libs/langchain_v1/langchain/agents/middleware/context_editing.py#L79-L155
[C3]: https://github.com/langchain-ai/langchain/blob/d9caf083577c43a91c5bf6bd1f7dfc289581184d/libs/langchain_v1/langchain/agents/middleware/human_in_the_loop.py#L759-L850
[C4]: https://github.com/langchain-ai/langchain/blob/d9caf083577c43a91c5bf6bd1f7dfc289581184d/libs/core/langchain_core/chat_history.py#L23-L60
[C5]: https://github.com/langchain-ai/langchain/blob/d9caf083577c43a91c5bf6bd1f7dfc289581184d/libs/core/langchain_core/runnables/history.py#L522-L580
[C6]: https://github.com/langchain-ai/langchain/blob/d9caf083577c43a91c5bf6bd1f7dfc289581184d/libs/core/langchain_core/vectorstores/base.py#L450-L540
[C7]: https://github.com/langchain-ai/langchain/blob/d9caf083577c43a91c5bf6bd1f7dfc289581184d/libs/langchain/langchain_classic/memory/chat_memory.py#L17-L44
[C8]: https://github.com/langchain-ai/langchain/blob/d9caf083577c43a91c5bf6bd1f7dfc289581184d/libs/core/tests/benchmarks/test_tool_schema_conversion.py#L1-L69
[C9]: https://github.com/langchain-ai/langchain/blob/d9caf083577c43a91c5bf6bd1f7dfc289581184d/libs/langchain_v1/langchain/agents/factory.py#L824-L915
[C10]: https://github.com/langchain-ai/langchain/blob/d9caf083577c43a91c5bf6bd1f7dfc289581184d/libs/langchain_v1/langchain/agents/middleware/tool_call_limit.py#L308-L388
[C11]: https://github.com/langchain-ai/langchain/blob/d9caf083577c43a91c5bf6bd1f7dfc289581184d/libs/langchain_v1/langchain/agents/middleware/tool_retry.py#L295-L413
[C12]: https://github.com/langchain-ai/langchain/blob/d9caf083577c43a91c5bf6bd1f7dfc289581184d/libs/langchain_v1/pyproject.toml
[C13]: https://github.com/langchain-ai/langchain/blob/d9caf083577c43a91c5bf6bd1f7dfc289581184d/libs/langchain_v1/langchain/agents/middleware/summarization.py#L746-L1005
[V1]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/integrations/langchain/vantadb_langchain/vectorstore.py#L254-L576
[V2]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/context_engine/engine.rs#L71-L266
[V3]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/utils/task_checkpoint.rs#L122-L188
[V4]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/record/approval.rs#L1-L168
[V5]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/record/l1_writer.rs#L176-L420

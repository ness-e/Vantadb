---
title: "LangGraph: persistencia de ejecucion y memoria separadas"
kind: research
status: proposed
description: "Contratos de checkpoint, replay, interrupcion y store contrastados con las integraciones existentes de VantaDB"
tags: [research, memory, engineering]
---

# LangGraph: persistencia de ejecucion y memoria separadas

## Indice

- [1. Dictamen y arquitectura](#1-dictamen-y-arquitectura)
- [2. Flujo y fichas candidatas](#2-flujo-y-fichas-candidatas)
- [3. Riesgos y evidencia](#3-riesgos-y-evidencia)
- [4. Prerrequisitos](#4-prerrequisitos)
- [5. No trasladar](#5-no-trasladar)
- [6. Metodo y cobertura](#6-metodo-y-cobertura)
- [7. Benchmarks existentes](#7-benchmarks-existentes)
- [8. Matriz frente a VantaDB](#8-matriz-frente-a-vantadb)
- [9. Debilidades sugeridas](#9-debilidades-sugeridas)
- [10. Interfaz e inspeccion](#10-interfaz-e-inspeccion)

## 1. Dictamen y arquitectura

**Propuesta:** priorizar conformidad del adaptador LangGraph existente, no
incorporar un orquestador Python ni recrear Pregel dentro del motor Rust.
El contraste mas util es el contrato de escrituras intermedias y reanudacion.

- Repo: `langchain-ai/langgraph`, carpeta `C:\Users\Eros\langgraph`.
- HEAD local: `e5e37b5d70dbfd93c7006bd90a658c6179a95946`.
- Snapshot: **44 598 estrellas**, MIT, no archivado, segun
  [inventario congelado](github-candidates.json).
- Verificacion: HEAD resoluble, indice poblado y `status --porcelain -uno`
  sin cambios al comenzar la lectura; no se estudio un checkout incompleto.
- Stack observado: Python, LangChain Core, canales/reducers, runtime Pregel,
  serializers tipados, adaptadores SQLite/PostgreSQL y SDK HTTP separado.

**Codigo:** hay dos contratos distintos:

1. `BaseCheckpointSaver`: estado de ejecucion por thread, namespace y checkpoint;
   versiones de canales, padres, metadata y pending writes.
2. `BaseStore`: objetos JSON por tupla namespace/key, operaciones de busqueda,
   filtros e indexacion opcional. Es memoria entre ejecuciones, no cursor de
   ejecucion ni autorizacion por si mismo.

Una conversacion puede residir en un canal de mensajes. No todo checkpoint es
memoria semantica y un store no garantiza reanudar efectos externos.

## 2. Flujo y fichas candidatas

### Flujo observado

`StateGraph -> nodos -> writes por tarea -> canales/reducers -> checkpoint
de superstep -> saver`. El runtime guarda errores e interrupciones como writes
de control. `CheckpointTuple` devuelve checkpoint, padre y pending writes.
`PostgresSaver.put` separa valores primitivos inline de blobs versionados;
`put_writes` usa claves de tarea/posicion y distingue canales especiales.

`interrupt(value)` exige checkpointer y reanuda **desde el principio del nodo**.
La correspondencia entre multiples interrupciones usa el orden dentro de la
tarea. Un efecto antes del interrupt puede ejecutarse otra vez. La aprobacion
del usuario no equivale a una transaccion con el servicio externo.

`BinaryOperatorAggregate` aplica reducer a updates; `Overwrite` evita el
reducer y rechaza multiples overwrites en un step. El orden, identidad de
mensajes y serializacion del marcador forman parte del contrato observable.
Guardar una lista JSON sin esos detalles no implementa equivalencia de replay.

### Ficha A: escrituras intermedias con identidad estable

- **Objetivo:** mejorar la especificacion/conformidad del checkpointer ya
  existente frente a reintentos, errores y actualizaciones de control.
- **Unidad minima:** `PostgresSaver.put_writes`, lineas 352-384 [L2], SQL de
  conflicto, lineas 134-162 [L3], y `PregelLoop.put_writes`, 463-535 [L4].
  No extraer el scheduler completo.
- **Seam existente:** `integrations/langchain/vantadb_langchain/checkpointer.py`,
  `VantaDBCheckpointer.put/put_writes`, lineas 232-296; checkpoints Rust en
  `vanta-memory/src/utils/task_checkpoint.rs`, `TaskCheckpointManager`,
  lineas 122-188. Son dominios distintos que no deben fusionarse a ciegas.
- **Contrato Rust propuesto:** `PendingWriteKey { tenant, thread, namespace,
  checkpoint, task, slot }` y operacion `persist_writes(batch, conflict_policy)`.
  El slot especial debe derivarse del canal y de la version del protocolo;
  el ordinario conserva posicion. Es una propuesta, no API actual.
- **Pasos/dependencias:** fijar version soportada del paquete; documentar
  semantica insert/replace; comprobar atomicidad disponible del SDK; definir
  migracion y probar manualmente conformidad upstream en instancia desechable.
- **FMEA:** read-before-write no atomico, cambio de slots al filtrar controles,
  carrera entre workers y perdida de un error actualizado. Requiere comparar
  revision actual de Vanta: no se declara bug reproducido.
- **Aceptacion:** repetir un batch no duplica updates ordinarios; reemplazos
  de control son los previstos por esa version; namespace/thread no colisionan;
  reinicio entre batch y snapshot conserva las tareas ya persistidas.
- **No objetivos:** exactly-once de herramientas ni nuevo workflow engine.

### Ficha B: contrato explicito de replay y aprobacion

- **Objetivo:** que una reanudacion distinga pendiente, aprobada y aplicada
  sin confundir persistencia de decision con ejecucion del efecto.
- **Unidad minima:** `interrupt`, lineas 913-935 y 1023-1049 [L5];
  `PregelRunner.commit`, 575-615 [L6].
- **Seam:** `TaskCheckpointManager` y scheduler ya existentes; adaptador
  Python para la semantica LangGraph, no reinterpretacion de nodos en Rust.
- **Contrato Rust propuesto:** `ApprovalRecord { operation_id, input_digest,
  decision_revision, actor, status }`; `resume` valida identidad/revision.
  El ejecutor externo recibe una clave idempotente y registra su recibo.
- **Pasos/dependencias:** separar intencion, decision y recibo; fijar quien
  autentica al aprobador; enlazar provenance/checkpoint; acordar recuperacion
  cuando el remoto acepta pero no llega el recibo.
- **FMEA:** repetir pago/envio, aprobacion sobre input modificado, orden de
  interrupts cambiado por una nueva version y tenant incorrecto. Si el remoto
  no soporta idempotencia, declarar resultado desconocido y reconciliar.
- **Aceptacion:** crash antes/despues de aprobacion y antes/despues de recibo
  no presenta un efecto no confirmado como completado; decision obsoleta
  rechazada; el replay conserva identidad de operacion y auditoria.
- **No objetivos:** prometer exactly-once distribuido ni adoptar Studio.

### Ficha C: capacidades declaradas de store y serde

- **Objetivo:** evitar que el adaptador acepte opciones que no puede preservar.
- **Unidad minima:** `SearchOp` y `PutOp` [L7], `index` en 490-520;
  `BinaryOperatorAggregate.update`, 107-150 [L8]; fallback de reconstruccion
  de `JsonPlusSerializer`, 83-101 [L9].
- **Seam:** `VantaDBStore.batch/_search_semantic`, lineas 97-139 y 231-271
  de `integrations/langchain/vantadb_langchain/store.py`; context engine
  existente para budgets, sin asignar un presupuesto independiente al adapter.
- **Contrato Rust propuesto:** `StoreCapabilities { semantic, ttl,
  field_indexing, ordered_batch }` y envelope `codec_version/type/payload`.
  Namespace es una tupla sin colisiones; ACL se aplica antes de buscar.
- **Pasos/dependencias:** enumerar opciones del contrato upstream congelado;
  traducir filtros soportados; rechazar las restantes; preservar tipo/version
  y distinguir payload recuperable de objeto reconstruido.
- **FMEA:** namespace como prefijo textual ambiguo, serializer con clases
  peligrosas, filtros post-top-k que reducen recall y perdida silenciosa de
  tipos. No se afirma que todo ello ocurra en Vanta.
- **Aceptacion:** round-trip de namespace y mensajes; rechazo de codec/tipo
  desconocido con diagnostico; busqueda nunca cruza tenant; opciones no
  implementadas fallan explicitamente, no se ignoran.
- **No objetivos:** portar clases Python ni reducers arbitrarios a Rust.

## 3. Riesgos y evidencia

| Evidencia | Observacion | Consecuencia transferible |
|---|---|---|
| Codigo | `interrupt` documenta reejecucion del nodo [L5] | Idempotencia del efecto es responsabilidad aparte |
| Codigo | `put_writes` y `put` son entradas distintas [L2] | Snapshot no prueba commit atomico con todo efecto |
| Codigo | `durability="exit"` difiere writes/checkpoints [L4] | Medir ese modo no certifica recuperacion tras crash |
| Codigo | Codec retorna payload cuando no reconstruye [L9] | Preservar bytes no garantiza igualdad semantica del objeto |
| Hipotesis | Cambiar reducer/codigo entre checkpoint y resume | Necesita migracion/version y reproduccion manual |

Los SQL de conflicto proporcionan deduplicacion de escrituras del saver, no
una prueba de exactly-once extremo a extremo. No se inspeccionaron todas las
fronteras transaccionales de psycopg ni se produjo un crash real.
**Issues upstream: no consultados**; no se atribuye CVE ni incidencia vigente.

## 4. Prerrequisitos

Revalidar el worktree Vanta, que cambia concurrentemente, y las versiones
publicadas de sus adapters. Definir tenant/thread/namespace y politica de
retencion/borrado antes de ampliar semanticas. La memoria derivada, los
checkpoints y los recibos de efectos necesitan ciclos de vida diferentes.
Mantener LLM/embeddings opcionales en las primitivas de persistencia.
Reutilizar provenance, dedup, scheduler y presupuesto compartido existentes.

## 5. No trasladar

No traducir Pregel ni construir otro sistema de agentes dentro del motor.
No usar checkpointer como indice semantico de toda la conversacion ni namespace
como autenticacion. No almacenar/reconstruir objetos Python no confiables.
No extender al embebido dependencias de servidor/Studio. MIT exige revisar
copyright, notices y dependencias antes de reutilizar; reescribir en Rust no
exime de una posible derivacion.

## 6. Metodo y cobertura

Rige [METHOD](METHOD.md), incluido baseline Vanta
`c561158253bb88192590fa406c61e0a27dd7df8d`. Lectura estatica de los rangos
enlazados; no ejecucion, dependencias, scripts, tests ni servidores.

Ledger de archivos fuente **examinados**, con URL inmutable:

- [L1: checkpoint/base, tipos y contrato, 93-147 y 215-360](https://github.com/langchain-ai/langgraph/blob/e5e37b5d70dbfd93c7006bd90a658c6179a95946/libs/checkpoint/langgraph/checkpoint/base/__init__.py#L93-L360).
- [L2: PostgresSaver.put/put_writes, 275-384](https://github.com/langchain-ai/langgraph/blob/e5e37b5d70dbfd93c7006bd90a658c6179a95946/libs/checkpoint-postgres/langgraph/checkpoint/postgres/__init__.py#L275-L384).
- [L3: SQL de conflicto, 134-162](https://github.com/langchain-ai/langgraph/blob/e5e37b5d70dbfd93c7006bd90a658c6179a95946/libs/checkpoint-postgres/langgraph/checkpoint/postgres/base.py#L134-L162).
- [L4: PregelLoop writes, 463-535; snapshots, 1300-1360](https://github.com/langchain-ai/langgraph/blob/e5e37b5d70dbfd93c7006bd90a658c6179a95946/libs/langgraph/langgraph/pregel/_loop.py#L463-L535).
- [L5: interrupt, 913-1049](https://github.com/langchain-ai/langgraph/blob/e5e37b5d70dbfd93c7006bd90a658c6179a95946/libs/langgraph/langgraph/types.py#L913-L1049).
- [L6: PregelRunner.commit, 575-630](https://github.com/langchain-ai/langgraph/blob/e5e37b5d70dbfd93c7006bd90a658c6179a95946/libs/langgraph/langgraph/pregel/_runner.py#L575-L630).
- [L7: store/base, 160-240 y 475-520](https://github.com/langchain-ai/langgraph/blob/e5e37b5d70dbfd93c7006bd90a658c6179a95946/libs/checkpoint/langgraph/store/base/__init__.py#L160-L240).
- [L8: binop, 35-150](https://github.com/langchain-ai/langgraph/blob/e5e37b5d70dbfd93c7006bd90a658c6179a95946/libs/langgraph/langgraph/channels/binop.py#L35-L150).
- [L9: jsonplus, 40-102](https://github.com/langchain-ai/langgraph/blob/e5e37b5d70dbfd93c7006bd90a658c6179a95946/libs/checkpoint/langgraph/checkpoint/serde/jsonplus.py#L40-L102).
- [L10: runner de microbench, 1-180](https://github.com/langchain-ai/langgraph/blob/e5e37b5d70dbfd93c7006bd90a658c6179a95946/libs/langgraph/bench/__main__.py#L1-L180).

**No examinado:** implementacion completa de SQLite, store PostgreSQL,
conformance suite, todos los canales, cifrado, SDK remoto, despliegue ni UI
externa. Un archivo listado no implica lectura integral. Pendiente manual:
conformidad del adapter, concurrencia y crash/replay en backend desechable.

## 7. Benchmarks existentes

[L10] usa `pyperf.Runner`, `uvloop`, grafos secuenciales/fanout/wide-state y
ReAct; entradas sinteticas, UUID por thread e `InMemorySaver` opcional.
Funciones `run/arun` consumen stream completo; otras cortan al primer evento.
Se observan inputs aleatorios sin congelacion explicita en esos rangos.
**Metricas:** tiempo de ejecucion/primer evento/compilacion segun funcion;
no calidad de memoria, relevancia ni consistencia de una base persistente.
Dependencias: runtime Python y paquetes de benchmark; ese runner usa
`durability="exit"` y por tanto no representa fsync de cada superstep.
Efectos: CPU/RAM, acumulacion de estados en saver y salidas del runner.
No se ejecuto ni se atribuyen scores locales. Comparacion futura exclusivamente
por las pistas y controles de [METHOD](METHOD.md#protocolo-de-comparacion-de-benchmarks);
no se añade harness, juez, umbral ni configuracion.

## 8. Matriz frente a VantaDB

| Capacidad | Vanta | Diferencia que merece validar |
|---|---|---|
| BM25/HNSW/RRF y provenance | Existe | No reemplazar por store remoto |
| LangChain/LlamaIndex bindings | Existe | Extender conformidad, no recrear |
| LangGraph saver/store | Existe | `put_writes`, index/TTL y versiones soportadas |
| Sesiones y checkpoints de tareas | Existe | No equivalen automaticamente a canales Pregel |
| Scheduler, dedup y supersession | Existe | Integracion con recibos/replay no acreditada aqui |
| Context assembly/compactacion/shared budget | Existe | Preservar identidad de mensajes al compactar |
| Exactly-once con efectos externos | No acreditado | Tampoco lo prueba LangGraph |
| Runtime Pregel embebido | No aplicable | Orquestacion fuera del motor |

## 9. Debilidades sugeridas

El contraste sugiere documentar mejor la **garantia** del adapter frente a
repeticion y capacidades no soportadas, y exponer diagnosticos de checkpoint,
decision y efecto. No demuestra ausencia de checkpoints, budgets o dedup.
La diferencia visible entre claves ordinarias y especiales es una pregunta
de conformidad prioritaria, no un bug nuevo certificado del worktree.

## 10. Interfaz e inspeccion

Runtime y savers Python son fuente local. El SDK HTTP aparece en el arbol,
pero no se inspecciono su backend. **UI grafica/Studio: no localizada como
implementacion local en las rutas revisadas**; una integracion con LangSmith
no vuelve OSS a su dashboard. Escritorio nativo: no localizado. No se inicio
ningun servidor. Vanta Studio ya existe en `develop-desktop`; la oportunidad
seria inspeccionar linaje/replay y aprobaciones alli, no crear otra aplicacion.

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
- [11. Ledger complementario y cierre de autoria](#11-ledger-complementario-y-cierre-de-autoria)

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

### Mapa, modelos y fronteras de persistencia

Ampliacion conservadora del informe existente, con referencia Vanta fija
`97c9623c8ef5a70fdb51aff215bfbb3b05b15612`; no se recalculan inventarios.

| Capa/ruta | Modelo y responsabilidad | Frontera |
|---|---|---|
| `libs/langgraph/langgraph/graph/`, `channels/` | Grafo, canales y reducers; `BinaryOperatorAggregate` [L8] | Runtime del host, no storage Vanta |
| `pregel/_loop.py`, `_runner.py` | Supersteps, tareas, writes, interrupcion/error y snapshots [L4][L6] | Decide cuando llamar al saver; no transaccion con tools externas |
| `libs/checkpoint/.../checkpoint/base/` | `CheckpointTuple`, `channel_values/channel_versions/versions_seen`, metadata, parent y pending writes [L1] | Checkpoint serializable no basta sin codigo/reducers compatibles |
| `checkpoint/serde/jsonplus.py` | Tipo + bytes, reconstructores/allowlist [L9][L13] | Deserializacion Python privilegiada; no entrada de usuario |
| `checkpoint-postgres/` | Checkpoints, blobs versionados, writes y SQL de conflicto [L2][L3] | PostgreSQL/psycopg opcionales, no requisitos del embebido |
| `checkpoint-sqlite/` | Tablas checkpoints/writes, lock y cursor [L12] | Lock de instancia no demuestra aislamiento entre procesos |
| `store/base/`, `store/memory/` | `Item`, `SearchItem`, `GetOp/PutOp/SearchOp/ListNamespacesOp`; JSON por namespace/key [L7][L11] | KV/retrieval entre runs, separado de replay |
| `libs/sdk-py`, `libs/sdk-js`, `libs/cli` | Clientes/arranque de Agent Server | Servidor y UI externos no se infieren del SDK |

Un `Item` tiene key, tuple namespace, value y timestamps; `SearchItem` añade
score. Embeddings/indexacion son opt-in del store, no necesarios para
checkpoint. Versions de canal no son revision transaccional de la memoria
del usuario ni control de acceso. El `thread_id` selecciona ejecucion: el host
debe vincularlo a un principal autorizado, no aceptarlo como credencial.

**Orden de inversion:** P0 fijar compatibilidad/garantias del adapter; P1 A
(writes), D (visibilidad de batch) y E (borrado); P1 B si hay efectos sensibles;
P2 C/F (capacidades/serde y linaje). No son dependencias universales ni nuevas
funciones implementadas.

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

### Recorrido concreto del adapter Vanta existente

Lectura del snapshot fijo, no equivalencia certificada:

- `VantaDBCheckpointer.put` [V6] serializa checkpoint completo y metadata a
  `{type,data(base64)}`, conserva parent ID y hace un `Client.put`. Los valores
  estan inline: no se necesita copiar el blob split PostgreSQL para cumplir
  identidad. Constantes reales `langgraph.checkpoints` y `langgraph.writes`;
  el comentario de cabecera con `langgraph:...` no es el valor ejecutado.
- `_ckpt_key` usa array JSON `[thread, namespace, checkpoint]`; no hay que
  «arreglar» escapar `/` en estas claves. `get_tuple/list` escanean/paginan,
  ordenan IDs y reconstruyen `CheckpointTuple`; `_writes_for` carga writes
  por thread y prefijo de clave. No se acredita lectura snapshot entre ambas
  familias ni orden total de resultados concurrentes.
- `put_writes`, 269–296, enumera posiciones y hace get-before-put/skip para
  **todo canal**. El referente [L3] distingue `WRITES_IDX_MAP` y upsert de
  controles. Es diferencia estatica de protocolo en esta referencia, no
  reproduccion ni prueba sobre versiones posteriores del adapter.
- `delete_thread`, 298–313, pagina y borra ambos namespaces por thread.
  No toca `BaseStore`, trazas, archivos de herramientas ni proveedor LLM.
  Reiniciar/cancelar puede dejar un subconjunto: no declara transaccion global.
- `VantaDBStore` [V7] valida segmentos no vacios/sin `/`, compara prefijos
  **por tupla** y soporta suffix/wildcards normalizados para listar namespaces.
  Por ello una colision textual `user/1`–`user/12` no esta demostrada.
- `_put` guarda dict JSON, copia escalares a metadata y, si se configuro
  embedder, vectoriza todo el payload antes de `Client.put`. `_get/_delete`
  usan namespace/key. `_search` lista todas las paginas; la variante semantica
  obtiene `limit+offset` por namespace, fusiona y ordena cosine clamped [0,1].
  No es BM25/RRF ni probabilidad de verdad.
- `batch` aplica operaciones secuencialmente; `PutOp.ttl` no nulo ya produce
  `NotImplementedError`. No se observa que pase `op.index` a `_put`. No
  presentar TTL silencioso como defecto existente: el delta es index/capability
  y visibilidad de batch. `abatch/aput/...` son wrappers sync; no garantizan
  offload, concurrencia no bloqueante ni cancelacion del trabajo nativo.

**Versiones antes de conformidad:** [V8] declara Core `>=0.3,<1`, checkpoint
`>=2,<5` y Python `>=3.11`. El LangChain moderno del clon separado requiere
Core `>=1.7.5,<2`. Esos rangos Core no intersectan. Esto no invalida el adapter
para su familia declarada; impide prometer compatibilidad con el agente moderno
sin una migracion explicita. No se resolvieron ni instalaron dependencias.

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

**Cierre A — ownership y recuperacion:** el runtime es owner de task/channel
e indice especial; el adapter no reasigna slots tras filtrar mensajes. La
unidad de commit debe declararse por batch o por write con estado parcial
recuperable. Serializar primero evita un error de codec a mitad del bucle;
no resuelve un fallo de IO posterior. Cancelar no implica rollback de writes
confirmados. Alternativa minima: mantener protocolo Python y ampliar pruebas
de conformidad; Rust solo si falta una primitiva real de persistencia. Migrar
claves ordinarias/especiales exige version de formato, lectura antigua y
desactivacion sin reinterpretar slots historicos. IDs de task/checkpoint y
codigo de fallo bastan para diagnostico; no volcar valores sensibles.

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
  Caso LG-1 futuro: checkpoint creado **antes de retirar una fuente o revocar
  permiso**; autorizar acceso al checkpoint antes de decode y revalidar en
  host antes de resume, recall/nueva entrega al modelo y efecto. Resultado
  permitido sólo para datos aún autorizados; denegación o redacción explícita
  para lo revocado, nunca reinyección automática desde la copia histórica.
- **No objetivos:** prometer exactly-once distribuido ni adoptar Studio.

**Cierre B:** preferir `interrupt/Command` upstream con registro de decision
en el host. La cola Vanta de aprobacion de capturas [V9] ya existe, pero no es
aprobacion durable de una tool. Compartir referencias/provenance, no asumir
intercambiabilidad. Antes de dispatch se valida revision/claim; tras dispatch
la cancelacion solo bloquea intentos nuevos. Migracion aditiva deja decisiones
antiguas sin recibo como desconocidas; rollback deshabilita dispatch, no borra
auditoria ni revierte una operacion remota. A y F conservan task/parent IDs;
B no depende de inventar un scheduler o un motor de agentes.

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

**Precision de C:** `ordered_batch` significa declarar el modelo de visibilidad,
no imponer read-your-writes como contrato universal; vease D. En [L13],
`pickle_fallback=False` por defecto **no** implica seguridad general:
msgpack es permisivo por defecto salvo strict/allowlist. Vanta delega serde al
paquete instalado [V6]; fijar version, politica y escritor autorizado es
prerrequisito. La migracion debe probar lectura de checkpoints viejos, mantener
bytes originales y rechazar tipos incompatibles antes de ejecutar nodos.
Cancelled/error de decode no se representa como checkpoint vacio exitoso.
Alternativa: envelope opaco conservado en Rust y decode solamente en host
autorizado. No sustituir reducers Python por JSON ni reindexar memoria por
cambiar la allowlist de checkpoints.

### Ficha D: visibilidad de batch e indexacion selectiva

- **Caso:** un agente guarda preferencia y la consulta en un batch; no debe
  obtener un resultado distinto por asumir que todos los stores aplican en
  orden. Ademas `index=False` no debe filtrar campos secretos al embedder.
- **Fuente minima:** `InMemoryStore.batch/abatch`, 206–234,
  `_prepare_ops`, 375–402, `_extract_texts`, 418–440 [L11].
  Reads/search se preparan antes de aplicar puts; la ultima operacion por
  namespace/key queda en el map. Vanta recorre secuencialmente [V7].
  El referente es una implementacion, no prueba de garantia universal BaseStore.
- **Delta/seam:** `VantaDBStore.batch/_put`, no un nuevo KV. Contrato Rust
  **propuesto**, si hace falta: `StoreBatchPlan { read_revision, mutations,
  index_policy } -> BatchReceipt { applied_ids, visibility, projection_state }`;
  errors `UnsupportedIndexPolicy/Conflict/PartialCommit`.
- **Pasos:** fijar semantica de la version soportada → caracterizar
  put/get/delete/search en un batch → validar todas las capacidades antes de
  mutar → seleccionar campos antes de embedding → aplicar primario y registrar
  estado de proyeccion. Reindexacion solo de registros cuya politica cambie;
  no enviar retrospectivamente campos excluidos.
- **Alternativas/deps:** documentar diferencias permitidas o rechazar
  operaciones compuestas no soportadas; no copiar InMemoryStore como DB.
  C declara capacidades; A no sustituye este contrato de memoria.
- **Ownership/concurrencia/commit:** host valida scope e index policy;
  modelo no elige ACL. Cancelar embedding antes de put no debe crear valor;
  un batch parcialmente aplicado se informa, no se «deshace» inventando
  transaccion. Revision/single-writer si el SDK no proporciona CAS.
- **FMEA:** index ignorado → PII al proveedor; detectar llamadas del embedder
  con campos centinela y mitigar validando antes. Batch divergente → contexto
  viejo/nuevo inesperado; detectar con secuencia mixta y declarar modelo.
- **Aceptacion manual futura:** index False/lista, fallo de embedding,
  dos writers y put/get/delete conservan el contrato declarado; logs por IDs
  y politica, no texto. **No objetivos:** otro ANN, TTL nuevo ni runtime.

### Ficha E: borrado de thread con alcance y cierre verificables

- **Caso:** retirar una conversacion no debe dejar writes reactivables ni
  borrar memorias compartidas legitimamente por otros threads.
- **Fuente:** `SqliteSaver.delete_thread`, 498–515 [L12]; adapter Vanta
  `delete_thread`, 298–313 [V6]. Checkpoints y writes son familias distintas.
- **Delta/seam:** mejorar ciclo de vida del saver existente; usar lifecycle/
  provenance de `vanta-memory`, sin convertir delete_thread en borrado global.
  Contrato Rust **propuesto** `ThreadDeletion { scope, thread, generation,
  families } -> DeletionReceipt { complete, remaining, blocked_writers }`;
  errores `Unauthorized/GenerationConflict/Partial`.
- **Pasos:** autenticar → cerrar generacion o serializar con writers →
  enumerar familias → borrar reanudablemente → verificar ausencia →
  emitir recibo. Dependencias: A define keys, F linaje; retencion legal
  de auditoria/recibos es politica aparte. No borrar BaseStore sin referencias
  y permiso explicitos; backups/exportaciones tienen su propio lifecycle.
- **Alternativa:** bloqueo de thread en host y procedimiento de mantenimiento,
  si no existe primitiva transaccional. Version aditiva del recibo; cancelar
  deja progreso reanudable; no restaurar contenidos borrados al desactivar.
- **FMEA:** paginar mientras se borra/sigue escribiendo → restos; detectar
  reescaneo y generation guard. Thread ID ajeno → borrado cruzado; validar
  principal fuera del LLM. Error parcial nunca es borrado completo.
  Retirar un hecho de BaseStore no sanea checkpoints anteriores: families
  distintas ([V6][V7]). El host posee política/retención y acceso a la copia;
  E no convierte delete_thread en borrado global de fuentes/backups.
- **Aceptacion futura:** mas de una pagina, crash/cancelacion y writer tardio
  no dejan thread reanudable tras recibo completo; otro thread/store intacto.
  Metricas de familias/IDs, no payload. **No objetivos:** borrado certificado
  de todos los servicios externos ni versionado nuevo de Pregel.
  Añadir caso futuro LG-1: fuente retirada/permiso revocado con checkpoint
  previo retenido. Aunque el thread no se borre por política de auditoría,
  su recall/entrega no reinyecta contenido revocado; denegar resume o
  redactar explícitamente según host. No es una fuga reproducida.

### Ficha F: linaje y modo de durabilidad como datos de inspeccion

- **Caso/fuente:** un operador distingue reanudar el ultimo estado, replay
  desde un padre y una nueva rama; `CheckpointTuple` [L1], parent guardado
  por `put` [V6], y modos de persistencia en loop [L4].
- **Delta/seam:** exponer al adapter/Studio lo ya persistido, no implementar
  time travel en Rust. Contrato **propuesto** `CheckpointView { thread,
  namespace, checkpoint, parent, graph_revision, codec_revision,
  durability_mode, pending_task_ids }`, desconocido explicito donde falte.
- **Pasos/deps:** documentar campos disponibles → registrar versiones
  del host/mode → verificar compatibilidad antes de resume → consultar
  linaje paginado con ACL → mostrar separado de hechos/decisiones/recibos.
  A/B/C aportan identidades; no exigir embedding. Migracion aditiva no inventa
  revision de grafo historica; rollback solo oculta vista nueva.
- **Ownership/commit/cancelacion:** host decide rama/durability y confirma
  snapshots; UI es lectora por defecto. Cancelar consulta no cancela workflow;
  seleccionar checkpoint no despacha tools. Alternativa: inspector de metadatos
  en Python mientras el Rust conserva envelopes opacos.
- **FMEA:** modo exit rotulado durable por superstep → falsa recuperacion;
  detectar modo efectivo. Reducer/codigo cambiado → replay divergente;
  detectar revision incompatible y pedir migracion explicita.
  Checkpoint anterior conserva copia de hecho retirado → rehidratación no
  autorizada → revalidar en host, no tratar checkpoint como fuente factual.
- **Aceptacion futura:** padre/namespace y rama visibles sin revelar payload
  ajeno; revision desconocida no afirma replay equivalente. Sin ejecucion aqui.
  Reanudar padre previo a revocación/retirada prueba política LG-1 de B/E:
  acceso antes de decode y filtro/redacción/denegación antes de recall,
  nueva entrega o efecto, incluso si el grafo/codec son compatibles.
  **No objetivos:** determinismo de LLM, undo remoto o duplicar Studio.

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
### Incidencias publicas efectivamente leidas

Consulta del 2026-11-07; reportes upstream, no reproducciones propias:

| Fuente/estado confirmado | Mecanismo y alcance | Decision para Vanta |
|---|---|---|
| [#9563, SQLite saver put_writes can partially persist a failed batch](https://github.com/langchain-ai/langgraph/issues/9563), **Open**, abierto 2026-11-03 | Reporta 3.1.2/3.1.3: ABORT en segundo insert deja prefijo; sync commit en finally, async publica despues con otro commit; incluye cancelacion. Comentario propone savepoint y regresiones, no acredita merge | [L12] conserva `executemany` y `cursor` con commit en finally sin rollback en ese helper. Aplicar A y E como preguntas discriminantes; Vanta usa otro backend, no hereda el bug SQLite |
| [#8855, _scratchpad rescans pending_writes once per task](https://github.com/langchain-ai/langgraph/issues/8855), **open**, creado 2026-09-08 segun API publica | Reporta dos scans por tarea, coste O(tasks × writes), medidas del autor contra `81bf17b` y propuesta de indice por step | [L15] conserva los scans para resume global y task; las cifras no son nuestras. El adapter puede evitar scans propios innecesarios, pero no se propone portar/optimizar Pregel en Rust |

Se leyo cuerpo completo #9563 mediante HTML GitHub tras limite de API; para
#8855 se leyo el cuerpo devuelto por busqueda oficial API. No se inspeccionaron
PRs/diffs vinculados ni artefactos experimentales externos. Comentarios con
«reproduced/passed» no certifican resultado propio ni release corregido.
No se asigna CVE, gravedad numerica ni afirmacion general sobre todos los savers.

**Registro de acceso:** DuckDuckGo `site:github.com/langchain-ai/langgraph/issues
checkpoint pending writes serializer` devolvio 0; busqueda API `pending writes`
dio candidatos. GET directo API #9563 por PowerShell dio 403 y fetchWebContent
informo rate-limit; HTML GitHub sirvio de fallback. Para Agno, Bing/metasearch
tampoco dio candidatos: ninguna ausencia en buscadores se interpreta como
ausencia de incidencias.

**Seguridad:** [L13] advierte que un escritor directo en checkpoint DB puede
alcanzar ejecucion de codigo al deserializar. Separar namespace de checkpoint
de escritura de memoria no confiable y restringir acceso al archivo/servidor;
base64 no cifra ni autentica. La allowlist pertenece al host Python, no a un
LLM ni a un codec Rust inventado. No se audito cifrado/key management.

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

Rigen [METHOD](METHOD.md), [BENCHMARK-NOTES](BENCHMARK-NOTES.md) y
[EXPANSION-CONTRACT](EXPANSION-CONTRACT.md). Baseline comparativo:
`97c9623c8ef5a70fdb51aff215bfbb3b05b15612`; `c561158` es antecedente,
no estado actual ni prueba de un defecto vigente. Lectura estatica de los rangos
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

**No examinado:** SQLite completo (solo cursor/writes/delete), store PostgreSQL,
conformance suite, todos los canales, cifrado, SDK remoto completo, despliegue ni UI
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

### Defaults, denominadores y defectos del runner

La ampliacion sigue `bench/__main__.py` **completo**, 1–520, y el caso ReAct
[L14], no solo el fragmento inicial [L10]:

| Aspecto | Evidencia estatica |
|---|---|
| Entry point | Modulo `bench.__main__` → tabla de graphs → `Runner.bench_async_func/bench_func`; compilacion aparte, 499–516; serde allowlist, 518–520 |
| Unidad/dataset | Una invocacion completa de grafo; fanout con 100 sujetos de 1 000 caracteres aleatorios; sequential 10/1000; estados anchos/dicts/Pydantic y ReAct 10/100 tools. No corpus de preguntas ni split |
| Preparacion | Graphs se construyen/compilan al formar tabla; UUID thread por invocacion; generador random sin semilla explicita en archivo. No extrapolar cache/split reproducible |
| Modelo/tools | ReAct usa `FakeMessagesListChatModel`, secuencia sintetica de calls/resultados y tool lambda; no answerer/juez real, embeddings, tokenizer ni ground truth |
| Medicion | `run/arun` materializan stream completo; tiempo por invocacion de graph, no por nodo ni pregunta. pyperf fija procesos/rondas/calibracion/warmup: valores efectivos no fijados en este archivo |
| Durabilidad | `durability="exit"` y saver ausente o `InMemorySaver`; excluye fsync de backend persistente, network DB y recuperacion crash por superstep |
| Errores/vacios | No wrapper de puntuacion ni descarte estadistico propio; excepcion interrumpe benchmark. Input vacio sequential es caso sintetico, no abstencion semantica |
| Concurrencia | Fanout interno no equivale a numero de usuarios concurrentes; funciones async y sync se registran separadamente; no se infiere throughput multitenant |
| Efectos | CPU/RAM, nuevo estado por UUID en saver reutilizado; no reset explicito por llamada. Artefactos/opciones de pyperf dependen de CLI; no se ejecutaron ni descargaron datos |

**Hallazgo discriminante:** 478–485 define nombres strings en
`GRAPHS_FOR_1st_EVENT_LATENCY`, pero comprueba `if graph not in ...`, donde
`graph` es el objeto compilado, no `name`. Por lectura, esas entradas no se
registran normalmente. La existencia de helpers que cortan tras el primer
evento no demuestra una medida publicada de first-event latency. No se
modifica el runner ni se generaliza el defecto a sus medidas de graph completo.

**Adaptacion concreta:** el adapter Vanta ya satisface entradas de saver,
config y tuple; una comparacion de persistencia tendria que conservar
checkpoint/task IDs, mode efectivo y errores, separando conversion serde de
storage. Sustituir `InMemorySaver` por otro backend deja de medir el mismo
camino: no se etiqueta como mejora del runtime ni de memoria semantica.
Para retrieval/memoria se aplica el protocolo existente de METHOD/Vanta,
no este microbenchmark. No se diseña harness ni configuracion nuevos.

## 8. Matriz frente a VantaDB

| Capacidad | Vanta | Evidencia y diferencia que merece validar |
|---|---|---|
| BM25/HNSW/RRF y provenance | Existe | [VANTA-REFERENCE](VANTA-REFERENCE.md); no reemplazar por store remoto |
| LangChain/LlamaIndex bindings | Existe | Referencia y [V8]; conformidad versionada, no recrear |
| LangGraph saver/store | Existe | [V6][V7]; persistencia no implica equivalencia en todas las opciones |
| Slots especiales y reemplazo control | Parcial | [V6] usa indices ordinarios/skip frente a [L3]; A |
| TTL del adapter | No acreditado como capacidad | Ya rechazado explicitamente [V7], no ignorado |
| Indexacion por campos/index=False | Parcial | [L11] vs `_put/batch` [V7]; C/D, sin negar vectores existentes |
| Prefijos de namespace por segmento | Existe | [V7]; no proponer correccion de colision no demostrada |
| Sesiones y checkpoints de tareas | Existe | Referencia, `TaskCheckpointManager`; no son canales Pregel |
| Scheduler, dedup y supersession | Existe | Referencia; recibo externo durable no acreditado aqui |
| Aprobacion de capturas | Existe | [V9]; no equivale a decision/recibo B |
| Context assembly/compactacion/shared budget | Existe | Referencia; mantener IDs al compactar |
| Borrado de thread | Existe, cierre concurrente parcial | [V6]; E verifica generaciones, paginas y recibo |
| Permiso vigente al resume de copia histórica | **No acreditado end-to-end** | B/E/F: host revalida checkpoint/fuente antes de decode/resume/entrega; LG-1 es aceptación futura, no fuga probada |
| Exactly-once con efectos externos | No acreditado | [L5][L6]; tampoco lo prueba LangGraph |
| Runtime Pregel embebido | No aplicable | Orquestacion fuera del motor |

## 9. Debilidades sugeridas

El contraste sugiere documentar mejor la **garantia** del adapter frente a
repeticion y capacidades no soportadas, y exponer diagnosticos de checkpoint,
decision y efecto. No demuestra ausencia de checkpoints, budgets o dedup.
Retirar fuente/permiso y conservar checkpoint padre es escenario de contrato,
no prueba de vulnerabilidad Vanta; la aceptación LG-1 evita confundir
autorización del retrieval vivo con saneamiento de una copia histórica.
La diferencia visible entre claves ordinarias y especiales es una pregunta
de conformidad prioritaria, no un bug nuevo certificado del worktree.

Negativos concretos del snapshot: `get-before-put` de writes sin claim atomico
visible [V6], `index` no trasladado [V7], wrappers async sincronicos y rangos
Core incompatibles con LangChain moderno [V8]. Son mecanismos leidos, no
pruebas ejecutadas de perdida de datos, exploit o bloqueo. Evaluar A/C/D
antes de ampliar promesas de API. El defecto first-event pertenece al runner
upstream; no dice nada sobre el rendimiento medido de Vanta.

## 10. Interfaz e inspeccion

Runtime y savers Python son fuente local; el backend completo del SDK HTTP y
frontend Studio no se inspeccionaron. [Documentacion oficial Studio](https://docs.langchain.com/langsmith/studio),
leida el 2026-11-07: UI web con modo Graph (nodos/estados/trazas) y Chat
(`MessagesState`), threads, time travel y long-term memory sobre Agent Server
API. Backend local o desplegado no implica frontend OSS en este clon. No-code
builder se documenta como Cloud, no self-hosted. Escritorio nativo no acreditado
en estas fuentes; no afirmar ausencia global de clientes historicos.

Flujo verificable por contratos: UI selecciona thread → Agent Server →
saver devuelve tuple/parent/pending writes [L1] → runtime decide resume [L5].
La pagina acredita el producto, **no** el codigo de su frontend ni su auth.
Vanta Studio existente en `develop-desktop` es el seam de F/B/E: inspector
separado de hechos, checkpoint, decision y recibo. Leer una traza no debe
despachar tools. No se inicio frontend/servidor ni se creo otro dashboard.
La UI propuesta debe indicar denegación/redacción al inspeccionar/reanudar
un checkpoint previo a revocación, sin previews que el backend no autorice.

## 11. Ledger complementario y cierre de autoria

Fuentes adicionales leidas; todos los enlaces de codigo fijan SHA:

- [L11: InMemoryStore, batch 206–234 y operaciones 375–440](https://github.com/langchain-ai/langgraph/blob/e5e37b5d70dbfd93c7006bd90a658c6179a95946/libs/checkpoint/langgraph/store/memory/__init__.py#L206-L440).
- [L12: SQLite cursor 188–208, writes/delete 458–515](https://github.com/langchain-ai/langgraph/blob/e5e37b5d70dbfd93c7006bd90a658c6179a95946/libs/checkpoint-sqlite/langgraph/checkpoint/sqlite/__init__.py#L188-L208).
- [L13: JsonPlusSerializer, seguridad/defaults 104–147](https://github.com/langchain-ai/langgraph/blob/e5e37b5d70dbfd93c7006bd90a658c6179a95946/libs/checkpoint/langgraph/checkpoint/serde/jsonplus.py#L104-L147).
- [L14: runner completo, 1–520](https://github.com/langchain-ai/langgraph/blob/e5e37b5d70dbfd93c7006bd90a658c6179a95946/libs/langgraph/bench/__main__.py#L467-L520) y [ReAct fake, 1–63](https://github.com/langchain-ai/langgraph/blob/e5e37b5d70dbfd93c7006bd90a658c6179a95946/libs/langgraph/bench/react_agent.py#L1-L63).
- [L15: _scratchpad, 1280–1332](https://github.com/langchain-ai/langgraph/blob/e5e37b5d70dbfd93c7006bd90a658c6179a95946/libs/langgraph/langgraph/pregel/_algo.py#L1280-L1332), contraste limitado a scans; no auditoria de todo `_algo`.
- [V6: saver Vanta, 41–350](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/integrations/langchain/vantadb_langchain/checkpointer.py#L41-L350).
- [V7: store Vanta, 39–296](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/integrations/langchain/vantadb_langchain/store.py#L39-L296).
- [V8: dependencias del adapter](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/integrations/langchain/pyproject.toml#L25-L32).
- [V9: CaptureApprovalQueue](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/record/approval.rs#L1-L168).

[L11]: https://github.com/langchain-ai/langgraph/blob/e5e37b5d70dbfd93c7006bd90a658c6179a95946/libs/checkpoint/langgraph/store/memory/__init__.py#L206-L440
[L12]: https://github.com/langchain-ai/langgraph/blob/e5e37b5d70dbfd93c7006bd90a658c6179a95946/libs/checkpoint-sqlite/langgraph/checkpoint/sqlite/__init__.py#L458-L515
[L13]: https://github.com/langchain-ai/langgraph/blob/e5e37b5d70dbfd93c7006bd90a658c6179a95946/libs/checkpoint/langgraph/checkpoint/serde/jsonplus.py#L104-L147
[L14]: https://github.com/langchain-ai/langgraph/blob/e5e37b5d70dbfd93c7006bd90a658c6179a95946/libs/langgraph/bench/__main__.py#L467-L520
[L15]: https://github.com/langchain-ai/langgraph/blob/e5e37b5d70dbfd93c7006bd90a658c6179a95946/libs/langgraph/langgraph/pregel/_algo.py#L1280-L1332
[V6]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/integrations/langchain/vantadb_langchain/checkpointer.py#L41-L350
[V7]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/integrations/langchain/vantadb_langchain/store.py#L39-L296
[V8]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/integrations/langchain/pyproject.toml#L25-L32
[V9]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/record/approval.rs#L1-L168

**Licencias:** MIT comprobada en `libs/checkpoint/LICENSE` y `libs/cli/LICENSE`;
el inventario del proyecto no sustituye revision de cada saver/dependencia
(psycopg, SQLite wrapper, ormsgpack, Core). LangSmith/Agent Server externo
tienen terminos propios no acreditados por esas licencias. Reescribir en Rust
no elimina atribucion/notices si existe material derivado.

**Cobertura de autoria:** arquitectura y recomendaciones §§1–2; extraccion/
Rust A–F; incidencias §3; prerrequisitos §4; rechazos §5; advertencias §§3/6;
benchmarks §7; matriz §8; negativos Vanta §9; UI §10. Estatica sustentada en
los rangos declarados; atomicidad/conformidad real y UI/frontend siguen
parciales, no aceptados. No se declara revision independiente.

**Lagunas restantes:** version instalada/resolucion de dependencias, conformance
suite, crash/cancelacion reales, cifrado, semantica transaccional de todos los
backends, auth HTTP completa y codigo SaaS. No se ejecutaron pruebas/servicios.
**Division futura (>300 lineas):** separar specs de saver/replay (A/B/E/F) y
store/serde (C/D) con evidencia comun. No crear ahora archivos adicionales,
indices ni backlog; revision y aceptacion final corresponden al responsable.

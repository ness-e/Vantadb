---
title: "CrewAI: memoria unificada, consolidacion y scopes"
kind: research
status: proposed
description: "Analisis estatico del pipeline de memoria unificada y sus seams de almacenamiento, privacidad y durabilidad para VantaDB"
tags: [research, memory, engineering]
---

# CrewAI: memoria unificada, consolidacion y scopes

## Indice

- [1. Dictamen y arquitectura](#1-dictamen-y-arquitectura)
- [2. Flujo y fichas candidatas](#2-flujo-y-fichas-candidatas)
- [3. Riesgos e issues](#3-riesgos-e-issues)
- [4. Prerrequisitos](#4-prerrequisitos)
- [5. No trasladar](#5-no-trasladar)
- [6. Evidencia y cobertura](#6-evidencia-y-cobertura)
- [7. Benchmarks existentes](#7-benchmarks-existentes)
- [8. Matriz frente a VantaDB](#8-matriz-frente-a-vantadb)
- [9. Debilidades sugeridas](#9-debilidades-sugeridas)
- [10. Interfaz](#10-interfaz)

## 1. Dictamen y arquitectura

**Propuesta:** extraer el seam de almacenamiento y contratos de consolidacion
verificable. No trasladar el framework de roles, crews o Flows al Rust embebido.

- Repo `crewAIInc/crewAI`, carpeta `C:\Users\Eros\crewAI`.
- HEAD local `73f48320dbe6e2dc7a9bc54fefc6a13637b9485c`.
- **60 593 estrellas**, MIT, no archivado en el
  [inventario congelado](github-candidates.json).
- La lectura se hizo tras completar checkout: HEAD resoluble y status tracked
  limpio. Las listas de archivos borrados durante clonacion se descartaron.
- Stack observado: Python/Pydantic, Flow, ThreadPoolExecutor, modelos y
  embeddings enchufables, StorageBackend con LanceDB o Qdrant Edge.

Este HEAD implementa **memoria unificada**, no se describe como si siguiera
necesariamente un diseño historico de short/long/entity stores separados.
`MemoryRecord` tiene contenido, scope, categorias, importancia, metadata,
timestamps, embedding, source y private. Persistir una memoria no guarda el
cursor de ejecucion de la crew. Los outputs de kickoff tienen storage aparte.

## 2. Flujo y fichas candidatas

### Flujo real

`remember/remember_many -> save pool -> EncodingFlow -> batch_embed ->
intra_batch_dedup -> busqueda de similares -> analisis LLM de campos y/o
consolidacion -> execute_plans -> StorageBackend`.
Campos ya dados y sin similitud relevante permiten fast insert sin analisis
LLM; aun pueden requerir embeddings. El factory de storage es global al proceso,
se consulta para specs string y la instancia explicita gana [R5].

`recall(shallow)` hace busqueda vectorial directa; `deep` usa `RecallFlow`
para subconsultas/scopes y exploracion adicional. `drain_writes` es barrera
de lectura de futures pendientes, no log durable. Fallos de saves en background
se reportan mediante evento sin hacer fallar la tarea [R1].
Recall filtra privacidad por source y puede permitir `include_private=True`;
esa opcion no es autorizacion independiente del caller.

### Ficha A: adapter StorageBackend sobre Vanta existente

- **Objetivo:** ofrecer integracion estrecha sin reproducir crews/Flows.
- **Unidad minima:** `StorageBackend.save/search/delete/update`,
  45-104 [R6], `set_memory_storage_factory/resolve_memory_storage`,
  33-55 [R5].
- **Seam Vanta:** binding Python/SDK embebido e integraciones existentes
  LangChain/LlamaIndex; reutilizar operaciones CRUD/retrieval de Vanta.
- **Contrato Rust propuesto:** `ScopedRecordQuery { tenant, namespace,
  categories, metadata, vector, limit, score_kind }`, adapter Python que
  implementa protocolo y declara capacidades; no publicar API nueva aun.
- **Pasos/dependencias:** fijar version de CrewAI y dimension/metrica;
  mapear IDs/scopes por segmentos; filtrar antes de top-k; implementar
  factory opt-in y lifecycle explicito; probar conformidad manual despues.
- **FMEA:** factory global afecta otras aplicaciones del proceso; string
  desconocido se interpreta como path; scopes parciales se cruzan; score
  distancia versus similitud. No afirmar que Vanta tenga esos fallos.
- **Aceptacion:** rutas vecinas `/team/a` y `/team/ab` no se mezclan;
  IDs/dimension/filtros consistentes; borrar scope no afecta otro tenant;
  override explicito no cambia instancias ya construidas.
- **No objetivos:** copiar LanceDB ni sustituir BM25/HNSW/RRF.

### Ficha B: plan de consolidacion con invariantes de propiedad

- **Objetivo:** conservar scope, privacidad y provenance al modificar hechos.
- **Unidad minima:** `ConsolidationAction/Plan`, 103-134 [R4];
  `EncodingFlow.execute_plans`, 388-517 [R2].
- **Seam:** dedup/supersession y provenance existentes de `vanta-memory`;
  scheduler/checkpoints para consolidacion diferida.
- **Contrato Rust propuesto:** `ConsolidationOp { target_id, expected_revision,
  authorized_scope, change, source_refs }`. El modelo sugiere, no autoriza.
  Campos de privacidad no cambian por omision.
- **Pasos/dependencias:** congelar candidatos; validar IDs devueltos por LLM
  contra conjunto autorizado; conservar metadata de propiedad; persistir
  plan/decision; aplicar CAS o lote atomico; registrar supersession.
- **FMEA:** target fuera de candidatos, conflicto first-wins entre planes,
  borrado parcial y cambio involuntario de private/source. En [R2] un update
  reconstruye `MemoryRecord` sin esos dos campos; [R3] les da defaults.
  Esa transformacion estatica es observable; no se ejecuto exploit.
- **Aceptacion:** update de contenido conserva private/source y referencias;
  IDs fuera del snapshot se rechazan; crash conserva plan recuperable;
  conflicto de revision no pisa escritura nueva; fuentes siguen auditables.
- **No objetivos:** introducir otro juicio LLM obligatorio o borrar fuentes
  originales por recomendacion libre del modelo.

### Ficha C: recibo durable separado de save en background

- **Objetivo:** que el consumidor sepa si memoria fue aceptada, persistida
  o fallo, sin confundir una Future completada con durabilidad.
- **Unidad minima:** `Memory._submit_save/_on_save_done/drain_writes/close`,
  302-375 [R1]; barrera de recall, 716-724 [R1].
- **Seam:** scheduler y `TaskCheckpointManager` actuales, no nuevo executor.
- **Contrato Rust propuesto:** `MemoryWriteReceipt { operation_id, revision,
  status: Accepted|Persisted|Failed, error_class }` con consulta por ID.
- **Pasos/dependencias:** persistir intencion antes del trabajo costoso;
  capturar fallo durable; distinguir timeout del lector de fallo del write;
  coordinar shutdown y borrado; opcion sync solo si el caller la pide.
- **FMEA:** proceso muere con pool pendiente, evento perdido durante shutdown,
  recall espera pero silencia fallo, retry duplica extraccion o consolida
  dos veces. No basta un lock Python para garantizar crash safety.
- **Aceptacion:** tras reinicio cada operacion tiene estado o es reanudable;
  fallo visible sin depender de bus de eventos; retry con mismo ID no duplica;
  borrar scope invalida trabajo viejo que intenta escribir despues.
- **No objetivos:** exactly-once de herramientas ni reemplazar scheduler.

## 3. Riesgos e issues

| Nivel | Evidencia | Consecuencia |
|---|---|---|
| Codigo | Update reconstruye registro sin `source/private` [R2] | Con defaults [R3], esa rama pierde esos valores; validar end-to-end |
| Codigo | Planes superpuestos: primera accion por ID gana [R2] | Orden del batch influye; no prueba consolidacion semantica correcta |
| Codigo | Error de busqueda puede tratar item como nuevo [R2] | Degradacion aumenta duplicados; necesita diagnostico diferenciado |
| Codigo | Filtros categorias/metadata post-limit en LanceDB [R8] | Oversampling finito no asegura llenar k con resultados elegibles |
| Codigo | `drain_writes` no relanza fallos [R1] | Read barrier no implica que todos los saves hayan triunfado |
| Hipotesis | LLM propone delete de ID no autorizado | Revisar frontera completa antes de etiquetar vulnerabilidad |

No se consultaron issues upstream. La omision de campos es observacion
estatica del HEAD, no afirmacion de incidente explotado ni fallo de Vanta.
No asumir atomicidad entre delete/update/insert: el protocolo [R6] no la exige.
La ruta completa de autorizacion y las capas de locks no se auditaron.

## 4. Prerrequisitos

Revalidar worktree Vanta y contratos actuales antes de proponer tareas.
Ya existen context_assemble, presupuesto compartido, dedup, supersession,
provenance, sesiones, scheduler y checkpoints; no son carencias a rellenar.
Acordar scope/tenant, metricas de embeddings, privacidad por defecto y
retencion de intencion/evento/hecho. Separar campos inferidos por LLM de
campos controlados por aplicacion. Aprobacion humana debe ser explicita,
versionada y previa al cambio sensible; no se acreditó un gate HITL en el
pipeline de memoria examinado.

## 5. No trasladar

No adoptar pesos de importance/recency como calibracion universal. No portar
el Flow engine ni el pool a Rust. No hacer depender almacenamiento de LLM
o del factory global. No usar `include_private` como sustituto de ACL.
Conservar metadatos confiables aunque el modelo proponga otros.
MIT y dependencias requieren revision de copyright/licencias; reescribir en
otro lenguaje no elimina derivacion.

## 6. Evidencia y cobertura

Rige [METHOD](METHOD.md). Ledger de lectura de fuentes por rangos:

- [R1: unified_memory.py, 302-375, 435-465 y 686-735](https://github.com/crewAIInc/crewAI/blob/73f48320dbe6e2dc7a9bc54fefc6a13637b9485c/lib/crewai/src/crewai/memory/unified_memory.py#L302-L375).
- [R2: encoding_flow.py, 113-205, 224-275, 310-365 y 388-517](https://github.com/crewAIInc/crewAI/blob/73f48320dbe6e2dc7a9bc54fefc6a13637b9485c/lib/crewai/src/crewai/memory/encoding_flow.py#L388-L517).
- [R3: types.py, 20-105](https://github.com/crewAIInc/crewAI/blob/73f48320dbe6e2dc7a9bc54fefc6a13637b9485c/lib/crewai/src/crewai/memory/types.py#L20-L105).
- [R4: analyze.py, 94-134](https://github.com/crewAIInc/crewAI/blob/73f48320dbe6e2dc7a9bc54fefc6a13637b9485c/lib/crewai/src/crewai/memory/analyze.py#L94-L134).
- [R5: storage/factory.py, 1-55](https://github.com/crewAIInc/crewAI/blob/73f48320dbe6e2dc7a9bc54fefc6a13637b9485c/lib/crewai/src/crewai/memory/storage/factory.py#L1-L55).
- [R6: storage/backend.py, 45-105](https://github.com/crewAIInc/crewAI/blob/73f48320dbe6e2dc7a9bc54fefc6a13637b9485c/lib/crewai/src/crewai/memory/storage/backend.py#L45-L105).
- [R7: memory_scope.py, 89-126](https://github.com/crewAIInc/crewAI/blob/73f48320dbe6e2dc7a9bc54fefc6a13637b9485c/lib/crewai/src/crewai/memory/memory_scope.py#L89-L126).
- [R8: LanceDBStorage.search, 395-431](https://github.com/crewAIInc/crewAI/blob/73f48320dbe6e2dc7a9bc54fefc6a13637b9485c/lib/crewai/src/crewai/memory/storage/lancedb_storage.py#L395-L431).
- [R9: RecallFlow, 89-140 y 354-388](https://github.com/crewAIInc/crewAI/blob/73f48320dbe6e2dc7a9bc54fefc6a13637b9485c/lib/crewai/src/crewai/memory/recall_flow.py#L89-L140).
- [R10: ExperimentRunner, 26-180](https://github.com/crewAIInc/crewAI/blob/73f48320dbe6e2dc7a9bc54fefc6a13637b9485c/lib/crewai/src/crewai/experimental/evaluation/experiment/runner.py#L26-L180).
- [R11: TaskEvaluator.evaluate, 52-113](https://github.com/crewAIInc/crewAI/blob/73f48320dbe6e2dc7a9bc54fefc6a13637b9485c/lib/crewai/src/crewai/utilities/evaluators/task_evaluator.py#L52-L113).
- [R12: renderer HTML, 1-65](https://github.com/crewAIInc/crewAI/blob/73f48320dbe6e2dc7a9bc54fefc6a13637b9485c/lib/crewai/src/crewai/flow/visualization/renderers/interactive.py#L1-L65).

**No examinado:** Flow completo, runtime crew, todos los prompts/LLMs,
Qdrant Edge, rutas enterprise, todos los locks y suite de tests.
**Pendiente manual:** invariantes private/source, migracion, fallos parciales,
concurrencia y autorizacion. No se ejecuto codigo, dependencias ni servicios.

## 7. Benchmarks existentes

`ExperimentRunner` [R10] recibe lista de casos con `inputs`, `expected_score`
y opcional identifier; ejecuta crew/agentes, recoge evaluaciones por metricas,
promedia scores y compara con lo esperado. El caso que lanza excepcion vuelve
score cero y passed falso en esa ruta. No hay dataset de memoria fijado aqui.
`TaskEvaluator` [R11] es otra superficie: usa LLM del agente para evaluar output
y obtiene sugerencias, calidad 0-10 y entidades. No confundir ambos runners
ni interpretar juicio LLM como recall.

Dependencias/efectos: proveedores/modelos, herramientas de la crew, DB/memorias,
callbacks/eventos, coste y posible red. Reutilizar crew entre casos requiere
controlar estado para no contaminar holdout. No se midio ningun score local;
la existencia del runner no valida cifras de marketing. Comparacion solo por
[METHOD](METHOD.md#protocolo-de-comparacion-de-benchmarks), sin nuevo harness,
prompt, rubric, grader, threshold ni configuracion.

## 8. Matriz frente a VantaDB

| Capacidad | Vanta | Diferencia a evaluar |
|---|---|---|
| BM25/HNSW/RRF y bindings Python | Existe | Adapter de protocolo CrewAI, si se necesita |
| Dedup/supersession/provenance | Existe | Plan condicionado y conservacion de propiedad |
| Sesiones/scheduler/checkpoints | Existe | Recibos de saves, no pool durable ficticio |
| Context assembly/shared budget | Existe | Recall deep debe consumir budget global |
| Scopes y ACL consistentes en adapter CrewAI | No acreditado | Contrato y filtro antes de top-k |
| Razon explicita de recall/evidence gaps | Parcial en contraste | Mapear diagnosticos existentes, no duplicar |
| Orquestador CrewAI en motor Rust | No aplicable | Mantener frontera de adapter |

## 9. Debilidades sugeridas

Inspeccionar estado durable de consolidacion y de writes fallidos, y conservar
propiedad privada al transformar registros, son objetivos de calidad de
contrato. No demuestran que Vanta pierda privacidad o carezca de auditoria.
Antes de tareas se revisaran rutas vigentes, no bugs historicos del worktree.

## 10. Interfaz

**Disponible en fuente:** renderer de grafos Flow en HTML/Jinja/JS/CSS [R12].
No equivale a dashboard de memoria ni escritorio nativo, **no localizado**
en este alcance. UI enterprise/alojada no se presume OSS. No se renderizo ni
inicio servicio. Vanta ya tiene Studio en `develop-desktop`; mostrar scopes,
planes y recibos alli seria extension, no un frontend nuevo.

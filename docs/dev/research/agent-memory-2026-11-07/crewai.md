---
title: "CrewAI: memoria unificada, consolidacion y scopes"
kind: research
status: proposed
description: "Analisis estatico del pipeline de memoria unificada y sus seams de almacenamiento, privacidad y durabilidad para VantaDB"
tags: [research, memory, engineering]
---

# CrewAI: memoria unificada, consolidacion y scopes

## Indice

- [0. Identidad, mapa y fronteras](#0-identidad-mapa-y-fronteras)
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
- [Fuentes adicionales y cierre](#fuentes-adicionales-y-cierre)

## 0. Identidad, mapa y fronteras

**Corte:** inspección documental del 2026-11-07; ninguna ejecución funcional.
El SHA de CrewAI se volvió a resolver sin fetch ni checkout:
`73f48320dbe6e2dc7a9bc54fefc6a13637b9485c`. El manifiesto registra clon shallow,
omisión de historial anterior/otras ramas/tags, sin submódulos inicializados ni
descarga LFS; no se modificó ese registro. No interpretar la versión del paquete
como identidad de todos sus servicios ni comparar estrellas de otra fecha.
La base Vanta de esta ampliación es
`97c9623c8ef5a70fdb51aff215bfbb3b05b15612`, rama `develop`; no la observación
histórica `c561158` con cambios dirty. Se releyeron seams concretos (§4 y §9).

### Mapa funcional y stack

| Superficie | Código inspeccionado o localizado | Responsabilidad y límite |
|---|---|---|
| Paquete público | `lib/crewai/pyproject.toml:1-160` [CA] | Python `>=3.10,<3.14`, Pydantic 2, OpenAI, instructor, OpenTelemetry, Chroma y LanceDB; extras opcionales Qdrant/otros proveedores. Dependencias declaradas, no entorno instalado |
| Orquestación | `lib/crewai/src/crewai/crew.py:1894-2006,2109-2179` [CB] | tareas, resultados, drenaje de memorias y replay de cola de tareas; no transacción sobre herramientas y hechos |
| Dominio memoria | `memory/unified_memory.py`, `types.py`, `analyze.py` | fachada, registro, configuración y planes inferidos; `Memory` no es cursor durable del agente |
| Procesamiento | `memory/encoding_flow.py`, `recall_flow.py` | embebido por lotes, dedup, análisis/consolidación; recuperación simple o exploratoria |
| Infraestructura | `memory/storage/backend.py`, `factory.py`, `lancedb_storage.py` | protocolo, selección de backend y persistencia; la forma Python no acredita ACID de varias operaciones |
| Vistas | `memory/memory_scope.py:89-126` | scope/slice encaminan llamadas; alcance estructural no equivale a ACL |
| Resultados de ejecución | `utilities/task_output_storage_handler.py:1-89` [CC] | delega a `KickoffTaskOutputsSQLiteStorage`; recurso separado de memoria semántica |
| Evaluación | `experimental/evaluation/experiment/{runner,result}.py` | casos definidos por consumidor, agregación y comparación de artefactos; no dataset de memoria incorporado en esas clases |
| UI inspeccionada | `flow/visualization/renderers/interactive.py:1-65` | exportación HTML; no fuente de una aplicación empresarial completa |

El deployment mínimo de la biblioteca es un proceso Python con almacenamiento
seleccionado y proveedores configurados. El pipeline puede llamar embeddings/LLM
y emitir eventos con datos; no necesita que Vanta adopte una dependencia remota
obligatoria. CrewAI no se reduce a memoria: crews, agentes, tareas y Flows son
consumidores/orquestación. Su código de replay no convierte la base semántica
en journal de efectos externos.

### Modelo de datos y contrato público

`MemoryRecord` [R3] combina UUID por defecto, texto, ruta jerárquica `scope`,
categorías, metadata arbitraria, importancia acotada, fechas, embedding,
`source: Optional[str]` y `private: bool`. El embedding se excluye de la
serialización Pydantic de ese modelo; esto no significa que el backend no lo
persista. `MemoryMatch` añade score compuesto, razones y `evidence_gaps`.
`ScopeInfo` describe jerarquía/cantidades, no autoridad del propietario.

El score mezcla similitud, decaimiento temporal e importancia
(`types.py::compute_composite_score` [CD]); su valor no es una probabilidad de
verdad ni un score RRF transportable. El decaimiento leído usa edad desde
`created_at`; el `touch` de recall afecta acceso, no redefine ese nacimiento.
No se propone copiar sus pesos. Los parámetros de consolidación y el factor
de oversampling pertenecen al upstream congelado, no a una nueva configuración
de Vanta ni a un criterio de aceptación cuantitativo.

`StorageBackend` [CE] abarca save/search/delete/update/get/list/count/reset,
jerarquía/categorías y métodos asíncronos. La fachada añade remember, extract,
recall, forget, update y vistas. No son intercambiables estas tres operaciones:

1. **Guardar fuente:** persistir un turno o artefacto original.
2. **Guardar memoria:** crear/revisar un hecho derivado con embedding y provenance.
3. **Guardar ejecución:** conservar outputs y el punto desde el que se reejecuta.

Un scope limita búsquedas; un `source` filtra privados en recall; ninguno
autentica al llamador. El flag `include_private` pertenece a código confiable.
Los contenidos de herramientas/modelo no deben poder elegirlo ni autorizar
deletes. La lectura se limita a estas rutas: no certifica seguridad global.

## 1. Dictamen y arquitectura

**Propuesta:** extraer el seam de almacenamiento y contratos de consolidacion
verificable. No trasladar el framework de roles, crews o Flows al Rust embebido.

**Corrección discriminante del cierre:** el snapshot Vanta fijado **ya incluye**
`integrations/crewai/vantadb_crewai/memory.py::VantaDBMemoryBackend` [CV6].
Por tanto A significa revisar/extender ese adapter, no crear una integración
nueva. También existe `vectorstore.py::VantaDBTool` [CV7], herramienta de
búsqueda distinta del backend de memoria unificada. El paquete declara
`crewai>=1.14,<2` y licencia Apache-2.0 [CV8]; no se comprobó compatibilidad
dinámica con el SHA CrewAI inspeccionado ni publicación del paquete.

Prioridad razonada: **B/C primero como revisión de contratos existentes**:
hay un fallo concreto de conservación de propiedad y una barrera que silencia
fallos. **A/E después**, sólo si hay consumidores CrewAI: una integración tiene
coste de compatibilidad Python/protocolo y no aporta un índice nuevo.
**D condicional**: diagnosticar búsquedas parciales es útil; adoptar planificación
LLM multirama sin necesidad medida añade coste y superficie de autorización.
No son decisiones de implementación; no se fijan pesos, prompts ni umbrales.

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

### Tres trazas de extremo a extremo y sus fallos

**Entrada de agente, herramientas y handoff real.** Además de la API de memoria,
`Agent._retrieve_memory_context:778-844` [CK] obtiene memoria de agente o crew,
consulta con descripción de tarea y `limit=5`, formatea y añade al prompt.
Fallo ordinario degrada a tarea sin memoria; `HookAborted` se propaga. No es
correcto inferir que ausencia de contexto significa cero hechos almacenados.
`CrewAgentExecutor.execute_tool_action:1920-2008` [CL] ejecuta la herramienta,
incorpora observación y continúa; los abortos deliberados no se convierten en
errores recuperables para el LLM. Su `finalize:2663-2735` tiene un guard local
contra doble finalización, no una garantía de exactamente una ejecución externa.
`BaseAgentExecutor._save_to_memory:31-69` [CM] extrae de
task/agent/expected/result y llama `remember_many`; evita ciertos outputs de
delegación y read-only. La llamada puede encolar trabajo; no es commit de
las herramientas ni backup de todos sus efectos.

La delegación a coworker y el replay de tareas son handoffs de orquestación:
no transfieren automáticamente una snapshot consistente de la memoria global.
El timeout en `Agent._execute_with_timeout:1050-1088` [CN] llama `future.cancel`;
el fragmento no acredita revocar un efecto ya ejecutándose. No se propone
portar esa cancelación basada en hilos ni presentar «timeout» como rollback.

**Traza 1 — escritura y consolidación de un hecho.**

1. El host llama `remember(content, scope, categories, importance, source,
   private)`; `unified_memory.py:435-527` [CF] entra por el mismo pool serializado
   que los lotes y espera el resultado. No describir este método síncrono como
   fire-and-forget; `remember_many` y el uso en background tienen otro contrato.
2. `_encode_batch:377-433` [CF] construye un `EncodingFlow`, le pasa storage/LLM/
   embedder/config e ítems con campos de entrada. `batch_embed:113-120` calcula
   los vectores; `intra_batch_dedup:122-153` descarta similitudes casi idénticas.
   Esto no establece identidad estable de una petición repetida tras reinicio.
3. `parallel_find_similar:155-222` consulta candidatos en scope. Si una búsqueda
   falla, se degrada a no encontrar similares en la ruta leída; puede aumentar
   inserciones. No es evidencia de que un hecho nuevo sea semánticamente nuevo.
4. `parallel_analyze:224-386` distingue campos proporcionados/por inferir y
   similitud; el LLM puede proponer `ConsolidationPlan`, no tiene autoridad
   intrínseca sobre registros. Reembebido del contenido revisado ocurre antes
   de las mutaciones de `execute_plans:388-517`.
5. Se deduplican acciones por ID, primera acción gana. Deletes, updates y saves
   se invocan por separado. El comentario de write lock no demuestra una
   transacción de storage: en el cuerpo leído no hay una frontera backend única
   para toda la secuencia. Puede haber serialización del caller/pool; no se
   extrapola exclusión entre procesos.
6. El update de consolidación reconstruye `MemoryRecord` sin private/source;
   la inserción sí los lleva. §3 vincula este hecho estático con un issue real.
   `result_record` puede ser el actualizado o un similar preexistente cuando
   `insert_new=False`: devolver un registro no significa que se insertó uno nuevo.

**Errores/replay/rollback:** antes de escribir, fallo de embedding/análisis no
equivale a hecho persistido. Después de un delete, una excepción de update/save
no recibe aquí compensación durable. Un retry sin ID de operación puede crear
otro UUID. Ni la Future ni el evento de éxito general de la crew acreditan que
todos los hechos se hayan confirmado. La candidata B exige plan autorizado y
frontera de aplicación; C hace observable la fase durable, sin prometer rollback
de herramientas que la crew ya ejecutó.

**Traza 2 — query, recuperación y construcción de contexto.**

1. `recall:686-826` [CG] espera las escrituras pendientes mediante
   `drain_writes`; este espera y consulta excepciones, pero no las relanza.
   No hay garantía de que todas hayan tenido éxito ni watermark de confirmación.
2. La ruta shallow genera embedding, invoca `search(scope,categories,limit)`
   y filtra `private` por source. Un embedding vacío produce lista vacía;
   no permite distinguir por sí solo «sin evidencia» de «consulta no embebida».
3. LanceDB [R8] acota candidatos y aplica parte de los filtros después. Filtrar
   privacidad posteriormente también reduce cardinalidad. Oversampling finito
   no garantiza devolver k autorizados ni debe convertirse en autorización.
4. La ruta deep [CH] analiza query, categorías/scopes y subconsultas, busca el
   producto embeddings × scopes y explora según confianza. Cada búsqueda fallida
   se registra y se omite: un resultado parcial no prueba búsqueda exhaustiva.
5. Sintetiza/deduplica por registro y ordena matches; la fachada toca
   `last_accessed` cuando puede y no es read_only. Un fallo de touch no invalida
   el recall. Se emiten eventos con query/resultados y tiempo.
6. `MemoryMatch.format` renderiza contenido y metadata para consumidores.
   La selección de memoria no equivale al presupuesto final de todos los
   mensajes/tools del modelo; el adapter Vanta debe pasar por su assembler.

**Fronteras:** el tiempo `query_time_ms` empieza después del drain: incluye
embedding y el camino shallow/deep más touch, pero excluye esperar saves.
No es latencia retrieval pura ni end-to-end de agente. El store entrega datos,
no instrucciones de mayor confianza. Una segunda consulta puede observar
last_accessed cambiado; no es necesariamente una lectura sin efectos.

**Traza 3 — actualización/borrado, persistencia de sesión y reejecución.**

1. `update:862-909` [CI] lee por ID, devuelve existente si read_only, reembebe
   si cambia contenido y usa `model_copy(update=...)`. Preserva campos omitidos,
   a diferencia de la reconstrucción de consolidación. No se observa revisión
   esperada/CAS en la firma pública: no afirmar control optimista automático.
2. `forget:828-860` traduce root_scope y filtros a `delete`; `reset:1028-1049`
   es otra superficie. Una vista de lectura no es una capacidad de autorización
   portable a cualquier llamada por ID. Revisar ownership en el adapter.
3. `TaskOutputStorageHandler` [CC] persiste tareas y outputs en SQLite aparte.
   `Crew.replay:2109-2179` [CB] carga esos outputs, busca la tarea, valida
   compatibilidad, reutiliza resultados anteriores y ejecuta el sufijo.
4. La memoria semántica no se restaura al estado del output antiguo mediante
   ese replay. Las herramientas del sufijo pueden volver a producir efectos;
   no hay prueba de exactly-once ni rollback de pagos/escrituras externas.
5. Al construir la salida, `Crew._drain_memory_writes` [CB] recorre memoria
   global, por agente y manager, desenvuelve vistas y deduplica instancias.
   Drena antes de completion/teardown de listeners; resuelve orden de eventos,
   no una transacción conjunta de output, memoria y herramientas.

**Escenario adverso discriminante:** iniciar lote, borrar scope y reintentar
trabajo viejo; observar qué generación puede escribir, además del conteo final.
No se reprodujo una resurrección. Un reset lock/pool local no basta para
certificar esa propiedad frente a varios procesos. Un rollback futuro deberá
conservar fuentes y versión de intención, no volver a correr tareas esperando
que la memoria vuelva sola al pasado.

### Ficha A: adapter StorageBackend sobre Vanta existente

- **Objetivo:** completar conformidad del adapter ya presente sin reproducir crews/Flows.
- **Unidad minima:** `StorageBackend.save/search/delete/update`,
  45-104 [R6], `set_memory_storage_factory/resolve_memory_storage`,
  33-55 [R5].
- **Seam Vanta:** `VantaDBMemoryBackend.save/search/update/delete` [CV6],
  sobre binding Python/SDK embebido; reutilizarlo, no otro backend paralelo.
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

**Delta y contrato de aplicación A.** Vanta ya ofrece CRUD, filtros, vectores,
bindings y el adapter: el valor es revisar conformidad con este protocolo, no otro motor. Se prefiere
adapter opt-in a traducir `StorageBackend` entero a un trait Rust público.
Entrada confiable: tenant, raíz permitida, identidad del embedding y capability
de escritura; entrada no confiable: texto, filtros y contenido sugerido por el
modelo. Salida: registros con ID/revisión y score etiquetado; errores propuestos
`Denied`, `DimensionMismatch`, `UnsupportedFilter`, `Conflict`, `StoreFailure`.
No reinterpretar un score RRF como similitud coseno para pasar un `min_score`.

**Orden/ownership A:** resolver instancia explícita → validar capability →
normalizar scope por segmentos → llamar SDK → convertir resultados. Commit es el
de la operación SDK, no el retorno de la factory. `close` libera el handle que
el adapter posee, nunca cierra una DB prestada por otro consumidor. No migrar
datos ni reindexar salvo cambio explícito de modelo/dimensión; entonces construir
proyección separada y cambiar referencia sólo cuando esté disponible. Desactivar
factory devuelve la selección al host sin borrar datos. Dependencias: sólo
binding/protocolo y contrato de scores; B/C no son prerrequisitos del read-only.
Detección FMEA: comparar tenant/scope/score-kind del receipt con la petición,
registrando IDs y causas, no contenido privado.

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

**Delta/alternativas B:** `l1_writer.rs::plan_write`, `persist_planned` y
`apply_dedup_batch` [CV1] ya separan decisiones y escritura; la propuesta añade
precondiciones de una transformación multirregistro y conservación de ownership.
Primera opción: extender el plan existente; adapter sólo traduce; no aceptar
acciones arbitrarias del LLM ni implementar otra deduplicación. Entrada propuesta:
snapshot de candidatos autorizados y operaciones con revisión esperada. Salida:
receipt por plan y por objetivo; `StaleRevision`, `UnknownTarget`, `Denied`,
`EmbeddingFailed`, `CommitFailed`, sin anunciar éxito parcial como total.

**Persistencia/concurrencia B:** calcular texto/vector fuera del commit →
revalidar objetivo/tenant/generación → persistir transformación y supersession
en la frontera real disponible → emitir eventos después. `apply_dedup_batch`
contiene fases y efectos post-commit: su nombre no prueba una transacción
general para cualquier plan. Si no existe frontera adecuada, limitar inicialmente
a una operación y declarar el resto no soportado. El worker es dueño del intento,
no de registros de otro tenant; leases no reemplazan revisión al publicar.
Migrar sólo versión del plan si se persiste, conservar originales y permitir
desactivar nuevas consolidaciones sin deshacer hechos posteriores. B no necesita
el adapter A. C sólo es necesario para prometer recuperación diferida; D no lo es.
FMEA adicional: plan viejo tras borrado → resurrección → detectar generación
obsoleta y rechazar; fallo de proyección tras primario → marcar pendiente de
reindexación, no repetir la mutación como si nunca hubiera ocurrido.

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

**Delta/alternativas C:** `PipelineWorker::run_once/run_task` y
`TaskCheckpointManager::{begin,advance,complete,fail}` [CV2] ya poseen
claim/retry/estados. Primero comprobar si esos IDs/reportes cubren el receipt;
preferir exponerlos antes de crear una tabla adicional. Los estados propuestos
se refinan con `CancelledBeforeApply` y `OutcomeUnknown` cuando la frontera no
pueda acreditar aplicación: «cancelado» nunca significa automáticamente
«no escrito». El receipt identifica sesión, operación, intento y generación.

**Secuencia C:** admisión durable con ID existente → claim bajo ownership del
worker → cálculo opcional LLM → revalidación de generación → escritura →
checkpoint terminal → notificación best-effort. Si escritura y checkpoint no
comparten commit, conservar identidad aplicada para reconciliar al reintentar.
Timeout del cliente no revoca una escritura confirmada; cancelación previa
impide iniciar efectos nuevos si el worker la observa. El host posee shutdown
y finalización, no una Future abandonada. Rollout compatible como proyección de
estados existentes; desactivar el adapter no borra operaciones admitidas.
No requiere A ni D; depende del contrato de escritura de B sólo para planes B.
Aceptación manual adicional: cancelación antes/después de commit, caída entre
commit y notificación, lector que exige watermark y borrado con trabajo viejo;
receipt final coherente con registros supervivientes, sin logs de payload.

### Ficha D: plan de recall autorizado y resultado parcial explicable

**Problema:** una pregunta larga puede requerir varias búsquedas, pero el agente
no debe confundir «subconsulta falló», «no hay evidencia» y «scope denegado».
Unidad mínima observada: `RecallFlow._do_search/analyze_query_step/
synthesize_results`, `recall_flow.py:89-222,354-388` [CH]. Hay producto de
subqueries × scopes, paralelismo acotado a cuatro workers en esa ruta, omisión
de búsquedas fallidas, dedup por ID y gaps adjuntos al primer match.
Un resultado vacío no puede transportar ese gap del primer match.

**Delta/seam:** Vanta ya tiene `RecallConfig`, `InjectionPolicy`,
`perform_auto_recall_governed` y `RecallGovernance` [CV3]. Extender su reporte
con procedencia por rama del recall, no reemplazar BM25/HNSW/RRF ni recalibrar
su scoring compuesto opcional. Contrato Rust propuesto:
`RecallBranch { query_ref, allowed_scope, parent_branch }` →
`RecallBranchOutcome { evidence_refs, status, cost_ref }`, donde status distingue
completada/vacía/denegada/fallida/cancelada. La lista de fallos existe aunque
no se recupere ningún hecho. `Denied` y `BudgetExhausted` no son scores bajos.

**Pasos/commit/ownership:** 1) host fija scopes y presupuesto existente;
2) planner opcional sugiere ramas; 3) intersección con capability y validación
antes de recuperar; 4) retrieval existente; 5) dedup por ID/revisión;
6) assembler [CV4] decide contexto final y reporte. El planner no amplía ACL,
no modifica raíz por encontrar un nombre de scope y no autoriza `include_private`.
No hay commit de memoria por una consulta pura; si hay touch, separarlo de
éxito de lectura. Cancelar deja originales intactos y observa tareas pendientes;
no entregar resultados tardíos a una generación de consulta nueva.

**Alternativas/deps:** primero instrumentar recall simple; adapter que conserve
`RecallGovernance` es preferible a portar `RecallFlow`. Multiquery queda opcional
y dependiente de evidencia de necesidad, sin nuevo prompt ni parámetros aquí.
Sólo depende de A para integración CrewAI; puede ampliar Vanta sin A/B/C.
No migración de índices; reporte aditivo versionado y desactivable.
FMEA: scope sugerido hostil → fuga → comparar con capability antes del search;
rama caída → falsa exhaustividad → estado parcial explícito; fan-out → consumo
excesivo → presupuesto/cancelación del host; orden de futuros → dedup inestable
→ identidad/revisión y orden de salida definido por el consumidor.
Aceptación manual: ramas vacía/fallida/denegada distinguibles sin matches,
ACL consistente en cada rama y presupuesto global respetado con la excepción
del prefijo protegido documentada por Vanta. No objetivo: prometer mejor QA,
copiar confidence/oversampling/pesos ni crear otro orquestador.

### Ficha E: capability de herramientas de memoria y confirmación honesta

**Problema:** el LLM puede pedir guardar y afirmar éxito cuando sólo se encoló
un lote; un modo consulta no debe ofrecer mutaciones. `memory_tools.py`
`RecallMemoryTool._run`, `RememberTool._run`, `create_memory_tools:13-130` [CJ]
deduplica matches por ID entre queries, guarda uno síncronamente, anuncia
background para varios y omite Remember si `read_only`. No es gate HITL:
es una superficie de herramientas del agente.

**Delta/seam:** reutilizar herramientas MCP existentes y binding hacia
`vanta-memory`; no añadir otro tool runtime. Contrato Rust propuesto para la
proyección del resultado: `MemoryToolResult { operation_ref, applied_refs,
state, governance_ref }`. El host deriva capability read/write del usuario,
no de argumentos del modelo; un texto «Saved» exige estado aplicado acreditado.
Input: contenido no confiable y operación/scope sellados por host.
Errores: `Denied`, `ReadOnly`, `InvalidInput`, `Pending`, `StoreFailure`.
`Pending` es resultado operativo, no rechazo ni confirmación durable.

**Pasos/ownership:** 1) inventariar tools ya expuestas; 2) ocultar mutadores para
capability de lectura y volver a autorizar al ejecutar; 3) usar captura existente
con ID de operación; 4) devolver receipt C si asíncrono; 5) mostrar estado/ref
en Studio sin payload por defecto. Revocación entre listado e invocación debe
denegar en la segunda frontera. Writes usan commit existente; no hay commit
adicional por renderizar. Cancelar respuesta no revierte write confirmado.
Dependencias: binding/MCP y A sólo para fachada CrewAI; C para saves diferidos.
Alternativas: documentar Pending en tools actuales antes que añadir API pública;
rechazar copiar texto fijo como único estado. Sin migración/reindexación.
Rollback: retirar exposición de herramienta, conservar operaciones ya admitidas.

**FMEA/aceptación futura:** UI dice guardado antes de commit → uso de hecho
inexistente → estado derivado de receipt; capability revocada → write tardío
→ comprobar al aplicar; metadata de herramienta revela secreto → usar refs
autorizadas; batch repetido → duplicado → ID existente de captura.
Caso normal guarda una vez; lote falla parcialmente y explica cada operación;
read-only no ofrece mutadores y los rechaza si se invocan por nombre.
No objetivos: aprobación genérica de pagos, nuevo scheduler, extracción LLM
obligatoria ni cambio de prompts del agente.

## 3. Riesgos e issues

| Nivel | Evidencia | Consecuencia |
|---|---|---|
| Codigo | Update reconstruye registro sin `source/private` [R2] | Con defaults [R3], esa rama pierde esos valores; validar end-to-end |
| Codigo | Planes superpuestos: primera accion por ID gana [R2] | Orden del batch influye; no prueba consolidacion semantica correcta |
| Codigo | Error de busqueda puede tratar item como nuevo [R2] | Degradacion aumenta duplicados; necesita diagnostico diferenciado |
| Codigo | Filtros categorias/metadata post-limit en LanceDB [R8] | Oversampling finito no asegura llenar k con resultados elegibles |
| Codigo | `drain_writes` no relanza fallos [R1] | Read barrier no implica que todos los saves hayan triunfado |
| Hipotesis | LLM propone delete de ID no autorizado | Revisar frontera completa antes de etiquetar vulnerabilidad |

Se leyó el issue público [#8659](https://github.com/crewAIInc/crewAI/issues/8659),
«Consolidation silently drops private and source, turning a private memory
public», creado el 2026-10-27, **open observado el 2026-11-07** por API GitHub.
El cuerpo aporta reproducer helper con storage de captura y revisión
`bb4d892e6`; declara asistencia IA. No se ejecutó. La causa estática coincide con
`execute_plans:477-490` del SHA aquí fijado y los defaults de [R3].
No acredita incidente real, exploit remoto ni que toda consolidación seleccione
update. PR/fix integrado: no acreditado por la fuente leída.

**Frontera:** el host marca memoria privada de un agente → consolidación
reconstruye registro sin esa marca → recall filtra usando la marca persistida.
Es pérdida de propiedad en esa rama, no autenticación demostrada entre tenants.
Discriminación futura: crear privado con source, forzar por entrada legítima
una consolidación update, consultar como otro source y comprobar antes/después;
separar validación de transformación de prueba completa de exposición.
Vanta: B conserva tenancy/provenance al transformar; no atribuirle este bug
sin recorrer sus escritores. La omision de campos es observacion
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
- [R10: ExperimentRunner, 26-165](https://github.com/crewAIInc/crewAI/blob/73f48320dbe6e2dc7a9bc54fefc6a13637b9485c/lib/crewai/src/crewai/experimental/evaluation/experiment/runner.py#L26-L165).
- [R11: TaskEvaluator.evaluate, 52-113](https://github.com/crewAIInc/crewAI/blob/73f48320dbe6e2dc7a9bc54fefc6a13637b9485c/lib/crewai/src/crewai/utilities/evaluators/task_evaluator.py#L52-L113).
- [R12: renderer HTML, 1-65](https://github.com/crewAIInc/crewAI/blob/73f48320dbe6e2dc7a9bc54fefc6a13637b9485c/lib/crewai/src/crewai/flow/visualization/renderers/interactive.py#L1-L65).

**No examinado:** Flow completo, runtime crew, todos los prompts/LLMs,
Qdrant Edge, rutas enterprise, todos los locks y suite de tests.
«Runtime crew no examinado» significa no recorrido completo: sí se leyeron
las rutas acotadas de ejecución/replay/flush y agente indicadas en §0/§2.
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

### Lectura discriminante de evaluación, no resultados

`ExperimentRunner.run → create_default_evaluator → kickoff →
get_agent_evaluation → _extract_scores → _assert_scores` [R10,CO].
`create_default_evaluator:344-365` construye seis evaluadores:
goal alignment, semantic quality, tool selection, parameter extraction,
tool invocation y reasoning efficiency; recibe `llm=None`. La identidad efectiva
del modelo/juez depende de la configuración de esas clases y del host: no queda
congelada por `ExperimentRunner`. `get_agent_evaluation:208-241` usa por defecto
`SIMPLE_AVERAGE`; `_extract_scores` promedia por métrica únicamente scores no
nulos de los resultados agregados de agentes. No es promedio de todas las
preguntas de un dataset público de memoria.

| Aspecto | Observación del runner fijado | Límite / aplicación |
|---|---|---|
| Unidad/dataset | Diccionario por caso, `inputs`, `expected_score`, identifier opcional | Sin LoCoMo/LongMemEval, split ni revisión de dataset en estas clases |
| Identidad | Fallback MD5 de `str(test_case)` | Identificador de conveniencia, no identidad semántica durable ni firma |
| Preparación | Casos secuenciales; reset de resultados del evaluator entre casos | No resetea aquí toda memoria/DB/tools de la crew |
| Ground truth | `expected_score` compara resultado; no se pasa como inputs por esta función | No prueba ausencia de respuestas en los inputs elegidos por consumidor |
| Población | `ExperimentResults.results` incluye retornos de `_run_test_case` | `inputs`/`expected_score` se leen antes del try: esquema roto puede abortar, no convertirse en cero |
| Denominador | `result_display.py::summary:14-27` [CP]: passed / len(results); vacío → N/A | Fallo capturado cuenta como failed; score ausente no equivale a retrieval miss |
| Comparación | Número/número usa >=; dict/dict compara sólo claves de la intersección | Una clave esperada ausente no causa fallo si otra coincidente satisface; no es igualdad completa de contratos |
| Defaults | `print_summary=False`; `compare_with_baseline(save_current=True)` [CQ] | Comparar puede escribir/apendear baseline; no acción inocua de lectura |
| Métricas/artefactos | `to_json` excluye agent_evaluations y guarda metadata/results | Artefacto resumido no incluye necesariamente toda explicación del juez |
| Latencia/coste | No instrumenta percentiles, warmup ni coste por fase en esas clases | Proveedores, extracción, tools y juez pueden cobrar; ninguna cifra medida aquí |
| Reproducibilidad | Semilla, tokenizer, cache, presupuesto y versión de modelo no fijados por runner | Deben proceder de configuración existente, no valores inventados en el informe |

La discrepancia entre docstring y comparación dict/dict es observación de código,
no invalidación de cualquier cifra CrewAI. `TaskEvaluator` [R11] es un juicio
LLM distinto; no se suman escalas ni se lo llama benchmark de recuperación.
Su resultado calidad 0-10 no es probabilidad ni recall por ID de sesión.

**Adaptación concreta usando METHOD/BENCHMARK-NOTES existentes:** A traduce
storage en el escenario CrewAI; E conserva estado Pending y C identifica writes;
D reporta recuperación parcial. Mantener iguales agente/tools/modelo y memoria
train-only, separar importación/embedding/extracción/retrieval/generación/juez.
No usar `expected_score` como anotación de relevancia, ni contaminar casos con
memoria de otros inputs, ni ajustar sobre holdout. Si se evalúa sólo retrieval,
usar el runner de esa pista existente de Vanta, no renombrar este Success Rate.
Esta tarea no crea harness ni modifica `_assert_scores`, prompts o sampling.

## 8. Matriz frente a VantaDB

| Capacidad | Vanta | Diferencia a evaluar |
|---|---|---|
| BM25/HNSW/RRF, bindings Python y adapter CrewAI | Existe [CV6-CV8] | Revisar conformidad del adapter, no crearlo otra vez |
| Dedup/supersession/provenance | Existe | Plan condicionado y conservacion de propiedad |
| Sesiones/scheduler/checkpoints | Existe | Recibos de saves, no pool durable ficticio |
| Context assembly/shared budget | Existe | Recall deep debe consumir budget global |
| Scopes y ACL consistentes en adapter CrewAI | Parcial [CV6] | Prefijos por segmentos existen; filtros posteriores a top-k no prueban ACL end-to-end |
| Razon explicita de recall/evidence gaps | Parcial en contraste | Mapear diagnosticos existentes, no duplicar |
| Orquestador CrewAI en motor Rust | No aplicable | Mantener frontera de adapter |

Evidencia/decisión por fila: motor/binding y sesiones en
[VANTA-REFERENCE](VANTA-REFERENCE.md#capacidades-y-limites-discriminantes);
dedup en [CV1], worker/checkpoint en [CV2], ACL/reporte en [CV3],
presupuesto y marcadores en [CV4]. Son capacidades del snapshot `97c9623…`,
no certificación del HEAD concurrente. «Parcial» en razones/gaps significa que
Vanta tiene reporte gobernado, pero no se acreditó equivalencia con todas las
ramas deep; no ausencia de observabilidad.

| Capacidad adicional | Vanta | Decisión sustentada |
|---|---|---|
| Ranking recency/relevance/importance | Existe como opción de recall; `perform_auto_recall_scored` en auto_recall | No portar pesos CrewAI; D conserva etiquetas de score |
| Protección de contenido tainted | Existe en `InjectionPolicy` [CV3] | Extender política, no introducir ACL paralela con `private` |
| Fuente→derivado y publicación multirregistro condicionada | Parcial: provenance/dedup existen [CV1]; CAS general no acreditado | B exige inventario de frontera antes de API fuerte |
| Receipt durable de cualquier save CrewAI | No acreditado | C sólo exposición/adaptación, no scheduler nuevo |
| Cancelación de efecto externo del agente | No aplicable al motor | Pertenece al host/herramienta, no al almacenamiento |

## 9. Debilidades sugeridas

Inspeccionar estado durable de consolidacion y de writes fallidos, y conservar
propiedad privada al transformar registros, son objetivos de calidad de
contrato. No demuestran que Vanta pierda privacidad o carezca de auditoria.
Antes de tareas se revisaran rutas vigentes, no bugs historicos del worktree.

**Negativo local sustentado, acotado:** `utils/checkpoint.rs::mutate:112-117`
en snapshot `97c9623…` [CV5] lee, aplica closure y escribe en pasos separados.
No acredita CAS dentro de ese helper; no demuestra lost update real sin callers
concurrentes. Para C/B, verificar serialización del worker, escritor externo y
lease al publicar. No asociar automáticamente este helper con WAL/recovery o
con todas las escrituras L1. No se acredita pérdida de private/source en Vanta.
Falso positivo descartado: proponer dedup, cuotas, ACL por segmentos, scoring
compuesto, assembler o scheduler «desde cero» contradice la referencia fijada.

## 10. Interfaz

**Disponible en fuente:** renderer de grafos Flow en HTML/Jinja/JS/CSS [R12].
No equivale a dashboard de memoria ni escritorio nativo, **no localizado**
en este alcance. UI enterprise/alojada no se presume OSS. No se renderizo ni
inicio servicio. Vanta ya tiene Studio en `develop-desktop`; mostrar scopes,
planes y recibos alli seria extension, no un frontend nuevo.

`InteractiveRenderer` [R12] carga template con Jinja y construye HTML con
grafo; no se inspeccionó un cliente CRUD completo de registros ni editor de ACL.
El flujo inspeccionado es exportación de topología, no confirmación de commits.
Eventos de recall incluyen texto/query/resultados [CG,CK]: un inspector no debe
suscribirse y persistir todos los payloads sin política de acceso/redacción.
Oportunidad Studio: vista fuente→plan B→receipt C→evidencia entregada D,
con estado parcial, revisión y scope; enlaces a fuente sólo para usuario
autorizado, y acción de borrar con alcance explícito. No nueva aplicación ni
integración de un servicio enterprise no inspeccionado.

## Fuentes adicionales y cierre

Todos los enlaces CA-CQ fijan el SHA de CrewAI de §0; CV1-CV5 fijan el snapshot
Vanta. Referencias R1-R12 y hallazgos parciales válidos se conservaron.

[CA]: https://github.com/crewAIInc/crewAI/blob/73f48320dbe6e2dc7a9bc54fefc6a13637b9485c/lib/crewai/pyproject.toml#L1-L160
[CB]: https://github.com/crewAIInc/crewAI/blob/73f48320dbe6e2dc7a9bc54fefc6a13637b9485c/lib/crewai/src/crewai/crew.py#L1894-L2179
[CC]: https://github.com/crewAIInc/crewAI/blob/73f48320dbe6e2dc7a9bc54fefc6a13637b9485c/lib/crewai/src/crewai/utilities/task_output_storage_handler.py#L1-L89
[CD]: https://github.com/crewAIInc/crewAI/blob/73f48320dbe6e2dc7a9bc54fefc6a13637b9485c/lib/crewai/src/crewai/memory/types.py#L129-L280
[CE]: https://github.com/crewAIInc/crewAI/blob/73f48320dbe6e2dc7a9bc54fefc6a13637b9485c/lib/crewai/src/crewai/memory/storage/backend.py#L45-L212
[CF]: https://github.com/crewAIInc/crewAI/blob/73f48320dbe6e2dc7a9bc54fefc6a13637b9485c/lib/crewai/src/crewai/memory/unified_memory.py#L377-L527
[CG]: https://github.com/crewAIInc/crewAI/blob/73f48320dbe6e2dc7a9bc54fefc6a13637b9485c/lib/crewai/src/crewai/memory/unified_memory.py#L686-L826
[CH]: https://github.com/crewAIInc/crewAI/blob/73f48320dbe6e2dc7a9bc54fefc6a13637b9485c/lib/crewai/src/crewai/memory/recall_flow.py#L89-L388
[CI]: https://github.com/crewAIInc/crewAI/blob/73f48320dbe6e2dc7a9bc54fefc6a13637b9485c/lib/crewai/src/crewai/memory/unified_memory.py#L828-L909
[CJ]: https://github.com/crewAIInc/crewAI/blob/73f48320dbe6e2dc7a9bc54fefc6a13637b9485c/lib/crewai/src/crewai/tools/memory_tools.py#L13-L130
[CK]: https://github.com/crewAIInc/crewAI/blob/73f48320dbe6e2dc7a9bc54fefc6a13637b9485c/lib/crewai/src/crewai/agent/core.py#L778-L844
[CL]: https://github.com/crewAIInc/crewAI/blob/73f48320dbe6e2dc7a9bc54fefc6a13637b9485c/lib/crewai/src/crewai/agents/agent_executor.py#L1920-L2008
[CM]: https://github.com/crewAIInc/crewAI/blob/73f48320dbe6e2dc7a9bc54fefc6a13637b9485c/lib/crewai/src/crewai/agents/agent_builder/base_agent_executor.py#L31-L69
[CN]: https://github.com/crewAIInc/crewAI/blob/73f48320dbe6e2dc7a9bc54fefc6a13637b9485c/lib/crewai/src/crewai/agent/core.py#L1050-L1088
[CO]: https://github.com/crewAIInc/crewAI/blob/73f48320dbe6e2dc7a9bc54fefc6a13637b9485c/lib/crewai/src/crewai/experimental/evaluation/agent_evaluator.py#L208-L365
[CP]: https://github.com/crewAIInc/crewAI/blob/73f48320dbe6e2dc7a9bc54fefc6a13637b9485c/lib/crewai/src/crewai/experimental/evaluation/experiment/result_display.py#L14-L27
[CQ]: https://github.com/crewAIInc/crewAI/blob/73f48320dbe6e2dc7a9bc54fefc6a13637b9485c/lib/crewai/src/crewai/experimental/evaluation/experiment/result.py#L9-L143
[CV1]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/record/l1_writer.rs#L176-L459
[CV2]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/services/pipeline_worker.rs#L213-L353
[CV3]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/hooks/auto_recall.rs#L123-L435
[CV4]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/context_engine/engine.rs#L197-L349
[CV5]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/utils/checkpoint.rs#L88-L117

**Cobertura externa leída:** API de GitHub, issue #8659 (cuerpo y metadatos),
consulta del 2026-11-07; no se abrió PR ni se cambió estado. Descubrimiento
`websearch.search` DuckDuckGo y `metasearch.search_web` Bing/DuckDuckGo con
`site:github.com/crewAIInc/crewAI/issues memory private consolidation` produjo
cero resultados; el fetch de API GitHub recuperó el issue. No atribuir al
resultado de búsqueda vacío ausencia de incidencias. Otros hits truncados no
cuentan como issues estudiados; no se revisó historia completa ni todos los fixes.

**Cobertura de los diez ejes:** arquitectura/trazas §0-2; recomendaciones y cinco
fichas §1-2; incidencia/seguridad §3; prerrequisitos §4; rechazo/alternativas
§2/5; método/lecturas §6; runners/denominadores/costes §7; matriz §8; negativos
acotados §9; UI §10. No se acredita ejecución, soporte del adapter ni revisión
adversarial independiente final. Siguen pendientes manuales las aceptaciones
A-E, lectura de servicios enterprise, backends no recorridos y garantías
interproceso/crash físico. Son límites de esta investigación, no bugs probados.

**Plan de división si pasa a especificación:** separar A/E como integración
Python/MCP, B/C como contratos de escritura y D como diagnóstico de recall,
conservar este informe como evidencia común y no duplicar algoritmos ya presentes.
No se crea ninguno de esos archivos ahora. No tests, apps, builds, benchmarks,
instalaciones, servicios, checks globales ni operaciones Git de escritura.

### Precisiones de cierre: updates, metadata y visibilidad

La revisión conserva las cinco fichas A-E, pero discrimina tres mutaciones
que no tienen la misma garantía. Todas las referencias nuevas mantienen el SHA
CrewAI `73f48320…` y Vanta `97c9623…`; no se recalcula el inventario.

| Camino | Qué preserva / reemplaza | Frontera real y fallo posible |
|---|---|---|
| `Memory.update:888-909` [CI] | `model_copy` conserva `private/source`; `metadata` suministrada reemplaza el diccionario completo, no hace merge por clave | Read→embed→write sin revisión esperada en esta fachada: posible escritura vieja si otro actor cambia el registro |
| `EncodingFlow.execute_plans:473-490` [R2] | Conserva `existing.metadata`, pero omite los campos **de primer nivel** `private/source` | La omisión toma defaults; no es pérdida de todas las claves de metadata ni defecto de serialización JSON |
| `LanceDBStorage.update:344-363` [CR] | Serializa registro antes del delete; `_record_to_row:272-311` persiste metadata JSON, source y private por separado | `store_lock` protege delete/add localmente; dos `_do_write` no constituyen reemplazo backend indivisible ni recuperación tras crash |

`metadata` no tiene una clasificación de confidencialidad por clave en
`MemoryRecord:36-39` [R3]. `MemoryMatch.format:92-106` [R3] incluye valores
no nulos de ese diccionario al generar contexto. Marcar el registro privado
gobierna recall por source; no cifra sus atributos, no redacta eventos ni
autentica al consumidor. Por tanto B debe conservar **campos de control**
y A/D/E deben limitar **proyecciones de datos**, dos problemas distintos.
El contrato propuesto se precisa como patch tipado: omitir metadata conserva,
reemplazar metadata es explícito, y cambiar ownership requiere capability;
un campo sugerido por el LLM no puede sobrescribir el sello confiable.

**Adapter Vanta efectivamente existente [CV6]:**

1. `_to_meta:113-125` copia metadata de usuario y escribe claves reservadas
   `__mem_scope/categories/importance/created_at/last_accessed/source/private`;
   `_from_stored:128-142` las separa de metadata de usuario. No tratar esos
   nombres como un nuevo tipo Rust ni como ACL autenticada.
2. `save:165-173` hace `put` por registro; `update:187-188` reutiliza `save`.
   No reconstruir aquí otro backend. La transformación upstream que omite
   `private` llega al mapper como `False`: cambiar el motor no repara esa entrada.
   Resolver política de control en la frontera de B, no adivinar desde el texto.
3. `search:190-226` busca `limit*2+10` candidatos en un namespace y después
   filtra scopes/categorías/metadata y score. Esa cantidad es un default
   observado, **no** una recomendación nueva. Más candidatos ajenos pueden
   desplazar a los elegibles: k incompleto no significa store vacío.
4. `get_record:175-179` y `search:201-206` capturan `Exception` y devuelven
   `None`/`[]`. Es un negativo local concreto: en esas rutas la salida no
   distingue error de backend de ausencia. No demuestra que Vanta entero
   silencie errores ni que el primario pierda registros.

**A/B: pasos adicionales y aceptación futura, no ejecutados.** Inventariar
compatibilidad del adapter actual → separar fallo de vacío sin romper clientes →
conservar filtros/sellos confiables antes de selección cuando el API lo permita →
definir revisión/receipt sin prometer CAS inexistente. Si no se puede prefiltrar,
declarar cardinalidad parcial y no utilizar el resultado como autorización.
Conservar lectura del formato `__mem_*`; una migración de nombres/versiones
requiere lectura compatible, no reindexación masiva implícita.
Desactivar nuevas funciones deja legibles registros anteriores. Detectar fallo
mediante clase/operation ID, nunca volcando metadata privada.

Casos manuales pendientes: update sólo de texto conserva ownership;
reemplazo explícito de metadata no borra sellos; fallo de JSON se detecta antes
de delete; fallo entre delete/add no se anuncia como commit total; búsqueda
fallida difiere de vacía; resultados autorizados escasos se declaran parciales.
No se propone copiar locks/retries LanceDB ni imponer LLM a almacenamiento.

### Tracing externo no equivale a UI OSS ni a memoria

Además del HTML Flow, existe código **cliente** de observabilidad:
`Telemetry.__init__:90-139` [CS] configura exporter OTLP por lotes hacia
`CREWAI_TELEMETRY_BASE_URL/v1/traces`, condicionado a su comprobación de disable.
`set_tracer:141-159` usa provider propio y explícitamente evita instalarlo
globalmente en el host. `crew_creation:285-318` separa presencia de inputs de
payload opt-in, pero también envía metadata de fingerprint en la rama leída;
`task_ended:552-577` incorpora output si `share_crew`. No inferir de ese flag
una redacción universal de toda metadata o de todos los listeners.

Otra superficie es `FirstTimeTraceHandler.handle_execution_completion:47-67`
[CT]: consulta consentimiento, inicializa/envía eventos y muestra enlace si
hay `ephemeral_url`. `TraceBatchManager:424-430` [CU] forma URL de CrewAI Plus
o enlace efímero con access code. **Frontend de ese servicio no examinado**;
no se acredita aplicación desktop OSS ni CRUD de memoria por esa URL.
El código local de exportación es MIT; condiciones/retención del servicio no
quedan resueltas por esa licencia. No se iniciaron handlers ni se abrió UI.

Delta Studio: mantener desactivable la exportación y mostrar estado de entrega
sin confundirlo con receipt durable de C. Un span exportado puede sobrevivir al
borrado del hecho; la política de borrado del store no acredita borrado remoto.
No se añade sexta ficha sólo para renombrar eventos: redacción es dependencia
transversal A-E y la UI propuesta extiende Studio existente.

**Fuentes públicas reabiertas en este cierre:** página completa de
[#8659](https://github.com/crewAIInc/crewAI/issues/8659), cuerpo y metadata,
observada abierta el 2026-11-07. La página enlaza #8660 como desarrollo asociado;
su diff/merge no se leyeron y no se declara solución integrada.
El fetch API `/repos/crewAIInc/crewAI/issues/8659` devolvió rate limit;
fallback HTML por `websearch.fetchWebContent` sí recuperó el contenido.
Esta limitación de acceso no cambia la metadata congelada del repositorio.
Lagunas: backend Qdrant, frontend/retención del servicio, compatibilidad real
del adapter y aceptación A-E; revisión independiente y aceptación final pendientes.

[CR]: https://github.com/crewAIInc/crewAI/blob/73f48320dbe6e2dc7a9bc54fefc6a13637b9485c/lib/crewai/src/crewai/memory/storage/lancedb_storage.py#L272-L363
[CS]: https://github.com/crewAIInc/crewAI/blob/73f48320dbe6e2dc7a9bc54fefc6a13637b9485c/lib/crewai/src/crewai/telemetry/telemetry.py#L90-L159
[CT]: https://github.com/crewAIInc/crewAI/blob/73f48320dbe6e2dc7a9bc54fefc6a13637b9485c/lib/crewai/src/crewai/events/listeners/tracing/first_time_trace_handler.py#L47-L67
[CU]: https://github.com/crewAIInc/crewAI/blob/73f48320dbe6e2dc7a9bc54fefc6a13637b9485c/lib/crewai/src/crewai/events/listeners/tracing/trace_batch_manager.py#L424-L430
[CV6]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/integrations/crewai/vantadb_crewai/memory.py#L91-L245
[CV7]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/integrations/crewai/vantadb_crewai/vectorstore.py#L23-L105
[CV8]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/integrations/crewai/pyproject.toml#L5-L28

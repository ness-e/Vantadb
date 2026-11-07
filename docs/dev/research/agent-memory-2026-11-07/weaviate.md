---
title: "Informe estático — weaviate/weaviate"
kind: research
status: proposed
description: "Arquitectura, flujos de almacenamiento, incidencias y runners de Weaviate; contratos de adaptación incremental para VantaDB"
tags: [research, vector, weaviate]
---

# Informe estático — weaviate/weaviate

> **Identidad congelada:** `weaviate/weaviate`, clon `C:\Users\Eros\weaviate`,
> rama `main`, SHA **`85bd085e6fd6bcf2c6269c880dc49181c86bbb92`**.
> [Manifiesto de adición](clone-additions-manifest.json): captura
> `2026-11-07T17:16:48.105226+00:00`, 16.942 estrellas, shallow, sin gitlinks ni
> rutas LFS observadas. Excepción expresa al filtro original, no nueva selección.
> No se actualizó metadata, no hubo fetch/pull ni ejecución de producto.
> **Comparación:** VantaDB **`97c9623c8ef5a70fdb51aff215bfbb3b05b15612`**;
> Studio se identifica separadamente en §12. Lectura pública: 2026-11-07.
> **Licencia:** [LICENSE][license] asigna BSD-3-Clause fuera de `wl/`;
> dentro de `wl/` y descendientes, Weaviate License empresarial separada,
> derechos reservados y clave de licencia. [LICENSE-BSD][bsd] exige conservar
> copyright/condiciones/disclaimer en fuentes y redistribución binaria, y no
> usar nombres como endorsement. `NOASSERTION` de la captura no niega esta regla.
> Ni reescribir en Rust ni estudiar interfaces exime una posible derivación;
> modelos, assets, clientes y dependencias requieren revisión propia.

## Índice

[Arquitectura](#1-arquitectura) · [Tres flujos](#2-flujos-trazados) ·
[Recomendaciones](#3-recomendaciones) · [Fichas Rust](#4-fichas-candidatas) ·
[Incidencias](#5-incidencias) · [Prerrequisitos](#6-prerrequisitos-vanta) ·
[Rechazos](#7-que-no-trasladar) · [Advertencias](#8-advertencias) ·
[Benchmarks](#9-benchmarks) · [Matriz](#10-matriz-vanta) ·
[Negativos](#11-negativos-vanta) · [UI](#12-escritorio--ui) ·
[Fuentes y cierre](#13-fuentes-cobertura-y-cierre-de-autoria)

## 1. Arquitectura

### 1.1 Producto, stack y fronteras

Es un servidor de objetos/vectores, no una librería de memoria episódica.
`DB → Index por colección → Shard → Store/buckets` separa catálogo, recursos
de almacenamiento y estrategias. Un shard combina objetos, índices inverted y
uno o más índices vectoriales: no es una única transacción entre todos ellos.
La multitenencia puede dirigir a shards; esa topología no sustituye la
autenticación/autorización en los usecases. RAFT/catálogo tampoco demuestra
que cada objeto o vector esté confirmado por consenso.

| Área | Mapa y responsabilidad observada |
|---|---|
| Entradas | `cmd/weaviate-server`; handlers REST, GraphQL y gRPC. `grpc/v1/service.go::BatchObjects:262`, `Search:319/349` son superficies distintas, no APIs embebidas |
| Dominio | `usecases/objects`, `usecases/traverser`, schema/backup; `entities/models`, `entities/storobj`, `entities/filters` |
| Repositorio | `adapters/repos/db/index.go`; `shard_write_put.go`, `shard_write_batch_objects.go`, `shard_read.go`, `shard_write_delete.go` |
| KV | `lsmkv`: buckets Replace, Set/MapCollection, RoaringSet/Range e Inverted. Memtables, segmentos y logs propios; filtros y postings no son el grafo ANN |
| Vector | `vector/hnsw`, `flat`, `dynamic`, `hfresh`, `noop`; config en `entities/vectorindex`. Multivector es forma de vector/config de índice, no necesariamente un backend excluyente |
| Ciclo de vida | `shard_lazyloader.go`, `entities/loadlimiter`; backup captura referencia a shard y bloquea carga fría concurrente |
| Distribución | `cluster/` y replicación; no auditados exhaustivamente. Se leen locks/hashtree/changelog en los flujos locales, no todo el protocolo |
| Integraciones | `modules/`: vectorización, generación, reranking y backup externos. Una operación con vector aportado no requiere LLM |
| Herramientas | Runners `test/benchmark`, `test/benchmark_bm25`, benchmarks Go y demo web real `tools/dev/bench/demo_indexing_mistakes_ui` |

[go.mod:1-125,283][gomod] declara Go 1.27 y dependencias de gRPC,
go-openapi, GraphQL, Hashicorp RAFT/memberlist, bbolt, sroar, Prometheus,
OpenTelemetry y SDKs cloud. Declaración de dependencia no acredita uso en
cada request ni licencia transitiva compatible. Hay servicio local y despliegue
remoto; las rutas de benchmark pueden levantar Docker y borrar colecciones,
razón adicional para no ejecutarlas aquí.

### 1.2 Modelo e identidad

[models.Object:31-62][object] separa `Class`, UUID `ID`, `Properties`, tiempos,
`Tenant`, vector legacy y vectores con nombre. El `storobj.Object` de escritura
añade docID interno y formas `Vectors map[string][]float32` /
`MultiVectors map[string][][]float32`, como acredita [batch:311-350][batch].
UUID lógico ≠ docID usado por ANN/postings: un update puede conservar docID
o retirarlo y generar otro. `models.Class.VectorConfig` configura targets
por colección; tipo de índice, distancia, compresión y modelo de embedding
son dimensiones distintas.

No se ha acreditado en este motor un contrato de sesión conversacional,
consolidación L0/L1/L2 o entrega al contexto del modelo equivalente a Vanta.
Properties puede almacenar esos datos, pero posibilidad de almacenarlos no
equivale a política de memoria implementada. Distancia vectorial tampoco es
certeza factual, ACL ni score BM25 directamente comparable.

### 1.3 Correcciones de la versión rechazada

Se retiran «WAL por proyección como recomendación», `utils/checkpoint.rs` como
WAL, «C13 ya verificado» como tipo de Vanta, y sharding como requisito.
No se conserva «fallbacks agotados» ni «no runner localizado»: se leyeron
issues y runners concretos. Hay una demo web en el checkout, aunque no prueba
un desktop nativo ni el código de la consola SaaS. La compatibilidad de
compresión no es el producto cartesiano de nombres de config.

## 2. Flujos trazados

### 2.1 Batch → objeto LSM → inverted → vector/cola → respuesta

Orden acreditado por llamadas, no por el nombre/orden de archivos:

1. [Shard::putBatch:57-90][batch] selecciona sync o async. En sync,
   `Objects` ejecuta init → storeInObjectStore → markDeletedInVectorStorage
   → storeAdditionalStorageWithWorkers → flushWALs; **después** `putBatch`
   espera `wg.Wait`. Por ello no se puede concluir que el flush final de
   `Objects` sea una barrera posterior a todos los workers.
2. `init` crea errores por ordinal y detecta UUID repetidos; `findDuplicates`
   (522-537) ignora todas salvo última aparición. No es una respuesta
   explícita «duplicate» ni semántica idéntica al versionado de Vanta.
3. `storeSingleBatchInLSM` (149-187) comprueba cancelación inicial y paraleliza
   con límite CPU. `storeObjectOfBatchInLSM` (189-225) rechaza targets
   retirados antes de persistir, parsea UUID y llama `putObjectLSM`; comprueba
   contexto otra vez **después**, de modo que un error no prueba cero efectos.
4. [putObjectLSM:325-415][put] obtiene bucket/release, espera hashtree fuera
   del RLock, toma lock de replicación y lock por UUID; fetch anterior →
   determineInsertStatus → serialización → upsertObjectDataLSM →
   AppendChangeLogPut → hashtree. Sale del lock y actualiza inverted.
   [determineInsertStatus:503-551][put] distingue nuevo/noop/props/vector;
   hasta contenido idéntico con tiempo adelantado conserva docID y actualiza
   fila/tiempo para reconciliación: clock no es revisión MVCC.
5. [Memtable::put:304-343][memtable] toma lock y escribe commitlog antes
   de insertar en árbol/secondary. Esto no prueba fsync: `writeWAL:739-749`
   solo invoca `flushBuffers`; [commitLogger:401-452][commitlog] distingue
   `file.Sync`, cierre y `writer.Flush`. [Bucket::WriteWAL:2529-2540][bucket]
   declara explícitamente que **no vuelve atómico un batch**.
6. Se borran viejos docIDs vectoriales en bulk y se insertan nuevos mediante
   workers; índices geo también pueden fallar por ítem. En async (77-90,
   298-365), agrupa por target y `queue.Insert` empuja registros codificados
   a DiskQueue. [VectorIndexQueue:147-235][queue] aplica directamente cuando
   async=false; async=true es encolado, no readiness ANN.
7. `flushWALs` (490-520) asigna fallos de store, vector/geo y longitudes.
   Algunos fallos afectan todos los ordinales, y error al insertar grupo async
   se asigna al ordinal 0. No trasladar esa pérdida de atribución a C13.
   **Admisión previa:** `PutObjectBatch:33-51` comprueba restricciones
   previas del shard (estado read-only y cuota) y rechaza **todos** los ordinales antes de cualquier
   escritura — es la frontera que corresponde a `Err` de admisión en C13.
   **Semántica de error (leída):** `setErrorAtIndex`/`checkContext`/`flushWALs`
   (463-519) **asignan**, no acumulan: un error posterior de flush
   **sobrescribe** la causa previa del ítem, el contexto de error queda en
   sombra y los ordinales sin error quedan omitidos (nil). Alcance descrito,
   sin extrapolarlo como bug de Vanta.

**Fronteras/error/cancelación:** fila canónica, inverted, ANN/cola y flush son
etapas con errores independientes, sin rollback global acreditado.
`checkContext:472-487` devuelve false con contexto vencido, pero su bucle
continúa ante `err == nil`; no asumir que rellena errores de todos los ítems
exitosos. Es lectura de rama, no reproducción de un bug ni parche autorizado.
El `putOne:48-97` individual sí ejecuta fuente → vector → geo → WriteWALs
→ tracker. Su orden tampoco debe extrapolarse al batch concurrente.

### 2.2 Vector search + filter → allow-list → ANN/exacto → objetos

1. [ObjectVectorSearch:711-754][read] mantiene grant de admisión hasta
   retornar. Si hay filtro, `buildAllowList:1034-1045` llama
   `inverted.NewSearcher(...).DocIDs` con schema, tokenización, tenant y
   readiness de índices; falla explícitamente si no se puede construir.
2. Por target toma `AcquireVectorIndex` y defer release; valida single vs
   multivector, selecciona top-k o búsqueda por distancia, y entrega allowList
   al índice (756-858). Errores de target/shape/índice abortan, no se degradan
   silenciosamente a otro dominio.
3. [hnsw.SearchByVector:81-95][hsearch] toma lock de compresión, normaliza y
   usa flatSearch cuando la allow-list es pequeña y está permitido; si no,
   knnSearchByVector con EF. No es «todos los filtros post-ANN».
4. Tras Wait cierra allowList; combina IDs/distancias de targets y puede
   agrupar/ordenar (861-899). Fetch final toma bucket/release y usa
   `ObjectsByDocIDWithDists` (901-923): quita un objeto ausente **junto a su
   distancia**, no desplaza scores hacia el siguiente hit.

**Concurrencia/errores:** grants, refs y allow-list tienen ownership explícito;
contexto llega a búsquedas/fetch pero no equivale a snapshot global de todos
los buckets/índices. Hay consistent views y snapshots de tombstones locales
([memtable:752-777][memtable]); no acreditan MVCC multiíndice transaccional.
La consulta no hace commit; sus tiempos filter/vector/objects son desglosables.
Resultado vacío no debe confundirse con índice ausente o cold fallido.

### 2.3 Delete → tombstone fuente → inverted → flush → ANN/cola

1. [deleteObject:33-85][delete] comprueba read-only, espera hashtree,
   toma RLock de replicación, bucket/release y lock por UUID. Si no existe,
   retorna false,nil. Lee docID y updateTime; con `skipIfLocalNewer` compara
   contra deletionTime y conserva una fila igual o más nueva.
2. `Delete`/`DeleteWith` escribe tombstone con secondary docID (87-105).
   Changelog usa updateTime o tiempo explícito de borrado, **no time.Now**;
   se actualiza hashtree, luego cleanup inverted.
3. `cleanupInvertedIndexOnDelete:163-223` analiza la fila previa, resta
   longitudes, quita docID de bitmapFactory antes de eliminar postings:
   evita que un NotEquals invertido acepte un ID retirado mientras aún está
   en ANN. Incluye mirrors de migración, nested y tracking de dimensiones.
4. `store.WriteWALs` ocurre en 116-118, **antes** de queue.Delete y Flush
   vector/geo (120-158). Async delete puede ser una tarea encolada. Fallar
   después del tombstone no deshace el borrado canónico.

**Invariantes y cancelación:** lock por UUID ordena carreras locales con put;
timestamps LWW son política de reconciliación, no garantía de clock global ni
LSN/fsync. Cancelar no resucita fila. Tombstone lógico ≠ limpieza física ANN
inmediata; búsqueda final debe evitar entregar referencias huérfanas.
No se ejecutó crash/replay. C16 estudia reconciliación y diagnóstico, no un
WAL por cada tipo de proyección.

## 3. Recomendaciones

| Prioridad | Candidato seleccionado | Valor y condición de adopción |
|---|---|---|
| P1 | C13, recibo por ítem | Hacer visibles aplicación parcial, durabilidad y readiness; conservar put_batch y WriteReceipt existentes |
| P1 | C7, capacidades de scope/filtro | Exponer plan/capabilities y errores; Vanta ya tiene pre/in/post-filter y búsqueda exacta selectiva |
| P1 | C4/C14, publicación revisionada | Extender rebuild/derivados existentes con generación y publicación condicionada; no copiar orquestación RAFT |
| P1 | C16, diagnóstico/recovery/backup | Integrar auditorías reales y manifiestos; distinguir fuente/índice y no prometer recuperación medida |
| P2 | C15, matriz índice/compresión | Validar combinaciones por colección lógica/namespace; distinguir modelo, dominio y cuantización |
| P3 opcional | C9, lifecycle/leases | Admitir cargas y retener recursos sobre cuotas/scheduler; descarga solo de recursos separables acreditados |

P1/P2/P3 son orden de estudio, no tickets aprobados ni umbrales de benchmark.
Se agrupan C4/C14 en una sola ficha porque fuente-derivado y publicación de
revisión comparten estado/invariantes; no se cuenta dos veces el mismo DTO.
Rechazos y costes ocultos específicos se justifican en §7.

## 4. Fichas candidatas

### 4.1 C13 — Recibo por ítem, no una promesa de atomicidad global

1. **Problema de agente.** Un importador reintenta un lote tras timeout y no sabe qué
   memorias ya se aplicaron ni cuáles están listas para búsqueda. Un simple `Err`
   o la lista accepted/rejected pierde información sobre efectos parciales.
2. **Fuente y unidad observada.** [batch:57-147,189-225,490-520][batch] contiene
   `objectsBatcher.Objects`, rama async, `flushWALs` y errores por posición.
   [put:325-415][put] publica el objeto antes de actualizar inverted y vector.
   [queue:147-235][queue] distingue aplicación directa de encolado persistente.
   La unidad extraída es el contrato por operación/etapa, **no** copiar sus logs.
3. **Delta Vanta.** [Embedded::put_batch:1280-1323,1478-1529][v-write]
   ya valida todo el lote y confirma por chunks; devuelve
   `Result<Vec<MemoryRecord>>`. Además **ya existe** `put_certified` y
   `attestation::WriteReceipt` (1243-1277), atestación de contenido puntual.
   Ninguno acredita el recibo de progreso de cuatro ejes aquí propuesto.
4. **Alternativas decididas.** Extender SDK con API opt-in, manteniendo `put_batch`.
   Rechazados 2PC para todo el lote, traducción del protocolo Weaviate y tratar
   `WriteReceipt` criptográfico como readiness. Adapter sin cambios al motor solo
   puede reportar `Unknown` en etapas que no observa: no debe fabricar confirmación.
5. **Contrato Rust PROPUESTO.** `put_batch_observed(Vec<MemoryInput>,
   ReceiptOptions) -> Result<BatchMutationReport, BatchAdmissionError>`.
   `BatchMutationReport { items: Vec<MutationReceipt>,
   terminal_error: Option<ItemFailure> }`; `Err` **solo** negativa de
   admisión sin efectos (la validación read-only que rechaza todo el lote);
   un error tardío tras admisión devuelve `Ok(report)` con estados
   Applied/Unknown por ítem y `terminal_error` cuando afecta a todo el
   lote — `Ok` **no** significa todos los ítems exitosos; el consumidor
   inspecciona estados y `failures`.
   `MutationReceipt { ordinal: usize, namespace: String, key: String,
   version: Option<u64>, acceptance: Acceptance, application: Application,
   durability: Durability, readiness: Vec<ProjectionReadiness> }`.
   Estados separados: aceptación Accepted/Rejected; aplicación
   NotApplied/Applied/Unknown; durabilidad Buffered/Synced/Unknown;
   readiness Pending/Ready/Failed/Unknown por tipo y generación.
   `MutationReceipt` incorpora `failures: Vec<ItemFailure>`, donde
   `ItemFailure { stage, code, retryable }` mantiene **orden de ocurrencia**:
   la primera causa raíz no se sobrescribe y los fallos secundarios se
   agregan (a diferencia de la asignación que sobrescribe vista en flushWALs).
   Códigos Validation, Quota, Io, Cancelled, StaleRevision; jamás payload.
   Invariantes: una entrada por ordinal, duplicados explícitos, cuatro ejes
   independientes — ninguna inferencia `Accepted => Applied => Synced => Ready`;
   contrato propuesto, no implementado, sin WAL nuevo.
6. **Seams EXISTENTES.** `src/sdk/api/memory.rs::put_batch_inner` y su llamada
   `engine.batch_insert_with_opts`; `src/storage/engine/ops.rs::flush_pending_hnsw`;
   `src/wal.rs::WalWriter::sync` (1024-1028);
   `src/sdk/serialization/impl_index.rs::replace_derived_indexes`.
   Dominio: `vanta-memory/src/core/record/l1_writer.rs::apply_dedup_batch`,
   sin duplicar dedup ni convertir el scheduler en fuente de verdad del motor.
7. **Acciones ordenadas.** (a) Instrumentar los retornos reales por chunk/ítem;
   (b) conservar identidad y revisión escrita; (c) exponer durabilidad solo cuando
   la llamada de sync aplicable se confirme; (d) reportar índices por generación;
   (e) mantener error terminal con recibos de lo ya aplicado. Inicialmente recibo
   efímero, sin nuevo formato WAL; tras crash, Unknown y reconciliación contra
   registro actual. Persistirlo exigiría spec separada. Rollback: desactivar API
   nueva, nunca borrar chunks confirmados ni prometer rollback por cancelación.
8. **Dependencias/relaciones.** C13 funciona síncrono sin C9. C4/C14 aporta
   generaciones para readiness; C16 reconcilia tras recuperación. Reintento requiere
   precondición de revisión o token idempotente explícito: repetir `put` hoy puede
   aumentar `version`, por lo que no se promete exactamente-una-vez.
9. **FMEA.** Timeout tras KV aplicado → cliente repite y duplica revisión;
   detectar versión/ordinal y reportar Unknown, mitigar con consulta/revisión.
   Error de fsync → Applied pero no Synced; detectar error de I/O, impedir éxito
   durable. Índice atrasado → Applied pero Pending; medir lag por generación y
   permitir esperar con deadline. Cancelación no revierte mutaciones ya publicadas.
   Ítem que falla pronto y flush/cancel posterior → conservan causa raíz +
   secundarias + recibos ya aplicados, nunca un `Err` que oculte progreso.
10. **Aceptación futura.** Normal: lote mixto conserva orden y todos los estados;
    un duplicado tiene resultado propio. Adversa: fallo en segundo chunk,
    fallo de índice y cancelación entre KV/sync no ocultan el primer chunk ni
    producen Ready prematuro. Métricas por etapa/código, conteos y duración;
    claves identificadas solo mediante identificador opaco o hash apropiado.
11. **Mínimo/no objetivos/abierto.** Mínimo: recibos honestos por chunk
    expandibles a ítems, sin cambiar semántica existente. No nuevo WAL, cola
    obligatoria ni transacción distribuida. Abierto: retención de recibos,
    política de espera y representación portable de durabilidad entre backends.

### 4.2 C9 — Lifecycle de recursos opcional sobre namespaces y scheduler

1. **Problema de agente.** Muchos proyectos/tenants inactivos consumen memoria;
   una ráfaga de recall no debe cargar simultáneamente todos sus recursos.
   Es administración de recursos, no una nueva frontera de autorización.
2. **Fuente/flujo.** [LazyLoadShard:59-75,150-204][lazy]:
   mutex → comprobar puntero y bloqueo → reserva de memoria → Acquire(ctx)
   → releer clase → NewShard → publicación atómica; falla sin publicar objeto
   parcialmente cargado y libera permiso mediante defer.
   [backupShardWithHardlinks:448-482][backup] bloquea carga fría o retiene
   referencia contra shutdown. La unidad útil es el lease de recurso + admisión.
3. **Delta Vanta.** Ya existen cuotas [Config:750-782][v-config],
   `namespace_stats` y [MemoryScheduler][v-scheduler]. No está acreditada una
   unidad de índice físicamente descargable por namespace: HNSW compartido
   impide prometer que se pueda liberar uno sin rediseño. Por eso el mínimo
   controla trabajos/cachés cargables, no divide el fichero de base de datos.
4. **Alternativas.** Elegida extensión opt-in de recursos existentes.
   Un pool de bases embebidas por tenant es adapter posible, pero añade handles,
   backup y costes operativos; no se selecciona como requisito. Rechazados
   shard obligatorio, sustitución del scheduler y usar RecallScope como lease.
5. **Rust PROPUESTO.** `acquire_scope_resources(ScopeResourceRequest {
   namespace, purpose: Read|Reindex|Backup, deadline, expected_generation
   }) -> Result<ScopeResourceLease, ResourceAdmissionError>`.
   Lease RAII con `generation` y referencia al recurso; estados
   Resident/Loading/Cold/Draining/Failed. Errores Quota, Deadline,
   Cancelled, Busy, StaleGeneration. Invariantes: una carga compartida por
   generación, cuotas no se omiten, no unload mientras haya leases, auth previa.
6. **Seams reales.** `src/sdk/api/namespaces.rs::namespace_stats`;
   `src/config.rs::Config.max_namespace_records/max_namespace_bytes`;
   `vanta-memory/src/services/scheduler.rs::run_pass`,
   `MemoryScheduler::shutdown`; `src/storage/engine/ops.rs::flush_pending_hnsw`
   para la barrera existente, no para inventar unload por namespace.
7. **Acciones.** Inventariar recursos realmente separables → añadir admisión y
   estado de trabajo → coalescer cargas → lease/release → drenaje con barrera
   → solo después estudiar descarga opt-in. Estado de residencia efímero;
   configuración versionada con defaults compatibles; trabajos recuperables en
   checkpoint existente. Desactivar vuelve a eager/residente; no mover datos ni
   eliminar índices. Rollback espera leases, no mata usuarios activos.
8. **Dependencias.** Cuotas y scheduler existentes. C7 autoriza/resuelve scope
   antes de admitir recursos; C16 necesita lease para snapshot; C13 no depende
   de tener tenants fríos. Sin evidencia de ahorro no pasar a partición física.
9. **FMEA.** Ráfaga fría → thundering herd/OOM; detectar cola y reservas,
   mitigar single-flight y límite. Cancelar un solicitante compartido → carga
   requerida por otro; ownership del job separado del waiter. Backup+unload →
   archivos cambiantes; lease y barrera antes de snapshot. Drop+load →
   publicación obsoleta; validar generación antes de publicar.
10. **Aceptación futura.** Normal: dos lectores comparten una carga y liberan
    leases. Adversa: memoria agotada, deadline, backup y drop concurrentes no
    dejan permisos fugados ni acceso sin auth. Observar residentes/cargando,
    rechazos y tiempo de espera con labels acotados; sin nombres/payloads libres.
11. **Mínimo/no objetivos/abierto.** Admisión y leases sobre recursos existentes.
    No sharding, RAFT ni tenant=ACL. Abierto: qué caché concreta justifica cold,
    equidad entre tenants y límite de memoria medible, sin fijar umbrales aquí.

### 4.3 C7 — Capacidades de búsqueda/filtro y scope verificables

1. **Problema de agente.** Recall con filtro por proyecto/estado no debe devolver
   vacío indistinguible de «índice requerido no existe», ni cambiar silenciosamente
   la semántica de un filtro al pasar de SDK a otro transporte.
2. **Fuente/flujo.** [ObjectVectorSearch:736-865,1034-1045][read] construye
   allow-list, adquiere target y valida forma; [SearchByVector:81-95][hsearch]
   elige búsqueda exacta para subconjunto pequeño. La [demo:48-75][demo-js]
   distingue HTTP/GraphQL errors; el ejemplo de filtro sin índice no es una
   prueba de vulnerabilidad de auth. Unidad mínima: capabilities y plan explícito.
3. **Delta Vanta.** [vector_memory_search:129-235][v-vector] ya valida
   dimensión, estima selectividad, hace prefilter exacto o máscara InFilter.
   [MemorySearchRequest:102-172][v-request] ya incluye namespace, metadata,
   tiempo, cuarentena, perfiles y rangos. `RecallScope`, `RecallConfig`,
   `InjectionPolicy` ya gobiernan recall. Delta: exposición consistente de
   soporte/plan/causa; no añadir ANN ni afirmar que Vanta solo postfiltra.
4. **Alternativas.** Extender request/explain y adapters existentes. Rechazado
   fork de HNSW/ACORN, usar filter como autorización y «fallback a todo el
   namespace» ante un operador no soportado. Exact scan es fallback solo si
   conserva semántica y presupuesto, con plan visible.
5. **Rust PROPUESTO.** `search_capabilities(namespace: &str) ->
   Result<SearchCapabilities>` y `plan_search(&MemorySearchRequest,
   &AuthorizedScope) -> Result<ScopedSearchPlan, SearchPlanError>`.
   Salida: operadores soportados, plan Exact/MaskedAnn/PostFiltered,
   índice/generación, unidades de score, motivo de fallback.
   Errores UnsupportedFilter, MissingIndex, UnauthorizedScope,
   DomainMismatch, BudgetExceeded, StalePlan. `AuthorizedScope` es una
   propuesta, no alias de RecallScope. Invariantes: filtro no se elimina;
   scope autorizado se intersecta antes de recuperar/inyectar; plan no es grant.
6. **Seams reales.** `src/sdk/search/mod.rs::search_impl`;
   `src/sdk/search/vector.rs::vector_memory_search`;
   `src/sdk/serialization/vector_types.rs::MemorySearchRequest`;
   `vanta-memory/src/core/hooks/auto_recall.rs::perform_auto_recall_governed`
   e `InjectionPolicy`. Reusar explain existente, no un segundo buscador.
7. **Acciones.** Inventariar operadores por transporte → describir capacidades
   derivadas de código → validar plan → ejecutar ruta existente → emitir
   explain sin contenido → probar paridad. Capabilities efímeras, versionadas
   por schema/generación; no migración de datos. Un cambio de índices invalida
   plan/cursor; rollback quita API opt-in sin ampliar acceso ni perder filtros.
8. **Dependencias.** C15 informa compatibilidad/dominio; C4/C14 informa
   generación; C9 puede admitir recursos **después** de auth. No necesita
   un catálogo distribuido ni exige nuevos índices para funcionar.
9. **FMEA.** Capabilities cacheadas → plan usa índice retirado; detectar
   generación, replanificar o StalePlan. Filtro desconocido ignorado → fuga
   de scope; validación fail-closed. Deadline en exact scan → costo excesivo;
   cancelación/contadores y BudgetExceeded, no resultado «completo» truncado.
10. **Aceptación futura.** Normal: filtro selectivo usa ruta declarada y
    mismos IDs que baseline existente. Adversa: operador no soportado, cambio
    de generación y scope denegado no producen consulta sin filtro. Métricas
    por plan/código, cardinalidad y duración; no valores del filtro/payload.
11. **Mínimo/no objetivos/abierto.** Catálogo de capabilities + explicación
    del routing actual. No reinventar ANN/BM25/RRF ni garantizar auth universal
    por inspeccionar un hook. Abierto: matriz exacta de paridad SDK/server/Studio.

### 4.4 C15 — Matriz índice × compresión por colección lógica

1. **Problema de agente.** Un proyecto cambia de embedder y asume que vectores
   de la misma dimensión son intercambiables; otro habilita una compresión
   no soportada y obtiene errores tardíos o degradación oculta.
2. **Fuente/flujo.** [hnsw.UserConfig:24-128][hconfig] separa distancia,
   PQ/BQ/SQ/RQ, multivector y EF. [flat.parseCompression:194-228][fconfig]
   rechaza múltiples métodos, PQ y SQ; RQ acepta bits 1 u 8. [dynamic:
   24-54,90-104][dconfig] tiene threshold 10.000 y rechaza multivector.
   [GetCompressedBucketName:33-38][helpers] separa targets; issue #9274
   acredita el coste de mezclar almacenamiento derivado con identidad incompleta.
3. **Delta Vanta.** [VecIndex:23-61][v-index] y routing de
   `vector_memory_search` ya abstraen índices. [sq8_quantize:140-164][v-quant]
   ya implementa cuantización. No se acredita equivalencia con PQ/BQ/RQ de
   Weaviate ni manifest público por colección con todo el producto de opciones.
   Colección lógica sobre namespace es adaptación propuesta, no tipo existente.
4. **Alternativas.** Elegida validación/catálogo sobre índices actuales.
   Rechazadas importación de todos los backends, tratar `noop` como brute-force,
   y auto-migrar al detectar otra dimensión. Adapter puede declarar manifest,
   pero no asegurar representación física inexistente.
5. **Rust PROPUESTO.** `validate_index_spec(CollectionIndexSpec {
   namespace, revision: u64, index: IndexType, compression: CompressionSpec,
   domain: EmbeddingDomain { model_id, model_revision, dimensions, metric,
   normalization }, vector_shape }) -> Result<ValidatedIndexSpec, SpecError>`.
   Salida incluye tabla de compatibilidad y necesidad de rebuild.
   Errores UnsupportedCombination, DomainMismatch, RebuildRequired,
   StaleRevision, UnknownModel. Modelo es identidad semántica; cuantización
   representación; dominio incluye métrica/normalización. No inferir uno del otro.
6. **Seams reales.** `src/index/mod.rs::VecIndex`,
   `src/index_port.rs::IndexType`, `src/sdk/search/vector.rs::vector_memory_search`,
   `src/vector/quantization.rs::sq8_quantize`;
   `src/sdk/serialization/impl_index.rs::write_derived_index_state`.
   Este último es seam de estado existente a extender, no manifest ya completo.
7. **Acciones.** Enumerar combinaciones efectivas → validar al configurar y
   escribir/buscar → persistir manifest opt-in → construir nueva generación
   con C4/C14 → publicar solo al validar. Legado: dominio Unknown, no atribuir
   un modelo por dimensiones. Rollback conserva índice/manifest anterior y
   lectores pinneados; cambiar modelo requiere reembedding explícito.
8. **Dependencias.** C7 consume capacidades, C4/C14 publica cambios y C16
   diagnostica archivos/generaciones; C13 informa readiness. No exige C9.
9. **FMEA.** Modelo distinto misma dimensión → vecinos semánticamente
   inválidos; detectar identity mismatch, rechazar mezcla. Dos targets comparten
   path → corrupción derivada; namespace/target/generación en identidad física.
   Cuantizador incompatible → datos ilegibles; validar antes de persistir y
   mantener fuente original o política de reingesta explícita.
10. **Aceptación futura.** Normal: configuración persistida y visible;
    adversa: combinaciones inválidas y modelo cambiado no se degradan a otro
    índice. Reinicio conserva dominio/manifest y valida generación. Observar
    tipo, representación y motivo de rechazo, nunca vectores/model credentials.
11. **Mínimo/no objetivos/abierto.** Matriz de combinaciones realmente
    existentes y manifest opcional. No implementar PQ/HFresh/MUVERA por
    comparación nominal. Abierto: granularidad colección/namespace y política
    de vectores legacy sin identidad, antes de una migración obligatoria.

### 4.5 C4/C14 — Fuente, derivados y publicación revisionada

1. **Problema de agente.** Reindexar memoria mientras llegan ediciones/borrados
   no debe hacer visible una mezcla de generaciones ni resucitar una revisión
   retirada. Un checkpoint de progreso no acredita datos derivados durables.
2. **Fuente/flujo.** [MigrationRecord:28-119][migration] tipa
   Iterating/Iterated/Merged/Swapped/Promoted e identidad
   TaskVersion+StrategyCode+UnitID, no solo contador global.
   [MigrationRecordStore.Put:542-566,writeFileAtomic:731-761][migration-store]
   escribe temporal → Sync → Close → RenameAndSync.
   [Store.SwapBucketPointer:813-859][store] cambia mapa bajo lock y devuelve
   bucket desplazado; `FinalizeBucketSwap:862-925` es **solo startup**.
   [migrationReconciler:155-245][reconcile] conserva/frena/reconcilia estado
   incompleto. La documentación de reindex tiene secciones históricas
   declaradas: no se tratan como protocolo vigente.
3. **Delta Vanta.** [replace_derived_indexes][v-derived] ya reemplaza derivados;
   [ensure_indexes_current:22-76][v-derived] comprueba schema/conteos y
   reconstruye; [rebuild_derived_indexes_with_report:16-68][v-rebuild] ya
   reconstruye desde nodos canónicos. Delta: publicación por generación y
   cobertura de revisión explícitas, no «crear provenance/rebuild desde cero».
4. **Alternativas.** Extender estado/rebuild existentes. Mínimo válido puede
   mantener mantenimiento offline exclusivo. Online mediante staging solo si
   se acredita catch-up de escrituras y leases. Rechazados RAFT/2PC global,
   nuevo WAL de proyección y usar tiempo de pared como revisión.
5. **Rust PROPUESTO.** `prepare_projection(ProjectionBuildRequest {
   namespace, kind, expected_source_revision, generation, spec_revision
   }) -> Result<ProjectionBuildHandle, ProjectionError>`;
   `publish_projection(handle, expected_current_generation) ->
   Result<PublishedProjection, ProjectionError>`.
   Handle posee staging y checkpoint; salida incluye watermark verificable.
   Errores StaleSource, StaleGeneration, Cancelled, IncompleteCatchup, Io,
   UnsupportedFormat. Invariantes: CAS al publicar; nunca Ready antes de
   validar; lector retiene generación; progreso no avanza antes de datos.
6. **Seams reales.** `src/sdk/serialization/impl_rebuild.rs::
   rebuild_derived_indexes_with_report/rebuild_text_index_with_report`;
   `src/sdk/serialization/impl_index.rs::DerivedIndexState` (tipo usado allí),
   `write_derived_index_state/replace_derived_indexes`;
   `src/storage/engine/ops.rs::drain_hnsw_batch_locked`.
   Checkpoint de tareas: `vanta-memory/src/utils/task_checkpoint.rs::
   TaskCheckpointManager`. No modificar `src/wal.rs` por esta adaptación.
7. **Acciones.** Clasificar fuente vs derivados → identificar revisión fuente
   disponible (sin inventar contador común) → construir aislado → confirmar
   persistencia/catch-up o bloquear escrituras en mantenimiento → CAS publicar
   → retener anterior hasta liberar lectores → GC. Manifest versionado
   compatible; legado se abre en generación inicial explícita tras validación.
   Cancelación pre-publicación descarta staging; post-publicación no deshace
   lectores ni elimina fuente. Rollback publica generación anterior solo si
   cubre revisiones requeridas; si no, rebuild desde fuente.
8. **Dependencias.** C15 congela spec; C13 informa readiness; C16 verifica
   integridad; C9 aporta lease opcional, no sharding obligatorio. El scheduler
   existente ejecuta trabajo, no autoriza unilateralmente publicación.
9. **FMEA.** Rebuild sin recoger delete concurrente → resurrección;
   detectar watermark/version mismatch, bloquear publicación o catch-up.
   Crash tras rename antes de estado → manifest ambiguo; conservar generaciones
   y reconciliar, no borrar por nombre inferido. Cleanup viejo con lector vivo
   → handle inválido; refcount/lease y GC diferido.
10. **Aceptación futura.** Normal: lectura permanece en generación anterior
    hasta swap válido. Adversa: update/delete/cancel/restart en cada frontera
    no publica parcial ni elimina fuente; spec obsoleta falla. Medir generación,
    fase, lag y códigos; no texto, embedding ni rutas locales sensibles.
11. **Mínimo/no objetivos/abierto.** Rebuild offline revisionado y diagnóstico;
    online es siguiente decisión, no promesa de esta ficha. No copiar swap
    de directorios en vivo. Abierto: autoridad de revisión por backend, contrato
    de catch-up y persistencia atómica del manifest.

### 4.6 C16 — Diagnóstico de recuperación, integridad y backup

1. **Problema de agente.** Tras reinicio, «proceso vivo» no prueba memoria
   recuperable ni índices listos. Operadores necesitan discriminar pérdida de
   fuente, derivado reconstruible e incompatibilidad antes de reingerir.
2. **Fuente/flujo.** [recoveredRun:255-409][recover] separa stage,
   commit por rename con fsync de directorio y mount. Un commit parcial
   mantiene WAL como marcador provisional; colisión de nombre se rechaza
   antes de reemplazar. [backupShardWithHardlinks:416-491][backup] mantiene
   backupLock, bloqueo de lazyload y preventShutdown antes de snapshot.
   Issues #4262/#9274 aportan mecanismos, no una certificación de crash safety.
3. **Delta Vanta.** [verify_wal_file:1841-1874][v-wal-verify], auditoría textual shallow/deep
   en [cmd_audit_index:118-133][v-audit], rebuild y [BackupManifest:25-59][v-backup]
   con CRC32C ya existen. Delta: informe agregado fuente↔vector↔texto↔derivados,
   revisión y acciones propuestas; no nuevo verificador/WAL ni backup desde cero.
4. **Alternativas.** Extender CLI/SDK y presentar read-only en Studio.
   Rechazados borrar WAL/índices automáticamente al ver fallo, tratar checksum
   como prueba de coherencia semántica y restaurar sobre base activa compartida.
5. **Rust PROPUESTO.** `diagnose_recovery(RecoveryDiagnosticRequest {
   namespace: Option<String>, depth: Shallow|Deep,
   expected_generation: Option<u64> }) ->
   Result<RecoveryDiagnosticReport, DiagnosticError>`.
   Report incluye integridad fuente/WAL, generaciones, conteos huérfanos/
   faltantes, backup manifest compatible y remedios RequiresRebuild/
   RequiresRestore/NeedsInspection. Errores Busy, Permission, Io,
   UnsupportedFormat, StaleSnapshot; Unknown no se convierte en Healthy.
6. **Seams reales.** `src/cli_handlers/index.rs::cmd_audit_index`,
   `src/wal.rs::verify_wal_file`, `src/sdk/serialization/impl_rebuild.rs`,
   `src/cli_handlers/backup.rs::cmd_backup/cmd_restore`.
   Studio: `desktop/src/vanta.ts::metrics/namespaceStats`,
   `desktop/src-tauri/src/commands/data.rs::vanta_namespace_stats`; estos
   comandos no ofrecen ya el informe nuevo: habría que extender DTO/bridge.
7. **Acciones.** Agregar resultados existentes sin mutar → declarar cobertura
   por componente → pin de generación/lease → correlacionar IDs/revisiones
   sin payload → sugerir remedio → operador autoriza repair/restore existente.
   Informe efímero; export opcional sin datos. Migración solo de versión de
   diagnóstico, no WAL. Restauración futura en directorio aislado y validación
   previa a sustituir handle; rollback conserva backup/fuente originales.
8. **Dependencias.** C4/C14 identifica generaciones, C15 formatos/domino,
   C13 evita confundir encolado con listo; leases C9 si disponibles. Puede
   empezar como agregación read-only antes de cualquiera de las ampliaciones.
9. **FMEA.** Conteos iguales con IDs diferentes → falso Healthy; deep compara
   identidad/revisión, no solo cantidad. Backup contra unload/reindex →
   copia inconsistente; detectar generación cambiante y usar snapshot/lease.
   Archivo corrupto → parser falla; conservar evidencia, no saltar silencioso.
   Diagnóstico contiene rutas/IDs → filtración; redacción y permisos administrativos.
10. **Aceptación futura.** Normal: base consistente muestra componentes
    comprobados y límites. Adversa: derivado ausente vs fuente dañada produce
    remedios distintos, fallo parcial no dice Healthy, cancelación libera
    handles. Métricas por componente/estado, discrepancias agregadas y duración.
11. **Mínimo/no objetivos/abierto.** Agregar verificación/auditoría/backup y
    navegación read-only. No reparar automáticamente ni nuevo WAL/replicación.
    Abierto: coste deep, cobertura vectorial y contrato uniforme de snapshot
    entre backends; sin medirlos no se promete cero downtime.

## 5. Incidencias

### 5.1 Fuentes públicas efectivamente leídas

Revisión documental **2026-11-07**. Estado/fechas proceden de GitHub REST
`GET /repos/weaviate/weaviate/issues/{n}` o Search API, no de deducirlos
del código. El checkout fijado no se actualizó para seguir main.

| Incidencia, estado y fechas acreditadas | Mecanismo y revisión declarada | Contraste con pin y aplicabilidad |
|---|---|---|
| [#3948](https://github.com/weaviate/weaviate/issues/3948), **closed**, creada 2023-12-27; cierre/última actualización 2024-02-21 | Updates solo de propiedades retiraban docID y acumulaban tombstones HNSW; ejemplo del autor repite 10.000 updates. Es su reproducción, no medida nuestra | [determineInsertStatus:503-551][put] conserva docID si vector igual; si tiempo avanza evita noop para refrescar fila/hashtree. Se acredita implementación actual del mecanismo, no fecha exacta de introducción ni rendimiento. C4/C14: separar revisión fuente de necesidad de reembedding; Vanta no queda acusado de crear esos tombstones |
| [#3949](https://github.com/weaviate/weaviate/issues/3949), **closed**, creada 2023-12-27; cierre/actualización 2024-02-21 | Solicita batch mínimamente invasivo: noop, props-only, vector cambiado y nuevo; advierte lectura previa/replicación puede encarecer import inicial | [batch:367-386][batch] omite skipUpsert y [put:503-551][put] determina estado. Rechazar «reintento siempre idempotente»: una revisión temporal nueva puede actualizar fuente aunque vector igual. Vanta `put_batch` versiona duplicados; C13 necesita contrato propio, no copiar equivalencia de contenido |
| [#4262](https://github.com/weaviate/weaviate/issues/4262), **open**, creada 2024-02-19; actualización 2025-07-19 | Crash/OOM durante bulk, replay independiente deja IDs en searchable sin objeto, o fuente sin posting. Declara 1.23+ y posiblemente anteriores. Propone sincronizar intersección o reindexar desde fuente, con costo alto | [comentario 2024-05-23](https://github.com/weaviate/weaviate/issues/4262#issuecomment-2126884361) lo califica comportamiento previsto a mejorar; los bots stale no son fix. El pin filtra objetos ausentes en fetch y tiene recovery/reindex, pero **no** se acredita que solucione toda la inconsistencia cross-bucket. Motiva diagnóstico C16 y readiness C13, no demuestra el mismo fallo en Vanta |
| [#9274](https://github.com/weaviate/weaviate/issues/9274), **closed**, creada 2025-09-22; actualización 2025-10-02 | Named vectors comprimidos + async + terminación abrupta; `stable-v1.31-443654b`/1.31.16 en comentario. Reporta directorio comprimido vacío y que esperar indexing no evitó el fallo | [comentario de cierre](https://github.com/weaviate/weaviate/issues/9274#issuecomment-3359546284) enlaza [PR #9240](https://github.com/weaviate/weaviate/pull/9240). API de PR confirma **merged 2025-10-02T07:00:03Z**, merge SHA `278683c8a2df92437d62b19cee0f04e8a45ae232`. Diff leído identifica colisión del bucket comprimido entre targets; [helpers:33-38][helpers] conserva sufijo por target en el pin. Se acredita ese fix concreto, no que todo crash+async esté resuelto |

Para #9274 se leyeron body, comentarios y metadatos/diff de la PR, en
particular helper de nombres y cambios de llamadas; no se ejecutó el adjunto
Python ni se certificaron todas las regresiones de la PR. #4262 ilustra una
controversia de contrato («bug» frente a pérdida admitida), no dos hechos
contradictorios resueltos por escoger el título más alarmante.

**Validación discriminante futura:** mantener mismo objeto lógico, cambiar
solo metadata frente a vector y frente a timestamp; observar fuente, docID y
readiness por target. Para recuperación, interrumpir en fronteras de fuente/
postings/cola/flush sobre datos desechables y comparar identidades, no solo
conteos. Ninguno de esos experimentos se ejecutó ni se convierte aquí en
un harness, configuración o umbral nuevo.

### 5.2 Cascada de acceso y límites

- `websearch.search` DuckDuckGo para issues batch/filter/WAL devolvió **0**
  resultados; `metasearch.compare_engines` para issues WAL con Bing y
  DuckDuckGo también devolvió conjuntos vacíos. Son fallos de descubrimiento,
  no prueba de ausencia ni agotamiento de fuentes.
- Se descubrieron esquemas mediante `mcp_list_tools`; dos llamadas
  `github-mcp-server.issue_read` para #3948/#3949 fallaron con
  **Failed to connect to MCP server**. No se adivinaron nombres de herramientas.
- `websearch.fetchWebContent` a GitHub #3948 sí devolvió body legible.
  Fallback HTTP directo mediante `Invoke-RestMethod` a API pública:
  issues #3948/#3949, `/search/issues?q=repo:weaviate/weaviate+is:issue+WAL+corruption`,
  comentarios de #4262/#9274 y PR #9240 + `/files` **funcionaron**.
  Search API entregó también bodies #5337/#8624/#4125; no se desarrollan como
  diagnósticos actuales porque no se siguieron sus fixes/código completos.
- Una primera proyección PowerShell de arrays de comentarios de #3948/#3949
  mostró propiedades null: error de presentación, no ausencia acreditada de
  comentarios. No se usa para afirmar que no existan.
- Se leyó documentación oficial Explorer mediante búsqueda web y fetch;
  el fetch devolvió 6.000 caracteres y quedó **truncado**. Solo se usan
  introducción, acceso, Browse/Hybrid/Semantic/Keyword presentes en esa parte.
  No fue necesario Argus/Jina/browser para las cuatro incidencias resueltas;
  no se dice haber agotado fallbacks no usados.

## 6. Prerrequisitos Vanta

| Candidato | Ya disponible en `97c9623c8ef5a70fdb51aff215bfbb3b05b15612` | Trabajo previo local y límite |
|---|---|---|
| C13 | put_batch por chunks, put_certified/WriteReceipt, WAL sync, derivados | Definir atribución de error y estado Unknown antes de API nueva. No falsear atomicidad global ni implementar nuevo WAL |
| C9 | cuotas Config, namespace_stats, MemoryScheduler y shutdown | Inventariar recursos cargables y ownership; no prometer unload de HNSW compartido. No bloquea C13/C7 |
| C7 | MemorySearchRequest, pre/in/post-filter, exact scan, explain, RecallScope/InjectionPolicy | Tabla real de soporte por transporte; comprobar frontera auth del caller. No rediseño ANN |
| C15 | IndexType/VecIndex, routing y cuantización existentes | Determinar manifest por namespace y legado Unknown; publicar compatibilidad efectiva, no toda combinación upstream |
| C4/C14 | replace_derived_indexes, ensure_indexes_current, rebuild con reportes, TaskCheckpointManager | Identificar autoridad de revisión y barreras por backend; si no hay catch-up acreditado, mantenimiento offline |
| C16 | verify_wal_file, auditoría texto deep, backup/restore con manifiesto CRC | Acordar snapshot y cobertura por componente; agregado read-only primero. Reparación/restore solo con autorización |

No hay bloqueo universal C1 ni se exige cambiar L1, MVCC, checkpoint global
y WAL a la vez. [persist_planned/apply_dedup_batch][v-l1] son seams de
dominio separados de la persistencia del motor. Plan de dedup no acredita
commit atómico del lote ni entrega al contexto del modelo.

## 7. Que no trasladar

| Idea evaluada | Decisión y coste oculto |
|---|---|
| Nuevo WAL por proyección | **Rechazada.** El orden local de commitlog no resuelve atomicidad cross-bucket; #4262 lo hace explícito. Vanta ya tiene `src/wal.rs`, sync y verificación. Multiplicar logs amplía recovery/formato y deuda operativa |
| Shard físico por tenant obligatorio | **Rechazada.** Sobrecarga de archivos/handles/backups; no crea auth. C9 se limita a admisión/leases opcionales sobre recursos realmente separables |
| RAFT, gossip, two-phase cluster-wide de reindex | **No aplicable al núcleo embebido.** Serían otro producto/consistencia operativa, no una mejora de memoria local |
| Trasladar HNSW/ACORN/HFresh/MUVERA completos | **No seleccionado.** Vanta ya dispone índices/routing; este estudio no mide beneficio ni audita todos esos algoritmos/licencias. C7/C15 extraen contratos, no un fork ANN |
| `noop` = búsqueda exacta | **Rechazada inferencia nominal.** Brute force acreditado en flat/SearchByVector; no adjudicarlo a noop sin seguir su implementación |
| Módulos LLM/vectorizer obligatorios | **Rechazado.** Añaden red, claves, costes y dependencia de proveedor al motor. Embeddings aportados y primitivas sin LLM deben seguir funcionando |
| Noop por contenido idéntico en toda capa | **Rechazado como regla universal.** Weaviate refresca timestamps para reconciliación; Vanta usa versión/provenance. Se debe decidir semántica, no optimizar suprimiendo eventos necesarios |
| Copiar UI demo/proxy o consola SaaS | **Rechazado.** La demo modifica schema y no es desktop; el SaaS tiene otra frontera. Studio ya existe y debe extender su bridge tipado |
| Checksum = backup coherente = search ready | **Rechazada equivalencia.** Byte integrity, coherencia de revisión y disponibilidad de índices son contratos independientes |

## 8. Advertencias

1. Un método que devuelve error después del upsert no garantiza ausencia
   de efectos. Un batch con errores por ítem tampoco demuestra atomicidad.
   En sync los workers y flush tienen el orden descrito en §2, no el idealizado.
2. `writer.Flush`, fsync de archivo, fsync de directorio, publicación de
   puntero y mount son barreras diferentes. No extrapolar el fsync de rename
   en recovery a la respuesta de todas las operaciones.
3. `LastUpdateTimeUnix`, tiempo de borrado y revisión lógica no son LSN,
   MVCC ni clocks sin deriva. Un lock por UUID ordena un proceso, no toda
   réplica. Consistent view local no acredita snapshot transaccional multiíndice.
4. Dimensión igual no prueba mismo modelo; cuantización no define dominio.
   [Flat config][fconfig] incluso declara campos PQ/SQ que luego rechaza:
   listar campos no acredita compatibilidad. Multivector tampoco es
   intercambiable con single-vector o multi-target.
5. Estado closed/stale no certifica fix. Los títulos históricos y test names
   se usan como pistas; las conclusiones se apoyan en body/código concreto.
   No se comprobó exhaustivamente vigencia de todas las incidencias del proyecto.
6. La demo y runners son herramientas con efectos destructivos; no arranque
   automático ni comandos «de prueba» sobre un servicio compartido. Su código
   contiene defaults de red que no deben copiarse a producto.
7. La comparación conserva pin y licencia por subárbol. No se revisó `wl/`
   para adopción ni todas las dependencias/modelos; lectura estática no
   certifica seguridad multitenant, rendimiento ni crash recovery.

## 9. Benchmarks

Se corrige el negativo anterior: **sí existen runners completos**, además de
benchmarks Go. Lo siguiente describe código leído, no resultados obtenidos.
La aplicación respeta [METHOD](METHOD.md) y [BENCHMARK-NOTES](BENCHMARK-NOTES.md),
sin crear harness, prompts, scoring, configuraciones ni umbrales.

### 9.1 Runner de servicio SIFT

Fuentes [benchmark.go:39-155,203-261][bench-main] y
[benchmark_sift.go:33-59,72-134,137-245][bench-sift]:

- Defaults: `name=SIFT`, `numberEntries=100000`, `numBatches=1`, `fail=-1`;
  usa `http://localhost:8080/v1/`. Fichero con prefijo de dimensiones por
  vector; no se descargó dataset ni se constató completitud local.
  **Lectura efectiva (leída en `benchmark_sift.go:72-175`):** el bucle con
  `i=0` hace `append` **antes** de comprobar `i>=maxObjects`; con fichero
  suficiente y `maxObjects` no negativo lee `maxObjects+1` objetos, y las
  queries usan `maxObjects/100` → lee `floor(maxObjects/100)+1`. Con los
  defaults congelados (100000/numBatches=1) lee **100001 objetos y 1001
  queries**: nominal ≠ efectivo, condicionado a fichero suficiente. Sin
  datos ni mediciones nuevas, solo el código.
- Crea clase `Benchmark`, vectorizer none, distancia `l2-squared`,
  ef=-1, efConstruction=64, maxConnections=64. No son los defaults generales
  de HNSW en el pin, ni necesariamente comparables a Cosine en Vanta.
- `performRequest` mide milisegundos enteros hasta `http.Client.Do`, **antes**
  de leer todo response body. BatchAdd suma duración de batches concurrentes,
  no tiempo de pared global de importación. Query acumula tiempos de cada
  request: no distribuciones p50/p95/QPS calibradas.
- Envío de objetos = `numBatches*floor(len(objects)/numBatches)`; un resto
  posible no se envía. EOF anticipado puede truncar la lectura o provocar
  panic según la guardia `len<maxObjects`. Verifica HTTP 200 pero no
  interpreta errores por objeto del body del batch.
  GetObjects comprueba 79 resultados (o menos si maxObjects menor);
  queries GraphQL comprueban data/no errors, **no vecinos contra ground truth**.
- El porcentaje impreso es `100*(nuevo-viejo)/nuevo`; el gate usa
  `viejo*(1+fail/100) < nuevo`. No equiparar ambas fórmulas ni atribuir
  significancia estadística. Sobrescribe `benchmark_results.json`.
- **Efectos:** `clearExistingObjects` elimina **todas las clases** existentes;
  puede ejecutar `run_ci_server.sh`, construir/arrancar Docker y después
  `docker-compose down --remove-orphans`. No ejecutar contra runtime compartido.

**Adaptación real a Vanta:** pista motor/import/query, nunca calidad de memoria.
Mantener métrica/distancia/cardinalidad y presupuesto equivalentes usando
las APIs existentes; separar confirmación por ítem (C13) de latencia/readiness.
No trasladar sumas de latencias como throughput ni crear nuevos scripts aquí.

### 9.2 Runner BM25/híbrido con datasets JSONL

Fuentes [root.go:37-50][bm-root], [import.go:87-175][bm-import],
[query.go:117-232][bm-query], [ParseQueries:23-91][bm-data],
[Scores:20-70][bm-scores] y [AnalyzeLatencies:20-57][bm-times]:

- Defaults: origin localhost:8080; dataset-config `datasets.yml`, batch 100,
  todas las queries (`count=-1`), sin filtro, alpha=0 (BM25 puro),
  ranked_fusion, limit=100, sin vectorizer/explanations/detailed output.
  El fichero ejemplo se llama `datasets_default.yml`: no asumir que está
  instalado como el default. Enumera MS MARCO, NFCorpus, FiQA, Quora y
  Webis-Touche; solo algunos declaran matching_results/property_with_id.
- Import borra/recrea la colección del dataset; añade `modulo_10/100/1000`,
  usa Batch API y `HandleBatchResponse`. Esto sí valida respuesta de ítems,
  a diferencia del SIFT anterior. No se ejecutó import ni se generaron datos.
- Queries secuenciales GraphQL; alpha>0 cambia a híbrido. Filtro artificial
  `modulo_100 > FilterObjectPercentage` no equivale por sí solo a ese porcentaje
  exacto de documentos descartados. No se recalcula aquí relevancia tras filtro.
- Duración `time.Since(before)` se toma tras respuesta sin errors; incluye
  llamada cliente/decodificación, no solo kernel BM25. Ante error retorna,
  no agrega muestra fallida; no certificar corrida incompleta como completa.
  `QueriesPerSecond=1/mean.Seconds()` es inversa de media secuencial, no carga
  concurrente. Percentiles son índices sobre muestras ordenadas p50/p90/p99,
  **no p95** aunque un comentario del struct diga p95 para el campo P99.
- nDCG usa relevancia binaria, `DCG/IDCG` con todos los matchingIds en el
  ideal; `numQueries` crece solo en `AddResult`. El caller omite ese agregado
  cuando falta gold o propiedad ID. P@1 divide hits por queries puntuadas;
  P@5 divide hits por queries puntuadas **y por 5**, no recall-any@5.
  Latencia usa todas las queries completadas, calidad solo las puntuadas:
  denominadores distintos. Un dataset sin gold puede medir velocidad sin
  medir calidad; no inventar score 0 como fallo semántico.

**Adaptación real:** útil para motor lexical/híbrido y C7 (selectividad/plan);
mantener IDs y anotaciones relevantes, declarar modo con/sin vectorizer y
errores operativos. No compararlo con `recall_all` de sesiones LongMemEval
ni con QA generada/juzgada. Vanta ya tiene BM25/RRF: no añadirlos como novedad.

### 9.3 Benchmarks Go: unidad de trabajo y efectos

| Runner leído | Datos/defaults/unidad | Límites y adaptación |
|---|---|---|
| [BenchmarkHnswNeurips23:35-143][bench-hnsw] | Runbooks simple/clustered YAML, mapa de random-10k, MSTuring/SpaceV 1M/10M; b.N repite **secuencia completa** de crear índice e insert/delete/search. Search llama `SearchByVector(...,0,nil)`, comprueba error, no recall | ns/op corresponde a secuencia, no a un query top-k equivalente. Missing datasets → Skip sin `-download`; con él puede descargar a disco. [helper:226-265][bench-hhelper] usa M=30, efConstruction=60, EF=36, cosine, commitlogger noop y dummy store: no mide persistencia/fsync de producto. Reusar solo como estudio de carga mutation/search, no prueba de calidad/durabilidad |
| [BenchmarkWALReplay:31-149,184-220][bench-replay] | 600.000 entradas, payload400, MapCollection y Replace; threshold 0/2/8/32/128/256/512 MiB; sin memory limit o192MiB. Copia WAL a temp y hace GC fuera del timer; mide apertura/replay. Reporta `MB-peak-heap` dividiendo por1024² | Denominador una recuperación por iteración, no objetos/s. PeakHeap toma HeapAlloc cada2ms, incluye basura no barrida y muestreo altera runtime; no RSS. Replace cuenta additions por chunk con coste que puede crecer cuadrático. Comparar recovery C16 bajo condiciones declaradas sin copiar thresholds ni reescribir WAL |

Ninguno certifica aprendizaje de memoria episódica, utilidad contextual,
seguridad de tenants o respuesta de agente. No hay tabla de superioridad
Weaviate/VantaDB validada por esta autoría.

### 9.4 Contraste con runner de memoria ya existente en Vanta

[evals/eval_locomo.py:620-641,813-859][v-locomo] mide otra capa:
recall@5/@10 sobre **session_id** gold, MRR, nDCG@5 y máximo F1 léxico
entre respuesta gold y payload recuperado. F1 aquí no es respuesta generada
por LLM ni juicio semántico de un evaluador. La latencia de search usa
perf_counter y convierte a milisegundos. Defaults: top_k10, backend memory,
sin límite, category all; sin dataset-path usa sintético seed42 incluso
sin pedir smoke. No se confunde ese default con dataset público LoCoMo real.

La construcción de VantaLocomoStore está en try/except general y, ante
excepción, continúa con **FakeLocomoStore**. Es una ruta explícita del runner,
no prueba de fallo del motor: una salida de smoke/fake no debe etiquetarse
como certificación de VantaDB ni compararse con throughput persistente de
Weaviate. Gold vacío produce recall1.0 en esas fórmulas, así que composición
del dataset y denominadores importan. El reporte JSON es un efecto de
ejecutarlo; no se generó/modificó en esta tarea.

Conclusión por capa: SIFT/Go exploran motor; BM25 explora retrieval de documentos;
este runner explora sesiones y solapamiento léxico. Ninguno sustituye calidad
contextual de agente ni autoriza nuevos prompts/scoring. Se reutilizan runners
existentes conservando su protocolo y etiquetando dataset/store/mode efectivos.

## 10. Matriz Vanta

**Pin de todas las celdas Vanta:** `97c9623c8ef5a70fdb51aff215bfbb3b05b15612`.
Cada enlace V de cada fila incorpora ese SHA, ruta y rango; no apunta al
worktree mutable. «No acreditado» se limita a la superficie leída.

| Capacidad / evidencia Weaviate `85bd085e6fd6bcf2c6269c880dc49181c86bbb92` | Vanta: estado + fuente fijada por fila | Delta y decisión |
|---|---|---|
| Batch con resultado por posición: [PutObjectBatch/objectsBatcher][batch] | **Parcial:** [V:put_batch:1280-1323,1478-1529][v-write] valida upfront y commit por chunks, devuelve Result global; [V:put_certified:1243-1277][v-write] sí tiene atestación WriteReceipt | C13 añade dimensiones independientes por ítem; no «recibos inexistentes» ni atomicidad global |
| Buffer, sync y recovery: [commitLogger][commitlog], [recoveredRun][recover] | **Existe:** [V:WalWriter::sync][v-wal], [V:verify_wal_file][v-wal-verify] y registros WAL tipados | No nuevo WAL por proyección; C16 agrega diagnóstico, C13 reporta garantía real |
| Filtro antes de vector: [buildAllowList][read], [HNSW exact/knn][hsearch] | **Existe:** [V:vector_memory_search:129-235][v-vector], selectividad, exact scan, InFilter y routing | C7 expone capacidades/errores/plan; no reinventa prefilter ni ANN |
| HNSW/flat/dynamic y config: [hnsw][hconfig], [flat][fconfig], [dynamic][dconfig] | **Existe** abstracción/alternativas: [V:VecIndex/IndexType][v-index] y [V:routing][v-vector] | No port de índices completos. No se acredita paridad Dynamic/HFresh/MUVERA |
| Compatibilidad compresión por target: [flat:194-228][fconfig], [helpers][helpers] | **Parcial:** [V:sq8_quantize][v-quant]; [V:MemorySearchRequest][v-request] valida ruta/request, no acredita matriz equivalente por colección | C15 manifest/validación; no decir que tener cuantización equivale a PQ/BQ/RQ/SQ upstream |
| Scope/tenant y recursos: [LazyLoadShard][lazy] | **Existe** scope de recall: [V:RecallScope/RecallConfig/InjectionPolicy][v-recall]; **no acreditada** descarga física por namespace en [V:ruta vector compartida][v-vector] | C9 opcional para recursos; C7 scope gobernado. Tenant físico no es auth |
| Cuotas/load admission: [loadIfCold:165-172][lazy] | **Existe:** [V:Config.max_namespace_records/max_namespace_bytes y backpressure][v-config]; [V:MemoryScheduler][v-scheduler] | No crear cuotas/scheduler nuevos; añadir leases/admisión específica solo si hay recurso concreto |
| Fuente-derivado/reindex: [MigrationRecord/Store swap][migration], [store][store] | **Existe** reemplazo, estados/rebuild: [V:ensure_indexes_current/replace_derived_indexes][v-derived], [V:rebuild con reportes][v-rebuild] | C4/C14 publica revisión/generación; no negar reconstrucción ni provenance existentes |
| Auditoría/backup: [backupShardWithHardlinks][backup], [reconcile][reconcile] | **Existe:** [V:cmd_audit_index deep/shallow][v-audit], [V:BackupManifest/backup/restore][v-backup], [V:verificación WAL][v-wal] | C16 agregado por componente con límites. No certificación automática de recuperación |
| Memoria de agente | **Existe en Vanta:** [V:L1 plan/persist/dedup][v-l1], [V:recall gobernado][v-recall] | **No acreditado equivalente Weaviate** en paths leídos; tratar Weaviate como sustrato de retrieval, no adoptar «memoria» solo por almacenar properties |
| Contexto híbrido/presupuesto | **Existe en Vanta:** [V:RecallConfig e InjectionPolicy][v-recall] y [referencia contextual](VANTA-REFERENCE.md) | No reemplazar ensamblador/context budget con parámetros ANN; caracteres/tokens/top-k son unidades distintas |

Studio tiene otro pin y se contrasta explícitamente en §12; no se introduce
su código en la referencia del motor para aparentar una sola revisión.

## 11. Negativos Vanta

### Hechos acotados sustentados

- **API batch no entrega el progreso propuesto:** `put_batch` tiene retorno
  global y su contrato documenta chunks previos confirmados ante fallo posterior
  ([1280-1323][v-write]). Es una limitación de visibilidad para importadores,
  **no** evidencia de pérdida, atomicidad engañosa no documentada o ausencia de
  recibos: `put_certified`/WriteReceipt sí existen. C13 resuelve un delta distinto.
- **El helper de vigencia derivada leído usa schema y conteos:**
  `ensure_derived_indexes_current_with` compara counts y schema version
  ([33-76][v-derived]); dos conjuntos diferentes con igual cardinalidad
  no se distinguen solo por esos predicados. Esto motiva la comprobación
  por identidad/revisión de C16. **No demuestra un bug observado:** antes de
  afirmar corrupción silenciosa debe seguirse escritura, auditoría deep y
  recuperación, y construir una discrepancia real. El producto ya ofrece
  `audit_text_index_deep`, no se niega.

### Riesgos condicionales, no defectos atribuidos

- Persistencia canónica y `replace_derived_indexes` son etapas separadas
  en batch ([1478-1529][v-write]); una falla tardía podría exigir reparación/
  comunicación específica. No se demostró que el pin deje índices incorrectos:
  `ensure_indexes_current` y rebuild existen. Prueba futura debe cortar en
  fronteras concretas y comprobar fuente/derivados tras reapertura.
- No se acreditó manifest índice/compresión/modelo por colección ni recursos
  descargables por namespace. Es límite de cobertura, no ausencia global:
  no convierte en bug el uso de un índice compartido para embebido.
- El issue upstream #4262 no se traslada como «Vanta tiene el mismo crash».
  Distintos backends, WAL y barreras impiden ese salto lógico.
- El propio [verificador WAL:1837-1841][v-wal-verify] declara que truncar
  en frontera limpia, reescribir todo consistentemente o eliminar un segmento
  no se distingue sin ancla externa. Además está bajo feature CLI/test.
  C16 debe conservar esa cobertura limitada, no prometer detectar cualquier
  pérdida ni asumir que ese símbolo crate-private es API pública de SDK.

### Falsos positivos descartados

No se sostienen «sin híbrido», «sin filtros pre-ANN», «sin cuotas/scheduler»,
«sin dedup/provenance», «sin checkpoint», «sin auditoría», «sin recibos» ni
«sin Studio». Tampoco `MemoryScope` como tipo implementado:
los nombres acreditados son **RecallScope, RecallConfig e InjectionPolicy**.
`utils/checkpoint.rs::mutate` no es el WAL ni prueba un fallo MVCC del motor.
No se añade a este informe un bug de seguridad desde una muestra de query hooks.

## 12. Escritorio / UI

### 12.1 UI real incluida y producto externo

La afirmación anterior «sin UI propia» era demasiado amplia. Hay una **demo
web implementada**, no aplicación desktop: [README de demo:1-68][demo-readme],
[app.js][demo-js] y [start.py][demo-proxy].

Flujo leído: HTML/JS → `pingServer` GET `/api/v1/meta` → `gqlRaw` POST
`/api/v1/graphql` con no-store → proxy Python hacia cluster → lectura JSON/
errores → tarjeta de latencia/resultados. `timedGql:71-75` mide en navegador
hasta parseo; no es latencia pura de servidor. Tres tarjetas muestran filtro
de rango sin índice, tokenización de paths como palabras y igualdad sin índice.
El import asociado se documenta destructivo: recrea colección y anuncia
1M objetos sintéticos seed42; se leyó esa declaración, no se midió su carga.

`changeTokenization:101-130` trata 200 como noop y 202 como tarea, hace
poll a `/v1/schema/{CLASS}/indexes` hasta ready o120s. `deleteIndex` y reset
**mutan schema**, por lo que no se deben presentar como controles de diagnóstico
inocuos. `gqlRaw` distingue HTTP errors de GraphQL errors incluso con HTTP200.
Son lecciones útiles para C7/C13/C14: vacío, failed y pending son estados
distintos, y aceptar una tarea no publica todavía el índice.

El proxy inyecta Bearer desde env, preserva errores upstream y solo pasa
Content-Type/Accept de entrada (`start.py:65-124`). **Discrepancia de frontera
observada:** comentarios dicen loopback/local-only, pero `main:145` construye
`ThreadedServer(("", PORT), Handler)`, dirección comodín. No se comprobó bind
efectivo/firewall ni explotación; no copiar esa configuración ni describirla
como aislada por garantía del comentario. No se arrancó/paró el proxy.

La [documentación oficial Explorer][explorer], leída el2026-11-07 parcialmente,
acredita UI **externa de Weaviate Cloud**, acceso por cluster→Explorer→colección,
inspección de propiedades/metadata/vectores y modos Browse/Hybrid/Semantic/
Keyword/Aggregate. Indica que Query tool se retiró el2026-09-14. Es
documentación viva, no implementación pinneada de consola ni prueba de que una
instalación OSS tenga esa UI. La parte leída presenta alpha0.5 en Explorer,
distinto del alpha0 BM25 del runner. No se estudió un shell desktop nativo;
ausencia en esta muestra no es ausencia de todo cliente externo del ecosistema.

### 12.2 Seams Studio existentes, no otro frontend

Worktree `C:\Users\Eros\VantaDB Proyect\develop-desktop`,
SHA **`0ad38c542b39a589035b0e17fdedb4c09dca24b9`**.
`desktop/package.json` y `src-tauri/Cargo.toml` acreditan React19/Vite7 y
Tauri2; crate embebido con subset de features y vanta-memory sin defaults,
no una obligación de proveedor LLM. Lectura estática, no UI ejecutada.

| Seam real del pin Studio | Flujo existente | Extensión propuesta, no implementada |
|---|---|---|
| [vanta.ts:566-579][studio-api], `metrics/namespaceStats` | transport.call `vanta_metrics` / `vanta_namespace_stats` | C16 muestra diagnóstico por componente/generación; no confundir count con integridad |
| [commands/data.rs:147-159][studio-command], `vanta_namespace_stats` | state.manager.namespace_stats | Añadir consulta read-only tipada solo tras contrato SDK, con Unsupported por transporte |
| [connections/native.rs:657-672][studio-native] | blocking sobre db.namespace_stats y mapeo DTO/error | Integrar diagnóstico existente del motor sin bloquear hilo UI ni abrir base paralela |
| `connections/manager.rs::namespace_stats:266-276`, `trait.rs:226-234`; `server.rs:294-300` | Selección de conexión nativa/server y soporte explícito | Mantener paridad declarada; no mostrar Healthy si transporte no inspecciona WAL |
| `src/components/home/HomeOverview.tsx`, paneles TTL/distribución | Consume estadísticas; metadata/expiración son observaciones distintas | Añadir pendientes/rebuild/cold solo si backend los ofrece; no simularlos con temporizadores UI |

No se localizó en estos seams un comando que ya exponga la nueva matriz
de generaciones/recuperación. Por ello C16 requiere DTO/bridge adicional
sobre producto existente, no «un seam listo» ni duplicar Studio. Mutaciones
repair/restore se separan de lectura, requieren confirmación y muestran límites
de cancelación. Observabilidad por defecto sin payloads, vectores ni claves.

## 13. Fuentes, cobertura y cierre de autoria

### 13.1 Inventario de lectura y lagunas

| Cobertura efectivamente leída | Límite explícito |
|---|---|
| LICENSE/LICENSE-BSD, go.mod; modelos Object/Class y handlers gRPC de entrada | No inventario de licencias transitivas, código empresarial `wl/`, todos los providers o modelos |
| shard batch, put/status, queue, vector+filter search, delete; LSM memtable/WriteWAL/commitlog y recovery por segmentos | No todo flush/compaction/replication/handler end-to-end. Trazas completas dentro de frontera shard/storage, no prueba de todos los transports |
| LazyLoadShard::loadIfCold, loadlimiter; backupShardWithHardlinks | No auditoría exhaustiva de cold/offloaded cloud, ni ahorro de memoria medido |
| Configs HNSW/flat/dynamic; MigrationRecord, store atómico, reconcile y bucket swap | No validación completa de HFresh/MUVERA ni de todas las estrategias de reindex. Docs runtime-reindex contienen historia advertida; se priorizó código vigente |
| Issues #3948/#3949 bodies; #4262/#9274 bodies+comentarios; PR9240 metadata y partes de diff; Search API adicional | No revisión de todos los issues/advisories, adjuntos ni reproducción; extras no elevados a hallazgo vigente |
| SIFT main/runner, BM25 import/query/defaults/queries/scores/latencias, Go HNSW y WALReplay/helper | No ejecución, datasets descargados, scores recalculados ni runbooks nuevos |
| Demo JS/proxy/README, primeros6000 caracteres doc Explorer; seams Studio pin0ad38c542 | No código de consola SaaS, browser interactivo ni shell desktop de terceros |
| Vanta97c9623: APIs batch/recibo, búsqueda/filtros, WAL, índices/rebuild/backup; referencia de memoria y scheduler | No certificación multitenant, equivalencia de todos los bindings ni crash testing |

Se cargaron documentation-skill y coordinated-web-search. Se revisó catálogo
de skills y codebase-memory antes de consultar CodeGraph read-only para seams
Vanta; no se modificó ni regeneró ningún índice, estado de mantenimiento o
grafo. El grafo es navegación; los enlaces citan el pin y rangos realmente
leídos. Las únicas escrituras autorizadas son este informe y el recibo temporal.

### 13.2 Continuidad y división futura

Este informe supera300 líneas por mantener los diez ejes autónomos. **No se
crean specs en esta tarea.** Si se aprueba implementación, separar:

1. Spec C13: API de resultados por ítem y semántica por backend; compatibilidad
   con WriteReceipt y chunk commits.
2. Spec C7/C15: capabilities, manifest/dominio y matriz efectiva de soporte.
3. Spec C4/C14: revisión fuente, staging, publicación, catch-up y rollback;
   comenzar offline si online no tiene evidencia suficiente.
4. Spec C16: reporte de integridad, snapshot/backup y bridge Studio read-only.
5. Spec C9 opcional: recursos separables, admisión y leases, solo con evidencia
   de necesidad; no forzar partición física como dependencia de las anteriores.

Dependencias: C7↔C15 comparten compatibilidad; C4/C14 provee generación a
C13/C16; C9 aporta leases cuando haya unload, no es prerrequisito universal.
Pendiente manual: revisión independiente de fuentes/contratos, decisiones de
ownership/revisión, paridad transportes y posterior autorización expresa para
cualquier ejecución. No se ofrece una batería de comandos destructivos como
«paso siguiente» ni se cambia el protocolo de benchmarks común.

**Cierre de autoría:** se sustituyen las inferencias rechazadas por tres flujos
locales trazados, seis fichas de adaptación, cuatro incidencias desarrolladas,
runners específicos y UI/seams acreditados. Es **entrega condicionada a revisión**,
no autoaceptación, aprobación de features o validación dinámica. No apps/tests/
builds/benchmarks, instalaciones, servicios, commits ni push. Metadata,
contratos, índices, JSON, backlog y otros informes permanecen fuera de alcance.

### 13.3 Enlaces inmutables y públicos

Los identificadores de fuente usados arriba fijan **SHA completo + archivo +
rango**. Cuando se cita otro subrango en el texto, es del mismo archivo/pin.
URLs de issues/API/documentación son evidencia pública viva con fecha de
lectura en §5; no se presentan como blobs inmutables.

[license]: https://github.com/weaviate/weaviate/blob/85bd085e6fd6bcf2c6269c880dc49181c86bbb92/LICENSE#L1-L17
[bsd]: https://github.com/weaviate/weaviate/blob/85bd085e6fd6bcf2c6269c880dc49181c86bbb92/LICENSE-BSD#L1-L27
[gomod]: https://github.com/weaviate/weaviate/blob/85bd085e6fd6bcf2c6269c880dc49181c86bbb92/go.mod#L1-L283
[object]: https://github.com/weaviate/weaviate/blob/85bd085e6fd6bcf2c6269c880dc49181c86bbb92/entities/models/object.go#L31-L62
[batch]: https://github.com/weaviate/weaviate/blob/85bd085e6fd6bcf2c6269c880dc49181c86bbb92/adapters/repos/db/shard_write_batch_objects.go#L33-L537
[put]: https://github.com/weaviate/weaviate/blob/85bd085e6fd6bcf2c6269c880dc49181c86bbb92/adapters/repos/db/shard_write_put.go#L33-L551
[queue]: https://github.com/weaviate/weaviate/blob/85bd085e6fd6bcf2c6269c880dc49181c86bbb92/adapters/repos/db/vector_index_queue.go#L147-L235
[read]: https://github.com/weaviate/weaviate/blob/85bd085e6fd6bcf2c6269c880dc49181c86bbb92/adapters/repos/db/shard_read.go#L711-L1045
[hsearch]: https://github.com/weaviate/weaviate/blob/85bd085e6fd6bcf2c6269c880dc49181c86bbb92/adapters/repos/db/vector/hnsw/search.go#L81-L145
[delete]: https://github.com/weaviate/weaviate/blob/85bd085e6fd6bcf2c6269c880dc49181c86bbb92/adapters/repos/db/shard_write_delete.go#L33-L223
[memtable]: https://github.com/weaviate/weaviate/blob/85bd085e6fd6bcf2c6269c880dc49181c86bbb92/adapters/repos/db/lsmkv/memtable.go#L304-L777
[bucket]: https://github.com/weaviate/weaviate/blob/85bd085e6fd6bcf2c6269c880dc49181c86bbb92/adapters/repos/db/lsmkv/bucket.go#L2529-L2540
[commitlog]: https://github.com/weaviate/weaviate/blob/85bd085e6fd6bcf2c6269c880dc49181c86bbb92/adapters/repos/db/lsmkv/commitlogger.go#L401-L452
[lazy]: https://github.com/weaviate/weaviate/blob/85bd085e6fd6bcf2c6269c880dc49181c86bbb92/adapters/repos/db/shard_lazyloader.go#L59-L245
[hconfig]: https://github.com/weaviate/weaviate/blob/85bd085e6fd6bcf2c6269c880dc49181c86bbb92/entities/vectorindex/hnsw/config.go#L24-L128
[fconfig]: https://github.com/weaviate/weaviate/blob/85bd085e6fd6bcf2c6269c880dc49181c86bbb92/entities/vectorindex/flat/config.go#L22-L230
[dconfig]: https://github.com/weaviate/weaviate/blob/85bd085e6fd6bcf2c6269c880dc49181c86bbb92/entities/vectorindex/dynamic/config.go#L24-L115
[helpers]: https://github.com/weaviate/weaviate/blob/85bd085e6fd6bcf2c6269c880dc49181c86bbb92/adapters/repos/db/helpers/helpers.go#L25-L43
[migration]: https://github.com/weaviate/weaviate/blob/85bd085e6fd6bcf2c6269c880dc49181c86bbb92/adapters/repos/db/inverted_reindex_record.go#L28-L125
[migration-store]: https://github.com/weaviate/weaviate/blob/85bd085e6fd6bcf2c6269c880dc49181c86bbb92/adapters/repos/db/inverted_reindex_record_store.go#L542-L761
[reconcile]: https://github.com/weaviate/weaviate/blob/85bd085e6fd6bcf2c6269c880dc49181c86bbb92/adapters/repos/db/inverted_reindex_reconcile.go#L155-L245
[store]: https://github.com/weaviate/weaviate/blob/85bd085e6fd6bcf2c6269c880dc49181c86bbb92/adapters/repos/db/lsmkv/store.go#L813-L925
[recover]: https://github.com/weaviate/weaviate/blob/85bd085e6fd6bcf2c6269c880dc49181c86bbb92/adapters/repos/db/lsmkv/bucket_recover_from_wal.go#L255-L409
[backup]: https://github.com/weaviate/weaviate/blob/85bd085e6fd6bcf2c6269c880dc49181c86bbb92/adapters/repos/db/backup.go#L416-L491
[bench-main]: https://github.com/weaviate/weaviate/blob/85bd085e6fd6bcf2c6269c880dc49181c86bbb92/test/benchmark/benchmark.go#L39-L261
[bench-sift]: https://github.com/weaviate/weaviate/blob/85bd085e6fd6bcf2c6269c880dc49181c86bbb92/test/benchmark/benchmark_sift.go#L33-L245
[bm-root]: https://github.com/weaviate/weaviate/blob/85bd085e6fd6bcf2c6269c880dc49181c86bbb92/test/benchmark_bm25/cmd/root.go#L37-L72
[bm-import]: https://github.com/weaviate/weaviate/blob/85bd085e6fd6bcf2c6269c880dc49181c86bbb92/test/benchmark_bm25/cmd/import.go#L87-L175
[bm-query]: https://github.com/weaviate/weaviate/blob/85bd085e6fd6bcf2c6269c880dc49181c86bbb92/test/benchmark_bm25/cmd/query.go#L117-L232
[bm-data]: https://github.com/weaviate/weaviate/blob/85bd085e6fd6bcf2c6269c880dc49181c86bbb92/test/benchmark_bm25/lib/dataset_queries.go#L23-L91
[bm-scores]: https://github.com/weaviate/weaviate/blob/85bd085e6fd6bcf2c6269c880dc49181c86bbb92/test/benchmark_bm25/lib/scores.go#L20-L70
[bm-times]: https://github.com/weaviate/weaviate/blob/85bd085e6fd6bcf2c6269c880dc49181c86bbb92/test/benchmark_bm25/lib/durations.go#L20-L57
[bench-hnsw]: https://github.com/weaviate/weaviate/blob/85bd085e6fd6bcf2c6269c880dc49181c86bbb92/adapters/repos/db/vector/hnsw/benchmark_test.go#L35-L185
[bench-hhelper]: https://github.com/weaviate/weaviate/blob/85bd085e6fd6bcf2c6269c880dc49181c86bbb92/adapters/repos/db/vector/hnsw/index_test.go#L226-L265
[bench-replay]: https://github.com/weaviate/weaviate/blob/85bd085e6fd6bcf2c6269c880dc49181c86bbb92/adapters/repos/db/lsmkv/bucket_recover_chunked_benchmark_test.go#L31-L220
[demo-readme]: https://github.com/weaviate/weaviate/blob/85bd085e6fd6bcf2c6269c880dc49181c86bbb92/tools/dev/bench/demo_indexing_mistakes_ui/README.md#L1-L68
[demo-js]: https://github.com/weaviate/weaviate/blob/85bd085e6fd6bcf2c6269c880dc49181c86bbb92/tools/dev/bench/demo_indexing_mistakes_ui/app.js#L1-L130
[demo-proxy]: https://github.com/weaviate/weaviate/blob/85bd085e6fd6bcf2c6269c880dc49181c86bbb92/tools/dev/bench/demo_indexing_mistakes_ui/start.py#L27-L151
[explorer]: https://docs.weaviate.io/cloud/tools/explorer-tool
[v-write]: https://github.com/ness-e/Vantadb/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/api/memory.rs#L1223-L1529
[v-config]: https://github.com/ness-e/Vantadb/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/config.rs#L750-L782
[v-wal]: https://github.com/ness-e/Vantadb/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/wal.rs#L1024-L1028
[v-wal-verify]: https://github.com/ness-e/Vantadb/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/wal.rs#L1837-L1874
[v-vector]: https://github.com/ness-e/Vantadb/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/search/vector.rs#L67-L235
[v-request]: https://github.com/ness-e/Vantadb/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/serialization/vector_types.rs#L102-L172
[v-index]: https://github.com/ness-e/Vantadb/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/index/mod.rs#L23-L61
[v-quant]: https://github.com/ness-e/Vantadb/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/vector/quantization.rs#L140-L173
[v-derived]: https://github.com/ness-e/Vantadb/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/serialization/impl_index.rs#L16-L242
[v-rebuild]: https://github.com/ness-e/Vantadb/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/serialization/impl_rebuild.rs#L16-L120
[v-audit]: https://github.com/ness-e/Vantadb/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/cli_handlers/index.rs#L118-L156
[v-backup]: https://github.com/ness-e/Vantadb/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/cli_handlers/backup.rs#L25-L131
[v-recall]: https://github.com/ness-e/Vantadb/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/hooks/auto_recall.rs#L133-L423
[v-l1]: https://github.com/ness-e/Vantadb/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/record/l1_writer.rs#L289-L418
[v-scheduler]: https://github.com/ness-e/Vantadb/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/services/scheduler.rs#L79-L271
[v-locomo]: https://github.com/ness-e/Vantadb/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/evals/eval_locomo.py#L620-L859
[studio-api]: https://github.com/ness-e/Vantadb/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/desktop/src/vanta.ts#L566-L579
[studio-command]: https://github.com/ness-e/Vantadb/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/desktop/src-tauri/src/commands/data.rs#L147-L159
[studio-native]: https://github.com/ness-e/Vantadb/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/desktop/src-tauri/src/connections/native.rs#L657-L672

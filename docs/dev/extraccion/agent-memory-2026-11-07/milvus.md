---
title: "Milvus: ciclo de segmentos, consistencia y preparación de ingesta"
kind: research
status: proposed
description: "Contraste estático de readiness e índices de Milvus con el motor embebido existente de VantaDB"
tags: [research, memory, engineering]
---

# Milvus: ciclo de segmentos, consistencia y preparación de ingesta

## Índice

- [Identidad, arquitectura y tres flujos](#1-recomendación-y-arquitectura)
- [Mecanismos y fichas Rust propuestas](#2-lectura-de-código-y-fichas-candidatas)
- [Incidencias y seguridad](#3-riesgos-issues-y-seguridad)
- [Prerrequisitos](#4-prerrequisitos-vantadb) · [Rechazos](#5-qué-no-trasladar)
- [Método/licencia](#6-método-licencia-y-límites) · [Benchmarks](#7-benchmarks-upstream)
- [Matriz](#8-matriz-de-brechas) · [Negativos propios](#9-debilidades-sugeridas)
- [UI real](#10-ui-e-inspección) · [Fuentes y cierre](#11-fuentes-cobertura-y-cierre-de-autoría)

## 1. Recomendación y arquitectura

**Dictamen:** extraer vocabulario verificable de readiness y publicación,
no reproducir coordinadores y nodos distribuidos.
Local: `C:\Users\Eros\milvus`; HEAD con fuentes verificado:
`eba8be50f84776210d3946d83840dea18924846f`.
[Snapshot](github-candidates.json): **46 612 estrellas, Apache-2.0**.
Clon y omisiones: [manifiesto](clone-manifest.json), captura 2026-11-07.
Referencia VantaDB: **`97c9623c8ef5a70fdb51aff215bfbb3b05b15612`**;
las referencias anteriores a c561158/dirty se conservan como antecedentes,
no como defectos actuales. No se recalculó metadata ni actualizó ningún clon.

```text
cliente → Proxy (Go, schema/IDs/timestamps)
  escritura → StreamingClient → StreamingNode/WAL
    RecoveryStorage → checkpoints/metadatos + binlogs/estadísticas
    DataCoord / index jobs → artefactos de índice
  consulta → QueryCoord / QueryNode delegator
    espera garantía temporal → fija segmentos legibles
    segmentos growing/sealed → segcore C++ / índices Knowhere
metadata/catalog: etcd; datos/índices: almacenamiento local u objetos
componentes adicionales: Rust/Tantivy; backend WAL según despliegue
```

La guía oficial del checkout distingue **persistencia de segmentos** de
**construcción de índices** y de consumo para consulta. «Insert terminó»,
«segmento sealed», «índice cargado» y «read-your-writes» no son sinónimos.
En esta revisión no se presupone que cada consulta espere lo mismo:
el camino de resultado parcial y las tablas externas tienen excepciones M2.

### 1.1 Stack, modelo y superficies operativas

| Capa | Implementación observada | Contrato y frontera |
|---|---|---|
| Cliente/API | SDK externo pymilvus, gRPC/protobuf; HTTP Gin en `internal/distributed/proxy/httpserver` | Insert/upsert/delete, Search ANN, Query por predicado/PK, DDL de schema/index/load |
| Proxy Go | `internal/proxy/{dml,dql,ddl}`, `impl.go` | Valida esquema, IDs, tipos y routing; encola tareas y espera estado, no indexa por sí solo |
| Streaming | `internal/streamingnode`, `internal/distributed/streaming` | VChannels lógicos sobre PChannels; append/TimeTick y recovery según backend |
| Coordinación | DataCoord, QueryCoord y catálogo compartido | Catálogo, estado de segmentos/index jobs, distribución y carga |
| QueryNode | `internal/querynodev2/{delegator,segments}` | tSafe, MVCC de consulta, pin de segmentos, ejecución native y reducción |
| C++/índices | `internal/core`, segcore/Knowhere | Índices y operaciones nativas; ownership Go/C++ y recursos de carga no son transacciones del cliente |
| Rust auxiliar | `internal/core/thirdparty/tantivy/tantivy-binding/Cargo.toml` | Wrapper de búsqueda textual/tokenización; no convertir Milvus en «motor Rust» por esta dependencia |
| Persistencia | WAL backend, catálogo etcd, objetos/binlogs/índices | RecoveryStorage materializa datos; construir índice y cargarlo son etapas posteriores |
| Operación/UI | WebUI embebida, métricas; Attu externo | Plano administrativo distinto de CRUD de memoria; detalle en §10 |

`go.mod:1-30` fija Go y dependencias gRPC/Gin, Casbin, Prometheus, MinIO y
otras; no se reconstruyó la closure de dependencias. README:22-43 diferencia
Standalone, distribuido y **Milvus Lite como componente aparte**. El camino
streaming de este checkout no certifica Lite, cada versión estable ni Zilliz
Cloud. Standalone tampoco elimina WAL, catálogo y fases de preparación.

El modelo leído usa database/collection ID, schema con versión, field IDs,
PK, vector con dimensión/métrica, partición y VChannel. `InsertTask` conserva
schema/version y resultado de mutación; `SegmentLoadInfo` incluye campos,
binlogs e índices. `IndexedFieldInfo::IsLoaded` es estado por campo, no prueba
que toda la colección sea consultable. Growing/sealed, nivel de segmento,
versión de carga, timestamp lógico y posición física del WAL no son un ID único
intercambiable. Search devuelve candidatos/scores; Query puede devolver filas
o `count(*)`. Ninguno es por sí mismo memoria conversacional ni checkpoint de
ejecución de agente.

Embedding no es obligatorio para vectores suministrados por el cliente.
README:156 documenta `pymilvus[model]`/rerank y ecosistema de proveedores como
opciones; no se leyó su implementación externa ni se asume un modelo default.
Funciones como BM25/MinHash producen campos derivados del schema; #54126
demuestra que «indexable» y «retornable al cliente» son contratos distintos.
No se ejecutaron modelos ni llamadas pagadas.

### 1.2 Flujo de ingesta: aceptación no equivale a índice listo

1. Proxy crea tarea de inserción. En M9
   `InsertTask::Execute:35-120` obtiene VChannels, empaqueta registros con
   versión de schema y decoración de idempotencia, y llama
   `streaming.WAL().AppendMessagesWithOptions(ctx, msgs, IdempotencyKey)`.
   `resp.UnwrapFirstError()` puede proyectarse a `result.Status` aunque la
   función devuelva `nil`: revisar estado de respuesta, no solo error Go.
2. En éxito, `result.Timestamp = resp.MaxTimeTick()` alimenta consistencia de
   sesión. No es evidencia de que todo campo esté indexado/cargado.
   El `ctx` cruza append; cancelación no acredita ausencia de commit.
   La guía M15 documenta recuperación antes de reanudar un writer cancelado.
3. WAL → RecoveryStorage materializa binlogs/estadísticas y estado recuperable.
   La guía M15 exige snapshots de componentes antes de publicar checkpoint y
   truncar log; la construcción de índices queda **fuera** de RecoveryStorage.
   Este es hecho documental, no auditoría íntegra de replay.
4. **Seal/flush:** M10 `Server::Flush:91-129` y
   `flushCollection:143-223` capturan checkpoints antes de seal. En streaming,
   `broadcastManualFlush:132-141` usa ACK SyncUp y espera completitud L1/L0
   en VChannels; el camino alternativo sella segmentos y devuelve IDs/flushTs.
   Por tanto, tampoco es correcto decir que *todo* ACK de Milvus es solo enqueue.
   El alcance del ACK se limita a la operación: flush no construye ni carga
   automáticamente todos los índices de consulta.
5. **Index:** M11 `CreateIndex:121-352` valida campo/parámetros y publica DDL
   por broadcast; `GetIndexState:495-541` agrega segmentos elegibles y conserva
   `FailReason`. Un ACK de creación no sustituye estado final por segmento/campo.
6. **Load:** M12 `LoadCollectionTask::Execute:3178-3267` valida campos
   solicitados e índices vectoriales y llama QueryCoord; este
   `LoadCollection:276-320` difunde configuración de carga. M4
   `segmentLoader::Load:247-296,390-430` reserva recursos con liberación defer,
   carga segmentos y luego hace `Segment.Put`. El contrato `LoadSegment`
   explícitamente no los publica. Un estado «loaded» local no prueba por sí
   solo distribución servible en cada réplica.

Errores lógicos de schema/índice son diferentes de agotamiento de recursos,
timeout o append de resultado desconocido. No hay transacción monolítica que
abarque SDK, objetos, job de indexación, carga y publicación en QueryNode.

### 1.3 Flujo de consulta y consistencia

Proxy/DQL recibe Search/Query, prepara plan/partición/output, ejecuta fanout,
reduce y organiza resultados (`internal/proxy/dql/README.md:1-55`, documento).
En M1 `parseGuaranteeTsFromConsistency:886-899`, Strong usa tMax, Bounded
resta GracefulTime y Eventually usa 1; Session conserva timestamp suministrado
en esa función. No se infiere el default de cada SDK desde este switch.

M2 `shardDelegator::Search:589-648` valida canal y lifetime, calcula garantía,
espera `waitTSafe` si no aplica excepción y fija `MvccTimestamp` cuando
corresponde. Pinnea segmentos readable y hace `defer Unpin(version)`.
Growing puede ser buscable antes de flush; sealed no significa cargado.
M13 `waitTSafe:1291-1365` tiene fast path, modo downgrade configurable,
rechazo por lag, espera de condición con contexto/stall timeout y error si
delegator cierra. Esto es control explícito de frescura/disponibilidad, no
garantía de que toda solicitud Strong siempre tenga éxito.

La excepción partial search puede saltarse la espera y usar tSafe actual;
tablas externas usan snapshot materializado y timestamp máximo en este pin.
No tratar estos caminos como lectura fuerte estándar. Cancelar libera la
espera Go; no se auditó toda propagación de cancelación al trabajo C++ ni
todos los reducers. El pin protege distribución durante la operación, no
otorga a un cliente permisos sobre segmentos de otro scope.

### 1.4 Flujo update/delete y replay

M14 `UpsertTask::Execute:32-75` deriva a append normal o update parcial con
CAS. `appendUpsertAttempt:120-173` empaqueta insert+delete, añade CAS si
corresponde y publica; devuelve timestamp de sesión en éxito.
La política `canRetryPartialUpdateCASConflict:458-467` solo permite reintento
automático para operaciones REPLACE. Su comentario advierte que una operación
relativa puede reaplicarse en canales ya comprometidos si se reintenta toda
la solicitud tras conflicto en otro. No se acredita atomicidad entre todos
los VChannels por ver insert+delete en el mismo slice Go.

M14b `DeleteTask::Execute:21-90` rechaza expresión vacía, construye mensajes
por canal/PK, hace append y guarda sessionTS. Los deletes complejos resuelven
primero PKs mediante QueryStream/QueryView
(`task_delete.go:556-631,688-777`); resolución y append no son un borrado
serializable por predicado demostrado. El timestamp rige visibilidad lógica;
los deletes deben materializarse/reaplicarse también al cargar sealed.
La guía M15 separa consumo completo, persistencia y truncación del WAL.
Eliminar de resultados, compactar y borrar objetos/backups son garantías
distintas. No se leyó toda la cadena de GC físico.

### 1.5 Prioridad para VantaDB

P0 documental: **A/B**, fases y espera cancelable después de escritura,
aprovechando el WAL y retrieval existentes. P1 condicionado al inventario:
**C/D/E**, publicación con ownership, checkpoint de consumo y retry seguro.
P2: **F/G**, motivos de mantenimiento/proyección explícitos e inspección en
Studio. Estas son propuestas, no tareas aprobadas ni prueba de brecha actual.
No priorizar una nueva infraestructura distribuida para un problema de contrato local.

## 2. Lectura de código y fichas candidatas

| Ref. | Fuente exacta al commit local | Hecho estático |
|---|---|---|
| M1 | [`internal/proxy/util.go:886-907`](https://github.com/milvus-io/milvus/blob/eba8be50f84776210d3946d83840dea18924846f/internal/proxy/util.go#L886-L907) | Traduce Strong/Bounded/Eventually a garantía temporal; Bounded usa graceful time |
| M2 | [`internal/querynodev2/delegator/delegator.go:603-646`](https://github.com/milvus-io/milvus/blob/eba8be50f84776210d3946d83840dea18924846f/internal/querynodev2/delegator/delegator.go#L603-L646) | Espera tSafe o permite parcial; fija MVCC y pin de segmentos |
| M3 | [`internal/querynodev2/segments/segment.go:65-115`](https://github.com/milvus-io/milvus/blob/eba8be50f84776210d3946d83840dea18924846f/internal/querynodev2/segments/segment.go#L65-L115) | Tipo growing/sealed, versión y estado IsLoaded por campo indexado |
| M4 | [`internal/querynodev2/segments/segment_loader.go:81-107`](https://github.com/milvus-io/milvus/blob/eba8be50f84776210d3946d83840dea18924846f/internal/querynodev2/segments/segment_loader.go#L81-L107) | Reserva/liberación de recursos; cargar datos e índices no publica en SegmentManager |
| M5 | [`internal/datacoord/meta/segment_info.go:38-95`](https://github.com/milvus-io/milvus/blob/eba8be50f84776210d3946d83840dea18924846f/internal/datacoord/meta/segment_info.go#L38-L95) | Separa estado privado con lock del almacén compartido; EnsureStats no muta lectores |
| M6 | [`internal/streamingnode/server/wal/interceptors/shard/policy/seal_policy.go:30-95`](https://github.com/milvus-io/milvus/blob/eba8be50f84776210d3946d83840dea18924846f/internal/streamingnode/server/wal/interceptors/shard/policy/seal_policy.go#L30-L95) | Motivos tipados de seal: fence, capacidad, lifetime, idle, memoria |
| M7 | [`internal/proxy/dml/task_insert.go:30-51`](https://github.com/milvus-io/milvus/blob/eba8be50f84776210d3946d83840dea18924846f/internal/proxy/dml/task_insert.go#L30-L51) | Task mantiene schema/version/timestamp e inputs de idempotencia |
| M8 | [`tests/python_client/bulk_insert/test_bulk_insert_perf.py:21-164`](https://github.com/milvus-io/milvus/blob/eba8be50f84776210d3946d83840dea18924846f/tests/python_client/bulk_insert/test_bulk_insert_perf.py#L21-L164) | Runner pytest bulk, generador y upload MinIO, medias/success rate y monitores |

Documentación contrastada: guías
[`streaming-system.md:7-43`](https://github.com/milvus-io/milvus/blob/eba8be50f84776210d3946d83840dea18924846f/docs/agent_guides/streaming-system/streaming-system.md#L7-L43),
[`recovery-storage.md:1-45`](https://github.com/milvus-io/milvus/blob/eba8be50f84776210d3946d83840dea18924846f/docs/agent_guides/streaming-system/wal/recovery-storage.md#L1-L45)
y `wal/shard-management.md:1-33`; no contradicción observada en estos rangos.
RecoveryStorage se cita como **documentación**, no como verificación completa
del motor de replay. No se leyó toda la arquitectura WAL enlazada.

Las fuentes M fijan `eba8be50f84776210d3946d83840dea18924846f`.
Las referencias V1–V7 del §11 fijan el snapshot Vanta. **Todos los tipos y
firmas siguientes son propuestas**, no descripción de APIs ya implementadas.
La primera acción de cada candidato es comprobar su equivalente existente.

### Ficha candidata A — estado compuesto de preparación de ingesta

- **Problema agente:** tras importar recuerdos, un agente interpreta el ACK
  como permiso para consultar todas las ramas y recibe falsos negativos.
- **Comportamiento mínimo/fuente:** M3 estado por campo, M4 carga frente a
  publicación; M9 ACK de append, M10 flush, M11 index y M12 load son fronteras
  diferentes. No transformar sus estados distribuidos en un booleano local.
- **Delta/seam:** V2 particiones e `InternalMetadata`, V4 hybrid y diagnósticos
  SDK existentes. Vanta ya tiene WAL/SyncMode; añadir cobertura explicada por
  rama solo si no la expresa su health actual.
- **Alternativas:** documentar ACK (coste mínimo, no consultable);
  extender recibo/diagnóstico (preferida); ledger paralelo o importar Milvus
  (rechazados por doble verdad y despliegue).
- **Interfaz Rust propuesta:** `ingest_readiness(namespace, mutation_id,
  source_revision) -> Result<IngestReadiness, ReadinessError>`; output
  `durable_position`, `dense_generation`, `text_generation`,
  `sparse_generation` y capacidades, cada uno observed/pending/unknown.
  Invariantes: cada cobertura pertenece a la misma revisión/scope; append
  sin fsync no se etiqueta durable; índice opcional no se etiqueta fallido.
- **Pasos:** inventariar recibos/health; seguir actualizaciones por backend;
  definir generación efectiva; adjuntar reporte al SDK/diagnóstico, no nueva
  ruta de ingesta. Consultar estado no ejecuta rebuild implícito.
- **Persistencia/migración/rollback:** derivar estado de hechos existentes;
  si requiere marcador nuevo, versionarlo en metadata del mismo commit
  cubierto. Estado legacy desconocido hasta auditar; rollback oculta campo
  diagnóstico, no cambia WAL ni borra fuente. Rebuild solo de derivados.
- **Dependencias:** identidad de mutación/revisión, C publicación, D checkpoint;
  no depende de LLM ni de un scheduler nuevo.
- **FMEA:** marcador se adelanta al índice → falso ready; detectar revisiones
  dispares, publicar después del artefacto. Rebuild cambia generación →
  cobertura vieja; invalidarla explícitamente. Global por tenant incorrecto →
  exposición lateral; scope por namespace autorizado. Cancelación antes del
  commit no avanza estado, después exige resolver outcome.
- **Aceptación futura:** favorable: escritura y consulta observan todas las
  ramas prometidas; adversos: disco lleno, índice ausente, cancelación y
  reapertura producen estados honestos. Métricas por fase sin payload ni
  IDs de recuerdos. No resultado de pruebas en esta entrega.
- **No-goals:** réplica, coordinador, persistencia «fuerte» para in-memory o
  cambio de ranking/budgets.

### Ficha candidata B — barrera local cancelable de lectura después de escritura

- **Problema agente:** ejecutar una búsqueda dependiente de una corrección sin
  dormir arbitrariamente ni aceptar silenciosamente una versión anterior.
- **Fuente/comportamiento:** M1 convierte consistencia en garantía; M2 fija
  snapshot legible; M13 espera tSafe con contexto, lag/stall y errores. Se
  extrae la semántica de espera, no el timestamp distribuido.
- **Delta/seam:** V1 append/sync, V3 vector y V4 hybrid. El recibo de write y
  la publicación deben compartir referencia; posición WAL no equivale a
  revisión MVCC ni a coverage de cada índice.
- **Alternativas:** lectura del primario (no prueba ANN); polling cliente
  (ineficiente/ambiguo); espera por watermark propio (condicionada al
  inventario); importar TSO/Bounded (rechazado para mínimo embebido).
- **Interfaz propuesta:** `await_visible(ReadAfter { mutation_id, namespace,
  required_surfaces, source_revision }, cancel) -> Result<VisibilityOutcome,
  VisibilityError>`; outcomes `Observed`, `TimedOutUnknown`, `Unsupported`.
  `Observed` debe acreditar las superficies solicitadas; error I/O no es
  pendiente ni lectura vacía. Ningún lock de publicación se retiene esperando.
- **Pasos:** inventariar await/read-your-writes actual; mapear recibo y
  superficies; integrar notificación en progreso existente; propagar estado
  explícito a SDK y reporte de consulta.
- **Persistencia/migración/rollback:** waiter efímero; token solo se acepta
  tras reopen si su identidad/revisión es recuperable. Tokens viejos no
  soportados se rechazan; no convertir clocks de pared en orden durable.
  Desactivar barrera vuelve a semántica declarada, no a éxito ficticio.
- **Dependencias:** A expone hechos, C publica, E controla retries de outcome
  desconocido. No usa el checkpoint de tarea V6 como watermark de índice.
- **FMEA:** esperar solo WAL → falso read-your-writes; detectar cobertura
  A. Cancelación tras aplicación → cliente reescribe; conservar token/outcome.
  Publisher bloqueado por waiter → deadlock; soltar locks y espera cancelable.
  Reinicio/regresión → espera imposible; error de instancia/generación.
- **Aceptación futura:** favorable: observar mutación sin sleep; adversos:
  deadline/cancelación dejan de esperar y liberan recursos, sin rollback
  implícito; unsupported no tiene éxito. Contadores wait/lag/outcome sin texto.
- **No-goals:** consistencia global, TSO, garantías de todas las rutas de
  memoria solo porque el motor dispone de hybrid.

### Ficha candidata C — preparación, publicación y ownership de generación

- **Problema agente:** índices cargados pero no visibles, snapshots mixtos
  durante rebuild o archivos retirados mientras otra consulta los usa.
- **Fuente/comportamiento:** M4 contrato `LoadSegment` y reserva/liberación;
  `Load:247-296,390-430` publica en manager tras carga; M2 pin/unpin;
  M5 distingue metadata compartida sincronizada de estado privado bajo lock.
- **Delta/seam:** publicador real localizado en el pin Vanta:
  `src/sdk/api/admin.rs::Embedded::rebuild_index:19-32` coordina vector,
  derivados, texto, sparse y scalar **secuencialmente** con `?` por fase;
  `src/sdk/serialization/impl_rebuild.rs:16-58,155-180` escribe batch de
  derivados y luego estado/health. Con V2/V3/V4/V5 ese es el seam: el delta
  útil es **reportar fases y ownership** de ese flujo, no suponer ausencia
  de swap ni crear otro manifiesto/lease. Esta lectura no prueba carrera ni
  atomicidad insuficiente por sí sola — faltan locks del caller e índices
  concretos; no se declara bug de carrera.
- **Alternativas:** extender las fases de `rebuild_index` existente
  (preferida); swap de generación preparado, manifiesto versionado, CAS o
  lease de lectores **solo como alternativas condicionadas** si la
  concurrencia lo exige; copia distribuida de segmentos/coordinadores
  (rechazada).
- **Interfaz propuesta:** `prepare_index(source_revision, cancel)
  -> Result<PreparedIndexGeneration, PrepareError>` y
  `publish_index(prepared, expected_current) -> Result<PublishedIndexGeneration,
  PublishError>`. Prepared posee temporales/reservas, sin permiso implícito
  para publicar dos veces. Reader obtiene lease de revisión; Stale, I/O,
  Cancelled y ResourceDenied no avanzan puntero.
- **Pasos:** mapear qué cambia respecto a las fases de `rebuild_index`
  (vector/derivados/texto/sparse/scalar), ownership y locks de cada
  publicador real; preparar artefactos y validar relación payload/vector;
  publicar revisión coherente; retirar solo sin lectores. Cobertura
  desconocida documentada. Reservas se liberan en fallo/cancelación,
  no dependen de final feliz.
- **Persistencia/migración/rollback:** manifiesto versionado si falta;
  conservar generación anterior y registro de huérfanos recuperable.
  Migrar por rebuild desde fuente, no reinterpretar formato viejo.
  Rollback valida tombstones/revisión vigente antes de volver a un índice
  anterior; nunca revierte fuente para maquillar readiness.
- **Dependencias:** A/B y D; scheduler/cuotas existentes para límites.
  E evita repetir promoción ante respuesta perdida.
- **FMEA:** preparación falla tras reservar → leak; RAII/estado terminal.
  Publish vector antes de payload → snapshot mixto; validar conjunto y
  revisar generación en lectura. GC anticipado → acceso inválido; leases.
  Cancelación durante commit → estado desconocido; releer manifiesto durable.
- **Aceptación futura:** normal: lector viejo y nuevo conservan referencias
  válidas; adversos: fallo/cancelación/CAS obsoleto no publican incompleto;
  reapertura distingue preparado huérfano de publicado. Telemetría sin datos.
- **No-goals:** index builder remoto, política nueva de tamaños/colas,
  refactor de scheduler ni certificación completa del FFI Go/C++.

### Ficha candidata D — checkpoint de consumo como prefijo completo, no máximo visto

- **Problema agente:** un rebuild/restart omite un delete pendiente porque el
  checkpoint saltó hasta la operación más nueva, aunque una anterior falló.
- **Fuente:** M15 `recovery-storage.md:3-12,38-65` documenta prefijo continuo,
  snapshots antes del checkpoint/truncación, poison que bloquea progreso y
  transacciones ensambladas antes de filtrar el límite. No se verificó toda
  la implementación; es inspiración documental con límite explícito.
- **Delta/seam:** V1 recovery WAL y V2 tombstones/derivados; V6 checkpoints de
  tareas no sustituyen un cursor de materialización. No se afirma que Vanta
  publique incorrectamente un máximo.
- **Alternativas:** aplicación síncrona actual (preferible si basta); extender
  cursor existente (si hay pipeline asíncrono); nuevo RecoveryStorage (rechazado).
- **Interfaz propuesta:** `advance_applied(frontier, completed_batch,
  expected_revision) -> Result<AppliedFrontier, FrontierError>`.
  Input scope/instancia, rango de operaciones y evidencias por consumidor;
  output prefijo confirmado, no max observado. Errores gap/conflict/corrupt/
  unsupported no avanzan la frontera; delete también cuenta como trabajo.
- **Pasos:** inventariar recovery/cursors; identificar productores/consumidores;
  demostrar orden de persistencia; registrar progreso junto al derivado;
  permitir replay idempotente. No crear scheduler ni nuevo harness.
- **Persistencia/migración/rollback:** cursor versionado vinculado a artefactos
  C; legacy sin frontera acreditada se reaudita/reconstruye. Mantener log
  recuperable antes de migrar. No permitir rollback de formato que el reader
  antiguo no entiende; dejar compatibilidad o rechazar downgrade explícitamente.
- **Dependencias:** A/C, orden por clave y tombstones existentes. Cancelación
  conserva último prefijo durable y no lo mueve al final del batch intentado.
- **FMEA:** hueco ignorado → pérdida silenciosa; detectar rango incompleto y
  bloquear avance. Poison → cola detenida; error visible con posición/scope,
  sin saltarlo. Purga por «visto» → recovery imposible; exigir confirmaciones
  de consumidores requeridos. Métricas de lag sin texto.
- **Aceptación futura:** favorable: replay desde cursor produce mismo estado;
  adversos: fallo intermedio, delete retrasado y cancelación no adelantan
  frontera; downgrade incompatible falla antes de mutar.
- **No-goals:** formato Milvus, CChannel/TimeTick, GC de backups ni garantía
  de compatibilidad de recovery magic 3 que upstream declara pendiente.

### Ficha candidata E — reintento condicionado por semántica e idempotencia

- **Problema agente:** retry tras timeout duplica un incremento o borra/
  inserta parcialmente una memoria ya comprometida.
- **Fuente:** M9 `Execute:79-118` conserva idempotency key y resultado original;
  el comentario advierte que misma key con payload distinto de igual forma
  no acredita igualdad de contenido. M14 `32-75,120-173,458-467` limita retries
  CAS a REPLACE y rechaza reintentar error no-CAS/de resultado desconocido.
- **Delta/seam:** V7 `persist_planned/apply_dedup_batch` y SDK de escritura:
  dedup semántico existente no es idempotencia de transporte. Revisar commit/
  error actual antes de proponer key nueva.
- **Alternativas:** retry ciego (rechazado); solo operaciones absolutas;
  outcome consultable A y key propia (condicionado a necesidad).
- **Interfaz propuesta:** `classify_retry(operation_kind, commit_outcome,
  operation_id, source_revision) -> RetryDecision`; variantes `SafeSameRequest`,
  `ResolveOutcomeFirst`, `Conflict`, `Never`. Input scope y fingerprint si el
  contrato propio exige igualdad; no afirmar exactly-once por tener una key.
  Output no ejecuta retry ni decide permisos.
- **Pasos:** trazar errores antes/después de commit; distinguir reemplazo de
  operación relativa; reutilizar IDs; validar identidad/payload cuando aplique;
  documentar al SDK cancelación y resultado desconocido.
- **Persistencia/migración/rollback:** si hay ledger de idempotencia, escribir
  resultado con la operación y versionar su retención; legacy sin key =
  no acreditado, no deduplicado inventado. Desactivar nuevos retries no borra
  resultados antiguos antes de expirar conforme al contrato.
- **Dependencias:** A/B outcome, D replay y revisión de escritura V7.
  El owner del retry es una sola capa; evitar multiplicación SDK+servidor.
- **FMEA:** key reutilizada → devolución de resultado ajeno; detectar scope/
  fingerprint. Retry relativo tras parcial → doble efecto; no automático.
  Cancelación poscommit → duplicado; resolver outcome antes de repetir.
  Registro vencido → identidad ambigua; error explícito, no éxito fingido.
- **Aceptación futura:** misma operación absoluta/key conserva resultado;
  adversos: payload distinto, error mixto, timeout y cancelación no duplican
  operación relativa. Observabilidad de decisiones, sin payload/fingerprint
  reversible expuesto. Sin nuevos pesos, umbrales ni política LLM.
- **No-goals:** clonar ventana de dedup Milvus o cambiar toda la semántica de
  dedup/supersession Vanta.

### Ficha candidata F — motivo de cierre/mantenimiento separado de readiness

- **Problema agente:** compactaciones o flushes disparados por presión de
  memoria se confunden con «memoria lista» y consumen recursos de consultas.
- **Fuente:** M6 `PolicyFenced/Capacity/Lifetime/Idle/NodeMemory:30-95` produce
  motivos tipados; M10 demuestra que flush tiene contrato propio. #49435
  aporta reporte de contención entre carga y trabajo crítico de tSafe.
- **Delta/seam:** scheduler/cuotas V6 y config V7 ya existen. Exponer motivo/
  fase y revisar separación de trabajo, no inventar budgets ni pool nuevo.
- **Alternativas:** logs libres; enum en reporte de mantenimiento existente
  (preferida); portar políticas y valores de seal (rechazado).
- **Interfaz propuesta:** `maintenance_status(task_id, namespace)
  -> Result<MaintenanceReport, StatusError>`; reason manual/capacity/recovery/
  pressure, fase y revisión, nunca `reason => ready`. Reusar tipos vigentes;
  unknown reason se preserva con versión, no se convierte en éxito.
- **Pasos:** inventariar scheduler/reportes; enlazar razón real del disparo;
  observar contención y cobertura de cancelación; integrar Studio existente.
- **Persistencia/migración/rollback:** reporte opcional versionado; no cambia
  formato de datos. Jobs anteriores = motivo desconocido. Desactivar reporte
  no cancela mantenimiento ya comprometido; rollback no elimina checkpoint.
- **Dependencias:** A para cobertura y C/D para fase; no obliga a cambiar
  parámetros de scheduling. Cancelación debe distinguir solicitud de parada
  de tarea efectivamente detenida.
- **FMEA:** pressure marcado ready → consulta prematura; detectar por A.
  trabajo bulk retiene recurso de publicación → starvation; observar espera/
  ocupación y separar ownership si se demuestra. Pérdida del reason →
  diagnóstico falso; unknown explícito. No registrar documentos.
- **Aceptación futura:** normal: razón/fase coincide con ejecución; adversos:
  presión/cancelación/restart no prometen readiness ni bloquean indefinidamente
  trabajo crítico. Validación funcional, sin configurar un benchmark nuevo.
- **No-goals:** heurísticas numéricas de seal, scheduler paralelo o importar
  pools Go/C++ como arquitectura Rust.

### Ficha candidata G — campos derivados indexables no implican proyección pública

> **Estado: requisito documental exploratorio — NO unidad de código
> extraída.** La fuente mínima es la issue #54126 (sostiene el requisito,
> no el comportamiento del pin). El validador/expansor wildcard upstream y
> el serializador de resultados Vanta/binding a extender no se localizaron
> en esta lectura; G no cuenta como extracción completada en la cobertura.

- **Problema agente:** devolver un campo interno por wildcard filtra datos
  derivados o introduce vectores internos en el contexto del modelo.
- **Fuente:** incidencia leída #54126; su expectativa separa outputs MinHash/
  BM25 de vectores binarios aportados por el usuario; refiere `82ecdb68` y
  su único comentario pregunta si sigue deseado tras #54129, sin demostrar
  implementación. Evidencia de requisito upstream, no demostración de fallo
  vigente en el pin ni en Vanta.
- **Delta/seam:** V2 particiones internas, V4 resultados SDK y V7 gobernanza
  de inyección. Ya hay provenance/ACL; revisar proyección efectiva por binding.
- **Alternativas:** lista explícita del caller (necesaria, no cubre wildcard);
  centralizar capabilities de campo en serialización existente (preferida);
  eliminar capacidad de indexación (rechazada: problema diferente).
- **Interfaz propuesta:** `validate_projection(requested_fields, schema_revision,
  caller_scope) -> Result<ProjectionPlan, ProjectionError>`; declara retornable,
  interno y no soportado. Wildcard expande solo campos retornables autorizados;
  petición explícita incompatible falla, no reconstruye internamente el valor.
- **Pasos:** inventariar DTOs/bindings/wildcards; reutilizar ACL; distinguir
  usuario/derivado; aplicar misma decisión a Query/Search y rutas de reload.
- **Persistencia/migración/rollback:** flags de schema versionados solo si no
  existen; no reescribir vectores. Legacy clasificado conservadoramente hasta
  inventario; rollback de API no autoriza exposición de campos antes denegados.
- **Dependencias:** schema/revisión C y gobernanza V7; validación antes de I/O
  costoso, cancelación no deja artefactos persistentes.
- **FMEA:** wildcard omite ACL → filtración; detectar expansión centralizada.
  cache de schema vieja → proyección inválida; vincular revisión. Rechazar
  binarios normales por parecido nominal → regresión; distinguir origen.
- **Aceptación futura:** normal: campos públicos siguen retornándose;
  adversos: wildcard/explicit, growing/equivalente local y reopen mantienen
  el mismo contrato, sin contenido sensible en error.
- **No-goals:** MinHash nuevo, cambio de BM25, nueva rúbrica de evaluación ni
  considerar un filtro de consulta como autorización.

## 3. Riesgos, issues y seguridad

**Código:** M2 permite no esperar el mismo tSafe cuando se habilita parcial.
No es un defecto; impide describir todas las consultas como fuertes por defecto.
**Hipótesis transferibles:** readiness parcial mal comunicado, referencias
durante compactación y pérdida de clasificación retryable en fronteras C++/Go.
No se siguió cada constructor de error: no se afirma un bug de propagación.
### 3.1 Incidencias públicas efectivamente leídas

**[#49435 — Timestamp lag too large con Strong count(*) y upsert](https://github.com/milvus-io/milvus/issues/49435).**
[Cuerpo API](https://api.github.com/repos/milvus-io/milvus/issues/49435)
leído íntegramente: creado `2026-04-29T03:22:11Z`, estado **closed**,
cerrado/actualizado `2026-09-04T09:07:07Z`. Reporta imagen
`2.6-20260428-20aa94f2`, standalone streaming, SIFT 2m, carga concurrente
de upsert y `Strong count(*)`; 26 query failures entre **7901 queries**,
no 26/104 líneas de log ni 26/15730 como denominador específico de query.
El reporte distingue 7829 upserts sin fallos, logs repetidos y trace IDs.
No se resolvió un SHA completo de la versión reportada: no se atribuye al pin.

M13 demuestra espera tSafe/stall y explica el síntoma operacional, no su
causa raíz histórica. Se leyeron los últimos cuatro
[comentarios](https://api.github.com/repos/milvus-io/milvus/issues/49435/comments?per_page=100):
el de agosto 13 atribuye mejora a aislar pools de carga/mutación con #50812;
agosto 26 reporta otra reproducción en `2.6-20260825-a0f27ebb`;
septiembre 4 declara backport #53139 merged y luego «not reproduced recently»
en `2.6-20260903-8c6f47d0`. **Son reportes de autores**, no verificación
independiente del merge, de Argo/Grafana privados ni de todos los commits.
No se leyó íntegra la discusión de 47 comentarios. Cerrado no acredita
solución universal; tampoco es correcto llamarlo defecto actual sin más.

Transferencia: B debe distinguir timeout/lag de resultado vacío y F no debe
permitir que mantenimiento monopolice recursos críticos. Vanta es Rust/local:
no comparte automáticamente el pool cgo del reporte. Validación futura
discriminante: publicación/consulta dependiente durante rebuild con recursos
limitados, observar progreso y cancelación; no importar sus valores de pools.

**[#54126 — MinHash outputs no deberían ser retornables](https://github.com/milvus-io/milvus/issues/54126).**
[API leída](https://api.github.com/repos/milvus-io/milvus/issues/54126):
creado `2026-10-11T14:48:41Z`, actualizado `2026-10-26T07:12:57Z`,
estado **open**. El cuerpo refiere una observación anterior contra
`82ecdb68`, explícitamente **no** una reproducción del master más reciente.
Solicita rechazo de output explícito y omisión en wildcard para campos
MinHash generados, igual que BM25, manteniéndolos indexables y sin bloquear
vectores binarios normales. Debe aplicar a Query/Search, growing/sealed y
flush/reload. El
[único comentario leído](https://api.github.com/repos/milvus-io/milvus/issues/54126/comments)
de octubre 18 pregunta si sigue deseado tras cierre de #54129; no acredita
fix ni merge. No se inspeccionó #54129 ni toda #52234.

Transferencia: G separa internal/indexable/retrievable y origen del campo;
no demuestra filtración vigente en Milvus pin ni defecto Vanta. El criterio
discriminante es misma proyección/ACL antes/después de rebuild/reopen y con
wildcard/explicit, no «no devolver ningún vector».

**Requisito documental distinto de issue leído:** M15 declara TODO #40451
para asignación mixed-version/rollback tras recovery magic 3; nodos antiguos
rechazan marcador nuevo. No se leyó ese issue: la limitación procede del
documento fijado, no se inventa estado, fecha o PR de #40451. D debe prever
compatibilidad/downgrade antes de proponer migración.

**Configuración estática:** [`configs/milvus.yaml:1391-1415`](https://github.com/milvus-io/milvus/blob/eba8be50f84776210d3946d83840dea18924846f/configs/milvus.yaml#L1391-L1415)
incluye `authorizationEnabled: false` y `defaultRootPassword: Milvus`.
Frontera: clientes de red → servicio; revisar auth/TLS y rotación real.
No demuestra exposición de una instancia; variables/deployments pueden cambiarlo.
Las credenciales MinIO del runner M8 son de fixture, no secretos descubiertos.
El mismo bloque declara `adminAuthEnabled: false`, explica plano de métricas/
WebUI y recomienda HTTPS/proxy de confianza. Hay gate implementado y
`AdminAuth: true` en registro de WebUI M17; no decir «sin autenticación»
globalmente ni confundir flag administrativo con RBAC de datos.
La página estática no contiene por sí misma datos de clúster; las APIs sí.
No se auditó la configuración real, rotación/cache de credenciales completa,
todos los endpoints ni el transporte efectivo de un despliegue.
Contenido hostil y filtros no otorgan permisos; telemetría/debug puede revelar
operación y requiere scope autorizado. Un LLM no autoriza Query ni Delete.

## 4. Prerrequisitos VantaDB

Se revalida contra el snapshot V1–V7, no el estado dirty histórico.
V3 `vector_memory_search:129-248` ya selecciona prefilter/scan/máscara;
V4 ya fusiona lexical+dense+sparse con RRF y presupuesto candidato.
V1 WAL/SyncMode y V2 backend batch/particiones ya existen. Ninguna propuesta
justifica «añadir planner», «añadir WAL» o «añadir cuotas».

| Propuesta | Evidencia que falta cerrar antes de tarea de producto |
|---|---|
| A | Recibo efectivo de escritura y cobertura por índice/backend; reutilizar health |
| B | Equivalente de await/read-your-writes y token recuperable si ya existe |
| C | Ownership/locks, publicación y reclamación reales de cada índice |
| D | Si hay consumidor asíncrono que necesite cursor propio; no imponerlo al camino síncrono |
| E | Errores pre/poscommit, IDs de mutación y dueño único de retry |
| F | Scheduler/reportes actuales y evidencia de contención, antes de separar recursos |
| G | Proyección y ACL vigentes en SDK/bindings — pendiente: localizar validación de output_fields, expansión wildcard y proyección antes/después de reload en el pin + equivalente Vanta ruta/símbolo; requisito documental, extracción no completada |

Los tests de drop/reopen mencionados por la ficha anterior no equivalen a
interrupción de energía ni se ejecutaron aquí. Las auditorías propias siguen
siendo punto de integración, no evidencia automática de cada garantía.

## 5. Qué no trasladar

No portar etcd/objeto/TSO/PChannels a un motor local por el vocabulario readiness.
No tomar reglas de seal como parámetros de Vanta ni usar flush como condición
universal de búsqueda. No llamar «sin índice» a todo segmento growing.
No convertir arquitectura distribuida en requisito para memoria de agentes.
No importar valores de seal, maxLag, timeouts o recursos de los issues como
tuning recomendado. No usar polling/sleep para enmascarar el contrato de
visibilidad, ni reintentar operaciones relativas de resultado desconocido.
No cambiar prompts, métricas ni scoring de evaluadores para aparentar ventaja.
Knowhere/GPU, etcd y backend de objetos añaden operación, ownership y superficie
de ataque que el mínimo local no necesita.

## 6. Método, licencia y límites

Sólo estático: rangos M1–M18 y guías citadas, más incidencias públicas del §3.
No se ejecutó producto, tests, builds, benchmarks ni servicios. Shallow sin historia completa/tags; LFS y
submódulos sin materializar según manifiesto; Knowhere/segcore y backends WAL
externos no auditados. Omitidos: CDC, todas las políticas de recovery/GC,
errores cgo end-to-end, índices GPU y todos los modos de despliegue.

Apache-2.0 en LICENSE del pin; M5/M17 contienen aviso Apache/LF AI para código
propio. Separar: servidor Go/C++ propio, wrapper Rust, Knowhere/índices
transitivos, backends WAL externos, SDK pymilvus, Attu y assets web compilados.
No se completó inventario legal por dependencia ni procedencia de cada asset;
no extender licencia raíz a modelos/pesos/Cloud ni a cada tercero. Revisar
LICENSE/NOTICE, cabeceras y obligaciones antes de reutilizar; traducir Go/C++
a Rust no elimina derivación ni patentes.

El informe preserva separación de readiness/publicación ya hallada; corrige
la cobertura escasa de flujos, issues no leídas y UI solo externa. Evidencia
documental de recovery no se presenta como prueba ejecutada. Una función Go
que retorna `nil` puede conservar un Status de error en protobuf. Un ACK de
flush SyncUp tiene más alcance que enqueue, pero no es «todo ready».

## 7. Benchmarks upstream

### 7.1 Runner pytest de bulk insert realmente leído

**Entry point/cadena:** M8 `TestChaos::test_bulk_insert_perf:137-164`
prepara datos y MinIO, llama `start_monitor_threads`, espera `total() > 10`,
aplica asserts, imprime resultado y termina checkers. M18
`chaos_commons.py:41-51` inicia thread daemon de `ch.keep_running`.
`BulkInsertChecker::keep_running:3329-3333` llama primero `prepare()` y después
`run_task → bulk_insert → do_bulk_insert/wait_for_bulk_insert_tasks_completed`.
El wait declara timeout 720; no es un deadline global de la prueba.

**Defaults/dataset:** `bulk_insert/conftest.py:4-10` declara `file_type=json`,
nb=50000, dim=768, varchar_len=2000, with_varchar_field=true.
M8 `prepare_bulk_insert:99-135` genera esquema/datos JSON o NPY, escribe
`/tmp/bulk_insert_data` y sube a bucket obtenido del clúster. Unidad: registros/
vectores de importación y operaciones bulk, no sesiones/turnos/preguntas.
No se leyó una semilla, un split train/holdout ni ground truth de retrieval.
No hay extractor, answerer, LLM ni juez en este camino.

**Denominadores/latencias:** M18 `Checker::total:888-892` es `_succ + _fail`;
`succ_rate` divide éxito por ese total, o devuelve 0 si total=0.
`trace:457-503` añade a `rsp_times` y `average_time` **solo operaciones cuyo
resultado devuelto es verdadero**; errores false incrementan `_fail`, pero
no entran en tiempos. `check_result:894-907` calcula media/mínimo/máximo sobre
`rsp_times`, no sobre todas las operaciones intentadas. `bulk_insert` medido
incluye submit+wait, excluye generación/upload previo y creación/estadísticas
de colección de `run_task`. Upload tiene medición separada M8:128-132.
No hay p50/p95/p99 ni ANN recall calculados en el runner leído.
`assert_statistic:21-38` contiene umbrales upstream de success/total; se
describen, no se adoptan como aceptación de Vanta.

**Fallos y divergencia estática importante:** `exception_handler:505-535`
captura excepciones y devuelve false, pero no incrementa `_fail` por sí mismo.
Si `trace` no llega al retorno de `bulk_insert`, esos contadores no cubren
todas las tentativas. Más aún, el runner configura archivos con `update`,
pero `keep_running` vuelve a llamar `prepare()`; esta crea RemoteBulkWriter,
datos de tamaño default 100000 y reemplaza `self.files`. `init_health_checkers`
de M8 no pasa minio_endpoint/bucket_name, que quedan None en el constructor
M18. **Riesgo estático**, no ejecución: puede fallar en preparación, alterar
el dataset efectivo o terminar el thread sin avanzar total; no afirmar que
los nb/archivos de M8 llegan intactos al backend. Hace falta comprobar el
entorno/wrappers antes de comparar. La espera `while total <= 10` no tiene
deadline global ni verifica liveness del daemon.

**Efectos/requisitos:** Milvus/pymilvus/pytest reales, acceso Kubernetes,
MinIO, red de upload, archivos/tempdirs, nuevas colecciones/jobs y CPU/RAM.
La rama `prepare()` añade otra ruta de RemoteBulkWriter y carga remota.
No se cuantificaron costes ni se acreditó limpieza completa de buckets/
colecciones. `teardown_method:67-69` solo espera/loguea threads; `terminate`
solo se alcanza al final feliz y resetea contadores. Usar un entorno aislado
si se autoriza ejecución posterior, nunca namespace/credenciales compartidos.
No se arrancó ni ejecutó nada en esta autoría.

**Adaptación concreta sin diseñar harness:** un adapter Vanta tendría que
mapear esquema/PK/vectores y registros bulk a ingesta SDK real, conservar
outcomes y el punto exacto de medición, y distinguir A/B de «job completado».
MinIO/Kubernetes/utility.do_bulk_insert no son portables al motor embebido;
comparar solo latencia del SDK con submit+wait remoto sería inválido.
No se cambia este runner, sus thresholds, datos, prompts o scoring.
Se aplica [METHOD](METHOD.md#protocolo-de-comparacion-de-benchmarks) y
[BENCHMARK-NOTES](BENCHMARK-NOTES.md): fallos no desaparecen del denominador,
latencias por fase y memoria/QA/ANN se separan.

### 7.2 Qué no acredita el reporte de benchmark de #49435

El cuerpo del issue aporta solicitudes, fallos y tiempos de un runner fouram
externo, no una reproducción aquí. Sus porcentajes/latencias no son resultado
de M8 ni comparación contra Vanta. No se inspeccionó código de fouram/Locust,
datasets completos, configuración privada de Argo ni toda población de sus
reintentos; se conserva el desglose del autor y su límite. No hay score de
memoria o éxito de agente deducible de `count(*)`.
Se localizaron benchmarks C++ en `internal/core/benchmark`, pero no se
auditaron; no se afirma ausencia global de runners ANN ni se certifican claims.

## 8. Matriz de brechas

| Capacidad | VantaDB | Contraste defendible |
|---|---|---|
| ANN/híbrido BM25 + dense/sparse RRF | Existe | V3/V4/V2; A no añade retrieval |
| Prefilter/scan/máscara y filtros | Existe | V3:129-248; no «planner ausente» |
| Payload/text/sparse persistentes y versiones/tombstones | Existe | V2:34-57; D contrasta revisión/retención |
| WAL versionado, SyncMode, backend batch | Existe | V1/V2; mapear recibo→publicación→B, no importar TSO |
| Auditoría textual/deep/repair y rebuild de índices | Existe | V5/V6; otras auditorías según referencia, cobertura completa no recertificada |
| Barrera SDK por token y superficies | No acreditada completa | V1 no prueba API pública de espera; B condicionado |
| Estado compuesto por generación de cada rama | Parcial en esta lectura | Health/metadata existen V2; A requiere inventario efectivo |
| Publicación/reclamación con leases | No acreditada end-to-end | C revisa ownership antes de añadir algo |
| Retry idempotente de transporte | No acreditado completo | Dedup V7 sí existe, no equivale a E |
| Cuotas, budgets y scheduler | Existe | V7/referencia; F extiende diagnóstico, no otro scheduler |
| Sesiones, provenance, dedup/supersession y checkpoints | Existe | V7; separar recurso de memoria de estado de ejecución |
| Proyección gobernada/context assembly | Existe; G específico no acreditado | V7; no afirmar ausencia global de ACL |
| Studio | Existe en otro worktree | Referencia fijada; UI Milvus sugiere estados, no nuevo desktop |
| TSO/coordinadores/PChannels | No aplicable al mínimo local | M1/M15; no adoptar runtime distribuido |

## 9. Debilidades sugeridas

Preguntar si las garantías publicadas distinguen persistencia, preparación
del índice y visibilidad de consulta. Si ya se distinguen, el resultado es
documentar/instrumentar el contrato, no añadir otro estado.
Las unidades A/B pueden servir a ingestors y SDK sin traer Milvus como runtime.
No se afirma superioridad, ausencia de capacidades Vanta ni crash safety medido.
Se descartan falsos negativos «sin híbrido», «sin filtro», «sin cuotas»,
«sin scheduler», «sin sesiones/checkpoints/provenance» y «sin UI».
V2 conserva marcadores de health y particiones derivadas: su presencia no
prueba todas las garantías de A/C/D, pero impide llamar inexistente la idea.
Las hipótesis de snapshot mixto, retry inseguro o cursor adelantado no se
elevan a defectos locales sin trazar callers y producir evidencia propia.

Negativo del evaluador propio acreditado **por la referencia**, no por
Milvus: `evals/eval_locomo.py::main:811-868` permite sintético/fallback a
FakeLocomoStore fuera de self-test. `run_locomo_evaluation:624-645` mide
retrieval por sesión y F1 léxico, no el count/ANN/QA de terceros. Esto exige
etiquetar backend/modo/error antes de comparar y no niega todas las
mediciones históricas de Vanta. No se modificó el evaluador.

## 10. UI e inspección

[`README.md:156`](https://github.com/milvus-io/milvus/blob/eba8be50f84776210d3946d83840dea18924846f/README.md#L156)
remite a **Attu externo** para administración y Birdwatcher/Prometheus/Grafana.
La ficha anterior era incompleta: **sí existe WebUI local embebida** en este
checkout, distinta de Attu, desktop y SaaS.

M17 `internal/http/server.go:50-53` embebe `webui`; `177-179` registra
condicionalmente y `RegisterWebUIHandler:274-300` sirve assets/SPA fallback
y telemetría con `AdminAuth`/`BrowserDocument`. Config
`pkg/util/paramtable/http_param.go:254-261` declara
`proxy.http.enableWebUI=true` y middleware en puerto de métricas.
Esto no implica que el gate administrativo esté habilitado: §3 distingue
`adminAuthEnabled` de disponibilidad de página.

`internal/http/webui/index.html:1-31` carga JS/CSS compilados y monta `root`
con título **Milvus Management**. Del bundle
`assets/index-nmS7rbPW.js:1498` se leyó el tramo de cliente HTTP:
Axios base `/api/v1`, endpoints cluster/db/collection/index, distribución/
target/querynode, jobs de import/index/compaction y slow queries. El helper
puede usar datos fixture según modo; no toda pantalla demuestra una llamada
real al clúster. `router.go:96-169` declara esas rutas.
Flujo acreditado: navegador → shell embebido → cliente HTTP → APIs
administrativas → presentación de estados del clúster. No se reconstruyó
todo el source frontend original ni se ejecutó/renderizó el bundle.
No se acredita un editor general de memorias, cancelación de todos los jobs
desde UI ni una aplicación desktop nativa desde esa muestra.

Attu y Zilliz Cloud siguen siendo superficies externas no inspeccionadas;
no confundir sus funcionalidades/licencias con WebUI del repo.
Vanta Studio ya vive en `develop-desktop` con memoria/actividad/diagnóstico;
los estados A/B/C/F serían insumos para esa UI existente: fase/revisión de
índice, lag de espera, fallo tipado y razón de mantenimiento. Exponer
diagnóstico read-only primero y acciones administrativas bajo autorización
explícita; página visible no equivale a permisos sobre datos o jobs.

## 11. Fuentes, cobertura y cierre de autoría

### 11.1 Fuentes complementarias fijadas

Además de M1–M8 de §2, se leyeron los siguientes rangos del SHA completo
`eba8be50f84776210d3946d83840dea18924846f`. Documento, código y reporte
upstream no se tratan como el mismo nivel de prueba.

| Ref. | Fuente / símbolo y rango |
|---|---|
| M9 | [task_insert_streaming.go:30-120](https://github.com/milvus-io/milvus/blob/eba8be50f84776210d3946d83840dea18924846f/internal/proxy/dml/task_insert_streaming.go#L30-L120), `InsertTask::Execute`; [task_insert_idempotency.go:30-110](https://github.com/milvus-io/milvus/blob/eba8be50f84776210d3946d83840dea18924846f/internal/proxy/dml/task_insert_idempotency.go#L30-L110), key/decoración |
| M10 | [datacoord/services.go:91-223](https://github.com/milvus-io/milvus/blob/eba8be50f84776210d3946d83840dea18924846f/internal/datacoord/services.go#L91-L223), `Flush`, `broadcastManualFlush`, `flushCollection` |
| M11 | [index_service.go:121-225,320-352](https://github.com/milvus-io/milvus/blob/eba8be50f84776210d3946d83840dea18924846f/internal/datacoord/index_service.go#L320-L352), `CreateIndex`; [GetIndexState:495-541](https://github.com/milvus-io/milvus/blob/eba8be50f84776210d3946d83840dea18924846f/internal/datacoord/index_service.go#L495-L541) |
| M12 | [proxy/ddl/task.go:3178-3267](https://github.com/milvus-io/milvus/blob/eba8be50f84776210d3946d83840dea18924846f/internal/proxy/ddl/task.go#L3178-L3267), `LoadCollectionTask::Execute`; [querycoordv2/services.go:276-320](https://github.com/milvus-io/milvus/blob/eba8be50f84776210d3946d83840dea18924846f/internal/querycoordv2/services.go#L276-L320); complemento M4 `segment_loader.go:247-296,390-430` |
| M13 | [delegator.go:1291-1365](https://github.com/milvus-io/milvus/blob/eba8be50f84776210d3946d83840dea18924846f/internal/querynodev2/delegator/delegator.go#L1291-L1365), `waitTSafe`; ampliación de M2 `Search:589-648` |
| M14 | [task_upsert_streaming.go:32-75,120-173,458-467](https://github.com/milvus-io/milvus/blob/eba8be50f84776210d3946d83840dea18924846f/internal/proxy/dml/task_upsert_streaming.go#L120-L173), Execute/CAS/append y retry seguro |
| M14b | [task_delete_streaming.go:21-90](https://github.com/milvus-io/milvus/blob/eba8be50f84776210d3946d83840dea18924846f/internal/proxy/dml/task_delete_streaming.go#L21-L90), `DeleteTask::Execute`; [task_delete.go:556-631,688-777](https://github.com/milvus-io/milvus/blob/eba8be50f84776210d3946d83840dea18924846f/internal/proxy/dml/task_delete.go#L556-L631), resolución compleja por QueryStream/QueryView |
| M15 | [streaming-system.md:1-43](https://github.com/milvus-io/milvus/blob/eba8be50f84776210d3946d83840dea18924846f/docs/agent_guides/streaming-system/streaming-system.md#L1-L43); [recovery-storage.md:1-65](https://github.com/milvus-io/milvus/blob/eba8be50f84776210d3946d83840dea18924846f/docs/agent_guides/streaming-system/wal/recovery-storage.md#L1-L65), documentación y limitación de downgrade |
| M16 | [httpserver/constant.go:61-69,119-124](https://github.com/milvus-io/milvus/blob/eba8be50f84776210d3946d83840dea18924846f/internal/distributed/proxy/httpserver/constant.go#L61-L69); [dql/README.md:1-55](https://github.com/milvus-io/milvus/blob/eba8be50f84776210d3946d83840dea18924846f/internal/proxy/dql/README.md#L1-L55), superficies/flujo documental |
| M17 | [http/server.go:50-53,177-179,274-300](https://github.com/milvus-io/milvus/blob/eba8be50f84776210d3946d83840dea18924846f/internal/http/server.go#L274-L300), [router.go:96-169](https://github.com/milvus-io/milvus/blob/eba8be50f84776210d3946d83840dea18924846f/internal/http/router.go#L96-L169), [webui/index.html](https://github.com/milvus-io/milvus/blob/eba8be50f84776210d3946d83840dea18924846f/internal/http/webui/index.html), [bundle:1498](https://github.com/milvus-io/milvus/blob/eba8be50f84776210d3946d83840dea18924846f/internal/http/webui/assets/index-nmS7rbPW.js#L1498), [http_param.go:254-261](https://github.com/milvus-io/milvus/blob/eba8be50f84776210d3946d83840dea18924846f/pkg/util/paramtable/http_param.go#L254-L261); `auth.go` por símbolos y `rbac.go:1-80`, no auditoría integral |
| M18 | [checker.py:457-535,888-907,3233-3337](https://github.com/milvus-io/milvus/blob/eba8be50f84776210d3946d83840dea18924846f/tests/python_client/chaos/checker.py#L3233-L3337), `trace`, exception handler, contadores y BulkInsertChecker; [chaos_commons.py:41-51](https://github.com/milvus-io/milvus/blob/eba8be50f84776210d3946d83840dea18924846f/tests/python_client/chaos/chaos_commons.py#L41-L51); [bulk conftest:4-10](https://github.com/milvus-io/milvus/blob/eba8be50f84776210d3946d83840dea18924846f/tests/python_client/bulk_insert/conftest.py#L4-L10) |

### 11.2 Referencia VantaDB y seams existentes

Fuentes fijadas a `97c9623c8ef5a70fdb51aff215bfbb3b05b15612`, no a un HEAD
posterior ni a archivos dirty. Lectura `git show` para V1–V6; V7 usa el
inventario previamente fijado, con ese límite.

| Ref. | Evidencia y uso |
|---|---|
| V1 | [wal.rs:850-907,1023-1028](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/wal.rs#L850-L907), append/batch/sync; formatos y separación de flush |
| V2 | [backend.rs:1-57](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/backend.rs#L1-L57), capabilities opcionales/particiones/default/tombstones/payload/text/sparse/internal/versions |
| V3 | [vector.rs:129-248](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/search/vector.rs#L129-L248), selección efectiva de filtro/máscara/scan |
| V4 | [hybrid.rs:13-60](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/search/hybrid.rs#L13-L60), retrieval y fusión existentes |
| V5 | [audit.rs:7-36](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/search/audit.rs#L7-L36), audit/deep/repair |
| V6 | [cli_handlers/index.rs:13-67](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/cli_handlers/index.rs#L13-L67), rebuild existente; scheduler/checkpoints según referencia V7 |
| V7 | [VANTA-REFERENCE](VANTA-REFERENCE.md#capacidades-y-limites-discriminantes), `l1_writer::{persist_planned,apply_dedup_batch}`, dedup/lifecycle, `InjectionPolicy`, context engine, scheduler, cuotas, checkpoints y Studio |

### 11.3 Cobertura leída/no leída y red pública

Leídos: contratos locales, rangos M1–M18, README/config/Go workspace/licencia
y V1–V6; V7 por referencia. Incidencias leídas: cuerpos #49435/#54126,
últimos cuatro comentarios de la primera y único comentario de la segunda.
El resultado inicial de búsqueda de issues fue exploración, no sustituto de
leer cada cuerpo. No se consultó nueva metadata de repos ni se recalcularon
capturas congeladas.

Skills cargadas: documentation-skill y coordinated-web-search; CodeGraph
solo consulta de lectura para ubicar seams, sin mantenimiento/reindexación.
Router: DuckDuckGo `site:github.com/milvus-io/milvus/issues consistency flush
load search` devolvió cero; metasearch/Bing `Milvus github issue search flush
consistency data not found`, cero; fallback `websearch.fetchWebContent` a
GitHub API de issues y lectura JSON pública de cuerpos/comentarios para
evitar truncación. Los estados/fechas proceden de esos cuerpos API.
No hay error de acceso que permita alegar «sin issues» ni se inventa
confirmación de merge desde un buscador.

No auditados: implementación completa de WAL/recovery/GC/CDC, backends
Kafka/Pulsar/Woodpecker/RMQ, Knowhere/GPU, todos los codecs y propagación cgo,
source frontend completo, Attu/Lite/Cloud/SDK externos, autenticación de todo
deployment, licencias transitivas, fouram y benchmarks C++ restantes.
Sin pérdida de energía, rendimiento, calidad, UI o corrección dinámica
certificados. La potencial divergencia de preparación M8/M18 es hallazgo
estático que necesita validación, no bug reproducido ni arreglo realizado.

### 11.4 Dependencias y validación pendiente

Orden recomendado si se aprueban specs: inventario de equivalentes → A;
B depende de identidad/outcome y publicación observables; C/D se acoplan a
ownership/replay; E depende de commit outcome, no de duplicar dedup; F/G
pueden ampliar reporte/proyección existentes sin esperar una nueva arquitectura.
No todos los candidatos son bloqueantes ni está aprobada su implementación.

Este informe excede 300 líneas para conservar evidencia autónoma. En fase
activa se separaría en specs de A/B (readiness/espera), C/D (publicación/replay),
E (retry) y F/G (diagnóstico/proyección), con trazabilidad hacia este pin.
No se crean esos archivos, IDs ni backlog en esta tarea.
Validaciones manuales futuras: cancelación posappend, publicación incompleta,
replay con delete/hueco, lectura concurrente con rebuild, retry absoluto/
relativo de resultado desconocido y proyección tras reopen. Runners actuales
solo después de resolver entorno/efectos; no se diseñan aquí configuraciones,
preguntas, scoring, umbrales ni harnesses.

### Recibo durable de autoría — Milvus

- Archivo exclusivo completado: `milvus.md`; autoría del segundo agente,
  **no revisión independiente ni aceptación final**.
- Se preservan hallazgos válidos de tSafe, estados de segmento, separación
  load/publicación, reserva de recursos, riesgos y runner bulk.
- Ampliaciones: stack/modelo/APIs/deployment, tres flujos con errores/contexto/
  fronteras de commit, propuestas A–F (D de inspiración documental) y
  requisito exploratorio G sin extracción de código completada, issues reales,
  prerrequisitos por candidato, matriz Vanta fijada y WebUI embebida real.
- Hallazgo adicional de benchmark: preparación de BulkInsertChecker puede
  reemplazar archivos y excepciones no cubren todo el denominador; riesgo
  estático explicitado, no corrección ni medición propia.
- Fuentes: M1–M18 y V1–V7 con límites; #49435/#54126 y comentarios indicados
  efectivamente leídos en GitHub API. No se declara un fix actual por cierre.
- Lagunas: WAL/GC/FFI completo, backends/SDK/UI externos, licencias transitivas,
  fouram y validación dinámica. Responsable principal decide revisión/aceptación.
- Operaciones: lectura local, git show, consultas públicas y apply_patch;
  sin apps/tests/builds/benchmarks/install/servicios, sin staging/commit/push
  ni escritura git. Recibo externo: `%TEMP%\vantadb-author-milvus.md`.
- Comprobación documental acotada: anclas propias y rutas de fuentes fijadas
  presentes; `git diff --check` de los dos informes sin observaciones.
  No se ejecutó un validador global ni se tocaron archivos concurrentes.

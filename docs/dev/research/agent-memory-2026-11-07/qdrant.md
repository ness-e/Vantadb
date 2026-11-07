---
title: "Qdrant: planificación filtrada, fusión y visibilidad persistente"
kind: research
status: proposed
description: "Lectura estática de Qdrant y unidades acotadas para contrastar con el retrieval existente de VantaDB"
tags: [research, memory, engineering]
---

# Qdrant: planificación filtrada, fusión y visibilidad persistente

## Índice y contrato de lectura

- [1. Recomendación y arquitectura](#1-recomendación-y-arquitectura).
- [2. Evidencia y unidades extraíbles](#2-evidencia-y-unidades-extraíbles).
- [3. Riesgos, defectos e issues transferibles](#3-riesgos-defectos-e-issues-transferibles).
- [4. Prerrequisitos en VantaDB](#4-prerrequisitos-en-vantadb).
- [5. Qué no trasladar](#5-qué-no-trasladar).
- [6. Método, licencia y cobertura](#6-método-licencia-y-cobertura).
- [7. Benchmark upstream, no ejecutado](#7-benchmark-upstream-no-ejecutado).
- [8. Matriz frente al estado existente](#8-matriz-frente-al-estado-existente).
- [9. Debilidades sugeridas, no bugs certificados](#9-debilidades-sugeridas-no-bugs-certificados).
- [10. UI, escritorio e inspección](#10-ui-escritorio-e-inspección).
- [11. Cierre de autoría y separación futura](#11-cierre-de-autoría-y-separación-futura).

Ampliación documental del 2026-11-07: conserva las fichas A-C y completa flujos,
contratos e incidencias. La comparación es contra VantaDB
`97c9623c8ef5a70fdb51aff215bfbb3b05b15612`, no contra el antiguo c561158 ni contra
los cambios concurrentes del workspace. Las fuentes de código upstream fijan
`222b2e44ab1c3f17e8246396ead6558e45041df7`; las páginas de issues son observaciones
públicas de esta lectura, no parte de la metadata congelada.

**Contrato transversal propuesto, no implementado:** se consume
[C13 MutationReceipt](CONSOLIDATED.md#c13--mutationreceipt-recibo-de-mutación-multialmacén):
`MutationReceipt { operation_id, acceptance, primary_commit,
primary_durability: Option<DurablePosition>, projections: Vec<ProjectionStatus>,
visibility_generation, outcome }`. Aceptación, commit primario, durabilidad,
readiness por proyección y visibilidad son **dimensiones independientes**, no
una escalera `accepted → applied → durable`. Un primario durable puede tener
texto/ANN pendientes; no se exige fsync individual de derivados reconstruibles.
La posición siempre necesita dominio de backend. Timeout/cancelación de espera
no prueban rollback; conservar conocimiento parcial no ofrece exactly-once global.
Los demás tipos hipotéticos de este informe son vistas/operaciones de C13 o
contratos de búsqueda, nunca recibos de escritura alternativos.

**Precisión local de la propuesta C13, sin editar el contrato común:**
`acceptance: Accepted | Rejected | Unknown` expresa admisión, no commit.
`Accepted` exige evidencia de admisión; `Rejected` exige rechazo confirmado
antes de admitir la operación; `Unknown` conserva la falta de evidencia.
Validación/autorización fallida antes de dispatch puede retornar error con
rechazo conocido. Error de transporte, timeout o cancelación **tras dispatch**
deja outcome desconocido salvo evidencia adicional; no fabrica `Rejected`.
`outcome` describe el resultado observado, no empaqueta una escalera de estados.
Son combinaciones válidas: aceptación conocida con `primary_commit` desconocido;
primario confirmado y `PrimarySynced` con ANN pendiente; y timeout posterior
a aplicación conocida sin perder el commit ni inventar readiness.

## 1. Recomendación y arquitectura

**Dictamen:** estudiar el contrato de selección de ruta y de visibilidad, no
reemplazar el HNSW/híbrido de VantaDB ni importar un clúster.
Fuente local `C:\Users\Eros\qdrant`; HEAD verificado con árbol fuente:
`222b2e44ab1c3f17e8246396ead6558e45041df7`.
Snapshot autorizado: **35 458 estrellas, Apache-2.0**, según
[inventario congelado](github-candidates.json); preparación en
[manifiesto](clone-manifest.json). El SHA describe master observado, no un release certificado.

```text
REST src/actix / gRPC src/tonic — Rust, Tokio
  lib/storage — colecciones, autorización, consenso
    lib/collection — shards/réplicas, submit, workers
      lib/shard — segmentos, WAL, plan de prefetch/rescore
        lib/segment — payload/index/ID tracker/vector storage
          dense HNSW | sparse inverted | texto BM25
lib/edge — API embebida sobre shard/segment, no exige servidor/Raft
```

Lectura: consulta → hojas del plan → búsqueda por segmento → estimación de
cardinalidad → scan o grafo → rescore/fusión en su nivel adecuado → top-k.
Escritura observada: reserva de cola → WAL bajo lock → dispatch ordenado →
callback opcional. No equiparar callback de aplicación con barrera física.
Existe API Edge en este checkout: sería incorrecto describir todo Qdrant como
exclusivamente distribuido; tampoco se auditó Edge como sustituto embebido.

### 1.1 Modelo, API y fronteras

El objeto recuperable es un **punto**, no un turno conversacional ni un
checkpoint de agente. `PointOperations`, `PointStructPersisted` y
`VectorStructPersisted` permiten upsert, delete, vectores nombrados y payload:
[`point_ops.rs:80-123,180-230`](https://github.com/qdrant/qdrant/blob/222b2e44ab1c3f17e8246396ead6558e45041df7/lib/shard/src/operations/point_ops.rs#L80-L230).
El ID externo identifica al punto; el offset interno y el número de operación
son dominios diferentes. Dense, sparse y payload no deben confundirse con tres
fuentes de verdad independientes de una memoria del agente.

`PlannedQuery` conserva árboles de prefetch mediante referencias a hojas
search/scroll/text; `RootPlan` decide qué payload/vectores devolver, y
`RescoreStages` distingue scoring por shard de fusión global Q3. Así se puede
expresar un retrieval multietapa sin introducir un LLM dentro del motor.
Embeddings, extracción de hechos, sesiones, instrucciones y ensamblado de
contexto siguen perteneciendo al cliente o capa de memoria. Esta muestra no
audita cada proveedor de inferencia opcional de la API.

| Capa leída | Responsabilidad y límite |
|---|---|
| `src/actix`, `src/tonic` | Superficies HTTP/gRPC; no equivalen al API Rust Edge |
| `lib/storage` | Colecciones, acceso y coordinación; consenso no acredita atomicidad de todos los índices |
| `lib/collection` | Envío, workers y réplicas; lifetime de espera distinto al de mutación |
| `lib/shard` | Plan, segmentos, resolución de targets y WAL; el filtro no es una autorización |
| `lib/segment` | Mapeo ID/offset, payload, vectores, tombstones, índices y versiones |
| `lib/edge` | Reutilización embebida de shard/segment; `serverless` no es una garantía general del servidor |

El stack leído utiliza Rust, Tokio/oneshot/mpsc para el control asíncrono y
trabajo bloqueante separado en workers. Los manifests de
[`edge/Cargo.toml:1-42`](https://github.com/qdrant/qdrant/blob/222b2e44ab1c3f17e8246396ead6558e45041df7/lib/edge/Cargo.toml#L1-L42)
declaran dependencias locales bm25/common/segment/shard/sparse/wal; `serverless`
no está habilitado en el crate publicado según su comentario. No se deduce
paridad entre publicación, bindings Python/FFI y árbol monorepo.

### 1.2 Flujos seguidos y límites de confirmación

1. **Enviar:** `LocalShard::submit_update` comprueba espacio WAL y clasifica
   operaciones dependientes del estado. Camino ordinario: `update_lock.read`
   → `append_and_dispatch` → reservar slot mpsc **antes** de append → WAL bajo
   lock → señal que posee operación o su posición WAL. Así un canal saturado
   frena admisión antes de acumular más trabajo persistido (Q5).
2. **Aplicar:** `update_worker_fn` recibe la señal y delega a trabajo bloqueante.
   Si existe callback, `update_worker_internal` llama explícitamente a
   `wal.flush()` **antes** de `CollectionUpdater::update` Q10. Por tanto no
   basta decir «callback no tiene relación con persistencia»: sí hay una
   barrera en este camino; lo no acreditado aquí es su equivalencia a
   resistencia frente a power loss de todo backend/índice.
3. **Publicar respuesta:** `await_update_result` puede devolver
   `Acknowledged`, `Completed`, `ClockRejected` o `WaitTimeout`. Un timeout
   con `operation_id` después de dispatch deja trabajo potencialmente aplicado.
   El receiver es owned y no toma prestado el shard; permite soltar guards
   exteriores antes de esperar. El worker notifica errores de aplicación;
   fallos al actualizar `applied_seq` se registran por separado Q10.
   Hay una asimetría concreta en
   [`await_update_result:180-202`](https://github.com/qdrant/qdrant/blob/222b2e44ab1c3f17e8246396ead6558e45041df7/lib/collection/src/shards/local_shard/shard_ops.rs#L180-L202):
   `(Some(receiver), None)` hace `let _ = receiver.await??` y fabrica
   `Completed`; con timeout desestructura `InternalUpdateResult` y conserva
   `status`. El worker puede enviar **por el canal exitoso** `WaitTimeout`
   desde la espera deferred
   [`update_worker.rs:201-240`](https://github.com/qdrant/qdrant/blob/222b2e44ab1c3f17e8246396ead6558e45041df7/lib/collection/src/update_workers/update_worker.rs#L201-L240).
   Requiere `wait_for_deferred && prevent_unoptimized`, sender y fallo de esa
   espera. La alcanzabilidad conjunta de configuración/caller y reproducción
   siguen pendientes: no es un bug de Vanta ni una prueba de pérdida durable.
4. **Consultar:** `PlannedQuery::add` valida profundidad, suma limit+offset
   saturando y genera hojas. Q1 estima cardinalidad ajustada a vectores
   disponibles: exact/HNSW deshabilitado → plain; intervalo inequívoco →
   plain/grafo; zona incierta → estimación adicional. No es un permiso para
   modificar thresholds en Vanta ni prueba de recall.
5. **Cambiar/borrar:** operaciones payload/vector y delete convergen en
   `CollectionUpdater::update`, que serializa actualización antes de locks de
   segmentos Q10. Delete por ID pasa por versión de punto Q12. El camino
   por filtro primero drena operaciones anteriores bajo fence, materializa
   targets y registra su variante por IDs Q11. No es un borrado transaccional
   entre todos los shards/réplicas.
6. **Derivados y retiro:** `delete_point_internal` Q12 puede conservar payload
   físico/postings en modo append-only y ocultarlos con tombstone del ID
   tracker. El código no propaga siempre delete físico a todos los vectores:
   explica el riesgo de orden de flush. «No aparece en retrieval» no significa
   borrado físico de snapshots/backups ni unlearning del modelo.
7. **Mantenimiento/cancelación:** `truncate_unapplied_wal` toma fence, cambia
   canal, detiene y espera workers, trunca desde primer pendiente y reinicia
   (Q6). Es administración destructiva explícita, no reacción automática a un
   timeout de usuario. Una tarea `spawn_blocking` iniciada no se considera
   deshecha por abandonar su future.

## 2. Evidencia y unidades extraíbles

**Código leído, enlaces al SHA local.** Rangos son la evidencia, no cobertura total.

| Ref. | Archivo y líneas | Observación |
|---|---|---|
| Q1 | [`lib/segment/src/index/hnsw_index/hnsw/read_view/dispatch.rs:40-190`](https://github.com/qdrant/qdrant/blob/222b2e44ab1c3f17e8246396ead6558e45041df7/lib/segment/src/index/hnsw_index/hnsw/read_view/dispatch.rs#L40-L190) | Exact desactiva cuantización; límites min/max deciden scan/grafo; zona incierta consulta estimación adicional |
| Q2 | [`lib/segment/src/index/struct_payload_index/read_view/filtering.rs:20-150`](https://github.com/qdrant/qdrant/blob/222b2e44ab1c3f17e8246396ead6558e45041df7/lib/segment/src/index/struct_payload_index/read_view/filtering.rs#L20-L150) | Estimaciones por campo/condición; IDs resueltos reutilizables; desconocido no se convierte en exacto |
| Q3 | [`lib/shard/src/query/planned_query.rs:13-93`](https://github.com/qdrant/qdrant/blob/222b2e44ab1c3f17e8246396ead6558e45041df7/lib/shard/src/query/planned_query.rs#L13-L93) | Hojas de búsqueda/scroll/texto y rescore por shard o colección |
| Q4 | [`lib/segment/src/common/reciprocal_rank_fusion.rs:13-98`](https://github.com/qdrant/qdrant/blob/222b2e44ab1c3f17e8246396ead6558e45041df7/lib/segment/src/common/reciprocal_rank_fusion.rs#L13-L98) | Peso comprime posiciones; valida número de pesos; no desempata por ID |
| Q5 | [`lib/collection/src/shards/local_shard/shard_ops.rs:100-217`](https://github.com/qdrant/qdrant/blob/222b2e44ab1c3f17e8246396ead6558e45041df7/lib/collection/src/shards/local_shard/shard_ops.rs#L100-L217) | WAL y cola preservan orden; Completed/Acknowledged/WaitTimeout son estados distintos |
| Q6 | [`lib/collection/src/shards/local_shard/wal_ops.rs:21-74`](https://github.com/qdrant/qdrant/blob/222b2e44ab1c3f17e8246396ead6558e45041df7/lib/collection/src/shards/local_shard/wal_ops.rs#L21-L74) | Truncar pendientes detiene workers, cambia canal, drena y reinicia bajo fences |
| Q7 | [`lib/segment/benches/conditional_search.rs:20-180`](https://github.com/qdrant/qdrant/blob/222b2e44ab1c3f17e8246396ead6558e45041df7/lib/segment/benches/conditional_search.rs#L20-L180) | Criterion compara payload plano/estructurado y filter context |
| Q8 | [`src/actix/web_ui.rs:10-43`](https://github.com/qdrant/qdrant/blob/222b2e44ab1c3f17e8246396ead6558e45041df7/src/actix/web_ui.rs#L10-L43) | Servidor de estáticos; carpeta ausente impide habilitar dashboard |
| Q9 | [`lib/edge/src/lib.rs:1-54`](https://github.com/qdrant/qdrant/blob/222b2e44ab1c3f17e8246396ead6558e45041df7/lib/edge/src/lib.rs#L1-L54) | Exporta EdgeShard, builders, lectura y variantes serverless feature-gated |
| Q10 | [`update_worker.rs:133-244,345-399`](https://github.com/qdrant/qdrant/blob/222b2e44ab1c3f17e8246396ead6558e45041df7/lib/collection/src/update_workers/update_worker.rs#L133-L399) y [`collection_updater.rs:35-107`](https://github.com/qdrant/qdrant/blob/222b2e44ab1c3f17e8246396ead6558e45041df7/lib/collection/src/collection_manager/collection_updater.rs#L35-L107) | Flush condicionado por espera, aplicación bloqueante, orden de locks y feedback |
| Q11 | [`resolve_submit.rs:1-124`](https://github.com/qdrant/qdrant/blob/222b2e44ab1c3f17e8246396ead6558e45041df7/lib/collection/src/shards/local_shard/resolve_submit.rs#L1-L124), [`resolve.rs:1-98`](https://github.com/qdrant/qdrant/blob/222b2e44ab1c3f17e8246396ead6558e45041df7/lib/shard/src/resolve.rs#L1-L98) | Fence → Plunger → resolución → un registro WAL; no lock WAL durante drain |
| Q12 | [`entry.rs:619-649`](https://github.com/qdrant/qdrant/blob/222b2e44ab1c3f17e8246396ead6558e45041df7/lib/segment/src/segment/entry.rs#L619-L649), [`segment_ops.rs:645-689`](https://github.com/qdrant/qdrant/blob/222b2e44ab1c3f17e8246396ead6558e45041df7/lib/segment/src/segment/segment_ops.rs#L645-L689) | Delete versionado y tombstones; retención física distinta de visibilidad |
| Q13 | [`planned_query.rs:117-245`](https://github.com/qdrant/qdrant/blob/222b2e44ab1c3f17e8246396ead6558e45041df7/lib/shard/src/query/planned_query.rs#L117-L245) | Profundidad, offset y materialización tardía; scroll puede hidratar hoja |
| Q14 | [`hnsw_search_graph.rs:15-116`](https://github.com/qdrant/qdrant/blob/222b2e44ab1c3f17e8246396ead6558e45041df7/lib/segment/benches/hnsw_search_graph.rs#L15-L116), [`fixture.rs:17-78`](https://github.com/qdrant/qdrant/blob/222b2e44ab1c3f17e8246396ead6558e45041df7/lib/segment/benches/fixture.rs#L17-L78) | Microbenchmark de grafo con caché persistente, no recall ANN |

### Ficha candidata A — explicación de la decisión filtrada

- **Objetivo:** distinguir decisión basada en evidencia exacta de estimación incierta.
- **Mínimo mecanismo / fuente exacta:** transportar min/expected/max y motivo
  de ruta de Q1/Q2; no copiar umbral, muestreo ni conectividad del grafo.
- **Seam Vanta existente:** `src/cost_estimator` y
  [`src/sdk/search/vector.rs:157-200`](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/search/vector.rs#L157-L200):
  ya hay prefilter, selectividad real, scan y máscara.
- **Tipos propuestos, no existentes acreditados:** `FilterEstimate { lower,
  expected, upper, origin }`, `RouteDecision { reason, route }`; ampliar una
  explicación existente antes que crear otro planner.
- **Acciones:** inventariar estimadores actuales; definir procedencia y ámbito
  namespace de los conteos; diseñar proyección diagnóstica sin cambiar ranking.
- **Dependencias:** índices payload sanos, tombstones/namespace, generación del
  índice y control de coste del diagnóstico.
- **FMEA:** conteo global con filtro por namespace induce plan erróneo; conteo
  obsoleto engaña al operador; explicación puede filtrar cardinalidad de otro tenant.
- **Aceptación funcional propuesta:** el motivo coincide con la rama tomada;
  unknown permanece unknown; el diagnóstico no cambia hits ni permisos.
- **No objetivos:** nuevos thresholds, tuning, prometer recall o QPS.

### Ficha candidata B — fronteras de aceptación, aplicación y persistencia

- **Objetivo:** que una confirmación de ingesta no prometa más de lo realizado.
- **Mínimo mecanismo / fuente exacta:** separar estados del receipt (Q5) y
  fencing de operaciones pendientes (Q6); no trasladar workers/Raft.
- **Seam Vanta existente:** [`src/wal.rs:999-1028`](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/wal.rs#L999-L1028),
  `src/backend.rs`, APIs de inserción e índices derivados.
- **Contrato propuesto:** C13 con la precisión local `acceptance` definida arriba,
  sin un `WriteVisibility` paralelo; `primary_commit`, `primary_durability`, `projections` y
  `visibility_generation` describen hechos distintos. Ausencia de posición
  durable no equivale a cero durable ni a fallo de commit.
- **Acciones:** trazar ACK actual hasta WAL/backend/índices, documentar garantías
  por SyncMode, reutilizar receipts existentes si los hay y sólo luego proponer API.
- **Dependencias:** identidad estable de operación, propagación de errores de
  fsync, acuerdo de orden entre publicación vector/text/payload.
- **FMEA:** timeout tras commit produce retry duplicado; un flag applied antes
  de publicar todos los índices crea falsa read-after-write; truncar sin fence pierde trabajo.
  Descartar feedback deferred y fabricar Completed (Q5/Q10) borra evidencia:
  conservar campos recibidos y separar timeout de espera de resultado de mutación.
- **Aceptación funcional propuesta:** no se anuncia durable sin barrera exitosa;
  resultado desconocido por timeout no se presenta como rollback.
- **No objetivos:** replicación, colas nuevas ni garantía de exactly-once.

### Ficha candidata C — contrato explícito de contribuciones híbridas

- **Objetivo:** hacer comparables explicaciones de dense/sparse/texto sin asumir
  que dos implementaciones llamadas RRF tienen semántica idéntica.
- **Mínimo / fuente:** tabla de rank/peso/contribución y nivel de fusión (Q3/Q4).
  Q4 usa `1 / ((position+1)/weight + k-1)`, no simplemente multiplicar
  `weight/(k+rank+1)`. Esa diferencia es contractual, no una recomendación de cambiar Vanta.
- **Seam:** [`src/sdk/search/fusion.rs:73-140`](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/search/fusion.rs#L73-L140)
  y `HybridFusionReport`; híbrido de tres brazos ya existe.
- **Tipos propuestos:** extender reporte con `FusionContribution` y
  `FusionFormulaId`, no duplicar `HybridFusionReport`.
- **Acciones/dependencias:** revisar reporte actual y sparse, serialización SDK
  y desempate; explicar fórmula efectiva sin alterar scores.
- **FMEA:** ocultar truncamiento por brazo hace incompleta la explicación;
  serializar metadatos no autorizados expone memoria; pesos inválidos contaminan score.
- **Aceptación funcional propuesta:** suma reportada reproduce score base;
  empate conserva determinismo Vanta; campos sensibles siguen permisos.
- **No objetivos:** adoptar el default `k=2` de Qdrant ni sustituir el RRF actual.

### Precisiones de diseño y ejecución futura de A-C

Estas fichas no autorizan cambios. **A:** caso agente = recordar un expediente
con filtro de proyecto/tenant y explicar por qué se usó scan. Entrada propuesta
`explain_route(authorized_scope, request, observed_generation)`; salida
`Result<RouteDecision, ExplainError>`. `FilterEstimate` incluye ámbito,
generación y origen; `lower <= expected <= upper` sólo se exige cuando los tres
son conocidos. `Unknown`, índice indisponible y permiso denegado no se convierten
en cero matches. Alternativa preferida: ampliar
`Embedded::explain_memory_search`, no introducir un planner de consultas nuevo.
Ese seam ya ejecuta búsquedas al explicar: véase V3, no asumir un dry-run barato.
El delta de A es la evidencia de decisión física, no añadir la explicación básica.

Pasos A: leer todos los retornos de `vector_memory_search`; asociar motivos a la
misma rama que devuelve hits; conservar IDs/generación mediante ownership de la
consulta; proyectar sólo conteos autorizados al SDK/Studio. Primero diagnóstico
opt-in compatible, después exponerlo por bindings. No necesita migrar datos ni
reindexar por sí solo; al desactivar diagnóstico el algoritmo no cambia.
Concurrencia: no recalcular conteos sobre un índice posterior fingiendo la
generación usada. Cancelación libera diagnóstico y no inicia una búsqueda extra.
Aceptación manual futura: filtros sin resultados, índices vacíos, updates y
tombstones concurrentes, scope ajeno, estimación desconocida y explicación
desactivada; contrastar rama observada con reporte, sin fijar umbrales nuevos.

**B:** caso agente = ingestor interrumpido que necesita saber si reintentar una
escritura y qué índices faltan. Entrada propuesta
`inspect_mutation(scope, operation_id) -> Result<MutationReceipt, ReceiptError>`;
esperar es una operación adicional cancelable, no un modo de reemitir la escritura.
Errores propuestos: acceso denegado, ID desconocido/expirado, backend sin barrera
e IO. Unknown/expirado no se convierte en `NotCommitted`.
`put_certified` ya emite `attestation::WriteReceipt` V4: acredita binding de
contenido, no el mismo vector de dimensiones que C13. Integrar referencias entre
ambos si aporta valor; no renombrar ni invalidar la atestación existente.

Pasos B: inventario por backend y SyncMode → mapear commit primario/barrera
real → enlazar proyecciones y revisión → serialización aditiva por SDK →
consulta/espera con autoridad original. Si se persisten receipts, versionar
metadatos y retención con el primario; un receipt efímero tras reinicio sólo
puede responder desconocido. Rebuild de índices no fabrica un commit nuevo.
Desactivar exposición no retira datos ni revierte commits. Ownership: el
worker posee la mutación despachada; el cliente sólo posee su espera. Soltar
locks de escritura/lectura de réplica antes del await; Q5/Q10 lo ilustran.
Aceptación futura: rechazo confirmado antes de admisión, aceptado con commit
desconocido, timeout tras dispatch y tras aplicación observada, feedback deferred
no Completed, fsync fallido, primario `PrimarySynced` con sparse pendiente, reapertura y operación
expirada. Ninguno exige hacer fsync individual de un derivado reconstruible.

**C:** caso agente = operador depurando por qué una evidencia lexical desplazó
a una semántica. Entrada = listas ordenadas efectivamente consumidas, pesos
validados y fórmula vigente; salida = contribuciones/ranks y fórmula dentro
del reporte existente, no un cálculo paralelo reconstruido desde hits finales.
Errores: configuración inválida o modo no soportado; no inventar contribuciones
cuando sólo se conserva top-k final. Vanta ya dispone de pesos, perfiles,
presupuesto de candidatos, boosts y desempate determinista V2.

Pasos C: identificar dos/tres brazos y boosts → obtener trazas en la pasada de
fusión → extender campos opcionales → mapear bindings/Studio. No migración
on-disk ni reindexación: es instrumentación del resultado. Alternativa de
adapter sólo para comparar respuestas externas, sin normalizar fórmulas distintas
a escondidas. Concurrencia: cada contribución pertenece al request y candidato,
no a un mapa global mutable; cancelación descarta reporte, no muta memoria.
Aceptación futura: brazos vacíos, duplicados por identidad namespace/key,
empates y pesos inválidos; score base más boost reportado reproduce score final.
Loguear fórmula, generaciones y conteos por defecto, no payload ni consulta.
Relación: A explica ruta física, C explica ranking; B explica escritura.
Ninguna sustituye provenance documental ni budgets ya existentes.

### Ficha candidata D — plan trazable y materialización tardía

- **Caso agente:** recuperar muchos candidatos de documentos largos y devolver
  únicamente el contenido del top final autorizado, evitando hidratar cada
  candidato por segmento/brazo.
- **Upstream mínimo:** Q3/Q13. `PlannedQuery::add` valida profundidad máxima,
  conserva fuentes con índices estables y separa rescore por shard/global.
  `root_plan_without_prefetches` deja las hojas search sin payload/vectores y
  materializa en raíz; scroll sí puede hacerlo directamente. No es un DAG
  general de agentes ni una promesa de snapshot universal.
- **Delta/seam real:** `src/sdk/search/vector.rs::vector_memory_search` ya
  obtiene nodos con `engine.get_many` después de candidatos; por tanto la
  optimización básica **ya existe parcialmente**. Integración en
  `src/sdk/search/hybrid.rs`, `fusion.rs` y `explain.rs`; no añadir un segundo
  motor. Inventariar si todos los brazos necesitan payload para filtros/ranking
  antes de posponerlo; algunos no pueden retrasarse sin cambiar semántica.
- **Contrato Rust propuesto:** `CandidateRef { scoped_id, source_revision,
  channel, rank }`, `MaterializationPlan { required_fields, source_refs }`,
  `hydrate_final(scope, candidates, plan) -> Result<Vec<AuthorizedHit>,
  HydrationError>`. Nombres hipotéticos. Errores: revisión retirada,
  candidato borrado, campo inaccesible, IO y cancelado; nunca rellenar con
  contenido de otra revisión silenciosamente.
- **Invariantes/ownership:** referencias owned o guard de generación válido
  hasta terminar hidratación. Ni un offset de segmento ni `usize` de otro
  request sirven como identidad persistente. La ACL se aplica a selección y
  materialización. Read-only: no commit de escritura ni receipt nuevo.
- **Pasos/alternativas:** medir por instrumentación existente cuántas lecturas
  se evitan, no con un harness nuevo; localizar materialización actual; definir
  campos necesarios por brazo; extender reporte de plan; introducir ruta
  opcional conservando fallback anterior. Si el inventario demuestra que ya
  existe para todos los brazos, sólo documentar, no reimplementar.
- **Persistencia/desactivación:** plan por petición, sin nuevo formato de datos.
  No requiere rebuild. Desactivar vuelve al flujo anterior, no a un ranking
  distinto. Depende de A para explicación de ruta y de C para nivel de fusión,
  no de un coordinador distribuido.
- **FMEA:** delete/reemplazo entre score e hidratación → contenido no corresponde
  al vector → detectar revisión desigual → omitir con motivo/reintentar según
  contrato, nunca mezclar. Error de índice de hoja → fuente incorrecta →
  validar referencias al construir plan. Campos completos antes de ACL →
  fuga por logs/caché → autorización previa y redacción.
- **Aceptación manual futura:** top vacío, offset grande, filtro en payload,
  supresión concurrente, tres brazos con el mismo ID y cancelación durante IO;
  comparar identidad/revisión y orden bajo el contrato existente.
- **No objetivos:** importar profundidad/defaults de Qdrant, diseñar nuevos
  prefetch budgets, planificar herramientas de agente o prometer menos latencia
  sin medición. El coste puede trasladarse al mantenimiento de snapshots.

### Ficha candidata E — congelar targets de mutaciones dependientes del estado

- **Caso agente:** olvidar por filtro una clase de recuerdos y asegurar que
  recovery no vuelva a evaluar esa intención contra un estado diferente.
- **Fuente/unidad:** Q11, `submit_update_filter_resolving` y `resolve_operation`.
  Fence de submits → Plunger hasta aplicar el prefijo previo → resolución →
  append de **una** operación por IDs usando su clock tag. No mantener lock
  WAL mientras el worker pueda necesitarlo; el código documenta ese deadlock.
- **Delta Vanta verificado:** V5 `Embedded::delete_by_filter` ya enumera keys
  primero y luego invoca `delete` individual. **No** está acreditado que guarde
  el filtro crudo para replay. Sí queda por decidir semántica ante revisión
  reemplazada entre ambas fases; el código leído no promete una transacción
  atómica de todo el conjunto. El issue Qdrant no acredita resurrección Vanta.
- **Seam:** `src/sdk/api/namespaces.rs::delete_by_filter`,
  `src/sdk/api/memory.rs::delete_inner`, WAL y tombstones existentes. No tocar
  checkpoints del agente ni trasladar clock tags de réplica.
- **Contrato propuesto:** `ResolvedMutation { authorized_scope,
  operation_id, selection_revision, targets: Vec<ScopedRecordRevision> }`;
  aplicación devuelve C13 `MutationReceipt`. Revisión esperada por target o
  política explícita «borrar la key actual»; decisión pendiente del producto.
  Errores: conflicto de revisión, scope inválido, selección incompleta, IO.
- **Pasos:** trazar selección/replay real → definir momento semántico →
  reutilizar representación por IDs y guards actuales → conservar targets/
  revisión cuando el contrato lo exija → propagar progreso parcial. Si fuera
  preciso un formato durable nuevo, versionar y conservar lectura de WAL
  antiguo; no reescribir historia para aparentar garantía retrospectiva.
- **Alternativas:** mantener delete individual documentado; CAS por revisión;
  batch del backend con alcance real; rechazar un modo atómico no soportado.
  Un sidecar durable trae GC y una segunda frontera crash-consistency, por lo
  que no es la opción predeterminada.
- **Ownership/cancelación:** selección owns referencias autorizadas. Cancelar
  antes de dispatch puede descartar plan; tras commit de una parte, C13 expone
  lo confirmado/pendiente. Soltar el guard sólo cuando se preservó el orden
  requerido. No detener todos los ingestors por una espera de cliente.
- **FMEA:** replay vuelve a resolver → cambia conjunto → inspeccionar tipo WAL
  y versión → fijar IDs. Key reutilizada durante selección → borrar revisión
  nueva → CAS o política documentada. Filtro masivo → un registro gigante/
  memoria retenida → detectar capacidad con controles existentes, sin copiar
  ciegamente la regla Qdrant de un único registro.
- **Aceptación futura:** selección vacía, delete/reinsert concurrente, fallo a
  mitad de lote, reinicio antes/después del límite durable y targets de tenant
  distinto. La prueba observa identidades/versiones, no sólo counts finales.
- **Dependencias/no objetivos:** C13 y contrato de revisión/tombstones; no
  dependencia obligatoria de A/C/D. No 2PC, exactly-once global ni igualdad
  automática entre réplicas que resolvieron estados diferentes.

## 3. Riesgos, defectos e issues transferibles

**Estático:** Q4 documenta que no desempata; resultados empatados no son una
fuente de orden total portable. No es por sí mismo un bug upstream.
**Hipótesis:** estimación obsoleta, timeout tras aplicación y publicación parcial
son modos de fallo a discriminar, no incidencias reproducidas.
**Incidencias públicas efectivamente leídas, observación 2026-11-07:**

| Fuente, fecha y estado confirmado por página/API | Mecanismo y alcance reportado | Contraste estático y decisión Vanta |
|---|---|---|
| [#9575 — WAL replay can resurrect filter-deleted points](https://github.com/qdrant/qdrant/issues/9575), creado 2026-06-25; página cerrado como completed el 2026-07-15 | Interacción InsertOnly + DeleteByFilter + compactación que retiró versión de delete; replay vuelve a decidir targets y cambia resultado. Autor aporta secuencia y prueba diferencial | Q11 contiene resolución+fence en nuestro SHA: evidencia del mecanismo presente, no certificado runtime ni inferencia basada sólo en cierre. Candidata E; Vanta ya selecciona keys V5 |
| [#10851 — Exhausting the replica set clock pool turns new updates into an unbounded busy wait](https://github.com/qdrant/qdrant/issues/10851), creado 2026-09-29; API open | Autor reporta 64 leases retenidos durante updates/réplica lenta; el siguiente espera con polling `yield_now`, acotado sólo por timeout opcional. Reproducción declarada en dev e857c5d9 y similitud en 1.18.0/1.17.1 | [`clock_set.rs:1-53`](https://github.com/qdrant/qdrant/blob/222b2e44ab1c3f17e8246396ead6558e45041df7/lib/collection/src/shards/replica_set/clock_set.rs#L1-L53), [`update.rs:239-275`](https://github.com/qdrant/qdrant/blob/222b2e44ab1c3f17e8246396ead6558e45041df7/lib/collection/src/shards/replica_set/update.rs#L239-L275) conservan cap/polling; no se ejecutó PoC. No hay evidencia de ese clock pool en Vanta: lección para lifetime de admisión, no bug transferido |
| [#9869 — Write operations accept timeout=0 despite minimum:1](https://github.com/qdrant/qdrant/issues/9869), creado 2026-07-16; página/API open, parked | Reporte de REST 1.18.2: schema rechaza cero pero handler lo acepta. El issue ofrece dos contratos alternativos, no determina que cero sea realmente rollback o no-wait | Prioridad de consistencia schema/SDK/runtime. Página enlaza desarrollo #10793 y commits; merge/release no consultados ni acreditados. No afirmar que nuestro SHA conserva o arregla toda la validación HTTP |

En #9575 se leyeron cuerpo y discusión sobre el orden entre modificación de
payload y resolución; la solución observada se justifica por el código, no por
la frase «closed». En #10851 se leyó el reporte y se contrastó cap/polling; el
PoC no se ejecutó y no se auditó cada ruta de fan-out. No atribuir estas
incidencias a Edge ni a todos los despliegues. Timeout antes de adquirir el
clock y timeout después del append son fronteras diferentes.

**Acceso/fallback:** `websearch.search` (DuckDuckGo) y
`metasearch.search_web` (Bing, solicitud Bing/Brave) devolvieron cero resultados
para búsquedas `site:github.com/.../issues` sobre consistency/wait/timeout.
Se recurrió a `websearch.fetchWebContent` de GitHub Search API y páginas
individuales. `Invoke-RestMethod` pudo leer metadata/cuerpo de #10851; después
GitHub API devolvió rate-limit al consultar otros comentarios/issues. Las
páginas HTML de #9575/#9869 sí se leyeron con fetchWebContent. No se presenta
el fallback como cobertura de todos los comentarios/PRs ni como auditoría de CVEs.

**Seguridad/configuración:** [`config/config.yaml:389-446`](https://github.com/qdrant/qdrant/blob/222b2e44ab1c3f17e8246396ead6558e45041df7/config/config.yaml#L389-L446)
declara `host: 0.0.0.0` y claves comentadas. Es evidencia del fichero default,
no prueba del despliegue real ni auditoría de todos los middlewares.
La frontera crítica es cliente no confiable → API; no extrapolarla a Edge
sin red. Vanta ya tiene gate de autenticación fuera de loopback.

## 4. Prerrequisitos en VantaDB

La ampliación revalida los seams siguientes sobre el SHA fijo; un futuro
implementador deberá contrastar cambios posteriores, no perseguir el HEAD
concurrente para cambiar la referencia. El baseline
[VANTA-REFERENCE](VANTA-REFERENCE.md) acredita HNSW/BM25/RRF y auditoría textual.
`src/backend.rs:34-55` declara payload/text/sparse como particiones.
`tests/durability_recovery.rs:30-80` comprueba reapertura después de drop:
su nombre «crash» no demuestra pérdida abrupta de proceso/energía.
Conservar validaciones existentes; no inventar una ausencia de WAL o persistencia.

| Ref. Vanta al SHA 97c9623c8ef5a70fdb51aff215bfbb3b05b15612 | Hallazgo revalidado y uso |
|---|---|
| V1 [`vector.rs:145-260`](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/search/vector.rs#L145-L260) | Validación dimensional, estimador, scan, máscara y selección de índice; A/D no reinician ANN |
| V2 [`fusion.rs:73-207`](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/search/fusion.rs#L73-L207) | Fusión ponderada, múltiples brazos, reporte y entity boost; C amplía trazabilidad |
| V3 [`explain.rs:9-117`](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/search/explain.rs#L9-L117) | Explicación ya existe y llama a lexical/vector/sparse; reporte de fusión `None` en ramas multibrazo leídas |
| V4 [`memory.rs:1223-1301`](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/api/memory.rs#L1223-L1301) | `put`, `put_certified` con WriteReceipt y atomicidad documentada por chunk; no receipt desde cero |
| V5 [`namespaces.rs:251-312`](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/api/namespaces.rs#L251-L312), [`memory.rs:1686-1737`](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/api/memory.rs#L1686-L1737) | Delete por filtro en dos fases; delete individual serializado por purge lock y reemplazo de derivados; purge certificate existente |
| V6 [`wal.rs:999-1028`](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/wal.rs#L999-L1028), [`backend.rs:30-55`](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/backend.rs#L30-L55) | SyncMode y flush/sync_data; particiones de primario y derivados, no garantía única universal |
| V7 [`admin.rs:17-32`](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/api/admin.rs#L17-L32) | Rebuild vector/derived/text/sparse/scalar ya existe; B/E no crean otra reparación |

**Orden de dependencias:** B primero por contrato, junto con dominio durable
de cada backend y relación con WriteReceipt; A/C pueden evaluarse sin esperar
una nueva API de escritura. D sólo después de inventariar hidratación actual;
E sólo si se requiere un contrato más fuerte que delete individual existente.
Ni la falta de réplicas ni el receipt hipotético bloquean documentar/mejorar
diagnósticos. Reusar ACL/InjectionPolicy, IDs y revisión existentes.

No se encontró literal `MutationReceipt` mediante `git grep` sobre `src`,
`vanta-memory`, `vantadb-mcp` en el snapshot: **tipo C13 no acreditado como
implementado en esas superficies**. Esto no niega receipts de atestación,
certificados de purge, otros nombres ni capacidades fuera de la muestra.

## 5. Qué no trasladar

No importar sharding/consenso, estructura interna completa del payload index,
parámetros por defecto ni API Edge para resolver una explicación diagnóstica.
No confundir formato del WAL con garantía transaccional de todos los índices.
No portar fórmula RRF sin preservar contratos y comparabilidad histórica.

Edge requiere un estudio de bindings, formatos y límites propio antes de
considerar un adapter: que sea Rust embebido no lo hace reemplazo gratis de
Vanta. Tampoco trasladar sidecars de targets o clocks distribuidos si ya existe
representación por IDs. La explicación A/C es preferible a copiar optimizadores
o defaults. Conservar la distinción Q12 entre borrado lógico y físico; no
convertirla en promesa de borrado de backups ni en un nuevo sistema de
certificados, porque V5 ya tiene esa superficie.

## 6. Método, licencia y cobertura

Sólo lectura estática; no builds, tests, servicios, benchmarks ni instalaciones.
Lectura de rangos Q1-Q9 y configuración citada, no auditoría integral.
Historial shallow, tags, LFS y submódulos no materializados según manifiesto.
Omitidos: consenso, GPU, optimizadores completos, IO/sparse kernels, RBAC
end-to-end, packaging de UI y fallos reales de almacenamiento.

Licencia por componente inspeccionado: [LICENSE raíz](https://github.com/qdrant/qdrant/blob/222b2e44ab1c3f17e8246396ead6558e45041df7/LICENSE)
Apache-2.0; manifests de `lib/segment` y `lib/edge` declaran Apache-2.0.
Eso no acredita licencias transitivas de Tokio/Actix/Criterion, SIMD/GPU ni del
frontend externo. El dashboard servido no permite asignar licencia a un bundle
que no se ha leído. Mantener avisos y revisar derivación antes de portar,
incluso reescribiendo (el original ya es Rust). Las propuestas no copian
código upstream al producto.

Los históricos OpenViking/ANN/WAL/backend son antecedentes, no defectos actuales.
Rangos, comentarios y tests son hechos estáticos, no resultados. Un contrato
de callback de Qdrant no transfiere fsync, atomicidad o aislamiento a otro
backend. Un único microbenchmark no describe distribución de filtros de
memoria real; una incidencia distribuida no invalida la API Edge.

## 7. Benchmark upstream, no ejecutado

Q7 es un microbenchmark **Criterion** del payload, no un benchmark end-to-end ANN.
Entrada sintética: 100 000 puntos, seed 42, muestras de 1 000 IDs; filtros
must/match-any y rutas query-points/filter-context sobre plain/structured.
Mide iteraciones temporizadas y escribe tamaños medios de resultados/cardinalidad.
No produce por sí solo recall, coste cloud ni p50/p95/p99 de peticiones de servidor.
Necesita toolchain/dependencias Rust y almacenamiento temporal; profiling no
Windows condicionado por cfg. TempDir delimita limpieza normal, no cleanup
garantizado después de terminación abrupta; CPU/RAM/disco se deben registrar.
No hay embeddings, generación ni juez en ese runner.

### 7.1 Cadena completa de Q7 y denominadores

`criterion_main!(benches)` → grupo `conditional-search-group` → funciones
plain/struct → fixtures
[`payload_context_fixture.rs:24-139`](https://github.com/qdrant/qdrant/blob/222b2e44ab1c3f17e8246396ead6558e45041df7/lib/segment/src/fixtures/payload_context_fixture.rs#L24-L139).
IDs secuenciales y payload sintético; struct crea campos keyword/integer/
float/geo/text/bool, mientras plain evalúa condiciones sobre almacenamiento
en memoria. No hay split train/test, preguntas ni ground truth externo.
Filtros must y match-any provienen de
[`payload_fixtures.rs:239-275`](https://github.com/qdrant/qdrant/blob/222b2e44ab1c3f17e8246396ead6558e45041df7/lib/segment/src/fixtures/payload_fixtures.rs#L239-L275).

Q7 completo, [`conditional_search.rs:20-212`](https://github.com/qdrant/qdrant/blob/222b2e44ab1c3f17e8246396ead6558e45041df7/lib/segment/benches/conditional_search.rs#L20-L212):
`query_points` cuenta matches del universo 100 000; `context-check` sólo
comprueba muestras aleatorias de 1 000 offsets, con posibles repetidos.
El cociente impreso es **entero** `result_size.checked_div(query_count)`,
no porcentaje recall. Los contadores se actualizan dentro de `b.iter`;
no separan fases de warmup/medición en ese cociente auxiliar. La generación
de filtros/muestras y el chequeo/asignaciones incluidos en el closure
también cuestan tiempo: no aislar ficticiamente sólo el índice. En match-any
el filtro se construye fuera del closure, una diferencia de alcance temporal.

`Criterion::default()` fija sus propios defaults, no se leyeron sus fuentes
para atribuir tiempos de warmup o número exacto de muestras. El runner es
local, sin concurrencia de peticiones de servidor, modelos ni tokenizer.
Errores `unwrap` abortan el caso; no son muestras de latencia exitosas ni
abstenciones de QA. No se obtuvo ningún resultado propio.

### 7.2 Segundo runner leído: grafo HNSW Q14

`criterion_main` → `hnsw_benchmark` → `fixture::make_cached_graph` → closures
`uncompressed`, `compressed` y `plain`. Defaults en fuente: 1 000 000 vectores,
dimensión 64, Cosine, M=16, ef_construct=100, ef=100, top=10 y seed 42.
Se documentan, **no se recomiendan como parámetros Vanta**.
Fixture genera vectores determinísticamente, construye parte del grafo en
paralelo Rayon y carga enlaces cached. El benchmark temporiza generación del
query/scorer + search; la construcción y `compress_ram()` están fuera de
`b.iter`. La rama plain recorre todos los puntos y busca un máximo de score,
**no construye el mismo top-10**. No es una comparación equivalente de
calidad/tarea ni calcula recall contra vecinos exactos.

La caché se escribe en `CARGO_TARGET_TMPDIR/package/crate/...`; sólo se
**reutiliza** si `BENCH_CACHE` está presente:
[`bench_cache.rs:12-88`](https://github.com/qdrant/qdrant/blob/222b2e44ab1c3f17e8246396ead6558e45041df7/lib/common/common/src/bench_cache.rs#L12-L88).
Sin esa variable reconstruye preservando temporalmente `.old`; manipula
directorios `.tmp/.old`. El propio código advierte caché obsoleta. No es
`TempDir` que garantice retirar el grafo al terminar. Seeds no prueban
determinismo del scheduling paralelo de construcción. No hay split, juez,
LLM, red pagada ni denominador de preguntas. Toolchain, CPU/RAM/disco y
profiler no Windows son costes operativos no medidos.

**Adaptación concreta, no harness diseñado:** para Q7, un adapter de lectura
de filtros tendría que mapear tipos/arrays/condiciones e identidad namespace
a Vanta conservando la población y los matches esperados; comparar context
checks de 1 000 contra enumeración total sería incorrecto. Para Q14 no se
pueden intercambiar directamente `GraphLayers`/offsets con el SDK público:
reutilizar dataset/semántica sólo bajo un runner de motor equivalente ya
aprobado. Nunca comparar esta cifra local con latencia HTTP ni LoCoMo QA.
Usar [METHOD](METHOD.md#protocolo-de-comparacion-de-benchmarks) y
[BENCHMARK-NOTES](BENCHMARK-NOTES.md); aquí no se diseñaron adapters de
ejecución, harnesses, scoring, thresholds, prompts ni nuevas configuraciones.

## 8. Matriz frente al estado existente

| Capacidad | VantaDB en referencia fija | Fuente y decisión |
|---|---|---|
| ANN filtrado / scan selectivo | Existe: vector.rs:157-200 | Revisar explicación del grado de certeza, no añadir ANN |
| Dense + sparse + BM25/RRF | Existe: hybrid.rs:33-55 | Contrato de fórmula y contribuciones por brazo |
| Payload/text/sparse derivados | Existe: backend.rs:34-55 | Auditar conteos y generaciones, no asumir índice ausente |
| WAL y SyncMode | Existe: wal.rs:999-1028 | Verificar receipt aplicado/durable por backend |
| Auditorías ANN/híbrido/WAL/backend | Parcial: alcance integral no revalidado | VANTA-REFERENCE + V7; no resetear hallazgos como nuevos |
| Explicación de búsqueda | Existe, detalle físico parcial | V3 llama retrieval; A amplía motivo/certeza, no crea explain |
| Receipt atestado y certificado de purge | Existe | V4/V5; no sustituirlos con C13 |
| C13 multialmacén uniforme SDK | No acreditado con ese tipo/contrato | B; aceptación Accepted/Rejected/Unknown separada de commit/durabilidad/proyecciones; atestación existente distinta |
| Materialización después de candidatos | Parcial/existe en vector | V1 usa get_many; D exige inventario de brazos antes de optimizar |
| Delete por filtro | Existe; atomicidad conjunta no acreditada | V5 dos fases; E sólo contrato de revisión/targets |
| Raft/sharding de Qdrant | No aplicable como requisito de memoria local | No propuesto |

## 9. Debilidades sugeridas, no bugs certificados

La cuestión útil no es «falta búsqueda híbrida», sino si un operador puede
reconstruir por qué se eligió una ruta y qué índices habían visto una escritura.
Auditar consistencia del vocabulario accepted/applied/durable y semántica de
pesos entre SDKs tiene más valor acotado que migrar el motor.
Prioridad condicional: B por honestidad contractual; A/C por inspección.
No se afirma que los contratos actuales fallen.

**Negativos concretos sustentados, no regresiones demostradas:**

- V3 tiene comentario «without executing», pero en las ramas leídas invoca
  lexical/vector/sparse y fusiona hits; tratar explain como dry-run sin coste
  sería incorrecto. No se concluye que el resultado sea erróneo.
- V3 devuelve `fusion_report: None` en ramas de tres brazos o texto+sparse de
  la muestra. La capacidad de reportar existe, pero no acredita la misma
  cobertura de diagnóstico por modo. C es delta real de observabilidad.
- V4 declara validación upfront global y commit por chunks, no rollback
  batch-wide; V5 enumera y borra individualmente. La consecuencia contractual
  es conservar progreso parcial, no llamar corrupto al motor ni inventar
  resurrección de #9575.

**Falsos positivos descartados:** no faltan filtros, HNSW/RRF/BM25, budgets,
provenance, delete, certificados, rebuild ni Studio. Las posibles carreras de
target/revisión necesitan seguir caller/recovery y una comprobación manual
discriminante; no se funden con RMW de checkpoint, L1 dedup o consenso.

## 10. UI, escritorio e inspección

Código disponible de **servido** de dashboard: Q8; depende de carpeta estática.
README anuncia Web UI, pero esa declaración no demuestra que el frontend esté
incluido en este checkout. No se inspeccionó su repositorio separado ni se
localizó aplicación de escritorio nativa en el ámbito leído.
Vanta Studio ya dispone de diagnóstico retrieval en `develop-desktop`; A-C
serían datos para esa superficie, no la propuesta de crear otra UI.

Flujo web acreditado: `web_ui_folder` aplica
`enable_static_content.unwrap_or(true)`, resuelve `static_content_dir` o
`./static`, y sólo retorna directorio existente. `web_ui_factory` monta
`/dashboard`, sirve `index.html` y agrega `X-Frame-Options: DENY` Q8.
No confundir esa cabecera anti-framing con autenticación/autorización.
El test del mismo archivo puede saltar si no hay carpeta: su presencia
no certifica que el frontend estuviera empaquetado. No se abrió navegador.

Studio se usa como destino existente según [VANTA-REFERENCE](VANTA-REFERENCE.md);
la estructura actual de componentes no se considera idéntica al snapshot
desktop `0ad38c542`. No se certificó una ruta visual nueva. Presentación
propuesta: motivos físicos y fórmula en diagnóstico retrieval; C13 con
columnas separadas de commit/durabilidad/readiness, no un único semáforo.
Acceso por scope y modo redacted son requisitos, no elementos implementados.

## 11. Cierre de autoría y separación futura

Este informe supera 300 líneas por evidencia, no por una cuota. Si se
aprueban propuestas se separarían **specs futuras** de contrato C13 (B),
diagnóstico físico/fusión (A/C), materialización (D) y selección versionada
de mutaciones (E); el informe conservaría dictamen, arquitectura y fuentes.
No se crean esos archivos, IDs de backlog ni configuraciones en esta tarea.

Cobertura leída: Q1-Q14 en rangos citados, manifests/licencia indicados,
fixtures de benchmarks y fuentes V1-V7 del SHA fijo; issues #9575/#9869 y
reporte de #10851 con contraste puntual. No cobertura exhaustiva del servidor,
Edge, SDKs externos, UI frontend, consenso, seguridad end-to-end, PRs o
crash/power-loss. La metadata congelada no se recalcula ni se actualizan clones.

Aceptaciones manuales futuras están en cada ficha; sólo después de autorización
del propietario. Pendientes sustantivos: dominios durables de todos los backends,
propagación de C13 por bindings, revisión retenida en hidratación/selección,
paridad explain/search, merge/release de fixes upstream y frontend separado.
La propuesta B conserva `acceptance` independiente y feedback completo; queda
pendiente verificar alcanzabilidad/reproducción de la rama deferred sin timeout
en Qdrant. No se afirma ese defecto en Vanta ni rollback por timeout postdispatch.
No se ejecutaron tests/builds/benchmarks/apps, validadores globales ni servicios;
no staging, commit o push por el límite explícito del brief. **Cierre de autoría
con límites, no revisión independiente ni aceptación final.**

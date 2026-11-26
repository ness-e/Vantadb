---
title: "Meilisearch: tareas visibles, filtros y explicación híbrida"
kind: research
status: proposed
description: "Contraste estático de contratos de ingesta y ranking sin proponer cambios al repositorio upstream"
tags: [research, memory, engineering]
---

# Meilisearch: tareas visibles, filtros y explicación híbrida

## Índice

- [0. Identidad, modelo y trazas](#0-identidad-modelo-y-trazas)
- [1. Recomendación y arquitectura](#1-recomendación-y-arquitectura)
- [2. Fuentes y unidades extraíbles](#2-fuentes-y-unidades-extraíbles)
- [3. Riesgos, issues y límites de seguridad](#3-riesgos-issues-y-límites-de-seguridad)
- [4. Prerrequisitos VantaDB](#4-prerrequisitos-vantadb)
- [5. Qué no trasladar](#5-qué-no-trasladar)
- [6. Método, licencia y cobertura](#6-método-licencia-y-cobertura)
- [7. Benchmarks upstream, sin ejecución](#7-benchmarks-upstream-sin-ejecución)
- [8. Matriz de brechas](#8-matriz-de-brechas)
- [9. Debilidades sugeridas](#9-debilidades-sugeridas)
- [10. UI disponible frente a externa](#10-ui-disponible-frente-a-externa)
- [11. Contratos completos y settings](#11-cierre-contractual-estados-visibilidad-y-settings)
- [12. Incidencias, runner y UI real](#12-fuentes-públicas-benchmarks-y-ui-real)

## 0. Identidad, modelo y trazas

Ampliación de la ficha existente, conservando E1–E9 y las oportunidades válidas.
La referencia Vanta para **todas las conclusiones nuevas** es
`97c9623c8ef5a70fdb51aff215bfbb3b05b15612`; c561158 queda sólo como antecedente.
No se certifica el HEAD concurrente ni el comportamiento de un servicio.
Se aplican [EXPANSION-CONTRACT](../agent-memory-2026-11-07/EXPANSION-CONTRACT.md),
[VANTA-REFERENCE](../agent-memory-2026-11-07/VANTA-REFERENCE.md), [METHOD](../agent-memory-2026-11-07/METHOD.md) y
[BENCHMARK-NOTES](../agent-memory-2026-11-07/BENCHMARK-NOTES.md), sin cambiar sus protocolos.

**Segunda lectura en profundidad (2026-11-20)** sobre el mismo pin
`c18c81973ad5d3e126f4583cdd16a9ec1719aa4a` (HEAD releído por `git log`):
añade E10–E15 con mecanismos internos no cubiertos por E1–E9 — bucle del
scheduler, cola persistida frente a overlay "processing" en RAM, autobatching,
snapshot/dump, ranking rules y búsqueda federada. El eje de incidencias
upstream (forge: issues/PRs/discussions/Actions/releases) quedó en esa
pasada **NO acreditado por prohibición expresa** del propietario del clon
(`C:\Users\Eros\meilisearch\AGENTS.md`): ni forge ni web/mirrors.
**E3 acreditado 2026-11-20 tras levantar la restricción**: el propietario
autorizó el acceso a la forge pública ese mismo día y §12.1 registra la
tercera pasada con cuerpos de issues/PRs/releases realmente leídos.
Nada aquí modifica el clon.

| Frontera | Fuente y significado |
|---|---|
| Identidad del clon | `C:\Users\Eros\meilisearch`, `main`, `c18c81973ad5d3e126f4583cdd16a9ec1719aa4a`, HEAD releído el 2026-11-07 |
| Captura preservada | [clone-manifest.json](../agent-memory-2026-11-07/clone-manifest.json), 2026-11-07T06:56:19.7819824Z; shallow, sin actualizar inventarios |
| Identidad documental | Índice por `index_uid`, documentos JSON y primary key externa; IDs internos de milli no deben confundirse con IDs de aplicación |
| Identidad operativa | `task.uid`, kind, status, payload `content_file` UUID, batch; una tarea de ingesta no es un documento ni una conversación |
| Datos derivados | Índices de palabras, facetas, vectores, estadísticas y explicaciones; la evidencia de una fase no acredita commit de todas las demás |
| Modelo de agente | No acreditado como memoria de agente: no se observan sesiones/provenance/supersession de Vanta en el contrato de documento |
| API/despliegue | Servidor Rust/Actix HTTP, daemon y almacenes LMDB vía heed; SDKs externos no auditados; no es una biblioteca embebida intercambiable con Vanta |
| Licencia | Metadata `NOASSERTION` conservada; árbol declara MIT AND BUSL-1.1, con frontera EE (§6), no “todo MIT” |

### 0.1 Mapa funcional y ownership

`crates/meilisearch` concentra rutas, auth, errores HTTP, opciones y arranque;
`meilisearch-types` serializa tareas/configuración; `index-scheduler` coordina
cola, batches y commits; `milli` implementa documentos, índices, filtros y search;
`xtask` conduce workloads y procesos; `tracing-trace` procesa spans.
`workloads/` describe activos y operaciones. `assets/` no acredita por sí sola
fuente frontend completa. Las dependencias externas de embeddings/red, índices
vectoriales y las funciones EE no quedan auditadas por esta lectura.

El ownership atraviesa `Payload` HTTP → fichero temporal/persistido → UUID en
la cola → scheduler → transacción mutable del índice. `Arc`/`Data` permiten
compartir servicios, no convertir cada HTTP en una transacción distribuida.
`spawn_blocking` puede haber persistido el fichero antes de devolver un join
error: el caller debe distinguir error de parseo, error de tarea Tokio y error
de commit. No se observó un rollback universal de esas tres fronteras.

### 0.2 Escritura, actualización y fallo antes de encolar

Fuente principal:
[documents.rs::document_addition:1534-1695](https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/crates/meilisearch/src/routes/indexes/documents.rs#L1534-L1695).

1. La ruta obtiene un scheduler protegido por acción `DOCUMENTS_ADD`; comprueba
   MIME y delimitador. JSON, NDJSON y CSV tienen ramas diferentes; content type
   ausente o delimitador incompatible devuelve error antes de crear una tarea.
2. Crea `(uuid, update_file)`. NDJSON copia/valida y persiste desde una tarea
   bloqueante; JSON/CSV convierten primero a la representación de ingesta.
   El fichero persiste **antes** de registrar el `KindWithContent`.
3. `ReplaceDocuments` y `UpdateDocuments` entran al mismo helper desde
   [replace_documents/update_documents:1367-1525](https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/crates/meilisearch/src/routes/indexes/documents.rs#L1367-L1525).
   No se debe llamar “upsert” indistintamente a reemplazo y actualización
   parcial. `on_missing_document` puede ser Skip o Create; el ID y la política
   forman parte de la operación, no sólo el JSON final.
4. Un error ordinario del registro intenta `delete_update_file(uuid)`;
   un join error de parseo también intenta limpieza y conserva el error original
   salvo los límites explícitos del código. El comentario FIXME de 1668 advierte
   otra frontera de persistencia; no lo elevamos a fuga reproducida en todos
   los caminos ni afirmamos que el comentario sustituya seguir el control flow.
5. Registrar tarea usa transacción de cola y **commit→notify** (E2). Después
   puede haber proxy de red. HTTP 202 sólo reconoce tarea, no documento visible.
   La pérdida de respuesta después del commit no implica que sea seguro repetir
   una mutación sin identidad/reconciliación.

### 0.3 Procesamiento, borrado y cancelación

E6 muestra `apply_index_operation` con una `index_wtxn`, luego
`index_wtxn.commit()` y estadísticas separadas. Un fallo de stats se registra
pero no debe declarar fallida una operación de índice ya aplicada.
Esto acredita la separación local, **no** atomicidad entre todas las bases,
proxy/red y publicación de estado terminal después de reinicio.

[delete_document:426-467](https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/crates/meilisearch/src/routes/indexes/documents.rs#L426-L467)
exige `DOCUMENTS_DELETE`, construye `DocumentDeletion` con índice e IDs,
registra y responde con recibo. Borrado por filtro/batch son otras rutas de
la misma superficie; la muestra no certifica que una ACL de búsqueda autorice
borrar ni que desaparezcan payloads de backups/tareas históricas.

En [lib.rs:994-1018](https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/crates/index-scheduler/src/lib.rs#L994-L1018)
la cancelación solicita detener tareas procesándose; no es undo de commits.
El scheduler puede agrupar tareas: orden de tarea, estado de batch y número de
documentos no son contadores intercambiables.

Mecánica de cancelación leída en la segunda pasada (E10/E12):

- `MustStopProcessing` es un `AtomicBool` cooperativo consultado dentro de los
  bucles de indexación/volcado
  ([`milli/src/must_stop_processing.rs:4-19`](https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/crates/milli/src/must_stop_processing.rs#L4-L19);
  consultas en
  [`process_dump_creation.rs:90-93,122-124,180-182`](https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/crates/index-scheduler/src/scheduler/process_dump_creation.rs#L90-L182)).
  Se activa al registrar una `TaskCancelation` que interseca el bitmap de
  procesamiento ([`lib.rs:996-999`](https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/crates/index-scheduler/src/lib.rs#L996-L999)).
- Un aborto devuelve `AbortedIndexation`/`AbortedTask`: el `wtxn` se aborta y
  las tareas vuelven a cola conservando `started_at`
  ([`scheduler/mod.rs:331-347`](https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/crates/index-scheduler/src/scheduler/mod.rs#L331-L347));
  `MaxDatabaseSizeReached` redimensiona el índice y reintenta
  ([`mod.rs:353-367`](https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/crates/index-scheduler/src/scheduler/mod.rs#L353-L367)).
- Sólo las tareas **enqueued** reciben `Status::Canceled` + `canceled_by`;
  el rollback de upgrade y la limpieza de temporales de compactación son casos
  específicos, no undo general
  ([`process_batch.rs:1217-1242,1298-1347`](https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/crates/index-scheduler/src/scheduler/process_batch.rs#L1217-L1347)).
- Tras reinicio, el canal broadcast se siembra con `StartProcessing`
  ([`scheduler/mod.rs:132-134`](https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/crates/index-scheduler/src/scheduler/mod.rs#L132-L134)):
  como "processing" es overlay en RAM, una tarea en cola reaparece `enqueued`.
  No se auditó la reanudación si el crash ocurre entre commit de índice
  (process_batch.rs:197) y commit de estado de tarea (mod.rs:452).

Recovery parcialmente acreditado (re-encolado tras aborto y semilla de arranque);
restauración desde snapshot/dump, corrupción de cola y propagación de
cancelación tras proxy quedan sin auditar.

### 0.4 Consulta, filtros y búsqueda híbrida

En E3, `Search` comparte `rtxn`, filtro, sort, distinct, deadline y estrategia;
la derivación keyword reinicia offset a cero y aumenta limit con offset.
Se ejecuta primero keyword. `results_good_enough` puede omitir el brazo
semántico; sin embedder también retorna keyword. Si hace falta y no se recibió
vector, entra al embedding de consulta, que añade dependencia/coste de proveedor.
Un error de embedding no es evidencia de que el corpus no tenga recuerdos.

Los scores son secuencias tipadas (Words/Typo/Proximity/Vector/Sort/Pin/Scale);
el merge pondera valores de score y compara criterios de sort separadamente.
No es una suma de probabilidades ni RRF de Vanta. E5 distingue atributo no
filterable y límite de profundidad. Filtro inválido, filtro válido vacío,
timeout/degradación y brazo omitido deben conservar identidades de error
distintas. Los campos `candidates` y documentos devueltos tampoco representan
necesariamente la misma población (§3 y §7).

## 1. Recomendación y arquitectura

**Dictamen:** estudiar trazabilidad de tarea y semántica de explicación; no
sustituir BM25/RRF de VantaDB por ranking de buscador de sitios.
Local `C:\Users\Eros\meilisearch`; HEAD con fuente verificado:
`c18c81973ad5d3e126f4583cdd16a9ec1719aa4a`.
[Snapshot](../agent-memory-2026-11-07/github-candidates.json): **59 806 estrellas; licencia NOASSERTION**.
[Manifiesto](../agent-memory-2026-11-07/clone-manifest.json) conserva las omisiones del clon.

```text
HTTP/Actix (Rust) → auth / rutas / tipos
  mutación → index-scheduler → cola persistida/tarea → batch
    milli → transacción heed/LMDB → índices texto/facetas/vector
  lectura → Search → filtro/candidatos → reglas léxicas
    execute_hybrid → keyword + vector si procede → merge y detalles
  mini-dashboard → recursos generados a partir de assets externos
xtask bench → workloads HTTP + spans → benchboard externo
```

No es correcto equiparar su híbrido con RRF: E3 compara componentes de score
con ratios y ordenación. Una consulta híbrida puede acabar sólo léxica si los
resultados cumplen el criterio interno o no hay embedder.
Ese hecho no implica que la misma política convenga a memoria de agentes.

## 2. Fuentes y unidades extraíbles

| Ref. | Fuente inmutable leída | Contrato observado |
|---|---|---|
| E1 | [`crates/meilisearch-types/src/tasks/mod.rs:587-625`](https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/crates/meilisearch-types/src/tasks/mod.rs#L587-L625) | Enqueued/Processing/Succeeded/Failed/Canceled serializados |
| E2 | [`crates/index-scheduler/src/lib.rs:987-1018`](https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/crates/index-scheduler/src/lib.rs#L987-L1018) | Registra en transacción; commit antes de notificar; fallo limpia datos persistidos de tarea |
| E3 | [`crates/milli/src/search/hybrid.rs:28-81`](https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/crates/milli/src/search/hybrid.rs#L28-L81) y [`283-327`](https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/crates/milli/src/search/hybrid.rs#L283-L327) | Compara scores ponderados/sort y conserva filtro; puede omitir semantic search |
| E4 | [`crates/milli/src/score_details.rs:10-41`](https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/crates/milli/src/score_details.rs#L10-L41) | Words/Typo/Proximity/Vector/Skipped/Pin/Scale no son una única relevancia escalar |
| E5 | [`crates/milli/src/search/facet/filter/mod.rs:65-102`](https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/crates/milli/src/search/facet/filter/mod.rs#L65-L102) | Errores explícitos de atributo no filterable y profundidad |
| E6 | [`crates/index-scheduler/src/scheduler/process_batch.rs:174-219`](https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/crates/index-scheduler/src/scheduler/process_batch.rs#L174-L219) | Commit del índice separado de estadísticas posteriores no críticas |
| E7 | [`crates/meilisearch/src/option.rs:214-232`](https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/crates/meilisearch/src/option.rs#L214-L232) | Env/master-key son configuración explícita |
| E8 | [`crates/xtask/src/bench/workload.rs:22-85`](https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/crates/xtask/src/bench/workload.rs#L22-L85) y [`139-183`](https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/crates/xtask/src/bench/workload.rs#L139-L183) | Workload, reports, borrado DB, arranque y terminación de servicio |
| E9 | [`crates/meilisearch/src/lib.rs:822-854`](https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/crates/meilisearch/src/lib.rs#L822-L854) | Rutas mini-dashboard condicionadas a feature y enable_frontend |
| E10 | [`crates/index-scheduler/src/scheduler/mod.rs:168-500`](https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/crates/index-scheduler/src/scheduler/mod.rs#L168-L500) y [`crates/index-scheduler/src/lib.rs:626-673`](https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/crates/index-scheduler/src/lib.rs#L626-L673) | Bucle tick: hilo dedicado, `stop_processing` después del commit de cola, aborto re-encola; estado "processing" sólo en RAM |
| E11 | [`crates/index-scheduler/src/queue/tasks.rs:33-52`](https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/crates/index-scheduler/src/queue/tasks.rs#L33-L52) y [`crates/index-scheduler/src/queue/mod.rs:263-361`](https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/crates/index-scheduler/src/queue/mod.rs#L263-L361) | Ocho DB LMDB: tareas + bitmaps invertidos por status/kind/índice/fechas; cleanup encola `TaskDeletion`; TODO reconoce que `Status::Processing` no debería serializarse |
| E12 | [`crates/index-scheduler/src/scheduler/create_batch.rs:540-839`](https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/crates/index-scheduler/src/scheduler/create_batch.rs#L540-L839) y [`crates/index-scheduler/src/scheduler/autobatcher.rs:89-133`](https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/crates/index-scheduler/src/scheduler/autobatcher.rs#L89-L133) | Prioridad estricta de tipos; autobatching por índice con límites de cantidad y bytes; `BatchStopReason` documenta por qué se cerró el batch |
| E13 | [`crates/index-scheduler/src/scheduler/process_snapshot_creation.rs:116-258`](https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/crates/index-scheduler/src/scheduler/process_snapshot_creation.rs#L116-L258) y [`process_dump_creation.rs:90-209`](https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/crates/index-scheduler/src/scheduler/process_dump_creation.rs#L90-L209) | Snapshot = copia cruda de envs LMDB + update files de tareas enqueued; dump = exportación lógica JSON con vectores re-inyectados; ambos son tareas con recibo |
| E14 | [`crates/meilisearch/src/search/federated/types.rs:44-242`](https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/crates/meilisearch/src/search/federated/types.rs#L44-L242), [`perform.rs:562-615,1233-1307`](https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/crates/meilisearch/src/search/federated/perform.rs#L562-L615) y [`weighted_scores.rs:5-14`](https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/crates/meilisearch/src/search/federated/weighted_scores.rs#L5-L14) | Búsqueda federada existe: multi-query/multi-índice con `weight` por query, merge por comparación de score values (no RRF); `remote_errors` por remoto sin tumbar la respuesta; remotos requieren feature `network` (EE) |
| E15 | [`crates/milli/src/criterion.rs:27-58,121-131`](https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/crates/milli/src/criterion.rs#L27-L131) | Ranking rules = criterios lexicográficos secuenciales (words, typo, proximity, attributeRank, sort, wordPosition, exactness), configurables; no mezcla ponderada |

### Ficha candidata A — recibo durable de tarea y resultado no ambiguo

- **Objetivo:** que aceptar un trabajo de ingesta no se confunda con completar
  indexación; mantener diagnóstico de fallo/cancelación.
- **Unidad mínima / fuente:** vocabulario E1 y orden commit→notify E2;
  separación de estadísticas no críticas E6. La segunda lectura añade: overlay
  "processing" en RAM frente a cola persistida (E10/E11), `stop_processing`
  ordenado tras el commit (E10, `scheduler/mod.rs:452-459`), aborto que
  re-encola sin falsos terminales (E10, `mod.rs:331-347`), batches que comparten
  un único commit de índice (E12; `process_batch.rs:183-197`) y recibo por
  ítem (`receivedDocuments`/`indexedDocuments`, `matchedTasks`/`canceledTasks`;
  `task_view.rs:137-181`). Snapshot/dump también son tareas con recibo (E13).
- **Seam Vanta existente:** scheduler/ingestors/checkpoints y API de actividad
  ya acreditados en [METHOD](../agent-memory-2026-11-07/METHOD.md#vantadb-como-referencia);
  `src/backend.rs` y `src/wal.rs:999-1028`.
- **Tipos propuestos:** extender receipt actual con `TaskOutcome` y
  `OperationPhase`, sólo si falta; no declarar que esos tipos ya existen.
- **Acciones:** inventariar estados actuales por ingestor/SDK; trazar fallo
  antes/después de persistencia y publicación; documentar relación tarea/batch.
- **Dependencias:** ID estable, autorización del historial y recuperación
  después de reinicio; revisar retención de errores con datos sensibles.
- **FMEA:** fallo de estadísticas revierte falsamente una tarea ya aplicada;
  cancelar no significa deshacer commit; tarea huérfana parece eternamente processing.
- **Aceptación funcional propuesta:** confirmación de cola no promete búsqueda
  inmediata; resultado terminal persiste o declara límite; timeout no es rollback.
- **No objetivos:** scheduler nuevo, webhook nuevo ni copiar toda la cola LMDB.

### Ficha candidata B — explicación tipada de modalidad y degradación

- **Objetivo:** mostrar qué brazo se ejecutó y por qué se omitió/degradó otro.
- **Unidad mínima / fuente:** E3/E4 distinguen detalles de score, dirección de
  sort, truncamiento/degradación y pin de posicionamiento.
- **Seam Vanta:** `src/sdk/search/hybrid.rs:33-59`,
  `src/sdk/search/fusion.rs:112-140`, `HybridFusionReport` y explicación actual.
- **Tipos propuestos:** añadir `RetrievalArmOutcome::{Executed, Omitted,
  Failed}` y motivo al reporte existente; no trasladar la fórmula de E3.
- **Acciones:** revisar cobertura de explicación por ruta/binding; representar
  ausencia de vector o presupuesto sin inventar score cero; preservar RRF.
- **Dependencias:** propagación de errores, controles de exposición de filtros,
  tipos de score y versión del contrato SDK.
- **FMEA:** omisión descrita como ausencia de evidencia penaliza relevancia;
  «score alto» confundido con probabilidad; detalles revelan atributos privados.
- **Aceptación funcional propuesta:** explicación permite distinguir brazo no
  ejecutado de ejecutado sin hits; campos mantienen permisos y no cambian ranking.
- **No objetivos:** early exit nuevo, `semanticRatio` nuevo ni reglas editoriales pin/scale.

### Ficha candidata C — capacidad del filtro como contrato explícito

- **Objetivo:** diferenciar filtro inválido/no soportado de cero resultados.
- **Unidad mínima / fuente:** categorías de error E5, sin imponer allowlist
  de un buscador a todos los metadatos de memoria.
- **Seam:** `MemoryMetadata`, filtros de `src/sdk/search/vector.rs:153-200`
  y partición PayloadIndex (`src/backend.rs:45-50`).
- **Tipos propuestos:** `FilterCapability`/error estructurado integrado en
  errores actuales, no un parser paralelo.
- **Acciones/dependencias:** verificar operadores y validación existentes,
  soporte por backend y mapeo SDK; redactar sólo diferencias acreditadas.
- **FMEA:** fallback silencioso ignora filtro de tenant; mensaje enumera campos
  no autorizados; profundidad sin límite agota recursos.
- **Aceptación funcional propuesta:** unsupported no retorna éxito sin filtro;
  mensajes no revelan esquema ajeno; semántica se conserva en todos los brazos.
- **No objetivos:** nuevos operadores/índices, política de seguridad basada sólo en metadata.

## 3. Riesgos, issues y límites de seguridad

**Estático:** E6 separa commit de estadísticas; un error accesorio no debe
interpretarse automáticamente como fallo de mutación.
**Hipótesis:** publicación de estados y resultados divergen tras un crash;
no se auditó todo recovery para afirmarlo como bug upstream.
**Issues/PR/discussions/releases upstream:** las dos primeras pasadas no los
consultaron (prohibición del propietario, levantada el 2026-11-20). §12.1
registra el listado original, el bloqueo y la tercera pasada con cuerpos
leídos: **#6743 confirma upstream la ventana crash-entre-commits** que §0.3
declaró sin auditar. No se propone ni modifica contenido upstream.

**Seguridad/configuración:** E7 permite master-key opcional en el tipo, pero
[`crates/meilisearch/src/main.rs:107-124`](https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/crates/meilisearch/src/main.rs#L107-L124)
rechaza producción sin clave o con clave demasiado corta. No inferir «inseguro
por defecto» sólo desde Option. Frontera: API red y endpoints de diagnóstico;
env/deploy real, autorización fina, webhooks y SSRF no auditados.
No se cambiaron configuraciones; el alcance público limitado se detalla en §12.

## 4. Prerrequisitos VantaDB

Vanta ya tiene scheduler, checkpoints, ingestors, BM25/HNSW/RRF, sparse y
explicaciones; revisar estados actuales y Studio antes de añadir otro concepto.
Revalidar worktree dirty y auditorías ANN/híbrido/WAL/backend:
el contraste no vuelve nuevos los hallazgos históricos.
[`src/sdk/search/hybrid.rs:33-59`](https://github.com/ness-e/Vantadb/blob/c561158253bb88192590fa406c61e0a27dd7df8d/src/sdk/search/hybrid.rs#L33-L59)
acredita fusión nativa, no «plugin externo».
WAL SyncMode y backend persistente existen; no se certificó crash safety.

## 5. Qué no trasladar

No cambiar RRF por ratios ni importar ranking editorial/typo tolerance por
analogía con memoria. No agregar colas y estados duplicados.
No usar «tarea succeeded» como prueba universal de fsync o calidad semántica.
No transformar atributos filterable en sustituto de autorización de namespace.
Segunda lectura: no importar el merge federado de score values ponderados
(E14) ni la cadena de criterios lexicográficos E15 como modelo de relevancia
de memoria; la federación multi-índice/multi-remoto responde a un producto de
búsqueda distribuida, no a un scope de recall embebido — se documenta como
hecho, no como candidata. No copiar bitmaps LMDB ni la máquina de
autobatching (E11/E12): en Vanta equivaldría a otra cola y otro scheduler.
No confundir snapshot-crudo-de-env (E13) con checkpoint lógico exportable:
son dos artefactos con semántica de restauración distinta.

## 6. Método, licencia y cobertura

Lectura estática E1–E9, main/config, LICENSE, LICENSE-EE y workloads;
§11–§12 amplían settings, tareas/batches, runner y frontend por CDN;
segunda lectura E10–E15 cubre scheduler interno, cola, autobatching,
snapshot/dump, federación y ranking rules.
No se ejecutaron scripts/builds/tests/benchmarks/servidores ni se instaló nada.
No se modificó el repositorio Meilisearch ni su forge.
Shallow, historia/tags omitidos; LFS/submódulos no materializados según manifiesto.
Omitidos: recovery integral, parser de filtros completo, kernels vectoriales,
auth end-to-end, módulos EE y todos los detalles de red/federación.

`NOASSERTION` del snapshot es autoritativo como metadata, no permiso ni ausencia
de licencia. [`LICENSE`](https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/LICENSE)
declara **MIT AND BUSL-1.1**; LICENSE-EE delimita archivos explícitamente EE,
uso de producción y Change Date de cuatro años. No se clasificó cada archivo
ni dependencia: licencia per-file pendiente/unknown donde no se verificó.
Un port a Rust no exime de obligaciones ni modifica esa frontera.

## 7. Benchmarks upstream, sin ejecución

[`BENCHMARKS.md:1-18`](https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/BENCHMARKS.md#L1-L18)
distingue Criterion antiguo de **xtask/workloads** integrado.
E8 arranca servicio real, envía HTTP y recopila spans/reportes; benchboard
almacena/presenta resultados fuera de este repo.
[`workloads/movies.json:1-42`](https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/workloads/movies.json#L1-L42)
fija asset remoto y SHA256, settings y carga con WaitForTask; el asset puede
costar descarga/almacenamiento y no está demostrado materializado en el clon.

Métricas documentadas: duración agregada por nombre de span y tiempo de
operaciones, no equivalentes a p50/p95/p99 de peticiones ni recall ANN/QA.
No se atribuyen percentiles calculados por dashboard externo no leído.
Infraestructura/costes: binario o build community, toolchain/deps Rust,
servicio HTTP, assets, traces/reportes y dashboard. Sin coste LLM/juez
acreditado en movies; otros workloads de embeddings no fueron auditados.
E8 **borra la DB antes de cada run** y mata proceso al completar; no se verificó
cleanup excepcional ni aislamiento de directorios con datos compartidos.
No ejecutarlo aquí. Comparación sólo conforme a
[METHOD](../agent-memory-2026-11-07/METHOD.md#protocolo-de-comparacion-de-benchmarks), sin nuevos ajustes,
umbrales, prompts, scoring o harnesses.

## 8. Matriz de brechas

| Capacidad | VantaDB | Diferencia útil, no ausencia inventada |
|---|---|---|
| Híbrido texto/dense/sparse RRF | Existe | Meili híbrido no es equivalente matemático |
| Filtros y payload index | Existe | Revisar errores/capacidades, no añadir otro filtro |
| Scheduler/ingestors/checkpoints | Existe | Contrato tarea/commit/visibilidad por acreditar end-to-end |
| Explicación retrieval/Studio | Existe; cobertura de rutas parcial | Omitido vs ejecutado sin hits puede enriquecer contrato |
| WAL/backend y auditorías ANN/híbrido | Existen; vigencia pendiente dirty | No reemplazarlos con estado de task |
| Ranking editorial pin/scale | No aplicable al mínimo de memoria | No propuesto |
| Búsqueda federada multi-índice/remoto | No existe (ni se propone): E14 documenta merge por score values ponderados y `remote_errors` parciales en upstream | Converge sólo en declarar errores parciales si hubiera fan-out futuro |
| Snapshot/dump como operación con recibo | Checkpoints/auditoría existen | E13: recibo/estado observable de mantenimiento converge con A/C9; no otra infra |
| UI desktop nueva | No necesaria | Studio ya cubre inspección |

## 9. Debilidades sugeridas

La principal pregunta es si SDK/actividad distinguen enqueued, applied,
cancelado, fallido e incierto de forma consistente. La segunda es si la
explicación distingue modalidad omitida de «no hubo evidencia».
No se afirma que fallen: las fichas son propuestas de contratos acotados
condicionadas a revalidación, no decisiones ni superioridad medida.

## 10. UI disponible frente a externa

E9 demuestra servido del mini-dashboard. La feature está habilitada en
`crates/meilisearch/Cargo.toml:143-171`, pero assets se descargan desde otro
repo/release mediante `build.rs:19-62`; no se ejecutó ese script ni se consultó
el release. No confundir assets empaquetados con frontend fuente auditado.
Benchboard es externo; no se inspeccionó aplicación desktop nativa.
Vanta Studio existente en `develop-desktop` es el destino natural de datos
diagnósticos, no se plantea copiar una interfaz ni crear otra.

## 11. Cierre contractual: estados, visibilidad y settings

**Autoría entregada para revisión; no aceptación documental.** Se conserva
la ampliación §0 y las fichas A–C. Lectura 2026-11-07, upstream
`c18c81973ad5d3e126f4583cdd16a9ec1719aa4a`, Vanta
`97c9623c8ef5a70fdb51aff215bfbb3b05b15612`; no HEAD mutable.

### 11.1. Stack, estado y fronteras adicionales

[Cargo.toml:1–40][M1] declara workspace 1.56.1, Rust edition 2021.
Actix/Tokio sirven rutas; serde/deserr validan contratos; heed/LMDB persisten
índices/cola; milli usa estructuras léxicas/facetas/vectoriales. Workspace
incluye auth, file-store, filter-parser, tracing y xtask; los clientes de
embeddings/red son dependencias separadas. No todos los backends, módulos EE
ni transitive crates se auditaron. Raíz MIT AND BUSL-1.1 y LICENSE-EE acotan
archivos expresamente EE; MIT workspace no anula esa frontera per-file.
Mini-dashboard es componente separado, no licencia de todo el producto.

`Task` tiene `batch_uid: Option<BatchId>` (`tasks/mod.rs:20–45`).
`batches.rs:1–24` expone `progress: Option<ProgressView>`: progreso de batch
no es porcentaje de documentos de una tarea; fase “Committing” no es aún
terminal Succeeded. En E6 índice commit y stats usan transacciones distintas.
`process_batch.rs:1234–1295` trata cancelación de tareas enqueued y processing
por caminos distintos; rollback de **upgrade** es caso específico, no undo
general de ingesta/cancelación [M2]. Estado de cola, visibilidad de índice,
reporte de stats y retención de fichero content_file son cuatro recursos.

**Tres flujos de referencia:** §0.2 traza replace/update desde HTTP hasta
commit de cola; §0.4 traza keyword→semantic opcional→merge; §0.3 y M2 trazan
delete/cancel hasta mutación de índice. No se certifica atomicidad entre todas
esas fronteras. Nueva lectura de settings:
`register_new_settings:641–698` valida salvo duplicado de red, aplica política
SETTINGS_UPDATE, construye `KindWithContent::SettingsUpdate`, registra mediante
spawn_blocking y puede hacer proxy después [M3]. PATCH/DELETE setting aceptado
es tarea, no anuncio de reindex concluido. Search lee `rtxn` del índice; una
consulta iniciada antes del commit no tiene por qué ver settings/documentos
nuevos. Revisión del primario y generación de proyección deben distinguirse.

**Cola y ciclo de vida (segunda lectura, E10–E13):** la cola es un env LMDB
con `all_tasks` más bitmaps invertidos por status/kind/índice/fechas/canceled_by
(`queue/tasks.rs:33–52`); `register` crea `status: Enqueued`, `batch_uid: None`
(`queue/mod.rs:263–297`) dentro de un `wtxn` cuyo commit precede al despertar
del scheduler (`lib.rs:964–1016`). "Processing" es overlay en RAM
(`processing.rs:11–18`): el bitmap persistido no debería serializarlo
(TODO en `queue/tasks.rs:38`), así que tras restart una tarea en curso vuelve
a `enqueued` vía `StartProcessing` (`scheduler/mod.rs:132–134`). `tick`
procesa un batch por iteración en hilo dedicado y hace commit de resultados
antes de limpiar el overlay — el comentario explícito de `mod.rs:457–458` exige
`stop_processing` **después** del commit para que un batch no desaparezca de la
vista. Backpressure: `NoSpaceLeftInTaskQueue` al superar 40 % del map size
(`lib.rs:966–972`, umbral en `lib.rs:110`) y `cleanup_task_queue` auto-encola
una `TaskDeletion`
cuando se rebasa `max_number_of_tasks` (`queue/mod.rs:300–361`). Los update
files se borran tras el commit del batch (`scheduler/mod.rs:461–488`) y los
webhooks se notifican después del commit, en hilo aparte, NDJSON+gzip y
best-effort — un fallo sólo se loguea (`lib.rs:1182–1280`). Prioridad de
batching: cancelación > upgrade > network > taskDeletion > compaction >
export > snapshot > dump > resto por índice (`create_batch.rs:540–705`);
autobatching agrupa tareas del mismo índice con límites de cantidad y de bytes
de content_file (`create_batch.rs:768–824`), y cada batch registra su
`BatchStopReason` (`tasks/mod.rs:999–1049`).

**Aceptación frente a resultado observable:** HTTP 202 devuelve
`SummarizedTaskView` (`routes/mod.rs:148–183`); `cancel_tasks` responde 200 con
FIXME reconociendo que debería ser 202 (`routes/tasks.rs:415–419`). La lectura
usa un snapshot LMDB: `perform_search` abre `index.read_txn()`
(`search/mod.rs:1870`), así que el commit de índice de `process_batch.rs:197`
es la única frontera de visibilidad — ni el 202 ni `status: succeeded`
intervienen en qué ve el lector. `DetailsView` separa recibido de aplicado por
ítem (`received_documents`/`indexed_documents`, `matched_tasks`/`canceled_tasks`;
`task_view.rs:137–181`). Hecho leído; no se afirma equivalencia con el modelo
de concurrencia de Vanta.

**Snapshot, dump y federación (E13–E15):** `process_snapshots_to_disk` copia
versión, env del scheduler, update files **sólo de tareas enqueued**, cada
índice y el env de auth, y empaqueta en tar.gz read-only
(`process_snapshot_creation.rs:116–258`); el propio código documenta la
ventana TOCTOU entre copiar el env y releer enqueued y prefiere copiar de más
(`:128–138`). El dump es exportación lógica (documentos JSON, vectores
re-inyectados en `_vectors`, tareas, batches, claves, webhooks;
`process_dump_creation.rs:90–209`). La federación existe como multi-search con
`federation`: `weight` por query, merge por comparación de
`WeightedScoreValue` con fallback a `weightedRankingScore` global
(`perform.rs:562–615`, `weighted_scores.rs:5–14`), errores por remoto en
`remote_errors` sin tumbar la respuesta (`perform.rs:1233–1307`,
`types.rs:226–228`) y flag `degraded` sólo para analytics
(`types.rs:237–241`); las queries `remote` requieren la feature `network` (EE).
Hecho leído de código; la inferencia "no es RRF" está acreditada por la
comparación de iteradores de score, no por documentación externa. Las
ranking rules E15 son criterios lexicográficos secuenciales configurables —
otro mecanismo distinto de la mezcla ponderada.

### 11.2. A — recibo/resultado de tarea: once campos completados

1. **Caso agente:** se captura una memoria/documento y el caller necesita saber
   si puede recuperarla, si falló o si pidió cancelar después de aplicarse.
2. **Fuente/unidad:** E1/E2/E6, M2 y la segunda lectura E10–E13: estados,
   commit→notify, batch/progress, commit de índice separado de stats, overlay
   processing-en-RAM, `stop_processing` tras commit, aborto que re-encola,
   backpressure 40 % y cleanup por `TaskDeletion`; no portar scheduler LMDB.
3. **Delta:** Vanta tiene scheduler/checkpoints/ingestors/jobs y recibos de
   escritura; falta acreditar contrato uniforme captura→aplicación→proyecciones
   en todas sus rutas. No negar lo implementado.
4. **Alternativas:** ampliar outcomes/receipts actuales, adapter de ingestión
   o vista derivada de checkpoint; rechazar cola/runtime/WAL nuevos.
5. **Rust propuesto:** `inspect_mutation(operation_id, scope) ->
   Result<MutationView, MutationViewError>`, y transición interna con
   `expected_revision/expected_phase`; salida identifica revisión fuente,
   applied/visibility, fase y resultado parcial. `UnknownOperation`, `Denied`,
   `RevisionConflict`, `StorageFailed` distintos de Pending/Cancelled.
   Invariantes: accepted ≠ ready; CancelRequested ≠ undo; terminal no se deduce
   de stats; retry conserva operación, no promete exactly-once HTTP.
6. **Seam real:** `vanta-memory/src/ingest/worker.rs::{begin,execute}`,
   `ingest/callback.rs::{IngestProgress,ProgressTracker::update_progress}`,
   `utils/task_checkpoint.rs::TaskCheckpointManager::{begin,advance,fail}`.
7. **Pasos/persistencia:** inventariar estados actuales → mapear frontera
   real de publicación → enlazar operación/checkpoint/receipt → propagar al
   SDK/Studio. Migración aditiva, legacy unknown; no convertir job UI en ledger
   durable sin evidencia. Rollback deshabilita nueva vista, conserva reparación.
8. **Relaciones:** C9 lifecycle, C4 revisión, C14 proyección, C16 inspector;
   no exige C7 ni llamadas a modelo.
9. **FMEA:** stats falla tras commit → caller cree no escrito → retry duplica;
   detectar fase/receipt, devolver aplicado con diagnóstico accesorio.
   Cancel compite con commit → UI promete rollback → falsa certeza; mostrar
   aplicado/parcial y owner de cancelación. Dos writers → fase vieja pisa nueva;
   fence de revisión en infraestructura existente.
10. **Aceptación futura:** normal captura→visible; adversa fallo antes/después
    de commit, pérdida de respuesta, cancel concurrente, restart y operación
    inaccesible por ACL. Observar operation/revisión/fase/conteos/error class,
    sin payload, claves de API ni nombres de documentos.
11. **Mínimo/no-goals/abierto:** un ingestor y transport existente; no webhooks,
    cola nueva ni garantía de fsync nueva. Retención e identidad de operación
    entre bindings requieren decisión.

`WriteReceipt` ya existe en [attestation.rs:654–738][MV1]: versión, superficies
store/WAL y SHA256 de una proyección de contenido; **no firma digital** ni
comprobación de todos los derivados, linaje o entrega al modelo. El nombre
`MutationReceipt`, si se usa para contrato multi-etapa, es **propuesto**.
Extender/adaptar evidencia actual, no afirmar “faltan recibos”.

### 11.3. B/C: contratos completos de explicación y filtros

| Campo | B — explicación por brazo | C — capacidad/error de filtro |
|---|---|---|
| Caso/fuente | Agente distingue no se ejecutó semántica de cero hits; E3/E4 muestran early-return y scores heterogéneos | Filtro de permisos no soportado nunca se ignora; E5 valida atributo filterable/profundidad |
| Delta | `HybridFusionReport`/explicación existen en `src/sdk/search/{hybrid,fusion}.rs`; cobertura por brazo/binding no acreditada integralmente | Filtros/payload index existentes; declarar capacidad efectiva del backend, no parser nuevo |
| Alternativas | Ampliar reporte actual, adapter de serialización o no exponer detalle sensible; rechazar importar fórmula de ratios | Extender errores/validación actuales, capability manifest adapter o rechazo explícito; no allowlist universal arbitraria |
| Rust **propuesto** | `explain_arms(query_id, revision, existing_report) -> Result<ArmExplanation, ExplainError>`; ExecutedEmpty/ExecutedHits/Omitted/Failed con dominio score y motivo | `validate_filter(backend_capabilities, scope, filter, schema_revision) -> Result<ValidatedFilter, FilterCapabilityError>`; salida plan validado, no ampliación de ACL |
| Errores/invariantes | `Denied`, `UnavailableReport`, `VersionUnsupported`; no score=probabilidad ni Omitted=Empty, ranking inalterado | `UnsupportedOperator`, `InvalidDepth`, `UnknownAttribute`, `Denied`, `SchemaChanged`; no success ignorando filtro, mensaje no enumera esquema ajeno |
| Seam real | `src/sdk/search/hybrid.rs`, `fusion.rs::HybridFusionReport`, transporte actual de explicación | `src/sdk/search/vector.rs` validación/pushdown y payload index `src/backend.rs`; usar errores presentes antes de ampliar |
| Pasos/persistencia/rollback | Inventariar todos los retornos → mapear estados → propagar SDK/UI. Cambio aditivo versionado de reporte, sin migrar índice; rollback omite diagnóstico nuevo, no cambia selección | Inventariar capacidades por ruta/backend → validar antes de ejecución → preservar filtro en brazos → serializar error. Sin reindex salvo capacidad nueva futura; rollback nunca activa fallback inseguro |
| Dependencias/revisión | C7 tipos de score/budget, A/C9 diagnóstico de estado, C16 UI; query/revisión fijadas por caller, read-only/cancel sin commit | C4 scope/revisión de schema y C7; revisión inválida obliga revalidación, sin lock durante I/O de proveedor |
| FMEA/detección | Error tratado como cero → respuesta engañosa; comparar estado real de cada rama, sanitizar detalles; detalle no debe filtrar query privada | Drift backend→filtro ignorado→fuga; detectar rechazo/capability mismatch antes de search. Profundidad abusiva→agotamiento; límites existentes y error explícito |
| Aceptación/observabilidad | Normal ambos brazos; adversa timeout/omitido/fallo provider/sin vector; conteos/motivos/dominio score sin consulta/contenido | Normal filtro válido/no hits; adversa operador unsupported, schema viejo y tenant cruzado; códigos/versión de capability sin nombres privados |
| Mínimo/no-goals/abierto | Una ruta ya explicada; no early-exit, semanticRatio ni pesos nuevos. Compatibilidad SDK y exposición autorizada abiertas | Operadores actuales; no ranking ni otro índice. Matriz de paridad entre bindings/backend queda por fijar |

### 11.4. D — cambio de configuración como generación verificable

1. **Problema:** un agente cambia embedder/campos indexados y el usuario cree
   que “settings actualizado” implica evidencia reindexada bajo ese esquema.
2. **Unidad/fuente:** tarea SettingsUpdate validada y registrada [M3], índices
   aplicados dentro de transacción E6; no se porta su schema/ranking.
3. **Delta:** Vanta ya tiene config, índices y auditoría; interesa vincular
   fingerprint de derivación/revisión a visibilidad y reparación, no otra config.
4. **Alternativas:** extender metadata/generaciones existentes, adapter que
   traduzca settings o sólo diagnóstico read-only; rechazar motor Meili.
5. **Rust propuesto:** `inspect_derivation(source_revision, index_generation,
   expected_fingerprint) -> Result<DerivationView, DerivationError>`;
   salida Current/Stale/Pending/UnknownEvidence por proyección.
   `Denied`, `CorruptMetadata`, `ReadFailed`, `RevisionConflict`; dimensión
   igual no acredita espacio vectorial igual, metadata ausente no prueba fallo.
6. **Seam:** `src/wiki/state.rs::WikiState`, ingestors, `TaskCheckpointManager`,
   auditoría `src/cli_handlers/index.rs`; no cambiar WAL ni scheduler.
7. **Pasos/persistencia:** inventariar fingerprint actual → marcar revisión
   productora → comparar sin mutar → trabajo de rebuild existente sólo tras
   autorización → publicar y diagnosticar. Migración aditiva legacy unknown;
   rollback conserva generación anterior y no borra evidencia de reparación.
8. **Dependencias:** C4 identidad, C14 diagnose/repair, A/C9 lifecycle; B/C7
   comunica modalidad degradada, C16 UI. No requiere LLM para inspeccionar.
9. **FMEA:** cambio embedder → ANN viejo sirve vecinos incoherentes → recall
   incorrecto; detectar fingerprint y separar lectura/rebuild. Repair corre
   contra sucesora → sobrescribe → pérdida de frescura; fence de revisión.
   ME-2: confundir settings-add con ingestión medida → atribuir descarga/
   cómputo a fase equivocada; conservar precommands/commands y modelo efectivo.
10. **Aceptación futura:** normal update con proyección vigente; adversa cancel,
    fallo embedder, legado sin metadata y reparación concurrente; registrar
    fingerprint/revisión/fase, sin modelo credentials/document payload.
    Distinguir añadir embedder sobre documentos previos de insertar documentos
    con embedder ya preparado (§12.2); cache/red/modelo no inferidos del nombre.
11. **Mínimo/no-goals/abierto:** inspección de una proyección existente, sin
    settings editor nuevo ni migrador paralelo; política serve-stale/fail-closed
    y compatibilidad del proveedor pendientes de aprobación.

## 12. Fuentes públicas, benchmarks y UI real

### 12.1. Incidencias: **E3 acreditado 2026-11-20 tras levantar la restricción**

**Historia del acceso (se conserva para trazabilidad).** Consulta 2026-11-07:
`websearch.search` DDG/Bing
`site:github.com/meilisearch/meilisearch/issues task cancel settings batch`
devolvió vacío/Bing 301; `fetchWebContent` sí leyó el **listado** oficial.
Después se recibió como instrucción activa `C:\Users\Eros\meilisearch\AGENTS.md`,
que prohibía acceder a features de forge: las dos primeras pasadas (2026-11-07
y la segunda lectura del 2026-11-20) no abrieron cuerpos, PRs, discussions,
releases ni logs. **El propietario levantó la restricción el propio
2026-11-20**: esta tercera pasada leyó cuerpos de issues, PRs y releases en
`github.com/meilisearch/meilisearch` via web pública. La investigación sigue
siendo un informe de Vanta; no propone modificar Meilisearch ni escribe en
su clon. **Alcance declarado:** issues/PRs/releases con cuerpo leído; el foro
de discussions (`github.com/orgs/meilisearch/discussions`) se vio sólo como
listado —un título relevante quedó sin cuerpo («Silent document loss …
succeeded additions vanished with no task-ledger trace», 2026-08-24)— y
Actions/CI no se inspeccionó. `meilisearch/milli` está archivado desde
2023-04-04 con 0 issues abiertos: las incidencias viven en el monorepo.
Los cuatro títulos del listado de nov-07 (#6696, #6691, #6665, #6717) quedan
ahora leídos y se citan con su estado real.

**Incidencia crítica — upstream reconoce la ventana crash-entre-commits:**

- **#6743** "An index swap runs twice after a crash between its two commits,
  so the indexes are swapped back" — **Open** (2026-11-18, v1.57.0 y `main`
  `d15a086`). <https://github.com/meilisearch/meilisearch/issues/6743>.
  `Batch::IndexSwap` commitea el efecto (`process_batch.rs:434`) en su propia
  wtxn; `task.status = Succeeded` sólo se escribe en un segundo commit de
  estados (`mod.rs:452`); "processing" nunca persiste. Un crash entre ambos
  commits deja la tarea `enqueued` con el efecto ya aplicado: al reiniciar
  se re-ejecuta y los índices quedan como antes del swap, con la cola
  mostrando `succeeded` una vez y sin error. Reproducido con SIGKILL 13–45 ms
  tras el 202. Es exactamente la ventana que §0.3/E10 declaró sin auditar
  (commit de índice `process_batch.rs:197` vs commit de estado
  `mod.rs:452`): **el defecto queda confirmado por upstream para operaciones
  no idempotentes**; el propio cuerpo advierte que el split existe en todo
  batch que commitea efecto antes que estado, inocuo sólo si la tarea es
  idempotente.
- **#1679** "Redesigning the update store" — **Closed** (2021, milestone
  v0.25.0, tech discussion). <https://github.com/meilisearch/meilisearch/issues/1679>.
  Antecedente citado por #6743: ya reconocía "processed task, but not marked
  as such, and processed again on startup" como restricción de diseño.

**Cola de tareas, batching y cancelación (E2/E6/E10–E12):**

- **#4743** "Tasks stuck in `processing` state, lots of tasks" — **Closed**
  (labels indexing/performance/support).
  <https://github.com/meilisearch/meilisearch/issues/4743>. Mantainer: tras
  crash+restart se reprocesa la última tarea "processing" una y otra vez; con
  recursos insuficientes entra en bucle de crash. Confirma operativamente el
  overlay en RAM y el re-encolado E10/E11.
- **#4791** "Catch the `MDB_TXN_FULL` error and reduce the batch size to
  eventually succeed" — **Open** (enhancement, Kerollmops).
  <https://github.com/meilisearch/meilisearch/issues/4791>. El autobatching
  puede producir una transacción demasiado grande para LMDB; upstream admite
  que debería reintentar reduciendo el tamaño del batch como ya hace con
  `MDB_MAP_FULL` (redimensiona el mapa y reintenta, cf. `mod.rs:353-367`).
  PR #4795 quedó cerrada sin merge.
- **#4598** "Index gets into bugged state when `documentDeletion` task fails
  due to transaction size (`MDB_TXN_FULL`)" — **Closed**.
  <https://github.com/meilisearch/meilisearch/issues/4598>. Tras el fallo,
  `numberOfDocuments` y `fieldDistribution` divergían: instancia de la
  divergencia efecto/estadísticas del patrón E6.
- **#5334** "Allow the task cancelation even when the task queue is half
  full" — **Closed** (completado v1.14.0 vía **PR #5492** merged).
  <https://github.com/meilisearch/meilisearch/issues/5334> ·
  <https://github.com/meilisearch/meilisearch/pull/5492>. Con la cola al 40 %
  (`NoSpaceLeftInTaskQueue`, §11.1) ni siquiera se podía registrar una
  cancelación; ahora `TaskCancelation` salta el chequeo de ocupación si
  cancela algo. Confirma que la cancelación es una tarea más de la cola.
- **#5362** "Tasks appear hung. Unable to cancel or delete processing tasks.
  Error os error 112" — **Closed**.
  <https://github.com/meilisearch/meilisearch/issues/5362>. Antecedente de
  #5334: disco lleno impedía cancelar/borrar tareas.
- **#6508** "Cancelled search requests remain in SearchQueue and can cause
  spurious TooManySearchRequests" — **Open** (bug; PR #6521 referenciado).
  <https://github.com/meilisearch/meilisearch/issues/6508>. Waiters
  cancelados conservan slot en la cola de admisión de búsqueda (no la de
  tareas): evicción de requests vivos y métrica `searches_waiting` inflada.
- **#6578** "SearchQueue: `Permit::drop(self)` sends two release signals …
  potential parallelism violation" — **Open**.
  <https://github.com/meilisearch/meilisearch/issues/6578>. Doble señal de
  liberación por permiso: el paralelismo configurado puede excederse.
  Concurrencia de búsqueda, no del scheduler de tareas.

**Snapshot/dump (E13):**

- **#6665** "[bug] A silent S3 snapshot endpoint freezes the index
  scheduler" — **Open** (2026-10-04).
  <https://github.com/meilisearch/meilisearch/issues/6665>. El snapshot S3
  (feature EE) conduce el streaming HTTP con `Handle::block_on` **en el único
  hilo del scheduler**, sin timeout ni cota al request en vuelo: un endpoint
  que acepta TCP y nunca responde congela toda la cola —altas, settings,
  cancelaciones y dumps quedan `enqueued` indefinidamente mientras `/health`
  sigue devolviendo 200— y la cancelación tampoco interrumpe (el stop flag
  sólo se consulta entre partes multipart). Era el "snapshot S3 bloqueado"
  del listado nov-07, ahora leído.
- **PR #6160** "Attempt to fix task DB corruption on snapshots" — **Merged**
  (2026-02-16), publicado en **release v1.35.1**.
  <https://github.com/meilisearch/meilisearch/pull/6160> ·
  <https://github.com/meilisearch/meilisearch/releases/tag/v1.35.1>.
  Snapshot S3 streaming de la task DB bajo escrituras concurrentes: enviar
  una tarea durante el snapshot podía corromperla; el fix bloquea la task
  DB para escritura y usa la copia nativa LMDB del env. Coherente con la
  ventana TOCTOU que el propio código E13 documenta
  (`process_snapshot_creation.rs:128-138`, "prefiere copiar de más").
- **#5280** "Fix db corruption when meili dies early in the dumpless upgrade
  process" — **Closed** (bug, v1.13.0).
  <https://github.com/meilisearch/meilisearch/issues/5280>. Otra ventana
  crash-entre-escrituras: VERSION escrito antes de que index-scheduler
  registrara su versión.

**Federated search, híbrido y ranking (E3/E4/E14/E15):**

- **#6723** "Federated search ignores custom ranking rules placed after a
  relevancy rule when merging indexes (since v1.54.0)" — **Closed**, fijado
  por **PR #6731** "Fix inter-index merge in federated search" (**Merged**
  2026-11-12). <https://github.com/meilisearch/meilisearch/issues/6723> ·
  <https://github.com/meilisearch/meilisearch/pull/6731>. Regresión en
  `ScoreDetails::score_values`/`weighted_score_values`
  (`milli/src/score_details.rs`, tocado en #6549): el adaptador `coalesce` se
  reconstruye en cada llamada del closure y descarta un detalle tras cada
  frontera de grupo, así que el merge inter-índice saltaba reglas. El pin
  c18c819 (HEAD 2026-11-07) **precede al merge del fix**: la regresión está
  presente en el código leído.
- **#5613** "Fix distinct for federated search" — **Closed** (bug; fijado
  por **PR #6214**, merged 2026-03-19).
  <https://github.com/meilisearch/meilisearch/issues/5613> ·
  <https://github.com/meilisearch/meilisearch/pull/6214>. `distinct` no se
  aplicaba sobre la lista final federada y se permitía un distinct distinto
  por query.
- **#6540** "Duplicate pinned documents in federated search" — **Closed**
  (bug; fijado por PR #6543).
  <https://github.com/meilisearch/meilisearch/issues/6540>. Pins duplicados
  cuando varias queries federadas golpean el mismo índice.
- **#6482** "totalHits and totalPages ignore pagination.maxTotalHits with
  page-based pagination" — **Open** (bug); **PR #6496** "Respect
  pagination.maxTotalHits in total hits metadata" — **Open**.
  <https://github.com/meilisearch/meilisearch/issues/6482> ·
  <https://github.com/meilisearch/meilisearch/pull/6496>. Regresión: la
  búsqueda single-index pasó por el path federado
  (`perform_federated_search`), que calculaba el total estimado sin capar
  `maxTotalHits`. Relacionado: **#6702** "maxTotalHits setting does not
  limit the estimatedTotalHits value anymore" — **Open** (bug, 2026-10-28).
  <https://github.com/meilisearch/meilisearch/issues/6702>.
- **#6590** "When hybrid search is enabled, `estimatedTotalHits` equals
  total index document count and facets ignore the query" — **Open**
  (v1.53.0). <https://github.com/meilisearch/meilisearch/issues/6590>.
  Instancia upstream de que `candidates` y hits/facets no son la misma
  población (§0.4, §3, §7).

**Settings y derivaciones (Candidata D):**

- **#6691** "Removing an embedder does not work: action succeeds, but has
  no effect" — **Open** (2026-10-17).
  <https://github.com/meilisearch/meilisearch/issues/6691>. `PATCH
  settings/embedders {"name": null}` produce una `settingsUpdate`
  `succeeded` y el embedder sigue listado y aplicándose a documentos
  nuevos. Instancia directa de "succeeded ≠ efecto aplicado" (M3, D).
- **#6696** "Prefix search stops matching words after a deletion or update:
  stored prefix entries are deleted but not recomputed" — **Open**
  (2026-10-25). <https://github.com/meilisearch/meilisearch/issues/6696>.
  Tras un batch en que una palabra desaparece, las entradas de prefijo se
  borran y no se recomputan salvo que otra palabra del mismo prefijo cambie
  en ese batch; `prefix_iter_mut` además borra prefijos más largos (`c`
  elimina `co`, `com`, `comp`…) y el fallback se capa en 1 000 palabras.
  Derivaciones silenciosamente divergentes tras un batch `succeeded`.
- **#6717** "Settings update adding disableOnAttributes drops most matches
  for 2–4 char prefixes on other attributes (v1.56.1)" — **Open**
  (2026-11-03). <https://github.com/meilisearch/meilisearch/issues/6717>.
  Una `settingsUpdate` `succeeded` degrada prefijos en atributos ajenos al
  cambio de typoTolerance.
- **#4493** "[Tracking-issue] Settings diff-indexing" — tracking issue
  (descripción y comentarios de mantenimiento vistos; cabecera de estado no
  verificada). <https://github.com/meilisearch/meilisearch/issues/4493>.
  Upstream reconoce que settings→reindex es parcial por partes: borrar un
  embedder reindexa los siguientes para no dejar "hueco", cambios de
  `apiKey`/`documentTemplate` tienen semánticas de reindex distintas.

**Payloads/huella (§0.2):**

- **#6048** "Make sure payloads are effectively deleted on error" —
  **Open** (bug, abierto por Kerollmops; cerrado not-planned y reabierto en
  2026-08). <https://github.com/meilisearch/meilisearch/issues/6048>.
  Payloads no se borran del disco cuando el error ocurre antes de
  `FileStore`. Da estado upstream al FIXME de `documents.rs:1668` que §0.2
  registró sin elevar a fuga.

**Releases consultadas:** página oficial de releases leída el 2026-11-20:
última **v1.57.0** (2026-11-16, tag `1b78265`), junto a v1.56.2/v1.56.1/
v1.56.0, v1.55.x y v1.54.3; la v1.35.1 se cita por el fix de corrupción de
task DB. Ninguna release publicada cierra #6743 al momento de la lectura.

**Lectura del eje para Vanta:** el defecto transferible dominante es el
split efecto/estado (#6743): toda mutación no idempotente necesita
idempotencia, fence de revisión o commit único —ver backlog **X-MEI-05**—.
#6665 añade que una tarea de mantenimiento con I/O externo síncrono en el
hilo del scheduler congela toda la cola: timeouts, chequeos de cancelación
dentro del I/O y no bloquear el writer único son controles transferibles.
#6696/#6717/#6691 refuerzan D: un `succeeded` no acredita derivaciones
consistentes ni siquiera el cambio de settings pedido. Estos hallazgos
elevan la hipótesis de §3 de "no auditado" a "reconocido upstream" en los
puntos citados; el resto de recovery sigue sin auditar.

Hallazgos estáticos acreditados (sin cambios): E2/E6 separan commit/notify/
stats; M3 encola settings antes de su aplicación; E3 permite omitir
semantic. Segunda lectura: E10 acredita commit-de-cola antes de limpiar el
overlay processing y re-encolado tras aborto; E11 el layout de la cola; E12
la prioridad/límites de batching; E13 snapshot/dump como tareas; E14 la
federación ponderada; E15 los criterios lexicográficos.
La lectura estática no demostró corrupción por crash; la forge sí confirma
doble aplicación de efecto (#6743) y una corrupción de task DB en snapshot
S3 ya corregida (#6160/v1.35.1). Una tarea `succeeded` no garantiza efecto
aplicado en todos los caminos (#6691) ni derivaciones consistentes
(#6696/#6717), pero tampoco "siempre miente".
Seguridad: autorización de search/document/settings/task son acciones
diferentes; explicar filtros nunca autoriza enumerar esquema ajeno.
Una clave en UI/URL, payload de tarea y log de traces cruzan fronteras de
confianza distintas; no copiar exposición diagnóstica sin redacción.

### 12.2. Benchmark realmente leído, sin harness nuevo

**ME-2 — dos workloads de embeddings, no sólo movies:**

| Fuente al pin c18c819 | Preparación fuera de captura | Operación dentro de captura |
|---|---|---|
| [embeddings-settings-add.json:1-65][ME2A] | PATCH searchable/filterable/sortable `DontWait`; POST movies-100 `WaitForTask` | PATCH añade embedder `default`, source `huggingFace`, model/revision/documentTemplate/distribution **null**, `WaitForTask`: embedding/reindex sobre documentos previos |
| [embeddings-movies-subset-hf.json:1-61][ME2B] | PATCH campos con `WaitForTask`; PATCH embedder `default`, source `huggingFace` sin modelo/revisión explícitos, `WaitForTask` | POST movies-100 `WaitForTask`: inserción/embedding con configuración ya preparada |

Ambos declaran **run_count=5**, `--max-indexing-threads=4` y asset remoto
`movies-100.json`, SHA256
`d215e395e4240f12f03b8f1f68901eac82d9e7ded5b462cbf4a6b8efde76c6c6`.
El nombre no acredita por sí solo cardinalidad/esquema; asset no descargado.
`run_workload_commands:40-85` [ME2R] ejecuta precommands **antes** de
start_report y commands **después**; no son dos mediciones intercambiables.
Preparar embedder puede descargar/cargar modelo fuera de captura en el
segundo; en el primero esa preparación puede ocurrir durante settings medidos.
No se acredita qué descargas acertaron caché ni cache fría de kernel/modelo.

**Identidad del modelo/defaults:** los JSON no fijan artefactos del modelo.
La lectura acotada encuentra `hf::EmbedderOptions::new/default:60-75` con
`BAAI/bge-base-en-v1.5`, revisión declarada
`617ca489d9e86b49b8167676d8220688b99db36e` [ME2H].
`settings.rs::hugging_face:2126-2150` parte de ese default, resetea revisión
si model está Set y aplica revision sólo si Set [ME2S]. Null/ausencia no
deben publicarse como identidad efectiva sin verificar resolución de settings,
estado persistido y bytes del modelo. No se descargó ni hash-verificó modelo.
`hf.rs:208-270` [ME2H] selecciona CUDA si disponible o CPU, resuelve repo/
revisión y config/tokenizer/weights vía hf_hub: red/caché/CPU-GPU y carga no
desaparecen porque no haya juez LLM. Templates/tokenización/modelo efectivo
condicionan coste; no factura ni superioridad medidas.

**Población/fallos/efectos:** cinco runs solicitados por workload frente a
reportes completados y spans recogidos; no cinco preguntas ni denominador QA.
WaitForTask espera cola global vacía, no éxito de task exacta (detalle abajo):
task fallida puede terminar la espera; error HTTP/procesador puede abortar
antes del reporte. Se conservan fallos, no se imputan ceros ni se recalcula
scoring. Cada run borra DB y arranca/termina proceso según runner leído;
no resetea necesariamente hf_hub, asset ni caches de SO. Efectos incluyen
modelo/dataset descargados, cómputo embedding, disco/traces, DB y dashboard.
**Nada se ejecutó**; no instrucciones para modificar Meilisearch.
Adaptación futura D/A: distinguir configuración aceptada, embedding aplicado
y proyección visible usando ingestors/receipts existentes de Vanta, conservar
fases y errores bajo METHOD; no nuevo harness/config/scoring/denominador.

`BenchWorkload:22–34` define name/run_count/assets/precommands/commands/target;
`execute_run:139–183` llama delete_db, arranca binario community o indicado,
ejecuta workload y termina proceso. Nada de ello se ejecutó aquí.
`movies.json:1–42` fija **run_count=10**, dataset remoto movies con SHA256,
PATCH settings searchable/filterable/sortable con DontWait y POST documentos
con WaitForTask. Unidad de ingesta: documento JSON; cantidad exacta y esquema
del asset no materializado no se inventan. No gold QA, answerer, juez o holdout.

`common/command.rs:93–166` agrupa comandos y su sincronización; WaitForTask
llama al polling de tareas enqueued/processing del servicio, **no prueba por
sí sola que terminaron con éxito**, ni espera sólo la tarea de esa petición.
En servicio compartido otras tareas contaminan espera/medición: aislamiento
del runner es parte de su contrato operativo, no detalle opcional.
`workload.rs::start_report:184–207` usa target por defecto `indexing::=trace`;
`stop_report:247–305` cierra captura, espera writer con timeout, convierte
traces vía `span_stats::to_call_stats`, sube al dashboard y escribe reporte [M4].
Estos son agregados de spans, no percentiles HTTP ni recall ANN/QA.
No se atribuyen fórmulas/percentiles del benchboard externo no leído.

Cada run reinicia DB/proceso; no asumir warmup ni cache fría de kernel/asset.
Errores de HTTP/trace/processor propagan `Result`; fallo antes de finalizar
puede impedir producir reporte, no equivale a dato ausente que se descarta del
denominador. Población declarada: runs solicitados frente a reportes completados
y span events recogidos; no convertir diez ejecuciones en diez preguntas QA.
Costes: build/binario, descargas/assets, disco DB/traces, CPU/RAM del servicio
y red de dashboard. Movies no acredita gasto LLM; otros workloads pueden.
Reset **destructivo**, arranque y terminación son efectos del runner leído,
no órdenes autorizadas en esta sesión. Retención de traces y credenciales de
dashboard requieren revisión; no se auditó cleanup excepcional integral.

Adaptación Vanta futura: mapear documento y primary key al ingestor existente,
settings a capacidades declaradas (sin fingir equivalencia de ranking), y
WaitForTask a outcome de publicación A. Conservar errores/runs y medir motor
separado de retrieval/agente según [METHOD](../agent-memory-2026-11-07/METHOD.md) y
[BENCHMARK-NOTES](../agent-memory-2026-11-07/BENCHMARK-NOTES.md). No crear runner, dataset, scoring,
prompts, thresholds ni configuraciones nuevos durante esta autoría.

### 12.3. Matriz por capacidad y negativos propios

| Capacidad | Upstream observado | Vanta pin 97c9623: estado/fuente | Delta y decisión |
|---|---|---|---|
| Retrieval léxico/vector | E3 pondera scores, early semantic omit | Existe `src/sdk/search/{hybrid,fusion}.rs`, HNSW/BM25/RRF | B explica, no copiar ratios |
| Filtro | E5 filterable y profundidad | Existe vector/payload index, `src/sdk/search/vector.rs`, backend | C declara error/capacidad, no autorización por metadata |
| Provenance/sesiones | Documento/tarea no sesión autobiográfica | Existe `MemoryRecord` y sesiones; `core/record/{l1_dedup,lifecycle}.rs` | No sustituir dedup/supersession |
| Budgets/cuotas | Deadline search/limits, no budget de agente equivalente | Existen `assemble_with_recall`, `apply_recall_budget`, `src/config.rs:760–762` | C7 mantiene unidades y ruta real de recall |
| Ingestors/scheduler/checkpoint | Cola/tareas/batches/progress E1–E6; overlay processing-en-RAM E10/E11, re-encolado tras aborto E10 | Existen worker/callback/TaskCheckpointManager | A/C9 amplía contrato, no scheduler nuevo |
| Batching/agrupación | Autobatching por índice con límites y `BatchStopReason` E12 | No aplicable: Vanta no agrupa mutaciones ajenas en un commit compartido | No trasladar; documenta que conteos por batch ≠ conteos por ítem |
| Snapshot/dump | Copia cruda de envs vs exportación lógica, ambas como tareas E13 | Checkpoints/auditoría existen; recibo de mantenimiento no uniforme | A/C9/C16: resultado observable de operaciones de mantenimiento |
| Ranking rules | Criterios lexicográficos configurables E15 | RRF/scores existen; no hay cadena de criterios | No trasladar (§5); E4 sigue siendo la evidencia de explicación |
| Búsqueda federada | Existe E14: weight por query, merge por score values, `remote_errors` parciales, remoto=EE | No existe fan-out multi-backend en Vanta | Hecho documentado; sólo converge el patrón errores-parciales-declarados, no se propone |
| Evidencia de escritura | Queue/index commits diferentes | Existe WriteReceipt MV1; cobertura integral de proyecciones parcial | D/C14 no otra WAL ni firma inventada |
| Settings/reindex | SettingsUpdate M3; workloads ME2A/ME2B separan añadir embedder de insertar con él | Config/ingestors/auditoría existen; manifest integral no acreditado | D/C4 liga revisión/fingerprint/visibilidad; modelo efectivo/caché no inferidos |
| UI | Mini-dashboard empaquetado y cliente externo | Studio existe MS1; jobs no acreditados al pin | C16 propone fases/resultado en diagnóstico existente |

Negativos Vanta: **ningún bug nuevo reproducido**. No acreditados end-to-end:
uniformidad de outcome/visibilidad en todos los bindings y reporte de brazos
omitidos por todas las rutas. No se declara ausencia global. Se descartan
como falsos positivos “sin híbrido”, “sin budget/quotas”, “sin receipt” y
“sin scheduler/Studio”; auditorías históricas no se reetiquetan como actuales.
Prioridad: A cuando se amplía ingesta; B/C pueden evolucionar aparte; D sólo
si la configuración cambia derivaciones. No bloquear toda mejora por una WAL
hipotética ni aprobar implementaciones desde una comparación documental.

### 12.4. Dashboard real y Studio: código, no producto inventado

Servidor: `lib.rs::dashboard:822–854` registra bytes MIME de `generated.rs`
en `/` y assets sólo con feature/enable_frontend. `build.rs:19–76` descarga
zip, compara SHA1 y genera recursos; **no se ejecutó**. Metadata
`Cargo.toml:170–172` fija mini-dashboard v0.4.2 y SHA1
`6a8a76d9ed79357959bc6d4e3816f753a605a27b`, no un SHA de commit de frontend.
No se accedió a ese release tras la instrucción de forge; la tercera pasada
de 2026-11-20 tampoco lo inspeccionó (fuera del alcance de issues/PRs/
releases del motor).

Lectura complementaria **de código público**, no forge:
[package.json mini-dashboard v0.4.2][MD1] y [src/App.jsx][MD2] vía CDN jsDelivr.
Declara React 18, Vite, SDK `meilisearch` 0.57.0,
instant-meilisearch 0.31.3 e InstantSearch; **no shell desktop**.
`App` construye ambos clientes con baseUrl/apiKey, comprueba `isHealthy`,
consulta índices con stats, elige índice y maneja modal de API key.
API key permanece en memoria y elimina legado localStorage; currentIndex sí
se persiste. También acepta `api_key` en URL: memoria no elimina riesgos de
historial/referrer. No afirmar que la UI controla permisos del backend.
La fuente se leyó a tag **v0.4.2**, no SHA inmutable resuelto; no se certifica
que el zip generado corresponda byte a byte al árbol CDN ni su licencia
per-file/dependencias. `src/App.js` dio 404; listado CDN `/src/` permitió
resolver App.jsx; API de listado de jsDelivr dio 503. No se ocultaron fallbacks.
No se inspeccionaron todos los componentes de búsqueda/task/settings; no
inventar dashboard de gestión completa a partir del launcher.

Studio: cuerpo releído con `git show` en pin separado
**`0ad38c542b39a589035b0e17fdedb4c09dca24b9`**, ruta
`feature-freez/desktop/src/components/memory/MemoryLens.tsx` [MS1].
`ScenesPanel:57-96`, `PersonaPanel:172-206`, `SkillsPanel:266-297` y
`GenlogPanel/openAnchor:373-412` cargan escenas/snapshot/skills/genlog y
abren registro mediante `get(anchorId)`. **ME-1:** JobChip/jobs no existen
en ese árbol; son `??` bajo `desktop/` en disco. No se acreditan cuerpos dirty
ni se atribuye a ese SHA cancel/repoll/IDs parciales; MS2/MS3 retirados.
C16 **propone** llevar A/B/D a Studio, separando batch, tarea, documento y
revisión. FMEA: task succeeded o cancel visual aparentan publicación/undo →
mostrar outcome desde owner. Aceptación futura: cancel antes/después de apply,
error parcial y restart no ocultan IDs aplicados ni declaran visible lo pendiente.
No se certifica UI shipped de jobs, su persistencia ni render; no otro runtime.

**Fuentes/cobertura/cierre:** E1–E9, §0, M1–M4, licencias raíz/EE, cliente
mini-dashboard focal y seams Vanta/Studio; segunda lectura E10–E15 sobre el
mismo pin: bucle tick/overlay processing (E10), almacenamiento de cola (E11),
autobatching (E12), snapshot/dump (E13), federación (E14) y ranking rules
(E15). Tercera pasada 2026-11-20: cuerpos de issues/PRs/releases de la forge
pública (§12.1). Sin recuperación integral tras crash entre commits —#6743
confirma la ventana, no su cierre—, todos los kernels/providers, autorización
end-to-end, componente EE ni SBOM.
Frontend a tag no SHA, discussions vistas sólo en listado y Actions/CI no
inspeccionado son límites reales que la revisión debe conservar. Si pasa a
specs (>300 líneas), separar A lifecycle/receipts, B/C retrieval y D
generation/UI. Este informe conserva los diez ejes, sin autoaceptación ni
specs/harness creados.
ME-1/ME-2 corregidos documentalmente; eje de incidencias **acreditado
2026-11-20 tras levantar la restricción**, sin cierre ficticio de cobertura:
los límites residuales se enumeran arriba, no se ocultan.

[ME2A]: https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/workloads/embeddings-settings-add.json#L1-L65
[ME2B]: https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/workloads/embeddings-movies-subset-hf.json#L1-L61
[ME2R]: https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/crates/xtask/src/bench/workload.rs#L40-L183
[ME2H]: https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/crates/milli/src/vector/embedder/hf.rs#L60-L270
[ME2S]: https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/crates/milli/src/vector/settings.rs#L2126-L2150

[M1]: https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/Cargo.toml#L1-L40
[M2]: https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/crates/index-scheduler/src/scheduler/process_batch.rs#L1234-L1295
[M3]: https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/crates/meilisearch/src/routes/indexes/settings.rs#L641-L698
[M4]: https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/crates/xtask/src/bench/workload.rs#L247-L305
[MV1]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/attestation.rs#L654-L738
[MS1]: https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/memory/MemoryLens.tsx#L57-L412
[MD1]: https://cdn.jsdelivr.net/gh/meilisearch/mini-dashboard@v0.4.2/package.json
[MD2]: https://cdn.jsdelivr.net/gh/meilisearch/mini-dashboard@v0.4.2/src/App.jsx

## Apéndice — Incógnitas, lagunas de evidencia y falsos positivos descartados

> Consolidación transversal del patrón «Apéndice E» del estudio OpenViking.
> Esta sección no añade análisis nuevo: reúne lo que el propio informe ya
> declara como no acreditado o pendiente y los callejones ya descartados,
> para que una re-auditoría no repita trabajo. Las secciones originales
> citadas siguen siendo la fuente detallada.

### Incógnitas y lagunas de evidencia

- No se acredita qué descargas acertaron caché ni cache fría de kernel/modelo.
- Null/ausencia no deben publicarse como identidad efectiva sin verificar resolución de settings, estado persistido y bytes del modelo.
- No acreditados end-to-end: uniformidad de outcome/visibilidad en todos los bindings y reporte de brazos omitidos por todas las rutas.
- `build.rs:19–76` descarga zip, compara SHA1 y genera recursos; **no se ejecutó**.
- No se acreditan cuerpos dirty ni se atribuye a ese SHA cancel/repoll/IDs parciales; MS2/MS3 retirados.

### Falsos positivos y descartes registrados

- No se registraron falsos positivos formales en el cierre de este informe.

### Limitaciones declaradas en este informe

- § 3. Riesgos, issues y límites de seguridad
- § 5. Qué no trasladar
- § 6. Método, licencia y cobertura
- § 11. Cierre contractual: estados, visibilidad y settings

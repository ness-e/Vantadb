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
Se aplican [EXPANSION-CONTRACT](EXPANSION-CONTRACT.md),
[VANTA-REFERENCE](VANTA-REFERENCE.md), [METHOD](METHOD.md) y
[BENCHMARK-NOTES](BENCHMARK-NOTES.md), sin cambiar sus protocolos.

| Frontera | Fuente y significado |
|---|---|
| Identidad del clon | `C:\Users\Eros\meilisearch`, `main`, `c18c81973ad5d3e126f4583cdd16a9ec1719aa4a`, HEAD releído el 2026-11-07 |
| Captura preservada | [clone-manifest.json](clone-manifest.json), 2026-11-07T06:56:19.7819824Z; shallow, sin actualizar inventarios |
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
documentos no son contadores intercambiables. Recovery integral, retención de
payloads y propagación de cancelación tras proxy quedan sin auditar.

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
[Snapshot](github-candidates.json): **59 806 estrellas; licencia NOASSERTION**.
[Manifiesto](clone-manifest.json) conserva las omisiones del clon.

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

### Ficha candidata A — recibo durable de tarea y resultado no ambiguo

- **Objetivo:** que aceptar un trabajo de ingesta no se confunda con completar
  indexación; mantener diagnóstico de fallo/cancelación.
- **Unidad mínima / fuente:** vocabulario E1 y orden commit→notify E2;
  separación de estadísticas no críticas E6.
- **Seam Vanta existente:** scheduler/ingestors/checkpoints y API de actividad
  ya acreditados en [METHOD](METHOD.md#vantadb-como-referencia);
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
**Issues/PR/discussions/releases upstream:** la primera pasada no los consultó.
§12 registra exactamente el listado visto en esta ampliación y el bloqueo
posterior por instrucción explícita; no se presenta un listado como lectura
de cuerpos ni se propone/modifica contenido upstream.

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

## 6. Método, licencia y cobertura

Lectura estática E1-E9, main/config, LICENSE, LICENSE-EE y workloads;
§11–§12 amplían settings, tareas/batches, runner y frontend por CDN.
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
[METHOD](METHOD.md#protocolo-de-comparacion-de-benchmarks), sin nuevos ajustes,
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

### 11.2. A — recibo/resultado de tarea: once campos completados

1. **Caso agente:** se captura una memoria/documento y el caller necesita saber
   si puede recuperarla, si falló o si pidió cancelar después de aplicarse.
2. **Fuente/unidad:** E1/E2/E6 y M2: estados, commit→notify, batch/progress,
   commit de índice separado de stats; no portar scheduler LMDB.
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

### 12.1. Incidencias: acceso limitado, no lectura ficticia

Consulta 2026-11-07: `websearch.search` DDG/Bing
`site:github.com/meilisearch/meilisearch/issues task cancel settings batch`
devolvió vacío/Bing 301; `fetchWebContent` sí leyó el **listado** oficial.
Después se recibió como instrucción activa `C:\Users\Eros\meilisearch\AGENTS.md`,
que prohíbe acceder a features de forge; **no se abrieron cuerpos, PRs,
discussions, releases ni logs posteriormente**. La investigación sigue siendo
un informe de Vanta; no propone modificar Meilisearch ni escribe en su clon.

El listado mostró títulos #6696 (prefix tras delete/update), #6691 (remove
embedder), #6665 (snapshot S3 bloqueado) y #6717 (settings/prefix), todos en la
vista Open. **No son incidencias leídas**: causa, revisión afectada, corrección
y vigencia no acreditadas; no se convierten en bugs actuales ni en evidencia
de una brecha Vanta. Se conserva la URL del listado
<https://github.com/meilisearch/meilisearch/issues> como alcance observado,
no como sustituto de cada cuerpo. Eje de incidencias públicas **no acreditado
por prohibición expresa**, no “completo por tener enlaces”. En esta corrección
no se accedió a issues/PRs/discussions/Actions/releases ni se buscó fallback,
mirror o bypass. No se propone ningún patch al clon upstream.

Hallazgos estáticos sí acreditados: E2/E6 separan commit/notify/stats;
M3 encola settings antes de su aplicación; E3 permite omitir semantic.
Ninguno demuestra crash corruption ni que una tarea Succeeded siempre mienta.
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
separado de retrieval/agente según [METHOD](METHOD.md) y
[BENCHMARK-NOTES](BENCHMARK-NOTES.md). No crear runner, dataset, scoring,
prompts, thresholds ni configuraciones nuevos durante esta autoría.

### 12.3. Matriz por capacidad y negativos propios

| Capacidad | Upstream observado | Vanta pin 97c9623: estado/fuente | Delta y decisión |
|---|---|---|---|
| Retrieval léxico/vector | E3 pondera scores, early semantic omit | Existe `src/sdk/search/{hybrid,fusion}.rs`, HNSW/BM25/RRF | B explica, no copiar ratios |
| Filtro | E5 filterable y profundidad | Existe vector/payload index, `src/sdk/search/vector.rs`, backend | C declara error/capacidad, no autorización por metadata |
| Provenance/sesiones | Documento/tarea no sesión autobiográfica | Existe `MemoryRecord` y sesiones; `core/record/{l1_dedup,lifecycle}.rs` | No sustituir dedup/supersession |
| Budgets/cuotas | Deadline search/limits, no budget de agente equivalente | Existen `assemble_with_recall`, `apply_recall_budget`, `src/config.rs:760–762` | C7 mantiene unidades y ruta real de recall |
| Ingestors/scheduler/checkpoint | Cola/tareas/batches/progress E1–E6 | Existen worker/callback/TaskCheckpointManager | A/C9 amplía contrato, no scheduler nuevo |
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
No se accedió a ese release tras la instrucción de forge.

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
mini-dashboard focal y seams Vanta/Studio. Sin recuperación integral,
todos los kernels/providers, autorización end-to-end, componente EE ni SBOM.
Issues no acreditados por prohibición y frontend a tag no SHA son límites reales
que la revisión debe conservar. Si pasa a specs (>300 líneas), separar A
lifecycle/receipts, B/C retrieval y D generation/UI. Este informe conserva
los diez ejes, sin autoaceptación ni specs/backlog/harness creados.
ME-1/ME-2 corregidos documentalmente; eje de incidencias sigue no acreditado
por prohibición, no un cierre ficticio de cobertura ni permiso para bypass.

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

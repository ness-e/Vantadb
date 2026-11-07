---
title: "Meilisearch: tareas visibles, filtros y explicación híbrida"
kind: research
status: proposed
description: "Contraste estático de contratos de ingesta y ranking sin proponer cambios al repositorio upstream"
tags: [research, memory, engineering]
---

# Meilisearch: tareas visibles, filtros y explicación híbrida

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
**Issues/PR/discussions/releases upstream:** **no consultados**, respetando
`AGENTS.md` del repositorio; no se propone ni modifica contenido upstream.

**Seguridad/configuración:** E7 permite master-key opcional en el tipo, pero
[`crates/meilisearch/src/main.rs:107-124`](https://github.com/meilisearch/meilisearch/blob/c18c81973ad5d3e126f4583cdd16a9ec1719aa4a/crates/meilisearch/src/main.rs#L107-L124)
rechaza producción sin clave o con clave demasiado corta. No inferir «inseguro
por defecto» sólo desde Option. Frontera: API red y endpoints de diagnóstico;
env/deploy real, autorización fina, webhooks y SSRF no auditados.
No se cambiaron configuraciones ni se inspeccionaron forges externas.

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

Sólo lectura estática E1-E9, main/config, LICENSE, LICENSE-EE y workloads.
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

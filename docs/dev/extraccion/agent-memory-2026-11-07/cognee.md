---
title: "Cognee: pipelines de conocimiento, procedencia y aislamiento por dataset"
kind: research
status: proposed
description: "Analisis estatico de cognify, memify, retrieval y borrado de Cognee con propuestas incrementales para VantaDB"
tags: [research, memory, engineering]
---

# Cognee: pipelines de conocimiento, procedencia y aislamiento por dataset

**Contrato:** [METHOD.md](METHOD.md). Solo lectura estatica y documentacion:
sin importar el paquete, instalar, ejecutar tests, modelos, servicios o benchmarks.
Las fichas son candidatas, no arquitectura aprobada.

| Identidad congelada | Valor |
|---|---|
| `full_name` | `topoteretes/cognee` |
| Estrellas | **33118**, [snapshot](github-candidates.json), no recalculadas |
| Captura | `2026-11-07T06:56:01.1946268Z` |
| SHA local, `git rev-parse HEAD` | `8853378eae5366addb064e758e560a456146ae1a` |
| Clon | `C:\Users\Eros\cognee`; status corto sin cambios observados |
| Licencia declarada | Apache-2.0; obligaciones de archivos y dependencias por revisar antes de adopcion |

## Indice

- [1. Recomendaciones y arquitectura](#1-recomendaciones-y-arquitectura)
- [2. Partes extraibles y flujo real](#2-partes-extraibles-y-flujo-real)
- [3. Riesgos e incidencias](#3-riesgos-e-incidencias)
- [4. Prerrequisitos en VantaDB](#4-prerrequisitos-en-vantadb)
- [5. Que no trasladar](#5-que-no-trasladar)
- [6. Advertencias y cobertura](#6-advertencias-y-cobertura)
- [7. Benchmarks existentes](#7-benchmarks-existentes)
- [8. Matriz frente a VantaDB](#8-matriz-frente-a-vantadb)
- [9. Debilidades sugeridas, no bugs atribuidos](#9-debilidades-sugeridas-no-bugs-atribuidos)
- [10. Interfaz e inspeccion](#10-interfaz-e-inspeccion)

**Plan de particion:** si evoluciona este estudio largo, separar ledger y fichas
en una tarea autorizada, conservando aqui el dictamen y los diez ejes.
No se crean ahora documentos adicionales.

## 1. Recomendaciones y arquitectura

**Dictamen:** el aporte reutilizable es el contrato entre documento original,
chunk recuperable, hecho derivado, propietarios/procedencia y estado de tarea.
No es una razon para incorporar toda la plataforma Python a un motor embebido.
VantaDB ya posee ingestors, sesiones, dedup, supersession, scheduler, checkpoints,
auditorias e indice hibrido; el contraste debe mejorar esos seams, no duplicarlos.

```text
cognee
├── api/v1/
│   ├── remember / recall / improve / forget    superficie memoria
│   ├── add / cognify / search / update         etapas de bajo nivel
│   └── datasets / users / permissions          autorizacion
├── modules/pipelines/                         tareas, ejecucion y runs
├── tasks/
│   ├── ingestion                              originales, hash y metadatos
│   ├── graph                                  extraccion + resumen
│   └── storage                                grafo antes de vectores
├── infrastructure/databases/
│   ├── relational                             usuarios, ACL, datasets, runs
│   ├── graph                                  entidades, aristas, referencias
│   └── vector                                 chunks, entidades y relaciones
├── modules/retrieval/                         estrategias/contexto/completion
├── eval_framework/                            LoCoMo y BEAM actuales
└── frontend, MCP, connectors                  capas opcionales separadas
```

**Stack confirmado:** paquete `cognee` 1.6.7, Python >=3.10,<3.15; FastAPI,
SQLAlchemy/Alembic, aiosqlite, LiteLLM, OpenAI, Pydantic, LanceDB/pylance,
Ladybug, fastembed/onnxruntime, dlt/pandas y Enola son parte del conjunto base:
[pyproject.toml:1-140](https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/pyproject.toml#L1-L140).
Neo4j, Postgres/pgvector, Turso y adaptadores comunitarios no equivalen a
requisitos simultaneos. SQLite es registro relacional compartido por defecto;
el aislamiento grafo/vector depende del handler del dataset, no de convertir
cada tabla SQL de usuarios en una base independiente.
No se midio huella instalada ni costo operativo.

### Licencias y componentes, no una etiqueta para todo el despliegue

| Superficie | Fuente en el SHA fijado | Alcance acreditado |
|---|---|---|
| Núcleo Python/API y materiales del monorepo | [LICENSE Apache-2.0][C-license] y [NOTICE.md][C-notice] | NOTICE identifica Topoteretes UG, copyright 2024; conservar atribuciones aplicables si se adopta material |
| `cognee-frontend` | [package.json][C-ui-package], paquete privado; LICENSE raíz | Fuente web presente; no presupone licencia de un servicio cloud/pod ni derechos sobre datos mostrados |
| Core de conectores | [pyproject.toml][C-connectors-package] y LICENSE propio | Declara Apache-2.0 y `license-files`; no concede scopes OAuth de cada servicio |
| Conector GitHub, muestra de componente separado | [LICENSE propio Apache-2.0][C-connector-license] | Su código operativo no fue auditado; no extender esta lectura a todos los conectores |
| Modelos, DBs, bibliotecas, datasets LoCoMo/BEAM | Dependencias y fuentes identificadas en §1/§7 | Términos de pesos, datos y cada backend por revisar; no heredan licencia del wrapper |

El inventario de licencias encuentra archivos separados en conectores Apollo,
Asana, Gmail, Drive, Calendar, Linear, Notion, Obsidian, OneNote y Slack.
Inventariar no equivale a leer cada licencia ni a un SBOM completo.
Estudiar contratos y proponer Rust no exime de revisar derivación y notices.

### Continuación, corte propio y decisiones

Se conserva la investigación anterior y se completa por lectura estática el
2026-11-07. El clon sigue en `8853378eae5366addb064e758e560a456146ae1a`
(`git rev-parse HEAD`); no se hizo fetch ni se materializaron dependencias.
La comparación es con **VantaDB `97c9623c8ef5a70fdb51aff215bfbb3b05b15612`**,
según [contrato](EXPANSION-CONTRACT.md) y [referencia](VANTA-REFERENCE.md).
METHOD conserva antecedentes históricos: no usar su antiguo dirty como corte.

| Prioridad razonada | Unidad | Condición para adoptarla |
|---|---|---|
| Primera, si se amplía ingesta | A: hitos por revisión | No anunciar grafo listo porque se guardó el original |
| Primera, antes de derivados compartidos | B: ownership y recibo | Desenlazar una fuente no equivale a destruir el hecho |
| Primera, al tocar jobs | C/D: resultado de etapas y orden de locks | No convertir fallos parciales en éxito ni abarcar LLM con locks innecesarios |
| Condicional a documentos mutables | E: diff validado y publicación | Mantener identidad, explicar rebuild y no prometer atomicidad entre stores |
| Transversal | F: contexto autorizado y capacidad del backend | La configuración de una conexión no concede permisos |
| Posterior, opt-in | G: promoción de sesiones | Cursor por consumidor y fuente, no aprendizaje automático irrestricto |
| Lectura, independiente de escritura | H: canales y referencias | Extender diagnóstico/context assembly, no reemplazar BM25/HNSW/RRF |

### Modelo e identidades que explican los flujos

| Recurso | Código leído en el SHA Cognee | Semántica y frontera |
|---|---|---|
| `Dataset` | [Dataset.py:9-38][C-dataset] | UUID, owner/tenant, ACL y configuración; relación Data de lectura sobre `dataset_id` |
| `Data` | [Data.py:10-75][C-data] | UUID estable, rutas original/procesado, hashes, metadata externa/sistema, estado por pipeline; hash es lookup, no identidad |
| Alias histórico | `Data.legacy_id:52-57` | Conserva ID anterior tras partir una fila compartida; no cadena arbitraria de aliases |
| `DocumentChunk` | [DocumentChunk.py:34-86][C-chunk] | Texto, posición, hash, presupuesto y chunker, documento, fuentes y metadata permitida; mover posición no cambia necesariamente identidad |
| Nodos/aristas derivados | [add_data_points.py:74-143,307-363][C-store] | Dedup del batch y anexión de referencias; una arista puede tener más de una procedencia |
| Run/operación | [improve.py:219-284][C-improve] | Resultado de etapa y sello persistido; un proceso que termina sin excepción aún puede tener estado failed/noop |
| Sesión/Q&A/trazas | [improve.py:152-206][C-stages] | Entrada para promoción; watermark Q&A/trazas por usuario+sesión, distillation por dataset |

No hay equivalencia automática `Data = memoria L1 = checkpoint`. El documento
es fuente editable; el chunk, unidad recuperable; la entidad/arista, derivado;
la sesión, contexto conversacional; el run, estado operativo. Los originals y
procesados pueden estar en archivos/objetos mientras relaciones y ACL viven en
SQL y búsqueda en grafo/vector. Confirmar uno no confirma los demás.
`system_metadata` se declara no editable por usuario; `external_metadata` sí.
Ningún hash textual demuestra autorización ni exactly-once.

**Despliegue y confianza:** biblioteca async en proceso, API FastAPI, MCP,
CLI y frontend son superficies diferentes. Los handlers deciden la separación
grafo/vector; el registro SQL de usuarios/ACL no se vuelve privado por dataset.
El remoto aplica su configuración: `update.py:180-194` advierte que ciertos
overrides locales no tienen campo HTTP y no viajan. Ni firma común ni
`ContextVar` garantizan paridad de backend o aislamiento entre procesos.
La extracción puede enviar originales al proveedor; documentos, respuestas y
feedback son datos hostiles, nunca autoridad para modificar permisos.

## 2. Partes extraibles y flujo real

### Ingesta y extraccion

`add` ejecuta migraciones, resuelve usuario/dataset con permiso write,
normaliza fuentes DLT y rechaza un documento existente cambiado antes de escribir
el batch; para eso existe `update`. Materializa streams de trabajos background
y pospone o omite limpieza de filas DLT hasta saber si reemplazos se guardaron:
[add.py:313-408](https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee/api/v1/add/add.py#L313-L408).
`ingest_data` guarda archivos, calcula hash, busca contenido en scope
dataset+owner+tenant y deduplica dentro del batch; IDs pueden ser explicitamente
fijados o nuevos UUID:
[ingest_data.py:350-398](https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee/tasks/ingestion/ingest_data.py#L350-L398).
La fila relacional se confirma antes de limpiar archivos reemplazados.

`cognify` tiene dos hitos reales: indexacion temprana sin LLM que hace chunks
buscables, y despues extraccion/enriquecimiento del grafo. Enruta explicitamente
STANDARD, DLT_SOURCE, CODE y CODE_REPO; no todo input debe pasar por LLM:
[cognify.py:543-620](https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee/api/v1/cognify/cognify.py#L543-L620).
Las tareas standard chunkifican, encuentran chunks ya indexados,
`extract_graph_and_summarize`, `add_data_points` y, si esta habilitado,
`record_provenance`:
[cognify.py:677-767](https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee/api/v1/cognify/cognify.py#L677-L767).

**LLM opcional requiere matiz:** indexacion/ingesta y rutas deterministas no
lo necesitan; `graph_extractor=auto` elige LLM con credencial util o GLiNER demo
sin ella. Este ultimo no es generacion LLM remota, pero usa modelos y puede
**instalar torch/gliner2 al primer uso**. `GLINER_AUTO_INSTALL=false` exige
preaprovisionar el runtime. Resumen por extraccion es distinto de resumen LLM:
[config.py:15-50](https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee/modules/cognify/config.py#L15-L50).
No se ejecuto esa ruta ni se descargaron modelos.

### Dedup, persistencia, cambios y borrado

`add_data_points` transforma modelos en nodos/aristas deduplicados en el batch,
adjunta fuentes adicionales a aristas ya existentes sin reescribir sus pesos,
y escribe **grafo antes que vectores** para que rollback/borrado puedan localizar
el artefacto mediante referencias:
[add_data_points.py:74-143](https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee/tasks/storage/add_data_points.py#L74-L143),
[307-363](https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee/tasks/storage/add_data_points.py#L307-L363).
Es ordenacion y recuperabilidad, no atomicidad distribuida demostrada.

`update` verifica write y existencia, conserva `data_id`, intenta diff por chunks
y devuelve estado/contadores/causa estructurada si necesita rebuild:
[update.py:219-304](https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee/api/v1/update/update.py#L219-L304).
No se acredita historial bitemporal universal de cada hecho por existir runs.
El ledger de procedencia audit-grade es opt-in y distinto de ownership
necesario para borrar artefactos compartidos.

`datasets.delete_data` autoriza delete, toma lock, resuelve ID dentro del dataset,
borra por ownership incluso en modelos custom sin fila Data y devuelve recibo
`success`/`not_found`, conteos y comprobacion de fila restante:
[datasets.py:457-575](https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee/api/v1/datasets/datasets.py#L457-L575).
La invalidacion de sesiones es best-effort; retiro de decisiones sobre hechos
borrados tiene semantica distinta y puede hacer fallar la operacion.
`forget(memory_only=True)` borra derivados conservando originales; no equivale
a purga fisica de archivos de almacenamiento.

### Memify no es el pipeline default vigente

`memify` esta deprecado: triplet embeddings se retiraron; sin tareas verifica
dataset y **no hace enriquecimiento**, devuelve `{}`. Tareas nombradas siguen
via runner legacy; las alternativas son `improve` y `run_custom_pipeline`:
[memify.py:149-250](https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee/api/v1/memify/memify.py#L149-L250).
`improve` orquesta etapas con `StageResult`, watermarks, errores parciales,
etapa fatal de persistencia QA y corte al agotar presupuesto:
[improve.py:1-23](https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee/api/v1/improve/improve.py#L1-L23).
Su review de conflictos se omite sin LLM; indice de contexto global es otra etapa.
No se asume que toda funcion historicamente llamada memify siga activa.

### Recuperacion, contexto y autorizacion

`authorized_search` resuelve datasets con read antes de routing/fan-out:
[search.py:220-288](https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee/modules/search/methods/search.py#L220-L288).
Hybrid recupera en paralelo chunks/summaries y entidades/hechos, proyecta
metadata permitida y adjunta conflictos antes de finalizar:
[hybrid_retriever.py:138-208](https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee/modules/retrieval/hybrid_retriever.py#L138-L208).
`get_context_from_objects` formatea contexto hibrido y global; `only_context`
evita confundir contexto obtenido con respuesta generada. Otros tipos abarcan
chunk, lexical, graph completion, codigo, temporal y agentic; no se auditaron
todos. Top-k por canal/dataset no prueba presupuesto global equivalente al de
`context_assemble` VantaDB.

La autorizacion de datos y el selector de almacenamiento son separados:
`ContextVar` mantiene configuracion por tarea; `backend_access_control_enabled`
valida compatibilidad de ambos handlers y explicitar false desactiva ese modo:
[context_global_variables.py:95-125](https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee/context_global_variables.py#L95-L125).
Matriz real Ladybug/Kuzu, Neo4j, LanceDB, pgvector, Turso y variantes de schema:
[supported_dataset_database_handlers.py:35-85](https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee/infrastructure/databases/dataset_database_handler/supported_dataset_database_handlers.py#L35-L85).
Un adaptador externo no obtiene aislamiento por implementar solo search/add.

### Ficha candidata A: disponibilidad progresiva verificable

**Problema y unidad mínima.** Un agente sube un manual y pregunta antes de que
termine su enriquecimiento. `get_indexing_tasks/get_default_tasks`,
[cognify.py:677-767][C-ready], separan índice temprano y extracción de grafo.
Hay estados utilizables intermedios, no una operación indivisible “cognify”.

**Delta y alternativas.** Vanta ya tiene ingesta, workers y checkpoints.
Extender su resultado por revisión cuesta menos que otro pipeline; esperar a
todo simplifica consistencia, pero oculta chunks disponibles. Servir todo sin
indicar generación arriesga mezclar revisiones. No se afirma que falten todos
los estados propios: [worker Vanta][CV-worker] y [checkpoint de tarea][CV-task]
son los seams a comparar.

**Contrato Rust propuesto, no existente:** `MemoryReadiness { source_id,
source_revision, index_generation, chunk_state, enrichment_state, failures }`.
Entrada: documento autorizado y revisión esperada. Salida: hitos confirmados y
trabajo pendiente; `StaleRevision`, `IndexUnavailable`, `Cancelled` y fallo de
proveedor no son “sin contenido”. Cada hito pertenece a una misma revisión;
“stored” no implica “searchable” ni “enriched”.

**Pasos y commit.** Inventariar estados de `ingest/worker.rs` y
`MemoryTaskHandler`; asociar los hitos al checkpoint existente; publicar índice
temprano solo tras confirmación del primario; enriquecer fuera de la mutación
crítica; confirmar generación y resultado; mostrarlo en actividad existente.
El primario es autoridad; proyecciones quedan pendientes si fallan. Migración
aditiva de estado, con legado “desconocido”, no ready por defecto. No reembebido
obligatorio al completar grafo; cambiar chunker/extractor exige generación nueva.
Rollback desactiva el productor nuevo, no elimina originales ya confirmados.

**Concurrencia/cancelación y dependencias.** B protege ownership; C registra
etapas; D controla publicación concurrente. Dos jobs no promocionan una revisión
antigua sobre la nueva. Cancelar entre hitos preserva lo confirmado y deja
reanudación identificada, no revierte por ficción una escritura consumida.

**FMEA.** Hito anticipado → consulta vacía presentada como ready; detectar
generación no visible y confirmar antes de anunciar. Revisión mezclada →
respuesta obsoleta; comparar revisión de fuente/índice. Reintento de grafo →
embeddings duplicados; reutilizar identidad y recibo de etapa.
**Aceptación manual futura:** detener enriquecimiento y recuperar chunks de la
revisión confirmada con estado parcial; reanudar sin duplicar; editar/borrar la
fuente mientras trabaja y rechazar el resultado viejo. Telemetría de hito,
revisión, conteos y error, sin texto. **Mínimo/no objetivos:** readiness por
documento; no importar runner Python, inventar SLA ni prometer menor latencia.

### Ficha candidata B: procedencia con multiples propietarios

**Problema y lectura.** Dos documentos respaldan una relación; borrar el primero
no debe eliminar el conocimiento del segundo. `add_data_points` adjunta fuentes
a aristas ya vistas y ordena grafo antes que vector [C-store].
`datasets.delete_data` autoriza, bloquea, resuelve ID, desenlaza ownership y
devuelve recibo [C-delete]. Su ledger audit-grade opt-in es distinto del
ownership necesario para eliminar. El custom graph no exige siempre fila Data.

**Delta/alternativas.** [MemoryRecord][CV-record] ya conserva fuentes y versión.
La propuesta añade semántica de conjunto de respaldos y “último propietario”,
no procedencia desde cero. Recontar fuentes al borrar evita contador derivado
inconsistente; un refcount es rápido pero por sí solo no permite reparación.
Otra opción es no compartir derivados: más almacenamiento, menos coordinación.

**Contrato Rust propuesto:** `SourceRef { scope, source_id, source_revision,
chunk_id }`, `DerivedOwners` y `SourceRemovalReceipt { detached, removed,
preserved_shared, pending_projections, verification }`. Entrada: intención
autorizada, revisión e ID de operación; salida por recurso, no éxito vacío.
Refs únicas; nunca compartir entre ámbitos incompatibles. `NotFound`,
`Conflict`, `ProjectionFailed` y `VerificationIncomplete` son distintos.

**Seam, orden y persistencia.** Extender `plan_write`/`persist_planned` y
`apply_dedup_batch` [CV-write], lifecycle y auditoría, sin sidecar opaco paralelo.
Primero inventario de cardinalidad existente; luego refs versionadas y preview;
quitar la ref; borrar el derivado solo cuando no queden respaldos válidos;
invalidar índices/caches y confirmar recibo. Primario+refs necesitan frontera
coherente; `put_batch` no hace atómico el delete anterior del writer propio.
Legado sin ownership se clasifica explícitamente; no inferir propietarios desde
texto. Reconstruir proyección por generación; rollback conserva lectura de refs.

**Concurrencia/cancelación y dependencias.** Una fuente añadida mientras se
elimina otra no puede perderse por RMW. Validar revisión al aplicar el plan;
retry idempotente por operación/ref; cancelación conserva pendientes de borrado.
Depende de F para alcance y D para exclusión real; no requiere grafo Python.
Auditoría puede conservar IDs/razones sin conservar el texto purgado.

**FMEA.** Refcount cero obsoleto → se destruye hecho compartido: detectar contra
refs del primario y usar mutación condicional. Índice huérfano → revive texto:
validar fuente viva al leer y reconciliar. Fallo tras detach → retry destructivo:
recibo por operación. **Aceptación manual:** dos fuentes y dos ámbitos; borrar
una, ambas, un ID custom/ausente; añadir otra concurrentemente; interrumpir
cada frontera. Recibo distingue aproximado/verificado y no deja la UI vender
purga completa. **No objetivos:** deducir ausencia de provenance Vanta, fijar
retención legal o certificar borrado de backups/soporte físico.

### Ficha candidata C: resultado de enriquecimiento por etapa

**Problema/fuente.** “El job terminó” no dice si feedback, Q&A, preferencias o
conflictos se publicaron. [execute_stages:219-284][C-improve] registra cada
etapa; aborta la fatal `persist_session_qa`, salvo error posterior del hook;
agota budget sin repetir llamadas condenadas y marca la operación failed aunque
el error no fatal no escape. Etapas sin LLM se omiten con causa [C-stages].

**Delta/alternativas.** [PipelineWorker::run_task:286-339][CV-worker] ya tiene
retries, leases y dead letters; el delta es detalle por etapa confirmada y
clasificación de errores. Una transacción de motor sirve para datos locales,
no cubre el proveedor LLM. No introducir otra cola ni una segunda política de
reintentos que compita con la actual.

**Contrato Rust propuesto:** `EnrichmentOutcome { operation_id, source_revision,
stages: Vec<StageOutcome>, resumable_from, usage }`; cada etapa distingue
Applied/Unchanged/Skipped/Failed/UnknownCommit y motivo. Input: plan autorizado,
checkpoint y efectos declarados. Fatalidad y posibilidad de retry pertenecen
a la etapa, no a una heurística sobre texto de excepción. Usage desconocido
no se reporta como cero; presupuesto monetario no es presupuesto de contexto.

**Integración/acciones.** Inventariar `TaskHandler::handle` y generation log;
mapear resultados existentes sin perder errores; persistir resultado tras cada
efecto confirmado; avanzar cursor solo con la confirmación correspondiente;
reanudar por operación. Migración compatible y versión del plan. Desactivar una
etapa deja sus derivados existentes identificables, no los borra por rollback.
No ampliar locks sobre HTTP/LLM para simular transacción.

**Concurrencia/cancelación.** Lease expirado no autoriza al worker viejo a seguir
publicando: validar propietario/revisión en commit. Cancelar deja las etapas
confirmadas y marca lo no intentado; timeout después de escribir requiere
reconciliación, no repetir todo. Depende de A/F y de la idempotencia de cada
efecto, no bloquea diagnóstico H. G estudia los cursores de consumo.

**FMEA.** Fallo no fatal contado exitoso → watermark engañoso: estado agregado
conservador y razón por etapa. Cursor antes del write → pérdida de Q&A: confirmar
juntos o recuperación explícita. 402 reintentado por cada etapa → gasto/ruido:
clasificar agotamiento sin confundirlo con dato vacío.
**Aceptación manual:** éxito, noop por falta de LLM, error fatal/no fatal,
timeout antes/después del commit, cancelación y lease vencido; ninguna etapa
confirmada se ejecuta dos veces sin semántica idempotente. Logs de fase/estado,
no conversación. **No objetivos:** copiar fail-open para datos críticos ni
rediseñar scheduler, budgets de evaluación o prompts.

### Ficha candidata D: orden de locks declarado como contrato

**Problema y unidad.** Un job retiene un permiso global limitado mientras espera
un dataset cuyo escritor espera ese permiso. [dataset_lock.py:1-133][C-lock]
declara orden **dataset → queue slot**, verifica inversión y explica el límite
in-process. `held_datasets` se hereda a tareas hijas; no prueba exclusión entre
hijos que ejecuten escrituras en paralelo.

**Delta/alternativas.** Vanta tiene locks de sesión, leases y scheduler
[CV-worker][CV-scheduler]; no necesita locks Cognee. La unidad transferible es
orden contractual y ownership de guard, no “poner un mutex global”.
Serializar todo simplifica pero bloquea scopes independientes; revisión
optimista reduce locks y exige detectar conflictos. Se decide por recurso.

**Contrato Rust propuesto:** `DatasetMutationGuard`/`ProjectionPermit` representan
guardias del sistema existente; inputs scope autorizado, propietario de tarea
y cancelación; output guard no transferible implícitamente a hijos. Errores
`LockOrderViolation`, `Cancelled`, `LeaseLost`, no un timeout convertido a noop.
El nombre es ilustrativo: Vanta no tiene por ello un Dataset de Cognee.

**Acciones/commit.** Dibujar orden real de sesión, storage, índice y recursos
externos; identificar reentrancia deliberada; adquirir en orden total; publicar
en sección corta y liberar en RAII. No conservar una transacción mientras se
espera LLM. Persistencia del plan pertenece a C, no al lock. Migración de datos
no necesaria; cambio de orden requiere despliegue coherente de los callers.
Rollback no puede mezclar órdenes viejos/nuevos en un mismo proceso.

**Concurrencia/cancelación/dependencias.** Cancelación antes de adquirir no tiene
efecto; después libera permisos y registra lo ya confirmado. Lease entre
procesos requiere fencing de publicación: un lock local no lo sustituye.
Depende del inventario de primitivas reales, no obliga a sustituir SDK/storage.

**FMEA.** Orden invertido → deadlock: detectar esperas/owner y fallo temprano.
Contexto heredado → falsa reentrancia: identidad de ejecución explícita.
Lease perdido → escritor zombie: validar fence al commit.
**Aceptación manual:** dos mutaciones y borrado del mismo scope más otro scope,
cancelación durante espera, tarea hija y lease expirado; progreso independiente
y ausencia de efectos después de perder autoridad. Observabilidad de wait/hold
por clase de recurso, sin claves sensibles.
**No objetivos:** locks distribuidos por analogía, garantías multiproceso desde
un mutex ni prometer fairness no acreditada.

### Ficha candidata E: actualización incremental que valida el plan completo

**Problema/lectura.** Editar una página de un manual no debería cambiar la
identidad del documento ni reinterpretar silenciosamente todos sus chunks.
[incremental.py:662-750,864-925,1063-1210][C-incremental] autoriza write+delete,
relee baseline dentro del lock, prepara contenido sin cambiar Data, valida que
el plan reconstruye el texto y escribe antes del publish SQL.
`update.py:219-349` preserva ID incluso al reconstruir y devuelve causa tipada
de fallback. No convierte permiso denegado en fallback.

**Matiz adversarial.** El docstring “fresh primero, nunca huecos” no cubre todo:
`1144-1150` elimina primero los chunks **redatados que conservan ID** para no
borrar la extracción nueva después. Un crash allí deja hueco reparable mediante
rebuild posterior. Publish SQL atómico no implica snapshot consistente de
grafo/vector durante la transición. Esto es código observado, no crash reproducido.

**Delta/alternativas.** Vanta tiene ingestors, dedup y supersession; el delta es
plan por documento/revisión con reuso, validación y explicación. Extender
`ingest/worker.rs` [CV-ingest] y writer [CV-write] evita otro motor documental.
Rebuild completo sigue siendo una alternativa explícita cuando no se conserva
baseline, chunker o ownership; rechazar actualización también es mejor que
adivinar. No usar hash de texto como única revisión del trabajo concurrente.

**Contrato Rust propuesto:** `DocumentRevision`, `ChunkUpdatePlan { base,
policy_version, kept, moved, replaced, fresh, invalidated_dependencies }` y
`DocumentUpdateOutcome::{Applied,Unchanged,RebuildRequired,Conflict,Partial}`.
Input autorizado con revisión esperada; output contadores y causa sin texto.
Cada chunk debe mapear a su fuente; IDs de contenido y posición son diferentes;
el plan debe cubrir el nuevo documento bajo el contrato del chunker.

**Pasos/commit/migración.** Registrar versión del chunker y dependencias de
extracción; plan puro; validar reconstrucción; preparar embeddings fuera del
commit; revalidar baseline; publicar cambios y manifest de generación; reparar
proyecciones. Si las primitivas no permiten flip coherente, declarar parcial
en lugar de prometer rollback. Legado sin baseline cae a rebuild identificado;
reindexar solo generación afectada. Rollback deja leer generación previa si
todavía es válida y no resucita una fuente borrada.

**Concurrencia/cancelación/dependencias.** A muestra readiness, B mantiene
ownership, C recibos y D exclusión. Una edición simultánea invalida el plan;
cancelar antes de publish deja staging no activo para GC explícito, no ready.
Después de commit devuelve identidad de operación para reconciliar.
**FMEA:** texto igual con contexto temporal distinto → hecho viejo: registrar
dependencias y redatar derivados; chunker incompatible → omisión: rechazo
tipado; borrar antes de nueva extracción → hueco: generación/recuperación
explícitas. **Aceptación manual:** inserción al principio, eliminación, texto
repetido, cambio de fecha, chunker legado, permiso insuficiente y crash entre
etapas; identidad estable y causa de rebuild visible. **No objetivos:** copiar
ChunkPolicy, pesos/prompts, atomicidad distribuida ni ahorro medido sin ejecución.

### Ficha candidata F: contexto autorizado y capacidad declarada del backend

**Problema y evidencia.** Cambiar la conexión correcta no acredita que el
llamador pueda leer ese dataset. [authorized_search:220-288][C-search] resuelve
read antes de routing/fan-out; [handlers:35-85][C-handlers] y
[context_global_variables.py:95-125][C-context] comprueban compatibilidad de
grafo y vector. Un backend no soportado no hereda aislamiento por su interfaz.
Delete autoriza antes de resolver el recurso [C-delete].

**Delta/alternativas.** [InjectionPolicy/RecallConfig][CV-recall] y tenancy del
writer [CV-write] ya existen. Extender propagación/diagnóstico de capacidad
por transporte es distinto de inventar IAM o una base por tenant. Un adapter
single-tenant honesto es alternativa a prometer multitenancy no soportada;
filtrar después de recuperar no repara el envío previo de datos al LLM.

**Contrato Rust propuesto:** `AuthorizedOperationContext { principal, scope,
authorization_revision, operation_id }`, creado solo por frontera gobernada;
`BackendMemoryCapabilities` declara garantías verificables de consulta,
mutación y publicación. Entrada de jobs y adapters usa ese contexto, no metadata
libre. Errores `UnsupportedIsolation`, `ScopeDenied`, `StaleAuthorization`,
`CapabilityUnavailable`; desconocido nunca se trata como permitido.

**Pasos/commit y compatibilidad.** Inventariar rutas SDK/MCP/HTTP/ingest; separar
identidad de metadata; comprobar acceso antes del fan-out y al commit de jobs
largos; propagar contexto inmutable; exponer ausencia de capacidad. Versionar
contrato del adapter sin alterar primario para lectura. Legado requiere política
explícita, no asignación al usuario actual. Revocar permiso invalida una
publicación pendiente; cancelación corta ramas pendientes y no borra resultados
confirmados. Rollback de UI no desactiva controles backend.

**FMEA/aceptación.** `ContextVar` heredado de otro trabajo → fuga: detectar
principal/scope inconsistente; comprobar dos tareas y contextos anidados.
Backend declara soporte que no implementa → filtrado incompleto: contrato
condicionado, no sello de seguridad. Sesión cacheada con ACL antigua → devolver
contenido revocado: invalidación por revisión. Aceptar acceso normal,
revocación durante trabajo, ID ajeno, backend unsupported, error de routing y
no-enumeración. Logs de clase de rechazo, sin nombres de datasets ajenos.
Depende de identidad existente; gobierna A–G. **No objetivos:** copiar globals
Python, autenticar con tags, imponer Postgres/Neo4j ni certificar todos los transports.

### Ficha candidata G: promoción de sesión con watermark de consumo explícito

**Problema/fuente.** Una sesión genera Q&A, feedback y trazas que el operador
quiere promover a conocimiento permanente en un destino concreto.
[improve.py:152-206][C-stages] enumera etapas y declara Q&A/trazas por
**usuario+sesión**, no por dataset; contenido ya promovido a otro destino no
se repite, mientras distillation sí tiene scope por dataset. Esa semántica
puede ser deliberada, pero debe mostrarse al consumidor.

**Delta/alternativas.** Vanta captura L0→L1 y ya tiene
[MemoryTaskHandler][CV-worker], reflection y escenas [CV-reflection].
No añadir “memoria de sesiones” paralela: distinguir captura, promoción,
interpretación y consumo por destino. Un único destino simplifica permisos;
un cursor por consumidor permite varios destinos pero aumenta retención y
obligaciones de revocación. No se prescribe cuál debe ser default.

**Contrato Rust propuesto:** `PromotionCursor { source_session, source_revision,
consumer_scope, policy_version, last_confirmed_unit }` y
`PromotionOutcome { applied, already_consumed, skipped, failed }`. Input:
sesión autorizada, destino autorizado e intención explícita; output ancla a
fuentes y recibo C. Feedback no autoriza cambiar identidad ni elevar una
preferencia inferida a regla de sistema.

**Pasos/commit.** Comparar cursores de pipeline existentes; definir unidad de
consumo y destino; snapshot de fuentes; proponer derivados; validar revisión y
permisos; confirmar outputs y avance de cursor coherentemente. Si no caben en
una mutación, reconciliar por operación. Migración conserva cursor previo con
semántica conocida; no reprocesar todas las sesiones sin consentimiento.
Cambiar política crea generación nueva solo donde se solicite.

**Concurrencia/cancelación/FMEA.** Dos consumidores avanzan cursor global →
el segundo pierde datos: detectar motivo “ya consumido en otro destino”, no
éxito vacío. Cursor antes del output → pérdida: confirmar después/coherentemente.
Retry o lease perdido → lección duplicada: identidad idempotente y revisión.
Fuente borrada durante LLM → revivir texto: invalidar al commit. Cancelar
conserva cursor confirmado y deja pendiente lo restante. Depende de B/C/F.
**Aceptación manual:** mismo destino repetido, dos destinos, edición/borrado de
sesión, permiso revocado y reintento tras timeout; contadores por consumidor
sin texto personal. Rollback desactiva promoción nueva, no restaura datos
purgados. **No objetivos:** entrenamiento online, aprendizaje de instrucciones
hostiles ni copiar flags/prompts de feedback/evaluación.

### Ficha candidata H: canales tipados, referencias y contexto sin answerer

**Problema/lectura.** Un agente necesita evidencia inspeccionable, no una
respuesta que oculte el origen. [HybridRetriever:120-208][C-hybrid] diferencia
grafo vacío (`NoDataError`) de miss, recupera canales en paralelo, carga pesos
opcionales y proyecta metadata permitida; `only_context` separa contexto de
completion. Metadata extra es opt-in y no toda metadata entra al prompt.
No interpretar top-k de chunks/entidades/hechos como budget global.

**Delta/contrato propuesto.** Extender `IntegratedContext` y
`assemble_with_recall` [CV-assembly], no otro RRF:
`EvidenceBundle { source_revision, channel, references, truncations,
channel_status }`; entrada consulta y contexto autorizado F, salida referencias
vigentes y texto gobernado, sin inferencia obligatoria. `EmptySource`,
`NoMatches`, `ChannelFailed`, `BudgetTruncated` son distintos.

**Pasos y efectos.** Inventariar explain actual; preservar identidad de fuente
al unir canales; reservar presupuesto en assembler; aplicar allowlist antes
de serializar; devolver bundle sin answerer o pasarlo al generador explícito.
Read-only, sin migración/reindexación; traza opt-in versionada. Cancelar detiene
ramas y no publica un bundle completo ficticio; revisión de lectura evita
anexar texto borrado durante expansión. Alternativa: solo chunks, menor
complejidad pero sin relaciones; rechazar pesos de personalización no auditados.
Depende de F y readiness A, no de promoción G.

**FMEA/aceptación.** Fallo de rama = cero hits → falso diagnóstico: estado de
canal; metadata libre → prompt injection/secretos: allowlist y taint existentes;
fan-out rebasa contexto → assembler mantiene autoridad y reporta truncamiento.
Aceptar origen vacío, consulta sin hits, fallo parcial, duplicado en dos
canales, referencia revocada y modo sin generación; traza no altera resultados.
Rollback desactiva campos opt-in. **No objetivos:** nueva fórmula/rúbrica,
pesos “truth” como verdad acreditada ni UI/generador paralelo.

## 3. Riesgos e incidencias

| Evidencia | Condicion y consecuencia | Leccion |
|---|---|---|
| Codigo confirmado | Lock dataset es solo del proceso; su docstring niega exclusion entre workers | No confundir proteccion de embedding DB con transaccion multiproceso |
| Codigo confirmado | Contexto de lock se hereda a tareas hijas | Revisar si una tarea hija debe poder mutar concurrentemente antes de llamarlo reentrante |
| Codigo confirmado | Grafo/vector fallan independientemente; orden grafo primero | Orden recuperable necesita rollback efectivo, no lo reemplaza |
| Codigo confirmado | `provenance_tracking=False` por defecto | Ownership para borrar y ledger audit-grade no son sinonimos |
| Codigo confirmado | Instalar GLiNER puede ser side effect del primer cognify | Entorno cerrado debe fijar runtime y desactivar auto-install |
| Codigo confirmado | Limpieza de sesiones despues de delete es no fatal | Borrado en grafo no certifica ausencia en caches derivados |
| Hipotesis | Ingesta no fijada por ID concurrente puede crear dos documentos del mismo contenido | No es corrupcion demostrada: upstream distingue documento de contenido |
| Operacional | Backend sin handler de ACL y acceso false comparte almacenamiento | Desplegarlo como multi-tenant requiere otro limite, no una bandera de marketing |

El alcance in-process y el orden de locks son explicitos:
[dataset_lock.py:1-27](https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee/infrastructure/locks/dataset_lock.py#L1-L27).
El comentario de ingesta distingue retry de PK fijada y documentos duplicados:
[ingest_data.py:636-665](https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee/tasks/ingestion/ingest_data.py#L636-L665).
No se reproduce ninguna carrera ni se afirma explotacion probada.

### Incidencias públicas: laguna de lectura cerrada, no garantía de corrección

La ejecución anterior solo tenía texto indexado tras fallo de GitHub MCP.
En esta continuación se leyeron **cuerpos completos y comentarios** mediante
API pública oficial (`/repos/topoteretes/cognee/issues/{n}` y `/comments`),
sin recalcular metadata del proyecto. Observación remota **2026-11-07**.
`websearch.fetchWebContent` también permitió leer el cuerpo JSON de #3235.

| Incidencia efectivamente leída | Reporte, fecha y estado observado | Contraste con código fijado y transferencia |
|---|---|---|
| [#3235: delete_data skips dataset membership check when relational Data row is missing](https://github.com/topoteretes/cognee/issues/3235) | Creado/cerrado 2026-06-21; `closed/not_planned`. Reporta dev de ese día, ACL activo, un colaborador con delete y UUID sin Data; cuatro comentarios incluyen repro/screenshot del reportante | [datasets.py:457-575][C-delete] autoriza dataset, resuelve ID bajo lock, desenlaza ownership y distingue `not_found`. Sigue existiendo custom graph sin Data, deliberadamente. No afirmar bypass cross-dataset ni fix fusionado por estar cerrado. B/F: pertenencia demostrable sin exigir falsamente que todo recurso sea fila SQL |
| [#2291: delete_dataset fails in non-public postgres schema](https://github.com/topoteretes/cognee/issues/2291) | Creado 2026-03-07, cerrado 2026-04-05, `completed`. Reporta 0.5.2 con tablas en `cognee`, `search_path` configurado y error `public.datasets`; comentario añade 0.5.4.dev1 | Diferencia entre rutas ORM y SQL calificado según el reporte. Dos comentarios, uno bot con PRs relacionadas: no se verificó un PR causal. No se auditó por completo el adaptador actual; vigencia desconocida. F: capabilities/configuración efectiva, no trasladar un schema SQL al motor embebido |
| [#1366: Cognee CLI delete, add breakdown of affected data](https://github.com/topoteretes/cognee/issues/1366) | Creado 2025-09-10, cerrado 2025-10-06, `completed`. Solicita preview con counts y confirmación; doce comentarios devueltos incluyen discusión del flujo y feedback del mantenedor | Comentario 2025-09-16 dice que `cognee.delete()` requería arreglo separado y comenta `get_deletion_counts`. No acredita que el preview repare el borrado ni una PR fusionada. B/Studio: preview y recibo son contratos diferentes; no adoptar el fallback propuesto «sin counts, borrar igual» como garantía de purga |

**Validación discriminante futura, no ejecutada:** #3235 requiere custom ID
con ownership, UUID inexistente y UUID de otro dataset con permisos distintos;
la ausencia de Data sola no decide autorización. #2291 requiere crear/listar/
borrar en un schema no-default y comprobar el destino efectivo de cada ruta.
#1366 requiere que cambios entre preview y ejecución invaliden la estimación
o se informen, y que un fallo posterior no se represente como éxito por la UI.
Ningún screenshot ajeno es una reproducción propia. Queda sin verificar la
relación entre cierres/PRs y versiones distribuidas.

## 4. Prerrequisitos en VantaDB

- Revalidar capacidades y cambios concurrentes indicados en METHOD.
- Contrato de disponibilidad, revision de fuente y revision de indice coherente.
- Propiedad de derivados compartidos e idempotencia de retries explicitadas.
- Scope autorizado tanto en ingesta como en retrieval, borrado y history.
- Definir si un fallo de proyeccion bloquea, deja parcial o permite reparacion.
- Distinguir contexto recuperado de respuesta y budget por dataset de global.
- Revisar licencias y dependencias si se adopta codigo, incluso traducido a Rust.

## 5. Que no trasladar

No adoptar plataforma completa ni todas las dependencias/adaptadores; no copiar
auto-install en runtime, globals de configuracion como autorizacion, reentrancia
implicitamente heredada, LLM mandatory en primitivas ni fail-open de sesiones
para garantias de purga. No revivir memify/triplet embeddings deprecados.
No migrar estrategias agentic o prompts de benchmark al motor embebido.
“Graph before vector” no es sustituto universal de WAL/transaccion VantaDB.

## 6. Advertencias y cobertura

**Leido por rangos, no auditoria total:** `api/v1/add/add.py`,
`cognify/cognify.py`, `memify/memify.py`, `search/search.py`;
`datasets/datasets.py` (delete), `update/update.py`, `improve/improve.py`,
`forget/forget.py` (contratos/rutas); `tasks/ingestion/ingest_data.py`,
`tasks/storage/add_data_points.py`; resolvers de datasets;
`modules/retrieval/hybrid_retriever.py`, `modules/search/methods/search.py`;
`context_global_variables.py`, `infrastructure/locks/dataset_lock.py`,
registro de handlers, `modules/cognify/config.py`, `pyproject.toml`.
Evaluacion/UI: archivos identificados en secciones 7 y 10.

**No examinado:** cada task de extraccion, modelos de merge semantico, todos los
retrievers, implementaciones completas de ACL/auth y DB workers, recovery y
rollback bajo fallo, todos los conectores/community adapters, UI completa,
tests y corpus completos. Las rutas deterministas y GLiNER no se probaron.
No se ejecutaron validadores docs/schema ni regeneracion de indices.

La documentacion upstream cambia de API y de defaults; fijar SHA es esencial.
No convertir evidencia historica de 2025 en garantia de la version 1.6.7.

## 7. Benchmarks existentes

### LoCoMo actual, no el Hotpot archivado

Runner `cognee/eval_framework/locomo/run_locomo_eval.py` coordina subprocesses
`ingest.py`, `answer.py` y `aggregate.py`. Entrada `locomo10.json`, descargable si
falta, se transforma en ventanas fechadas de seis turnos. Guarda config,
dataset hash, respuesta y verdict por pregunta:
[README.md:9-40](https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee/eval_framework/locomo/README.md#L9-L40).
Es memoria + retrieval + QA + juez, **no recall de evidencias aislado**.

Metrica `LocomoLLMJudgeMetric`: 1 correcto, 0 incorrecto y `None` si no parsea;
el agregado cuenta `unscored` y `answer_errors` por separado y calcula medias
sobre scores presentes:
[llm_judge.py:158-181](https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee/eval_framework/locomo/metrics/llm_judge.py#L158-L181),
[aggregate.py:136-159](https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee/eval_framework/locomo/aggregate.py#L136-L159).
Open-domain y determinadas gold questions auditadas se excluyen antes de
retrieval; no comparar con LoCoMo completo ni ocultar ese denominador.

Flags existentes `--data-path`, `--conversation-index`, `--run-dir`,
`--max-sessions`, `--max-questions`, `--num-runs`, concurrencia y config permiten
variantes, no calidad certificada. **Peligro:** `ingest.py:146-149` con
`--prune-first` borra datos y metadata/usuarios/ACL:
[ingest.py:110-149](https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee/eval_framework/locomo/ingest.py#L110-L149).
`.env` prevalece sobre opciones de entorno del driver; comprobar DB paths
efectivos antes de cualquier intento. Costes: embeddings, extraccion, generacion,
juez y opciones de resolucion de fechas/global context; modelos/API configurados.

### BEAM

Entradas de conversacion/sesion preprocesadas, `beam/local_ingest.py`,
`beam/eval/run_sweep.py`, `aggregate_cross_run.py`; fuentes BEAM y BEAM-10M de
Hugging Face declaradas en
[REPORT.md:33-83](https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee/eval_framework/beam/REPORT.md#L33-L83).
La metrica promedia criterios 0/0.5/1 por pregunta; Kendall tau es otra metrica
registrada para ordenacion, no la misma exactitud:
[beam_rubric.py:1-7](https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee/eval_framework/beam/eval/metrics/beam_rubric.py#L1-L7),
[aggregate_cross_run.py:12-41](https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee/eval_framework/beam/eval/aggregate_cross_run.py#L12-L41).
No se reproduce ni redefine prompt/rubrica upstream.

El propio reporte reconoce seleccion de routing 10M sobre las mismas preguntas
reportadas, orquestacion especifica Modal no publicada y ausencia de ablaciones.
No es holdout comparable a la pista fija 100K. `local_ingest --prune-first`
tambien borra sistema; harvesting borra/reconstruye sesiones y cuesta llamadas.
No usar instancia compartida.

**Ejecución local efectiva (CO-1), distinta del reporte Modal:** en
[run_sweep.py:148-177,309-404](https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee/eval_framework/beam/eval/run_sweep.py#L309-L404)
el corpus debe estar **preingerido**; defaults: split 100K, conversación 0,
una repetición y concurrencia 4. Exige `--config-json-path`, sin preset
distribuido: modelos y configuración efectiva no quedan fijados por REPORT.
10M requiere preguntas exportadas; si se aporta `questions_path`, split es
metadata, no selección efectiva de preguntas. Consultar corpus existente
excluye su construcción/ingesta/embeddings del coste de esa pasada.

**Denominador por métrica:** en
[aggregate_cross_run.py:24-85,92-114](https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee/eval_framework/beam/eval/aggregate_cross_run.py#L24-L114)
solo scores **numéricos** entran en las medias; `question_count` cuenta
entradas, no la población numérica de cada métrica. Ausente/no puntuado no
equivale automáticamente a cero ni a respuesta juzgada. No se calcularon
scores nuevos ni se alteró la configuración o la rúbrica.

**FMEA/aceptación CO-1:** corpus residual/config desconocida → ejecución no
reproducible; conteo de entradas presentado como juzgadas → denominador falso.
Conservar en el recibo fuente de preguntas, configuración efectiva o Unknown,
estado del corpus y conteos numéricos por métrica según el protocolo existente.
No equiparar una ejecución local al reporte Modal ni inferir ahorro total de
consultar un corpus ya construido.

**Historico:** `evals/old/hotpot_qa_24_2025/` conserva runners Modal y EM/F1/jueces,
no es benchmark vigente segun `evals/README.md`. No se usan sus claims como
resultado de este SHA ni se reproducen cifras de calidad.

**Pendiente manual:** revisar flags/caminos efectivos y preaprovisionar
dependencias antes de decidir comandos. Toda comparacion se somete al
[protocolo comun](METHOD.md#protocolo-de-comparacion-de-benchmarks).
No se crea grader, prompts, thresholds, harness ni configuracion de scoring.

### Defaults y efectos del driver LoCoMo, lectura complementaria

Fuente [run_locomo_eval.py:64-119,309-375][C-locomo-driver]:
default `conversations="0"`, etapa `all`, una repetición, ocho preguntas
concurrentes y una conversación a la vez; ventanas de seis turnos y chunk size
4096. Config por defecto `locomo_hybrid_20_20_20.json`, answer/extraction
`openai/gpt-5-mini` y judge `openai/gpt-5.1`, salvo overrides de entorno/CLI.
Son valores upstream observados, **no** nuevos ajustes para Vanta.

`child_env` separa DATA/SYSTEM root por conversación y fija acceso backend
false, caching true, auto-feedback false y telemetría deshabilitada si no hay
override previo. Mide esa variante, no el servicio multitenant completo ni el
comportamiento con feedback automático. El comentario «full run» no cambia
el default de una conversación. La presencia de paths separados no asegura
por sí sola que un `.env` cargado después no cambie el destino efectivo.

Reusar un run-dir puede saltar ingesta ya marcada completed y conserva
memorias para reconsultar; no equivale a experimento limpio tras cambiar
modelo, chunker o corpus. Los subprocesses escriben logs y reportes, pueden
descargar dataset y hacen llamadas pagadas. El código del driver construye
procesos con el mismo intérprete; no son operaciones read-only. En esta tarea
solo se leyó su fuente, no se invocó.

**Adaptación aplicable sin construir harness:** mapear cada conversación y
ventana a las sesiones/fuentes existentes de Vanta, conservar fechas y IDs,
devolver referencias y distinguir fase de extracción, retrieval, respuesta y
juez. Readiness A impide responder con una generación anterior silenciosa;
F gobierna el destino del adapter. La comparación sigue METHOD/BENCHMARK-NOTES,
sin trasladar filtros de preguntas, presets o prompts para crear otro scoring.

## 8. Matriz frente a VantaDB

| Capacidad | Estado VantaDB al corte fijado y fuente | Incremento a evaluar y decisión |
|---|---|---|
| BM25/HNSW/RRF | Existe en motor, [referencia](VANTA-REFERENCE.md); ruta configurable [CV-recall] | [HybridRetriever][C-hybrid]: H, distinguir canales/contexto, no nueva fusión |
| Dedup/supersession | Existe: [MemoryRecord/DedupAction][CV-record] y [writer][CV-write] | [Update incremental][C-incremental]: E condicional a documentos mutables |
| Procedencia/auditorías | Existe: [fuentes/version][CV-record] y log post-commit [CV-write] | [Ownership de derivados][C-store], [delete][C-delete]: B, cardinalidad/recibo, no provenance desde cero |
| Scheduler/checkpoints | Existe: [scheduler][CV-scheduler], [TaskCheckpointManager][CV-task] | [cognify][C-ready]/[improve][C-improve]: A/C, hitos por revisión, no otra cola |
| Sesiones/context assembly | Existe: [assemble_with_recall][CV-assembly] y [MemoryTaskHandler][CV-worker] | [improve][C-stages]/[delete][C-delete]: G, consumo por destino y revocación |
| Grafo/vector aislado por dataset equivalente | No acreditado en esta muestra propia; no ausencia global | [Handlers][C-handlers]: F, estudiar contrato de capacidades, no exigir una DB por tenant |
| Budget global de contexto | Existe: [assembler][CV-assembly]; excepciones en [referencia](VANTA-REFERENCE.md) | H no sustituye budget global por top-k de cada dataset |
| Studio memoria/actividad | Existe en otro worktree; fuentes de §10 | A/B/C se presentan en inspectores existentes, no nueva aplicación |
| Equivalencia de BEAM local/publicado | No acreditada en esta lectura | CO-1: corpus preingerido/config efectiva y población numérica por métrica; no `question_count` como divisor universal |
| Auto-install GLiNER | No aplicable como mejora del motor | Rechazar instalación implícita, no negar rutas deterministas de extracción |

## 9. Debilidades sugeridas, no bugs atribuidos

Investigar si la UI puede distinguir “guardado pero no enriquecido”, si las
fuentes compartidas sobreviven al borrado de una procedencia y si los resultados
parciales preservan causalidad de errores. Verificar que scope del job no se
reconstruye desde metadata y que fan-out respeta budget compartido.
Son preguntas incrementales, no ausencias ni defectos actuales acreditados.
Hallazgos historicos de VantaDB no se trasladan al worktree mutable.

**Negativo estático revalidado al corte propio**, no deducido de Cognee:
[apply_dedup_batch:389-456][CV-write] borra destinos antes de `put_batch` y
registra efectos después; el comentario explicita fallo parcial entre fases.
E/B/C no pueden prometer reemplazo+ownership+recibo indivisibles sobre esa
secuencia sin resolver su frontera de publicación. El mecanismo está leído;
la pérdida tras un crash concreto no fue reproducida.

**Cobertura parcial, no defecto probado:** `TaskHandler::handle` devuelve
`Result<(), String>` en [pipeline_worker][CV-worker]; eso no acredita por sí solo
un resultado durable de cada subetapa. El candidato C debe inventariar el
generation log y resultados internos antes de añadir otro DTO. Tampoco los
campos `version/source_message_ids` de [MemoryRecord][CV-record] prueban
precondición atómica de revisión. Se descartan «falta provenance», «sin scheduler»,
«sin budgets» y «sin UI». La aceptación de esas garantías requiere casos
adversos futuros, no solo presencia de tipos.

## 10. Interfaz e inspeccion

Fuente web real en `cognee-frontend`, no solo capturas. Ruta knowledge-graph
actual delega a BusinessPage; esta usa TenantProvider y BusinessView:
[page.tsx:1-7](https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee-frontend/src/app/(app)/knowledge-graph/page.tsx#L1-L7),
[BusinessPage.tsx:1-23](https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee-frontend/src/app/(app)/business/BusinessPage.tsx#L1-L23).
Se inventariaron datasets/search/sessions/activity/settings e integraciones;
no se certifica que todas funcionen sin plataforma externa/pod.
Frontend self-host/demo y producto alojado no se presumen equivalentes.

API Python async y routers FastAPI estan en el clon. CLI `cognee`/`cognee-cli`
y servidor MCP separado son entradas declaradas por el proyecto; conectores
Apollo/Asana/GitHub/Gmail/Notion/Slack y otros tienen paquetes separados.
No se instalaron ni se auditaron permisos de todos esos adapters.
Desktop nativo equivalente a Tauri: **no localizado**, no “inexistente”.
Para Studio, evaluar inspeccion de ready-state, propietario de hechos y recibo
de borrado antes de pensar en una nueva pantalla que ya exista.

### Flujo web comprobado por lectura, no por arranque

El manifest [frontend/package.json][C-ui-package] fija Next.js 16.3.8,
React 19, Mantine 8, TanStack Query, D3 y react-force-graph. Es frontend web,
no un binario desktop. Flujo concreto:

1. `knowledge-graph/page.tsx` delega en `BusinessPage`; obtiene
   `cogniInstance` de TenantProvider y no monta BusinessView sin instancia.
   ErrorBoundary captura errores de render, no certifica disponibilidad del pod.
2. [BusinessView:122-170][C-ui-view] obtiene datasets legibles del FilterProvider
   y reduce el índice de gobernanza mediante `narrowToReadable`. El payload de
   provenance de tenant no se usa como autorización de lectura ni para renderizar
   nombres ajenos cuando el fetch legible falla.
3. [SearchBar:29-74][C-ui-search] primero ofrece coincidencias locales por nombre;
   al preguntar llama `searchDataset` con el dataset activo y
   `searchType="GRAPH_COMPLETION"`. [searchDataset.ts:1-29][C-ui-api] añade top-k
   y referencias desde settings y envía POST `/v1/search` con timeout propio.
   Muestra texto de respuesta o mensaje de error: no es una medición de recall.
4. [useBusinessQaSurface:46-82][C-ui-qa] diferencia respuesta manual de eventos
   con IDs de nodos. La pregunta manual no acredita qué vértices respaldaron
   la respuesta; un spotlight posterior de eventos es otra señal.
5. [useBusinessLiveUpdates:21-90][C-ui-live] distingue socket disponible de
   handshake sano, usa polling como alternativa y mantiene cleanup de ambos.
   Su cola visible limitada no es un log durable de entrega al modelo.
6. [BusinessView:370-394][C-ui-whatif] calcula qué entidades quedarían aisladas
   si una desapareciera y resalta IDs en el canvas. Es **simulación visual**
   sobre `semanticLinks`, no preview transaccional de delete, GC de ownership
   ni prueba de todos los recursos afectados en backend.

**Delta para Studio, sin reinventarlo:** en el worktree propio fijado
`0ad38c542b39a589035b0e17fdedb4c09dca24b9`,
`feature-freez/desktop/src/components/memory/MemoryLens.tsx:381-409` ya consulta generation
log y abre anchors;
`feature-freez/desktop/src/components/lens/retrieval/RetrievalLens.tsx:138-164,193-228`
ya usa explain y audit. Conservar esos inspectores y añadir readiness A,
recibo B/C y referencias H cuando el transporte los soporte.
`feature-freez/desktop/src/components/activity/ActivityPanel.tsx:52-89`
separa error de carga/audit no configurado y eventos recibidos:
«sin capacidad» no debe mostrarse como «sin eventos». Se releyeron esos cuerpos
mediante `git show` del SHA completo; no se atribuyen los bytes dirty actuales
de `desktop/...` a ese commit ni al snapshot Rust `97c9623`.

**Aceptación manual de UI pendiente:** cambiar dataset durante search no debe
anexar respuesta al nuevo scope; error de canal no se presenta como respuesta
vacía; permisos revocados no reaparecen vía eventos/cache; what-if no habilita
un delete con alcance supuesto. No se inició frontend ni navegador.
Si la UI presenta BEAM, debe distinguir entradas, scores numéricos y no
puntuadas, sin dibujar `question_count` como preguntas juzgadas (CO-1).
Correcciones [CO-1/CO-2](reviews/memory-code-remaining.md#cognee) aplicadas,
pendientes de reconfirmación independiente, no aceptación.
Fuentes Studio inmutables releídas con `git show` (2026-11-07):
[MemoryLens](https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/memory/MemoryLens.tsx#L381-L409),
[ActivityPanel](https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/activity/ActivityPanel.tsx#L52-L89),
[RetrievalLens](https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/lens/retrieval/RetrievalLens.tsx#L138-L228).

### Cierre de autoría y cobertura remanente

- **Conservado:** las ocho fichas A–H y sus flujos, FMEA y límites; no se
  sustituyeron por dos propuestas genéricas. Se resolvieron las referencias
  abreviadas que antes no tenían definición, para hacer navegables sus fuentes.
- **Relaciones:** F gobierna identidad de A–H; A/C registran disponibilidad y
  efectos; B/D/E gobiernan mutación de derivados; G depende de B/C/F; H es
  lectura y reutiliza el assembler. Las semejanzas con recibos de Mem0 y
  revisiones de Hindsight son puntos a consolidar, no tres sistemas nuevos.
- **Leído adicionalmente:** licencias/NOTICE/manifests mencionados, cuerpos y
  comentarios de tres incidencias, driver LoCoMo y flujo web anterior.
  Corrección CO-1: `run_sweep.py:148-177,309-404`, defaults 1/4 y
  `aggregate_cross_run.py:24-114`; corpus preingerido/población numérica.
  CO-2: cuerpos Studio al SHA/rutas inmutables de §10.
  El resto de la cobertura por rangos de §6 se preserva, sin declararla exhaustiva.
- **Lagunas:** PR causal de cierres, adaptador Postgres completo, todos los
  handlers de ACL, implementación completa de cada etapa/modelo de merge,
  recovery multiproceso, conectores, derechos de datasets/pesos y SBOM.
  BEAM Modal no publicado y selección sobre preguntas reportadas siguen siendo
  límites; no se crean protocolos alternativos para cubrirlos artificialmente.
- **Pruebas futuras:** las aceptaciones por ficha y §10 siguen sin ejecutarse.
  Autorización, crash safety, ahorro y mejora de calidad no están certificados.
  Diez ejes documentados no equivalen a aceptación independiente.
- **Partición futura:** por superar 300 líneas, mantener esta comparación como
  principal; si se aprueba implementación, separar specs de readiness/recibos,
  ownership/update/locks, promoción y contexto/UI. No crear esos archivos ahora.

No se modificó código, harness, prompts, contratos, índices ni metadata.
No staging, commits, push o validadores globales: cierre de autoría únicamente.

[C-dataset]: https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee/modules/data/models/Dataset.py#L9-L38
[C-data]: https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee/modules/data/models/Data.py#L10-L75
[C-chunk]: https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee/modules/chunking/models/DocumentChunk.py#L34-L86
[C-ready]: https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee/api/v1/cognify/cognify.py#L543-L767
[C-store]: https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee/tasks/storage/add_data_points.py#L74-L363
[C-delete]: https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee/api/v1/datasets/datasets.py#L457-L575
[C-improve]: https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee/api/v1/improve/improve.py#L219-L284
[C-stages]: https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee/api/v1/improve/improve.py#L152-L206
[C-incremental]: https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee/api/v1/update/incremental.py#L662-L1210
[C-lock]: https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee/infrastructure/locks/dataset_lock.py#L1-L133
[C-search]: https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee/modules/search/methods/search.py#L220-L288
[C-hybrid]: https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee/modules/retrieval/hybrid_retriever.py#L120-L208
[C-context]: https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee/context_global_variables.py#L95-L125
[C-handlers]: https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee/infrastructure/databases/dataset_database_handler/supported_dataset_database_handlers.py#L35-L85
[C-license]: https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/LICENSE
[C-notice]: https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/NOTICE.md
[C-ui-package]: https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee-frontend/package.json
[C-connectors-package]: https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee-connectors/core/pyproject.toml#L1-L25
[C-connector-license]: https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee-connectors/connectors/github/LICENSE
[C-locomo-driver]: https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee/eval_framework/locomo/run_locomo_eval.py#L64-L375
[C-ui-view]: https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee-frontend/src/app/(app)/business/BusinessView.tsx#L122-L170
[C-ui-search]: https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee-frontend/src/modules/business/panels/SearchBar.tsx#L29-L74
[C-ui-api]: https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee-frontend/src/modules/datasets/searchDataset.ts#L1-L29
[C-ui-qa]: https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee-frontend/src/modules/business/useBusinessQaSurface.ts#L46-L82
[C-ui-live]: https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee-frontend/src/modules/business/useBusinessLiveUpdates.ts#L21-L90
[C-ui-whatif]: https://github.com/topoteretes/cognee/blob/8853378eae5366addb064e758e560a456146ae1a/cognee-frontend/src/app/(app)/business/BusinessView.tsx#L370-L394
[CV-record]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/abstractions/types.rs#L71-L154
[CV-write]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/record/l1_writer.rs#L131-L456
[CV-worker]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/services/pipeline_worker.rs#L44-L451
[CV-scheduler]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/services/scheduler.rs#L79-L145
[CV-task]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/utils/task_checkpoint.rs#L122-L214
[CV-ingest]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/ingest/worker.rs#L22-L120
[CV-reflection]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/reflection/mod.rs#L134-L319
[CV-recall]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/hooks/auto_recall.rs#L116-L335
[CV-assembly]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/context_engine/engine.rs#L197-L348

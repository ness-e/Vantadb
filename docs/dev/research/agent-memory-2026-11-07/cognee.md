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

- **Objetivo/unidad:** distinguir original guardado, chunks recuperables y grafo
  enriquecido. Fuente: `get_indexing_tasks/get_default_tasks`, `cognify.py:677-767`.
- **Seam:** ingestors, scheduler/checkpoints y auditoria VantaDB existentes.
- **Contrato hipotetico:** `MemoryReadiness { source_revision, chunk_index,
  enrichment, failures }`; disponibilidad es estado, no booleano generico.
- **Acciones/dependencias:** revalidar estados actuales; identificar etapas sin
  LLM y fuente de verdad; conectar diagnostico sin nueva cola paralela.
- **FMEA:** anunciar ready tras fallo posterior; servir chunks de version vieja;
  volver a embeber todo al enriquecer.
- **Aceptacion/manual futura:** detener enriquecimiento, consultar chunks de la
  revision confirmada y comprobar estado parcial; reanudar sin duplicar IDs.
- **No objetivos:** adoptar runner Python ni declarar menor latencia sin medir.

### Ficha candidata B: procedencia con multiples propietarios

- **Objetivo/unidad:** borrar un documento sin eliminar hecho respaldado por otro.
  Fuente: `add_data_points`, `storage/add_data_points.py:307-343`;
  `datasets.delete_data`, `datasets.py:457-575`.
- **Seam:** procedencia/supersession y auditoria de indice existentes.
- **Contrato hipotetico:** `SourceRef { scope, source_revision, chunk_id }`,
  `DerivedOwners`, `DeletionReceipt`; referencias no contadores opacos.
- **Acciones/dependencias:** inventariar ownership actual; separar evidencia
  compartida de identificacion del hecho; confirmar orden de actualizacion.
- **FMEA:** borrar fuente inicial elimina hecho compartido; referencia huerfana;
  auditoria conserva contenido sensible; retry duplica referencias.
- **Aceptacion/manual:** dos documentos respaldan un hecho; borrar uno y luego
  ambos, inspeccionar retrieval y recibos en cada almacen/proyeccion.
- **No objetivos:** inventar ausencia de procedencia ni prometer purga fisica.

### Ficha candidata C: resultado de enriquecimiento por etapa

- **Objetivo/unidad:** resultado completo/parcial/noop y fallo fatal distinguibles.
  Fuente: `improve.execute_stages`, `improve.py:1-23,219-225`.
- **Seam:** scheduler/checkpoints/context budget VantaDB, no otro orquestador.
- **Contrato hipotetico:** `EnrichmentOutcome { stages: Vec<StageOutcome>,
  resume_token, budget_used }`; fallo operativo distinto de ausencia de cambios.
- **Acciones/dependencias:** mapear errores existentes, declarar side effects,
  idempotencia y ownership de cada etapa; LLM siempre fuera del nucleo obligatorio.
- **FMEA:** continuar tras perder dato primario; cerrar job exitoso con etapa
  fallida; watermark global hace omitir otro dataset.
- **Aceptacion/manual:** fallo antes/despues de persistir, agotamiento de budget y
  reintento; verificar estados, historial y no repeticion de efectos confirmados.
- **No objetivos:** copiar politica fail-open para operaciones criticas.

### Ficha candidata D: orden de locks declarado como contrato

- **Objetivo/unidad:** documentar jerarquia dataset -> recurso limitado, antes
  de mezclar mutacion, borrado y jobs. Fuente: `get_dataset_lock/dataset_lock`,
  `infrastructure/locks/dataset_lock.py:1-27,85-133`.
- **Seam:** concurrencia del scheduler, indices y checkpoints VantaDB.
- **Contrato hipotetico:** `DatasetMutationGuard` y `ProjectionPermit`, con
  adquisicion y liberacion ordenadas; no se prescribe libreria nueva.
- **Acciones/dependencias:** dibujar orden actual y reentrancia, distinguir
  tarea/hilo/proceso; revisar cancelacion y tareas hijas.
- **FMEA:** inversion de semaphore y lock, falsa reentrancia por contexto
  heredado, bloqueo local vendido como exclusion entre procesos.
- **Aceptacion/manual:** dos mutaciones y un borrado del mismo dataset mas otro
  dataset; inspeccionar progreso y cancelacion, sin inferir seguridad de un sleep.
- **No objetivos:** implementar locks distribuidos por analogia.

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

**Incidencias publicas, lectura limitada al texto indexado por busqueda web:**

- [#3235](https://github.com/topoteretes/cognee/issues/3235) discute borrar UUID
  custom sin fila Data. En el SHA leido el branch sigue existiendo, pero exige
  permiso delete, usa ownership y devuelve `not_found` cuando no encuentra nada.
  No se reproduce el antiguo “success para cualquier UUID” ni se infiere acceso
  a otro dataset solo por no existir una fila SQL.
- [#2291](https://github.com/topoteretes/cognee/issues/2291) relata schema Postgres
  `public` hardcoded. Adaptador relacional completo no auditado: vigencia
  desconocida, no defecto certificado del SHA.
- [#1366](https://github.com/topoteretes/cognee/issues/1366) pide preview del borrado
  CLI; propuesta UX, no prueba de aislamiento ni borrado inseguro actual.

GitHub MCP no conecto; no se leyeron comentarios ni se valido estado abierto/
cerrado. No atribuir un fix o defecto presente a partir del estado del issue.

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

**Historico:** `evals/old/hotpot_qa_24_2025/` conserva runners Modal y EM/F1/jueces,
no es benchmark vigente segun `evals/README.md`. No se usan sus claims como
resultado de este SHA ni se reproducen cifras de calidad.

**Pendiente manual:** revisar flags/caminos efectivos y preaprovisionar
dependencias antes de decidir comandos. Toda comparacion se somete al
[protocolo comun](METHOD.md#protocolo-de-comparacion-de-benchmarks).
No se crea grader, prompts, thresholds, harness ni configuracion de scoring.

## 8. Matriz frente a VantaDB

| Capacidad | Estado VantaDB segun METHOD | Incremento a evaluar |
|---|---|---|
| BM25/HNSW/RRF | Existe | Mantener motor; distinguir canales y contexto |
| Dedup/supersession | Existe | Diff de documento/ownership compartido |
| Procedencia/auditorias | Existe | Verificar cardinalidad de fuentes y recibo de borrado |
| Scheduler/checkpoints | Existe | Hitos de indexacion/enriquecimiento por revision |
| Sesiones y context assembly | Existe | Estados parciales y cache invalidation transversal |
| Grafo/vector por dataset | No acreditado equivalente en este estudio | No inferir ausencia; comparar modelo de aislamiento actual |
| Budget global de contexto | Existe | No sustituirlo por top-k por canal/dataset |
| Studio memoria/actividad | Existe en otro worktree | Solo UX de hitos/ownership no cubierta |
| Auto-install GLiNER | No aplicable como mejora del motor | Evitar dependencia implicita en runtime |

## 9. Debilidades sugeridas, no bugs atribuidos

Investigar si la UI puede distinguir “guardado pero no enriquecido”, si las
fuentes compartidas sobreviven al borrado de una procedencia y si los resultados
parciales preservan causalidad de errores. Verificar que scope del job no se
reconstruye desde metadata y que fan-out respeta budget compartido.
Son preguntas incrementales, no ausencias ni defectos actuales acreditados.
Hallazgos historicos de VantaDB no se trasladan al worktree mutable.

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

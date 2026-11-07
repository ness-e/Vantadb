---
title: "Serviette: cambios incrementales y retractions sobre documentos vivos"
kind: research
status: proposed
description: "Analisis estatico del dataflow Pathway, identidad de chunks y limites de memoria de Serviette frente a VantaDB"
tags: [research, memory, engineering]
---

# Serviette: cambios incrementales y retractions sobre documentos vivos

## Índice

- [1. Arquitectura e identidad](#1-recomendaciones-para-vantadb-y-contexto-arquitectonico-del-proyecto)
- [2. Fuentes y fichas A/B](#2-partes-extraibles-flujo-real-lectura-de-codigo-y-mapeo-a-rustvantadb)
- [3. Riesgos](#3-defectos-riesgos-e-issues-transferibles-con-su-nivel-de-evidencia)
- [4. Prerrequisitos](#4-prerrequisitos-en-vantadb-antes-de-incorporar-la-idea)
- [5. Rechazos](#5-componentes-y-practicas-que-no-deben-trasladarse)
- [6. Método](#6-advertencias-metodologicas)
- [7. Benchmarks](#7-benchmarks-existentes-runner-entrada-metricas-dependencias-y-riesgos)
- [8. Matriz](#8-matriz-existeparcialno-acreditadono-aplicable-frente-a-vantadb)
- [9. Negativos](#9-debilidades-de-vantadb-sugeridas-por-el-contraste-no-bugs-inventados)
- [10. UI](#10-escritorio-interfaz-e-inspeccion-implementado-externo-o-no-localizado)
- [11. Cierre: flujos/contratos](#11-cierre-de-autoría-servidor-materialización-y-contratos-completos)
- [12. Evidencia pública/benchmark/Studio](#12-incidencias-benchmarks-matriz-y-studio)

## 1. Recomendaciones para VantaDB y contexto arquitectonico del proyecto.

**Dictamen:** estudiar identidad reproducible de derivados y compatibilidad de
replay. No reducir Serviette a un ejemplo de RAG estatico: su valor distintivo es
propagar altas, cambios y bajas de fuentes a un dataflow incremental.
No confundir frescura documental con memoria autobiografica de un agente.

- Repositorio `pathwaycom/serviette`, clon `C:\Users\Eros\serviette`.
- HEAD comprobado `84e92a33728cf843026f4011d86568b403e38fb4`.
- [Snapshot](github-candidates.json): **58 725 estrellas**, no archivado,
  licencia API `MIT`; [LICENSE real](https://github.com/pathwaycom/serviette/blob/84e92a33728cf843026f4011d86568b403e38fb4/LICENSE#L1-L21)
  MIT, copyright 2026 Sergey Kulik. La licencia del wrapper no resuelve las
  condiciones de Pathway, parsers, modelos ni conectores.

Stack [declarado en pyproject](https://github.com/pathwaycom/serviette/blob/84e92a33728cf843026f4011d86568b403e38fb4/pyproject.toml#L1-L70):
Python >=3.10, Pathway xpack-llm, FastAPI/Uvicorn, Pydantic, DuckDB y parsers.
Mapa: conectores `only_metadata` → fetch de bytes bajo demanda → parse cacheado
no determinista → split reproducible → embedding → sinks nativos con upsert/
retraction → accessors de consulta → hybrid/rerank → respuesta con fuentes.
Indexer y servidor son procesos distintos; el chat HTML puede servirse
same-origin desde el API. No es necesario imitar esos procesos en Vanta embebido.

## 2. Partes extraibles, flujo real, lectura de codigo y mapeo a Rust/VantaDB.

| Fuente leida | Evidencia concreta |
|---|---|
| [S1 graph.py:1-35](https://github.com/pathwaycom/serviette/blob/84e92a33728cf843026f4011d86568b403e38fb4/serviette/indexer/graph.py#L1-L35) | Dataflow y retractions: no volver a leer bytes eliminados para deshacer derivados |
| [S2 graph.py:550-660](https://github.com/pathwaycom/serviette/blob/84e92a33728cf843026f4011d86568b403e38fb4/serviette/indexer/graph.py#L550-L660) | `parse_document`, `split_text`, `make_id`; splitter puro solo en whitelist; flatten y embedding |
| [S3 sources.py:1-65](https://github.com/pathwaycom/serviette/blob/84e92a33728cf843026f4011d86568b403e38fb4/serviette/indexer/sources.py#L1-L65) | `read_source`: filesystem/GDrive/S3/SharePoint/PyFilesystem separados de fetchers |
| [S4 fingerprint.py:145-225](https://github.com/pathwaycom/serviette/blob/84e92a33728cf843026f4011d86568b403e38fb4/serviette/indexer/fingerprint.py#L145-L225) | `build_fingerprint`/`check_fingerprint`: config y versiones condicionan replay; drift requiere confirmacion |
| [S5 main.py:1-65](https://github.com/pathwaycom/serviette/blob/84e92a33728cf843026f4011d86568b403e38fb4/serviette/server/main.py#L1-L65) | API de retrieve/RAG y UI same-origin; resultados con texto/metadata/score |
| [S6 hybrid.py:1-55](https://github.com/pathwaycom/serviette/blob/84e92a33728cf843026f4011d86568b403e38fb4/serviette/server/hybrid.py#L1-L55) | `KeywordHybridMixin`: BM25 local + RRF, refresh por señal y timer; limite de cardinalidad degrada a vector |
| [S7 documents.py:1-65](https://github.com/pathwaycom/serviette/blob/84e92a33728cf843026f4011d86568b403e38fb4/serviette/server/documents.py#L1-L65) | Catalogo, lectura de documento y diff con presupuesto de caracteres; no entidad formal de version |
| [S8 main.py:517-630](https://github.com/pathwaycom/serviette/blob/84e92a33728cf843026f4011d86568b403e38fb4/serviette/server/main.py#L517-L630) | Contextos por documento/comparacion y salida con fuentes/aviso de presupuesto |

La tabla fuente retiene metadata, no todos los bytes. El parseo memoizado permite
retirar resultados anteriores aun cuando el fichero ya no exista. El splitter
determinista puede recomputarse sin almacenar una segunda copia completa; para
otros splitters se conserva memoizacion. `make_id` combina metadata canonica de
documento/version y texto; excluir timestamp de observacion evita cambios de ID
por un simple sondeo. No equivale a tener historia inmutable de documentos:
S7 dice expresamente que dos versiones se tratan como dos documentos.

El contexto documental usa caracteres y reconoce cobertura parcial; fuentes
devueltas no certifican que cada afirmacion generada este sustentada. S6 tampoco
ofrece frescura estrictamente linealizable de la pata BM25: durante refresh
puede servirse el indice previo.

### Ficha candidata A — fingerprint de reproducibilidad

- **Objetivo / unidad minima:** impedir bajas calculadas con un splitter distinto
  al usado para las altas. `build_fingerprint` y `check_fingerprint` (S4:145-225).
- **Seam Vanta:** `src/wiki/chunker.rs`, `src/wiki/state.rs`,
  `src/wiki/ingestors.rs`; no crear un segundo pipeline de ingestion.
- **Contrato propuesto:** `DerivationFingerprint { parser, splitter, embedding,
  normalization, implementation_version }`; `inspect_compatibility`,
  `begin_generation`, `rebuild_required`. Secretos excluidos del digest.
- **Acciones:** identificar IDs/versiones existentes; persistir fingerprint con
  generacion; rechazar replay incompatible y ofrecer rebuild explicito en Studio.
- **Dependencias:** serializacion canonica estable, registro de fuentes,
  migracion y procedencia. Ningun servicio obligatorio.
- **FMEA:** libreria cambia silenciosamente → chunks huerfanos; identidad incluye
  credenciales → filtracion; fingerprint ilegible → falsa compatibilidad. Fallar
  cerrado conservando diagnostico, no copiar el rewrite automatico de S4:199-204.
- **Aceptacion pendiente manual:** cambio de splitter exige nueva generacion;
  rotar clave no invalida; restart identico mantiene IDs; fingerprint corrupto
  no habilita replay. **No objetivos:** fijar versiones globales por capricho o
  reproducir Pathway entero.

### Ficha candidata B — manifest de derivados y retirada incremental

- **Objetivo / unidad minima:** materializar deltas sin releer un documento ya
  borrado. `build_graph`/`parse_document`/`make_id` (S1/S2), no el runtime dataflow.
- **Seam Vanta:** wiki store/state, lifecycle/supersession de
  `vanta-memory/src/core/record/`, scheduler y checkpoints existentes.
- **Contrato propuesto:** `SourceDelta { scope, source_id, revision, kind }` y
  `DerivedManifest { revision, chunk_ids, fingerprint }`; acciones `plan_delta`,
  `publish_revision`, `retract_revision`, `inspect_pending`.
- **Acciones:** contrastar manifest actual; almacenar conjunto de IDs publicado;
  retirar por ese conjunto y revision; publicar sucesora sin mezclar generaciones.
- **Dependencias:** transaccion e indices consistentes; embedding opt-in;
  reconciliacion reutilizando scheduler, no Redis/Pathway en core.
- **FMEA:** baja repetida elimina revision nueva; parse fallido se confunde con
  documento vacio; update parcial deja indice mixto. Fence por revision, estado
  de error distinto y publicacion de manifest consistente.
- **Aceptacion pendiente:** alta/edicion/baja/replay; retirada sin bytes fuente;
  parser fallido conserva diagnostico y no finge exito; revision vieja no borra
  nueva. **No objetivos:** watch universal de todos los conectores o semantica
  distribuida exactly-once no demostrada.

## 3. Defectos, riesgos e issues transferibles, con su nivel de evidencia.

**Codigo:** en S2:578-588 un `ParseError` termina en texto vacio, log y necesidad
de tocar/re-subir para reintentar. Riesgo operativo: un documento no aparece y
un consumidor confunde ausencia con contenido irrelevante. No demuestra perdida
del original. S4 permite aceptar drift; fingerprint ilegible se reescribe. Son
fronteras de seguridad de datos a no normalizar como simple warning.

**Hipotesis de seguridad:** servidor de corpus sin cuentas integradas,
publicado fuera de una red confiable, puede exponer fuentes a usuarios no
previstos; no se audito middleware completo ni se demuestra explotabilidad.
Los fetchers cruzan fuentes externas → host del indexer; permisos de filesystem/
credenciales deben limitarse. La primera pasada no consultó issues; §12 añade
las incidencias públicas efectivamente leídas y su vigencia limitada.

## 4. Prerrequisitos en VantaDB antes de incorporar la idea.

Revalidar fingerprints, dedup, wiki incremental y supersession actuales.
Definir atomicidad del manifest con BM25/HNSW; no basta borrar solo vectores.
Precisar fuente vacia frente a parser fallido. Reusar assembly con presupuesto
compartido en vez de adoptar limites de caracteres como presupuesto de tokens.
Revision legal separada de wrapper, Pathway y conectores.

## 5. Componentes y practicas que no deben trasladarse.

No adoptar Python/Pathway como dependencia obligatoria del motor Rust; ni su
licencia/key de runtime sin revision. El [README](https://github.com/pathwaycom/serviette/blob/84e92a33728cf843026f4011d86568b403e38fb4/README.md#L20-L90)
exige `PATHWAY_LICENSE_KEY`; SharePoint tiene condiciones adicionales descritas
en S3. No replicar BM25 en RAM ni degradacion silenciosa de hybrid donde Vanta ya
tiene indice textual. No reemplazar memoria de sesiones por catalogo documental.

## 6. Advertencias metodologicas.

**Ledger:** rangos S1-S8, pyproject, LICENSE, README y runners/documentacion de §7
leidos. No inspeccionados todos los ocho accessors, implementacion interna
Pathway, autenticacion completa ni fuentes remotas en ejecucion. La garantia
de retraction depende del conector/sink y persistencia configurados; no se
certifica end-to-end. No hay pruebas, installs, servicios ni UI ejecutados.
Pendiente manual: edit/delete/restart, costes, permisos y cobertura de citas.

## 7. Benchmarks existentes: runner, entrada, metricas, dependencias y riesgos.

- [FRAMES run_bench.py:1-70](https://github.com/pathwaycom/serviette/blob/84e92a33728cf843026f4011d86568b403e38fb4/benchmarks/frames/run_bench.py#L1-L70):
  preguntas JSONL → `/rag` → autorater upstream; accuracy y recall de articulos
  gold en contextos utilizados. Juez configurable, default `gpt-4o-mini`; texto
  del juez upstream no se reproduce ni modifica. Requiere servidor, corpus
  indexado y API OpenAI para juez. Tokens del juez registrados; generacion vive
  en config del servidor, embeddings dependen del proveedor. No interpretar el
  coste del juez como coste total. No acredita limpieza automatica del indice.
- [Indexacion realtime README:1-62](https://github.com/pathwaycom/serviette/blob/84e92a33728cf843026f4011d86568b403e38fb4/benchmarks/realtime-data-indexing/README.md#L1-L62):
  compose Qdrant/indexer/server, corpus preparado y embeddings locales
  `static-retrieval-mrl-en-v1` truncados a 256 dimensiones. Mide tiempo estimado
  por maximo del contador aproximado y PSS muestreado del indexer; excluye RAM
  de Qdrant/API, kernel y cache no mapeada. Exact-count comprueba completitud,
  no instante de finalizacion HNSW. Preparacion y descarga de modelo fuera del
  tiempo. Sin juez ni API de pago en esa pista; CPU/disco no son coste cero.
  Runners/compose localizados, no ejecutados ni auditados integralmente:
  limpieza de volumen/dataset debe aprobarse sobre entorno desechable.

No son resultados propios ni pruebas de memoria conversacional.
Aplicar solo [METHOD](METHOD.md#protocolo-de-comparacion-de-benchmarks).

## 8. Matriz existe/parcial/no acreditado/no aplicable frente a VantaDB.

| Superficie Vanta | Estado baseline | Contraste util |
|---|---|---|
| BM25/HNSW/RRF y procedencia | Existe | No duplicar hybrid en RAM |
| Sesiones, supersession, dedup opcional | Existe | Serviette no demuestra memoria personal equivalente |
| Assembly/compactacion compartida | Existe | Caracteres upstream no sustituyen tokens |
| Wiki, scheduler, checkpoints | Existe | Fingerprint y manifest integral: no acreditado aqui |
| Studio Tauri: conexiones/memoria | Existe | Diagnosticar frescura por fuente |
| Studio: escenas/skills/actividad/retrieval | Existe | Estado parser/error/generacion como extension |
| Cuentas multiusuario Serviette | No aplicable como modelo de Vanta | Producto upstream declara no integradas |

## 9. Debilidades de VantaDB sugeridas por el contraste, no bugs inventados.

Revisar si reindexar con otro chunker deja claro el cambio de generacion y si
la UI diferencia «fuente sin texto» de «parseo fallido». Son preguntas de contrato,
no afirmaciones de bug. La base actual ya cubre ingestores, dedup y recuperacion;
no se promete mejora de recall, memoria ni latencia sin comparacion autorizada.

## 10. Escritorio, interfaz e inspeccion: implementado, externo o no localizado.

UI HTML en `serviette/frontend/static/index.html`; servidor same-origin S5,
frontend separado opcional. No se localizo shell desktop nativo en la superficie
revisada. README declara sin hosted offering, cuentas ni memoria conversacional
integradas; no inferir estas capacidades de un demo de chat. Para Vanta usar
Studio existente, no recomendar otro chat/consola ni servidor obligatorio.

## 11. Cierre de autoría: servidor, materialización y contratos completos

**Autoría entregada; revisión independiente y aceptación pendientes.**
Fecha de lectura 2026-11-07; pin upstream y captura de §1 se conservan.
Comparación Vanta fijada a **`97c9623c8ef5a70fdb51aff215bfbb3b05b15612`**,
no estado histórico dirty ni cambios concurrentes. No se ejecutó software.

### 11.1. Stack, modelo y tres flujos con fronteras

Mapa: `config/` valida YAML/modelos, `indexer/sources.py` crea conectores,
`fetchers`/registry de parsers obtiene texto, `graph.py` construye dataflow,
`prepare.py` prepara backend, `server/accessors/` consulta ocho destinos,
`embedder.py`/`llm.py` son adaptadores asíncronos, `hybrid.py` añade BM25,
`documents.py` resuelve catálogo/lectura/diff, `frontend/static/index.html`
es cliente real; `up.py` supervisa procesos y `cli.py` es CLI.
[pyproject.toml:1–125][S9] declara 0.1.8, Python >=3.10, Pathway xpack-llm,
FastAPI/Uvicorn/httpx/Pydantic, DuckDB y OpenAI; pypdf/unstructured para formatos
comunes; extras para conectores, OCR/Docling, proveedores y bases.
No hay SDK cliente de memoria separado acreditado en ese paquete: `create_app`
es API embebible de servidor; consumidores usan HTTP JSON. No confundir
`AsyncVectorAccessor` interno con SDK público durable de ingestión.

**Modelo:** tabla por fuente contiene `_metadata`; tras parse/split/flatten
cada fila tiene chunk_id, texto, metadata y embedding. `_metadata_as_json`
conserva campos; `_metadata_key_json:734–761` excluye sólo `seen_at`.
`CHUNK_ID_SCHEME=2` identifica derivación: ID SHA256 combina metadata de versión,
separador nulo y texto. `seen_at` es observación, no revision/source-of-truth;
cambiar otro campo produce nueva identidad [S10]. Repetir texto dentro del
mismo documento/metadata puede dar igual ID: no es identidad de ocurrencia
ni offset. No se atribuye pérdida sin estudiar multiplicidades del sink.

**F1 — fuente→materialización.** `build_graph`/`make_parse_udf:550–668`:
conector only_metadata → fetch con retries → `registry.parse` memoizado →
splitter (whitelist determinista) → flatten → embed de texto con prefijo
opcional → `_write_sink`. Texto almacenado e ID no incluyen prefijo de embedding.
Un `ParseError` devuelve vacío y log; no existe en ese tramo un resultado
tipado persistido “parser falló”. `split_text("")` produce cero chunks:
la consulta no puede distinguir ese fallo de una fuente legítimamente vacía.
Fetch/transforms memoizados permiten retract sin reabrir bytes eliminados [S2].
`_write_sink:673–719` selecciona formato por backend; Qdrant omite chunk_id
porque su conector lo gestiona internamente. No imponer idéntica identidad
física ni atomicidad a todos los ocho destinos.

**F2 — consulta→respuesta.** `create_app:146–195` construye embedder/accessor;
LLM y reranker son opcionales. `_search:335–384` hace decompose opcional →
pool vector/hybrid → interleave por subconsulta (no sumar scores ni RRF entre
subconsultas) → rerank/MMR opt-in → top-k; quita embeddings de salida.
`retrieve:386–388` no necesita answerer; `/rag` sin LLM responde 501.
Modo documental resuelve nombres y usa budget por request; search simple
sin catálogo sigue otra rama [S11]. Resultado declara `answer`, `sources`,
`mode`, `documents`, `notice`; una lista de fuentes no demuestra entailment.
`IndexNotReadyError` se traduce en handler propio (`main.py:290–313`);
readiness no es health del proceso ni ausencia semántica de hits.

**F3 — edición/baja/reinicio/cancelación.** Metadata modificada emite retract
de fila previa + alta; misma función pura de ID debe retirar derivado viejo.
Cambiar splitter/esquema sin fingerprint compatible deja esa prueba inválida.
`run_indexer:976–1028` verifica fingerprint antes de preparar backend, construye
grafo y llama `pw.run` con persistencia; cache UDF a disco depende de capacidad
de la versión Pathway. Directorio de persistencia es del runtime, no manifest
universal de Vanta [S12]. Los commits los realizan conectores/sinks, no
`create_app`; no se acreditó transacción documento→todos los índices.
`up.py::run:534–643` atiende SIGINT/SIGTERM y termina hijos en finally; es
cancelación del proceso supervisor, no API por documento ni rollback.
El router `main.py:698–703` expone health/retrieve/rag/stats/documents/config,
no endpoint público cancel-ingest en esa superficie. Desconectar HTTP no
acredita detener parse/indexer ni llamada remota; son owners distintos.

### 11.2. A/B: cierre de los once campos de §2

| Campo complementario | A — fingerprint reproducible | B — manifest/retraction |
|---|---|---|
| Delta frente a existente | `src/wiki/chunker.rs::chunk_text` e ingestors ya transforman; fingerprint interoperable de replay completo no acreditado | Linaje/supersession existen; ownership de todos los derivados por revisión y retirada sin bytes no acreditados integralmente |
| Alternativas | Extender metadata de ingestor/estado actual o adapter host; rechazar invalidar por clave secreta/runtime completo | Extender provenance/manifest existente; transacción sólo si cubre la unidad; rechazar nuevo sink/runtime/WAL |
| Rust **propuesto** inputs/outputs | `inspect_compatibility(source_id, expected_revision, stored, current) -> Result<ReplayCompatibility, FingerprintError>`; salida Compatible/RebuildRequired/UnknownEvidence | `plan_delta(SourceDelta, expected_revision, manifest) -> Result<DeltaPlan, DeltaError>`; `apply(plan) -> DeltaPublication`, con IDs retirados/publicados y pendientes |
| Errores/invariantes | `CorruptFingerprint`, `UnknownLegacy`, `RevisionConflict`, `ReadFailed`; digest sin secretos y mismo espacio vectorial, no sólo dimensión | `Denied`, `MissingManifest`, `RevisionConflict`, `ParseFailed`, `ProjectionPending`, `Cancelled`; vacío confirmado distinto de lectura fallida |
| Seam real | `src/wiki/ingestors.rs::Ingestor:45–89`, `src/wiki/chunker.rs::chunk_text:19`, `src/wiki/state.rs::WikiState` [SV1] | `vanta-memory/src/ingest/merge.rs::commit:239–289`, `core/record/lifecycle.rs`, `utils/task_checkpoint.rs::TaskCheckpointManager` [SV2] |
| Pasos/migración/rollback | Inventariar identidad actual → serialización canónica → persistir por generación → compatibilidad antes de replay. Legacy unknown requiere evidencia/rebuild explícito. Rollback deshabilita producción nueva, conserva fingerprint anterior | Capturar manifest antes de perder fuente → validar expected revision → plan → publicar por backend existente → proyectar/retirar → avanzar checkpoint. Campos aditivos; no inventar manifest legacy. Rollback detiene trabajo nuevo, termina/repara operaciones ya iniciadas |
| Owner/cancel/concurrencia | Worker posee generación; I/O/modelo fuera de lock; check de revisión antes de publicar, no check-then-write separado como garantía | Una operación identificada por fuente/revisión; serializar/CAS existente, cancel conserva prefijo aplicado y diagnóstico; retry viejo no elimina sucesora |
| FMEA/detección | Cambio de librería → IDs distintos al replay → huérfanos; detectar fingerprint y forzar decisión, no sobrescribir corrupto como compatible | Parse fallido en update → cero chunks retira válidos → pérdida de disponibilidad; detectar resultado C antes de promover; conservar generación previa según política explícita |
| Aceptación futura/observabilidad | Normal restart idéntico; adversa drift/corrupto/rotación de clave; digest/revisión/estado, sin valores de config sensibles | Normal alta/update/delete sin bytes; adversa cancel/replay/crash/sucesora; operación/fase/conteos/error sin texto |
| Mínimo/no-goals/abierto | Un ingestor, sin versionar entorno entero; lista canónica de campos/model identity abierta | Un backend y derivados actuales, sin exactly-once de red; visibilidad y retención de generaciones abiertas |

### 11.3. C — lifecycle de parse/materialización y cancelación observable

1. **Problema agente:** una captura aceptada no aparece en recall; usuario necesita
   saber si está vacía, falló parser, espera índice o pidió cancelar.
2. **Fuente/unidad:** `ParseError→""` [S2], runtime/persistencia [S12],
   stats `main.py::_IndexChangeTracker:97–143` y supervisión `up.py:371–449`.
   No tomar logs heurísticos como una máquina durable ya implementada.
3. **Delta:** Vanta tiene `IngestProgress`, `ProgressTracker`, checkpoints y
   Studio existente; jobs UI no acreditados al pin, ampliar fase/resultados
   como propuesta sobre worker/checkpoints, no inventar lifecycle.
4. **Alternativas:** extender reportes actuales, adapter que traduzca estados de
   parser o UI sólo con diagnóstico unknown; rechazar cola/scheduler adicional.
5. **Rust propuesto:** `record_stage(operation_id, source_revision,
   expected_phase, outcome) -> Result<IngestStageView, StageError>`; outcome
   separa ParsedEmpty/Parsed/ParseFailed/ProjectionPending/CancelRequested.
   Errores `RevisionConflict`, `OutOfOrder`, `StorageFailed`, `UnknownOperation`;
   solicitado ≠ cancelado, parseado ≠ visible, health ≠ ready.
6. **Seam real:** `ingest/worker.rs::{begin,execute}`,
   `ingest/callback.rs::{IngestProgress,ProgressTracker::update_progress}`,
   `TaskCheckpointManager::{advance,fail}`; presentación propuesta en Studio,
   sin atribuir JobChip al pin (véase SE-1, §12.4).
7. **Pasos/persistencia:** inventariar enums actuales → mapear transiciones y
   resultados parciales → persistir sólo lo que checkpoint soporte realmente →
   propagar a SDK/Studio → reconciliar fase al retry. Migración aditiva unknown
   para legado; rollback oculta nueva vista, nunca reetiqueta fallo como éxito.
8. **Dependencias:** C9 transversal, B para visibilidad, A/C4 para revisión,
   C16 UI. No exige modelo ni cambio del motor.
9. **FMEA:** parser excepciona → vacío → corpus parece completo; detectar outcome
   antes de publicar y exponer fallo. Cancel tardío → commit realizado → UI
   dice revertido; detectar applied IDs/revisión y mostrar parcial. Stats
   regresan tras delete → timestamp falso de frescura; separar observación local
   de checkpoint autoritativo, sin convertir wall clock en revisión.
10. **Aceptación futura:** normal captura→parse→índice; adversa vacío real,
    parser roto, fallo al persistir estado, cancel antes/después de chunk y dos
    runs. Telemetría: IDs/fases/conteos/latencia/error class, sin nombres/texto.
11. **Mínimo/no-goals/abierto:** un parser y worker existente; no detener procesos
    globales para cancelar un documento. Retención de errores y garantías del
    checkpoint al crash requieren decisión; no se presume durabilidad de UI jobs.

### 11.4. D — evidencia documental parcial con presupuesto compartido

1. **Problema:** comparar revisiones extensas sin presentar un prefijo como si
   representase todo el documento ni gastar el budget dos veces en solapes.
2. **Fuente/unidad:** `documents.py` catálogo/lectura/diff [S7] y
   `main.py::_answer_documents:552–630` [S8]; `RagResponse.notice/mode` [S11].
3. **Delta:** budgets de recall/assembly existen; añadir cobertura de documento
   y modo comparativo ligado a revisión, no otro context assembler.
4. **Alternativas:** extender evidencia actual con omitted spans, host documental
   opt-in o sólo retrieve; rechazar nuevo diff semántico/modelo obligatorio.
5. **Rust propuesto:** `document_evidence(source_refs, expected_revisions,
   scope, remaining_budget) -> Result<DocumentEvidenceView, EvidenceViewError>`;
   salida spans incluidos/omitidos, unidad de presupuesto, coverage y modo.
   `Denied`, `AmbiguousSource`, `StaleRevision`, `ReadFailed`, `BudgetExhausted`;
   no llamar completo a truncado ni usar nombre como autorización.
6. **Seam:** `CandidatePage.sources`, `RecallScope`/`InjectionPolicy`,
   `assemble_with_recall`/`IntegratedContext`; inspección de fuente en Studio.
7. **Pasos:** resolver IDs/revisión → cargar sólo autorizado → preservar spans
   al recortar → ensamblar en ruta actual → emitir coverage tras render.
   Persistencia opcional de vista, no copia del documento; legacy unknown.
   Rollback desactiva comparación y conserva retrieval ordinario.
8. **Dependencias:** C4/B para revisiones; C7 presupuesto; C14 stale, C16 UI.
   Independiente del dataflow y de backend externo de Serviette.
9. **FMEA:** versión homónima → mezcla documentos → diff incorrecto; detectar
   identidad no única y exigir selección. Cap caracteres tratado como tokens →
   ventana desborda; declarar estimador y budget real existente.
10. **Aceptación futura:** normal dos revisiones autorizadas; adversa truncado,
    Unicode, versión retirada, lectura IO y presupuesto insuficiente; observar
    IDs/revisiones/span counts/coverage, no preguntas, diferencias o contenido.
11. **Mínimo/no-goals/abierto:** lectura de una o dos fuentes, sin chat nuevo ni
    pesos/umbrales de evaluación. Política de conservación/cobertura por formato
    y ubicación de summaries quedan abiertas.

## 12. Incidencias, benchmarks, matriz y Studio

### 12.1. Issues públicos leídos y vigencia

Consulta 2026-11-07: `websearch.search` DDG/Bing
`site:github.com/pathwaycom/serviette/issues parse ingestion cancel` devolvió
vacío y Bing 301. Fallback real: `fetchWebContent` leyó listado oficial de
issues y cuerpos/actividad de #53/#46 sin truncamiento. No se agotó en un
endpoint vacío ni se inventó un issue moderno de cancelación.

| Issue / estado observado | Causa/revisión/fix acreditados | Aplicabilidad |
|---|---|---|
| [#53, connector Jsonlines](https://github.com/pathwaycom/serviette/issues/53), Open, creado 2023-08-15 | Reporte de mapeo de objetos anidados a schemas; comentario atribuye a soporte del conector de entonces. No SHA, versión ni fix acreditado en página | **Histórico**, no defecto demostrado del pin 2026. A/B no dependen de afirmar que Pathway actual rechaza JSON anidado. Validar schema/metadata de conector concreto, no inferirlo del título |
| [#46, streaming OpenAI](https://github.com/pathwaycom/serviette/issues/46), Open, 2023-08-08 | Solicitud sobre antiguo wrapper “LLM App”; no repro, versión ni corrección enlazada acreditados | No prueba que proveedor actual carezca de streaming. El cliente actual leído sí consume JSON completo [S14]; cancelar stream/ingesta son asuntos distintos |

La antigüedad es un límite de la evidencia pública hallada, no permiso para
presentarla como incidencia actual. Para el fallo parse→vacío hay código
actual S2, pero **no se localizó issue público específico** en el listado
recorrido; se conserva como observación estática, no reporte upstream.
No se acreditan CVEs, merge ni arreglos desde estado Open/Closed.
Frontera de confianza: metadata/bytes externos → fetch/parser del indexer y
API red → catálogo/textos. Scope por consulta no concede permisos de fuente;
autorización integral, SSRF de cada fetcher y deployment no auditados.

### 12.2. Runners: denominadores, costes y efectos concretos

**FRAMES** [S15:57–139]: defaults port 8987, k=8, concurrency=6,
limit=0 (todas), judge `gpt-4o-mini`. JSONL usa prompt/answer/wiki_links;
`ask` llama `/api/v1/rag` con timeout 900 y retries=2.
Gold se convierte a conjunto de slugs; recall = intersección/gold y **gold
vacío devuelve 1.0**. Juez recibe pregunta/respuesta/gold y el código reconoce
`"TRUE"` o sufijo `TRUE`, no una validación semántica independiente.
Accuracy y recall promedian `judged`; excepción no capturada en pool puede
abortar antes del agregado/output, no se declara error como miss automático.
Tiempo answer incluye HTTP/generación de todas las preguntas concurrentes,
no índice preparado ni fase juez; judge_usage sólo registra tokens del juez.
No split holdout/seed de generación acreditado en ese runner. Escribe JSONL
con predicción/gold/fragmento del juez; contiene datos sensibles potenciales.
No borra corpus ni arranca servidor en ese archivo; requieren preparación
externa. Adaptación Vanta: pregunta→recall autorizado→fuentes con identidad de
artículo→answerer del host; no sustituir artículo gold por sesión ni cambiar juez.

**Realtime** [S16]: runner realmente leído en esta ampliación, no sólo README.
`main:288–335` defaults size=100mb, top-k=5, puertos 8300/8301, keep-up false;
lee `datasets/<size>/docs` y questions.json `{query,expected_file}`.
Valida configuración de proveedores sin coste pagado declarado y prepara
modelo/cache fuera de medida, pero recursos locales cuestan CPU/RAM/disco.
Antes de arrancar ejecuta **compose down -v**, destructivo sobre esos volúmenes.
Escribe compose resuelto redaccionando licencia, samples y reportes.
`_accuracy:226–250` cuenta pregunta hit si cualquier metadata.path termina en
`"/"+expected_file`; denominador = todas las preguntas, HTTP error se propaga.
No mide QA/juez, abstención ni recall de todos los documentos relevantes.
`main:393–416` aborta ante parse failure, lector de progreso roto o indexer
caído; stats no disponible **no** prueba inactividad. Contadores aproximados,
actividad CPU/parse y comprobación exact-count tienen funciones distintas;
no datan construcción completa del HNSW ni crash safety.
PSS muestreado sólo cubre proceso indexer según README, no Qdrant/API/cache
del kernel. Repeticiones/percentiles de latencia no se infieren de ese reporte.
Adaptación Vanta: mapear ingestor→conteo de derivados visibles y
`expected_file`→fuente revisionada; conservar pista motor/retrieval separada
de FRAMES. Runtime, compose, reset y descargas **no ejecutados**.
Aplicar [METHOD](METHOD.md) y [notas](BENCHMARK-NOTES.md), sin diseñar
harnesses, prompts, scoring, thresholds, corpus ni configs nuevos.

### 12.3. Matriz específica y negativos Vanta

| Capacidad | Upstream acreditado | Vanta pin 97c9623: fuente/estado | Delta / decisión |
|---|---|---|---|
| Híbrido | BM25 RAM/RRF y refresh S6 | Existe `src/sdk/search/{hybrid,fusion}.rs`, HNSW/BM25/RRF | No copiar índice duplicado ni early fallback silencioso |
| Identidad/derivados | S10 excluye seen_at, fingerprint S4 | Existe provenance, `Ingestor`, `chunk_text`; integral fingerprint no acreditado | A/C4/C14 sólo extienden contrato |
| Memoria de agente | Documentos, no historial autobiográfico | Existen sesiones, dedup/supersession `core/record/` | No sustituir dominio por catálogo |
| Budget/cuotas | Budget chars documental S8/S11 | Existen `assemble_with_recall`, `apply_recall_budget`, `src/config.rs:760–762` | D/C7 conserva unidades; cuota bytes no budget LLM |
| Lifecycle | Parse vacío, logs/runtime S2/S12 | Existen worker/callback, scheduler/checkpoints SV2 | C/C9 amplía resultado, no nuevo scheduler |
| Borrado/recuperación | Retractions por conector, no commit global | Existe lifecycle/provenance; cobertura integral de derivados parcial | B valida operación y revisión |
| Evidencia de escritura | Materialización por sink | Existe WriteReceipt SV3 | No confundir ACK/captura/visible |
| UI | HTML same-origin S14 | Existe Studio SS1; jobs no acreditados al pin | C16 propone fases/resultado en inspector, no servidor nuevo |

`WriteReceipt` [SV3] vincula versión/contenido store/WAL mediante SHA256:
**no firma digital** y no cubre toda proyección/linaje ni entrega al modelo.
`MutationReceipt` para fase/visibilidad sería **propuesto**; no afirmar que
Vanta carece de recibos ni convertirlo en otra WAL.
Negativos propios: no bug nuevo acreditado; fingerprint/retraction integral
y separación de errores por todos los ingestors quedan no acreditados,
no ausentes. Se descartan como falsos positivos “sin budget”, “sin provenance”,
“sin scheduler/Studio” y extrapolar un conector de 2023 a Vanta actual.
Orden: A/B antes de habilitar replay nuevo; C en paralelo sobre worker actual;
D sólo si se necesita comparación documental. C no requiere Pathway ni LLM.

### 12.4. Interfaz real, Studio y cierre

Cliente web [S14:422–458] hace POST `/api/v1/rag`, renderiza answer, notice y
sources o error, y refresca stats. `index.html:269,292–306` consulta catálogo,
config y stats; busy es estado del formulario, no task durable.
No usa `AbortController` en esa llamada leída; no convertir cerrar página en
cancelación de proveedor/indexer. `main.py:717–721` sirve HTML same-origin,
frontend standalone es opción de despliegue; no se acredita app desktop/SaaS.

Studio: cuerpo releído mediante `git show` del pin separado
**`0ad38c542b39a589035b0e17fdedb4c09dca24b9`**, ruta
`feature-freez/desktop/src/components/memory/MemoryLens.tsx` [SS1].
`ScenesPanel:57-96`, `PersonaPanel:172-206`, `SkillsPanel:266-297` y
`GenlogPanel/openAnchor:373-412` cargan escenas/snapshot/skills/genlog y
abren registro real mediante `get(anchorId)`. **SE-1:** JobChip/jobs no están
en el árbol de ese SHA; aparecen `??` bajo `desktop/` en disco. No se atribuyen
sus bytes/rangos ni cancelación/IDs parciales al pin; SS2/SS3 retirados.
Presentar cancel/progreso de jobs queda **propuesto**, no UI shipped acreditada.
C16 debería presentar revisión, parser outcome, proyección y timestamp de
observación separado del checkpoint, no afirmar “todo actualizado” desde health.
FMEA/aceptación de C (§11.3) siguen futuras: request de cancel no significa
rollback; fallo/restart debe conservar resultado real y parcialidad por owner,
sin esconder ParseFailed como vacío. Durabilidad del registro UI no acreditada.

**Fuentes/cobertura/límites:** S1–S16 leídos focalmente, drivers/handlers/HTML,
dos incidencias públicas, contratos y seams Vanta por lectura pin; no todos
los accessors ni implementación Pathway/connector, SBOM/model licenses, auth
end-to-end, tests o runtime. MIT de wrapper no licencia Pathway xpack, extras,
datos ni modelos; reescribir en Rust no resuelve derivación. Si se aprueba,
separar specs A/B (replay), C (lifecycle), D (evidencia/UI); aquí quedan los diez
ejes autónomos. No se crean specs, backlog ni infraestructura de evaluación.
SE-1 propagado a C, matriz y UI; revisión independiente posterior, no autoaceptación.

[S9]: https://github.com/pathwaycom/serviette/blob/84e92a33728cf843026f4011d86568b403e38fb4/pyproject.toml#L1-L125
[S10]: https://github.com/pathwaycom/serviette/blob/84e92a33728cf843026f4011d86568b403e38fb4/serviette/indexer/graph.py#L721-L761
[S11]: https://github.com/pathwaycom/serviette/blob/84e92a33728cf843026f4011d86568b403e38fb4/serviette/server/main.py#L335-L422
[S12]: https://github.com/pathwaycom/serviette/blob/84e92a33728cf843026f4011d86568b403e38fb4/serviette/indexer/graph.py#L976-L1028
[S14]: https://github.com/pathwaycom/serviette/blob/84e92a33728cf843026f4011d86568b403e38fb4/serviette/frontend/static/index.html#L422-L458
[S15]: https://github.com/pathwaycom/serviette/blob/84e92a33728cf843026f4011d86568b403e38fb4/benchmarks/frames/run_bench.py#L57-L139
[S16]: https://github.com/pathwaycom/serviette/blob/84e92a33728cf843026f4011d86568b403e38fb4/benchmarks/realtime-data-indexing/run_bench.py#L226-L423
[SV1]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/wiki/ingestors.rs#L45-L89
[SV2]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/ingest/worker.rs#L35-L110
[SV3]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/attestation.rs#L654-L738
[SS1]: https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/memory/MemoryLens.tsx#L57-L412

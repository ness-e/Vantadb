---
title: "Serviette: cambios incrementales y retractions sobre documentos vivos"
kind: research
status: proposed
description: "Analisis estatico del dataflow Pathway, identidad de chunks y limites de memoria de Serviette frente a VantaDB"
tags: [research, memory, engineering]
---

# Serviette: cambios incrementales y retractions sobre documentos vivos

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
credenciales deben limitarse. Issues remotos **no consultados**.

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

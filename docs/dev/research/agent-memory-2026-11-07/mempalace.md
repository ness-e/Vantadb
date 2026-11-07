---
title: "MemPalace: corpus literal, contratos de backend y limites de recall"
kind: research
status: proposed
description: "Estudio estatico del corpus verbatim, recuperacion, backends Python y Rust y runners de MemPalace frente a VantaDB"
tags: [research, memory, engineering]
---

# MemPalace: corpus literal, contratos de backend y limites de recall

Checkout verificado con `git rev-parse HEAD` y fuentes presentes:
`C:\Users\Eros\mempalace`, `MemPalace/mempalace`, rama `develop`,
SHA `f8e35362189bd0a5abf4c0df1dd5bf90ba059176`.
**60 446 estrellas**, MIT, no archivado: [snapshot](github-candidates.json);
sin reconsulta. Preparación/omisiones en [manifest](clone-manifest.json).
Estudio estático bajo [METHOD](METHOD.md), sin resultados propios.

**Contenido:** [1. Arquitectura](#1-recomendaciones-para-vantadb-y-contexto-arquitectonico-del-proyecto) ·
[2. Extracción](#2-partes-extraibles-flujo-real-lectura-de-codigo-y-mapeo-a-rustvantadb) ·
[3. Riesgos](#3-defectos-riesgos-e-issues-transferibles-con-su-nivel-de-evidencia) ·
[4. Prerrequisitos](#4-prerrequisitos-en-vantadb-antes-de-incorporar-la-idea) ·
[5. No trasladar](#5-componentes-y-practicas-que-no-deben-trasladarse) ·
[6. Método](#6-advertencias-metodologicas) · [7. Benchmarks](#7-benchmarks-existentes-runner-entrada-metricas-dependencias-y-riesgos) ·
[8. Matriz](#8-matriz-existeparcialno-acreditadono-aplicable-frente-a-vantadb) ·
[9. Contraste](#9-debilidades-de-vantadb-sugeridas-por-el-contraste-no-bugs-inventados) ·
[10. Interfaz](#10-escritorio-interfaz-e-inspeccion-implementado-externo-o-no-localizado).

## 1. Recomendaciones para VantaDB y contexto arquitectonico del proyecto

**Dictamen:** preservar evidencia literal y hacer explícitos los contratos de
backend son ideas útiles; la metáfora del palacio no justifica reemplazar la
memoria semántica existente. Vanta ya tiene L0, procedencia, sesiones, ingestors,
escenas, embeddings y contexto con presupuesto compartido.

| Componente | Stack y responsabilidad |
|---|---|
| CLI/MCP | Python >=3.10; ingest, consulta e integración de agentes |
| Palacio | Wings/personas/proyectos; rooms/agrupaciones; drawers/texto original |
| Índice auxiliar | Closets/punteros a drawers; señales de ranking, no fuente literal |
| Persistencia | Chroma por defecto; contrato `BaseBackend`/`BaseCollection`; alternativas SQLite, Rust exact y remotas |
| Grafo | Módulo SQLite separado anunciado por documentación; no auditado en profundidad |
| Núcleo nativo | `crates/mempalace-core`, rusqlite y Rayon; PyO3 en crate de binding |
| Presentación | VitePress/Vue y landing; no confundir sitio con explorador del corpus |

El README anuncia **96,6% R@5 raw** y cero API; es un claim publicado de
retrieval, no exactitud QA ni medición de este estudio. «Verbatim» significa
evitar paráfrasis del corpus: la ruta live leída hace `.strip()`, de modo que
no promete preservación byte a byte de whitespace [M3].

## 2. Partes extraibles, flujo real, lectura de codigo y mapeo a Rust/VantaDB

### Recorrido comprobado

1. **Write/retain:** `file_conversation_exchange` valida wing/room, asigna
   metadata canónica, fechas, agente, identidad de origen y `extract_mode`;
   `upsert` conserva texto del intercambio sin abstracción LLM [M3].
2. **Filing/mining:** `write_file_drawers` escribe nuevos lotes antes de eliminar
   stale drawers del mismo archivo/modo. Marca conjunto incompleto mediante
   `chunk_total`, purga stale y solo entonces finaliza metadata [M4].
3. **Extract/consolidate:** closets y clasificación organizan acceso, pero no
   sustituyen drawers. AAAK es abreviación experimental, lossy y opt-in según
   CLAUDE; no es el almacenamiento predeterminado. Rutas diary/LLM del benchmark
   son variantes distintas, no prueba de que toda captura sea extractiva.
4. **Read/recall:** `search_memories` consulta drawers directamente; closets
   añaden boost por archivo. Ausencia/fallo de closets degrada a drawer-only,
   no bloquea los originales. Hay pools vectoriales, ranking lexical y filtros;
   no toda combinación es idéntica al runner LongMemEval [M5].
5. **Rust real:** `VectorIndex::load_from_sqlite` abre solo lectura y selecciona
   colección explícita; `query`/`query_parallel` hacen búsqueda exacta y filtro
   wing. Python sigue siendo escritor y dueño del backend. `RustExactCollection`
   acelera casos soportados y cae a SQLite para filtros/includes incompatibles;
   no es un reemplazo completo de ingest ni un motor ANN [M2][M6].
6. **Reflect:** no se localizó un equivalente general a retain/recall/reflect
   de Hindsight en los cuerpos examinados. No convertir la clasificación o
   el dialecto en una supuesta reflexión autónoma.

**Identidad/aislamiento:** `PalaceRef.id` es obligatorio; `namespace` solo si el
backend anuncia capacidad. Si no la soporta debe rechazarla, no ignorarla.
`EmbedderIdentity` separa nombre/dimensión; compatibilidad del modelo importa
aun con igual dimensión [M1]. Wings/rooms son organización, no autorización.

**Borrado/retry:** [M4] muestra recuperación por re-mine con marcador incompleto,
no una transacción global entre todas las colecciones/archivos. Invalida AAAK
afectado y conserva resúmenes de drawers sin cambios. No demuestra borrado
de backups, grafo y todos los caches ni exactamente-una-vez para eventos live.

### Ficha A — contrato de capacidades e identidad del backend

- **Fuente/unidad:** `PalaceRef:153-189`, `EmbedderIdentity:193-204`,
  `validate_write_batch:462-474`, `BaseCollection:477-582` [M1].
  Problema: fallos silenciosos al intercambiar storage o embedders.
- **Seam Vanta:** [writer/EmbedFn](../../../../vanta-memory/src/core/record/l1_writer.rs)
  `51-110`; [recall_candidates](../../../../vanta-memory/src/core/record/l1_reader.rs)
  `321-380` ya admite vector opcional. No agregar otro store por comodidad.
- **Hipótesis Rust:** `EmbeddingIdentity { model, revision, dimension, metric }`
  y `BackendCapabilities`; la operación rechaza filtros/scopes no soportados.
  Son contratos candidatos, no tipos actuales acreditados.
- **Incremento/pasos:** inventariar validaciones actuales del SDK; unificar
  detección de capacidades; validar identidad al abrir; propagar errores
  tipados hasta adaptador. Dependencias: catálogo de modelos y formato persistido.
- **FMEA:** aceptar igual dimensión con distinto modelo mezcla espacios;
  fallback que quite scope puede filtrar datos; defaults de health no prueban salud.
- **Aceptación funcional:** modelo incompatible no modifica índice; backend
  incapaz de namespace devuelve error; batch desalineado se rechaza antes de
  escribir. **No objetivo:** portar Chroma ni copiar un ABC Python a Rust.

### Ficha B — índice abstracto que nunca excluye el corpus literal

- **Fuente/unidad:** `search_memories`, `searcher/query.py:315-369` [M5];
  `_closet_boosts:106-158` y `file_conversation_exchange:78-158` [M3].
  Problema: una abstracción incompleta oculta evidencia que sí existe.
- **Seam Vanta:** [L0Recorder](../../../../vanta-memory/src/core/conversation/l0_recorder.rs)
  `record_turn:274`, `read_messages:378`; escenas y
  [assemble_with_recall](../../../../vanta-memory/src/context_engine/engine.rs)
  `252-264` ya manejan contexto y recuperación.
- **Hipótesis Rust:** `EvidencePointer { source_id, span, revision }`,
  `CandidateOrigin::Raw | Derived`; consulta al corpus permitido más boost
  de derivados, siempre bajo el presupuesto actual.
- **Incremento/pasos:** mapear ruta raw ya existente; verificar permisos antes
  de ambos brazos; conservar puntero al texto; explicar boosts; integrar en
  ensamblador. Dependencias: retención L0, provenance, filtros y token estimator.
- **FMEA:** doble presupuesto y duplicación; ventanas que exponen otra sesión;
  índice derivado stale; preservar secretos literalmente sin política de borrado.
- **Aceptación funcional:** derivado vacío o fallido no bloquea evidencia raw
  autorizada; referencia resuelve a revisión correcta; borrado no deja previews
  recuperables. **No objetivo:** prohibir síntesis L1/persona ni prometer recall total.

### Ficha C — reindexación recuperable por archivo, sin destruir primero

- **Fuente/unidad:** `write_file_drawers:150-227` y
  `StaleDrawerPurgeFailed:11-16`, `palace/writes.py` [M4].
  Problema: perder última evidencia válida durante reingest parcial.
- **Seam Vanta:** [ingest/merge](../../../../vanta-memory/src/ingest/merge.rs)
  `CandidatePage:16-23`, `CommitReport:34-41`; checkpoints y scheduler existentes.
- **Hipótesis Rust:** `SourceImportGeneration { source, revision, stage }`,
  `ImportOutcome::PendingCleanup | Committed`; elegir marcador/commit adecuado
  al SDK existente, no imitar `chunk_total+1` literalmente.
- **Incremento/pasos:** revisar atomicidad real de ingest; registrar generación;
  preparar nuevos fragmentos; publicar/retirar stale; recuperar cleanup;
  invalidar derivados por referencias. Dependencias: IDs estables y checkpoints.
- **FMEA:** marcador se borra antes de cleanup; retry duplica; IDs chocan;
  conservar temporalmente dos versiones confunde recall.
- **Aceptación funcional:** interrupción entre lotes deja evidencia legible;
  siguiente pasada converge; purge fallido no aparece como importación completa;
  otro modo/fuente no se borra. **No objetivo:** certificar crash safety sin pruebas.

### Ledger de cobertura

| Fuente | Archivos/rangos de código inspeccionados | Cobertura |
|---|---|---|
| [M1] | `backends/base.py:153-207,462-582`, firmas opcionales | scopes, identidad y contrato |
| [M2] | `backends/rust_exact.py:1-154` | elegibilidad de nativo/fallback y snapshot |
| [M3] | `convo_miner.py:78-172` | ruta live y metadata |
| [M4] | `palace/writes.py:1-140,150-228` | vector reuse y publicación recuperable |
| [M5] | `searcher/query.py:315-375`, búsqueda de funciones/filtros | piso raw y closets |
| [M6] | `crates/mempalace-core/src/lib.rs:105-215,335-405` | carga SQLite y exact scan paralelo |
| [M7] | `benchmarks/longmemeval_bench.py:53-80,146-243,3148-3208` y selección de modos/split | métricas y runner |
| [M8] | `website/.vitepress/theme/Landing.vue:1-29` | sitio Vue, no dashboard |

También README, CLAUDE, licencia y pyproject. No inspeccionados íntegramente:
todos los backends remotos, MCP HTTP/auth, grafo temporal, model eval, loaders
de cada formato, binding PyO3 y toda heurística de hybrid_v2/v3/v4.

## 3. Defectos, riesgos e issues transferibles, con su nivel de evidencia

- **Código:** identidad y namespace tienen contrato explícito; no equivale a
  prueba de conformance de cada adaptador. Un fallback sin filtro sería crítico.
- **Código:** ruta live usa `filed_at` al generar ID [M3:130-131]; sin estudiar
  la receta y llamador no se puede asegurar idempotencia del mismo evento reenviado.
- **Código:** benchmark raw rellena rankings con índices no retornados por
  Chroma [M7:235-239]. Si el corte supera los hits reales, cuenta orden artificial;
  distinguir retrieval de ese relleno antes de interpretar recall.
- **Hipótesis:** archivos viejos/derivados y caches nativos podrían tener ventanas
  de visibilidad diferentes. Requiere prueba de actualización concurrente.
- **Issues:** consulta GitHub MCP intentada, conexión fallida; **no consultados**,
  sin números inventados ni extrapolación de comentarios a incidencias vigentes.

## 4. Prerrequisitos en VantaDB antes de incorporar la idea

Revalidar código actual y cambios concurrentes, especialmente storage/search.
Definir unidad raw recuperable, retención, borrado e identidad del modelo.
Confirmar qué contratos ya cumplen ingestors/checkpoints y evitar duplicarlos.
Todo scope debe aplicarse antes de retrieval/expansión y sobrevivir a fallbacks.
Embeddings y abstracción LLM deben seguir siendo opcionales.

## 5. Componentes y practicas que no deben trasladarse

No reemplazar escenas/personas por alas/habitaciones. No imponer verbatim como
único nivel: L0 preserva evidencia y L1 abstrae con procedencia. No importar
AAAK como compresión sin pérdida. No trasladar boosts o preguntas heurísticas
del benchmark a producción sin validación independiente. No crear un segundo
índice exacto Rust olvidando HNSW/BM25/RRF ya existentes en Vanta.

## 6. Advertencias metodologicas

Rama `develop` y clon superficial: no representa automáticamente una release.
Sin historial anterior/tags/submódulos/LFS descargado; sin punteros LFS en manifest.
MIT raíz, copyright MemPalace Contributors 2026; revisar notices y dependencias
de cada módulo/modelo. Rust/PyO3 no exime de licencia ni demuestra equivalencia.
README/CLAUDE describen intención; recuperación efectiva requiere ejecución.

## 7. Benchmarks existentes: runner, entrada, metricas, dependencias y riesgos

`longmemeval_bench.py` construye por pregunta un corpus desde
`haystack_sessions`, IDs y fechas; raw/session concatena **solo turnos user**,
raw/turn crea documento por turno user [M7:163-241]. No es QA: no genera respuesta
ni usa juez en esa ruta. `recall_any` es indicador de al menos un gold session,
`recall_all` de todos; nDCG usa relevancia binaria [M7:53-80]. Un R@5 publicitado
no debe sustituir esas definiciones por «porcentaje de preguntas bien respondidas».

Otros modos incluyen habitaciones, hybrid, variantes con nombres/frases/fechas,
palace y diary; rerank LLM opcional Anthropic/Ollama. No mezclar raw local
sin llamadas LLM con diary/rerank de coste/modelo diferente. El script permite
split dev/held-out con archivo `lme_split_50_450.json`; su existencia no prueba
que todas las heurísticas históricas se ajustaran solo con dev.
Dependencias: Chroma, embeddings y posible descarga de modelos; LLM/clave
solo según modo. Costes reales de descarga/import/embed/rerank no medidos aquí.

`_fresh_collection` borra/recrea colección por pregunta y silencia error del
borrado; no ejecutarlo sobre storage compartido. No se acredita cierre/cleanup
completo ni clasificación exhaustiva de errores de todos los modos. El log de
resultados contiene pregunta, respuesta gold y previews: nunca reingerirlo
como memoria de test. La comparación Vanta se rige **solo** por
[METHOD](METHOD.md#protocolo-de-comparacion-de-benchmarks); no se diseñó otro runner.

## 8. Matriz existe/parcial/no acreditado/no aplicable frente a VantaDB

| Capacidad | Estado | Diferencia bajo estudio |
|---|---|---|
| Evidencia L0/sesiones/procedencia | Existe | piso raw autorizado frente a derivados |
| Escenas/context assembly | Existe | organización, no metáfora adicional |
| BM25/HNSW/RRF | Existe | no sustituir por exact scan nativo |
| Ingest/checkpoints/scheduler | Existe | convergencia por fuente en fallos parciales |
| Identidad persistida de modelo en toda ruta | No acreditado aquí | revalidar guards actuales |
| Conformance entre todos los backends externos | No aplicable al motor embebido | útil como principio de adaptadores |
| Recall perfecto / exactitud QA del README | No acreditado | claim upstream, no resultado Vanta |

## 9. Debilidades de VantaDB sugeridas por el contraste, no bugs inventados

Preguntas: ¿un derivado puede excluir silenciosamente raw?, ¿cambio de modelo
con igual dimensión se detecta?, ¿reingest fallido deja estado recuperable visible?,
¿borrado invalida previews y embeddings auxiliares? No se afirma fallo vigente:
METHOD reconoce capacidades ya implementadas y el worktree está sucio.
La deuda potencial es verificar composición e interoperabilidad, no sumar capas.

## 10. Escritorio, interfaz e inspeccion: implementado, externo o no localizado

Se inspeccionó fuente Vue de landing [M8]: compone secciones de marketing y
documentación. No acredita un visor operativo de recuerdos. CLI/MCP son
interfaces programáticas; dashboard/desktop transaccional **no localizado en
esta muestra**, no ausencia absoluta. Studio Vanta ya existe en `develop-desktop`.

**Pendiente manual:** backend conformance, mismatch de modelos, scopes bajo
fallback, reingest interrumpido, borrado de derivados, validez de split y
costes/errores de cada runner. No se ejecutaron tests, builds, servicios,
benchmarks ni validadores documentales; tampoco commits.

[M1]: https://github.com/MemPalace/mempalace/blob/f8e35362189bd0a5abf4c0df1dd5bf90ba059176/mempalace/backends/base.py#L153-L582
[M2]: https://github.com/MemPalace/mempalace/blob/f8e35362189bd0a5abf4c0df1dd5bf90ba059176/mempalace/backends/rust_exact.py#L1-L154
[M3]: https://github.com/MemPalace/mempalace/blob/f8e35362189bd0a5abf4c0df1dd5bf90ba059176/mempalace/convo_miner.py#L78-L172
[M4]: https://github.com/MemPalace/mempalace/blob/f8e35362189bd0a5abf4c0df1dd5bf90ba059176/mempalace/palace/writes.py#L150-L228
[M5]: https://github.com/MemPalace/mempalace/blob/f8e35362189bd0a5abf4c0df1dd5bf90ba059176/mempalace/searcher/query.py#L315-L375
[M6]: https://github.com/MemPalace/mempalace/blob/f8e35362189bd0a5abf4c0df1dd5bf90ba059176/crates/mempalace-core/src/lib.rs#L105-L405
[M7]: https://github.com/MemPalace/mempalace/blob/f8e35362189bd0a5abf4c0df1dd5bf90ba059176/benchmarks/longmemeval_bench.py#L146-L243
[M8]: https://github.com/MemPalace/mempalace/blob/f8e35362189bd0a5abf4c0df1dd5bf90ba059176/website/.vitepress/theme/Landing.vue#L1-L29

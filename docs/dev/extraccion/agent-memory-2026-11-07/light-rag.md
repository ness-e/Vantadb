---
title: "LightRAG: recuperación por entidades y relaciones con linaje reconstruible"
kind: research
status: proposed
description: "Lectura estática de LightRAG y candidatos acotados de enriquecimiento, trazabilidad y reconstrucción para VantaDB"
tags: [research, memory, engineering]
---

# LightRAG: recuperación por entidades y relaciones con linaje reconstruible

## Índice

- [1. Recomendaciones y arquitectura](#1-recomendaciones-para-vantadb-y-contexto-arquitectónico)
- [2. Flujos y fichas A–D](#2-partes-extraíbles-flujo-real-y-mapeo-a-rust)
- [3. Riesgos e incidencias públicas](#3-defectos-riesgos-e-issues-transferibles)
- [4. Prerrequisitos VantaDB](#4-prerrequisitos-en-vantadb)
- [5. Rechazos](#5-componentes-y-prácticas-que-no-deben-trasladarse)
- [6. Evidencia y baseline](#6-advertencias-metodológicas-y-evidencia)
- [7. Benchmarks](#7-benchmarks-existentes-y-adaptación-del-protocolo)
- [8. Matriz](#8-matriz-frente-a-vantadb)
- [9. Negativos propios condicionados](#9-debilidades-sugeridas-no-bugs-inventados)
- [10. UI y cobertura](#10-escritorio-interfaz-e-inspección-ledger-de-cobertura)
- [11. Contratos, materialización y recuperación](#11-contratos-materialización-y-recuperación)
- [12. Cierre de autoría: multimodalidad, contratos y Studio](#12-cierre-de-autoría-multimodalidad-contratos-y-studio)

## 1. Recomendaciones para VantaDB y contexto arquitectónico

**Dictamen:** estudiar un enriquecedor opcional de candidatos mediante entidades/relaciones y, sobre todo, su contabilidad de contribuciones. No sustituir el motor de VantaDB ni importar el servicio completo. No hay mediciones propias de calidad, latencia o coste.

- Repositorio: `HKUDS/LightRAG`, clon `C:\Users\Eros\LightRAG`.
- Revisión obtenida mediante `git rev-parse HEAD`: `aaebfe1bb8d55deb6a4304d357a3bf4c92fcd40a`.
- Snapshot de [github-candidates.json](github-candidates.json): **40 591 estrellas**, MIT, no archivado, observado `2026-11-07T06:56:23.2154091Z`. No se reconsultaron estrellas.
- Contratos: [EXPANSION-CONTRACT](EXPANSION-CONTRACT.md), [METHOD](METHOD.md), [BENCHMARK-NOTES](BENCHMARK-NOTES.md) y [VANTA-REFERENCE](VANTA-REFERENCE.md). Se conserva la lectura parcial y se amplía focalmente; no es auditoría integral. Manifiesto: [clone-manifest.json](clone-manifest.json).
- Stack acreditado: Python asíncrono; `LightRAG` compone roles LLM, migraciones y pipeline. Cuatro interfaces separan KV, vectores, grafo y estado documental. Defaults: JSON KV, NanoVectorDB y JSON de estados; los adaptadores son intercambiables, no una transacción distribuida [L1, L2].
- La WebUI tiene fuente React/TypeScript, Vite y Sigma/Graphology [L12]; no es un escritorio nativo certificado.

**Mapa y fronteras del paquete.** `pyproject.toml:5–53,66–155` declara
`lightrag-hku`, Python >=3.10 y dependencias base `aiohttp`, NetworkX,
NanoVectorDB, NumPy, Pydantic, `json_repair`, Tenacity y tiktoken. El extra
`api` incorpora FastAPI/Uvicorn, autenticación y parsers; `offline-storage`
añade clientes de bases opcionales. No son todos requisitos del motor Rust.
`llm_roles.py` configura los roles; `pipeline.py` coordina parse/chunk/extract;
`operate.py` materializa y recupera; `kg/` implementa almacenes;
`storage_migrations.py` migra; `api/routers/` expone documentos, grafo y query;
`lightrag_webui/` es cliente web [L14]. Entrada embebida:
`initialize_storages` → `ainsert`/`aquery`/`aquery_data`/`adelete_by_doc_id`;
HTTP: `query_routes.py:376,745,1114` registra `/query`, `/query/stream`,
`/query/data`. Streaming y respuesta estructurada no son la misma superficie.

**Modelo persistido.** Documento y estado se separan de chunks; cada chunk
referencia documento, contenido y cachés de extracción. El nodo usa nombre
normalizado y descripción/tipo; la arista lleva extremos, keywords, descripción,
peso y fuentes. `full_entities[doc_id]` y `full_relations[doc_id]` son anchors
documento→candidatos, mientras `entity_chunks`/`relation_chunks` responden
objeto→contribuciones. Ninguno equivale a historial autobiográfico de sesiones.
Embeddings de entidades, relaciones y chunks son proyecciones diferentes:
el hash `ent-`/`rel-` identifica registro, no autenticidad de la evidencia [L15].

**Diferencia decisiva:** en este código `local` busca embeddings de **entidades** con palabras clave de bajo nivel; `global` busca embeddings de **relaciones** con palabras clave de alto nivel. `hybrid` combina ambas; `mix` añade búsqueda vectorial de chunks; `naive` evita el grafo. No confundir su `global` con el map/reduce sobre comunidades de Microsoft GraphRAG. La ruta examinada resume descripciones de entidades/relaciones, no acredita construcción Leiden ni informes de comunidades [L3–L6].

VantaDB ya posee BM25/HNSW/RRF, dedup, procedencia, escenas, sesiones, supersession y ensamblado con presupuesto compartido. El delta útil sería **evidencia relacional que desemboca en el recall y presupuesto existentes**, no otro buscador híbrido ni otro `context_assemble`.

## 2. Partes extraíbles, flujo real y mapeo a Rust

### Trazado de indexación y consulta

1. `ainsert` delega en el pipeline; `process_single_document` selecciona chunker, valida resultados y persiste chunks tanto en KV como VDB. La revisión tiene dispatch y parsers adicionales: no reducirla al ejemplo histórico de segmentación fija [L2].
2. `extract_entities` llama al modelo de extracción y admite resultados que pasan por normalización/validación. Los handlers textuales producen entidad `{entity_name, entity_type, description, source_id, file_path, timestamp}` y relación `{src_id, tgt_id, description, keywords, weight, source_id, file_path, timestamp}`; rechazan nombres vacíos, autorrelaciones y descripciones vacías. En esa entrada textual el peso inicial es `1.0`, no confianza calibrada [L3].
3. `_merge_nodes_then_upsert` recupera el nodo por **nombre normalizado**, agrega fuentes y descripciones y actualiza grafo/vector. `_merge_edges_then_upsert` agrega contribuciones de la pareja. Esta resolución es identidad por clave normalizada y agregación; no acredita desambiguación semántica general de alias/homónimos [L4].
4. `entity_chunks` y `relation_chunks` conservan contribuciones completas; el `source_id` visible en grafo puede ser una vista truncada. Una fila de tracking presente pero vacía es autoritativa: no se debe repoblar desde el nodo [L4].
5. `kg_query` extrae keywords, prepara contexto y permite devolver contexto o prompt sin generar respuesta. Si ambas listas quedan vacías, usa la pregunta como keyword baja únicamente cuando su longitud es menor de 50 caracteres; en otro caso devuelve la respuesta de fallo. No es un fallback universal a búsqueda vectorial [L5].
6. `_perform_kg_search` calcula embeddings necesarios según modo, busca entidades/relaciones y expande vecinos. `_get_node_data` conserva orden vectorial de entidades y obtiene aristas por rango/peso; `_get_edge_data` conserva orden vectorial de relaciones y recupera extremos [L6].
7. `_apply_token_truncation`, mezcla de chunks y `_build_context_str` limitan entidades, relaciones y texto. Este último carga el template efectivo, cuenta consulta y overhead, reserva margen y pasa el remanente a `process_chunks_unified` antes de generar referencias [L7].
8. Las referencias finales se generan **después del recorte**, agrupadas por `file_path`, frecuencia y primera aparición. Varias porciones del mismo fichero comparten número; `unknown_source` no obtiene número. El prompt puede pedir citas, pero la lista de fuentes no demuestra que cada afirmación generada esté respaldada [L8].

### Fichas candidatas — propuestas, no APIs existentes ni implementación aprobada

**A. Expansión relacional con evidencia**

- Problema/unidad mínima: recuperar evidencia indirecta alrededor de hits ya obtenidos; extraer la separación `_get_node_data`/`_get_edge_data`, no el orquestador [L6, `operate.py:6200–6257,6529–6585`].
- Seam existente: `vanta-memory/src/core/hooks/auto_recall.rs:391–432` y `core/ontology/mod.rs:147–212`; ambos ya tienen recall/linaje.
- Contrato Rust **propuesto**: `GraphEvidenceExpander::expand(scope, seeds, remaining_budget) -> Result<Vec<EvidenceCandidate>, EnrichmentError>`, con IDs de fuente y revisión en cada candidato.
- Dependencias/acciones: mapear IDs de memoria a entidades, resolver autorización antes de vecinos, adaptar resultados al ranking/assembly actual; LLM opcional solo para extracción/keywords.
- FMEA: homónimos fusionados → evidencia equivocada; expansión sin scope → fuga; hub → desplazamiento del contexto. Mitigación: identidad contextual, scope transitivo y contabilización dentro del presupuesto existente.
- Aceptación pendiente: cada candidato añadido resuelve fuente autorizada vigente; vacío/error se distingue de “sin vecinos”; consulta con enriquecedor desactivado conserva baseline.
- No objetivos: reemplazar BM25/HNSW/RRF, implantar todos los modos upstream o introducir un servicio Python.

**B. Registro completo de contribuciones para reconstrucción**

- Unidad mínima/símbolos: presencia de tracking y unión de fuentes en `_merge_nodes_then_upsert`, `operate.py:2490–2520`; prueba previa al purge en `lightrag.py:5679–5745` [L4, L9].
- Seam existente: `apply_triplet_with_source` y metadatos `source_episode_ids`/`derived_from` en `vanta-memory/src/core/ontology/mod.rs:147–212`.
- Contrato Rust **propuesto**: `ContributionLedger::sources(scope, derived_id, revision) -> Result<ContributionState, LedgerError>`; estado distingue `Missing`, `PresentEmpty` y `Present(set)`.
- Dependencias/acciones: definir autoría por documento/episodio y revisión, registrar antes de mutar derivados, reconstruir desde supervivientes; enlazar con borrado/supersession existentes.
- FMEA: confundir ausencia con vacío deja derivados huérfanos o elimina compartidos; tracking recortado pierde capacidad de reconstrucción. Rechazar borrado sin prueba, no inferir historia.
- Aceptación pendiente: quitar una fuente compartida conserva las demás; reinicio/reintento conserva atribución; contribuciones vacías no equivalen a fallo de lectura.
- No objetivos: copiar journals Python ni declarar que VantaDB carece de procedencia o borrado.

**C. Proyección citada del contexto efectivamente enviado**

- Unidad mínima/símbolos: `generate_reference_list_from_chunks`, `utils.py:7735–7798`, y `render_chunks_context_text:7801–7822` [L8].
- Seam existente: `MemoryRecord.source_message_ids`, `vanta-memory/src/core/abstractions/types.rs:71–97`; `assemble_with_recall`, `context_engine/engine.rs:252–264`.
- Contrato Rust **propuesto**: `EvidenceManifest::from_delivered(delivered_spans, scope, revision) -> Result<Manifest, CitationError>`; ordinal de presentación separado del ID durable.
- Dependencias/acciones: conservar span/documento/revisión a través de recorte y renderizado; proyectar solo evidencias entregadas, sin volver a recuperar después.
- FMEA: citar por ruta mutable o por ordinal global atribuye versión equivocada; presupuesto sobre texto distinto del enviado desborda ventana. Usar IDs/revisiones y serialización compartida.
- Aceptación pendiente: cada ordinal resuelve exactamente un span entregado; fuentes descartadas no aparecen como enviadas; ruta desconocida queda explícita, no inventada.
- No objetivos: un nuevo context assembler ni certificar entailment por tener una cita.

**D. Resultado degradado tipado por etapa**

- Unidad mínima/símbolos: vacíos/errores diferenciados de `kg_query`, `operate.py:4754–4796`, y falta de tokenizer en `_build_context_str:5890–5903` [L5, L7].
- Seam existente: `perform_auto_recall_governed`, `auto_recall.rs:399–408`, y retorno `Result` de `assemble_with_recall`.
- Contrato Rust **propuesto**: `EnrichmentOutcome { candidates, stage_status, omitted_evidence }`, con error de autorización no degradable a búsqueda sin filtros.
- Dependencias/acciones: acordar taxonomía con telemetría/recall existentes y propagar estado a diagnóstico; no introducir políticas de scoring.
- FMEA: “sin datos” oculta timeout o denegación; fallback cambia silenciosamente de corpus. Mantener scope y motivo de degradación.
- Aceptación pendiente: timeout, keyword vacía y cero hits se observan como estados distintos; ninguna ruta amplia permisos.
- No objetivos: copiar el umbral de longitud de pregunta ni trasladar excepciones genéricas como política.

## 3. Defectos, riesgos e issues transferibles

| Evidencia | Hallazgo y frontera | Consecuencia / cautela |
|---|---|---|
| Código [L3, L4] | Identidad nominal normalizada, no resolución semántica acreditada | Homónimos/alias requieren evidencia externa; riesgo, no bug reproducido |
| Código [L4, L9] | Escrituras separadas y reconstrucción con tracking/journal | Analizar residuos y recuperación por backend; un estado intermedio no prueba por sí solo pérdida de datos |
| Código [L8] | Cita agregada por ruta | Sirve para inspección documental, no garantiza versión, span ni veracidad de afirmación |
| Código [L5–L7] | Fallbacks concretos: keyword corta, embedding batch que puede fallar, contexto vacío | No prometer continuidad de todos los modos ni presupuestado de todas las llamadas |
| Código [L10] | Workspace validado y usado como subdirectorio JSON | Partición de almacenamiento no equivale a autorización por usuario; no se certificaron todos los backends |
| Hipótesis | Prompt injection dentro de descripciones y evidencia recuperada | Tratar contenidos como datos; validar configuración/modelo en prueba discriminante futura |

El purge actual tiene pruebas de recuperación, coordinación por workspace y reconstrucción de supervivientes; no trasladar relatos de defectos antiguos como si siguieran abiertos. `adelete_by_doc_id(delete_llm_cache=False)` no promete borrar por defecto el caché LLM [L9]. Una política de eliminación de datos debe incluirlo explícitamente.

### Incidencias públicas leídas en este cierre

Consulta del 2026-11-07, separada de la metadata congelada. Se leyeron las
páginas oficiales y el cuerpo de la búsqueda pública API; no se ejecutaron
repros ni se inspeccionaron todos los PR enlazados.

| Fuente / estado confirmado | Mecanismo y alcance | Decisión para Vanta |
|---|---|---|
| [#3659, RFC single-pass deferred KG rebuild](https://github.com/HKUDS/LightRAG/issues/3659), abierta, creada 2026-08-16; API indica actualización 2026-11-07 | Describe rebuild repetido por documento en un batch y borrado parcial presentado como `PROCESSED`. Propone `DELETING`, conservar anchors e intención durable hasta reconstrucción agregada. Es **RFC**, no funcionalidad acreditada por publicarse. La cifra de llamadas redundantes citada de #2795 es reporte de terceros no reproducido aquí | Distinguir contabilidad B de protocolo de operación E. No optimizar batch perdiendo recuperación, ni clasificar destructivamente contra documentos todavía no retirados |
| [#3978, Vector storage model/dimension change](https://github.com/HKUDS/LightRAG/issues/3978), abierta, creada 2026-09-16 según página | Distingue espacio vectorial de dimensión y refusal de servicio de capacidad de rebuild. **Su propia tabla “Current state” se declara histórica/no actualizada** mientras el diseño y PRs evolucionan. El cuerpo explica excepción tipada, markers fuera del ANN y fuentes autoritativas de rebuild; no acredita que todos los backends sigan fallando hoy | No comparar vectores de igual dimensión como equivalentes; una recuperación no debe capturar outage/credenciales como permiso de drop. Aplicabilidad requiere revisar implementación Vanta, no trasladar la tabla como bug actual |

En #3659 las descripciones de PR #2819/#3389 cerrados son narrativa del RFC,
no lectura independiente de sus diffs ni demostración de merge. En #3978 se
leyó el cuerpo/diseño visible; la extracción de página quedó truncada en el
tramo posterior de actividad: comentarios completos y estado de cada PR
quedan **pendientes**. No se adjudica una vulnerabilidad ni corrección a Vanta.

**Acceso/fallback:** búsqueda `websearch.search` y metasearch Bing/DuckDuckGo
para `site:github.com/HKUDS/LightRAG/issues delete document graph sources`
no devolvió resultados; `github-mcp-server` no conectó. GET PowerShell a la API
dio 403; `websearch.fetchWebContent` sí leyó búsqueda API y páginas GitHub.
La cuota pública API se agotó durante los detalles; se usó HTML oficial.
No se ocultan esos fallos ni se interpreta una búsqueda vacía como ausencia.
Los tests leídos siguen acreditando casos escritos, nunca resultados propios.

## 4. Prerrequisitos en VantaDB

1. Revalidar implementación/Backlog antes de crear tareas: la comparación no autoriza reservar IDs ni duplicar trabajo.
2. Consolidar identidad de evidencia y contribuciones con revisión; reutilizar procedencia ya existente.
3. Mantener autorización en todo salto grafo → documento → contexto y también en cachés/diagnóstico; no usar solo `session_id` como supuesto de tenant seguro.
4. Enchufar enriquecimiento al presupuesto **compartido** de `assemble_with_recall`; definir ownership de cancelación/reintento con el worker existente.
5. Validar licencia por archivo/dependencia/modelo antes de portar material. [LICENSE](https://github.com/HKUDS/LightRAG/blob/aaebfe1bb8d55deb6a4304d357a3bf4c92fcd40a/LICENSE#L1-L21) es MIT y exige conservar aviso; MIT del repo no licencia automáticamente proveedores, modelos, corpus ni adaptadores externos.

## 5. Componentes y prácticas que no deben trasladarse

- El producto completo de cuatro almacenes/servicio/UI para resolver una necesidad de enriquecimiento embebido.
- Nombres normalizados como identidad universal; rutas de fichero como identificador durable de evidencia.
- Defaults de peso como confianza factual; resumen LLM como sustituto de las fuentes originales.
- Recuperación o cachés que amplíen permisos ante error; “tiene workspace” como prueba de aislamiento multitenant.
- Duplicar dedup, escenas, RRF o assembly que VantaDB ya tiene.

## 6. Advertencias metodológicas y evidencia

Lectura estática del SHA indicado, sin servicios, ejecución de código, tests, builds, instalación ni benchmarks. Las URLs de código fijan la revisión local; las consultas de issues se distinguen en §3.

| Ref. | Archivo/símbolo leído y evidencia |
|---|---|
| L1 | [lightrag.py:684–715](https://github.com/HKUDS/LightRAG/blob/aaebfe1bb8d55deb6a4304d357a3bf4c92fcd40a/lightrag/lightrag.py#L684-L715): composición y almacenes |
| L2 | [pipeline.py:4991–5857](https://github.com/HKUDS/LightRAG/blob/aaebfe1bb8d55deb6a4304d357a3bf4c92fcd40a/lightrag/pipeline.py#L4991-L5857): selección de chunker, upserts y merge; lectura focal de etapas |
| L3 | [operate.py:710–850](https://github.com/HKUDS/LightRAG/blob/aaebfe1bb8d55deb6a4304d357a3bf4c92fcd40a/lightrag/operate.py#L710-L850): handlers de extracción |
| L4 | [operate.py:2429–2520](https://github.com/HKUDS/LightRAG/blob/aaebfe1bb8d55deb6a4304d357a3bf4c92fcd40a/lightrag/operate.py#L2429-L2520): merge y tracking; también cabecera de `_merge_edges_then_upsert:2782–2860` |
| L5 | [operate.py:4693–4824](https://github.com/HKUDS/LightRAG/blob/aaebfe1bb8d55deb6a4304d357a3bf4c92fcd40a/lightrag/operate.py#L4693-L4824): keywords/fallback/context-only |
| L6 | [operate.py:5280–5410](https://github.com/HKUDS/LightRAG/blob/aaebfe1bb8d55deb6a4304d357a3bf4c92fcd40a/lightrag/operate.py#L5280-L5410), [6200–6585](https://github.com/HKUDS/LightRAG/blob/aaebfe1bb8d55deb6a4304d357a3bf4c92fcd40a/lightrag/operate.py#L6200-L6585): ramas y resolución de hits/vecinos; lectura focal |
| L7 | [operate.py:5873–6071](https://github.com/HKUDS/LightRAG/blob/aaebfe1bb8d55deb6a4304d357a3bf4c92fcd40a/lightrag/operate.py#L5873-L6071): contexto y tokens |
| L8 | [utils.py:7735–7822](https://github.com/HKUDS/LightRAG/blob/aaebfe1bb8d55deb6a4304d357a3bf4c92fcd40a/lightrag/utils.py#L7735-L7822): referencias y render |
| L9 | [lightrag.py:5679–5745](https://github.com/HKUDS/LightRAG/blob/aaebfe1bb8d55deb6a4304d357a3bf4c92fcd40a/lightrag/lightrag.py#L5679-L5745), [6727–6828](https://github.com/HKUDS/LightRAG/blob/aaebfe1bb8d55deb6a4304d357a3bf4c92fcd40a/lightrag/lightrag.py#L6727-L6828): prueba de purge y reserva; llamadas de reconstrucción localizadas en `6690,6981,7162` |
| L10 | [json_kv_impl.py:65–113](https://github.com/HKUDS/LightRAG/blob/aaebfe1bb8d55deb6a4304d357a3bf4c92fcd40a/lightrag/kg/json_kv_impl.py#L65-L113): workspace; `api/auth.py:156–257` inspeccionado focalmente para JWT, no auditoría de permisos |
| L11 | [eval_rag_quality.py:455–546](https://github.com/HKUDS/LightRAG/blob/aaebfe1bb8d55deb6a4304d357a3bf4c92fcd40a/lightrag/evaluation/eval_rag_quality.py#L455-L546), [offline_retrieval_check.py:134–193](https://github.com/HKUDS/LightRAG/blob/aaebfe1bb8d55deb6a4304d357a3bf4c92fcd40a/lightrag/evaluation/offline_retrieval_check.py#L134-L193): runners, no resultados propios |
| L12 | [WebUI package.json:1–50](https://github.com/HKUDS/LightRAG/blob/aaebfe1bb8d55deb6a4304d357a3bf4c92fcd40a/lightrag_webui/package.json#L1-L50), [AppRouter.tsx:85–115](https://github.com/HKUDS/LightRAG/blob/aaebfe1bb8d55deb6a4304d357a3bf4c92fcd40a/lightrag_webui/src/AppRouter.tsx#L85-L115): fuente web |
| L13 | [test_aquery_data_endpoint.py:332–380](https://github.com/HKUDS/LightRAG/blob/aaebfe1bb8d55deb6a4304d357a3bf4c92fcd40a/tests/api/routes/test_aquery_data_endpoint.py#L332-L380): comparación de referencias mediante HTTP, requiere servidor |

**Baseline del cierre:** `97c9623c8ef5a70fdb51aff215bfbb3b05b15612`,
fijado por [VANTA-REFERENCE](VANTA-REFERENCE.md), no un HEAD móvil.
La lectura `c561158.../dirty` queda como antecedente histórico. Se contrastaron
con `git show` los seams `ontology/mod.rs::apply_triplet_with_source:147–212`,
`ingest/merge.rs::{CandidatePage:18,commit:239}`,
`context_engine/engine.rs::{IntegratedContext:228,assemble_with_recall:252}`
y `src/wiki/ingestors.rs::Ingestor:45–89`; enlaces [LV1]–[LV3].
No se presenta `MemoryScope` hipotético como tipo existente: usar los contratos
reales `RecallScope`, `RecallConfig` e `InjectionPolicy` del baseline.

## 7. Benchmarks existentes y adaptación del protocolo

- `RAGEvaluator` en `lightrag/evaluation/eval_rag_quality.py` carga JSON de preguntas/respuestas de referencia; consulta API con referencias y contenido de chunks; usa RAGAS con LLM/embeddings de evaluación. Métricas: faithfulness, answer relevancy, context recall y context precision; el promedio RAGAS excluye NaN y un caso sin métricas válidas falla [L11]. La agregación separa casos válidos: conservar errores/denominadores según METHOD, no solo el promedio.
- Dependencias/entrada: API ya indexada, credenciales del modelo generador y juez, `ragas`, `datasets`, proveedor de embeddings y fichero explícito; `sample_dataset.json` es material de ejemplo, no certificación general. El coste incluye retrieval, generación y juez; el tiempo de indexación queda fuera si se usa un servicio ya preparado.
- `offline_retrieval_check.py` carga documentos Markdown y oráculo de ficheros relevantes; usa su propio scoring lexical con IDF, calcula recall@k y cobertura. **No ejecuta el grafo LightRAG ni demuestra calidad de sus embeddings.** Sin juez LLM; el modo estricto del runner es upstream, no criterio propuesto para VantaDB.
- `test_references_consistency` [L13] inspecciona endpoints; no es benchmark de veracidad. No se ejecutó.
- Adaptación única: pistas retrieval y respuesta separadas de [METHOD](METHOD.md#protocolo-de-comparacion-de-benchmarks), mismo corpus/split/modelo/budget, baseline Vanta actual frente a un delta. Registrar intentos, fallos, abstenciones, cardinalidad y costes por etapa. No se redactan prompts, jueces, umbrales ni configuraciones nuevas.
- Riesgo operativo: no usar workspace compartido; preparar/eliminar datos para indexación puede destruir material o contaminar otro ensayo. No se autoriza ejecutar limpieza, ingestión ni llamadas pagadas. Las cifras de proveedor no son mediciones de esta investigación.

**Runner RAGAS, defaults y denominadores ampliados:** `RAGEvaluator.__init__`
(`eval_rag_quality.py:129–244`) toma por defecto `sample_dataset.json`,
API `http://localhost:9621`, juez `gpt-4o-mini`, cinco retries y timeout 180 s
del cliente del juez. El **código** de embeddings de evaluación usa
`text-embedding-3-large:178–180`, aunque el docstring dice `3-small`: no
resolver la discrepancia escogiendo la documentación. `generate_rag_response`
(`301–401`) consulta la API y toma sus chunks/referencias; el corpus ya debe
estar indexado. Ground truth se entrega al evaluador RAGAS, no por ese hecho
al retrieval. La tabla de scores no es un hit por ID de sesión.

`evaluate_responses:573–637` usa `EVAL_MAX_CONCURRENT=2`, semáforo de
generación al doble y otro de evaluación; cada caso construye métricas nuevas.
`evaluate_single_case:535–564` promedia sólo métricas no NaN de ese caso;
si ninguna es válida devuelve error con `metrics={}`. En
`_calculate_benchmark_stats:789–887`, éxito = `bool(metrics)`, tasa =
exitosos/`len(results)` y cada media usa su propio conteo no NaN. Por tanto,
las cuatro medias pueden tener denominadores distintos: conservar resultados
por pregunta y errores, no homologarlas a accuracy de todas las intentadas.
No se auditó la versión instalada de RAGAS ni su juez interno. No hay split
holdout acreditado por el mero hecho de cargar el sample.

**Runner offline:** `QueryResult`/`summarize` y CLI en
`offline_retrieval_check.py:32–61,182–254` calculan recall sobre ficheros gold,
MRR del primer relevante, conteo de cobertura completa y cero hits.
La media divide por número de consultas; `--top-k` default **2**, orden
lexical de desempate por nombre y sólo scores positivos en `audit_samples`.
Un oráculo ausente o lectura inválida falla, no elimina la pregunta para
mejorar la media; no se acreditó pista de abstención conversacional.
Es una comprobación lexical de documentación de muestra, sin indexar el
grafo, llamar LLM ni escribir el corpus. RAGAS sí escribe `evaluation/results`,
CSV/JSON y puede imprimir fragmentos de respuesta/ground truth; hay riesgo
de datos sensibles y gasto del generador, juez y embedder. No se encontró
contabilidad completa de tokens/precios de todas esas fases en los tramos
leídos. Un promedio RAGAS no certifica memoria longitudinal.

Adapter futuro, **sin cambiar harness**: traducir pregunta y lista de chunks
del recall autorizado Vanta al contrato esperado por `RAGEvaluator`, usando
un answerer del host sólo en pista QA. En la pista offline un adapter que
invocara Vanta dejaría de ser el scoring lexical upstream: identificarlo
como pista distinta, no atribuir el score original a Vanta. Dataset/modelos,
fallos, unidad y presupuesto siguen [METHOD](METHOD.md) sin prompts o
thresholds nuevos.

## 8. Matriz frente a VantaDB

| Capacidad | Estado Vanta | Delta acreditable / límite |
|---|---|---|
| BM25/HNSW/RRF, dedup, escenas, sesiones | Existe | Baseline METHOD; no reimplementar |
| Procedencia por mensajes/episodios y derivados | Existe | Campos actuales y `apply_triplet_with_source`; no partir de cero |
| Ontología temporal y grafo | Existe / integración parcial por verificar | Enriquecimiento dual de entidad/relación no acreditado en esta lectura |
| Context assembly y compactación compartida | Existe | Proyección citada del contexto enviado sería el delta |
| Ledger completo fuente→objeto para purge/rebuild | Parcial | Linaje existe; equivalencia al ledger/journal LightRAG no acreditada |
| Citas documentales con revisión/span verificable | Parcial | Procedencia existente no certifica ese contrato end-to-end |
| Aislamiento en todos los saltos/backends | No acreditado integralmente | Hay governance; no negar controles existentes ni certificarlos sin ruta completa |
| Comunidades Leiden + global map/reduce | No acreditado | Tampoco es la función del `global` LightRAG examinado |

## 9. Debilidades sugeridas, no bugs inventados

La comparación sugiere comprobar **integridad del linaje al eliminar una fuente compartida**, identidad de entidades y visibilidad de degradación. No demuestra carencias absolutas de VantaDB. El worker ya llama a recall y `assemble_with_recall` (`services/pipeline_worker.rs:795–849`); hay además entrada gobernada de recall. El trabajo futuro sería probar cobertura transversal de esas rutas con evidencia de revisión actual, no añadir una segunda cadena de memoria.

## 10. Escritorio, interfaz e inspección; ledger de cobertura

**Implementado en fuente:** WebUI React y routers de login/workspace [L12], API de documentos/grafo/consulta localizadas. **No acreditado:** escritorio nativo, despliegue funcional, equivalencia de todos los controles UI con permisos de backend. La primera pasada no inspeccionó Studio; §12 añade lectura de su código, sin certificar runtime.

| Superficie | Cobertura estática / pendiente |
|---|---|
| Indexación | Entradas, selección de chunker y persistencia/merge focales; parsers multimodales completos no examinados |
| Retrieval | Keywords, entidad/relación, vecinos, contexto, referencias y rutas de vacío; reranker/provider completo no auditado |
| Actualización/borrado | Tracking, prueba de purge, reservas y llamadas rebuild; no demostración de crash safety de cada backend |
| Seguridad | Workspace JSON y puntos de autenticación localizados; autorización completa, deployments y backends remotos no auditados |
| Tests | Cuerpo de test HTTP de referencias leído; suites de concurrencia/purge no ejecutadas ni exhaustivamente revisadas |
| Runtime/documentación | Todo runtime y gates de enlaces/frontmatter pendientes manuales; no scripts, commits ni push en esta tarea |

## 11. Contratos, materialización y recuperación

### Trazas complementarias y prioridades

**Materialización real, no un grafo hipotético.** En
`operate.py::merge_nodes_and_edges:3514–3568,3666–3725`, Phase 0 escribe
el superset de candidatos por documento, incluso filas vacías; llama a
`index_done_callback` en ambos anchors antes del primer cambio de grafo.
Un fallo de flush se propaga como `IndexFlushError`; el hook
`on_anchors_durable` adelanta el marcador de escritura sólo después [L16].
Así se descubre un merge parcial sin depender de chunks que serán borrados.
El superset no prueba que cada candidato llegase a materializarse.

En `_merge_nodes_then_upsert:2727–2770`, el payload vectorial se construye
y recorta antes del upsert de nodo; el I/O vectorial sucede después, con
retry acotado. En `_merge_edges_then_upsert:3158–3240,3372–3465` se pueden
crear extremos `UNKNOWN`, registrar sus contribuciones y vectorizarlos;
la identidad vectorial ordena extremos, pero el upsert del grafo conserva
la dirección recibida. La arista se escribe antes de borrar vectores previos
en ambos órdenes y publicar su nuevo vector. Hay estados intermedios reales,
no commit atómico grafo+KV+ANN. Un resumen deduplicado conserva descripciones,
no todos los spans originales; no usar su peso como probabilidad [L15].

**Baja/reproceso.** `_purge_kg_contributions:6015–6250` valida prueba de
recuperación y journal; `prepared → derived_committed → anchors_pending →
completed` permite saltar fases ya persistidas. La prueba distingue anchors,
estado `pre_graph`, journal y scope realmente vacío; falta de prueba no
autoriza borrar portadores de atribución. `_purge` actualiza tracking,
retira objetos sin supervivientes, reconstruye compartidos y hace flush
de derivados antes de retirar chunks; luego elimina anchors [L17].
`_insert_done` no es una transacción distribuida. Fallo conserva fase/error:
no reinterpretarlo como documento sano ni reiniciar toda la extracción
sin revisar el journal. El contrato de diseño precisa además los residuos
aceptados de cachés y commits por backend [L18], no una garantía universal.

`adelete_by_doc_id:6780–6828` prearma token de ownership antes de reservar
workspace, y organiza salida por `try/finally`; cancelación entre reserva
y logging también debe alcanzar liberación/handoff. Eso no significa que
un batch cancelado revierta prefijos ya durables. Actualizar una fuente
exige retirar contribuciones obsoletas y publicar nuevas, no sólo sobrescribir
su texto. La ruta completa de cada parser/reproceso queda fuera de cobertura.

**Orden recomendado:** P0 B+E si se habilitan derivados compartidos; P1 C y D
para hacer visible lo realmente enviado y los errores; P2 F+A sólo ante un
caso que necesite evidencia relacional. No es una hoja de implementación
aprobada. B es el dato de ownership, E su protocolo; F produce derivados,
A los consume; C y D pueden ampliarse sin exigir todo el grafo.

### Cierre contractual A–D

Los tipos de esta sección son **propuestos**, no símbolos existentes. Usar
`RecallScope`/`InjectionPolicy` reales al integrar; `Revision`/IDs nombran
conceptos por concretar, no nuevos tipos acreditados.

| Ficha | Entradas, salida y errores propuestos | Ownership, cancelación, concurrencia y publicación |
|---|---|---|
| A | Seeds autorizadas con ID/revisión, scope y remanente → candidatos con camino y evidencia; `Denied`, `StaleRevision`, `Unavailable`, `BudgetExhausted` distintos de cero vecinos | Caller conserva deadline; expander sólo lee, no publica. Fijar revisión por consulta; cancelar tareas del modelo no cambia fuentes. Revisar ACL también al resolver vecinos; remanente no es una cuota de almacenamiento |
| B | Documento/revisión + derivado → `Missing`, `PresentEmpty`, conjunto completo; lectura corrupta/IO no se convierte en vacío | Escritor de contribución posee la revisión y registra antes de exponer derivado. Serializar/CAS por fuente usando backend actual; retry conserva operación. No truncar ledger aunque sí se limite la vista de contexto |
| C | Spans **ya entregados** + revisión autorizada → manifest de citas; `UnknownSource`, `MissingSpan`, `StaleRevision` explícitos | Assembler posee el texto final. Cancelación previa al envío no produce receipt de entrega; no afirmar ACK del proveedor con una preparación local. Manifest inmutable por petición, sin releer fuente mutable después |
| D | Resultado de cada etapa + diagnóstico sanitizado → outcome con omisiones; `Denied` nunca permite fallback sin filtros | Worker actual posee cancelación/retry. Reportar éxito parcial sin sobreescribir fallo persistido; evitar logs de texto/credenciales. No requiere un nuevo almacén ni scheduler |

**Alternativas y acciones por ficha.** A: extender ontología/recall primero,
adapter opcional del host después; rechazar importar Python o todos los modos.
Inventariar vínculos → resolver ACL/revisión → añadir candidatos al ranking
actual → exponer origen; desactivar devuelve al recall existente.
B: extender `source_episode_ids`/`derived_from` antes de introducir un ledger
separado; inventariar cobertura → migración aditiva marcada como incompleta →
backfill sólo con evidencia → activar retirada probada. Nunca inventar
ancestros de registros legados; rollback deshabilita nuevas derivaciones,
no borra las pruebas de recuperación.
C: ampliar `IntegratedContext` antes que duplicar assembly; fijar spans →
preservarlos durante recorte → producir manifest tras render → mostrarlo
en inspector. Alternativa mínima: fuente/revisión sin prometer coordenadas
exactas. Migración sólo si se persiste el receipt, separada del primario.
D: ampliar reportes/errores actuales antes que DTO paralelo; acordar estados →
mapear en frontera de adapter → propagar hasta cliente/Studio. Al apagarlo
no perder errores de autorización ni cambiar la selección silenciosamente.

**FMEA/aceptación futura complementaria.** En A, homónimo o hub se detecta
inspeccionando camino/IDs, no por una buena respuesta aislada; probar scope
cruzado, timeout, revisión retirada y opt-out sin mezcla de permisos.
En B, vacío autoritativo y fila ausente deben producir decisiones diferentes;
ensayar dos fuentes compartidas, retry y fallo entre ledger/índice.
En C, comparar manifest con bytes/spans realmente enviados, incluyendo
truncamiento y fuente renombrada; una cita no prueba entailment.
En D, simular cero hits, keywords vacías, timeout y denegación y distinguirlos
sin exponer contenido sensible. Son comprobaciones manuales futuras,
no thresholds, scoring, pruebas ejecutadas ni autoaceptación.

### E. Operación de retirada con intención recuperable

- **Caso agente/unidad:** eliminar un episodio compartido sin dejar evidencia
  revocada ni destruir otra memoria. Upstream: journal/proofs del purge [L17,
  L18]; #3659 motiva batch diferido pero **no acredita su implementación**.
- **Delta/seam:** `core/ontology/mod.rs::apply_triplet_with_source:147–212`
  ya añade linaje; `utils/task_checkpoint.rs::TaskCheckpointManager` y
  `src/wiki/state.rs::WikiState` ya expresan trabajo/errores. Falta acreditar
  que esos caminos exponen una intención única para reparar todos los derivados.
- **Contrato Rust propuesto:** `RetractionPlan { operation_id, source_revision,
  affected_ids, proof }`; `prepare(...) -> Result<Plan, RetractionError>`,
  `advance(plan, expected_phase) -> Receipt`. Errores `MissingProof`,
  `RevisionConflict`, `StorageUnavailable`, `RepairIncomplete` separados.
  Invariante: no retirar el último portador de atribución antes de reparar
  o retirar los objetos que nombra.
- **Acciones/dependencias:** 1) mapear olvidado/supersession y flush reales;
  2) reutilizar checkpoint para intención sin segundo journal;
  3) persistir prueba B y operación antes de mutar;
  4) avanzar fases comprobando revisión y reparar ANN/textual/grafo;
  5) conservar diagnóstico al fallar, retirar anchors al final. Legado sin
  prueba exige reconciliación explícita, no marcar `pre_graph` retrospectivo.
- **Alternativas:** ampliar lifecycle existente preferido; transacción del
  backend si cubre realmente la unidad; adapter de fuentes como alcance
  menor; rechazar un journal de ficheros adicional y batch todo-o-nada ficticio.
- **Ownership/commit:** worker conserva token y fuente; cancelación deja
  prefijo recuperable con fase durable, no “rollback” supuesto. Conflicto
  de revisión no puede borrar sucesora; reintento idéntico es idempotente.
  Al desactivar se impide crear trabajo nuevo, sin abandonar operaciones.
- **FMEA:** crash tras retirada vectorial → residuo textual/grafo; detectarlo
  por fase/manifiesto y reparar antes de confirmar. Borrado batch clasificado
  contra todo el batch → cancelación destruye objetos de fuente no retirada;
  clasificar contra estado vivo por fuente, no intención global.
- **Aceptación pendiente/no objetivos:** inspeccionar fallo en cada frontera,
  reintento/restart y última fuente; no confirmar éxito con rebuild fallido.
  No añadir runtime distribuido ni copiar nombres de estados sin sus consumidores.

### F. Materialización opt-in de entidades/relaciones con contribuciones

- **Caso/unidad:** agente necesita conectar dos episodios por una relación
  explícita y explicar de dónde salió. Extraer validación, agregación y
  materialización [L3, L15, L16], no asumir desambiguación general por LLM.
- **Delta/seam:** ampliar `apply_triplet_with_source` y grafo existente;
  no introducir otra ontología. A consume esta evidencia; B/E son requisito
  cuando se comparte y elimina, C acredita qué parte llegó al modelo.
- **Contrato Rust propuesto:** `plan_materialization(scope, source_revision,
  assertions) -> Result<MaterializationPlan, MaterializeError>` con entidades
  de identidad contextual, relaciones tipadas, contribuciones y proyecciones.
  `commit(plan, expected_revision) -> Receipt`; `InvalidAssertion`,
  `IdentityAmbiguous`, `Denied`, `RevisionConflict`, `ProjectionPending`.
- **Acciones:** inventariar esquema bitemporal/linaje → validar assertion sin
  mutar → registrar superset/ownership → publicar primario → proyectar índices
  existentes con diagnóstico. Extracción LLM fuera de locks/commit y opcional;
  cancelación previa descarta plan, posterior exige receipt/reparación E.
  Migración aditiva sin convertir pesos históricos en confianza; feature flag
  impide nueva extracción, conserva lectura/retirada de derivados existentes.
- **Alternativas:** tripletas deterministas del caller primero; adapter de
  extracción opcional después; reutilizar schema actual frente a traducción
  directa de nombres/hashes upstream. No copiar normalización universal.
- **FMEA:** alias fusionados → relación falsa, detectada por IDs/contribuciones;
  proveedor falla tras escribir grafo → vector pendiente, visible por receipt;
  carreras sobre misma entidad → actualización perdida, mitigada por fence
  de revisión/serialización del backend. No sostener lock mientras llama al LLM.
- **Aceptación pendiente/no objetivos:** dos documentos comparten objeto,
  contradicción temporal no destruye fuente antigua, baja reconstruye sólo
  supervivientes y fallo vectorial no se anuncia como publicación íntegra.
  No comunidades Leiden, entrenamiento ni mejora de QA prometida.

### Contraste concreto, UI y cobertura final de autoría

Todas las rutas Vanta siguientes se refieren a
`97c9623c8ef5a70fdb51aff215bfbb3b05b15612` [VANTA-REFERENCE](VANTA-REFERENCE.md).

| Capacidad | Estado y fuente Vanta | Decisión específica |
|---|---|---|
| Recall autorizado | Existe: `core/hooks/auto_recall.rs::perform_auto_recall_governed`, `InjectionPolicy` | A debe atravesarlo; existencia no certifica todos los transports |
| Presupuesto | Existe: `context_engine/engine.rs::assemble_with_recall:252`, `apply_recall_budget:1159–1189` | Tokens estimados y caracteres no intercambiables; prefijo protegido puede exceder budget |
| Tripletas con fuente | Existe: `core/ontology/mod.rs:147–212` [LV1] | F añade materialización/identidad contextual, no “grafo desde cero” |
| Procedencia de chunks | Existe: `src/wiki/ingestors.rs:1–89` [LV3] | `source/page/chunk` no acredita spans versionados C |
| Merge documental | Existe: `ingest/merge.rs::commit:239–289` | Serial por página; no deducir transacción de toda ingesta |
| Reconstrucción integral compartida | Parcial: linaje y checkpoints existentes | B/E requieren verificar cobertura por objeto y backend; no bug demostrado |
| Studio | Existe según referencia: worktree `develop-desktop` `0ad38c542` | Extender inspector/actividad; no se certificó runtime ni se auditó ese frontend en esta ficha |

WebUI upstream: `AppRouter.tsx:65–128` redirige a login, bloquea rutas no
autenticadas y ofrece pantalla de error de migración de settings; no basta
para acreditar autorización backend. Flujo mínimo: cliente autorizado →
API query estructurada/streaming → entidades/relaciones/chunks/referencias →
visualización; grafo usa Sigma/Graphology, no shell Tauri. Las oportunidades
Studio son manifest de entrega, fase de retirada y proyección pendiente,
no otro chat. Autorización completa, todos los adaptadores, parsers y estados
post-crash siguen no acreditados; comentario completo de issues truncados
no leído. Consulta pública adicional de #3659/#3978 leyó sus cuerpos visibles;
metasearch Bing/DuckDuckGo devolvió vacío, no prueba de inexistencia.

**Separación futura (>300 líneas):** este informe permanece autónomo por el
encargo; si se aprueba trabajo, separar specs B/E (retirada), C/D (entrega)
y F/A (enriquecimiento), manteniendo aquí decisiones, matriz y enlaces.
No se crean specs, harnesses, prompts, JSON ni tareas. Cierre de autoría,
pendiente de revisión independiente y aceptación del responsable.

[L14]: https://github.com/HKUDS/LightRAG/blob/aaebfe1bb8d55deb6a4304d357a3bf4c92fcd40a/pyproject.toml#L5-L155
[L15]: https://github.com/HKUDS/LightRAG/blob/aaebfe1bb8d55deb6a4304d357a3bf4c92fcd40a/lightrag/operate.py#L2727-L3465
[L16]: https://github.com/HKUDS/LightRAG/blob/aaebfe1bb8d55deb6a4304d357a3bf4c92fcd40a/lightrag/operate.py#L3514-L3725
[L17]: https://github.com/HKUDS/LightRAG/blob/aaebfe1bb8d55deb6a4304d357a3bf4c92fcd40a/lightrag/lightrag.py#L6015-L6725
[L18]: https://github.com/HKUDS/LightRAG/blob/aaebfe1bb8d55deb6a4304d357a3bf4c92fcd40a/docs/design/PurgeRecoveryContract.md
[LV1]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/ontology/mod.rs#L147-L212
[LV2]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/context_engine/engine.rs#L228-L280
[LV3]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/wiki/ingestors.rs#L1-L89

## 12. Cierre de autoría: multimodalidad, contratos y Studio

**Estado: autoría entregada para revisión independiente, no aceptada.**
Ampliación del 2026-11-07 sobre los mismos clones/pins. Esta sección completa
los contratos anteriores; no sustituye sus límites por una garantía de runtime.
Se releen #3659/#3978 mediante `websearch.fetchWebContent`: ambos aparecen
**Open**; el HTML sigue truncando actividad. Consulta multi-engine
`site:github.com/HKUDS/LightRAG/issues delete document graph error` devolvió
cero resultados y Bing 301; acceso directo oficial sí devolvió los cuerpos.
No se deduce del RFC un fix integrado ni se actualiza la metadata congelada.

### 12.1. Flujo multimodal y asincronía realmente observados

En [pipeline.py::_run_pipeline_batch:2619–2698][LM1], el batch fija snapshot
del registro de parsers y crea colas acotadas parse/analyze/process. La
concurrencia de parse se agrupa por parser; inserción usa semáforo. Un token
identifica al owner del run; el watcher traslada cancelación compartida a
`threading.Event` para parsers síncronos/bridge LLM. No es una nueva cola durable:
el estado documental sigue siendo el ancla de recuperación.

`_analyze_worker:4805–4948` comprueba cancelación antes de la llamada y
captura `PipelineCancelledException` durante ella. Distingue análisis omitido,
completado y soft-failed: no escribe timestamp de éxito cuando no hay prueba;
si falla escribe `FAILED`, y si también falla esa escritura registra el error
secundario. Esto no garantiza que cancelar una llamada HTTP detenga cobro o
ejecución remota [LM2]. Las transiciones no revierten chunks ya persistidos.

`analyze_multimodal:6841–6945` consulta opciones `i/t/e` por documento;
sidecars de imágenes/tablas/ecuaciones se conservan aunque no se activen.
Sin bloques/opciones marca skipped, no éxito VLM. Su lectura de opciones tiene
fallback local a vacío al capturar excepción: describirlo no equivale a
aprobarlo como política de autorización. `_analyze_text_modality:7411–7470`
requiere rol EXTRACT para tablas/ecuaciones, valida formato HTML/JSON y trata
tabla sin cuerpo como skip; una ecuación vacía falla [LM3].

`process_single_document:5593–5689` añade chunks multimodales después de los
textuales, hard-split antes de embedding y backfill de spans contra bloques
para rutas elegibles. No intenta ubicar un resumen arbitrario de callback como
si fuese texto original. `_build_mm_chunks_from_sidecars:7956–8075` sólo
materializa análisis `success`, rechaza `failure` y filtra modalidades de la
revisión actual: optar fuera no debe resucitar sidecars antiguos. Conserva
heading/sidecar y `llm_cache_list` para retirada de caché [LM4].
Así, imagen→descripción→chunk→extracción→entidad/relación tiene linaje, pero
la descripción sigue siendo salida del modelo, no evidencia visual verificada.
Los roles de extracción/generación son necesarios en sus rutas upstream;
en Vanta la ingesta determinista y el recall deben funcionar sin proveedor.

### 12.2. G — derivados multimodales autorizados y retractables

1. **Problema agente:** recuperar una tabla de un informe, explicar página y
   revisión, y retirar su descripción al revocar el documento o la modalidad.
2. **Fuente/unidad:** sidecar tipado + resultado de análisis + chunk publicado,
   no copiar parsers ni prompts: [LM2–LM4], SHA LightRAG de §1.
3. **Delta:** `Ingestor`/`IngestChunk` ya existen en `src/wiki/ingestors.rs:45–89`;
   no está acreditada allí la unión revisionada modalidad→artefacto→contribución.
4. **Alternativas:** extender metadata del ingestor, adapter opt-in del host o
   referencias a objetos externos; rechazar VLM obligatorio y otro almacén binario.
5. **Rust propuesto:** `plan_modality(source_id, expected_revision, modality,
   artifact_ref, analysis_outcome) -> Result<ModalityPlan, ModalityError>`;
   salida con IDs de chunks/contribuciones y fingerprint del extractor.
   Errores `Denied`, `InvalidArtifact`, `AnalysisFailed`, `RevisionConflict`,
   `Cancelled`, `ProjectionPending`; no convertir failed en skipped.
   Invariantes: derivado no es original, opt-out no revive éxito antiguo y
   ningún path recibido concede acceso al filesystem.
6. **Seam real:** ingestors citados → `ingest/merge.rs::{CandidatePage,commit}`
   y `core/ontology/mod.rs::apply_triplet_with_source`; para estado,
   `ingest/callback.rs::{IngestProgress,ProgressTracker::update_progress}`.
7. **Pasos/persistencia:** inventariar campos → validar artefacto/revisión sin
   mutación → analizar fuera de locks → guardar contribuciones B → publicar
   generación F → retirar por E. Migración aditiva marca legacy desconocido;
   backfill sólo desde originales retenidos. Rollback detiene nueva extracción
   y preserva capacidad de leer/retirar lo ya publicado.
8. **Dependencias:** C4 para identidad/revisión; B/E y C14 para stale/retirada;
   D/C9 para progreso; C/C7 para presupuesto/cita. No bloquea recall textual.
9. **FMEA:** opt-out tras éxito → sidecar reutilizado → evidencia revocada;
   detectar fingerprint/opciones distintos y filtrar antes de publicar.
   Cancelación durante VLM → salida tardía → revisión obsoleta; fence al commit.
   URL/ruta hostil → parser accede a recurso privado → fuga; allowlist del host
   y resolución autorizada, no petición arbitraria desde el motor.
10. **Aceptación futura:** normal tabla con fuente resoluble; adversos fallo de
    parser, cancelación, modalidad retirada, dos updates simultáneos y artefacto
    fuera de scope. Observar IDs opacos, revisión, fase, conteos, duración y
    clase de error; sin imágenes/texto/prompts/credenciales por defecto.
11. **Mínimo/no-goals/abierto:** una modalidad y un ingestor existente; no OCR
    universal, fórmula nueva de ranking ni corpus de evaluación nuevo.
    Retención de binarios y licencia de cada parser/modelo quedan por decidir.

### 12.3. Completitud por candidata y relaciones transversales

Los puntos 1–8 de A–F se desarrollan en §2/§11. Esta tabla concreta detección,
observabilidad y decisiones abiertas de sus puntos 9–11; no son implementaciones.

| Ficha | Trigger → mecanismo → efecto; detección/mitigación | Aceptación futura normal/adversa, observabilidad, mínimo y abierto |
|---|---|---|
| A | Hub/homónimo → expansión mezcla identidades → contexto falso; detectar IDs/camino y revisionar semillas antes de resolver vecinos | Normal camino autorizado; adversa seed stale/timeout/tenant cruzado; contadores por brazo/omisión sin payload. Mínimo un salto acotado por budget existente; política de resolución de alias abierta |
| B | Ledger truncado → faltan supervivientes → rebuild destructivo; detectar coverage unknown frente a vacío autoritativo y bloquear retirada | Normal fuente compartida; adversa corrupción/IO/retry; conteos de contribuciones y revisión, no descripciones. Mínimo extender linaje existente; representación/retención abiertas |
| C | Recorte cambia evidencia → manifest previo al render cita lo no enviado; detectar comparación contra salida final y emitir después | Normal ordinal resuelve span; adversa rename/revisión/abort preenvío; IDs/digests/estado prepared-delivered separados, sin cuerpo. Mínimo fuente/revisión; ACK host/proveedor y precisión de coordenada abiertos |
| D | Timeout tratado como cero hits → fallback oculta fallo → respuesta engañosa; detectar estado por etapa y mantener ACL | Normal éxito/vacío distintos; adversa denied/timeout; clase/cantidad de omisiones sin query. Mínimo ampliar reporte; compatibilidad serializada de SDK abierta |
| E | Cancel/crash entre derivados y anchors → atribución perdida → no se puede reparar; detectar fase durable y conservar portadores hasta completar | Normal baja con supervivientes; adversa interrupción cada frontera/revisión sucesora; operation/fase/error sin texto. Mínimo una fuente/un backend; política de reanudación autorizada abierta |
| F | Dos planes de entidad comparten revisión vieja → último escritor pierde contribución; detectar expected-revision y serializar commit actual | Normal tripletas con dos fuentes; adversa vector fallido/alias/contradicción; IDs/revisión/estado de proyección sin assertion. Mínimo entradas deterministas; vocabulario/identidad y extractor opt-in abiertos |

**C7:** preservar dominio del score (entidad/relación/chunk) y presupuesto
compartido, sin importar los pesos/modos como RRF. **C9:** análisis omitido,
fallido y retirado son estados diferentes; usar worker/checkpoints actuales.
**C4:** ID estable de fuente distinto del nombre normalizado y ordinal de cita.
**C14:** ledger íntegro para retirar y fingerprint para detectar stale; diagnose
no dispara reparación destructiva automáticamente. **C16:** inspector muestra
revisión/fase realmente leídas, no “grafo listo” inferido de ACK.

`WriteReceipt` **ya existe** en [src/attestation.rs:654–738][LVR]:
versión, proyección de contenido, superficies store/WAL y SHA256; **no firma
digital**, ni prueba todos los derivados ni ACK de entrega al modelo. Cualquier
`MutationReceipt` para estados E/F sería **propuesto**, extensión/adaptación a
evaluar, nunca afirmar que falta todo recibo o que sustituye WAL/sync.

### 12.4. Matriz complementaria y flujo de interfaz

| Capacidad | Upstream observado | Vanta pin 97c9623: estado/fuente | Delta / decisión |
|---|---|---|---|
| Retrieval | Entidades/relaciones/chunks, modos L5–L7 | Existe: `src/sdk/search/{hybrid,fusion}.rs`, HNSW/BM25/RRF | A añade evidencia, no fusión nueva |
| Memoria | Documentos/chunks y cachés no son sesiones de agente | Existe: `core/abstractions/types.rs::MemoryRecord`, dedup/supersession en `core/record/` | Conservar dominio conversacional |
| Budget/quotas | Caps de contexto y etapas, no cuota universal | Existen: `assemble_with_recall`, `apply_recall_budget`, `src/config.rs:760–762` | No confundir chars/tokens/bytes |
| Ingesta | Colas parse/analyze/process y estado documental LM1–LM2 | Existe: `ingest/worker.rs::{begin,execute}`, `TaskCheckpointManager`, scheduler de referencia | G se monta ahí; cobertura integral parcial |
| Provenance/retirada | Anchors, tracking, journal L16–L18 | Existe linaje LV1; equivalencia de retirada integral no acreditada | B/E prueban por revisión, no nueva WAL |
| Receipt | Estado/flush por store | Existe LVR; receipt de toda materialización no acreditado | Propuesta acotada, no firma |
| UI | React web y API LM5 | Studio existente LS1, pin separado; jobs no acreditados en ese pin | Extender diagnóstico; cancel/progreso nuevos sólo propuestos |

WebUI real: `queryTextStream` hace POST `/query/stream` con `AbortSignal`;
`deleteDocuments` envía IDs/opciones a `/documents/delete_document`;
`getPipelineStatus` y `cancelPipeline` consultan/solicitan cancelación, cuyo
retorno es `cancellation_requested`, **no trabajo ya revertido** [LM5].
Son rutas del cliente leído, no prueba end-to-end de autenticación.

Studio: cuerpos releídos mediante `git show` al SHA separado
`0ad38c542b39a589035b0e17fdedb4c09dca24b9`, ruta
`feature-freez/desktop/src/components/memory/MemoryLens.tsx` [LS1].
`ScenesPanel:57-96` carga/abre escenas; `PersonaPanel:172-206` carga snapshot;
`SkillsPanel:266-297` agrupa versiones; `GenlogPanel/openAnchor:373-412`
consulta generation-log y `get(anchorId)` real. No son los rangos del worktree.
**LR-1:** `JobChip.tsx` y `commands/jobs.rs` no existen en el árbol del pin;
en disco son `??` bajo `desktop/`, no evidencia shipped ni recuperable por SHA.
Se retiran sus enlaces y garantías atribuidas; no se acreditan sus cuerpos
dirty en esta corrección. C16 propone presentar fases/contribuciones/proyección
en Studio existente; cancelación entre chunks, IDs parciales y registro durable
de jobs quedan como contrato futuro, no comportamiento del pin.
FMEA: UI equipara cancel-request con rollback → ocultar derivados aplicados;
mitigar mostrando solicitud/resultado por recurso desde su owner. Aceptación
futura: cancel antes/después de escritura y reinicio muestran fase real y
parcialidad sin revivir fuentes ni exponer previews revocados. No se ejecutó UI.

**Cobertura/lagunas restantes:** parsers completos, todas las ramas de roles,
backend recovery y autorización extremo a extremo no auditados; PRs y
comentarios truncados de issues no acreditados; no métricas propias ni SBOM
transitivo de parsers/modelos. §7 conserva runners/defaults/denominadores y
adapter Vanta; no se diseñó harness. Frente a §11, se amplió la lectura multimodal
y MemoryLens de Studio; jobs quedan sin acreditar al pin, no “UI ausente”.
LR-1 corregido no implica aceptación final ni prueba de durabilidad UI.
Separación futura: specs G/F producción, B/E retirada, A/C/D consumo/UI; este
informe sigue autónomo y pendiente de revisión, sin crear esas specs.

[LM1]: https://github.com/HKUDS/LightRAG/blob/aaebfe1bb8d55deb6a4304d357a3bf4c92fcd40a/lightrag/pipeline.py#L2619-L2698
[LM2]: https://github.com/HKUDS/LightRAG/blob/aaebfe1bb8d55deb6a4304d357a3bf4c92fcd40a/lightrag/pipeline.py#L4805-L4948
[LM3]: https://github.com/HKUDS/LightRAG/blob/aaebfe1bb8d55deb6a4304d357a3bf4c92fcd40a/lightrag/pipeline.py#L6841-L6945
[LM4]: https://github.com/HKUDS/LightRAG/blob/aaebfe1bb8d55deb6a4304d357a3bf4c92fcd40a/lightrag/pipeline.py#L7956-L8075
[LM5]: https://github.com/HKUDS/LightRAG/blob/aaebfe1bb8d55deb6a4304d357a3bf4c92fcd40a/lightrag_webui/src/api/lightrag.ts#L879-L1145
[LVR]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/attestation.rs#L654-L738
[LS1]: https://github.com/ness-e/VantaDB/blob/0ad38c542b39a589035b0e17fdedb4c09dca24b9/feature-freez/desktop/src/components/memory/MemoryLens.tsx#L57-L412

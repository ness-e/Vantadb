---
title: "LightRAG: recuperación por entidades y relaciones con linaje reconstruible"
kind: research
status: proposed
description: "Lectura estática de LightRAG y candidatos acotados de enriquecimiento, trazabilidad y reconstrucción para VantaDB"
tags: [research, memory, engineering]
---

# LightRAG: recuperación por entidades y relaciones con linaje reconstruible

## 1. Recomendaciones para VantaDB y contexto arquitectónico

**Dictamen:** estudiar un enriquecedor opcional de candidatos mediante entidades/relaciones y, sobre todo, su contabilidad de contribuciones. No sustituir el motor de VantaDB ni importar el servicio completo. No hay mediciones propias de calidad, latencia o coste.

- Repositorio: `HKUDS/LightRAG`, clon `C:\Users\Eros\LightRAG`.
- Revisión obtenida mediante `git rev-parse HEAD`: `aaebfe1bb8d55deb6a4304d357a3bf4c92fcd40a`.
- Snapshot de [github-candidates.json](github-candidates.json): **40 591 estrellas**, MIT, no archivado, observado `2026-11-07T06:56:23.2154091Z`. No se reconsultaron estrellas.
- Contrato rector: [METHOD.md](METHOD.md); revisión superficial, no auditoría integral. El manifiesto de preparación es [clone-manifest.json](clone-manifest.json).
- Stack acreditado: Python asíncrono; `LightRAG` compone roles LLM, migraciones y pipeline. Cuatro interfaces separan KV, vectores, grafo y estado documental. Defaults: JSON KV, NanoVectorDB y JSON de estados; los adaptadores son intercambiables, no una transacción distribuida [L1, L2].
- La WebUI tiene fuente React/TypeScript, Vite y Sigma/Graphology [L12]; no es un escritorio nativo certificado.

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

**Issues upstream no consultados.** No se atribuyen CVE, regresiones ni correcciones históricas. Los tests leídos son evidencia de intención/camino, no resultado ejecutado.

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

Lectura estática del SHA indicado, sin servicios, ejecución de código, tests, builds, instalación ni benchmarks. Las URLs son enlaces inmutables a la revisión local, no una consulta de issues o de la versión publicada.

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

**Baseline local mutable:** VantaDB HEAD observado `c561158253bb88192590fa406c61e0a27dd7df8d`. Al leer estaban modificados `core/ontology/mod.rs`, varios archivos `context_engine/*` e `ingest/merge.rs`. Las líneas Vanta de las fichas describen esa lectura, no un snapshot certificado del worktree ni el estado de un bug histórico. No se declara ninguno abierto/cerrado. Revalidación obligatoria al convertir propuesta en tarea.

## 7. Benchmarks existentes y adaptación del protocolo

- `RAGEvaluator` en `lightrag/evaluation/eval_rag_quality.py` carga JSON de preguntas/respuestas de referencia; consulta API con referencias y contenido de chunks; usa RAGAS con LLM/embeddings de evaluación. Métricas: faithfulness, answer relevancy, context recall y context precision; el promedio RAGAS excluye NaN y un caso sin métricas válidas falla [L11]. La agregación separa casos válidos: conservar errores/denominadores según METHOD, no solo el promedio.
- Dependencias/entrada: API ya indexada, credenciales del modelo generador y juez, `ragas`, `datasets`, proveedor de embeddings y fichero explícito; `sample_dataset.json` es material de ejemplo, no certificación general. El coste incluye retrieval, generación y juez; el tiempo de indexación queda fuera si se usa un servicio ya preparado.
- `offline_retrieval_check.py` carga documentos Markdown y oráculo de ficheros relevantes; usa su propio scoring lexical con IDF, calcula recall@k y cobertura. **No ejecuta el grafo LightRAG ni demuestra calidad de sus embeddings.** Sin juez LLM; el modo estricto del runner es upstream, no criterio propuesto para VantaDB.
- `test_references_consistency` [L13] inspecciona endpoints; no es benchmark de veracidad. No se ejecutó.
- Adaptación única: pistas retrieval y respuesta separadas de [METHOD](METHOD.md#protocolo-de-comparacion-de-benchmarks), mismo corpus/split/modelo/budget, baseline Vanta actual frente a un delta. Registrar intentos, fallos, abstenciones, cardinalidad y costes por etapa. No se redactan prompts, jueces, umbrales ni configuraciones nuevas.
- Riesgo operativo: no usar workspace compartido; preparar/eliminar datos para indexación puede destruir material o contaminar otro ensayo. No se autoriza ejecutar limpieza, ingestión ni llamadas pagadas. Las cifras de proveedor no son mediciones de esta investigación.

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

**Implementado en fuente:** WebUI React y routers de login/workspace [L12], API de documentos/grafo/consulta localizadas. **No acreditado:** escritorio nativo, despliegue funcional, equivalencia de todos los controles UI con permisos de backend. Studio Vanta ya existe en `develop-desktop` con memoria/actividad/escenas/skills/diagnóstico según METHOD; no se inspeccionó ese worktree.

| Superficie | Cobertura estática / pendiente |
|---|---|
| Indexación | Entradas, selección de chunker y persistencia/merge focales; parsers multimodales completos no examinados |
| Retrieval | Keywords, entidad/relación, vecinos, contexto, referencias y rutas de vacío; reranker/provider completo no auditado |
| Actualización/borrado | Tracking, prueba de purge, reservas y llamadas rebuild; no demostración de crash safety de cada backend |
| Seguridad | Workspace JSON y puntos de autenticación localizados; autorización completa, deployments y backends remotos no auditados |
| Tests | Cuerpo de test HTTP de referencias leído; suites de concurrencia/purge no ejecutadas ni exhaustivamente revisadas |
| Runtime/documentación | Todo runtime y gates de enlaces/frontmatter pendientes manuales; no scripts, commits ni push en esta tarea |

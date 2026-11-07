---
title: "Microsoft GraphRAG: comunidades jerárquicas y recuperación global, local y DRIFT"
kind: research
status: proposed
description: "Análisis estático de extracción, comunidades, contexto y actualización de GraphRAG con deltas acotados para VantaDB"
tags: [research, memory, engineering]
---

# Microsoft GraphRAG: comunidades jerárquicas y recuperación global, local y DRIFT

## 1. Recomendaciones para VantaDB y contexto arquitectónico

**Dictamen:** el diferencial es una **vista materializada jerárquica de comunidades con fuentes**, no “tener un grafo” ni reemplazar retrieval híbrido. Evaluar primero contexto de grafo explicable; global y DRIFT añaden múltiples llamadas LLM y un problema de actualización de derivados que no debe introducirse sin contrato.

- Repositorio/clon: `microsoft/graphrag`, `C:\Users\Eros\graphrag`.
- SHA obtenido por `git rev-parse HEAD`: `34b2b67ae4601ad0b47bcd179fbe3f2a23aa0857`.
- [Snapshot congelado](github-candidates.json): **36 585 estrellas**, MIT, no archivado, `2026-11-07T06:56:22.4417547Z`; sin reconsulta de estrellas.
- Aplican [METHOD.md](METHOD.md) y [clone-manifest.json](clone-manifest.json). No se ejecutaron servicios, scripts, código, tests ni benchmarks.
- Stack de la revisión: Python `>=3.11,<3.14`, paquete `graphrag` con componentes separados de input/chunking/LLM/cache/storage/vectors; Pandas, Arrow/Parquet, Pydantic, NetworkX y `graspologic-native`. El manifiesto fija la línea de Leiden/NLP porque cambios de dependencias alteran golden data [G1].

VantaDB ya tiene grafo/ontología, BM25/HNSW/RRF, procedencia, dedup, sesiones, escenas y assembly compartido. Las fichas proponen **derivados opcionales sobre esas primitivas**, no adoptar la plataforma Python completa.

**No son los modos de LightRAG:** aquí `global` opera sobre informes de comunidades con map/reduce; `local` selecciona entidades y mezcla tablas relacionadas/fuentes; **DRIFT** usa informes para iniciar un grafo de acciones de búsqueda local con preguntas de seguimiento y reducción. LightRAG `global` recupera relaciones por embeddings de keywords altas, no este pipeline.

## 2. Partes extraíbles, flujo real y mapeo a Rust

### Indexación, modelo y resolución

1. El pipeline carga documentos/text units; `extract_graph` aplica `GraphExtractor` por fila y agrega resultados. La extracción LLM inicial puede hacer **gleanings** adicionales y consultas de continuación; no es un único pase necesariamente [G2].
2. `_process_result` normaliza nombres/tipos a mayúsculas y genera registros con `source_id` del text unit. `_merge_entities` agrupa por **`title,type`**, agrega descripciones, IDs de text units y frecuencia. Relaciones se agrupan por pareja `source,target`, sumando peso. Se filtran relaciones huérfanas [G2].
3. `summarize_descriptions` resume descripciones agregadas de entidades y relaciones; no confundir ese resumen con informes de comunidades [G3].
4. `cluster_graph` normaliza dirección de aristas, elimina duplicados de pareja conservando la última fila, opcionalmente usa el componente conexo mayor y ejecuta Leiden jerárquico con seed. Emite `(nivel, comunidad, padre, miembros)` [G4]. La dirección semántica de la relación no se conserva como tal en el clustering.
5. `create_community_reports` lee entidades, relaciones, comunidades y claims opcionales; construye contexto local y resume por niveles. Cuando un contexto no cabe, el builder puede sustituir detalle por informes de subcomunidades y recortar lo que aún no cabe [G5].
6. `CommunityReport` contiene `community_id`, resumen, contenido completo, rank y atributos; el modelo también contempla embedding, tamaño y período. Entidades/relaciones conservan referencias a text units; los IDs legibles usados en citas son distintos del ID durable [G6].
7. Resolución de entidades **no equivale a entity linking semántico general**. La actualización `_group_and_resolve_entities` empareja solo por título, conserva primer ID/tipo/grado y concatena descripciones/fuentes; difiere de la agrupación inicial por título+tipo [G7]. No se acredita conciliación de homónimos o alias.

### Consulta, contexto, citas y actualización

- **Local:** `LocalSearchMixedContext.build_context` puede unir preguntas anteriores a la consulta. `map_query_to_entities` busca embeddings de descripciones y permite incluir/excluir nombres; vacío usa rank. Mezcla informes, entidades, relaciones, covariates y text units con fracciones del presupuesto restante tras historial. Las tablas pueden devolver candidatos con `in_context` para distinguir seleccionado de entregado [G8].
- **Global:** `GlobalSearch.search` obtiene lotes de informes, realiza map paralelo y reduce puntos. La reducción descarta score cero, ordena y corta por `max_data_tokens`. Sin puntos y sin `allow_general_knowledge`, devuelve `NO_DATA_ANSWER`; permitir conocimiento general cambia grounding. Las categorías de llamadas/tokens se devuelven en el resultado [G9].
- **DRIFT:** el primer contextualizador selecciona informes y el primer genera respuesta/seguimientos; `query_state` mantiene un grafo de acciones. Cada época ordena incompletas, limita seguimientos y llama a local search; al alcanzar profundidad/agotamiento serializa y puede reducir [G10]. No es un recorrido determinista de todos los nodos ni garantía de exhaustividad.
- **Citas:** prompts piden `[Data: Sources (...); Reports (...); ...]` con IDs legibles; la aplicación recibe además registros/contexto. Esto permite inspección, pero las citas escritas por el modelo no son una prueba automática de entailment ni de que todos los claims llegaron a la respuesta [G11].
- **Incremental:** `get_delta_docs` compara **títulos**, genera nuevos/eliminados; `load_update_documents` devuelve solo `new_inputs`. Por esa ruta un fichero con igual título y contenido cambiado no se reconoce como nuevo. No acreditar propagación de borrados por el mero hecho de existir `deleted_inputs`. La conciliación concatena/agrupa estados previos y delta y recalcula ciertos derivados [G7, G12].
- **Fallo/aislamiento:** local search captura excepciones y devuelve un `SearchResult`; no hay que confundirlo con abstención legítima. Los builders reciben diccionarios, almacenes y estado ya seleccionados; estas firmas no certifican ACL por tenant. Compartir tablas, vector store o una instancia con `query_state` exige aislamiento externo [G8, G10].

### Fichas candidatas — contratos propuestos, no existentes

**A. Snapshot de comunidad con dependencia explícita**

- Problema/unidad: responder preguntas transversales sin enviar todo el corpus; extraer el registro jerárquico y su vinculación a fuentes, no el pipeline completo. Símbolos: `cluster_graph`, `cluster_graph.py:20–99`, `CommunityReport`, `community_report.py:12–66` [G4, G6].
- Seam Vanta: `vanta-memory/src/core/ontology/mod.rs:19–51,147–212` ya almacena hechos/grafo/linaje; `MemoryRecord` admite metadatos.
- Contrato Rust **propuesto**: `CommunitySnapshot { scope, revision, members, parent, source_revisions, summary_ref }`; proveedor opcional `CommunitySummarizer::summarize(snapshot) -> Result<DerivedEvidence, SummaryError>`.
- Dependencias/acciones: separar partición pura de generación LLM, definir IDs/revisiones y guardar contribuciones antes del resumen; invalidar ante cambio de miembros/fuentes.
- FMEA: resumen obsoleto tras borrado, comunidades entre tenants o exclusión LCC que oculta fuentes; conservar cobertura, autorización y estado `stale`.
- Aceptación pendiente: cada resumen resuelve miembros/fuentes vigentes autorizadas; cambios invalidan derivados; falta de resumen permite baseline actual.
- No objetivos: segundo grafo, importar Leiden obligatoriamente, resumir escenas existentes otra vez o activar generación dentro del motor embebido.

**B. Contexto de grafo con candidatos y entrega diferenciados**

- Unidad/símbolos: `LocalSearchMixedContext.build_context`, `mixed_context.py:91–221`; flags `in_context` en `224–375` [G8].
- Seam Vanta: `perform_auto_recall_governed`, `core/hooks/auto_recall.rs:399–408`; `assemble_with_recall`, `context_engine/engine.rs:252–264`.
- Contrato Rust **propuesto**: `GraphContextProjection { candidates, delivered_ids, source_manifest, exclusion_reasons }`; consume presupuesto remanente del assembly existente.
- Dependencias/acciones: adaptar tablas a candidatos de memoria, filtrar por governance antes de expansión y conservar IDs por etapa.
- FMEA: confundir candidato con evidencia enviada o presupuestar cada tabla de forma independiente; usar una sola contabilidad efectiva.
- Aceptación pendiente: diagnóstico separa recuperado/entregado; IDs de citas pertenecen a entregados; historial y grafo comparten el presupuesto actual.
- No objetivos: copiar proporciones upstream como configuración Vanta ni duplicar compactación/context_assemble.

**C. Invalidador de derivados por revisión documental**

- Unidad/símbolos: contraste de `get_delta_docs`, `incremental_index.py:29–58`, y `_group_and_resolve_entities`, `entities.py:14–74` [G7, G12].
- Seam Vanta: `vanta-memory/src/ingest/worker.rs:167–250` y `ingest/merge.rs`; dedup/supersession/linaje existentes.
- Contrato Rust **propuesto**: `DerivedInvalidation::plan(scope, old_revision, new_revision, dependents) -> Result<InvalidationPlan, LineageError>`; título no es clave de revisión.
- Dependencias/acciones: reutilizar hash/versión y sincronización actuales; enumerar dependencias, invalidar resúmenes y embeddings afectados sin borrar fuentes compartidas.
- FMEA: mismo título oculta cambio o agrupación solo nominal mezcla tipos; evidenciar identidad y revisión independientes de presentación.
- Aceptación pendiente: actualización de contenido con nombre constante afecta derivados; borrar una fuente no elimina pruebas supervivientes; reintentos idempotentes.
- No objetivos: reimplementar ingestors wiki, dedup o una limpieza completa del corpus por cada edición.

**D. Exploración iterativa con estado acotado por consulta**

- Unidad/símbolos: bucle de acciones y serialización de `DRIFTSearch.search`, `drift_search/search.py:188–305` [G10].
- Seam Vanta: worker/scheduler/checkpoints existentes y `perform_auto_recall_governed`; integración sería de orquestación, no del motor.
- Contrato Rust **propuesto**: `EvidenceExploration::step(query_scope, frontier, remaining_resources) -> Result<ExplorationStep, ExplorationError>`, con fuentes y motivo de terminación.
- Dependencias/acciones: sesión de exploración propia, cancelación y telemetría reutilizadas; llamadas LLM opcionales; entregar evidencias a assembly actual.
- FMEA: seguimientos redundantes inflan costes o estado de una pregunta contamina otra; ownership por consulta/tenant y terminación explícita.
- Aceptación pendiente: consulta nueva no hereda acciones ajenas, cancelación mantiene trazas y no entrega parciales como completos; baseline sin exploración permanece disponible.
- No objetivos: portar DRIFT completo, garantizar cobertura por “razonamiento” ni definir budgets/umbrales nuevos en esta investigación.

## 3. Defectos, riesgos e issues transferibles

| Nivel | Evidencia / configuración | Evaluación |
|---|---|---|
| Código [G12] | Delta documental por título y consumo de `new_inputs` | Limitación concreta de esa ruta para revisiones/borrados, no prueba de ausencia de otras herramientas |
| Código [G2, G7] | Identidad por título/tipo inicial y solo título incremental | Riesgo de colisión semántica y divergencia entre etapas; requiere caso discriminante, no afirmar corrupción observada |
| Código [G4] | Grafo no dirigido, keep-last y opción LCC | Pérdida deliberada de detalle para clustering; documentar qué evidencia deja de representar |
| Código [G9, G10] | Scores LLM y múltiples llamadas | Coste y selección dependen del modelo; score no es confianza calibrada ni recall medido |
| Código [G11] | Citas redactadas por modelo | Requieren resolución/validación aparte; un ID válido no demuestra soporte de la afirmación |
| Hipótesis | Instancia DRIFT/almacén reutilizados entre usuarios | Riesgo de contexto cruzado si el integrador comparte estado; no fuga reproducida ni ACL auditada |
| Código [G1] | Dependencias Leiden/NLP condicionan golden data | Cambio de biblioteca puede alterar particiones aun sin cambiar aplicación |

**Issues upstream no consultados.** No se importan defectos históricos ni se deduce corrección por cierre de incidencias. Se leyó test de combinación de relaciones previas/delta [G13]; no se ejecutó.

## 4. Prerrequisitos en VantaDB

1. Revalidar estado actual y tareas existentes antes de adoptar candidatos; METHOD impide presentar primitivas existentes como ausentes.
2. Inventariar dependencias fuente→hecho→comunidad→resumen→embedding con scope/revisión, incluyendo políticas de borrado.
3. Definir quién es dueño de orquestación pagada, cancelación, errores, caché y límites; LLM fuera de primitivas puras del motor.
4. Mantener autorización también al leer resúmenes y al resolver citas; un resumen puede filtrar un documento aunque el documento no se entregue.
5. [LICENSE MIT](https://github.com/microsoft/graphrag/blob/34b2b67ae4601ad0b47bcd179fbe3f2a23aa0857/LICENSE#L1-L21) no exime atribución ni licencias de modelos, corpus, dependencias y conectores. Una traducción Rust puede seguir siendo derivación.

## 5. Componentes y prácticas que no deben trasladarse

- Adoptar clustering, resúmenes y DRIFT a la vez sin aislar el beneficio de cada delta.
- Título como identidad de revisión; mayúsculas como resolución de entidades completa.
- Tablas/artefactos Parquet y vector store independientes como supuesto de atomicidad distribuida.
- Hacer equivalentes “no hubo contexto”, “falló el backend” y “abstención”; generar conocimiento general como fallback silencioso.
- Duplicar BM25/HNSW/RRF, dedup, escenas, scheduler o assembly de Vanta.

## 6. Advertencias metodológicas y evidencia

La revisión sigue el SHA del clon superficial, no una versión estable publicada. “Código” acredita un flujo leído; “hipótesis” exige reproducción futura. No hubo consultas de issues, mediciones ni auditoría integral de seguridad.

| Ref. | Evidencia inmutable y archivos leídos |
|---|---|
| G1 | [packages/graphrag/pyproject.toml:25–72](https://github.com/microsoft/graphrag/blob/34b2b67ae4601ad0b47bcd179fbe3f2a23aa0857/packages/graphrag/pyproject.toml#L25-L72): stack y pins sensibles a golden data |
| G2 | [extract_graph.py:23–129](https://github.com/microsoft/graphrag/blob/34b2b67ae4601ad0b47bcd179fbe3f2a23aa0857/packages/graphrag/graphrag/index/operations/extract_graph/extract_graph.py#L23-L129), [graph_extractor.py:90–178](https://github.com/microsoft/graphrag/blob/34b2b67ae4601ad0b47bcd179fbe3f2a23aa0857/packages/graphrag/graphrag/index/operations/extract_graph/graph_extractor.py#L90-L178): extracción, gleanings y agregación |
| G3 | [summarize_descriptions.py:27–140](https://github.com/microsoft/graphrag/blob/34b2b67ae4601ad0b47bcd179fbe3f2a23aa0857/packages/graphrag/graphrag/index/operations/summarize_descriptions/summarize_descriptions.py#L27-L140): puntos de resumen de descripciones, inspección focal |
| G4 | [cluster_graph.py:20–99](https://github.com/microsoft/graphrag/blob/34b2b67ae4601ad0b47bcd179fbe3f2a23aa0857/packages/graphrag/graphrag/index/operations/cluster_graph.py#L20-L99): Leiden jerárquico |
| G5 | [create_community_reports.py:41–140](https://github.com/microsoft/graphrag/blob/34b2b67ae4601ad0b47bcd179fbe3f2a23aa0857/packages/graphrag/graphrag/index/workflows/create_community_reports.py#L41-L140), [graph_context/context_builder.py:196–263](https://github.com/microsoft/graphrag/blob/34b2b67ae4601ad0b47bcd179fbe3f2a23aa0857/packages/graphrag/graphrag/index/operations/summarize_communities/graph_context/context_builder.py#L196-L263): informes y sustitución/recorte |
| G6 | [community_report.py:12–66](https://github.com/microsoft/graphrag/blob/34b2b67ae4601ad0b47bcd179fbe3f2a23aa0857/packages/graphrag/graphrag/data_model/community_report.py#L12-L66): modelo de informe |
| G7 | [update/entities.py:14–74](https://github.com/microsoft/graphrag/blob/34b2b67ae4601ad0b47bcd179fbe3f2a23aa0857/packages/graphrag/graphrag/index/update/entities.py#L14-L74): resolución incremental |
| G8 | [mixed_context.py:91–221](https://github.com/microsoft/graphrag/blob/34b2b67ae4601ad0b47bcd179fbe3f2a23aa0857/packages/graphrag/graphrag/query/structured_search/local_search/mixed_context.py#L91-L221), [entity_extraction.py:41–96](https://github.com/microsoft/graphrag/blob/34b2b67ae4601ad0b47bcd179fbe3f2a23aa0857/packages/graphrag/graphrag/query/context_builder/entity_extraction.py#L41-L96): contexto local y búsqueda de entidades; flags hasta `mixed_context.py:375` |
| G9 | [global_search/search.py:142–214](https://github.com/microsoft/graphrag/blob/34b2b67ae4601ad0b47bcd179fbe3f2a23aa0857/packages/graphrag/graphrag/query/structured_search/global_search/search.py#L142-L214), [306–400](https://github.com/microsoft/graphrag/blob/34b2b67ae4601ad0b47bcd179fbe3f2a23aa0857/packages/graphrag/graphrag/query/structured_search/global_search/search.py#L306-L400): map/reduce y tokens |
| G10 | [drift_search/search.py:188–305](https://github.com/microsoft/graphrag/blob/34b2b67ae4601ad0b47bcd179fbe3f2a23aa0857/packages/graphrag/graphrag/query/structured_search/drift_search/search.py#L188-L305): primer, acciones, seguimiento y reducción |
| G11 | [local_search_system_prompt.py:18–26](https://github.com/microsoft/graphrag/blob/34b2b67ae4601ad0b47bcd179fbe3f2a23aa0857/packages/graphrag/graphrag/prompts/query/local_search_system_prompt.py#L18-L26), [global_search_map_system_prompt.py:64–82](https://github.com/microsoft/graphrag/blob/34b2b67ae4601ad0b47bcd179fbe3f2a23aa0857/packages/graphrag/graphrag/prompts/query/global_search_map_system_prompt.py#L64-L82): protocolo de citas upstream, no prompt nuevo |
| G12 | [incremental_index.py:29–81](https://github.com/microsoft/graphrag/blob/34b2b67ae4601ad0b47bcd179fbe3f2a23aa0857/packages/graphrag/graphrag/index/update/incremental_index.py#L29-L81), [load_update_documents.py:48–62](https://github.com/microsoft/graphrag/blob/34b2b67ae4601ad0b47bcd179fbe3f2a23aa0857/packages/graphrag/graphrag/index/workflows/load_update_documents.py#L48-L62): delta y consumo de entradas |
| G13 | [test_update_relationships.py:1–85](https://github.com/microsoft/graphrag/blob/34b2b67ae4601ad0b47bcd179fbe3f2a23aa0857/tests/unit/indexing/update/test_update_relationships.py#L1-L85): fixtures y merge previo/delta |
| G14 | [test_cache_benchmark.py:22–98](https://github.com/microsoft/graphrag/blob/34b2b67ae4601ad0b47bcd179fbe3f2a23aa0857/tests/unit/cache/test_cache_benchmark.py#L22-L98): microbenchmark existente |
| G15 | [unified-search-app/app/home_page.py:1–65](https://github.com/microsoft/graphrag/blob/34b2b67ae4601ad0b47bcd179fbe3f2a23aa0857/unified-search-app/app/home_page.py#L1-L65): UI Streamlit con paneles de informes y citas |

**Baseline Vanta mutable:** HEAD `c561158253bb88192590fa406c61e0a27dd7df8d`; se leyeron `core/ontology/mod.rs:19–64,147–222`, `core/hooks/auto_recall.rs:391–455`, `ingest/worker.rs:167–250`, `ingest/merge.rs:1–110` y seams de assembly/worker. Ontología, archivos de `context_engine` y merge estaban modificados concurrentemente. No se declara vigente ni corregido ningún defecto histórico de VantaDB; las líneas son orientación para revalidar, no certificación del worktree.

## 7. Benchmarks existentes y protocolo de comparación

- Runner realmente localizado: `tests/unit/cache/test_cache_benchmark.py::test_bulk_read_cache_benchmark` [G14], opt-in mediante opción pytest. Prellena claves/valores representativos, compara `get_many` en Memory/JSON/SQLite y comprueba conjunto de claves.
- Métricas: mediana de tiempo de lecturas calientes, entradas/segundo y relación frente a SQLite. Dependencias: pytest asíncrono, caches y almacenamiento de fichero; usa `tmp_path`. **Sin juez LLM ni medida de retrieval, QA o memoria.** No extrapolar un benchmark de caché a “GraphRAG más rápido”.
- En las búsquedas acotadas de rutas `eval`, `benchmark` y notebooks no se acreditó un runner completo de calidad QA con dataset/juez congelados dentro del clon. Hay ejemplos DRIFT y datos de demostración, no evidencia de reproducción de resultados publicados. No se inventan instrucciones de ejecución ni un juez nuevo.
- Adaptación: aplicar pistas motor/retrieval/respuesta de [METHOD](METHOD.md#protocolo-de-comparacion-de-benchmarks) por separado. Para global/DRIFT registrar extracción, informes, embeddings, map, seguimiento, reducción y juez con presupuesto/generación equivalentes al baseline. El número de llamadas importa tanto como la latencia retrieval.
- Costes/riesgos: crear informes y consultas genera llamadas pagadas; compartir caché puede contaminar un ensayo; actualizaciones y regeneración de artefactos necesitan copia desechable. El microbenchmark escribe almacenes temporales. No se ejecutó ninguno ni se midió consumo propio.
- No se crean prompts, rubricas, graders, harnesses, umbrales, sampling ni configuraciones; los resultados de papers/proveedor quedan como claims no reproducidos.

## 8. Matriz frente a VantaDB

| Capacidad | Estado Vanta | Delta / límite de acreditación |
|---|---|---|
| BM25/HNSW/RRF, dedup, escenas y sesiones | Existe | No reemplazar |
| Grafo y hechos temporales con procedencia | Existe | No equiparar grafo básico con informes de comunidad |
| Compactación/assembly compartido | Existe | Integrar candidatos, no nuevo assembler |
| Partición jerárquica y resumen de comunidades versionado | No acreditado | Candidato A; no afirmar ausencia absoluta en todo el repo |
| Contexto con procedencia | Existe / parcial para tabla de candidatos-entregados | Candidato B concreta diagnóstico, no añade procedencia desde cero |
| Actualización por revisión/invalidez de derivados | Parcial | Ingest/supersession existentes; cobertura de comunidades no acreditada |
| Global map/reduce sobre comunidades | No acreditado | Coste y grounding pendientes |
| DRIFT aislado por consulta | No acreditado | Orquestación opcional sobre scheduler/checkpoints existentes |
| Autorización end-to-end de derivados y citas | No acreditado integralmente | No negar governance existente; requiere recorrido completo |

## 9. Debilidades sugeridas, no defectos inventados

Comprobar si un resumen conserva **dependencias y revisión** suficientemente precisas para invalidarlo sin perder evidencia compartida; si el diagnóstico distingue candidatos de contexto enviado; y si cualquier exploración tiene ownership de consulta/tenant. Estas preguntas no prueban bugs actuales. El delta no justifica reconstruir dedup, escenas ni RRF. Antes de implementación se debe releer el worktree cambiante y la tarea vigente.

## 10. Escritorio, interfaz e inspección; ledger de cobertura

**Fuente disponible:** `unified-search-app` es una aplicación Streamlit con búsqueda, informes y citas [G15], no solo una captura alojada. No se verificaron despliegue, todos los motores soportados por esa UI ni escritorio nativo. Studio Vanta ya dispone de memoria/actividad/escenas/skills y diagnóstico retrieval en `develop-desktop` según METHOD; no se inspeccionó ese worktree.

| Superficie | Leído / no examinado / pendiente |
|---|---|
| Indexación | Extracción LLM, merge, clustering, construcción de informes; NLP alternativo, todos los loaders y embeddings no auditados |
| Retrieval | Núcleos global/local/DRIFT, contexto y citas focales; selección dinámica, streaming y proveedores completos pendientes |
| Consistencia | Delta documental/resolución incremental; borrado integral, fallos de escritura Parquet/vector y crash recovery no certificados |
| Seguridad | Fronteras de objetos recibidos por builders; tenants, ACL de servicios Azure y despliegues no examinados |
| Tests/benchmark | Test de merge y microbenchmark de caché leídos; no ejecución ni calidad QA acreditada |
| UI | Entrada y componentes referenciados; implementación completa de resolución visual de citas no auditada |
| Runtime y documentación | Validaciones/gates de docs pendientes manuales; sin ejecución, commits ni push |

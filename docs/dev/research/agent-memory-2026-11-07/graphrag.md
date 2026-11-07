---
title: "Microsoft GraphRAG: comunidades jerárquicas y recuperación global, local y DRIFT"
kind: research
status: proposed
description: "Análisis estático de extracción, comunidades, contexto y actualización de GraphRAG con deltas acotados para VantaDB"
tags: [research, memory, engineering]
---

# Microsoft GraphRAG: comunidades jerárquicas y recuperación global, local y DRIFT

## Índice

- [0. Alcance, arquitectura y contratos observados](#0-alcance-arquitectura-y-contratos-observados)
- [1. Recomendaciones para VantaDB y contexto arquitectónico](#1-recomendaciones-para-vantadb-y-contexto-arquitectónico)
- [2. Partes extraíbles, flujo real y mapeo a Rust](#2-partes-extraíbles-flujo-real-y-mapeo-a-rust)
- [3. Defectos, riesgos e issues transferibles](#3-defectos-riesgos-e-issues-transferibles)
- [4. Prerrequisitos en VantaDB](#4-prerrequisitos-en-vantadb)
- [5. Componentes y prácticas que no deben trasladarse](#5-componentes-y-prácticas-que-no-deben-trasladarse)
- [6. Advertencias metodológicas y evidencia](#6-advertencias-metodológicas-y-evidencia)
- [7. Benchmarks existentes y protocolo de comparación](#7-benchmarks-existentes-y-protocolo-de-comparación)
- [8. Matriz frente a VantaDB](#8-matriz-frente-a-vantadb)
- [9. Debilidades sugeridas, no defectos inventados](#9-debilidades-sugeridas-no-defectos-inventados)
- [10. Escritorio, interfaz e inspección; ledger de cobertura](#10-escritorio-interfaz-e-inspección-ledger-de-cobertura)
- [Cierre: fuentes complementarias y dependencias](#cierre-fuentes-complementarias-y-dependencias)
- [Cierre complementario: prioridades, UI y trazabilidad Vanta](#cierre-complementario-prioridades-ui-y-trazabilidad-vanta)

## 0. Alcance, arquitectura y contratos observados

Ampliación estática del **2026-11-07**, conforme al [contrato completo](EXPANSION-CONTRACT.md),
[METHOD](METHOD.md) y [BENCHMARK-NOTES](BENCHMARK-NOTES.md), con
[OpenViking](../../Backlog-OpenViking-VantaDB.md) como referencia de profundidad, no de algoritmo.
Se conserva el SHA del manifiesto; no se actualizó ningún clon. Los hallazgos anteriores
válidos se mantienen; se corrigen su baseline Vanta histórico y el eje de issues no leído.
No se afirma lectura total, reproducción de incidencias ni superioridad medida.

### Mapa, stack y superficies

| Carpeta / frontera | Responsabilidad y coste contractual |
|---|---|
| `packages/graphrag/graphrag/{api,cli,config}` | Biblioteca y CLI Typer; configuración de index/query/update. El paquete declara versión 3.3.0; no equivale a probar la distribución publicada |
| `index/{workflows,operations,update,run}` | Workflows sobre DataFrames; extracción, agregación, Leiden, informes, embeddings y conciliación incremental |
| `data_model/`, `query/context_builder/` | Entidades, relaciones, unidades de texto, comunidades e informes; proyección a tablas de contexto y citas |
| `query/structured_search/{local_search,global_search,drift_search}` | Tres algoritmos distintos; el coste LLM no está confinado al answerer final |
| `packages/graphrag-{input,chunking,llm,cache,storage,vectors}` | Interfaces separadas de entrada, cómputo y persistencia. Abstracción de storage no demuestra transacción conjunta de tablas y vectores |
| `tests/{unit,integration,smoke,notebook,verbs}`, `unified-search-app/` | Pruebas, ejemplos y UI Streamlit; no confundir runner de notebooks con evaluación QA |

Python 3.11–3.13, Pandas/NumPy/PyArrow, Pydantic, NetworkX, spaCy/NLTK y
`graspologic-native>=1.2,<1.3`; clientes Azure y proveedores LLM externos [G1].
El pin de Leiden preserva golden data: una partición depende de implementación y seed,
no sólo del corpus. MIT raíz/paquete y cabeceras de los módulos leídos; no se hizo
inventario legal completo de wheels, modelos, corpus ni aplicación externa.

### Modelo y tres recorridos reales

**Ingesta:** input reader → documentos → text units → entidades/relaciones con fuentes →
resúmenes de descripciones → clustering → comunidades/informes → embeddings.
Los títulos normalizados son claves de agrupación, no identidades humanas certificadas.
`id` durable y `human_readable_id` de presentación no son intercambiables.
`CommunityReport` referencia comunidad; los miembros y text units permiten resolver
fuentes, pero un informe puede haber sido construido a partir de informes hijos recortados
[G2–G6]. Claims/covariates son otra tabla opcional, no una garantía de verdad.

**Commit/fallo:** `run_pipeline` crea storage, table provider y cache, carga `context.json`;
cada workflow escribe sus salidas y emite `PipelineRunResult`. `_run_pipeline` captura
excepción y emite resultado con `error`, sin deshacer los workflows previos
[G16:117–157][G16]. No equivale a captura de toda cancelación ni a compensación.
Por tanto, callback de fin de workflow ≠ publicación atómica de una generación completa.
La caché evita llamadas repetidas, no prueba frescura, aislamiento o borrado de derivados.
Una cancelación/crash entre tablas requiere reconciliación externa; no se certifica
semántica exactly-once ni se infiere que todas las salidas parciales sean consultables.

**Consulta:** local mapea pregunta/historial a entidades y tablas; global divide informes
en lotes, map LLM paralelo y reduce; DRIFT crea acciones/seguimientos que llaman local
[G8–G10]. Global devuelve tiempos, llamadas y tokens por categoría; el score del map es
producido por modelo, no probabilidad calibrada. Sin puntos positivos y sin conocimiento
general devuelve respuesta prefijada. Permitir conocimiento general modifica grounding.
La selección de candidatos ocurre antes del recorte: `in_context` debe conservarse.

**Update/delete:** `get_delta_docs` compara títulos. Mismo título con contenido cambiado
no entra en `new_inputs`; un borrado aparece en `deleted_inputs`, pero
`load_update_documents` devuelve sólo nuevos y detiene normalmente si no hay ninguno
[G12]. `run_pipeline` copia tablas anteriores a `update_output/<timestamp>/previous`
y construye delta separado [G16:54–90][G16]. La conciliación de entidades conserva IDs previos;
la de comunidades remapea IDs de comunidad/padre, no sus listas de entidades/relaciones
[G17:44–84]. Un rebuild puede retirar datos del índice consultado, pero **no demuestra**
purga de backups/caché. No se localizó una retraction integral acreditada en esa ruta.

### Baseline Vanta revalidado antes de proponer

`git rev-parse HEAD` devolvió **`97c9623c8ef5a70fdb51aff215bfbb3b05b15612`**;
`git status --short` no mostró cambios al inicio. `c561158/dirty` es antecedente,
no baseline actual. Se releen seams concretos [V1–V5]: ontología con
`apply_triplet_with_source`, ingestors `{file}#{chunk_index}`, estado wiki,
`assemble_with_recall` y scheduler. Tener grafo no acredita comunidades; tener
`DerivedIndexState` tampoco acredita un manifest de comunidad: contiene versión de
esquema y conteos de índices auxiliares, no revisiones de fuentes.

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

**Contrato y ejecución propuestos para A (tipos hipotéticos, no API existente):**
entrada = snapshot autorizado del grafo + revisiones fuente + versión del particionador;
salida = descriptor de comunidad y referencia al resumen, o `StaleInput`,
`UnauthorizedSource`, `GenerationFailed`. `parent` debe pertenecer a la misma generación;
miembros no resueltos impiden publicar. Una comunidad es una vista, no un propietario
de las fuentes: borrar la vista nunca borra sus miembros.

1. Inventariar agrupaciones de escenas/ontología antes de crear otro tipo.
2. Proyectar miembros estables y dependencias desde `apply_triplet_with_source` [V1];
   particionar fuera del core, conservando dirección semántica en el grafo canónico.
3. Persistir descriptor versionado en el almacenamiento existente; generar resumen
   por worker/scheduler actual, no por un segundo sistema de jobs [V4].
4. Validar revisiones y publicar referencia sólo tras completar dependencias.
5. Migración aditiva: registros sin descriptor siguen en retrieval baseline;
   nueva versión del algoritmo crea generación nueva, no renumera citas existentes.
   Rollback desactiva selección de comunidades y conserva fuentes/revisiones; GC
   posterior sólo sobre derivados sin lectores/dependientes vigentes.

**Alternativas:** agrupar escenas existentes si satisfacen la pregunta; usar adapter
GraphRAG aislado para experimentación; reimplementar sólo descriptor/invalidación si
aporta valor; rechazar Leiden obligatorio. Persistir informes agrega almacenamiento,
tokens de indexación y regeneración incluso si nunca se consulta global.
**FMEA completa:** cambio de fuente → resumen conserva revisión antigua → respuesta
obsoleta; detectar mismatch al seleccionar/publicar y marcar stale. LCC excluye islas →
cobertura sesgada; registrar miembros excluidos y permitir baseline. Colisión de tenant →
mezcla de informes; validar pertenencia antes de agrupar, no después de resumir.
**Aceptación/observabilidad:** alta/cambio/baja de una fuente invalida sólo dependientes;
dos tenants con nombres iguales no comparten resumen; error LLM conserva fuente.
Medir comunidades por generación/estado, dependientes sin resolver, regeneraciones,
tokens y duración por fase; sin texto de fuentes en métricas. Queda abierta la política
de partición; no se fijan tamaño de comunidad ni pesos nuevos.

**B. Contexto de grafo con candidatos y entrega diferenciados**

- Unidad/símbolos: `LocalSearchMixedContext.build_context`, `mixed_context.py:91–221`; flags `in_context` en `224–375` [G8].
- Seam Vanta: `perform_auto_recall_governed`, `core/hooks/auto_recall.rs:399–408`; `assemble_with_recall`, `context_engine/engine.rs:252–264`.
- Contrato Rust **propuesto**: `GraphContextProjection { candidates, delivered_ids, source_manifest, exclusion_reasons }`; consume presupuesto remanente del assembly existente.
- Dependencias/acciones: adaptar tablas a candidatos de memoria, filtrar por governance antes de expansión y conservar IDs por etapa.
- FMEA: confundir candidato con evidencia enviada o presupuestar cada tabla de forma independiente; usar una sola contabilidad efectiva.
- Aceptación pendiente: diagnóstico separa recuperado/entregado; IDs de citas pertenecen a entregados; historial y grafo comparten el presupuesto actual.
- No objetivos: copiar proporciones upstream como configuración Vanta ni duplicar compactación/context_assemble.

**Contrato operativo B:** entrada = candidatos autorizados con identidad/revisión y
presupuesto remanente real; salida = proyección de lo que **efectivamente** sobrevivió
al renderizado. `delivered_ids` es subconjunto de candidatos y cada ordinal resuelve
una evidencia de esa entrega. Error de permisos no habilita búsqueda sin filtros.
Seam [V2]: `assemble_with_recall:252–325` ya compacta, inyecta MMD y consume remanente;
el delta es una proyección/receipt ligado a esa serialización, no otro ensamblador.

**Pasos/persistencia:** (1) conservar identidad y revisión al convertir hits de grafo;
(2) aplicar governance antes de cada expansión; (3) registrar exclusión/recorte desde
el assembly actual; (4) producir manifest después del render final; (5) sólo si el host
confirma entrega, anexar receipt de sesión. No persistir todo candidato por defecto.
Migrar de forma aditiva y opt-in; clientes sin receipt mantienen respuesta actual.
Rollback deja de emitir el campo opcional; nunca reinterpreta ordinales viejos.
**Alternativas:** diagnóstico transitorio basta para depurar; receipt persistente
requiere contrato de sesión; adapter de UI sin cambios al motor es preferible si ya
existe metadata suficiente. Depende de A sólo para mostrar comunidades, no para hechos.
**FMEA:** recorte posterior al manifest → cita a contenido no enviado → detectar IDs
fuera del render y construir manifest al final; presupuesto por tabla → desborde total →
usar contabilidad compartida; retry de transporte → doble ledger → clave idempotente
por entrega/revisión. **Aceptación:** cero presupuesto, sin hits, varias tablas y
expansión denegada producen diagnósticos distintos sin filtrar evidencia sensible.
Contadores: seleccionados/renderizados/confirmados, exclusiones por motivo y tokens
efectivos. No se certifica entailment ni se registra la pregunta completa por defecto.

**C. Invalidador de derivados por revisión documental**

- Unidad/símbolos: contraste de `get_delta_docs`, `incremental_index.py:29–58`, y `_group_and_resolve_entities`, `entities.py:14–74` [G7, G12].
- Seam Vanta: `vanta-memory/src/ingest/worker.rs:167–250` y `ingest/merge.rs`; dedup/supersession/linaje existentes.
- Contrato Rust **propuesto**: `DerivedInvalidation::plan(scope, old_revision, new_revision, dependents) -> Result<InvalidationPlan, LineageError>`; título no es clave de revisión.
- Dependencias/acciones: reutilizar hash/versión y sincronización actuales; enumerar dependencias, invalidar resúmenes y embeddings afectados sin borrar fuentes compartidas.
- FMEA: mismo título oculta cambio o agrupación solo nominal mezcla tipos; evidenciar identidad y revisión independientes de presentación.
- Aceptación pendiente: actualización de contenido con nombre constante afecta derivados; borrar una fuente no elimina pruebas supervivientes; reintentos idempotentes.
- No objetivos: reimplementar ingestors wiki, dedup o una limpieza completa del corpus por cada edición.

**Contrato C:** `plan` recibe revisión esperada y nueva junto a referencias de
dependencia verificables; devuelve operaciones idempotentes y revisiones a invalidar,
no aplica borrado directamente. Un título es presentación; la identidad incluye scope
y fuente estable. Errores `UnknownLineage`, `RevisionConflict`, `ReadFailed` no equivalen
a ausencia. Una contribución compartida sólo desaparece cuando no quedan fuentes
autorizadas vigentes; invalidar un resumen no elimina hechos supervivientes.

**Seam y pasos:** `src/wiki/ingestors.rs:69–89` ya determina claves, y
`ingest/merge.rs::commit` ya consolida por página [V3]. (1) Enumerar claves y linaje
publicados; (2) diferenciar alta/update/baja por revisión; (3) persistir plan/progreso
en almacenamiento/checkpoints existentes; (4) invalidar referencias antes de exponer
derivados nuevos; (5) regenerar por scheduler existente; (6) extender auditoría de
índices, no crear otro verificador. Migración: registros legacy con linaje insuficiente
se marcan desconocidos y requieren rebuild acotado aprobado, nunca se asumen vacíos.
Rollback deshabilita consumo del derivado stale y conserva originales y plan auditable.
Depende del descriptor A para comunidades; no bloquea el diagnóstico read-only B.

**Alternativas/coste:** rebuild del subcorpus simplifica garantías a coste de tokens/I/O;
tracking fino reduce trabajo pero añade índices inversos y recuperación. La ruta
upstream de concatenación no se adopta como contrato de borrado.
**FMEA:** baja repetida sobre revisión vieja → elimina sucesora → detectar expected
revision y rechazar; lectura fallida → falso missing → detener plan destructivo;
crash tras invalidar → disponibilidad parcial → reanudar idempotentemente conservando
baseline. **Aceptación:** mismo título/contenido nuevo, rename, fuente compartida,
fallo de lectura y retry de baja; sin referencias colgantes en snapshot publicado.
Observar stale/missing/unknown por generación, progreso y descartes por fence, sin payload.

**D. Exploración iterativa con estado acotado por consulta**

- Unidad/símbolos: bucle de acciones y serialización de `DRIFTSearch.search`, `drift_search/search.py:188–305` [G10].
- Seam Vanta: worker/scheduler/checkpoints existentes y `perform_auto_recall_governed`; integración sería de orquestación, no del motor.
- Contrato Rust **propuesto**: `EvidenceExploration::step(query_scope, frontier, remaining_resources) -> Result<ExplorationStep, ExplorationError>`, con fuentes y motivo de terminación.
- Dependencias/acciones: sesión de exploración propia, cancelación y telemetría reutilizadas; llamadas LLM opcionales; entregar evidencias a assembly actual.
- FMEA: seguimientos redundantes inflan costes o estado de una pregunta contamina otra; ownership por consulta/tenant y terminación explícita.
- Aceptación pendiente: consulta nueva no hereda acciones ajenas, cancelación mantiene trazas y no entrega parciales como completos; baseline sin exploración permanece disponible.
- No objetivos: portar DRIFT completo, garantizar cobertura por “razonamiento” ni definir budgets/umbrales nuevos en esta investigación.

**Contrato D:** entrada = scope autorizado, frontier de acciones de **una consulta**,
recursos remanentes del host y token de cancelación; salida = evidencias, acciones nuevas,
consumo y `TerminationReason` (`Exhausted`, `Cancelled`, `Failed`, `Completed`).
Es estado de ejecución, no memoria autobiográfica ni una respuesta garantizada.
La evidencia recuperada no autoriza ejecutar instrucciones. Identidad de operación
separa tenant/sesión/consulta; una reanudación valida generación/fuentes.

**Implementación futura:** extender la orquestación que ya llama recall y assembly;
registrar padre/estado de acción y llamadas antes de reintentar; pasar cada lote al
presupuesto compartido; finalizar con recibo explícito de parcialidad. Persistencia
sólo para consultas reanudables; no reutilizar el checkpoint global de persona como
almacén de acciones. Migración sin cambio de memorias; rollback selecciona ruta sin
exploración y cancela tareas pendientes, conserva trazas mínimas de operación.
Depende de B y cancelación del worker, no de resolver todos los problemas históricos.
**Alternativas:** una expansión determinista de vecinos es más barata y explicable;
global sin seguimientos para síntesis transversal; DRIFT sólo si evaluación lo justifica.
**FMEA:** followups equivalentes → gasto repetido → detectar repetición de acciones y
consumo, detener según límites ya acordados por host; estado compartido → contaminación
de preguntas → ownership exclusivo; timeout del cliente → generación continúa →
propagar cancelación y reflejar consumo residual, no declarar coste cero.
**Aceptación:** consultas concurrentes aisladas, cancelación en cada fase, fallo parcial
visible y fallback autorizado. Telemetría por fase/acción, cola, tokens y motivo final,
sin prompts o texto sensible. No añade scheduler, ANN, rúbrica ni umbral.

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

### Incidencias públicas contrastadas el 2026-11-07

La búsqueda keyless no devolvió resultados útiles; la comparación metasearch
tampoco localizó incidencias pertinentes. Se consultaron búsqueda API GitHub y
páginas completas oficiales. La API de detalle acabó limitada por 403; las
páginas y las respuestas previas sí permitieron verificar lo siguiente.
Estado es el observado en la consulta, no una predicción de resolución.

| Incidencia / fechas verificadas | Mecanismo y contraste con el SHA | Transferencia y validación futura |
|---|---|---|
| [#2590](https://github.com/microsoft/graphrag/issues/2590), abierta; creada/actualizada 2026-10-05 | Reporta 3.2.0: `deleted_inputs` se calcula pero no se consume; removal-only termina normalmente. El SHA fijado conserva ese camino [G12]. El reporte también señala copias `previous` y caché; no se reprodujeron sus cifras ni se certifica erasure | C distingue baja de alta y enumera originales, informes, vectores, caché y backups. Futuro caso: retirar fuente única y comprobar cada superficie, no sólo respuesta QA |
| [#2540](https://github.com/microsoft/graphrag/issues/2540), abierta; creada 2026-09-03, actualizada 2026-10-02 | Merge de entidad conserva ID anterior y produce remapeo; comunidades delta retienen referencias a IDs descartados. `index/update/communities.py:14–84` remapea comunidad/padre, no miembros. La firma no recibe mapas de entidad/relación [G17] | A/C deben verificar cierre referencial después del merge. Futuro caso: entidad y relación comunes a dos documentos; todos los miembros publicados deben resolver al ID superviviente |
| [#2614](https://github.com/microsoft/graphrag/issues/2614), abierta; creada/actualizada 2026-10-27 | JSON válido no objeto llega a `data.get("result")`; el SHA conserva la suposición en `JsonCache.get:33–46` [G18]. El autor refiere main post-monorepo, no todas las versiones publicadas | F valida envelope y diferencia corrupción de miss. No es vulnerabilidad de Vanta ni prueba de que el almacenamiento deba aceptar entrada no confiable |

Comentarios de #2540 afirman reproducciones independientes; son evidencia
publicada por terceros, no ejecución propia. Que el issue continúe abierto no
acredita todos sus efectos; el contraste de código limita la conclusión a las
ramas descritas. No se importan CVE ni afirmaciones MSRC del reportero como
verificación de seguridad independiente.

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

La revisión sigue el SHA del clon superficial, no una versión estable publicada.
“Código” acredita un flujo leído; “hipótesis” exige reproducción futura.
La ampliación incorpora las consultas públicas de §3; no hubo mediciones ni
auditoría integral de seguridad.

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
| G12 | [incremental_index.py:29–78](https://github.com/microsoft/graphrag/blob/34b2b67ae4601ad0b47bcd179fbe3f2a23aa0857/packages/graphrag/graphrag/index/update/incremental_index.py#L29-L78), [load_update_documents.py:48–62](https://github.com/microsoft/graphrag/blob/34b2b67ae4601ad0b47bcd179fbe3f2a23aa0857/packages/graphrag/graphrag/index/workflows/load_update_documents.py#L48-L62): delta y consumo de entradas |
| G13 | [test_update_relationships.py:1–85](https://github.com/microsoft/graphrag/blob/34b2b67ae4601ad0b47bcd179fbe3f2a23aa0857/tests/unit/indexing/update/test_update_relationships.py#L1-L85): fixtures y merge previo/delta |
| G14 | [test_cache_benchmark.py:22–98](https://github.com/microsoft/graphrag/blob/34b2b67ae4601ad0b47bcd179fbe3f2a23aa0857/tests/unit/cache/test_cache_benchmark.py#L22-L98): microbenchmark existente |
| G15 | [unified-search-app/app/home_page.py:1–65](https://github.com/microsoft/graphrag/blob/34b2b67ae4601ad0b47bcd179fbe3f2a23aa0857/unified-search-app/app/home_page.py#L1-L65): UI Streamlit con paneles de informes y citas |
| G16 | [run_pipeline.py:54–157](https://github.com/microsoft/graphrag/blob/34b2b67ae4601ad0b47bcd179fbe3f2a23aa0857/packages/graphrag/graphrag/index/run/run_pipeline.py#L54-L157): copia previous, delta y ejecución con efectos parciales; `except Exception` no equivale a rollback ni a captura universal de cancelación |

**Baseline de la ampliación:** `97c9623c8ef5a70fdb51aff215bfbb3b05b15612`,
conforme a [VANTA-REFERENCE](VANTA-REFERENCE.md). La primera pasada
`c561158.../dirty` es antecedente, no evidencia vigente. Se contrastaron mediante
`git show` del snapshot `ingestors.rs::ingest:69–89`, `scan_ingestable_sources:622`,
`ingest/merge.rs::{CandidatePage:18,merge_page:135,commit:239}`,
`WikiStore::{begin_processing:162,complete:185,fail:199}`,
`engine.rs::assemble_with_recall:252`,
`auto_recall.rs::{InjectionPolicy:136,RecallConfig:302,perform_auto_recall_governed:402}`
y `l1_writer.rs::{persist_planned:289,apply_dedup_batch:389}`.
No se infiere atomicidad multíndice del nombre de un método. Los enlaces V1–V5
del cierre quedan fijados al snapshot del contrato; estos símbolos fijan el delta de adopción.
CodeGraph se consultó para localizar seams actuales, sin reindexar ni cambiar metadata.

## 7. Benchmarks existentes y protocolo de comparación

- Runner realmente localizado: `tests/unit/cache/test_cache_benchmark.py::test_bulk_read_cache_benchmark` [G14], opt-in mediante opción pytest. Prellena claves/valores representativos, compara `get_many` en Memory/JSON/SQLite y comprueba conjunto de claves.
- Métricas: mediana de tiempo de lecturas calientes, entradas/segundo y relación frente a SQLite. Dependencias: pytest asíncrono, caches y almacenamiento de fichero; usa `tmp_path`. **Sin juez LLM ni medida de retrieval, QA o memoria.** No extrapolar un benchmark de caché a “GraphRAG más rápido”.
- Defaults leídos en [G14:18–57]: **1 000 entradas, siete rondas**, un `get_many`
  previo verifica las claves y calienta el camino. `gc.collect` ocurre antes
  del cronómetro; `_populate` y creación del almacén quedan fuera. Throughput =
  1 000 / mediana de segundos; el ratio es tiempo backend / tiempo SQLite.
  Una clave ausente dispara assert: no se elimina silenciosamente del
  denominador ni se mide un caso incompleto como más rápido. No hay dataset
  semántico, split, modelo, tokenizer, seed ni juez; no son necesarios para
  esta pista de caché. Versiones de OS/disco/SQLite y carga competidora sí importan.
- En las búsquedas acotadas de rutas `eval`, `benchmark` y notebooks no se acreditó un runner completo de calidad QA con dataset/juez congelados dentro del clon. Hay ejemplos DRIFT y datos de demostración, no evidencia de reproducción de resultados publicados. No se inventan instrucciones de ejecución ni un juez nuevo.
- Adaptación: aplicar pistas motor/retrieval/respuesta de [METHOD](METHOD.md#protocolo-de-comparacion-de-benchmarks) por separado. Para global/DRIFT registrar extracción, informes, embeddings, map, seguimiento, reducción y juez con presupuesto/generación equivalentes al baseline. El número de llamadas importa tanto como la latencia retrieval.
- **Adaptación específica de G14 (propuesta, no harness nuevo):** el seam público
  existente es [`Embedded::get:1611–1625`](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/sdk/api/memory.rs#L1611-L1625).
  Un adapter `Cache.get_many` mapearía key a `(namespace desechable, key)` y
  payload a envelope JSON `result`/metadata de G14, devolviendo mapa sólo tras
  verificar claves y valores completos. Prellenado mediante `put/put_batch`,
  encode y creación del store fuera del cronómetro; dentro quedan gets,
  materialización y decode equivalentes. Es lectura KV vía SDK, **no** medida
  pura de cache ni ANN/recall. Si sólo puede hacer gets individuales, registrar
  esa diferencia frente al bulk upstream y no afirmar paridad de batching.
  El motor ya tiene [`CacheLayer::lookup_volatile:17–73`](https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/storage/engine/cache.rs#L17-L73):
  almacena `UnifiedNode` por u128 y tombstones, no JSON por string. Se rechaza
  exponerlo o reemplazarlo para acomodar el benchmark. Conservar 1 000 entradas,
  warm read, siete rondas, GC fuera del tiempo y mediana del runner existente;
  una clave/valor omitido invalida la ronda, no mejora su throughput.
  Aislar namespace/backend/caches y retirar sólo datos propios; no borrar una
  base compartida. La preparación modifica almacenamiento y el warmup altera
  cache. No hay retry de calidad ni modelos necesarios para esta adaptación.
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

## Cierre: fuentes complementarias y dependencias

### Mecanismos adicionales independientes de DRIFT

**E. Síntesis transversal map/reduce con evidencia por lote**

- **Caso de agente:** resumir cambios o decisiones dispersos entre sesiones sin
  convertir cada pregunta en exploración iterativa. Fuente SHA
  `34b2b67ae4601ad0b47bcd179fbe3f2a23aa0857`,
  `query/structured_search/global_search/search.py::search:142–220`,
  `_reduce_response:306–385` [G9]. Unidad: resultados map con fuente/lote y
  reducción explícita; no copiar los prompts ni interpretar score LLM como verdad.
- **Delta/seam:** vistas A y proyección B alimentan el worker existente y
  `assemble_with_recall` del snapshot Vanta. Motor híbrido y compactación ya
  existen; ésta sería una estrategia opcional del host para síntesis global.
- **Contrato Rust propuesto:** `GlobalSynthesisInput { scope, query_id,
  source_generation, batches } -> SynthesisOutcome { evidence_refs,
  partial_batches, usage_by_phase, answer_ref }`. Errores diferenciados:
  `InvalidMapPayload`, `StaleGeneration`, `Cancelled`, `ProviderFailure`.
  Cada punto conserva fuentes; un lote fallido no equivale a relevancia cero.
- **Pasos y commit:** seleccionar snapshot autorizado; lanzar maps con recursos
  del host; validar shape/referencias; reducir sólo material entregable;
  registrar entrega mediante B. Cachear maps por revisión/versión de modelo,
  nunca convertirlos automáticamente en hechos durables. Publicación de una
  síntesis sólo después de validar la generación; cambio concurrente marca
  resultado parcial/stale. Rollback desactiva estrategia, no borra memorias.
- **Dependencias/alternativas:** requiere B, A sólo si se usan informes;
  también puede operar sobre evidencias existentes. Alternativas: una respuesta
  única desde baseline, agregación determinista de metadatos o DRIFT cuando
  realmente haga falta buscar. Más lotes añaden tokens, persistencia de caché
  y coste de invalidación; no hay ahorro probado por paralelizar.
- **FMEA:** un map emite JSON malformado → se pierde evidencia → rechazo
  tipado; mezcla de generaciones → síntesis contradictoria → fence previo
  a publicación; cancelación cliente → maps continúan cobrando → ownership
  por consulta y reporte de uso residual. No persistir preguntas en métricas.
- **Aceptación manual futura:** lote vacío, fallido y score cero distinguibles;
  ningún punto cita una fuente denegada; repetición con otra revisión no
  reutiliza map viejo; cancelación no deja una respuesta marcada completa.
  No-goals: nuevo scheduler, nuevo budget, juez o scoring de evaluación.

**F. Envelope versionado de caché derivada, no memoria primaria**

- **Caso:** evitar que un cache corrupto bloquee el recall de un agente.
  Unidad negativa [G18] `JsonCache.get:33–46`; `set:50–55` serializa objeto,
  pero el lector no valida el tipo. Fuente = SHA GraphRAG anterior.
- **Delta/seam:** extender el adaptador de generación que use caché, si existe
  en la ruta elegida, y `core/memory_generation_log/`; no cambiar WAL ni
  prometer reparación de registros primarios. Tipo Rust **propuesto**:
  `DerivedCacheEnvelope { schema_version, scope, source_revisions,
  model_prompt_fingerprint, result }`, lectura devuelve `Hit`, `Miss`,
  `InvalidEnvelope`, `Stale` o error de I/O. Deserialización nunca autoriza acceso.
- **Pasos:** inventariar cache concreto; serializar envelope con claves de
  derivación; validar antes de uso; aislar/quarantinar entrada inválida;
  regenerar sólo con permiso del host. Persistir reemplazo con revisión
  esperada; lector concurrente ve viejo o nuevo envelope válido, no fragmento.
  Migración legacy = desconocido, no hit certificado. Rollback omite cache.
- **Deps/alternativas:** comparte identidad con C; no requiere A/E. Cache
  en memoria evita migración a cambio de recomputación; caché durable exige
  retención/erasure y control de secretos. No introducir almacenamiento nuevo
  si el backend actual ya expresa el contrato.
- **FMEA:** JSON array aceptado → excepción repetida → schema tipado y diagnóstico;
  I/O fallido tratado como miss → oculta degradación → estado explícito;
  regeneraciones simultáneas → gasto duplicado → single-owner por clave y
  cancelación, sin prometer exactly-once de un proveedor externo.
- **Aceptación manual futura:** array, número, versión desconocida y revisión
  antigua no cuentan como hits; cache inaccesible conserva error; primario
  permanece intacto. Observabilidad: hits/misses/stale/corrupt, fase y uso,
  nunca prompts. No-goals: cifrado global, nuevo cache engine ni copiar plantilla.

### Memoria durable, costes y límites finales

Informes A son **materializaciones derivadas** persistibles; DRIFT D es estado
de ejecución; E produce respuesta/síntesis; F conserva cómputo reutilizable.
Ninguno, por persistir JSON/Parquet o reducir tokens de consulta, demuestra
mejora de memoria longitudinal, retención de hechos o erasure integral.
El coste debe separar extracción/gleanings, resumen de descripciones, informes,
embeddings, lookup/contexto, map, followups, reduce y evaluación externa.
La caché reduce recomputación cuando acierta; también retiene contenido que
puede necesitar borrado. Durabilidad primaria no equivale a frescura de vistas.

**Fuentes añadidas leídas:** [G17 comunidades](https://github.com/microsoft/graphrag/blob/34b2b67ae4601ad0b47bcd179fbe3f2a23aa0857/packages/graphrag/graphrag/index/update/communities.py#L14-L149);
[G18 caché JSON](https://github.com/microsoft/graphrag/blob/34b2b67ae4601ad0b47bcd179fbe3f2a23aa0857/packages/graphrag-cache/graphrag_cache/json_cache.py#L33-L55);
issues #2590/#2540/#2614. No se inspeccionaron soluciones de forks ni se
ejecutaron los repros publicados. El resto del inventario G1–G16 conserva su
alcance focal, no pasa a ser auditoría exhaustiva por esta ampliación.

**Dependencias:** C sostiene invalidación de A/E/F; B puede aprobarse sola
como diagnóstico; D y E son alternativas de orquestación, no pasos obligatorios.
Quedan pendientes revisión adversarial independiente, validación de todas las
rutas de autorización, pruebas manuales de crash/reintentos y comparaciones
bajo METHOD; no hay resultados ni adopción aprobada.

**Plan de división si se activan specs:** identidad/linaje y remapeo A/C;
proyección y caché B/F; estrategias opcionales D/E; evaluación y UI como
anexos de cada spec. Se mantiene aquí el dossier autónomo y su TOC por superar
300 líneas; no se crean ahora archivos ni entradas de backlog.

## Cierre complementario: prioridades, UI y trazabilidad Vanta

Esta pasada **conserva** las fichas A–F y su investigación previa; no renueva
metadata ni convierte la autoría en revisión independiente. Los tres issues
#2590/#2540/#2614 se volvieron a leer en sus páginas oficiales, incluido el
alcance de los repros y comentarios. En #2614 un objeto sin `result` produce
`None` por `.get`, no necesariamente `AttributeError`: la afirmación más amplia
de un comentario no sustituye al mecanismo del código. Los payloads no objeto
sí llegan a la llamada inválida descrita en G18. No se ejecutó ningún repro.

### Decisión por mecanismo y capacidad existente

| Orden condicionado | Candidato / razón | Baseline y fuente de contraste | Puerta de decisión |
|---|---|---|---|
| Primero | B, evidencia seleccionada frente a entregada | `IntegratedContext`, `assemble_with_recall` [V2]; G8/G11 | Ampliar diagnóstico ya existente, no otro assembler |
| Primero si hay derivados compartidos | C, invalidación/remapeo y cierre referencial | `CandidatePage`, `commit` [V3]; `Ingestor::ingest` [V5]; G7/G12/G17 | Inventariar lo que ya publica cada ruta; sólo ampliar el contrato que falte |
| Después de C | A, jerarquía materializada | `apply_triplet_with_source` [V1]; G4–G6 | Evidencia y snapshot autorizados antes de resumir; usar grafo actual |
| Independiente, si hay caché derivada durable | F, envelope tipado | G18 y #2614 | No atribuir el defecto de JSON de GraphRAG a caches Vanta no leídas |
| Posterior y opt-in | E, map/reduce global | G9 y worker [V4] | Beneficio semántico por demostrar; coste de map/reduce separado del lookup |
| Posterior, alternativa a E | D, exploración iterativa | G10 y worker [V4] | Ownership por consulta, cancelación y recursos del host; no otro scheduler |

La matriz de §8 usa «existe» para primitives, «parcial» para cobertura entre
rutas y «no acreditado» para mecanismos concretos. No usa una muestra de
ontología para negar comunidades en todo Vanta. Para UI, Studio **existe** en
el worktree descrito por [VANTA-REFERENCE](VANTA-REFERENCE.md); el inspector de
comunidades A/B sigue **no acreditado**. Un servicio GraphRAG obligatorio en
el motor es **no aplicable** al diseño embebido y se rechaza.

### UI: flujo efectivamente leído, no demostración visual

`unified-search-app/app/home_page.py::main:20–237` inicializa estado Streamlit,
lee la pregunta y habilita columnas Basic/Local/Global/DRIFT según sidebar.
En `205–233` evita repetir `question_in_progress`, llama `run_all_searches`
y proyecta citas desde `st.session_state.response_lengths`; captura la
excepción para mostrarla. «Community Explorer» es otra pestaña (`79–84`).
Esto acredita flujo web y estado de sesión de presentación, **no** memoria
durable del agente, cancelación remota ni aislamiento del índice por usuario.
Una comparación de cuatro columnas puede activar varios generadores: no
medirla como una sola llamada de retrieval.

Fuente ampliada: [home_page.py:79–237](https://github.com/microsoft/graphrag/blob/34b2b67ae4601ad0b47bcd179fbe3f2a23aa0857/unified-search-app/app/home_page.py#L79-L237).
El source no certifica que todas las dependencias del frontend funcionen con
el paquete actual. No se arrancó Streamlit ni se auditaron todos sus adaptadores.
MIT de las cabeceras leídas no resuelve modelos, corpus o despliegue Azure.

### Cobertura de cierre y lagunas conservadas

Los diez ejes tienen contenido específico en §1–10; las propuestas no son
implementaciones ni aceptaciones. Quedan sin acreditar runner QA congelado
dentro de la búsqueda acotada, equivalencia con memoria longitudinal,
erasure en todos los almacenes/backups, coherencia tras crash y autorización
completa de derivados. La presencia del microbenchmark de caché sólo cubre
su propia unidad y denominador. No hay resultados de latencia/calidad propios.
G16 fija la fuente del flujo incremental: `except Exception` informa fallo
de workflow tras posibles escrituras; cancelación debe inspeccionarse aparte,
no se convierte en garantía de compensación. La adaptación G14 es lookup SDK
con envelope y completitud, distinta de los ensayos semánticos global/DRIFT.
Una extracción de la idea a Rust no elimina obligaciones MIT o de componentes.
Las futuras specs se separarían según el plan de división ya consignado,
manteniendo este dossier y sus evidencias; no se crean esos archivos ahora.

[V1]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/core/ontology/mod.rs#L147-L212
[V2]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/context_engine/engine.rs#L228-L325
[V3]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/ingest/merge.rs#L199-L289
[V4]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/vanta-memory/src/services/pipeline_worker.rs#L795-L849
[V5]: https://github.com/ness-e/VantaDB/blob/97c9623c8ef5a70fdb51aff215bfbb3b05b15612/src/wiki/ingestors.rs#L45-L89
[G16]: https://github.com/microsoft/graphrag/blob/34b2b67ae4601ad0b47bcd179fbe3f2a23aa0857/packages/graphrag/graphrag/index/run/run_pipeline.py#L54-L157

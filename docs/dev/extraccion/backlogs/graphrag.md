---
title: "Backlog de extracción — GraphRAG"
kind: research
status: active
description: "Seis candidatas del informe GraphRAG: snapshot de comunidad, proyección de contexto, invalidación por revisión, exploración, síntesis map/reduce y envelope de caché"
tags: [extraccion, backlog, graphrag]
---

# Backlog de extracción — GraphRAG

> **Origen:** informe `docs/dev/extraccion/agent-memory-2026-11-07/graphrag.md`
> **Estado:** candidatas documentales — ninguna está implementada ni aprobada.
> Requiere revalidación contra HEAD antes de especificar.

| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado | Descripción | Relaciones | Dependencias |
|---|---|---|---|---|---|---|---|---|---|
| X-GRR-01 | 🟠 | Snapshot de comunidad con dependencia explícita a fuentes | `vanta-memory/src/core/ontology/mod.rs:19-51,147-212` | 🟠 | 🟠 P1 | ⬜ Candidata | Descriptor versionado de comunidad (scope, revisión, miembros, padre, revisiones de fuente, referencia a resumen) generado por worker existente · Responder preguntas transversales sin enviar todo el corpus exige vistas materializadas que invaliden sus derivados ante cambios · Acción mínima: proyectar miembros/dependencias desde el grafo actual, particionar fuera del core y publicar solo con revisiones válidas · **Trazabilidad:** `informe §2 ficha A · evidencia graphrag/index/operations/cluster_graph.py:20-99, graphrag/data_model/community_report.py:12-66` | X-GRR-03 sostiene su invalidación; X-GRR-05 opcionalmente la consume; converge con C5/C14 de `CONSOLIDATED.md` | X-GRR-03; revisiones de fuente accesibles; scheduler/worker existente (informe §4) |
| X-GRR-02 | 🟡 | Proyección de contexto: candidatos recuperados vs evidencia entregada | `vanta-memory/src/core/hooks/auto_recall.rs:399-408`, `vanta-memory/src/context_engine/engine.rs:252-264` | 🟡 | 🔴 P0 | ⬜ Candidata | Manifest de entrega: `delivered_ids` ⊆ candidatos, razones de exclusión y presupuesto remanente único del assembly actual · Un diagnóstico debe separar "recuperado" de "efectivamente enviado al modelo"; citas solo sobre entregados · Acción mínima: registrar exclusión/recorte desde el assembly existente y emitir manifest tras el render final · **Trazabilidad:** `informe §2 ficha B · evidencia graphrag/query/structured_search/local_search/mixed_context.py:91-221 (flags in_context 224-375)` | X-GRR-04 y X-GRR-05 la requieren; converge con C7/C8 de `CONSOLIDATED.md` | Presupuesto compartido del assembly existente; governance aplicada antes de expansión |
| X-GRR-03 | 🟠 | Invalidación de derivados por revisión documental, no por título | `vanta-memory/src/ingest/worker.rs:167-250`, `vanta-memory/src/ingest/merge.rs::commit`, `src/wiki/ingestors.rs:69-89` | 🟠 | 🔴 P0 | ⬜ Candidata | Plan de invalidación idempotente que diferencia alta/update/baja por revisión con cierre referencial de dependientes · El delta por título deja cambios de contenido invisibles y borrados sin propagar · Acción mínima: enumerar linaje publicado, persistir plan/progreso en checkpoints existentes e invalidar referencias antes de exponer derivados nuevos · **Trazabilidad:** `informe §2 ficha C · evidencia graphrag/index/update/incremental_index.py:29-78, graphrag/index/update/entities.py:14-74, graphrag/index/update/communities.py:14-84 · issues #2590/#2540` | X-GRR-01, X-GRR-05 y X-GRR-06 dependen de su identidad/revisión; converge con C5/C6/C14 de `CONSOLIDATED.md` | Linaje, hash/versión y auditoría de índices existentes; legado con linaje insuficiente queda marcado desconocido |
| X-GRR-04 | 🟡 | Exploración iterativa con estado acotado por consulta | `vanta-memory/src/services/pipeline_worker.rs`, scheduler, `auto_recall.rs::perform_auto_recall_governed` | 🟠 | 🔵 P3 | ⬜ Candidata | Paso de exploración con frontier propio por consulta, motivo de terminación explícito y consumo visible del host · Dos consultas no deben compartir acciones ni una cancelación dejar parciales marcados como completos · Acción mínima: orquestación sobre worker/cancelación existentes que entrega evidencias al assembly actual; no otro scheduler · **Trazabilidad:** `informe §2 ficha D · evidencia graphrag/query/structured_search/drift_search/search.py:188-305` | X-GRR-02 requerida; X-GRR-05 es alternativa; converge con C8 de `CONSOLIDATED.md` | X-GRR-02; cancelación del worker; ownership por tenant/sesión/consulta |
| X-GRR-05 | 🟡 | Síntesis transversal por lotes con fuentes y uso por fase | `vanta-memory/src/services/pipeline_worker.rs`, `vanta-memory/src/context_engine/engine.rs::assemble_with_recall` | 🟠 | 🔵 P3 | ⬜ Candidata | Estrategia opt-in de síntesis por lotes sobre snapshot autorizado: cada punto conserva fuentes, un lote fallido no equivale a relevancia cero y el uso se desglosa por fase · Preguntas transversales no deben requerir exploración iterativa ni mezclar generaciones · Acción mínima: maps con recursos del host, validación de shape/referencias, reducción solo de material entregable y registro por X-GRR-02 · **Trazabilidad:** `informe cierre complementario ficha E · evidencia graphrag/query/structured_search/global_search/search.py:142-220, _reduce_response:306-385` | X-GRR-02 requerida; X-GRR-01 solo si usa informes; X-GRR-04 alternativa; converge con C8 de `CONSOLIDATED.md` | X-GRR-02; fence de generación antes de publicar; sin scheduler/budget nuevo |
| X-GRR-06 | 🟡 | Envelope versionado de caché derivada | `vanta-memory/src/core/memory_generation_log/`, adaptador de generación con caché (inventariar) | 🟡 | 🟡 P2 | ⬜ Candidata | Envelope tipado {schema_version, scope, source_revisions, model_prompt_fingerprint, result} con lectura Hit/Miss/InvalidEnvelope/Stale/error · Un cache corrupto no debe bloquear recall ni un payload no objeto contar como hit · Acción mínima: validar envelope antes de uso, cuarentena de inválidos y regeneración solo con permiso del host · **Trazabilidad:** `informe cierre complementario ficha F · evidencia graphrag-cache/graphrag_cache/json_cache.py:33-55 · issue #2614` | X-GRR-03 comparte identidad/revisión; converge con C14 de `CONSOLIDATED.md` | Inventario de la caché concreta en la ruta elegida; sin almacenamiento nuevo si el backend actual expresa el contrato |

## Fichas completas

> Expansión de cada candidata a partir del informe `docs/dev/extraccion/agent-memory-2026-11-07/graphrag.md` (§2 fichas A–D y «Cierre: fuentes complementarias» fichas E–F). Todos los contratos son **hipotéticos** — propuestas de comportamiento para VantaDB, no APIs existentes ni puertos de código upstream. Las referencias `archivo:líneas` citan el clon `C:\Users\Eros\graphrag` en SHA `34b2b67ae4601ad0b47bcd179fbe3f2a23aa0857` (prefijo `packages/graphrag/` abreviado); las rutas VantaDB citan el baseline del informe `97c9623c8ef5a70fdb51aff215bfbb3b05b15612` y requieren revalidación contra HEAD.

### X-GRR-01 — Snapshot de comunidad con dependencia explícita a fuentes

- **Problema:** responder preguntas transversales sin enviar todo el corpus exige vistas materializadas con dependencia explícita a fuentes; extraer el registro jerárquico y su vinculación a fuentes, no el pipeline completo. Una comunidad es una vista, no propietaria de las fuentes: borrar la vista nunca borra sus miembros.
- **Contrato hipotético (tipos propuestos):** `CommunitySnapshot { scope, revision, members, parent, source_revisions, summary_ref }`; proveedor opcional `CommunitySummarizer::summarize(snapshot) -> Result<DerivedEvidence, SummaryError>`. Entrada = snapshot autorizado del grafo + revisiones de fuente + versión del particionador; salida = descriptor de comunidad y referencia al resumen, o `StaleInput`/`UnauthorizedSource`/`GenerationFailed`. `parent` debe pertenecer a la misma generación; miembros no resueltos impiden publicar.
- **Fuente de evidencia (clone):** `graphrag/index/operations/cluster_graph.py:20-99` (Leiden jerárquico, `(nivel, comunidad, padre, miembros)`) [G4]; `graphrag/data_model/community_report.py:12-66` (modelo de informe) [G6]; sustitución/recorte de contexto en `graphrag/index/workflows/create_community_reports.py:41-140` y `graph_context/context_builder.py:196-263` [G5].
- **Seam VantaDB:** `vanta-memory/src/core/ontology/mod.rs:19-51,147-212` (`apply_triplet_with_source`, hechos/grafo/linaje ya almacenados; `MemoryRecord` admite metadatos) [V1]; worker/scheduler existente para la generación [V4].
- **Acciones:**
  1. Inventariar agrupaciones de escenas/ontología existentes antes de crear otro tipo.
  2. Proyectar miembros estables y dependencias desde `apply_triplet_with_source`; particionar fuera del core, conservando dirección semántica en el grafo canónico.
  3. Persistir descriptor versionado en el almacenamiento existente; generar el resumen por worker/scheduler actual, no por un segundo sistema de jobs.
  4. Validar revisiones y publicar la referencia solo tras completar dependencias.
  5. Migración aditiva: registros sin descriptor siguen en retrieval baseline; una nueva versión del algoritmo crea generación nueva, no renumera citas existentes. Rollback desactiva selección de comunidades y conserva fuentes/revisiones; GC posterior solo sobre derivados sin lectores/dependientes vigentes.
- **Dependencias:** X-GRR-03 sostiene su invalidación (informe: «C sostiene invalidación de A/E/F»); revisiones de fuente accesibles; scheduler/worker existente. X-GRR-05 la consume solo si usa informes; converge con C5/C14 de `CONSOLIDATED.md`.
- **FMEA:** cambio de fuente → resumen conserva revisión antigua → respuesta obsoleta (detectar mismatch al seleccionar/publicar y marcar `stale`); exclusión del componente conexo mayor (LCC) → cobertura sesgada (registrar miembros excluidos y permitir baseline); colisión de tenant → mezcla de informes (validar pertenencia antes de agrupar, no después de resumir); resumen obsoleto tras borrado.
- **Criterios de aceptación propuestos:** cada resumen resuelve miembros/fuentes vigentes autorizadas; alta/cambio/baja de una fuente invalida solo dependientes; dos tenants con nombres iguales no comparten resumen; error del generador conserva la fuente; falta de resumen permite el baseline actual. Observabilidad: comunidades por generación/estado, dependientes sin resolver, regeneraciones, tokens y duración por fase — sin texto de fuentes en métricas.
- **Criterio negativo:** un resumen nunca se presenta como vigente con `source_revisions` antiguas; borrar la vista nunca borra sus miembros; miembros no resueltos impiden publicar.
- **Independencia:** no es independiente de X-GRR-03 (la invalidación por revisión la sostiene); X-GRR-01 no bloquea el diagnóstico read-only de X-GRR-02. Alternativas documentadas: agrupar escenas existentes si satisfacen la pregunta, adapter GraphRAG aislado para experimentación, o reimplementar solo descriptor/invalidación.
- **No objetivos:** un segundo grafo; importar Leiden obligatoriamente; resumir otra vez las escenas existentes; activar generación dentro del motor embebido. Persistir informes agrega almacenamiento, tokens de indexación y regeneración aunque nunca se consulte global.

### X-GRR-02 — Proyección de contexto: candidatos recuperados vs evidencia entregada

- **Problema:** un diagnóstico debe separar «recuperado» de «efectivamente enviado al modelo»; las citas solo pueden referirse a lo entregado. La selección de candidatos ocurre antes del recorte y ambos estados deben conservarse.
- **Contrato hipotético (tipos propuestos):** `GraphContextProjection { candidates, delivered_ids, source_manifest, exclusion_reasons }`, consumiendo el presupuesto remanente del assembly existente. Entrada = candidatos autorizados con identidad/revisión y presupuesto remanente real; salida = proyección de lo que efectivamente sobrevivió al renderizado. `delivered_ids` ⊆ `candidates` y cada ordinal resuelve una evidencia de esa entrega.
- **Fuente de evidencia (clone):** `graphrag/query/structured_search/local_search/mixed_context.py:91-221` (`LocalSearchMixedContext.build_context`; mezcla tablas con fracciones del presupuesto restante) y flags `in_context` en `:224-375` [G8]; `graphrag/query/context_builder/entity_extraction.py:41-96` [G8].
- **Seam VantaDB:** `vanta-memory/src/core/hooks/auto_recall.rs:399-408` (`perform_auto_recall_governed`); `vanta-memory/src/context_engine/engine.rs:252-264` (`assemble_with_recall`, que ya compacta, inyecta MMD y consume remanente [V2]). El delta es una proyección/receipt ligado a esa serialización, no otro ensamblador.
- **Acciones:**
  1. Conservar identidad y revisión al convertir hits de grafo.
  2. Aplicar governance antes de cada expansión.
  3. Registrar exclusión/recorte desde el assembly actual.
  4. Producir el manifest después del render final.
  5. Solo si el host confirma entrega, anexar el receipt de sesión; no persistir todo candidato por defecto.
- **Dependencias:** presupuesto compartido del assembly existente; governance aplicada antes de expansión. Depende de X-GRR-01 solo para mostrar comunidades, no para hechos. X-GRR-04 y X-GRR-05 la requieren; converge con C7/C8 de `CONSOLIDATED.md`.
- **FMEA:** recorte posterior al manifest → cita a contenido no enviado (construir el manifest al final y detectar IDs fuera del render); presupuesto por tabla → desborde total (una sola contabilidad compartida); retry de transporte → doble ledger (clave idempotente por entrega/revisión); confundir candidato con evidencia enviada.
- **Criterios de aceptación propuestos:** diagnóstico separa recuperado/entregado; IDs de citas pertenecen a entregados; historial y grafo comparten el presupuesto actual; cero presupuesto, sin hits, varias tablas y expansión denegada producen diagnósticos distintos sin filtrar evidencia sensible. Contadores: seleccionados/renderizados/confirmados, exclusiones por motivo y tokens efectivos.
- **Criterio negativo:** un error de permisos no habilita una búsqueda sin filtros; ninguna cita puede apuntar a candidato no entregado; no se certifica entailment ni se registra la pregunta completa por defecto.
- **Independencia:** sí — el informe la declara aprobable sola como diagnóstico («B puede aprobarse sola como diagnóstico»); migración aditiva y opt-in, clientes sin receipt mantienen la respuesta actual y el rollback deja de emitir el campo opcional sin reinterpretar ordinales viejos.
- **No objetivos:** copiar proporciones upstream como configuración Vanta; duplicar compactación/`context_assemble`; adapter de UI sin cambios al motor es alternativa preferible si ya existe metadata suficiente; diagnóstico transitorio basta para depurar sin receipt persistente.

### X-GRR-03 — Invalidación de derivados por revisión documental, no por título

- **Problema:** el delta por título deja cambios de contenido invisibles (`get_delta_docs` compara títulos; `load_update_documents` devuelve solo `new_inputs`) y los borrados sin propagar — un fichero con igual título y contenido cambiado no se reconoce como nuevo.
- **Contrato hipotético (tipos propuestos):** `DerivedInvalidation::plan(scope, old_revision, new_revision, dependents) -> Result<InvalidationPlan, LineageError>`; devuelve operaciones idempotentes y revisiones a invalidar, no aplica borrado directamente. Un título es presentación; la identidad incluye scope y fuente estable. Errores `UnknownLineage`, `RevisionConflict`, `ReadFailed` no equivalen a ausencia.
- **Fuente de evidencia (clone):** `graphrag/index/update/incremental_index.py:29-78` (`get_delta_docs` por título, `deleted_inputs` no consumido — issue [#2590](https://github.com/microsoft/graphrag/issues/2590)) [G12]; `graphrag/index/update/entities.py:14-74` (resolución incremental solo por título) [G7]; `graphrag/index/update/communities.py:14-84` (remapea comunidad/padre, no miembros — issue [#2540](https://github.com/microsoft/graphrag/issues/2540)) [G17]; `graphrag/index/workflows/load_update_documents.py:48-62` [G12].
- **Seam VantaDB:** `vanta-memory/src/ingest/worker.rs:167-250`, `vanta-memory/src/ingest/merge.rs::commit` (consolida por página [V3]); `src/wiki/ingestors.rs:69-89` (claves deterministas [V5]); dedup/supersession/linaje existentes.
- **Acciones:**
  1. Enumerar claves y linaje publicados.
  2. Diferenciar alta/update/baja por revisión.
  3. Persistir plan/progreso en almacenamiento/checkpoints existentes.
  4. Invalidar referencias antes de exponer derivados nuevos.
  5. Regenerar por el scheduler existente.
  6. Extender la auditoría de índices existente, no crear otro verificador.
- **Dependencias:** linaje, hash/versión y auditoría de índices existentes; registros legacy con linaje insuficiente se marcan desconocidos y requieren rebuild acotado aprobado — nunca se asumen vacíos. X-GRR-01, X-GRR-05 y X-GRR-06 dependen de su identidad/revisión; converge con C5/C6/C14 de `CONSOLIDATED.md`.
- **FMEA:** baja repetida sobre revisión vieja → elimina sucesora (detectar expected revision y rechazar); lectura fallida → falso missing (detener el plan destructivo); crash tras invalidar → disponibilidad parcial (reanudar idempotentemente conservando baseline); agrupación solo nominal que mezcla tipos (identidad y revisión independientes de presentación).
- **Criterios de aceptación propuestos:** actualización de contenido con nombre constante afecta derivados; borrar una fuente no elimina pruebas supervivientes; reintentos idempotentes; cubre mismo título/contenido nuevo, rename, fuente compartida, fallo de lectura y retry de baja; sin referencias colgantes en el snapshot publicado. Observar stale/missing/unknown por generación, progreso y descartes por fence, sin payload.
- **Criterio negativo:** una contribución compartida solo desaparece cuando no quedan fuentes autorizadas vigentes; un error de lectura nunca se trata como ausencia ni dispara borrado; invalidar un resumen no elimina hechos supervivientes.
- **Independencia:** es la base de X-GRR-01/X-GRR-05/X-GRR-06 («C sostiene invalidación de A/E/F») pero ella misma solo depende de linaje/auditoría existentes; depende del descriptor de X-GRR-01 para comunidades, no para su propia operación de invalidación.
- **No objetivos:** reimplementar ingestors wiki, dedup ni una limpieza completa del corpus por cada edición; la ruta upstream de concatenación no se adopta como contrato de borrado.

### X-GRR-04 — Exploración iterativa con estado acotado por consulta

- **Problema:** dos consultas no deben compartir acciones ni una cancelación dejar parciales marcados como completos; el estado de ejecución es por consulta, no memoria autobiográfica ni respuesta garantizada.
- **Contrato hipotético (tipos propuestos):** `EvidenceExploration::step(query_scope, frontier, remaining_resources) -> Result<ExplorationStep, ExplorationError>` con fuentes y motivo de terminación explícito. Entrada = scope autorizado, frontier de acciones de **una consulta**, recursos remanentes del host y token de cancelación; salida = evidencias, acciones nuevas, consumo y `TerminationReason` (`Exhausted | Cancelled | Failed | Completed`). La identidad de operación separa tenant/sesión/consulta; una reanudación valida generación/fuentes.
- **Fuente de evidencia (clone):** `graphrag/query/structured_search/drift_search/search.py:188-305` (bucle de acciones DRIFT, grafo de `query_state`, serialización y reducción) [G10].
- **Seam VantaDB:** `vanta-memory/src/services/pipeline_worker.rs` + scheduler/checkpoints existentes [V4] y `vanta-memory/src/core/hooks/auto_recall.rs::perform_auto_recall_governed`; integración de orquestación, no del motor.
- **Acciones:**
  1. Extender la orquestación que ya llama recall y assembly — no crear otro scheduler.
  2. Registrar padre/estado de acción y llamadas antes de reintentar.
  3. Pasar cada lote al presupuesto compartido existente.
  4. Finalizar con recibo explícito de parcialidad; persistir solo para consultas reanudables (no reutilizar el checkpoint global de persona como almacén de acciones).
- **Dependencias:** X-GRR-02 requerida (entrega evidencias al assembly actual); cancelación del worker; ownership por tenant/sesión/consulta; llamadas LLM opcionales. Converge con C8 de `CONSOLIDATED.md`.
- **FMEA:** followups equivalentes → gasto repetido (detectar repetición de acciones y consumo, detener según límites acordados por el host); estado compartido → contaminación de preguntas (ownership exclusivo por consulta); timeout del cliente → la generación continúa (propagar cancelación y reflejar consumo residual, no declarar coste cero).
- **Criterios de aceptación propuestos:** consultas concurrentes aisladas; cancelación en cada fase mantiene trazas; fallo parcial visible y nunca entregado como completo; consulta nueva no hereda acciones ajenas; baseline sin exploración permanece disponible. Telemetría por fase/acción, cola, tokens y motivo final — sin prompts ni texto sensible.
- **Criterio negativo:** la evidencia recuperada no autoriza ejecutar instrucciones; un parcial cancelado nunca se marca `Completed`; no se declara consumo cero cuando el proveedor ya cobró.
- **Independencia:** parcial — depende de X-GRR-02 y de la cancelación del worker, pero no de resolver todos los problemas históricos; X-GRR-04 y X-GRR-05 son alternativas de orquestación entre sí, no pasos obligatorios (alternativa más barata documentada: expansión determinista de vecinos).
- **No objetivos:** portar DRIFT completo; garantizar cobertura por «razonamiento»; definir budgets/umbrales nuevos en esta investigación; no añade scheduler, ANN, rúbrica ni umbral.

### X-GRR-05 — Síntesis transversal por lotes con fuentes y uso por fase

- **Problema:** resumir cambios o decisiones dispersos entre sesiones sin convertir cada pregunta en exploración iterativa; preguntas transversales no deben mezclar generaciones ni tratar un lote fallido como relevancia cero.
- **Contrato hipotético (tipos propuestos):** `GlobalSynthesisInput { scope, query_id, source_generation, batches } -> SynthesisOutcome { evidence_refs, partial_batches, usage_by_phase, answer_ref }`; errores diferenciados `InvalidMapPayload`, `StaleGeneration`, `Cancelled`, `ProviderFailure`. Cada punto conserva fuentes; la evidencia recuperada no autoriza ejecutar instrucciones.
- **Fuente de evidencia (clone):** `graphrag/query/structured_search/global_search/search.py::search:142-220` (lotes de informes, map paralelo) y `_reduce_response:306-385` [G9].
- **Seam VantaDB:** `vanta-memory/src/services/pipeline_worker.rs` y `vanta-memory/src/context_engine/engine.rs::assemble_with_recall`; estrategia opt-in del host — motor híbrido y compactación ya existen.
- **Acciones:**
  1. Seleccionar snapshot autorizado.
  2. Lanzar maps con recursos del host.
  3. Validar shape/referencias de cada lote.
  4. Reducir solo material entregable.
  5. Registrar la entrega mediante X-GRR-02; cachear maps por revisión/versión de modelo, nunca convertirlos automáticamente en hechos durables. Publicar la síntesis solo tras validar la generación; un cambio concurrente marca el resultado parcial/stale.
- **Dependencias:** X-GRR-02 requerida; X-GRR-01 solo si usa informes; fence de generación antes de publicar; sin scheduler/budget nuevo. Converge con C8 de `CONSOLIDATED.md`.
- **FMEA:** un map emite JSON malformado → pérdida de evidencia (rechazo tipado); mezcla de generaciones → síntesis contradictoria (fence previo a publicación); cancelación del cliente → maps continúan cobrando (ownership por consulta y reporte de uso residual). No persistir preguntas en métricas.
- **Criterios de aceptación propuestos:** lote vacío, fallido y score cero son distinguibles; ningún punto cita una fuente denegada; repetición con otra revisión no reutiliza map viejo; cancelación no deja una respuesta marcada completa.
- **Criterio negativo:** un lote fallido nunca equivale a relevancia cero; los maps cacheados no se convierten en hechos durables por sí solos; el score del modelo no se interpreta como verdad.
- **Independencia:** parcial — requiere X-GRR-02; puede operar sobre evidencias existentes sin X-GRR-01; es alternativa de orquestación a X-GRR-04, no paso obligatorio. Alternativas documentadas: respuesta única desde baseline o agregación determinista de metadatos.
- **No objetivos:** nuevo scheduler, nuevo budget, juez ni scoring de evaluación; no copiar los prompts upstream.

### X-GRR-06 — Envelope versionado de caché derivada

- **Problema:** un caché corrupto no debe bloquear el recall ni un payload no objeto contar como hit; `JsonCache.get` asume objeto sin validar el tipo leído (issue [#2614](https://github.com/microsoft/graphrag/issues/2614)).
- **Contrato hipotético (tipos propuestos):** `DerivedCacheEnvelope { schema_version, scope, source_revisions, model_prompt_fingerprint, result }`; la lectura devuelve `Hit | Miss | InvalidEnvelope | Stale` o error de I/O. La deserialización nunca autoriza acceso.
- **Fuente de evidencia (clone):** `graphrag-cache/graphrag_cache/json_cache.py:33-55` (`get:33-46` lee `data.get("result")` sin validar tipo; `set:50-55` serializa objeto) [G18]; issue #2614.
- **Seam VantaDB:** `vanta-memory/src/core/memory_generation_log/` y el adaptador de generación que use caché en la ruta elegida (inventariar); no cambiar WAL ni prometer reparación de registros primarios.
- **Acciones:**
  1. Inventariar el caché concreto de la ruta elegida.
  2. Serializar el envelope con claves de derivación (scope, revisiones, fingerprint modelo/prompt).
  3. Validar el envelope antes de usarlo.
  4. Aislar/cuarentenar la entrada inválida.
  5. Regenerar solo con permiso del host; persistir el reemplazo con revisión esperada (el lector concurrente ve el envelope viejo o el nuevo válido, nunca un fragmento).
- **Dependencias:** comparte identidad/revisión con X-GRR-03; no requiere X-GRR-01 ni X-GRR-05; sin almacenamiento nuevo si el backend actual ya expresa el contrato. Converge con C14 de `CONSOLIDATED.md`.
- **FMEA:** JSON array aceptado → excepción repetida (schema tipado y diagnóstico); I/O fallido tratado como miss → oculta degradación (estado explícito); regeneraciones simultáneas → gasto duplicado (single-owner por clave y cancelación, sin prometer exactly-once de un proveedor externo).
- **Criterios de aceptación propuestos:** array, número, versión desconocida y revisión antigua no cuentan como hits; caché inaccesible conserva el error; el primario permanece intacto. Observabilidad: hits/misses/stale/corrupt, fase y uso — nunca prompts.
- **Criterio negativo:** un payload no objeto nunca cuenta como hit; migración legacy equivale a «desconocido», no a hit certificado; la deserialización no concede permisos.
- **Independencia:** sí, condicionada — el informe la clasifica «independiente, si hay caché derivada durable»; caché en memoria es alternativa que evita migración a cambio de recomputación, y una caché durable exige retención/erasure y control de secretos.
- **No objetivos:** cifrado global, nuevo motor de caché ni copiar la plantilla upstream; rollback omite el caché.
